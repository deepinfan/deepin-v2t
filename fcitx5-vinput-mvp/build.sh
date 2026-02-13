#!/bin/bash
# Fcitx5 V-Input 插件构建脚本
#
# Phase 0: 需要先安装 Fcitx5 开发库
# Phase 1: 完整构建和安装

set -e

echo "=== V-Input Fcitx5 插件构建 ==="
echo

# 检查 Fcitx5 开发库
if ! pkg-config --exists fcitx5-core; then
    echo "❌ 错误: 未找到 Fcitx5 开发库"
    echo
    echo "请先安装 Fcitx5 开发包:"
    echo "  sudo apt install fcitx5-dev libfcitx5core-dev"
    echo "  或"
    echo "  sudo dnf install fcitx5-devel"
    exit 1
fi

FCITX5_VERSION=$(pkg-config --modversion fcitx5-core)
echo "✓ 找到 Fcitx5 Core: $FCITX5_VERSION"
echo

# 检查 vinput-core 库
if [ ! -f "../target/release/libvinput_core.so" ]; then
    echo "❌ 错误: 未找到 libvinput_core.so"
    echo
    echo "请先构建 vinput-core:"
    echo "  cd ../vinput-core"
    echo "  cargo build --release"
    exit 1
fi

echo "✓ 找到 libvinput_core.so"
echo

# 创建构建目录
mkdir -p build
cd build

echo "配置 CMake..."
cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/usr

echo
echo "编译..."
make -j$(nproc)

echo
echo "✅ 构建成功!"
echo
echo "安装命令 (需要 root 权限):"
echo "  cd build"
echo "  sudo make install"
echo
echo "安装后重启 Fcitx5:"
echo "  fcitx5 -r"
echo
echo "Phase 0 验证完成："
echo "  - Fcitx5 插件骨架已创建"
echo "  - FFI 接口集成完成"
echo "  - Phase 1 将实现完整语音识别"
