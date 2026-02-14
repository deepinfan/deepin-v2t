//! sherpa-onnx 在线识别器安全封装

use crate::error::{VInputError, VInputResult};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;

// 引入 bindgen 生成的绑定
include!(concat!(env!("OUT_DIR"), "/sherpa_bindings.rs"));

/// 在线识别器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineRecognizerConfig {
    /// 模型目录路径
    pub model_dir: String,
    /// 采样率 (Hz)
    pub sample_rate: i32,
    /// 特征维度
    pub feat_dim: i32,
    /// 解码方法 ("greedy_search" 或 "modified_beam_search")
    pub decoding_method: String,
    /// 最大活跃路径数
    pub max_active_paths: i32,
    /// 热词文件路径（可选）
    pub hotwords_file: Option<String>,
    /// 热词得分
    pub hotwords_score: f32,
}

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
        let encoder_path = model_dir.join("encoder-epoch-99-avg-1.onnx");
        let decoder_path = model_dir.join("decoder-epoch-99-avg-1.onnx");
        let joiner_path = model_dir.join("joiner-epoch-99-avg-1.onnx");
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

    /// 获取识别结果
    pub fn get_result(&self, recognizer: &OnlineRecognizer) -> String {
        unsafe {
            let result_ptr = SherpaOnnxGetOnlineStreamResult(recognizer.as_ptr(), self.inner);
            if result_ptr.is_null() {
                return String::new();
            }

            let text_ptr = (*result_ptr).text;
            let text = if !text_ptr.is_null() {
                CStr::from_ptr(text_ptr)
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };

            SherpaOnnxDestroyOnlineRecognizerResult(result_ptr);
            text
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
