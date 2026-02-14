//! V-Input 配置模块
//!
//! 统一的配置管理，从 ~/.config/vinput/config.toml 加载

use crate::asr::OnlineRecognizerConfig;
use crate::hotwords::HotwordsConfig;
use crate::punctuation::PunctuationConfig;
use crate::vad::VadConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// V-Input 完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VInputConfig {
    /// VAD 配置
    pub vad: VadConfig,
    /// ASR 配置
    pub asr: OnlineRecognizerConfig,
    /// 标点配置
    pub punctuation: PunctuationConfig,
    /// 热词配置
    pub hotwords: HotwordsConfig,
}

impl Default for VInputConfig {
    fn default() -> Self {
        // 默认模型路径
        let default_model_dir = std::env::var("VINPUT_MODEL_DIR")
            .unwrap_or_else(|_| "/home/deepin/deepin-v2t/models/streaming".to_string());

        let mut asr_config = OnlineRecognizerConfig::default();
        asr_config.model_dir = default_model_dir;

        Self {
            vad: VadConfig::push_to_talk_default(),
            asr: asr_config,
            punctuation: PunctuationConfig::default(),
            hotwords: HotwordsConfig::default(),
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
        tracing::info!("📊 标点配置: pause_ratio={}, min_tokens={}, allow_exclamation={}",
            config.punctuation.streaming_pause_ratio,
            config.punctuation.streaming_min_tokens,
            config.punctuation.allow_exclamation
        );
        Ok(config)
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        // 确保目录存在
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
