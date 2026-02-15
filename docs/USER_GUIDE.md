# V-Input 用户手册

## 简介

V-Input 是一个离线中文语音输入法，基于 Fcitx5 框架，支持实时语音识别、智能标点、文本规范化等功能。

## 系统要求

- 操作系统: Linux (Deepin, Ubuntu, Arch, Fedora 等)
- 桌面环境: 支持 Fcitx5 的任何桌面环境
- 音频系统: PipeWire 或 PulseAudio
- 内存: 至少 2GB RAM
- 存储: 至少 500MB 可用空间（用于模型文件）

## 安装

### Debian/Ubuntu/Deepin

```bash
sudo dpkg -i vinput_0.1.0_amd64.deb
sudo apt-get install -f  # 安装依赖
```

### Arch Linux

```bash
yay -S vinput
# 或
makepkg -si  # 从 PKGBUILD 构建
```

### Fedora/RHEL

```bash
sudo rpm -i vinput-0.1.0-1.x86_64.rpm
```

## 配置

### 启用输入法

1. 打开 Fcitx5 配置工具
2. 在"输入法"选项卡中，点击"添加输入法"
3. 搜索"V-Input"并添加
4. 将 V-Input 设置为默认输入法或添加到输入法列表

### 配置文件

配置文件位于: `~/.config/vinput/config.toml`

示例配置:

```toml
[hotwords]
words = { "深度学习" = 2.8, "人工智能" = 2.5 }
global_weight = 2.5
max_words = 10000

[punctuation]
style = "Professional"
pause_ratio = 3.5
min_tokens = 5
allow_exclamation = false
question_strict = true

[vad]
mode = "push-to-toggle"
start_threshold = 0.5
end_threshold = 0.3
min_speech_duration = 250
min_silence_duration = 300

[asr]
model_dir = "/usr/share/vinput/models/streaming"
sample_rate = 16000
hotwords_score = 1.5

[endpoint]
min_speech_duration_ms = 300
max_speech_duration_ms = 30000
trailing_silence_ms = 800
force_timeout_ms = 60000
vad_assisted = true
vad_silence_confirm_frames = 5
```

## 使用方法

### 基本操作

1. **开始录音**: 按下空格键
2. **停止录音**: 再次按下空格键
3. **撤销**: Ctrl+Z
4. **重试**: Ctrl+Y

### 录音模式

V-Input 支持三种录音模式:

1. **按住说话 (Push-to-Talk)**: 按住热键时录音，松开后停止
2. **按键切换 (Push-to-Toggle)**: 按一次开始录音，再按一次停止（默认）
3. **连续识别 (Continuous)**: 持续监听并自动识别语音

### 文本规范化 (ITN)

V-Input 自动将语音识别结果规范化:

- **数字**: "一千二百三十四" → "1234"
- **日期**: "二零二六年三月五日" → "2026年3月5日"
- **货币**: "三百块钱" → "¥300"
- **百分比**: "百分之五十" → "50%"

ITN 模式:
- **自动模式**: 启用全部规范化规则（默认）
- **仅数字模式**: 仅转换数字
- **原始模式**: 跳过全部规范化

### 智能标点

V-Input 根据语音停顿和语调自动添加标点符号:

- **逗号**: 检测到短暂停顿时自动插入
- **句号**: 检测到句子结束时自动插入
- **问号**: 检测到疑问语调时自动插入

标点风格:
- **Professional**: 专业风格，标点较少
- **Casual**: 随意风格，标点较多
- **Minimal**: 最少标点

### 热词管理

热词可以提升特定词汇的识别准确率。

#### 添加热词

1. 打开 V-Input 设置界面
2. 切换到"热词管理"选项卡
3. 输入词汇和权重（1.0-5.0）
4. 点击"添加"

#### 热词文件格式

创建 `~/.config/vinput/hotwords.txt`:

```
# 热词列表
深度学习 2.8
人工智能 2.5
机器学习 2.6
神经网络 2.7
```

### 音频设备选择

1. 打开 V-Input 设置界面
2. 切换到"基本设置"选项卡
3. 在"音频输入设备"下拉菜单中选择麦克风
4. 点击"应用"保存设置

## 常见问题

### 无法识别语音

1. 检查麦克风是否正常工作
2. 确认音频设备选择正确
3. 检查 PipeWire/PulseAudio 是否运行
4. 查看日志: `journalctl --user -u fcitx5 -f`

### 识别准确率低

1. 确保环境安静，减少背景噪音
2. 说话清晰，语速适中
3. 添加专业术语到热词列表
4. 调整 VAD 阈值（在高级设置中）

### 标点符号不准确

1. 调整停顿检测阈值（`pause_ratio`）
2. 更改标点风格（Professional/Casual/Minimal）
3. 说话时适当停顿

### 撤销/重试不工作

1. 确认使用的是最新版本
2. 检查快捷键是否冲突
3. 查看 Fcitx5 日志确认命令是否发送

## 高级功能

### 端点检测

端点检测自动判断句子结束时机:

- **最小语音长度**: 至少说话多久才开始识别
- **最大语音长度**: 超过此时长自动结束
- **尾随静音**: 说话结束后等待多久
- **强制超时**: 最长录音时间

### 性能优化

1. **冷启动优化**: 首次启动可能较慢，后续会更快
2. **内存优化**: 关闭不需要的功能（如热词）
3. **延迟优化**: 使用 Push-to-Talk 模式减少延迟

## 故障排除

### 查看日志

```bash
# Fcitx5 日志
journalctl --user -u fcitx5 -f

# V-Input Core 日志
VINPUT_LOG=1 fcitx5 -r
```

### 重置配置

```bash
rm -rf ~/.config/vinput
fcitx5 -r
```

### 重新安装

```bash
# Debian/Ubuntu
sudo apt-get remove --purge vinput
sudo apt-get install vinput

# Arch
yay -R vinput
yay -S vinput
```

## 支持与反馈

- GitHub Issues: https://github.com/yourusername/vinput/issues
- 文档: https://github.com/yourusername/vinput/wiki
- 邮件: support@example.com

## 许可证

V-Input 采用 MIT 许可证发布。详见 LICENSE 文件。
