use vinput_core::itn::rules::DateRule;

fn main() {
    let input = "今年是二零二六年";
    println!("输入: \"{}\"", input);
    println!("包含'年'的次数: {}", input.matches('年').count());
    
    // 手动查找所有"年"的位置
    for (i, ch) in input.char_indices() {
        if ch == '年' {
            println!("找到'年'在位置: {}", i);
        }
    }
    
    let result = DateRule::convert_chinese(input).unwrap();
    println!("输出: \"{}\"", result);
}
