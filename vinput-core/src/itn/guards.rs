//! Guard 模块
//!
//! ContextGuard 和 ColloquialGuard 实现
//!
//! ContextGuard: 跳过 URL、文件路径、代码片段等
//! ColloquialGuard: 防止口语数量表达误转为金额

use crate::itn::Block;

/// ContextGuard - 上下文守卫
///
/// 检测并跳过不应进行 ITN 的上下文：
/// - URL
/// - 文件路径
/// - CamelCase
/// - snake_case
/// - 全大写单词
/// - 代码片段
pub struct ContextGuard;

impl ContextGuard {
    /// 检查 Block 是否应该跳过 ITN
    ///
    /// 返回 true 表示应该跳过
    pub fn should_skip(block: &Block) -> bool {
        let content = &block.content;

        // URL 检测
        if Self::is_url(content) {
            return true;
        }

        // 文件路径检测
        if Self::is_file_path(content) {
            return true;
        }

        // CamelCase 检测
        if Self::is_camel_case(content) {
            return true;
        }

        // snake_case 检测
        if Self::is_snake_case(content) {
            return true;
        }

        // 全大写单词检测
        if Self::is_all_caps(content) {
            return true;
        }

        false
    }

    /// 检测是否为 URL
    fn is_url(text: &str) -> bool {
        text.starts_with("http://")
            || text.starts_with("https://")
            || text.starts_with("ftp://")
            || text.starts_with("www.")
            || text.contains("://")
    }

    /// 检测是否为文件路径
    fn is_file_path(text: &str) -> bool {
        // Unix 路径
        if text.starts_with('/') || text.starts_with("./") || text.starts_with("../") {
            return true;
        }

        // Windows 路径
        if text.len() >= 3 && text.chars().nth(1) == Some(':') {
            return true;
        }

        // 包含路径分隔符
        if text.contains('/') && (text.matches('/').count() >= 2 || text.contains('.')) {
            return true;
        }

        false
    }

    /// 检测是否为 CamelCase
    fn is_camel_case(text: &str) -> bool {
        if text.len() < 2 {
            return false;
        }

        // 至少包含一个小写字母和一个大写字母
        let has_lower = text.chars().any(|c| c.is_lowercase());
        let has_upper = text.chars().any(|c| c.is_uppercase());

        if !has_lower || !has_upper {
            return false;
        }

        // 大写字母后面紧跟小写字母
        let chars: Vec<char> = text.chars().collect();
        for i in 0..chars.len() - 1 {
            if chars[i].is_uppercase() && chars[i + 1].is_lowercase() {
                return true;
            }
        }

        false
    }

    /// 检测是否为 snake_case
    fn is_snake_case(text: &str) -> bool {
        text.contains('_') && text.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
    }

    /// 检测是否为全大写单词
    fn is_all_caps(text: &str) -> bool {
        if text.len() < 2 {
            return false;
        }

        text.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
    }
}

/// ColloquialGuard - 口语守卫
///
/// 防止口语数量表达误转为金额
pub struct ColloquialGuard;

impl ColloquialGuard {
    /// Currency Keyword 白名单
    const CURRENCY_KEYWORDS: &'static [&'static str] = &[
        "dollar", "dollars", "usd", "euro", "euros", "yuan", "rmb", "yen", "pounds",
        "人民币", "美元", "欧元", "日元", "英镑", "块钱", "元",
    ];

    /// 禁止结构关键词（数量词）
    const FORBIDDEN_QUANTIFIERS: &'static [&'static str] =
        &["个", "的", "块的", "件", "份", "次", "台", "张", "条"];

    /// 检查是否存在货币关键词
    ///
    /// 返回 (是否存在货币关键词, 货币符号)
    pub fn has_currency_keyword(text: &str) -> (bool, Option<&'static str>) {
        let text_lower = text.to_lowercase();

        for &keyword in Self::CURRENCY_KEYWORDS {
            if text_lower.contains(&keyword.to_lowercase()) {
                let symbol = match keyword {
                    "dollar" | "dollars" | "usd" | "美元" => Some("$"),
                    "euro" | "euros" | "欧元" => Some("€"),
                    "yuan" | "rmb" | "人民币" | "元" | "块钱" => Some("¥"),
                    "yen" | "日元" => Some("¥"),
                    "pounds" | "英镑" => Some("£"),
                    _ => None,
                };
                return (true, symbol);
            }
        }

        (false, None)
    }

    /// 检查是否包含禁止结构（数量词）
    ///
    /// 如果数字后出现数量词，则禁止金额转换
    pub fn has_forbidden_quantifier(text: &str) -> bool {
        for &quantifier in Self::FORBIDDEN_QUANTIFIERS {
            if text.contains(quantifier) {
                return true;
            }
        }
        false
    }

    /// 检查是否可以进行金额转换
    ///
    /// 返回 (是否允许, 货币符号)
    pub fn can_convert_to_currency(text: &str) -> (bool, Option<&'static str>) {
        // 必须存在货币关键词
        let (has_currency, symbol) = Self::has_currency_keyword(text);
        if !has_currency {
            return (false, None);
        }

        // 不能包含禁止结构
        if Self::has_forbidden_quantifier(text) {
            return (false, None);
        }

        (true, symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::itn::{BlockType, Tokenizer};

    #[test]
    fn test_context_guard_url() {
        assert!(ContextGuard::is_url("http://example.com"));
        assert!(ContextGuard::is_url("https://github.com"));
        assert!(ContextGuard::is_url("ftp://server.com"));
        assert!(ContextGuard::is_url("www.google.com"));
        assert!(!ContextGuard::is_url("hello world"));
    }

    #[test]
    fn test_context_guard_file_path() {
        assert!(ContextGuard::is_file_path("/usr/bin/bash"));
        assert!(ContextGuard::is_file_path("./config.toml"));
        assert!(ContextGuard::is_file_path("../parent/file.txt"));
        assert!(ContextGuard::is_file_path("C:\\Windows\\System32"));
        assert!(!ContextGuard::is_file_path("hello"));
    }

    #[test]
    fn test_context_guard_camel_case() {
        assert!(ContextGuard::is_camel_case("CamelCase"));
        assert!(ContextGuard::is_camel_case("myVariable"));
        assert!(ContextGuard::is_camel_case("HTTPServer"));
        assert!(!ContextGuard::is_camel_case("lowercase"));
        assert!(!ContextGuard::is_camel_case("UPPERCASE"));
    }

    #[test]
    fn test_context_guard_snake_case() {
        assert!(ContextGuard::is_snake_case("snake_case"));
        assert!(ContextGuard::is_snake_case("my_variable"));
        assert!(ContextGuard::is_snake_case("test_123"));
        assert!(!ContextGuard::is_snake_case("CamelCase"));
        assert!(!ContextGuard::is_snake_case("normal"));
    }

    #[test]
    fn test_context_guard_all_caps() {
        assert!(ContextGuard::is_all_caps("HTTP"));
        assert!(ContextGuard::is_all_caps("API"));
        assert!(ContextGuard::is_all_caps("URL"));
        assert!(!ContextGuard::is_all_caps("Http"));
        assert!(!ContextGuard::is_all_caps("api"));
    }

    #[test]
    fn test_colloquial_guard_currency_keyword() {
        assert!(ColloquialGuard::has_currency_keyword("one hundred dollars").0);
        assert!(ColloquialGuard::has_currency_keyword("三百元").0);
        assert!(ColloquialGuard::has_currency_keyword("五十块钱").0);
        assert!(!ColloquialGuard::has_currency_keyword("一百个").0);
    }

    #[test]
    fn test_colloquial_guard_forbidden_quantifier() {
        assert!(ColloquialGuard::has_forbidden_quantifier("一百个"));
        assert!(ColloquialGuard::has_forbidden_quantifier("五块的"));
        assert!(ColloquialGuard::has_forbidden_quantifier("三件"));
        assert!(!ColloquialGuard::has_forbidden_quantifier("一百元"));
    }

    #[test]
    fn test_colloquial_guard_can_convert() {
        // 允许：有货币关键词，无禁止结构
        assert!(ColloquialGuard::can_convert_to_currency("one hundred dollars").0);
        assert!(ColloquialGuard::can_convert_to_currency("三百元").0);

        // 禁止：无货币关键词
        assert!(!ColloquialGuard::can_convert_to_currency("一百").0);

        // 禁止：有货币关键词，但有禁止结构
        assert!(!ColloquialGuard::can_convert_to_currency("一百个元").0);
        assert!(!ColloquialGuard::can_convert_to_currency("五块的东西").0);
    }

    #[test]
    fn test_context_guard_should_skip() {
        // 测试 CamelCase（英文 block）
        let blocks = Tokenizer::tokenize("CamelCase");
        assert!(ContextGuard::should_skip(&blocks[0]));

        // 测试普通单词（英文 block）- 不应跳过
        let blocks = Tokenizer::tokenize("hello");
        assert!(!ContextGuard::should_skip(&blocks[0]));

        // 测试全大写（英文 block）
        let blocks = Tokenizer::tokenize("HTTP");
        assert!(ContextGuard::should_skip(&blocks[0]));

        // 测试小写单词 - 不应跳过
        let blocks = Tokenizer::tokenize("world");
        assert!(!ContextGuard::should_skip(&blocks[0]));
    }
}
