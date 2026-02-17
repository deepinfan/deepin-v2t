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
    mut producer: AudioRingProducer,
    running: Arc<AtomicBool>,
    quit_signal: Arc<AtomicBool>,
) -> VInputResult<()> {
    use std::process::{Command, Stdio};
    use std::io::Read;

    tracing::info!("PipeWire 流线程启动");
    tracing::info!("PipeWire 真实音频捕获模式 (pw-record)");

    // 启动 pw-record 子进程
    let mut child = Command::new("pw-record")
        .arg("--rate").arg(config.sample_rate.to_string())
        .arg("--channels").arg(config.channels.to_string())
        .arg("--format").arg(match config.format {
            AudioFormat::F32LE => "f32",
            AudioFormat::S16LE => "s16",
        })
        .arg("--quality").arg("8")  // 重采样质量：8（高质量，平衡性能）
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
    use std::process::Command;

    tracing::info!("开始枚举音频输入设备");

    // 使用 pactl 枚举音频源
    let output = Command::new("pactl")
        .args(&["list", "sources", "short"])
        .output()
        .map_err(|e| VInputError::PipeWire(
            format!("执行 pactl 失败: {}", e)
        ))?;

    if !output.status.success() {
        return Err(VInputError::PipeWire(
            "pactl 命令执行失败".to_string()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    // 解析 pactl 输出
    // 格式: ID\tNAME\tDRIVER\tFORMAT\tSTATE
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let id = parts[0].trim();
        let name = parts[1].trim();

        // 过滤掉 monitor 设备（这些是输出设备的监听器）
        if name.contains(".monitor") {
            continue;
        }

        // 生成友好的描述
        let description = generate_device_description(name);

        devices.push(AudioDevice {
            id: id.to_string(),
            name: name.to_string(),
            description,
            is_default: false, // 稍后标记默认设备
        });
    }

    // 获取默认设备
    if let Ok(default_output) = Command::new("pactl")
        .args(&["get-default-source"])
        .output()
    {
        if default_output.status.success() {
            let default_name = String::from_utf8_lossy(&default_output.stdout).trim().to_string();
            for device in &mut devices {
                if device.name == default_name {
                    device.is_default = true;
                    break;
                }
            }
        }
    }

    tracing::info!("找到 {} 个音频输入设备", devices.len());
    for device in &devices {
        tracing::debug!("  - {} ({}){}",
            device.description,
            device.name,
            if device.is_default { " [默认]" } else { "" }
        );
    }

    Ok(devices)
}

/// 生成友好的设备描述
fn generate_device_description(name: &str) -> String {
    // 尝试从设备名称提取友好描述
    if name.starts_with("alsa_input.") {
        // ALSA 设备
        if name.contains("Mic1") {
            return "内置麦克风 1".to_string();
        } else if name.contains("Mic2") {
            return "内置麦克风 2".to_string();
        } else if name.contains("Headset") {
            return "耳机麦克风".to_string();
        } else if name.contains("USB") {
            return "USB 麦克风".to_string();
        }
        return "ALSA 音频输入".to_string();
    } else if name.starts_with("bluez_") {
        return "蓝牙音频输入".to_string();
    } else if name.contains("echo-cancel") {
        return "回声消除音频源".to_string();
    } else if name == "null-sink" {
        return "空音频源".to_string();
    }

    // 默认使用设备名称
    name.to_string()
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
