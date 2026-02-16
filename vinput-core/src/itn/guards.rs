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

/// ChineseWordGuard - 中文常用词守卫
///
/// 防止常用词（如 "一起"、"一些"）中的数字被误转换
pub struct ChineseWordGuard;

impl ChineseWordGuard {
    /// 不应转换的常用词白名单
    const PROTECTED_WORDS: &'static [&'static str] = &[
        // "一" 字开头的常用词
        "一起", "一些", "一般", "一下", "一样", "一直", "一定",
        "一边", "一共", "一旦", "一致", "一刻", "一切", "一向",
        "一律", "一再", "一度", "一时", "一概", "一并", "一贯",
        "一如", "一经", "一味", "一身", "一番", "一帆", "一路",
        "一开始", "一会儿", "一瞬间", "一辈子", "一方面",
        // 指示词
        "这些", "那些", "哪些", "某些",
        // 其他常用词
        "万一", "统一", "唯一", "单一", "第一",
    ];

    /// 非数字后缀（数字字符后跟这些字符应跳过转换）
    /// 这些后缀表示词汇而非数字单位
    const NON_NUMERIC_SUFFIXES: &'static [char] = &[
        '起', '些', '般', '下', '样', '直', '定', '边', '共',
        '旦', '致', '刻', '切', '向', '律', '再', '度', '时',
        '概', '并', '贯', '如', '经', '味', '身', '番', '帆', '路',
    ];

    /// 检查是否应该跳过数字转换
    ///
    /// 返回 true 表示应该保留原文，不进行数字转换
    pub fn should_skip_conversion(text: &str) -> bool {
        // 策略 1: 检查完整词白名单
        if Self::PROTECTED_WORDS.contains(&text) {
            return true;
        }

        // 策略 2: 检查是否为 "数字字符 + 非数字后缀" 模式
        if Self::is_non_numeric_pattern(text) {
            return true;
        }

        false
    }

    /// 检查是否为非数字模式（如 "一起"、"二般"）
    ///
    /// 规则：两个字符，第一个是数字字符，第二个是非数字后缀
    fn is_non_numeric_pattern(text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() != 2 {
            return false;
        }

        // 第一个字符是中文数字字符
        let is_first_digit = matches!(
            chars[0],
            '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        );

        // 第二个字符是非数字后缀
        let is_second_suffix = Self::NON_NUMERIC_SUFFIXES.contains(&chars[1]);

        is_first_digit && is_second_suffix
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
    fn test_chinese_word_guard_protected_words() {
        // 常用词应该被保护
        assert!(ChineseWordGuard::should_skip_conversion("一起"));
        assert!(ChineseWordGuard::should_skip_conversion("一些"));
        assert!(ChineseWordGuard::should_skip_conversion("一般"));
        assert!(ChineseWordGuard::should_skip_conversion("一下"));
        assert!(ChineseWordGuard::should_skip_conversion("一样"));
        assert!(ChineseWordGuard::should_skip_conversion("一开始"));
        assert!(ChineseWordGuard::should_skip_conversion("一会儿"));
        assert!(ChineseWordGuard::should_skip_conversion("一瞬间"));
        assert!(ChineseWordGuard::should_skip_conversion("这些"));
        assert!(ChineseWordGuard::should_skip_conversion("那些"));

        // 数字表达不应该被保护
        assert!(!ChineseWordGuard::should_skip_conversion("一千"));
        assert!(!ChineseWordGuard::should_skip_conversion("二十"));
        assert!(!ChineseWordGuard::should_skip_conversion("三百"));
        assert!(!ChineseWordGuard::should_skip_conversion("一"));
        assert!(!ChineseWordGuard::should_skip_conversion("十"));
    }

    #[test]
    fn test_chinese_word_guard_non_numeric_pattern() {
        // 数字 + 非数字后缀应该被保护
        assert!(ChineseWordGuard::is_non_numeric_pattern("一起"));
        assert!(ChineseWordGuard::is_non_numeric_pattern("二般"));
        assert!(ChineseWordGuard::is_non_numeric_pattern("三下"));

        // 数字 + 数字单位不应该被保护
        assert!(!ChineseWordGuard::is_non_numeric_pattern("一十"));
        assert!(!ChineseWordGuard::is_non_numeric_pattern("二百"));
        assert!(!ChineseWordGuard::is_non_numeric_pattern("三千"));

        // 单个字符或多于两个字符不匹配模式
        assert!(!ChineseWordGuard::is_non_numeric_pattern("一"));
        assert!(!ChineseWordGuard::is_non_numeric_pattern("一二三"));
    }

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
