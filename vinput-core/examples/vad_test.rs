//! Silero VAD 测试示例
//!
//! 用法：
//!   cargo run --example vad_test
//!   cargo run --example vad_test -- path/to/audio.wav

use hound;
use std::env;
use std::path::Path;
use std::time::Instant;
use vinput_core::vad::{SileroVAD, SileroVADConfig, VADState};
use vinput_core::VInputResult;

fn main() -> VInputResult<()> {
    // 初始化日志
    vinput_core::init_logging();

    println!("=== V-Input Silero VAD 测试 ===\n");

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

    // 配置 VAD
    let config = SileroVADConfig {
        model_path: "models/vad/silero_vad_v5.onnx".to_string(),
        sample_rate: 16000,
        threshold: 0.5,
        min_speech_duration_ms: 250,
        min_silence_duration_ms: 100,
    };

    println!("🔧 VAD 配置:");
    println!("   - 模型: {}", config.model_path);
    println!("   - 采样率: {} Hz", config.sample_rate);
    println!("   - 阈值: {}", config.threshold);
    println!("   - 最小语音: {} ms", config.min_speech_duration_ms);
    println!("   - 最小静音: {} ms\n", config.min_silence_duration_ms);

    // 创建 VAD
    println!("⏳ 加载 VAD 模型...");
    let mut vad = SileroVAD::new(config)?;
    println!("✅ VAD 加载成功\n");

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
    println!("   - 位深度: {} bit\n", spec.bits_per_sample);

    if spec.sample_rate != 16000 {
        println!("⚠️  警告: 音频采样率不是 16kHz，可能影响 VAD 性能\n");
    }

    // 读取所有样本
    let samples: Vec<f32> = if spec.sample_format == hound::SampleFormat::Int {
        if spec.bits_per_sample == 16 {
            reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / 32768.0)
                .collect()
        } else {
            eprintln!("❌ 错误: 不支持的位深度: {}", spec.bits_per_sample);
            std::process::exit(1);
        }
    } else {
        reader
            .samples::<f32>()
            .map(|s| s.unwrap())
            .collect()
    };

    // 转为单声道
    let mono_samples: Vec<f32> = if spec.channels == 2 {
        samples
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        samples
    };

    println!("   - 样本数: {}", mono_samples.len());
    println!("   - 时长: {:.2} 秒\n", mono_samples.len() as f64 / spec.sample_rate as f64);

    // VAD 处理
    println!("🎙️  开始 VAD 检测...\n");

    let chunk_size = 512; // 32ms @ 16kHz
    let total_chunks = mono_samples.len() / chunk_size;
    let mut speech_chunks = 0;
    let mut silence_chunks = 0;
    let mut transitions = 0;
    let mut total_inference_time = std::time::Duration::ZERO;

    println!("帧 | 时间(ms) | 概率 | 状态 | 转换");
    println!("---|----------|------|------|------");

    for (i, chunk) in mono_samples.chunks(chunk_size).enumerate() {
        if chunk.len() != chunk_size {
            continue; // 跳过不完整的最后一帧
        }

        // 测量推理时间
        let start = Instant::now();
        let prob = vad.process_chunk(chunk)?;
        let (state, state_changed) = vad.detect(chunk)?;
        let inference_time = start.elapsed();

        total_inference_time += inference_time;

        // 统计
        match state {
            VADState::Speech => speech_chunks += 1,
            VADState::Silence => silence_chunks += 1,
        }

        if state_changed {
            transitions += 1;
        }

        // 每10帧输出一次
        if i % 10 == 0 || state_changed {
            let time_ms = (i * chunk_size) as f64 / spec.sample_rate as f64 * 1000.0;
            let state_str = match state {
                VADState::Speech => "🔊 语音",
                VADState::Silence => "🔇 静音",
            };
            let transition_str = if state_changed { "✨" } else { "" };

            println!(
                "{:3} | {:8.0} | {:.3} | {} | {}",
                i, time_ms, prob, state_str, transition_str
            );
        }
    }

    println!("\n✅ VAD 检测完成！\n");

    // 统计信息
    let avg_inference_us = total_inference_time.as_micros() as f64 / total_chunks as f64;
    let speech_ratio = speech_chunks as f64 / total_chunks as f64;

    println!("📊 统计信息:");
    println!("   - 总帧数: {}", total_chunks);
    println!("   - 语音帧: {} ({:.1}%)", speech_chunks, speech_ratio * 100.0);
    println!("   - 静音帧: {} ({:.1}%)", silence_chunks, (1.0 - speech_ratio) * 100.0);
    println!("   - 状态转换: {} 次", transitions);
    println!("   - 平均推理时间: {:.2} μs/帧", avg_inference_us);
    println!("   - 平均推理时间: {:.2} ms/帧", avg_inference_us / 1000.0);

    if avg_inference_us < 1000.0 {
        println!("\n🎉 推理延迟 < 1ms！性能优秀！");
    } else {
        println!("\n⚠️  推理延迟 > 1ms，Phase 1 需要优化");
    }

    println!("\n💡 Phase 0 MVP 说明:");
    println!("   当前使用基于能量的简单启发式");
    println!("   Phase 1 将使用完整的 Silero VAD ONNX 推理");
    println!("   状态机逻辑已完成，可直接用于生产");

    Ok(())
}
