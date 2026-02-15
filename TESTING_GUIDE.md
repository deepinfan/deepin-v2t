# V-Input 实际使用测试指南

## 安装步骤

### 1. 编译 Release 版本

```bash
cd /home/deepin/deepin-v2t

# 编译核心库 (release)
cd vinput-core
cargo build --release
cd ..

# 编译 Fcitx5 插件
cd fcitx5-vinput/build
cmake ..
make
cd ../..
```

### 2. 安装插件

```bash
sudo ./install-fcitx5-plugin.sh
```

安装脚本会执行以下操作：
- 安装 Fcitx5 插件到系统目录
- 复制核心库到 `/usr/local/lib/`
- 复制模型文件到 `/usr/share/vinput/models/`
- 创建默认配置文件

### 3. 重启 Fcitx5

```bash
fcitx5 -r
```

或者完全重启：
```bash
killall fcitx5
fcitx5 &
```

### 4. 配置输入法

1. 打开 Fcitx5 配置工具：
   ```bash
   fcitx5-configtool
   ```

2. 在"输入法"选项卡中，点击"添加输入法"

3. 搜索 "V-Input" 或 "vinput"

4. 添加到输入法列表

5. 可以设置为默认输入法或使用快捷键切换

---

## 使用测试

### 基本功能测试

#### 测试 1: 简单录音识别

1. 切换到 V-Input 输入法
2. 打开任意文本编辑器（如 gedit、kate）
3. **按下空格键**开始录音（应该看到 "🎤 录音中..." 提示）
4. 说话："今天天气很好"
5. **再次按下空格键**停止录音
6. 等待识别结果自动上屏

**预期结果**: 文本 "今天天气很好。" 自动输入到编辑器

#### 测试 2: 数字识别 (ITN)

1. 开始录音
2. 说话："我花了一千二百三十四块钱"
3. 停止录音

**预期结果**: "我花了¥1234"

#### 测试 3: 日期识别 (ITN)

1. 开始录音
2. 说话："今天是二零二六年三月五日"
3. 停止录音

**预期结果**: "今天是2026年3月5日。"

#### 测试 4: 智能标点

1. 开始录音
2. 说话："今天天气很好（停顿）我们去公园吧"
3. 停止录音

**预期结果**: "今天天气很好，我们去公园吧。"

#### 测试 5: 问句识别

1. 开始录音
2. 说话："你今天吃饭了吗"（用疑问语调）
3. 停止录音

**预期结果**: "你今天吃饭了吗？"

#### 测试 6: 撤销功能

1. 完成一次识别
2. 按 **Ctrl+Z**

**预期结果**: 刚才输入的文本被删除

#### 测试 7: 重试功能

1. 撤销后，按 **Ctrl+Y**

**预期结果**: 文本重新输入

---

## 高级测试

### 测试 8: 热词功能

1. 运行 GUI 设置：
   ```bash
   ./run-settings.sh
   ```

2. 切换到"🔥 热词管理"页面

3. 添加热词：
   - 词汇: "深度操作系统"
   - 权重: 3.0

4. 点击"应用"保存

5. 重启 Fcitx5: `fcitx5 -r`

6. 测试识别："我使用深度操作系统"

**预期结果**: "深度操作系统" 识别准确率提升

### 测试 9: 标点风格切换

1. 打开 GUI 设置

2. 切换到"📝 标点控制"页面

3. 选择不同的标点风格：
   - Professional（专业）：标点较少
   - Balanced（平衡）：适中
   - Expressive（表达）：标点较多

4. 测试相同的句子，观察标点差异

### 测试 10: 录音模式切换

1. 打开 GUI 设置

2. 切换到"⚙️ 基本设置"页面

3. 选择录音模式：
   - **按住说话 (Push-to-Talk)**: 按住空格说话，松开停止
   - **按键切换 (Push-to-Toggle)**: 按一次开始，再按一次停止（默认）
   - **连续识别 (Continuous)**: 自动检测语音

4. 测试不同模式的使用体验

---

## 故障排查

### 问题 1: 找不到 V-Input 输入法

**解决方案**:
```bash
# 检查插件是否安装
ls -la /usr/local/lib/fcitx5/vinput.so

# 检查核心库
ls -la /usr/local/lib/libvinput_core.so

# 重启 Fcitx5
fcitx5 -r
```

### 问题 2: 无法录音

**解决方案**:
```bash
# 检查 PipeWire 是否运行
pw-cli info 0

# 检查麦克风权限
pactl list sources short

# 查看日志
journalctl --user -u fcitx5 -f
```

### 问题 3: 识别不准确

**解决方案**:
1. 确保环境安静
2. 说话清晰，语速适中
3. 添加专业术语到热词列表
4. 调整 VAD 阈值（在 GUI 高级设置中）

### 问题 4: 标点不准确

**解决方案**:
1. 调整停顿检测阈值（`pause_ratio`）
2. 更改标点风格
3. 说话时适当停顿

### 问题 5: 查看详细日志

```bash
# 启用调试日志
VINPUT_LOG=1 fcitx5 -r

# 查看实时日志
journalctl --user -u fcitx5 -f

# 查看最近的错误
journalctl --user -u fcitx5 --priority=err -n 50
```

---

## 性能测试

### 测试指标

1. **冷启动时间**: 首次启动 Fcitx5 到可以使用的时间
2. **识别延迟**: 停止录音到文本上屏的时间
3. **内存占用**: 运行时的内存使用
4. **CPU 占用**: 识别时的 CPU 使用率

### 测试命令

```bash
# 查看内存占用
ps aux | grep fcitx5

# 查看 CPU 占用
top -p $(pgrep fcitx5)

# 测试识别延迟（使用 time 命令）
time echo "测试" | vinput-test
```

---

## 配置文件位置

- **用户配置**: `~/.config/vinput/config.toml`
- **系统配置**: `/etc/vinput/config.toml`
- **模型文件**: `/usr/share/vinput/models/`
- **插件文件**: `/usr/local/lib/fcitx5/vinput.so`
- **核心库**: `/usr/local/lib/libvinput_core.so`

---

## 卸载

```bash
# 删除插件
sudo rm /usr/local/lib/fcitx5/vinput.so

# 删除核心库
sudo rm /usr/local/lib/libvinput_core.so

# 删除模型文件
sudo rm -rf /usr/share/vinput

# 删除配置
rm -rf ~/.config/vinput
sudo rm -rf /etc/vinput

# 重启 Fcitx5
fcitx5 -r
```

---

## 测试清单

使用以下清单确保所有功能正常：

- [ ] 基本录音识别
- [ ] 数字转换 (ITN)
- [ ] 日期转换 (ITN)
- [ ] 货币转换 (ITN)
- [ ] 智能标点（逗号）
- [ ] 智能标点（句号）
- [ ] 智能标点（问号）
- [ ] 撤销功能 (Ctrl+Z)
- [ ] 重试功能 (Ctrl+Y)
- [ ] 热词添加
- [ ] 热词生效
- [ ] 标点风格切换
- [ ] 录音模式切换
- [ ] GUI 设置界面
- [ ] 配置保存/加载

---

## 反馈

如果遇到问题或有改进建议，请：

1. 查看日志: `journalctl --user -u fcitx5 -f`
2. 提交 Issue: https://github.com/yourusername/vinput/issues
3. 附上详细的错误信息和日志

---

**测试完成后，请记录测试结果并反馈！**
