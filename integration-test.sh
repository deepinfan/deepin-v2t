#!/bin/bash
# V-Input 集成测试脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "V-Input 集成测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试结果统计
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 测试函数
run_test() {
    local test_name="$1"
    local test_command="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "[$TOTAL_TESTS] 测试: $test_name ... "

    if eval "$test_command" > /tmp/vinput_test_$TOTAL_TESTS.log 2>&1; then
        echo -e "${GREEN}✓ 通过${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}✗ 失败${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "  错误日志: /tmp/vinput_test_$TOTAL_TESTS.log"
        return 1
    fi
}

echo "1. 环境检查"
echo "----------------------------------------"

# 检查 Rust 工具链
run_test "Rust 工具链" "rustc --version"

# 检查 Cargo
run_test "Cargo 包管理器" "cargo --version"

# 检查 CMake
run_test "CMake 构建工具" "cmake --version"

# 检查 PipeWire
run_test "PipeWire 音频系统" "pw-cli info 0"

# 检查 Fcitx5
run_test "Fcitx5 输入法框架" "fcitx5 --version"

echo ""
echo "2. 模型文件检查"
echo "----------------------------------------"

# 检查 VAD 模型
run_test "VAD 模型文件" "test -f models/silero-vad/silero_vad.onnx"

# 检查 ASR 模型
run_test "ASR Encoder 模型" "test -f models/streaming/encoder-epoch-99-avg-1.onnx"
run_test "ASR Decoder 模型" "test -f models/streaming/decoder-epoch-99-avg-1.onnx"
run_test "ASR Joiner 模型" "test -f models/streaming/joiner-epoch-99-avg-1.onnx"
run_test "ASR Tokens 文件" "test -f models/streaming/tokens.txt"

echo ""
echo "3. 核心库编译测试"
echo "----------------------------------------"

# 编译 vinput-core
run_test "编译 vinput-core (debug)" "cd $SCRIPT_DIR/vinput-core && cargo build"

# 运行单元测试
run_test "vinput-core 单元测试" "cd $SCRIPT_DIR/vinput-core && cargo test --lib -- --test-threads=1"

echo ""
echo "4. FFI 接口测试"
echo "----------------------------------------"

# 检查生成的 C 头文件
run_test "生成 C 头文件" "test -f $SCRIPT_DIR/target/vinput_core.h"

# 检查生成的动态库
run_test "生成动态库 (debug)" "test -f $SCRIPT_DIR/target/debug/libvinput_core.so"

echo ""
echo "5. Fcitx5 插件编译测试"
echo "----------------------------------------"

# 编译 Fcitx5 插件
run_test "配置 Fcitx5 插件" "cd $SCRIPT_DIR/fcitx5-vinput/build && cmake .."
run_test "编译 Fcitx5 插件" "cd $SCRIPT_DIR/fcitx5-vinput/build && make"
run_test "检查插件文件" "test -f $SCRIPT_DIR/fcitx5-vinput/build/vinput.so"

echo ""
echo "6. GUI 设置界面测试"
echo "----------------------------------------"

# 编译 GUI
run_test "编译 GUI 设置界面" "cd $SCRIPT_DIR/vinput-gui && cargo build"
run_test "检查 GUI 可执行文件" "test -f $SCRIPT_DIR/target/debug/vinput-settings"

echo ""
echo "7. 功能模块测试"
echo "----------------------------------------"

# ITN 测试
run_test "ITN 货币规则测试" "cd $SCRIPT_DIR/vinput-core && cargo test --lib itn::engine::tests::test_protected_common_words"

# 热词引擎测试
run_test "热词引擎测试" "cd $SCRIPT_DIR/vinput-core && cargo test --lib hotwords::engine::tests"

# 撤销机制测试
run_test "撤销机制测试" "cd $SCRIPT_DIR/vinput-core && cargo test --lib undo::tests"

echo ""
echo "8. 配置文件测试"
echo "----------------------------------------"

# 创建测试配置
TEST_CONFIG_DIR="/tmp/vinput-test-config"
mkdir -p "$TEST_CONFIG_DIR"

cat > "$TEST_CONFIG_DIR/config.toml" << 'EOF'
[vad]
mode = "push-to-talk"

[vad.silero]
model_path = "models/silero-vad/silero_vad.onnx"
sample_rate = 16000
frame_size = 512

[asr]
model_dir = "models/streaming"
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

run_test "配置文件格式验证" "test -f $TEST_CONFIG_DIR/config.toml"

echo ""
echo "=========================================="
echo "测试结果汇总"
echo "=========================================="
echo ""
echo "总测试数: $TOTAL_TESTS"
echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
echo -e "${RED}失败: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    echo ""
    echo "系统已准备就绪，可以进行以下操作:"
    echo "1. 安装 Fcitx5 插件: cd fcitx5-vinput/build && sudo make install"
    echo "2. 重启 Fcitx5: fcitx5 -r"
    echo "3. 运行 GUI 设置: ./run-settings.sh"
    echo ""
    exit 0
else
    echo -e "${RED}✗ 部分测试失败，请检查错误日志${NC}"
    echo ""
    echo "错误日志位置: /tmp/vinput_test_*.log"
    echo ""
    exit 1
fi
