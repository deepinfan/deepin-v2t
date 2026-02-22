//! 标点符号处理模块
//!
//! 使用 CT-Transformer ONNX 模型为 ASR 输出添加标点符号。
//! 需要启用 `vad-onnx` feature 以使用真实模型推理。

#[cfg(feature = "vad-onnx")]
pub mod ct_transformer;

#[cfg(feature = "vad-onnx")]
pub use ct_transformer::CtTransformerPunct;
