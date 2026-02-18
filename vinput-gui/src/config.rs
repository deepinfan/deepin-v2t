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
            trailing_silence_ms: 1000,      // 更新为新的默认值
            force_timeout_ms: 60000,
            vad_assisted: true,
            vad_silence_confirm_frames: 8,  // 更新为新的默认值
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
                start_threshold: 0.7,  // 更新为新的默认值
                end_threshold: 0.35,   // 更新为新的默认值
                min_speech_duration: 100,
                min_silence_duration: 700,  // 更新为新的默认值
            },
            asr: AsrConfig {
                model_dir: "/usr/share/droplet-voice-input/models".to_string(),  // 使用系统路径
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
            tracing::info!("配置文件不存在，尝试从示例文件创建: {:?}", path);

            // 尝试从系统示例文件复制
            let example_path = PathBuf::from("/usr/share/droplet-voice-input/config.toml.example");
            if example_path.exists() {
                tracing::info!("从系统示例文件复制: {:?}", example_path);

                // 确保目录存在
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // 复制示例文件
                fs::copy(&example_path, &path)?;
                tracing::info!("配置文件创建成功: {:?}", path);
            } else {
                tracing::info!("示例文件不存在，使用默认配置并保存");

                // 使用默认配置并保存
                let default_config = Self::default();
                default_config.save()?;
                tracing::info!("默认配置已保存: {:?}", path);

                return Ok(default_config);
            }
        }

        // 读取配置文件
        let content = fs::read_to_string(&path)?;
        let config: VInputConfig = toml::from_str(&content)?;
        tracing::info!("配置加载成功: {:?}", path);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = VInputConfig::default();

        // 验证默认值
        assert_eq!(config.asr.model_dir, "/usr/share/droplet-voice-input/models");
        assert_eq!(config.asr.sample_rate, 16000);
        assert_eq!(config.vad.start_threshold, 0.7);
        assert_eq!(config.vad.min_silence_duration, 700);
        assert_eq!(config.endpoint.trailing_silence_ms, 1000);
        assert_eq!(config.endpoint.vad_silence_confirm_frames, 8);
    }

    #[test]
    fn test_config_serialization() {
        let config = VInputConfig::default();

        // 序列化为 TOML
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");

        // 验证包含关键字段
        assert!(toml_str.contains("model_dir"));
        assert!(toml_str.contains("start_threshold"));
        assert!(toml_str.contains("trailing_silence_ms"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[hotwords]
words = {}
global_weight = 2.5
max_words = 10000

[punctuation]
style = "Professional"
pause_ratio = 3.5
min_tokens = 5
allow_exclamation = false
question_strict = true

[vad]
mode = "push-to-talk"
start_threshold = 0.7
end_threshold = 0.35
min_speech_duration = 100
min_silence_duration = 700

[asr]
model_dir = "/usr/share/droplet-voice-input/models"
sample_rate = 16000
hotwords_score = 1.5

[endpoint]
min_speech_duration_ms = 300
max_speech_duration_ms = 30000
trailing_silence_ms = 1000
force_timeout_ms = 60000
vad_assisted = true
vad_silence_confirm_frames = 8
"#;

        let config: VInputConfig = toml::from_str(toml_str).expect("Failed to deserialize");

        // 验证反序列化的值
        assert_eq!(config.asr.model_dir, "/usr/share/droplet-voice-input/models");
        assert_eq!(config.vad.start_threshold, 0.7);
        assert_eq!(config.vad.min_silence_duration, 700);
        assert_eq!(config.endpoint.trailing_silence_ms, 1000);
    }

    #[test]
    fn test_config_roundtrip() {
        let original = VInputConfig::default();

        // 序列化
        let toml_str = toml::to_string_pretty(&original).expect("Failed to serialize");

        // 反序列化
        let deserialized: VInputConfig = toml::from_str(&toml_str).expect("Failed to deserialize");

        // 验证关键字段一致
        assert_eq!(original.asr.model_dir, deserialized.asr.model_dir);
        assert_eq!(original.vad.start_threshold, deserialized.vad.start_threshold);
        assert_eq!(original.endpoint.trailing_silence_ms, deserialized.endpoint.trailing_silence_ms);
    }

    #[test]
    fn test_vad_config_values() {
        let config = VInputConfig::default();

        // 验证 VAD 参数在合理范围内
        assert!(config.vad.start_threshold >= 0.0 && config.vad.start_threshold <= 1.0);
        assert!(config.vad.end_threshold >= 0.0 && config.vad.end_threshold <= 1.0);
        assert!(config.vad.start_threshold > config.vad.end_threshold);
        assert!(config.vad.min_silence_duration > 0);
    }

    #[test]
    fn test_endpoint_config_values() {
        let config = VInputConfig::default();

        // 验证端点检测参数在合理范围内
        assert!(config.endpoint.min_speech_duration_ms > 0);
        assert!(config.endpoint.max_speech_duration_ms > config.endpoint.min_speech_duration_ms);
        assert!(config.endpoint.trailing_silence_ms > 0);
        assert!(config.endpoint.vad_silence_confirm_frames > 0);
    }

    #[test]
    fn test_hotwords_config() {
        let config = VInputConfig::default();

        // 验证热词配置
        assert!(config.hotwords.words.is_empty());
        assert!(config.hotwords.global_weight > 0.0);
        assert!(config.hotwords.max_words > 0);
    }

    #[test]
    fn test_punctuation_config() {
        let config = VInputConfig::default();

        // 验证标点配置
        assert_eq!(config.punctuation.style, "Professional");
        assert!(config.punctuation.pause_ratio > 0.0);
        assert!(config.punctuation.min_tokens > 0);
    }
}

