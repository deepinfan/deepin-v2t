//! Transient Filter - 短爆发噪声过滤器
//!
//! 过滤键盘敲击、鼠标点击等短暂高能量噪声

use crate::vad::config::TransientFilterConfig;
use std::time::{Duration, Instant};

/// 短爆发检测状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransientState {
    /// 正常状态
    Normal,
    /// 检测到可能的短爆发
    PossibleTransient,
}

/// 短爆发噪声过滤器
pub struct TransientFilter {
    config: TransientFilterConfig,
    state: TransientState,
    transient_start_time: Option<Instant>,
    peak_rms: f32,
}

impl TransientFilter {
    /// 创建新的短爆发过滤器
    pub fn new(config: TransientFilterConfig) -> Self {
        Self {
            config,
            state: TransientState::Normal,
            transient_start_time: None,
            peak_rms: 0.0,
        }
    }

    /// 处理音频帧，判断是否为短爆发噪声
    ///
    /// # 参数
    /// - `samples`: 音频样本 (f32, [-1.0, 1.0])
    /// - `is_speech`: VAD 判断是否为语音
    ///
    /// # 返回
    /// - `true`: 正常语音，应该处理
    /// - `false`: 疑似短爆发噪声，应该过滤
    pub fn process(&mut self, samples: &[f32], is_speech: bool) -> bool {
        if !self.config.enabled {
            return true; // 禁用时，所有帧都通过
        }

        let rms = self.calculate_rms(samples);
        let now = Instant::now();

        match self.state {
            TransientState::Normal => {
                if is_speech && rms > self.config.rms_threshold {
                    // 检测到高能量语音，进入可能的短爆发状态
                    self.state = TransientState::PossibleTransient;
                    self.transient_start_time = Some(now);
                    self.peak_rms = rms;
                    tracing::trace!("TransientFilter: Normal → PossibleTransient (RMS={:.3})", rms);

                    // 暂时允许通过，等待持续时间判断
                    true
                } else {
                    // 低能量或非语音，正常通过
                    true
                }
            }

            TransientState::PossibleTransient => {
                // 更新峰值 RMS
                if rms > self.peak_rms {
                    self.peak_rms = rms;
                }

                if !is_speech {
                    // 语音结束，检查持续时间
                    if let Some(start_time) = self.transient_start_time {
                        let duration = now.duration_since(start_time);

                        if duration < Duration::from_millis(self.config.max_duration_ms) {
                            // 持续时间太短，判定为短爆发噪声
                            tracing::debug!(
                                "TransientFilter: Filtered transient noise (duration={:?}, peak_rms={:.3})",
                                duration,
                                self.peak_rms
                            );
                            self.reset_state();
                            return false; // 过滤掉
                        } else {
                            // 持续时间足够长，是正常语音
                            tracing::trace!(
                                "TransientFilter: Confirmed as speech (duration={:?})",
                                duration
                            );
                            self.reset_state();
                            return true;
                        }
                    }

                    self.reset_state();
                    true
                } else if rms < self.config.rms_threshold * 0.5 {
                    // RMS 显著下降，可能是短爆发结束
                    if let Some(start_time) = self.transient_start_time {
                        let duration = now.duration_since(start_time);

                        if duration < Duration::from_millis(self.config.max_duration_ms) {
                            tracing::debug!(
                                "TransientFilter: Filtered transient (RMS drop, duration={:?})",
                                duration
                            );
                            self.reset_state();
                            return false;
                        }
                    }

                    self.reset_state();
                    true
                } else {
                    // 继续观察
                    true
                }
            }
        }
    }

    /// 计算 RMS 能量
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// 重置状态
    fn reset_state(&mut self) {
        self.state = TransientState::Normal;
        self.transient_start_time = None;
        self.peak_rms = 0.0;
    }

    /// 重置过滤器
    pub fn reset(&mut self) {
        self.reset_state();
        tracing::debug!("TransientFilter reset");
    }

    /// 获取当前状态（用于调试）
    pub fn state(&self) -> &str {
        match self.state {
            TransientState::Normal => "Normal",
            TransientState::PossibleTransient => "PossibleTransient",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transient_filter_normal_speech() {
        let config = TransientFilterConfig {
            enabled: true,
            max_duration_ms: 80,
            rms_threshold: 0.05,
        };

        let mut filter = TransientFilter::new(config);

        // 模拟正常语音（高能量，持续时间长）
        let speech: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin() * 0.1).collect();

        // 持续多帧，模拟正常语音
        for _ in 0..20 {
            assert!(filter.process(&speech, true)); // 应该通过
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn test_transient_filter_keyboard_click() {
        let config = TransientFilterConfig {
            enabled: true,
            max_duration_ms: 80,
            rms_threshold: 0.05,
        };

        let mut filter = TransientFilter::new(config);

        // 模拟键盘敲击（高能量，短持续时间）
        let click: Vec<f32> = (0..512).map(|i| (i as f32 * 0.05).sin() * 0.15).collect();
        let silence = vec![0.0f32; 512];

        // 第一帧：高能量
        assert!(filter.process(&click, true)); // 暂时通过

        // 短暂延迟（< 80ms）
        std::thread::sleep(Duration::from_millis(20));

        // 第二帧：能量下降或静音
        let result = filter.process(&silence, false);

        // 应该被过滤（持续时间太短）
        assert!(!result);
    }

    #[test]
    fn test_transient_filter_disabled() {
        let config = TransientFilterConfig {
            enabled: false,
            max_duration_ms: 80,
            rms_threshold: 0.05,
        };

        let mut filter = TransientFilter::new(config);

        // 禁用时，所有帧都应该通过
        let click: Vec<f32> = (0..512).map(|i| (i as f32 * 0.05).sin() * 0.15).collect();
        assert!(filter.process(&click, true));
        assert!(filter.process(&click, false));
    }

    #[test]
    fn test_transient_filter_low_energy() {
        let config = TransientFilterConfig {
            enabled: true,
            max_duration_ms: 80,
            rms_threshold: 0.05,
        };

        let mut filter = TransientFilter::new(config);

        // 低能量音频，不应该触发短爆发检测
        let low_energy: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin() * 0.01).collect();

        assert!(filter.process(&low_energy, true)); // 应该通过
        assert_eq!(filter.state(), "Normal");
    }

    #[test]
    fn test_transient_filter_reset() {
        let config = TransientFilterConfig {
            enabled: true,
            max_duration_ms: 80,
            rms_threshold: 0.05,
        };

        let mut filter = TransientFilter::new(config);

        // 触发检测
        let click: Vec<f32> = (0..512).map(|i| (i as f32 * 0.05).sin() * 0.15).collect();
        filter.process(&click, true);
        assert_eq!(filter.state(), "PossibleTransient");

        // 重置
        filter.reset();
        assert_eq!(filter.state(), "Normal");
    }
}
