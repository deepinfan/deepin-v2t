# V-Input Phase 2 实施计划

*创建时间: 2026-02-14*
*基于设计文档: VAD、ITN、标点、热词、撤销系统*

## 总体目标

Phase 2 的目标是实现完整的语音识别功能，从 Phase 1 的占位实现升级为真实的语音输入系统。

### 核心组件

根据设计文档，Phase 2 需要实现以下核心组件：

1. **VAD (Voice Activity Detection)** - Silero VAD
2. **ASR (Automatic Speech Recognition)** - Sherpa-ONNX Streaming Zipformer
3. **ITN (Inverse Text Normalization)** - cn2an-rs + 规则层
4. **标点系统** - Zipformer 自带 + 规则层
5. **热词引擎** - 自定义词汇增强
6. **撤销/重试机制** - 错误纠正

### 技术栈

```
模型栈:
├── ASR: sherpa-onnx-streaming-zipformer-zh-int8-2025-06-30
├── VAD: Silero VAD (ONNX)
├── ITN: cn2an-rs + 规则引擎
└── 标点: Zipformer 输出 + 规则增强

架构:
PipeWire → Ring Buffer → VAD → ASR → ITN → 标点 → 热词 → Fcitx5
                          ↓
                    Endpoint Detection
```

## Phase 2.1: VAD 集成（优先级：最高）

### 设计要求（基于 VAD 设计文档）

#### 双输入模式
- **PushToTalk**: 按住说话（当前实现）
- **AutoDetect**: 自动检测语音（待实现）

#### 系统架构
```
Audio Input → Energy Gate → Silero VAD → Hysteresis Controller
    → Speech/Silence Timer → Pre-roll Buffer → ASR
```

#### 核心参数

**PushToTalk 模式**:
```rust
VadConfig {
    start_threshold: 0.6,
    end_threshold: 0.35,
    min_speech_duration_ms: 100,
    min_silence_duration_ms: 500,
}
```

**AutoDetect 模式**:
```rust
VadConfig {
    start_threshold: 0.68,
    end_threshold: 0.35,
    min_speech_duration_ms: 180,
    min_silence_duration_ms: 900,
}
```

#### 实施任务

- [ ] Task 2.1.1: 集成 Silero VAD ONNX 模型
  - 下载 Silero VAD v4 模型
  - 使用 ort (onnxruntime) 加载模型
  - 实现推理接口

- [ ] Task 2.1.2: 实现 Energy Gate
  - 10ms 帧 RMS 计算
  - 动态噪声基线估计
  - 阈值触发逻辑

- [ ] Task 2.1.3: 实现 Hysteresis Controller
  - 双阈值状态机
  - Speech/Silence 转换
  - 防抖动机制

- [ ] Task 2.1.4: 实现 Pre-roll Buffer
  - 缓存前 200-300ms 音频
  - 防止吞首字
  - 与 ASR 集成

- [ ] Task 2.1.5: 实现短爆发噪声过滤器
  - 键盘/鼠标噪声识别
  - 持续时间检测
  - 过滤逻辑

### 文件结构

```
vinput-core/src/vad/
├── mod.rs              # 模块入口
├── silero.rs           # Silero VAD 封装（已存在，需完善）
├── energy_gate.rs      # Energy Gate 实现
├── hysteresis.rs       # 迟滞控制器
├── pre_roll_buffer.rs  # Pre-roll 缓冲区
└── config.rs           # VAD 配置
```

## Phase 2.2: ASR 集成（优先级：最高）

### 设计要求

#### 模型
- **sherpa-onnx-streaming-zipformer-zh-int8-2025-06-30**
- 支持流式识别
- 支持端点检测
- 支持热词

#### 实施任务

- [ ] Task 2.2.1: 下载并验证模型
  - 下载 Zipformer 中文模型
  - 验证模型文件完整性
  - 测试基本推理

- [ ] Task 2.2.2: 完善 OnlineRecognizer
  - 实现完整的流式识别 API
  - 支持热词配置
  - 支持端点检测回调

- [ ] Task 2.2.3: 集成 VAD 与 ASR
  - VAD 触发 ASR 开始
  - 音频数据流送入 ASR
  - ASR 端点触发 VAD 结束

- [ ] Task 2.2.4: 性能优化
  - 线程模型优化
  - 内存复用
  - 延迟优化（目标 < 200ms）

### 文件结构

```
vinput-core/src/asr/
├── mod.rs              # 模块入口
├── recognizer.rs       # OnlineRecognizer 实现（已存在，需完善）
├── streaming.rs        # 流式识别逻辑
└── config.rs           # ASR 配置
```

## Phase 2.3: ITN 集成（优先级：高）

### 设计要求（基于 ITN 设计文档）

#### 核心功能
- 数字转换（"一千二百三十四" → "1234"）
- 日期转换（"二零二六年二月十四日" → "2026年2月14日"）
- 时间转换（"下午三点半" → "15:30"）
- 货币转换（"三块五" → "3.5元"）
- 符号转换（"顿号" → "、"）

#### 技术方案
- **cn2an-rs**: Rust 实现的中文数字转换库
- **规则引擎**: 正则表达式 + 状态机

#### 实施任务

- [ ] Task 2.3.1: 集成 cn2an-rs
  - 添加依赖
  - 封装转换接口
  - 测试基本转换

- [ ] Task 2.3.2: 实现日期/时间规则
  - 日期格式识别
  - 时间格式识别
  - 相对时间处理（"明天"、"下周"）

- [ ] Task 2.3.3: 实现货币/符号规则
  - 货币单位识别
  - 符号映射表
  - 上下文处理

- [ ] Task 2.3.4: 实现规则引擎
  - 规则匹配器
  - 优先级处理
  - 冲突解决

### 文件结构

```
vinput-core/src/itn/
├── mod.rs              # 模块入口
├── numbers.rs          # 数字转换（cn2an-rs）
├── datetime.rs         # 日期时间转换
├── currency.rs         # 货币转换
├── symbols.rs          # 符号转换
└── rules.rs            # 规则引擎
```

## Phase 2.4: 标点系统（优先级：中）

### 设计要求（基于标点设计文档）

#### 数据源
1. **Zipformer 输出**: 模型自带标点
2. **规则增强**: 基于语速、停顿的规则

#### 策略
- **CPS (Characters Per Second)** 语速检测
- **停顿时长** 映射标点
- **上下文规则** 智能选择

#### 实施任务

- [ ] Task 2.4.1: 解析 Zipformer 标点输出
  - 提取时间戳
  - 标点符号映射
  - 置信度评估

- [ ] Task 2.4.2: 实现 CPS 计算
  - 滑动窗口统计
  - 语速分级
  - 标点阈值调整

- [ ] Task 2.4.3: 实现规则引擎
  - 停顿时长 → 标点映射
  - 上下文规则（问句、感叹）
  - 冲突解决策略

### 文件结构

```
vinput-core/src/punctuation/
├── mod.rs              # 模块入口（已存在）
├── parser.rs           # 解析 Zipformer 输出
├── cps_detector.rs     # 语速检测
├── rules.rs            # 规则引擎
└── config.rs           # 标点配置
```

## Phase 2.5: 热词引擎（优先级：中）

### 设计要求（基于热词设计文档）

#### 功能
- 自定义词汇增强
- 上下文相关词汇
- 动态热词更新

#### 实施任务

- [ ] Task 2.5.1: 实现热词管理
  - 热词存储（文件/数据库）
  - 热词加载
  - 热词更新接口

- [ ] Task 2.5.2: 集成到 ASR
  - Sherpa-ONNX 热词 API
  - 动态热词注入
  - 权重配置

- [ ] Task 2.5.3: GUI 配置界面（可选）
  - 热词列表管理
  - 导入/导出
  - 分类管理

### 文件结构

```
vinput-core/src/hotwords/
├── mod.rs              # 模块入口（已存在）
├── manager.rs          # 热词管理器
├── storage.rs          # 存储后端
└── config.rs           # 热词配置
```

## Phase 2.6: 撤销/重试机制（优先级：低）

### 设计要求（基于撤销设计文档）

#### 功能
- 撤销最近一次输入
- 重新识别（使用不同参数）
- 历史记录管理

#### 实施任务

- [ ] Task 2.6.1: 实现历史记录
  - 音频缓存
  - 识别结果缓存
  - LRU 清理

- [ ] Task 2.6.2: 实现撤销逻辑
  - Fcitx5 撤销命令
  - 状态回滚
  - UI 提示

- [ ] Task 2.6.3: 实现重试逻辑
  - 参数调整策略
  - 多候选展示
  - 用户选择

### 文件结构

```
vinput-core/src/undo/
├── mod.rs              # 模块入口（已存在）
├── history.rs          # 历史记录
├── cache.rs            # 音频缓存
└── retry.rs            # 重试逻辑
```

## 实施优先级与顺序

### 第一阶段：基础功能（2-3 周）

1. **Phase 2.1: VAD 集成** ⭐⭐⭐
   - 必需：语音检测是核心功能
   - 依赖：Silero VAD 模型

2. **Phase 2.2: ASR 集成** ⭐⭐⭐
   - 必需：语音识别核心
   - 依赖：Sherpa-ONNX 模型

### 第二阶段：增强功能（1-2 周）

3. **Phase 2.3: ITN 集成** ⭐⭐
   - 重要：提升识别结果可用性
   - 依赖：cn2an-rs 库

4. **Phase 2.4: 标点系统** ⭐⭐
   - 重要：自然语言体验
   - 依赖：ASR 时间戳

### 第三阶段：扩展功能（1 周）

5. **Phase 2.5: 热词引擎** ⭐
   - 可选：提升专业词汇识别
   - 依赖：ASR 热词 API

6. **Phase 2.6: 撤销/重试** ⭐
   - 可选：错误纠正
   - 依赖：历史记录

## 技术准备

### 模型文件

需要下载的模型：

1. **Silero VAD v4**
   - URL: https://github.com/snakers4/silero-vad
   - 大小: ~2MB
   - 格式: ONNX

2. **Sherpa-ONNX Zipformer (中文)**
   - URL: https://github.com/k2-fsa/sherpa-onnx
   - 大小: ~200MB (int8 量化)
   - 格式: ONNX

### Rust 依赖

需要添加的 crate：

```toml
[dependencies]
# 现有依赖
ort = "2.0"              # ONNX Runtime (已有)

# 新增依赖
cn2an = "0.1"            # 中文数字转换
regex = "1.10"           # 正则表达式
chrono = "0.4"           # 日期时间处理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"       # 配置文件
```

### 开发环境

- Rust 1.70+
- ONNX Runtime 1.16+
- PipeWire (已配置)
- Fcitx5 开发库 (已安装)

## 验收标准

### Phase 2.1 验收（VAD）

- [ ] 能够自动检测语音开始
- [ ] 能够自动检测语音结束
- [ ] 误触发率 < 5%（办公环境测试）
- [ ] 漏触发率 < 2%
- [ ] 吞首字率 = 0%

### Phase 2.2 验收（ASR）

- [ ] 能够实时转写语音
- [ ] 识别准确率 > 90%（标准普通话）
- [ ] 端到端延迟 < 500ms
- [ ] 支持流式输出
- [ ] 支持端点检测

### Phase 2.3 验收（ITN）

- [ ] 数字转换准确率 > 95%
- [ ] 日期转换准确率 > 90%
- [ ] 时间转换准确率 > 90%
- [ ] 不影响非目标文本

### Phase 2.4 验收（标点）

- [ ] 句号插入准确率 > 80%
- [ ] 逗号插入准确率 > 70%
- [ ] 问号插入准确率 > 85%
- [ ] 不影响阅读流畅性

## 风险评估

### 高风险项

1. **ASR 模型性能**
   - 风险：模型可能不适配 Deepin 环境
   - 缓解：提前验证模型，准备备选方案

2. **VAD 准确性**
   - 风险：办公环境噪声干扰
   - 缓解：充分测试，调整参数

3. **延迟控制**
   - 风险：端到端延迟过高
   - 缓解：性能分析，逐层优化

### 中风险项

1. **ITN 规则覆盖度**
   - 风险：规则可能不够全面
   - 缓解：迭代改进，用户反馈

2. **内存占用**
   - 风险：多模型加载内存压力大
   - 缓解：模型量化，懒加载

## 下一步行动

### 立即开始（本次会话）

1. 创建 Phase 2 目录结构
2. 添加必要的依赖
3. 下载 Silero VAD 模型
4. 实现 VAD 基础框架

### 后续规划

1. 每周完成 1-2 个 Phase
2. 每个 Phase 完成后进行测试
3. 根据测试结果调整参数
4. 迭代优化用户体验

---

*计划制定完成，准备开始 Phase 2.1 实施*
