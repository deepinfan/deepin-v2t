#!/bin/bash

# 快速重新安装脚本（修复 Unknown command type 问题）

set -e

echo "=== 修复混合模式命令类型问题 ==="
echo ""

cd /home/deepin/deepin-v2t/fcitx5-vinput/build

echo "📥 安装 Fcitx5 插件..."
sudo make install

echo ""
echo "🔄 重启 Fcitx5..."
fcitx5 -r
sleep 2

echo ""
echo "✅ 安装完成！"
echo ""
echo "问题修复："
echo "- UpdatePreedit (命令类型 7) 现在应该可以正常工作"
echo "- ClearPreedit (命令类型 8) 现在应该可以正常工作"
echo ""
echo "请重新测试："
echo "  说 \"今天天气很好\" 或 \"三百块钱\""
echo ""
