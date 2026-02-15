//! 撤销/重试机制
//!
//! 记录识别历史，支持撤销最近的识别结果

use std::collections::VecDeque;

/// 识别历史记录
#[derive(Debug, Clone)]
pub struct RecognitionHistory {
    /// 历史记录（最多保留 N 条）
    history: VecDeque<RecognitionEntry>,
    /// 最大历史记录数
    max_history: usize,
}

/// 单条识别记录
#[derive(Debug, Clone)]
pub struct RecognitionEntry {
    /// 识别结果文本
    pub text: String,
    /// 时间戳
    pub timestamp: std::time::SystemTime,
    /// 是否已撤销
    pub undone: bool,
}

impl RecognitionHistory {
    /// 创建新的历史记录管理器
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// 添加新的识别记录
    pub fn push(&mut self, text: String) {
        let entry = RecognitionEntry {
            text,
            timestamp: std::time::SystemTime::now(),
            undone: false,
        };

        self.history.push_back(entry);

        // 保持历史记录数量限制
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// 撤销最近的识别结果
    ///
    /// 返回被撤销的文本，如果没有可撤销的记录则返回 None
    pub fn undo(&mut self) -> Option<String> {
        // 从后往前查找第一个未撤销的记录
        for entry in self.history.iter_mut().rev() {
            if !entry.undone {
                entry.undone = true;
                return Some(entry.text.clone());
            }
        }

        None
    }

    /// 重试（恢复）最近撤销的识别结果
    ///
    /// 返回被恢复的文本，如果没有可恢复的记录则返回 None
    pub fn redo(&mut self) -> Option<String> {
        // 从后往前查找第一个已撤销的记录
        for entry in self.history.iter_mut().rev() {
            if entry.undone {
                entry.undone = false;
                return Some(entry.text.clone());
            }
        }

        None
    }

    /// 获取当前有效的识别历史（未撤销的）
    pub fn get_active_history(&self) -> Vec<String> {
        self.history
            .iter()
            .filter(|e| !e.undone)
            .map(|e| e.text.clone())
            .collect()
    }

    /// 清空历史记录
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// 获取历史记录数量
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// 检查是否可以撤销
    pub fn can_undo(&self) -> bool {
        self.history.iter().any(|e| !e.undone)
    }

    /// 检查是否可以重试
    pub fn can_redo(&self) -> bool {
        self.history.iter().any(|e| e.undone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_undo() {
        let mut history = RecognitionHistory::new(10);

        history.push("第一句话".to_string());
        history.push("第二句话".to_string());
        history.push("第三句话".to_string());

        assert_eq!(history.len(), 3);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        // 撤销最后一句
        let undone = history.undo();
        assert_eq!(undone, Some("第三句话".to_string()));
        assert!(history.can_undo());
        assert!(history.can_redo());

        // 再撤销一句
        let undone = history.undo();
        assert_eq!(undone, Some("第二句话".to_string()));

        // 获取有效历史
        let active = history.get_active_history();
        assert_eq!(active, vec!["第一句话"]);
    }

    #[test]
    fn test_redo() {
        let mut history = RecognitionHistory::new(10);

        history.push("第一句话".to_string());
        history.push("第二句话".to_string());

        // 撤销
        history.undo();
        assert!(history.can_redo());

        // 重试
        let redone = history.redo();
        assert_eq!(redone, Some("第二句话".to_string()));
        assert!(!history.can_redo());

        // 有效历史应该恢复
        let active = history.get_active_history();
        assert_eq!(active, vec!["第一句话", "第二句话"]);
    }

    #[test]
    fn test_max_history() {
        let mut history = RecognitionHistory::new(3);

        history.push("第一句话".to_string());
        history.push("第二句话".to_string());
        history.push("第三句话".to_string());
        history.push("第四句话".to_string());

        // 应该只保留最近的 3 条
        assert_eq!(history.len(), 3);

        let active = history.get_active_history();
        assert_eq!(active, vec!["第二句话", "第三句话", "第四句话"]);
    }

    #[test]
    fn test_clear() {
        let mut history = RecognitionHistory::new(10);

        history.push("第一句话".to_string());
        history.push("第二句话".to_string());

        history.clear();
        assert_eq!(history.len(), 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
