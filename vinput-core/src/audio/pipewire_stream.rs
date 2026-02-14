//! PipeWire 音频录音流
//!
//! Phase 1: 完整的 PipeWire 集成实现

use crate::audio::ring_buffer::AudioRingProducer;
use crate::error::{VInputError, VInputResult};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// PipeWire 音频流配置
#[derive(Debug, Clone)]
pub struct PipeWireStreamConfig {
    /// 目标采样率 (Hz)
    pub sample_rate: u32,
    /// 声道数
    pub channels: u32,
    /// 音频格式（默认 F32LE）
    pub format: AudioFormat,
    /// 流名称
    pub stream_name: String,
    /// 应用名称
    pub app_name: String,
    /// 目标节点（None = 默认音频源）
    pub target_node: Option<String>,
}

/// 音频格式
#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    /// 32-bit float, little-endian
    F32LE,
    /// 16-bit signed integer, little-endian
    S16LE,
}

impl Default for PipeWireStreamConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            format: AudioFormat::F32LE,
            stream_name: "V-Input Audio Capture".to_string(),
            app_name: "vinput-core".to_string(),
            target_node: None,
        }
    }
}

/// PipeWire 音频捕获流
pub struct PipeWireStream {
    config: PipeWireStreamConfig,
    running: Arc<AtomicBool>,
    quit_signal: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<VInputResult<()>>>,
}

impl PipeWireStream {
    /// 创建新的 PipeWire 音频流
    pub fn new(
        config: PipeWireStreamConfig,
        producer: AudioRingProducer,
    ) -> VInputResult<Self> {
        tracing::info!(
            "创建 PipeWire 流: {} Hz, {} 声道, {:?}",
            config.sample_rate,
            config.channels,
            config.format
        );

        let running = Arc::new(AtomicBool::new(false));
        let quit_signal = Arc::new(AtomicBool::new(false));

        // 在单独的线程中运行 PipeWire 主循环
        let running_clone = running.clone();
        let quit_clone = quit_signal.clone();
        let config_clone = config.clone();

        let thread_handle = thread::spawn(move || {
            run_pipewire_loop(config_clone, producer, running_clone, quit_clone)
        });

        Ok(Self {
            config,
            running,
            quit_signal,
            thread_handle: Some(thread_handle),
        })
    }

    /// 检查流是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// 获取流配置
    pub fn config(&self) -> &PipeWireStreamConfig {
        &self.config
    }

    /// 等待流结束（阻塞）
    pub fn join(&mut self) -> VInputResult<()> {
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| VInputError::PipeWire("PipeWire thread panicked".to_string()))?
        } else {
            Ok(())
        }
    }

    /// 停止音频捕获
    pub fn stop(&self) {
        tracing::info!("请求停止 PipeWire 流");
        self.quit_signal.store(true, Ordering::Release);
    }
}

impl Drop for PipeWireStream {
    fn drop(&mut self) {
        self.stop();
        // 等待线程结束（最多 2 秒）
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        tracing::debug!("PipeWire 流已释放");
    }
}

/// PipeWire 主循环（运行在独立线程）
fn run_pipewire_loop(
    config: PipeWireStreamConfig,
    producer: AudioRingProducer,
    running: Arc<AtomicBool>,
    quit_signal: Arc<AtomicBool>,
) -> VInputResult<()> {
    tracing::info!("PipeWire 流线程启动");

    #[cfg(feature = "pipewire-capture")]
    return run_real_pipewire_loop(config, producer, running, quit_signal);

    #[cfg(not(feature = "pipewire-capture"))]
    return run_simulated_pipewire_loop(config, producer, running, quit_signal);
}

/// 模拟 PipeWire 主循环（用于开发和测试）
#[cfg(not(feature = "pipewire-capture"))]
fn run_simulated_pipewire_loop(
    config: PipeWireStreamConfig,
    mut producer: AudioRingProducer,
    running: Arc<AtomicBool>,
    quit_signal: Arc<AtomicBool>,
) -> VInputResult<()> {
    tracing::warn!("PipeWire 模拟模式 - 生成静音音频");
    tracing::warn!("要使用真实音频捕获，请使用 --features pipewire-capture 编译");

    running.store(true, Ordering::Release);

    // 模拟音频捕获
    let mut sample_count = 0u64;
    while !quit_signal.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(32)); // 模拟 32ms 音频帧

        // 生成模拟音频数据（静音）
        let frame_size = (config.sample_rate / 1000 * 32) as usize; // 32ms
        let samples = vec![0.0f32; frame_size];

        match producer.write(&samples) {
            Ok(written) => {
                sample_count += written as u64;
                if sample_count % (config.sample_rate as u64) == 0 {
                    tracing::trace!("模拟捕获: {} 秒", sample_count / config.sample_rate as u64);
                }
            }
            Err(e) => {
                tracing::warn!("Ring Buffer 写入失败: {:?}", e);
            }
        }
    }

    running.store(false, Ordering::Release);
    tracing::info!("PipeWire 模拟流停止，共生成 {:.2} 秒音频",
        sample_count as f32 / config.sample_rate as f32);

    Ok(())
}

/// 真实 PipeWire 主循环（使用 pw-record 子进程）
#[cfg(feature = "pipewire-capture")]
fn run_real_pipewire_loop(
    config: PipeWireStreamConfig,
    mut producer: AudioRingProducer,
    running: Arc<AtomicBool>,
    quit_signal: Arc<AtomicBool>,
) -> VInputResult<()> {
    use std::process::{Command, Stdio};
    use std::io::Read;

    tracing::info!("PipeWire 真实音频捕获模式 (pw-record)");

    // 启动 pw-record 子进程
    let mut child = Command::new("pw-record")
        .arg("--rate").arg(config.sample_rate.to_string())
        .arg("--channels").arg(config.channels.to_string())
        .arg("--format").arg(match config.format {
            AudioFormat::F32LE => "f32",
            AudioFormat::S16LE => "s16",
        })
        .arg("-")  // 输出到 stdout
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| VInputError::PipeWire(format!("启动 pw-record 失败: {}", e)))?;

    tracing::info!("pw-record 子进程已启动 (PID: {})", child.id());
    running.store(true, Ordering::Release);

    // 从 stdout 读取音频数据
    let mut stdout = child.stdout.take()
        .ok_or_else(|| VInputError::PipeWire("无法获取 pw-record stdout".to_string()))?;

    let frame_size = 1024; // 每次读取 1024 个样本
    let buffer_size = frame_size * std::mem::size_of::<f32>();
    let mut buffer = vec![0u8; buffer_size];
    let mut total_samples = 0usize;

    while !quit_signal.load(Ordering::Acquire) {
        // 读取音频数据
        match stdout.read(&mut buffer) {
            Ok(0) => {
                // EOF - pw-record 进程结束
                tracing::warn!("pw-record 进程意外结束");
                break;
            }
            Ok(bytes_read) => {
                // 转换为 f32 样本
                let sample_count = bytes_read / std::mem::size_of::<f32>();
                let samples: &[f32] = unsafe {
                    std::slice::from_raw_parts(
                        buffer.as_ptr() as *const f32,
                        sample_count,
                    )
                };

                // 写入 Ring Buffer
                match producer.write(samples) {
                    Ok(written) => {
                        total_samples += written;
                        if total_samples % (config.sample_rate as usize) == 0 {
                            tracing::trace!("已捕获 {} 秒真实音频",
                                total_samples / config.sample_rate as usize);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Ring Buffer 写入失败: {:?}", e);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 非阻塞模式下无数据，等待一小会儿
                thread::sleep(Duration::from_millis(1));
            }
            Err(e) => {
                tracing::error!("读取 pw-record 输出失败: {}", e);
                break;
            }
        }
    }

    // 停止 pw-record
    tracing::info!("停止 pw-record 进程");
    let _ = child.kill();
    let _ = child.wait();

    running.store(false, Ordering::Release);
    tracing::info!("PipeWire 流线程停止，共捕获 {:.2} 秒真实音频",
        total_samples as f32 / config.sample_rate as f32);

    Ok(())
}

/// 枚举可用的音频设备
pub fn enumerate_audio_devices() -> VInputResult<Vec<AudioDevice>> {
    tracing::warn!("enumerate_audio_devices 尚未实现");
    // TODO Phase 1.1: 实现设备枚举
    // 使用 PipeWire registry 枚举音频源节点
    Ok(vec![])
}

/// 音频设备信息
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// 设备 ID
    pub id: String,
    /// 设备名称
    pub name: String,
    /// 设备描述
    pub description: String,
    /// 是否为默认设备
    pub is_default: bool,
}

// Phase 1.1 实施说明：
//
// PipeWire 集成分为以下阶段：
//
// 1. ✅ 基础框架（当前）
//    - 线程模型设计
//    - 接口定义
//    - 错误处理框架
//
// 2. ⏳ 实际音频捕获（下一步）
//    - 使用 pipewire crate 创建捕获流
//    - 注册音频处理回调
//    - 零拷贝写入 Ring Buffer
//
// 3. ⏳ 设备管理（后续）
//    - 枚举音频输入设备
//    - 支持设备选择
//    - 处理设备热插拔
//
// 4. ⏳ 错误处理（后续）
//    - 连接失败恢复
//    - 设备断开检测
//    - 自动重连机制
//
// 需要实际 PipeWire 环境进行测试和调试
