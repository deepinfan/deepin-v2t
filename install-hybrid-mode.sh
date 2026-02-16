#!/bin/bash

# 混合模式安装脚本

set -e

echo "=== 混合模式流式上屏 - 安装脚本 ==="
echo ""

# 1. 编译 Rust 核心库
echo "📦 步骤 1/4: 编译 Rust 核心库..."
cd vinput-core
cargo build --release
echo "✅ Rust 核心库编译完成"
echo ""

# 2. 编译 Fcitx5 插件
echo "🔧 步骤 2/4: 编译 Fcitx5 插件..."
cd ../fcitx5-vinput
mkdir -p build
cd build
cmake ..
make
echo "✅ Fcitx5 插件编译完成"
echo ""

# 3. 安装插件
echo "📥 步骤 3/4: 安装 Fcitx5 插件..."
echo "需要 sudo 权限..."
sudo make install
echo "✅ 插件安装完成"
echo ""

# 4. 重启 Fcitx5
echo "🔄 步骤 4/4: 重启 Fcitx5..."
fcitx5 -r
sleep 2
echo "✅ Fcitx5 已重启"
echo ""

echo "=== 安装完成 ==="
echo ""
echo "测试方法："
echo "1. 按空格键开始录音"
echo "2. 说话测试："
echo "   - 普通文本：\"今天天气很好\" → 观察流式上屏"
echo "   - 包含数字：\"三百块钱\" → 观察 Preedit 预览"
echo "   - 混合文本：\"今天买了三百块\" → 观察混合效果"
echo "3. 再次按空格键停止录音"
echo ""
echo "预期效果："
echo "- 普通文本：边说边上屏（黑色），最后 2 个字在 Preedit（灰色）"
echo "- 包含数字：全部在 Preedit（灰色），句子结束时上屏 ITN 结果"
echo "- 混合文本：部分上屏 + 部分 Preedit"
echo ""
echo "日志查看："
echo "  journalctl --user -u fcitx5 -f"
echo ""
