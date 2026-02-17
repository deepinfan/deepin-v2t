# Paraformer 模型调试步骤

## 当前状态

已添加详细的调试日志来追踪 Paraformer 模型的识别流程。

## 测试步骤

### 1. 安装新版本

```bash
cd /home/deepin/deepin-v2t
./install-and-test-paraformer.sh
```

这个脚本会：
- 安装新编译的库
- 停止 fcitx5
- 检查模型文件
- 启动 fcitx5 并显示详细日志

### 2. 测试语音识别

1. 切换到 V-Input 输入法
2. 按空格开始录音
3. 说话：**今天天气很好**
4. 松开空格停止录音
5. 观察日志输出

### 3. 关键日志标记

查找以下日志来诊断问题：

#### 模型加载
```
🔍 加载 Paraformer 模型:
  Encoder: ...
  Decoder: ...
  Tokens: ...
```

#### ASR 流创建
```
✅ ASR 流创建成功
```

#### 音频数据送入
```
✅ 注入 Pre-roll 音频: XXX 样本
🎤 已送入 XX 帧音频到 ASR (每帧 512 样本)
```

#### Sherpa-ONNX 原始结果
```
🔍 Sherpa-ONNX 原始结果:
  - text: '...'
  - count: X
  - tokens_arr.is_null(): false
  - timestamps.is_null(): false
```

#### 最终识别结果
```
📊 ASR 识别结果详情:
  - text: '...'
  - text.len(): X
  - token_count: X
  - is_empty(): false
```

## 预期问题诊断

### 情况 1: 模型加载失败
如果看到 "ModelLoad error"，说明模型文件路径或格式有问题。

### 情况 2: ASR 流创建失败
如果没有看到 "✅ ASR 流创建成功"，说明 Sherpa-ONNX 无法创建流。

### 情况 3: 没有音频数据
如果没有看到 "🎤 已送入 XX 帧音频"，说明音频没有被送入 ASR。

### 情况 4: Sherpa-ONNX 返回空结果
如果看到：
```
🔍 Sherpa-ONNX 原始结果:
  - text: ''
  - count: 0
```
说明 Paraformer 模型没有识别出任何内容。

可能原因：
1. 模型配置不正确（sample_rate, feature_dim 等）
2. 音频格式不匹配
3. Paraformer 模型需要特殊的配置参数

## 下一步

根据日志输出，我们可以确定问题出在哪个环节：
1. 模型加载
2. ASR 流创建
3. 音频数据送入
4. Sherpa-ONNX 识别

请运行测试脚本并提供完整的日志输出。
