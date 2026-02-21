//! VAD 配置模块
//!
//! 定义 VAD 系统的所有配置参数

use serde::{Deserialize, Serialize};

/// VAD 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Silero VAD 配置
    #[serde(default = "default_silero_config")]
    pub silero: SileroConfig,

    /// Energy Gate 配置
    #[serde(default = "default_energy_gate_config")]
    pub energy_gate: EnergyGateConfig,

    /// 迟滞控制器配置
    #[serde(default = "default_hysteresis_config")]
    pub hysteresis: HysteresisConfig,

    /// Pre-roll Buffer 配置
    #[serde(default = "default_pre_roll_config")]
    pub pre_roll: PreRollConfig,

    /// 短爆发过滤器配置
    #[serde(default = "default_transient_filter_config")]
    pub transient_filter: TransientFilterConfig,
}

// 默认值函数
fn default_silero_config() -> SileroConfig {
    SileroConfig {
        model_path: "models/silero-vad/silero_vad.onnx".to_string(),
        sample_rate: 16000,
        frame_size: 512,
    }
}

fn default_energy_gate_config() -> EnergyGateConfig {
    EnergyGateConfig {
        enabled: true,
        noise_multiplier: 2.5,
        baseline_alpha: 0.95,
        initial_baseline: 0.001,
    }
}

fn default_hysteresis_config() -> HysteresisConfig {
    HysteresisConfig {
        start_threshold: 0.7,           // 提高到 0.7（原 0.6）- 减少背景噪音误触发
        end_threshold: 0.35,            // 保持 0.35
        min_speech_duration_ms: 100,    // 保持 100ms
        min_silence_duration_ms: 700,   // 增加到 700ms（原 500ms）- 给最后一个字更多时间
    }
}

fn default_pre_roll_config() -> PreRollConfig {
    PreRollConfig {
        enabled: true,
        duration_ms: 250,
        capacity: 4000,
    }
}

fn default_transient_filter_config() -> TransientFilterConfig {
    TransientFilterConfig {
        enabled: true,
        max_duration_ms: 80,
        rms_threshold: 0.05,
    }
}

/// Silero VAD 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SileroConfig {
    /// 模型文件路径
    pub model_path: String,

    /// 采样率 (Hz)
    pub sample_rate: u32,

    /// 帧大小（样本数）
    pub frame_size: usize,
}

/// Energy Gate 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyGateConfig {
    /// 启用 Energy Gate
    pub enabled: bool,

    /// 噪声基线倍数（RMS > noise_floor × multiplier 才通过）
    pub noise_multiplier: f32,

    /// 噪声基线更新系数（平滑因子）
    pub baseline_alpha: f32,

    /// 初始噪声基线
    pub initial_baseline: f32,
}

/// 迟滞控制器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HysteresisConfig {
    /// 启动阈值（Silence → Speech）
    pub start_threshold: f32,

    /// 结束阈值（Speech → Silence）
    pub end_threshold: f32,

    /// 最小语音持续时间 (ms)
    pub min_speech_duration_ms: u64,

    /// 最小静音持续时间 (ms)
    pub min_silence_duration_ms: u64,
}

/// Pre-roll Buffer 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreRollConfig {
    /// 启用 Pre-roll Buffer
    pub enabled: bool,

    /// Pre-roll 时长 (ms)
    pub duration_ms: u64,

    /// Buffer 容量（样本数）
    pub capacity: usize,
}

/// 短爆发过滤器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientFilterConfig {
    /// 启用过滤器
    pub enabled: bool,

    /// 最大允许的短爆发持续时间 (ms)
    pub max_duration_ms: u64,

    /// RMS 阈值（超过此值视为可能的短爆发）
    pub rms_threshold: f32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self::push_to_talk_default()
    }
}

impl VadConfig {
    /// PushToTalk 模式的默认配置
    pub fn push_to_talk_default() -> Self {
        Self {
            silero: SileroConfig {
                model_path: "models/silero-vad/silero_vad.onnx".to_string(),
                sample_rate: 16000,
                frame_size: 512, // 32ms @ 16kHz
            },
            energy_gate: EnergyGateConfig {
                enabled: true,
                noise_multiplier: 2.5,
                baseline_alpha: 0.95,
                initial_baseline: 0.001,
            },
            hysteresis: HysteresisConfig {
                start_threshold: 0.6,
                end_threshold: 0.35,
                min_speech_duration_ms: 100,
                min_silence_duration_ms: 500,
            },
            pre_roll: PreRollConfig {
                enabled: true,
                duration_ms: 250,
                capacity: 4000, // 250ms @ 16kHz
            },
            transient_filter: TransientFilterConfig {
                enabled: true,
                max_duration_ms: 80,
                rms_threshold: 0.05,
            },
        }
    }
}
