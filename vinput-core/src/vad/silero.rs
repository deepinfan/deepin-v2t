//! Silero VAD ONNX 推理
//!
//! 基于 Silero VAD v5 模型的语音活动检测
//! Phase 1: 完整的 ONNX Runtime 推理实现
//!
//! 需要启用 `vad-onnx` feature

#![cfg(feature = "vad-onnx")]

use crate::error::{VInputError, VInputResult};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Value;
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

/// Silero VAD 检测器 (Phase 1: 完整 ONNX 推理)
///
/// Silero VAD v6.2 接口（经 ONNX 解析确认）：
/// - Inputs:  input[f32, (batch, seq)], state[f32, (2, batch, 128)], sr[int64, scalar]
/// - Outputs: output[f32, (batch, 1)], stateN[f32, (2, batch, 128)]
pub struct SileroVAD {
    config: SileroVADConfig,
    state: VADState,
    speech_frames: u32,
    silence_frames: u32,
    // ONNX Runtime session
    session: Session,
    /// LSTM state tensor: shape [2, 1, 128] = 256 elements
    /// 对应模型输入/输出名 "state" / "stateN"
    lstm_state: Vec<f32>,
    /// 采样率（int64 scalar），传给模型的 "sr" 输入
    sr: i64,
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

        // 加载 ONNX 模型
        let model_bytes = std::fs::read(&config.model_path)
            .map_err(|e| VInputError::VadModelLoad(format!("Failed to read model file: {}", e)))?;

        let session = Session::builder()
            .map_err(|e| {
                VInputError::VadModelLoad(format!("Failed to create session builder: {}", e))
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| {
                VInputError::VadModelLoad(format!("Failed to set optimization level: {}", e))
            })?
            .with_intra_threads(1)
            .map_err(|e| {
                VInputError::VadModelLoad(format!("Failed to set intra threads: {}", e))
            })?
            .commit_from_memory(&model_bytes)
            .map_err(|e| VInputError::VadModelLoad(format!("Failed to load model: {}", e)))?;

        // 记录模型输入/输出信息，帮助验证输入顺序
        for (i, input) in session.inputs().iter().enumerate() {
            tracing::info!("Silero 模型 input[{}]: name='{}'", i, input.name());
        }
        for (i, output) in session.outputs().iter().enumerate() {
            tracing::info!("Silero 模型 output[{}]: name='{}'", i, output.name());
        }

        // state: [2, 1, 128] = 256 elements（合并的 LSTM 状态）
        let lstm_state = vec![0.0f32; 2 * 1 * 128];
        let sr = config.sample_rate as i64;

        Ok(Self {
            config,
            state: VADState::Silence,
            speech_frames: 0,
            silence_frames: 0,
            session,
            lstm_state,
            sr,
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

        // 准备输入张量
        // 顺序（经 ONNX 解析确认）：input, state, sr
        use ort::inputs;

        // input: [1, T] f32
        let input_tensor = Value::from_array((vec![1usize, samples.len()], samples.to_vec()))
            .map_err(|e| VInputError::VadInference(format!("Failed to create input tensor: {}", e)))?;

        // state: [2, 1, 128] f32
        let state_tensor = Value::from_array((vec![2usize, 1, 128], self.lstm_state.clone()))
            .map_err(|e| VInputError::VadInference(format!("Failed to create state tensor: {}", e)))?;

        // sr: int64 [1]（ort v2 不支持真正的 0 维张量，用 shape [1] 代替）
        let sr_tensor = Value::from_array((vec![1usize], vec![self.sr]))
            .map_err(|e| VInputError::VadInference(format!("Failed to create sr tensor: {}", e)))?;

        // 执行推理（顺序：input, state, sr）
        let outputs = self.session
            .run(inputs![input_tensor, state_tensor, sr_tensor])
            .map_err(|e| VInputError::VadInference(format!("Inference failed: {}", e)))?;

        // 提取输出
        // outputs[0]: "output"  [batch, 1] f32 → speech probability
        // outputs[1]: "stateN"  [2, batch, 128] f32 → updated state

        let speech_prob_tensor = &outputs[0];
        let (_shape, speech_prob_data) = speech_prob_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| VInputError::VadInference(format!("Failed to extract speech prob: {}", e)))?;
        let speech_prob = speech_prob_data[0];

        // 更新 LSTM state
        let new_state_tensor = &outputs[1];
        let (_state_shape, new_state_slice) = new_state_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| VInputError::VadInference(format!("Failed to extract stateN: {}", e)))?;
        self.lstm_state.copy_from_slice(new_state_slice);

        tracing::trace!("VAD inference: prob={:.3}", speech_prob);

        Ok(speech_prob)
    }

    /// 检测语音活动（带状态管理）
    ///
    /// 返回当前状态和是否发生状态转换
    pub fn detect(&mut self, samples: &[f32]) -> VInputResult<(VADState, bool)> {
        let prob = self.process_chunk(samples)?;
        let is_speech = prob >= self.config.threshold;

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
        // 重置 LSTM state
        self.lstm_state.fill(0.0);
        tracing::debug!("VAD reset (including LSTM state)");
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

// Phase 1 实现完成：
// ✅ 集成 ort crate (ONNX Runtime Rust 绑定)
// ✅ 加载 silero_vad_v5.onnx 模型
// ✅ 管理 LSTM 隐藏状态 (h, c)
// ✅ 执行实际的模型推理
// ✅ 返回准确的语音概率
// ✅ 完整的状态机逻辑
//
// 性能目标：< 1ms/帧 (需要基准测试验证)
