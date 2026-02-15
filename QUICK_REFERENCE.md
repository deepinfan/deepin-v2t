# V-Input 快速参考卡片

## 🎤 基本操作

| 操作 | 快捷键 |
|------|--------|
| 开始录音 | 空格键 |
| 停止录音 | 空格键 |
| 撤销 | Ctrl+Z |
| 重试 | Ctrl+Y |

## 📝 识别示例

```
说话: "今天天气很好"
结果: 今天天气很好。

说话: "我花了三百块钱"
结果: 我花了¥300

说话: "今天是二零二六年三月五日"
结果: 今天是2026年3月5日。

说话: "百分之五十"
结果: 50%
```

## ⚙️ 常用命令

```bash
# 重启 Fcitx5
fcitx5 -r

# 打开配置工具
fcitx5-configtool

# 运行 GUI 设置
./run-settings.sh

# 查看日志
journalctl --user -u fcitx5 -f

# 启用调试日志
VINPUT_LOG=1 fcitx5 -r
```

## 🔧 配置文件

- 用户配置: `~/.config/vinput/config.toml`
- 系统配置: `/etc/vinput/config.toml`
- 模型目录: `/usr/share/vinput/models/`

## 🐛 快速故障排查

```bash
# 检查插件
ls -la /usr/local/lib/fcitx5/vinput.so

# 检查核心库
ls -la /usr/local/lib/libvinput_core.so

# 检查 PipeWire
pw-cli info 0

# 检查麦克风
pactl list sources short
```

## 📚 文档

- 用户手册: `docs/USER_GUIDE.md`
- 测试指南: `TESTING_GUIDE.md`
- 开发者文档: `docs/DEVELOPER_GUIDE.md`

## 💡 提示

1. 说话清晰，语速适中
2. 环境尽量安静
3. 适当停顿会自动添加逗号
4. 疑问语调会自动添加问号
5. 专业术语可添加到热词列表

---

**祝您使用愉快！** 🎉
