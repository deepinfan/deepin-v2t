#!/bin/bash

# 修复安装路径问题

set -e

echo "=== 修复插件安装路径 ==="
echo ""

cd /home/deepin/deepin-v2t/fcitx5-vinput/build

echo "📋 当前插件信息："
ls -lh vinput.so
echo ""

echo "📥 复制插件到正确位置..."
sudo cp vinput.so /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

echo ""
echo "✅ 插件已更新："
ls -lh /usr/lib/x86_64-linux-gnu/fcitx5/vinput.so

echo ""
echo "🔄 重启 Fcitx5..."
fcitx5 -r
sleep 2

echo ""
echo "✅ 完成！现在应该可以正常工作了。"
echo ""
echo "测试命令："
echo "  说 \"今天天气很好\" → 应该看到流式上屏"
echo "  说 \"三百块钱\" → 应该看到 Preedit 预览"
echo ""
