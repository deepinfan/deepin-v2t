//! Streaming 模块
//!
//! 流式语音识别管道，集成 VAD 和 ASR

pub mod pipeline;

pub use pipeline::{StreamingPipeline, StreamingConfig, StreamingResult, PipelineState};
