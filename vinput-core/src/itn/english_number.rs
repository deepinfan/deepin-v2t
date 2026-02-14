//! 英文数字解析模块
//!
//! 将英文数字表达转换为阿拉伯数字
//!
//! 支持：zero ~ nineteen, twenty, thirty, ..., ninety, hundred, thousand, million, billion, point

use crate::error::{VInputError, VInputResult};

/// 英文数字解析器
pub struct EnglishNumberParser;

impl EnglishNumberParser {
    /// 将英文数字字符串转换为阿拉伯数字
    ///
    /// # 参数
    /// - `text`: 英文数字文本（例如："one thousand two hundred thirty four"）
    ///
    /// # 返回
    /// - `Ok(String)`: 转换后的数字字符串
    /// - `Err`: 如果不是有效的英文数字表达
    pub fn convert(text: &str) -> VInputResult<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Ok(String::new());
        }

        // 检查是否包含小数点
        if let Some(point_idx) = words.iter().position(|&w| w == "point") {
            let integer_part = &words[..point_idx];
            let decimal_part = &words[point_idx + 1..];

            let integer = if integer_part.is_empty() {
                0
            } else {
                Self::parse_integer(integer_part)?
            };

            let decimal = Self::parse_decimal(decimal_part)?;

            return Ok(format!("{}.{}", integer, decimal));
        }

        // 纯整数
        let integer = Self::parse_integer(&words)?;
        Ok(format!("{}", integer))
    }

    /// 解析整数部分
    fn parse_integer(words: &[&str]) -> VInputResult<i64> {
        if words.is_empty() {
            return Ok(0);
        }

        let mut result = 0i64;
        let mut current = 0i64;

        for &word in words {
            let word_lower = word.to_lowercase();
            let num = match word_lower.as_str() {
                // 基础数字 0-19
                "zero" => Some(0),
                "one" => Some(1),
                "two" => Some(2),
                "three" => Some(3),
                "four" => Some(4),
                "five" => Some(5),
                "six" => Some(6),
                "seven" => Some(7),
                "eight" => Some(8),
                "nine" => Some(9),
                "ten" => Some(10),
                "eleven" => Some(11),
                "twelve" => Some(12),
                "thirteen" => Some(13),
                "fourteen" => Some(14),
                "fifteen" => Some(15),
                "sixteen" => Some(16),
                "seventeen" => Some(17),
                "eighteen" => Some(18),
                "nineteen" => Some(19),
                // 十位数 20-90
                "twenty" => Some(20),
                "thirty" => Some(30),
                "forty" => Some(40),
                "fifty" => Some(50),
                "sixty" => Some(60),
                "seventy" => Some(70),
                "eighty" => Some(80),
                "ninety" => Some(90),
                _ => None,
            };

            if let Some(n) = num {
                current += n;
            } else {
                match word_lower.as_str() {
                    "hundred" => {
                        if current == 0 {
                            return Err(VInputError::ItnConversion(
                                "Invalid expression: hundred without number".to_string(),
                            ));
                        }
                        current *= 100;
                    }
                    "thousand" => {
                        if current == 0 {
                            current = 1;
                        }
                        result += current * 1000;
                        current = 0;
                    }
                    "million" => {
                        if current == 0 {
                            current = 1;
                        }
                        result += current * 1_000_000;
                        current = 0;
                    }
                    "billion" => {
                        if current == 0 {
                            current = 1;
                        }
                        result += current * 1_000_000_000;
                        current = 0;
                    }
                    "and" => continue,
                    _ => {
                        return Err(VInputError::ItnConversion(format!(
                            "Invalid English number word: {}",
                            word
                        )));
                    }
                }
            }
        }

        result += current;
        Ok(result)
    }

    /// 解析小数部分
    fn parse_decimal(words: &[&str]) -> VInputResult<String> {
        let mut result = String::new();

        for &word in words {
            let digit = match word.to_lowercase().as_str() {
                "zero" => '0',
                "one" => '1',
                "two" => '2',
                "three" => '3',
                "four" => '4',
                "five" => '5',
                "six" => '6',
                "seven" => '7',
                "eight" => '8',
                "nine" => '9',
                _ => {
                    return Err(VInputError::ItnConversion(format!(
                        "Invalid digit in decimal part: {}",
                        word
                    )));
                }
            };
            result.push(digit);
        }

        Ok(result)
    }

    /// 检查文本是否为有效的英文数字
    pub fn is_english_number(text: &str) -> bool {
        let valid_words = [
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
            "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
            "seventeen", "eighteen", "nineteen", "twenty", "thirty", "forty", "fifty",
            "sixty", "seventy", "eighty", "ninety", "hundred", "thousand", "million",
            "billion", "point", "and",
        ];

        text.split_whitespace()
            .all(|word| valid_words.contains(&word.to_lowercase().as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_digit() {
        assert_eq!(EnglishNumberParser::convert("zero").unwrap(), "0");
        assert_eq!(EnglishNumberParser::convert("one").unwrap(), "1");
        assert_eq!(EnglishNumberParser::convert("nine").unwrap(), "9");
    }

    #[test]
    fn test_teens() {
        assert_eq!(EnglishNumberParser::convert("ten").unwrap(), "10");
        assert_eq!(EnglishNumberParser::convert("eleven").unwrap(), "11");
        assert_eq!(EnglishNumberParser::convert("nineteen").unwrap(), "19");
    }

    #[test]
    fn test_tens() {
        assert_eq!(EnglishNumberParser::convert("twenty").unwrap(), "20");
        assert_eq!(EnglishNumberParser::convert("thirty").unwrap(), "30");
        assert_eq!(EnglishNumberParser::convert("ninety").unwrap(), "90");
    }

    #[test]
    fn test_compound() {
        assert_eq!(
            EnglishNumberParser::convert("twenty one").unwrap(),
            "21"
        );
        assert_eq!(
            EnglishNumberParser::convert("ninety nine").unwrap(),
            "99"
        );
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(
            EnglishNumberParser::convert("one hundred").unwrap(),
            "100"
        );
        assert_eq!(
            EnglishNumberParser::convert("five hundred").unwrap(),
            "500"
        );
        assert_eq!(
            EnglishNumberParser::convert("nine hundred ninety nine").unwrap(),
            "999"
        );
    }

    #[test]
    fn test_thousands() {
        assert_eq!(
            EnglishNumberParser::convert("one thousand").unwrap(),
            "1000"
        );
        assert_eq!(
            EnglishNumberParser::convert("one thousand two hundred thirty four").unwrap(),
            "1234"
        );
    }

    #[test]
    fn test_millions() {
        assert_eq!(
            EnglishNumberParser::convert("one million").unwrap(),
            "1000000"
        );
        assert_eq!(
            EnglishNumberParser::convert("three million five hundred thousand").unwrap(),
            "3500000"
        );
    }

    #[test]
    fn test_billions() {
        assert_eq!(
            EnglishNumberParser::convert("one billion").unwrap(),
            "1000000000"
        );
        assert_eq!(
            EnglishNumberParser::convert("two billion").unwrap(),
            "2000000000"
        );
    }

    #[test]
    fn test_decimal() {
        assert_eq!(
            EnglishNumberParser::convert("three point one four").unwrap(),
            "3.14"
        );
        assert_eq!(
            EnglishNumberParser::convert("zero point five").unwrap(),
            "0.5"
        );
    }

    #[test]
    fn test_with_and() {
        assert_eq!(
            EnglishNumberParser::convert("one hundred and twenty three").unwrap(),
            "123"
        );
    }

    #[test]
    fn test_is_english_number() {
        assert!(EnglishNumberParser::is_english_number(
            "one thousand two hundred"
        ));
        assert!(EnglishNumberParser::is_english_number("three point one four"));
        assert!(!EnglishNumberParser::is_english_number("hello world"));
        assert!(!EnglishNumberParser::is_english_number("123"));
    }

    #[test]
    fn test_invalid_expression() {
        assert!(EnglishNumberParser::convert("hundred").is_err());
        assert!(EnglishNumberParser::convert("invalid word").is_err());
    }
}
