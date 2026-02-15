//! 中文数字转换模块
//!
//! 将中文数字表达转换为阿拉伯数字
//!
//! 支持的字符集：零一二三四五六七八九十百千万亿点负

use crate::error::{VInputError, VInputResult};

/// 中文数字转换器
pub struct ChineseNumberConverter;

impl ChineseNumberConverter {
    /// 将中文数字字符串转换为阿拉伯数字
    ///
    /// # 参数
    /// - `text`: 中文数字文本（例如："一千二百三十四"）
    ///
    /// # 返回
    /// - `Ok(String)`: 转换后的数字字符串（例如："1234"）
    /// - `Err`: 如果不是有效的中文数字表达
    ///
    /// # 示例
    /// ```
    /// # use vinput_core::itn::chinese_number::ChineseNumberConverter;
    /// let result = ChineseNumberConverter::convert("一千二百三十四").unwrap();
    /// assert_eq!(result, "1234");
    /// ```
    pub fn convert(text: &str) -> VInputResult<String> {
        if text.is_empty() {
            return Ok(String::new());
        }

        // 检查是否包含负号
        let (is_negative, text) = if text.starts_with('负') {
            (true, &text[3..]) // "负" 是 3 字节
        } else {
            (false, text)
        };

        // 检查是否包含小数点
        // 注意：只有当 "点" 前后都有数字字符时才视为小数点
        if let Some(dot_pos) = text.find('点') {
            let integer_part = &text[..dot_pos];
            let decimal_part = &text[dot_pos + 3..]; // "点" 是 3 字节

            // 检查 "点" 前面是否有数字字符
            let has_digit_before = integer_part.chars().any(|c| matches!(c,
                '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
                '十' | '百' | '千' | '万' | '亿'
            ));

            // 检查 "点" 后面是否有数字字符
            let has_digit_after = !decimal_part.is_empty() && decimal_part.chars().next().map_or(false, |c| matches!(c,
                '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
            ));

            // 只有当 "点" 前后都有数字字符时才作为小数点处理
            if has_digit_before && has_digit_after {
                let integer = if integer_part.is_empty() {
                    0
                } else {
                    Self::parse_integer(integer_part)?
                };

                let decimal = Self::parse_decimal(decimal_part)?;

                let result = if is_negative {
                    format!("-{}.{}", integer, decimal)
                } else {
                    format!("{}.{}", integer, decimal)
                };

                return Ok(result);
            }
            // 如果 "点" 前后没有数字字符，则不是小数点，继续作为整数处理
        }

        // 纯整数
        let integer = Self::parse_integer(text)?;
        let result = if is_negative {
            format!("-{}", integer)
        } else {
            format!("{}", integer)
        };

        Ok(result)
    }

    /// 解析整数部分
    fn parse_integer(text: &str) -> VInputResult<i64> {
        if text.is_empty() {
            return Ok(0);
        }

        // 特殊情况：单个"零"
        if text == "零" {
            return Ok(0);
        }

        let mut result = 0i64;
        let mut current = 0i64;
        let mut last_unit = 0u32; // 记录上一个单位的大小

        for ch in text.chars() {
            match ch {
                // 数字 0-9
                '零' => {
                    // 零在中间起占位作用，不累加
                    continue;
                }
                '一' => current = 1,
                '二' => current = 2,
                '三' => current = 3,
                '四' => current = 4,
                '五' => current = 5,
                '六' => current = 6,
                '七' => current = 7,
                '八' => current = 8,
                '九' => current = 9,

                // 单位：十、百、千
                '十' => {
                    if current == 0 {
                        current = 1; // "十" 表示 "一十"
                    }
                    current *= 10;
                    result += current;
                    current = 0;
                    last_unit = 10;
                }
                '百' => {
                    if current == 0 {
                        return Err(VInputError::ItnConversion(
                            "Invalid expression: 百 without number".to_string(),
                        ));
                    }
                    current *= 100;
                    result += current;
                    current = 0;
                    last_unit = 100;
                }
                '千' => {
                    if current == 0 {
                        return Err(VInputError::ItnConversion(
                            "Invalid expression: 千 without number".to_string(),
                        ));
                    }
                    current *= 1000;
                    result += current;
                    current = 0;
                    last_unit = 1000;
                }

                // 大单位：万、亿
                '万' => {
                    if current > 0 {
                        result += current;
                    }
                    result *= 10000;
                    current = 0;
                    last_unit = 10000;
                }
                '亿' => {
                    if current > 0 {
                        result += current;
                    }
                    result *= 100000000;
                    current = 0;
                    last_unit = 100000000;
                }

                _ => {
                    return Err(VInputError::ItnConversion(format!(
                        "Invalid character in Chinese number: {}",
                        ch
                    )));
                }
            }
        }

        // 添加剩余的 current
        result += current;

        Ok(result)
    }

    /// 解析小数部分
    fn parse_decimal(text: &str) -> VInputResult<String> {
        let mut result = String::new();

        for ch in text.chars() {
            let digit = match ch {
                '零' => '0',
                '一' => '1',
                '二' => '2',
                '三' => '3',
                '四' => '4',
                '五' => '5',
                '六' => '6',
                '七' => '7',
                '八' => '8',
                '九' => '9',
                _ => {
                    return Err(VInputError::ItnConversion(format!(
                        "Invalid character in decimal part: {}",
                        ch
                    )));
                }
            };
            result.push(digit);
        }

        Ok(result)
    }

    /// 检查文本是否为有效的中文数字
    pub fn is_chinese_number(text: &str) -> bool {
        if text.is_empty() {
            return false;
        }

        text.chars().all(|ch| {
            matches!(
                ch,
                '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
                '十' | '百' | '千' | '万' | '亿' | '点' | '负'
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_digit() {
        assert_eq!(ChineseNumberConverter::convert("零").unwrap(), "0");
        assert_eq!(ChineseNumberConverter::convert("一").unwrap(), "1");
        assert_eq!(ChineseNumberConverter::convert("九").unwrap(), "9");
    }

    #[test]
    fn test_tens() {
        assert_eq!(ChineseNumberConverter::convert("十").unwrap(), "10");
        assert_eq!(ChineseNumberConverter::convert("一十").unwrap(), "10");
        assert_eq!(ChineseNumberConverter::convert("二十").unwrap(), "20");
        assert_eq!(ChineseNumberConverter::convert("九十九").unwrap(), "99");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(ChineseNumberConverter::convert("一百").unwrap(), "100");
        assert_eq!(ChineseNumberConverter::convert("五百").unwrap(), "500");
        assert_eq!(ChineseNumberConverter::convert("九百九十九").unwrap(), "999");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(ChineseNumberConverter::convert("一千").unwrap(), "1000");
        assert_eq!(
            ChineseNumberConverter::convert("一千二百三十四").unwrap(),
            "1234"
        );
        assert_eq!(
            ChineseNumberConverter::convert("九千九百九十九").unwrap(),
            "9999"
        );
    }

    #[test]
    fn test_ten_thousands() {
        assert_eq!(ChineseNumberConverter::convert("一万").unwrap(), "10000");
        assert_eq!(
            ChineseNumberConverter::convert("十万").unwrap(),
            "100000"
        );
        assert_eq!(
            ChineseNumberConverter::convert("一百万").unwrap(),
            "1000000"
        );
    }

    #[test]
    fn test_yi() {
        assert_eq!(
            ChineseNumberConverter::convert("一亿").unwrap(),
            "100000000"
        );
        assert_eq!(
            ChineseNumberConverter::convert("十亿").unwrap(),
            "1000000000"
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            ChineseNumberConverter::convert("三万五千").unwrap(),
            "35000"
        );
        assert_eq!(
            ChineseNumberConverter::convert("二十万零五").unwrap(),
            "200005"
        );
    }

    #[test]
    fn test_decimal() {
        assert_eq!(ChineseNumberConverter::convert("三点一四").unwrap(), "3.14");
        assert_eq!(
            ChineseNumberConverter::convert("零点五").unwrap(),
            "0.5"
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(ChineseNumberConverter::convert("负一").unwrap(), "-1");
        assert_eq!(
            ChineseNumberConverter::convert("负三点一四").unwrap(),
            "-3.14"
        );
    }

    #[test]
    fn test_is_chinese_number() {
        assert!(ChineseNumberConverter::is_chinese_number("一千二百三十四"));
        assert!(ChineseNumberConverter::is_chinese_number("三点一四"));
        assert!(ChineseNumberConverter::is_chinese_number("负五"));
        assert!(!ChineseNumberConverter::is_chinese_number("hello"));
        assert!(!ChineseNumberConverter::is_chinese_number("123"));
    }

    #[test]
    fn test_invalid_expression() {
        // 百、千前必须有数字
        assert!(ChineseNumberConverter::convert("百").is_err());
        assert!(ChineseNumberConverter::convert("千").is_err());
    }
}
