//! 测试 ASR 模块导入

use vinput_core::asr::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use vinput_core::VInputResult;

fn main() -> VInputResult<()> {
    // 验证类型可以导入和使用
    let _config = OnlineRecognizerConfig::default();

    println!("✓ ASR module types import successfully");
    println!("✓ OnlineRecognizer available");
    println!("✓ OnlineRecognizerConfig available");
    println!("✓ OnlineStream available");

    Ok(())
}
