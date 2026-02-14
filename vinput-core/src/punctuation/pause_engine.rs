//! 停顿检测引擎
//!
//! 基于 VAD 时间戳和 token 时长检测停顿，判断是否插入逗号

use crate::punctuation::config::StyleProfile;
use std::time::Duration;

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
    /// 返回是否应该在此 token 前插入逗号
    pub fn add_token(&mut self, token: TokenInfo) -> bool {
        let should_insert = self.should_insert_comma(&token);

        if should_insert {
            tracing::debug!("  🎯 检测到停顿，将在 '{}' 前插入逗号", token.text);
            self.last_comma_position = Some(self.token_history.len());
        }

        self.token_history.push(token);
        should_insert
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

        // Sherpa-ONNX 的 timestamps 是连续的，Token 之间没有间隙
        // 停顿包含在上一个 Token 的时长中
        // 因此我们检测上一个 Token 的时长是否异常长
        if let Some(last_token) = self.token_history.last() {
            let last_token_duration = last_token.duration_ms();

            // 检查最小时长（避免误判短 Token）
            if last_token_duration < self.profile.min_pause_duration_ms {
                tracing::debug!("    ⏭  上一Token '{}' 时长 {}ms < 最小阈值 {}ms，跳过",
                    last_token.text, last_token_duration, self.profile.min_pause_duration_ms);
                return false;
            }

            // 计算平均 token 时长
            let avg_duration = self.calculate_avg_token_duration();
            if avg_duration == 0 {
                return false;
            }

            // 计算时长比例（上一个 Token 的时长 / 平均时长）
            let duration_ratio = last_token_duration as f32 / avg_duration as f32;

            tracing::debug!("    ⏱  停顿检测: 上一Token='{}' 时长={}ms, 平均={}ms, 比例={:.2}, 阈值={:.2}",
                last_token.text, last_token_duration, avg_duration, duration_ratio, self.profile.streaming_pause_ratio);

            // 如果上一个 Token 的时长显著超过平均值，说明包含了停顿
            // 在当前 Token 前插入逗号
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
        let mut engine = PauseEngine::new(StyleProfile::professional());

        // 添加第一个 token - 不应插入逗号（token 数不足）
        assert!(!engine.add_token(TokenInfo::new("你好".to_string(), 0, 200)));
    }

    #[test]
    fn test_pause_engine_min_tokens() {
        let mut engine = PauseEngine::new(StyleProfile::professional());

        // 添加 5 个 token（小于 min_tokens: 6）
        for i in 0..5 {
            let token = TokenInfo::new(
                format!("token{}", i),
                i * 200,
                i * 200 + 180,
            );
            assert!(!engine.add_token(token));
        }

        assert_eq!(engine.token_count(), 5);
    }

    #[test]
    fn test_pause_engine_with_pause() {
        let mut engine = PauseEngine::new(StyleProfile::professional());

        // 添加 6 个正常 token
        for i in 0..6 {
            let token = TokenInfo::new(
                format!("token{}", i),
                i * 200,
                i * 200 + 180,
            );
            assert!(!engine.add_token(token));
        }

        // 添加一个带长停顿的 token
        // 上一个 token 结束于 1180ms
        // 这个 token 开始于 2000ms，停顿 820ms
        // 平均 token 时长约 180ms，停顿比例 = 820/180 ≈ 4.5 > 3.5
        let paused_token = TokenInfo::new(
            "next".to_string(),
            2000,
            2180,
        );
        assert!(engine.add_token(paused_token));
    }

    #[test]
    fn test_pause_engine_min_tokens_between_commas() {
        let mut engine = PauseEngine::new(StyleProfile::professional());

        // 添加 6 个 token
        for i in 0..6 {
            engine.add_token(TokenInfo::new(
                format!("token{}", i),
                i * 200,
                i * 200 + 180,
            ));
        }

        // 触发第一个逗号
        engine.add_token(TokenInfo::new("next".to_string(), 2000, 2180));

        // 立即尝试触发第二个逗号（但 tokens_since_comma < 4）
        let token2 = TokenInfo::new("another".to_string(), 3000, 3180);
        assert!(!engine.add_token(token2));
    }

    #[test]
    fn test_pause_engine_reset() {
        let mut engine = PauseEngine::new(StyleProfile::professional());

        engine.add_token(TokenInfo::new("test".to_string(), 0, 200));
        assert_eq!(engine.token_count(), 1);

        engine.reset();
        assert_eq!(engine.token_count(), 0);
    }

    #[test]
    fn test_calculate_avg_duration() {
        let mut engine = PauseEngine::new(StyleProfile::professional());

        // 添加 3 个 token，时长分别为 100ms, 200ms, 300ms
        engine.add_token(TokenInfo::new("t1".to_string(), 0, 100));
        engine.add_token(TokenInfo::new("t2".to_string(), 100, 300));
        engine.add_token(TokenInfo::new("t3".to_string(), 300, 600));

        let avg = engine.calculate_avg_token_duration();
        assert_eq!(avg, (100 + 200 + 300) / 3); // 200ms
    }

    #[test]
    fn test_token_info_duration() {
        let token = TokenInfo::new("test".to_string(), 100, 350);
        assert_eq!(token.duration_ms(), 250);
    }
}
