#!/bin/bash
# 测试 FP32 模型和 CPU 优化

echo "🔧 安装新版本（FP32 模型 + CPU 优化）..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

echo "📊 模型信息："
echo "   - 使用 FP32 完整精度模型（非 INT8）"
echo "   - 模型大小: encoder=315MB, decoder=14MB, joiner=13MB"
echo "   - 线程数: 4"
echo "   - max_active_paths: 2 (降低 CPU 占用)"
echo ""

echo "🚀 启动 fcitx5..."
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
echo "📊 预期改进："
echo "   ✅ 不再有重复字符（天天天天 → 天）"
echo "   ✅ 识别准确率提高"
echo "   ✅ CPU 占用降低（max_active_paths: 4→2）"
echo "   ⚠️  首次加载时间稍长（FP32 模型更大）"
echo ""
echo "💡 监控 CPU 占用："
echo "   打开另一个终端运行: top -p \$(pgrep fcitx5)"
echo ""
echo "开始监控..."
echo "============"
echo ""

# 启动 fcitx5
VINPUT_LOG=info fcitx5 2>&1 | tee /tmp/vinput-fp32-test.log
