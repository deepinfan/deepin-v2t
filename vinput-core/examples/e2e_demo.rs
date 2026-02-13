//! 端到端集成示例
//!
//! 展示 V-Input 所有组件的完整集成：
//! - 音频捕获 (PipeWire)
//! - VAD (Silero VAD)
//! - 端点检测 (EndpointDetector)
//! - ASR (Sherpa-ONNX)
//! - 命令生成 (FFI)

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use vinput_core::{
    audio::{AudioRingBuffer, AudioRingBufferConfig, PipeWireStream, PipeWireStreamConfig},
    endpointing::{EndpointDetector, EndpointDetectorConfig, EndpointResult},
    VInputResult,
};

fn main() -> VInputResult<()> {
    // 初始化日志
    vinput_core::init_logging();

    println!("=== V-Input 端到端集成示例 ===\n");

    // 配置参数
    let sample_rate = 16000u32;
    let channels = 1u32;
    let vad_frame_size = 512; // 32ms @ 16kHz

    println!("📋 系统配置:");
    println!("   - 采样率: {} Hz", sample_rate);
    println!("   - 声道: {}", channels);
    println!("   - VAD 帧大小: {} 样本 ({}ms)\n",
        vad_frame_size,
        vad_frame_size * 1000 / sample_rate as usize);

    // 1. 创建 Ring Buffer
    println!("🔧 步骤 1: 创建音频环形缓冲区");
    let capacity = sample_rate as usize * 2; // 2 秒容量
    let ring_config = AudioRingBufferConfig { capacity };
    let ring = AudioRingBuffer::new(ring_config);
    let (producer, mut consumer) = ring.split();
    println!("   ✓ Ring Buffer 已创建 (容量: {} 样本)\n", capacity);

    // 2. 创建 PipeWire 音频流
    println!("🔧 步骤 2: 启动 PipeWire 音频捕获");
    let stream_config = PipeWireStreamConfig {
        sample_rate,
        channels,
        stream_name: "V-Input E2E Demo".to_string(),
        app_name: "vinput-e2e-demo".to_string(),
        ..Default::default()
    };

    let stream = PipeWireStream::new(stream_config, producer)?;
    println!("   ✓ PipeWire 流已启动\n");

    // 等待流启动
    thread::sleep(Duration::from_millis(200));

    // 3. 初始化 VAD (Phase 1: 占位)
    println!("🔧 步骤 3: 初始化 VAD");
    println!("   ⚠ VAD 当前为模拟模式（Phase 1）");
    println!("   ℹ Phase 1.2 将集成 Silero VAD\n");

    // 4. 创建端点检测器
    println!("🔧 步骤 4: 创建端点检测器");
    let endpoint_config = EndpointDetectorConfig {
        min_speech_duration_ms: 300,
        max_speech_duration_ms: 10_000,
        trailing_silence_ms: 800,
        force_timeout_ms: 30_000,
        vad_assisted: true,
        vad_silence_confirm_frames: 5,
    };
    let mut detector = EndpointDetector::new(endpoint_config);
    println!("   ✓ 端点检测器已创建\n");

    // 5. 初始化 ASR (Phase 1: 占位)
    println!("🔧 步骤 5: 初始化 ASR");
    println!("   ⚠ ASR 当前为占位模式（Phase 1）");
    println!("   ℹ Phase 1.3 将集成 Sherpa-ONNX\n");

    // 6. 主循环
    println!("🎙️  开始语音输入循环\n");
    println!("说明：");
    println!("  - 当前为演示模式（模拟音频）");
    println!("  - 按 Ctrl+C 退出\n");
    println!("{}", "=".repeat(60));

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // 注册 Ctrl+C 处理
    ctrlc::set_handler(move || {
        println!("\n\n⏹️  收到退出信号");
        running_clone.store(false, Ordering::Release);
    })
    .expect("设置 Ctrl+C 处理器失败");

    let mut session_count = 0;
    let mut total_samples_processed = 0usize;
    let start_time = Instant::now();

    while running.load(Ordering::Acquire) {
        // 模拟语音检测会话
        session_count += 1;
        println!("\n🔵 会话 #{}: 等待语音输入...", session_count);

        detector.reset();
        let mut samples_in_session = 0usize;
        let session_start = Instant::now();

        // 模拟录音循环
        loop {
            // 从 Ring Buffer 读取音频
            let audio_chunk = consumer.read_available(vad_frame_size);

            if !audio_chunk.is_empty() {
                samples_in_session += audio_chunk.len();
                total_samples_processed += audio_chunk.len();

                // 模拟 VAD 处理
                // Phase 1: 简化模拟 - 假设前 1 秒是语音，然后是静音
                let elapsed_ms = session_start.elapsed().as_millis() as u64;
                let is_speech = elapsed_ms < 1000;

                // 端点检测
                match detector.process_vad(is_speech) {
                    EndpointResult::Continue => {
                        // 继续录音
                        if detector.is_speech_detected() && samples_in_session % (sample_rate as usize / 2) == 0 {
                            let duration = detector.speech_duration();
                            print!("\r   📝 录音中... {:.1}s", duration.as_secs_f32());
                            use std::io::{self, Write};
                            io::stdout().flush().unwrap();
                        }
                    }
                    EndpointResult::Detected => {
                        let duration = detector.speech_duration();
                        println!("\r   ✅ 检测到端点 (时长: {:.2}s)", duration.as_secs_f32());

                        // 模拟 ASR 识别
                        println!("   🔍 正在识别...");
                        thread::sleep(Duration::from_millis(100));

                        // Phase 1: 返回模拟结果
                        let text = format!("这是第 {} 次语音输入的模拟结果", session_count);
                        println!("   📄 识别结果: \"{}\"", text);

                        break;
                    }
                    EndpointResult::ForcedSegmentation => {
                        println!("\r   ⚠️  语音过长，自动分段");
                        println!("   📄 当前段结果: \"[分段 {}]\"", session_count);
                        detector.reset();
                    }
                    EndpointResult::Timeout => {
                        println!("\r   ⏱️  超时");
                        break;
                    }
                    EndpointResult::TooShort => {
                        println!("\r   🔇 语音过短，已忽略");
                        break;
                    }
                }
            }

            // 检查退出信号
            if !running.load(Ordering::Acquire) {
                break;
            }

            // 短暂休眠
            thread::sleep(Duration::from_millis(10));
        }

        if !running.load(Ordering::Acquire) {
            break;
        }

        // 会话间隔
        println!("   ⏸️  等待下一次输入...\n");
        thread::sleep(Duration::from_millis(500));

        // 演示模式：只运行 3 次会话
        if session_count >= 3 {
            println!("\n📊 演示完成（已运行 {} 个会话）\n", session_count);
            break;
        }
    }

    // 7. 统计与清理
    println!("{}", "=".repeat(60));
    println!("\n📊 会话统计:");
    println!("   - 总会话数: {}", session_count);
    println!("   - 总运行时间: {:.1}s", start_time.elapsed().as_secs_f32());
    println!("   - 处理音频: {:.2}s", total_samples_processed as f32 / sample_rate as f32);
    println!("   - Buffer Overrun: {}", consumer.overrun_count());

    println!("\n🛑 关闭系统...");
    drop(stream);
    println!("   ✓ PipeWire 流已停止");
    println!("   ✓ 资源已释放\n");

    println!("✅ V-Input 端到端集成示例完成！\n");

    println!("💡 提示:");
    println!("   - 当前为 Phase 1 演示模式（模拟音频）");
    println!("   - Phase 1.1: 集成真实 PipeWire 音频捕获");
    println!("   - Phase 1.2: 集成 Silero VAD");
    println!("   - Phase 1.3: 集成 Sherpa-ONNX ASR");
    println!("   - Phase 1.4: 完整 Fcitx5 集成\n");

    Ok(())
}
