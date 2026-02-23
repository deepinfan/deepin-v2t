#!/usr/bin/env bash
# 本地安装脚本（用于测试）

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "${EUID}" -ne 0 ]; then
  echo "请使用 sudo 运行此脚本"
  exit 1
fi

# 支持 debug/release 两种构建类型，默认 release
# 用法: sudo install-local.sh [debug]
BUILD_TYPE="${1:-release}"
if [ "${BUILD_TYPE}" != "debug" ] && [ "${BUILD_TYPE}" != "release" ]; then
  echo "用法: $0 [debug|release]"
  exit 1
fi
echo "安装构建类型: ${BUILD_TYPE}"

LIBDIR="/usr/lib"
MULTIARCH=""
if command -v dpkg-architecture >/dev/null 2>&1; then
  MULTIARCH="$(dpkg-architecture -qDEB_HOST_MULTIARCH 2>/dev/null || true)"
fi
if [ -n "${MULTIARCH}" ] && [ -d "/usr/lib/${MULTIARCH}" ]; then
  LIBDIR="/usr/lib/${MULTIARCH}"
fi
FCITX_DIR="${LIBDIR}/fcitx5"

require_file() {
  local path="$1"
  local hint="$2"
  if [ ! -f "${path}" ]; then
    echo "缺少文件: ${path}"
    echo "${hint}"
    exit 1
  fi
}

require_file "${ROOT_DIR}/target/${BUILD_TYPE}/libvinput_core.so" "请先运行: cargo build --workspace"
require_file "${ROOT_DIR}/target/${BUILD_TYPE}/vinput-settings" "请先运行: cargo build --workspace"
require_file "${ROOT_DIR}/fcitx5-vinput/build/vinput.so" "请先编译插件: cd fcitx5-vinput && mkdir -p build && cd build && cmake .. && make"
require_file "${ROOT_DIR}/fcitx5-vinput/vinput.conf" "缺少输入法配置文件"
require_file "${ROOT_DIR}/fcitx5-vinput/vinput-addon.conf" "缺少 addon 配置文件"
require_file "${ROOT_DIR}/config.toml.example" "缺少示例配置文件"

install -d "${LIBDIR}" "${FCITX_DIR}" \
  /usr/share/fcitx5/inputmethod \
  /usr/share/fcitx5/addon \
  /usr/share/droplet-voice-input/models \
  /usr/share/droplet-voice-input/models/punct-ct-transformer \
  /usr/share/droplet-voice-input/models/silero-vad \
  /usr/share/droplet-voice-input \
  /usr/bin

install -m 644 "${ROOT_DIR}/target/${BUILD_TYPE}/libvinput_core.so" "${LIBDIR}/"

# debug 构建时同时覆盖 target/release/ 下的库，
# 因为 vinput.so 的 RPATH 硬编码指向 target/release/，优先于系统目录。
# 不覆盖 release 构建则 debug-logs 日志永远不会出现。
if [ "${BUILD_TYPE}" = "debug" ]; then
  mkdir -p "${ROOT_DIR}/target/release"
  cp "${ROOT_DIR}/target/debug/libvinput_core.so" "${ROOT_DIR}/target/release/libvinput_core.so"
  echo "已将 debug 库复制到 target/release/（覆盖 RPATH 硬编码路径）"
fi
install -m 644 "${ROOT_DIR}/fcitx5-vinput/build/vinput.so" "${FCITX_DIR}/"
install -m 644 "${ROOT_DIR}/fcitx5-vinput/vinput.conf" /usr/share/fcitx5/inputmethod/vinput.conf
install -m 644 "${ROOT_DIR}/fcitx5-vinput/vinput-addon.conf" /usr/share/fcitx5/addon/vinput.conf
install -m 755 "${ROOT_DIR}/target/${BUILD_TYPE}/vinput-settings" /usr/bin/vinput-settings
install -m 644 "${ROOT_DIR}/config.toml.example" /usr/share/droplet-voice-input/config.toml.example

if [ -f "${ROOT_DIR}/deps/sherpa-onnx/lib/libsherpa-onnx-c-api.so" ]; then
  install -m 644 "${ROOT_DIR}/deps/sherpa-onnx/lib/libsherpa-onnx-c-api.so" "${LIBDIR}/"
fi
if [ -f "${ROOT_DIR}/deps/sherpa-onnx/lib/libonnxruntime.so" ]; then
  install -m 644 "${ROOT_DIR}/deps/sherpa-onnx/lib/libonnxruntime.so" "${LIBDIR}/"
fi

MODEL_DIR="${ROOT_DIR}/models/streaming"
for file in encoder.int8.onnx decoder.int8.onnx tokens.txt; do
  if [ -f "${MODEL_DIR}/${file}" ]; then
    install -m 644 "${MODEL_DIR}/${file}" /usr/share/droplet-voice-input/models/
  else
    echo "警告: 缺少模型文件 ${MODEL_DIR}/${file}"
  fi
done

PUNCT_MODEL_DIR="${ROOT_DIR}/models/punct-ct-transformer"
for file in model.int8.onnx tokens.json config.yaml; do
  if [ -f "${PUNCT_MODEL_DIR}/${file}" ]; then
    install -m 644 "${PUNCT_MODEL_DIR}/${file}" /usr/share/droplet-voice-input/models/punct-ct-transformer/
  else
    echo "警告: 缺少标点模型文件 ${PUNCT_MODEL_DIR}/${file}"
  fi
done

SILERO_MODEL_DIR="${ROOT_DIR}/models/silero-vad"
if [ -f "${SILERO_MODEL_DIR}/silero_vad.onnx" ]; then
  install -m 644 "${SILERO_MODEL_DIR}/silero_vad.onnx" /usr/share/droplet-voice-input/models/silero-vad/
else
  echo "警告: 缺少 Silero VAD 模型文件 ${SILERO_MODEL_DIR}/silero_vad.onnx"
fi

if command -v ldconfig >/dev/null 2>&1; then
  ldconfig
fi

if [ -n "${SUDO_USER:-}" ] && [ "${SUDO_USER}" != "root" ]; then
  USER_HOME="$(getent passwd "${SUDO_USER}" | cut -d: -f6)"
  if [ -n "${USER_HOME}" ]; then
    USER_CONFIG_DIR="${USER_HOME}/.config/vinput"
    if [ ! -f "${USER_CONFIG_DIR}/config.toml" ]; then
      install -d "${USER_CONFIG_DIR}"
      cp /usr/share/droplet-voice-input/config.toml.example "${USER_CONFIG_DIR}/config.toml"
      chown -R "${SUDO_USER}:${SUDO_USER}" "${USER_CONFIG_DIR}"
    fi
  fi
fi

echo "安装完成。"
echo "下一步:"
echo "  1) 重启  pkill fcitx5 && VINPUT_LOG=info fcitx5"
echo "  2) 在 fcitx5 配置中添加『水滴语音输入法』"
echo "  3) 运行设置界面: vinput-settings"
