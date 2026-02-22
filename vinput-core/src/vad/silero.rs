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
            model_path: "models/silero-vad/silero_vad.onnx".to_string(),
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
    /// - 对于 16kHz: 256 samples (16ms) 或 512 samples (32ms，内部拆分为 2×256)
    /// - 对于 8kHz: 128 samples (16ms) 或 256 samples (32ms，内部拆分为 2×128)
    ///
    /// 输出：
    /// - speech_prob: 语音概率 [0.0, 1.0]
    ///
    /// 注意：Silero VAD v6.2 ONNX 模型内部使用 STFT 窗口=256，因此每次推理
    /// 需要恰好 256 个样本（16kHz）。接受 512 样本时，内部拆分为 2×256 并取最大概率。
    pub fn process_chunk(&mut self, samples: &[f32]) -> VInputResult<f32> {
        // Silero v6.2 ONNX 模型需要 256 样本/推理（STFT window=256）
        // 接受 256 或 512 样本，512 时内部拆分
        let sub_chunk_size = if self.config.sample_rate == 16000 {
            256
        } else {
            128
        };

        if samples.len() == sub_chunk_size {
            // 直接单次推理
            self.run_inference_sub_chunk(samples)
        } else if samples.len() == sub_chunk_size * 2 {
            // 拆分为 2 个子块，取最大语音概率
            let p1 = self.run_inference_sub_chunk(&samples[..sub_chunk_size])?;
            let p2 = self.run_inference_sub_chunk(&samples[sub_chunk_size..])?;
            tracing::trace!("VAD split inference: p1={:.3}, p2={:.3}", p1, p2);
            Ok(p1.max(p2))
        } else {
            Err(VInputError::AsrInference(format!(
                "Invalid chunk size: {}. Expected {} or {} for {} Hz",
                samples.len(),
                sub_chunk_size,
                sub_chunk_size * 2,
                self.config.sample_rate
            )))
        }
    }

    /// 对 256 样本（16kHz）执行单次 ONNX 推理
    ///
    /// Silero v6.2 内部以 256 样本为单位处理：
    /// - STFT window=256 → T=1 帧
    /// - CNN encoder 输出 [1, 128, 1]（3D）→ LSTM 接受正确形状
    /// 若传入 512 样本，STFT 产生 T=2 帧，CNN 输出变为 4D 导致 prob≈0.001
    fn run_inference_sub_chunk(&mut self, samples: &[f32]) -> VInputResult<f32> {
        use ort::inputs;

        // input: [1, T] f32，T=256 for 16kHz
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

        // outputs[0]: "output"  [batch, 1] f32 → speech probability
        // outputs[1]: "stateN"  [2, batch, 128] f32 → updated LSTM state
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
// ✅ 加载 silero_vad.onnx 模型 (v6.2)
// ✅ 管理 LSTM 隐藏状态 (h, c)
// ✅ 执行实际的模型推理
// ✅ 返回准确的语音概率
// ✅ 完整的状态机逻辑
//
// 性能目标：< 1ms/帧 (需要基准测试验证)
