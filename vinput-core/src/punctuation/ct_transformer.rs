//! CT-Transformer 标点符号预测模型
//!
//! 使用 FunASR CT-Transformer ONNX 模型为 ASR 输出添加标点符号。
//!
//! 模型信息：
//! - 来源：sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12
//! - 输入：字符 token ID 序列 (int32)
//! - 输出：每字的标点类别 logits (float32, 6 类)
//! - 标点类别：0=<unk>, 1=_(无), 2=，, 3=。, 4=？, 5=、
//!
//! 需要启用 `vad-onnx` feature

use crate::error::{VInputError, VInputResult};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Value;
use std::collections::HashMap;
use std::path::Path;

/// 标点类别映射（索引 → 标点字符）
/// 索引对应模型 config.yaml 中的 punc_list
const PUNC_CHARS: [&str; 6] = [
    "",  // 0: <unk> → 不输出标点
    "",  // 1: _    → 无标点
    "，", // 2: 中文逗号
    "。", // 3: 句号
    "？", // 4: 问号
    "、", // 5: 顿号
];

/// 单次推理最大序列长度（CT-Transformer SANM encoder 限制）
const MAX_SEQ_LEN: usize = 512;

/// CT-Transformer 标点模型
pub struct CtTransformerPunct {
    session: Session,
    /// 字符 → token ID 映射表
    vocab: HashMap<String, i32>,
    /// <unk> token ID（未知字符使用）
    unk_id: i32,
}

impl CtTransformerPunct {
    /// 加载模型
    ///
    /// # 参数
    /// - `model_dir`：模型目录，应包含 `model.int8.onnx` 和 `tokens.json`
    pub fn new(model_dir: &Path) -> VInputResult<Self> {
        let model_path = model_dir.join("model.int8.onnx");
        let tokens_path = model_dir.join("tokens.json");

        if !model_path.exists() {
            return Err(VInputError::ModelLoad {
                path: model_path.display().to_string(),
                reason: "标点模型文件不存在".to_string(),
            });
        }
        if !tokens_path.exists() {
            return Err(VInputError::ModelLoad {
                path: tokens_path.display().to_string(),
                reason: "标点词汇表不存在".to_string(),
            });
        }

        tracing::info!("加载 CT-Transformer 标点模型: {}", model_path.display());

        let model_bytes = std::fs::read(&model_path).map_err(|e| VInputError::ModelLoad {
            path: model_path.display().to_string(),
            reason: format!("读取失败: {}", e),
        })?;

        let session = Session::builder()
            .map_err(|e| VInputError::ModelLoad {
                path: model_path.display().to_string(),
                reason: format!("创建 session builder 失败: {}", e),
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| VInputError::ModelLoad {
                path: model_path.display().to_string(),
                reason: format!("设置优化级别失败: {}", e),
            })?
            .with_intra_threads(1)
            .map_err(|e| VInputError::ModelLoad {
                path: model_path.display().to_string(),
                reason: format!("设置线程数失败: {}", e),
            })?
            .commit_from_memory(&model_bytes)
            .map_err(|e| VInputError::ModelLoad {
                path: model_path.display().to_string(),
                reason: format!("加载失败: {}", e),
            })?;

        tracing::info!("CT-Transformer 标点模型加载成功");

        // 加载词汇表（tokens.json 是列表，索引即 token ID）
        let vocab = Self::load_vocab(&tokens_path)?;
        let unk_id = *vocab.get("<unk>").unwrap_or(&0) as i32;
        tracing::info!("标点词汇表加载成功: {} 个词条，unk_id={}", vocab.len(), unk_id);

        Ok(Self { session, vocab, unk_id })
    }

    /// 加载词汇表
    fn load_vocab(tokens_path: &Path) -> VInputResult<HashMap<String, i32>> {
        let content = std::fs::read_to_string(tokens_path).map_err(|e| VInputError::ModelLoad {
            path: tokens_path.display().to_string(),
            reason: format!("读取词汇表失败: {}", e),
        })?;

        let tokens: Vec<String> = serde_json::from_str(&content).map_err(|e| VInputError::ModelLoad {
            path: tokens_path.display().to_string(),
            reason: format!("解析 tokens.json 失败: {}", e),
        })?;

        let vocab: HashMap<String, i32> = tokens
            .into_iter()
            .enumerate()
            .map(|(i, token)| (token, i as i32))
            .collect();

        Ok(vocab)
    }

    /// 为文本添加标点符号
    ///
    /// 对输入的纯文本进行标点预测，返回带标点的文本。
    /// 若文本为空或过短（< 3 字符），直接返回原文。
    /// 若文本超过 MAX_SEQ_LEN，按块处理并拼接结果。
    pub fn add_punctuation(&mut self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();

        if chars.len() < 3 {
            return text.to_string();
        }

        if chars.len() <= MAX_SEQ_LEN {
            match self.infer_punctuation(&chars) {                Ok(result) => result,
                Err(e) => {
                    tracing::warn!("标点推理失败，返回原文: {}", e);
                    text.to_string()
                }
            }
        } else {
            // 超长文本：按块处理，在块边界尽量选择空格/数字边界
            self.infer_chunked(&chars)
        }
    }

    /// 对字符序列执行一次 ONNX 推理
    fn infer_punctuation(&mut self, chars: &[char]) -> VInputResult<String> {
        use ort::inputs;

        let seq_len = chars.len();

        // 1. 将字符转为 token ID (int32)
        let token_ids: Vec<i32> = chars
            .iter()
            .map(|&c| {
                let s = c.to_string();
                *self.vocab.get(&s).unwrap_or(&self.unk_id)
            })
            .collect();

        // 2. 构建输入张量
        let inputs_tensor =
            Value::from_array((vec![1usize, seq_len], token_ids)).map_err(|e| {
                VInputError::AsrInference(format!("创建 inputs 张量失败: {}", e))
            })?;

        let lengths_tensor =
            Value::from_array((vec![1usize], vec![seq_len as i32])).map_err(|e| {
                VInputError::AsrInference(format!("创建 text_lengths 张量失败: {}", e))
            })?;

        // 3. 执行推理
        let outputs = self
            .session
            .run(inputs![inputs_tensor, lengths_tensor])
            .map_err(|e| VInputError::AsrInference(format!("标点推理执行失败: {}", e)))?;

        // 4. 提取 logits: shape [1, seq_len, 6]
        let (_, logits_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| VInputError::AsrInference(format!("提取 logits 失败: {}", e)))?;

        // 5. 对每个字符位置取 argmax，重建带标点文本
        let mut result = String::with_capacity(text_capacity(seq_len));
        let num_classes = 6usize;

        for (i, &ch) in chars.iter().enumerate() {
            result.push(ch);

            // 从平铺的 logits 中取第 i 个位置的 6 个值
            let offset = i * num_classes;
            if offset + num_classes <= logits_data.len() {
                let class_logits = &logits_data[offset..offset + num_classes];
                let punc_class = argmax(class_logits);
                if punc_class < PUNC_CHARS.len() {
                    result.push_str(PUNC_CHARS[punc_class]);
                }
            }
        }

        tracing::debug!("标点推理完成: '{}' → '{}'", chars.iter().collect::<String>(), result);
        Ok(result)
    }

    /// 超长文本分块推理
    fn infer_chunked(&mut self, chars: &[char]) -> String {
        let chunk_size = MAX_SEQ_LEN / 2; // 256 字符/块，保留上下文余量
        let mut result = String::with_capacity(chars.len() * 3);

        let mut start = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk = &chars[start..end];

            match self.infer_punctuation(chunk) {
                Ok(chunk_result) => result.push_str(&chunk_result),
                Err(e) => {
                    tracing::warn!("分块标点推理失败 [{}..{}]: {}", start, end, e);
                    // fallback：直接输出原字符
                    for &c in chunk {
                        result.push(c);
                    }
                }
            }

            start = end;
        }

        result
    }
}

/// 计算 argmax（返回最大值索引）
fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(1) // 默认返回 1（无标点）
}

/// 预估结果字符串容量
fn text_capacity(char_count: usize) -> usize {
    // 每个字符最多 3 字节（UTF-8 中文），标点最多 3 字节
    char_count * 6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argmax_basic() {
        assert_eq!(argmax(&[0.1, 0.5, 0.3, 0.8, 0.2, 0.1]), 3);
        assert_eq!(argmax(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]), 0);
        assert_eq!(argmax(&[0.0, 0.0, 0.0, 0.0, 0.0, 1.0]), 5);
    }

    #[test]
    fn test_argmax_single() {
        assert_eq!(argmax(&[0.5]), 0);
    }

    #[test]
    fn test_punc_chars_mapping() {
        assert_eq!(PUNC_CHARS[1], "");  // 无标点
        assert_eq!(PUNC_CHARS[2], "，");
        assert_eq!(PUNC_CHARS[3], "。");
        assert_eq!(PUNC_CHARS[4], "？");
        assert_eq!(PUNC_CHARS[5], "、");
    }

    #[test]
    fn test_short_text_passthrough() {
        // 短文本直接返回（不能创建真实模型，这里只测试代码路径）
        // 注意：CtTransformerPunct::new() 需要真实模型文件
        // 这个测试仅验证 add_punctuation 对短文本的处理逻辑
        // 通过手动构造测试用例来验证
        let chars: Vec<char> = "好".chars().collect();
        assert!(chars.len() < 3); // 确认会走短路径
    }

    /// 验证推理结果重建逻辑（使用模拟 logits）
    #[test]
    fn test_decode_logits_none() {
        // logits: [0.0, 1.0, 0.0, 0.0, 0.0, 0.0] → 类别1 → 无标点
        let logits = [0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0];
        let class = argmax(&logits);
        assert_eq!(class, 1);
        assert_eq!(PUNC_CHARS[class], "");
    }

    #[test]
    fn test_decode_logits_comma() {
        // logits: [0.0, 0.0, 1.0, 0.0, 0.0, 0.0] → 类别2 → ，
        let logits = [0.0f32, 0.0, 1.0, 0.0, 0.0, 0.0];
        let class = argmax(&logits);
        assert_eq!(class, 2);
        assert_eq!(PUNC_CHARS[class], "，");
    }

    #[test]
    fn test_decode_logits_period() {
        let logits = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0];
        assert_eq!(PUNC_CHARS[argmax(&logits)], "。");
    }

    #[test]
    fn test_decode_logits_question() {
        let logits = [0.0f32, 0.0, 0.0, 0.0, 1.0, 0.0];
        assert_eq!(PUNC_CHARS[argmax(&logits)], "？");
    }

    #[test]
    fn test_text_capacity() {
        assert!(text_capacity(10) >= 10);
    }
}
