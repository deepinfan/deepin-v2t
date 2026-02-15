//! Audio 音频捕获模块
//!
//! 基于 PipeWire 的实时音频录制和 Ring Buffer 传输

pub mod ring_buffer;
pub mod pipewire_stream;
pub mod audio_queue;

pub use ring_buffer::{AudioRingBuffer, AudioRingBufferConfig, AudioRingConsumer, AudioRingProducer};
pub use pipewire_stream::{PipeWireStream, PipeWireStreamConfig, AudioDevice, enumerate_audio_devices};
pub use audio_queue::{AudioQueueManager, AudioQueueConfig, AudioQueueStats};
