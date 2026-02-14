//! 标点控制系统
//!
//! Punctuation Engine - 基于停顿和规则的标点插入系统
//!
//! 核心组件：
//! - `config`: StyleProfile 配置
//! - `pause_engine`: 停顿检测引擎
//! - `rules`: 规则层（逻辑连接词、问号等）
//! - `engine`: 标点主引擎

pub mod config;
pub mod engine;
pub mod pause_engine;
pub mod rules;

// 导出核心类型
pub use config::StyleProfile;
pub use engine::PunctuationEngine;
pub use pause_engine::TokenInfo;

/// 标点配置（别名）
pub type PunctuationConfig = StyleProfile;
