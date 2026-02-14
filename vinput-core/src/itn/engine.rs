//! ITN Engine - 主管道
//!
//! 集成所有 ITN 模块的主引擎

use std::ops::Range;

use crate::itn::{
    Block, BlockType, ChineseNumberConverter, EnglishNumberParser, Tokenizer,
};
use crate::itn::guards::{ColloquialGuard, ContextGuard};
use crate::itn::rules::{CurrencyRule, DateRule, PercentageRule, UnitRule};

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

        // Step 2: 先替换所有中文数字序列
        // 这样即使是混合文本（如 "我有一千块钱"）也能正确转换数字部分
        content = Self::replace_chinese_numbers(&content);

        // Step 4: ColloquialGuard + CurrencyRule - 金额转换
        if self.mode == ITNMode::Auto {
            // 注意：现在 content 中的数字已经是阿拉伯数字了（如 "我有1000块钱"）
            // 所以 CurrencyRule 不应该再在前面加符号，而是应该跳过
            // 暂时禁用 CurrencyRule，因为它会错误地在整个句子前加 ¥
            // TODO: 重新设计 CurrencyRule 来处理已转换的数字

            // Step 5: PercentageRule - 百分比转换
            if content.starts_with("百分之") {
                if let Ok(converted) = PercentageRule::convert_chinese(&content) {
                    content = converted;
                }
            }

            // Step 6: DateRule - 日期转换
            if DateRule::is_date_expression(&content) {
                if let Ok(converted) = DateRule::convert_chinese(&content) {
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

                // 尝试转换
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
}

