#!/bin/bash
# 快速安装和重启测试

echo "🔧 安装新版本..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🔄 重启 fcitx5..."
fcitx5 -r

echo "✅ 安装完成，请测试语音识别"
