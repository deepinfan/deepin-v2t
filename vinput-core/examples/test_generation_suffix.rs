use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        ("零零后", "零零后"),
        ("九零后", "九零后"),
        ("八零后", "八零后"),
        ("七零后", "七零后"),
        ("六零后", "六零后"),
        // 对比：应该转换的
        ("二零二六", "2026"),
        ("九十", "90"),  // "九十" 有单位，应该转换
    ];

    for (input, expected) in test_cases {
        let result = engine.process(input);
        let status = if result.text == expected { "✅" } else { "❌" };
        println!("{} \"{}\" → \"{}\" (预期: \"{}\")", status, input, result.text, expected);
    }
}
