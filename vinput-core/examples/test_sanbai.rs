use vinput_core::itn::guards::ChineseWordGuard;

fn main() {
    let tests = vec!["三百", "三百人", "三百人民"];

    for input in tests {
        let should_skip = ChineseWordGuard::should_skip_conversion(input);
        println!("\"{}\" → should_skip: {}", input, should_skip);
    }
}
