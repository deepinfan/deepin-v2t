# Deepin-V-Input：Deepin Linux 下的离线中文语音输入法

### 需求与技术规格说明书 

## 1. 项目愿景 (Vision)

开发一个针对 Linux 生态的离线中文语音输入法（支持中英混合）。通过 Rust 的高性能和内存安全特性，采用 Zipformer Transducer 流式识别模型，实现零常驻资源占用、极速冷启动、以及与 Fcitx5 的完美集成。在不同硬件配置下，用较低的资源实现较为准确的商业语音输入法质量和体验。

---

## 2. 核心技术架构 (System Architecture)

### 2.1 技术栈 (Technical Stack)

* **开发语言**: **Rust** (核心引擎) + **C++/Qt5 (QML)** (GUI)。
* **ASR 引擎**: [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (基于 ONNX Runtime)。
* **流式模型**: sherpa-onnx-streaming-zipformer-zh-int8-2025-06-30。
* **VAD**: Silero VAD (ONNX)。
* **ITN**: cn2an-rs+规则层
* **标点处理**： Zipformer 自带标点输出+规则层

* **音频采集**: PipeWire Stream API
* **输入法框架**: Fcitx5 (直接采用其C接口)。

---

## 3. 核心功能需求 (Functional Requirements)

1. **并行启动与加载 (Cold Start Optimization)**:
* 进程启动瞬间**立即**开启麦克风录音，数据存入异步 Ring Buffer。
* 同步启动 `onnxruntime` 进行懒加载。
* 模型预热（Warm-up）机制。在 Fcitx5 插件加载时，异步进行一次“空推理”（输入静音数据），确保用户按下热键开始说话时，推理链条已经完全处于就绪状态
* 默认使用cpu进行处理


2. **撤销/重试机制 (Undo & Retry)**
* **Ctrl+Z 撤销最近提交**:
  - 撤销最近一次 CommitString，文本回退到 Preedit 区域
  - 允许用户重新编辑或重新进行语音输入
  - Preedit 状态保持，用户可直接修改或删除
* **长按热键重新识别**:
  - 长按语音输入热键（如 Ctrl+Shift+V）2 秒触发重录模式
  - 保留原始音频缓冲，重新进行 ASR 推理
  - 显示 "重新识别中..." 提示
* **时间窗口拉回机制**:
  - Commit 后 3 秒内，Ctrl+Shift+Z 可拉回 Preedit 区域
  - 适用于用户刚提交后立即发现错误的场景
  - 超过 3 秒后该 Commit 无法撤销（防止误操作）
* **撤销历史栈管理**:
  - 保留最近 5 次 Commit 的撤销记录
  - 使用环形缓冲区实现，避免内存泄漏
  - 每条记录包含: (Preedit 文本, 原始音频缓冲, 时间戳, 光标位置)
* **实现策略**:
  - 在 Commit 前克隆 Preedit 状态和音频数据
  - 音频缓冲最多保存 5×10s = 50s (约 1.5MB @ 16kHz)
  - Ctrl+Z 触发时：调用 Fcitx5 DeleteSurrounding + SetPreedit 恢复状态
  - 重录触发时：调用 Recognizer::Reset + 送入历史音频重新识别


3. **ITN (反向文本正则化)**
* 自动识别口语数字并转换：例如“二零二六点二” -> “2026.2”。
* 支持日期、百分比及常用金融数字格式转换。
* 参见 V-Input ITN 系统设计说明书.txt

4. **智能标点**:
* 基于 Zipformer 自带标点输出 + 规则层后处理
* 参见 V-Input 标点控制系统设计说明书.txt

5. **热词和上下文增强 (Contextual Hotwords)**
* 使用 Zipformer Transducer 原生热词增强（modified_beam_search）
* 支持从 `hotwords.txt` 动态加载。用户自定义的人名、术语、公司名拥有更高的识别权重。
* 热词更新只在“空闲状态”重建 recognizer，Streaming 进行时禁止修改

6. **fcitx5集成**
* vinput-core (Rust): 编译为 .so 动态库（cdylib）。封装 sherpa-onnx 推理引擎。
* fcitx5-vinput (C++/Native): Fcitx5 的原生插件，通过 FFI 直接调用 Rust 核心，不经过任何进程间通信。
* Fcitx5 是单线程主循环,不要在 FFI 调用中做模型推理阻塞主线程,采用ASR 线程,VAD 线程,主线程只负责接收最终文本

---

## 4. GUI 设置界面需求 (Settings UI)

### 4.1 技术实现

* **框架**: Qt5/QML (轻量化设计，独立进程运行)。
* **运行**: 独立于 ASR 后端的轻量二进制工具。

### 4.2 界面功能

* **输入设备选择**：可以选择使用哪个输入设备，并可进行测试
* **音频监控**: 实时显示麦克风音量波动条。
* **VAD 滑块**: 调节断句灵敏度（200ms - 1500ms）。
* **热键配置**：用户可配置启动和停止语音输入的热键
* **模型管理**：可以下载、删除、升级相关模型
* **热词编辑器**: 多行文本框，支持实时保存并重载配置。

---

## 5. 交互规范 (UX Specification)

### 5.1 视觉反馈层级

* **Preedit 文本状态**:
  - **录音检测中**: Fcitx5 SetPreedit 渲染灰色带下划线的预览文字
  - **识别处理中**: 蓝色虚线下划线 + 光标脉动动画（使用 TextFormatFlag::Underline | TextFormatFlag::HighLight）
  - **VAD 断句触发**: Preedit 文本短暂闪烁 300ms（模拟"确认"效果）
  - **Commit 完成**: 一次性调用 CommitString 替换 Preedit 区域，文本由目标应用正常渲染

* **系统托盘图标状态**（5 种状态）:
  - **空闲**: 灰色麦克风图标（#808080）
  - **热键按下/录音中**: 红色麦克风图标（#FF0000）+ 脉动动画（0.8s 周期）
  - **静音检测**: 橙色麦克风图标（#FFA500）
  - **识别处理中**: 蓝色麦克风图标（#0080FF）+ 顺时针旋转动画（1.2s 周期）
  - **错误状态**: 红色麦克风 + 警告标志叠加

* **状态转换时序**:
  ```
  空闲 → [热键按下] → 录音中 → [VAD检测到语音] → 识别中
       → [VAD静音800ms] → 静音检测 → [Commit] → 空闲
  ```

### 5.2 文本提交策略

* **原子替换**: VAD 段结束后，一次性调用 CommitString 替换整个 Preedit 区域
  - 避免逐字提交导致的闪烁和应用兼容性问题
  - 确保撤销功能可以回退整个句子

* **分段提交优化**（可选）:
  - 若 Zipformer 输出超过 100 个字符，自动在逗号、句号处分段提交
  - 每段独立 Commit，降低单次撤销的粒度
  - 通过 GUI 设置 "自动分段长度" 控制（默认关闭）

### 5.3 光标与焦点控制

* **光标定位**: 语音输入期间，光标强制锁定在 Preedit 文字句尾
  - 使用 SetPreedit(text, cursor_pos = text.len())
  - 防止用户在流式输出中途移动光标导致插入位置混乱

* **自动 Finalize 触发条件**:
  - 若用户手动移动光标（检测到 KeyEvent::CursorMove）→ 立即 Commit 当前 Preedit + 退出语音模式
  - 若用户敲键盘（检测到非热键的 KeyEvent::KeyPress）→ 立即 Commit + 退出
  - 若用户切换应用窗口（FocusOut 事件）→ 自动 Commit + 保留语音模式（可配置）

* **焦点保护**:
  - 语音输入过程中锁定输入焦点，防止其他应用抢占
  - 使用 Fcitx5 的 FocusGroup 机制确保焦点稳定性

### 5.4 错误场景处理

* **识别失败**:
  - **触发条件**: VAD 超时（无语音检测 > 5s）或 ASR 返回空结果
  - **视觉反馈**: Preedit 显示 `[识别失败，请重试]`（灰色斜体）
  - **持续时间**: 2 秒后自动清除 Preedit
  - **系统托盘**: 图标短暂变为红色 + 警告标志 500ms

* **权限不足**:
  - **触发条件**: PipeWire 音频采集失败（Permission Denied）
  - **处理流程**:
    1. 弹出 Qt 对话框: "V-Input 需要麦克风权限"
    2. 提供 "一键修复" 按钮（调用 `pkexec vinput-fix-permissions`）
    3. 显示手动修复指南链接（指向文档）
  - **日志记录**: 写入 journald，便于用户排查

* **模型加载失败**:
  - **触发条件**: ONNX 模型文件损坏或不兼容
  - **处理流程**:
    1. 显示错误对话框: "模型加载失败: {error_message}"
    2. 提供 "重新下载模型" 按钮（跳转到 GUI 模型管理页）
    3. 提供 "查看日志" 按钮（打开日志文件或 journalctl -u vinput）
  - **降级策略**: 尝试加载 FP32 模型（如果 INT8 失败）

* **音频设备断开**:
  - **触发条件**: PipeWire Stream 断开事件（设备拔出）
  - **处理流程**:
    1. 自动切换到系统默认音频输入设备
    2. 系统通知: "麦克风已断开，已切换至默认设备"
    3. GUI 设置界面实时更新设备列表
  - **无缝恢复**: 若原设备重新连接，询问用户是否切换回去

* **热键冲突**:
  - **检测机制**: 启动时检测系统已注册的全局热键
  - **冲突提示**: "热键 Ctrl+Shift+V 已被占用，请重新配置"
  - **智能推荐**: 根据当前占用情况推荐备选热键（Ctrl+Alt+V / Super+V 等）
  - **Wayland 特殊处理**: 提示用户通过 xdg-desktop-portal 配置

---

## 6. 性能优化项

### 6.1实时无锁数据流
* 废弃所有带锁的队列
* 引入 ringbuf crate：建立 SPSC（单生产者单消费者）模型。
* 预分配策略：在加载模型前，先预分配 3.0 秒容量的环形缓冲区，避免在录音过程中产生任何内存分配（Allocation-free path）。
* 实时性保证：录音回调函数中不得包含任何 println!、锁或复杂的逻辑，确保其满足实时性要求（Hard RT-safe）。
* 异常处理：如果缓冲区溢出（Overrun），通过原子计数器记录丢帧次数，但绝不要阻塞录音线程。"


### 6.2零开销日志系统 (Zero-Overhead Logging)

* 编译时门控 (Compile-time Gating)：使用 Rust 的 feature flags。除非在编译时显式指定 --features debug-logs，否则所有的调试代码（Debug/Trace）在编译阶段就会被彻底移除，生成的二进制文件中完全不包含这些字符串和逻辑。
* 运行时静默 (Runtime Silence):禁止 Stdout Spam：生产环境下，默认关闭所有标准输出（stdout / stderr）。
* 按需开启：仅在检测到特定的环境变量（如 VINPUT_LOG=1）时，才将核心错误日志定向至系统日志（如 journald）或指定的临时文件。
* 零性能损耗 (Performance Neutrality)使用 log 库或 tracing 库的宏。在 Release 模式下，非 Error 级别的日志在检查级别阶段就应立即返回，不进行字符串格式化。4
* 不要使用 env_logger 直接使用 tracing + subscriber feature gating


### 6.3 自适应标点与断句判定系统 (Adaptive Punctuation & Endpointing)
功能定义： 实现一套由声学特征、语速反馈和语义上下文驱动的动态断句逻辑，取代传统的硬编码（Fixed Timeout）静音检测，以解决慢语速误断句与快语速响应迟钝的痛点。
动态语速补偿 (Dynamic CPS Adaptation)：
* 系统需实时追踪当前句子的字符产出速度（Characters Per Second）。
* 根据 CPS 动态缩放静音判定阈值：高语速状态下收紧阈值（最低至 200ms）以提升出字爆发力；低语速或思考状态下自动放宽阈值（最高至 1500ms），保障长句完整性。不要直接线性缩放，建议用 sigmoid 曲线：
声学能量梯度分析 (RMS Energy Gradient)：
* 在 VAD 判定为静音前的最后 100ms 窗口内，监控音频能量（RMS）的衰减斜率。100ms 窗口与前 300ms 均值做对比
* 陡峭衰减判定为物理性闭口，立即触发断句；平缓拖尾判定为语气未完或犹豫，自动延长等待时长。
语义后缀护卫 (Semantic Guarding)：
* 对 ASR 实时输出的末尾 Token 进行轻量级正则扫描。
* 拦截逻辑：若结尾为助词（如“的、地、得”）或连词（如“因为、但是”），强制锁定断句触发，直至语义进一步明确。
* 自动提问：识别到结尾疑问词（如“吗、呢、吧”）时，结合声学语调暗示，将句号自动转换为问号。

Zipformer 实时输出 + VAD 动态断句 + 规则优化

---


## 7. 技术实现规格 (Technical Specs)

### 7.1 性能目标

* **冷启动速度**:
  - **SSD**: < 600ms 达到录音就绪（目标值，实测 400-800ms）
  - **HDD**: < 1.5s 达到录音就绪（机械硬盘随机读取限制）
  - **优化措施**:
    - 模型 mmap（内存映射）+ 离线图优化（预序列化 .ort）
    - 音频预缓冲（启动时立即开始录音，模型异步加载）
    - 懒加载策略（GUI 设置界面与核心引擎分离）

* **内存占用**:
  - **闲置**: 0 MB (进程退出，零常驻)
  - **工作峰值**: < 250MB (含模型 mmap 实际 RSS)
    - Zipformer INT8 模型: ~160MB (encoder 154MB + decoder 4.9MB + joiner 1MB)
    - Silero VAD: ~2.24MB
    - ONNX Runtime Session 开销: ~30-50MB
    - Ring Buffer (3s 音频): ~100KB (16kHz × 2 bytes × 3s = 96KB)
    - ITN/热词/标点引擎: <5MB
    - 撤销历史栈（5 次 × 10s）: ~1.5MB
  - **内存管理策略**:
    - mmap 模型文件，操作系统按需加载页面（实际 RSS 低于模型文件大小）
    - 使用 `madvise(MADV_WILLNEED)` 预热常用模型页
    - 空闲 60s 后自动卸载模型并退出进程

* **端到端延迟**:
  - **流式输出**: < 100ms（从 VAD 检测到语音 → 首个 token 出现在 Preedit）
  - **Commit 延迟**: < 50ms（从 VAD 断句触发 → CommitString 完成）
  - **总延迟**: < 150ms（用户感知延迟）

* **识别准确率**（基于 Zipformer 模型）:
  - **中文 CER**: 5-8%（AISHELL-1 测试集，无热词）
  - **热词增强后**: CER 降低 1-2%（领域术语场景）
  - **标点准确率**: F1 > 85%（基于规则层 + 模型自带标点）

### 7.2 资源限制

* **ONNX Runtime 线程控制**:
  - `intra_op_num_threads = 2`（CPU 核心数的 1/4，避免与系统争抢）
  - `inter_op_num_threads = 1`（单会话推理，无需并行）
  - 禁止 ONNX Runtime 自动线程扩展

* **实时优先级**:
  - **不在应用层设置 SCHED_FIFO**（避免权限问题和跨发行版兼容性问题）
  - 依赖 PipeWire Stream API 的 process 回调（已在 RT 图线程中运行）
  - PipeWire 通过 RTKit/RLIMIT_RTPRIO 自动获取实时优先级

### 7.3 授权合规

* **授权合规**: 全线采用 Apache-2.0 / MIT / BSD 协议
  - Rust 核心: Apache-2.0 或 MIT（双授权）
  - Fcitx5 插件: LGPL-2.1+（动态链接 Fcitx5，符合 LGPL 要求）
  - sherpa-onnx: Apache-2.0
  - ONNX Runtime: MIT
  - Qt5: LGPL-3.0（动态链接）
  - 模型文件: Apache-2.0（Zipformer）+ MIT（Silero VAD）

* **许可证兼容性**:
  - Rust 核心（Apache-2.0/MIT）→ 编译为 .so（cdylib）
  - C++ 插件（LGPL-2.1+）→ 动态链接 Fcitx5 和 Rust 核心 .so
  - 动态链接不传染 LGPL 到 Apache-2.0 核心（业界共识）

