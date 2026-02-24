#!/usr/bin/env bash
# record_test.sh - 录制测试音频并标注
#
# 用法: ./tools/record_test.sh [--name 描述名称]
# 生成: vinput-core/tests/testdata/NNN_描述.wav
#        vinput-core/tests/testdata/NNN_描述.expected

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTDATA_DIR="${SCRIPT_DIR}/../vinput-core/tests/testdata"
mkdir -p "${TESTDATA_DIR}"

# 检查录音工具
if ! command -v pw-record &>/dev/null; then
    echo "错误: 需要 pw-record (pipewire-utils)" >&2
    exit 1
fi

# 解析参数
CUSTOM_NAME=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --name|-n) CUSTOM_NAME="$2"; shift 2 ;;
        *) echo "未知参数: $1" >&2; exit 1 ;;
    esac
done

# 确定测试编号（找下一个可用，支持 NNN 和 NNN_描述 两种格式）
idx=1
while ls "${TESTDATA_DIR}/$(printf '%03d' ${idx})"*.wav &>/dev/null 2>&1; do
    idx=$((idx + 1))
done
NUM=$(printf '%03d' ${idx})

# 构建文件名
if [[ -n "${CUSTOM_NAME}" ]]; then
    SLUG=$(echo "${CUSTOM_NAME}" | tr ' ' '_' | tr -cd '[:alnum:]_\u4e00-\u9fff')
    BASENAME="${NUM}_${SLUG}"
else
    BASENAME="${NUM}"
fi

WAV_FILE="${TESTDATA_DIR}/${BASENAME}.wav"
EXPECTED_FILE="${TESTDATA_DIR}/${BASENAME}.expected"

echo ""
echo "=== 录音测试 #${NUM} ==="
echo ""
echo "文件: ${WAV_FILE}"
echo "格式: 16kHz, 单声道, 16-bit PCM"
echo ""
echo "按 Enter 开始录音，说完后再按 Enter 停止..."
read -r

echo "🎙  正在录音... （按 Enter 停止）"
echo ""

# 用 pw-record 直接录制 WAV（与 fcitx5-vinput 使用相同的音频设备）
pw-record \
    --rate=16000 \
    --channels=1 \
    --format=s16 \
    "${WAV_FILE}" &
RECORD_PID=$!

# 等待用户按 Enter
read -r
kill "${RECORD_PID}" 2>/dev/null || true
wait "${RECORD_PID}" 2>/dev/null || true

echo ""
echo "✅ 录音完成"
echo ""

# 显示录音信息
if command -v soxi &>/dev/null; then
    DURATION=$(soxi -D "${WAV_FILE}" 2>/dev/null | xargs printf "%.1f" 2>/dev/null || echo "?")
    echo "时长: ${DURATION} 秒"
elif command -v ffprobe &>/dev/null; then
    ffprobe -v quiet -show_entries format=duration -of csv=p=0 "${WAV_FILE}" 2>/dev/null | \
        xargs printf "时长: %.1f 秒\n" || true
else
    SIZE=$(du -h "${WAV_FILE}" | cut -f1)
    echo "文件大小: ${SIZE}"
fi
echo ""

# 请求标注
echo "请输入正确的识别结果（包含标点，例如: 今天天气很好。）："
echo "（提示：标点使用中文全角，如句号。逗号，问号？感叹号！）"
echo ""
read -r EXPECTED

if [[ -z "${EXPECTED}" ]]; then
    echo "⚠️  标注为空，跳过保存"
    rm -f "${WAV_FILE}"
    exit 1
fi

echo "${EXPECTED}" > "${EXPECTED_FILE}"

echo ""
echo "✅ 已保存："
echo "   录音: ${WAV_FILE}"
echo "   标注: ${EXPECTED_FILE}  →  '${EXPECTED}'"
echo ""
echo "现在运行测试（debug 模式 + 详细日志）："
echo ""
echo "  RUST_LOG=info cargo test --test pipeline_e2e_tests -- --nocapture 2>&1 | grep -v 'running 0 tests'"
echo ""
echo "只运行这一个测试文件："
echo ""
echo "  RUST_LOG=info cargo test --test pipeline_e2e_tests ${BASENAME} -- --nocapture"
