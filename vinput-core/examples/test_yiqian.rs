use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let tests = vec![
        "一千",
        "一起",
        "我们一起去了一千个地方",
    ];

    for input in tests {
        let result = engine.process(input);
        println!("\"{}\" → \"{}\"", input, result.text);
    }
}
