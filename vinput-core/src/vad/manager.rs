//! VAD Manager - 简化版
//!
//! 能量门 + Silero VAD，输出 is_speech / speech_prob / pre_roll

use crate::error::VInputResult;
use crate::vad::{config::VadConfig, energy_gate::EnergyGate};

#[cfg(feature = "vad-onnx")]
use crate::vad::silero::SileroVAD;

/// VAD 处理结果
pub struct VadResult {
    pub is_speech: bool,
    pub speech_prob: f32,
    /// 语音开始时的 pre-roll 音频（仅在 is_speech 首次为 true 时有值）
    pub pre_roll: Option<Vec<f32>>,
}

/// VAD 管理器
pub struct VadManager {
    config: VadConfig,
    energy_gate: EnergyGate,

    #[cfg(feature = "vad-onnx")]
    silero: SileroVAD,

    /// pre-roll 环形缓冲（静音期间持续更新，语音开始时取出）
    pre_roll: std::collections::VecDeque<Vec<f32>>,
    /// pre-roll 最大帧数（512ms / 32ms = 16 帧）
    pre_roll_max_frames: usize,

    /// warmup 缓冲（静音帧，full_reset 后重放给 Silero 建立噪声基线）
    #[cfg(feature = "vad-onnx")]
    warmup: std::collections::VecDeque<Vec<f32>>,

    /// 当前是否处于语音状态
    is_speech: bool,
    /// 连续语音帧数（用于起始确认）
    speech_frames: u32,
    /// 连续静音帧数（用于结束确认，由 pipeline 层决策）
    #[allow(dead_code)]
    silence_frames: u32,

    /// 诊断
    diag_frame: u64,
    diag_gate_pass: u64,
    diag_max_prob: f32,
    diag_max_rms: f32,
}

impl VadManager {
    #[cfg(feature = "vad-onnx")]
    pub fn new(config: VadConfig) -> VInputResult<Self> {
        use crate::vad::silero::{SileroVAD, SileroVADConfig};
        let silero_cfg = SileroVADConfig {
            model_path: config.silero.model_path.clone(),
            sample_rate: config.silero.sample_rate,
            threshold: config.hysteresis.start_threshold,
            min_speech_duration_ms: config.hysteresis.min_speech_duration_ms as u32,
            min_silence_duration_ms: config.hysteresis.min_silence_duration_ms as u32,
        };
        let silero = SileroVAD::new(silero_cfg)?;
        let pre_roll_max_frames = 16; // 512ms @ 32ms/frame
        Ok(Self {
            energy_gate: EnergyGate::new(config.energy_gate.clone()),
            silero,
            pre_roll: std::collections::VecDeque::with_capacity(pre_roll_max_frames),
            pre_roll_max_frames,
            warmup: std::collections::VecDeque::with_capacity(16),
            is_speech: false,
            speech_frames: 0,
            silence_frames: 0,
            diag_frame: 0,
            diag_gate_pass: 0,
            diag_max_prob: 0.0,
            diag_max_rms: 0.0,
            config,
        })
    }

    #[cfg(not(feature = "vad-onnx"))]
    pub fn new(config: VadConfig) -> VInputResult<Self> {
        Ok(Self {
            energy_gate: EnergyGate::new(config.energy_gate.clone()),
            pre_roll: std::collections::VecDeque::with_capacity(16),
            pre_roll_max_frames: 16,
            is_speech: false,
            speech_frames: 0,
            silence_frames: 0,
            diag_frame: 0,
            diag_gate_pass: 0,
            diag_max_prob: 0.0,
            diag_max_rms: 0.0,
            config,
        })
    }

    /// 处理一帧音频，返回 VAD 结果
    #[cfg(feature = "vad-onnx")]
    pub fn process(&mut self, samples: &[f32]) -> VInputResult<VadResult> {
        // 能量门
        let passed = self.energy_gate.process(samples);

        let speech_prob = if passed {
            self.diag_gate_pass += 1;
            let p = self.silero.process_chunk(samples)?;
            if p > self.diag_max_prob { self.diag_max_prob = p; }
            p
        } else {
            0.0
        };

        // 诊断
        let rms = {
            let s: f32 = samples.iter().map(|&x| x * x).sum();
            (s / samples.len() as f32).sqrt()
        };
        if rms > self.diag_max_rms { self.diag_max_rms = rms; }
        self.diag_frame += 1;
        if self.diag_frame % 50 == 0 {
            tracing::info!(
                "VAD 诊断 [帧 {}]: EnergyGate通过={:.0}%, 最高RMS={:.4}, 最高prob={:.3}, thresh={:.2}, 状态={}",
                self.diag_frame,
                self.diag_gate_pass as f64 / 50.0 * 100.0,
                self.diag_max_rms, self.diag_max_prob,
                self.config.hysteresis.start_threshold,
                if self.is_speech { "Speech" } else { "Silence" },
            );
            self.diag_gate_pass = 0;
            self.diag_max_prob = 0.0;
            self.diag_max_rms = 0.0;
        }

        let threshold = self.config.hysteresis.start_threshold;
        let is_above = speech_prob >= threshold;

        let mut pre_roll_out = None;

        if !self.is_speech {
            // 静音状态：更新 pre-roll 和 warmup 缓冲
            if self.pre_roll.len() >= self.pre_roll_max_frames {
                self.pre_roll.pop_front();
            }
            self.pre_roll.push_back(samples.to_vec());

            if is_above {
                self.speech_frames += 1;
            } else {
                self.speech_frames = 0;
                // 只在静音帧更新 warmup
                if self.warmup.len() >= 16 { self.warmup.pop_front(); }
                self.warmup.push_back(samples.to_vec());
            }

            // 连续 2 帧（64ms）以上高概率 → 确认语音开始
            if self.speech_frames >= 2 {
                self.is_speech = true;
                self.silence_frames = 0;
                // 取出 pre-roll
                let audio: Vec<f32> = self.pre_roll.iter().flatten().copied().collect();
                pre_roll_out = Some(audio);
                self.pre_roll.clear();
                tracing::info!("VAD: Silence → Speech (prob={:.3})", speech_prob);
            }
        } else {
            // 语音状态：由 pipeline 层通过帧计数决定端点
            if !is_above {
                self.silence_frames += 1;
            } else {
                self.silence_frames = 0;
            }
        }

        Ok(VadResult {
            is_speech: self.is_speech,
            speech_prob,
            pre_roll: pre_roll_out,
        })
    }

    #[cfg(not(feature = "vad-onnx"))]
    pub fn process(&mut self, samples: &[f32]) -> VInputResult<VadResult> {
        let passed = self.energy_gate.process(samples);
        let speech_prob = if passed { 0.8 } else { 0.2 };
        let is_above = passed;
        let mut pre_roll_out = None;

        if !self.is_speech {
            if self.pre_roll.len() >= self.pre_roll_max_frames {
                self.pre_roll.pop_front();
            }
            self.pre_roll.push_back(samples.to_vec());

            if is_above { self.speech_frames += 1; } else { self.speech_frames = 0; }

            if self.speech_frames >= 2 {
                self.is_speech = true;
                let audio: Vec<f32> = self.pre_roll.iter().flatten().copied().collect();
                pre_roll_out = Some(audio);
                self.pre_roll.clear();
            }
        }

        Ok(VadResult { is_speech: self.is_speech, speech_prob, pre_roll: pre_roll_out })
    }

    /// 软重置（保留 LSTM 状态）
    pub fn reset(&mut self) {
        self.energy_gate.reset();
        self.is_speech = false;
        self.speech_frames = 0;
        self.silence_frames = 0;
        self.pre_roll.clear();
        self.diag_frame = 0;
        self.diag_gate_pass = 0;
        self.diag_max_prob = 0.0;
        self.diag_max_rms = 0.0;
        #[cfg(feature = "vad-onnx")]
        self.silero.reset();
    }

    /// 完整重置（清零 LSTM + warmup 重放）
    #[cfg(feature = "vad-onnx")]
    pub fn full_reset(&mut self) {
        self.energy_gate.reset();
        self.is_speech = false;
        self.speech_frames = 0;
        self.silence_frames = 0;
        self.pre_roll.clear();
        self.diag_frame = 0;
        self.diag_gate_pass = 0;
        self.diag_max_prob = 0.0;
        self.diag_max_rms = 0.0;

        self.silero.full_reset();
        // 重放静音帧，建立噪声基线
        let count = self.warmup.len();
        for frame in &self.warmup {
            let _ = self.silero.process_chunk(frame);
        }
        if count > 0 {
            tracing::debug!("VadManager full_reset: warmup 重放 {} 帧", count);
        }
    }

    pub fn is_speech(&self) -> bool { self.is_speech }
    pub fn noise_baseline(&self) -> f32 { self.energy_gate.noise_baseline() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vad::config::{EnergyGateConfig, HysteresisConfig, SileroConfig, VadConfig, PreRollConfig, TransientFilterConfig};

    fn test_config() -> VadConfig {
        VadConfig {
            energy_gate: EnergyGateConfig {
                enabled: true,
                noise_multiplier: 2.5,
                baseline_alpha: 0.95,
                initial_baseline: 0.001,
            },
            hysteresis: HysteresisConfig {
                start_threshold: 0.6,
                end_threshold: 0.35,
                min_speech_duration_ms: 100,
                min_silence_duration_ms: 500,
                max_candidate_gap_frames: 2,
            },
            silero: SileroConfig {
                model_path: "models/silero-vad/silero_vad.onnx".to_string(),
                sample_rate: 16000,
                frame_size: 512,
            },
            pre_roll: PreRollConfig { enabled: true, duration_ms: 250, capacity: 4000 },
            transient_filter: TransientFilterConfig { enabled: true, max_duration_ms: 80, rms_threshold: 0.05 },
        }
    }

    fn silence() -> Vec<f32> { vec![0.0f32; 512] }
    fn speech() -> Vec<f32> { (0..512).map(|i| (i as f32 * 0.05).sin() * 0.1).collect() }

    #[cfg(not(feature = "vad-onnx"))]
    #[test]
    fn test_silence_does_not_trigger_speech() {
        let mut vad = VadManager::new(test_config()).unwrap();
        for _ in 0..10 {
            let r = vad.process(&silence()).unwrap();
            assert!(!r.is_speech);
            assert!(r.pre_roll.is_none());
        }
    }

    #[cfg(not(feature = "vad-onnx"))]
    #[test]
    fn test_speech_triggers_after_4_frames() {
        let mut vad = VadManager::new(test_config()).unwrap();
        // 前 1 帧：is_speech 仍为 false
        let r = vad.process(&speech()).unwrap();
        assert!(!r.is_speech, "frame 0 should not be speech yet");
        assert!(r.pre_roll.is_none());
        // 第 2 帧：触发 Speech，输出 pre_roll
        let r = vad.process(&speech()).unwrap();
        assert!(r.is_speech);
        assert!(r.pre_roll.is_some());
    }

    #[cfg(not(feature = "vad-onnx"))]
    #[test]
    fn test_pre_roll_contains_audio() {
        let mut vad = VadManager::new(test_config()).unwrap();
        for _ in 0..3 { vad.process(&silence()).unwrap(); }
        vad.process(&speech()).unwrap(); // frame 1
        let r = vad.process(&speech()).unwrap(); // frame 2 → triggers
        let pr = r.pre_roll.unwrap();
        assert!(!pr.is_empty(), "pre_roll should contain audio");
    }

    #[cfg(not(feature = "vad-onnx"))]
    #[test]
    fn test_reset_clears_state() {
        let mut vad = VadManager::new(test_config()).unwrap();
        // 触发语音
        for _ in 0..2 { vad.process(&speech()).unwrap(); }
        assert!(vad.is_speech());
        // 重置后恢复静音状态
        vad.reset();
        assert!(!vad.is_speech());
        // 重置后需要重新积累 2 帧才能触发
        let r = vad.process(&speech()).unwrap();
        assert!(!r.is_speech, "frame 0 after reset should not be speech");
        let r = vad.process(&speech()).unwrap();
        assert!(r.is_speech);
    }
}
