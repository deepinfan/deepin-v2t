#!/bin/bash
# Paraformer 详细调试

echo "🔧 安装新版本..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

echo "📊 检查模型文件..."
ls -lh /home/deepin/deepin-v2t/models/streaming/

echo ""
echo "🚀 启动 fcitx5（DEBUG 模式）..."
echo ""
echo "⚠️  测试说明："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说话: 今天天气很好"
echo "   4. 松开空格停止录音"
echo "   5. 观察日志"
echo ""
echo "开始监控..."
echo "============"
echo ""

# 启动 fcitx5 并显示所有日志
VINPUT_LOG=debug fcitx5 2>&1 | tee /tmp/vinput-paraformer-debug.log | \
    grep -E "(模型|ASR|识别|Token|Paraformer|encoder|decoder)" --color=always
