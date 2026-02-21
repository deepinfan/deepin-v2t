//! Streaming Pipeline - VAD-ASR 流式识别管道
//!
//! 将 VAD 检测结果与 ASR 识别器连接，实现端到端的流式语音识别

use crate::asr::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use crate::endpointing::{EndpointDetector, EndpointDetectorConfig, EndpointResult};
use crate::error::VInputResult;
use crate::punctuation::{PunctuationEngine, StyleProfile};
use crate::vad::{VadConfig, VadManager, VadState};
use std::time::Instant;

/// 流式管道配置
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// VAD 配置
    pub vad_config: VadConfig,
    /// ASR 配置
    pub asr_config: OnlineRecognizerConfig,
    /// 标点风格配置
    pub punctuation_profile: StyleProfile,
    /// 端点检测配置
    pub endpoint_config: EndpointDetectorConfig,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            vad_config: VadConfig::push_to_talk_default(),
            asr_config: OnlineRecognizerConfig::default(),
            punctuation_profile: StyleProfile::default(),
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
    /// 是否应该添加逗号（检测到停顿）
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
    punctuation_engine: PunctuationEngine,
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
}

impl StreamingPipeline {
    /// 创建新的流式管道
    pub fn new(config: StreamingConfig) -> VInputResult<Self> {
        tracing::info!("📍 StreamingPipeline::new - 接收到的标点配置: pause_ratio={}, min_tokens={}",
            config.punctuation_profile.streaming_pause_ratio,
            config.punctuation_profile.streaming_min_tokens
        );
        tracing::info!("🎯 端点检测配置: trailing_silence={}ms, min_speech={}ms",
            config.endpoint_config.trailing_silence_ms,
            config.endpoint_config.min_speech_duration_ms
        );

        let vad_manager = VadManager::new(config.vad_config.clone())?;
        let asr_recognizer = OnlineRecognizer::new(&config.asr_config)?;
        let punctuation_engine = PunctuationEngine::new(config.punctuation_profile.clone());
        let endpoint_detector = EndpointDetector::new(config.endpoint_config.clone());

        Ok(Self {
            config,
            vad_manager,
            asr_recognizer,
            punctuation_engine,
            endpoint_detector,
            asr_stream: None,
            pipeline_state: PipelineState::Idle,
            speech_start_time: None,
            asr_endpoint_grace_remaining: 0,
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
                                self.asr_frames += 1;
                                tracing::info!(
                                    "✅ 注入 Pre-roll 音频: {} 样本",
                                    pre_roll_audio.len()
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
                    if self.asr_endpoint_grace_remaining == 0 {
                        // 缓冲期结束：刷新并提交
                        stream.input_finished();
                        self.pipeline_state = PipelineState::Completed;
                        tracing::info!("Pipeline: ASR 端点缓冲期结束，准备上屏");
                    } else {
                        // 缓冲期内继续喂音频（已在上方的 Recognizing 分支处理）
                        tracing::debug!("Pipeline: ASR 端点缓冲期剩余 {} 帧", self.asr_endpoint_grace_remaining);
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

        // 7. 检测是否应该添加逗号（停顿检测）
        let should_add_comma = false; // TODO: 实现停顿检测逻辑

        Ok(StreamingResult {
            partial_result,
            stable_text,
            unstable_text,
            should_add_comma,
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

    /// 重置管道状态
    pub fn reset(&mut self) -> VInputResult<()> {
        tracing::debug!("Pipeline: Resetting");

        // 销毁 ASR 流
        if let Some(mut stream) = self.asr_stream.take() {
            stream.reset(&self.asr_recognizer);
        }

        // 重置 VAD
        self.vad_manager.reset();

        // 重置标点引擎
        self.punctuation_engine.reset_sentence();

        // 重置端点检测器
        self.endpoint_detector.reset();

        // 重置状态
        self.pipeline_state = PipelineState::Idle;
        self.speech_start_time = None;
        self.asr_endpoint_grace_remaining = 0;

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

    /// 分离稳定和不稳定文本
    ///
    /// 保留最后 N 个字符在 Preedit（不稳定），其余部分可以立即上屏（稳定）
    ///
    /// 智能过滤：如果整个识别结果包含中文数字，则全部保留在 Preedit，
    /// 避免 ITN 转换时无法修改已上屏的数字
    fn split_stable_unstable(&self, text: &str) -> (String, String) {
        // 🎯 优先检查：如果整个文本包含中文数字，全部保留在 Preedit
        if Self::contains_chinese_number(text) {
            return (String::new(), text.to_string());
        }

        // 如果不包含数字，按正常逻辑分离
        const KEEP_LAST_CHARS: usize = 2; // 保留最后2个字符在 Preedit

        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= KEEP_LAST_CHARS {
            // 全部不稳定
            return (String::new(), text.to_string());
        }

        let stable_count = chars.len() - KEEP_LAST_CHARS;
        let stable: String = chars[..stable_count].iter().collect();
        let unstable: String = chars[stable_count..].iter().collect();

        (stable, unstable)
    }

    /// 检查文本是否包含中文数字字符
    ///
    /// 用于判断是否需要延迟上屏，等待 ITN 处理
    fn contains_chinese_number(text: &str) -> bool {
        text.chars().any(|c| matches!(c,
            '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
            '十' | '百' | '千' | '万' | '亿' | '点'
        ))
    }

    /// 获取实时识别结果（带实时标点处理）
    ///
    /// 用于在识别过程中显示带标点的 Preedit
    /// 不会重置管道状态，不会添加句尾标点
    pub fn get_partial_result_with_punctuation(&mut self) -> String {
        if let Some(stream) = &self.asr_stream {
            // 获取详细结果（包含 Token 和时间戳）
            let detailed_result = stream.get_detailed_result(&self.asr_recognizer);

            if detailed_result.is_empty() {
                return String::new();
            }

            // 处理每个 Token，添加逗号（但不添加句尾标点）
            let mut text_with_commas = String::new();

            for token in &detailed_result.tokens {
                // 转换为 TokenInfo
                let token_info = token.to_token_info();

                // 处理 Token（可能在前面添加逗号）
                if let Some(processed_token) = self.punctuation_engine.process_token(token_info) {
                    text_with_commas.push_str(&processed_token);
                }
            }

            text_with_commas
        } else {
            String::new()
        }
    }

    /// 获取最终识别结果（带标点）
    ///
    /// 调用此方法后会自动重置管道状态
    pub fn get_final_result_with_punctuation(&mut self) -> String {
        // 通知解码器输入已结束，触发最终 beam search 完成
        // 对于轻声末字：ASR 缓冲区里有这些帧，但未经 input_finished() 就无法提交
        if let Some(stream) = &mut self.asr_stream {
            tracing::info!("🔚 调用 input_finished()，刷新 ASR 解码器缓冲区");
            stream.input_finished();
        }

        // 最终一次解码，处理 input_finished() 后的剩余帧
        if let Some(stream) = &mut self.asr_stream {
            if stream.is_ready(&self.asr_recognizer) {
                stream.decode(&self.asr_recognizer);
                tracing::info!("🔚 最终解码完成");
            }
        }

        let result = if let Some(stream) = &self.asr_stream {
            // 获取详细结果（包含 Token 和时间戳）
            let detailed_result = stream.get_detailed_result(&self.asr_recognizer);

            tracing::info!("📊 ASR 识别结果详情:");
            tracing::info!("  - text: '{}'", detailed_result.text);
            tracing::info!("  - text.len(): {}", detailed_result.text.len());
            tracing::info!("  - token_count: {}", detailed_result.tokens.len());
            tracing::info!("  - is_empty(): {}", detailed_result.is_empty());

            if detailed_result.is_empty() {
                tracing::warn!("⚠️  识别结果为空（text 为空字符串）");
                String::new()
            } else {
                // 打印所有 Token 信息（INFO 级别，帮助分析断句）
                for (i, token) in detailed_result.tokens.iter().enumerate() {
                    tracing::info!("  Token[{}]: '{}' ({}ms - {}ms, duration={}ms)",
                        i, token.text, token.start_time_ms, token.end_time_ms, token.duration_ms());
                }

                // 第一步：构建纯文本（无逗号），同时收集逻辑连接词的逗号位置
                // PauseEngine 的时间戳方法对 Paraformer 模型无效，跳过停顿检测
                // 只保留 RuleLayer 的逻辑连接词逗号（如"因为"、"所以"）
                let mut plain_text = String::new();
                let mut logic_comma_positions: Vec<usize> = Vec::new(); // 逻辑词前的字符位置
                let mut token_char_count = 0usize;

                for token in &detailed_result.tokens {
                    let token_info = token.to_token_info();
                    let word = token_info.text.trim().to_string();
                    if word.is_empty() || word == "NE" {
                        continue;
                    }
                    // 检查逻辑连接词规则（在词【前】插入逗号，不依赖时间戳，始终有效）
                    let is_logic_word = crate::punctuation::rules::RuleLayer::is_logic_word(&word);
                    if is_logic_word && token_char_count >= 8 {
                        logic_comma_positions.push(token_char_count);
                    }
                    plain_text.push_str(&word);
                    token_char_count += word.chars().count();
                }

                tracing::info!("📝 纯文本: '{}', 逻辑词逗号位置: {:?}",
                    plain_text, logic_comma_positions);

                // 第二步：将逻辑连接词逗号位置插入纯文本
                let mut comma_positions = logic_comma_positions;
                comma_positions.sort_unstable();
                comma_positions.dedup();

                // 第四步：将逗号插入到纯文本的对应字符位置
                const MIN_CHARS_BETWEEN_COMMAS: usize = 3;
                let mut final_text = String::with_capacity(plain_text.len() + comma_positions.len() * 3);
                let mut last_comma_at: Option<usize> = None;

                for (i, ch) in plain_text.char_indices().map(|(_, c)| c).enumerate() {
                    // 检查此位置是否应插入逗号
                    if i > 0 && comma_positions.contains(&i) {
                        let ok = match last_comma_at {
                            None => i >= MIN_CHARS_BETWEEN_COMMAS,
                            Some(last) => i >= last + MIN_CHARS_BETWEEN_COMMAS,
                        };
                        if ok {
                            tracing::info!("  ✅ 在第 {} 个字符前插入逗号", i);
                            final_text.push('，');
                            last_comma_at = Some(i);
                        }
                    }
                    final_text.push(ch);
                }

                tracing::info!("📝 插入逗号后: '{}'", final_text);

                // 检测 VAD 能量变化（用于问号检测）
                let energy_rising = self.endpoint_detector.analyze_energy_trend();

                // 获取语音持续时间用于标点决策
                // ⚠️  用 asr_frames × 32ms 而非墙上时钟
                //     理由：快速处理（测试回放）时墙上时钟远短于实际音频时长
                let speech_duration_ms = self.asr_frames * 32;

                tracing::debug!("🔚 准备添加句尾标点: speech_duration_ms={}, energy_rising={}",
                    speech_duration_ms, energy_rising);

                // 🎯 如果最后一个字符是逗号，替换为句尾标点
                if final_text.ends_with('，') {
                    final_text.pop(); // 移除最后的逗号
                    tracing::debug!("  检测到末尾逗号，将替换为句尾标点");
                }

                // 添加句尾标点
                // 用 determine_ending(final_text) 而非 finalize_sentence()
                // 原因：finalize_sentence 依赖 current_sentence（由 process_token 填充），
                //       但当前流程直接构建 final_text，current_sentence 始终为空
                let ending = self.punctuation_engine.determine_ending(
                    &final_text,
                    speech_duration_ms,
                    energy_rising,
                );

                tracing::info!("  句尾标点: '{}'（基于文本: '{}'）", ending, final_text);
                final_text.push_str(&ending);

                tracing::info!("✅ 标点处理完成: '{}'", final_text);
                final_text
            }
        } else {
            tracing::warn!("⚠️  ASR 流为空");
            String::new()
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
    fn test_pipeline_state_transitions() {
        assert_eq!(PipelineState::Idle, PipelineState::Idle);
        assert_ne!(PipelineState::Idle, PipelineState::Recognizing);
    }
}
