# 配置文件自动创建修复

## 问题描述

首次启动 `vinput-settings` 时，如果 `~/.config/vinput/config.toml` 不存在，任何操作都会导致程序出错。

**原因**：
- GUI 加载配置时，如果文件不存在，返回默认配置但不保存
- 用户修改设置后保存时，尝试写入不存在的配置文件
- 导致保存失败或程序崩溃

## 解决方案

修改 `vinput-gui/src/config.rs` 的 `load()` 方法，首次启动时自动创建配置文件。

### 配置文件创建流程

```
启动 vinput-settings
    ↓
检查 ~/.config/vinput/config.toml 是否存在
    ↓
    ├─ 存在 → 直接加载
    │
    └─ 不存在
        ↓
        检查 /usr/share/droplet-voice-input/config.toml.example
        ↓
        ├─ 存在 → 复制为用户配置
        │
        └─ 不存在 → 使用默认配置并保存
```

### 代码修改

**修改前**：
```rust
pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
    let path = Self::config_path();
    if !path.exists() {
        return Ok(Self::default());  // ❌ 只返回默认值，不保存
    }
    // ...
}
```

**修改后**：
```rust
pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
    let path = Self::config_path();

    if !path.exists() {
        // 尝试从系统示例文件复制
        let example_path = PathBuf::from("/usr/share/droplet-voice-input/config.toml.example");
        if example_path.exists() {
            // 确保目录存在
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 复制示例文件
            fs::copy(&example_path, &path)?;
        } else {
            // 使用默认配置并保存
            let default_config = Self::default();
            default_config.save()?;  // ✅ 自动保存
            return Ok(default_config);
        }
    }

    // 读取配置文件
    let content = fs::read_to_string(&path)?;
    let config: VInputConfig = toml::from_str(&content)?;
    Ok(config)
}
```

## 默认配置更新

同时更新了默认配置，使用正确的系统路径和最新的参数：

### 1. 模型路径
```rust
// 修改前
model_dir: "/home/deepin/deepin-v2t/models/streaming"  // ❌ 开发路径

// 修改后
model_dir: "/usr/share/droplet-voice-input/models"     // ✅ 系统路径
```

### 2. VAD 参数
```rust
// 修改前
start_threshold: 0.5,
min_silence_duration: 300,

// 修改后
start_threshold: 0.7,           // 减少背景噪音
min_silence_duration: 700,      // 防止末尾字丢失
```

### 3. 端点检测参数
```rust
// 修改前
trailing_silence_ms: 800,
vad_silence_confirm_frames: 5,

// 修改后
trailing_silence_ms: 1000,      // 更长的等待时间
vad_silence_confirm_frames: 8,  // 更多的确认帧
```

## 用户体验改进

### 修复前
1. 首次启动 vinput-settings
2. 修改任何设置
3. 点击"保存" → ❌ 出错或崩溃

### 修复后
1. 首次启动 vinput-settings
2. 自动创建 `~/.config/vinput/config.toml`
3. 修改任何设置
4. 点击"保存" → ✅ 正常保存

## 配置文件位置

- **用户配置**：`~/.config/vinput/config.toml`
- **系统示例**：`/usr/share/droplet-voice-input/config.toml.example`

## 日志输出

首次启动时会看到以下日志：

```
配置文件不存在，尝试从示例文件创建: /home/user/.config/vinput/config.toml
从系统示例文件复制: /usr/share/droplet-voice-input/config.toml.example
配置文件创建成功: /home/user/.config/vinput/config.toml
配置加载成功: /home/user/.config/vinput/config.toml
```

或者（如果示例文件不存在）：

```
配置文件不存在，尝试从示例文件创建: /home/user/.config/vinput/config.toml
示例文件不存在，使用默认配置并保存
默认配置已保存: /home/user/.config/vinput/config.toml
```

## 测试方法

### 测试 1：首次启动
```bash
# 删除配置文件
rm -f ~/.config/vinput/config.toml

# 启动设置程序
vinput-settings

# 检查配置文件是否自动创建
ls -la ~/.config/vinput/config.toml
cat ~/.config/vinput/config.toml
```

### 测试 2：修改设置
```bash
# 启动设置程序
vinput-settings

# 修改任何设置（如热词、标点等）
# 点击"保存"按钮
# 应该正常保存，不会出错
```

### 测试 3：配置持久化
```bash
# 修改设置并保存
vinput-settings

# 关闭程序
# 重新打开
vinput-settings

# 检查设置是否保留
```

## 已修复

- ✅ 首次启动自动创建配置文件
- ✅ 优先从系统示例文件复制
- ✅ 示例文件不存在时使用默认配置
- ✅ 更新默认配置为正确的系统路径
- ✅ 更新默认 VAD 参数
- ✅ 重新编译和打包

新的 DEB 包：`droplet-voice-input_0.1.0_amd64.deb` (2026-02-18)
