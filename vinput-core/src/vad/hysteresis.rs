//! Hysteresis Controller - 迟滞控制器
//!
//! 实现双阈值状态机，防止语音/静音边界抖动

use crate::vad::config::HysteresisConfig;
use std::time::{Duration, Instant};

/// VAD 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadState {
    /// 静音状态
    Silence,
    /// 语音候选状态（刚检测到可能的语音）
    SpeechCandidate,
    /// 确认的语音状态
    Speech,
    /// 静音候选状态（检测到可能的静音）
    SilenceCandidate,
}

/// 迟滞控制器
pub struct HysteresisController {
    config: HysteresisConfig,
    state: VadState,
    state_enter_time: Option<Instant>,
    consecutive_speech_frames: usize,
    consecutive_silence_frames: usize,
}

impl HysteresisController {
    /// 创建新的迟滞控制器
    pub fn new(config: HysteresisConfig) -> Self {
        Self {
            config,
            state: VadState::Silence,
            state_enter_time: None,
            consecutive_speech_frames: 0,
            consecutive_silence_frames: 0,
        }
    }

    /// 处理 VAD 概率值，返回当前状态和是否发生状态转换
    ///
    /// # 参数
    /// - `speech_prob`: Silero VAD 输出的语音概率 [0.0, 1.0]
    ///
    /// # 返回
    /// - `(VadState, bool)`: (当前状态, 是否发生转换)
    pub fn process(&mut self, speech_prob: f32) -> (VadState, bool) {
        let old_state = self.state;
        let now = Instant::now();

        match self.state {
            VadState::Silence => {
                if speech_prob > self.config.start_threshold {
                    self.consecutive_speech_frames += 1;
                    self.consecutive_silence_frames = 0;

                    // 进入语音候选状态
                    self.state = VadState::SpeechCandidate;
                    self.state_enter_time = Some(now);
                    tracing::debug!("VAD: Silence → SpeechCandidate (prob={:.3})", speech_prob);
                } else {
                    self.consecutive_silence_frames += 1;
                    self.consecutive_speech_frames = 0;
                }
            }

            VadState::SpeechCandidate => {
                if speech_prob > self.config.start_threshold {
                    self.consecutive_speech_frames += 1;
                    self.consecutive_silence_frames = 0;

                    // 检查是否满足最小语音持续时间
                    if let Some(enter_time) = self.state_enter_time {
                        let duration = now.duration_since(enter_time);
                        if duration
                            >= Duration::from_millis(self.config.min_speech_duration_ms)
                        {
                            self.state = VadState::Speech;
                            tracing::info!("VAD: SpeechCandidate → Speech (duration={:?})", duration);
                        }
                    }
                } else {
                    // 概率下降，返回静音
                    self.state = VadState::Silence;
                    self.state_enter_time = None;
                    self.consecutive_speech_frames = 0;
                    self.consecutive_silence_frames += 1;
                    tracing::debug!("VAD: SpeechCandidate → Silence (prob={:.3})", speech_prob);
                }
            }

            VadState::Speech => {
                if speech_prob < self.config.end_threshold {
                    self.consecutive_silence_frames += 1;
                    self.consecutive_speech_frames = 0;

                    // 进入静音候选状态
                    self.state = VadState::SilenceCandidate;
                    self.state_enter_time = Some(now);
                    tracing::info!("VAD: Speech → SilenceCandidate (prob={:.3})", speech_prob);
                } else {
                    self.consecutive_speech_frames += 1;
                    self.consecutive_silence_frames = 0;
                    // 每 30 帧（约 1 秒）记录一次概率，帮助诊断
                    if self.consecutive_speech_frames % 30 == 1 {
                        tracing::info!(
                            "VAD: Speech 持续 (prob={:.3}, end_thresh={:.2}, start_thresh={:.2}, frames={})",
                            speech_prob,
                            self.config.end_threshold,
                            self.config.start_threshold,
                            self.consecutive_speech_frames,
                        );
                    }
                }
            }

            VadState::SilenceCandidate => {
                if speech_prob >= self.config.start_threshold {
                    // 强语音信号（≥ start_threshold），立即恢复语音状态
                    self.state = VadState::Speech;
                    self.state_enter_time = None;
                    self.consecutive_silence_frames = 0;
                    self.consecutive_speech_frames += 1;
                    tracing::info!("VAD: SilenceCandidate → Speech (prob={:.3}, 强语音)", speech_prob);
                } else if speech_prob < self.config.end_threshold {
                    // 明确静音（< end_threshold），继续计时
                    self.consecutive_silence_frames += 1;
                    self.consecutive_speech_frames = 0;

                    // 检查是否满足最小静音持续时间
                    if let Some(enter_time) = self.state_enter_time {
                        let duration = now.duration_since(enter_time);
                        if duration
                            >= Duration::from_millis(self.config.min_silence_duration_ms)
                        {
                            self.state = VadState::Silence;
                            tracing::info!("VAD: SilenceCandidate → Silence (duration={:?})", duration);
                        }
                    }
                }
                // else: end_threshold ≤ prob < start_threshold → 保持 SilenceCandidate（死区）
                // 背景噪声在此区间时，计时继续运行，不返回语音状态
            }
        }

        let state_changed = old_state != self.state;
        (self.state, state_changed)
    }

    /// 强制设置状态（用于外部控制，如 PushToTalk）
    pub fn force_state(&mut self, state: VadState) {
        if self.state != state {
            tracing::debug!("VAD: Force state {:?} → {:?}", self.state, state);
            self.state = state;
            self.state_enter_time = Some(Instant::now());
            self.consecutive_speech_frames = 0;
            self.consecutive_silence_frames = 0;
        }
    }

    /// 重置控制器状态
    pub fn reset(&mut self) {
        self.state = VadState::Silence;
        self.state_enter_time = None;
        self.consecutive_speech_frames = 0;
        self.consecutive_silence_frames = 0;
        tracing::debug!("HysteresisController reset");
    }

    /// 获取当前状态
    pub fn state(&self) -> VadState {
        self.state
    }

    /// 是否处于语音状态（Speech 或 SpeechCandidate）
    pub fn is_speech(&self) -> bool {
        matches!(
            self.state,
            VadState::Speech | VadState::SpeechCandidate
        )
    }

    /// 是否处于静音状态（Silence 或 SilenceCandidate）
    pub fn is_silence(&self) -> bool {
        matches!(
            self.state,
            VadState::Silence | VadState::SilenceCandidate
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hysteresis_silence_to_speech() {
        let config = HysteresisConfig {
            start_threshold: 0.6,
            end_threshold: 0.3,
            min_speech_duration_ms: 50,
            min_silence_duration_ms: 100,
        };

        let mut controller = HysteresisController::new(config);

        // 初始状态应该是 Silence
        assert_eq!(controller.state(), VadState::Silence);

        // 高概率触发 → SpeechCandidate
        let (state, _) = controller.process(0.7);
        assert_eq!(state, VadState::SpeechCandidate);

        // 持续高概率 + 等待时间 → Speech
        std::thread::sleep(Duration::from_millis(60));
        let (state, _) = controller.process(0.8);
        assert_eq!(state, VadState::Speech);
    }

    #[test]
    fn test_hysteresis_speech_to_silence() {
        let config = HysteresisConfig {
            start_threshold: 0.6,
            end_threshold: 0.3,
            min_speech_duration_ms: 50,
            min_silence_duration_ms: 100,
        };

        let mut controller = HysteresisController::new(config);

        // 强制设置为 Speech
        controller.force_state(VadState::Speech);
        assert_eq!(controller.state(), VadState::Speech);

        // 低概率触发 → SilenceCandidate
        let (state, _) = controller.process(0.2);
        assert_eq!(state, VadState::SilenceCandidate);

        // 持续低概率 + 等待时间 → Silence
        std::thread::sleep(Duration::from_millis(110));
        let (state, _) = controller.process(0.1);
        assert_eq!(state, VadState::Silence);
    }

    #[test]
    fn test_hysteresis_prevent_flapping() {
        let config = HysteresisConfig {
            start_threshold: 0.6,
            end_threshold: 0.3,
            min_speech_duration_ms: 100,
            min_silence_duration_ms: 200,
        };

        let mut controller = HysteresisController::new(config);

        // 短暂的高概率不应该立即转为 Speech
        controller.process(0.7); // → SpeechCandidate
        let (state, _) = controller.process(0.5); // 低于启动阈值
        assert_eq!(state, VadState::Silence); // 应该返回 Silence
    }
}
