fn main() {
    let test_cases = vec![
        "零零后",
        "九零后",
        "一千个地方",
        "你好世界",
        "00后",
        "90后",
    ];

    for text in test_cases {
        let has_chinese_num = contains_chinese_number(text);
        println!("\"{}\" → has_chinese_number: {}", text, has_chinese_num);
    }
}

fn contains_chinese_number(text: &str) -> bool {
    text.chars().any(|c| matches!(c,
        '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' |
        '十' | '百' | '千' | '万' | '亿' | '点'
    ))
}
