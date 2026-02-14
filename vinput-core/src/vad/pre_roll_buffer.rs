//! Pre-roll Buffer - 预回滚缓冲区
//!
//! 在 VAD 触发前缓存音频，防止语音开始时的词语丢失

use crate::vad::config::PreRollConfig;
use std::collections::VecDeque;

/// Pre-roll Buffer 状态
pub struct PreRollBuffer {
    config: PreRollConfig,
    /// 循环缓冲区，存储音频样本
    buffer: VecDeque<f32>,
    /// 是否已启用
    enabled: bool,
}

impl PreRollBuffer {
    /// 创建新的 Pre-roll Buffer
    pub fn new(config: PreRollConfig) -> Self {
        let capacity = if config.enabled {
            config.capacity
        } else {
            0
        };

        Self {
            enabled: config.enabled,
            config,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    /// 添加音频帧到缓冲区
    ///
    /// # 参数
    /// - `samples`: 音频样本 (f32, [-1.0, 1.0])
    pub fn push(&mut self, samples: &[f32]) {
        if !self.enabled {
            return;
        }

        // 添加新样本
        for &sample in samples {
            // 如果缓冲区满了，移除最旧的样本
            if self.buffer.len() >= self.config.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(sample);
        }
    }

    /// 获取缓冲区内的所有音频数据
    ///
    /// # 返回
    /// - `Vec<f32>`: 缓冲的音频样本
    pub fn retrieve(&self) -> Vec<f32> {
        if !self.enabled {
            return Vec::new();
        }

        self.buffer.iter().copied().collect()
    }

    /// 获取缓冲区内最近 N 个样本
    ///
    /// # 参数
    /// - `count`: 要获取的样本数
    ///
    /// # 返回
    /// - `Vec<f32>`: 最近的 N 个样本（如果不足 N 个，返回所有可用样本）
    pub fn retrieve_last(&self, count: usize) -> Vec<f32> {
        if !self.enabled {
            return Vec::new();
        }

        let start = if self.buffer.len() > count {
            self.buffer.len() - count
        } else {
            0
        };

        self.buffer.iter().skip(start).copied().collect()
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
        tracing::debug!("PreRollBuffer cleared");
    }

    /// 重置缓冲区（清空并重新初始化）
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.buffer.reserve(self.config.capacity);
        tracing::debug!("PreRollBuffer reset");
    }

    /// 获取当前缓冲区大小（样本数）
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 缓冲区是否已满
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.config.capacity
    }

    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        self.config.capacity
    }

    /// 计算当前缓冲时长 (ms)
    ///
    /// # 参数
    /// - `sample_rate`: 采样率 (Hz)
    pub fn buffered_duration_ms(&self, sample_rate: u32) -> u64 {
        if sample_rate == 0 {
            return 0;
        }

        let samples = self.buffer.len() as u64;
        (samples * 1000) / sample_rate as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_roll_buffer_basic() {
        let config = PreRollConfig {
            enabled: true,
            duration_ms: 250,
            capacity: 100,
        };

        let mut buffer = PreRollBuffer::new(config);

        // 初始状态
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 100);

        // 添加样本
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        buffer.push(&samples);

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
        assert!(!buffer.is_full());

        // 获取所有样本
        let retrieved = buffer.retrieve();
        assert_eq!(retrieved, samples);
    }

    #[test]
    fn test_pre_roll_buffer_overflow() {
        let config = PreRollConfig {
            enabled: true,
            duration_ms: 250,
            capacity: 10, // 小容量
        };

        let mut buffer = PreRollBuffer::new(config);

        // 添加超过容量的样本
        let samples1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let samples2 = vec![6.0, 7.0, 8.0, 9.0, 10.0];
        let samples3 = vec![11.0, 12.0]; // 总共 12 个样本，超过容量 10

        buffer.push(&samples1);
        buffer.push(&samples2);
        buffer.push(&samples3);

        // 应该保留最新的 10 个样本
        assert_eq!(buffer.len(), 10);
        assert!(buffer.is_full());

        let retrieved = buffer.retrieve();
        assert_eq!(retrieved, vec![3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
    }

    #[test]
    fn test_pre_roll_buffer_retrieve_last() {
        let config = PreRollConfig {
            enabled: true,
            duration_ms: 250,
            capacity: 100,
        };

        let mut buffer = PreRollBuffer::new(config);

        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        buffer.push(&samples);

        // 获取最后 5 个样本
        let last_5 = buffer.retrieve_last(5);
        assert_eq!(last_5, vec![6.0, 7.0, 8.0, 9.0, 10.0]);

        // 请求的样本数超过缓冲区大小
        let last_20 = buffer.retrieve_last(20);
        assert_eq!(last_20, samples); // 应该返回所有可用样本
    }

    #[test]
    fn test_pre_roll_buffer_disabled() {
        let config = PreRollConfig {
            enabled: false,
            duration_ms: 250,
            capacity: 100,
        };

        let mut buffer = PreRollBuffer::new(config);

        // 禁用时不应该存储任何数据
        let samples = vec![1.0, 2.0, 3.0];
        buffer.push(&samples);

        assert!(buffer.is_empty());
        assert_eq!(buffer.retrieve(), Vec::<f32>::new());
    }

    #[test]
    fn test_pre_roll_buffer_clear_and_reset() {
        let config = PreRollConfig {
            enabled: true,
            duration_ms: 250,
            capacity: 100,
        };

        let mut buffer = PreRollBuffer::new(config);

        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        buffer.push(&samples);
        assert_eq!(buffer.len(), 5);

        // 清空
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // 重置
        buffer.push(&samples);
        buffer.reset();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_pre_roll_buffer_duration() {
        let config = PreRollConfig {
            enabled: true,
            duration_ms: 250,
            capacity: 4000, // 250ms @ 16kHz
        };

        let mut buffer = PreRollBuffer::new(config);

        // 添加 16000 个样本（1 秒 @ 16kHz）
        let samples: Vec<f32> = (0..1600).map(|i| i as f32 * 0.001).collect();
        buffer.push(&samples);

        // 计算缓冲时长
        let duration_ms = buffer.buffered_duration_ms(16000);
        assert_eq!(duration_ms, 100); // 1600 samples / 16000 Hz * 1000 = 100ms
    }
}
