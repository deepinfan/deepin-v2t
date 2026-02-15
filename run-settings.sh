#!/bin/bash
# V-Input 设置界面启动脚本

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VINPUT_SETTINGS="$SCRIPT_DIR/target/release/vinput-settings"

if [ ! -f "$VINPUT_SETTINGS" ]; then
    echo "错误: 找不到 vinput-settings 可执行文件"
    echo "请先运行: cd vinput-gui && cargo build --release"
    exit 1
fi

echo "启动 V-Input 设置界面..."
"$VINPUT_SETTINGS"
