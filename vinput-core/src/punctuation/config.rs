//! 标点系统配置模块
//!
//! 定义 StyleProfile 和标点系统参数

use serde::{Deserialize, Serialize};

/// 标点风格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    /// Streaming 阶段停顿比例阈值
    pub streaming_pause_ratio: f32,

    /// Streaming 阶段最小 token 数
    pub streaming_min_tokens: usize,

    /// 距离上次逗号的最小 token 数
    pub min_tokens_between_commas: usize,

    /// 最小停顿时长（毫秒）
    pub min_pause_duration_ms: u64,

    /// 是否允许感叹号
    pub allow_exclamation: bool,

    /// 问号严格模式
    pub question_strict_mode: bool,

    /// 逻辑连接词插入强度 (0.0 - 2.0)
    pub logic_word_strength: f32,

    /// 逻辑连接词插入最小 token 数
    pub logic_word_min_tokens: usize,
}

impl StyleProfile {
    /// Professional 风格（默认，推荐）
    ///
    /// 特点：
    /// - 稳重克制，适合办公、技术文档、会议记录
    /// - 标点精简，避免过度标注
    /// - 逻辑连接词谨慎插入
    /// - 问号需严格匹配
    pub fn professional() -> Self {
        Self {
            streaming_pause_ratio: 3.5,
            streaming_min_tokens: 6,
            min_tokens_between_commas: 4,
            min_pause_duration_ms: 500,
            allow_exclamation: false,
            question_strict_mode: true,
            logic_word_strength: 0.8,
            logic_word_min_tokens: 8,
        }
    }

    /// Balanced 风格（可选）
    ///
    /// 特点：
    /// - 更自然，标点略多
    /// - 接近人工书写习惯
    /// - 问号检测宽松
    pub fn balanced() -> Self {
        Self {
            streaming_pause_ratio: 2.8,
            streaming_min_tokens: 4,
            min_tokens_between_commas: 3,
            min_pause_duration_ms: 400,
            allow_exclamation: false,
            question_strict_mode: false,
            logic_word_strength: 1.0,
            logic_word_min_tokens: 6,
        }
    }

    /// Expressive 风格（可选）
    ///
    /// 特点：
    /// - 情绪表达明显
    /// - 接近口语化
    /// - 允许感叹号
    /// - 标点丰富
    pub fn expressive() -> Self {
        Self {
            streaming_pause_ratio: 2.2,
            streaming_min_tokens: 3,
            min_tokens_between_commas: 2,
            min_pause_duration_ms: 300,
            allow_exclamation: true,
            question_strict_mode: false,
            logic_word_strength: 1.2,
            logic_word_min_tokens: 5,
        }
    }
}

impl Default for StyleProfile {
    fn default() -> Self {
        Self::professional()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_professional_profile() {
        let profile = StyleProfile::professional();
        assert_eq!(profile.streaming_pause_ratio, 3.5);
        assert_eq!(profile.streaming_min_tokens, 6);
        assert!(!profile.allow_exclamation);
        assert!(profile.question_strict_mode);
    }

    #[test]
    fn test_balanced_profile() {
        let profile = StyleProfile::balanced();
        assert_eq!(profile.streaming_pause_ratio, 2.8);
        assert_eq!(profile.streaming_min_tokens, 4);
    }

    #[test]
    fn test_expressive_profile() {
        let profile = StyleProfile::expressive();
        assert_eq!(profile.streaming_pause_ratio, 2.2);
        assert!(profile.allow_exclamation);
    }

    #[test]
    fn test_default_is_professional() {
        let default_profile = StyleProfile::default();
        let professional = StyleProfile::professional();

        assert_eq!(default_profile.streaming_pause_ratio, professional.streaming_pause_ratio);
        assert_eq!(default_profile.streaming_min_tokens, professional.streaming_min_tokens);
    }
}
