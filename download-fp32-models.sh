#!/bin/bash
# 下载 FP32 模型文件

MODEL_DIR="/home/deepin/deepin-v2t/models/streaming"
BASE_URL="https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20/resolve/main"

echo "📥 下载 FP32 模型文件..."
echo "目标目录: $MODEL_DIR"
echo ""

cd "$MODEL_DIR" || exit 1

# 下载 encoder
if [ ! -f "encoder-epoch-99-avg-1.onnx" ]; then
    echo "下载 encoder-epoch-99-avg-1.onnx (315 MB)..."
    wget -q --show-progress "$BASE_URL/encoder-epoch-99-avg-1.onnx"
else
    echo "✓ encoder-epoch-99-avg-1.onnx 已存在"
fi

# 下载 decoder
if [ ! -f "decoder-epoch-99-avg-1.onnx" ]; then
    echo "下载 decoder-epoch-99-avg-1.onnx (13 MB)..."
    wget -q --show-progress "$BASE_URL/decoder-epoch-99-avg-1.onnx"
else
    echo "✓ decoder-epoch-99-avg-1.onnx 已存在"
fi

# 下载 joiner
if [ ! -f "joiner-epoch-99-avg-1.onnx" ]; then
    echo "下载 joiner-epoch-99-avg-1.onnx (3.2 MB)..."
    wget -q --show-progress "$BASE_URL/joiner-epoch-99-avg-1.onnx"
else
    echo "✓ joiner-epoch-99-avg-1.onnx 已存在"
fi

echo ""
echo "✅ 模型下载完成"
echo ""
echo "📊 模型文件列表:"
ls -lh *.onnx | awk '{print $9, $5}'
