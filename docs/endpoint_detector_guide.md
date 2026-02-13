# 端点检测器使用指南

## 概述

V-Input 端点检测器 (EndpointDetector) 提供智能的语音边界识别功能，结合 VAD (Voice Activity Detection) 和 ASR 端点检测，准确判断用户何时开始和结束说话。

## 核心特性

### 1. 双重端点检测
- **VAD 辅助**：基于语音活动检测的实时端点判断
- **ASR 端点**：利用 Sherpa-ONNX 内置的端点检测

### 2. 智能过滤
- **最小语音长度**：过滤点击音等短暂噪声
- **最大语音长度**：自动分段长语音
- **静音确认**：多帧静音确认避免误判

### 3. 超时保护
- **尾部静音超时**：语音结束后等待足够的静音
- **强制超时**：防止无限等待

## 配置参数

```rust
pub struct EndpointDetectorConfig {
    /// 最小语音长度（毫秒）
    /// 默认: 300ms
    /// 低于此长度的音频段会被忽略
    pub min_speech_duration_ms: u64,

    /// 最大语音长度（毫秒）
    /// 默认: 30000ms (30秒)
    /// 超过此长度会自动分段
    pub max_speech_duration_ms: u64,

    /// 语音结束后的静音等待时间（毫秒）
    /// 默认: 800ms
    /// 用于确认用户说话已结束
    pub trailing_silence_ms: u64,

    /// 强制超时（毫秒）
    /// 默认: 60000ms (60秒)
    /// 即使没有检测到端点，超时后也会强制结束
    pub force_timeout_ms: u64,

    /// 是否启用 VAD 辅助端点检测
    /// 默认: true
    pub vad_assisted: bool,

    /// VAD 检测到静音后的确认帧数
    /// 默认: 5 帧 (约 160ms @ 32ms/frame)
    /// 连续 N 帧静音才确认语音结束
    pub vad_silence_confirm_frames: usize,
}
```

## 端点检测结果

```rust
pub enum EndpointResult {
    /// 继续录音
    Continue,

    /// 检测到端点，可以结束
    Detected,

    /// 达到最大长度，强制分段
    ForcedSegmentation,

    /// 超时，强制结束
    Timeout,

    /// 语音过短，忽略
    TooShort,
}
```

## 使用示例

### 基础用法

```rust
use vinput_core::endpointing::{EndpointDetector, EndpointDetectorConfig, EndpointResult};

// 创建检测器（使用默认配置）
let mut detector = EndpointDetector::default_config();

// 或使用自定义配置
let config = EndpointDetectorConfig {
    min_speech_duration_ms: 500,    // 最小 500ms
    trailing_silence_ms: 1000,      // 1秒尾部静音
    ..Default::default()
};
let mut detector = EndpointDetector::new(config);
```

### VAD 辅助端点检测

```rust
// 在音频处理循环中
loop {
    // 从 VAD 获取语音检测结果
    let is_speech = vad.process_frame(&audio_frame);

    // 处理 VAD 结果
    match detector.process_vad(is_speech) {
        EndpointResult::Continue => {
            // 继续录音
        }
        EndpointResult::Detected => {
            // 检测到端点，开始识别
            println!("语音结束，开始识别...");
            break;
        }
        EndpointResult::ForcedSegmentation => {
            // 语音过长，分段处理
            println!("语音过长，分段识别...");
            process_segment();
            detector.reset();
        }
        EndpointResult::Timeout => {
            // 超时，强制结束
            println!("超时，停止录音");
            break;
        }
        EndpointResult::TooShort => {
            // 语音过短，忽略
            println!("噪声过滤");
            detector.reset();
        }
    }
}
```

### ASR 端点检测

```rust
// 与 Sherpa-ONNX 集成
loop {
    // 处理音频帧
    stream.accept_waveform(&audio_data);

    // 检查 ASR 端点
    let asr_endpoint = stream.is_endpoint(&recognizer);

    match detector.process_asr_endpoint(asr_endpoint) {
        EndpointResult::Detected => {
            let text = stream.get_result(&recognizer);
            println!("识别结果: {}", text);
            break;
        }
        EndpointResult::TooShort => {
            println!("语音过短，忽略");
            detector.reset();
        }
        _ => {
            // 继续
        }
    }
}
```

### 完整流程示例

```rust
use vinput_core::endpointing::{EndpointDetector, EndpointResult};
use vinput_core::vad::SileroVAD;
use vinput_core::asr::{OnlineRecognizer, OnlineStream};

fn voice_input_loop(
    vad: &mut SileroVAD,
    recognizer: &OnlineRecognizer,
) -> String {
    let mut detector = EndpointDetector::default_config();
    let mut stream = recognizer.create_stream();

    println!("开始录音...");

    loop {
        // 获取音频帧（16kHz, 单声道）
        let audio_frame = capture_audio_frame();

        // 1. VAD 检测
        let is_speech = vad.process_frame(&audio_frame);

        // 2. 端点检测
        let endpoint_result = detector.process_vad(is_speech);

        // 3. ASR 处理
        if is_speech {
            stream.accept_waveform(&audio_frame);

            // 检查 ASR 端点
            let asr_endpoint = stream.is_endpoint(recognizer);
            if asr_endpoint {
                match detector.process_asr_endpoint(true) {
                    EndpointResult::Detected => {
                        let result = stream.get_result(recognizer);
                        return result.text;
                    }
                    _ => {}
                }
            }
        }

        // 4. 处理端点结果
        match endpoint_result {
            EndpointResult::Continue => {
                // 继续录音
            }
            EndpointResult::Detected => {
                // VAD 检测到端点
                let result = stream.get_result(recognizer);
                return result.text;
            }
            EndpointResult::ForcedSegmentation => {
                // 语音过长，分段处理
                let result = stream.get_result(recognizer);
                println!("分段结果: {}", result.text);

                // 重置检测器，继续下一段
                detector.reset();
                stream.reset();
            }
            EndpointResult::Timeout => {
                println!("超时");
                break;
            }
            EndpointResult::TooShort => {
                println!("语音过短");
                detector.reset();
                stream.reset();
            }
        }
    }

    String::new()
}
```

## 状态查询

```rust
// 检查是否检测到语音
if detector.is_speech_detected() {
    println!("正在说话");
}

// 获取当前语音持续时间
let duration = detector.speech_duration();
println!("已录制 {} 秒", duration.as_secs());

// 获取会话持续时间
let session = detector.session_duration();
println!("会话进行 {} 秒", session.as_secs());
```

## 最佳实践

### 1. 参数调优

针对不同场景调整参数：

```rust
// 快速响应场景（短语音）
let config = EndpointDetectorConfig {
    min_speech_duration_ms: 200,    // 更短的最小长度
    trailing_silence_ms: 500,        // 更短的尾部静音
    ..Default::default()
};

// 长语音场景（听写、语音备忘录）
let config = EndpointDetectorConfig {
    min_speech_duration_ms: 500,
    max_speech_duration_ms: 120_000,  // 2分钟
    trailing_silence_ms: 1500,        // 更长的尾部静音
    ..Default::default()
};

// 噪声环境
let config = EndpointDetectorConfig {
    vad_silence_confirm_frames: 8,   // 更多确认帧
    trailing_silence_ms: 1000,
    ..Default::default()
};
```

### 2. 结合 VAD 和 ASR

同时使用两种端点检测方式获得最佳效果：

```rust
// VAD 提供快速响应
let vad_result = detector.process_vad(is_speech);

// ASR 提供准确边界
let asr_result = detector.process_asr_endpoint(asr_endpoint);

// 任一方式检测到端点都可以结束
if matches!(vad_result, EndpointResult::Detected) ||
   matches!(asr_result, EndpointResult::Detected) {
    finalize_recognition();
}
```

### 3. 错误处理

```rust
match detector.process_vad(is_speech) {
    EndpointResult::Timeout => {
        log_error("端点检测超时，可能麦克风故障");
        show_user_error("录音超时，请重试");
    }
    EndpointResult::TooShort => {
        // 静默处理，无需通知用户
        reset_ui();
    }
    _ => {}
}
```

## 性能考虑

- **VAD 处理**: 每帧 32ms，CPU 占用 < 1%
- **端点检测**: 无需深度计算，仅状态管理
- **内存占用**: < 1KB

## 调试

启用日志查看详细端点检测过程：

```bash
export VINPUT_LOG=debug
cargo run --features debug-logs --example endpoint_demo
```

日志输出示例：
```
端点检测: 语音开始
端点检测: 进入尾部静音阶段
端点检测: 检测到端点 (语音: 2345ms, 静音: 850ms)
```
