//! 音频队列管理器
//!
//! 管理音频数据在 Capture → VAD → ASR 管道中的传递

use crate::audio::ring_buffer::{AudioRingBuffer, AudioRingBufferConfig, AudioRingConsumer, AudioRingProducer};
use crate::error::VInputResult;

/// 音频队列管理器配置
#[derive(Debug, Clone)]
pub struct AudioQueueConfig {
    /// 捕获到 VAD 的队列大小（采样点数）
    pub capture_to_vad_capacity: usize,
    /// VAD 到 ASR 的队列大小（采样点数）
    pub vad_to_asr_capacity: usize,
    /// 背压阈值（队列使用率百分比，0-100）
    pub backpressure_threshold: u8,
}

impl Default for AudioQueueConfig {
    fn default() -> Self {
        Self {
            // 捕获到 VAD：1 秒缓冲 @ 16kHz
            capture_to_vad_capacity: 16000,
            // VAD 到 ASR：2 秒缓冲 @ 16kHz
            vad_to_asr_capacity: 32000,
            // 背压阈值：80%
            backpressure_threshold: 80,
        }
    }
}

/// 音频队列管理器
///
/// 管理两个队列：
/// 1. Capture → VAD
/// 2. VAD → ASR
pub struct AudioQueueManager {
    config: AudioQueueConfig,

    /// Capture → VAD 队列（生产者端）
    capture_to_vad_producer: AudioRingProducer,
    /// Capture → VAD 队列（消费者端）
    capture_to_vad_consumer: AudioRingConsumer,

    /// VAD → ASR 队列（生产者端）
    vad_to_asr_producer: AudioRingProducer,
    /// VAD → ASR 队列（消费者端）
    vad_to_asr_consumer: AudioRingConsumer,

    /// 背压控制标志
    backpressure_active: bool,
}

impl AudioQueueManager {
    /// 创建新的音频队列管理器
    pub fn new(config: AudioQueueConfig) -> Self {
        // 创建 Capture → VAD 队列
        let capture_to_vad = AudioRingBuffer::new(AudioRingBufferConfig {
            capacity: config.capture_to_vad_capacity,
        });
        let (capture_to_vad_producer, capture_to_vad_consumer) = capture_to_vad.split();

        // 创建 VAD → ASR 队列
        let vad_to_asr = AudioRingBuffer::new(AudioRingBufferConfig {
            capacity: config.vad_to_asr_capacity,
        });
        let (vad_to_asr_producer, vad_to_asr_consumer) = vad_to_asr.split();

        Self {
            config,
            capture_to_vad_producer,
            capture_to_vad_consumer,
            vad_to_asr_producer,
            vad_to_asr_consumer,
            backpressure_active: false,
        }
    }

    /// 从音频捕获写入数据（由捕获线程调用）
    pub fn write_from_capture(&mut self, samples: &[f32]) -> VInputResult<usize> {
        // 检查背压
        if self.should_apply_backpressure_capture() {
            self.backpressure_active = true;
            // 策略：丢弃新帧，保留旧帧（保持音频连续性）
            return Ok(0);
        }

        self.backpressure_active = false;
        self.capture_to_vad_producer.write(samples)
    }

    /// 读取数据供 VAD 处理（由 VAD 线程调用）
    pub fn read_for_vad(&mut self, output: &mut [f32]) -> usize {
        self.capture_to_vad_consumer.read(output)
    }

    /// 从 VAD 写入数据（由 VAD 线程调用）
    pub fn write_from_vad(&mut self, samples: &[f32]) -> VInputResult<usize> {
        // 检查背压
        if self.should_apply_backpressure_vad() {
            // 策略：丢弃新帧
            return Ok(0);
        }

        self.vad_to_asr_producer.write(samples)
    }

    /// 读取数据供 ASR 处理（由 ASR 线程调用）
    pub fn read_for_asr(&mut self, max_samples: usize) -> Vec<f32> {
        self.vad_to_asr_consumer.read_available(max_samples)
    }

    /// 检查 Capture → VAD 队列是否应该应用背压
    fn should_apply_backpressure_capture(&self) -> bool {
        let usage = self.capture_to_vad_usage_percent();
        usage > self.config.backpressure_threshold
    }

    /// 检查 VAD → ASR 队列是否应该应用背压
    fn should_apply_backpressure_vad(&self) -> bool {
        let usage = self.vad_to_asr_usage_percent();
        usage > self.config.backpressure_threshold
    }

    /// 获取 Capture → VAD 队列使用率（百分比）
    pub fn capture_to_vad_usage_percent(&self) -> u8 {
        let used = self.capture_to_vad_consumer.available_samples();
        let capacity = self.capture_to_vad_consumer.capacity();
        ((used as f32 / capacity as f32) * 100.0) as u8
    }

    /// 获取 VAD → ASR 队列使用率（百分比）
    pub fn vad_to_asr_usage_percent(&self) -> u8 {
        let used = self.vad_to_asr_consumer.available_samples();
        let capacity = self.vad_to_asr_consumer.capacity();
        ((used as f32 / capacity as f32) * 100.0) as u8
    }

    /// 是否正在应用背压
    pub fn is_backpressure_active(&self) -> bool {
        self.backpressure_active
    }

    /// 获取 Capture → VAD 队列的 overrun 计数
    pub fn capture_to_vad_overrun_count(&self) -> u64 {
        self.capture_to_vad_consumer.overrun_count()
    }

    /// 获取 VAD → ASR 队列的 overrun 计数
    pub fn vad_to_asr_overrun_count(&self) -> u64 {
        self.vad_to_asr_consumer.overrun_count()
    }

    /// 重置 overrun 计数器
    pub fn reset_overrun_counters(&self) {
        self.capture_to_vad_consumer.reset_overrun_count();
        self.vad_to_asr_consumer.reset_overrun_count();
    }

    /// 获取队列统计信息
    pub fn get_stats(&self) -> AudioQueueStats {
        AudioQueueStats {
            capture_to_vad_usage: self.capture_to_vad_usage_percent(),
            vad_to_asr_usage: self.vad_to_asr_usage_percent(),
            capture_to_vad_overruns: self.capture_to_vad_overrun_count(),
            vad_to_asr_overruns: self.vad_to_asr_overrun_count(),
            backpressure_active: self.backpressure_active,
        }
    }
}

/// 音频队列统计信息
#[derive(Debug, Clone, Copy)]
pub struct AudioQueueStats {
    /// Capture → VAD 队列使用率（百分比）
    pub capture_to_vad_usage: u8,
    /// VAD → ASR 队列使用率（百分比）
    pub vad_to_asr_usage: u8,
    /// Capture → VAD 队列 overrun 计数
    pub capture_to_vad_overruns: u64,
    /// VAD → ASR 队列 overrun 计数
    pub vad_to_asr_overruns: u64,
    /// 是否正在应用背压
    pub backpressure_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_queue_manager_basic() {
        let mut manager = AudioQueueManager::new(AudioQueueConfig::default());

        // 从捕获写入数据
        let samples = vec![1.0; 100];
        let written = manager.write_from_capture(&samples).unwrap();
        assert_eq!(written, 100);

        // VAD 读取数据
        let mut buffer = vec![0.0; 100];
        let read = manager.read_for_vad(&mut buffer);
        assert_eq!(read, 100);
        assert_eq!(buffer, samples);
    }

    #[test]
    fn test_vad_to_asr_queue() {
        let mut manager = AudioQueueManager::new(AudioQueueConfig::default());

        // VAD 写入数据
        let samples = vec![2.0; 200];
        let written = manager.write_from_vad(&samples).unwrap();
        assert_eq!(written, 200);

        // ASR 读取数据
        let data = manager.read_for_asr(200);
        assert_eq!(data.len(), 200);
        assert_eq!(data[0], 2.0);
    }

    #[test]
    fn test_backpressure() {
        let config = AudioQueueConfig {
            capture_to_vad_capacity: 100,
            vad_to_asr_capacity: 100,
            backpressure_threshold: 80,
        };
        let mut manager = AudioQueueManager::new(config);

        // 写入 90 个样本（超过 80% 阈值）
        let samples = vec![1.0; 90];
        manager.write_from_capture(&samples).unwrap();

        // 再写入应该触发背压
        let more_samples = vec![1.0; 20];
        let written = manager.write_from_capture(&more_samples).unwrap();
        assert_eq!(written, 0); // 被丢弃
        assert!(manager.is_backpressure_active());
    }

    #[test]
    fn test_usage_percent() {
        let config = AudioQueueConfig {
            capture_to_vad_capacity: 1000,
            vad_to_asr_capacity: 1000,
            backpressure_threshold: 80,
        };
        let mut manager = AudioQueueManager::new(config);

        // 写入 500 个样本
        let samples = vec![1.0; 500];
        manager.write_from_capture(&samples).unwrap();

        // 使用率应该是 50%
        assert_eq!(manager.capture_to_vad_usage_percent(), 50);
    }

    #[test]
    fn test_queue_stats() {
        let mut manager = AudioQueueManager::new(AudioQueueConfig::default());

        // Write 200 samples (200/16000 = 1.25% usage)
        let samples = vec![1.0; 200];
        manager.write_from_capture(&samples).unwrap();

        let stats = manager.get_stats();
        assert!(stats.capture_to_vad_usage > 0);
        assert_eq!(stats.vad_to_asr_usage, 0);
        assert!(!stats.backpressure_active);
    }
}
