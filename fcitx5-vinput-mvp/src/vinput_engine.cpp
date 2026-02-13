/*
 * V-Input Engine for Fcitx5
 * Phase 0: Basic FFI Integration
 */

#include "vinput_engine.h"
#include <fcitx-utils/log.h>
#include <fcitx/inputcontext.h>

namespace fcitx {

VInputEngine::VInputEngine(Instance* instance)
    : instance_(instance), vinput_core_initialized_(false) {

    FCITX_INFO() << "V-Input Engine: 初始化";

    // 初始化 V-Input Core (FFI)
    VInputVInputFFIResult result = vinput_core_init();
    if (result == Success) {
        vinput_core_initialized_ = true;
        const char* version = vinput_core_version();
        FCITX_INFO() << "V-Input Core 初始化成功, version: " << version;
    } else {
        FCITX_ERROR() << "V-Input Core 初始化失败: " << result;
    }

    // Phase 1: 这里会初始化音频捕获、VAD、ASR 等组件
}

VInputEngine::~VInputEngine() {
    FCITX_INFO() << "V-Input Engine: 关闭";

    // 关闭 V-Input Core (FFI)
    if (vinput_core_initialized_) {
        VInputVInputFFIResult result = vinput_core_shutdown();
        if (result == Success) {
            FCITX_INFO() << "V-Input Core 关闭成功";
        } else {
            FCITX_ERROR() << "V-Input Core 关闭失败: " << result;
        }
        vinput_core_initialized_ = false;
    }
}

void VInputEngine::activate(const InputMethodEntry& entry, InputContextEvent& event) {
    FCITX_DEBUG() << "V-Input: activate";

    // Phase 0: 仅记录
    // Phase 1: 启动音频捕获
    if (vinput_core_initialized_) {
        // 示例：发送 StartRecording 事件
        // VInputVInputEvent start_event;
        // start_event.event_type = StartRecording;
        // start_event.data = nullptr;
        // start_event.data_len = 0;
        // vinput_core_send_event(&start_event);
    }
}

void VInputEngine::deactivate(const InputMethodEntry& entry, InputContextEvent& event) {
    FCITX_DEBUG() << "V-Input: deactivate";

    // Phase 0: 仅记录
    // Phase 1: 停止音频捕获
    if (vinput_core_initialized_) {
        // 示例：发送 StopRecording 事件
        // VInputVInputEvent stop_event;
        // stop_event.event_type = StopRecording;
        // stop_event.data = nullptr;
        // stop_event.data_len = 0;
        // vinput_core_send_event(&stop_event);
    }
}

void VInputEngine::reset(const InputMethodEntry& entry, InputContextEvent& event) {
    FCITX_DEBUG() << "V-Input: reset";

    // Phase 0: 仅记录
    // Phase 1: 重置识别状态
}

void VInputEngine::keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) {
    FCITX_DEBUG() << "V-Input: keyEvent - "
                  << keyEvent.key().toString();

    // Phase 0: 基本按键处理
    // 示例：空格键触发语音输入（未实现）
    if (keyEvent.key().check(FcitxKey_space) &&
        keyEvent.isRelease() == false) {

        FCITX_INFO() << "检测到空格键（Phase 0 暂不处理）";

        // Phase 1: 实际语音识别流程：
        // 1. 空格键按下：开始录音
        // 2. 空格键释放：停止录音，等待识别结果
        // 3. 接收命令：提交文本到 InputContext

        // 示例代码（Phase 1）：
        // auto* inputContext = keyEvent.inputContext();
        // VInputVInputCommand command;
        // if (vinput_core_try_recv_command(&command) == Success) {
        //     inputContext->commitString(command.text);
        //     vinput_command_free(&command);
        // }
    }

    // Phase 0: 放过所有按键
    return keyEvent.filterAndAccept();
}

} // namespace fcitx

// 注册 Fcitx5 插件
FCITX_ADDON_FACTORY(fcitx::VInputEngineFactory)
