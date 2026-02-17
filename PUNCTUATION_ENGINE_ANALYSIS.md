# 标点引擎逻辑分析报告

## 问题诊断

### 1. 配置加载问题 ⚠️

**发现**：配置文件 `~/.config/vinput/config.toml` 中的标点配置可能没有被正确加载。

**证据**：
- `StyleProfile::default()` 返回 `professional_preset()`
- Professional 预设值：
  - `streaming_pause_ratio: 2.5`
  - `streaming_min_tokens: 6`
- 用户配置文件：
  - `pause_ratio: 2.0`
  - `min_tokens: 3`

**问题**：如果配置加载失败或被忽略，会使用硬编码的预设值，导致：
- 需要至少 6 个 Token 才开始检测逗号（而不是配置的 3 个）
- 停顿阈值是 2.5 倍（而不是配置的 2.0 倍）

### 2. 停顿检测逻辑

停顿检测算法（`pause_engine.rs:91-136`）：

```rust
fn should_insert_comma(&self, token: &TokenInfo) -> bool {
    // 1. 检查 token 数量
    if self.token_history.len() < self.profile.streaming_min_tokens {
        return false;  // ❌ 如果 < 6 个 Token，永远不插入逗号
    }

    // 2. 检查距离上次逗号的 token 数
    if let Some(last_pos) = self.last_comma_position {
        let tokens_since_comma = self.token_history.len() - last_pos;
        if tokens_since_comma < self.profile.min_tokens_between_commas {
            return false;  // ❌ 两个逗号之间至少间隔 4 个 Token
        }
    }

    // 3. 检查上一个 Token 的时长
    if let Some(last_token) = self.token_history.last() {
        let last_token_duration = last_token.duration_ms();

        // 检查最小时长
        if last_token_duration < self.profile.min_pause_duration_ms {
            return false;  // ❌ 如果 < 500ms，不检测停顿
        }

        // 计算平均 token 时长
        let avg_duration = self.calculate_avg_token_duration();

        // 计算时长比例
        let duration_ratio = last_token_duration as f32 / avg_duration as f32;

        // 如果上一个 Token 的时长显著超过平均值，说明包含了停顿
        return duration_ratio > self.profile.streaming_pause_ratio;
        // ❌ 需要 > 2.5 倍才插入逗号
    }

    false
}
```

### 3. 逗号插入条件（全部必须满足）

1. ✅ Token 数量 >= `streaming_min_tokens` (6)
2. ✅ 距离上次逗号 >= `min_tokens_between_commas` (4)
3. ✅ 上一个 Token 时长 >= `min_pause_duration_ms` (500ms)
4. ✅ 时长比例 > `streaming_pause_ratio` (2.5)

**问题**：这些条件非常严格，导致很少插入逗号。

### 4. Token 时间戳来源

Token 时间戳来自 Sherpa-ONNX ASR 模型：

```rust
// pipeline.rs:437
let token_info = token.to_token_info();
```

**Sherpa-ONNX 特性**：
- Token 之间的时间戳是连续的，没有间隙
- 停顿包含在上一个 Token 的时长中
- 如果用户说话快，Token 时长都很短，很难触发停顿检测

### 5. 重复字符问题

可能原因：
1. **ASR 模型输出重复** - INT8 量化可能导致解码异常
2. **Preedit 更新逻辑** - 多次更新同一文本
3. **ITN 处理** - 转换过程中重复

需要查看日志确认。

## 解决方案

### 方案 A：确保配置正确加载（推荐）

检查配置加载路径和日志：

```bash
# 1. 查看配置文件
cat ~/.config/vinput/config.toml

# 2. 启用调试日志，查看配置加载
RUST_LOG=vinput_core=debug fcitx5 2>&1 | grep -E "(配置|pause_ratio|min_tokens)"
```

### 方案 B：降低硬编码的预设值

如果配置加载失败，修改 `config.rs:65-75` 的 Professional 预设：

```rust
fn professional_preset() -> Self {
    Self {
        streaming_pause_ratio: 2.0,  // 从 2.5 降低到 2.0
        streaming_min_tokens: 3,     // 从 6 降低到 3
        min_tokens_between_commas: 2, // 从 4 降低到 2
        min_pause_duration_ms: 400,  // 从 500 降低到 400
        // ...
    }
}
```

### 方案 C：添加调试日志

在 `pause_engine.rs:74-88` 添加详细日志：

```rust
let should_insert = self.should_insert_comma(&token);

if should_insert {
    tracing::info!("  🎯 检测到停顿，将在 '{}' 前插入逗号", token.text);
} else {
    tracing::debug!("  ⏭  未检测到停顿: token='{}', history_len={}, min_tokens={}",
        token.text, self.token_history.len(), self.profile.streaming_min_tokens);
}
```

## 测试建议

1. **验证配置加载**：
   ```bash
   RUST_LOG=vinput_core=info fcitx5 2>&1 | grep "标点配置"
   ```
   应该看到：`pause_ratio=2.0, min_tokens=3`

2. **测试逗号插入**：
   - 说至少 6 个词（如果配置未加载）
   - 在词之间停顿 1 秒以上
   - 观察是否插入逗号

3. **查看 Token 时长**：
   ```bash
   RUST_LOG=vinput_core=debug fcitx5 2>&1 | grep "停顿检测"
   ```
   应该看到：`上一Token='xxx' 时长=XXXms, 平均=XXXms, 比例=X.XX`

## 下一步行动

1. 先运行调试脚本确认配置是否加载
2. 如果配置未加载，修复加载逻辑
3. 如果配置已加载但仍无逗号，降低预设值
4. 添加详细日志分析 Token 时长

---

生成时间: 2026-02-17
