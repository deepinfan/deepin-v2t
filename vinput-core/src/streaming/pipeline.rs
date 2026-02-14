//! Streaming Pipeline - VAD-ASR 流式识别管道
//!
//! 将 VAD 检测结果与 ASR 识别器连接，实现端到端的流式语音识别

use crate::asr::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use crate::error::VInputResult;
use crate::vad::{VadConfig, VadManager, VadState};
use std::time::Instant;

/// 流式管道配置
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// VAD 配置
    pub vad_config: VadConfig,
    /// ASR 配置
    pub asr_config: OnlineRecognizerConfig,
    /// 最大静音等待时间 (ms) - 超过此时间后强制结束识别
    pub max_silence_duration_ms: u64,
    /// 启用端点检测
    pub enable_endpoint_detection: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            vad_config: VadConfig::push_to_talk_default(),
            asr_config: OnlineRecognizerConfig::default(),
            max_silence_duration_ms: 3000,
            enable_endpoint_detection: true,
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
    pipeline_state: PipelineState,

    /// 语音开始时间
    speech_start_time: Option<Instant>,
    /// 最后一次语音活动时间
    last_speech_time: Option<Instant>,

    /// 累积的音频帧数（用于调试）
    total_frames: u64,
    /// 送入 ASR 的音频帧数
    asr_frames: u64,
}

impl StreamingPipeline {
    /// 创建新的流式管道
    pub fn new(config: StreamingConfig) -> VInputResult<Self> {
        let vad_manager = VadManager::new(config.vad_config.clone())?;
        let asr_recognizer = OnlineRecognizer::new(&config.asr_config)?;

        Ok(Self {
            config,
            vad_manager,
            asr_recognizer,
            asr_stream: None,
            pipeline_state: PipelineState::Idle,
            speech_start_time: None,
            last_speech_time: None,
            total_frames: 0,
            asr_frames: 0,
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

        // 2. 根据 VAD 状态管理 ASR 流
        match (self.pipeline_state, vad_result.state) {
            // 从空闲状态检测到语音开始
            (PipelineState::Idle, VadState::Speech) if vad_result.state_changed => {
                tracing::info!("Pipeline: Speech detected, starting ASR");

                // 创建新的 ASR 流
                let mut stream = self.asr_recognizer.create_stream()?;

                // 注入 Pre-roll 音频（如果有）
                if let Some(pre_roll_audio) = &vad_result.pre_roll_audio {
                    if !pre_roll_audio.is_empty() {
                        stream.accept_waveform(
                            pre_roll_audio,
                            self.config.vad_config.silero.sample_rate as i32,
                        );
                        self.asr_frames += 1;
                        tracing::debug!(
                            "Pipeline: Injected {} pre-roll samples",
                            pre_roll_audio.len()
                        );
                    }
                }

                // 存储流的生命周期（需要 unsafe transmute 来绕过生命周期检查）
                // 安全性：stream 的生命周期由 self.asr_stream 管理，在 reset 时会被销毁
                let stream_static: OnlineStream<'static> = unsafe {
                    std::mem::transmute(stream)
                };
                self.asr_stream = Some(stream_static);

                self.pipeline_state = PipelineState::Recognizing;
                self.speech_start_time = Some(now);
                self.last_speech_time = Some(now);
            }

            // 识别中，继续送入音频
            (PipelineState::Recognizing, VadState::Speech | VadState::SpeechCandidate) => {
                if self.asr_stream.is_some() {
                    let samples_vec = samples.to_vec();
                    self.feed_audio_to_asr_internal(&samples_vec)?;
                    self.last_speech_time = Some(now);
                }
            }

            // 检测到语音结束
            (PipelineState::Recognizing, VadState::Silence) if vad_result.state_changed => {
                tracing::info!("Pipeline: Speech ended, finalizing ASR");

                if let Some(stream) = &mut self.asr_stream {
                    // 标记输入结束
                    stream.input_finished();

                    // 最后一次解码
                    if stream.is_ready(&self.asr_recognizer) {
                        stream.decode(&self.asr_recognizer);
                    }
                }

                self.pipeline_state = PipelineState::Completed;
            }

            // 识别中，检查静音超时
            (PipelineState::Recognizing, VadState::SilenceCandidate) => {
                if self.asr_stream.is_some() {
                    let samples_vec = samples.to_vec();
                    self.feed_audio_to_asr_internal(&samples_vec)?;
                }

                // 检查是否超过最大静音时间
                if let Some(last_time) = self.last_speech_time {
                    let silence_duration = now.duration_since(last_time);
                    if silence_duration.as_millis() as u64 > self.config.max_silence_duration_ms {
                        tracing::warn!(
                            "Pipeline: Max silence duration exceeded ({:?}), finalizing",
                            silence_duration
                        );

                        if let Some(stream) = &mut self.asr_stream {
                            stream.input_finished();
                        }
                        self.pipeline_state = PipelineState::Completed;
                    }
                }
            }

            _ => {
                // 其他状态组合，不做处理
            }
        }

        // 3. 执行 ASR 解码（如果流准备好）
        if self.pipeline_state == PipelineState::Recognizing {
            if let Some(stream) = &mut self.asr_stream {
                if stream.is_ready(&self.asr_recognizer) {
                    stream.decode(&self.asr_recognizer);
                }

                // 检查端点检测
                if self.config.enable_endpoint_detection && stream.is_endpoint(&self.asr_recognizer) {
                    tracing::info!("Pipeline: Endpoint detected by ASR");
                    stream.input_finished();
                    self.pipeline_state = PipelineState::Completed;
                }
            }
        }

        // 4. 获取识别结果
        let partial_result = if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        };

        let is_final = self.pipeline_state == PipelineState::Completed;

        let duration_ms = self.speech_start_time
            .map(|start| now.duration_since(start).as_millis() as u64)
            .unwrap_or(0);

        Ok(StreamingResult {
            partial_result,
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
        }
        Ok(())
    }

    /// 重置管道状态
    pub fn reset(&mut self) -> VInputResult<()> {
        tracing::debug!("Pipeline: Resetting");

        // 销毁 ASR 流
        if let Some(mut stream) = self.asr_stream.take() {
            stream.reset(&self.asr_recognizer);
        }

        // 重置 VAD
        self.vad_manager.reset();

        // 重置状态
        self.pipeline_state = PipelineState::Idle;
        self.speech_start_time = None;
        self.last_speech_time = None;

        Ok(())
    }

    /// 强制设置 VAD 状态（用于 PushToTalk 模式）
    pub fn force_vad_state(&mut self, state: VadState) {
        self.vad_manager.force_state(state);
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

    /// 获取最终识别结果
    ///
    /// 调用此方法后会自动重置管道状态
    pub fn get_final_result(&mut self) -> String {
        let result = if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        };

        // 重置管道以准备下一次识别
        let _ = self.reset();

        result
    }
}

impl Drop for StreamingPipeline {
    fn drop(&mut self) {
        // 确保 ASR 流在管道销毁前被清理
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
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.max_silence_duration_ms, 3000);
        assert!(config.enable_endpoint_detection);
    }

    #[test]
    fn test_pipeline_state_transitions() {
        assert_eq!(PipelineState::Idle, PipelineState::Idle);
        assert_ne!(PipelineState::Idle, PipelineState::Recognizing);
    }
}
