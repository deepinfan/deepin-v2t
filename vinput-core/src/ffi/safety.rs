//! FFI 安全封装
//!
//! 使用 catch_unwind 防止 panic 跨越 FFI 边界

use super::types::VInputFFIResult;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// FFI 安全调用包装器
///
/// 捕获所有 panic，防止其跨越 FFI 边界
pub fn ffi_safe_call<F, T>(f: F) -> Result<T, VInputFFIResult>
where
    F: FnOnce() -> Result<T, VInputFFIResult>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic_err) => {
            // 记录 panic 信息
            if let Some(msg) = panic_err.downcast_ref::<&str>() {
                tracing::error!("FFI panic: {}", msg);
            } else if let Some(msg) = panic_err.downcast_ref::<String>() {
                tracing::error!("FFI panic: {}", msg);
            } else {
                tracing::error!("FFI panic: unknown error");
            }

            Err(VInputFFIResult::InternalError)
        }
    }
}

/// 将 Rust Result 转换为 FFI 结果码
pub fn to_ffi_result<T>(result: crate::VInputResult<T>) -> Result<T, VInputFFIResult> {
    result.map_err(|e| {
        tracing::error!("FFI error: {:?}", e);
        VInputFFIResult::InternalError
    })
}

/// 验证指针非空
#[inline]
pub fn check_null<T>(ptr: *const T, param_name: &str) -> Result<(), VInputFFIResult> {
    if ptr.is_null() {
        tracing::error!("Null pointer: {}", param_name);
        Err(VInputFFIResult::NullPointer)
    } else {
        Ok(())
    }
}

/// 验证可变指针非空
#[inline]
pub fn check_null_mut<T>(ptr: *mut T, param_name: &str) -> Result<(), VInputFFIResult> {
    if ptr.is_null() {
        tracing::error!("Null pointer: {}", param_name);
        Err(VInputFFIResult::NullPointer)
    } else {
        Ok(())
    }
}
