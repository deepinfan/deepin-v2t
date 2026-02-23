//! Energy Gate - 第一层音频过滤
//!
//! 通过能量检测过滤环境噪声，减少送入 Silero VAD 的帧数

use crate::vad::config::EnergyGateConfig;

/// Energy Gate 状态
pub struct EnergyGate {
    config: EnergyGateConfig,
    frame_count: u64,
}

impl EnergyGate {
    pub fn new(config: EnergyGateConfig) -> Self {
        Self {
            config,
            frame_count: 0,
        }
    }

    /// 处理音频帧，返回是否应该送入 VAD
    ///
    /// 简单规则：RMS > initial_baseline 即通过（固定阈值，无动态基线）
    pub fn process(&mut self, samples: &[f32]) -> bool {
        if !self.config.enabled {
            return true;
        }

        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();
        let pass = rms > self.config.initial_baseline;

        self.frame_count += 1;
        if self.frame_count % 100 == 0 {
            tracing::trace!("EnergyGate: RMS={:.6}, threshold={:.6}, pass={}", rms, self.config.initial_baseline, pass);
        }

        pass
    }

    /// 重置 Energy Gate 状态
    pub fn reset(&mut self) {
        self.frame_count = 0;
        tracing::debug!("EnergyGate reset");
    }

    pub fn noise_baseline(&self) -> f32 {
        self.config.initial_baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_gate_silence() {
        let config = EnergyGateConfig {
            enabled: true,
            noise_multiplier: 2.5,
            baseline_alpha: 0.95,
            initial_baseline: 0.001,
        };

        let mut gate = EnergyGate::new(config);

        // 静音样本
        let silence = vec![0.0f32; 512];
        assert!(!gate.process(&silence)); // 应该不通过
    }

    #[test]
    fn test_energy_gate_speech() {
        let config = EnergyGateConfig {
            enabled: true,
            noise_multiplier: 2.5,
            baseline_alpha: 0.95,
            initial_baseline: 0.001,
        };

        let mut gate = EnergyGate::new(config);

        // 模拟语音（较高能量）
        let speech: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin() * 0.1).collect();
        assert!(gate.process(&speech)); // 应该通过
    }

    #[test]
    fn test_energy_gate_disabled() {
        let config = EnergyGateConfig {
            enabled: false,
            noise_multiplier: 2.5,
            baseline_alpha: 0.95,
            initial_baseline: 0.001,
        };

        let mut gate = EnergyGate::new(config);

        // 禁用时，所有帧都应该通过
        let silence = vec![0.0f32; 512];
        assert!(gate.process(&silence));
    }
}
