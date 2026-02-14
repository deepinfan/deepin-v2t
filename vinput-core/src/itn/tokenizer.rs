//! Tokenizer - ITN 分段器
//!
//! 将输入文本分为不同类型的 Block，为后续处理做准备

use std::ops::Range;

/// Block 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// 中文字符块
    Chinese,
    /// 英文字符块
    English,
    /// 符号块
    Symbol,
    /// 数字块
    Number,
}

/// 文本 Block
#[derive(Debug, Clone)]
pub struct Block {
    /// Block 类型
    pub block_type: BlockType,
    /// 在原始文本中的范围
    pub span: Range<usize>,
    /// Block 内容
    pub content: String,
}

impl Block {
    /// 创建新的 Block
    pub fn new(block_type: BlockType, span: Range<usize>, content: String) -> Self {
        Self {
            block_type,
            span,
            content,
        }
    }

    /// 判断是否为中文块
    pub fn is_chinese(&self) -> bool {
        self.block_type == BlockType::Chinese
    }

    /// 判断是否为英文块
    pub fn is_english(&self) -> bool {
        self.block_type == BlockType::English
    }

    /// 判断是否为数字块
    pub fn is_number(&self) -> bool {
        self.block_type == BlockType::Number
    }

    /// 判断是否为符号块
    pub fn is_symbol(&self) -> bool {
        self.block_type == BlockType::Symbol
    }
}

/// Tokenizer - 文本分段器
pub struct Tokenizer;

impl Tokenizer {
    /// 将文本分段为 Block
    ///
    /// 规则：
    /// - 不跨 Block 转换
    /// - 不修改 Block 内部字符顺序
    /// - 不跨标点处理
    pub fn tokenize(text: &str) -> Vec<Block> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut blocks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            let char_type = Self::classify_char(ch);

            // 找到相同类型的连续字符
            let start = i;
            while i < chars.len() && Self::classify_char(chars[i]) == char_type {
                i += 1;
            }

            // 计算字节范围
            let start_byte = text.char_indices().nth(start).map(|(pos, _)| pos).unwrap_or(0);
            let end_byte = if i < chars.len() {
                text.char_indices().nth(i).map(|(pos, _)| pos).unwrap_or(text.len())
            } else {
                text.len()
            };

            let content = text[start_byte..end_byte].to_string();

            blocks.push(Block::new(
                char_type,
                start_byte..end_byte,
                content,
            ));
        }

        blocks
    }

    /// 对字符进行分类
    fn classify_char(ch: char) -> BlockType {
        // 数字（ASCII 数字）
        if ch.is_ascii_digit() {
            return BlockType::Number;
        }

        // 英文（ASCII 字母）
        if ch.is_ascii_alphabetic() {
            return BlockType::English;
        }

        // 中文（CJK 统一表意文字）
        // Unicode 范围: U+4E00 ~ U+9FFF
        if (ch >= '\u{4E00}' && ch <= '\u{9FFF}') ||
           // 中文数字字符
           matches!(ch, '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
                       '十' | '百' | '千' | '万' | '亿' | '点' | '负') {
            return BlockType::Chinese;
        }

        // 其他符号
        BlockType::Symbol
    }

    /// 检查字符是否为标点符号
    pub fn is_punctuation(ch: char) -> bool {
        matches!(ch,
            ',' | '.' | '!' | '?' | ';' | ':' | '\'' | '"' |
            '，' | '。' | '！' | '？' | '；' | '：' | '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}' |
            '(' | ')' | '[' | ']' | '{' | '}' |
            '（' | '）' | '【' | '】' | '《' | '》'
        )
    }

    /// 在标点处分割 Block 列表
    ///
    /// 规则：不跨标点处理
    pub fn split_by_punctuation(blocks: Vec<Block>) -> Vec<Vec<Block>> {
        let mut segments = Vec::new();
        let mut current_segment = Vec::new();

        for block in blocks {
            if block.is_symbol() && block.content.chars().any(Self::is_punctuation) {
                // 遇到标点，结束当前段
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                    current_segment = Vec::new();
                }
                // 标点本身作为独立段
                segments.push(vec![block]);
            } else {
                current_segment.push(block);
            }
        }

        // 添加最后一段
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_char() {
        assert_eq!(Tokenizer::classify_char('a'), BlockType::English);
        assert_eq!(Tokenizer::classify_char('Z'), BlockType::English);
        assert_eq!(Tokenizer::classify_char('5'), BlockType::Number);
        assert_eq!(Tokenizer::classify_char('中'), BlockType::Chinese);
        assert_eq!(Tokenizer::classify_char('一'), BlockType::Chinese);
        assert_eq!(Tokenizer::classify_char(','), BlockType::Symbol);
        assert_eq!(Tokenizer::classify_char('，'), BlockType::Symbol);
    }

    #[test]
    fn test_tokenize_simple() {
        let blocks = Tokenizer::tokenize("hello123中文");

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, BlockType::English);
        assert_eq!(blocks[0].content, "hello");
        assert_eq!(blocks[1].block_type, BlockType::Number);
        assert_eq!(blocks[1].content, "123");
        assert_eq!(blocks[2].block_type, BlockType::Chinese);
        assert_eq!(blocks[2].content, "中文");
    }

    #[test]
    fn test_tokenize_chinese_number() {
        let blocks = Tokenizer::tokenize("一千二百三十四");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::Chinese);
        assert_eq!(blocks[0].content, "一千二百三十四");
    }

    #[test]
    fn test_tokenize_with_symbols() {
        let blocks = Tokenizer::tokenize("hello,world");

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].content, "hello");
        assert_eq!(blocks[1].content, ",");
        assert_eq!(blocks[2].content, "world");
    }

    #[test]
    fn test_split_by_punctuation() {
        let blocks = Tokenizer::tokenize("hello,world.test");
        let segments = Tokenizer::split_by_punctuation(blocks);

        assert_eq!(segments.len(), 5); // hello, [,], world, [.], test
        assert_eq!(segments[0][0].content, "hello");
        assert_eq!(segments[1][0].content, ",");
        assert_eq!(segments[2][0].content, "world");
        assert_eq!(segments[3][0].content, ".");
        assert_eq!(segments[4][0].content, "test");
    }

    #[test]
    fn test_is_punctuation() {
        assert!(Tokenizer::is_punctuation(','));
        assert!(Tokenizer::is_punctuation('。'));
        assert!(Tokenizer::is_punctuation('！'));
        assert!(!Tokenizer::is_punctuation('a'));
        assert!(!Tokenizer::is_punctuation('中'));
    }

    #[test]
    fn test_block_methods() {
        let block = Block::new(BlockType::Chinese, 0..3, "中文".to_string());
        assert!(block.is_chinese());
        assert!(!block.is_english());
        assert!(!block.is_number());
        assert!(!block.is_symbol());
    }
}
