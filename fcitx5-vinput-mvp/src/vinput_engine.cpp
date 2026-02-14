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
    if (result == VInputVInputFFIResult::Success) {
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
        if (result == VInputVInputFFIResult::Success) {
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

    // Phase 1: 空格键触发语音输入
    if (keyEvent.key().check(FcitxKey_space)) {

        if (keyEvent.isRelease()) {
            // 空格键释放：停止录音
            FCITX_INFO() << "空格键释放 - 停止录音";

            if (vinput_core_initialized_) {
                VInputVInputEvent stop_event;
                stop_event.event_type = StopRecording;
                stop_event.data = nullptr;
                stop_event.data_len = 0;

                VInputVInputFFIResult result = vinput_core_send_event(&stop_event);
                if (result == VInputVInputFFIResult::Success) {
                    FCITX_INFO() << "停止录音事件已发送";

                    // 获取输入上下文
                    auto* inputContext = keyEvent.inputContext();

                    // 循环接收所有命令
                    while (true) {
                        VInputVInputCommand command;
                        result = vinput_core_try_recv_command(&command);

                        if (result == VInputVInputFFIResult::Success) {
                            // 处理命令
                            std::string text;
                            if (command.text != nullptr && command.text_len > 0) {
                                text = std::string(command.text, command.text_len);
                            }

                            switch (command.command_type) {
                                case VInputVInputCommandType::CommitText:
                                    FCITX_INFO() << "CommitText: " << text;
                                    inputContext->commitString(text);
                                    break;

                                case VInputVInputCommandType::ShowCandidate:
                                    FCITX_INFO() << "ShowCandidate: " << text;
                                    // TODO: 显示候选词列表
                                    // inputContext->inputPanel().setCandidateList(...);
                                    break;

                                case VInputVInputCommandType::HideCandidate:
                                    FCITX_INFO() << "HideCandidate";
                                    // TODO: 隐藏候选词列表
                                    // inputContext->inputPanel().reset();
                                    break;

                                case VInputVInputCommandType::Error:
                                    FCITX_ERROR() << "Error: " << text;
                                    // TODO: 显示错误消息
                                    break;

                                default:
                                    FCITX_WARN() << "Unknown command type: "
                                                << static_cast<int>(command.command_type);
                                    break;
                            }

                            // 释放命令资源
                            vinput_command_free(&command);

                        } else if (result == VInputVInputFFIResult::NoData) {
                            // 无更多命令
                            break;
                        } else {
                            FCITX_ERROR() << "接收命令失败: " << result;
                            break;
                        }
                    }

                    // 消费此按键事件
                    return keyEvent.filterAndAccept();
                } else {
                    FCITX_ERROR() << "发送停止录音事件失败: " << result;
                }
            }

        } else {
            // 空格键按下：开始录音
            FCITX_INFO() << "空格键按下 - 开始录音";

            if (vinput_core_initialized_) {
                VInputVInputEvent start_event;
                start_event.event_type = StartRecording;
                start_event.data = nullptr;
                start_event.data_len = 0;

                VInputVInputFFIResult result = vinput_core_send_event(&start_event);
                if (result == VInputVInputFFIResult::Success) {
                    FCITX_INFO() << "开始录音事件已发送";
                    // 消费此按键事件
                    return keyEvent.filterAndAccept();
                } else {
                    FCITX_ERROR() << "发送开始录音事件失败: " << result;
                }
            }
        }
    }

    // 其他按键：不处理
    return keyEvent.filterAndAccept();
}

} // namespace fcitx

// 注册 Fcitx5 插件
FCITX_ADDON_FACTORY(fcitx::VInputEngineFactory)
