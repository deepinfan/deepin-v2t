//! E2E 流水线测试
//!
//! 使用真实录音验证完整的 ASR + 端点检测 + 标点 + ITN 流水线。
//!
//! 测试数据位于 tests/testdata/：
//!   - NNN_描述.wav      16kHz 单声道 WAV
//!   - NNN_描述.expected  期望的最终识别结果（含标点和 ITN）
//!
//! 运行方式（带详细日志）：
//!   RUST_LOG=info cargo test --test pipeline_e2e_tests -- --nocapture

use std::path::{Path, PathBuf};
use vinput_core::{
    asr::OnlineRecognizerConfig,
    endpointing::EndpointDetectorConfig,
    itn::{ITNEngine, ITNMode},
    punctuation::StyleProfile,
    streaming::{PipelineState, StreamingConfig, StreamingPipeline},
    vad::{VadConfig, VadState},
};

/// ASR 模型目录
const MODELS_DIR: &str = "/usr/share/droplet-voice-input/models";

/// 测试数据目录（相对于 CARGO_MANIFEST_DIR）
fn testdata_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("testdata")
}

/// 检查 ASR 模型是否存在（若不存在则跳过测试）
fn models_available() -> bool {
    let dir = Path::new(MODELS_DIR);
    dir.join("encoder.int8.onnx").exists()
        && dir.join("decoder.int8.onnx").exists()
        && dir.join("tokens.txt").exists()
}

/// 初始化日志（忽略重复初始化错误）
fn init_log() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_test_writer()
        .try_init();
}

/// 读取 WAV 文件并返回 f32 样本（16kHz 单声道）
///
/// 容忍被 Ctrl+C 截断的 WAV 文件：遇到读取错误时停止而不是 panic
fn load_wav(path: &Path) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path)
        .unwrap_or_else(|e| panic!("无法读取 WAV {:?}: {}", path, e));

    let spec = reader.spec();
    assert_eq!(
        spec.sample_rate, 16000,
        "WAV 必须是 16kHz，实际: {}",
        spec.sample_rate
    );
    assert_eq!(
        spec.channels, 1,
        "WAV 必须是单声道，实际: {}",
        spec.channels
    );

    // take_while(Ok) 遇到截断错误时停止，不 panic
    match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .take_while(|s| s.is_ok())
            .map(|s| s.unwrap() as f32 / 32768.0)
            .collect(),
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .take_while(|s| s.is_ok())
            .map(|s| s.unwrap())
            .collect(),
    }
}

/// 创建测试用 StreamingPipeline
///
/// 使用 PushToTalk 默认 VAD 配置（不需要 Silero 模型）
fn create_pipeline() -> StreamingPipeline {
    let asr_config = OnlineRecognizerConfig {
        model_dir: MODELS_DIR.to_string(),
        sample_rate: 16000,
        ..Default::default()
    };

    // 使用配置文件中的标点风格（Custom）
    let punctuation_profile = StyleProfile {
        streaming_pause_ratio: 2.15,
        streaming_min_tokens: 5,
        allow_exclamation: false,
        question_strict_mode: true,
        ..Default::default()
    };

    let endpoint_config = EndpointDetectorConfig {
        min_speech_duration_ms: 300,
        trailing_silence_ms: 600,
        force_timeout_ms: 60_000,
        vad_silence_confirm_frames: 5,
        ..Default::default()
    };

    let config = StreamingConfig {
        vad_config: VadConfig::push_to_talk_default(),
        asr_config,
        punctuation_profile,
        endpoint_config,
    };

    StreamingPipeline::new(config).expect("创建 StreamingPipeline 失败")
}

/// 对单个 WAV 文件运行完整流水线，返回最终识别文本
///
/// 模拟 PushToTalk 模式：
///   1. force_vad_state(Speech) → 立即开始录音
///   2. 按 512 样本（32ms）分块喂入
///   3. 如果 ASR 端点提前触发，停止喂音频
///   4. get_final_result_with_punctuation() 获取含标点+ITN 的最终文本
fn run_pipeline(wav_path: &Path) -> String {
    let samples = load_wav(wav_path);
    let duration_s = samples.len() as f64 / 16000.0;
    println!("  音频: {} 样本 ({:.2}s)", samples.len(), duration_s);

    assert!(
        samples.len() >= 512,
        "WAV 文件太短（{} 样本，至少需要 512）",
        samples.len()
    );
    let mut pipeline = create_pipeline();

    // PushToTalk：强制进入语音状态
    pipeline.force_vad_state(VadState::Speech);

    let mut completed = false;
    for chunk in samples.chunks(512) {
        // 最后一块不足 512 则补零（ASR 需要固定大小帧）
        let mut frame = chunk.to_vec();
        frame.resize(512, 0.0);

        let result = pipeline.process(&frame).expect("处理音频帧失败");

        if result.pipeline_state == PipelineState::Completed {
            completed = true;
            break;
        }
    }

    if completed {
        tracing::info!("📍 ASR 端点提前触发，停止喂音频");
    } else {
        tracing::info!("📍 音频喂入完毕，获取最终结果");
    }

    let punctuated = pipeline.get_final_result_with_punctuation();

    // 应用 ITN（与 FFI 层保持一致）
    let itn = ITNEngine::new(ITNMode::Auto);
    let itn_result = itn.process(&punctuated);
    if !itn_result.changes.is_empty() {
        tracing::info!("✏️  ITN: {} 处变更 → '{}'", itn_result.changes.len(), itn_result.text);
    }
    itn_result.text
}

/// 收集 testdata 目录下所有 (wav, expected) 测试对
fn collect_test_cases() -> Vec<(PathBuf, String)> {
    let dir = testdata_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut cases: Vec<(PathBuf, String)> = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("读取 testdata 目录失败")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "wav").unwrap_or(false))
        .collect();

    // 按文件名排序（NNN 编号顺序）
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let wav_path = entry.path();
        let expected_path = wav_path.with_extension("expected");

        if !expected_path.exists() {
            eprintln!(
                "⚠️  跳过 {:?}：未找到对应的 .expected 文件",
                wav_path.file_name().unwrap()
            );
            continue;
        }

        let expected = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("读取 {:?} 失败: {}", expected_path, e))
            .trim()
            .to_string();

        if expected.is_empty() {
            eprintln!(
                "⚠️  跳过 {:?}：.expected 文件为空",
                wav_path.file_name().unwrap()
            );
            continue;
        }

        cases.push((wav_path, expected));
    }

    cases
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试入口
// ─────────────────────────────────────────────────────────────────────────────

/// 主 E2E 测试：遍历所有测试对
#[test]
fn test_all_recordings() {
    init_log();

    if !models_available() {
        eprintln!("⏭  跳过 E2E 测试：模型不在 {}", MODELS_DIR);
        return;
    }

    let cases = collect_test_cases();

    if cases.is_empty() {
        eprintln!("⏭  跳过 E2E 测试：testdata/ 目录中没有测试数据");
        eprintln!("   使用 ./tools/record_test.sh 录制测试音频");
        return;
    }

    println!("\n=== E2E 流水线测试 ({} 个用例) ===\n", cases.len());

    let mut passed = 0usize;
    let mut failed = 0usize;

    for (wav_path, expected) in &cases {
        let name = wav_path.file_stem().unwrap().to_string_lossy();
        println!("── 测试: {} ──", name);
        println!("  期望: {:?}", expected);

        let actual = run_pipeline(wav_path);
        println!("  实际: {:?}", actual);

        if &actual == expected {
            println!("  ✅ PASS\n");
            passed += 1;
        } else {
            println!("  ❌ FAIL");
            print_diff(expected, &actual);
            println!();
            failed += 1;
        }
    }

    println!("=== 结果: {} 通过, {} 失败 ===\n", passed, failed);

    if failed > 0 {
        panic!("{} 个测试用例失败", failed);
    }
}

/// 打印字符级别的差异，帮助定位标点/ITN 问题
fn print_diff(expected: &str, actual: &str) {
    let exp_chars: Vec<char> = expected.chars().collect();
    let act_chars: Vec<char> = actual.chars().collect();

    let max_len = exp_chars.len().max(act_chars.len());
    let mut diff_positions = Vec::new();

    for i in 0..max_len {
        let e = exp_chars.get(i);
        let a = act_chars.get(i);
        if e != a {
            diff_positions.push(i);
        }
    }

    if diff_positions.is_empty() {
        return;
    }

    println!("  差异位置:");
    for pos in &diff_positions {
        let e = exp_chars.get(*pos).copied();
        let a = act_chars.get(*pos).copied();
        println!(
            "    位置 {}: 期望 {:?}，实际 {:?}",
            pos,
            e.map(|c| c.to_string()).unwrap_or_else(|| "<结束>".to_string()),
            a.map(|c| c.to_string()).unwrap_or_else(|| "<结束>".to_string()),
        );
    }

    // 长度差异
    if exp_chars.len() != act_chars.len() {
        println!(
            "    长度: 期望 {} 字符，实际 {} 字符",
            exp_chars.len(),
            act_chars.len()
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 单文件测试（按文件名过滤，便于 --test 指定）
// ─────────────────────────────────────────────────────────────────────────────

/// 对 testdata/ 下每个 WAV 文件生成独立的测试函数
/// 测试函数名格式：test_recording_NNN（可用 cargo test test_recording_001）

macro_rules! recording_test {
    ($fn_name:ident, $file_stem:expr) => {
        #[test]
        fn $fn_name() {
            init_log();

            if !models_available() {
                eprintln!("⏭  跳过：模型不在 {}", MODELS_DIR);
                return;
            }

            let dir = testdata_dir();
            // 支持带描述后缀的文件名（如 001_jintian_tianqi）
            let mut wav_path = None;
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().map(|x| x == "wav").unwrap_or(false) {
                        let stem = p.file_stem().unwrap_or_default().to_string_lossy();
                        if stem.starts_with($file_stem) {
                            wav_path = Some(p);
                            break;
                        }
                    }
                }
            }

            let wav_path = match wav_path {
                Some(p) => p,
                None => {
                    eprintln!("⏭  跳过：testdata/{}_*.wav 不存在", $file_stem);
                    return;
                }
            };

            let expected_path = wav_path.with_extension("expected");
            if !expected_path.exists() {
                eprintln!("⏭  跳过：.expected 文件不存在");
                return;
            }

            let expected = std::fs::read_to_string(&expected_path)
                .expect("读取 .expected 失败")
                .trim()
                .to_string();

            println!("\n[{}]", $file_stem);
            println!("  WAV:    {:?}", wav_path.file_name().unwrap());
            println!("  期望: {:?}", expected);

            let actual = run_pipeline(&wav_path);
            println!("  实际: {:?}", actual);

            if actual == expected {
                println!("  ✅ PASS");
            } else {
                print_diff(&expected, &actual);
                panic!(
                    "识别结果不匹配\n  期望: {:?}\n  实际: {:?}",
                    expected, actual
                );
            }
        }
    };
}

// 预注册测试用例 001-020（不存在时会自动跳过）
recording_test!(test_recording_001, "001");
recording_test!(test_recording_002, "002");
recording_test!(test_recording_003, "003");
recording_test!(test_recording_004, "004");
recording_test!(test_recording_005, "005");
recording_test!(test_recording_006, "006");
recording_test!(test_recording_007, "007");
recording_test!(test_recording_008, "008");
recording_test!(test_recording_009, "009");
recording_test!(test_recording_010, "010");
recording_test!(test_recording_011, "011");
recording_test!(test_recording_012, "012");
recording_test!(test_recording_013, "013");
recording_test!(test_recording_014, "014");
recording_test!(test_recording_015, "015");
recording_test!(test_recording_016, "016");
recording_test!(test_recording_017, "017");
recording_test!(test_recording_018, "018");
recording_test!(test_recording_019, "019");
recording_test!(test_recording_020, "020");
