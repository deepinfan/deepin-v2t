use vinput_core::itn::ChineseNumberConverter;

fn main() {
    let test_cases = vec!["九零", "九十", "九", "零", "二零二六"];

    for input in test_cases {
        match ChineseNumberConverter::convert(input) {
            Ok(result) => println!("\"{}\" → \"{}\"", input, result),
            Err(e) => println!("\"{}\" → Error: {:?}", input, e),
        }
    }
}
