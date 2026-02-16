//! 测试混合模式的智能过滤逻辑

fn contains_chinese_number(text: &str) -> bool {
    text.chars().any(|c| matches!(c,
        '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
        '十' | '百' | '千' | '万' | '亿' | '点'
    ))
}

fn split_stable_unstable(text: &str) -> (String, String) {
    // 优先检查：如果整个文本包含中文数字，全部保留在 Preedit
    if contains_chinese_number(text) {
        println!("  ⚠️  检测到中文数字，全部保留在 Preedit");
        return (String::new(), text.to_string());
    }

    // 如果不包含数字，按正常逻辑分离
    const KEEP_LAST_CHARS: usize = 2;
    let chars: Vec<char> = text.chars().collect();

    if chars.len() <= KEEP_LAST_CHARS {
        return (String::new(), text.to_string());
    }

    let stable_count = chars.len() - KEEP_LAST_CHARS;
    let stable: String = chars[..stable_count].iter().collect();
    let unstable: String = chars[stable_count..].iter().collect();

    (stable, unstable)
}

fn main() {
    println!("=== 混合模式智能过滤测试 ===\n");

    // 测试场景 1：普通文本（无数字）
    println!("场景 1：普通文本");
    let test_cases_1 = vec![
        "今",
        "今天",
        "今天天",
        "今天天气",
        "今天天气很好",
    ];

    for text in test_cases_1 {
        let (stable, unstable) = split_stable_unstable(text);
        println!("  \"{}\" → stable: \"{}\", unstable: \"{}\"", text, stable, unstable);
    }

    println!();

    // 测试场景 2：包含数字
    println!("场景 2：包含数字");
    let test_cases_2 = vec![
        "三",
        "三百",
        "三百块",
        "三百块钱",
    ];

    for text in test_cases_2 {
        let (stable, unstable) = split_stable_unstable(text);
        println!("  \"{}\" → stable: \"{}\", unstable: \"{}\"", text, stable, unstable);
    }

    println!();

    // 测试场景 3：混合文本
    println!("场景 3：混合文本（数字在中间）");
    let test_cases_3 = vec![
        "今",
        "今天",
        "今天买",
        "今天买了",
        "今天买了三",
        "今天买了三百",
        "今天买了三百块",
    ];

    for text in test_cases_3 {
        let (stable, unstable) = split_stable_unstable(text);
        println!("  \"{}\" → stable: \"{}\", unstable: \"{}\"", text, stable, unstable);
    }

    println!();

    // 测试场景 4：小数
    println!("场景 4：小数");
    let test_cases_4 = vec![
        "零点五",
        "三点一四",
        "温度是三十七点五度",
    ];

    for text in test_cases_4 {
        let (stable, unstable) = split_stable_unstable(text);
        println!("  \"{}\" → stable: \"{}\", unstable: \"{}\"", text, stable, unstable);
    }

    println!();

    // 测试场景 5：边界情况
    println!("场景 5：边界情况");
    let test_cases_5 = vec![
        "标点",           // 不包含数字（"点" 前后无数字字符）
        "标点符号",       // 不包含数字
        "一点钟",         // 包含数字
        "差一点",         // 包含数字
    ];

    for text in test_cases_5 {
        let (stable, unstable) = split_stable_unstable(text);
        println!("  \"{}\" → stable: \"{}\", unstable: \"{}\"", text, stable, unstable);
    }

    println!("\n=== 测试完成 ===");
}
