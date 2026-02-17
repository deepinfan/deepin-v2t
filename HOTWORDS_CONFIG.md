# 热词配置指南

## 热词配置路径

V-Input 支持两种热词配置方式：

### 方式 1：在主配置文件中配置（推荐）

**配置文件路径**：`~/.config/vinput/config.toml`

**配置格式**：
```toml
[hotwords]
global_weight = 2.5
max_words = 10000

[hotwords.words]
"深度学习" = 3.0
"人工智能" = 2.5
"神经网络" = 2.5
"机器学习" = 2.0
"自然语言处理" = 2.0
```

**优点**：
- ✅ 配置简单，一个文件管理所有配置
- ✅ 支持为每个词设置不同的权重
- ✅ 自动生效，无需重启

### 方式 2：使用独立的热词文件

**配置文件路径**：`~/.config/vinput/config.toml`

**配置格式**：
```toml
[asr]
model_dir = "/home/deepin/deepin-v2t/models/streaming"
sample_rate = 16000
hotwords_file = "/home/deepin/.config/vinput/hotwords.txt"  # 指定热词文件路径
hotwords_score = 1.5
```

**热词文件格式**（`~/.config/vinput/hotwords.txt`）：
```
深度学习 3.0
人工智能 2.5
神经网络 2.5
机器学习 2.0
自然语言处理 2.0
```

**优点**：
- ✅ 热词文件独立管理
- ✅ 支持大量热词（10000+）
- ✅ 可以动态切换不同的热词文件

## 完整配置示例

### 示例 1：使用主配置文件（推荐）

**文件**：`~/.config/vinput/config.toml`

```toml
[hotwords]
global_weight = 2.5
max_words = 10000

[hotwords.words]
# 技术术语
"深度学习" = 3.0
"人工智能" = 2.5
"神经网络" = 2.5
"机器学习" = 2.0
"自然语言处理" = 2.0

# 编程语言
"Python" = 2.0
"JavaScript" = 2.0
"TypeScript" = 2.0
"Rust" = 2.0

# 框架和工具
"PyTorch" = 2.5
"TensorFlow" = 2.5
"Kubernetes" = 2.0
"Docker" = 2.0

[punctuation]
style = "Professional"
pause_ratio = 2.0
min_tokens = 3
allow_exclamation = false
question_strict = true

[vad]
mode = "PushToTalk"
start_threshold = 0.5
end_threshold = 0.3
min_speech_duration = 250
min_silence_duration = 300

[asr]
model_dir = "/home/deepin/deepin-v2t/models/streaming"
sample_rate = 16000
hotwords_score = 1.5
max_active_paths = 2

[endpoint]
min_speech_duration_ms = 300
max_speech_duration_ms = 30000
trailing_silence_ms = 800
force_timeout_ms = 60000
vad_assisted = true
vad_silence_confirm_frames = 5
```

### 示例 2：使用独立热词文件

**主配置文件**：`~/.config/vinput/config.toml`

```toml
[hotwords]
global_weight = 2.5
max_words = 10000

[asr]
model_dir = "/home/deepin/deepin-v2t/models/streaming"
sample_rate = 16000
hotwords_file = "/home/deepin/.config/vinput/hotwords.txt"
hotwords_score = 1.5
max_active_paths = 2

# ... 其他配置 ...
```

**热词文件**：`~/.config/vinput/hotwords.txt`

```
深度学习 3.0
人工智能 2.5
神经网络 2.5
机器学习 2.0
自然语言处理 2.0
Python 2.0
JavaScript 2.0
TypeScript 2.0
Rust 2.0
PyTorch 2.5
TensorFlow 2.5
Kubernetes 2.0
Docker 2.0
```

## 热词权重说明

### 权重范围

- **1.0 - 1.5**：轻微提升（适合常见词汇）
- **1.5 - 2.5**：中等提升（适合专业术语）
- **2.5 - 4.0**：强力提升（适合罕见词汇）
- **4.0+**：极强提升（可能导致误识别，不推荐）

### 全局权重（global_weight）

- 默认值：2.5
- 作用：统一调整所有热词的权重
- 建议范围：1.5 - 3.5

### 热词得分（hotwords_score）

- 默认值：1.5
- 作用：控制热词在解码时的影响力
- 建议范围：1.0 - 2.0

## 配置生效方式

### 方式 1：重启 fcitx5

```bash
fcitx5 -r
```

### 方式 2：重启 fcitx5 进程

```bash
pkill fcitx5
fcitx5 &
```

### 方式 3：注销重新登录

注销当前会话，重新登录即可。

## 验证热词是否生效

### 1. 查看日志

```bash
VINPUT_LOG=info fcitx5 2>&1 | grep -i hotword
```

应该看到类似输出：
```
未配置热词，跳过热词引擎初始化
# 或
热词引擎初始化成功，加载了 15 个热词
```

### 2. 测试识别

说一些配置的热词，观察识别准确率是否提高。

例如，配置了 "深度学习" 后：
- 修改前：可能识别为 "深度雪洗"
- 修改后：正确识别为 "深度学习"

## 热词文件格式

### 格式 1：简单格式（每行一个词）

```
深度学习
人工智能
神经网络
```

**说明**：使用默认权重（global_weight）

### 格式 2：带权重格式

```
深度学习 3.0
人工智能 2.5
神经网络 2.5
```

**说明**：每个词可以指定不同的权重

### 格式 3：TOML 格式（在 config.toml 中）

```toml
[hotwords.words]
"深度学习" = 3.0
"人工智能" = 2.5
"神经网络" = 2.5
```

**说明**：推荐格式，支持中文词汇和特殊字符

## 常见问题

### Q1: 热词不生效怎么办？

**A**: 检查以下几点：

1. **配置文件路径是否正确**：
   ```bash
   ls -la ~/.config/vinput/config.toml
   ```

2. **配置格式是否正确**：
   ```bash
   cat ~/.config/vinput/config.toml | grep -A 10 "\[hotwords\]"
   ```

3. **是否重启了 fcitx5**：
   ```bash
   fcitx5 -r
   ```

4. **查看日志确认加载**：
   ```bash
   VINPUT_LOG=info fcitx5 2>&1 | grep -i hotword
   ```

### Q2: 热词文件路径可以是相对路径吗？

**A**: 不推荐。建议使用绝对路径：
```toml
hotwords_file = "/home/deepin/.config/vinput/hotwords.txt"
```

如果使用相对路径，可能会因为工作目录不同而找不到文件。

### Q3: 可以同时使用两种配置方式吗？

**A**: 可以，但不推荐。如果同时配置：
- `[hotwords.words]` 中的热词
- `hotwords_file` 指定的文件

两者会合并，但可能导致混淆。建议只使用一种方式。

### Q4: 热词数量有限制吗？

**A**: 有限制：
- 默认最大数量：10000 个
- 可以通过 `max_words` 配置调整
- 热词过多会增加内存占用和识别延迟

### Q5: 如何添加英文热词？

**A**: 直接添加即可：
```toml
[hotwords.words]
"Python" = 2.0
"JavaScript" = 2.0
"TensorFlow" = 2.5
```

或在热词文件中：
```
Python 2.0
JavaScript 2.0
TensorFlow 2.5
```

### Q6: 热词权重设置多少合适？

**A**: 根据词汇类型：
- **常见词汇**：1.5 - 2.0
- **专业术语**：2.0 - 2.5
- **罕见词汇**：2.5 - 3.5
- **极罕见词汇**：3.5 - 4.0

不建议超过 4.0，可能导致误识别。

## 热词使用场景

### 场景 1：技术文档输入

```toml
[hotwords.words]
"深度学习" = 3.0
"卷积神经网络" = 3.0
"循环神经网络" = 3.0
"Transformer" = 3.0
"BERT" = 3.0
"GPT" = 3.0
```

### 场景 2：编程代码输入

```toml
[hotwords.words]
"Python" = 2.5
"JavaScript" = 2.5
"TypeScript" = 2.5
"async" = 2.0
"await" = 2.0
"Promise" = 2.0
"callback" = 2.0
```

### 场景 3：医学术语输入

```toml
[hotwords.words]
"高血压" = 2.5
"糖尿病" = 2.5
"冠心病" = 2.5
"心肌梗死" = 3.0
"脑卒中" = 3.0
```

### 场景 4：法律文书输入

```toml
[hotwords.words]
"原告" = 2.0
"被告" = 2.0
"诉讼" = 2.0
"判决" = 2.0
"上诉" = 2.0
"仲裁" = 2.5
```

## 性能影响

### 内存占用

- 每个热词约占用 50-100 字节
- 1000 个热词约占用 50-100 KB
- 10000 个热词约占用 500KB - 1MB

### CPU 占用

- 热词数量 < 1000：几乎无影响
- 热词数量 1000-5000：轻微影响（< 1%）
- 热词数量 5000-10000：中等影响（1-2%）
- 热词数量 > 10000：显著影响（> 2%）

### 识别延迟

- 热词数量 < 1000：无影响
- 热词数量 1000-5000：< 5ms
- 热词数量 5000-10000：5-10ms
- 热词数量 > 10000：> 10ms

## 最佳实践

1. **只添加真正需要的热词**
   - 不要添加常见词汇（如"你好"、"谢谢"）
   - 只添加容易识别错误的专业术语

2. **合理设置权重**
   - 从低权重开始（1.5-2.0）
   - 如果仍然识别错误，逐步提高权重
   - 避免设置过高权重（> 4.0）

3. **定期清理无用热词**
   - 删除不再使用的热词
   - 保持热词列表精简

4. **按场景分类管理**
   - 可以创建多个热词文件
   - 根据不同场景切换热词文件

5. **测试验证**
   - 添加热词后进行测试
   - 确认识别准确率是否提高
   - 避免引入误识别

## 故障排查

### 问题 1：配置文件不存在

```bash
# 创建配置目录
mkdir -p ~/.config/vinput

# 创建配置文件
cat > ~/.config/vinput/config.toml << 'EOF'
[hotwords]
global_weight = 2.5
max_words = 10000

[hotwords.words]
"深度学习" = 3.0
"人工智能" = 2.5

[asr]
model_dir = "/home/deepin/deepin-v2t/models/streaming"
sample_rate = 16000
hotwords_score = 1.5
EOF
```

### 问题 2：配置格式错误

```bash
# 验证 TOML 格式
python3 -c "import toml; toml.load(open('$HOME/.config/vinput/config.toml'))"
```

### 问题 3：热词文件找不到

```bash
# 检查文件是否存在
ls -la ~/.config/vinput/hotwords.txt

# 检查文件权限
chmod 644 ~/.config/vinput/hotwords.txt
```

---

**文档时间**: 2026-02-17
**文档作者**: Claude Code
