//! 测试音频设备枚举

use vinput_core::audio::enumerate_audio_devices;

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== 音频设备枚举测试 ===\n");

    match enumerate_audio_devices() {
        Ok(devices) => {
            println!("找到 {} 个音频输入设备:\n", devices.len());
            for (i, device) in devices.iter().enumerate() {
                println!("设备 {}:", i + 1);
                println!("  ID: {}", device.id);
                println!("  名称: {}", device.name);
                println!("  描述: {}", device.description);
                println!("  默认: {}", if device.is_default { "是" } else { "否" });
                println!();
            }
        }
        Err(e) => {
            eprintln!("❌ 枚举设备失败: {}", e);
        }
    }
}
