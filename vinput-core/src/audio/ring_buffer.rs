//! Lock-free SPSC Ring Buffer 封装（MVP 版本）
//!
//! Phase 0: 使用简化实现验证接口设计
//! Phase 1: 完整的 rtrb 零拷贝实现

use crate::error::{VInputError, VInputResult};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 音频环形缓冲区配置
#[derive(Debug, Clone)]
pub struct AudioRingBufferConfig {
    /// 缓冲区容量（采样点数）
    pub capacity: usize,
}

impl Default for AudioRingBufferConfig {
    fn default() -> Self {
        Self {
            // 默认 1 秒音频 @ 16kHz
            capacity: 16000,
        }
    }
}

/// 音频环形缓冲区（生产者 + 消费者对）
pub struct AudioRingBuffer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    capacity: usize,
    overrun_count: Arc<std::sync::atomic::AtomicU64>,
}

impl AudioRingBuffer {
    /// 创建新的音频环形缓冲区
    pub fn new(config: AudioRingBufferConfig) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(config.capacity))),
            capacity: config.capacity,
            overrun_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// 分离为生产者和消费者
    pub fn split(self) -> (AudioRingProducer, AudioRingConsumer) {
        (
            AudioRingProducer {
                buffer: self.buffer.clone(),
                capacity: self.capacity,
                overrun_count: self.overrun_count.clone(),
            },
            AudioRingConsumer {
                buffer: self.buffer,
                capacity: self.capacity,
                overrun_count: self.overrun_count,
            },
        )
    }
}

/// 音频环形缓冲区生产者（用于音频回调）
pub struct AudioRingProducer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    capacity: usize,
    overrun_count: Arc<std::sync::atomic::AtomicU64>,
}

// MVP: 使用 Mutex，Phase 1 将使用真正的 lock-free 实现
unsafe impl Send for AudioRingProducer {}

impl AudioRingProducer {
    /// 写入音频数据
    pub fn write(&mut self, samples: &[f32]) -> VInputResult<usize> {
        let mut buffer = self.buffer.lock().unwrap();

        let available_space = self.capacity.saturating_sub(buffer.len());

        if available_space < samples.len() {
            // 缓冲区空间不足
            let can_write = available_space;
            buffer.extend(samples.iter().take(can_write).copied());

            let lost = samples.len() - can_write;
            self.overrun_count
                .fetch_add(lost as u64, std::sync::atomic::Ordering::Relaxed);

            return Err(VInputError::RingBufferOverrun {
                lost_frames: lost as u64,
            });
        }

        buffer.extend(samples.iter().copied());
        Ok(samples.len())
    }

    /// 获取可写空间大小
    #[inline]
    pub fn free_space(&self) -> usize {
        let buffer = self.buffer.lock().unwrap();
        self.capacity.saturating_sub(buffer.len())
    }

    /// 获取 overrun 计数
    pub fn overrun_count(&self) -> u64 {
        self.overrun_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// 音频环形缓冲区消费者（用于处理线程）
pub struct AudioRingConsumer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    capacity: usize,
    overrun_count: Arc<std::sync::atomic::AtomicU64>,
}

unsafe impl Send for AudioRingConsumer {}

impl AudioRingConsumer {
    /// 读取音频数据到提供的缓冲区
    ///
    /// 返回实际读取的样本数
    #[inline]
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let mut buffer = self.buffer.lock().unwrap();
        let to_read = output.len().min(buffer.len());

        for i in 0..to_read {
            output[i] = buffer.pop_front().unwrap();
        }

        to_read
    }

    /// 非阻塞读取，返回当前可用的所有数据
    pub fn read_available(&mut self, max_samples: usize) -> Vec<f32> {
        let mut buffer = self.buffer.lock().unwrap();
        let to_read = max_samples.min(buffer.len());

        buffer.drain(..to_read).collect()
    }

    /// 获取当前可读样本数
    #[inline]
    pub fn available_samples(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// 获取缓冲区容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 获取 overrun 计数
    pub fn overrun_count(&self) -> u64 {
        self.overrun_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 重置 overrun 计数
    pub fn reset_overrun_count(&self) {
        self.overrun_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let config = AudioRingBufferConfig { capacity: 1024 };
        let ring = AudioRingBuffer::new(config);
        let (mut producer, mut consumer) = ring.split();

        // 写入数据
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        assert!(producer.write(&samples).is_ok());

        // 读取数据
        let mut buffer = vec![0.0; 4];
        let read = consumer.read(&mut buffer);
        assert_eq!(read, 4);
        assert_eq!(buffer, samples);
    }

    #[test]
    fn test_ring_buffer_overrun() {
        let config = AudioRingBufferConfig { capacity: 10 };
        let ring = AudioRingBuffer::new(config);
        let (mut producer, _consumer) = ring.split();

        // 写入超过容量的数据
        let samples = vec![1.0; 20];
        let result = producer.write(&samples);

        assert!(result.is_err());
        assert!(producer.overrun_count() > 0);
    }

    #[test]
    fn test_ring_buffer_available() {
        let config = AudioRingBufferConfig { capacity: 1024 };
        let ring = AudioRingBuffer::new(config);
        let (mut producer, mut consumer) = ring.split();

        // 写入数据
        let samples = vec![1.0; 100];
        producer.write(&samples).unwrap();

        // 检查可用数据
        assert_eq!(consumer.available_samples(), 100);

        // 读取部分数据
        let data = consumer.read_available(50);
        assert_eq!(data.len(), 50);
        assert_eq!(consumer.available_samples(), 50);
    }
}

// Phase 0 说明：
// 此版本使用 Mutex + VecDeque 实现，适用于 MVP 验证
//
// Phase 1 改进：
// - 使用 rtrb 的真正 lock-free SPSC 实现
// - 零拷贝音频传输
// - RT-safe 音频回调
//
// 当前实现已足够验证：
// - Ring Buffer 接口设计正确
// - 生产者/消费者模式可行
// - Overrun 检测机制有效
