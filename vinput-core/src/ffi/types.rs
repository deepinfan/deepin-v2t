//! FFI C-compatible 类型定义

use std::os::raw::{c_char, c_int};

/// FFI 结果码
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VInputFFIResult {
    /// 成功
    Success = 0,
    /// 空指针错误
    NullPointer = -1,
    /// 无效参数
    InvalidArgument = -2,
    /// 初始化失败
    InitFailed = -3,
    /// 未初始化
    NotInitialized = -4,
    /// 内部错误
    InternalError = -5,
    /// 无数据可读
    NoData = -6,
    /// 音频错误
    AudioError = -7,
}

/// V-Input 事件类型
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VInputEventType {
    /// 开始录音
    StartRecording = 1,
    /// 停止录音
    StopRecording = 2,
    /// 音频数据
    AudioData = 3,
    /// 识别结果
    RecognitionResult = 4,
    /// VAD 状态变化
    VADStateChanged = 5,
}

/// V-Input 事件（从 Fcitx5 -> Rust Core）
#[repr(C)]
pub struct VInputEvent {
    /// 事件类型
    pub event_type: VInputEventType,
    /// 数据指针（可选）
    pub data: *const u8,
    /// 数据长度
    pub data_len: usize,
}

/// V-Input 命令类型
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VInputCommandType {
    /// 提交文本
    CommitText = 1,
    /// 显示候选
    ShowCandidate = 2,
    /// 隐藏候选
    HideCandidate = 3,
    /// 错误消息
    Error = 4,
}

/// V-Input 命令（从 Rust Core -> Fcitx5）
#[repr(C)]
pub struct VInputCommand {
    /// 命令类型
    pub command_type: VInputCommandType,
    /// 文本数据（UTF-8，以 null 结尾）
    pub text: *mut c_char,
    /// 文本长度（不含 null）
    pub text_len: usize,
}

// VInputCommand 需要在 VecDeque 中存储，因此需要 Send
// 由于 text 指针是从 CString::into_raw() 获得的，我们确保其生命周期正确
unsafe impl Send for VInputCommand {}
unsafe impl Sync for VInputCommand {}

/// 不透明的 V-Input Core 句柄
#[repr(C)]
pub struct VInputHandle {
    _private: [u8; 0],
}

// 实现 Send + Sync（实际实现在 Rust 侧）
unsafe impl Send for VInputHandle {}
unsafe impl Sync for VInputHandle {}

impl VInputEvent {
    /// 创建新事件
    pub fn new(event_type: VInputEventType) -> Self {
        Self {
            event_type,
            data: std::ptr::null(),
            data_len: 0,
        }
    }

    /// 创建带数据的事件
    pub fn with_data(event_type: VInputEventType, data: &[u8]) -> Self {
        Self {
            event_type,
            data: data.as_ptr(),
            data_len: data.len(),
        }
    }
}

impl VInputCommand {
    /// 创建空命令
    pub fn new(command_type: VInputCommandType) -> Self {
        Self {
            command_type,
            text: std::ptr::null_mut(),
            text_len: 0,
        }
    }
}
