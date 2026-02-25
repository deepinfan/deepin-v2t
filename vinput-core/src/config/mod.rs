//! V-Input 配置模块
//!
//! 统一的配置管理，从 ~/.config/vinput/config.toml 加载

use crate::asr::OnlineRecognizerConfig;
use crate::vad::VadConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 端点检测配置（简化版，只保留实际使用的参数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// 语音结束后的静音等待时间（毫秒）
    #[serde(default = "default_trailing_silence_ms")]
    pub trailing_silence_ms: u64,

    /// 最大语音长度（毫秒），连续说话超过此时长自动断句
    #[serde(default = "default_max_speech_duration_ms")]
    pub max_speech_duration_ms: u64,
}

fn default_trailing_silence_ms() -> u64 { 800 }
fn default_max_speech_duration_ms() -> u64 { 20000 }

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            trailing_silence_ms: 800,
            max_speech_duration_ms: 20000,
        }
    }
}

/// V-Input 完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VInputConfig {
    /// VAD 配置
    pub vad: VadConfig,
    /// ASR 配置
    pub asr: OnlineRecognizerConfig,
    /// 标点模型目录（CT-Transformer ONNX）
    /// 默认：./models/punct-ct-transformer（开发环境）
    /// 或 /usr/share/droplet-voice-input/models/punct-ct-transformer（系统安装）
    #[serde(default = "default_punct_model_dir")]
    pub punct_model_dir: String,
    /// 端点检测配置（自动断句上屏）
    #[serde(default)]
    pub endpoint: EndpointConfig,
}

fn default_punct_model_dir() -> String {
    resolve_punct_model_dir()
}

/// 解析标点模型目录（优先级：环境变量 > 系统路径 > 开发路径）
pub fn resolve_punct_model_dir() -> String {
    if let Ok(dir) = std::env::var("VINPUT_PUNCT_MODEL_DIR") {
        return dir;
    }
    let system_path = "/usr/share/droplet-voice-input/models/punct-ct-transformer";
    if std::path::Path::new(system_path).exists() {
        return system_path.to_string();
    }
    "./models/punct-ct-transformer".to_string()
}

impl Default for VInputConfig {
    fn default() -> Self {
        // 默认模型路径优先级：
        // 1. 环境变量 VINPUT_MODEL_DIR
        // 2. 系统安装路径 /usr/share/droplet-voice-input/models
        // 3. 开发路径 ./models/streaming
        let default_model_dir = std::env::var("VINPUT_MODEL_DIR")
            .unwrap_or_else(|_| {
                let system_path = "/usr/share/droplet-voice-input/models";
                if std::path::Path::new(system_path).exists() {
                    system_path.to_string()
                } else {
                    "./models/streaming".to_string()
                }
            });

        let mut asr_config = OnlineRecognizerConfig::default();
        asr_config.model_dir = default_model_dir;

        Self {
            vad: VadConfig::push_to_talk_default(),
            asr: asr_config,
            punct_model_dir: resolve_punct_model_dir(),
            endpoint: EndpointConfig::default(),
        }
    }
}

impl VInputConfig {
    /// 加载配置文件
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            tracing::info!("配置文件不存在，使用默认配置: {:?}", config_path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&content)?;

        tracing::info!("📋 加载配置成功: {:?}", config_path);
        tracing::info!("📌 标点模型目录: {}", config.punct_model_dir);
        tracing::info!("🎯 端点检测: trailing_silence={}ms, max_speech={}ms",
            config.endpoint.trailing_silence_ms,
            config.endpoint.max_speech_duration_ms
        );
        Ok(config)
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        tracing::info!("保存配置成功: {:?}", config_path);
        Ok(())
    }

    /// 获取配置文件路径
    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("无法获取配置目录")?;

        Ok(config_dir.join("vinput").join("config.toml"))
    }
}
