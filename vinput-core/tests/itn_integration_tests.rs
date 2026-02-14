//! ITN 集成测试
//!
//! 测试完整的 ITN 管道

use vinput_core::itn::{ITNEngine, ITNMode};

#[test]
fn test_complete_pipeline_chinese_numbers() {
    let engine = ITNEngine::new(ITNMode::Auto);

    // 简单数字
    assert_eq!(engine.process("一").text, "1");
    assert_eq!(engine.process("十").text, "10");
    assert_eq!(engine.process("一百").text, "100");
    assert_eq!(engine.process("一千").text, "1000");

    // 复合数字
    assert_eq!(engine.process("一千二百三十四").text, "1234");
    assert_eq!(engine.process("三万五千").text, "35000");

    // 小数
    assert_eq!(engine.process("三点一四").text, "3.14");

    // 负数
    assert_eq!(engine.process("负五").text, "-5");
}

#[test]
fn test_complete_pipeline_percentages() {
    let engine = ITNEngine::new(ITNMode::Auto);

    assert_eq!(engine.process("百分之十").text, "10%");
    assert_eq!(engine.process("百分之二十").text, "20%");
    assert_eq!(engine.process("百分之五十").text, "50%");
    assert_eq!(engine.process("百分之一百").text, "100%");
}

#[test]
fn test_complete_pipeline_dates() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let result = engine.process("三月五号");
    assert!(result.text.contains("日"));
    assert!(!result.text.contains("号"));
}

#[test]
fn test_complete_pipeline_context_guard() {
    let engine = ITNEngine::new(ITNMode::Auto);

    // URL 应该被跳过
    assert_eq!(engine.process("http://example.com").text, "http://example.com");

    // CamelCase 应该被跳过
    assert_eq!(engine.process("CamelCase").text, "CamelCase");

    // All-caps 应该被跳过
    assert_eq!(engine.process("HTTP").text, "HTTP");
}

#[test]
fn test_complete_pipeline_mode_switching() {
    let text = "一千";

    // Auto 模式 - 应该转换
    let engine_auto = ITNEngine::new(ITNMode::Auto);
    assert_eq!(engine_auto.process(text).text, "1000");

    // NumbersOnly 模式 - 应该转换
    let engine_numbers = ITNEngine::new(ITNMode::NumbersOnly);
    assert_eq!(engine_numbers.process(text).text, "1000");

    // Raw 模式 - 不应该转换
    let engine_raw = ITNEngine::new(ITNMode::Raw);
    assert_eq!(engine_raw.process(text).text, "一千");
}

#[test]
fn test_complete_pipeline_rollback() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let original = "一千二百三十四";
    let result = engine.process(original);

    assert_eq!(result.text, "1234");
    assert_eq!(result.changes.len(), 1);

    let rolled_back = ITNEngine::rollback(&result);
    assert_eq!(rolled_back, original);
}

#[test]
fn test_complete_pipeline_change_tracking() {
    let engine = ITNEngine::new(ITNMode::Auto);

    let result = engine.process("一千二百三十四");

    assert_eq!(result.changes.len(), 1);
    assert_eq!(result.changes[0].original_text, "一千二百三十四");
    assert_eq!(result.changes[0].normalized_text, "1234");
}

#[test]
fn test_complete_pipeline_no_changes_when_skipped() {
    let engine = ITNEngine::new(ITNMode::Auto);

    // ContextGuard 应该跳过，不应该有变更
    let result = engine.process("CamelCase");
    assert_eq!(result.changes.len(), 0);

    // Raw 模式应该跳过，不应该有变更
    let engine_raw = ITNEngine::new(ITNMode::Raw);
    let result = engine_raw.process("一千");
    assert_eq!(result.changes.len(), 0);
}

#[test]
fn test_complete_pipeline_english_numbers() {
    let engine = ITNEngine::new(ITNMode::Auto);

    // Single words
    assert_eq!(engine.process("one").text, "1");
    assert_eq!(engine.process("twenty").text, "20");
}

#[test]
fn test_complete_pipeline_mixed_content() {
    let engine = ITNEngine::new(ITNMode::Auto);

    // 混合中文和数字（已有数字不转换）
    let result = engine.process("有123个");
    assert!(result.text.contains("123"));
}
