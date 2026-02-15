#!/bin/bash
# V-Input Fcitx5 插件安装脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "V-Input Fcitx5 插件安装"
echo "=========================================="
echo ""

# 检查是否有 sudo 权限
if [ "$EUID" -ne 0 ]; then
    echo "此脚本需要 root 权限运行"
    echo "请使用: sudo ./install-fcitx5-plugin.sh"
    exit 1
fi

echo "1. 检查编译产物..."
if [ ! -f "$SCRIPT_DIR/fcitx5-vinput/build/vinput.so" ]; then
    echo "错误: 找不到 vinput.so"
    echo "请先运行: cd fcitx5-vinput/build && cmake .. && make"
    exit 1
fi

if [ ! -f "$SCRIPT_DIR/target/release/libvinput_core.so" ]; then
    echo "警告: 找不到 release 版本的 libvinput_core.so"
    echo "尝试使用 debug 版本..."
    if [ ! -f "$SCRIPT_DIR/target/debug/libvinput_core.so" ]; then
        echo "错误: 找不到 libvinput_core.so"
        echo "请先运行: cd vinput-core && cargo build --release"
        exit 1
    fi
    VINPUT_CORE_LIB="$SCRIPT_DIR/target/debug/libvinput_core.so"
else
    VINPUT_CORE_LIB="$SCRIPT_DIR/target/release/libvinput_core.so"
fi

echo "✓ 编译产物检查完成"
echo ""

echo "2. 安装 Fcitx5 插件..."
cd "$SCRIPT_DIR/fcitx5-vinput/build"
make install
echo "✓ Fcitx5 插件安装完成"
echo ""

echo "3. 安装核心库..."
# 复制核心库到系统目录
cp "$VINPUT_CORE_LIB" /usr/local/lib/libvinput_core.so
chmod 755 /usr/local/lib/libvinput_core.so
ldconfig
echo "✓ 核心库安装完成"
echo ""

echo "4. 创建模型目录..."
mkdir -p /usr/share/vinput/models
echo "✓ 模型目录创建完成"
echo ""

echo "5. 复制模型文件..."
if [ -d "$SCRIPT_DIR/models/silero-vad" ]; then
    cp -r "$SCRIPT_DIR/models/silero-vad" /usr/share/vinput/models/
    echo "✓ VAD 模型复制完成"
fi

if [ -d "$SCRIPT_DIR/models/streaming" ]; then
    cp -r "$SCRIPT_DIR/models/streaming" /usr/share/vinput/models/
    echo "✓ ASR 模型复制完成"
fi
echo ""

echo "6. 创建配置目录..."
mkdir -p /etc/vinput
cat > /etc/vinput/config.toml << 'EOF'
[vad]
mode = "push-to-talk"

[vad.silero]
model_path = "/usr/share/vinput/models/silero-vad/silero_vad.onnx"
sample_rate = 16000
frame_size = 512

[asr]
model_dir = "/usr/share/vinput/models/streaming"
sample_rate = 16000
hotwords_score = 1.5

[punctuation]
style = "Professional"
streaming_pause_ratio = 3.5
streaming_min_tokens = 5
allow_exclamation = false
question_strict = true

[hotwords]
global_weight = 2.5
max_words = 10000

[endpoint]
min_speech_duration_ms = 300
max_speech_duration_ms = 30000
trailing_silence_ms = 800
force_timeout_ms = 60000
vad_assisted = true
vad_silence_confirm_frames = 5
EOF
echo "✓ 默认配置文件创建完成"
echo ""

echo "=========================================="
echo "安装完成！"
echo "=========================================="
echo ""
echo "下一步操作:"
echo "1. 重启 Fcitx5: fcitx5 -r"
echo "2. 打开 Fcitx5 配置工具"
echo "3. 添加 'V-Input' 输入法"
echo "4. 使用空格键开始/停止录音"
echo ""
echo "提示:"
echo "- 配置文件: ~/.config/vinput/config.toml"
echo "- 系统配置: /etc/vinput/config.toml"
echo "- 查看日志: journalctl --user -u fcitx5 -f"
echo ""
