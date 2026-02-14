/*
 * V-Input Engine for Fcitx5
 * Complete implementation with FFI integration
 */

#include "vinput_engine.h"
#include <fcitx-utils/log.h>
#include <fcitx/inputcontext.h>

namespace fcitx {

VInputEngine::VInputEngine(Instance* instance)
    : instance_(instance), vinput_core_initialized_(false), is_recording_(false) {

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
    is_recording_ = false;
}

void VInputEngine::deactivate(const InputMethodEntry& entry, InputContextEvent& event) {
    FCITX_DEBUG() << "V-Input: deactivate";

    // 失活时停止录音
    if (is_recording_) {
        stopRecording();
    }
}

void VInputEngine::reset(const InputMethodEntry& entry, InputContextEvent& event) {
    FCITX_DEBUG() << "V-Input: reset";

    // 重置时停止录音
    if (is_recording_) {
        stopRecording();
    }
}

void VInputEngine::keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) {
    FCITX_DEBUG() << "V-Input: keyEvent - " << keyEvent.key().toString();

    // 空格键触发语音输入（Press to Talk）
    if (keyEvent.key().check(FcitxKey_space)) {
        if (keyEvent.isRelease()) {
            // 空格键释放：停止录音
            FCITX_INFO() << "空格键释放 - 停止录音";
            stopRecording();
            keyEvent.filterAndAccept();
            return;
        } else {
            // 空格键按下：开始录音
            FCITX_INFO() << "空格键按下 - 开始录音";
            startRecording();
            keyEvent.filterAndAccept();
            return;
        }
    }
}

void VInputEngine::startRecording() {
    if (!vinput_core_initialized_) {
        FCITX_ERROR() << "V-Input Core 未初始化";
        return;
    }

    if (is_recording_) {
        FCITX_WARN() << "已经在录音中";
        return;
    }

    // 发送 StartRecording 事件
    VInputVInputEvent event;
    event.event_type = StartRecording;
    event.data = nullptr;
    event.data_len = 0;

    VInputVInputFFIResult result = vinput_core_send_event(&event);
    if (result == VInputVInputFFIResult::Success) {
        is_recording_ = true;
        FCITX_INFO() << "开始录音成功";

        // TODO: 显示录音指示器
        // 可以在状态栏显示 "🎤 Recording..."
    } else {
        FCITX_ERROR() << "开始录音失败: " << result;
    }
}

void VInputEngine::stopRecording() {
    if (!vinput_core_initialized_) {
        FCITX_ERROR() << "V-Input Core 未初始化";
        return;
    }

    if (!is_recording_) {
        FCITX_WARN() << "没有在录音";
        return;
    }

    // 发送 StopRecording 事件
    VInputVInputEvent event;
    event.event_type = StopRecording;
    event.data = nullptr;
    event.data_len = 0;

    VInputVInputFFIResult result = vinput_core_send_event(&event);
    if (result == VInputVInputFFIResult::Success) {
        is_recording_ = false;
        FCITX_INFO() << "停止录音成功";

        // 获取输入上下文
        auto* ic = instance_->mostRecentInputContext();
        if (ic) {
            processCommands(ic);
        }
    } else {
        FCITX_ERROR() << "停止录音失败: " << result;
    }
}

void VInputEngine::processCommands(InputContext* ic) {
    if (!ic) {
        return;
    }

    // 循环接收所有命令
    while (true) {
        VInputVInputCommand command;
        VInputVInputFFIResult result = vinput_core_try_recv_command(&command);

        if (result == VInputVInputFFIResult::Success) {
            // 处理命令
            std::string text;
            if (command.text != nullptr && command.text_len > 0) {
                text = std::string(command.text, command.text_len);
            }

            switch (command.command_type) {
                case VInputVInputCommandType::CommitText:
                    FCITX_INFO() << "CommitText: " << text;
                    ic->commitString(text);
                    break;

                case VInputVInputCommandType::ShowCandidate:
                    FCITX_INFO() << "ShowCandidate: " << text;
                    // TODO: 显示候选词列表
                    // ic->inputPanel().setCandidateList(...);
                    break;

                case VInputVInputCommandType::HideCandidate:
                    FCITX_INFO() << "HideCandidate";
                    // TODO: 隐藏候选词列表
                    // ic->inputPanel().reset();
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
            // 没有更多命令
            break;
        } else {
            FCITX_ERROR() << "接收命令失败: " << result;
            break;
        }
    }
}

} // namespace fcitx

// 注册插件
FCITX_ADDON_FACTORY(fcitx::VInputEngineFactory)
