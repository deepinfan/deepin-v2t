//! 停顿检测引擎
//!
//! 基于 VAD 时间戳和 token 时长检测停顿，判断是否插入逗号

use crate::punctuation::config::StyleProfile;

/// Token 信息
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Token 文本
    pub text: String,
    /// Token 开始时间（毫秒）
    pub start_ms: u64,
    /// Token 结束时间（毫秒）
    pub end_ms: u64,
}

impl TokenInfo {
    /// 创建新的 TokenInfo
    pub fn new(text: String, start_ms: u64, end_ms: u64) -> Self {
        Self {
            text,
            start_ms,
            end_ms,
        }
    }

    /// Token 时长（毫秒）
    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }
}

/// Token 处理结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenAction {
    /// 跳过此 token（时长过短）
    Skip,
    /// 正常添加 token
    Normal,
    /// 在 token 前插入逗号
    InsertComma,
}

/// 停顿检测引擎
pub struct PauseEngine {
    profile: StyleProfile,
    token_history: Vec<TokenInfo>,
    last_comma_position: Option<usize>,
}

impl PauseEngine {
    /// 创建新的停顿检测引擎
    pub fn new(profile: StyleProfile) -> Self {
        Self {
            profile,
            token_history: Vec::new(),
            last_comma_position: None,
        }
    }

    /// 添加新 token
    ///
    /// 返回 Token 处理动作
    pub fn add_token(&mut self, token: TokenInfo) -> TokenAction {
        // ✅ 检查 token 内容，过滤空白字符和无意义字符
        let trimmed = token.text.trim();
        if trimmed.is_empty() || trimmed == " " || trimmed == "NE" {
            tracing::debug!("    ⏭  Token '{}' 是空白或无意义字符，跳过", token.text);
            // 不添加到历史，直接跳过
            return TokenAction::Skip;
        }

        let should_insert = self.should_insert_comma(&token);

        if should_insert {
            tracing::debug!("  🎯 检测到停顿，将在 '{}' 前插入逗号", token.text);
            self.last_comma_position = Some(self.token_history.len());
        }

        self.token_history.push(token);

        if should_insert {
            TokenAction::InsertComma
        } else {
            TokenAction::Normal
        }
    }

    /// 判断是否应该插入逗号
    fn should_insert_comma(&self, token: &TokenInfo) -> bool {
        // 检查 token 数量
        if self.token_history.len() < self.profile.streaming_min_tokens {
            return false;
        }

        // 检查距离上次逗号的 token 数
        if let Some(last_pos) = self.last_comma_position {
            let tokens_since_comma = self.token_history.len() - last_pos;
            if tokens_since_comma < self.profile.min_tokens_between_commas {
                return false;
            }
        }

        if let Some(last_token) = self.token_history.last() {
            // 方法1：检查相邻 Token 时间间隙
            // sherpa-onnx 的停顿更可靠地体现在 end_ms → start_ms 的空白上
            let gap_ms = token.start_ms.saturating_sub(last_token.end_ms);
            if gap_ms >= 200 {
                tracing::debug!("  🎯 停顿检测(间隙法): '{}' 前有 {}ms 间隙 >= 200ms，插入逗号",
                    token.text, gap_ms);
                return true;
            }

            // 方法2：检查上一个 Token 时长比例（timestamps 连续时的备用方法）
            // Sherpa-ONNX 的 timestamps 连续时，停顿包含在上一个 Token 的时长中
            let last_token_duration = last_token.duration_ms();

            if last_token_duration < self.profile.min_pause_duration_ms {
                tracing::debug!("    ⏭  上一Token '{}' 时长 {}ms < 最小停顿阈值 {}ms，不检测停顿",
                    last_token.text, last_token_duration, self.profile.min_pause_duration_ms);
                return false;
            }

            let avg_duration = self.calculate_avg_token_duration();
            if avg_duration == 0 {
                return false;
            }

            let duration_ratio = last_token_duration as f32 / avg_duration as f32;

            tracing::debug!("    ⏱  停顿检测(时长法): 上一Token='{}' 时长={}ms, 平均={}ms, 比例={:.2}, 阈值={:.2}",
                last_token.text, last_token_duration, avg_duration, duration_ratio, self.profile.streaming_pause_ratio);

            return duration_ratio > self.profile.streaming_pause_ratio;
        }

        false
    }

    /// 计算最近 N 个 token 的平均时长
    fn calculate_avg_token_duration(&self) -> u64 {
        const WINDOW_SIZE: usize = 10;

        let window = if self.token_history.len() > WINDOW_SIZE {
            &self.token_history[self.token_history.len() - WINDOW_SIZE..]
        } else {
            &self.token_history[..]
        };

        if window.is_empty() {
            return 0;
        }

        let total: u64 = window.iter().map(|t| t.duration_ms()).sum();
        total / window.len() as u64
    }

    /// 重置引擎（用于新的 VAD 段）
    pub fn reset(&mut self) {
        self.token_history.clear();
        self.last_comma_position = None;
    }

    /// 获取当前 token 数量
    pub fn token_count(&self) -> usize {
        self.token_history.len()
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
    fn test_pause_engine_basic() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        // 添加第一个 token - 不应插入逗号（token 数不足）
        let action = engine.add_token(TokenInfo::new("你好".to_string(), 0, 600));
        assert_eq!(action, TokenAction::Normal);
    }

    #[test]
    fn test_pause_engine_min_tokens() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        // 添加 5 个 token（小于 min_tokens: 6）
        for i in 0..5 {
            let token = TokenInfo::new(
                format!("token{}", i),
                i * 200,
                i * 700 + 600,
            );
            let action = engine.add_token(token);
            assert_eq!(action, TokenAction::Normal);
        }

        assert_eq!(engine.token_count(), 5);
    }

    #[test]
    fn test_pause_engine_with_pause() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        // 添加 5 个正常 token（时长 600ms）
        for i in 0..5 {
            let token = TokenInfo::new(
                format!("token{}", i),
                i * 700,
                i * 700 + 600,
            );
            engine.add_token(token);
        }

        // 添加第 6 个 token，时长异常长（包含停顿）
        // 正常 600ms + 停顿 1700ms = 2300ms
        let long_token = TokenInfo::new(
            "token5".to_string(),
            3500,  // 5 * 700
            5800,  // 3500 + 2300
        );
        engine.add_token(long_token);

        // 添加下一个 token
        // 因为上一个 token 时长 2300ms，远超平均 600ms
        // 比例 = 2300 / 600 ≈ 3.83 > 3.5，应该插入逗号
        let next_token = TokenInfo::new(
            "next".to_string(),
            5800,
            6400,
        );
        let action = engine.add_token(next_token);
        assert_eq!(action, TokenAction::InsertComma);
    }

    #[test]
    fn test_pause_engine_skip_short_token() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        // min_pause_duration_ms = 500ms
        // 添加一个时长 < 500ms 的 token，应该被跳过
        let short_token = TokenInfo::new(" ".to_string(), 0, 41);
        let action = engine.add_token(short_token);
        assert_eq!(action, TokenAction::Skip);

        // 验证没有添加到历史
        assert_eq!(engine.token_count(), 0);
    }

    #[test]
    fn test_pause_engine_min_tokens_between_commas() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        // 添加 6 个 token
        for i in 0..6 {
            engine.add_token(TokenInfo::new(
                format!("token{}", i),
                i * 200,
                i * 700 + 600,
            ));
        }

        // 触发第一个逗号
        engine.add_token(TokenInfo::new("next".to_string(), 5000, 5600));

        // 立即尝试触发第二个逗号（但 tokens_since_comma < 4）
        let token2 = TokenInfo::new("another".to_string(), 7000, 7600);
        let action = engine.add_token(token2);
        assert_eq!(action, TokenAction::Normal);
    }

    #[test]
    fn test_pause_engine_reset() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        engine.add_token(TokenInfo::new("test".to_string(), 0, 600));
        assert_eq!(engine.token_count(), 1);

        engine.reset();
        assert_eq!(engine.token_count(), 0);
    }

    #[test]
    fn test_calculate_avg_duration() {
        let mut engine = PauseEngine::new(StyleProfile::from_preset("Professional"));

        // 添加 3 个 token，时长分别为 600ms, 700ms, 800ms（都 >= 500ms 不会被跳过）
        engine.add_token(TokenInfo::new("t1".to_string(), 0, 600));
        engine.add_token(TokenInfo::new("t2".to_string(), 600, 1300));
        engine.add_token(TokenInfo::new("t3".to_string(), 1300, 2100));

        let avg = engine.calculate_avg_token_duration();
        assert_eq!(avg, (600 + 700 + 800) / 3); // 700ms
    }

    #[test]
    fn test_token_info_duration() {
        let token = TokenInfo::new("test".to_string(), 100, 350);
        assert_eq!(token.duration_ms(), 250);
    }
}
