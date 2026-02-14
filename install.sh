#!/bin/bash
# V-Input Fcitx5 安装脚本

set -e

echo "=== V-Input Fcitx5 安装脚本 ==="
echo ""

# 检查权限
if [ "$EUID" -ne 0 ]; then
    echo "请使用 sudo 运行此脚本"
    exit 1
fi

# 编译 vinput-core
echo "1. 编译 vinput-core..."
cd vinput-core
cargo build --release
cd ..

# 编译 fcitx5-vinput
echo "2. 编译 fcitx5-vinput..."
cd fcitx5-vinput
mkdir -p build
cd build
cmake ..
make
cd ../..

# 安装库文件
echo "3. 安装库文件..."
install -Dm755 vinput-core/target/release/libvinput_core.so /usr/lib/libvinput_core.so
install -Dm755 fcitx5-vinput/build/libvinput.so /usr/lib/fcitx5/vinput.so

# 安装配置文件
echo "4. 安装配置文件..."
install -Dm644 fcitx5-vinput/vinput.conf /usr/share/fcitx5/inputmethod/vinput.conf
install -Dm644 fcitx5-vinput/vinput-addon.conf /usr/share/fcitx5/addon/vinput.conf

# 创建配置目录
echo "5. 创建用户配置目录..."
mkdir -p /etc/vinput
mkdir -p ~/.config/vinput

echo ""
echo "✅ V-Input 安装完成！"
echo ""
echo "使用方法:"
echo "1. 重启 Fcitx5: fcitx5 -r"
echo "2. 在 Fcitx5 配置中添加 V-Input 输入法"
echo "3. 按 Ctrl+Space 长按开始语音输入"
echo ""
