#!/bin/bash
# 捕获 vinput 日志的脚本

LOG_FILE="/tmp/vinput_debug.log"

echo "=== V-Input 调试日志捕获 ===" > "$LOG_FILE"
echo "开始时间: $(date)" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

# 设置环境变量（注意：是 VINPUT_LOG 不是 RUST_LOG）
export VINPUT_LOG=debug

# 杀掉现有的 fcitx5
killall fcitx5 2>/dev/null
sleep 1

# 启动 fcitx5，将输出重定向到日志文件
fcitx5 >> "$LOG_FILE" 2>&1 &

echo "Fcitx5 已重启，日志输出到: $LOG_FILE"
echo "请测试语音输入，然后运行以下命令查看日志："
echo "  tail -f $LOG_FILE | grep -E 'ITN|split_stable'"
echo ""
echo "或者查看完整日志："
echo "  less $LOG_FILE"
