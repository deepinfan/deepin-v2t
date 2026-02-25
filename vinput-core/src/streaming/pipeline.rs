//! VAD-ASR 流式识别管道（简化重构版）
//!
//! 流程：音频帧 → 能量门 → Silero VAD → ASR → 标点 → ITN → 上屏

use crate::asr::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use crate::error::VInputResult;
use crate::vad::{VadConfig, VadManager};

#[cfg(feature = "vad-onnx")]
use crate::punctuation::CtTransformerPunct;

/// 流式管道配置
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub vad_config: VadConfig,
    pub asr_config: OnlineRecognizerConfig,
    pub punct_model_dir: String,
    /// 尾部静音帧数阈值（每帧32ms），达到后触发端点
    pub trailing_silence_frames: u32,
    /// 最大连续语音帧数（超过后强制断句），0 表示不限制
    pub max_speech_frames: u32,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            vad_config: VadConfig::push_to_talk_default(),
            asr_config: OnlineRecognizerConfig::default(),
            punct_model_dir: crate::config::resolve_punct_model_dir(),
            trailing_silence_frames: 47, // ~1500ms
            max_speech_frames: 625,      // ~20s (20000ms / 32ms)
        }
    }
}

/// 管道状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Idle,
    Recognizing,
    Completed,
}

/// 流式识别结果
#[derive(Debug, Clone)]
pub struct StreamingResult {
    pub partial_result: String,
    pub stable_text: String,
    pub unstable_text: String,
    pub should_add_comma: bool,
    pub is_final: bool,
    pub vad_state: VadState,
    pub pipeline_state: PipelineState,
    pub speech_prob: f32,
    pub duration_ms: u64,
}

/// 简化的 VAD 状态（对外暴露）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadState {
    Silence,
    Speech,
}

/// VAD-ASR 流式识别管道
pub struct StreamingPipeline {
    config: StreamingConfig,
    vad_manager: VadManager,
    asr_recognizer: OnlineRecognizer,
    asr_stream: Option<OnlineStream<'static>>,
    /// 已完成的 ASR 流（等待上层获取结果）
    completed_stream: Option<OnlineStream<'static>>,
    pipeline_state: PipelineState,

    /// 语音帧计数（用于最小语音长度过滤）
    speech_frames: u32,
    /// 连续静音帧计数（用于端点检测）
    silence_frames: u32,
    /// 总帧计数（用于 duration_ms 估算）
    speech_start_frame: u64,
    total_frames: u64,
    /// prob 滑动窗口（8帧=256ms，用于端点检测去抖）
    prob_window: [f32; 8],
    prob_window_pos: usize,
    /// 说话期间 prob_max 峰值（用于动态端点阈值）
    speech_prob_peak: f32,
    /// RMS 滑动窗口（16帧=512ms，用于能量端点检测）
    rms_window: [f32; 16],
    rms_window_pos: usize,
    /// 说话期间 rms_max 峰值（用于动态 RMS 阈值）
    speech_rms_peak: f32,

    /// CT-Transformer 标点模型
    #[cfg(feature = "vad-onnx")]
    punct_model: Option<CtTransformerPunct>,
}

impl StreamingPipeline {
    pub fn new(config: StreamingConfig) -> VInputResult<Self> {
        let vad_manager = VadManager::new(config.vad_config.clone())?;
        let asr_recognizer = OnlineRecognizer::new(&config.asr_config)?;

        #[cfg(feature = "vad-onnx")]
        let punct_model = {
            let model_dir = std::path::Path::new(&config.punct_model_dir);
            match CtTransformerPunct::new(model_dir) {
                Ok(model) => {
                    tracing::info!("CT-Transformer 标点模型加载成功: {}", config.punct_model_dir);
                    Some(model)
                }
                Err(e) => {
                    tracing::warn!("CT-Transformer 标点模型加载失败（将不添加标点）: {}", e);
                    None
                }
            }
        };

        Ok(Self {
            config,
            vad_manager,
            asr_recognizer,
            asr_stream: None,
            completed_stream: None,
            pipeline_state: PipelineState::Idle,
            speech_frames: 0,
            silence_frames: 0,
            speech_start_frame: 0,
            total_frames: 0,
            prob_window: [0.0; 8],
            prob_window_pos: 0,
            speech_prob_peak: 0.0,
            rms_window: [0.0; 16],
            rms_window_pos: 0,
            speech_rms_peak: 0.0,
            #[cfg(feature = "vad-onnx")]
            punct_model,
        })
    }

    /// 处理一帧音频（512 samples = 32ms @ 16kHz）
    pub fn process(&mut self, samples: &[f32]) -> VInputResult<StreamingResult> {
        self.total_frames += 1;

        // 1. VAD 处理
        let vad_result = self.vad_manager.process(samples)?;

        // prob 滑动窗口最大值（8帧=256ms）
        self.prob_window[self.prob_window_pos] = vad_result.speech_prob;
        self.prob_window_pos = (self.prob_window_pos + 1) % 8;
        let prob_max = self.prob_window.iter().cloned().fold(0.0f32, f32::max);

        // RMS 滑动窗口最大值（16帧=512ms）
        let rms = {
            let s: f32 = samples.iter().map(|&x| x * x).sum();
            (s / samples.len() as f32).sqrt()
        };
        self.rms_window[self.rms_window_pos] = rms;
        self.rms_window_pos = (self.rms_window_pos + 1) % 16;
        let rms_max = self.rms_window.iter().cloned().fold(0.0f32, f32::max);

        let base_threshold = self.config.vad_config.hysteresis.end_threshold;
        // 动态 prob 阈值：说话峰值的 5%
        let end_threshold = (self.speech_prob_peak * 0.05).max(base_threshold);
        // 动态 RMS 阈值：说话 RMS 峰值的 10%（适应不同麦克风音量）
        let rms_threshold = (self.speech_rms_peak * 0.10).max(0.003);
        // 端点判断：prob 窗口 OR RMS 窗口任一高于阈值就算语音
        let frame_is_speech = prob_max >= end_threshold || rms_max >= rms_threshold;

        // 2. 状态机
        match self.pipeline_state {
            PipelineState::Idle => {
                // VAD 确认语音（pre_roll 有值 = 刚从静音转为语音）→ 立即启动 ASR
                if vad_result.is_speech && vad_result.pre_roll.is_some() {
                    self.start_asr(vad_result.pre_roll.as_deref())?;
                }
            }

            PipelineState::Recognizing => {
                // 所有帧都送入 ASR（静音也送，避免截断）
                if let Some(stream) = &mut self.asr_stream {
                    stream.accept_waveform(samples, self.config.vad_config.silero.sample_rate as i32);
                }

                // 更新说话期间峰值（用于动态端点阈值）
                if prob_max > self.speech_prob_peak {
                    self.speech_prob_peak = prob_max;
                }
                if rms_max > self.speech_rms_peak {
                    self.speech_rms_peak = rms_max;
                }

                if frame_is_speech {
                    self.silence_frames = 0;
                    self.speech_frames += 1;
                    // 检查是否超过最大语音长度（连续说话强制断句）
                    if self.config.max_speech_frames > 0 && self.speech_frames >= self.config.max_speech_frames {
                        tracing::info!("达到最大语音长度 {}ms，强制断句", self.speech_frames * 32);
                        self.finalize_asr();
                    }
                } else {
                    self.silence_frames += 1;
                    // 达到尾部静音阈值 → 触发端点
                    if self.silence_frames >= self.config.trailing_silence_frames {
                        self.finalize_asr();
                    }
                }
                // 每帧记录 debug 日志（方便定位端点问题）
                tracing::debug!(
                    "EP prob={:.3} max={:.3}/{:.3}(dyn) rms={:.4} rms_max={:.4}/{:.4}(dyn) speech={}/{} sil={}/{}",
                    vad_result.speech_prob, prob_max, end_threshold, rms, rms_max, rms_threshold,
                    self.speech_frames, self.config.max_speech_frames, self.silence_frames, self.config.trailing_silence_frames
                );
                if self.silence_frames > 0 && (self.silence_frames <= 5 || self.silence_frames % 5 == 0) {
                    tracing::info!(
                        "EP silence prob={:.3} max={:.3}/{:.3}(dyn) rms={:.4} rms_max={:.4}/{:.4}(dyn) sil={}/{}",
                        vad_result.speech_prob, prob_max, end_threshold, rms, rms_max, rms_threshold,
                        self.silence_frames, self.config.trailing_silence_frames
                    );
                }
                // 每 100 帧（约 3.2 秒）输出一次语音帧计数
                if self.speech_frames > 0 && self.speech_frames % 100 == 0 {
                    tracing::info!(
                        "EP 语音持续中: speech={}/{} ({}s/{}s)",
                        self.speech_frames, self.config.max_speech_frames,
                        self.speech_frames * 32 / 1000, self.config.max_speech_frames * 32 / 1000
                    );
                }

                // 实时解码
                if self.pipeline_state == PipelineState::Recognizing {
                    if let Some(stream) = &mut self.asr_stream {
                        if stream.is_ready(&self.asr_recognizer) {
                            stream.decode(&self.asr_recognizer);
                        }
                    }
                }
            }

            PipelineState::Completed => {
                // 继续运行 VAD，检测到语音立即启动新的 ASR 流
                // completed_stream 保存了旧流的结果，不会丢失
                if vad_result.is_speech {
                    tracing::info!("Completed 状态检测到语音，立即启动新的 ASR 流");
                    // 重置状态（但不 full_reset VAD，保持 LSTM 连续性）
                    self.reset_state();
                    // 启动新的 ASR 流（可能没有 pre_roll，但没关系）
                    self.start_asr(vad_result.pre_roll.as_deref())?;
                }
            }
        }

        // 3. 获取部分结果
        let partial_result = if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        };

        let duration_ms = if self.speech_start_frame > 0 {
            (self.total_frames - self.speech_start_frame) * 32
        } else {
            0
        };

        let (stable_text, unstable_text) = split_stable_unstable(&partial_result);

        Ok(StreamingResult {
            partial_result,
            stable_text,
            unstable_text,
            should_add_comma: false,
            is_final: self.pipeline_state == PipelineState::Completed,
            vad_state: if frame_is_speech { VadState::Speech } else { VadState::Silence },
            pipeline_state: self.pipeline_state,
            speech_prob: vad_result.speech_prob,
            duration_ms,
        })
    }

    /// 启动 ASR 流并注入 pre-roll
    fn start_asr(&mut self, pre_roll: Option<&[f32]>) -> VInputResult<()> {
        let mut stream = self.asr_recognizer.create_stream()?;

        if let Some(audio) = pre_roll {
            if !audio.is_empty() {
                stream.accept_waveform(audio, self.config.vad_config.silero.sample_rate as i32);
                tracing::info!("ASR 启动，注入 pre-roll: {} 样本", audio.len());
            }
        }

        let stream_static: OnlineStream<'static> = unsafe { std::mem::transmute(stream) };
        self.asr_stream = Some(stream_static);
        self.pipeline_state = PipelineState::Recognizing;
        self.speech_start_frame = self.total_frames;
        tracing::info!("Pipeline: Speech detected, ASR started");
        Ok(())
    }

    /// 触发端点：drain decode → Completed，将流移到 completed_stream
    fn finalize_asr(&mut self) {
        if let Some(mut stream) = self.asr_stream.take() {
            stream.input_finished();
            while stream.is_ready(&self.asr_recognizer) {
                stream.decode(&self.asr_recognizer);
            }
            // 将完成的流保存到 completed_stream，等待上层获取结果
            self.completed_stream = Some(stream);
        }
        self.pipeline_state = PipelineState::Completed;
        tracing::info!("Pipeline: 端点检测完成，静音帧={}", self.silence_frames);
    }

    /// 新录音会话开始（完整重置含 LSTM）
    pub fn on_recording_started(&mut self) -> VInputResult<()> {
        self.drop_asr_stream();
        #[cfg(feature = "vad-onnx")]
        self.vad_manager.full_reset();
        #[cfg(not(feature = "vad-onnx"))]
        self.vad_manager.reset();
        self.reset_state();
        tracing::info!("Pipeline: 新录音会话开始，完整重置（含 LSTM）");
        Ok(())
    }

    /// 更新配置（热重载）
    pub fn update_config(&mut self, config: StreamingConfig) {
        self.config = config;
        tracing::info!("Pipeline 配置已更新");
    }

    /// 句间重置（full_reset LSTM）
    pub fn reset(&mut self) -> VInputResult<()> {
        self.drop_asr_stream();
        #[cfg(feature = "vad-onnx")]
        self.vad_manager.full_reset();
        #[cfg(not(feature = "vad-onnx"))]
        self.vad_manager.reset();
        self.reset_state();
        tracing::debug!("Pipeline: reset");
        Ok(())
    }

    fn drop_asr_stream(&mut self) {
        if let Some(mut stream) = self.asr_stream.take() {
            stream.reset(&self.asr_recognizer);
        }
    }

    fn reset_state(&mut self) {
        self.pipeline_state = PipelineState::Idle;
        self.speech_frames = 0;
        self.silence_frames = 0;
        self.speech_start_frame = 0;
        self.prob_window = [0.0; 8];
        self.prob_window_pos = 0;
        self.speech_prob_peak = 0.0;
        self.rms_window = [0.0; 16];
        self.rms_window_pos = 0;
        self.speech_rms_peak = 0.0;
    }

    /// 获取实时部分结果（Preedit 用）
    pub fn get_partial_result_with_punctuation(&mut self) -> String {
        if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        }
    }

    /// 获取最终结果（带标点），从 completed_stream 获取
    pub fn get_final_result_with_punctuation(&mut self) -> String {
        // 从 completed_stream 获取结果
        let plain_text = if let Some(stream) = &self.completed_stream {
            let result = stream.get_detailed_result(&self.asr_recognizer);
            tracing::info!("ASR 识别结果: text='{}', tokens={}", result.text, result.tokens.len());
            for (i, t) in result.tokens.iter().enumerate() {
                tracing::info!("  Token[{}]: '{}' ({}ms - {}ms)", i, t.text, t.start_time_ms, t.end_time_ms);
            }
            let text: String = result.tokens.iter()
                .map(|t| t.text.trim())
                .filter(|w| !w.is_empty() && *w != "NE")
                .collect();
            tracing::info!("纯文本: '{}'", text);
            text
        } else {
            tracing::warn!("completed_stream 为空");
            String::new()
        };

        let result = if plain_text.is_empty() {
            String::new()
        } else {
            #[cfg(feature = "vad-onnx")]
            {
                if let Some(model) = &mut self.punct_model {
                    let p = model.add_punctuation(&plain_text);
                    tracing::info!("CT-Transformer 标点结果: '{}'", p);
                    p
                } else {
                    plain_text
                }
            }
            #[cfg(not(feature = "vad-onnx"))]
            { plain_text }
        };

        // 清理 completed_stream（释放内存）
        if let Some(mut stream) = self.completed_stream.take() {
            stream.reset(&self.asr_recognizer);
        }

        // 不改变 pipeline_state，让 process() 方法根据情况自动转换状态
        // 如果在 Completed 状态下检测到新语音，会自动启动新流
        // 如果没有新语音，会保持 Completed 状态直到下次语音到来

        result
    }

    /// 获取最终结果（不带标点）
    pub fn get_final_result(&mut self) -> String {
        let result = if let Some(stream) = &self.asr_stream {
            stream.get_result(&self.asr_recognizer)
        } else {
            String::new()
        };
        let _ = self.reset();
        result
    }

    /// 强制设置 VAD 状态（PushToTalk 模式）
    pub fn force_vad_state(&mut self, speech: bool) {
        if speech && self.pipeline_state == PipelineState::Idle {
            if let Ok(stream) = self.asr_recognizer.create_stream() {
                let stream_static: OnlineStream<'static> = unsafe { std::mem::transmute(stream) };
                self.asr_stream = Some(stream_static);
                self.pipeline_state = PipelineState::Recognizing;
                self.speech_start_frame = self.total_frames;
                tracing::info!("PushToTalk: 立即启动 ASR 流");
            }
        }
    }

    pub fn pipeline_state(&self) -> PipelineState { self.pipeline_state }
    pub fn vad_state(&self) -> VadState {
        if self.vad_manager.is_speech() { VadState::Speech } else { VadState::Silence }
    }
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            total_frames: self.total_frames,
            asr_frames: self.speech_frames as u64,
            speech_duration_ms: if self.speech_start_frame > 0 {
                (self.total_frames - self.speech_start_frame) * 32
            } else { 0 },
        }
    }
}

impl Drop for StreamingPipeline {
    fn drop(&mut self) {
        self.drop_asr_stream();
    }
}

/// 分离稳定和不稳定文本（最后2字保留在 Preedit）
fn split_stable_unstable(text: &str) -> (String, String) {
    if contains_chinese_number(text) {
        return (String::new(), text.to_string());
    }
    const KEEP: usize = 2;
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= KEEP {
        return (String::new(), text.to_string());
    }
    let n = chars.len() - KEEP;
    (chars[..n].iter().collect(), chars[n..].iter().collect())
}

fn contains_chinese_number(text: &str) -> bool {
    text.chars().any(|c| matches!(c,
        '零'|'一'|'二'|'三'|'四'|'五'|'六'|'七'|'八'|'九'|
        '十'|'百'|'千'|'万'|'亿'|'点'
    ))
}

/// 管道统计信息
#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_frames: u64,
    pub asr_frames: u64,
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

    #[test]
    fn test_split_stable_unstable_short() {
        let (stable, unstable) = split_stable_unstable("你好");
        assert_eq!(stable, "");
        assert_eq!(unstable, "你好");
    }

    #[test]
    fn test_split_stable_unstable_long() {
        let (stable, unstable) = split_stable_unstable("今天天气很好");
        assert_eq!(stable, "今天天气");
        assert_eq!(unstable, "很好");
    }

    #[test]
    fn test_split_stable_unstable_chinese_number() {
        // 含中文数字时全部归入 unstable
        let (stable, unstable) = split_stable_unstable("一二三四五");
        assert_eq!(stable, "");
        assert_eq!(unstable, "一二三四五");
    }

    #[test]
    fn test_contains_chinese_number() {
        assert!(contains_chinese_number("一百万"));
        assert!(!contains_chinese_number("今天天气"));
    }
}
