//! VAD Manager - 统一的 VAD 管理器
//!
//! 集成所有 VAD 组件，提供统一的接口

use crate::error::VInputResult;
use crate::vad::{
    config::VadConfig, energy_gate::EnergyGate, hysteresis::HysteresisController,
    hysteresis::VadState, pre_roll_buffer::PreRollBuffer, transient_filter::TransientFilter,
};

#[cfg(feature = "vad-onnx")]
use crate::vad::silero::SileroVAD;

/// VAD 处理结果
#[derive(Debug, Clone)]
pub struct VadResult {
    /// 当前 VAD 状态
    pub state: VadState,
    /// 是否发生状态转换
    pub state_changed: bool,
    /// Silero VAD 输出的语音概率
    pub speech_prob: f32,
    /// Pre-roll 音频数据（仅在状态转换为 Speech 时有效）
    pub pre_roll_audio: Option<Vec<f32>>,
    /// 是否通过 Energy Gate
    pub passed_energy_gate: bool,
    /// 是否通过 Transient Filter
    pub passed_transient_filter: bool,
}

/// VAD 管理器（集成所有组件）
pub struct VadManager {
    config: VadConfig,
    energy_gate: EnergyGate,
    hysteresis: HysteresisController,
    pre_roll_buffer: PreRollBuffer,
    transient_filter: TransientFilter,

    #[cfg(feature = "vad-onnx")]
    silero_vad: SileroVAD,

    /// 上一次的 VAD 状态（用于检测状态转换）
    last_state: VadState,

    /// 诊断计数器（周期性打印 VAD 内部状态）
    diag_frame_count: u64,
    diag_energy_gate_pass: u64,
    diag_max_prob: f32,
    diag_max_rms: f32,
}

impl VadManager {
    /// 创建新的 VAD 管理器
    #[cfg(feature = "vad-onnx")]
    pub fn new(config: VadConfig) -> VInputResult<Self> {
        // 创建 Silero VAD 配置
        let silero_config = crate::vad::silero::SileroVADConfig {
            model_path: config.silero.model_path.clone(),
            sample_rate: config.silero.sample_rate,
            threshold: config.hysteresis.start_threshold,
            min_speech_duration_ms: config.hysteresis.min_speech_duration_ms as u32,
            min_silence_duration_ms: config.hysteresis.min_silence_duration_ms as u32,
        };

        let silero_vad = SileroVAD::new(silero_config)?;

        Ok(Self {
            energy_gate: EnergyGate::new(config.energy_gate.clone()),
            hysteresis: HysteresisController::new(config.hysteresis.clone()),
            pre_roll_buffer: PreRollBuffer::new(config.pre_roll.clone()),
            transient_filter: TransientFilter::new(config.transient_filter.clone()),
            silero_vad,
            last_state: VadState::Silence,
            diag_frame_count: 0,
            diag_energy_gate_pass: 0,
            diag_max_prob: 0.0,
            diag_max_rms: 0.0,
            config,
        })
    }

    /// 处理音频帧
    ///
    /// # 参数
    /// - `samples`: 音频样本 (f32, [-1.0, 1.0])
    ///   - 对于 16kHz: 512 samples (32ms)
    ///   - 对于 8kHz: 256 samples (32ms)
    ///
    /// # 返回
    /// - `VadResult`: VAD 处理结果
    #[cfg(feature = "vad-onnx")]
    pub fn process(&mut self, samples: &[f32]) -> VInputResult<VadResult> {
        // 1. Energy Gate - 第一层过滤
        let passed_energy_gate = self.energy_gate.process(samples);

        let (speech_prob, state, state_changed) = if passed_energy_gate {
            // 2. Silero VAD - 核心检测
            let prob = self.silero_vad.process_chunk(samples)?;

            // 每帧记录 silero 原始概率（DEBUG 级别，info 模式下不显示）
            tracing::debug!("VAD prob={:.3}", prob);

            // 诊断统计
            self.diag_energy_gate_pass += 1;
            if prob > self.diag_max_prob {
                self.diag_max_prob = prob;
            }

            // 3. Hysteresis Controller - 状态管理
            let (new_state, changed) = self.hysteresis.process(prob);

            (prob, new_state, changed)
        } else {
            tracing::debug!("VAD EnergyGate blocked");
            // Energy Gate 未通过，直接返回低概率
            let (new_state, changed) = self.hysteresis.process(0.0);
            (0.0, new_state, changed)
        };

        // 更新 RMS 诊断统计
        let rms = {
            let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
            (sum_sq / samples.len() as f32).sqrt()
        };
        if rms > self.diag_max_rms {
            self.diag_max_rms = rms;
        }

        self.diag_frame_count += 1;

        // 每 50 帧（约 1.6 秒）打印一次诊断信息（INFO 级别，始终可见）
        const DIAG_INTERVAL: u64 = 50;
        if self.diag_frame_count % DIAG_INTERVAL == 0 {
            let pass_ratio = self.diag_energy_gate_pass as f64 / DIAG_INTERVAL as f64;
            tracing::info!(
                "VAD 诊断 [帧 {}]: EnergyGate通过={:.0}%, 最高RMS={:.4}, 最高prob={:.3}, start_thresh={:.2}, 当前状态={:?}",
                self.diag_frame_count,
                pass_ratio * 100.0,
                self.diag_max_rms,
                self.diag_max_prob,
                self.config.hysteresis.start_threshold,
                state,
            );
            // 重置窗口统计
            self.diag_energy_gate_pass = 0;
            self.diag_max_prob = 0.0;
            self.diag_max_rms = 0.0;
        }

        // 4. Transient Filter - 短爆发过滤
        let is_speech = matches!(state, VadState::Speech | VadState::SpeechCandidate);
        let passed_transient_filter = self.transient_filter.process(samples, is_speech);

        // 5. Pre-roll Buffer 管理
        let mut pre_roll_audio = None;

        match state {
            VadState::Silence => {
                // 静音时，持续更新 Pre-roll Buffer
                self.pre_roll_buffer.push(samples);
            }
            VadState::SpeechCandidate => {
                // 语音候选状态，继续缓冲
                self.pre_roll_buffer.push(samples);
            }
            VadState::Speech => {
                // 刚进入语音状态
                if state_changed {
                    // 提取 Pre-roll 音频（语音开始前的缓冲）
                    pre_roll_audio = Some(self.pre_roll_buffer.retrieve());
                    tracing::debug!(
                        "VAD: Speech started, pre-roll buffer: {} samples ({} ms)",
                        pre_roll_audio.as_ref().unwrap().len(),
                        self.pre_roll_buffer
                            .buffered_duration_ms(self.config.silero.sample_rate)
                    );
                }
                // 进入语音后不再更新 Pre-roll Buffer
            }
            VadState::SilenceCandidate => {
                // 静音候选状态，不更新缓冲
            }
        }

        self.last_state = state;

        Ok(VadResult {
            state,
            state_changed,
            speech_prob,
            pre_roll_audio,
            passed_energy_gate,
            passed_transient_filter,
        })
    }

    /// 强制设置 VAD 状态（用于 PushToTalk 模式）
    pub fn force_state(&mut self, state: VadState) {
        self.hysteresis.force_state(state);
        self.last_state = state;

        if matches!(state, VadState::Speech) {
            // 进入语音状态时，清空 Pre-roll Buffer
            self.pre_roll_buffer.clear();
        }
    }

    /// 重置 VAD 状态
    #[cfg(feature = "vad-onnx")]
    pub fn reset(&mut self) {
        self.energy_gate.reset();
        self.hysteresis.reset();
        self.pre_roll_buffer.reset();
        self.transient_filter.reset();
        self.silero_vad.reset();
        self.last_state = VadState::Silence;
        self.diag_frame_count = 0;
        self.diag_energy_gate_pass = 0;
        self.diag_max_prob = 0.0;
        self.diag_max_rms = 0.0;
        tracing::debug!("VadManager reset");
    }

    /// 获取当前状态
    pub fn state(&self) -> VadState {
        self.hysteresis.state()
    }

    /// 获取配置
    pub fn config(&self) -> &VadConfig {
        &self.config
    }

    /// 获取 Pre-roll Buffer 引用（用于调试）
    pub fn pre_roll_buffer(&self) -> &PreRollBuffer {
        &self.pre_roll_buffer
    }

    /// 获取噪声基线（用于调试）
    pub fn noise_baseline(&self) -> f32 {
        self.energy_gate.noise_baseline()
    }
}

// 无 ONNX Runtime 的简化实现（用于编译测试）
#[cfg(not(feature = "vad-onnx"))]
impl VadManager {
    pub fn new(config: VadConfig) -> VInputResult<Self> {
        Ok(Self {
            energy_gate: EnergyGate::new(config.energy_gate.clone()),
            hysteresis: HysteresisController::new(config.hysteresis.clone()),
            pre_roll_buffer: PreRollBuffer::new(config.pre_roll.clone()),
            transient_filter: TransientFilter::new(config.transient_filter.clone()),
            last_state: VadState::Silence,
            diag_frame_count: 0,
            diag_energy_gate_pass: 0,
            diag_max_prob: 0.0,
            diag_max_rms: 0.0,
            config,
        })
    }

    pub fn process(&mut self, samples: &[f32]) -> VInputResult<VadResult> {
        // 简化版本：仅使用 Energy Gate
        let passed_energy_gate = self.energy_gate.process(samples);

        // 基于 Energy Gate 的简单概率估计
        let speech_prob = if passed_energy_gate { 0.8 } else { 0.2 };

        let (state, state_changed) = self.hysteresis.process(speech_prob);

        let is_speech = matches!(state, VadState::Speech | VadState::SpeechCandidate);
        let passed_transient_filter = self.transient_filter.process(samples, is_speech);

        let mut pre_roll_audio = None;
        if matches!(state, VadState::Silence | VadState::SpeechCandidate) {
            self.pre_roll_buffer.push(samples);
        } else if state_changed && matches!(state, VadState::Speech) {
            pre_roll_audio = Some(self.pre_roll_buffer.retrieve());
        }

        self.last_state = state;

        Ok(VadResult {
            state,
            state_changed,
            speech_prob,
            pre_roll_audio,
            passed_energy_gate,
            passed_transient_filter,
        })
    }

    pub fn reset(&mut self) {
        self.energy_gate.reset();
        self.hysteresis.reset();
        self.pre_roll_buffer.reset();
        self.transient_filter.reset();
        self.last_state = VadState::Silence;
        self.diag_frame_count = 0;
        self.diag_energy_gate_pass = 0;
        self.diag_max_prob = 0.0;
        self.diag_max_rms = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "vad-onnx"))]
    fn test_vad_manager_without_onnx() {
        let config = VadConfig::push_to_talk_default();
        let mut manager = VadManager::new(config).expect("Failed to create VadManager");

        // 模拟语音
        let speech: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin() * 0.1).collect();

        let result = manager.process(&speech).expect("Failed to process");

        assert!(result.passed_energy_gate);
        assert_eq!(manager.state(), result.state);
    }

    #[test]
    fn test_vad_manager_force_state() {
        let config = VadConfig::push_to_talk_default();
        let mut manager = VadManager::new(config).expect("Failed to create VadManager");

        assert_eq!(manager.state(), VadState::Silence);

        manager.force_state(VadState::Speech);
        assert_eq!(manager.state(), VadState::Speech);

        manager.force_state(VadState::Silence);
        assert_eq!(manager.state(), VadState::Silence);
    }
}
