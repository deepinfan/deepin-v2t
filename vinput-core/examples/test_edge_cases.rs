use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let test_cases = vec![
        ("统一", "统一"),
        ("归一", "归一"),
        ("真二", "真二"),
        ("三心二意", "三心二意"),
        ("一心一意", "一心一意"),
        ("二话不说", "二话不说"),
        ("三番五次", "三番五次"),
        ("四面八方", "四面八方"),
        // 对比：应该转换的
        ("一千", "1000"),
        ("三个", "3个"),
    ];

    for (input, expected) in test_cases {
        let result = engine.process(input);
        let status = if result.text == expected { "✅" } else { "❌" };
        println!("{} \"{}\" → \"{}\" (预期: \"{}\")", status, input, result.text, expected);
    }
}
