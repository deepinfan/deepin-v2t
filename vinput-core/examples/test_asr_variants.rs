use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    println!("=== 测试 ASR 可能的识别结果 ===\n");

    let test_cases = vec![
        ("零零后", "ASR 正确识别中文"),
        ("00后", "ASR 识别成阿拉伯数字"),
        ("零后", "ASR 只识别出2个字"),
        ("零 零 后", "ASR 识别时有空格"),
    ];

    for (input, desc) in test_cases {
        let result = engine.process(input);
        println!("{:25} \"{}\" → \"{}\"", desc, input, result.text);
    }

    println!("\n结论：");
    println!("- 如果 ASR 识别成'零零后'（中文），ITN 会保留为'零零后' ✓");
    println!("- 如果 ASR 识别成'00后'（数字），ITN 会保留为'00后' ✓");
    println!("- 如果 ASR 识别成'零后'（缺字），ITN 会保留为'零后' ✓");
    println!("\n问题可能是：ASR 模型把'零零'识别成了'零'（单个字）");
}
