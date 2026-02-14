//! 热词引擎模块
//!
//! Hotwords Engine - 提升特定词汇识别准确率

pub mod engine;
pub mod parser;

// 导出核心类型
pub use engine::HotwordsEngine;
pub use parser::{HotwordEntry, HotwordsParser};
