//! VAD (Voice Activity Detection) 模块
//!
//! 基于 Silero VAD 的语音活动检测

#[cfg(feature = "vad-onnx")]
pub mod silero;

#[cfg(feature = "vad-onnx")]
pub use silero::{SileroVAD, SileroVADConfig, VADState};
