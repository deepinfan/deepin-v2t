use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    println!("=== 智能判断 vs 白名单测试 ===\n");

    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        // 白名单中的词
        ("一开始", "一开始"),
        ("一起", "一起"),
        ("一会儿", "一会儿"),
        ("一瞬间", "一瞬间"),

        // 白名单中没有，但应该被智能判断保护的词
        ("一眨眼", "一眨眼"),  // "眨"不在后缀列表，但不是数字单位
        ("一溜烟", "一溜烟"),  // "溜"不在后缀列表
        ("一股脑", "一股脑"),  // "股"不在后缀列表
        ("一窝蜂", "一窝蜂"),  // "窝"不在后缀列表

        // 应该转换的数字
        ("一千", "1000"),
        ("一百", "100"),
        ("一个", "1个"),
        ("一块钱", "¥1"),
        ("一年", "1年"),

        // 混合场景
        ("一开始有一千块", "一开始有¥1000"),
        ("一起买了一百个", "一起买了100个"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (input, expected) in test_cases {
        let result = engine.process(input);
        let status = if result.text == expected { "✅" } else { "❌" };

        if result.text == expected {
            passed += 1;
        } else {
            failed += 1;
        }

        println!("{} 输入: \"{}\"", status, input);
        println!("   预期: \"{}\"", expected);
        println!("   实际: \"{}\"", result.text);

        if result.text != expected {
            println!("   ⚠️  测试失败！");
        }

        println!();
    }

    println!("=== 测试完成 ===");
    println!("通过: {}/{}", passed, passed + failed);
    println!("失败: {}/{}", failed, passed + failed);
}
