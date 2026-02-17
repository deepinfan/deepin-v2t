#!/bin/bash
# 测试标点修复

echo "🔧 安装新版本..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

echo "🚀 启动 fcitx5（DEBUG 模式）..."
echo ""
echo "⚠️  测试说明："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说话示例（词之间停顿 0.5-1 秒）："
echo "      今天 [停] 天气 [停] 很好 [停] 我想 [停] 出去 [停] 散步"
echo "   4. 松开空格停止录音"
echo "   5. 观察识别结果"
echo "   6. 按 Ctrl+C 停止"
echo ""
echo "📊 预期结果："
echo "   - Preedit 显示实时识别（无标点）"
echo "   - 最终上屏文本包含逗号和句号"
echo "   - 不再有增量上屏"
echo ""
echo "开始监控..."
echo "============"
echo ""

# 启动 fcitx5 并过滤关键日志
VINPUT_LOG=debug fcitx5 2>&1 | tee /tmp/vinput-punctuation-fix.log | \
    grep -E "(Preedit|上屏|Token\[|停顿检测|检测到停顿|最终结果)" --color=always
