# V-Input Phase 1 开发计划

**启动日期：** 2026-02-13
**预计完成：** 2026-05-15 (3 个月)
**当前状态：** 🚀 开发中

---

## 📋 任务总览

### 优先级 P0 - 核心功能（必须完成）

| # | 任务 | 状态 | 预计工作量 | 负责人 |
|---|------|------|-----------|--------|
| 14 | PipeWire 实际音频捕获 | 🔄 进行中 | 2-3 周 | - |
| 15 | VAD ONNX 推理实现 | 📋 待开始 | 1-2 周 | - |
| 16 | Fcitx5 完整集成 | 📋 待开始 | 2-3 周 | - |
| 17 | FFI 命令传递完善 | 📋 待开始 | 1 周 | - |

**P0 小计：** 6-9 周

### 优先级 P1 - 重要功能

| # | 任务 | 状态 | 预计工作量 |
|---|------|------|-----------|
| 18 | 端点检测优化 | 📋 待开始 | 1 周 |
| 19 | 错误处理增强 | 📋 待开始 | 1 周 |
| 20 | Phase 1 集成测试 | 📋 待开始 | 1 周 |

**P1 小计：** 3 周

### 优先级 P2 - 可选功能（时间允许）

- ITN 逆文本正规化
- 热词支持
- 撤销/重试机制
- 用户配置界面

**P2 小计：** 3-4 周

---

## 🎯 里程碑

### Milestone 1: 音频捕获 (Week 1-3) ✅ 目标
- ✅ PipeWire 设备枚举
- ✅ 实时音频流捕获
- ✅ Ring Buffer 集成
- ✅ 错误处理和恢复

**验收标准：**
- 可以成功枚举音频设备
- 可以捕获 16kHz 单声道音频
- 音频数据正确写入 Ring Buffer
- 设备断开可以自动恢复

### Milestone 2: VAD ONNX (Week 4-5) 📋 待开始
- [ ] ONNX Runtime 集成
- [ ] Silero VAD 模型推理
- [ ] 性能优化 < 1ms
- [ ] 状态管理完善

**验收标准：**
- VAD 推理性能 < 1ms/帧
- 检测准确率 > 90%
- 状态机工作正常
- 内存使用稳定

### Milestone 3: Fcitx5 集成 (Week 6-8) 📋 待开始
- [ ] 按键处理逻辑
- [ ] 候选词展示
- [ ] 语音输入触发
- [ ] Fcitx5 实际测试

**验收标准：**
- 空格键触发录音
- 识别结果正确显示
- 候选词可选择
- 用户体验流畅

### Milestone 4: 完善和测试 (Week 9-12) 📋 待开始
- [ ] FFI 命令传递
- [ ] 端点检测优化
- [ ] 错误处理增强
- [ ] 完整集成测试
- [ ] 性能优化
- [ ] 文档更新

**验收标准：**
- 所有功能正常工作
- 性能指标满足要求
- 无已知 bug
- 文档完整

---

## 🔧 技术栈更新

### 新增依赖

**Rust 库：**
- `pipewire = "0.9"` - PipeWire 音频捕获
- `ort = "2.0"` - ONNX Runtime (VAD 推理)
- `crossbeam-channel` - 线程间通信
- `parking_lot` - 高性能锁

**系统依赖：**
- `libpipewire-0.3-dev` - PipeWire 开发库
- `libspa-0.2-dev` - SPA 插件开发库
- `libonnxruntime-dev` - ONNX Runtime 开发库

### 架构更新

```
V-Input Phase 1 架构
├── Audio Capture Layer (NEW)
│   ├── PipeWire Device Manager
│   ├── Audio Stream Processor
│   └── Ring Buffer Producer
├── Processing Layer
│   ├── VAD ONNX Inference (NEW)
│   ├── ASR Online Recognition
│   └── Endpoint Detection (NEW)
├── FFI Layer (Enhanced)
│   ├── Event Queue (NEW)
│   ├── Command Queue (NEW)
│   └── State Synchronization (NEW)
└── Fcitx5 Integration (Complete)
    ├── Input Method Engine
    ├── Candidate Window
    └── Key Event Handler (NEW)
```

---

## 📊 开发进度跟踪

### Week 1 (2026-02-13 ~ 02-19)
**目标：** PipeWire 基础集成
- [ ] 更新 pipewire 依赖到 0.9
- [ ] 实现设备枚举
- [ ] 实现基础音频捕获
- [ ] 编写单元测试

### Week 2 (2026-02-20 ~ 02-26)
**目标：** PipeWire 完善
- [ ] 错误处理和恢复
- [ ] Ring Buffer 集成
- [ ] 性能优化
- [ ] 集成测试

### Week 3 (2026-02-27 ~ 03-05)
**目标：** PipeWire 收尾 + VAD ONNX 启动
- [ ] PipeWire 完整测试
- [ ] ONNX Runtime 集成
- [ ] Silero VAD 模型加载

---

## 🎨 设计决策

### 1. PipeWire 集成方式

**选择：** 使用官方 `pipewire-rs` crate

**理由：**
- 官方维护，API 稳定
- 文档完善
- 社区支持好

**替代方案：**
- `pipewire-native`: 纯 Rust 实现，但不够成熟
- 直接 FFI: 开发成本高

### 2. ONNX Runtime 选择

**选择：** 使用 `ort` crate

**理由：**
- 高性能
- 支持多平台
- Rust API 友好

**配置：**
- CPU 推理（Phase 1）
- 可选 GPU 加速（Phase 2）

### 3. 线程模型

**选择：** 多线程异步模型

```
Thread 1: PipeWire Audio Capture
    ↓ (Ring Buffer)
Thread 2: VAD + ASR Processing
    ↓ (Command Queue)
Thread 3: Fcitx5 Main Thread
```

**理由：**
- 降低延迟
- 充分利用多核
- 组件解耦

---

## 🔍 风险管理

### 已识别风险

| 风险 | 等级 | 缓解措施 | 负责人 |
|------|------|---------|--------|
| PipeWire API 复杂度 | 🟡 中 | 参考官方示例，渐进开发 | - |
| VAD ONNX 性能 | 🟡 中 | 提前性能测试，优化参数 | - |
| Fcitx5 测试环境 | 🟡 中 | 准备完整测试环境 | - |
| 线程同步复杂度 | 🟢 低 | 使用成熟并发库 | - |

---

## 📈 质量目标

### 性能指标

- **实时率:** > 1.0x (保持 Phase 0 的高性能)
- **端到端延迟:** < 100ms
- **VAD 延迟:** < 1ms
- **内存占用:** < 500 MB

### 稳定性指标

- **无崩溃时间:** > 24 小时
- **设备断开恢复:** < 1 秒
- **错误恢复率:** > 95%

### 用户体验指标

- **响应延迟:** < 100ms
- **识别准确率:** > 90%
- **操作流畅度:** 60 FPS+

---

## 📚 参考资料

### 技术文档

- [PipeWire Documentation](https://docs.pipewire.org/)
- [ONNX Runtime Docs](https://onnxruntime.ai/docs/)
- [Fcitx5 Developer Guide](https://fcitx-im.org/wiki/Development)
- [Silero VAD Model](https://github.com/snakers4/silero-vad)

### 代码示例

- `vinput-core/examples/` - Phase 0 示例代码
- [pipewire-rs examples](https://github.com/pipewire/pipewire-rs/tree/main/pipewire/examples)

---

## ✅ Definition of Done

Phase 1 完成的标准：

1. **功能完整性**
   - [ ] 可以从麦克风捕获音频
   - [ ] VAD 可以准确检测语音
   - [ ] ASR 可以识别中文语音
   - [ ] Fcitx5 可以正确输入识别结果

2. **性能达标**
   - [ ] 实时率 > 1.0x
   - [ ] 端到端延迟 < 100ms
   - [ ] 内存占用 < 500 MB

3. **稳定性验证**
   - [ ] 24 小时稳定性测试通过
   - [ ] 错误恢复测试通过
   - [ ] 设备热插拔测试通过

4. **文档完善**
   - [ ] API 文档更新
   - [ ] 用户手册编写
   - [ ] 安装指南更新

5. **测试覆盖**
   - [ ] 单元测试覆盖率 > 70%
   - [ ] 集成测试通过
   - [ ] 端到端测试通过

---

**Last Updated:** 2026-02-13
**Status:** 🚀 Phase 1 开发启动！

---

## 🚀 下一步

立即开始 **Task #14: PipeWire 实际音频捕获**

```bash
cd vinput-core
cargo add pipewire@0.9
```

Let's build it! 💪
