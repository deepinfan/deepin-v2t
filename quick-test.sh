#!/bin/bash
# 快速测试脚本

echo "🔧 重新安装..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

echo "🚀 启动 fcitx5..."
echo ""
echo "请测试语音输入，观察："
echo "1. 配置是否正确加载（pause_ratio=2.0, min_tokens=3）"
echo "2. 逗号是否正常插入"
echo "3. 是否还有重复字符"
echo ""

VINPUT_LOG=info fcitx5 2>&1 | tee /tmp/vinput-test.log | \
    grep -E "(标点配置|pause_ratio|加载配置)" --color=always
