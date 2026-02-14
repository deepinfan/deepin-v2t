# PipeWire 音频捕获实现指南

## 当前状态

V-Input 的音频捕获模块支持两种模式：

### 1. 模拟模式（默认）
- **用途**: 开发、测试、无 PipeWire 环境
- **行为**: 生成静音音频数据（16kHz, 单声道）
- **编译**: `cargo build --release`
- **优点**: 无需 PipeWire，可测试完整流程

### 2. 真实模式（需要 PipeWire）
- **用途**: 生产环境、真实音频捕获
- **行为**: 从麦克风捕获真实音频
- **编译**: `cargo build --release --features pipewire-capture`
- **要求**: PipeWire 守护进程运行、麦克风设备可用

## 架构设计

### 条件编译

```rust
// vinput-core/src/audio/pipewire_stream.rs

fn run_pipewire_loop(...) {
    #[cfg(feature = "pipewire-capture")]
    return run_real_pipewire_loop(...);  // 真实实现

    #[cfg(not(feature = "pipewire-capture"))]
    return run_simulated_pipewire_loop(...);  // 模拟实现
}
```

### 模拟实现（已完成）✅

```rust
fn run_simulated_pipewire_loop(...) {
    // 模拟 32ms 音频帧
    loop {
        thread::sleep(Duration::from_millis(32));
        let samples = vec![0.0f32, frame_size];
        producer.write(&samples)?;
    }
}
```

### 真实实现（需完善）⚠️

```rust
fn run_real_pipewire_loop(...) {
    // 1. 初始化 PipeWire
    pipewire::init();
    let mainloop = MainLoop::new(None)?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;

    // 2. 创建音频流
    let stream = Stream::new(&core, name, props)?;

    // 3. 注册回调处理音频数据
    stream.add_local_listener()
        .process(|stream| {
            // 从 PipeWire 读取音频
            // 写入 Ring Buffer
        })
        .register()?;

    // 4. 连接流
    stream.connect(Direction::Input, ...)?;

    // 5. 运行主循环
    while !quit {
        mainloop.iterate(Duration::from_millis(10));
    }
}
```

## 完成真实实现的步骤

### 第 1 步：环境准备

```bash
# 确保 PipeWire 运行
systemctl --user status pipewire

# 检查音频设备
pw-cli ls Node | grep -A 5 "Audio/Source"

# 测试 PipeWire 工具
pw-record --list-targets
```

### 第 2 步：修复类型推断问题

当前的 `run_real_pipewire_loop` 函数有类型推断错误。需要：

1. **明确回调参数类型**
```rust
.process(move |stream: &pipewire::stream::Stream| {
    // ...
})
```

2. **明确缓冲区类型**
```rust
if let Some(mut buffer) = stream.dequeue_buffer() {
    let datas: &mut [pipewire::spa::pod::Pod] = buffer.datas_mut();
    // ...
}
```

3. **使用显式类型转换**
```rust
let samples: &[f32] = unsafe {
    std::slice::from_raw_parts(
        slice.as_ptr() as *const f32,
        size as usize / std::mem::size_of::<f32>(),
    )
};
```

### 第 3 步：参考示例代码

查看 pipewire-rs 官方示例：
- https://gitlab.freedesktop.org/pipewire/pipewire-rs
- examples/audio-src.rs
- examples/audio-capture.rs

### 第 4 步：测试验证

```bash
# 编译真实模式
cargo build --release --features pipewire-capture

# 运行测试
cargo run --example pipewire_capture --features pipewire-capture

# 验证音频捕获
# 应该看到真实音频数据（非零值）
```

## API 参考

### pipewire-rs 0.9 关键 API

```rust
use pipewire::{
    properties,
    spa::{
        param::audio::{AudioFormat, AudioInfoRaw},
        utils::Direction,
    },
    stream::{Stream, StreamFlags, StreamListener},
    Context, MainLoop,
};

// 创建流
let stream = Stream::new(&core, "name", props)?;

// 注册监听器
let _listener = stream
    .add_local_listener()
    .process(|stream| {
        // 音频回调
    })
    .register()?;

// 连接流
stream.connect(
    Direction::Input,
    None,
    StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
    &mut params,
)?;

// 主循环
mainloop.iterate(Duration::from_millis(10));
```

### 音频参数配置

```rust
let audio_info = AudioInfoRaw::new()
    .format(AudioFormat::F32LE)  // 32位浮点
    .rate(16000)                  // 16kHz
    .channels(1);                 // 单声道

let params = vec![audio_info.into()];
```

### 音频数据读取

```rust
.process(|stream| {
    if let Some(mut buffer) = stream.dequeue_buffer() {
        let datas = buffer.datas_mut();
        if let Some(data) = datas.get(0) {
            if let Some(slice) = data.data() {
                // slice 是原始音频数据
                let samples = unsafe {
                    std::slice::from_raw_parts(
                        slice.as_ptr() as *const f32,
                        slice.len() / 4,  // f32 = 4 bytes
                    )
                };

                // 写入 Ring Buffer
                producer.write(samples)?;
            }
        }
    }
})
```

## 常见问题

### Q: 为什么不默认启用真实模式？

A: 因为：
1. PipeWire 环境不是所有系统都有
2. 开发测试时模拟模式更方便
3. 真实模式需要实际环境验证
4. 条件编译避免依赖问题

### Q: 模拟模式能测试什么？

A: 可以测试：
- ✅ FFI 接口
- ✅ Ring Buffer 流程
- ✅ VAD 集成
- ✅ ASR 集成
- ✅ 端点检测
- ✅ 命令生成
- ❌ 真实音频质量

### Q: 如何调试 PipeWire 问题？

```bash
# 启用详细日志
PIPEWIRE_DEBUG=4 cargo run --features pipewire-capture,debug-logs

# 查看 PipeWire 图
pw-dot > graph.dot && dot -Tpng graph.dot > graph.png

# 监控音频流
pw-top
```

### Q: 音频格式如何选择？

- **F32LE**: 推荐，精度高，无需转换
- **S16LE**: 兼容性好，但需要转换

V-Input 使用 F32LE (32位浮点)，因为：
1. Sherpa-ONNX 需要 float 输入
2. 避免 int16 → float 转换
3. PipeWire 原生支持

## 下一步工作

### 优先级 1: 完成真实实现
- [ ] 修复类型推断错误
- [ ] 在实际 PipeWire 环境测试
- [ ] 验证音频数据正确性

### 优先级 2: 设备管理
- [ ] 实现 `enumerate_audio_devices()`
- [ ] 支持设备选择
- [ ] 处理设备热插拔

### 优先级 3: 错误恢复
- [ ] 连接失败重试
- [ ] 设备断开检测
- [ ] 自动重连机制

### 优先级 4: 性能优化
- [ ] 延迟优化
- [ ] 零拷贝优化
- [ ] CPU 占用优化

## 测试清单

在真实环境中验证：

- [ ] PipeWire 守护进程正常运行
- [ ] 能够枚举音频设备
- [ ] 能够打开默认麦克风
- [ ] 音频数据非零（说明有真实输入）
- [ ] 采样率正确（16kHz）
- [ ] 声道数正确（单声道）
- [ ] 格式正确（F32LE）
- [ ] Ring Buffer 无溢出
- [ ] 无内存泄漏
- [ ] 能够正常关闭

## 参考资料

- [PipeWire 官方文档](https://docs.pipewire.org/)
- [pipewire-rs 文档](https://docs.rs/pipewire/)
- [Rust 音频编程指南](https://rust-audio.discourse.group/)
- [V-Input 架构文档](../ARCHITECTURE.md)

---

*最后更新: 2026-02-14*
*状态: 模拟模式完成，真实模式需完善*
