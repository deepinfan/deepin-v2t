//! 标点主引擎
//!
//! 整合 PauseEngine 和 RuleLayer，提供完整的标点处理

use crate::punctuation::config::StyleProfile;
use crate::punctuation::pause_engine::{PauseEngine, TokenInfo};
use crate::punctuation::rules::RuleLayer;

/// 标点处理结果
#[derive(Debug, Clone)]
pub struct PunctuationResult {
    /// 处理后的文本
    pub text: String,
    /// 是否有变更
    pub has_changes: bool,
}

/// 标点引擎
pub struct PunctuationEngine {
    pause_engine: PauseEngine,
    rule_layer: RuleLayer,
    profile: StyleProfile,
    current_sentence: Vec<String>,
}

impl PunctuationEngine {
    /// 创建新的标点引擎
    pub fn new(profile: StyleProfile) -> Self {
        tracing::info!("🎯 PunctuationEngine::new - 配置: pause_ratio={}, min_tokens={}, allow_exclamation={}",
            profile.streaming_pause_ratio,
            profile.streaming_min_tokens,
            profile.allow_exclamation
        );

        Self {
            pause_engine: PauseEngine::new(profile.clone()),
            rule_layer: RuleLayer::new(profile.clone()),
            profile,
            current_sentence: Vec::new(),
        }
    }

    /// 使用默认配置（Professional）
    pub fn default() -> Self {
        Self::new(StyleProfile::default())
    }

    /// 处理新的 token
    ///
    /// # 参数
    /// - `token`: Token 信息
    ///
    /// # 返回
    /// - `Some(token_with_comma)`: 如果需要在 token 前插入逗号
    /// - `None`: 不插入逗号，返回原 token
    pub fn process_token(&mut self, token: TokenInfo) -> Option<String> {
        let word = token.text.clone();

        // 1. 检查逻辑连接词规则
        let should_insert_comma_rule = self.rule_layer.should_insert_comma_before(
            &word,
            self.current_sentence.len(),
        );

        // 2. 检查停顿规则
        let should_insert_comma_pause = self.pause_engine.add_token(token);

        // 3. 决定是否插入逗号
        let insert_comma = should_insert_comma_rule || should_insert_comma_pause;

        // 4. 构造返回值
        self.current_sentence.push(word.clone());

        if insert_comma {
            Some(format!("，{}", word))
        } else {
            Some(word)
        }
    }

    /// 处理句子结束
    ///
    /// # 参数
    /// - `vad_silence_ms`: VAD 检测到的静音时长（毫秒）
    /// - `energy_rising`: 句尾能量是否上扬
    ///
    /// # 返回
    /// - 句子结尾标点（"。", "？", 或空字符串）
    pub fn finalize_sentence(
        &mut self,
        vad_silence_ms: u64,
        energy_rising: bool,
    ) -> String {
        let sentence_text = self.current_sentence.join("");

        // 1. 检查是否应该插入问号
        if self.rule_layer.should_end_with_question(&sentence_text, energy_rising) {
            self.reset_sentence();
            return "？".to_string();
        }

        // 2. 检查是否应该插入句号
        if self.rule_layer.should_insert_period(&sentence_text, vad_silence_ms) {
            self.reset_sentence();
            return "。".to_string();
        }

        // 3. 默认不插入句号（继续等待）
        "".to_string()
    }

    /// 重置句子状态（用于新的 VAD 段）
    pub fn reset_sentence(&mut self) {
        self.current_sentence.clear();
        self.pause_engine.reset();
    }

    /// 获取当前句子
    pub fn current_sentence(&self) -> String {
        self.current_sentence.join("")
    }

    /// 更新配置
    pub fn update_profile(&mut self, profile: StyleProfile) {
        self.profile = profile.clone();
        self.pause_engine.update_profile(profile.clone());
        self.rule_layer.update_profile(profile);
    }

    /// 获取当前配置
    pub fn profile(&self) -> &StyleProfile {
        &self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_punctuation_engine_basic() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        let token = TokenInfo::new("你好".to_string(), 0, 200);
        let result = engine.process_token(token);

        assert_eq!(result, Some("你好".to_string()));
    }

    #[test]
    fn test_punctuation_engine_with_pause() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        // 添加 6 个正常 token
        for i in 0..6 {
            let token = TokenInfo::new(
                format!("词{}", i),
                i * 200,
                i * 200 + 180,
            );
            engine.process_token(token);
        }

        // 添加一个带长停顿的 token
        let paused_token = TokenInfo::new("下一个".to_string(), 2000, 2180);
        let result = engine.process_token(paused_token);

        // 应该在 "下一个" 前插入逗号
        assert_eq!(result, Some("，下一个".to_string()));
    }

    #[test]
    fn test_punctuation_engine_logic_word() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        // 添加 8 个 token（达到 logic_word_min_tokens）
        for i in 0..8 {
            let token = TokenInfo::new(
                format!("词{}", i),
                i * 200,
                i * 200 + 180,
            );
            engine.process_token(token);
        }

        // 添加逻辑连接词
        let logic_token = TokenInfo::new("所以".to_string(), 1600, 1780);
        let result = engine.process_token(logic_token);

        // 应该在 "所以" 前插入逗号
        assert_eq!(result, Some("，所以".to_string()));
    }

    #[test]
    fn test_finalize_sentence_with_question() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        // 添加 "你好吗"
        engine.process_token(TokenInfo::new("你好".to_string(), 0, 200));
        engine.process_token(TokenInfo::new("吗".to_string(), 200, 350));

        // 结束时能量上扬，应该插入问号
        let ending = engine.finalize_sentence(1000, true);
        assert_eq!(ending, "？");
    }

    #[test]
    fn test_finalize_sentence_with_period() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        // 添加普通句子
        engine.process_token(TokenInfo::new("测试".to_string(), 0, 200));
        engine.process_token(TokenInfo::new("句子".to_string(), 200, 400));

        // VAD 静音超过 800ms，应该插入句号
        let ending = engine.finalize_sentence(900, false);
        assert_eq!(ending, "。");
    }

    #[test]
    fn test_finalize_sentence_no_punctuation() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        engine.process_token(TokenInfo::new("测试".to_string(), 0, 200));

        // VAD 静音不足，不插入句号
        let ending = engine.finalize_sentence(500, false);
        assert_eq!(ending, "");
    }

    #[test]
    fn test_reset_sentence() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        engine.process_token(TokenInfo::new("测试".to_string(), 0, 200));
        assert_eq!(engine.current_sentence(), "测试");

        engine.reset_sentence();
        assert_eq!(engine.current_sentence(), "");
    }

    #[test]
    fn test_update_profile() {
        let mut engine = PunctuationEngine::new(StyleProfile::professional());

        engine.update_profile(StyleProfile::balanced());
        assert_eq!(engine.profile().streaming_pause_ratio, 2.8);
    }

    #[test]
    fn test_default_engine() {
        let engine = PunctuationEngine::default();
        assert_eq!(engine.profile().streaming_pause_ratio, 3.5);
    }
}
