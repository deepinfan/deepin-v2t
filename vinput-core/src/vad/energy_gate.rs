//! Energy Gate - 第一层音频过滤
//!
//! 通过能量检测过滤环境噪声，减少送入 Silero VAD 的帧数

use crate::vad::config::EnergyGateConfig;

/// Energy Gate 状态
pub struct EnergyGate {
    config: EnergyGateConfig,
    noise_baseline: f32,
    frame_count: u64,
}

impl EnergyGate {
    /// 创建新的 Energy Gate
    pub fn new(config: EnergyGateConfig) -> Self {
        Self {
            noise_baseline: config.initial_baseline,
            config,
            frame_count: 0,
        }
    }

    /// 处理音频帧，返回是否应该送入 VAD
    ///
    /// # 参数
    /// - `samples`: 音频样本 (f32, [-1.0, 1.0])
    ///
    /// # 返回
    /// - `true`: 能量足够，应该送入 VAD
    /// - `false`: 能量过低，可能是环境噪声
    pub fn process(&mut self, samples: &[f32]) -> bool {
        if !self.config.enabled {
            return true; // 禁用时，所有帧都通过
        }

        // 计算 RMS (Root Mean Square)
        let rms = self.calculate_rms(samples);

        // 更新噪声基线（使用指数移动平均）
        self.update_baseline(rms);

        self.frame_count += 1;

        // 判断是否通过阈值
        let threshold = self.noise_baseline * self.config.noise_multiplier;
        let pass = rms > threshold;

        if self.frame_count % 100 == 0 {
            tracing::trace!(
                "EnergyGate: RMS={:.6}, baseline={:.6}, threshold={:.6}, pass={}",
                rms,
                self.noise_baseline,
                threshold,
                pass
            );
        }

        pass
    }

    /// 计算 RMS 能量
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// 更新噪声基线（指数移动平均）
    fn update_baseline(&mut self, current_rms: f32) {
        // 只在 RMS 较低时更新基线（避免语音抬高基线）
        if current_rms < self.noise_baseline * 2.0 {
            self.noise_baseline = self.config.baseline_alpha * self.noise_baseline
                + (1.0 - self.config.baseline_alpha) * current_rms;
        }
    }

    /// 重置 Energy Gate 状态
    pub fn reset(&mut self) {
        self.noise_baseline = self.config.initial_baseline;
        self.frame_count = 0;
        tracing::debug!("EnergyGate reset");
    }

    /// 获取当前噪声基线
    pub fn noise_baseline(&self) -> f32 {
        self.noise_baseline
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
