//! V-Input Core Engine
//!
//! 离线中文语音输入法核心引擎

#![warn(rust_2018_idioms)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod ffi;
pub mod audio;
pub mod vad;
pub mod asr;
pub mod streaming;
pub mod state_machine;
pub mod endpointing;
pub mod itn;
pub mod punctuation;
pub mod hotwords;
pub mod undo;
pub mod config;
pub mod error;

// Re-export key types
pub use error::{VInputError, VInputResult, ErrorSeverity, RecoveryStrategy, ResultExt};
pub use endpointing::{EndpointDetector, EndpointDetectorConfig, EndpointResult};

/// 初始化日志系统
///
/// 生产模式: 仅当 VINPUT_LOG=1 时启用 Error 级别到 journald
/// 调试模式 (--features debug-logs): 完整日志
///
/// 注意: 此函数可以安全地多次调用（Fcitx5 会加载插件两次）
pub fn init_logging() {
    #[cfg(feature = "debug-logs")]
    {
        use tracing_subscriber::{fmt, prelude::*, EnvFilter};

        let filter = EnvFilter::try_from_env("VINPUT_LOG")
            .unwrap_or_else(|_| EnvFilter::new("warn"));

        // 使用 try_init() 代替 init()，避免重复初始化时 panic
        let _ = tracing_subscriber::registry()
            .with(fmt::layer().with_target(false))
            .with(filter)
            .try_init();

        // 忽略错误（说明已经初始化过了）
    }

    #[cfg(not(feature = "debug-logs"))]
    {
        // 生产模式: 静默运行，不启用日志
        // 如需日志，请使用 --features debug-logs 编译
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
