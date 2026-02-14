//! 流式识别管道测试示例
//!
//! 演示 VAD-ASR 流式识别管道的完整流程
//!
//! 使用方法：
//! ```bash
//! cargo run --example streaming_pipeline_test --features vad-onnx
//! ```

use vinput_core::asr::OnlineRecognizerConfig;
use vinput_core::streaming::{PipelineState, StreamingConfig, StreamingPipeline};
use vinput_core::vad::VadConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    #[cfg(feature = "debug-logs")]
    vinput_core::init_logging();

    println!("=== V-Input 流式识别管道测试 ===\n");

    // 1. 配置管道
    let streaming_config = StreamingConfig {
        vad_config: VadConfig::push_to_talk_default(),
        asr_config: OnlineRecognizerConfig {
            model_dir: "models/streaming".to_string(), // Sherpa-ONNX 模型目录
            sample_rate: 16000,
            feat_dim: 80,
            decoding_method: "greedy_search".to_string(),
            max_active_paths: 4,
            hotwords_file: None,
            hotwords_score: 1.5,
        },
        max_silence_duration_ms: 3000,
        enable_endpoint_detection: true,
    };

    println!("📋 配置:");
    println!("  - VAD 模式: {:?}", streaming_config.vad_config.mode);
    println!("  - VAD 启动阈值: {}", streaming_config.vad_config.hysteresis.start_threshold);
    println!("  - ASR 模型: {}", streaming_config.asr_config.model_dir);
    println!("  - 采样率: {} Hz", streaming_config.asr_config.sample_rate);
    println!();

    // 2. 创建管道
    println!("🔧 创建流式识别管道...");
    let mut pipeline = match StreamingPipeline::new(streaming_config) {
        Ok(p) => {
            println!("✅ 管道创建成功");
            p
        }
        Err(e) => {
            eprintln!("❌ 管道创建失败: {}", e);
            eprintln!("\n请确保模型文件已下载到 models/ 目录：");
            eprintln!("  - models/silero-vad/silero_vad.onnx (VAD 模型)");
            eprintln!("  - models/streaming/*.onnx (ASR 模型)");
            return Err(e.into());
        }
    };
    println!();

    // 3. 模拟音频输入测试
    println!("🎤 模拟音频输入测试\n");

    // 模拟静音 (低能量)
    println!("1️⃣  发送静音帧...");
    let silence: Vec<f32> = vec![0.0; 512];
    for _ in 0..10 {
        let result = pipeline.process(&silence)?;
        print_result(&result, false);
    }

    // 模拟语音 (高能量)
    println!("\n2️⃣  发送语音帧...");
    let speech: Vec<f32> = (0..512)
        .map(|i| (i as f32 * 0.01).sin() * 0.1)
        .collect();

    for i in 0..50 {
        let result = pipeline.process(&speech)?;
        print_result(&result, i % 5 == 0); // 每 5 帧打印一次

        if result.is_final {
            println!("\n✅ 识别完成！最终结果: \"{}\"", result.partial_result);
            break;
        }
    }

    // 重置管道
    println!("\n3️⃣  重置管道...");
    pipeline.reset()?;
    println!("✅ 管道已重置");

    // 打印统计信息
    let stats = pipeline.stats();
    println!("\n📊 统计信息:");
    println!("  - 总帧数: {}", stats.total_frames);
    println!("  - ASR 帧数: {}", stats.asr_frames);
    println!("  - 语音时长: {} ms", stats.speech_duration_ms);

    println!("\n✅ 测试完成！");
    println!("\n💡 提示:");
    println!("  - 要使用真实麦克风输入，请参考 examples/realtime_recognition.rs");
    println!("  - 要启用详细日志，请使用: VINPUT_LOG=debug cargo run --example streaming_pipeline_test --features debug-logs,vad-onnx");

    Ok(())
}

fn print_result(result: &vinput_core::streaming::StreamingResult, verbose: bool) {
    if !verbose {
        return;
    }

    println!(
        "  VAD: {:?} | Pipeline: {:?} | Prob: {:.3} | Partial: \"{}\"",
        result.vad_state,
        result.pipeline_state,
        result.speech_prob,
        result.partial_result
    );
}
