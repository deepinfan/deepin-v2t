//! Streaming Pipeline - VAD-ASR 流式识别管道
//!
//! 将 VAD 检测结果与 ASR 识别器连接，实现端到端的流式语音识别

use crate::asr::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use crate::endpointing::{EndpointDetector, EndpointDetectorConfig, EndpointResult};
use crate::error::VInputResult;
use crate::vad::{VadConfig, VadManager, VadState};
use std::time::Instant;

#[cfg(feature = "vad-onnx")]
use crate::punctuation::CtTransformerPunct;

/// 流式管道配置
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// VAD 配置
    pub vad_config: VadConfig,
    /// ASR 配置
    pub asr_config: OnlineRecognizerConfig,
    /// 标点模型目录（CT-Transformer ONNX）
    pub punct_model_dir: String,
    /// 端点检测配置
    pub endpoint_config: EndpointDetectorConfig,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            vad_config: VadConfig::push_to_talk_default(),
            asr_config: OnlineRecognizerConfig::default(),
            punct_model_dir: crate::config::resolve_punct_model_dir(),
            endpoint_config: EndpointDetectorConfig::default(),
        }
    }
}

/// 管道状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    /// 空闲状态，等待语音输入
    Idle,
    /// 检测到语音，正在识别
    Recognizing,
    /// 识别完成，等待重置
    Completed,
}

/// 流式识别结果
#[derive(Debug, Clone)]
pub struct StreamingResult {
    /// 当前识别的部分结果（实时更新）
    pub partial_result: String,
    /// 稳定的文本（可以立即上屏）
    pub stable_text: String,
    /// 不稳定的文本（保留在 Preedit）
    pub unstable_text: String,
    /// 是否应该添加逗号（停顿检测，保留字段兼容性）
    pub should_add_comma: bool,
    /// 是否为最终结果
    pub is_final: bool,
    /// VAD 状态
    pub vad_state: VadState,
    /// 管道状态
    pub pipeline_state: PipelineState,
    /// 语音概率
    pub speech_prob: f32,
    /// 自上次语音开始以来的持续时间 (ms)
    pub duration_ms: u64,
}

/// VAD-ASR 流式识别管道
pub struct StreamingPipeline {
    config: StreamingConfig,
    vad_manager: VadManager,
    asr_recognizer: OnlineRecognizer,
    asr_stream: Option<OnlineStream<'static>>,
    endpoint_detector: EndpointDetector,
    pipeline_state: PipelineState,

    /// 语音开始时间
    speech_start_time: Option<Instant>,

    /// ASR endpoint 检测到后的缓冲帧数（还剩多少帧才真正提交）
    /// 0 表示没有待提交的 endpoint，> 0 表示仍在缓冲期（继续喂音频）
    asr_endpoint_grace_remaining: u32,

    /// 累积的音频帧数（用于调试）
    total_frames: u64,
    /// 送入 ASR 的音频帧数
    asr_frames: u64,

    /// CT-Transformer 标点模型（需要 vad-onnx feature）
    #[cfg(feature = "vad-onnx")]
    punct_model: Option<CtTransformerPunct>,
}

impl StreamingPipeline {
    /// 创建新的流式管道
    pub fn new(config: StreamingConfig) -> VInputResult<Self> {
        tracing::info!("🎯 端点检测配置: trailing_silence={}ms, min_speech={}ms",
            config.endpoint_config.trailing_silence_ms,
            config.endpoint_config.min_speech_duration_ms
        );

        let vad_manager = VadManager::new(config.vad_config.clone())?;
        let asr_recognizer = OnlineRecognizer::new(&config.asr_config)?;
        let endpoint_detector = EndpointDetector::new(config.endpoint_config.clone());

        // 加载 CT-Transformer 标点模型（仅在 vad-onnx feature 启用时）
        #[cfg(feature = "vad-onnx")]
        let punct_model = {
            let model_dir = std::path::Path::new(&config.punct_model_dir);
            match CtTransformerPunct::new(model_dir) {
                Ok(model) => {
                    tracing::info!("✅ CT-Transformer 标点模型加载成功: {}", config.punct_model_dir);
                    Some(model)
                }
                Err(e) => {
                    tracing::warn!("⚠️  CT-Transformer 标点模型加载失败（将不添加标点）: {}", e);
                    None
                }
            }
        };

        Ok(Self {
            config,
            vad_manager,
            asr_recognizer,
            endpoint_detector,
            asr_stream: None,
            pipeline_state: PipelineState::Idle,
            speech_start_time: None,
            asr_endpoint_grace_remaining: 0,
            total_frames: 0,
            asr_frames: 0,
            #[cfg(feature = "vad-onnx")]
            punct_model,
        })
    }

    /// 处理音频帧
    ///
    /// # 参数
    /// - `samples`: 音频样本 (f32, [-1.0, 1.0])
    ///   - 对于 16kHz: 512 samples (32ms)
    ///
    /// # 返回
    /// - `StreamingResult`: 流式识别结果
    pub fn process(&mut self, samples: &[f32]) -> VInputResult<StreamingResult> {
        self.total_frames += 1;

        // 1. VAD 处理
        let vad_result = self.vad_manager.process(samples)?;
        let now = Instant::now();

        // 1.5 将音频送入端点检测器（用于能量分析）
        if self.pipeline_state == PipelineState::Recognizing {
            self.endpoint_detector.feed_audio(samples);
        }

        // 2. 端点检测处理（使用 EndpointDetector）
        let is_speech = matches!(vad_result.state, VadState::Speech | VadState::SpeechCandidate);
        let endpoint_result = self.endpoint_detector.process_vad(is_speech);

        // 3. 根据端点检测结果处理状态
        match endpoint_result {
            EndpointResult::TooShort => {
                // 语音过短，忽略并重置
                tracing::info!("Pipeline: 语音过短，忽略");
                self.reset()?;
                self.pipeline_state = PipelineState::Idle;
            }
            EndpointResult::ForcedSegmentation => {
                // 语音过长，强制分段
                tracing::info!("Pipeline: 语音过长，强制分段");
                if let Some(stream) = &mut self.asr_stream {
                    stream.input_finished();
                }
                self.pipeline_state = PipelineState::Completed;
            }
            EndpointResult::Timeout => {
                // 强制超时
                tracing::warn!("Pipeline: 强制超时");
                if let Some(stream) = &mut self.asr_stream {
                    stream.input_finished();
                }
                self.pipeline_state = PipelineState::Completed;
            }
            EndpointResult::Detected => {
                // 检测到端点
                tracing::info!("Pipeline: VAD 端点检测完成");
                if let Some(stream) = &mut self.asr_stream {
                    stream.input_finished();
                }
                self.pipeline_state = PipelineState::Completed;
            }
            EndpointResult::Continue => {
                // 继续处理，根据 VAD 状态管理 ASR 流
                match (self.pipeline_state, vad_result.state) {
                    // 从空闲状态检测到语音开始
                    (PipelineState::Idle, VadState::Speech) if vad_result.state_changed => {
                        tracing::info!("Pipeline: Speech detected, starting ASR");

                        // 创建新的 ASR 流
                        let mut stream = self.asr_recognizer.create_stream()?;
                        tracing::info!("✅ ASR 流创建成功");

                        // 注入 Pre-roll 音频（如果有）
                        if let Some(pre_roll_audio) = &vad_result.pre_roll_audio {
                            if !pre_roll_audio.is_empty() {
                                stream.accept_waveform(
                                    pre_roll_audio,
                                    self.config.vad_config.silero.sample_rate as i32,
                                );
                                // 按实际帧数计数（512 samples/帧），而非固定 +1
                                self.asr_frames += (pre_roll_audio.len() as u64 + 511) / 512;
                                tracing::info!(
                                    "✅ 注入 Pre-roll 音频: {} 样本 ({} 帧)",
                                    pre_roll_audio.len(),
                                    (pre_roll_audio.len() as u64 + 511) / 512,
                                );
                            }
                        }

                        let stream_static: OnlineStream<'static> = unsafe {
                            std::mem::transmute(stream)
                        };
                        self.asr_stream = Some(stream_static);

                        self.pipeline_state = PipelineState::Recognizing;
                        self.speech_start_time = Some(now);
                    }

                    // 识别中，继续送入音频
                    (PipelineState::Recognizing, VadState::Speech | VadState::SpeechCandidate | VadState::SilenceCandidate) => {
                        if self.asr_stream.is_some() {
                            let samples_vec = samples.to_vec();
                            self.feed_audio_to_asr_internal(&samples_vec)?;
                        }
                    }

                    _ => {
                        // 其他状态组合，不做处理
                    }
                }
            }
        }

        // 4. 执行 ASR 解码（如果流准备好）并检查 ASR 端点
        if self.pipeline_state == PipelineState::Recognizing {
            if let Some(stream) = &mut self.asr_stream {
                if stream.is_ready(&self.asr_recognizer) {
                    stream.decode(&self.asr_recognizer);
                }

                if self.asr_endpoint_grace_remaining > 0 {
                    // 处于 ASR endpoint 缓冲期：继续喂音频，倒计时
                    self.asr_endpoint_grace_remaining -= 1;
                    tracing::debug!(
                        "Pipeline: ASR 端点缓冲期剩余 {} 帧",
                        self.asr_endpoint_grace_remaining,
                    );
                    if self.asr_endpoint_grace_remaining == 0 {
                        // 缓冲期结束：刷新并提交
                        stream.input_finished();
                        self.pipeline_state = PipelineState::Completed;
                        tracing::info!("Pipeline: ASR 端点缓冲期结束，准备上屏");
                    }
                } else {
                    // 正常检查 ASR 端点（只在缓冲期外检查，避免重复触发）
                    let asr_endpoint = stream.is_endpoint(&self.asr_recognizer);
                    let asr_result = self.endpoint_detector.process_asr_endpoint(asr_endpoint);

                    if asr_result == EndpointResult::Detected {
                        // 启动 5 帧（约 160ms）缓冲期，让 Paraformer 完成末字解码
                        const GRACE_FRAMES: u32 = 5;
                        tracing::info!("Pipeline: ASR 端点检测完成，等待 {}ms 缓冲期以确保末字完整",
                            GRACE_FRAMES * 32);
                        self.asr_endpoint_grace_remaining = GRACE_FRAMES;
                    }
                }
            }
        }

        // 5. 获取识别结果
        let partial_result = if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        };

        let is_final = self.pipeline_state == PipelineState::Completed;

        let duration_ms = self.speech_start_time
            .map(|start| now.duration_since(start).as_millis() as u64)
            .unwrap_or(0);

        // 6. 分离稳定和不稳定文本
        let (stable_text, unstable_text) = self.split_stable_unstable(&partial_result);

        Ok(StreamingResult {
            partial_result,
            stable_text,
            unstable_text,
            should_add_comma: false,
            is_final,
            vad_state: vad_result.state,
            pipeline_state: self.pipeline_state,
            speech_prob: vad_result.speech_prob,
            duration_ms,
        })
    }

    /// 将音频数据送入 ASR（内部方法，避免借用冲突）
    fn feed_audio_to_asr_internal(&mut self, samples: &[f32]) -> VInputResult<()> {
        if let Some(stream) = &mut self.asr_stream {
            stream.accept_waveform(
                samples,
                self.config.vad_config.silero.sample_rate as i32,
            );
            self.asr_frames += 1;

            // 每 50 帧（约 1.6 秒）打印一次日志
            if self.asr_frames % 50 == 0 {
                tracing::debug!("🎤 已送入 {} 帧音频到 ASR (每帧 {} 样本)",
                    self.asr_frames, samples.len());
            }
        }
        Ok(())
    }

    /// 通知管道新录音会话即将开始
    ///
    /// 与句子间 reset() 的区别：此方法进行完整重置（包括 Silero LSTM 状态清零），
    /// 防止上次录音会话结束后 LSTM 冻结在静音模式，导致本次录音开始时语音被误判。
    pub fn on_recording_started(&mut self) -> VInputResult<()> {
        tracing::info!("Pipeline: 新录音会话开始，完整重置（含 LSTM）");

        // 销毁 ASR 流
        if let Some(mut stream) = self.asr_stream.take() {
            stream.reset(&self.asr_recognizer);
        }

        // 完整重置 VAD（包括 LSTM 状态清零）
        #[cfg(feature = "vad-onnx")]
        self.vad_manager.full_reset();
        #[cfg(not(feature = "vad-onnx"))]
        self.vad_manager.reset();

        // 重置端点检测器
        self.endpoint_detector.reset();

        // 重置状态
        self.pipeline_state = PipelineState::Idle;
        self.speech_start_time = None;
        self.asr_endpoint_grace_remaining = 0;
        self.asr_frames = 0;

        Ok(())
    }

    /// 重置管道状态（句子提交后调用，保留 Silero LSTM 上下文）
    pub fn reset(&mut self) -> VInputResult<()> {
        tracing::debug!("Pipeline: Resetting");

        // 销毁 ASR 流
        if let Some(mut stream) = self.asr_stream.take() {
            stream.reset(&self.asr_recognizer);
        }

        // 重置 VAD
        self.vad_manager.reset();

        // 重置端点检测器
        self.endpoint_detector.reset();

        // 重置状态
        self.pipeline_state = PipelineState::Idle;
        self.speech_start_time = None;
        self.asr_endpoint_grace_remaining = 0;
        // asr_frames 必须归零：ASR token 时间戳从每条新流的 0ms 开始
        self.asr_frames = 0;

        Ok(())
    }

    /// 强制设置 VAD 状态（用于 PushToTalk 模式）
    ///
    /// 当强制进入 Speech 状态时，立即启动 ASR 流，避免等待 Silero LSTM 预热
    /// （Silero v6.2 需要约 20 帧 / 640ms 才能输出高置信度语音概率）。
    /// 若不立即启动，句子开头的音频会在 Silero 预热期间被丢弃。
    pub fn force_vad_state(&mut self, state: VadState) {
        self.vad_manager.force_state(state);

        // PushToTalk: 强制进入语音状态时，立即启动 ASR 流
        if matches!(state, VadState::Speech) && self.pipeline_state == PipelineState::Idle {
            match self.asr_recognizer.create_stream() {
                Ok(stream) => {
                    let stream_static: OnlineStream<'static> =
                        unsafe { std::mem::transmute(stream) };
                    self.asr_stream = Some(stream_static);
                    self.pipeline_state = PipelineState::Recognizing;
                    self.speech_start_time = Some(Instant::now());
                    tracing::info!("PushToTalk: 立即启动 ASR 流（跳过 Silero ~20 帧预热延迟）");
                }
                Err(e) => {
                    tracing::error!("PushToTalk: 创建 ASR 流失败: {}", e);
                }
            }
        }
    }

    /// 获取当前管道状态
    pub fn pipeline_state(&self) -> PipelineState {
        self.pipeline_state
    }

    /// 获取 VAD 状态
    pub fn vad_state(&self) -> VadState {
        self.vad_manager.state()
    }

    /// 获取统计信息（用于调试）
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            total_frames: self.total_frames,
            asr_frames: self.asr_frames,
            speech_duration_ms: self.speech_start_time
                .map(|start| Instant::now().duration_since(start).as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// 分离稳定和不稳定文本
    ///
    /// 保留最后 N 个字符在 Preedit（不稳定），其余部分可以立即上屏（稳定）
    fn split_stable_unstable(&self, text: &str) -> (String, String) {
        // 如果整个文本包含中文数字，全部保留在 Preedit（等待 ITN 处理）
        if Self::contains_chinese_number(text) {
            return (String::new(), text.to_string());
        }

        const KEEP_LAST_CHARS: usize = 2;

        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= KEEP_LAST_CHARS {
            return (String::new(), text.to_string());
        }

        let stable_count = chars.len() - KEEP_LAST_CHARS;
        let stable: String = chars[..stable_count].iter().collect();
        let unstable: String = chars[stable_count..].iter().collect();

        (stable, unstable)
    }

    /// 检查文本是否包含中文数字字符
    fn contains_chinese_number(text: &str) -> bool {
        text.chars().any(|c| matches!(c,
            '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
            '十' | '百' | '千' | '万' | '亿' | '点'
        ))
    }

    /// 获取实时识别结果（Preedit 展示用）
    ///
    /// CT-Transformer 不适合对流式部分结果做推理（缺乏完整上下文），
    /// 因此直接返回 ASR 纯文本，标点在最终结果时统一处理。
    pub fn get_partial_result_with_punctuation(&mut self) -> String {
        if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        }
    }

    /// 获取最终识别结果（带标点）
    ///
    /// 使用 CT-Transformer ONNX 模型为 ASR 输出添加标点。
    /// 调用此方法后会自动重置管道状态。
    pub fn get_final_result_with_punctuation(&mut self) -> String {
        // 通知解码器输入已结束，触发最终 beam search 完成
        if let Some(stream) = &mut self.asr_stream {
            tracing::info!("🔚 调用 input_finished()，刷新 ASR 解码器缓冲区");
            stream.input_finished();
        }

        // 最终一次解码
        if let Some(stream) = &mut self.asr_stream {
            if stream.is_ready(&self.asr_recognizer) {
                stream.decode(&self.asr_recognizer);
                tracing::info!("🔚 最终解码完成");
            }
        }

        // 收集纯文本（在独立作用域内完成，释放 asr_stream 借用）
        let plain_text = if let Some(stream) = &self.asr_stream {
            let detailed_result = stream.get_detailed_result(&self.asr_recognizer);

            tracing::info!("📊 ASR 识别结果: text='{}', tokens={}",
                detailed_result.text, detailed_result.tokens.len());

            if detailed_result.is_empty() {
                tracing::warn!("⚠️  识别结果为空");
                String::new()
            } else {
                for (i, token) in detailed_result.tokens.iter().enumerate() {
                    tracing::info!("  Token[{}]: '{}' ({}ms - {}ms)",
                        i, token.text, token.start_time_ms, token.end_time_ms);
                }

                let mut text = String::new();
                for token in &detailed_result.tokens {
                    let word = token.text.trim();
                    if !word.is_empty() && word != "NE" {
                        text.push_str(word);
                    }
                }
                tracing::info!("📝 纯文本: '{}'", text);
                text
            }
        } else {
            tracing::warn!("⚠️  ASR 流为空");
            String::new()
        };

        // asr_stream 借用已释放，现在可以同时访问 punct_model
        let result = if plain_text.is_empty() {
            String::new()
        } else {
            #[cfg(feature = "vad-onnx")]
            {
                if let Some(model) = &mut self.punct_model {
                    let punctuated = model.add_punctuation(&plain_text);
                    tracing::info!("✅ CT-Transformer 标点结果: '{}'", punctuated);
                    punctuated
                } else {
                    tracing::info!("ℹ️  无标点模型，返回纯文本");
                    plain_text
                }
            }
            #[cfg(not(feature = "vad-onnx"))]
            {
                tracing::info!("ℹ️  vad-onnx feature 未启用，返回纯文本");
                plain_text
            }
        };

        // 重置管道以准备下一次识别
        let _ = self.reset();

        result
    }

    /// 获取最终识别结果（不带标点，原始文本）
    ///
    /// 调用此方法后会自动重置管道状态
    pub fn get_final_result(&mut self) -> String {
        let result = if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        };

        let _ = self.reset();

        result
    }
}

impl Drop for StreamingPipeline {
    fn drop(&mut self) {
        if let Some(mut stream) = self.asr_stream.take() {
            stream.reset(&self.asr_recognizer);
        }
    }
}

/// 管道统计信息
#[derive(Debug, Clone)]
pub struct PipelineStats {
    /// 处理的总帧数
    pub total_frames: u64,
    /// 送入 ASR 的帧数
    pub asr_frames: u64,
    /// 语音持续时间 (ms)
    pub speech_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_state_transitions() {
        assert_eq!(PipelineState::Idle, PipelineState::Idle);
        assert_ne!(PipelineState::Idle, PipelineState::Recognizing);
    }
}
