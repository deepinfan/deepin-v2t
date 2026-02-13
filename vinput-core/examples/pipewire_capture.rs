//! PipeWire 音频捕获测试
//!
//! 用法：
//!   cargo run --example pipewire_capture
//!   cargo run --example pipewire_capture -- output.wav 10
//!
//! 参数：
//!   [output.wav] - 输出 WAV 文件路径（默认: capture.wav）
//!   [duration]   - 录音时长（秒，默认: 5）

use hound;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use vinput_core::audio::{AudioRingBuffer, AudioRingBufferConfig, PipeWireStream, PipeWireStreamConfig};
use vinput_core::VInputResult;

fn main() -> VInputResult<()> {
    // 初始化日志
    vinput_core::init_logging();

    println!("=== V-Input PipeWire 音频捕获测试 ===\n");

    // 解析参数
    let output_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "capture.wav".to_string());

    let duration_secs: u64 = env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    println!("📁 输出文件: {}", output_path);
    println!("⏱️  录音时长: {} 秒\n", duration_secs);

    // 配置参数
    let sample_rate = 16000u32;
    let channels = 1u32;

    // 创建 Ring Buffer（1秒容量）
    let ring_config = AudioRingBufferConfig {
        capacity: sample_rate as usize,
    };
    let ring = AudioRingBuffer::new(ring_config);
    let (producer, mut consumer) = ring.split();

    println!("🔧 音频配置:");
    println!("   - 采样率: {} Hz", sample_rate);
    println!("   - 声道数: {}", channels);
    println!("   - Ring Buffer 容量: {} 样本\n", sample_rate);

    // 创建 PipeWire 流
    let stream_config = PipeWireStreamConfig {
        sample_rate,
        channels,
        stream_name: "V-Input Capture Test".to_string(),
        app_name: "vinput-capture-test".to_string(),
    };

    println!("🎙️  启动 PipeWire 音频流...");
    let stream = PipeWireStream::new(stream_config, producer)?;

    // 在单独线程中运行 PipeWire 主循环
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let stream_handle = thread::spawn(move || {
        stream.run().ok();
    });

    // 等待流启动
    thread::sleep(Duration::from_millis(500));
    println!("✅ 音频流已启动\n");

    // 创建 WAV 写入器
    let spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut wav_writer = hound::WavWriter::create(&output_path, spec)
        .map_err(|e| vinput_core::VInputError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create WAV file: {}", e),
        )))?;

    println!("🔴 开始录音...\n");

    let start_time = Instant::now();
    let mut total_samples = 0usize;
    let mut last_report = Instant::now();
    let mut report_count = 0;

    // 录音循环
    while start_time.elapsed().as_secs() < duration_secs {
        // 从 Ring Buffer 读取音频数据
        let samples = consumer.read_available(8192);

        if !samples.is_empty() {
            // 写入 WAV 文件（转换为 i16）
            for sample in &samples {
                let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                wav_writer.write_sample(sample_i16).ok();
            }

            total_samples += samples.len();
        }

        // 每秒报告一次状态
        if last_report.elapsed().as_secs() >= 1 {
            report_count += 1;
            let elapsed = start_time.elapsed().as_secs();
            let buffer_usage = consumer.available_samples();
            let overrun = consumer.overrun_count();

            println!(
                "[{:02}秒] 已录制: {:.2} 秒 | Buffer: {} 样本 | Overrun: {}",
                elapsed,
                total_samples as f64 / sample_rate as f64,
                buffer_usage,
                overrun
            );

            last_report = Instant::now();
        }

        // 短暂休眠避免忙等待
        thread::sleep(Duration::from_millis(10));
    }

    println!("\n⏹️  停止录音...");
    running.store(false, Ordering::Release);

    // 读取剩余数据
    let remaining = consumer.read_available(consumer.available_samples());
    for sample in &remaining {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        wav_writer.write_sample(sample_i16).ok();
    }
    total_samples += remaining.len();

    // 完成写入
    wav_writer.finalize().ok();

    // 等待 PipeWire 线程结束（最多1秒）
    thread::sleep(Duration::from_millis(100));

    println!("\n✅ 录音完成！");
    println!("\n📊 统计信息:");
    println!("   - 总样本数: {}", total_samples);
    println!("   - 录音时长: {:.2} 秒", total_samples as f64 / sample_rate as f64);
    println!("   - 文件大小: {:.2} KB", total_samples * 2 / 1024);
    println!("   - Buffer Overrun: {}", consumer.overrun_count());

    if consumer.overrun_count() == 0 {
        println!("\n🎉 零丢帧！音频捕获完美！");
    } else {
        println!("\n⚠️  检测到 Buffer Overrun，可能需要增大 Ring Buffer 容量");
    }

    // 停止 PipeWire 流（通过 drop）
    drop(stream_handle);

    Ok(())
}
