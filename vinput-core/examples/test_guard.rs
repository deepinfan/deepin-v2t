use vinput_core::itn::guards::ChineseWordGuard;

fn main() {
    let tests = vec!["一千", "一千个", "一千个地", "一起", "一开始"];

    for input in tests {
        let should_skip = ChineseWordGuard::should_skip_conversion(input);
        println!("\"{}\" → should_skip: {}", input, should_skip);
    }
}
