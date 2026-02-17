use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let tests = vec![
        "一下",
        "三百",
        "一下三百",
        "一下子",
        "一下子三百",
        "一下子就来了三百",
        "一下子就来了三百人",
    ];

    for input in tests {
        let result = engine.process(input);
        println!("\"{}\" → \"{}\"", input, result.text);
    }
}
