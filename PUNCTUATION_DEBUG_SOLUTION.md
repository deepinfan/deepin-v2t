# 标点引擎问题诊断报告

## 问题根源 ✅ 已找到

### 1. 日志未启用
**原因**：生产版本编译时未启用 `debug-logs` feature，导致所有 `tracing::debug!` 和 `tracing::info!` 日志被完全禁用。

**证据**：
- `lib.rs:49-53`: 生产模式下日志完全静默
- `Cargo.toml:12-13`: `debug-logs` 是可选 feature
- 当前编译命令：`cargo build --release`（未启用 debug-logs）

**影响**：
- 无法看到配置加载日志
- 无法看到停顿检测日志
- 无法看到 Token 时长分析
- 无法诊断标点引擎问题

### 2. Zipformer 时间戳支持 ✅ 确认有
**结论**：Zipformer 模型**完全支持**时间戳。

**证据**：
- `recognizer.rs:326-357`: 正确提取 Sherpa-ONNX 的 timestamps 数组
- timestamps 单位：秒（浮点数）
- Token 时长计算：`end_time - start_time`（毫秒）

**时间戳格式**：
```rust
// timestamps[i] 是相对开始时间（秒）
let start_time_s = timestamps[i];
let end_time_s = timestamps[i + 1];  // 下一个 Token 的开始时间
let duration_ms = (end_time_s - start_time_s) * 1000.0;
```

## 解决方案

### 方案 A：启用调试日志（推荐）

重新编译并启用 debug-logs feature：

```bash
cd /home/deepin/deepin-v2t

# 1. 编译（启用调试日志）
cargo build --release --features debug-logs

# 2. 安装
sudo cp target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

# 3. 设置日志级别
export VINPUT_LOG=debug

# 4. 重启 fcitx5
fcitx5 -r

# 5. 测试并查看日志
journalctl --user -u fcitx5 -f | grep -E "(标点配置|停顿检测|Token)"
```

### 方案 B：修改默认日志级别

如果不想每次都设置环境变量，修改 `lib.rs:37-38`：

```rust
let filter = EnvFilter::try_from_env("VINPUT_LOG")
    .unwrap_or_else(|_| EnvFilter::new("info"));  // 改为 info
```

### 方案 C：永久启用调试日志

修改 `Cargo.toml:12`：

```toml
default = ["debug-logs"]  # 默认启用调试日志
```

## 标点引擎工作原理（已验证）

### 停顿检测算法

```
1. 获取 Token 时间戳（来自 Sherpa-ONNX）
   Token[0]: "今天" (0ms - 400ms)
   Token[1]: "天气" (400ms - 800ms)
   Token[2]: "很好" (800ms - 2000ms)  ← 包含停顿
   Token[3]: "我想" (2000ms - 2400ms)

2. 计算 Token 时长
   Token[0]: 400ms
   Token[1]: 400ms
   Token[2]: 1200ms  ← 异常长
   Token[3]: 400ms

3. 计算平均时长
   avg = (400 + 400 + 1200) / 3 = 667ms

4. 检测停顿
   Token[2] 时长 / 平均时长 = 1200 / 667 = 1.8
   如果 1.8 > pause_ratio (2.0)，则在 Token[3] 前插入逗号

5. 结果
   "今天天气很好，我想..."
```

### 逗号插入条件

全部满足才插入：
1. Token 数量 >= 3 (配置: min_tokens)
2. 距离上次逗号 >= 4 个 Token
3. 上一个 Token 时长 >= 500ms
4. 时长比例 > 2.0 (配置: pause_ratio)

## 测试步骤

### 1. 启用调试日志并重新编译

```bash
cd /home/deepin/deepin-v2t
cargo build --release --features debug-logs
sudo cp target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig
```

### 2. 启动 fcitx5 并查看日志

```bash
# 终端 1: 启动 fcitx5
pkill -9 fcitx5
VINPUT_LOG=debug fcitx5

# 终端 2: 查看日志
journalctl --user -u fcitx5 -f | grep -E "(标点配置|停顿检测|Token|逗号)"
```

### 3. 测试语音输入

说话示例（词之间停顿 1 秒）：
```
今天 [停顿1秒] 天气 [停顿1秒] 很好 [停顿1秒] 我想 [停顿1秒] 出去 [停顿1秒] 散步
```

预期结果：
```
今天，天气，很好，我想，出去，散步。
```

### 4. 查看日志输出

应该看到：
```
标点配置: pause_ratio=2.0, min_tokens=3
Token[0]: '今天' (0ms - 400ms, duration=400ms)
Token[1]: '天气' (400ms - 1500ms, duration=1100ms)
停顿检测: 上一Token='天气' 时长=1100ms, 平均=750ms, 比例=1.47, 阈值=2.00
停顿检测: 上一Token='很好' 时长=1300ms, 平均=800ms, 比例=1.63, 阈值=2.00
🎯 检测到停顿，将在 '我想' 前插入逗号
```

## 预期问题和解决

### 问题 1：仍然没有逗号

**可能原因**：
- 停顿不够长（需要 > 平均时长 × 2.0）
- Token 数量不足（< 3 个）
- 说话太快，Token 时长都很短

**解决**：
- 降低 `pause_ratio` 到 1.5
- 降低 `min_tokens` 到 2
- 说话时停顿更明显（1.5 秒以上）

### 问题 2：重复字符

**可能原因**：
- ASR 模型输出重复（INT8 量化影响）
- Preedit 更新逻辑问题

**解决**：
- 查看日志中的 Token 列表
- 检查是否是模型输出重复还是代码重复

## 下一步

1. ✅ 启用 debug-logs 重新编译
2. ✅ 设置 VINPUT_LOG=debug
3. ✅ 测试并查看详细日志
4. ✅ 根据日志调整配置

---

生成时间: 2026-02-17
