#!/bin/bash
# V-Input 调试脚本

echo "🔍 V-Input 调试信息收集"
echo "========================"
echo ""

# 1. 检查模型文件
echo "📁 模型文件检查:"
ls -lh /home/deepin/deepin-v2t/models/streaming/*.onnx 2>/dev/null | awk '{print $9, $5}'
ls -lh /home/deepin/deepin-v2t/models/streaming/bpe.* 2>/dev/null | awk '{print $9, $5}'
echo ""

# 2. 检查已安装库
echo "📦 已安装库检查:"
ls -lh /usr/local/lib/libvinput_core.so 2>/dev/null | awk '{print $9, $5, $6, $7}'
echo ""

# 3. 检查库使用的模型名称
echo "🔧 库使用的模型文件名:"
strings /usr/local/lib/libvinput_core.so 2>/dev/null | grep -E "epoch-99.*\.onnx" | head -5
echo ""

# 4. 检查配置文件
echo "⚙️  当前配置:"
cat ~/.config/vinput/config.toml
echo ""

# 5. 停止 fcitx5
echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

# 6. 启动 fcitx5 并收集日志
echo "🚀 启动 fcitx5 (调试模式)..."
echo "   日志将保存到: /tmp/vinput-debug.log"
echo ""
echo "⚠️  请执行以下操作："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说一段话（例如：今天天气很好，我想出去散步）"
echo "   4. 松开空格停止录音"
echo "   5. 观察识别结果"
echo "   6. 按 Ctrl+C 停止日志收集"
echo ""

RUST_LOG=vinput_core=debug fcitx5 2>&1 | tee /tmp/vinput-debug.log | grep -E "(Token|逗号|停顿|标点|重复|ASR|模型|加载|识别)"
