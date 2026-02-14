#!/bin/bash
# V-Input Fcitx5 卸载脚本

set -e

echo "=== V-Input Fcitx5 卸载脚本 ==="
echo ""

# 检查权限
if [ "$EUID" -ne 0 ]; then
    echo "请使用 sudo 运行此脚本"
    exit 1
fi

# 删除库文件
echo "1. 删除库文件..."
rm -f /usr/lib/libvinput_core.so
rm -f /usr/lib/fcitx5/vinput.so

# 删除配置文件
echo "2. 删除配置文件..."
rm -f /usr/share/fcitx5/inputmethod/vinput.conf
rm -f /usr/share/fcitx5/addon/vinput.conf

echo ""
echo "✅ V-Input 卸载完成！"
echo ""
echo "注意: 用户配置目录 ~/.config/vinput 已保留"
echo "如需完全删除，请手动运行: rm -rf ~/.config/vinput"
echo ""
