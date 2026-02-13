//! Silero VAD ONNX 推理
//!
//! 基于 Silero VAD v5 模型的语音活动检测

use crate::error::{VInputError, VInputResult};
use std::path::Path;

/// Silero VAD 配置
#[derive(Debug, Clone)]
pub struct SileroVADConfig {
    /// 模型路径
    pub model_path: String,
    /// 采样率 (8000 or 16000)
    pub sample_rate: u32,
    /// 语音检测阈值 (0.0-1.0)
    pub threshold: f32,
    /// 最小语音持续时间 (ms)
    pub min_speech_duration_ms: u32,
    /// 最小静音持续时间 (ms)
    pub min_silence_duration_ms: u32,
}

impl Default for SileroVADConfig {
    fn default() -> Self {
        Self {
            model_path: "models/vad/silero_vad_v5.onnx".to_string(),
            sample_rate: 16000,
            threshold: 0.5,
            min_speech_duration_ms: 250,
            min_silence_duration_ms: 100,
        }
    }
}

/// VAD 状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VADState {
    /// 静音
    Silence,
    /// 语音
    Speech,
}

/// Silero VAD 检测器（Phase 0 MVP）
///
/// 注意：当前为接口定义，完整的 ONNX 推理将在 Phase 1 实现
pub struct SileroVAD {
    config: SileroVADConfig,
    state: VADState,
    speech_frames: u32,
    silence_frames: u32,
    // Phase 1: ONNX Runtime session
    // session: ort::Session,
    // h: Vec<f32>,
    // c: Vec<f32>,
}

impl SileroVAD {
    /// 创建新的 VAD 检测器
    pub fn new(config: SileroVADConfig) -> VInputResult<Self> {
        // 验证模型文件存在
        if !Path::new(&config.model_path).exists() {
            return Err(VInputError::VadModelLoad(format!(
                "Model file not found: {}",
                config.model_path
            )));
        }

        // 验证采样率
        if config.sample_rate != 8000 && config.sample_rate != 16000 {
            return Err(VInputError::VadModelLoad(format!(
                "Invalid sample rate: {}. Must be 8000 or 16000",
                config.sample_rate
            )));
        }

        tracing::info!(
            "创建 Silero VAD: {} @ {} Hz",
            config.model_path,
            config.sample_rate
        );

        Ok(Self {
            config,
            state: VADState::Silence,
            speech_frames: 0,
            silence_frames: 0,
        })
    }

    /// 处理音频帧并返回语音概率
    ///
    /// 输入：
    /// - samples: 音频样本 (f32, [-1.0, 1.0])
    /// - 对于 16kHz: 512 samples (32ms)
    /// - 对于 8kHz: 256 samples (32ms)
    ///
    /// 输出：
    /// - speech_prob: 语音概率 [0.0, 1.0]
    ///
    /// Phase 0 MVP: 返回模拟概率用于接口验证
    /// Phase 1: 完整的 ONNX Runtime 推理
    pub fn process_chunk(&mut self, samples: &[f32]) -> VInputResult<f32> {
        // 验证输入长度
        let expected_len = if self.config.sample_rate == 16000 {
            512
        } else {
            256
        };

        if samples.len() != expected_len {
            return Err(VInputError::AsrInference(format!(
                "Invalid chunk size: {}. Expected {} for {} Hz",
                samples.len(),
                expected_len,
                self.config.sample_rate
            )));
        }

        // Phase 0 MVP: 基于音频能量的简单启发式
        // Phase 1: 替换为实际的 ONNX 推理
        let energy = samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32;
        let speech_prob = (energy * 10.0).min(1.0);

        tracing::trace!("VAD energy: {:.4}, prob: {:.3}", energy, speech_prob);

        Ok(speech_prob)
    }

    /// 检测语音活动（带状态管理）
    ///
    /// 返回当前状态和是否发生状态转换
    pub fn detect(&mut self, samples: &[f32]) -> VInputResult<(VADState, bool)> {
        let prob = self.process_chunk(samples)?;
        let is_speech = prob >= self.config.threshold;

        let old_state = self.state;
        let mut state_changed = false;

        match self.state {
            VADState::Silence => {
                if is_speech {
                    self.speech_frames += 1;
                    self.silence_frames = 0;

                    // 检查是否达到最小语音持续时间
                    let frame_ms = if self.config.sample_rate == 16000 {
                        32
                    } else {
                        32
                    };
                    if self.speech_frames * frame_ms >= self.config.min_speech_duration_ms {
                        self.state = VADState::Speech;
                        state_changed = true;
                        tracing::debug!("VAD: Silence -> Speech (prob={:.3})", prob);
                    }
                } else {
                    self.speech_frames = 0;
                }
            }
            VADState::Speech => {
                if !is_speech {
                    self.silence_frames += 1;
                    self.speech_frames = 0;

                    // 检查是否达到最小静音持续时间
                    let frame_ms = if self.config.sample_rate == 16000 {
                        32
                    } else {
                        32
                    };
                    if self.silence_frames * frame_ms >= self.config.min_silence_duration_ms {
                        self.state = VADState::Silence;
                        state_changed = true;
                        tracing::debug!("VAD: Speech -> Silence (prob={:.3})", prob);
                    }
                } else {
                    self.silence_frames = 0;
                }
            }
        }

        Ok((self.state, state_changed))
    }

    /// 重置 VAD 状态
    pub fn reset(&mut self) {
        self.state = VADState::Silence;
        self.speech_frames = 0;
        self.silence_frames = 0;
        tracing::debug!("VAD reset");
    }

    /// 获取当前状态
    pub fn state(&self) -> VADState {
        self.state
    }

    /// 获取配置
    pub fn config(&self) -> &SileroVADConfig {
        &self.config
    }
}

// Phase 0 说明：
// 此模块提供了 Silero VAD 的接口定义和基本逻辑
//
// 完整实现需要（Phase 1）：
// 1. 集成 ort crate (ONNX Runtime Rust 绑定)
// 2. 加载 silero_vad_v5.onnx 模型
// 3. 管理 LSTM 隐藏状态 (h, c)
// 4. 执行实际的模型推理
// 5. 返回准确的语音概率
//
// 当前 MVP 实现：
// - 基于音频能量的简单启发式
// - 完整的状态机逻辑（可直接用于 Phase 1）
// - 阈值和时间控制已实现
