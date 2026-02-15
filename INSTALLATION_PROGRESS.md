# V-Input 安装进度说明

## 当前状态

✅ **步骤 1/5 完成**: vinput-core (release) 编译成功

编译过程中的警告是正常的，不影响功能使用。

## 接下来的步骤

### 步骤 2/5: 安装插件

脚本现在正在等待您输入 **sudo 密码**。

请在终端中输入您的密码，然后按回车继续。

### 后续步骤（自动执行）

安装脚本将自动执行以下操作：

1. **安装 Fcitx5 插件**
   - 复制 `vinput.so` 到 `/usr/local/lib/fcitx5/`
   - 复制 `libvinput_core.so` 到 `/usr/local/lib/`

2. **安装模型文件**
   - 复制 VAD 模型到 `/usr/share/vinput/models/silero-vad/`
   - 复制 ASR 模型到 `/usr/share/vinput/models/streaming/`

3. **创建配置文件**
   - 创建 `/etc/vinput/config.toml`

4. **重启 Fcitx5**
   - 执行 `fcitx5 -r`

5. **显示使用说明**

## 安装完成后的操作

### 1. 添加 V-Input 输入法

```bash
fcitx5-configtool
```

在配置工具中：
1. 点击"输入法"选项卡
2. 点击"添加输入法"按钮
3. 搜索 "V-Input" 或 "vinput"
4. 添加到输入法列表

### 2. 开始使用

1. 切换到 V-Input 输入法（使用 Fcitx5 的输入法切换快捷键）
2. 打开任意文本编辑器（如 gedit、kate、文本编辑器）
3. **按下空格键**开始录音（应该看到 "🎤 录音中..." 提示）
4. 说话："今天天气很好"
5. **再次按下空格键**停止录音
6. 等待识别结果自动上屏

### 3. 测试示例

| 说话内容 | 预期结果 |
|---------|---------|
| "今天天气很好" | 今天天气很好。 |
| "我花了三百块钱" | 我花了¥300 |
| "今天是二零二六年三月五日" | 今天是2026年3月5日。 |
| "你今天吃饭了吗"（疑问语调） | 你今天吃饭了吗？ |

### 4. 快捷键

- **空格键**: 开始/停止录音
- **Ctrl+Z**: 撤销最后一次识别
- **Ctrl+Y**: 重试（恢复撤销的内容）

### 5. GUI 设置工具

```bash
./run-settings.sh
```

可以配置：
- 录音模式（按住说话/按键切换/连续识别）
- ITN 模式（自动/仅数字/原始）
- 音频设备选择
- 热词管理
- 标点风格
- VAD 阈值
- 等等...

## 故障排查

### 如果看不到 V-Input 输入法

```bash
# 检查插件是否安装
ls -la /usr/local/lib/fcitx5/vinput.so

# 检查核心库
ls -la /usr/local/lib/libvinput_core.so

# 重启 Fcitx5
fcitx5 -r
```

### 如果无法录音

```bash
# 检查 PipeWire 是否运行
pw-cli info 0

# 检查麦克风设备
pactl list sources short

# 查看实时日志
journalctl --user -u fcitx5 -f
```

### 查看详细日志

```bash
# 启用调试日志
VINPUT_LOG=1 fcitx5 -r

# 在另一个终端查看日志
journalctl --user -u fcitx5 -f
```

## 需要帮助？

- 查看完整测试指南: `TESTING_GUIDE.md`
- 查看用户手册: `docs/USER_GUIDE.md`
- 查看开发者文档: `docs/DEVELOPER_GUIDE.md`

## 预期的安装输出

安装成功后，您应该看到类似以下的输出：

```
✓ Fcitx5 插件安装完成
✓ 核心库安装完成
✓ 模型目录创建完成
✓ VAD 模型复制完成
✓ ASR 模型复制完成
✓ 默认配置文件创建完成

==========================================
安装完成！
==========================================

下一步操作:
1. 重启 Fcitx5: fcitx5 -r
2. 打开 Fcitx5 配置工具
3. 添加 'V-Input' 输入法
4. 使用空格键开始/停止录音
```

---

**请继续在终端中输入密码，让安装脚本继续执行。**
