#!/bin/bash

# 完整重新安装脚本 - 修复重复配置问题

set -e

echo "=== V-Input 完整重新安装 ==="
echo ""

cd /home/deepin/deepin-v2t

# 1. 清理旧配置
echo "🗑️  步骤 1/5: 清理旧配置..."
sudo rm -f /usr/local/share/fcitx5/addon/vinput*.conf
sudo rm -f /usr/local/share/fcitx5/inputmethod/vinput*.conf
sudo rm -f /usr/local/share/fcitx5/inputmethod/vocotype*.conf
sudo rm -f /usr/local/share/fcitx5/addon/vocotype*.conf
echo "✅ 旧配置已清理"
echo ""

# 2. 编译 Rust 核心库
echo "📦 步骤 2/5: 编译 Rust 核心库..."
cd vinput-core
cargo build --release
echo "✅ Rust 核心库编译完成"
echo ""

# 3. 重新编译 Fcitx5 插件
echo "🔧 步骤 3/5: 重新编译 Fcitx5 插件..."
cd ../fcitx5-vinput/build
rm -rf *
cmake ..
make
echo "✅ Fcitx5 插件编译完成"
echo ""

# 4. 安装
echo "📥 步骤 4/5: 安装插件和配置..."
sudo make install
echo "✅ 安装完成"
echo ""

# 5. 重启 Fcitx5
echo "🔄 步骤 5/5: 重启 Fcitx5..."
fcitx5 -r
sleep 2
echo "✅ Fcitx5 已重启"
echo ""

echo "=== 安装完成 ==="
echo ""
echo "✅ 修复内容："
echo "  - 统一配置文件名，避免重复"
echo "  - 强制安装到 /usr 而不是 /usr/local"
echo "  - 清理所有旧配置"
echo ""
echo "现在 Fcitx5 输入法列表中应该只有一个 V-Input"
echo ""
echo "测试方法："
echo "  1. 右键 Fcitx5 托盘图标 → 配置 → 输入法"
echo "  2. 确认只有一个 V-Input"
echo "  3. 添加 V-Input 到输入法列表"
echo "  4. 测试语音输入"
echo ""
