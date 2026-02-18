# VAD 参数调整 - 解决末尾字丢失和背景噪音问题

## 问题描述

### 问题 1：最后一个字丢失
**现象**：说话时最后一个字发音较轻，经常被截断丢失。

**原因**：端点检测的静音等待时间太短，最后一个字还没说完就被判定为结束。

### 问题 2：背景噪音干扰
**现象**：环境中其他人说话时，会被误识别并输出文字。

**原因**：VAD 启动阈值太低，把背景说话也当成了有效语音。

---

## 参数调整

### 1. VAD 启动阈值（减少背景噪音）

**文件**：`vinput-core/src/vad/config.rs`

```rust
// 修改前
start_threshold: 0.6,  // 较低，容易被背景噪音触发

// 修改后
start_threshold: 0.7,  // 提高阈值，减少误触发
```

**效果**：
- ✅ 需要更明确的语音信号才会开始识别
- ✅ 背景说话不容易触发
- ⚠️ 说话声音太小可能无法触发（需要稍微大声一点）

---

### 2. 静音持续时间（防止末尾字丢失）

**文件**：`vinput-core/src/vad/config.rs`

```rust
// 修改前
min_silence_duration_ms: 500,  // 500ms 静音后判定结束

// 修改后
min_silence_duration_ms: 700,  // 700ms 静音后判定结束
```

**效果**：
- ✅ 给最后一个字更多的识别时间
- ✅ 轻声说话的末尾字不容易丢失
- ⚠️ 说话停顿稍长（0.7 秒）才会结束

---

### 3. 端点检测尾部静音（进一步保护末尾字）

**文件**：`vinput-core/src/endpointing/detector.rs`

```rust
// 修改前
trailing_silence_ms: 800,           // 800ms 尾部静音
vad_silence_confirm_frames: 5,      // 5 帧确认（约 160ms）

// 修改后
trailing_silence_ms: 1000,          // 1000ms 尾部静音
vad_silence_confirm_frames: 8,      // 8 帧确认（约 256ms）
```

**效果**：
- ✅ 双重保护，确保末尾字不被截断
- ✅ 更稳定的端点检测
- ⚠️ 整体响应时间略微增加（约 0.2 秒）

---

## 参数对比表

| 参数 | 修改前 | 修改后 | 影响 |
|------|--------|--------|------|
| VAD 启动阈值 | 0.6 | **0.7** | 减少背景噪音误触发 |
| 最小静音时长 | 500ms | **700ms** | 防止末尾字丢失 |
| 尾部静音等待 | 800ms | **1000ms** | 进一步保护末尾字 |
| 静音确认帧数 | 5 帧 (160ms) | **8 帧 (256ms)** | 更稳定的端点检测 |

---

## 用户可调整参数

如果默认参数不适合你的使用场景，可以在配置文件中自定义：

**配置文件**：`~/.config/vinput/config.toml`

```toml
[vad.hysteresis]
# VAD 启动阈值（0.0-1.0）
# 越高越不容易被背景噪音触发，但需要更大声说话
start_threshold = 0.7

# VAD 结束阈值（0.0-1.0）
# 低于此值判定为静音
end_threshold = 0.35

# 最小静音持续时间（毫秒）
# 静音超过此时间才判定语音结束
min_silence_duration_ms = 700

# 最小语音持续时间（毫秒）
# 过滤掉短暂的点击音等
min_speech_duration_ms = 100

[endpoint]
# 尾部静音等待时间（毫秒）
# 语音结束后等待多久才上屏
trailing_silence_ms = 1000

# VAD 静音确认帧数
# 连续多少帧静音才确认结束
vad_silence_confirm_frames = 8
```

---

## 使用场景建议

### 场景 1：安静环境（办公室、家里）
**推荐配置**：默认配置即可
```toml
[vad.hysteresis]
start_threshold = 0.7
min_silence_duration_ms = 700
```

### 场景 2：嘈杂环境（咖啡厅、公共场所）
**推荐配置**：提高阈值，减少干扰
```toml
[vad.hysteresis]
start_threshold = 0.8          # 更高的启动阈值
min_silence_duration_ms = 800  # 更长的静音等待
```

### 场景 3：说话声音较小
**推荐配置**：降低阈值，增加灵敏度
```toml
[vad.hysteresis]
start_threshold = 0.6          # 较低的启动阈值
min_silence_duration_ms = 800  # 更长的静音等待（保护末尾字）
```

### 场景 4：快速输入（追求速度）
**推荐配置**：减少等待时间
```toml
[vad.hysteresis]
min_silence_duration_ms = 500

[endpoint]
trailing_silence_ms = 700
vad_silence_confirm_frames = 5
```

---

## 测试方法

### 测试 1：末尾字是否丢失
说以下句子，观察最后一个字是否完整：
- "今天天气很好" → 检查"好"是否完整
- "我要去吃饭" → 检查"饭"是否完整
- "这个问题很复杂" → 检查"杂"是否完整

### 测试 2：背景噪音抗干扰
在有其他人说话的环境中：
- 不说话时，观察是否会误触发
- 其他人说话时，观察是否会输出文字
- 自己说话时，观察是否能正常识别

### 测试 3：响应速度
说完一句话后，观察多久上屏：
- 理想：0.7-1.0 秒内上屏
- 可接受：1.0-1.5 秒内上屏
- 太慢：超过 1.5 秒

---

## 故障排除

### Q1: 末尾字还是丢失
**解决方案**：进一步增加静音等待时间
```toml
[vad.hysteresis]
min_silence_duration_ms = 900  # 增加到 900ms

[endpoint]
trailing_silence_ms = 1200     # 增加到 1200ms
```

### Q2: 背景噪音还是干扰
**解决方案**：进一步提高启动阈值
```toml
[vad.hysteresis]
start_threshold = 0.8  # 提高到 0.8
```

### Q3: 说话无法触发
**解决方案**：降低启动阈值或增大说话音量
```toml
[vad.hysteresis]
start_threshold = 0.65  # 降低到 0.65
```

### Q4: 响应太慢
**解决方案**：减少等待时间（但可能导致末尾字丢失）
```toml
[vad.hysteresis]
min_silence_duration_ms = 600

[endpoint]
trailing_silence_ms = 800
vad_silence_confirm_frames = 6
```

---

## 已更新

- ✅ 修改了默认参数
- ✅ 重新编译
- ✅ 重新打包 DEB

新的 DEB 包：`droplet-voice-input_0.1.0_amd64.deb` (2026-02-18)

安装后即可使用新的参数配置。
