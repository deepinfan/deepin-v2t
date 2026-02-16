//! ITN Engine - 主管道
//!
//! 集成所有 ITN 模块的主引擎

use std::ops::Range;

use crate::itn::{
    Block, BlockType, ChineseNumberConverter, EnglishNumberParser, Tokenizer,
};
use crate::itn::guards::{ChineseWordGuard, ContextGuard};
use crate::itn::rules::{DateRule, PercentageRule, UnitRule};

/// ITN 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ITNMode {
    /// 自动模式 - 启用全部规则
    Auto,
    /// 仅数字模式 - 仅执行数字转换
    NumbersOnly,
    /// 原始模式 - 跳过全部 ITN
    Raw,
}

/// ITN 变更记录（用于回滚）
#[derive(Debug, Clone)]
pub struct ITNChange {
    /// 原始文本范围
    pub original_span: Range<usize>,
    /// 原始文本
    pub original_text: String,
    /// 规范化后的文本
    pub normalized_text: String,
}

/// ITN 处理结果
#[derive(Debug, Clone)]
pub struct ITNResult {
    /// 规范化后的文本
    pub text: String,
    /// 变更记录列表
    pub changes: Vec<ITNChange>,
}

/// ITN 引擎
pub struct ITNEngine {
    mode: ITNMode,
}

impl ITNEngine {
    /// 创建新的 ITN 引擎
    pub fn new(mode: ITNMode) -> Self {
        Self { mode }
    }

    /// 处理文本
    pub fn process(&self, text: &str) -> ITNResult {
        // 原始模式：直接返回
        if self.mode == ITNMode::Raw {
            return ITNResult {
                text: text.to_string(),
                changes: Vec::new(),
            };
        }

        // Step 1: Tokenizer - 分割成 Block
        let blocks = Tokenizer::tokenize(text);

        // Step 2-10: 处理每个 block
        let mut processed_blocks = Vec::new();
        let mut changes = Vec::new();
        let mut current_offset = 0;

        for block in blocks {
            let processed = self.process_block(&block, current_offset);

            // 记录变更
            if processed.content != block.content {
                changes.push(ITNChange {
                    original_span: current_offset..(current_offset + block.content.len()),
                    original_text: block.content.clone(),
                    normalized_text: processed.content.clone(),
                });
            }

            current_offset += block.content.len();
            processed_blocks.push(processed);
        }

        // Step 11: MergeEngine - 合并结果
        let merged_text = Self::merge_blocks(&processed_blocks);

        ITNResult {
            text: merged_text,
            changes,
        }
    }

    /// 处理单个 Block
    fn process_block(&self, block: &Block, offset: usize) -> Block {
        // Step 3: ContextGuard - 跳过特定上下文
        if ContextGuard::should_skip(block) {
            return block.clone();
        }

        match block.block_type {
            BlockType::Chinese => self.process_chinese_block(block),
            BlockType::English => self.process_english_block(block),
            BlockType::Number => block.clone(), // 已经是数字，不需要转换
            BlockType::Symbol => block.clone(),  // 符号不转换
        }
    }

    /// 处理中文 Block
    fn process_chinese_block(&self, block: &Block) -> Block {
        let mut content = block.content.clone();

        // Step 2: DateRule - 日期转换（必须在数字转换之前！）
        // 因为年份需要逐位转换，不能被 replace_chinese_numbers 处理
        if self.mode == ITNMode::Auto {
            if DateRule::is_date_expression(&content) {
                if let Ok(converted) = DateRule::convert_chinese(&content) {
                    content = converted;
                }
            }
        }

        // Step 3: 替换所有中文数字序列
        // 这样即使是混合文本（如 "我有一千块钱"）也能正确转换数字部分
        content = Self::replace_chinese_numbers(&content);

        // Step 4: 应用货币规则（处理 "数字+块钱/元" 模式）
        if self.mode == ITNMode::Auto {
            content = Self::apply_currency_rules(&content);
        }

        // Step 5: PercentageRule - 百分比转换
        if self.mode == ITNMode::Auto {
            if content.starts_with("百分之") {
                if let Ok(converted) = PercentageRule::convert_chinese(&content) {
                    content = converted;
                }
            }
        }

        Block {
            content,
            block_type: block.block_type,
            span: block.span.clone(),
        }
    }

    /// 应用货币规则（处理已转换的数字）
    fn apply_currency_rules(text: &str) -> String {
        let mut result = text.to_string();

        // 处理 "数字+块钱/块/元" 模式
        // 注意：此时数字已经是阿拉伯数字了（如 "300块钱"）
        const PATTERNS: &[(&str, &str)] = &[
            ("块钱", "¥"),
            ("块", "¥"),
            ("元", "¥"),
            ("人民币", "¥"),
            ("美元", "$"),
            ("美金", "$"),
            ("刀", "$"),
            ("欧元", "€"),
            ("英镑", "£"),
        ];

        for (keyword, symbol) in PATTERNS {
            // 匹配 "数字+关键词" 模式
            // 例如: "300块钱" -> "¥300"
            let pattern_str = format!(r"(\d+(?:\.\d+)?){}", regex::escape(keyword));
            if let Ok(re) = regex::Regex::new(&pattern_str) {
                result = re.replace_all(&result, |caps: &regex::Captures<'_>| {
                    let number = &caps[1];
                    format!("{}{}", symbol, number)
                }).to_string();
            }
        }

        result
    }

    /// 替换文本中的所有中文数字序列
    fn replace_chinese_numbers(text: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // 检查当前字符是否为中文数字字符
            if Self::is_chinese_number_char(chars[i]) {
                // 收集连续的中文数字字符
                let start = i;
                while i < chars.len() && Self::is_chinese_number_char(chars[i]) {
                    i += 1;
                }

                // 提取中文数字序列
                let number_text: String = chars[start..i].iter().collect();

                // ✅ 守卫检查 0：检查前文是否有特殊前缀（如 "百分之"、"第"）
                // 如果有，应该由专门的规则处理，跳过数字转换
                let has_special_prefix = start >= 3 && {
                    let prefix: String = chars[(start.saturating_sub(3))..start].iter().collect();
                    prefix == "百分之" || prefix == "第" || prefix.ends_with("第")
                };

                if has_special_prefix {
                    result.push_str(&number_text);
                    continue;
                }

                // ✅ 守卫检查 0.5：检查前面是否有普通汉字（backward checking）
                // 只对单字符数字序列检查（如 "统一"、"归一"、"真二"）
                // 多字符数字序列（如 "一千"、"一百"）通常是数量表达，应该转换
                if number_text.chars().count() == 1 && start > 0 {
                    let prev_char = chars[start - 1];
                    // 如果前一个字符是汉字（非数字、非标点、非空格）
                    if Self::is_ordinary_chinese_char(prev_char) {
                        result.push_str(&number_text);
                        continue;
                    }
                }

                // ✅ 守卫检查 1：检查是否为完整的常用词（如 "这些"、"那些"）
                if ChineseWordGuard::should_skip_conversion(&number_text) {
                    result.push_str(&number_text);
                    continue;
                }

                // ✅ 守卫检查 1.5：检查是否为年份格式（4个基础数字，如 "二零二六"）
                // 这种格式应该逐位转换，不应该按数值计算
                if Self::is_year_format(&number_text) {
                    // 逐位转换
                    let year_digits = Self::convert_year_digits(&number_text);
                    result.push_str(&year_digits);
                    continue;
                }

                // ✅ 守卫检查 2：向前看最多3个字符，检查是否形成常用词
                // 例如："一" + "开始" = "一开始"，"一" + "会儿" = "一会儿"
                let mut found_protected_word = false;
                for lookahead in 1..=3 {
                    if i + lookahead <= chars.len() {
                        let next_chars: String = chars[i..(i + lookahead)].iter().collect();
                        let potential_word = format!("{}{}", number_text, next_chars);

                        if ChineseWordGuard::should_skip_conversion(&potential_word) {
                            // 这是常用词，保留原文，并跳过已处理的字符
                            result.push_str(&potential_word);
                            i += lookahead;  // 跳过已处理的字符
                            found_protected_word = true;
                            break;
                        }
                    }
                }

                if found_protected_word {
                    continue;
                }

                // 尝试转换为数字
                if let Ok(converted) = ChineseNumberConverter::convert(&number_text) {
                    result.push_str(&converted);
                } else {
                    // 转换失败，保留原文
                    result.push_str(&number_text);
                }
            } else {
                // 非数字字符，直接添加
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// 检查是否为中文数字字符
    fn is_chinese_number_char(ch: char) -> bool {
        matches!(ch, '零'|'一'|'二'|'三'|'四'|'五'|'六'|'七'|'八'|'九'|'十'|'百'|'千'|'万'|'亿'|'点'|'负')
    }

    /// 检查是否为普通汉字（非数字、非标点、非空格）
    fn is_ordinary_chinese_char(ch: char) -> bool {
        // 排除数字字符
        if Self::is_chinese_number_char(ch) {
            return false;
        }

        // 排除常见标点和符号
        if matches!(ch, '，' | '。' | '！' | '？' | '、' | '；' | '：' | '"' | '"' | '\'' | '（' | '）' | '【' | '】' | '《' | '》') {
            return false;
        }

        // 排除空格和英文字符
        if ch.is_whitespace() || ch.is_ascii() {
            return false;
        }

        // 检查是否为 CJK 统一汉字范围
        matches!(ch, '\u{4E00}'..='\u{9FFF}')
    }

    /// 检查是否为年份格式（4个基础数字，如 "二零二六"）
    fn is_year_format(text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() != 4 {
            return false;
        }

        // 所有字符都是基础数字（0-9）
        chars.iter().all(|&ch| matches!(ch, '零'|'一'|'二'|'三'|'四'|'五'|'六'|'七'|'八'|'九'))
    }

    /// 转换年份数字（逐位转换，用于 "二零二六" 格式）
    fn convert_year_digits(text: &str) -> String {
        text.chars()
            .filter_map(|ch| match ch {
                '零' => Some('0'),
                '一' => Some('1'),
                '二' => Some('2'),
                '三' => Some('3'),
                '四' => Some('4'),
                '五' => Some('5'),
                '六' => Some('6'),
                '七' => Some('7'),
                '八' => Some('8'),
                '九' => Some('9'),
                _ => None,
            })
            .collect()
    }

    /// 处理英文 Block
    fn process_english_block(&self, block: &Block) -> Block {
        let mut content = block.content.clone();

        // Step 3: EnglishNumberParser - 转换英文数字
        if EnglishNumberParser::is_english_number(&content) {
            if let Ok(converted) = EnglishNumberParser::convert(&content) {
                content = converted;
            }
        }

        // Step 7: UnitRule - 单位转换（仅在 Auto 模式）
        if self.mode == ITNMode::Auto {
            // 检查是否包含单位
            let words: Vec<&str> = content.split_whitespace().collect();
            if words.len() == 2 {
                if UnitRule::is_supported_unit(words[1]) {
                    content = UnitRule::format(words[0], words[1]);
                }
            }
        }

        Block {
            content,
            block_type: block.block_type,
            span: block.span.clone(),
        }
    }

    /// 合并 Block 列表为文本
    fn merge_blocks(blocks: &[Block]) -> String {
        blocks.iter().map(|b| &b.content).cloned().collect()
    }

    /// 设置模式
    pub fn set_mode(&mut self, mode: ITNMode) {
        self.mode = mode;
    }

    /// 获取当前模式
    pub fn mode(&self) -> ITNMode {
        self.mode
    }

    /// 回滚 ITN 结果
    ///
    /// 将规范化的文本回滚到原始文本
    pub fn rollback(result: &ITNResult) -> String {
        if result.changes.is_empty() {
            return result.text.clone();
        }

        let mut text = result.text.clone();

        // 从后往前回滚（避免偏移量问题）
        for change in result.changes.iter().rev() {
            // 注意：这是简化实现
            // 完整实现需要处理偏移量的变化
            text = text.replace(&change.normalized_text, &change.original_text);
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_mode_no_conversion() {
        let engine = ITNEngine::new(ITNMode::Raw);
        let result = engine.process("一千二百三十四");

        assert_eq!(result.text, "一千二百三十四");
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_chinese_number_conversion() {
        let engine = ITNEngine::new(ITNMode::Auto);
        let result = engine.process("一千二百三十四");

        assert_eq!(result.text, "1234");
        assert_eq!(result.changes.len(), 1);
    }

    #[test]
    fn test_english_number_conversion() {
        // Note: Multi-word English numbers like "one thousand two hundred thirty four"
        // are split by the tokenizer into separate blocks due to spaces.
        // For full multi-word support, the engine would need to look ahead and merge
        // consecutive English + Symbol + English blocks before conversion.
        // For MVP, we test with single-word numbers:

        let engine = ITNEngine::new(ITNMode::Auto);
        let result = engine.process("one");
        assert_eq!(result.text, "1");

        let result = engine.process("twenty");
        assert_eq!(result.text, "20");
    }

    #[test]
    fn test_percentage_conversion() {
        let engine = ITNEngine::new(ITNMode::Auto);
        let result = engine.process("百分之五十");

        assert_eq!(result.text, "50%");
    }

    #[test]
    fn test_context_guard_skip_url() {
        let engine = ITNEngine::new(ITNMode::Auto);
        let result = engine.process("http://example.com");

        // URL 应该被跳过，保持原样
        assert_eq!(result.text, "http://example.com");
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_context_guard_skip_camelcase() {
        let engine = ITNEngine::new(ITNMode::Auto);
        let result = engine.process("CamelCase");

        // CamelCase 应该被跳过
        assert_eq!(result.text, "CamelCase");
        assert_eq!(result.changes.len(), 0);
    }

    #[test]
    fn test_rollback() {
        let engine = ITNEngine::new(ITNMode::Auto);
        let result = engine.process("一千二百三十四");

        assert_eq!(result.text, "1234");

        let rolled_back = ITNEngine::rollback(&result);
        assert_eq!(rolled_back, "一千二百三十四");
    }

    #[test]
    fn test_mode_switching() {
        let mut engine = ITNEngine::new(ITNMode::Auto);
        let result1 = engine.process("一千");

        assert_eq!(result1.text, "1000");

        engine.set_mode(ITNMode::Raw);
        let result2 = engine.process("一千");

        assert_eq!(result2.text, "一千");
    }

    #[test]
    fn test_numbers_only_mode() {
        let engine = ITNEngine::new(ITNMode::NumbersOnly);
        let result = engine.process("一千二百三十四");

        // NumbersOnly 模式应该转换数字
        assert_eq!(result.text, "1234");
    }

    #[test]
    fn test_protected_common_words() {
        let engine = ITNEngine::new(ITNMode::Auto);

        // 应该保护的常用词（不转换）
        assert_eq!(engine.process("一起").text, "一起");
        assert_eq!(engine.process("一些").text, "一些");
        assert_eq!(engine.process("一般").text, "一般");
        assert_eq!(engine.process("一下").text, "一下");
        assert_eq!(engine.process("一样").text, "一样");
        assert_eq!(engine.process("一直").text, "一直");
        assert_eq!(engine.process("这些").text, "这些");
        assert_eq!(engine.process("那些").text, "那些");

        // 应该转换的数字表达
        assert_eq!(engine.process("一千").text, "1000");
        assert_eq!(engine.process("二十").text, "20");
        assert_eq!(engine.process("三百").text, "300");
        assert_eq!(engine.process("一万").text, "10000");
    }

    #[test]
    fn test_mixed_common_words_and_numbers() {
        let engine = ITNEngine::new(ITNMode::Auto);

        // 句子中混合常用词和数字
        assert_eq!(
            engine.process("我们一起去了一千个地方").text,
            "我们一起去了1000个地方"
        );

        assert_eq!(
            engine.process("一般情况下有二十个").text,
            "一般情况下有20个"
        );

        assert_eq!(
            engine.process("一下子就来了三百人").text,
            "一下子就来了300人"
        );

        assert_eq!(
            engine.process("这些东西一共五十块").text,
            "这些东西一共¥50"  // 货币规则：块 → ¥
        );
    }
}

