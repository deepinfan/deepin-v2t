use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        ("标点", "标点"),  // 应该保持不变
        ("标点符号", "标点符号"),  // 应该保持不变
        ("这是标点", "这是标点"),  // 应该保持不变
        ("一点钟", "1点钟"),  // 时间表达
        ("零点五", "0.5"),  // 小数
        ("三点一四", "3.14"),  // 小数
    ];

    println!("Testing '标点' ITN:");
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
