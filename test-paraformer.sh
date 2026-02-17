#!/bin/bash
# 测试 Paraformer 模型

echo "🔧 安装新版本（Paraformer 模型）..."
sudo cp /home/deepin/deepin-v2t/target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig

echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

echo "📊 模型信息："
echo "   - 模型类型: Paraformer (非 Transducer)"
echo "   - Encoder: encoder.int8.onnx (158MB)"
echo "   - Decoder: decoder.int8.onnx (69MB)"
echo "   - 总大小: ~227MB"
echo "   - 特点: 双语（中英文）、流式识别"
echo ""

echo "🚀 启动 fcitx5..."
echo ""
echo "⚠️  测试说明："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说话测试（中文或英文）："
echo "      中文: 今天天气很好，我想出去散步"
echo "      英文: Hello world, this is a test"
echo "      混合: 我在学习 Python 编程"
echo "   4. 松开空格停止录音"
echo "   5. 观察识别结果"
echo "   6. 按 Ctrl+C 停止"
echo ""
echo "📊 预期效果："
echo "   ✅ 支持中英文混合识别"
echo "   ✅ 识别速度快（Paraformer 优化）"
echo "   ✅ CPU 占用低（INT8 量化）"
echo "   ✅ 无重复字符问题"
echo ""
echo "💡 监控 CPU 占用："
echo "   打开另一个终端运行: top -p \$(pgrep fcitx5)"
echo ""
echo "开始监控..."
echo "============"
echo ""

# 启动 fcitx5
VINPUT_LOG=info fcitx5 2>&1 | tee /tmp/vinput-paraformer-test.log
