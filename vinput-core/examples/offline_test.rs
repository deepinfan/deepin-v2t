//! 离线 ASR 识别测试
//!
//! 用法：
//!   cargo run --example offline_test
//!   cargo run --example offline_test -- path/to/audio.wav

use hound;
use std::env;
use std::path::Path;
use vinput_core::asr::{OnlineRecognizer, OnlineRecognizerConfig};
use vinput_core::VInputResult;

fn main() -> VInputResult<()> {
    // 初始化日志（可选）
    vinput_core::init_logging();

    println!("=== V-Input ASR 离线识别测试 ===\n");

    // 获取音频文件路径
    let audio_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "models/streaming/test_wavs/0.wav".to_string());

    println!("📁 音频文件: {}", audio_path);

    // 检查文件是否存在
    if !Path::new(&audio_path).exists() {
        eprintln!("❌ 错误: 音频文件不存在: {}", audio_path);
        std::process::exit(1);
    }

    // 配置识别器
    let config = OnlineRecognizerConfig {
        model_dir: "models/streaming".to_string(),
        sample_rate: 16000,
        feat_dim: 80,
        decoding_method: "greedy_search".to_string(),
        max_active_paths: 4,
        hotwords_file: None,
        hotwords_score: 1.5,
    };

    println!("🔧 模型配置:");
    println!("   - 模型目录: {}", config.model_dir);
    println!("   - 采样率: {} Hz", config.sample_rate);
    println!("   - 解码方法: {}\n", config.decoding_method);

    // 创建识别器
    println!("⏳ 加载模型...");
    let recognizer = OnlineRecognizer::new(&config)?;
    println!("✅ 模型加载成功\n");

    // 读取 WAV 文件
    println!("📖 读取音频数据...");
    let mut reader = hound::WavReader::open(&audio_path)
        .map_err(|e| vinput_core::VInputError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to open WAV file: {}", e),
        )))?;

    let spec = reader.spec();
    println!("   - 采样率: {} Hz", spec.sample_rate);
    println!("   - 声道数: {}", spec.channels);
    println!("   - 位深度: {} bit", spec.bits_per_sample);
    println!("   - 样本数: {}\n", reader.len());

    // 读取音频样本并转换为 f32
    let samples: Vec<f32> = if spec.sample_format == hound::SampleFormat::Int {
        if spec.bits_per_sample == 16 {
            // 16-bit PCM
            reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / 32768.0)
                .collect()
        } else {
            eprintln!("❌ 错误: 不支持的位深度: {}", spec.bits_per_sample);
            std::process::exit(1);
        }
    } else if spec.sample_format == hound::SampleFormat::Float {
        // 32-bit float PCM
        reader
            .samples::<f32>()
            .map(|s| s.unwrap())
            .collect()
    } else {
        eprintln!("❌ 错误: 不支持的音频格式");
        std::process::exit(1);
    };

    // 如果是立体声，转为单声道（取平均）
    let mono_samples: Vec<f32> = if spec.channels == 2 {
        println!("   ℹ️  转换立体声为单声道");
        samples
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        samples
    };

    println!("   - 单声道样本数: {}\n", mono_samples.len());

    // 创建识别流
    println!("🎙️  开始识别...");
    let mut stream = recognizer.create_stream()?;

    // 输入音频数据
    stream.accept_waveform(&mono_samples, spec.sample_rate as i32);

    // 标记输入结束
    stream.input_finished();

    // 解码循环
    let mut segment_count = 0;
    loop {
        if stream.is_ready(&recognizer) {
            stream.decode(&recognizer);

            let result = stream.get_result(&recognizer);
            if !result.is_empty() {
                segment_count += 1;
                println!("   [片段 {}] {}", segment_count, result);
                stream.reset(&recognizer);
            }
        } else {
            break;
        }
    }

    // 获取最终结果
    stream.decode(&recognizer);
    let final_result = stream.get_result(&recognizer);
    if !final_result.is_empty() {
        segment_count += 1;
        println!("   [片段 {}] {}", segment_count, final_result);
    }

    println!("\n✅ 识别完成！");
    println!("   - 识别片段数: {}", segment_count);

    Ok(())
}
