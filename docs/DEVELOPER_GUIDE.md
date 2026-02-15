# V-Input 开发者文档

## 项目架构

V-Input 采用模块化架构，主要由以下组件构成:

```
vinput/
├── vinput-core/          # Rust 核心引擎
│   ├── src/
│   │   ├── audio/        # 音频捕获 (PipeWire)
│   │   ├── vad/          # 语音活动检测
│   │   ├── asr/          # 语音识别 (sherpa-onnx)
│   │   ├── streaming/    # 流式识别管道
│   │   ├── endpointing/  # 端点检测
│   │   ├── itn/          # 文本规范化
│   │   ├── punctuation/  # 智能标点
│   │   ├── hotwords/     # 热词引擎
│   │   ├── undo/         # 撤销/重试
│   │   ├── ffi/          # FFI 接口
│   │   └── config/       # 配置管理
│   └── Cargo.toml
├── fcitx5-vinput/        # Fcitx5 C++ 插件
│   ├── src/
│   │   └── vinput_engine.cpp
│   ├── include/
│   │   ├── vinput_engine.h
│   │   └── vinput_core.h  # 自动生成的 C 头文件
│   └── CMakeLists.txt
├── vinput-gui/           # Rust GUI 设置界面 (egui)
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── basic_settings_panel.rs
│   │   ├── hotwords_editor.rs
│   │   ├── punctuation_panel.rs
│   │   ├── vad_asr_panel.rs
│   │   └── endpoint_panel.rs
│   └── Cargo.toml
└── models/               # ASR 模型文件
    └── streaming/
```

## 核心模块

### 1. 音频捕获 (audio)

使用 PipeWire 捕获实时音频流。

**关键文件**:
- `vinput-core/src/audio/pipewire_stream.rs`
- `vinput-core/src/audio/ring_buffer.rs`

**工作流程**:
1. 创建 PipeWire 音频流
2. 启动 `pw-record` 子进程
3. 从 stdout 读取 f32 音频数据
4. 写入环形缓冲区供 VAD/ASR 消费

**示例代码**:
```rust
let config = PipeWireStreamConfig {
    sample_rate: 16000,
    channels: 1,
    format: AudioFormat::F32LE,
    ..Default::default()
};

let ring_buffer = AudioRingBuffer::new(AudioRingBufferConfig { capacity: 16000 });
let (producer, consumer) = ring_buffer.split();

let stream = PipeWireStream::new(config, producer)?;
```

### 2. 语音活动检测 (VAD)

使用 Silero VAD 模型检测语音活动。

**关键文件**:
- `vinput-core/src/vad/silero.rs`

**工作流程**:
1. 加载 Silero VAD ONNX 模型
2. 每 512 samples (32ms @ 16kHz) 运行一次推理
3. 输出语音概率 (0.0-1.0)
4. 根据阈值判断语音开始/结束

**示例代码**:
```rust
let vad = SileroVAD::new("models/silero_vad.onnx")?;

for chunk in audio_chunks {
    let probability = vad.process(chunk)?;
    if probability > 0.5 {
        println!("Speech detected!");
    }
}
```

### 3. 语音识别 (ASR)

使用 sherpa-onnx 进行流式语音识别。

**关键文件**:
- `vinput-core/src/asr/sherpa.rs`

**工作流程**:
1. 加载 Zipformer 流式模型
2. 创建识别流 (RecognitionStream)
3. 持续送入音频帧
4. 获取部分结果和最终结果

**示例代码**:
```rust
let recognizer = SherpaRecognizer::new(config)?;
let mut stream = recognizer.create_stream()?;

stream.accept_waveform(16000, &audio_samples);

let partial = stream.get_result();
println!("Partial: {}", partial.text);

if stream.is_endpoint() {
    let final_result = stream.get_result();
    println!("Final: {}", final_result.text);
    stream.reset();
}
```

### 4. 流式识别管道 (streaming)

集成 VAD + ASR + 标点 + 端点检测的完整管道。

**关键文件**:
- `vinput-core/src/streaming/pipeline.rs`

**工作流程**:
1. 接收音频帧
2. VAD 检测语音活动
3. ASR 识别文本
4. 端点检测判断句子结束
5. 智能标点添加标点符号
6. 返回带标点的最终结果

**示例代码**:
```rust
let pipeline = StreamingPipeline::new(config)?;

loop {
    let result = pipeline.process(&audio_frame)?;

    if !result.partial_result.is_empty() {
        println!("Partial: {}", result.partial_result);
    }

    if result.pipeline_state == PipelineState::Completed {
        let final_text = pipeline.get_final_result_with_punctuation();
        println!("Final: {}", final_text);
        pipeline.reset()?;
    }
}
```

### 5. 端点检测 (endpointing)

判断句子结束时机。

**关键文件**:
- `vinput-core/src/endpointing/detector.rs`

**检测策略**:
1. **最小语音长度**: 至少说话 300ms
2. **尾随静音**: 说话结束后等待 800ms
3. **最大语音长度**: 超过 30s 自动结束
4. **VAD 辅助**: VAD 检测到静音后确认
5. **能量检测**: RMS 梯度分析检测疑问语调

**示例代码**:
```rust
let mut detector = EndpointDetector::new(config);

detector.feed_audio(&audio_samples);

let result = detector.detect(
    vad_probability,
    asr_has_result,
    elapsed_ms,
);

match result {
    EndpointResult::Continue => { /* 继续录音 */ }
    EndpointResult::Endpoint => { /* 句子结束 */ }
    EndpointResult::Timeout => { /* 超时 */ }
}
```

### 6. 文本规范化 (ITN)

将语音识别结果规范化为书面文本。

**关键文件**:
- `vinput-core/src/itn/engine.rs`
- `vinput-core/src/itn/chinese_number.rs`
- `vinput-core/src/itn/rules.rs`

**规则**:
1. **数字转换**: "一千二百三十四" → "1234"
2. **日期转换**: "二零二六年三月五日" → "2026年3月5日"
3. **货币转换**: "三百块钱" → "¥300"
4. **百分比转换**: "百分之五十" → "50%"
5. **单位转换**: "five kilometers" → "5km"

**示例代码**:
```rust
let engine = ITNEngine::new(ITNMode::Auto);
let result = engine.process("我花了三百块钱");
println!("{}", result.text);  // "我花了¥300"
```

### 7. 智能标点 (punctuation)

根据停顿和语调自动添加标点符号。

**关键文件**:
- `vinput-core/src/punctuation/engine.rs`
- `vinput-core/src/punctuation/pause_engine.rs`

**策略**:
1. **逗号**: 检测停顿时长 > pause_ratio * avg_token_duration
2. **句号**: 句子结束 + 无疑问语调
3. **问号**: 句子结束 + 疑问语调（能量上升）

**示例代码**:
```rust
let engine = PunctuationEngine::new(config);

let tokens = vec![
    TokenInfo { text: "今天".to_string(), start_ms: 0, end_ms: 500 },
    TokenInfo { text: "天气".to_string(), start_ms: 1200, end_ms: 1700 },
    TokenInfo { text: "很好".to_string(), start_ms: 1800, end_ms: 2300 },
];

let result = engine.process(&tokens, false);
println!("{}", result);  // "今天，天气很好。"
```

### 8. 热词引擎 (hotwords)

提升特定词汇的识别准确率。

**关键文件**:
- `vinput-core/src/hotwords/engine.rs`
- `vinput-core/src/hotwords/parser.rs`

**使用方法**:
```rust
let mut engine = HotwordsEngine::new();
engine.add_hotword("深度学习".to_string(), 2.8)?;
engine.add_hotword("人工智能".to_string(), 2.5)?;

// 生成 sherpa-onnx 格式
let hotwords_str = engine.to_sherpa_format();

// 在 ASR 配置中使用
let asr_config = AsrConfig {
    hotwords_file: Some("/tmp/hotwords.txt"),
    hotwords_score: engine.global_weight(),
    ..Default::default()
};
```

### 9. 撤销/重试 (undo)

记录识别历史，支持撤销和重试。

**关键文件**:
- `vinput-core/src/undo.rs`

**使用方法**:
```rust
let mut history = RecognitionHistory::new(50);

// 添加识别结果
history.push("第一句话".to_string());
history.push("第二句话".to_string());

// 撤销
if let Some(undone) = history.undo() {
    println!("撤销: {}", undone);
}

// 重试
if let Some(redone) = history.redo() {
    println!("重试: {}", redone);
}
```

## FFI 接口

V-Input Core 通过 FFI 暴露 C 接口供 Fcitx5 插件调用。

**关键文件**:
- `vinput-core/src/ffi/exports.rs`
- `vinput-core/src/ffi/types.rs`
- `fcitx5-vinput/include/vinput_core.h` (自动生成)

### 初始化

```c
VInputVInputFFIResult result = vinput_core_init();
if (result == Success) {
    printf("V-Input Core initialized\\n");
}
```

### 注册回调

```c
void my_callback(const VInputVInputCommand* command) {
    if (command->command_type == CommitText) {
        printf("Commit: %s\\n", command->text);
    }
}

vinput_core_register_callback(my_callback);
```

### 发送事件

```c
VInputVInputEvent event;
event.event_type = StartRecording;
event.data = NULL;
event.data_len = 0;

vinput_core_send_event(&event);
```

### 接收命令

```c
VInputVInputCommand command;
VInputVInputFFIResult result = vinput_core_try_recv_command(&command);

if (result == Success) {
    // 处理命令
    vinput_command_free(&command);
}
```

## Fcitx5 插件

**关键文件**:
- `fcitx5-vinput/src/vinput_engine.cpp`
- `fcitx5-vinput/include/vinput_engine.h`

### 生命周期

```cpp
class VInputEngine : public InputMethodEngine {
public:
    VInputEngine(Instance* instance);
    ~VInputEngine() override;

    void activate(const InputMethodEntry& entry, InputContextEvent& event) override;
    void deactivate(const InputMethodEntry& entry, InputContextEvent& event) override;
    void reset(const InputMethodEntry& entry, InputContextEvent& event) override;
    void keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) override;
};
```

### 按键处理

```cpp
void VInputEngine::keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) {
    if (keyEvent.key().check(FcitxKey_space)) {
        if (is_recording_) {
            stopRecording();
        } else {
            startRecording();
        }
        keyEvent.filterAndAccept();
    }
}
```

### 命令处理

```cpp
void VInputEngine::handleCommand(const VInputVInputCommand* command) {
    auto* ic = g_vinput_engine_instance->instance_->mostRecentInputContext();

    switch (command->command_type) {
        case CommitText:
            ic->commitString(std::string(command->text, command->text_len));
            break;
        case UndoText:
            for (size_t i = 0; i < command->text_len; ++i) {
                ic->forwardKey(Key(FcitxKey_BackSpace));
            }
            break;
    }
}
```

## 构建系统

### Rust 核心引擎

```bash
cd vinput-core
cargo build --release
```

生成:
- `target/release/libvinput_core.so`
- `target/vinput_core.h` (自动生成)

### Fcitx5 插件

```bash
cd fcitx5-vinput
mkdir build && cd build
cmake ..
make
sudo make install
```

生成:
- `vinput.so` → `/usr/lib/fcitx5/vinput.so`
- `vinput.conf` → `/usr/share/fcitx5/addon/vinput.conf`
- `vinput-addon.conf` → `/usr/share/fcitx5/inputmethod/vinput.conf`

### GUI 设置界面

```bash
cd vinput-gui
cargo build --release
```

生成:
- `target/release/vinput-gui`

## 测试

### 单元测试

```bash
# 测试所有模块
cargo test

# 测试特定模块
cargo test --lib itn
cargo test --lib punctuation
cargo test --lib undo
```

### 集成测试

```bash
# 测试 ITN 货币规则
cargo run --example test_currency_itn

# 测试设备枚举
cargo run --example test_device_enum
```

### 手动测试

```bash
# 启动 Fcitx5 并查看日志
VINPUT_LOG=1 fcitx5 -r

# 查看 journald 日志
journalctl --user -u fcitx5 -f
```

## 调试

### Rust 日志

```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 仅 V-Input 日志
RUST_LOG=vinput_core=debug cargo run
```

### C++ 日志

Fcitx5 使用 `FCITX_INFO()`, `FCITX_DEBUG()`, `FCITX_ERROR()` 宏:

```cpp
FCITX_INFO() << "V-Input: 开始录音";
FCITX_DEBUG() << "VAD probability: " << prob;
FCITX_ERROR() << "识别失败: " << error;
```

### GDB 调试

```bash
# 调试 Fcitx5 插件
gdb --args fcitx5 -r

# 设置断点
(gdb) break VInputEngine::startRecording
(gdb) run
```

## 性能优化

### 冷启动优化

1. 预加载模型文件
2. 使用 mmap 加载大文件
3. 延迟初始化非关键组件

### 内存优化

1. 使用环形缓冲区避免频繁分配
2. 复用 ASR 流对象
3. 限制历史记录数量

### 延迟优化

1. 使用零拷贝音频传输
2. 异步处理非关键任务
3. 优化端点检测参数

## 贡献指南

### 代码风格

- Rust: 使用 `rustfmt` 格式化
- C++: 遵循 Fcitx5 代码风格
- 提交前运行 `cargo clippy`

### 提交流程

1. Fork 项目
2. 创建特性分支: `git checkout -b feature/my-feature`
3. 提交更改: `git commit -m "feat: add my feature"`
4. 推送分支: `git push origin feature/my-feature`
5. 创建 Pull Request

### 测试要求

- 所有新功能必须包含单元测试
- 测试覆盖率 > 80%
- 通过 CI 检查

## 许可证

V-Input 采用 MIT 许可证。详见 LICENSE 文件。
