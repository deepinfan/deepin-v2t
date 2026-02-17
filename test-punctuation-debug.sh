#!/bin/bash
# 标点引擎完整测试脚本（启用调试日志）

echo "🔍 标点引擎完整测试"
echo "===================="
echo ""

# 1. 安装新编译的库
echo "📦 安装调试版本库..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig
echo "✅ 安装完成"
echo ""

# 2. 检查配置
echo "📋 当前标点配置:"
cat ~/.config/vinput/config.toml | grep -A 6 "\[punctuation\]"
echo ""

# 3. 停止 fcitx5
echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

# 4. 启动 fcitx5（启用调试日志）
echo "🚀 启动 fcitx5（调试模式）..."
echo ""
echo "⚠️  测试说明："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说话示例（词之间停顿 1 秒）："
echo "      今天 [停1秒] 天气 [停1秒] 很好 [停1秒] 我想 [停1秒] 出去 [停1秒] 散步"
echo "   4. 松开空格停止录音"
echo "   5. 观察识别结果和日志"
echo "   6. 按 Ctrl+C 停止"
echo ""
echo "📊 关键日志标记："
echo "   - '标点配置' - 配置加载情况"
echo "   - 'Token[N]' - Token 和时间戳"
echo "   - '停顿检测' - Token 时长分析"
echo "   - '检测到停顿' - 逗号插入决策"
echo ""
echo "开始监控日志..."
echo "================"
echo ""

# 启动 fcitx5 并过滤日志
VINPUT_LOG=debug fcitx5 2>&1 | tee /tmp/vinput-debug-full.log | \
    grep -E "(标点配置|pause_ratio|min_tokens|Token\[|停顿检测|检测到停顿|处理 Token|句尾标点|原始 timestamps)" --color=always
