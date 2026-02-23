//! V-Input 配置管理
//!
//! 此配置结构必须与核心的 TOML 格式完全匹配，以便安全地读写
//! ~/.config/vinput/config.toml，不丢失核心依赖的任何字段。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// V-Input 完整配置（与实际 TOML 文件结构一一对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VInputConfig {
    /// 基本配置
    #[serde(default)]
    pub basic: BasicConfig,
    /// 热词配置（UI 不暴露，仅做透传保留）
    #[serde(default)]
    pub hotwords: HotwordsConfig,
    /// VAD 配置（嵌套结构）
    #[serde(default)]
    pub vad: VadConfig,
    /// ASR 配置
    pub asr: AsrConfig,
    /// 端点检测配置
    #[serde(default)]
    pub endpoint: EndpointConfig,
}

// ── 基本配置 ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicConfig {
    pub hotkey: String,
}

impl Default for BasicConfig {
    fn default() -> Self {
        Self { hotkey: "RCtrl".to_string() }
    }
}

// ── 热词配置（透传，不在 UI 显示）────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotwordsConfig {
    pub words: HashMap<String, f32>,
    pub global_weight: f32,
    pub max_words: usize,
}

impl Default for HotwordsConfig {
    fn default() -> Self {
        Self {
            words: HashMap::new(),
            global_weight: 2.5,
            max_words: 10000,
        }
    }
}

// ── VAD 配置（嵌套，与 [vad.*] TOML 节对应）─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    #[serde(default)]
    pub silero: VadSileroConfig,
    #[serde(default)]
    pub energy_gate: VadEnergyGateConfig,
    #[serde(default)]
    pub hysteresis: VadHysteresisConfig,
    #[serde(default)]
    pub pre_roll: VadPreRollConfig,
    #[serde(default)]
    pub transient_filter: VadTransientFilterConfig,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            silero: VadSileroConfig::default(),
            energy_gate: VadEnergyGateConfig::default(),
            hysteresis: VadHysteresisConfig::default(),
            pre_roll: VadPreRollConfig::default(),
            transient_filter: VadTransientFilterConfig::default(),
        }
    }
}

/// [vad.silero]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadSileroConfig {
    pub model_path: String,
    pub sample_rate: i32,
    pub frame_size: usize,
}

impl Default for VadSileroConfig {
    fn default() -> Self {
        Self {
            model_path: "/usr/share/droplet-voice-input/models/silero-vad/silero_vad.onnx".to_string(),
            sample_rate: 16000,
            frame_size: 512,
        }
    }
}

/// [vad.energy_gate]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadEnergyGateConfig {
    pub enabled: bool,
    pub noise_multiplier: f32,
    pub baseline_alpha: f32,
    pub initial_baseline: f32,
}

impl Default for VadEnergyGateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            noise_multiplier: 2.5,
            baseline_alpha: 0.95,
            initial_baseline: 0.001,
        }
    }
}

/// [vad.hysteresis] — UI 暴露此节的 4 个字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadHysteresisConfig {
    pub start_threshold: f32,
    pub end_threshold: f32,
    pub min_speech_duration_ms: u64,
    pub min_silence_duration_ms: u64,
    pub max_candidate_gap_frames: u32,
}

impl Default for VadHysteresisConfig {
    fn default() -> Self {
        Self {
            start_threshold: 0.25,
            end_threshold: 0.08,
            min_speech_duration_ms: 100,
            min_silence_duration_ms: 500,
            max_candidate_gap_frames: 3,
        }
    }
}

/// [vad.pre_roll]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadPreRollConfig {
    pub enabled: bool,
    pub duration_ms: u64,
    pub capacity: usize,
}

impl Default for VadPreRollConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_ms: 750,
            capacity: 12000,
        }
    }
}

/// [vad.transient_filter]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadTransientFilterConfig {
    pub enabled: bool,
    pub max_duration_ms: u64,
    pub rms_threshold: f32,
}

impl Default for VadTransientFilterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_duration_ms: 80,
            rms_threshold: 0.05,
        }
    }
}

// ── ASR 配置 ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    pub model_dir: String,
    pub sample_rate: i32,
    /// 透传保留，UI 不暴露
    #[serde(default = "default_hotwords_score")]
    pub hotwords_score: f32,
}

fn default_hotwords_score() -> f32 { 1.5 }

// ── 端点检测配置 ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub min_speech_duration_ms: u64,
    pub max_speech_duration_ms: u64,
    pub trailing_silence_ms: u64,
    pub force_timeout_ms: u64,
    pub vad_assisted: bool,
    pub vad_silence_confirm_frames: usize,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            min_speech_duration_ms: 300,
            max_speech_duration_ms: 30000,
            trailing_silence_ms: 1000,
            force_timeout_ms: 60000,
            vad_assisted: true,
            vad_silence_confirm_frames: 8,
        }
    }
}

// ── VInputConfig impl ─────────────────────────────────────────────────────────

impl Default for VInputConfig {
    fn default() -> Self {
        Self {
            basic: BasicConfig::default(),
            hotwords: HotwordsConfig::default(),
            vad: VadConfig::default(),
            asr: AsrConfig {
                model_dir: "/usr/share/droplet-voice-input/models".to_string(),
                sample_rate: 16000,
                hotwords_score: 1.5,
            },
            endpoint: EndpointConfig::default(),
        }
    }
}

impl VInputConfig {
    pub fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("vinput").join("config.toml")
        } else {
            PathBuf::from(".vinput-config.toml")
        }
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();

        if !path.exists() {
            tracing::info!("配置文件不存在，尝试从示例文件创建: {:?}", path);
            let example_path = PathBuf::from("/usr/share/droplet-voice-input/config.toml.example");
            if example_path.exists() {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&example_path, &path)?;
                tracing::info!("配置文件创建成功: {:?}", path);
            } else {
                tracing::info!("示例文件不存在，使用默认配置并保存");
                let default_config = Self::default();
                default_config.save()?;
                return Ok(default_config);
            }
        }

        let content = fs::read_to_string(&path)?;
        let config: VInputConfig = toml::from_str(&content)?;
        tracing::info!("配置加载成功: {:?}", path);
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
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
    fn test_default_config() {
        let config = VInputConfig::default();
        assert_eq!(config.asr.model_dir, "/usr/share/droplet-voice-input/models");
        assert_eq!(config.vad.hysteresis.start_threshold, 0.25);
        assert_eq!(config.vad.hysteresis.end_threshold, 0.08);
        assert_eq!(config.endpoint.trailing_silence_ms, 1000);
        assert_eq!(config.endpoint.vad_silence_confirm_frames, 8);
    }

    #[test]
    fn test_parse_actual_format() {
        // 与实际 ~/.config/vinput/config.toml 格式一致
        let toml_str = r#"
[basic]
hotkey = "RCtrl"

[hotwords]
global_weight = 2.5
max_words = 10000

[hotwords.words]

[vad.silero]
model_path = "/usr/share/droplet-voice-input/models/silero-vad/silero_vad.onnx"
sample_rate = 16000
frame_size = 512

[vad.energy_gate]
enabled = true
noise_multiplier = 2.5
baseline_alpha = 0.95
initial_baseline = 0.001

[vad.hysteresis]
start_threshold = 0.25
end_threshold = 0.08
min_speech_duration_ms = 100
min_silence_duration_ms = 500
max_candidate_gap_frames = 3

[vad.pre_roll]
enabled = true
duration_ms = 750
capacity = 12000

[vad.transient_filter]
enabled = true
max_duration_ms = 80
rms_threshold = 0.05

[asr]
model_dir = "/usr/share/droplet-voice-input/models"
sample_rate = 16000
hotwords_score = 1.5

[endpoint]
min_speech_duration_ms = 300
max_speech_duration_ms = 30000
trailing_silence_ms = 1500
force_timeout_ms = 60000
vad_assisted = true
vad_silence_confirm_frames = 5
"#;
        let config: VInputConfig = toml::from_str(toml_str).expect("parse failed");
        assert_eq!(config.vad.hysteresis.start_threshold, 0.25);
        assert_eq!(config.vad.hysteresis.end_threshold, 0.08);
        assert_eq!(config.vad.hysteresis.min_silence_duration_ms, 500);
        assert_eq!(config.endpoint.trailing_silence_ms, 1500);
        assert_eq!(config.asr.hotwords_score, 1.5);
        assert_eq!(config.hotwords.global_weight, 2.5);
    }

    #[test]
    fn test_roundtrip() {
        let original = VInputConfig::default();
        let toml_str = toml::to_string_pretty(&original).expect("serialize failed");
        let restored: VInputConfig = toml::from_str(&toml_str).expect("deserialize failed");
        assert_eq!(original.vad.hysteresis.start_threshold, restored.vad.hysteresis.start_threshold);
        assert_eq!(original.endpoint.trailing_silence_ms, restored.endpoint.trailing_silence_ms);
    }

    #[test]
    fn test_vad_thresholds_valid() {
        let cfg = VInputConfig::default();
        let h = &cfg.vad.hysteresis;
        assert!(h.start_threshold > 0.0 && h.start_threshold <= 1.0);
        assert!(h.end_threshold > 0.0 && h.end_threshold <= 1.0);
        assert!(h.start_threshold > h.end_threshold);
    }
}
