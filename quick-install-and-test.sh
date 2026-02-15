#!/bin/bash
# V-Input 快速安装和测试脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "V-Input 快速安装和测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}步骤 1/5: 编译 Release 版本${NC}"
echo "----------------------------------------"

# 编译核心库
echo "编译 vinput-core (release)..."
cd "$SCRIPT_DIR/vinput-core"
cargo build --release --quiet
echo -e "${GREEN}✓ vinput-core 编译完成${NC}"

# 编译 Fcitx5 插件
echo "编译 Fcitx5 插件..."
cd "$SCRIPT_DIR/fcitx5-vinput/build"
cmake .. > /dev/null 2>&1
make --quiet
echo -e "${GREEN}✓ Fcitx5 插件编译完成${NC}"

cd "$SCRIPT_DIR"
echo ""

echo -e "${YELLOW}步骤 2/5: 安装插件${NC}"
echo "----------------------------------------"
echo "需要 root 权限安装系统文件"
echo "请输入密码..."
sudo "$SCRIPT_DIR/install-fcitx5-plugin.sh"
echo ""

echo -e "${YELLOW}步骤 3/5: 重启 Fcitx5${NC}"
echo "----------------------------------------"
echo "正在重启 Fcitx5..."
fcitx5 -r
sleep 2
echo -e "${GREEN}✓ Fcitx5 已重启${NC}"
echo ""

echo -e "${YELLOW}步骤 4/5: 配置检查${NC}"
echo "----------------------------------------"

# 检查插件是否加载
if fcitx5-remote -a | grep -q vinput; then
    echo -e "${GREEN}✓ V-Input 插件已加载${NC}"
else
    echo -e "${RED}✗ V-Input 插件未加载${NC}"
    echo "请手动添加 V-Input 输入法："
    echo "1. 运行: fcitx5-configtool"
    echo "2. 添加 'V-Input' 输入法"
fi

# 检查配置文件
if [ -f "$HOME/.config/vinput/config.toml" ] || [ -f "/etc/vinput/config.toml" ]; then
    echo -e "${GREEN}✓ 配置文件存在${NC}"
else
    echo -e "${YELLOW}! 配置文件不存在，将使用默认配置${NC}"
fi

# 检查模型文件
if [ -f "/usr/share/vinput/models/silero-vad/silero_vad.onnx" ]; then
    echo -e "${GREEN}✓ VAD 模型文件存在${NC}"
else
    echo -e "${RED}✗ VAD 模型文件缺失${NC}"
fi

if [ -f "/usr/share/vinput/models/streaming/encoder-epoch-99-avg-1.onnx" ]; then
    echo -e "${GREEN}✓ ASR 模型文件存在${NC}"
else
    echo -e "${RED}✗ ASR 模型文件缺失${NC}"
fi

echo ""

echo -e "${YELLOW}步骤 5/5: 使用说明${NC}"
echo "----------------------------------------"
echo ""
echo "V-Input 已安装完成！"
echo ""
echo "使用方法："
echo "1. 切换到 V-Input 输入法"
echo "2. 打开任意文本编辑器"
echo "3. 按下空格键开始录音（看到 🎤 录音中...）"
echo "4. 说话"
echo "5. 再次按下空格键停止录音"
echo "6. 等待识别结果自动上屏"
echo ""
echo "快捷键："
echo "- 空格键: 开始/停止录音"
echo "- Ctrl+Z: 撤销"
echo "- Ctrl+Y: 重试"
echo ""
echo "测试示例："
echo "- 说 '今天天气很好' → 应该输出 '今天天气很好。'"
echo "- 说 '我花了三百块钱' → 应该输出 '我花了¥300'"
echo "- 说 '今天是二零二六年三月五日' → 应该输出 '今天是2026年3月5日。'"
echo ""
echo "配置工具："
echo "- 运行 GUI 设置: ./run-settings.sh"
echo "- Fcitx5 配置: fcitx5-configtool"
echo ""
echo "查看日志："
echo "- journalctl --user -u fcitx5 -f"
echo ""
echo "详细测试指南："
echo "- 查看 TESTING_GUIDE.md"
echo ""
echo -e "${GREEN}=========================================="
echo "安装完成！开始测试吧！"
echo "==========================================${NC}"
echo ""
