//! 热词引擎核心
//!
//! 管理热词列表，提供 sherpa-onnx 集成接口

use crate::error::{VInputError, VInputResult};
use crate::hotwords::parser::HotwordsParser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 热词引擎
pub struct HotwordsEngine {
    /// 热词列表（词汇 → 权重）
    hotwords: HashMap<String, f32>,

    /// 热词文件路径
    file_path: Option<PathBuf>,

    /// 热词总数限制
    max_hotwords: usize,

    /// 全局权重（用于 sherpa-onnx hotwords_score）
    global_weight: f32,
}

impl HotwordsEngine {
    /// 创建新的热词引擎
    pub fn new() -> Self {
        Self {
            hotwords: HashMap::new(),
            file_path: None,
            max_hotwords: 10000,
            global_weight: 2.5,
        }
    }

    /// 从文件加载热词
    pub fn load_from_file(&mut self, path: &Path) -> VInputResult<()> {
        let hotwords = HotwordsParser::load_file(path)?;

        // 限制总数
        if hotwords.len() > self.max_hotwords {
            return Err(VInputError::Hotword(format!(
                "Too many hotwords: {} > {}",
                hotwords.len(),
                self.max_hotwords
            )));
        }

        self.hotwords = hotwords;
        self.file_path = Some(path.to_path_buf());

        Ok(())
    }

    /// 添加单个热词
    pub fn add_hotword(&mut self, word: String, weight: f32) -> VInputResult<()> {
        // 验证权重
        if weight < 1.0 || weight > 5.0 {
            return Err(VInputError::Hotword(format!(
                "Weight out of range (1.0-5.0): {}",
                weight
            )));
        }

        // 检查数量限制
        if self.hotwords.len() >= self.max_hotwords && !self.hotwords.contains_key(&word) {
            return Err(VInputError::Hotword(format!(
                "Max hotwords limit reached: {}",
                self.max_hotwords
            )));
        }

        self.hotwords.insert(word, weight);
        Ok(())
    }

    /// 移除单个热词
    pub fn remove_hotword(&mut self, word: &str) -> bool {
        self.hotwords.remove(word).is_some()
    }

    /// 清空所有热词
    pub fn clear(&mut self) {
        self.hotwords.clear();
    }

    /// 获取 sherpa-onnx 所需的热词字符串
    ///
    /// 格式：每行一个词汇（无权重，权重统一由 global_weight 控制）
    pub fn to_sherpa_format(&self) -> String {
        self.hotwords.keys().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
    }

    /// 获取热词列表
    pub fn get_hotwords(&self) -> &HashMap<String, f32> {
        &self.hotwords
    }

    /// 获取热词数量
    pub fn count(&self) -> usize {
        self.hotwords.len()
    }

    /// 设置全局权重
    pub fn set_global_weight(&mut self, weight: f32) {
        self.global_weight = weight.clamp(1.0, 5.0);
    }

    /// 获取全局权重
    pub fn global_weight(&self) -> f32 {
        self.global_weight
    }

    /// 设置最大热词数
    pub fn set_max_hotwords(&mut self, max: usize) {
        self.max_hotwords = max;
    }
}

impl Default for HotwordsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hotwords_engine_new() {
        let engine = HotwordsEngine::new();
        assert_eq!(engine.count(), 0);
        assert_eq!(engine.global_weight(), 2.5);
    }

    #[test]
    fn test_add_hotword() {
        let mut engine = HotwordsEngine::new();
        engine.add_hotword("深度学习".to_string(), 2.8).unwrap();
        assert_eq!(engine.count(), 1);
    }

    #[test]
    fn test_add_hotword_invalid_weight() {
        let mut engine = HotwordsEngine::new();
        assert!(engine.add_hotword("test".to_string(), 6.0).is_err());
        assert!(engine.add_hotword("test".to_string(), 0.5).is_err());
    }

    #[test]
    fn test_remove_hotword() {
        let mut engine = HotwordsEngine::new();
        engine.add_hotword("test".to_string(), 2.5).unwrap();
        assert_eq!(engine.count(), 1);

        assert!(engine.remove_hotword("test"));
        assert_eq!(engine.count(), 0);

        assert!(!engine.remove_hotword("nonexistent"));
    }

    #[test]
    fn test_clear() {
        let mut engine = HotwordsEngine::new();
        engine.add_hotword("test1".to_string(), 2.5).unwrap();
        engine.add_hotword("test2".to_string(), 2.5).unwrap();
        assert_eq!(engine.count(), 2);

        engine.clear();
        assert_eq!(engine.count(), 0);
    }

    #[test]
    fn test_to_sherpa_format() {
        let mut engine = HotwordsEngine::new();
        engine.add_hotword("深度学习".to_string(), 2.8).unwrap();
        engine.add_hotword("人工智能".to_string(), 2.5).unwrap();

        let output = engine.to_sherpa_format();
        assert!(output.contains("深度学习"));
        assert!(output.contains("人工智能"));
    }

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Test hotwords").unwrap();
        writeln!(file, "深度学习 2.8").unwrap();
        writeln!(file, "人工智能 2.5").unwrap();
        file.flush().unwrap();

        let mut engine = HotwordsEngine::new();
        engine.load_from_file(file.path()).unwrap();

        assert_eq!(engine.count(), 2);
        assert_eq!(engine.get_hotwords()["深度学习"], 2.8);
    }

    #[test]
    fn test_max_hotwords_limit() {
        let mut engine = HotwordsEngine::new();
        engine.set_max_hotwords(2);

        engine.add_hotword("word1".to_string(), 2.5).unwrap();
        engine.add_hotword("word2".to_string(), 2.5).unwrap();

        // Should fail - limit reached
        assert!(engine.add_hotword("word3".to_string(), 2.5).is_err());
    }

    #[test]
    fn test_global_weight() {
        let mut engine = HotwordsEngine::new();
        assert_eq!(engine.global_weight(), 2.5);

        engine.set_global_weight(3.0);
        assert_eq!(engine.global_weight(), 3.0);

        // Test clamping
        engine.set_global_weight(10.0);
        assert_eq!(engine.global_weight(), 5.0);
    }
}
