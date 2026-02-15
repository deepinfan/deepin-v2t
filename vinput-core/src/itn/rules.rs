//! ITN 转换规则模块
//!
//! CurrencyRule, UnitRule, PercentageRule, DateRule

use crate::error::{VInputError, VInputResult};

/// CurrencyRule - 金额转换规则
pub struct CurrencyRule;

impl CurrencyRule {
    /// 转换金额表达
    ///
    /// # 参数
    /// - `amount`: 数字金额（例如："100"）
    /// - `symbol`: 货币符号（例如："$", "¥"）
    ///
    /// # 返回
    /// - 格式化的金额（例如："$100", "¥300"）
    pub fn format(amount: &str, symbol: &str) -> String {
        // Professional 默认：不加千分位，不强制两位小数
        format!("{}{}", symbol, amount)
    }

    /// 处理百万级表达
    ///
    /// 例如："3.5 million USD" → "3.5 million USD"（不强制展开）
    pub fn format_million(amount: &str, unit: &str, currency: &str) -> String {
        format!("{} {} {}", amount, unit, currency)
    }
}

/// UnitRule - 单位转换规则
pub struct UnitRule;

impl UnitRule {
    /// 支持的单位列表
    const SUPPORTED_UNITS: &'static [&'static str] = &[
        "GB", "MB", "KB", "TB",
        "CPU",
        "Hz", "MHz", "GHz",
        "ms", "s",
        "%",
    ];

    /// 检查是否为支持的单位
    pub fn is_supported_unit(unit: &str) -> bool {
        Self::SUPPORTED_UNITS.contains(&unit)
    }

    /// 格式化数字+单位
    ///
    /// # 参数
    /// - `number`: 数字（例如："100"）
    /// - `unit`: 单位（例如："GB"）
    ///
    /// # 返回
    /// - 格式化的结果（例如："100 GB"）- 保留空格
    pub fn format(number: &str, unit: &str) -> String {
        format!("{} {}", number, unit)
    }
}

/// PercentageRule - 百分比转换规则
pub struct PercentageRule;

impl PercentageRule {
    /// 转换中文百分比表达
    ///
    /// # 参数
    /// - `text`: 中文百分比（例如："百分之二十"）
    ///
    /// # 返回
    /// - 阿拉伯数字百分比（例如："20%"）
    pub fn convert_chinese(text: &str) -> VInputResult<String> {
        // 检查是否为百分比表达
        if !text.starts_with("百分之") {
            return Err(VInputError::ItnConversion(
                "Not a valid percentage expression".to_string(),
            ));
        }

        // 提取数字部分
        let number_part = &text[9..]; // "百分之" 是 9 字节

        // 这里应该调用 ChineseNumberConverter，但为了避免循环依赖
        // 我们简单实现一些常见的转换
        let number = match number_part {
            "十" => "10",
            "二十" => "20",
            "三十" => "30",
            "四十" => "40",
            "五十" => "50",
            "六十" => "60",
            "七十" => "70",
            "八十" => "80",
            "九十" => "90",
            "一百" | "百" => "100",
            _ => {
                // 对于复杂的数字，返回错误（需要在主管道中处理）
                return Err(VInputError::ItnConversion(format!(
                    "Unsupported percentage number: {}",
                    number_part
                )));
            }
        };

        Ok(format!("{}%", number))
    }

    /// 格式化数字为百分比
    pub fn format(number: &str) -> String {
        format!("{}%", number)
    }
}

/// DateRule - 日期转换规则
pub struct DateRule;

impl DateRule {
    /// 转换中文日期
    ///
    /// # 参数
    /// - `text`: 中文日期（例如："二零二六年三月五号"）
    ///
    /// # 返回
    /// - 数字日期（例如："2026年3月5日"）
    pub fn convert_chinese(text: &str) -> VInputResult<String> {
        let mut result = text.to_string();

        // Step 1: 转换 "号" → "日"
        result = result.replace("号", "日");

        // Step 2: 使用字符迭代器安全地处理年份、月份、日期

        // 处理年份（如 "二零二六年" → "2026年"）
        result = Self::convert_date_component(&result, '年', true);

        // 处理月份（如 "二月" → "2月"）
        result = Self::convert_date_component(&result, '月', false);

        // 处理日期（如 "十四日" → "14日"）
        result = Self::convert_date_component(&result, '日', false);

        Ok(result)
    }

    /// 转换日期组件（年/月/日）
    fn convert_date_component(text: &str, delimiter: char, is_year: bool) -> String {
        use crate::itn::ChineseNumberConverter;

        // 查找分隔符位置
        if let Some(delim_pos) = text.find(delimiter) {
            // 向前收集中文数字字符
            let chars: Vec<char> = text.chars().collect();
            let delim_idx = text.char_indices()
                .position(|(_, c)| c == delimiter)
                .unwrap();

            // 向前查找连续的中文数字
            let mut start_idx = delim_idx;
            while start_idx > 0 {
                let ch = chars[start_idx - 1];
                if Self::is_chinese_number_char(ch) {
                    start_idx -= 1;
                } else {
                    break;
                }
            }

            if start_idx < delim_idx {
                // 提取数字部分
                let number_chars: String = chars[start_idx..delim_idx].iter().collect();

                // 转换数字
                let converted = if is_year && Self::is_chinese_digit_sequence(&number_chars) {
                    // 年份逐位转换（如 "二零二六" → "2026"）
                    Self::convert_year_digits(&number_chars)
                } else {
                    // 月份和日期使用 ChineseNumberConverter（如 "十四" → "14"）
                    ChineseNumberConverter::convert(&number_chars).unwrap_or(number_chars.clone())
                };

                // 替换
                let before: String = chars[..start_idx].iter().collect();
                let after: String = chars[delim_idx..].iter().collect();
                return format!("{}{}{}", before, converted, after);
            }
        }

        text.to_string()
    }

    /// 检查是否为中文数字字符
    fn is_chinese_number_char(ch: char) -> bool {
        matches!(ch, '零'|'一'|'二'|'三'|'四'|'五'|'六'|'七'|'八'|'九'|'十'|'百'|'千'|'万'|'亿')
    }

    /// 检查是否为中文数字序列（仅基础数字，用于年份判断）
    fn is_chinese_digit_sequence(text: &str) -> bool {
        text.chars().all(|c| matches!(c, '零'|'一'|'二'|'三'|'四'|'五'|'六'|'七'|'八'|'九'))
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

    /// 检查是否为日期表达
    pub fn is_date_expression(text: &str) -> bool {
        (text.contains("年") && text.contains("月"))
            || (text.contains("年") && (text.contains("日") || text.contains("号")))
            || (text.contains("月") && (text.contains("日") || text.contains("号")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_rule_format() {
        assert_eq!(CurrencyRule::format("100", "$"), "$100");
        assert_eq!(CurrencyRule::format("300", "¥"), "¥300");
        assert_eq!(CurrencyRule::format("50.5", "€"), "€50.5");
    }

    #[test]
    fn test_currency_rule_million() {
        assert_eq!(
            CurrencyRule::format_million("3.5", "million", "USD"),
            "3.5 million USD"
        );
    }

    #[test]
    fn test_unit_rule_supported() {
        assert!(UnitRule::is_supported_unit("GB"));
        assert!(UnitRule::is_supported_unit("MHz"));
        assert!(UnitRule::is_supported_unit("%"));
        assert!(!UnitRule::is_supported_unit("unknown"));
    }

    #[test]
    fn test_unit_rule_format() {
        assert_eq!(UnitRule::format("100", "GB"), "100 GB");
        assert_eq!(UnitRule::format("2.4", "GHz"), "2.4 GHz");
    }

    #[test]
    fn test_percentage_rule_chinese() {
        assert_eq!(
            PercentageRule::convert_chinese("百分之二十").unwrap(),
            "20%"
        );
        assert_eq!(
            PercentageRule::convert_chinese("百分之五十").unwrap(),
            "50%"
        );
        assert_eq!(
            PercentageRule::convert_chinese("百分之一百").unwrap(),
            "100%"
        );
    }

    #[test]
    fn test_percentage_rule_format() {
        assert_eq!(PercentageRule::format("20"), "20%");
        assert_eq!(PercentageRule::format("50.5"), "50.5%");
    }

    #[test]
    fn test_date_rule_is_date() {
        assert!(DateRule::is_date_expression("二零二六年三月"));
        assert!(DateRule::is_date_expression("三月五号"));
        assert!(DateRule::is_date_expression("二零二六年三月五日"));
        assert!(!DateRule::is_date_expression("一百个"));
    }

    #[test]
    fn test_date_rule_convert_号_to_日() {
        let result = DateRule::convert_chinese("三月五号").unwrap();
        assert!(result.contains("日"));
        assert!(!result.contains("号"));
    }
}
