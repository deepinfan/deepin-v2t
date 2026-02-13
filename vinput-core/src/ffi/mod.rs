//! FFI (Foreign Function Interface) 模块
//!
//! C-compatible API for Fcitx5 integration

pub mod types;
pub mod safety;
pub mod exports;

// Re-export key types for cbindgen
pub use types::*;
pub use exports::*;
