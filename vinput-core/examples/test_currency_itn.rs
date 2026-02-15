use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        ("我花了三百块钱", "我花了¥300"),
        ("这个东西五十块", "这个东西¥50"),
        ("总共一千二百元", "总共¥1200"),
        ("价格是九十九美元", "价格是$99"),
        ("需要支付二十欧元", "需要支付€20"),
        ("花费了五英镑", "花费了£5"),
    ];

    println!("Testing ITN Currency Rules:");
    println!("{}", "=".repeat(60));

    for (input, expected) in test_cases {
        let result = engine.process(input);
        let result_text = &result.text;
        let status = if result_text == expected { "✓" } else { "✗" };
        println!("{} Input:    {}", status, input);
        println!("  Expected: {}", expected);
        println!("  Got:      {}", result_text);
        println!();
    }
}
