//! 测试子进程方式的 PipeWire 音频捕获
//!
//! 运行方式:
//! cargo run --example test_pipewire_subprocess --features pipewire-capture

use std::thread;
use std::time::Duration;
use vinput_core::audio::{
    AudioRingBuffer, AudioRingBufferConfig, PipeWireStream, PipeWireStreamConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PipeWire 子进程音频捕获测试 ===\n");

    // 创建 Ring Buffer (5秒容量，确保不溢出)
    println!("1. 创建 Ring Buffer (5秒容量 @ 16kHz)...");
    let ring_config = AudioRingBufferConfig {
        capacity: 16000 * 5, // 5 秒
    };
    let ring = AudioRingBuffer::new(ring_config);
    let (producer, mut consumer) = ring.split();
    println!("   ✓ Ring Buffer 已创建\n");

    // 创建 PipeWire 流配置
    println!("2. 配置 PipeWire 流 (16kHz, 单声道, F32LE)...");
    let stream_config = PipeWireStreamConfig {
        sample_rate: 16000,
        channels: 1,
        ..Default::default()
    };
    println!("   ✓ 配置完成\n");

    // 创建 PipeWire 流
    println!("3. 启动 PipeWire 音频捕获流...");
    let stream = PipeWireStream::new(stream_config, producer)?;
    println!("   ✓ 流已启动\n");

    #[cfg(feature = "pipewire-capture")]
    println!("🎙️  使用真实 PipeWire 捕获模式 (pw-record 子进程)");
    #[cfg(not(feature = "pipewire-capture"))]
    println!("🔇 使用模拟模式 (静音)");

    println!("\n⏱️  录音 3 秒，请对着麦克风说话...\n");

    // 等待 3 秒
    thread::sleep(Duration::from_secs(3));

    println!("⏹️  停止录音\n");
    stream.stop();

    // 等待流完全停止
    thread::sleep(Duration::from_millis(100));

    // 统计信息
    println!("📊 捕获统计:");
    let available = consumer.available_samples();
    let overrun = consumer.overrun_count();
    let duration = available as f32 / 16000.0;

    println!("   - 可用样本数: {}", available);
    println!("   - 录音时长: {:.2} 秒", duration);
    println!("   - Buffer 溢出: {}", overrun);

    // 检查音频质量
    println!("\n🔍 音频质量检查:");

    // 跳过前 500 个样本（初始缓冲区可能是零）
    let _ = consumer.read_available(500);

    // 读取后续样本进行分析
    let samples = consumer.read_available(5000);

    if samples.is_empty() {
        println!("   ✗ 没有捕获到音频数据");
        return Ok(());
    }

    // 检测是否有非零信号
    let non_zero_count = samples.iter().filter(|&&s| s.abs() > 0.0001).count();
    let has_signal = non_zero_count > 0;

    println!("   - 样本总数: {}", samples.len());
    println!("   - 非零样本: {}/{}", non_zero_count, samples.len());
    println!(
        "   - 音频信号: {}",
        if has_signal {
            "✓ 检测到"
        } else {
            "✗ 静音"
        }
    );

    // 计算振幅
    if has_signal {
        let max_amplitude = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let avg_amplitude = samples.iter().map(|s| s.abs()).sum::<f32>() / samples.len() as f32;

        println!("   - 最大振幅: {:.6}", max_amplitude);
        println!("   - 平均振幅: {:.6}", avg_amplitude);

        // 检查是否是真实语音（非静音）
        if max_amplitude > 0.01 {
            println!("\n✅ 真实音频信号检测成功！");
        } else {
            println!("\n⚠️  振幅较小，可能是环境噪音或麦克风音量过低");
        }
    }

    println!("\n🎉 PipeWire 子进程音频捕获测试完成！");

    #[cfg(feature = "pipewire-capture")]
    {
        if has_signal && duration >= 2.5 {
            println!("\n✅ 真实 PipeWire 捕获工作正常，可以集成到 V-Input！");
        } else if !has_signal {
            println!("\n⚠️  未检测到音频信号，请检查:");
            println!("   1. 麦克风是否连接");
            println!("   2. 麦克风是否被静音");
            println!("   3. PipeWire 音频设置");
            println!("   4. 运行 'pw-record --list-targets' 查看可用设备");
        } else {
            println!("\n⚠️  录音时长较短 ({:.2}s < 2.5s)，Ring Buffer 可能不足", duration);
        }
    }

    #[cfg(not(feature = "pipewire-capture"))]
    println!("\n💡 提示: 使用 --features pipewire-capture 启用真实音频捕获");

    Ok(())
}
