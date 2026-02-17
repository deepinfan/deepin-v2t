#!/bin/bash
# CPU 占用对比测试

echo "📊 CPU 占用对比测试"
echo "===================="
echo ""

# 检查是否安装了 htop
if ! command -v htop &> /dev/null; then
    echo "⚠️  未安装 htop，使用 top 代替"
    USE_TOP=1
else
    USE_TOP=0
fi

echo "🔧 安装新版本（FP32 + CPU 优化）..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 2

echo "🚀 启动 fcitx5..."
fcitx5 &
sleep 3

FCITX5_PID=$(pgrep fcitx5)

if [ -z "$FCITX5_PID" ]; then
    echo "❌ fcitx5 未启动"
    exit 1
fi

echo "✅ fcitx5 已启动 (PID: $FCITX5_PID)"
echo ""
echo "📊 开始监控 CPU 占用..."
echo "   请切换到 V-Input 输入法并进行语音输入测试"
echo "   按 Ctrl+C 停止监控"
echo ""
echo "💡 测试建议："
echo "   1. 说一段较长的话（30秒以上）"
echo "   2. 观察 CPU 占用百分比"
echo "   3. 对比优化前后的差异"
echo ""
echo "预期 CPU 占用（4核 CPU）："
echo "   - 优化前: 单核 80-90%, 总体 40-50%"
echo "   - 优化后: 单核 50-60%, 总体 25-35%"
echo ""
sleep 3

if [ $USE_TOP -eq 1 ]; then
    # 使用 top
    top -p $FCITX5_PID
else
    # 使用 htop（更直观）
    htop -p $FCITX5_PID
fi
