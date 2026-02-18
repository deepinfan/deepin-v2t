# 模型路径修复说明

## 问题

原代码硬编码了开发环境的模型路径：
```rust
"/home/deepin/deepin-v2t/models/streaming"
```

这导致安装到新系统后，程序无法找到模型文件。

## 解决方案

修改了 `vinput-core/src/config/mod.rs`，使用智能路径查找：

```rust
// 默认模型路径优先级：
// 1. 环境变量 VINPUT_MODEL_DIR
// 2. 系统安装路径 /usr/share/droplet-voice-input/models
// 3. 开发路径 ./models/streaming
let default_model_dir = std::env::var("VINPUT_MODEL_DIR")
    .unwrap_or_else(|_| {
        // 检查系统安装路径
        let system_path = "/usr/share/droplet-voice-input/models";
        if std::path::Path::new(system_path).exists() {
            system_path.to_string()
        } else {
            // 开发环境路径
            "./models/streaming".to_string()
        }
    });
```

## 路径优先级

1. **环境变量** `VINPUT_MODEL_DIR`
   - 用户可以自定义模型路径
   - 例如：`export VINPUT_MODEL_DIR=/path/to/models`

2. **系统安装路径** `/usr/share/droplet-voice-input/models`
   - DEB 包安装后的标准路径
   - 自动检测是否存在

3. **开发路径** `./models/streaming`
   - 相对于当前工作目录
   - 用于开发环境

## 配置文件

用户也可以在配置文件中指定模型路径：

`~/.config/vinput/config.toml`:
```toml
[asr]
model_dir = "/usr/share/droplet-voice-input/models"
sample_rate = 16000
max_active_paths = 2
```

配置文件中的路径优先级最高。

## 验证

安装后检查模型路径：

```bash
# 1. 检查模型文件是否存在
ls -la /usr/share/droplet-voice-input/models/

# 2. 启动 Fcitx5 并查看日志
fcitx5 --verbose=10 2>&1 | grep -i "model"

# 3. 检查配置
cat ~/.config/vinput/config.toml
```

## 已重新打包

新的 DEB 包已经包含此修复：
- 文件：`droplet-voice-input_0.1.0_amd64.deb`
- 大小：219 MB
- 日期：2026-02-18

安装后会自动使用正确的模型路径。
