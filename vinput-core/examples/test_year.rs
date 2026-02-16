use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        "今年是二零二六年",
        "二零二六年",
        "二零二六",
        "今天是二零二六年",
    ];

    for input in test_cases {
        let result = engine.process(input);
        println!("输入: \"{}\"", input);
        println!("输出: \"{}\"", result.text);
        if !result.changes.is_empty() {
            for change in &result.changes {
                println!("  变更: '{}' → '{}'", change.original_text, change.normalized_text);
            }
        }
        println!();
    }
}
