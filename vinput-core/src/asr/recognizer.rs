//! sherpa-onnx 在线识别器安全封装

use crate::error::{VInputError, VInputResult};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;

// 引入 bindgen 生成的绑定
include!(concat!(env!("OUT_DIR"), "/sherpa_bindings.rs"));

/// 识别的 Token 信息
#[derive(Debug, Clone)]
pub struct RecognizedToken {
    /// Token 文本
    pub text: String,
    /// Token 开始时间（毫秒）
    pub start_time_ms: u64,
    /// Token 结束时间（毫秒）
    pub end_time_ms: u64,
    /// 置信度（0.0-1.0）
    pub confidence: f32,
}

impl RecognizedToken {
    /// Token 时长（毫秒）
    pub fn duration_ms(&self) -> u64 {
        self.end_time_ms.saturating_sub(self.start_time_ms)
    }

    /// 转换为 PunctuationEngine 的 TokenInfo
    pub fn to_token_info(&self) -> crate::punctuation::TokenInfo {
        crate::punctuation::TokenInfo::new(
            self.text.clone(),
            self.start_time_ms,
            self.end_time_ms,
        )
    }
}

/// 识别结果（包含 Token 信息）
#[derive(Debug, Clone)]
pub struct RecognitionResult {
    /// 识别文本
    pub text: String,
    /// Token 列表（包含时间戳）
    pub tokens: Vec<RecognizedToken>,
}

impl RecognitionResult {
    /// 创建空结果
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            tokens: Vec::new(),
        }
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// 在线识别器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineRecognizerConfig {
    /// 模型目录路径
    pub model_dir: String,
    /// 采样率 (Hz)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: i32,
    /// 特征维度
    #[serde(default = "default_feat_dim")]
    pub feat_dim: i32,
    /// 解码方法 ("greedy_search" 或 "modified_beam_search")
    #[serde(default = "default_decoding_method")]
    pub decoding_method: String,
    /// 最大活跃路径数
    #[serde(default = "default_max_active_paths")]
    pub max_active_paths: i32,
    /// 热词文件路径（可选）
    #[serde(default)]
    pub hotwords_file: Option<String>,
    /// 热词得分
    #[serde(default = "default_hotwords_score")]
    pub hotwords_score: f32,
}

// 默认值函数
fn default_sample_rate() -> i32 { 16000 }
fn default_feat_dim() -> i32 { 80 }
fn default_decoding_method() -> String { "greedy_search".to_string() }
fn default_max_active_paths() -> i32 { 4 }
fn default_hotwords_score() -> f32 { 1.5 }

impl Default for OnlineRecognizerConfig {
    fn default() -> Self {
        Self {
            model_dir: String::new(),
            sample_rate: 16000,
            feat_dim: 80,
            decoding_method: "greedy_search".to_string(),
            max_active_paths: 4,
            hotwords_file: None,
            hotwords_score: 1.5,
        }
    }
}

/// 在线识别器（线程安全）
pub struct OnlineRecognizer {
    inner: *const SherpaOnnxOnlineRecognizer,
}

// sherpa-onnx 的 recognizer 是线程安全的
unsafe impl Send for OnlineRecognizer {}
unsafe impl Sync for OnlineRecognizer {}

impl OnlineRecognizer {
    /// 创建在线识别器
    pub fn new(config: &OnlineRecognizerConfig) -> VInputResult<Self> {
        // 验证模型路径
        let model_dir = Path::new(&config.model_dir);
        if !model_dir.exists() {
            return Err(VInputError::ModelLoad {
                path: config.model_dir.clone(),
                reason: "Model directory not found".to_string(),
            });
        }

        // 构建 C API 配置
        // 使用 INT8 量化模型（更小、更快，精度略有下降）
        let encoder_path = model_dir.join("encoder-epoch-99-avg-1.int8.onnx");
        let decoder_path = model_dir.join("decoder-epoch-99-avg-1.int8.onnx");
        let joiner_path = model_dir.join("joiner-epoch-99-avg-1.int8.onnx");
        let tokens_path = model_dir.join("tokens.txt");

        // 转换为 CString
        let encoder_cstr = CString::new(encoder_path.to_str().unwrap())
            .map_err(|e| VInputError::ModelLoad {
                path: config.model_dir.clone(),
                reason: format!("Invalid path encoding: {}", e),
            })?;
        let decoder_cstr = CString::new(decoder_path.to_str().unwrap())
            .map_err(|e| VInputError::ModelLoad {
                path: config.model_dir.clone(),
                reason: format!("Invalid path encoding: {}", e),
            })?;
        let joiner_cstr = CString::new(joiner_path.to_str().unwrap())
            .map_err(|e| VInputError::ModelLoad {
                path: config.model_dir.clone(),
                reason: format!("Invalid path encoding: {}", e),
            })?;
        let tokens_cstr = CString::new(tokens_path.to_str().unwrap())
            .map_err(|e| VInputError::ModelLoad {
                path: config.model_dir.clone(),
                reason: format!("Invalid path encoding: {}", e),
            })?;

        let provider_cstr = CString::new("cpu").unwrap();
        let decoding_method_cstr = CString::new(config.decoding_method.as_str()).unwrap();
        let hotwords_cstr = config
            .hotwords_file
            .as_ref()
            .map(|s| CString::new(s.as_str()).ok())
            .flatten();

        // 构建配置结构体
        let transducer_config = SherpaOnnxOnlineTransducerModelConfig {
            encoder: encoder_cstr.as_ptr(),
            decoder: decoder_cstr.as_ptr(),
            joiner: joiner_cstr.as_ptr(),
        };

        let model_config = SherpaOnnxOnlineModelConfig {
            transducer: transducer_config,
            paraformer: unsafe { std::mem::zeroed() },
            zipformer2_ctc: unsafe { std::mem::zeroed() },
            tokens: tokens_cstr.as_ptr(),
            num_threads: 2,
            provider: provider_cstr.as_ptr(),
            debug: 0,
            model_type: ptr::null(),
            modeling_unit: ptr::null(),
            bpe_vocab: ptr::null(),
            tokens_buf: ptr::null(),
            tokens_buf_size: 0,
            nemo_ctc: unsafe { std::mem::zeroed() },
            t_one_ctc: unsafe { std::mem::zeroed() },
        };

        let recognizer_config = SherpaOnnxOnlineRecognizerConfig {
            feat_config: SherpaOnnxFeatureConfig {
                sample_rate: config.sample_rate,
                feature_dim: config.feat_dim,
            },
            model_config,
            decoding_method: decoding_method_cstr.as_ptr(),
            max_active_paths: config.max_active_paths,
            enable_endpoint: 1,
            rule1_min_trailing_silence: 2.4,
            rule2_min_trailing_silence: 1.2,
            rule3_min_utterance_length: 20.0,
            hotwords_file: hotwords_cstr
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null()),
            hotwords_score: config.hotwords_score,
            ctc_fst_decoder_config: unsafe { std::mem::zeroed() },
            rule_fsts: ptr::null(),
            rule_fars: ptr::null(),
            blank_penalty: 0.0,
            hotwords_buf: ptr::null(),
            hotwords_buf_size: 0,
            hr: unsafe { std::mem::zeroed() },
        };

        // 调用 C API 创建识别器
        let recognizer = unsafe { SherpaOnnxCreateOnlineRecognizer(&recognizer_config) };

        if recognizer.is_null() {
            return Err(VInputError::ModelLoad {
                path: config.model_dir.clone(),
                reason: "Failed to create recognizer".to_string(),
            });
        }

        Ok(Self { inner: recognizer })
    }

    /// 预热模型缓存
    ///
    /// 运行一次 dummy 推理，预热 ONNX Runtime 缓存，减少首次推理延迟
    pub fn warmup(&self) -> VInputResult<()> {
        tracing::info!("🔥 开始预热 ASR 模型缓存...");
        let start = std::time::Instant::now();

        // 创建临时流
        let mut stream = self.create_stream()?;

        // 送入 dummy 音频（512 samples = 32ms @ 16kHz）
        let dummy_audio = vec![0.0f32; 512];
        stream.accept_waveform(&dummy_audio, 16000);

        // 执行一次解码
        if stream.is_ready(self) {
            stream.decode(self);
        }

        // 获取结果（忽略）
        let _ = stream.get_result(self);

        let elapsed = start.elapsed();
        tracing::info!("✅ 模型预热完成，耗时: {:.2}ms", elapsed.as_secs_f32() * 1000.0);

        Ok(())
    }

    /// 创建新的识别流
    pub fn create_stream(&self) -> VInputResult<OnlineStream<'_>> {
        let stream = unsafe { SherpaOnnxCreateOnlineStream(self.inner) };

        if stream.is_null() {
            return Err(VInputError::AsrInference(
                "Failed to create stream".to_string(),
            ));
        }

        Ok(OnlineStream {
            inner: stream,
            _recognizer: std::marker::PhantomData,
        })
    }

    /// 获取原始指针（仅供内部使用）
    pub(crate) fn as_ptr(&self) -> *const SherpaOnnxOnlineRecognizer {
        self.inner
    }
}

impl Drop for OnlineRecognizer {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                SherpaOnnxDestroyOnlineRecognizer(self.inner);
            }
        }
    }
}

/// 在线识别流
pub struct OnlineStream<'a> {
    inner: *const SherpaOnnxOnlineStream,
    _recognizer: std::marker::PhantomData<&'a OnlineRecognizer>,
}

// sherpa-onnx 的 stream 是线程安全的
// 虽然它是裸指针，但 C 库保证了线程安全性
unsafe impl Send for OnlineStream<'_> {}
unsafe impl Sync for OnlineStream<'_> {}

impl<'a> OnlineStream<'a> {
    /// 输入音频数据（16kHz, 单声道, f32 格式）
    pub fn accept_waveform(&mut self, samples: &[f32], sample_rate: i32) {
        unsafe {
            SherpaOnnxOnlineStreamAcceptWaveform(
                self.inner,
                sample_rate,
                samples.as_ptr(),
                samples.len() as i32,
            );
        }
    }

    /// 检查流是否准备好解码
    pub fn is_ready(&self, recognizer: &OnlineRecognizer) -> bool {
        unsafe { SherpaOnnxIsOnlineStreamReady(recognizer.as_ptr(), self.inner) != 0 }
    }

    /// 解码当前流
    pub fn decode(&mut self, recognizer: &OnlineRecognizer) {
        unsafe {
            SherpaOnnxDecodeOnlineStream(recognizer.as_ptr(), self.inner);
        }
    }

    /// 获取识别结果（仅文本）
    pub fn get_result(&self, recognizer: &OnlineRecognizer) -> String {
        self.get_detailed_result(recognizer).text
    }

    /// 获取详细识别结果（包含 Token 和时间戳）
    pub fn get_detailed_result(&self, recognizer: &OnlineRecognizer) -> RecognitionResult {
        unsafe {
            let result_ptr = SherpaOnnxGetOnlineStreamResult(recognizer.as_ptr(), self.inner);
            if result_ptr.is_null() {
                return RecognitionResult::empty();
            }

            let text_ptr = (*result_ptr).text;
            let text = if !text_ptr.is_null() {
                CStr::from_ptr(text_ptr)
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };

            // 提取 Tokens 和时间戳
            let mut tokens = Vec::new();
            let count = (*result_ptr).count as usize;

            if count > 0 && !(*result_ptr).tokens_arr.is_null() && !(*result_ptr).timestamps.is_null() {
                let tokens_arr = std::slice::from_raw_parts((*result_ptr).tokens_arr, count);
                let timestamps = std::slice::from_raw_parts((*result_ptr).timestamps, count);

                // 调试：打印原始 timestamps 数组
                tracing::debug!("📍 Sherpa-ONNX 原始 timestamps (秒): {:?}",
                    timestamps.iter().take(count.min(20)).collect::<Vec<_>>());

                for i in 0..count {
                    if !tokens_arr[i].is_null() {
                        let token_text = CStr::from_ptr(tokens_arr[i])
                            .to_string_lossy()
                            .into_owned();

                        // timestamps[i] 是相对开始时间（秒）
                        // 我们需要计算每个 token 的开始和结束时间
                        let start_time_s = timestamps[i];
                        let end_time_s = if i + 1 < count {
                            timestamps[i + 1]
                        } else {
                            start_time_s + 0.2  // 最后一个 token，估计 200ms 时长
                        };

                        tokens.push(RecognizedToken {
                            text: token_text,
                            start_time_ms: (start_time_s * 1000.0) as u64,
                            end_time_ms: (end_time_s * 1000.0) as u64,
                            confidence: 1.0,  // Sherpa-ONNX 不提供置信度
                        });
                    }
                }
            }

            let result = RecognitionResult {
                text,
                tokens,
            };

            SherpaOnnxDestroyOnlineRecognizerResult(result_ptr);
            result
        }
    }

    /// 检查是否检测到端点
    pub fn is_endpoint(&self, recognizer: &OnlineRecognizer) -> bool {
        unsafe { SherpaOnnxOnlineStreamIsEndpoint(recognizer.as_ptr(), self.inner) != 0 }
    }

    /// 重置流状态
    pub fn reset(&mut self, recognizer: &OnlineRecognizer) {
        unsafe {
            SherpaOnnxOnlineStreamReset(recognizer.as_ptr(), self.inner);
        }
    }

    /// 标记输入结束
    pub fn input_finished(&mut self) {
        unsafe {
            SherpaOnnxOnlineStreamInputFinished(self.inner);
        }
    }
}

impl<'a> Drop for OnlineStream<'a> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                SherpaOnnxDestroyOnlineStream(self.inner);
            }
        }
    }
}
