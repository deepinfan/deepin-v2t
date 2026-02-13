# Fcitx5 V-Input 插件

## Phase 0: 插件骨架 + FFI 集成验证

V-Input 的 Fcitx5 输入法引擎插件。

### 文件结构

```
fcitx5-vinput-mvp/
├── CMakeLists.txt              # CMake 构建配置
├── build.sh                    # 构建脚本
├── vinput.conf                 # 插件配置
├── vinput-im.conf              # 输入法配置
├── test_fcitx5_ffi.cpp         # FFI 集成测试
├── src/
│   ├── vinput_engine.h         # 引擎头文件
│   └── vinput_engine.cpp       # 引擎实现
```

### 依赖要求

1. **Fcitx5 开发库**（仅 Phase 1 构建需要）
   ```bash
   # Debian/Deepin/Ubuntu
   sudo apt install fcitx5-dev libfcitx5core-dev

   # Fedora
   sudo dnf install fcitx5-devel
   ```

2. **vinput-core 库**（必须）
   ```bash
   cd ../vinput-core
   cargo build --release
   ```

### Phase 0 验证（无需 Fcitx5）

测试 C++ 和 FFI 接口集成：

```bash
# 编译测试程序
g++ -o test_fcitx5_ffi test_fcitx5_ffi.cpp \
    -I../target -L../target/release -lvinput_core \
    -Wl,-rpath,../target/release -std=c++17

# 运行测试
./test_fcitx5_ffi
```

**测试结果：**
```
✓ 初始化成功
✓ 事件发送成功
✓ 命令接收正常
✓ 音频数据发送成功
✓ 关闭成功
```

### Phase 1 构建（需要 Fcitx5）

```bash
# 运行构建脚本
chmod +x build.sh
./build.sh

# 安装插件
cd build
sudo make install

# 重启 Fcitx5
fcitx5 -r
```

### 实现状态

#### ✅ Phase 0（已完成）
- [x] Fcitx5 插件骨架
- [x] FFI 接口集成
- [x] 基本生命周期管理
- [x] C++ FFI 调用验证

#### 🔄 Phase 1（待实现）
- [ ] 音频捕获集成
- [ ] VAD 状态监控
- [ ] ASR 识别结果处理
- [ ] 候选词显示
- [ ] 完整按键处理
- [ ] 实际 Fcitx5 运行测试

### 架构说明

```
Fcitx5 引擎 (C++)
    ↓
FFI 接口 (vinput_core.h)
    ↓
vinput-core (Rust)
    ├── PipeWire 音频捕获
    ├── Silero VAD 检测
    └── sherpa-onnx ASR 识别
```

### Phase 0 验证结论

✅ **FFI 集成验证成功**
- C++ 可以正确调用 Rust FFI 函数
- 类型定义完全兼容
- 生命周期管理正常
- 插件骨架结构合理

**下一步：** Phase 1 实现完整语音识别流程。
