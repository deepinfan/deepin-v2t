# V-Input 可执行代码架构设计

> **Version**: 1.0
> **Last Updated**: 2026-02-13
> **Status**: Initial Architecture
> **Author**: V-Input Architecture Team

---

## 目录

1. [架构总览](#1-架构总览)
2. [模块划分与目录结构](#2-模块划分与目录结构)
3. [核心接口设计](#3-核心接口设计)
4. [并发模型设计](#4-并发模型设计)
5. [数据流设计](#5-数据流设计)
6. [关键技术决策](#6-关键技术决策)
7. [性能关键路径优化](#7-性能关键路径优化)
8. [MVP 优先级](#8-mvp-优先级)
9. [潜在风险与缓解](#9-潜在风险与缓解)
10. [C++ Fcitx5 插件核心实现](#10-c-fcitx5-插件核心实现)
11. [实施建议](#11-实施建议)
12. [ADR 记录](#12-adr-记录)

---

## 1. 架构总览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         用户层 (User Layer)                              │
│                                                                         │
│  ┌─────────────┐   ┌─────────────────────┐   ┌───────────────────────┐ │
│  │ 系统托盘图标  │   │ vinput-settings     │   │ Fcitx5 Preedit/Commit │ │
│  │ (libappind.) │   │ (Qt5/QML独立进程)    │   │ (宿主应用显示)        │ │
│  └──────┬───────┘   └──────────┬──────────┘   └───────────┬───────────┘ │
│         │                      │ D-Bus                     │             │
└─────────┼──────────────────────┼───────────────────────────┼─────────────┘
          │                      │                           │
┌─────────┼──────────────────────┼───────────────────────────┼─────────────┐
│         │               配置层 (Config Layer)              │             │
│         │                      │                           │             │
│         │          ┌───────────▼──────────┐                │             │
│         │          │ config.toml          │                │             │
│         │          │ hotwords.txt         │                │             │
│         │          │ (inotify监听热重载)   │                │             │
│         │          └───────────┬──────────┘                │             │
│         │                      │                           │             │
└─────────┼──────────────────────┼───────────────────────────┼─────────────┘
          │                      │                           │
┌─────────▼──────────────────────▼───────────────────────────▼─────────────┐
│                    Fcitx5 插件层 (C++ Plugin)                             │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ fcitx5-vinput (C++ InputMethodEngine)                           │    │
│  │  ┌──────────────┐  ┌────────────────┐  ┌────────────────────┐  │    │
│  │  │ keyEvent()   │  │ processTick()  │  │ focusIn/Out()      │  │    │
│  │  │ 热键拦截     │  │ 命令队列轮询   │  │ 焦点事件转发       │  │    │
│  │  └──────┬───────┘  └───────┬────────┘  └────────┬───────────┘  │    │
│  │         │                  │                     │              │    │
│  │         │    FFI (extern "C")                    │              │    │
│  │         ▼──────────────────▼─────────────────────▼              │    │
│  │  ┌─────────────────────────────────────────────────────┐       │    │
│  │  │ vinput_core_send_event() / vinput_core_try_recv()   │       │    │
│  │  │ vinput_core_init()      / vinput_core_shutdown()    │       │    │
│  │  └─────────────────────────────────────────────────────┘       │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │
                                     │ libvinput_core.so (cdylib)
                                     │
┌────────────────────────────────────▼────────────────────────────────────┐
│                     Rust 核心引擎 (vinput-core)                          │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     FFI 安全边界层 (ffi/)                        │   │
│  │   catch_unwind + CString 转换 + 生命周期管理                    │   │
│  └──────────────────────────────┬───────────────────────────────────┘   │
│                                 │                                       │
│  ┌──────────────────────────────▼───────────────────────────────────┐   │
│  │              状态机主循环 (state_machine/) [SM线程]               │   │
│  │                                                                  │   │
│  │  Idle ──► WaitingForSpeech ──► Streaming ──► SilenceDetected    │   │
│  │   ▲              │                              │                │   │
│  │   │              ▼                              ▼                │   │
│  │   │         Error ◄────────────────────── Finalizing             │   │
│  │   │                                            │                 │   │
│  │   └────────────────────────────────────────────┘                 │   │
│  └──────────────┬─────────────────┬─────────────────┬──────────────┘   │
│                 │                 │                  │                   │
│     ┌───────────▼──────┐  ┌──────▼───────┐  ┌──────▼───────┐          │
│     │   VAD 线程       │  │  ASR 线程    │  │ Audio 线程   │          │
│     │   (vad/)         │  │  (asr/)      │  │ (audio/)     │          │
│     │                  │  │              │  │              │          │
│     │ Silero ONNX      │  │ sherpa-onnx  │  │ PipeWire     │          │
│     │ Energy Gate      │  │ Recognizer   │  │ Stream API   │          │
│     │ Hysteresis       │  │ Token Stream │  │              │          │
│     │ Pre-roll Buffer  │  │ Hotwords     │  │ SPSC Ring    │          │
│     └────────┬─────────┘  └──────┬───────┘  └──────┬───────┘          │
│              │ mpsc               │ mpsc            │ rtrb(SPSC)       │
│              ▼                    ▼                  ▼                   │
│     ┌─────────────────────────────────────────────────────────┐        │
│     │              后处理管线 (pipeline/)                       │        │
│     │  ┌──────┐   ┌───────────┐   ┌──────┐   ┌────────────┐ │        │
│     │  │ ITN  │──►│ Punctuation│──►│ Undo │──►│ FcitxCommand││        │
│     │  └──────┘   └───────────┘   └──────┘   └────────────┘ │        │
│     └─────────────────────────────────────────────────────────┘        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 依赖关系图

```
                    ┌──────────────────────┐
                    │   vinput-settings    │
                    │   (Qt5/QML独立进程)   │
                    └──────────┬───────────┘
                               │ D-Bus + config.toml
                               ▼
┌──────────────────┐    ┌──────────────────┐
│ fcitx5-vinput    │───►│ libvinput_core.so│
│ (C++ .so plugin) │ FFI│ (Rust cdylib)    │
└──────────────────┘    └────────┬─────────┘
         │                       │
         │ 动态链接               │ 链接
         ▼                       ▼
    libfcitx5core.so     libsherpa-onnx-c-api.so
    libfcitx5utils.so    libonnxruntime.so
                         libpipewire-0.3.so
```

**关键约束**: fcitx5-vinput 动态链接 libvinput_core.so，不静态编入。这确保许可证隔离 (LGPL vs Apache-2.0)。

---

## 2. 模块划分与目录结构

```
deepin-v2t/
├── Cargo.toml                    # Workspace 根配置
├── vinput-core/                  # Rust 核心引擎 (cdylib)
│   ├── Cargo.toml
│   ├── build.rs                  # sherpa-onnx 链接配置
│   ├── cbindgen.toml             # C 头文件自动生成
│   └── src/
│       ├── lib.rs                # crate 入口, 模块声明
│       │
│       ├── ffi/                  # FFI 安全边界层
│       │   ├── mod.rs
│       │   ├── exports.rs        # extern "C" 导出函数 (全部在此)
│       │   ├── types.rs          # C-repr 结构体 (VInputCommand, VInputEvent)
│       │   └── safety.rs         # catch_unwind 包装, CString 转换
│       │
│       ├── engine.rs             # VoiceInputEngine 总入口 (线程协调器)
│       │
│       ├── state_machine/        # 状态机核心
│       │   ├── mod.rs
│       │   ├── state.rs          # VoiceInputState 枚举
│       │   ├── event.rs          # VoiceInputEvent 枚举
│       │   ├── transition.rs     # 状态转换逻辑 (纯函数)
│       │   └── command.rs        # FcitxCommand 输出类型
│       │
│       ├── audio/                # 音频采集
│       │   ├── mod.rs
│       │   ├── pipewire_stream.rs  # PipeWire Stream 封装
│       │   ├── ring_buffer.rs      # rtrb SPSC 封装
│       │   └── device.rs           # 设备枚举 (PipeWire node 列表)
│       │
│       ├── vad/                  # 语音活动检测
│       │   ├── mod.rs
│       │   ├── silero.rs         # Silero ONNX 推理封装
│       │   ├── energy_gate.rs    # RMS 能量门控
│       │   ├── hysteresis.rs     # 双阈值迟滞控制器
│       │   ├── pre_roll.rs       # 350ms Pre-roll 环形缓冲
│       │   └── config.rs         # VadConfig (PushToTalk/AutoDetect 参数)
│       │
│       ├── asr/                  # ASR 引擎
│       │   ├── mod.rs
│       │   ├── recognizer.rs     # sherpa-onnx OnlineRecognizer 封装
│       │   ├── config.rs         # 模型路径/线程数等配置
│       │   └── token_stream.rs   # Token 流式输出抽象
│       │
│       ├── endpointing/          # 动态断句判定
│       │   ├── mod.rs
│       │   ├── pause_engine.rs   # 停顿判定引擎 (CPS + RMS + Semantic)
│       │   ├── cps_tracker.rs    # 字符产出速度跟踪器
│       │   └── semantic_guard.rs # 语义后缀护卫 (助词/连词拦截)
│       │
│       ├── itn/                  # 反向文本正则化
│       │   ├── mod.rs
│       │   ├── pipeline.rs       # ITN 管线入口 (按序执行各规则)
│       │   ├── tokenizer.rs      # Block 分段器
│       │   ├── cn_number.rs      # 中文数字转换 (cn2an-rs)
│       │   ├── en_number.rs      # 英文数字解析
│       │   ├── currency.rs       # 金额规则
│       │   ├── date.rs           # 日期规则
│       │   ├── percentage.rs     # 百分比规则
│       │   ├── unit.rs           # 单位规则
│       │   ├── context_guard.rs  # URL/代码片段跳过
│       │   └── colloquial_guard.rs # 口语表达保护
│       │
│       ├── punctuation/          # 标点引擎
│       │   ├── mod.rs
│       │   ├── streaming.rs      # Streaming 阶段逗号逻辑
│       │   ├── rules.rs          # 规则增强层 (逻辑连词/问号)
│       │   └── profile.rs        # StyleProfile 参数
│       │
│       ├── hotwords/             # 热词引擎
│       │   ├── mod.rs
│       │   ├── engine.rs         # HotwordsEngine (加载/合并/格式化)
│       │   ├── loader.rs         # hotwords.txt / hotwords.toml 解析
│       │   └── watcher.rs        # inotify 文件监听
│       │
│       ├── undo/                 # 撤销/重试
│       │   ├── mod.rs
│       │   ├── manager.rs        # UndoManager (VecDeque<UndoEntry>)
│       │   └── entry.rs          # UndoEntry 数据结构
│       │
│       ├── config/               # 配置管理
│       │   ├── mod.rs
│       │   ├── schema.rs         # VInputConfig (serde + TOML)
│       │   └── watcher.rs        # config.toml inotify 热重载
│       │
│       └── error.rs              # 统一错误类型 (thiserror)
│
├── fcitx5-vinput/                # C++ Fcitx5 插件
│   ├── CMakeLists.txt
│   ├── vinput-im.conf.in         # 输入法声明
│   ├── vinput.conf.in             # Addon 配置
│   └── src/
│       ├── vinput_engine.h        # VInputEngine : public InputMethodEngineV2
│       ├── vinput_engine.cpp      # 核心实现
│       ├── vinput_ffi.h           # cbindgen 生成的 C 头文件 (build时copy)
│       └── vinput_candidate.h     # 候选词列表 (v1.1 预留)
│
├── vinput-settings/              # Qt5/QML GUI (独立进程)
│   ├── CMakeLists.txt
│   ├── src/
│   │   ├── main.cpp              # 应用入口
│   │   ├── settings_backend.h    # C++ 后端 (配置读写, D-Bus)
│   │   ├── settings_backend.cpp
│   │   ├── audio_tester.h        # 音频测试 (录制/播放)
│   │   ├── audio_tester.cpp
│   │   ├── model_downloader.h    # 模型下载器 (HTTP Range)
│   │   └── model_downloader.cpp
│   ├── qml/
│   │   ├── main.qml              # 主窗口 + 侧边栏导航
│   │   ├── BasicSettings.qml     # 标签页1: 基本设置
│   │   ├── RecognitionSettings.qml # 标签页2: 识别设置
│   │   ├── HotwordsPage.qml      # 标签页3: 热词管理
│   │   ├── ModelManager.qml       # 标签页4: 模型管理
│   │   ├── AdvancedSettings.qml   # 标签页5: 高级设置
│   │   ├── AboutPage.qml          # 标签页6: 关于
│   │   └── components/
│   │       ├── HotkeyRecorder.qml  # 热键录制组件
│   │       ├── VolumeBar.qml       # 音量监控条
│   │       └── ProgressBar.qml     # 下载进度条
│   └── resources/
│       ├── icons/                  # 托盘图标 (5种状态)
│       └── vinput-settings.desktop
│
├── models/                         # 模型文件 (git-lfs 或 .gitignore)
│   └── zipformer/
│       ├── README.md
│       └── tokens.txt
│
├── scripts/
│   ├── build.sh                    # 一键构建
│   ├── install.sh                  # 安装到系统
│   └── download_model.sh          # 下载模型
│
└── packaging/
    ├── debian/                     # deb 打包
    ├── rpm/                        # rpm spec
    └── archlinux/                  # PKGBUILD
```

---

## 3. 核心接口设计

### 3.1 Rust FFI 导出接口

详见完整代码示例在 `vinput-core/src/ffi/exports.rs`:

**关键函数**:
- `vinput_core_init()` - 初始化引擎
- `vinput_core_shutdown()` - 关闭引擎
- `vinput_core_send_event()` - 发送事件 (C++ → Rust)
- `vinput_core_try_recv_command()` - 接收命令 (Rust → C++)
- `vinput_core_get_state()` - 查询当前状态
- `vinput_core_reload_config()` - 热重载配置
- `vinput_core_last_error()` - 获取最后错误描述

**设计原则**:
1. 所有函数用 `catch_unwind` 包裹, panic 不会传播到 C++ 侧
2. 所有指针参数检查 null
3. 字符串传递用 C 字符串 (以 \0 结尾)
4. 不暴露 Rust 内部类型, 只用 C-repr 结构体和 opaque pointer

### 3.2 FFI 类型定义

```rust
// vinput-core/src/ffi/types.rs

/// C-repr 事件类型 (C++ → Rust)
#[repr(C)]
pub enum VInputEventType {
    HotkeyPressed = 0,
    HotkeyReleased = 1,
    UserCancelled = 2,
    UndoRequested = 3,
    PullBackRequested = 4,
    RetryRequested = 5,
    FocusIn = 6,
    FocusOut = 7,
    AppChanged = 8,
}

/// C-repr 命令类型 (Rust → C++)
#[repr(C)]
pub enum VInputCommandType {
    SetPreedit = 0,
    CommitString = 1,
    ClearPreedit = 2,
    DeleteSurrounding = 3,
    ShowNotification = 4,
}

/// C-repr 状态
#[repr(C)]
pub enum VInputStateC {
    Idle = 0,
    WaitingForSpeech = 1,
    Streaming = 2,
    SilenceDetected = 3,
    Finalizing = 4,
    Error = 5,
}
```

### 3.3 状态机核心 Trait

```rust
// vinput-core/src/state_machine/state.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceInputState {
    Idle,
    WaitingForSpeech { start_time: std::time::Instant },
    Streaming { vad_start_time: std::time::Instant, token_count: usize },
    SilenceDetected { silence_start: std::time::Instant },
    Finalizing,
    Error { error_type: ErrorKind, can_retry: bool },
}

// vinput-core/src/state_machine/event.rs

#[derive(Debug, Clone)]
pub enum VoiceInputEvent {
    HotkeyPressed,
    HotkeyReleased,
    VadSpeechDetected,
    VadSilenceDetected { duration_ms: u32 },
    VadSegmentEnd,
    AsrTokenReceived { text: String, is_endpoint: bool },
    AsrFinalResult { text: String, confidence: f32 },
    PostProcessingDone { final_text: String },
    UserCancelled,
    UndoRequested,
    PullBackRequested,
    RetryRequested,
    Timeout,
    ErrorOccurred { kind: ErrorKind },
    FocusOut,
    AppChanged { app_id: String },
}

// vinput-core/src/state_machine/command.rs

#[derive(Debug, Clone)]
pub enum FcitxCommand {
    SetPreedit { text: String, cursor_pos: usize, format: TextFormat },
    CommitString { text: String },
    ClearPreedit,
    DeleteSurrounding { offset: i32, count: u32 },
    ShowNotification { message: String },
}
```

### 3.4 统一错误类型

```rust
// vinput-core/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VInputError {
    #[error("PipeWire error: {0}")]
    PipeWire(String),

    #[error("Audio device not found: {0}")]
    AudioDeviceNotFound(String),

    #[error("Ring buffer overrun: {lost_frames} frames lost")]
    RingBufferOverrun { lost_frames: u64 },

    #[error("Model load failed: {path} - {reason}")]
    ModelLoad { path: String, reason: String },

    #[error("ASR inference failed: {0}")]
    AsrInference(String),

    #[error("Recognizer not initialized")]
    RecognizerNotReady,

    #[error("Silero VAD model load failed: {0}")]
    VadModelLoad(String),

    #[error("Invalid state transition: {from:?} + {event} → ???")]
    InvalidTransition { from: String, event: String },

    #[error("Operation not allowed in state: {state:?}")]
    NotAllowedInState { state: String },

    #[error("Config parse error: {path} - {reason}")]
    ConfigParse { path: String, reason: String },

    #[error("Config file not found: {0}")]
    ConfigNotFound(String),

    #[error("Channel send error: receiver dropped")]
    ChannelSend,

    #[error("Channel recv error: sender dropped")]
    ChannelRecv,

    #[error("FFI null pointer: {param}")]
    NullPointer { param: String },

    #[error("Nothing to undo")]
    EmptyUndoHistory,

    #[error("Undo time window expired ({elapsed_ms}ms > {window_ms}ms)")]
    UndoTimeWindowExpired { elapsed_ms: u64, window_ms: u64 },
}

pub type VInputResult<T> = Result<T, VInputError>;
```

---

## 4. 并发模型设计

### 4.1 线程架构总图

```
┌─────────────────────────────────────────────────────────────────┐
│  PipeWire Graph Thread (RT, managed by PipeWire)                │
│  ┌────────────────────────────────────────────┐                │
│  │ process callback:                           │                │
│  │   memcpy audio → rtrb::Producer (SPSC)      │                │
│  │   atomic::fetch_add(overrun_counter) if full │                │
│  │   NO lock, NO alloc, NO log                  │                │
│  └────────────────────┬───────────────────────┘                │
│                       │ rtrb (lock-free SPSC)                   │
│                       ▼                                         │
│  VAD Thread (std::thread, normal priority)                      │
│  ┌────────────────────────────────────────────┐                │
│  │ loop:                                       │                │
│  │   read 10ms chunk from rtrb::Consumer       │                │
│  │   energy_gate.check(chunk)                  │                │
│  │   if passed: silero.infer(chunk)            │                │
│  │   hysteresis.update(prob)                   │                │
│  │   if speech_start: pre_roll.flush() → ASR   │                │
│  │   send VadEvent → event_tx (mpsc)           │                │
│  │   Also: copy audio → asr_audio_tx (rtrb)    │                │
│  └────────────────────┬───────────────────────┘                │
│                       │ mpsc (VadEvent) + rtrb (audio → ASR)    │
│                       ▼                                         │
│  ASR Thread (std::thread, normal priority)                      │
│  ┌────────────────────────────────────────────┐                │
│  │ loop:                                       │                │
│  │   read audio from rtrb::Consumer            │                │
│  │   recognizer.accept_waveform(samples)       │                │
│  │   if recognizer.is_ready():                 │                │
│  │     recognizer.decode()                     │                │
│  │     text = recognizer.get_result()          │                │
│  │     send AsrToken → event_tx (mpsc)         │                │
│  │   if finalize_requested:                    │                │
│  │     text = recognizer.finalize()            │                │
│  │     send AsrFinalResult → event_tx (mpsc)   │                │
│  └────────────────────┬───────────────────────┘                │
│                       │ mpsc (AsrEvent)                         │
│                       ▼                                         │
│  State Machine Thread (std::thread, normal priority)            │
│  ┌────────────────────────────────────────────┐                │
│  │ loop:                                       │                │
│  │   event = event_rx.recv_timeout(10ms)       │                │
│  │   (new_state, commands, effects) =          │                │
│  │       transition(state, event, ctx)         │                │
│  │   state = new_state                         │                │
│  │   for cmd in commands:                      │                │
│  │       command_tx.send(cmd)  // → Fcitx5     │                │
│  │   for effect in effects:                    │                │
│  │       dispatch_side_effect(effect)          │                │
│  │   check_timeouts()                          │                │
│  └────────────────────┬───────────────────────┘                │
│                       │ mpsc (FcitxCommand)                     │
│                       ▼                                         │
│  Fcitx5 Main Thread (Fcitx5 event loop, NOT ours)              │
│  ┌────────────────────────────────────────────┐                │
│  │ processTick(): // called by Fcitx5 每帧     │                │
│  │   while vinput_core_try_recv_command(&cmd): │                │
│  │     dispatch(cmd)                           │                │
│  │                                              │                │
│  │ keyEvent(event):                            │                │
│  │   if is_our_hotkey(event):                  │                │
│  │     vinput_core_send_event(HotkeyPressed)   │                │
│  └────────────────────────────────────────────┘                │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 线程间通信选型

| 通道 | 方向 | 类型 | crate | 理由 |
|------|------|------|-------|------|
| Audio -> VAD | PW RT -> normal | SPSC lock-free | `rtrb` | RT callback 不可阻塞, rtrb 是零分配 SPSC |
| VAD -> ASR (audio) | normal -> normal | SPSC lock-free | `rtrb` | 高吞吐音频流, 避免 mpsc 开销 |
| VAD -> SM (events) | normal -> normal | MPSC bounded | `std::sync::mpsc` 或 `crossbeam-channel` | 事件量低, 标准库足够 |
| ASR -> SM (events) | normal -> normal | MPSC bounded | 同上, 共用 event_tx | |
| SM -> Fcitx5 (cmds) | normal -> Fcitx5 main | MPSC bounded | `crossbeam-channel` | 非阻塞 try_recv |
| Fcitx5 -> SM (events) | Fcitx5 main -> normal | MPSC bounded | 共用 event_tx | |

---

## 5. 数据流设计

### 5.1 音频数据流

```
PipeWire daemon
  │ pw_stream_connect(PW_DIRECTION_INPUT, rate=16000, S16LE)
  ▼
process_callback(data: &[i16])  ← PipeWire RT graph thread
  │ rtrb::Producer::write_chunk(data)
  ▼
rtrb Ring Buffer (预分配 3s = 96KB)
  │ rtrb::Consumer::read_chunk(160)  // 10ms @ 16kHz
  ▼
VAD Thread
  ├── energy_gate: rms(chunk) > noise_floor * 2.5?
  ├── pre_roll_buffer.push(chunk)  // 环形, 350ms
  ├── silero_vad.infer(chunk) → speech_prob
  ├── hysteresis.update(prob)
  ├── [On Speech Start]: pre_roll_buffer.flush() → asr_audio_tx
  ├── [During Speech]: asr_audio_tx.write(chunk)
  └── [On Segment End]: event_tx.send(VadSegmentEnd)

asr_audio_tx (rtrb Ring Buffer, 10s = 320KB)
  │ rtrb::Consumer::read_chunk(...)
  ▼
ASR Thread
  ├── recognizer.accept_waveform(&samples)
  ├── while recognizer.is_ready(): decode_stream()
  ├── event_tx.send(AsrTokenReceived { text })
  └── [On finalize]: event_tx.send(AsrFinalResult { text })
```

### 5.2 事件流

```
事件源                      State Machine                     Fcitx5

VAD: VadSpeechDetected  ──► [WaitingForSpeech→Streaming]  ──► SetPreedit("")
ASR: AsrTokenReceived   ──► [Streaming→Streaming]          ──► SetPreedit("你好世")
VAD: VadSilenceDetected ──► [Streaming→SilenceDetected]    ──► TrayIcon→SilenceDetected
VAD: VadSegmentEnd      ──► [SilenceDetected→Finalizing]   ──► SetPreedit("处理中...")
PostProcessingDone      ──► [Finalizing→Idle]              ──► CommitString("你好世界。")
```

### 5.3 配置数据流

```
vinput-settings (Qt5/QML)
  ├── 用户修改配置 → write config.toml
  │                  │ inotify event
  │                  ▼
  │            VoiceInputEvent::ConfigReloaded
  │                  │
  │                  ▼
  │            SM Thread: reload config
  │
  ├── 用户编辑热词 → write hotwords.txt
  │                 │ inotify event
  │                 ▼
  │           if state == Idle:
  │               rebuild recognizer (~250ms)
  │           else:
  │               queue pending
  │
  └── D-Bus org.vinput.Settings
        ├── .ReloadConfig() → vinput_core_reload_config()
        ├── .GetState()     → vinput_core_get_state()
        └── .GetDevices()   → 列举 PipeWire 设备
```

---

## 6. 关键技术决策

### 6.1 Rust Crate 选型

| 领域 | Crate | 版本策略 | 理由 |
|------|-------|---------|------|
| **ASR** | `sherpa-onnx` (C API + bindgen) | 锁定版本 | 无 Rust crate, 需手动绑定 C API |
| **PipeWire** | `pipewire-rs` = "0.8" | 最新稳定 | 官方维护, API 完整 |
| **Lock-free SPSC** | `rtrb` = "0.3" | 最新 | 专为 RT audio 设计, 零分配 |
| **跨线程通道** | `crossbeam-channel` = "0.5" | 最新稳定 | bounded channel, try_recv 性能好 |
| **序列化** | `serde` + `toml` = "0.8" | 最新稳定 | TOML 配置标准方案 |
| **错误处理** | `thiserror` = "2" | 最新 | derive Error, 零开销 |
| **日志** | `tracing` = "0.1" | 最新稳定 | 编译时 feature gate, 零开销 |
| **文件监听** | `notify` = "7" | 最新稳定 | 跨平台 inotify 封装 |
| **正则** | `regex` = "1" | 最新稳定 | ITN context guard 用 |
| **C 绑定生成** | `cbindgen` (build dep) | 最新 | 自动生成 .h 头文件 |

### 6.2 sherpa-onnx 绑定策略

sherpa-onnx 没有官方 Rust crate。策略: **手动 bindgen 绑定其 C API**。

参见 `vinput-core/build.rs` 和 `vinput-core/src/asr/recognizer.rs` 完整实现。

### 6.3 日志方案

```toml
# vinput-core/Cargo.toml

[features]
default = []
debug-logs = ["tracing-subscriber", "tracing-journald"]

[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", optional = true }
tracing-journald = { version = "0.3", optional = true }
```

**生产模式**: 仅当 `VINPUT_LOG=1` 时启用 Error 级别到 journald
**调试模式** (`--features debug-logs`): 完整日志

### 6.4 配置管理

使用 `serde` + `toml` crate, 配置文件路径: `~/.config/vinput/config.toml`

详见 `vinput-core/src/config/schema.rs` 完整配置结构定义。

---

## 7. 性能关键路径优化

### 7.1 零分配音频 Callback

```rust
// 要求: 零分配, 无锁, 无 I/O, 无 panic

#[inline(always)]
pub fn write_samples(&mut self, samples: &[i16]) {
    if let Ok(mut chunk) = self.producer.write_chunk_uninit(samples.len()) {
        // rtrb::Producer::write_chunk_uninit 是零分配的
        // 逐样本复制 (编译器会向量化)
        // ...
        unsafe { chunk.commit_all() };
    } else {
        // Ring buffer 满: 只记录计数, 不阻塞
        self.overrun_counter.fetch_add(samples.len() as u64, Ordering::Relaxed);
    }
}
```

### 7.2 模型加载优化

```rust
// mmap + MADV_WILLNEED 预热
unsafe {
    let ptr = libc::mmap(...);
    libc::madvise(ptr, len, libc::MADV_WILLNEED);
    libc::munmap(ptr, len);
}
```

### 7.3 冷启动序列

```
T=0ms    进程启动
         ├── 立即: 创建 PipeWire Stream, 开始填充 Ring Buffer
         ├── 立即: 读取 config.toml (< 1ms)
         ├── 异步: spawn ASR 模型加载线程
         │         └── SherpaOnnxCreateOnlineRecognizer() ~400ms
         ├── 异步: spawn Silero VAD 加载 (~10ms)
         └── 同步: FFI handle 返回给 C++

T=500ms  ASR Recognizer 就绪 + 预热完成
T=500ms  首次热键按下可以正常工作
```

---

## 8. MVP 优先级

### Phase 0 (Week 1-2) -- 技术验证

**必须完成**:
- sherpa-onnx C API bindgen (绑定生成 + 编译通过)
- 一个独立的 Rust binary 能加载模型, 从 WAV 文件识别
- PipeWire 录音 + rtrb Ring Buffer (10分钟零丢帧)
- Silero VAD ONNX 推理 (概率输出正确)
- Fcitx5 C++ 插件骨架 (编译+加载成功, SetPreedit 硬编码文本)
- Rust cdylib FFI: init/shutdown/send_event/try_recv 四个函数可调通
- 端到端: 热键 → Preedit 显示硬编码文字 → 延时 Commit

**不做**:
- ITN, 标点, 撤销, 热词, GUI, 断句优化
- 多设备支持, Wayland 热键
- 配置文件, 错误处理 (直接 unwrap)

### Phase 1 (Week 3-6) -- 核心引擎必须模块

**必须完成**:
- state_machine/ (6状态, 所有转换, 单元测试)
- audio/pipewire_stream.rs (完整实现)
- vad/ (silero + energy_gate + hysteresis + pre_roll)
- asr/recognizer.rs (sherpa-onnx 封装, 流式输出)
- endpointing/pause_engine.rs (固定阈值断句, 800ms)
- itn/ (基础中文数字转换)
- punctuation/ (仅逗号, Professional 模式)
- ffi/ (完整 FFI 层)
- config/schema.rs (TOML 读取)
- error.rs (thiserror 完整错误类型)
- engine.rs (线程协调器)

**可延后**:
- itn/ 的 currency, date, percentage, unit, en_number
- punctuation/ 的 Balanced/Expressive 模式
- hotwords/ (Phase 3)
- undo/ (Phase 3)
- endpointing/ 的 CPS + RMS + semantic guard
- config/watcher.rs (热重载)

### 可延后到 Phase 3+ 的功能

**Phase 3**:
- undo/manager.rs + entry.rs (撤销/重试)
- hotwords/engine.rs + watcher.rs (热词 + inotify)
- Wayland 热键 (xdg-desktop-portal)
- GUI 完整 6 个标签页

**Phase 4**:
- endpointing/ CPS 自适应 + RMS 能量梯度 + 语义护卫
- punctuation/ Balanced + Expressive 模式
- itn/ 完整规则集
- 模型下载器 (HTTP Range + SHA256)
- 应用级热词 (contextual)

**v1.1+**:
- 历史记录自动热词
- 多候选结果
- WebSocket 服务端模式
- GPU 加速

---

## 9. 潜在风险与缓解

### 9.1 Fcitx5 FFI 崩溃隔离

**风险**: Rust 侧 panic 通过 FFI 传播到 C++, 导致 Fcitx5 整体崩溃。

**缓解**:
```rust
// 每个 extern "C" 函数都经过 ffi_boundary 包装
pub fn ffi_boundary<T, F>(f: F) -> Option<T>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error>>,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(val)) => Some(val),
        Ok(Err(e)) => {
            set_last_error(&e.to_string());
            None
        }
        Err(_panic_payload) => {
            set_last_error("FATAL: Rust panic caught at FFI boundary");
            None
        }
    }
}
```

### 9.2 PipeWire 实时性保证

**风险**: process callback 在 RT 图线程中执行, 任何阻塞操作会导致 xrun。

**缓解**:
- 严格禁止: Mutex, Box::new, println!, channel::send (可能阻塞)
- 只允许: rtrb::Producer (lock-free), AtomicU64 (原子操作), memcpy

**检测**: 使用 `PIPEWIRE_DEBUG=4` 检查 xrun 日志。

### 9.3 ONNX Runtime 线程安全

**风险**: ONNX Runtime Session 不是线程安全的。

**缓解**:
- OnlineRecognizer (ASR) 只在 ASR 线程内使用, 不跨线程
- Silero VAD Session 只在 VAD 线程内使用
- 实现 `Send` 但不实现 `Sync`

### 9.4 Wayland 热键兼容性

**风险**: Wayland 下无全局键盘抓取 API。

**缓解**:
- Phase 0-2: 仅支持 X11 / XWayland
- Phase 3: 通过 zbus 调用 xdg-desktop-portal GlobalShortcuts
- 运行时检测 $XDG_SESSION_TYPE

### 9.5 Fcitx5 主线程阻塞

**风险**: Fcitx5 是单线程事件循环, 在 FFI 调用中做阻塞操作会卡住整个输入法。

**缓解**:
- FFI 函数只做通道 send/recv, 不做任何 I/O 或计算
- `vinput_core_send_event()`: 非阻塞 mpsc send
- `vinput_core_try_recv_command()`: 非阻塞 try_recv
- `vinput_core_init()`: 模型加载在子线程异步进行, < 50ms 返回

### 9.6 内存泄漏风险

**风险**: FFI 传递的 CString 如果 C++ 侧忘记释放, 导致内存泄漏。

**缓解**:
```cpp
// RAII 封装
struct VInputString {
    char* ptr;
    VInputString(char* p) : ptr(p) {}
    ~VInputString() { vinput_core_free_string(ptr); }
    operator const char*() const { return ptr; }
    VInputString(const VInputString&) = delete;
};
```

---

## 10. C++ Fcitx5 插件核心实现

详见完整代码示例在 `fcitx5-vinput/src/vinput_engine.cpp`:

**关键方法**:
- `VInputEngine()` - 构造函数, 调用 vinput_core_init()
- `~VInputEngine()` - 析构函数, 调用 vinput_core_shutdown()
- `keyEvent()` - 热键拦截, 发送事件到 Rust
- `setupCommandPoller()` - 每 10ms 轮询命令队列
- `processCommands()` - 处理 Rust 返回的命令 (SetPreedit/CommitString)
- `activate()` - 输入法激活, 通知当前应用
- `deactivate()` - 输入法失焦

**RAII 内存管理**:
- 所有 Rust 返回的字符串用 RAII 包装, 自动释放

---

## 11. 实施建议

### 11.1 Phase 0 具体步骤 (2周)

**Week 1 -- Day 1-2: 环境搭建 + sherpa-onnx 绑定**
```bash
# 1. 安装依赖
sudo apt install libfcitx5core-dev fcitx5-modules-dev libpipewire-0.3-dev

# 2. 下载 sherpa-onnx 预编译库
wget https://github.com/k2-fsa/sherpa-onnx/releases/...

# 3. 创建 vinput-core skeleton
cargo init --lib vinput-core

# 4. 验证: Rust binary 加载模型, 识别 WAV 文件
cargo run --example offline_test
```

**Week 1 -- Day 3-4: PipeWire + rtrb**
```bash
# 1. pipewire-rs 最小录音示例
# 2. rtrb ring buffer 连接
# 3. 10分钟连续录音零丢帧测试
cargo run --example pipewire_capture
```

**Week 1 -- Day 5: Fcitx5 插件骨架**
```bash
# 1. CMake 配置
# 2. VInputEngine 骨架 (只实现 keyEvent 硬编码)
# 3. 编译 .so, 放入 ~/.local/lib/fcitx5/
# 4. fcitx5 --replace 验证插件加载
```

**Week 2 -- Day 1-3: FFI 连通**
```bash
# 1. cbindgen 生成 vinput_core.h
# 2. 实现 4 个核心 FFI 函数 (简化版, 无线程)
# 3. C++ 侧调用验证
# 4. 端到端: 热键 → Rust → Preedit → 3s → Commit
```

**Week 2 -- Day 4-5: Silero VAD + 基准测试**
```bash
# 1. Silero VAD ONNX 加载 + 推理 (独立测试)
# 2. 性能基准: 冷启动, 内存, 延迟, CPU
# 3. Go/No-Go 报告
```

### 11.2 关键开发约定

1. **每个 .rs 文件不超过 400 行**. 超过则拆分模块。
2. **状态转换逻辑是纯函数** (`transition.rs`), 不依赖外部状态, 100% 单元测试覆盖。
3. **FFI 层零业务逻辑**, 只做类型转换 + 通道 send/recv。
4. **所有阻塞操作在 SM 线程的 side_effect 中 dispatch**, 不在 Fcitx5 主线程执行。
5. **每个 commit 编译通过 + 测试通过**, 不接受 broken commit。

### 11.3 测试策略

```
vinput-core/tests/
├── state_machine_test.rs     # 所有 14 个状态转换 + 边界条件
├── itn_test.rs               # 20+ 转换规则 (TDD: 先写测试)
├── punctuation_test.rs       # 15+ 标点场景
├── vad_test.rs               # 双阈值/迟滞/短爆发过滤
├── undo_test.rs              # 撤销历史栈/时间窗口
├── ffi_test.rs               # FFI 函数 panic 安全性
└── integration/
    └── full_pipeline_test.rs # WAV → VAD → ASR → ITN → Punct
```

**测试数据**: 录制 20 句标准测试音频 (WAV, 16kHz, 单声道), 放在 `tests/fixtures/`。

---

## 12. ADR 记录

### ADR-001: 使用 rtrb 而非 crossbeam SPSC

**Context**: 音频 RT callback 需要零分配无锁队列。

**Decision**: 使用 `rtrb` crate。

**Positive**: 专为 RT audio 设计, write_chunk_uninit 零分配, 被 cpal 等音频 crate 广泛使用。

**Negative**: 只支持 SPSC, 但这正好符合我们的需求。

**Alternatives**: ringbuf (API 更复杂), crossbeam (无 uninit write), 手写 (维护成本)。

### ADR-002: 状态转换为纯函数

**Context**: 状态机逻辑是核心中的核心, 必须 100% 可测试。

**Decision**: `transition.rs` 中的 `handle_transition()` 是纯函数, 接收 (state, event, config), 返回 (new_state, commands, side_effects)。

**Positive**: 可以不启动任何线程/硬件, 纯粹测试所有状态转换。

**Negative**: 需要额外的 SideEffect 枚举和 dispatch 逻辑。

### ADR-003: 不使用 tokio

**Context**: 设计文档中部分伪代码使用了 tokio::spawn, 需要决定是否引入异步运行时。

**Decision**: 不使用 tokio。全部使用 std::thread + mpsc/crossbeam-channel。

**Positive**: 减少依赖, RT 音频线程与 async 不兼容, 状态机线程用 recv_timeout 即可。

**Negative**: ITN/标点后处理如果想异步需要手动 spawn 线程。但这些操作 < 1ms, 同步执行即可。

### ADR-004: Fcitx5 命令轮询间隔 10ms

**Context**: Fcitx5 主线程需要消费 Rust 产生的命令。

**Decision**: 使用 Fcitx5 EventLoop 的 addTimeEvent, 每 10ms 调用 try_recv。

**Positive**: 简单可靠, 10ms 延迟用户不可感知 (人类视觉反应 > 50ms)。

**Negative**: 空闲时也在轮询。但 try_recv 开销约 10ns, 100 次/秒 = 1us/秒, 可忽略。

**Alternative**: Fcitx5 addIOEvent 监听 eventfd。更复杂但零轮询。可作为后续优化。

---

## 总结

这份架构设计覆盖了从 FFI 边界到 RT 音频回调的所有关键路径。核心设计思想:

1. **Fcitx5 主线程绝不阻塞** -- FFI 只做通道 send/recv
2. **状态转换是纯函数** -- 100% 可测试, 行为可预测
3. **音频路径零分配** -- rtrb SPSC + 原子计数器处理溢出
4. **panic 不穿越 FFI** -- catch_unwind 包裹所有 extern "C"
5. **模块间只通过通道通信** -- 无共享可变状态, 无锁

**下一步**: 开始 Phase 0 技术验证, 创建项目骨架并验证关键技术路径可行性。

---

**文档版本历史**:
- v1.0 (2026-02-13): 初始架构设计完成
