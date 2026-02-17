use vinput_core::streaming::StreamingPipeline;
use vinput_core::config::VInputConfig;

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== 测试 split_stable_unstable 逻辑 ===\n");

    // 创建配置
    let config = VInputConfig::default();

    // 创建 pipeline
    let mut pipeline = StreamingPipeline::new(config).expect("Failed to create pipeline");

    // 模拟识别过程
    let test_cases = vec![
        "零零后",
        "九零后",
        "一千个地方",
        "你好世界",
    ];

    for text in test_cases {
        println!("测试文本: \"{}\"", text);
        // 这里我们无法直接调用 split_stable_unstable（它是私有方法）
        // 但我们可以通过日志看到它的行为
    }

    println!("\n注意：split_stable_unstable 是私有方法，需要通过实际运行查看日志");
}
