# 水滴语音输入法 - 安装排错指南

## 问题：在 Fcitx5 配置中找不到水滴语音输入法

### 快速检查步骤

#### 1. 运行自动检查脚本

```bash
./check-installation.sh
```

这个脚本会自动检查所有关键文件和配置。

#### 2. 手动检查关键文件

```bash
# 检查包是否已安装
dpkg -l | grep droplet-voice-input

# 检查 Fcitx5 配置文件
ls -la /usr/share/fcitx5/addon/vinput-addon.conf
ls -la /usr/share/fcitx5/inputmethod/vinput.conf

# 检查插件文件
ls -la /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

# 检查核心库
ls -la /usr/lib/x86_64-linux-gnu/libvinput_core.so
```

#### 3. 重启 Fcitx5

```bash
# 方法 1: 快速重启
fcitx5 -r

# 方法 2: 完全重启
pkill fcitx5
sleep 2
fcitx5 &

# 方法 3: 重新登录（最彻底）
# 注销并重新登录系统
```

#### 4. 更新动态链接库缓存

```bash
sudo ldconfig
```

#### 5. 检查 Fcitx5 日志

```bash
# 启动 Fcitx5 并查看详细日志
pkill fcitx5
fcitx5 --verbose=10 2>&1 | tee /tmp/fcitx5-debug.log

# 在另一个终端查看日志
tail -f /tmp/fcitx5-debug.log | grep -i vinput
```

---

## 常见问题和解决方案

### Q1: 找不到输入法

**症状**: 在 Fcitx5 配置的"添加输入法"列表中找不到"水滴语音输入法"

**可能原因**:
1. Fcitx5 未重启
2. 配置文件权限问题
3. 插件加载失败

**解决方案**:

```bash
# 1. 检查配置文件是否存在
ls -la /usr/share/fcitx5/addon/vinput-addon.conf
ls -la /usr/share/fcitx5/inputmethod/vinput.conf

# 2. 检查文件内容
cat /usr/share/fcitx5/addon/vinput-addon.conf
cat /usr/share/fcitx5/inputmethod/vinput.conf

# 3. 重启 Fcitx5
fcitx5 -r

# 4. 如果还是不行，完全重启
pkill fcitx5
sudo ldconfig
fcitx5 &
```

---

### Q2: 插件加载失败

**症状**: Fcitx5 日志中显示插件加载错误

**检查步骤**:

```bash
# 1. 检查插件文件
ls -la /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

# 2. 检查插件依赖
ldd /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

# 3. 查找缺失的库
ldd /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so | grep "not found"
```

**常见缺失依赖**:

```bash
# 如果缺少 libvinput_core.so
sudo ldconfig
ldconfig -p | grep libvinput_core

# 如果缺少 libsherpa-onnx-c-api.so
ldconfig -p | grep libsherpa-onnx-c-api

# 如果缺少 libonnxruntime.so
ldconfig -p | grep libonnxruntime
```

---

### Q3: Addon 配置问题

**症状**: vinput-addon.conf 存在但 Fcitx5 不识别

**检查 addon 配置**:

```bash
cat /usr/share/fcitx5/addon/vinput-addon.conf
```

应该包含:
```ini
[Addon]
Name=水滴语音输入法
Category=InputMethod
Version=0.1.0
Type=SharedLibrary
OnDemand=False
Library=vinput

[Addon/OptionalDependencies]
0=notifications
```

**关键点**:
- `Library=vinput` 必须匹配插件文件名 `vinput.so`
- `OnDemand=False` 表示启动时加载

---

### Q4: 输入法配置问题

**检查输入法配置**:

```bash
cat /usr/share/fcitx5/inputmethod/vinput.conf
```

应该包含:
```ini
[InputMethod]
Name=水滴语音输入法
Icon=audio-input-microphone
Label=语
LangCode=zh_CN
Addon=vinput
Configurable=False
```

**关键点**:
- `Addon=vinput` 必须匹配 addon 名称
- `LangCode=zh_CN` 指定语言

---

### Q5: 权限问题

**检查文件权限**:

```bash
# 检查配置文件权限（应该是 644）
ls -la /usr/share/fcitx5/addon/vinput-addon.conf
ls -la /usr/share/fcitx5/inputmethod/vinput.conf

# 检查插件权限（应该是 644）
ls -la /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

# 检查库文件权限（应该是 644）
ls -la /usr/lib/x86_64-linux-gnu/libvinput_core.so
```

**修复权限**:

```bash
sudo chmod 644 /usr/share/fcitx5/addon/vinput-addon.conf
sudo chmod 644 /usr/share/fcitx5/inputmethod/vinput.conf
sudo chmod 644 /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so
sudo chmod 644 /usr/lib/x86_64-linux-gnu/libvinput_core.so
```

---

### Q6: 模型文件问题

**检查模型文件**:

```bash
ls -lh /usr/share/droplet-voice-input/models/

# 应该看到:
# encoder.int8.onnx (158M)
# decoder.int8.onnx (69M)
# tokens.txt (74K)
```

**验证文件完整性**:

```bash
# 检查文件大小
du -h /usr/share/droplet-voice-input/models/encoder.int8.onnx
du -h /usr/share/droplet-voice-input/models/decoder.int8.onnx

# 如果文件损坏，重新安装
sudo apt-get install --reinstall droplet-voice-input
```

---

## 高级排错

### 使用 fcitx5-diagnose

```bash
fcitx5-diagnose | grep -A 10 -B 10 vinput
```

### 查看系统日志

```bash
# 用户日志
journalctl --user -u fcitx5 -n 100 | grep -i vinput

# 系统日志
dmesg | grep -i vinput
```

### 手动测试插件

```bash
# 检查符号
nm -D /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so | grep -i fcitx

# 检查依赖
objdump -p /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so | grep NEEDED
```

---

## 完全重新安装

如果以上方法都不行，尝试完全重新安装:

```bash
# 1. 完全卸载
sudo apt-get purge droplet-voice-input

# 2. 清理残留文件
sudo rm -rf /usr/share/droplet-voice-input
sudo rm -f /usr/lib/x86_64-linux-gnu/libvinput_core.so
sudo rm -f /usr/lib/x86_64-linux-gnu/libsherpa-onnx-c-api.so
sudo rm -f /usr/lib/x86_64-linux-gnu/libonnxruntime.so
sudo rm -f /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so
sudo rm -f /usr/share/fcitx5/addon/vinput-addon.conf
sudo rm -f /usr/share/fcitx5/inputmethod/vinput.conf

# 3. 更新库缓存
sudo ldconfig

# 4. 重新安装
sudo dpkg -i droplet-voice-input_0.1.0_amd64.deb

# 5. 再次更新库缓存
sudo ldconfig

# 6. 重启 Fcitx5
pkill fcitx5
sleep 2
fcitx5 &

# 7. 等待几秒后检查
sleep 5
fcitx5-remote -l
```

---

## 收集诊断信息

如果问题仍然存在，请收集以下信息并发布到论坛:

```bash
# 1. 系统信息
cat /etc/os-release
uname -a

# 2. Fcitx5 版本
fcitx5 --version

# 3. 包信息
dpkg -l | grep droplet-voice-input
dpkg -l | grep fcitx5

# 4. 文件检查
ls -la /usr/share/fcitx5/addon/vinput-addon.conf
ls -la /usr/share/fcitx5/inputmethod/vinput.conf
ls -la /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

# 5. 依赖检查
ldd /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

# 6. Fcitx5 日志
journalctl --user -u fcitx5 --no-pager | tail -100

# 7. 运行检查脚本
./check-installation.sh > /tmp/check-result.txt 2>&1
cat /tmp/check-result.txt
```

---

## 联系支持

首发于深度操作系统论坛: http://bbs.deepin.org

发帖时请提供:
1. 系统版本和 Fcitx5 版本
2. `check-installation.sh` 的完整输出
3. Fcitx5 日志（最后 100 行）
4. 问题的详细描述和截图
