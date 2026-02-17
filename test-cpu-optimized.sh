#!/bin/bash
# 测试 CPU 优化（INT8 + blank_penalty + 节流）

echo "🔧 安装新版本（激进 CPU 优化）..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

echo "📊 优化参数："
echo "   ✅ 模型: INT8 量化（encoder=174MB, decoder=13MB, joiner=3.1MB）"
echo "   ✅ blank_penalty: 2.5（解决重复字符问题）"
echo "   ✅ num_threads: 1（最小化 CPU 占用）"
echo "   ✅ max_active_paths: 2（降低搜索空间）"
echo "   ✅ Preedit 更新节流: 每 5 帧（~160ms）更新一次"
echo ""

echo "🚀 启动 fcitx5..."
echo ""
echo "⚠️  测试说明："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说话示例（词之间停顿 0.5-1 秒）："
echo "      今天 [停] 天气 [停] 很好 [停] 我想 [停] 出去 [停] 散步"
echo "   4. 松开空格停止录音"
echo "   5. 观察识别结果和 CPU 占用"
echo "   6. 按 Ctrl+C 停止"
echo ""
echo "📊 预期效果："
echo "   ✅ CPU 占用: 15-25%（4核 CPU）"
echo "   ✅ 无重复字符（blank_penalty 解决）"
echo "   ✅ 实时标点显示（节流更新）"
echo "   ✅ 识别准确率保持"
echo ""
echo "💡 监控 CPU 占用："
echo "   打开另一个终端运行: top -p \$(pgrep fcitx5)"
echo ""
echo "开始监控..."
echo "============"
echo ""

# 启动 fcitx5
VINPUT_LOG=info fcitx5 2>&1 | tee /tmp/vinput-cpu-optimized.log
