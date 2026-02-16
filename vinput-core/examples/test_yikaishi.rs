//! 测试 "一开始" 不被转换为 "1开始"

use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    println!("=== ITN \"一开始\" 测试 ===\n");

    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        ("一开始", "一开始"),  // 不应该转换
        ("一起", "一起"),      // 不应该转换
        ("一会儿", "一会儿"),  // 不应该转换
        ("一瞬间", "一瞬间"),  // 不应该转换
        ("一千", "1000"),      // 应该转换
        ("三百块钱", "¥300"),  // 应该转换
        ("一开始有三百块", "一开始有¥300"),  // 混合
    ];

    for (input, expected) in test_cases {
        let result = engine.process(input);
        let status = if result.text == expected { "✅" } else { "❌" };

        println!("{} 输入: \"{}\"", status, input);
        println!("   预期: \"{}\"", expected);
        println!("   实际: \"{}\"", result.text);

        if result.text != expected {
            println!("   ⚠️  测试失败！");
        }

        if !result.changes.is_empty() {
            println!("   变更:");
            for change in &result.changes {
                println!("     '{}' → '{}'", change.original_text, change.normalized_text);
            }
        }

        println!();
    }

    println!("=== 测试完成 ===");
}
