//! ITN 性能基准测试
//!
//! 验证 ITN 处理时间 < 1ms 的要求
//!
//! 运行：cargo run --release --example itn_performance

use std::time::Instant;
use vinput_core::itn::{ITNEngine, ITNMode};

fn main() {
    println!("=== V-Input ITN 性能基准测试 ===\n");

    let engine = ITNEngine::new(ITNMode::Auto);

    // 测试用例
    let test_cases = vec![
        "一千二百三十四",
        "三万五千",
        "百分之五十",
        "三点一四",
        "负五",
        "三月五号",
        "CamelCase",
        "HTTP",
    ];

    println!("要求：单次处理 < 1ms\n");

    // 单次测试
    println!("【单次处理时间】\n");
    for (i, text) in test_cases.iter().enumerate() {
        let start = Instant::now();
        let _result = engine.process(text);
        let duration = start.elapsed();

        let status = if duration.as_micros() < 1000 {
            "✓"
        } else {
            "✗"
        };

        println!("#{} {} \"{}\"", i + 1, status, text);
        println!("   时间: {:.3}μs ({:.6}ms)",
            duration.as_micros(), duration.as_secs_f64() * 1000.0);
    }

    // 批量测试
    println!("\n【批量处理 (1000次)】\n");
    let iterations = 1000;

    for (i, text) in test_cases.iter().enumerate() {
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = engine.process(text);
        }
        let total_duration = start.elapsed();
        let avg_duration = total_duration / iterations;

        let status = if avg_duration.as_micros() < 1000 {
            "✓"
        } else {
            "✗"
        };

        println!("#{} {} \"{}\"", i + 1, status, text);
        println!("   平均: {:.3}μs ({:.6}ms)",
            avg_duration.as_micros(), avg_duration.as_secs_f64() * 1000.0);
        println!("   总计: {:.3}ms", total_duration.as_secs_f64() * 1000.0);
    }

    // 压力测试
    println!("\n【压力测试 (10000次)】\n");
    let stress_iterations = 10000;
    let stress_text = "一千二百三十四";

    let start = Instant::now();
    for _ in 0..stress_iterations {
        let _result = engine.process(stress_text);
    }
    let total_duration = start.elapsed();
    let avg_duration = total_duration / stress_iterations;

    println!("文本: \"{}\"", stress_text);
    println!("迭代: {} 次", stress_iterations);
    println!("总时间: {:.3}ms", total_duration.as_secs_f64() * 1000.0);
    println!("平均时间: {:.3}μs ({:.6}ms)",
        avg_duration.as_micros(), avg_duration.as_secs_f64() * 1000.0);

    let status = if avg_duration.as_micros() < 1000 {
        "✓ 通过"
    } else {
        "✗ 未通过"
    };
    println!("状态: {}", status);

    println!("\n=== 性能测试完成 ===");
}
