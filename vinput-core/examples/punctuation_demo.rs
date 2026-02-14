//! 标点系统演示程序
//!
//! 演示标点控制系统的功能
//!
//! 运行：cargo run --example punctuation_demo

use vinput_core::punctuation::{PunctuationEngine, StyleProfile, TokenInfo};

fn main() {
    println!("=== V-Input 标点控制系统演示 ===\n");

    // 创建标点引擎（Professional 模式）
    let mut engine = PunctuationEngine::new(StyleProfile::professional());

    println!("【测试1: 基于停顿插入逗号】\n");

    // 模拟一段话："今天天气很好，我们去公园吧"
    // 前 6 个词正常速度，然后停顿，再说后面的
    let tokens = vec![
        TokenInfo::new("今天".to_string(), 0, 200),
        TokenInfo::new("天气".to_string(), 200, 400),
        TokenInfo::new("很".to_string(), 400, 500),
        TokenInfo::new("好".to_string(), 500, 700),
        TokenInfo::new("啊".to_string(), 700, 900),
        TokenInfo::new("真的".to_string(), 900, 1100),
        // 停顿 (1100ms - 2000ms = 900ms，约为 200ms 的 4.5 倍)
        TokenInfo::new("我们".to_string(), 2000, 2200),
        TokenInfo::new("去".to_string(), 2200, 2300),
        TokenInfo::new("公园".to_string(), 2300, 2500),
        TokenInfo::new("吧".to_string(), 2500, 2700),
    ];

    let mut result = String::new();
    for token in tokens {
        if let Some(text) = engine.process_token(token) {
            result.push_str(&text);
        }
    }

    println!("输入: 今天天气很好啊真的我们去公园吧");
    println!("输出: {}", result);
    println!();

    // 结束句子
    let ending = engine.finalize_sentence(1000, false);
    println!("句尾标点: {}", if ending.is_empty() { "(无)" } else { &ending });
    println!();

    println!("\n【测试2: 逻辑连接词插入逗号】\n");

    engine.reset_sentence();

    // 模拟："我很喜欢编程 所以 每天都在学习"
    let tokens2 = vec![
        TokenInfo::new("我".to_string(), 0, 150),
        TokenInfo::new("很".to_string(), 150, 250),
        TokenInfo::new("喜欢".to_string(), 250, 450),
        TokenInfo::new("编程".to_string(), 450, 650),
        TokenInfo::new("和".to_string(), 650, 750),
        TokenInfo::new("写作".to_string(), 750, 950),
        TokenInfo::new("真的".to_string(), 950, 1150),
        TokenInfo::new("不错".to_string(), 1150, 1350),
        // 逻辑连接词
        TokenInfo::new("所以".to_string(), 1350, 1550),
        TokenInfo::new("每天".to_string(), 1550, 1750),
        TokenInfo::new("都".to_string(), 1750, 1850),
        TokenInfo::new("在".to_string(), 1850, 1950),
        TokenInfo::new("学习".to_string(), 1950, 2150),
    ];

    let mut result2 = String::new();
    for token in tokens2 {
        if let Some(text) = engine.process_token(token) {
            result2.push_str(&text);
        }
    }

    println!("输入: 我很喜欢编程和写作真的不错所以每天都在学习");
    println!("输出: {}", result2);
    println!();

    println!("\n【测试3: 问号检测】\n");

    engine.reset_sentence();

    // 模拟："你好吗"（严格模式需要能量上扬）
    let tokens3 = vec![
        TokenInfo::new("你".to_string(), 0, 150),
        TokenInfo::new("好".to_string(), 150, 350),
        TokenInfo::new("吗".to_string(), 350, 550),
    ];

    let mut result3 = String::new();
    for token in tokens3 {
        if let Some(text) = engine.process_token(token) {
            result3.push_str(&text);
        }
    }

    println!("输入: 你好吗");
    println!("输出: {}", result3);

    // 不带能量上扬
    let ending1 = engine.finalize_sentence(1000, false);
    println!("句尾标点 (无能量上扬): {}", if ending1.is_empty() { "无（严格模式）" } else { &ending1 });

    // 重置并再次测试，这次带能量上扬
    engine.reset_sentence();
    for token in vec![
        TokenInfo::new("你".to_string(), 0, 150),
        TokenInfo::new("好".to_string(), 150, 350),
        TokenInfo::new("吗".to_string(), 350, 550),
    ] {
        engine.process_token(token);
    }
    let ending2 = engine.finalize_sentence(1000, true);
    println!("句尾标点 (有能量上扬): {}", ending2);

    println!();

    println!("\n【测试4: 风格切换】\n");

    // 切换到 Balanced 模式
    engine.update_profile(StyleProfile::balanced());

    engine.reset_sentence();

    // 同样的 "你好吗"，但 Balanced 模式不需要能量验证
    for token in vec![
        TokenInfo::new("你".to_string(), 0, 150),
        TokenInfo::new("好".to_string(), 150, 350),
        TokenInfo::new("吗".to_string(), 350, 550),
    ] {
        engine.process_token(token);
    }

    let ending3 = engine.finalize_sentence(1000, false);
    println!("风格: Balanced");
    println!("输入: 你好吗");
    println!("句尾标点 (无能量上扬): {}", ending3);

    println!("\n=== 演示完成 ===");
}
