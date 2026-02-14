//! 热词文件解析器
//!
//! 解析 hotwords.txt 和 hotwords.toml 格式的热词文件

use std::collections::HashMap;
use std::path::Path;
use crate::error::{VInputError, VInputResult};

/// 热词条目
#[derive(Debug, Clone)]
pub struct HotwordEntry {
    /// 词汇
    pub word: String,
    /// 权重 (1.0-5.0)
    pub weight: f32,
}

impl HotwordEntry {
    /// 创建新的热词条目
    pub fn new(word: String, weight: f32) -> Self {
        Self { word, weight }
    }

    /// 验证权重范围
    pub fn validate_weight(weight: f32) -> bool {
        weight >= 1.0 && weight <= 5.0
    }
}

/// 热词文件解析器
pub struct HotwordsParser;

impl HotwordsParser {
    /// 从文本文件解析热词
    ///
    /// 格式：
    /// ```text
    /// # 注释
    /// 词汇 权重
    /// 深度学习 2.8
    /// 人工智能
    /// ```
    pub fn parse_txt(content: &str) -> VInputResult<HashMap<String, f32>> {
        let mut hotwords = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 解析词汇和权重
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            let word = parts[0].to_string();
            let weight = if parts.len() >= 2 {
                parts[1].parse::<f32>().map_err(|_| {
                    VInputError::Hotword(format!(
                        "Invalid weight at line {}: '{}'",
                        line_num + 1,
                        parts[1]
                    ))
                })?
            } else {
                2.5 // 默认权重
            };

            // 验证权重
            if !HotwordEntry::validate_weight(weight) {
                return Err(VInputError::Hotword(format!(
                    "Weight out of range (1.0-5.0) at line {}: {}",
                    line_num + 1,
                    weight
                )));
            }

            hotwords.insert(word, weight);
        }

        Ok(hotwords)
    }

    /// 从 TOML 文件解析热词
    ///
    /// 格式：
    /// ```toml
    /// [default]
    /// 深度学习 = 2.8
    /// 人工智能 = 2.5
    ///
    /// [names]
    /// 张三 = 3.0
    /// ```
    pub fn parse_toml(content: &str) -> VInputResult<HashMap<String, HashMap<String, f32>>> {
        use toml::Value;

        let value: Value = content.parse().map_err(|e| {
            VInputError::Hotword(format!("Failed to parse TOML: {}", e))
        })?;

        let mut groups = HashMap::new();

        if let Some(table) = value.as_table() {
            for (group_name, group_value) in table {
                if let Some(group_table) = group_value.as_table() {
                    let mut hotwords = HashMap::new();

                    for (word, weight_value) in group_table {
                        let weight = if let Some(w) = weight_value.as_float() {
                            w as f32
                        } else if let Some(w) = weight_value.as_integer() {
                            w as f32
                        } else {
                            return Err(VInputError::Hotword(format!(
                                "Invalid weight for '{}' in group '{}'",
                                word, group_name
                            )));
                        };

                        // 验证权重
                        if !HotwordEntry::validate_weight(weight) {
                            return Err(VInputError::Hotword(format!(
                                "Weight out of range (1.0-5.0) for '{}': {}",
                                word, weight
                            )));
                        }

                        hotwords.insert(word.clone(), weight);
                    }

                    groups.insert(group_name.clone(), hotwords);
                }
            }
        }

        Ok(groups)
    }

    /// 加载热词文件（自动检测格式）
    pub fn load_file(path: &Path) -> VInputResult<HashMap<String, f32>> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            VInputError::Hotword(format!("Failed to read hotwords file: {}", e))
        })?;

        // 根据文件扩展名选择解析器
        if let Some(ext) = path.extension() {
            if ext == "toml" {
                // TOML 格式：合并所有分组
                let groups = Self::parse_toml(&content)?;
                let mut merged = HashMap::new();

                for (_, hotwords) in groups {
                    merged.extend(hotwords);
                }

                return Ok(merged);
            }
        }

        // 默认使用 txt 格式
        Self::parse_txt(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_txt_basic() {
        let content = r#"
# 注释
深度学习 2.8
人工智能 2.5
Transformer
        "#;

        let hotwords = HotwordsParser::parse_txt(content).unwrap();

        assert_eq!(hotwords.len(), 3);
        assert_eq!(hotwords.get("深度学习"), Some(&2.8));
        assert_eq!(hotwords.get("人工智能"), Some(&2.5));
        assert_eq!(hotwords.get("Transformer"), Some(&2.5)); // 默认权重
    }

    #[test]
    fn test_parse_txt_empty_lines() {
        let content = r#"

# 注释

深度学习 2.8

        "#;

        let hotwords = HotwordsParser::parse_txt(content).unwrap();
        assert_eq!(hotwords.len(), 1);
    }

    #[test]
    fn test_parse_txt_invalid_weight() {
        let content = "深度学习 invalid";
        assert!(HotwordsParser::parse_txt(content).is_err());
    }

    #[test]
    fn test_parse_txt_weight_out_of_range() {
        let content = "深度学习 6.0";
        assert!(HotwordsParser::parse_txt(content).is_err());

        let content = "深度学习 0.5";
        assert!(HotwordsParser::parse_txt(content).is_err());
    }

    #[test]
    fn test_parse_toml_basic() {
        let content = r#"
[default]
"深度学习" = 2.8
"人工智能" = 2.5

[names]
"张三" = 3.0
"李四" = 3.0
        "#;

        let groups = HotwordsParser::parse_toml(content).unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups["default"]["深度学习"], 2.8);
        assert_eq!(groups["names"]["张三"], 3.0);
    }

    #[test]
    fn test_validate_weight() {
        assert!(HotwordEntry::validate_weight(1.0));
        assert!(HotwordEntry::validate_weight(2.5));
        assert!(HotwordEntry::validate_weight(5.0));
        assert!(!HotwordEntry::validate_weight(0.9));
        assert!(!HotwordEntry::validate_weight(5.1));
    }

    #[test]
    fn test_hotword_entry() {
        let entry = HotwordEntry::new("深度学习".to_string(), 2.8);
        assert_eq!(entry.word, "深度学习");
        assert_eq!(entry.weight, 2.8);
    }
}
