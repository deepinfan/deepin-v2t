use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let input = "我们一起去了一千个地方";
    let result = engine.process(input);

    println!("输入: \"{}\"", input);
    println!("输出: \"{}\"", result.text);
    println!("预期: \"我们一起去了1000个地方\"");
    println!();

    // 测试各个部分
    println!("分段测试:");
    println!("\"一起\" → \"{}\"", engine.process("一起").text);
    println!("\"一千\" → \"{}\"", engine.process("一千").text);
    println!("\"一千个\" → \"{}\"", engine.process("一千个").text);
    println!("\"去了一千个\" → \"{}\"", engine.process("去了一千个").text);
}
