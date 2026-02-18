# 水滴语音输入法 DEB 打包计划

## 包信息

- **包名**: `droplet-voice-input`
- **版本**: `0.1.0`
- **架构**: `amd64`
- **维护者**: `Deepin Community <bbs.deepin.org>`
- **描述**: 水滴语音输入法 - 离线中文语音输入法
- **首页**: `http://bbs.deepin.org`

## 依赖关系

### 运行时依赖
```
Depends: fcitx5, libpipewire-0.3-0, libonnxruntime1.14.1 | libonnxruntime
```

### 推荐依赖
```
Recommends: pipewire-audio
```

## 文件列表和安装路径

### 1. 核心库 (4.4 MB)
```
源文件: target/release/libvinput_core.so
目标路径: /usr/lib/x86_64-linux-gnu/libvinput_core.so
权限: 0755
```

### 2. Fcitx5 插件 (78 KB)
```
源文件: fcitx5-vinput/build/vinput.so
目标路径: /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so
权限: 0755
```

### 3. Fcitx5 配置文件
```
源文件: fcitx5-vinput/vinput.conf
目标路径: /usr/share/fcitx5/inputmethod/vinput.conf
权限: 0644

源文件: fcitx5-vinput/vinput-addon.conf
目标路径: /usr/share/fcitx5/addon/vinput-addon.conf
权限: 0644
```

### 4. 设置程序 (18 MB)
```
源文件: target/release/vinput-settings
目标路径: /usr/bin/vinput-settings
权限: 0755
```

### 5. 桌面启动文件
```
创建文件: droplet-voice-input.desktop
目标路径: /usr/share/applications/droplet-voice-input.desktop
权限: 0644

内容:
[Desktop Entry]
Name=水滴语音输入法设置
Name[en]=Droplet Voice Input Settings
Comment=配置水滴语音输入法
Comment[en]=Configure Droplet Voice Input
Exec=vinput-settings
Icon=audio-input-microphone
Terminal=false
Type=Application
Categories=Settings;Utility;
Keywords=voice;input;speech;recognition;
```

### 6. AI 模型文件 (227 MB)
```
源文件: models/streaming/encoder.int8.onnx (158 MB)
目标路径: /usr/share/droplet-voice-input/models/encoder.int8.onnx
权限: 0644

源文件: models/streaming/decoder.int8.onnx (69 MB)
目标路径: /usr/share/droplet-voice-input/models/decoder.int8.onnx
权限: 0644

源文件: models/streaming/tokens.txt (74 KB)
目标路径: /usr/share/droplet-voice-input/models/tokens.txt
权限: 0644
```

### 7. 默认配置文件
```
创建文件: config.toml (基于 config.toml.example)
目标路径: /etc/droplet-voice-input/config.toml
权限: 0644

说明: 用户配置会保存在 ~/.config/vinput/config.toml
```

### 8. 文档文件 (可选，你说不打包)
```
不打包以下文件:
- README.md
- docs/
- *.md (所有 Markdown 文档)
```

## 安装后脚本 (postinst)

```bash
#!/bin/bash
set -e

# 更新动态链接库缓存
ldconfig

# 创建用户配置目录模板
mkdir -p /etc/skel/.config/vinput

# 提示用户
echo "水滴语音输入法安装完成！"
echo "请重启 Fcitx5 或重新登录以使用。"
echo ""
echo "使用方法："
echo "  1. 在 Fcitx5 配置中添加「水滴语音输入法」"
echo "  2. 切换到水滴语音输入法"
echo "  3. 按空格开始录音，说话后松开空格"
echo "  4. 运行 vinput-settings 打开设置界面"
echo ""
echo "首发于深度操作系统论坛: http://bbs.deepin.org"
```

## 卸载前脚本 (prerm)

```bash
#!/bin/bash
set -e

# 停止 Fcitx5（如果正在运行）
if pgrep -x fcitx5 > /dev/null; then
    echo "正在停止 Fcitx5..."
    pkill fcitx5 || true
fi
```

## 卸载后脚本 (postrm)

```bash
#!/bin/bash
set -e

if [ "$1" = "purge" ]; then
    # 完全卸载时删除配置文件
    rm -rf /etc/droplet-voice-input
    echo "配置文件已删除"
fi

# 更新动态链接库缓存
ldconfig
```

## 包大小估算

- 核心库: 4.4 MB
- Fcitx5 插件: 78 KB
- 设置程序: 18 MB
- AI 模型: 227 MB
- 配置文件: < 1 MB

**总计**: 约 250 MB (压缩后约 150-180 MB)

## 目录结构预览

```
/usr/
├── bin/
│   └── vinput-settings                    (18 MB)
├── lib/x86_64-linux-gnu/
│   ├── libvinput_core.so                  (4.4 MB)
│   └── fcitx5/
│       └── vinput.so                      (78 KB)
└── share/
    ├── applications/
    │   └── droplet-voice-input.desktop
    ├── fcitx5/
    │   ├── addon/
    │   │   └── vinput-addon.conf
    │   └── inputmethod/
    │       └── vinput.conf
    └── droplet-voice-input/
        └── models/
            ├── encoder.int8.onnx          (158 MB)
            ├── decoder.int8.onnx          (69 MB)
            └── tokens.txt                 (74 KB)

/etc/
└── droplet-voice-input/
    └── config.toml                        (默认配置)
```

## 配置文件说明

默认配置文件 `/etc/droplet-voice-input/config.toml` 内容：

```toml
[asr]
model_dir = "/usr/share/droplet-voice-input/models"
sample_rate = 16000
max_active_paths = 2

[vad]
threshold = 0.5
min_speech_duration_ms = 250
min_silence_duration_ms = 500

[punctuation]
style = "Professional"

[hotwords]
# 用户可以在 ~/.config/vinput/config.toml 中添加自定义热词
```

## 注意事项

1. **模型文件路径**: 代码中需要更新默认模型路径为 `/usr/share/droplet-voice-input/models`
2. **配置文件优先级**:
   - 用户配置: `~/.config/vinput/config.toml` (优先)
   - 系统配置: `/etc/droplet-voice-input/config.toml` (默认)
3. **权限**: 所有可执行文件 0755，配置文件 0644
4. **架构**: 目前仅支持 amd64，后续可扩展到 arm64

## 构建命令

```bash
# 1. 清理并重新编译 (Release 模式)
cargo clean
cargo build --release

# 2. 编译 Fcitx5 插件
cd fcitx5-vinput/build
cmake .. -DCMAKE_BUILD_TYPE=Release
make

# 3. 创建 deb 包
cd /home/deepin/deepin-v2t
./build-deb.sh
```

## 测试安装

```bash
# 安装
sudo dpkg -i droplet-voice-input_0.1.0_amd64.deb

# 修复依赖（如果有）
sudo apt-get install -f

# 测试
fcitx5 -r
vinput-settings

# 卸载
sudo apt-get remove droplet-voice-input

# 完全卸载（包括配置）
sudo apt-get purge droplet-voice-input
```

---

**请确认以上信息是否正确，特别是：**
1. 文件安装路径是否合适？
2. 模型文件放在 `/usr/share/droplet-voice-input/models/` 是否可以？
3. 依赖关系是否完整？
4. 是否需要添加其他文件？

确认后我将开始创建打包脚本。
