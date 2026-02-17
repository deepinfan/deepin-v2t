use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    println!("=== 直接测试 ITN 引擎 ===");

    let test_cases = vec![
        "零零后",
        "九零后",
        "八零后",
    ];

    for input in test_cases {
        let result = engine.process(input);
        println!("输入: \"{}\" → 输出: \"{}\"", input, result.text);
    }
}
