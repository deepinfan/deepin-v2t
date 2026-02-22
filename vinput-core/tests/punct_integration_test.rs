//! CT-Transformer 标点模型集成测试
//!
//! 测试真实 ONNX 模型的加载和推理。
//! 需要模型文件存在于 `models/punct-ct-transformer/` 目录。
//! 没有模型文件时，相关测试会被 `#[ignore]` 跳过。

/// 模型目录（相对于 workspace 根目录）
const MODEL_DIR: &str = "../models/punct-ct-transformer";

/// 检查模型文件是否存在
fn model_available() -> bool {
    std::path::Path::new(MODEL_DIR).join("model.int8.onnx").exists()
        && std::path::Path::new(MODEL_DIR).join("tokens.json").exists()
}

/// 跳过测试的辅助宏（模型不可用时）
macro_rules! require_model {
    () => {
        if !model_available() {
            eprintln!("跳过：标点模型不可用 ({})", MODEL_DIR);
            return;
        }
    };
}

#[cfg(feature = "vad-onnx")]
mod ct_transformer_tests {
    use super::*;
    use std::path::Path;
    use vinput_core::punctuation::CtTransformerPunct;

    #[test]
    fn test_model_load() {
        require_model!();
        let model = CtTransformerPunct::new(Path::new(MODEL_DIR));
        assert!(model.is_ok(), "模型加载失败: {:?}", model.err());
    }

    #[test]
    fn test_short_text_passthrough() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        // 少于 3 字的文本直接返回（不推理）
        let result = model.add_punctuation("好");
        assert_eq!(result, "好");
        let result2 = model.add_punctuation("OK");
        assert_eq!(result2, "OK");
    }

    #[test]
    fn test_empty_text() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        let result = model.add_punctuation("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_simple_chinese_sentence() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        let input = "今天天气很好我要出门逛街";
        let result = model.add_punctuation(input);
        eprintln!("输入: {}", input);
        eprintln!("输出: {}", result);
        // 应该包含标点（至少包含句号）
        assert!(
            result.contains('。') || result.contains('，') || result.contains('、'),
            "期望有标点符号，实际输出: {}",
            result
        );
        // 输出长度应该 >= 输入（有标点插入）
        assert!(result.chars().count() >= input.chars().count());
    }

    #[test]
    fn test_question_sentence() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        let input = "你好吗今天有什么计划";
        let result = model.add_punctuation(input);
        eprintln!("输入: {}", input);
        eprintln!("输出: {}", result);
        // 至少有某种标点
        assert!(
            result.contains('。') || result.contains('，') || result.contains('？'),
            "期望有标点符号，实际输出: {}",
            result
        );
    }

    #[test]
    fn test_mixed_chinese_english() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        let input = "我用Python写了一个程序然后测试通过了";
        let result = model.add_punctuation(input);
        eprintln!("输入: {}", input);
        eprintln!("输出: {}", result);
        // 输出应该包含原文字符
        assert!(result.contains("Python"));
        assert!(result.contains("程序"));
    }

    #[test]
    fn test_long_text_chunking() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        // 生成超过 512 字的文本
        let unit = "今天天气很好我们出去散步然后买了一些东西回家做饭";
        let input = unit.repeat(30); // ~750 字
        assert!(input.chars().count() > 512);

        let result = model.add_punctuation(&input);
        eprintln!("超长文本: {} 字 → {} 字", input.chars().count(), result.chars().count());
        // 输出应该至少与输入等长（有标点插入）
        assert!(result.chars().count() >= input.chars().count());
        // 应该包含标点
        assert!(result.contains('。') || result.contains('，'));
    }

    #[test]
    fn test_multiple_sentences() {
        require_model!();
        let mut model = CtTransformerPunct::new(Path::new(MODEL_DIR)).unwrap();
        let input = "我昨天去了超市买了很多东西回来之后发现忘记买盐了";
        let result = model.add_punctuation(input);
        eprintln!("输入: {}", input);
        eprintln!("输出: {}", result);
        assert!(!result.is_empty());
        // 有标点
        let has_punct = result.contains('，') || result.contains('。')
            || result.contains('？') || result.contains('、');
        assert!(has_punct, "未检测到标点: {}", result);
    }
}

/// 没有 vad-onnx feature 时的基础测试
#[cfg(not(feature = "vad-onnx"))]
mod no_feature_tests {
    #[test]
    fn test_ct_transformer_not_compiled_without_feature() {
        // 没有 vad-onnx feature 时，ct_transformer 模块不编译
        // 这个测试本身能编译通过就说明条件编译正确
    }
}
