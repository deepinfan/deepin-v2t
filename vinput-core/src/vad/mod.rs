//! VAD (Voice Activity Detection) 模块
//!
//! 基于 Silero VAD 的语音活动检测

pub mod silero;

pub use silero::{SileroVAD, SileroVADConfig, VADState};
