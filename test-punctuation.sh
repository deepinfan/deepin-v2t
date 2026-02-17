#!/bin/bash
# 标点引擎诊断脚本

echo "🔍 标点引擎诊断"
echo "================"
echo ""

# 1. 检查配置文件
echo "📋 配置文件内容:"
echo "----------------"
cat ~/.config/vinput/config.toml | grep -A 6 "\[punctuation\]"
echo ""

# 2. 编译并安装
echo "🔨 重新编译和安装..."
cd /home/deepin/deepin-v2t
cargo build --release 2>&1 | tail -5
echo ""

echo "📦 安装到系统..."
sudo cp target/release/libvinput_core.so /usr/local/lib/
sudo ldconfig
echo ""

# 3. 停止 fcitx5
echo "🛑 停止 fcitx5..."
pkill -9 fcitx5
sleep 1

# 4. 启动 fcitx5 并捕获日志
echo "🚀 启动 fcitx5 (调试模式)..."
echo ""
echo "⚠️  请执行以下操作："
echo "   1. 切换到 V-Input 输入法"
echo "   2. 按空格开始录音"
echo "   3. 说一段话（至少 10 个词，词之间停顿 1 秒）"
echo "      例如：今天天气很好 [停顿] 我想出去散步 [停顿] 然后去超市买点东西"
echo "   4. 松开空格停止录音"
echo "   5. 观察识别结果和日志"
echo "   6. 按 Ctrl+C 停止"
echo ""
echo "📊 关键日志标记："
echo "   - '标点配置' - 配置加载情况"
echo "   - '停顿检测' - Token 时长分析"
echo "   - '检测到停顿' - 逗号插入决策"
echo ""

# 启动并过滤关键日志
RUST_LOG=vinput_core=debug fcitx5 2>&1 | tee /tmp/vinput-punctuation-debug.log | \
    grep -E "(标点配置|pause_ratio|min_tokens|停顿检测|检测到停顿|Token\[|处理 Token|句尾标点)" --color=always
