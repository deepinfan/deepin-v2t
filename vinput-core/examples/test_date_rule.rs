use vinput_core::itn::rules::DateRule;

fn main() {
    let test_cases = vec![
        "今年是二零二六年",
        "二零二六年",
        "今天是二零二六年",
    ];
    
    for input in test_cases {
        println!("输入: \"{}\"", input);
        println!("is_date_expression: {}", DateRule::is_date_expression(input));
        
        if DateRule::is_date_expression(input) {
            match DateRule::convert_chinese(input) {
                Ok(result) => println!("DateRule 输出: \"{}\"", result),
                Err(e) => println!("DateRule 错误: {:?}", e),
            }
        }
        println!();
    }
}
