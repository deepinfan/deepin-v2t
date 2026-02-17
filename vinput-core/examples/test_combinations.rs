use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    // 测试不同的组合
    let tests = vec![
        "一起",
        "一千",
        "一起一千",
        "一起去了一千",
        "我们一起",
        "我们一起一千",
        "我们一起去了一千",
        "我们一起去了一千个",
        "我们一起去了一千个地方",
    ];

    for input in tests {
        let result = engine.process(input);
        println!("\"{}\" → \"{}\"", input, result.text);
    }
}
