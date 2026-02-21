//! 标点规则层
//!
//! 实现逻辑连接词检测、问号规则等

use crate::punctuation::config::StyleProfile;

/// 逻辑连接词列表
const LOGIC_WORDS: &[&str] = &[
    "因为", "所以", "但是", "然而", "如果", "虽然", "因此", "同时", "另外",
];

/// 问号关键词（严格模式）
const QUESTION_KEYWORDS: &[&str] = &[
    // 疑问语气词
    "吗", "呢", "么",
    // 反问/确认
    "是否", "是不是", "能否", "可以吗", "对吗", "行吗", "好吗",
    "能不能", "有没有", "会不会", "要不要", "该不该",
];

/// 规则层
pub struct RuleLayer {
    profile: StyleProfile,
}

impl RuleLayer {
    /// 创建新的规则层
    pub fn new(profile: StyleProfile) -> Self {
        Self { profile }
    }

    /// 检查是否应该在此词前插入逗号（逻辑连接词规则）
    ///
    /// # 参数
    /// - `word`: 当前词
    /// - `total_tokens`: 句子总 token 数
    pub fn should_insert_comma_before(&self, word: &str, total_tokens: usize) -> bool {
        // 检查 token 数量
        if total_tokens < self.profile.logic_word_min_tokens {
            return false;
        }

        // 检查是否为逻辑连接词
        if !Self::is_logic_word(word) {
            return false;
        }

        // 根据强度决定
        // logic_word_strength: 0.8 = 高置信度才触发
        // logic_word_strength: 1.0 = 正常触发
        // logic_word_strength: 1.2 = 宽松触发
        if self.profile.logic_word_strength >= 0.8 {
            return true;
        }

        false
    }

    /// 检查句子是否应该以问号结尾
    ///
    /// # 参数
    /// - `sentence`: 句子文本
    /// - `energy_rising`: 能量是否上扬（声学特征，PushToTalk 模式下不可靠）
    pub fn should_end_with_question(&self, sentence: &str, energy_rising: bool) -> bool {
        // 句子长度不足，不判断为问句
        if sentence.chars().count() < 2 {
            return false;
        }

        // 检查是否以问号关键词结尾
        let has_question_keyword = QUESTION_KEYWORDS.iter().any(|kw| sentence.ends_with(kw));

        if !has_question_keyword {
            return false;
        }

        // 严格模式下：
        // "吗/呢/么" 等单字语气词直接接受（PushToTalk 模式下能量上扬检测不可靠）
        // 仅对歧义较高的词语（如独立出现的 "吗"）保留能量校验作为辅助
        if self.profile.question_strict_mode {
            // "呢" 在句尾可能是陈述语气，需要额外校验
            if sentence.ends_with("呢") {
                // 有明确疑问上下文（句子中含疑问词）则直接接受
                let has_wh_word = ["什么", "怎么", "哪", "谁", "为什么", "几", "多少"]
                    .iter()
                    .any(|w| sentence.contains(w));
                if has_wh_word || energy_rising {
                    return true;
                }
                return false;
            }

            // 其他关键词（"吗", "是否", "能否" 等）直接接受
            return true;
        }

        // 非严格模式：有关键词即可
        true
    }

    /// 检查是否应该插入句号
    ///
    /// 基于 VAD 段结束判定
    pub fn should_insert_period(&self, sentence: &str, vad_silence_duration_ms: u64) -> bool {
        // 如果 VAD 检测到静音（> 0），使用标准规则：≥ 800ms 才插入句号
        if vad_silence_duration_ms > 0 {
            return vad_silence_duration_ms >= 800;
        }

        // 如果是手动停止（vad_silence_duration_ms == 0），且句子不为空，也添加句号
        // 这样用户手动结束时也能获得完整的标点
        !sentence.is_empty()
    }

    /// 检查是否为逻辑连接词
    pub fn is_logic_word(word: &str) -> bool {
        LOGIC_WORDS.contains(&word)
    }

    /// 更新配置
    pub fn update_profile(&mut self, profile: StyleProfile) {
        self.profile = profile;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_logic_word() {
        assert!(RuleLayer::is_logic_word("因为"));
        assert!(RuleLayer::is_logic_word("所以"));
        assert!(RuleLayer::is_logic_word("但是"));
        assert!(!RuleLayer::is_logic_word("你好"));
    }

    #[test]
    fn test_should_insert_comma_before() {
        let layer = RuleLayer::new(StyleProfile::from_preset("Professional"));

        // token 数不足
        assert!(!layer.should_insert_comma_before("所以", 5));

        // token 数足够，是逻辑连接词
        assert!(layer.should_insert_comma_before("所以", 10));

        // 不是逻辑连接词
        assert!(!layer.should_insert_comma_before("你好", 10));
    }

    #[test]
    fn test_should_end_with_question_strict_mode() {
        let layer = RuleLayer::new(StyleProfile::from_preset("Professional"));

        // 严格模式，"好吗" 是明确问句关键词，无论能量是否上扬都返回问号
        assert!(layer.should_end_with_question("你好吗", false));

        // 严格模式，有问号关键词且能量上扬
        assert!(layer.should_end_with_question("你好吗", true));

        // 非 "吗" 结尾的问号关键词 - "是否" 需要在句尾
        assert!(layer.should_end_with_question("可以是否", false));

        // "是否" 在句首不会被检测为问句（需要在句尾）
        assert!(!layer.should_end_with_question("是否可行", false));
    }

    #[test]
    fn test_should_end_with_question_non_strict() {
        let layer = RuleLayer::new(StyleProfile::from_preset("Balanced"));

        // 非严格模式，有问号关键词即可
        assert!(layer.should_end_with_question("你好吗", false));

        // "是否" 需要在句尾
        assert!(layer.should_end_with_question("可以是否", false));

        // "是否" 在句首不算
        assert!(!layer.should_end_with_question("是否可行", false));

        // "能否" 在句尾
        assert!(layer.should_end_with_question("这样能否", false));
    }

    #[test]
    fn test_should_insert_period() {
        let layer = RuleLayer::new(StyleProfile::from_preset("Professional"));

        // 静音不足 800ms
        assert!(!layer.should_insert_period("测试句子", 500));

        // 静音达到 800ms
        assert!(layer.should_insert_period("测试句子", 800));

        // 静音超过 800ms
        assert!(layer.should_insert_period("测试句子", 1000));
    }

    #[test]
    fn test_no_question_without_keyword() {
        let layer = RuleLayer::new(StyleProfile::from_preset("Professional"));

        assert!(!layer.should_end_with_question("这是一句普通的话", false));
        assert!(!layer.should_end_with_question("这是一句普通的话", true));
    }

    #[test]
    fn test_logic_word_min_tokens() {
        let mut profile = StyleProfile::from_preset("Professional");
        profile.logic_word_min_tokens = 12;

        let layer = RuleLayer::new(profile);

        // token 数 < 12
        assert!(!layer.should_insert_comma_before("所以", 10));

        // token 数 >= 12
        assert!(layer.should_insert_comma_before("所以", 12));
    }
}
