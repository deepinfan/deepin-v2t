# V-Input Fcitx5 部署测试指南

*生成时间: 2026-02-14*

## 前置条件检查

### 1. 检查系统环境

```bash
# 检查系统版本
lsb_release -a

# 检查 Fcitx5 是否运行
fcitx5 --version

# 检查 PipeWire 状态
systemctl --user status pipewire
```

### 2. 检查麦克风设备

```bash
# 列出音频设备
pw-cli ls Node | grep -A 5 "Audio/Source"

# 测试录音
pw-record --list-targets
```

## 部署步骤

### 第一步：安装开发依赖

#### Deepin/UOS 系统

```bash
# Deepin 系统可能已经预装了 Fcitx5 开发库
# 检查是否已安装
dpkg -l | grep fcitx5 | grep dev

# 如果已安装，只需确保构建工具齐全
sudo apt install -y cmake build-essential pkg-config

# 如果未安装 Fcitx5 开发库
sudo apt update
sudo apt install -y libfcitx5core-dev libfcitx5utils-dev \
    libfcitx5config-dev fcitx5-modules-dev \
    cmake build-essential pkg-config
```

**注意**: Deepin 的包名与其他发行版不同：
- ✅ Deepin: `libfcitx5core-dev`
- ✅ Ubuntu/Debian: `fcitx5-dev` 或 `libfcitx5core-dev`
- ✅ pkg-config: Deepin 使用 `Fcitx5Core`（首字母大写）

验证安装：
```bash
# Deepin 系统
pkg-config --modversion Fcitx5Core

# 其他系统
pkg-config --modversion fcitx5-core
```

#### Ubuntu/Debian 系统

```bash
# 更新软件源
sudo apt update

# 安装 Fcitx5 开发库
sudo apt install -y fcitx5-dev libfcitx5core-dev

# 安装构建工具
sudo apt install -y cmake build-essential pkg-config

# 验证安装
pkg-config --modversion fcitx5-core
```

#### Fedora/RHEL 系统

```bash
# 安装开发库
sudo dnf install -y fcitx5-devel

# 安装构建工具
sudo dnf install -y cmake gcc-c++ pkg-config

# 验证安装
pkg-config --modversion fcitx5-core
```

### 第二步：编译 Rust 核心库

```bash
cd /home/deepin/deepin-v2t/vinput-core

# 编译真实音频捕获版本
cargo build --release --features pipewire-capture

# 验证编译产物
ls -lh ../target/release/libvinput_core.so
```

**预期输出**: 显示 `libvinput_core.so` 文件（约 2-5 MB）

### 第三步：编译 Fcitx5 插件

```bash
cd /home/deepin/deepin-v2t/fcitx5-vinput-mvp

# 运行构建脚本
bash build.sh
```

**预期输出**:
```
=== V-Input Fcitx5 插件构建 ===
✓ 找到 Fcitx5 Core: x.x.x
✓ 找到 libvinput_core.so
配置 CMake...
编译...
✅ 构建成功!
```

### 第四步：安装插件

```bash
cd /home/deepin/deepin-v2t/fcitx5-vinput-mvp/build

# 安装到系统（需要 root 权限）
sudo make install

# 验证安装
ls -la /usr/lib/*/fcitx5/libvinput.so
ls -la /usr/share/fcitx5/addon/vinput.conf
ls -la /usr/share/fcitx5/inputmethod/vinput-im.conf
```

**预期输出**: 显示已安装的文件

### 第五步：重启 Fcitx5

```bash
# 重启 Fcitx5 以加载新插件
fcitx5 -r

# 或者完全重启
pkill fcitx5
sleep 1
fcitx5 &

# 验证插件已加载
fcitx5-remote -d | grep -i vinput
```

### 第六步：配置启用 V-Input

有两种方式配置：

#### 方式 1: 图形界面配置

1. 打开 Fcitx5 配置工具:
   ```bash
   fcitx5-configtool
   ```

2. 在"输入法"选项卡中:
   - 点击"添加输入法"
   - 搜索 "V-Input" 或 "语音输入"
   - 选中并添加

3. 点击"应用"保存配置

#### 方式 2: 命令行配置（推荐）

```bash
# 编辑 Fcitx5 配置文件
mkdir -p ~/.config/fcitx5/profile
cat >> ~/.config/fcitx5/profile/default.conf <<'EOF'

[Groups/0]
Name=Default
DefaultIM=vinput

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=vinput
Layout=

[InputMethod/vinput]
Enabled=True
EOF

# 重启 Fcitx5 应用配置
fcitx5 -r
```

### 第七步：测试语音输入

#### 测试 1: 检查插件状态

```bash
# 查看 Fcitx5 日志
journalctl --user -u fcitx5 -f
```

在另一个终端：
```bash
# 切换到 V-Input
fcitx5-remote -s vinput
```

**预期日志**: 显示 V-Input 插件已激活

#### 测试 2: 实际语音输入

1. 打开任意文本编辑器（如 gedit、kate）

2. 切换到 V-Input 输入法:
   - 方法 1: 使用快捷键（通常是 Ctrl+Space）
   - 方法 2: 点击系统托盘 Fcitx5 图标选择

3. 按下录音快捷键（需要在配置中设置）或直接开始说话

4. 对着麦克风说话：
   ```
   "打开文件"
   "保存文档"
   "你好世界"
   ```

5. 观察文本是否正确输入

## 调试指南

### 问题 1: 插件未加载

**症状**: fcitx5-remote -d 中看不到 vinput

**解决**:
```bash
# 检查插件文件是否存在
ls -la /usr/lib/*/fcitx5/libvinput.so

# 检查配置文件
cat /usr/share/fcitx5/addon/vinput.conf

# 查看 Fcitx5 日志
journalctl --user -u fcitx5 -n 100

# 尝试手动加载
fcitx5 -d
```

### 问题 2: 无法录音

**症状**: 按下录音键但没有反应

**解决**:
```bash
# 检查 PipeWire 状态
systemctl --user status pipewire

# 测试麦克风
pw-record --rate 16000 --channels 1 --format f32 - 2>/dev/null | hexdump -C | head -20

# 检查音频权限
groups $USER | grep -E "(audio|pipewire)"

# 查看 V-Input 日志（如果启用了调试）
RUST_LOG=debug fcitx5 -r
```

### 问题 3: 录音但无识别结果

**症状**: 能看到录音指示但没有文本输出

**原因**: Phase 1 的 ASR 是占位实现

**解决**:
1. 检查 FFI 接口是否正常:
   ```bash
   cd /home/deepin/deepin-v2t
   ./test_multi_commands
   ```

2. 当前 Phase 1 会生成测试文本"语音输入测试"，不是真实识别

3. 要获得真实识别，需要完成 ASR 集成（Phase 1.3）

### 问题 4: 编译错误

**症状**: build.sh 执行失败

**解决**:
```bash
# 检查依赖
pkg-config --list-all | grep fcitx5

# 手动编译
cd /home/deepin/deepin-v2t/fcitx5-vinput-mvp
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr
make VERBOSE=1
```

## 性能验证

### 测试音频捕获性能

```bash
# 运行音频捕获测试
cd /home/deepin/deepin-v2t
cargo run --example test_pipewire_subprocess --features pipewire-capture --release
```

**预期结果**:
- 捕获时长: ~2.9 秒
- Buffer 溢出: 0
- 非零样本: > 70%

### 测试 FFI 接口性能

```bash
# 运行 FFI 测试
cd /home/deepin/deepin-v2t
time ./test_multi_commands
time ./test_error_handling
```

**预期结果**:
- 命令传递: < 1ms
- 无内存泄漏
- 3 条命令正确接收

## 卸载指南

如果需要卸载 V-Input:

```bash
# 1. 从 Fcitx5 配置中移除
fcitx5-configtool  # 图形界面移除

# 2. 删除已安装文件
sudo rm /usr/lib/*/fcitx5/libvinput.so
sudo rm /usr/share/fcitx5/addon/vinput.conf
sudo rm /usr/share/fcitx5/inputmethod/vinput-im.conf

# 3. 重启 Fcitx5
fcitx5 -r
```

## 已知限制（Phase 1）

1. **ASR 为占位实现**:
   - 当前会输出测试文本"语音输入测试"
   - 不会真实识别语音内容
   - Phase 1.3 计划集成 Sherpa-ONNX

2. **VAD 为模拟实现**:
   - 使用简单的能量检测
   - 不如 Silero VAD 精确
   - Phase 1.2 计划集成

3. **候选词显示未实现**:
   - ShowCandidate 命令已定义
   - Fcitx5 端需要实现候选框 UI

4. **快捷键未配置**:
   - 需要手动在 Fcitx5 配置中添加
   - 或者使用全局快捷键工具

## 下一步优化

完成部署测试后，可以考虑：

1. **集成真实 ASR** (Sherpa-ONNX):
   - 实现 OnlineRecognizer 完整功能
   - 端到端语音识别测试

2. **集成真实 VAD** (Silero):
   - 提高语音检测准确度
   - 减少误触发

3. **UI 优化**:
   - 候选词显示
   - 录音状态指示
   - 错误提示

4. **性能调优**:
   - 降低延迟
   - 优化 CPU 占用
   - 电池续航优化

## 技术支持

### 查看日志

```bash
# Fcitx5 日志
journalctl --user -u fcitx5 -f

# V-Input 调试日志（如果启用）
RUST_LOG=vinput_core=debug fcitx5 -r 2>&1 | tee vinput-debug.log

# 系统音频日志
journalctl --user -u pipewire -f
```

### 报告问题

如果遇到问题，请收集以下信息：

1. 系统版本: `lsb_release -a`
2. Fcitx5 版本: `fcitx5 --version`
3. PipeWire 版本: `pw-cli --version`
4. 错误日志: `journalctl --user -u fcitx5 -n 100`
5. 音频设备: `pw-cli ls Node | grep Audio`

## 验收标准

部署测试成功的标志：

- ✅ Fcitx5 插件编译成功
- ✅ 插件已安装到系统目录
- ✅ Fcitx5 成功加载 V-Input
- ✅ 可以切换到 V-Input 输入法
- ✅ 音频捕获正常（测试程序验证）
- ✅ FFI 接口工作正常（测试程序验证）
- ✅ 按下录音键有反应
- ✅ 能够接收到命令（即使是测试文本）

**注意**: 由于 Phase 1 的 ASR 是占位实现，不要期望真实的语音识别结果。这是正常的！

---

*文档版本: 1.0*
*适用于: V-Input Phase 1*
*最后更新: 2026-02-14*
