#!/bin/bash

# 清理重复和旧的 Fcitx5 配置

set -e

echo "=== 清理 Fcitx5 重复配置 ==="
echo ""

echo "📋 当前配置文件："
echo ""
echo "✅ 正确的配置（保留）："
echo "  /usr/share/fcitx5/addon/vinput.conf"
echo "  /usr/share/fcitx5/inputmethod/vinput-im.conf"
echo ""
echo "❌ 重复/旧的配置（删除）："
echo "  /usr/local/share/fcitx5/addon/vinput-addon.conf"
echo "  /usr/local/share/fcitx5/inputmethod/vinput.conf"
echo "  /usr/local/share/fcitx5/inputmethod/vocotype-deepin.conf"
echo "  /usr/local/share/fcitx5/addon/vocotype.conf"
echo ""

read -p "确认删除这些文件？(y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "取消操作"
    exit 1
fi

echo ""
echo "🗑️  删除重复配置..."

# 删除 /usr/local 下的重复配置
sudo rm -f /usr/local/share/fcitx5/addon/vinput-addon.conf
sudo rm -f /usr/local/share/fcitx5/inputmethod/vinput.conf

# 删除旧的 vocotype/deepin 配置
sudo rm -f /usr/local/share/fcitx5/inputmethod/vocotype-deepin.conf
sudo rm -f /usr/local/share/fcitx5/addon/vocotype.conf

echo "✅ 配置文件已清理"
echo ""

echo "🔄 重启 Fcitx5..."
fcitx5 -r
sleep 2

echo ""
echo "✅ 完成！"
echo ""
echo "现在 Fcitx5 输入法列表中应该只有一个 V-Input"
echo "旧的 \"语音输入-Deepin\" 也已经移除"
echo ""
echo "请检查："
echo "  右键 Fcitx5 托盘图标 → 配置 → 输入法"
echo ""
