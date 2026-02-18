# 水滴语音输入法 DEB 打包计划（修订版）

## Debian 文件系统规范研究结果

根据 Debian Policy Manual 和实际系统分析：

### 1. 库文件路径 ✅
- **共享库**: `/usr/lib/x86_64-linux-gnu/`
  - 符合 Debian 多架构规范
  - 系统中 fcitx5 插件都在此路径

### 2. Fcitx5 插件路径 ✅
- **插件 .so 文件**: `/usr/lib/x86_64-linux-gnu/fcitx5/`
  - 参考: fcitx5-pinyin 使用此路径
  - 示例: `/usr/lib/x86_64-linux-gnu/fcitx5/libpinyin.so`

### 3. Fcitx5 配置文件 ✅
- **Addon 配置**: `/usr/share/fcitx5/addon/`
- **输入法配置**: `/usr/share/fcitx5/inputmethod/`
  - 参考: fcitx5-pinyin 使用此结构

### 4. 应用数据文件 ✅
- **大型数据文件**: `/usr/share/<package-name>/`
  - 参考: `/usr/share/seetaface-models/` (72MB AI 模型)
  - 参考: `/usr/share/fcitx5/pinyin/` (拼音词库)
  - 符合 FHS 规范

### 5. 可执行文件 ✅
- **用户命令**: `/usr/bin/`
  - 标准路径，无需修改

### 6. 桌面文件 ✅
- **Desktop Entry**: `/usr/share/applications/`
  - 标准路径

### 7. 系统配置 ⚠️ 需要调整
- **建议改为**: `/usr/share/droplet-voice-input/config.toml.example`
- **原因**:
  - `/etc/` 用于系统管理员可修改的配置
  - 我们的配置主要在用户目录 `~/.config/vinput/`
  - 提供示例配置更合适

---

## 修订后的文件列表

### 1. 核心库 (4.4 MB)
```
源文件: target/release/libvinput_core.so
目标路径: /usr/lib/x86_64-linux-gnu/libvinput_core.so
权限: 0644
说明: 共享库文件，不需要可执行权限
```

### 2. Fcitx5 插件 (78 KB)
```
源文件: fcitx5-vinput/build/vinput.so
目标路径: /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so
权限: 0644
说明: Fcitx5 插件模块
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
说明: GUI 设置程序
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
Keywords=voice;input;speech;recognition;语音;输入;
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

说明: 参考 seetaface-models 的做法，大型数据文件放在 /usr/share/
```

### 7. 示例配置文件
```
源文件: config.toml.example
目标路径: /usr/share/droplet-voice-input/config.toml.example
权限: 0644

说明:
- 不放在 /etc/，因为用户配置在 ~/.config/vinput/
- 提供示例配置供用户参考
- 首次运行时程序会自动创建用户配置
```

### 8. Sherpa-ONNX 依赖库 ⚠️ 重要
```
源文件: deps/sherpa-onnx/lib/libsherpa-onnx-c-api.so
目标路径: /usr/lib/x86_64-linux-gnu/libsherpa-onnx-c-api.so
权限: 0644

源文件: deps/sherpa-onnx/lib/libonnxruntime.so
目标路径: /usr/lib/x86_64-linux-gnu/libonnxruntime.so
权限: 0644

说明:
- 这两个库是必需的运行时依赖
- 目前系统中没有 onnxruntime 包，需要打包进去
- 或者声明为 Provides: libonnxruntime
```

---

## 依赖关系（修订）

### 运行时依赖
```
Depends: fcitx5, libpipewire-0.3-0 | libpipewire-0.3-modules
```

### 提供的库
```
Provides: libonnxruntime, libsherpa-onnx-c-api
```

### 推荐依赖
```
Recommends: pipewire-audio
```

---

## 目录结构（符合 Debian 规范）

```
/usr/
├── bin/
│   └── vinput-settings                           (0755, 18 MB)
├── lib/x86_64-linux-gnu/
│   ├── libvinput_core.so                         (0644, 4.4 MB)
│   ├── libsherpa-onnx-c-api.so                   (0644, ~50 MB)
│   ├── libonnxruntime.so                         (0644, ~100 MB)
│   └── fcitx5/
│       └── vinput.so                             (0644, 78 KB)
└── share/
    ├── applications/
    │   └── droplet-voice-input.desktop           (0644)
    ├── fcitx5/
    │   ├── addon/
    │   │   └── vinput-addon.conf                 (0644)
    │   └── inputmethod/
    │       └── vinput.conf                       (0644)
    └── droplet-voice-input/
        ├── models/
        │   ├── encoder.int8.onnx                 (0644, 158 MB)
        │   ├── decoder.int8.onnx                 (0644, 69 MB)
        │   └── tokens.txt                        (0644, 74 KB)
        └── config.toml.example                   (0644)
```

---

## 包大小估算（修订）

- 核心库: 4.4 MB
- Fcitx5 插件: 78 KB
- 设置程序: 18 MB
- AI 模型: 227 MB
- Sherpa-ONNX 库: ~50 MB
- ONNX Runtime 库: ~100 MB

**总计**: 约 400 MB (压缩后约 250-300 MB)

---

## 安装后脚本 (postinst)

```bash
#!/bin/bash
set -e

case "$1" in
    configure)
        # 更新动态链接库缓存
        ldconfig

        # 创建用户配置目录模板（如果不存在）
        if [ ! -d /etc/skel/.config/vinput ]; then
            mkdir -p /etc/skel/.config/vinput
            # 复制示例配置
            if [ -f /usr/share/droplet-voice-input/config.toml.example ]; then
                cp /usr/share/droplet-voice-input/config.toml.example \
                   /etc/skel/.config/vinput/config.toml
            fi
        fi

        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  💧 水滴语音输入法安装完成！"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "  使用方法："
        echo "    1. 重启 Fcitx5: fcitx5 -r"
        echo "    2. 在 Fcitx5 配置中添加「水滴语音输入法」"
        echo "    3. 切换到水滴语音输入法"
        echo "    4. 按空格开始录音，说话后松开空格"
        echo "    5. 运行 vinput-settings 打开设置界面"
        echo ""
        echo "  首发于深度操作系统论坛: http://bbs.deepin.org"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        ;;
esac

exit 0
```

---

## 卸载前脚本 (prerm)

```bash
#!/bin/bash
set -e

case "$1" in
    remove|upgrade|deconfigure)
        # 停止 Fcitx5（如果正在运行）
        if pgrep -x fcitx5 > /dev/null; then
            echo "正在停止 Fcitx5..."
            pkill fcitx5 || true
            sleep 1
        fi
        ;;
esac

exit 0
```

---

## 卸载后脚本 (postrm)

```bash
#!/bin/bash
set -e

case "$1" in
    remove|purge)
        # 更新动态链接库缓存
        ldconfig
        ;;

    purge)
        # 完全卸载时删除用户配置模板
        rm -rf /etc/skel/.config/vinput
        echo "配置文件模板已删除"
        ;;
esac

exit 0
```

---

## 关键修改点

### ✅ 符合规范的改动：

1. **库文件权限**: 从 0755 改为 0644
   - 共享库不需要可执行权限
   - 参考系统中其他 .so 文件都是 0644

2. **配置文件位置**: 从 `/etc/` 改为 `/usr/share/`
   - 提供示例配置而非系统配置
   - 用户配置在 `~/.config/vinput/`

3. **添加 sherpa-onnx 依赖库**
   - 必须打包 libsherpa-onnx-c-api.so
   - 必须打包 libonnxruntime.so
   - 否则程序无法运行

4. **依赖关系简化**
   - 移除不存在的 libonnxruntime 包依赖
   - 改为 Provides 声明

---

## 需要修改代码的地方

### 1. 默认模型路径

修改 `vinput-core/src/config/mod.rs`:

```rust
// 修改前
pub fn default_model_dir() -> String {
    "models/streaming".to_string()
}

// 修改后
pub fn default_model_dir() -> String {
    // 优先使用用户配置，其次使用系统路径
    if let Ok(home) = std::env::var("HOME") {
        let user_models = format!("{}/.local/share/droplet-voice-input/models", home);
        if std::path::Path::new(&user_models).exists() {
            return user_models;
        }
    }
    "/usr/share/droplet-voice-input/models".to_string()
}
```

---

## 确认事项

请确认以下修改：

1. ✅ 库文件权限改为 0644（不可执行）
2. ✅ 配置文件改为示例配置（/usr/share/）
3. ✅ 添加 sherpa-onnx 和 onnxruntime 库到包中
4. ✅ 模型文件路径 `/usr/share/droplet-voice-input/models/`
5. ⚠️ 需要修改代码中的默认模型路径
6. ⚠️ 包大小增加到约 400 MB（因为包含 onnxruntime）

**是否同意以上修改？确认后我将创建打包脚本。**
