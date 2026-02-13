# V-Input 分阶段实施计划

**项目周期**: 约 14-18 周 (3.5-4.5 个月)
**团队规模**: 1-2 人全职开发

---

## 📅 总体时间线

```
Phase 0: 技术验证       [████████] 2 周         ⚠️ 关键决策点
Phase 1: 核心引擎       [████████████████] 4 周
Phase 2: MVP 集成       [████████████] 3 周      👤 第一次验证
Phase 3: 功能完善       [████████████████] 4 周  👤 第二次验证
Phase 4: 优化打磨       [████████████] 3 周      👤 第三次验证
Phase 5: 发布准备       [████████] 2 周          👤 最终验证
```

---

## 🔬 Phase 0: 技术验证 (2 周) ⚠️ **关键决策点**

### 目标
验证所有核心技术可行性，消除阻塞性风险，确认技术选型正确。

### 工作内容

#### Week 1: 模型集成验证

**1.1 Zipformer 流式识别测试**
```bash
# 任务清单
[ ] 下载 sherpa-onnx-streaming-zipformer-zh-int8-2025-06-30 模型
[ ] 编写 sherpa-onnx Rust 绑定测试代码
[ ] 测试逐帧流式输出延迟
[ ] 验证热词增强功能 (hotwords.txt)
[ ] 测试模型自带标点输出
[ ] 记录性能指标 (RTF, 内存占用, 冷启动时间)
```

**输出**:
- ✅ Zipformer 流式输出延迟 < 100ms
- ✅ 热词功能正常工作
- ✅ 标点输出验证报告 (模型是否自带标点)

**1.2 Fcitx5 FFI 原型**
```bash
# 任务清单
[ ] 编写 C++ wrapper 暴露 Fcitx5 API
    - SetPreedit(text, cursor_pos, format)
    - CommitString(text)
    - DeleteSurrounding(offset, nchars)
[ ] Rust 通过 FFI 调用测试
[ ] 验证 catch_unwind 崩溃隔离
[ ] 多线程安全测试 (ASR线程 → Fcitx5主线程)
```

**输出**:
- ✅ fcitx5-vinput-ffi 原型代码 (可编译运行)
- ✅ FFI 调用成功率报告

---

#### Week 2: 音频管线验证

**2.1 PipeWire 录音集成**
```bash
# 任务清单
[ ] 使用 pipewire-rs crate 创建 Stream
[ ] 实现 process 回调 (16kHz, 单声道)
[ ] 集成 rtrb 无锁 Ring Buffer (SPSC)
[ ] 测试零丢帧 (连续录音 10 分钟)
[ ] 验证 RT 优先级自动获取
[ ] Pre-roll buffer 实现 (350ms)
```

**输出**:
- ✅ 音频录制稳定性报告 (零丢帧验证)
- ✅ PipeWire 集成示例代码

**2.2 性能基准测试**
```bash
# 任务清单
[ ] 冷启动时间测试 (SSD/HDD)
    - 模型 mmap 加载时间
    - 音频管线启动时间
[ ] 内存占用分析 (RSS/VSZ)
[ ] 端到端延迟测试
    - VAD 检测 → 首个 token 输出
    - VAD 断句 → CommitString
[ ] CPU 占用测试 (空闲/识别中)
```

**输出**:
- ✅ 性能基准报告 (Markdown)
  - 冷启动: SSD实测 XXX ms
  - 内存: 峰值 XXX MB
  - 延迟: 端到端 XXX ms
  - CPU: 空闲 X%, 识别中 X%

---

### ⚠️ Go / No-Go 决策点

**决策会议** (Week 2 结束)

**Go 条件** (必须全部满足):
- ✅ Zipformer 流式输出延迟 < 150ms
- ✅ Fcitx5 FFI 调用成功率 > 99%
- ✅ PipeWire 录音零丢帧
- ✅ 冷启动时间 SSD < 1s (可接受)

**若 No-Go**: 重新评估技术选型或调整架构

---

### 交付物
- ✅ Phase 0 技术验证报告.md
- ✅ 性能基准测试数据
- ✅ 原型代码 (可编译运行)
- ⚠️ Go/No-Go 决策文档

---

## 🛠️ Phase 1: 核心引擎开发 (4 周)

### 目标
实现 Rust 核心引擎 (vinput-core)，包含 ASR、VAD、ITN、标点等核心模块。

### 工作内容

#### Week 3-4: ASR 引擎与状态机

**3.1 sherpa-onnx 集成层**
```rust
// vinput-core/src/asr/
[ ] recognizer.rs        // OnlineRecognizer 封装
[ ] config.rs            // 模型配置管理
[ ] token_stream.rs      // Token 流式输出
[ ] hotwords.rs          // 热词加载和管理
```

**功能**:
- 模型加载 (mmap + 离线图优化)
- 流式识别 (逐帧 token 输出)
- 热词增强 (hotwords.txt 加载)
- 最终结果获取 (Finalize)

**3.2 流式状态机**
```rust
// vinput-core/src/state_machine/
[ ] state.rs             // 状态定义 (6 种状态)
[ ] event.rs             // 事件定义 (10 种事件)
[ ] transition.rs        // 状态转换逻辑
[ ] engine.rs            // 状态机引擎 (事件循环)
```

**功能**:
- Idle → WaitingForSpeech → Streaming → Finalizing
- 错误处理和超时机制
- 消息传递 (mpsc 通道)

**单元测试**:
```bash
[ ] 测试所有状态转换 (14 个转换)
[ ] 测试超时处理
[ ] 测试错误恢复
```

---

#### Week 5: VAD 与断句

**5.1 Silero VAD 集成**
```rust
// vinput-core/src/vad/
[ ] silero.rs            // Silero VAD ONNX 推理
[ ] detector.rs          // 双输入模式 (PushToTalk/AutoDetect)
[ ] hysteresis.rs        // 迟滞控制 (双阈值)
[ ] pre_roll.rs          // Pre-roll Buffer (350ms)
```

**功能**:
- 语音/静音检测 (speech_prob)
- 动态阈值 (start 0.68, end 0.35)
- 短爆发过滤 (<100ms 忽略)
- Pre-roll buffer 防吞首字

**5.2 断句判定**
```rust
// vinput-core/src/endpointing/
[ ] pause_engine.rs      // 动态停顿判定
[ ] cps_tracker.rs       // 字符产出速度跟踪
[ ] rms_analyzer.rs      // RMS 能量梯度分析
```

**功能** (Phase 1 仅基础版):
- 固定阈值断句 (800ms 默认)
- 基本助词拦截 ("的地得" 延长 300ms)
- CPS 联动断句 (3 档线性，Phase 2 完善)

---

#### Week 6: ITN 与标点

**6.1 ITN 引擎**
```rust
// vinput-core/src/itn/
[ ] cn2an.rs             // 中文数字转换 (使用 cn2an-rs)
[ ] en_number.rs         // 英文数字解析
[ ] currency.rs          // 金额转换
[ ] date.rs              // 日期转换
[ ] guard.rs             // ColloquialGuard (防止误转)
```

**功能**:
- "二零二六点二" → "2026.2"
- "三百块钱" → "¥300"
- "二零二六年三月" → "2026年3月"
- 口语数量表达保护

**6.2 标点引擎**
```rust
// vinput-core/src/punctuation/
[ ] streaming.rs         // Streaming 阶段逗号逻辑
[ ] rules.rs             // 规则增强层 (逻辑连词、问号)
[ ] profile.rs           // 风格配置 (Professional 默认)
```

**功能** (Phase 1 仅 Professional 模式):
- Streaming 阶段插入逗号 (停顿比例 3.5)
- 逻辑连词前插入逗号
- 问号严格模式 (需声学特征)

---

### 单元测试覆盖

```bash
# 测试目标: 80% 覆盖率
[ ] ASR 引擎测试 (10 个用例)
[ ] 状态机测试 (14 个转换 + 5 个边界)
[ ] VAD 测试 (12 个场景)
[ ] ITN 测试 (20 个转换规则)
[ ] 标点测试 (15 个场景)
```

---

### 交付物
- ✅ vinput-core crate (可编译，单元测试通过)
- ✅ 单元测试报告 (覆盖率 > 80%)
- ✅ API 文档 (cargo doc)
- ⚠️ 核心引擎功能完整，但未集成 Fcitx5

---

## 🔗 Phase 2: MVP 集成 (3 周) 👤 **第一次人工验证**

### 目标
将 Rust 核心引擎集成到 Fcitx5，实现基本的语音输入功能。

### 工作内容

#### Week 7: Fcitx5 插件开发

**7.1 C++ 插件框架**
```cpp
// fcitx5-vinput/
[ ] vinput_engine.h/cpp       // InputMethodEngine 实现
[ ] vinput_ffi.h/cpp          // Rust FFI C 接口封装
[ ] CMakeLists.txt            // 构建配置
```

**功能**:
- 实现 InputMethodEngine 接口
- 热键事件处理 (keyEvent)
- SetPreedit / CommitString 调用
- 多线程消息传递 (Fcitx5主线程 ↔ Rust状态机线程)

**7.2 FFI 绑定层**
```rust
// vinput-core/src/ffi/
[ ] exports.rs           // extern "C" 导出函数
[ ] fcitx_command.rs     // Fcitx5 命令类型
[ ] safety.rs            // catch_unwind 安全包装
```

**功能**:
- vinput_core_init()
- vinput_core_send_event()
- vinput_core_try_recv_command()
- 崩溃隔离 (所有 FFI 函数用 catch_unwind 包裹)

---

#### Week 8: 音频采集集成

**8.1 PipeWire 录音线程**
```rust
// vinput-core/src/audio/
[ ] pipewire_stream.rs   // PipeWire Stream API
[ ] ring_buffer.rs       // rtrb 无锁环形缓冲
[ ] device_manager.rs    // 音频设备枚举和选择
```

**功能**:
- 热键按下 → 开始录音
- 音频数据写入 Ring Buffer
- VAD 线程消费音频数据
- 设备断开自动切换

**8.2 线程协调**
```rust
// vinput-core/src/threads/
[ ] coordinator.rs       // 线程协调器
[ ] audio_thread.rs      // 音频录制线程
[ ] vad_thread.rs        // VAD 检测线程
[ ] asr_thread.rs        // ASR 推理线程
```

**架构**:
```
Audio Thread → Ring Buffer → VAD Thread → Event → State Machine
                                ↓
                            ASR Thread → Token → State Machine
                                                    ↓
                                                Fcitx5 Command
```

---

#### Week 9: 基础 GUI + 打包

**9.1 最小化设置界面**
```qml
// vinput-settings/qml/
[ ] main.qml             // 主窗口
[ ] BasicSettings.qml    // 基本设置页 (设备选择、热键)
[ ] About.qml            // 关于页
```

**功能** (仅 2 个标签页):
- 音频设备选择 + 测试按钮
- 热键配置
- VAD 断句滑块 (200-1500ms)
- 关于信息

**9.2 打包脚本**
```bash
[ ] deb 打包 (Ubuntu/Deepin)
[ ] rpm 打包 (Fedora)
[ ] PKGBUILD (Arch AUR)
```

---

### 👤 **第一次人工验证** (Week 9 结束)

#### 验证内容

**基础功能测试** (必须全部通过):
```
[ ] 按热键 → 开始录音 → 托盘图标变红
[ ] 说话 → Preedit 实时更新 (灰色下划线)
[ ] 静音 800ms → CommitString 提交文本
[ ] 识别失败显示 "[识别失败，请重试]"
[ ] 权限不足显示错误提示对话框
```

**准确率测试** (抽样 20 句):
```
[ ] 纯中文句子识别准确率 > 90%
[ ] 中英混合句子识别准确率 > 85%
[ ] 数字转换正确率 > 95%
[ ] 标点插入合理性 (主观评估)
```

**性能测试**:
```
[ ] 冷启动时间 < 1s (SSD)
[ ] 内存占用 < 300MB
[ ] 端到端延迟 < 200ms
[ ] 连续使用 30 分钟无崩溃
```

**用户体验测试**:
```
[ ] Preedit 文字可见且不闪烁
[ ] CommitString 后文字正常显示
[ ] 热键响应及时 (< 100ms)
[ ] 系统托盘图标状态正确
```

#### 验证方式
- **测试人员**: 1-2 人
- **测试时间**: 2-3 天
- **测试环境**: Deepin 23, Ubuntu 24.04, Arch Linux
- **测试应用**: LibreOffice Writer, VS Code, Firefox

#### 验证输出
- ✅ 测试报告 (通过/失败项清单)
- ✅ Bug 列表 (优先级 P0-P2)
- ✅ 用户体验反馈

#### Go / No-Go 决策
**Go 条件**:
- P0 Bug = 0
- 基础功能测试通过率 > 95%
- 准确率达到目标
- 性能指标达标

**若 No-Go**: 修复 P0 Bug 后重新验证

---

### 交付物
- ✅ fcitx5-vinput 插件 (.so 文件)
- ✅ vinput-settings GUI (可运行)
- ✅ deb/rpm/PKGBUILD 打包脚本
- ✅ MVP 测试报告
- ⚠️ Bug 修复清单 (若有)

---

## ✨ Phase 3: 功能完善 (4 周) 👤 **第二次人工验证**

### 目标
实现所有 P1 功能，达到商业产品的基本体验。

### 工作内容

#### Week 10: 撤销/重试机制

**10.1 撤销管理器**
```rust
// vinput-core/src/undo/
[ ] manager.rs           // UndoManager (历史栈)
[ ] entry.rs             // UndoEntry (Preedit + 音频缓冲)
[ ] commands.rs          // 撤销命令 (Undo/Redo/Retry)
```

**功能**:
- Ctrl+Z 撤销最近 Commit
  - DeleteSurrounding + SetPreedit
- 长按热键 2s 重新识别
  - 保留音频缓冲重新推理
- Ctrl+Shift+Z 时间窗口拉回 (3s)
- 历史栈管理 (5 条记录，< 2MB)

**10.2 Fcitx5 集成**
```cpp
// fcitx5-vinput/
[ ] 监听 Ctrl+Z 全局热键
[ ] 实现 DeleteSurrounding 调用
[ ] 撤销状态同步
```

**单元测试**:
```bash
[ ] 撤销功能测试 (10 个场景)
[ ] 时间窗口测试
[ ] 重新识别测试
[ ] 边界条件测试 (历史栈满、应用不支持 DeleteSurrounding)
```

---

#### Week 11: 热词引擎

**11.1 热词管理**
```rust
// vinput-core/src/hotwords/
[ ] engine.rs            // HotwordsEngine
[ ] loader.rs            // hotwords.txt 加载
[ ] watcher.rs           // inotify 文件监听
[ ] contextual.rs        // 上下文感知 (应用级)
```

**功能**:
- hotwords.txt 加载和解析
- 文件变化自动重载 (inotify)
- sherpa-onnx hotwords_file 配置
- 应用级热词切换 (可选，基础版)

**11.2 GUI 热词编辑器**
```qml
// vinput-settings/qml/
[ ] HotwordsPage.qml     // 热词管理页
[ ] HotwordEditor.qml    // 添加/编辑对话框
```

**功能**:
- 热词列表 (TableView)
- 添加/删除/编辑热词
- 权重滑块 (1.0-5.0)
- 导入/导出 .txt 文件
- 实时保存到 hotwords.txt

---

#### Week 12: Wayland 支持

**12.1 热键双路径**
```rust
// vinput-core/src/hotkey/
[ ] x11.rs               // XGrabKey
[ ] wayland.rs           // xdg-desktop-portal GlobalShortcuts
[ ] detector.rs          // 运行时检测 (X11/Wayland)
```

**功能**:
- X11: XGrabKey (已有)
- Wayland: 通过 zbus 调用 xdg-desktop-portal
- 运行时检测 $XDG_SESSION_TYPE
- 统一的热键事件接口

**12.2 Wayland 兼容性测试**
```bash
[ ] KDE Plasma Wayland 测试
[ ] Hyprland 测试
[ ] Sway 测试 (可选)
```

---

#### Week 13: GUI 完善

**13.1 完整设置界面** (6 个标签页)
```qml
[ ] BasicSettings.qml    // 基本设置 (已有，优化)
[ ] RecognitionSettings.qml  // 识别设置 (VAD、ITN、标点、撤销)
[ ] HotwordsPage.qml     // 热词管理 (已有)
[ ] ModelManager.qml     // 模型管理 (下载、删除、升级)
[ ] AdvancedSettings.qml // 高级设置 (日志、调试、实验性功能)
[ ] AboutPage.qml        // 关于 (已有，优化)
```

**13.2 模型管理器**
```rust
// vinput-settings/src/model_manager/
[ ] downloader.rs        // HTTP 断点续传
[ ] verifier.rs          // SHA256 校验
[ ] installer.rs         // 模型安装/卸载
```

**功能**:
- 模型列表展示
- 下载进度条 (速度、剩余时间)
- SHA256 校验
- 断点续传 (HTTP Range)
- 自动重试 (指数退避)

---

### 👤 **第二次人工验证** (Week 13 结束)

#### 验证内容

**新功能测试**:
```
[ ] Ctrl+Z 撤销功能正常 (10 个场景)
[ ] 长按热键重新识别 (5 个场景)
[ ] 热词增强效果验证 (准确率提升 1-2%)
[ ] Wayland 热键正常工作 (KDE Plasma)
[ ] GUI 所有页面可正常操作
[ ] 模型下载、校验、安装成功
```

**准确率测试** (抽样 50 句):
```
[ ] 纯中文句子识别准确率 > 92%
[ ] 中英混合句子识别准确率 > 88%
[ ] 热词增强场景准确率 > 90%
```

**稳定性测试**:
```
[ ] 连续使用 2 小时无崩溃
[ ] 撤销 100 次无内存泄漏
[ ] 热词重载 50 次无异常
```

**跨应用测试** (10 个应用):
```
[ ] LibreOffice Writer
[ ] VS Code
[ ] Firefox / Chrome
[ ] Telegram Desktop
[ ] WPS Office
[ ] Kate / Gedit
[ ] Konsole / GNOME Terminal
[ ] Thunderbird
[ ] Slack / 微信
[ ] 钉钉
```

**多发行版测试**:
```
[ ] Deepin 23
[ ] Ubuntu 24.04 LTS
[ ] Fedora 40
[ ] Arch Linux (最新)
[ ] openSUSE Tumbleweed (可选)
```

#### 验证方式
- **测试人员**: 3-5 人
- **测试时间**: 1 周
- **测试设备**: 3 台不同配置 (低端/中端/高端)
- **测试方法**: 实际工作场景 (撰写文档、编程、聊天)

#### 验证输出
- ✅ 功能测试报告 (通过率)
- ✅ 准确率测试报告 (统计数据)
- ✅ 稳定性测试报告 (崩溃日志)
- ✅ 跨应用兼容性报告
- ✅ 用户体验问卷 (5-10 人)
- ✅ P0/P1/P2 Bug 清单

#### Go / No-Go 决策
**Go 条件**:
- P0 Bug = 0
- P1 Bug < 3
- 功能测试通过率 > 98%
- 准确率达到目标
- 用户体验评分 > 7/10

**若 No-Go**: 修复 P0/P1 Bug 后重新验证 (1-2 周)

---

### 交付物
- ✅ vinput v0.9 (功能完整)
- ✅ 完整 GUI 设置界面
- ✅ 第二次验证报告
- ✅ 跨应用/跨发行版兼容性报告
- ⚠️ Bug 修复计划

---

## 🚀 Phase 4: 优化打磨 (3 周) 👤 **第三次人工验证**

### 目标
修复 Bug，优化性能，提升用户体验，准备公开发布。

### 工作内容

#### Week 14: Bug 修复与性能优化

**14.1 修复 Phase 3 发现的所有 P0/P1 Bug**
```bash
[ ] 根据 Bug 清单逐一修复
[ ] 回归测试确保不引入新问题
[ ] 更新单元测试覆盖新场景
```

**14.2 性能优化**
```rust
// 优化项:
[ ] 模型 mmap + madvise 预热
[ ] 离线图优化 (.ort 预序列化)
[ ] Ring Buffer 预分配优化
[ ] 状态机热路径优化
[ ] 内存池复用 (减少分配)
```

**目标**:
- 冷启动优化至 SSD <600ms
- 内存占用优化至 <250MB
- 端到端延迟优化至 <150ms

---

#### Week 15: 用户体验优化

**15.1 视觉效果优化**
```qml
[ ] Preedit 文字动画优化 (过渡更平滑)
[ ] 系统托盘图标动画优化 (脉动/旋转)
[ ] 错误提示样式优化 (颜色、持续时间)
[ ] GUI 深色模式适配
```

**15.2 交互优化**
```rust
[ ] 热键响应延迟优化 (<50ms)
[ ] 设置修改立即生效 (无需重启)
[ ] 错误提示更友好 (附带解决方案)
[ ] 加载状态提示 (Spinner 动画)
```

**15.3 无障碍支持**
```qml
[ ] 所有控件添加 Accessible 属性
[ ] 键盘导航测试 (Tab/Shift+Tab)
[ ] 屏幕阅读器测试 (Orca)
[ ] 高对比度模式测试
```

---

#### Week 16: 文档与教程

**16.1 用户文档**
```markdown
[ ] README.md             // 项目简介、快速入门
[ ] INSTALL.md            // 安装指南 (各发行版)
[ ] USER_GUIDE.md         // 用户手册 (图文教程)
[ ] FAQ.md                // 常见问题
[ ] TROUBLESHOOTING.md    // 故障排查
```

**16.2 开发者文档**
```markdown
[ ] ARCHITECTURE.md       // 架构设计
[ ] CONTRIBUTING.md       // 贡献指南
[ ] API.md                // API 文档
[ ] BUILD.md              // 构建指南
```

**16.3 视频教程** (可选)
```
[ ] 安装教程视频 (5 分钟)
[ ] 使用教程视频 (10 分钟)
[ ] 上传到 Bilibili / YouTube
```

---

### 👤 **第三次人工验证** (Week 16 结束)

#### 验证内容

**完整功能测试** (全面回归):
```
[ ] 基础功能 (语音输入、提交、撤销)
[ ] 高级功能 (热词、断句、标点)
[ ] GUI 所有页面和功能
[ ] 错误场景处理
[ ] 跨应用兼容性
[ ] 多发行版兼容性
```

**性能验证**:
```
[ ] 冷启动时间达标 (SSD <600ms)
[ ] 内存占用达标 (<250MB)
[ ] 端到端延迟达标 (<150ms)
[ ] CPU 占用合理 (空闲 <1%, 识别 <20%)
```

**用户体验测试** (10-20 人):
```
[ ] 真实用户试用 (3-7 天)
[ ] 用户体验问卷 (目标评分 >8/10)
[ ] 收集改进建议
[ ] 统计使用时长和频率
```

**压力测试**:
```
[ ] 连续使用 8 小时无崩溃
[ ] 识别 1000 次无内存泄漏
[ ] 模型重载 100 次无异常
[ ] 极端场景测试 (超长句子、快速切换应用)
```

#### 验证方式
- **测试人员**: 10-20 人 (开发者 + 用户)
- **测试时间**: 1 周
- **测试场景**: 真实工作场景 (日常使用)
- **反馈渠道**: GitHub Issues, 问卷调查, 社区论坛

#### 验证输出
- ✅ 全面测试报告 (功能/性能/体验)
- ✅ 用户反馈汇总 (建议 + Bug)
- ✅ 用户体验评分 (目标 >8/10)
- ✅ Release Candidate 版本 (v1.0-rc1)
- ⚠️ 最终 Bug 清单 (P0-P2)

#### Go / No-Go 决策
**Go 条件**:
- P0 Bug = 0
- P1 Bug < 2
- 用户体验评分 > 8/10
- 性能指标全部达标
- 文档完整

**若 No-Go**: 修复关键问题后发布 RC2 (1 周)

---

### 交付物
- ✅ vinput v1.0-rc1 (Release Candidate)
- ✅ 完整用户文档
- ✅ 完整开发者文档
- ✅ 第三次验证报告
- ⚠️ 最终 Bug 修复计划

---

## 🎉 Phase 5: 发布准备 (2 周) 👤 **最终验证**

### 目标
修复最后的关键 Bug，准备公开发布。

### 工作内容

#### Week 17: 最终 Bug 修复

**17.1 修复所有 P0/P1 Bug**
```bash
[ ] 根据 RC1 反馈修复 Bug
[ ] 回归测试
[ ] 发布 RC2 (若需要)
```

**17.2 发布物准备**
```bash
[ ] 生成 Release Notes
[ ] 构建所有平台安装包:
    - vinput_1.0.0_amd64.deb
    - vinput-1.0.0-1.x86_64.rpm
    - PKGBUILD (上传到 AUR)
[ ] 生成 SHA256 校验和
[ ] 创建 GitHub Release (Draft)
```

---

#### Week 18: 发布与推广

**18.1 公开发布**
```bash
[ ] GitHub Release (v1.0.0)
[ ] 上传安装包到 Release 页面
[ ] AUR 提交 PKGBUILD
[ ] 更新项目网站 (若有)
```

**18.2 社区推广**
```markdown
[ ] Deepin 论坛发布帖
[ ] V2EX 发布帖
[ ] 知乎文章 (项目介绍 + 技术分享)
[ ] Linux.cn / OSCHINA 投稿
[ ] Reddit r/linux, r/linux_devices
[ ] Bilibili 演示视频
```

**18.3 新闻稿**
```markdown
标题: V-Input: Linux 平台首个深度集成 Fcitx5 的离线语音输入法发布

亮点:
- 完全离线，无需网络
- 基于 Zipformer Transducer 流式模型
- 深度集成 Fcitx5，无缝支持所有应用
- 支持热词、撤销、智能标点
- 零常驻内存，冷启动 <600ms
- 跨发行版支持 (Deepin/Ubuntu/Fedora/Arch)
```

---

### 👤 **最终验证** (Week 18)

#### 验证内容

**安装测试** (5 个发行版):
```
[ ] Deepin 23 安装 deb
[ ] Ubuntu 24.04 LTS 安装 deb
[ ] Fedora 40 安装 rpm
[ ] Arch Linux 安装 AUR
[ ] openSUSE Tumbleweed 安装 rpm (可选)
```

**开箱即用测试** (新用户视角):
```
[ ] 安装后自动出现在 Fcitx5 输入法列表
[ ] 首次使用向导流畅
[ ] 默认配置可用 (无需调整)
[ ] 错误提示清晰易懂
[ ] 文档链接正确
```

**社区反馈监控**:
```
[ ] GitHub Issues 响应 (<24h)
[ ] 论坛帖子回复
[ ] 用户问题解答
[ ] Bug 紧急修复 (发布 v1.0.1 补丁)
```

#### 验证方式
- **测试人员**: 全员 + 社区志愿者
- **测试时间**: 1 周
- **监控指标**:
  - GitHub Stars 增长
  - 下载量统计
  - 用户反馈情绪 (正面/负面)
  - 关键 Bug 数量

#### 验证输出
- ✅ 安装测试报告 (各发行版)
- ✅ 社区反馈汇总
- ✅ 用户统计数据
- ✅ v1.0.1 补丁计划 (若需要)

---

### 交付物
- ✅ **vinput v1.0.0 正式版** 🎉
- ✅ 所有平台安装包
- ✅ 完整文档
- ✅ 社区推广完成
- ✅ 用户支持渠道建立

---

## 📊 各阶段总结

### 时间分配

| Phase | 工期 | 占比 | 人工验证 | 关键产出 |
|-------|------|------|---------|---------|
| **Phase 0: 技术验证** | 2 周 | 11% | ⚠️ Go/No-Go | 技术可行性确认 |
| **Phase 1: 核心引擎** | 4 周 | 22% | - | vinput-core 完成 |
| **Phase 2: MVP 集成** | 3 周 | 17% | 👤 第一次 | 基础可用版本 |
| **Phase 3: 功能完善** | 4 周 | 22% | 👤 第二次 | 功能完整版本 |
| **Phase 4: 优化打磨** | 3 周 | 17% | 👤 第三次 | Release Candidate |
| **Phase 5: 发布准备** | 2 周 | 11% | 👤 最终 | v1.0.0 正式版 |
| **总计** | **18 周** | **100%** | **5 次** | **生产级产品** |

---

### 人工验证时间点

| 验证点 | 阶段 | 时间 | 参与人数 | 持续时间 | 关键指标 |
|--------|------|------|---------|---------|---------|
| **Go/No-Go** | Phase 0 | Week 2 | 2-3 人 | 1 天 | 技术可行性 |
| **第一次验证** | Phase 2 | Week 9 | 1-2 人 | 2-3 天 | 基础功能 |
| **第二次验证** | Phase 3 | Week 13 | 3-5 人 | 1 周 | 功能完整性 |
| **第三次验证** | Phase 4 | Week 16 | 10-20 人 | 1 周 | 用户体验 |
| **最终验证** | Phase 5 | Week 18 | 全员 + 社区 | 1 周 | 生产就绪 |

---

### 各阶段风险与缓解

| 阶段 | 主要风险 | 缓解措施 |
|------|---------|---------|
| **Phase 0** | 技术不可行 | 充分验证后 Go/No-Go 决策 |
| **Phase 1** | 开发进度延误 | 聚焦核心功能，砍掉非必要特性 |
| **Phase 2** | 集成问题复杂 | FFI 原型提前验证，多线程架构清晰 |
| **Phase 3** | Bug 数量多 | 分层测试，尽早发现尽早修复 |
| **Phase 4** | 用户体验不达标 | 真实用户测试，快速迭代改进 |
| **Phase 5** | 发布后严重 Bug | 充分测试，建立快速响应机制 |

---

## 🎯 成功标准

### Phase 0 (技术验证)
- ✅ 所有关键技术验证通过
- ✅ Go 决策

### Phase 2 (MVP 集成)
- ✅ 基础功能可用
- ✅ 准确率 > 90%
- ✅ 无 P0 Bug

### Phase 3 (功能完善)
- ✅ 所有 P1 功能实现
- ✅ 准确率 > 92%
- ✅ 跨应用兼容性 > 95%

### Phase 4 (优化打磨)
- ✅ 用户体验评分 > 8/10
- ✅ 性能指标全部达标
- ✅ 文档完整

### Phase 5 (发布准备)
- ✅ **v1.0.0 正式发布** 🎉
- ✅ 社区反馈正面
- ✅ 下载量 > 1000 (首周)

---

## 💡 关键建议

### 1. 尽早验证，频繁验证
- Phase 0 的 Go/No-Go 决策至关重要，避免在错误方向浪费时间
- 每个 Phase 结束都需要验证，而不是等到最后

### 2. 保持 MVP 思维
- Phase 1-2 聚焦核心功能，不追求完美
- Phase 3-4 再逐步完善

### 3. 重视用户反馈
- Phase 3-5 的用户测试是产品成败的关键
- 真实用户的反馈比开发者自测更有价值

### 4. 控制范围蔓延
- 严格按照计划执行，不随意增加功能
- 新需求记录到 v1.1 Backlog

### 5. 文档与代码同步
- 不要等到最后才写文档
- 代码完成后立即更新文档

---

**项目就绪！可立即启动 Phase 0 技术验证！** 🚀
