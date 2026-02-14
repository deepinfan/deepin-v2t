//! 热词引擎模块
//!
//! Hotwords Engine - 提升特定词汇识别准确率

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod engine;
pub mod parser;

// 导出核心类型
pub use engine::HotwordsEngine;
pub use parser::{HotwordEntry, HotwordsParser};

/// 热词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotwordsConfig {
    /// 热词列表 (词汇 → 权重)
    pub words: HashMap<String, f32>,
    /// 全局权重
    pub global_weight: f32,
    /// 最大热词数
    pub max_words: usize,
}

impl Default for HotwordsConfig {
    fn default() -> Self {
        Self {
            words: HashMap::new(),
            global_weight: 2.5,
            max_words: 10000,
        }
    }
}
