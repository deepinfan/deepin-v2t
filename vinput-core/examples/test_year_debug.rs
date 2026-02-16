use vinput_core::itn::{ITNEngine, ITNMode, Tokenizer};

fn main() {
    let engine = ITNEngine::new(ITNMode::Auto);
    let input = "今年是二零二六年";
    
    println!("输入: \"{}\"", input);
    
    // 测试 Tokenizer
    let blocks = Tokenizer::tokenize(input);
    println!("\nTokenizer 结果:");
    for (i, block) in blocks.iter().enumerate() {
        println!("  Block {}: {:?} - \"{}\"", i, block.block_type, block.content);
    }
    
    // 测试 ITN
    let result = engine.process(input);
    println!("\nITN 结果: \"{}\"", result.text);
    
    if !result.changes.is_empty() {
        println!("\n变更:");
        for change in &result.changes {
            println!("  '{}' → '{}'", change.original_text, change.normalized_text);
        }
    }
}
