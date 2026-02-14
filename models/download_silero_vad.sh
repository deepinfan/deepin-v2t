#!/bin/bash
# Silero VAD 模型下载脚本

set -e

MODEL_DIR="/home/deepin/deepin-v2t/models/silero-vad"
MODEL_FILE="$MODEL_DIR/silero_vad.onnx"

echo "=== Silero VAD 模型下载 ==="
echo

mkdir -p "$MODEL_DIR"
cd "$MODEL_DIR"

# 尝试多个下载源
SOURCES=(
    "https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx"
    "https://raw.githubusercontent.com/snakers4/silero-vad/master/files/silero_vad.onnx"
    "https://huggingface.co/snakers4/silero-vad/resolve/main/files/silero_vad.onnx"
)

for url in "${SOURCES[@]}"; do
    echo "尝试从 $url 下载..."
    if wget -q --show-progress -O "$MODEL_FILE" "$url" 2>&1; then
        if [ -s "$MODEL_FILE" ]; then
            echo "✅ 下载成功！"
            break
        else
            echo "⚠️  下载的文件为空，尝试下一个源..."
            rm -f "$MODEL_FILE"
        fi
    else
        echo "❌ 下载失败，尝试下一个源..."
    fi
done

if [ ! -s "$MODEL_FILE" ]; then
    echo
    echo "❌ 所有下载源都失败"
    echo
    echo "请手动下载："
    echo "1. 访问: https://github.com/snakers4/silero-vad"
    echo "2. 下载 files/silero_vad.onnx"
    echo "3. 保存到: $MODEL_FILE"
    exit 1
fi

echo
echo "📊 模型信息:"
ls -lh "$MODEL_FILE"
file "$MODEL_FILE"

echo
echo "✅ Silero VAD 模型已就绪！"
