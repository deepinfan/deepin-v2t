#!/bin/bash
# 水滴语音输入法 DEB 打包脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  💧 水滴语音输入法 DEB 打包脚本${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 包信息
PACKAGE_NAME="droplet-voice-input"
VERSION="0.1.0"
ARCH="amd64"
MAINTAINER="Deepin Community <bbs.deepin.org>"
DESCRIPTION="水滴语音输入法 - 离线中文语音输入法"
HOMEPAGE="http://bbs.deepin.org"

# 工作目录
WORK_DIR="/tmp/${PACKAGE_NAME}_${VERSION}_${ARCH}"
DEB_DIR="${WORK_DIR}/DEBIAN"

echo -e "${YELLOW}📦 包信息:${NC}"
echo "  名称: ${PACKAGE_NAME}"
echo "  版本: ${VERSION}"
echo "  架构: ${ARCH}"
echo ""

# 清理旧的构建目录
if [ -d "${WORK_DIR}" ]; then
    echo -e "${YELLOW}🧹 清理旧的构建目录...${NC}"
    rm -rf "${WORK_DIR}"
fi

# 创建目录结构
echo -e "${YELLOW}📁 创建目录结构...${NC}"
mkdir -p "${DEB_DIR}"
mkdir -p "${WORK_DIR}/usr/bin"
mkdir -p "${WORK_DIR}/usr/lib/x86_64-linux-gnu/fcitx5"
mkdir -p "${WORK_DIR}/usr/share/applications"
mkdir -p "${WORK_DIR}/usr/share/fcitx5/addon"
mkdir -p "${WORK_DIR}/usr/share/fcitx5/inputmethod"
mkdir -p "${WORK_DIR}/usr/share/droplet-voice-input/models"

# 复制文件
echo -e "${YELLOW}📋 复制文件...${NC}"

echo "  ✓ 核心库 (2.6 MB)"
cp target/release/libvinput_core.so "${WORK_DIR}/usr/lib/x86_64-linux-gnu/"
chmod 644 "${WORK_DIR}/usr/lib/x86_64-linux-gnu/libvinput_core.so"

echo "  ✓ Sherpa-ONNX 库 (5.3 MB)"
cp deps/sherpa-onnx/lib/libsherpa-onnx-c-api.so "${WORK_DIR}/usr/lib/x86_64-linux-gnu/"
chmod 644 "${WORK_DIR}/usr/lib/x86_64-linux-gnu/libsherpa-onnx-c-api.so"

echo "  ✓ ONNX Runtime 库 (23 MB)"
cp deps/sherpa-onnx/lib/libonnxruntime.so "${WORK_DIR}/usr/lib/x86_64-linux-gnu/"
chmod 644 "${WORK_DIR}/usr/lib/x86_64-linux-gnu/libonnxruntime.so"

echo "  ✓ Fcitx5 插件 (49 KB)"
cp fcitx5-vinput/build/vinput.so "${WORK_DIR}/usr/lib/x86_64-linux-gnu/fcitx5/"
chmod 644 "${WORK_DIR}/usr/lib/x86_64-linux-gnu/fcitx5/vinput.so"

echo "  ✓ Fcitx5 配置文件"
cp fcitx5-vinput/vinput.conf "${WORK_DIR}/usr/share/fcitx5/inputmethod/"
cp fcitx5-vinput/vinput-addon.conf "${WORK_DIR}/usr/share/fcitx5/addon/"
chmod 644 "${WORK_DIR}/usr/share/fcitx5/inputmethod/vinput.conf"
chmod 644 "${WORK_DIR}/usr/share/fcitx5/addon/vinput-addon.conf"

echo "  ✓ 设置程序 (6.7 MB)"
cp target/release/vinput-settings "${WORK_DIR}/usr/bin/"
chmod 755 "${WORK_DIR}/usr/bin/vinput-settings"

echo "  ✓ 桌面启动文件"
cat > "${WORK_DIR}/usr/share/applications/droplet-voice-input.desktop" << 'EOF'
[Desktop Entry]
Name=水滴语音输入法设置
Name[en]=Droplet Voice Input Settings
Comment=配置水滴语音输入法
Comment[en]=Configure Droplet Voice Input
Exec=vinput-settings
Icon=audio-input-microphone
Terminal=false
Type=Application
Categories=Settings;Utility;
Keywords=voice;input;speech;recognition;语音;输入;
EOF
chmod 644 "${WORK_DIR}/usr/share/applications/droplet-voice-input.desktop"

echo "  ✓ AI 模型文件 (227 MB)"
cp models/streaming/encoder.int8.onnx "${WORK_DIR}/usr/share/droplet-voice-input/models/"
cp models/streaming/decoder.int8.onnx "${WORK_DIR}/usr/share/droplet-voice-input/models/"
cp models/streaming/tokens.txt "${WORK_DIR}/usr/share/droplet-voice-input/models/"
chmod 644 "${WORK_DIR}/usr/share/droplet-voice-input/models/"*

echo "  ✓ 示例配置文件"
cat > "${WORK_DIR}/usr/share/droplet-voice-input/config.toml.example" << 'EOF'
[asr]
model_dir = "/usr/share/droplet-voice-input/models"
sample_rate = 16000
max_active_paths = 2

[vad]
threshold = 0.5
min_speech_duration_ms = 250
min_silence_duration_ms = 500

[punctuation]
style = "Professional"

[hotwords]
# 用户可以在 ~/.config/vinput/config.toml 中添加自定义热词
EOF
chmod 644 "${WORK_DIR}/usr/share/droplet-voice-input/config.toml.example"


# 创建 control 文件
echo -e "${YELLOW}📝 创建 control 文件...${NC}"
cat > "${DEB_DIR}/control" << EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: fcitx5, libpipewire-0.3-0 | libpipewire-0.3-modules
Recommends: pipewire-audio
Provides: libonnxruntime, libsherpa-onnx-c-api
Maintainer: ${MAINTAINER}
Description: ${DESCRIPTION}
 水滴语音输入法是一个完全离线的中文语音输入法，基于 Fcitx5 框架。
 .
 核心特性：
  - 完全离线，保护隐私
  - 实时流式识别
  - 智能标点符号
  - 文本规范化 (ITN)
  - 热词支持
  - 撤销/重试功能
 .
 技术栈：
  - Rust - 核心引擎
  - sherpa-onnx - 语音识别
  - PipeWire - 音频捕获
  - Fcitx5 - 输入法框架
  - egui - 图形界面
 .
 首发于深度操作系统论坛: ${HOMEPAGE}
Homepage: ${HOMEPAGE}
EOF

# 创建 postinst 脚本
echo -e "${YELLOW}📝 创建 postinst 脚本...${NC}"
cat > "${DEB_DIR}/postinst" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    configure)
        ldconfig

        if [ ! -d /etc/skel/.config/vinput ]; then
            mkdir -p /etc/skel/.config/vinput
            if [ -f /usr/share/droplet-voice-input/config.toml.example ]; then
                cp /usr/share/droplet-voice-input/config.toml.example \
                   /etc/skel/.config/vinput/config.toml
            fi
        fi

        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "  💧 水滴语音输入法安装完成！"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        echo "  使用方法："
        echo "    1. 重启 Fcitx5: fcitx5 -r"
        echo "    2. 在 Fcitx5 配置中添加「水滴语音输入法」"
        echo "    3. 切换到水滴语音输入法"
        echo "    4. 按空格开始录音，说话后松开空格"
        echo "    5. 运行 vinput-settings 打开设置界面"
        echo ""
        echo "  首发于深度操作系统论坛: http://bbs.deepin.org"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        ;;
esac

exit 0
EOF
chmod 755 "${DEB_DIR}/postinst"

# 创建 prerm 脚本
echo -e "${YELLOW}📝 创建 prerm 脚本...${NC}"
cat > "${DEB_DIR}/prerm" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    remove|upgrade|deconfigure)
        if pgrep -x fcitx5 > /dev/null; then
            echo "正在停止 Fcitx5..."
            pkill fcitx5 || true
            sleep 1
        fi
        ;;
esac

exit 0
EOF
chmod 755 "${DEB_DIR}/prerm"

# 创建 postrm 脚本
echo -e "${YELLOW}📝 创建 postrm 脚本...${NC}"
cat > "${DEB_DIR}/postrm" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    remove|purge)
        ldconfig
        ;;

    purge)
        rm -rf /etc/skel/.config/vinput
        echo "配置文件模板已删除"
        ;;
esac

exit 0
EOF
chmod 755 "${DEB_DIR}/postrm"

# 计算安装大小
echo -e "${YELLOW}📊 计算安装大小...${NC}"
INSTALLED_SIZE=$(du -sk "${WORK_DIR}" | cut -f1)
echo "Installed-Size: ${INSTALLED_SIZE}" >> "${DEB_DIR}/control"

# 构建 deb 包
echo -e "${YELLOW}🔨 构建 DEB 包...${NC}"
DEB_FILE="${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "${WORK_DIR}" "${DEB_FILE}"

# 显示结果
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✅ DEB 包构建完成！${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}📦 包信息:${NC}"
ls -lh "${DEB_FILE}"
echo ""
echo -e "${YELLOW}📋 包内容:${NC}"
dpkg-deb --contents "${DEB_FILE}" | head -20
echo "..."
echo ""
echo -e "${YELLOW}🔍 包信息:${NC}"
dpkg-deb --info "${DEB_FILE}"
echo ""
echo -e "${GREEN}安装命令:${NC}"
echo "  sudo dpkg -i ${DEB_FILE}"
echo "  sudo apt-get install -f  # 修复依赖（如果需要）"
echo ""
echo -e "${GREEN}卸载命令:${NC}"
echo "  sudo apt-get remove ${PACKAGE_NAME}      # 保留配置"
echo "  sudo apt-get purge ${PACKAGE_NAME}       # 完全删除"
echo ""

