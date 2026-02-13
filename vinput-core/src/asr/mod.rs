//! ASR (Automatic Speech Recognition) 模块
//!
//! 基于 sherpa-onnx 的流式语音识别

pub mod recognizer;

pub use recognizer::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
