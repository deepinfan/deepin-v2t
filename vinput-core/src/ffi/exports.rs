//! FFI 导出函数 - 完整实现版本
//!
//! Rust cdylib FFI 接口，供 Fcitx5 C++ 插件调用
//! 完整集成: StreamingPipeline + ITN + Punctuation + Hotwords

use super::safety::{check_null, check_null_mut, ffi_safe_call};
use super::types::{VInputCommand, VInputEvent, VInputEventType, VInputFFIResult};
use crate::audio::{AudioRingBuffer, AudioRingBufferConfig, PipeWireStream, PipeWireStreamConfig};
use crate::config::VInputConfig;
use crate::hotwords::HotwordsEngine;
use crate::itn::{ITNEngine, ITNMode};
use crate::streaming::{StreamingConfig, StreamingPipeline};
use std::collections::VecDeque;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use std::thread;

/// 全局 V-Input Core 实例
static VINPUT_CORE: Mutex<Option<VInputCoreState>> = Mutex::new(None);

/// V-Input Core 完整状态
struct VInputCoreState {
    /// 流式识别管道
    pipeline: Arc<Mutex<StreamingPipeline>>,
    /// ITN 引擎
    itn_engine: ITNEngine,
    /// 热词引擎
    hotwords_engine: Option<HotwordsEngine>,
    /// 命令队列
    command_queue: VecDeque<VInputCommand>,
    /// 录音状态
    is_recording: bool,
    /// 音频处理线程句柄
    audio_thread: Option<thread::JoinHandle<()>>,
    /// 停止信号
    stop_signal: Arc<Mutex<bool>>,
    /// PipeWire 音频流
    pipewire_stream: Option<PipeWireStream>,
}

impl VInputCoreState {
    fn new() -> crate::error::VInputResult<Self> {
        tracing::info!("初始化 V-Input Core (完整版本)");

        // 加载配置
        let config = match VInputConfig::load() {
            Ok(cfg) => {
                tracing::info!("✅ 成功加载配置文件");
                cfg
            }
            Err(e) => {
                tracing::error!("❌ 加载配置文件失败: {}，使用默认配置", e);
                VInputConfig::default()
            }
        };

        // 创建流式管道
        tracing::info!("🔧 创建 StreamingPipeline，标点配置: pause_ratio={}, min_tokens={}",
            config.punctuation.streaming_pause_ratio,
            config.punctuation.streaming_min_tokens
        );

        let streaming_config = StreamingConfig {
            vad_config: config.vad.clone(),
            asr_config: config.asr.clone(),
            punctuation_profile: config.punctuation.clone(),
            max_silence_duration_ms: 3000,
            enable_endpoint_detection: true,
        };
        let pipeline = StreamingPipeline::new(streaming_config)?;

        // 创建后处理引擎
        let itn_engine = ITNEngine::new(ITNMode::Auto);

        // 创建热词引擎（可选）
        let hotwords_engine = if !config.hotwords.words.is_empty() {
            let mut engine = HotwordsEngine::new();

            // 添加所有配置的热词
            for (word, weight) in &config.hotwords.words {
                if let Err(e) = engine.add_hotword(word.clone(), *weight) {
                    tracing::warn!("添加热词失败 '{}': {}", word, e);
                }
            }

            tracing::info!("热词引擎初始化成功，加载 {} 个热词", engine.count());
            Some(engine)
        } else {
            tracing::info!("未配置热词，跳过热词引擎初始化");
            None
        };

        Ok(Self {
            pipeline: Arc::new(Mutex::new(pipeline)),
            itn_engine,
            hotwords_engine,
            command_queue: VecDeque::new(),
            is_recording: false,
            audio_thread: None,
            stop_signal: Arc::new(Mutex::new(false)),
            pipewire_stream: None,
        })
    }

    /// 启动录音
    fn start_recording(&mut self) {
        if self.is_recording {
            tracing::warn!("已经在录音中");
            return;
        }

        tracing::info!("启动录音和识别");
        self.is_recording = true;
        *self.stop_signal.lock().unwrap() = false;

        // 创建音频环形缓冲区 (1 秒 @ 16kHz = 16000 samples)
        let ring_buffer_config = AudioRingBufferConfig {
            capacity: 16000,
        };
        let ring_buffer = AudioRingBuffer::new(ring_buffer_config);
        let (producer, consumer) = ring_buffer.split();

        // 创建 PipeWire 音频流
        let pw_config = PipeWireStreamConfig {
            sample_rate: 16000,
            channels: 1,
            ..Default::default()
        };

        match PipeWireStream::new(pw_config, producer) {
            Ok(stream) => {
                tracing::info!("PipeWire 音频流创建成功");
                self.pipewire_stream = Some(stream);
                // 注意：不需要存储 ring_buffer，因为 split() 已经消费了它

                // 启动音频处理线程
                let pipeline = Arc::clone(&self.pipeline);
                let stop_signal = Arc::clone(&self.stop_signal);

                self.audio_thread = Some(thread::spawn(move || {
                    Self::audio_processing_loop(pipeline, consumer, stop_signal);
                }));
            }
            Err(e) => {
                tracing::error!("创建 PipeWire 流失败: {}, 停止录音", e);
                self.is_recording = false;
            }
        }
    }

    /// 音频处理循环（从环形缓冲区读取并送入管道）
    fn audio_processing_loop(
        pipeline: Arc<Mutex<StreamingPipeline>>,
        mut consumer: crate::audio::AudioRingConsumer,
        stop_signal: Arc<Mutex<bool>>,
    ) {
        tracing::info!("音频处理线程启动");

        // 512 samples = 32ms @ 16kHz
        const FRAME_SIZE: usize = 512;
        let mut frame_buffer = vec![0.0f32; FRAME_SIZE];

        loop {
            // 检查停止信号
            if *stop_signal.lock().unwrap() {
                tracing::info!("收到停止信号，退出音频处理");
                break;
            }

            // 从环形缓冲区读取音频
            let samples_read = consumer.read(&mut frame_buffer);

            if samples_read == 0 {
                // 缓冲区为空，短暂休眠
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // 只处理完整的帧
            if samples_read < FRAME_SIZE {
                tracing::debug!("读取到不完整帧: {} samples", samples_read);
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }

            // 送入管道处理
            if let Ok(mut pipe) = pipeline.lock() {
                match pipe.process(&frame_buffer) {
                    Ok(result) => {
                        if !result.partial_result.is_empty() {
                            tracing::debug!("识别中: {}", result.partial_result);
                        }
                    }
                    Err(e) => {
                        tracing::error!("管道处理错误: {}", e);
                        break;
                    }
                }
            }
        }

        tracing::info!("音频处理线程退出");
    }

    /// 停止录音并生成识别结果
    fn stop_recording(&mut self) {
        if !self.is_recording {
            tracing::warn!("没有在录音");
            return;
        }

        tracing::info!("停止录音");
        self.is_recording = false;

        // 停止 PipeWire 流
        if let Some(stream) = self.pipewire_stream.take() {
            stream.stop();
            tracing::debug!("PipeWire 流已停止");
        }

        // 发送停止信号
        *self.stop_signal.lock().unwrap() = true;

        // 等待音频线程结束
        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }

        // 获取识别结果（带智能标点）
        let raw_result_with_punct = if let Ok(mut pipe) = self.pipeline.lock() {
            pipe.get_final_result_with_punctuation()
        } else {
            String::new()
        };

        if raw_result_with_punct.is_empty() {
            tracing::info!("识别结果为空，不生成命令");
            return;
        }

        tracing::info!("🎤 识别结果（含智能标点）: [{}]", raw_result_with_punct);

        // 应用 ITN (文本规范化)
        tracing::info!("📝 开始 ITN 处理...");
        let itn_result = self.itn_engine.process(&raw_result_with_punct);
        let final_result = itn_result.text;

        if !itn_result.changes.is_empty() {
            tracing::info!("✏️  ITN 完成: {} 处变更", itn_result.changes.len());
            for change in &itn_result.changes {
                tracing::info!("    '{}' → '{}'", change.original_text, change.normalized_text);
            }
        } else {
            tracing::info!("📋 ITN: 无需变更（输入已是规范格式）");
        }

        tracing::info!("✅ 最终结果: [{}]", final_result);

        // 生成命令序列
        // 1. 显示候选词（可以有多个候选）
        self.command_queue
            .push_back(VInputCommand::show_candidate(&final_result));

        // 2. 提交最终文本
        self.command_queue
            .push_back(VInputCommand::commit_text(&final_result));

        // 3. 隐藏候选词
        self.command_queue
            .push_back(VInputCommand::hide_candidate());

        tracing::info!("生成 {} 个命令", self.command_queue.len());
    }

    /// 尝试接收命令
    fn try_recv_command(&mut self) -> Option<VInputCommand> {
        self.command_queue.pop_front()
    }
}

/// 初始化 V-Input Core
#[no_mangle]
pub extern "C" fn vinput_core_init() -> VInputFFIResult {
    match ffi_safe_call(|| {
        // 初始化日志
        crate::init_logging();
        tracing::info!("V-Input Core FFI: 初始化");

        let mut core = VINPUT_CORE.lock().unwrap();

        if core.is_some() {
            tracing::warn!("V-Input Core 已经初始化");
            return Ok(VInputFFIResult::Success);
        }

        // 创建 Core 状态
        match VInputCoreState::new() {
            Ok(state) => {
                *core = Some(state);
                tracing::info!("V-Input Core 初始化成功");
                Ok(VInputFFIResult::Success)
            }
            Err(e) => {
                tracing::error!("V-Input Core 初始化失败: {}", e);
                Err(VInputFFIResult::InitFailed)
            }
        }
    }) {
        Ok(result) => result,
        Err(e) => e,
    }
}

/// 关闭 V-Input Core
#[no_mangle]
pub extern "C" fn vinput_core_shutdown() -> VInputFFIResult {
    match ffi_safe_call(|| {
        tracing::info!("V-Input Core FFI: 关闭");

        let mut core = VINPUT_CORE.lock().unwrap();

        if core.is_none() {
            tracing::warn!("V-Input Core 未初始化");
            return Ok(VInputFFIResult::Success);
        }

        // 停止录音（如果正在录音）
        if let Some(ref mut state) = *core {
            if state.is_recording {
                state.stop_recording();
            }
        }

        // 清理资源
        *core = None;

        tracing::info!("V-Input Core 关闭成功");
        Ok(VInputFFIResult::Success)
    }) {
        Ok(result) => result,
        Err(e) => e,
    }
}

/// 发送事件到 V-Input Core
#[no_mangle]
pub extern "C" fn vinput_core_send_event(event: *const VInputEvent) -> VInputFFIResult {
    match ffi_safe_call(|| {
        check_null(event, "event")?;

        let event = unsafe { &*event };

        let mut core_lock = VINPUT_CORE.lock().unwrap();
        let core = core_lock
            .as_mut()
            .ok_or(VInputFFIResult::NotInitialized)?;

        match event.event_type {
            VInputEventType::StartRecording => {
                tracing::info!("接收事件: StartRecording");
                core.start_recording();
            }
            VInputEventType::StopRecording => {
                tracing::info!("接收事件: StopRecording");
                core.stop_recording();
            }
            _ => {
                tracing::debug!("接收事件: {:?} (暂不处理)", event.event_type);
            }
        }

        Ok(VInputFFIResult::Success)
    }) {
        Ok(result) => result,
        Err(e) => e,
    }
}

/// 尝试接收命令（非阻塞）
#[no_mangle]
pub extern "C" fn vinput_core_try_recv_command(command: *mut VInputCommand) -> VInputFFIResult {
    match ffi_safe_call(|| {
        check_null_mut(command, "command")?;

        let mut core_lock = VINPUT_CORE.lock().unwrap();
        let core = core_lock
            .as_mut()
            .ok_or(VInputFFIResult::NotInitialized)?;

        if let Some(cmd) = core.try_recv_command() {
            unsafe {
                *command = cmd;
            }
            tracing::debug!("返回命令: {:?}", unsafe { &*command }.command_type);
            Ok(VInputFFIResult::Success)
        } else {
            Err(VInputFFIResult::NoData)
        }
    }) {
        Ok(result) => result,
        Err(e) => e,
    }
}

/// 释放命令资源
#[no_mangle]
pub extern "C" fn vinput_command_free(command: *mut VInputCommand) {
    if command.is_null() {
        return;
    }

    unsafe {
        let cmd = &mut *command;

        if !cmd.text.is_null() {
            let _ = CString::from_raw(cmd.text);
            cmd.text = std::ptr::null_mut();
            cmd.text_len = 0;
        }
    }
}

/// 获取版本字符串
#[no_mangle]
pub extern "C" fn vinput_core_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}
