//! 语音端点检测模块
//!
//! 提供智能的语音边界检测，结合 VAD 和 ASR 端点检测

mod detector;

pub use detector::{
    EndpointDetector,
    EndpointDetectorConfig,
    EndpointResult,
};
