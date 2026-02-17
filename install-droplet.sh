#!/bin/bash
# 水滴语音输入法 - 完整安装脚本

echo "💧 水滴语音输入法 - 安装程序"
echo "================================"
echo ""

# 1. 安装核心库
echo "📦 安装核心库..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig
echo "✅ 核心库安装完成"
echo ""

# 2. 安装 Fcitx5 插件
echo "📦 安装 Fcitx5 插件..."
sudo cp /home/deepin/deepin-v2t/fcitx5-vinput/build/vinput.so /usr/lib/x86_64-linux-gnu/fcitx5/
sudo cp /home/deepin/deepin-v2t/fcitx5-vinput/vinput.conf /usr/share/fcitx5/inputmethod/
sudo cp /home/deepin/deepin-v2t/fcitx5-vinput/vinput-addon.conf /usr/share/fcitx5/addon/
echo "✅ Fcitx5 插件安装完成"
echo ""

# 3. 安装设置程序
echo "📦 安装设置程序..."
sudo cp /home/deepin/deepin-v2t/target/release/vinput-settings /usr/local/bin/
echo "✅ 设置程序安装完成"
echo ""

# 4. 重启 Fcitx5
echo "🔄 重启 Fcitx5..."
fcitx5 -r
sleep 2
echo "✅ Fcitx5 重启完成"
echo ""

echo "================================"
echo "✅ 安装完成！"
echo ""
echo "使用说明："
echo "  1. 切换到「水滴语音输入法」"
echo "  2. 按空格开始录音"
echo "  3. 说话后松开空格"
echo "  4. 运行 vinput-settings 打开设置界面"
echo ""
echo "首发于深度操作系统论坛: http://bbs.deepin.org"
echo ""
