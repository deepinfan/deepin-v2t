# V-Input 测试指南

**版本**: 1.0
**日期**: 2026-02-14
**状态**: Phase 2-4 核心功能测试

---

## 📋 测试概览

V-Input 系统包含三个主要组件，每个都可以独立测试：

1. **GUI 设置界面** - 配置管理和用户界面
2. **核心引擎** - VAD、ASR、ITN、标点、热词
3. **Fcitx5 集成** - 输入法插件（需要后续集成）

---

## 🎨 Phase 3: GUI 设置界面测试

### 测试目标
验证 GUI 配置界面的所有功能正常工作。

### 1. 启动 GUI

```bash
cd /home/deepin/deepin-v2t
cargo run -p vinput-gui --release
```

**预期结果**：
- ✅ 窗口正常打开（800x600）
- ✅ 所有中文文字正常显示（不是方框）
- ✅ 左侧显示三个选项卡：🔥热词管理、📝标点控制、🎤VAD/ASR

### 2. 测试热词管理

**操作步骤**：
1. 点击 "🔥 热词管理" 选项卡
2. 在输入框中输入热词，例如 "深度操作系统"
3. 调整权重滑块（1.0 - 5.0）
4. 点击 "添加" 按钮
5. 查看热词列表中是否出现
6. 点击热词旁的 "删除" 按钮
7. 调整 "全局热词权重" 滑块

**预期结果**：
- ✅ 能添加热词到列表
- ✅ 权重滑块可调整
- ✅ 能删除已添加的热词
- ✅ 全局权重设置生效
- ✅ 底部显示 "⚠ 配置已修改"

### 3. 测试标点控制

**操作步骤**：
1. 点击 "📝 标点控制" 选项卡
2. 尝试选择三种预设风格：
   - Professional（专业）
   - Balanced（平衡）
   - Expressive（表达）
3. 调整 "停顿检测阈值" 滑块
4. 修改 "最小 Token 数"
5. 切换 "允许感叹号" 开关
6. 切换 "问号严格模式" 开关

**预期结果**：
- ✅ 能切换三种预设风格
- ✅ 切换风格后参数自动更新
- ✅ 所有滑块和开关响应正常
- ✅ 参数说明文字清晰易懂
- ✅ 配置修改后状态栏提示 "⚠ 配置已修改"

### 4. 测试 VAD/ASR 面板

**操作步骤**：
1. 点击 "🎤 VAD/ASR" 选项卡
2. 切换 VAD 模式（Push-to-Talk / Continuous）
3. 调整 "启动阈值" 和 "结束阈值"
4. 修改 "最小语音时长" 和 "最小静音时长"
5. 查看 ASR 模型路径
6. 选择采样率（8kHz / 16kHz）
7. 调整热词分数

**预期结果**：
- ✅ VAD 模式切换正常
- ✅ 所有参数滑块响应流畅
- ✅ 参数值实时显示
- ✅ 路径显示正确

### 5. 测试配置保存

**操作步骤**：
1. 修改任意配置项
2. 点击底部 "应用" 按钮
3. 关闭 GUI 窗口
4. 重新启动 GUI
5. 检查配置是否保留

**预期结果**：
- ✅ 点击 "应用" 后状态变为 "✓ 已保存"
- ✅ 配置文件保存到 `~/.config/vinput/config.toml`
- ✅ 重启后配置保持不变

**检查配置文件**：
```bash
cat ~/.config/vinput/config.toml
```

### 6. 测试菜单功能

**操作步骤**：
1. 点击 "文件" 菜单
   - 测试 "保存配置"
   - 测试 "重置为默认"
   - 测试 "退出"
2. 点击 "帮助" 菜单
   - 测试 "关于"

**预期结果**：
- ✅ 所有菜单项可点击
- ✅ "保存配置" 保存成功
- ✅ "重置为默认" 恢复默认值
- ✅ "退出" 关闭窗口

---

## 🧠 Phase 2: 核心引擎测试

V-Input 核心引擎包含多个独立测试示例，每个都可以单独运行。

### 前置条件

确保模型文件已下载：
```bash
ls -lh models/streaming/
# 应该看到：
# - tokens.txt
# - encoder-*.onnx
# - decoder-*.onnx
# - joiner-*.onnx
```

如果没有，运行：
```bash
cd models
./download_model.sh  # 如果有这个脚本
```

### 测试 1: ITN 文本规范化

**测试目的**：验证中文数字、英文数字转换功能

```bash
cargo run --example itn_demo --release
```

**预期输出**：
```
=== ITN Engine Demo ===

测试 1: 中文数字转换
输入: 我有三个苹果
输出: 我有3个苹果

输入: 今天是二零二六年二月十四日
输出: 今天是2026年2月14日

测试 2: 英文数字保留
输入: I have twenty three books
输出: I have 23 books

测试 3: 百分比转换
输入: 增长了百分之五十
输出: 增长了50%

✅ 所有测试通过
```

### 测试 2: ITN 性能测试

**测试目的**：验证 ITN 超高性能（目标 < 1ms）

```bash
cargo run --example itn_performance --release
```

**预期输出**：
```
=== ITN Performance Benchmark ===

测试用例数: 10000
平均处理时间: 0.9-18μs
吞吐量: 55,000+ 句/秒

✅ 性能达标（超目标 55 倍！）
```

### 测试 3: 标点控制

**测试目的**：验证三种标点风格

```bash
cargo run --example punctuation_demo --release
```

**预期输出**：
```
=== Punctuation Engine Demo ===

风格 1: Professional
输入: 今天天气很好 我们去公园吧
输出: 今天天气很好，我们去公园。

风格 2: Balanced
输入: 今天天气很好 我们去公园吧
输出: 今天天气很好，我们去公园吧。

风格 3: Expressive
输入: 今天天气真好啊 我们去公园吧
输出: 今天天气真好啊！我们去公园吧！

✅ 所有风格测试通过
```

### 测试 4: 流式管道测试

**测试目的**：验证 VAD + ASR 流式管道

```bash
# 注意：需要实际音频输入或测试音频文件
cargo run --example streaming_pipeline_test --release --features vad-onnx
```

**预期行为**：
- ✅ 启动时初始化 Sherpa-ONNX 模型
- ✅ 可以接收音频数据
- ✅ VAD 检测语音活动
- ✅ ASR 输出识别结果
- ✅ 延迟 < 500ms（目标）

### 测试 5: 完整端到端测试

**测试目的**：测试完整的 VAD → ASR → ITN → Punctuation → Hotwords 流程

```bash
cargo run --example phase2_complete_e2e --release --features vad-onnx
```

**需要准备**：
- 测试音频文件（WAV 格式，16kHz 采样率）
- 或麦克风输入

**预期输出**：
```
=== Phase 2 Complete E2E Test ===

[VAD] 检测到语音活动
[ASR] 识别中...
[ITN] 文本规范化: "我有三个苹果" → "我有3个苹果"
[Punctuation] 添加标点: "我有3个苹果。"
[Hotwords] 应用热词增强
[最终] 我有3个苹果。

✅ E2E 测试通过
```

### 测试 6: 单元测试（全套）

**运行所有 133 个单元测试**：

```bash
cargo test --release
```

**预期结果**：
```
running 133 tests
test result: ok. 133 passed; 0 failed; 0 ignored

✅ 所有单元测试通过
```

**按模块测试**：
```bash
# VAD 模块
cargo test --release vad

# ITN 模块
cargo test --release itn

# 标点模块
cargo test --release punctuation

# 热词模块
cargo test --release hotwords
```

---

## 🔧 Phase 4: Fcitx5 集成测试（需要完善）

**注意**：Fcitx5 集成框架已完成，但实际 FFI 连接尚未完成。以下是未来测试步骤。

### 安装准备

```bash
# 编译核心库
cargo build -p vinput-core --release

# 编译 Fcitx5 插件（需要先实现 FFI 连接）
cd fcitx5-vinput
mkdir -p build && cd build
cmake ..
make
```

### 安装到系统

```bash
sudo ./install.sh
```

**预期操作**：
- ✅ 复制 `libvinput_core.so` 到 `/usr/lib/`
- ✅ 复制 `vinput.so` 到 `/usr/lib/fcitx5/`
- ✅ 复制配置文件到 `/usr/share/fcitx5/`

### 重启 Fcitx5

```bash
fcitx5 -r
```

### 添加 V-Input 输入法

1. 打开 Fcitx5 配置界面
2. 查找 "V-Input" 输入法
3. 添加到输入法列表

### 测试语音输入

1. 切换到 V-Input 输入法
2. **按住 Ctrl+Space** 键
3. 对着麦克风说话（中文）
4. 松开键
5. 应该出现候选词窗口
6. 按 1-9 选择候选词

**预期结果**：
- ✅ Ctrl+Space 触发录音
- ✅ 语音识别成功
- ✅ 候选词正确显示
- ✅ 选择后文本输入到应用

### 卸载

```bash
sudo ./uninstall.sh
```

---

## 🐛 故障排除

### 问题 1: GUI 中文显示方框

**解决方案**：
```bash
# 检查系统字体
fc-list :lang=zh-cn

# 确保有以下字体之一：
# - Source Han Sans SC
# - Noto Sans CJK
# - WenQuanYi Micro Hei

# 如果没有，安装：
sudo apt install fonts-noto-cjk
# 或
sudo apt install fonts-wqy-microhei
```

### 问题 2: 编译失败 - serde derive

**解决方案**：
```bash
# 确保 Cargo.toml 中 serde 启用 derive feature
# 已在 workspace 中配置：
# serde = { version = "1.0", features = ["derive"] }
```

### 问题 3: ONNX Runtime 链接失败

**解决方案**：
```bash
# 检查 ONNX Runtime 库
ls -lh deps/sherpa-onnx/lib/libonnxruntime.so*

# 确保 .cargo/config.toml 包含：
# ORT_LIB_LOCATION = { value = "deps/sherpa-onnx/lib", relative = true }
# ORT_PREFER_DYNAMIC_LINK = "1"
```

### 问题 4: 音频捕获失败

**解决方案**：
```bash
# 检查 PipeWire 服务
systemctl --user status pipewire

# 重启 PipeWire
systemctl --user restart pipewire

# 检查麦克风权限
pw-cli ls Node | grep -i mic
```

### 问题 5: 模型文件缺失

**解决方案**：
```bash
# 下载 Sherpa-ONNX 中文模型
cd models
# 手动下载或使用脚本
wget https://github.com/k2-fsa/sherpa-onnx/releases/download/...
```

---

## 📊 测试检查清单

### GUI 测试 ✅
- [ ] 窗口正常启动
- [ ] 中文显示正常
- [ ] 热词添加/删除
- [ ] 标点风格切换
- [ ] VAD/ASR 参数调整
- [ ] 配置保存/加载
- [ ] 菜单功能

### 核心引擎测试 ✅
- [ ] ITN 数字转换
- [ ] ITN 性能（< 1ms）
- [ ] 标点三种风格
- [ ] 流式管道启动
- [ ] 133 个单元测试通过

### Fcitx5 集成测试 ⏳
- [ ] 编译成功
- [ ] 安装脚本正常
- [ ] Fcitx5 识别插件
- [ ] 快捷键触发
- [ ] 语音识别
- [ ] 候选词显示
- [ ] 文本输入

---

## 🎯 性能指标

根据设计目标，以下是需要验证的性能指标：

| 指标 | 目标 | 测试方法 |
|------|------|----------|
| ITN 处理延迟 | < 1ms | itn_performance example |
| VAD 响应时间 | < 100ms | 实际音频测试 |
| ASR 延迟 | < 500ms | E2E 测试 |
| 内存占用 | < 500MB | `ps aux | grep vinput` |
| CPU 使用率 | < 30% | `top` 观察 |

---

## 📝 测试报告模板

测试完成后，可以记录结果：

```markdown
# V-Input 测试报告

**测试日期**: 2026-02-14
**测试人员**: [您的名字]
**测试版本**: Phase 2-4

## GUI 测试结果
- 启动: ✅/❌
- 中文显示: ✅/❌
- 配置保存: ✅/❌
- 问题: [描述任何问题]

## 核心引擎测试结果
- ITN: ✅/❌
- 标点: ✅/❌
- 单元测试: 133/133 通过
- 问题: [描述任何问题]

## 性能测试结果
- ITN 延迟: [实际值] μs
- 内存占用: [实际值] MB
- CPU 使用: [实际值] %

## 总体评价
[您的评价和建议]
```

---

**Happy Testing! 🎉**

如有任何问题，请参考：
- 技术文档: `docs/PROJECT_COMPLETE_SUMMARY.md`
- Phase 2 报告: `docs/PHASE2_COMPLETE_SUMMARY.md`
- Phase 3&4 报告: `docs/PHASE3_4_COMPLETION_REPORT.md`
