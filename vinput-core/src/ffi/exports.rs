//! FFI 导出函数 - 完整实现版本
//!
//! Rust cdylib FFI 接口，供 Fcitx5 C++ 插件调用
//! 完整集成: StreamingPipeline + ITN + Punctuation + Hotwords

use super::safety::{check_null, check_null_mut, ffi_safe_call};
use super::types::{VInputCommand, VInputCommandCallback, VInputEvent, VInputEventType, VInputFFIResult};
use crate::audio::{AudioRingBuffer, AudioRingBufferConfig, PipeWireStream, PipeWireStreamConfig};
use crate::config::VInputConfig;
use crate::hotwords::HotwordsEngine;
use crate::itn::{ITNEngine, ITNMode};
use crate::streaming::{StreamingConfig, StreamingPipeline};
use crate::undo::RecognitionHistory;
use std::collections::VecDeque;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use std::thread;

/// 全局 V-Input Core 实例
static VINPUT_CORE: Mutex<Option<VInputCoreState>> = Mutex::new(None);

/// 全局命令回调函数
static COMMAND_CALLBACK: Mutex<Option<VInputCommandCallback>> = Mutex::new(None);

/// V-Input Core 完整状态
struct VInputCoreState {
    /// 流式识别管道
    pipeline: Arc<Mutex<StreamingPipeline>>,
    /// ITN 引擎（共享，供音频线程使用）
    itn_engine: Arc<Mutex<ITNEngine>>,
    /// 热词引擎
    hotwords_engine: Option<HotwordsEngine>,
    /// 命令队列（共享，供音频线程使用）
    command_queue: Arc<Mutex<VecDeque<VInputCommand>>>,
    /// 识别历史（用于撤销/重试）
    recognition_history: Arc<Mutex<RecognitionHistory>>,
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
            endpoint_config: config.endpoint.clone(),
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
            itn_engine: Arc::new(Mutex::new(itn_engine)),
            hotwords_engine,
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            recognition_history: Arc::new(Mutex::new(RecognitionHistory::new(50))),
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
                let command_queue = Arc::clone(&self.command_queue);
                let itn_engine = Arc::clone(&self.itn_engine);
                let recognition_history = Arc::clone(&self.recognition_history);

                self.audio_thread = Some(thread::spawn(move || {
                    Self::audio_processing_loop(pipeline, consumer, stop_signal, command_queue, itn_engine, recognition_history);
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
        _command_queue: Arc<Mutex<VecDeque<VInputCommand>>>,
        itn_engine: Arc<Mutex<ITNEngine>>,
        recognition_history: Arc<Mutex<RecognitionHistory>>,
    ) {
        tracing::info!("音频处理线程启动");

        // 512 samples = 32ms @ 16kHz
        const FRAME_SIZE: usize = 512;
        let mut frame_buffer = vec![0.0f32; FRAME_SIZE];

        // 混合模式状态：记录已上屏的稳定文本
        let mut last_committed_stable = String::new();

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

                        // 🎯 混合模式：流式上屏稳定文本
                        use crate::streaming::PipelineState;
                        if result.pipeline_state == PipelineState::Recognizing {
                            // 计算新增的稳定文本
                            if result.stable_text.len() > last_committed_stable.len() {
                                let new_stable = &result.stable_text[last_committed_stable.len()..];

                                if !new_stable.is_empty() {
                                    tracing::debug!("📝 上屏稳定文本: [{}]", new_stable);

                                    // 立即上屏新增的稳定文本
                                    if let Some(callback) = *COMMAND_CALLBACK.lock().unwrap() {
                                        let cmd = VInputCommand::commit_text(new_stable);
                                        callback(&cmd as *const VInputCommand);
                                        vinput_command_free(&cmd as *const VInputCommand as *mut VInputCommand);
                                    }

                                    last_committed_stable = result.stable_text.clone();
                                }
                            }

                            // 更新 Preedit 显示不稳定文本
                            if !result.unstable_text.is_empty() {
                                if let Some(callback) = *COMMAND_CALLBACK.lock().unwrap() {
                                    let cmd = VInputCommand::update_preedit(&result.unstable_text);
                                    callback(&cmd as *const VInputCommand);
                                    vinput_command_free(&cmd as *const VInputCommand as *mut VInputCommand);
                                }
                            } else {
                                // 清除 Preedit（如果不稳定文本为空）
                                if let Some(callback) = *COMMAND_CALLBACK.lock().unwrap() {
                                    let cmd = VInputCommand::clear_preedit();
                                    callback(&cmd as *const VInputCommand);
                                    vinput_command_free(&cmd as *const VInputCommand as *mut VInputCommand);
                                }
                            }
                        }

                        // 🎯 检测到句子结束（端点检测）
                        if result.pipeline_state == PipelineState::Completed {
                            tracing::info!("🔔 检测到句子结束，处理最终结果");

                            // 清除 Preedit
                            if let Some(callback) = *COMMAND_CALLBACK.lock().unwrap() {
                                let cmd = VInputCommand::clear_preedit();
                                callback(&cmd as *const VInputCommand);
                                vinput_command_free(&cmd as *const VInputCommand as *mut VInputCommand);
                            }

                            // 获取带标点的最终结果
                            let raw_result_with_punct = pipe.get_final_result_with_punctuation();

                            if !raw_result_with_punct.is_empty() {
                                tracing::info!("🎤 识别结果（含智能标点）: [{}]", raw_result_with_punct);

                                // 应用 ITN
                                let final_result = if let Ok(itn) = itn_engine.lock() {
                                    tracing::info!("📝 开始 ITN 处理...");
                                    let itn_result = itn.process(&raw_result_with_punct);

                                    if !itn_result.changes.is_empty() {
                                        tracing::info!("✏️  ITN 完成: {} 处变更", itn_result.changes.len());
                                        for change in &itn_result.changes {
                                            tracing::info!("    '{}' → '{}'", change.original_text, change.normalized_text);
                                        }
                                    } else {
                                        tracing::info!("📋 ITN: 无需变更（输入已是规范格式）");
                                    }

                                    itn_result.text
                                } else {
                                    raw_result_with_punct
                                };

                                tracing::info!("✅ 最终结果: [{}]", final_result);

                                // 计算剩余未上屏的文本
                                let remaining_text = if final_result.len() > last_committed_stable.len() {
                                    &final_result[last_committed_stable.len()..]
                                } else {
                                    ""
                                };

                                if !remaining_text.is_empty() {
                                    tracing::info!("📝 上屏剩余文本: [{}]", remaining_text);

                                    // 上屏剩余文本
                                    if let Some(callback) = *COMMAND_CALLBACK.lock().unwrap() {
                                        let cmd = VInputCommand::commit_text(remaining_text);
                                        callback(&cmd as *const VInputCommand);
                                        vinput_command_free(&cmd as *const VInputCommand as *mut VInputCommand);
                                    }
                                }

                                // 记录到历史
                                if let Ok(mut history) = recognition_history.lock() {
                                    history.push(final_result.clone());
                                    tracing::debug!("已记录到历史，当前历史数: {}", history.len());
                                }

                                tracing::info!("✨ 混合模式上屏完成");
                            }

                            // 重置状态
                            last_committed_stable.clear();

                            // 重置 Pipeline 准备下一句
                            let _ = pipe.reset();
                            tracing::info!("🔄 Pipeline 已重置，准备接收下一句");
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

        tracing::info!("🛑 手动停止录音");
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
        let itn_result = if let Ok(itn) = self.itn_engine.lock() {
            itn.process(&raw_result_with_punct)
        } else {
            crate::itn::ITNResult {
                text: raw_result_with_punct.clone(),
                changes: Vec::new(),
            }
        };
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

        // 记录到历史
        if let Ok(mut history) = self.recognition_history.lock() {
            history.push(final_result.clone());
            tracing::debug!("已记录到历史，当前历史数: {}", history.len());
        }

        // 生成命令序列
        if let Ok(mut queue) = self.command_queue.lock() {
            queue.push_back(VInputCommand::show_candidate(&final_result));
            queue.push_back(VInputCommand::commit_text(&final_result));
            queue.push_back(VInputCommand::hide_candidate());
            tracing::info!("生成 {} 个命令", queue.len());
        }
    }

    /// 尝试接收命令
    fn try_recv_command(&mut self) -> Option<VInputCommand> {
        if let Ok(mut queue) = self.command_queue.lock() {
            queue.pop_front()
        } else {
            None
        }
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

/// 注册命令回调函数
///
/// C++ 插件在初始化时调用此函数，注册回调以接收实时命令
/// 当 Rust Core 检测到句子结束时，会直接调用此回调
#[no_mangle]
pub extern "C" fn vinput_core_register_callback(callback: VInputCommandCallback) -> VInputFFIResult {
    match ffi_safe_call(|| {
        tracing::info!("V-Input Core FFI: 注册命令回调");

        let mut callback_lock = COMMAND_CALLBACK.lock().unwrap();
        *callback_lock = Some(callback);

        tracing::info!("✅ 命令回调注册成功");
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
            VInputEventType::UndoRequest => {
                tracing::info!("接收事件: UndoRequest");
                if let Ok(mut history) = core.recognition_history.lock() {
                    if let Some(undone_text) = history.undo() {
                        tracing::info!("撤销文本: {}", undone_text);
                        // 生成撤销命令
                        if let Ok(mut queue) = core.command_queue.lock() {
                            queue.push_back(VInputCommand::undo_text(&undone_text));
                        }
                    } else {
                        tracing::warn!("没有可撤销的内容");
                    }
                }
            }
            VInputEventType::RedoRequest => {
                tracing::info!("接收事件: RedoRequest");
                if let Ok(mut history) = core.recognition_history.lock() {
                    if let Some(redone_text) = history.redo() {
                        tracing::info!("重试文本: {}", redone_text);
                        // 生成重试命令
                        if let Ok(mut queue) = core.command_queue.lock() {
                            queue.push_back(VInputCommand::redo_text(&redone_text));
                        }
                    } else {
                        tracing::warn!("没有可重试的内容");
                    }
                }
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

/// 音频设备信息（FFI 兼容）
#[repr(C)]
pub struct VInputAudioDevice {
    /// 设备 ID（需要调用者释放）
    pub id: *mut c_char,
    /// 设备名称（需要调用者释放）
    pub name: *mut c_char,
    /// 设备描述（需要调用者释放）
    pub description: *mut c_char,
    /// 是否为默认设备
    pub is_default: bool,
}

/// 音频设备列表（FFI 兼容）
#[repr(C)]
pub struct VInputAudioDeviceList {
    /// 设备数组指针
    pub devices: *mut VInputAudioDevice,
    /// 设备数量
    pub count: usize,
}

/// 枚举音频输入设备
///
/// # 返回值
/// 成功返回设备列表指针，失败返回 null
/// 调用者需要使用 vinput_audio_device_list_free 释放内存
#[no_mangle]
pub extern "C" fn vinput_enumerate_audio_devices() -> *mut VInputAudioDeviceList {
    use super::safety::to_ffi_result;

    ffi_safe_call(|| {
        use crate::audio::enumerate_audio_devices;

        // 枚举设备
        let devices = to_ffi_result(enumerate_audio_devices())?;

        // 转换为 FFI 兼容格式
        let mut ffi_devices: Vec<VInputAudioDevice> = Vec::with_capacity(devices.len());

        for device in devices {
            let id = CString::new(device.id).unwrap_or_default();
            let name = CString::new(device.name).unwrap_or_default();
            let description = CString::new(device.description).unwrap_or_default();

            ffi_devices.push(VInputAudioDevice {
                id: id.into_raw(),
                name: name.into_raw(),
                description: description.into_raw(),
                is_default: device.is_default,
            });
        }

        // 创建设备列表
        let list = Box::new(VInputAudioDeviceList {
            devices: ffi_devices.as_mut_ptr(),
            count: ffi_devices.len(),
        });

        // 防止 Vec 被释放
        std::mem::forget(ffi_devices);

        Ok(Box::into_raw(list))
    })
    .unwrap_or(std::ptr::null_mut())
}

/// 释放音频设备列表
#[no_mangle]
pub extern "C" fn vinput_audio_device_list_free(list: *mut VInputAudioDeviceList) {
    if list.is_null() {
        return;
    }

    unsafe {
        let list_box = Box::from_raw(list);

        // 释放每个设备的字符串
        if !list_box.devices.is_null() {
            let devices_slice = std::slice::from_raw_parts_mut(
                list_box.devices,
                list_box.count
            );

            for device in devices_slice {
                if !device.id.is_null() {
                    let _ = CString::from_raw(device.id);
                }
                if !device.name.is_null() {
                    let _ = CString::from_raw(device.name);
                }
                if !device.description.is_null() {
                    let _ = CString::from_raw(device.description);
                }
            }

            // 释放设备数组
            let _ = Vec::from_raw_parts(
                list_box.devices,
                list_box.count,
                list_box.count
            );
        }

        // list_box 自动释放
    }
}
