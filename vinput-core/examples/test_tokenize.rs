use vinput_core::itn::Tokenizer;

fn main() {
    let test_cases = vec![
        "今年是二零二六年",
        "二零二六年",
        "今天是二零二六年",
    ];
    
    for input in test_cases {
        let blocks = Tokenizer::tokenize(input);
        println!("输入: \"{}\"", input);
        for (i, block) in blocks.iter().enumerate() {
            println!("  Block {}: {:?} - \"{}\"", i, block.block_type, block.content);
        }
        println!();
    }
}
