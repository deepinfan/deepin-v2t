use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);

    println!("=== 测试可能的识别结果 ===\n");

    let test_cases = vec![
        ("零零后", "完整识别"),
        ("零后", "只识别2个字"),
        ("00后", "识别成数字"),
        ("零零", "没有'后'字"),
        ("九零后", "九零后"),
        ("九后", "只识别2个字"),
        ("90后", "识别成数字"),
    ];

    for (input, desc) in test_cases {
        let result = engine.process(input);
        println!("{:12} \"{}\" → \"{}\"", desc, input, result.text);
    }
}
