//! VAD (Voice Activity Detection) 模块
//!
//! 多层次语音活动检测系统
//!
//! ## 架构
//! 1. **Energy Gate** - 第一层过滤：基于能量阈值过滤环境噪声
//! 2. **Silero VAD** - 核心检测：基于深度学习的语音检测
//! 3. **Hysteresis Controller** - 状态管理：双阈值防抖状态机
//! 4. **Pre-roll Buffer** - 音频缓冲：防止语音开始丢失
//! 5. **Transient Filter** - 噪声过滤：过滤键盘敲击等短爆发噪声

// 配置模块（核心，无 feature 依赖）
pub mod config;

// Energy Gate（第一层过滤）
pub mod energy_gate;

// Hysteresis Controller（状态管理）
pub mod hysteresis;

// Pre-roll Buffer（音频缓冲）
pub mod pre_roll_buffer;

// Transient Filter（短爆发过滤）
pub mod transient_filter;

// VAD Manager（统一接口）
pub mod manager;

// Silero VAD（需要 ONNX Runtime）
#[cfg(feature = "vad-onnx")]
pub mod silero;

// 导出核心类型
pub use config::{
    EnergyGateConfig, HysteresisConfig, PreRollConfig, SileroConfig,
    TransientFilterConfig, VadConfig,
};
pub use energy_gate::EnergyGate;
pub use hysteresis::{HysteresisController, VadState};
pub use manager::{VadManager, VadResult};
pub use pre_roll_buffer::PreRollBuffer;
pub use transient_filter::TransientFilter;

#[cfg(feature = "vad-onnx")]
pub use silero::{SileroVAD, SileroVADConfig, VADState};
