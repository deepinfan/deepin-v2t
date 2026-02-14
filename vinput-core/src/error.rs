use thiserror::Error;

/// V-Input 核心错误类型
#[derive(Error, Debug)]
pub enum VInputError {
    // 音频错误
    #[error("PipeWire error: {0}")]
    PipeWire(String),

    #[error("Audio device not found: {0}")]
    AudioDeviceNotFound(String),

    #[error("Ring buffer overrun: {lost_frames} frames lost")]
    RingBufferOverrun { lost_frames: u64 },

    // ASR 错误
    #[error("Model load failed: {path} - {reason}")]
    ModelLoad { path: String, reason: String },

    #[error("ASR inference failed: {0}")]
    AsrInference(String),

    #[error("Recognizer not initialized")]
    RecognizerNotReady,

    // VAD 错误
    #[error("Silero VAD model load failed: {0}")]
    VadModelLoad(String),

    #[error("VAD inference failed: {0}")]
    VadInference(String),

    // ITN 错误
    #[error("ITN conversion failed: {0}")]
    ItnConversion(String),

    // 热词错误
    #[error("Hotwords error: {0}")]
    Hotword(String),

    // 状态机错误
    #[error("Invalid state transition: {from} + {event}")]
    InvalidTransition { from: String, event: String },

    #[error("Operation not allowed in state: {state}")]
    NotAllowedInState { state: String },

    // 配置错误
    #[error("Config parse error: {path} - {reason}")]
    ConfigParse { path: String, reason: String },

    #[error("Config file not found: {0}")]
    ConfigNotFound(String),

    // 通道错误
    #[error("Channel send error: receiver dropped")]
    ChannelSend,

    #[error("Channel recv error: sender dropped")]
    ChannelRecv,

    // FFI 错误
    #[error("FFI null pointer: {param}")]
    NullPointer { param: String },

    // 撤销错误
    #[error("Nothing to undo")]
    EmptyUndoHistory,

    #[error("Undo time window expired ({elapsed_ms}ms > {window_ms}ms)")]
    UndoTimeWindowExpired { elapsed_ms: u64, window_ms: u64 },

    // 其他错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 低优先级：不影响核心功能
    Low,
    /// 中等优先级：部分功能受影响
    Medium,
    /// 高优先级：核心功能受影响
    High,
    /// 严重：系统无法继续运行
    Critical,
}

/// 错误恢复策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// 可以重试
    Retry,
    /// 可以降级服务（例如：禁用某个功能）
    Degrade,
    /// 需要用户干预
    UserAction,
    /// 无法恢复，必须重启
    Restart,
}

impl VInputError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // 低严重度：可恢复的瞬时错误
            VInputError::RingBufferOverrun { lost_frames } if *lost_frames < 1000 => {
                ErrorSeverity::Low
            }
            VInputError::ChannelSend | VInputError::ChannelRecv => ErrorSeverity::Low,

            // 中等严重度：影响功能但可降级
            VInputError::AudioDeviceNotFound(_) => ErrorSeverity::Medium,
            VInputError::VadInference(_) => ErrorSeverity::Medium,
            VInputError::ItnConversion(_) => ErrorSeverity::Medium,
            VInputError::Hotword(_) => ErrorSeverity::Medium,
            VInputError::EmptyUndoHistory | VInputError::UndoTimeWindowExpired { .. } => {
                ErrorSeverity::Medium
            }

            // 高严重度：核心功能受影响
            VInputError::PipeWire(_) => ErrorSeverity::High,
            VInputError::AsrInference(_) => ErrorSeverity::High,
            VInputError::RecognizerNotReady => ErrorSeverity::High,
            VInputError::InvalidTransition { .. } | VInputError::NotAllowedInState { .. } => {
                ErrorSeverity::High
            }

            // 严重：系统级故障
            VInputError::ModelLoad { .. } | VInputError::VadModelLoad(_) => ErrorSeverity::Critical,
            VInputError::ConfigParse { .. } | VInputError::ConfigNotFound(_) => {
                ErrorSeverity::Critical
            }
            VInputError::NullPointer { .. } => ErrorSeverity::Critical,

            // 默认为高严重度
            _ => ErrorSeverity::High,
        }
    }

    /// 获取推荐的恢复策略
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            // 可重试的错误
            VInputError::PipeWire(_) => RecoveryStrategy::Retry,
            VInputError::ChannelSend | VInputError::ChannelRecv => RecoveryStrategy::Retry,
            VInputError::AsrInference(_) | VInputError::VadInference(_) => RecoveryStrategy::Retry,

            // 可降级的错误
            VInputError::AudioDeviceNotFound(_) => RecoveryStrategy::Degrade,
            VInputError::RingBufferOverrun { .. } => RecoveryStrategy::Degrade,
            VInputError::ItnConversion(_) | VInputError::Hotword(_) => RecoveryStrategy::Degrade,

            // 需要用户干预
            VInputError::ModelLoad { .. }
            | VInputError::VadModelLoad(_)
            | VInputError::ConfigNotFound(_) => RecoveryStrategy::UserAction,
            VInputError::RecognizerNotReady => RecoveryStrategy::UserAction,

            // 需要重启
            VInputError::NullPointer { .. } => RecoveryStrategy::Restart,
            VInputError::ConfigParse { .. } => RecoveryStrategy::Restart,

            // 默认可重试
            _ => RecoveryStrategy::Retry,
        }
    }

    /// 获取用户友好的错误描述
    pub fn user_message(&self) -> String {
        match self {
            VInputError::PipeWire(msg) => {
                format!("音频系统错误：{}", simplify_technical_message(msg))
            }
            VInputError::AudioDeviceNotFound(device) => {
                format!("未找到音频设备 \"{}\"，请检查麦克风连接", device)
            }
            VInputError::RingBufferOverrun { lost_frames } => {
                format!("音频缓冲区溢出，丢失了 {} 个音频帧。建议关闭其他占用 CPU 的程序", lost_frames)
            }
            VInputError::ModelLoad { path, reason } => {
                format!(
                    "无法加载模型文件 \"{}\"：{}。请检查文件是否存在",
                    path, reason
                )
            }
            VInputError::AsrInference(msg) => {
                format!("语音识别失败：{}。请重试", msg)
            }
            VInputError::RecognizerNotReady => {
                "语音识别引擎未就绪，请稍候再试".to_string()
            }
            VInputError::VadModelLoad(msg) => {
                format!("无法加载语音检测模型：{}。请检查安装", msg)
            }
            VInputError::VadInference(msg) => {
                format!("语音检测失败：{}。请重试", msg)
            }
            VInputError::ItnConversion(msg) => {
                format!("文本规范化失败：{}。将保留原始文本", msg)
            }
            VInputError::Hotword(msg) => {
                format!("热词加载失败：{}。将使用默认配置", msg)
            }
            VInputError::InvalidTransition { from, event } => {
                format!("操作顺序错误：当前状态 {} 不支持操作 {}", from, event)
            }
            VInputError::NotAllowedInState { state } => {
                format!("当前状态 \"{}\" 下无法执行此操作", state)
            }
            VInputError::ConfigParse { path, reason } => {
                format!("配置文件 \"{}\" 解析失败：{}", path, reason)
            }
            VInputError::ConfigNotFound(path) => {
                format!("配置文件 \"{}\" 不存在", path)
            }
            VInputError::ChannelSend | VInputError::ChannelRecv => {
                "内部通信错误，请重试".to_string()
            }
            VInputError::NullPointer { param } => {
                format!("内部错误：空指针 ({})", param)
            }
            VInputError::EmptyUndoHistory => "没有可撤销的操作".to_string(),
            VInputError::UndoTimeWindowExpired {
                elapsed_ms,
                window_ms,
            } => {
                format!(
                    "撤销超时：已过 {}ms，超过允许的 {}ms",
                    elapsed_ms, window_ms
                )
            }
            VInputError::Io(e) => format!("文件操作失败：{}", e),
            VInputError::Generic(msg) => msg.clone(),
        }
    }

    /// 获取错误码（用于日志和诊断）
    pub fn error_code(&self) -> &'static str {
        match self {
            VInputError::PipeWire(_) => "E1001",
            VInputError::AudioDeviceNotFound(_) => "E1002",
            VInputError::RingBufferOverrun { .. } => "E1003",
            VInputError::ModelLoad { .. } => "E2001",
            VInputError::AsrInference(_) => "E2002",
            VInputError::RecognizerNotReady => "E2003",
            VInputError::VadModelLoad(_) => "E3001",
            VInputError::VadInference(_) => "E3002",
            VInputError::ItnConversion(_) => "E4001",
            VInputError::Hotword(_) => "E4002",
            VInputError::InvalidTransition { .. } => "E4003",
            VInputError::NotAllowedInState { .. } => "E4004",
            VInputError::ConfigParse { .. } => "E5001",
            VInputError::ConfigNotFound(_) => "E5002",
            VInputError::ChannelSend => "E6001",
            VInputError::ChannelRecv => "E6002",
            VInputError::NullPointer { .. } => "E7001",
            VInputError::EmptyUndoHistory => "E8001",
            VInputError::UndoTimeWindowExpired { .. } => "E8002",
            VInputError::Io(_) => "E9001",
            VInputError::Generic(_) => "E9999",
        }
    }

    /// 记录错误到日志系统（带上下文）
    pub fn log(&self) {
        let severity = self.severity();
        let code = self.error_code();
        let recovery = self.recovery_strategy();

        match severity {
            ErrorSeverity::Low => {
                tracing::warn!(
                    error_code = code,
                    severity = ?severity,
                    recovery = ?recovery,
                    "{}",
                    self
                );
            }
            ErrorSeverity::Medium => {
                tracing::warn!(
                    error_code = code,
                    severity = ?severity,
                    recovery = ?recovery,
                    "{}",
                    self
                );
            }
            ErrorSeverity::High => {
                tracing::error!(
                    error_code = code,
                    severity = ?severity,
                    recovery = ?recovery,
                    "{}",
                    self
                );
            }
            ErrorSeverity::Critical => {
                tracing::error!(
                    error_code = code,
                    severity = ?severity,
                    recovery = ?recovery,
                    "CRITICAL: {}",
                    self
                );
            }
        }
    }
}

/// 简化技术性错误消息为用户友好格式
fn simplify_technical_message(msg: &str) -> &str {
    // 简化常见的技术错误消息
    if msg.contains("Connection refused") {
        "音频服务未运行"
    } else if msg.contains("Permission denied") {
        "权限不足"
    } else if msg.contains("Device or resource busy") {
        "设备正在被其他程序使用"
    } else {
        msg
    }
}

pub type VInputResult<T> = Result<T, VInputError>;

/// Result 扩展 trait，提供错误上下文功能
pub trait ResultExt<T> {
    /// 添加错误上下文并记录日志
    fn log_on_err(self) -> Result<T, VInputError>;

    /// 添加用户友好的错误消息
    fn with_user_message<F>(self, f: F) -> Result<T, VInputError>
    where
        F: FnOnce() -> String;
}

impl<T> ResultExt<T> for VInputResult<T> {
    fn log_on_err(self) -> Result<T, VInputError> {
        if let Err(ref e) = self {
            e.log();
        }
        self
    }

    fn with_user_message<F>(self, f: F) -> Result<T, VInputError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            tracing::error!("Error: {} | User message: {}", e, f());
            e
        })
    }
}

