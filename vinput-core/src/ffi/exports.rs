//! FFI 导出函数
//!
//! Rust cdylib FFI 接口，供 Fcitx5 C++ 插件调用
//! Phase 1: 基础集成框架（完整集成需要重构架构）

use super::safety::{check_null, check_null_mut, ffi_safe_call};
use super::types::{VInputCommand, VInputEvent, VInputEventType, VInputFFIResult};
use std::collections::VecDeque;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

/// 全局 V-Input Core 实例
static VINPUT_CORE: Mutex<Option<VInputCoreState>> = Mutex::new(None);

/// V-Input Core 状态 (Phase 1: 简化版本)
///
/// Note: 完整的 ASR/VAD/PipeWire 集成需要重构架构
/// 当前版本专注于验证 Fcitx5 集成流程
struct VInputCoreState {
    // 命令队列
    command_queue: VecDeque<VInputCommand>,
    // 录音状态
    is_recording: bool,
    // Phase 1.5: 将添加完整的 ASR、VAD、音频捕获
}

impl VInputCoreState {
    fn new() -> Self {
        tracing::info!("初始化 V-Input Core (简化版本)");

        Self {
            command_queue: VecDeque::new(),
            is_recording: false,
        }
    }

    /// 启动录音
    fn start_recording(&mut self) {
        if self.is_recording {
            tracing::warn!("已经在录音中");
            return;
        }

        tracing::info!("启动录音 (Phase 1 - 模拟模式)");
        self.is_recording = true;

        // Phase 1: 这里会启动 PipeWire 音频捕获
        // 当前仅设置状态标志
    }

    /// 停止录音并生成识别结果
    fn stop_recording(&mut self) {
        if !self.is_recording {
            tracing::warn!("没有在录音");
            return;
        }

        tracing::info!("停止录音 (Phase 1 - 生成测试命令)");
        self.is_recording = false;

        // Phase 1: 生成测试命令序列
        // 完整版本会从 ASR 获取真实识别结果并生成相应命令

        // 1. 显示候选词（模拟识别过程中的候选）
        self.command_queue
            .push_back(VInputCommand::show_candidate("语音|鱼音|雨音"));

        // 2. 提交最终识别结果
        self.command_queue
            .push_back(VInputCommand::commit_text("语音输入测试"));

        // 3. 隐藏候选词
        self.command_queue
            .push_back(VInputCommand::hide_candidate());

        tracing::debug!(
            "生成测试命令序列: 队列长度={}",
            self.command_queue.len()
        );
    }

    /// 尝试接收命令
    fn try_recv_command(&mut self) -> Option<VInputCommand> {
        self.command_queue.pop_front()
    }
}

/// 初始化 V-Input Core
///
/// # 安全性
/// - 必须在使用其他 FFI 函数前调用
/// - 可以多次调用（幂等）
/// - 线程安全
///
/// # 返回值
/// - Success: 初始化成功
/// - InitFailed: 初始化失败
#[no_mangle]
pub extern "C" fn vinput_core_init() -> VInputFFIResult {
    match ffi_safe_call(|| {
        // 初始化日志（如果尚未初始化）
        crate::init_logging();

        tracing::info!("V-Input Core FFI: 初始化");

        let mut core = VINPUT_CORE.lock().unwrap();

        if core.is_some() {
            tracing::warn!("V-Input Core 已经初始化");
            return Ok(VInputFFIResult::Success);
        }

        // 创建 Core 状态
        *core = Some(VInputCoreState::new());

        tracing::info!("V-Input Core 初始化成功");
        Ok(VInputFFIResult::Success)
    }) {
        Ok(result) => result,
        Err(e) => e,
    }
}

/// 关闭 V-Input Core
///
/// # 安全性
/// - 释放所有资源
/// - 可以多次调用（幂等）
/// - 线程安全
///
/// # 返回值
/// - Success: 关闭成功
#[no_mangle]
pub extern "C" fn vinput_core_shutdown() -> VInputFFIResult {
    match ffi_safe_call(|| {
        tracing::info!("V-Input Core FFI: 关闭");

        let mut core = VINPUT_CORE.lock().unwrap();

        if core.is_none() {
            tracing::warn!("V-Input Core 未初始化");
            return Ok(VInputFFIResult::Success);
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
///
/// # 参数
/// - event: 事件指针，不能为 NULL
///
/// # 安全性
/// - 必须先调用 vinput_core_init()
/// - event 指针必须有效
/// - 如果 event.data 非 NULL，必须指向有效的 data_len 字节数据
/// - 函数会复制数据，调用后可以立即释放 event
///
/// # 返回值
/// - Success: 事件已接收
/// - NotInitialized: Core 未初始化
/// - NullPointer: event 为 NULL
/// - InvalidArgument: 参数无效
#[no_mangle]
pub extern "C" fn vinput_core_send_event(event: *const VInputEvent) -> VInputFFIResult {
    match ffi_safe_call(|| {
        // 检查参数
        check_null(event, "event")?;

        let event = unsafe { &*event };

        // 检查 Core 是否已初始化
        let mut core_lock = VINPUT_CORE.lock().unwrap();
        let core = core_lock
            .as_mut()
            .ok_or(VInputFFIResult::NotInitialized)?;

        // Phase 1: 处理事件
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
///
/// # 参数
/// - command: 命令输出指针，不能为 NULL
///
/// # 安全性
/// - 必须先调用 vinput_core_init()
/// - command 指针必须有效
/// - 如果返回 Success，command.text 会被分配内存，调用者负责释放
/// - 使用 vinput_command_free() 释放 command
///
/// # 返回值
/// - Success: 成功接收命令，command 已填充
/// - NoData: 无命令可读
/// - NotInitialized: Core 未初始化
/// - NullPointer: command 为 NULL
#[no_mangle]
pub extern "C" fn vinput_core_try_recv_command(command: *mut VInputCommand) -> VInputFFIResult {
    match ffi_safe_call(|| {
        // 检查参数
        check_null_mut(command, "command")?;

        // 检查 Core 是否已初始化
        let mut core_lock = VINPUT_CORE.lock().unwrap();
        let core = core_lock
            .as_mut()
            .ok_or(VInputFFIResult::NotInitialized)?;

        // Phase 1: 从命令队列读取
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
///
/// # 参数
/// - command: 命令指针
///
/// # 安全性
/// - 只能释放由 vinput_core_try_recv_command() 返回的命令
/// - 可以对 NULL 指针调用（无操作）
/// - 释放后不能再次使用
#[no_mangle]
pub extern "C" fn vinput_command_free(command: *mut VInputCommand) {
    if command.is_null() {
        return;
    }

    unsafe {
        let cmd = &mut *command;

        // 释放文本内存
        if !cmd.text.is_null() {
            let _ = CString::from_raw(cmd.text);
            cmd.text = std::ptr::null_mut();
            cmd.text_len = 0;
        }
    }
}

/// 获取版本字符串
///
/// # 返回值
/// - 静态字符串指针，不需要释放
#[no_mangle]
pub extern "C" fn vinput_core_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

// Phase 0 说明：
// 当前 FFI 提供了基本的接口框架：
// - 初始化/关闭
// - 事件发送
// - 命令接收（空实现）
//
// Phase 1 完整实现：
// - 在 VInputCoreState 中集成 ASR、VAD、音频捕获
// - 实际处理事件（启动/停止录音、音频数据）
// - 从识别结果生成命令并入队
// - 实现命令队列的读取
//
// 接口设计已完成，可直接用于 Fcitx5 集成
