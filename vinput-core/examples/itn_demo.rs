//! ITN 系统演示程序
//!
//! 演示 ITN (Inverse Text Normalization) 的完整功能
//!
//! 运行：cargo run --example itn_demo

use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    println!("=== V-Input ITN 系统演示 ===\n");

    // 创建 ITN 引擎（Auto 模式）
    let engine = ITNEngine::new(ITNMode::Auto);

    // 测试用例
    let test_cases = vec![
        // 中文数字转换
        ("一千二百三十四", "1234"),
        ("三万五千", "35000"),
        ("三点一四", "3.14"),
        ("负五", "-5"),

        // 百分比转换
        ("百分之五十", "50%"),
        ("百分之二十", "20%"),

        // 日期转换
        ("三月五号", "三月五日"),

        // ContextGuard - 跳过转换
        ("CamelCase", "CamelCase"),
        ("HTTP", "HTTP"),

        // 混合测试
        ("我有一千块钱", "我有1000块钱"),
    ];

    println!("【测试用例】\n");
    for (i, (input, expected)) in test_cases.iter().enumerate() {
        let result = engine.process(input);
        let status = if &result.text == expected {
            "✓"
        } else {
            "✗"
        };

        println!("#{} {} 原始: \"{}\"", i + 1, status, input);
        println!("     输出: \"{}\"", result.text);
        println!("     期望: \"{}\"", expected);
        println!("     变更: {} 处", result.changes.len());

        if !result.changes.is_empty() {
            for change in &result.changes {
                println!("       - \"{}\" → \"{}\"",
                    change.original_text, change.normalized_text);
            }
        }
        println!();
    }

    // 演示回滚功能
    println!("\n【回滚功能演示】\n");
    let result = engine.process("一千二百三十四");
    println!("原始文本: \"一千二百三十四\"");
    println!("规范化后: \"{}\"", result.text);

    let rolled_back = ITNEngine::rollback(&result);
    println!("回滚结果: \"{}\"", rolled_back);
    println!();

    // 演示不同模式
    println!("\n【模式切换演示】\n");

    let test_text = "一千二百三十四";

    println!("测试文本: \"{}\"", test_text);
    println!();

    let engine_auto = ITNEngine::new(ITNMode::Auto);
    let result_auto = engine_auto.process(test_text);
    println!("Auto 模式: \"{}\"", result_auto.text);

    let engine_numbers = ITNEngine::new(ITNMode::NumbersOnly);
    let result_numbers = engine_numbers.process(test_text);
    println!("NumbersOnly 模式: \"{}\"", result_numbers.text);

    let engine_raw = ITNEngine::new(ITNMode::Raw);
    let result_raw = engine_raw.process(test_text);
    println!("Raw 模式: \"{}\"", result_raw.text);

    println!("\n=== 演示完成 ===");
}
