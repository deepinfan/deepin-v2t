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

/// 真实 PipeWire 主循环（需要 PipeWire 环境）
#[cfg(feature = "pipewire-capture")]
fn run_real_pipewire_loop(
    config: PipeWireStreamConfig,
    producer: AudioRingProducer,
    running: Arc<AtomicBool>,
    quit_signal: Arc<AtomicBool>,
) -> VInputResult<()> {
    use pipewire::{
        properties,
        spa::{
            param::audio::{AudioFormat as SpaAudioFormat, AudioInfoRaw},
            utils::Direction,
        },
        stream::{Stream, StreamFlags},
        Context, MainLoop,
    };
    use std::sync::Mutex;

    tracing::info!("PipeWire 真实音频捕获模式");

    // 初始化 PipeWire
    pipewire::init();

    // 创建主循环
    let mainloop = MainLoop::new(None)
        .map_err(|e| VInputError::PipeWire(format!("创建主循环失败: {}", e)))?;
    let context = Context::new(&mainloop)
        .map_err(|e| VInputError::PipeWire(format!("创建上下文失败: {}", e)))?;
    let core = context.connect(None)
        .map_err(|e| VInputError::PipeWire(format!("连接 PipeWire 失败: {}", e)))?;

    // 创建音频流
    let stream_props = properties! {
        "media.type" => "Audio",
        "media.category" => "Capture",
        "media.role" => "Communication",
        "app.name" => config.app_name.as_str(),
    };

    let stream = Stream::new(&core, config.stream_name.as_str(), stream_props)
        .map_err(|e| VInputError::PipeWire(format!("创建流失败: {}", e)))?;

    // 设置音频参数
    let audio_info = AudioInfoRaw::new()
        .format(match config.format {
            AudioFormat::F32LE => SpaAudioFormat::F32LE,
            AudioFormat::S16LE => SpaAudioFormat::S16LE,
        })
        .rate(config.sample_rate)
        .channels(config.channels);

    // 使用 Arc<Mutex> 包装 producer 以便在回调中使用
    let producer_shared = Arc::new(Mutex::new(producer));
    let producer_for_callback = producer_shared.clone();

    // 注册流事件监听器
    let _listener = stream
        .add_local_listener()
        .process({
            let producer = producer_for_callback;
            move |stream| {
                // 获取音频缓冲区
                if let Some(mut buffer) = stream.dequeue_buffer() {
                    let datas = buffer.datas_mut();
                    if !datas.is_empty() {
                        // 获取第一个数据块
                        let data = &datas[0];
                        if let Some(chunk) = data.chunk() {
                            let size = chunk.size();

                            if size > 0 {
                                // 读取音频数据
                                if let Some(slice) = data.data() {
                                    let samples = unsafe {
                                        std::slice::from_raw_parts(
                                            slice.as_ptr() as *const f32,
                                            size as usize / std::mem::size_of::<f32>(),
                                        )
                                    };

                                    // 写入 Ring Buffer
                                    if let Ok(mut prod) = producer.lock() {
                                        if let Err(e) = prod.write(samples) {
                                            tracing::warn!("Ring Buffer 写入失败: {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
        .register()
        .map_err(|e| VInputError::PipeWire(format!("注册监听器失败: {}", e)))?;

    // 连接流
    let mut params = vec![audio_info.into()];
    stream.connect(
        Direction::Input,
        None, // 使用默认音频源
        StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
        &mut params,
    )
    .map_err(|e| VInputError::PipeWire(format!("连接流失败: {}", e)))?;

    tracing::info!("PipeWire 流已连接，开始捕获真实音频");
    running.store(true, Ordering::Release);

    // 运行主循环
    while !quit_signal.load(Ordering::Acquire) {
        mainloop.iterate(Duration::from_millis(10));
    }

    running.store(false, Ordering::Release);
    tracing::info!("PipeWire 流线程停止");

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
