//! 语音端点检测器
//!
//! 基于 VAD 和 ASR 端点的智能语音边界检测
//! Phase 1.5: 端点检测优化

use std::time::{Duration, Instant};

/// 端点检测配置
#[derive(Debug, Clone)]
pub struct EndpointDetectorConfig {
    /// 最小语音长度（毫秒）
    /// 低于此长度的音频段会被忽略（过滤点击音等）
    pub min_speech_duration_ms: u64,

    /// 最大语音长度（毫秒）
    /// 超过此长度会自动分段
    pub max_speech_duration_ms: u64,

    /// 语音结束后的静音等待时间（毫秒）
    /// 用于确认用户说话已结束
    pub trailing_silence_ms: u64,

    /// 强制超时（毫秒）
    /// 即使没有检测到端点，超时后也会强制结束
    pub force_timeout_ms: u64,

    /// 是否启用 VAD 辅助端点检测
    pub vad_assisted: bool,

    /// VAD 检测到静音后的确认帧数
    /// 连续 N 帧静音才确认语音结束
    pub vad_silence_confirm_frames: usize,
}

impl Default for EndpointDetectorConfig {
    fn default() -> Self {
        Self {
            min_speech_duration_ms: 300,        // 300ms 最小语音
            max_speech_duration_ms: 30_000,     // 30s 最大语音（自动分段）
            trailing_silence_ms: 800,           // 800ms 尾部静音
            force_timeout_ms: 60_000,           // 60s 强制超时
            vad_assisted: true,                 // 启用 VAD 辅助
            vad_silence_confirm_frames: 5,      // 5 帧静音确认（约 160ms @ 32ms/frame）
        }
    }
}

/// 端点检测结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// 端点检测状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectorState {
    /// 等待语音开始
    WaitingForSpeech,
    /// 检测到语音
    SpeechDetected,
    /// 语音结束后的静音确认阶段
    TrailingSilence,
}

/// 端点检测器
///
/// 结合 VAD 和 ASR 端点检测，提供智能的语音边界识别
pub struct EndpointDetector {
    config: EndpointDetectorConfig,
    state: DetectorState,

    // 时间跟踪
    speech_start_time: Option<Instant>,
    silence_start_time: Option<Instant>,
    session_start_time: Instant,

    // VAD 状态跟踪
    consecutive_silence_frames: usize,
    consecutive_speech_frames: usize,
}

impl EndpointDetector {
    /// 创建新的端点检测器
    pub fn new(config: EndpointDetectorConfig) -> Self {
        Self {
            config,
            state: DetectorState::WaitingForSpeech,
            speech_start_time: None,
            silence_start_time: None,
            session_start_time: Instant::now(),
            consecutive_silence_frames: 0,
            consecutive_speech_frames: 0,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(EndpointDetectorConfig::default())
    }

    /// 重置检测器状态
    pub fn reset(&mut self) {
        self.state = DetectorState::WaitingForSpeech;
        self.speech_start_time = None;
        self.silence_start_time = None;
        self.session_start_time = Instant::now();
        self.consecutive_silence_frames = 0;
        self.consecutive_speech_frames = 0;
    }

    /// 处理 VAD 检测结果
    ///
    /// # 参数
    /// - `is_speech`: VAD 检测到的语音标志
    ///
    /// # 返回值
    /// 端点检测结果
    pub fn process_vad(&mut self, is_speech: bool) -> EndpointResult {
        // 更新连续帧计数
        if is_speech {
            self.consecutive_speech_frames += 1;
            self.consecutive_silence_frames = 0;
        } else {
            self.consecutive_silence_frames += 1;
            self.consecutive_speech_frames = 0;
        }

        match self.state {
            DetectorState::WaitingForSpeech => {
                if is_speech && self.consecutive_speech_frames >= 2 {
                    // 连续 2 帧语音，确认语音开始
                    self.state = DetectorState::SpeechDetected;
                    self.speech_start_time = Some(Instant::now());
                    tracing::debug!("端点检测: 语音开始");
                }
                EndpointResult::Continue
            }

            DetectorState::SpeechDetected => {
                let speech_duration = self.speech_start_time
                    .map(|t| t.elapsed())
                    .unwrap_or(Duration::ZERO);

                // 检查强制超时
                if self.session_start_time.elapsed().as_millis() as u64 > self.config.force_timeout_ms {
                    tracing::warn!("端点检测: 强制超时 ({}ms)", self.config.force_timeout_ms);
                    return EndpointResult::Timeout;
                }

                // 检查最大语音长度（自动分段）
                if speech_duration.as_millis() as u64 > self.config.max_speech_duration_ms {
                    tracing::info!("端点检测: 达到最大长度，强制分段 ({}ms)",
                        self.config.max_speech_duration_ms);
                    return EndpointResult::ForcedSegmentation;
                }

                // 检测静音
                if !is_speech && self.consecutive_silence_frames >= self.config.vad_silence_confirm_frames {
                    // 进入尾部静音确认阶段
                    self.state = DetectorState::TrailingSilence;
                    self.silence_start_time = Some(Instant::now());
                    tracing::debug!("端点检测: 进入尾部静音阶段");
                }

                EndpointResult::Continue
            }

            DetectorState::TrailingSilence => {
                let silence_duration = self.silence_start_time
                    .map(|t| t.elapsed())
                    .unwrap_or(Duration::ZERO);

                // 如果重新检测到语音，返回语音状态
                if is_speech && self.consecutive_speech_frames >= 2 {
                    tracing::debug!("端点检测: 重新检测到语音，继续");
                    self.state = DetectorState::SpeechDetected;
                    self.silence_start_time = None;
                    return EndpointResult::Continue;
                }

                // 检查静音持续时间
                if silence_duration.as_millis() as u64 >= self.config.trailing_silence_ms {
                    let total_speech_duration = self.speech_start_time
                        .map(|t| t.elapsed())
                        .unwrap_or(Duration::ZERO);

                    // 检查是否低于最小语音长度
                    if (total_speech_duration.as_millis() as u64) < self.config.min_speech_duration_ms {
                        tracing::debug!("端点检测: 语音过短 ({}ms < {}ms)，忽略",
                            total_speech_duration.as_millis(),
                            self.config.min_speech_duration_ms);
                        return EndpointResult::TooShort;
                    }

                    tracing::info!("端点检测: 检测到端点 (语音: {}ms, 静音: {}ms)",
                        total_speech_duration.as_millis(),
                        silence_duration.as_millis());
                    return EndpointResult::Detected;
                }

                EndpointResult::Continue
            }
        }
    }

    /// 处理 ASR 端点检测结果（来自 sherpa-onnx）
    ///
    /// # 参数
    /// - `asr_endpoint`: ASR 引擎报告的端点标志
    ///
    /// # 返回值
    /// 端点检测结果
    pub fn process_asr_endpoint(&mut self, asr_endpoint: bool) -> EndpointResult {
        if !asr_endpoint {
            return EndpointResult::Continue;
        }

        // ASR 检测到端点
        let speech_duration = self.speech_start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        // 检查最小语音长度
        if (speech_duration.as_millis() as u64) < self.config.min_speech_duration_ms {
            tracing::debug!("端点检测: ASR 端点但语音过短，忽略");
            return EndpointResult::Continue;
        }

        tracing::info!("端点检测: ASR 检测到端点 (语音: {}ms)",
            speech_duration.as_millis());
        EndpointResult::Detected
    }

    /// 获取当前语音持续时间
    pub fn speech_duration(&self) -> Duration {
        self.speech_start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// 获取会话持续时间
    pub fn session_duration(&self) -> Duration {
        self.session_start_time.elapsed()
    }

    /// 检查是否检测到语音
    pub fn is_speech_detected(&self) -> bool {
        self.state == DetectorState::SpeechDetected ||
        self.state == DetectorState::TrailingSilence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_detector_basic() {
        let config = EndpointDetectorConfig {
            min_speech_duration_ms: 100,
            max_speech_duration_ms: 1000,
            trailing_silence_ms: 200,
            force_timeout_ms: 5000,
            vad_assisted: true,
            vad_silence_confirm_frames: 3,
        };

        let mut detector = EndpointDetector::new(config);

        // 初始状态
        assert!(!detector.is_speech_detected());

        // 模拟语音开始（需要连续 2 帧）
        assert_eq!(detector.process_vad(true), EndpointResult::Continue);
        assert_eq!(detector.process_vad(true), EndpointResult::Continue);
        assert!(detector.is_speech_detected());

        // 模拟语音进行中
        for _ in 0..10 {
            assert_eq!(detector.process_vad(true), EndpointResult::Continue);
        }

        // 模拟静音（需要连续 3 帧确认）
        assert_eq!(detector.process_vad(false), EndpointResult::Continue);
        assert_eq!(detector.process_vad(false), EndpointResult::Continue);
        assert_eq!(detector.process_vad(false), EndpointResult::Continue);

        // 等待足够长的尾部静音（模拟）
        std::thread::sleep(std::time::Duration::from_millis(250));
        assert_eq!(detector.process_vad(false), EndpointResult::Detected);
    }

    #[test]
    fn test_too_short_speech() {
        let config = EndpointDetectorConfig {
            min_speech_duration_ms: 300,
            trailing_silence_ms: 100,
            ..Default::default()
        };

        let mut detector = EndpointDetector::new(config);

        // 短暂的语音
        detector.process_vad(true);
        detector.process_vad(true);

        std::thread::sleep(std::time::Duration::from_millis(50));

        // 立即静音
        for _ in 0..5 {
            detector.process_vad(false);
        }

        std::thread::sleep(std::time::Duration::from_millis(150));

        // 应该因为过短而被忽略
        assert_eq!(detector.process_vad(false), EndpointResult::TooShort);
    }
}
