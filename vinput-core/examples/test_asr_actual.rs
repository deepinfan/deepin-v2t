use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    println!("=== 模拟 ASR 识别结果 ===\n");

    // 根据日志，ASR 识别成了 "测试一下九后零后"
    let test_cases = vec![
        ("测试一下九后零后", "ASR 实际识别结果"),
        ("测试一下九零后零零后", "正确的识别结果"),
        ("九后", "单个九+后"),
        ("零后", "单个零+后"),
        ("九零后", "九零+后"),
        ("零零后", "零零+后"),
    ];

    for (input, desc) in test_cases {
        let result = engine.process(input);
        println!("{:20} \"{}\" → \"{}\"", desc, input, result.text);
    }
}
