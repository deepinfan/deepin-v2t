#!/bin/bash
# V-Input 快速部署脚本
# 自动化部署测试流程（需要手动执行 sudo 命令）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"  # 脚本就在项目根目录

echo "╔════════════════════════════════════════════════╗"
echo "║     V-Input Fcitx5 快速部署脚本               ║"
echo "╚════════════════════════════════════════════════╝"
echo

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 步骤计数
STEP=0

print_step() {
    STEP=$((STEP + 1))
    echo -e "\n${BLUE}【步骤 $STEP】$1${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# ========== 步骤 1: 环境检查 ==========
print_step "环境检查"

echo "检查 Fcitx5 是否运行..."
if pgrep -x fcitx5 > /dev/null; then
    print_success "Fcitx5 正在运行 (PID: $(pgrep -x fcitx5))"
    FCITX5_VERSION=$(fcitx5 --version 2>&1 | head -1 || echo "未知")
    echo "   版本: $FCITX5_VERSION"
else
    print_warning "Fcitx5 未运行"
fi

echo
echo "检查 PipeWire 状态..."
if systemctl --user is-active --quiet pipewire; then
    print_success "PipeWire 正在运行"
else
    print_error "PipeWire 未运行"
    echo "   请启动 PipeWire: systemctl --user start pipewire"
    exit 1
fi

echo
echo "检查麦克风设备..."
MIC_COUNT=$(pw-cli ls Node 2>/dev/null | grep -c "Audio/Source" || echo "0")
if [ "$MIC_COUNT" -gt 0 ]; then
    print_success "找到 $MIC_COUNT 个音频输入设备"
else
    print_warning "未找到音频输入设备，请连接麦克风"
fi

# ========== 步骤 2: 检查开发依赖 ==========
print_step "检查开发依赖"

NEED_INSTALL=false

echo "检查 Fcitx5 开发库..."
# Deepin 使用 Fcitx5Core，其他发行版使用 fcitx5-core
if pkg-config --exists Fcitx5Core; then
    FCITX5_DEV_VERSION=$(pkg-config --modversion Fcitx5Core)
    print_success "Fcitx5 开发库已安装 (版本 $FCITX5_DEV_VERSION) [Deepin]"
elif pkg-config --exists fcitx5-core; then
    FCITX5_DEV_VERSION=$(pkg-config --modversion fcitx5-core)
    print_success "Fcitx5 开发库已安装 (版本 $FCITX5_DEV_VERSION)"
else
    print_error "Fcitx5 开发库未安装"
    NEED_INSTALL=true
fi

echo
echo "检查 CMake..."
if command -v cmake &> /dev/null; then
    CMAKE_VERSION=$(cmake --version | head -1)
    print_success "CMake 已安装 ($CMAKE_VERSION)"
else
    print_error "CMake 未安装"
    NEED_INSTALL=true
fi

echo
echo "检查构建工具..."
if command -v g++ &> /dev/null; then
    GCC_VERSION=$(g++ --version | head -1)
    print_success "G++ 已安装 ($GCC_VERSION)"
else
    print_error "G++ 未安装"
    NEED_INSTALL=true
fi

if [ "$NEED_INSTALL" = true ]; then
    echo
    print_warning "缺少依赖，请运行以下命令安装："
    echo

    # 检测发行版
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" == "deepin" ]] || [[ "$ID_LIKE" =~ "deepin" ]]; then
            echo "检测到 Deepin 系统，开发库可能已安装但包名不同"
            echo "Deepin 已安装的包："
            echo "    - libfcitx5core-dev"
            echo "    - libfcitx5utils-dev"
            echo "    - libfcitx5config-dev"
            echo
            echo "如果缺少其他工具，请安装："
            echo "    sudo apt install -y cmake build-essential pkg-config"
        else
            echo "    sudo apt update"
            echo "    sudo apt install -y fcitx5-dev libfcitx5core-dev cmake build-essential"
        fi
    else
        echo "    sudo apt update"
        echo "    sudo apt install -y fcitx5-dev libfcitx5core-dev cmake build-essential"
    fi

    echo
    read -p "是否现在安装？(需要 sudo 权限) [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo apt update
        # Deepin 已经有 libfcitx5core-dev，只需要确保构建工具齐全
        sudo apt install -y cmake build-essential pkg-config
        print_success "依赖安装完成"
    else
        print_error "缺少依赖，退出"
        exit 1
    fi
fi

# ========== 步骤 3: 编译 Rust 核心库 ==========
print_step "编译 Rust 核心库 (真实音频模式)"

cd "$PROJECT_ROOT/vinput-core"

echo "清理旧的构建..."
cargo clean -q

echo "编译 vinput-core (--features pipewire-capture)..."
if cargo build --release --features pipewire-capture 2>&1 | grep -E "(Compiling|Finished)"; then
    print_success "vinput-core 编译成功"
else
    print_error "vinput-core 编译失败"
    exit 1
fi

LIB_PATH="$PROJECT_ROOT/target/release/libvinput_core.so"
if [ -f "$LIB_PATH" ]; then
    LIB_SIZE=$(du -h "$LIB_PATH" | cut -f1)
    print_success "生成 libvinput_core.so ($LIB_SIZE)"
else
    print_error "未找到 libvinput_core.so"
    exit 1
fi

# ========== 步骤 4: 编译 Fcitx5 插件 ==========
print_step "编译 Fcitx5 插件"

cd "$PROJECT_ROOT/fcitx5-vinput-mvp"

echo "创建构建目录..."
rm -rf build
mkdir -p build
cd build

echo "配置 CMake..."
if cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DCMAKE_VERBOSE_MAKEFILE=OFF; then
    print_success "CMake 配置成功"
else
    print_error "CMake 配置失败"
    exit 1
fi

echo
echo "编译插件..."
if make -j$(nproc); then
    print_success "Fcitx5 插件编译成功"
else
    print_error "编译失败"
    exit 1
fi

PLUGIN_PATH="$PROJECT_ROOT/fcitx5-vinput-mvp/build/vinput.so"
if [ -f "$PLUGIN_PATH" ]; then
    PLUGIN_SIZE=$(du -h "$PLUGIN_PATH" | cut -f1)
    print_success "生成 vinput.so ($PLUGIN_SIZE)"
else
    print_error "未找到 vinput.so"
    exit 1
fi

# ========== 步骤 5: 安装插件 ==========
print_step "安装插件"

echo "准备安装到系统目录（需要 sudo 权限）..."
echo
echo "将安装以下文件："
echo "  - /usr/lib/*/fcitx5/vinput.so"
echo "  - /usr/share/fcitx5/addon/vinput.conf"
echo "  - /usr/share/fcitx5/inputmethod/vinput-im.conf"
echo

read -p "是否继续安装？[y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if sudo make install; then
        print_success "插件安装成功"
    else
        print_error "安装失败"
        exit 1
    fi
else
    print_warning "跳过安装，手动安装命令:"
    echo "    cd $PROJECT_ROOT/fcitx5-vinput-mvp/build"
    echo "    sudo make install"
    exit 0
fi

# 验证安装
echo
echo "验证安装..."
INSTALLED_COUNT=0

if ls /usr/lib/*/fcitx5/vinput.so &>/dev/null; then
    print_success "vinput.so 已安装"
    INSTALLED_COUNT=$((INSTALLED_COUNT + 1))
fi

if [ -f "/usr/share/fcitx5/addon/vinput.conf" ]; then
    print_success "vinput.conf 已安装"
    INSTALLED_COUNT=$((INSTALLED_COUNT + 1))
fi

if [ -f "/usr/share/fcitx5/inputmethod/vinput-im.conf" ]; then
    print_success "vinput-im.conf 已安装"
    INSTALLED_COUNT=$((INSTALLED_COUNT + 1))
fi

if [ $INSTALLED_COUNT -eq 3 ]; then
    print_success "所有文件安装成功"
else
    print_warning "部分文件未安装 ($INSTALLED_COUNT/3)"
fi

# ========== 步骤 6: 重启 Fcitx5 ==========
print_step "重启 Fcitx5"

echo "重启 Fcitx5 以加载新插件..."
if fcitx5 -r; then
    print_success "Fcitx5 已重启"
else
    print_warning "自动重启失败，请手动执行: fcitx5 -r"
fi

sleep 2

# 验证插件加载
echo
echo "验证插件加载..."
if fcitx5-remote -d 2>/dev/null | grep -qi vinput; then
    print_success "V-Input 插件已加载"
else
    print_warning "未检测到 V-Input，请检查日志:"
    echo "    journalctl --user -u fcitx5 -n 50"
fi

# ========== 步骤 7: 运行测试 ==========
print_step "运行集成测试"

echo "测试音频捕获..."
cd "$PROJECT_ROOT"
echo
if cargo run --example test_pipewire_subprocess --features pipewire-capture --release 2>&1 | tail -30; then
    print_success "音频捕获测试完成"
else
    print_warning "音频捕获测试失败"
fi

echo
echo "测试 FFI 接口..."
if [ -x "$PROJECT_ROOT/test_multi_commands" ]; then
    if "$PROJECT_ROOT/test_multi_commands"; then
        print_success "FFI 多命令测试通过"
    else
        print_warning "FFI 测试失败"
    fi
else
    print_warning "test_multi_commands 未找到，跳过"
fi

# ========== 完成 ==========
echo
echo "╔════════════════════════════════════════════════╗"
echo "║            部署完成！                          ║"
echo "╚════════════════════════════════════════════════╝"
echo
print_success "V-Input 已成功部署到 Fcitx5"
echo
echo "下一步操作："
echo
echo "1. 配置 Fcitx5 添加 V-Input:"
echo "   ${BLUE}fcitx5-configtool${NC}"
echo
echo "2. 或使用命令行快速配置:"
echo "   ${BLUE}fcitx5-remote -s vinput${NC}"
echo
echo "3. 在文本编辑器中测试:"
echo "   - 打开 gedit 或其他编辑器"
echo "   - 切换到 V-Input 输入法"
echo "   - 对着麦克风说话"
echo
echo "4. 查看日志（调试用）:"
echo "   ${BLUE}journalctl --user -u fcitx5 -f${NC}"
echo
echo "5. 详细文档:"
echo "   ${BLUE}cat docs/DEPLOYMENT_GUIDE.md${NC}"
echo
print_warning "注意: Phase 1 的 ASR 是占位实现，会输出测试文本而非真实识别"
echo
