# V-Input 标点和 ITN 改进方案

## 🎯 当前问题

1. **无标点符号** - 识别结果没有标点，如："你好世界今天天气真好"
2. **ITN 未生效** - 数字等没有转换，如："一千" 没有转为 "1000"

## 📋 问题分析

### 1. 标点符号

**现状**：
- Sherpa-ONNX 模型可能不支持标点输出
- 需要后处理添加标点

**解决方案选项**：
- **方案 A**：使用基于规则的标点引擎（已实现 `PunctuationEngine`）
- **方案 B**：使用 AI 标点模型（如 punctuator）
- **方案 C**：使用 sherpa-onnx 的标点模型（如果有）

### 2. ITN（文本规范化）

**现状**：
- ITN 引擎已实现（`vinput-core/src/itn/`）
- 代码中已调用 `itn_engine.process()`
- 可能因为模型输出格式问题未生效

**ITN 支持的转换**：
- 数字：一千 → 1000
- 日期：二零二六年二月十四日 → 2026年2月14日
- 货币：十五块钱 → ¥15
- 百分比：百分之五十 → 50%
- 单位：三米 → 3米

## ✅ 快速修复方案

### 方案 1：启用后处理标点引擎

修改 `vinput-core/src/ffi/exports.rs` 的 `stop_recording()`：

```rust
// 获取识别结果
let raw_result = if let Ok(mut pipe) = self.pipeline.lock() {
    pipe.get_final_result()
} else {
    String::new()
};

if raw_result.is_empty() {
    tracing::warn!("识别结果为空，不生成命令");
    return;
}

tracing::info!("原始识别结果: {}", raw_result);

// 1. ITN (文本规范化)
let itn_result = self.itn_engine.process(&raw_result);
let mut final_result = itn_result.text;
tracing::info!("ITN 后: {}", final_result);

// 2. 添加标点（使用规则引擎）
if let Some(punct_engine) = &self.punctuation_engine {
    final_result = punct_engine.add_punctuation(&final_result);
    tracing::info!("标点后: {}", final_result);
}

tracing::info!("✅ 最终结果: {}", final_result);
```

### 方案 2：临时解决 - 手动添加句号

最简单的方案，先让每句话自动加句号：

```rust
// 临时方案：每句话结尾添加句号
if !final_result.ends_with(&['。', '！', '？', '.', '!', '?'][..]) {
    final_result.push('。');
}
```

### 方案 3：调试 ITN 为什么没生效

添加详细日志查看 ITN 的输入和输出：

```rust
tracing::info!("原始识别: [{}]", raw_result);
let itn_result = self.itn_engine.process(&raw_result);
tracing::info!("ITN 输入: [{}]", raw_result);
tracing::info!("ITN 输出: [{}]", itn_result.text);
tracing::info!("ITN 变更数: {}", itn_result.changes.len());
for change in &itn_result.changes {
    tracing::info!("  {} → {}", change.original_text, change.normalized_text);
}
```

## 🔧 实施步骤

### 立即可做（方案 2 - 最简单）

1. 修改 `vinput-core/src/ffi/exports.rs`
2. 在最终结果后添加句号
3. 重新编译并测试

### 短期改进（方案 1）

1. 初始化 `PunctuationEngine`
2. 在后处理流程中调用
3. 配置标点规则

### 长期方案

1. 研究 sherpa-onnx 是否支持标点模型
2. 集成专业的 AI 标点模型
3. 实现智能断句

## 📝 代码示例 - 快速修复

```rust
// vinput-core/src/ffi/exports.rs

fn stop_recording(&mut self) {
    // ... 省略前面的代码 ...

    // 获取识别结果
    let raw_result = if let Ok(mut pipe) = self.pipeline.lock() {
        pipe.get_final_result()
    } else {
        String::new()
    };

    if raw_result.is_empty() {
        tracing::warn!("识别结果为空");
        return;
    }

    tracing::info!("🎤 原始识别: {}", raw_result);

    // 1. ITN (文本规范化)
    let itn_result = self.itn_engine.process(&raw_result);
    let mut final_result = itn_result.text;

    if !itn_result.changes.is_empty() {
        tracing::info!("📝 ITN 转换: {} 处变更", itn_result.changes.len());
        for change in &itn_result.changes {
            tracing::debug!("  '{}' → '{}'", change.original_text, change.normalized_text);
        }
    }

    // 2. 临时标点方案：添加句号
    if !final_result.ends_with(&['。', '！', '？', '.', '!', '?'][..]) {
        final_result.push('。');
        tracing::debug!("✏️  自动添加句号");
    }

    tracing::info!("✅ 最终结果: {}", final_result);

    // ... 省略后面的代码 ...
}
```

## 🧪 测试用例

测试各种场景：

| 输入语音 | 期望输出 | 验证 |
|----------|----------|------|
| "你好世界" | "你好世界。" | 自动加句号 ✓ |
| "今天是二月十四日" | "今天是2月14日。" | ITN 数字转换 + 句号 |
| "我有一千块钱" | "我有1000块钱。" | ITN 数字转换 + 句号 |
| "百分之五十" | "50%。" | ITN 百分比转换 + 句号 |

## 🚀 下一步

你想：
1. **A. 快速修复** - 先加上自动句号（5分钟）
2. **B. 完整方案** - 集成标点引擎（30分钟）
3. **C. 调试 ITN** - 先看看 ITN 为什么没生效（10分钟）

我建议先选 **C**，然后 **A**，最后有时间做 **B**。
