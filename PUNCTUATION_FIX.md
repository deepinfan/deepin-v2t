# 标点符号不显示问题修复

## 问题根源

从日志分析发现，标点引擎工作正常，但标点符号没有出现在屏幕上。根本原因是：

### 1. 增量上屏机制

系统使用**增量上屏**（streaming commit）机制：
- 每次识别到稳定文本就立即上屏
- 这些增量上屏的文本是**原始ASR输出**，没有标点
- 例如：`今` → `天天天` → `天气` → `很` → `好所所` → ...

### 2. 标点处理时机

标点处理只在**最终结果**时进行：
- `get_final_result_with_punctuation()` 添加逗号和句号
- 但此时前面的文本已经通过增量上屏提交了

### 3. 偏移量计算错误

最终上屏时计算剩余文本的逻辑有问题：
```rust
// 错误的计算方式
let remaining_text = &final_result[last_committed_stable.len()..];
```

- `last_committed_stable` = "今天天天天气很好所所以..." (33字符，无标点)
- `final_result` = "今天天天天气很好，所所以..." (36字符，有2个逗号)
- `final_result[33..]` = "东西。" ← 跳过了 "买买"！

## 修复方案

**禁用增量上屏，改为一次性上屏**：

### 修改前（增量上屏）
```rust
// 识别过程中：增量上屏稳定文本（无标点）
if result.stable_text.len() > last_committed_stable.len() {
    let new_stable = &result.stable_text[last_committed_stable.len()..];
    commit_text(new_stable);  // ← 无标点
    last_committed_stable = result.stable_text.clone();
}

// 最终结果：上屏剩余文本（有标点，但偏移量错误）
let remaining_text = &final_result[last_committed_stable.len()..];
commit_text(remaining_text);  // ← 丢失部分文本
```

### 修改后（一次性上屏）
```rust
// 识别过程中：只更新 Preedit（显示实时识别）
let preedit_text = format!("{}{}", result.stable_text, result.unstable_text);
update_preedit(&preedit_text);  // ← 用户看到实时识别

// 最终结果：一次性上屏完整文本（有标点）
commit_text(&final_result);  // ← 完整文本，包含所有标点
```

## 修改文件

- `vinput-core/src/ffi/exports.rs`
  - 禁用增量上屏逻辑（line 210-233）
  - 改为只更新 Preedit
  - 最终结果一次性上屏（line 287-299）
  - 移除 `last_committed_stable` 变量

## 测试步骤

1. 重新编译：
   ```bash
   cd /home/deepin/deepin-v2t
   cargo build --release --features debug-logs
   ```

2. 安装并测试：
   ```bash
   ./test-punctuation-fix.sh
   ```

3. 测试语音输入（词之间停顿 0.5-1 秒）：
   ```
   今天 [停] 天气 [停] 很好 [停] 我想 [停] 出去 [停] 散步
   ```

4. 预期结果：
   - **Preedit 显示**：`今天天天天气很好所所以我准备出去逛逛街逛街逛然后然后去超市买买东西`
   - **最终上屏**：`今天天天天气很好，所所以我准备出去逛逛街逛逛街逛，然后然后去超市买买东西。`
   - ✅ 逗号和句号正确显示
   - ✅ 没有丢失字符

## 用户体验变化

### 修改前
- ✅ 识别过程中文字逐步出现在编辑器中
- ❌ 最终结果没有标点符号
- ❌ 可能丢失部分字符

### 修改后
- ✅ 识别过程中 Preedit 显示实时识别（下划线）
- ✅ 最终结果一次性上屏，包含完整标点
- ✅ 不会丢失任何字符
- ℹ️ 用户体验类似于拼音输入法（先显示拼音，确认后上屏）

## 后续优化（可选）

如果用户希望保留增量上屏体验，可以实现**实时标点处理**：
- 在增量上屏时也调用标点引擎
- 需要标点引擎支持增量模式
- 复杂度较高，暂不实现

---

**修复时间**: 2026-02-17
**修复人**: Claude Code
**问题编号**: #77
