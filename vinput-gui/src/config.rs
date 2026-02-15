//! V-Input 配置管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// V-Input 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VInputConfig {
    /// 热词配置
    pub hotwords: HotwordsConfig,
    /// 标点配置
    pub punctuation: PunctuationConfig,
    /// VAD 配置
    pub vad: VadConfig,
    /// ASR 配置
    pub asr: AsrConfig,
    /// 端点检测配置
    #[serde(default)]
    pub endpoint: EndpointConfig,
}

/// 热词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotwordsConfig {
    /// 热词列表 (词汇 → 权重)
    pub words: HashMap<String, f32>,
    /// 全局权重
    pub global_weight: f32,
    /// 最大热词数
    pub max_words: usize,
}

/// 标点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunctuationConfig {
    /// 风格名称
    pub style: String,
    /// 停顿检测阈值
    pub pause_ratio: f32,
    /// 最小 token 数
    pub min_tokens: usize,
    /// 允许感叹号
    pub allow_exclamation: bool,
    /// 问号严格模式
    pub question_strict: bool,
}

/// VAD 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// VAD 模式 (push-to-talk / continuous)
    pub mode: String,
    /// 启动阈值
    pub start_threshold: f32,
    /// 结束阈值
    pub end_threshold: f32,
    /// 最小语音时长 (ms)
    pub min_speech_duration: u64,
    /// 最小静音时长 (ms)
    pub min_silence_duration: u64,
}

/// ASR 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    /// 模型目录
    pub model_dir: String,
    /// 采样率
    pub sample_rate: i32,
    /// 热词文件路径
    pub hotwords_file: Option<String>,
    /// 热词分数
    pub hotwords_score: f32,
}

/// 端点检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// 最小语音长度（毫秒）
    pub min_speech_duration_ms: u64,
    /// 最大语音长度（毫秒）
    pub max_speech_duration_ms: u64,
    /// 语音结束后的静音等待时间（毫秒）
    pub trailing_silence_ms: u64,
    /// 强制超时（毫秒）
    pub force_timeout_ms: u64,
    /// 是否启用 VAD 辅助端点检测
    pub vad_assisted: bool,
    /// VAD 检测到静音后的确认帧数
    pub vad_silence_confirm_frames: usize,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            min_speech_duration_ms: 300,
            max_speech_duration_ms: 30000,
            trailing_silence_ms: 800,
            force_timeout_ms: 60000,
            vad_assisted: true,
            vad_silence_confirm_frames: 5,
        }
    }
}

impl Default for VInputConfig {
    fn default() -> Self {
        Self {
            hotwords: HotwordsConfig {
                words: HashMap::new(),
                global_weight: 2.5,
                max_words: 10000,
            },
            punctuation: PunctuationConfig {
                style: "Professional".to_string(),
                pause_ratio: 3.5,
                min_tokens: 5,
                allow_exclamation: false,
                question_strict: true,
            },
            vad: VadConfig {
                mode: "push-to-talk".to_string(),
                start_threshold: 0.5,
                end_threshold: 0.3,
                min_speech_duration: 250,
                min_silence_duration: 300,
            },
            asr: AsrConfig {
                model_dir: "/home/deepin/deepin-v2t/models/streaming".to_string(),
                sample_rate: 16000,
                hotwords_file: None,
                hotwords_score: 1.5,
            },
            endpoint: EndpointConfig::default(),
        }
    }
}

impl VInputConfig {
    /// 获取配置文件路径
    pub fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("vinput").join("config.toml")
        } else {
            PathBuf::from(".vinput-config.toml")
        }
    }

    /// 加载配置
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config: VInputConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
