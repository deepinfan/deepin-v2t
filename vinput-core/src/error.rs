use thiserror::Error;

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

pub type VInputResult<T> = Result<T, VInputError>;
