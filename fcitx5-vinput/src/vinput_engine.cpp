/*
 * V-Input Engine for Fcitx5
 * Complete implementation with FFI integration
 */

#include "vinput_engine.h"
#include <fcitx-utils/log.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputpanel.h>
#include <fcitx/text.h>

namespace fcitx {

// 全局 VInputEngine 实例指针（用于回调）
static VInputEngine* g_vinput_engine_instance = nullptr;

VInputEngine::VInputEngine(Instance* instance)
    : instance_(instance), vinput_core_initialized_(false), is_recording_(false) {

    FCITX_INFO() << "V-Input Engine: 初始化";

    // 保存全局实例指针
    g_vinput_engine_instance = this;

    // 初始化 V-Input Core (FFI)
    VInputVInputFFIResult result = vinput_core_init();
    if (result == VInputVInputFFIResult::Success) {
        vinput_core_initialized_ = true;
        const char* version = vinput_core_version();
        FCITX_INFO() << "V-Input Core 初始化成功, version: " << version;

        // 注册命令回调函数（替代轮询机制）
        result = vinput_core_register_callback(&VInputEngine::handleCommand);
        if (result == VInputVInputFFIResult::Success) {
            FCITX_INFO() << "✅ 命令回调注册成功（零延迟自动上屏）";
        } else {
            FCITX_ERROR() << "❌ 命令回调注册失败: " << result;
        }
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
    FCITX_DEBUG() << "V-Input: keyEvent - " << keyEvent.key().toString()
                  << ", isRelease=" << keyEvent.isRelease()
                  << ", recording=" << is_recording_;

    // 空格键触发语音输入（Push to Toggle）
    // 第一次按下：开始录音
    // 第二次按下：停止录音并识别
    if (keyEvent.key().check(FcitxKey_space)) {
        // 只处理按下事件，忽略释放事件
        if (keyEvent.isRelease()) {
            FCITX_DEBUG() << "忽略空格键释放事件";
            keyEvent.filterAndAccept();
            return;
        }

        // 空格键按下：切换录音状态
        if (is_recording_) {
            // 当前正在录音 → 停止录音
            FCITX_INFO() << "空格键按下 - 停止录音并识别";
            stopRecording();
        } else {
            // 当前未录音 → 开始录音
            FCITX_INFO() << "空格键按下 - 开始录音";
            startRecording();
        }

        keyEvent.filterAndAccept();
        return;
    }

    // Ctrl+Z: 撤销
    if (keyEvent.key().check(FcitxKey_z, KeyState::Ctrl)) {
        if (!keyEvent.isRelease()) {
            FCITX_INFO() << "Ctrl+Z - 撤销";
            requestUndo();
        }
        keyEvent.filterAndAccept();
        return;
    }

    // Ctrl+Y: 重试
    if (keyEvent.key().check(FcitxKey_y, KeyState::Ctrl)) {
        if (!keyEvent.isRelease()) {
            FCITX_INFO() << "Ctrl+Y - 重试";
            requestRedo();
        }
        keyEvent.filterAndAccept();
        return;
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

        // 显示录音指示器
        auto* ic = instance_->mostRecentInputContext();
        if (ic) {
            auto& inputPanel = ic->inputPanel();
            inputPanel.setAuxUp(Text("🎤 录音中..."));
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
        }
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

        // 清除录音指示器，显示识别中状态
        auto* ic = instance_->mostRecentInputContext();
        if (ic) {
            auto& inputPanel = ic->inputPanel();
            inputPanel.setAuxUp(Text("🔵 识别中..."));
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);

            processCommands(ic);

            // 识别完成后清除指示器
            inputPanel.reset();
            ic->updateUserInterface(UserInterfaceComponent::InputPanel);
        }
    } else {
        FCITX_ERROR() << "停止录音失败: " << result;
    }
}

void VInputEngine::handleCommand(const VInputVInputCommand* command) {
    if (!g_vinput_engine_instance) {
        FCITX_ERROR() << "VInputEngine 实例不存在";
        return;
    }

    if (!command) {
        FCITX_ERROR() << "命令指针为空";
        return;
    }

    // 获取当前输入上下文
    auto* ic = g_vinput_engine_instance->instance_->mostRecentInputContext();
    if (!ic) {
        FCITX_WARN() << "没有活动的输入上下文";
        return;
    }

    // 处理命令
    std::string text;
    if (command->text != nullptr && command->text_len > 0) {
        text = std::string(command->text, command->text_len);
    }

    switch (command->command_type) {
        case VInputVInputCommandType::CommitText:
            FCITX_INFO() << "✨ 回调上屏: " << text;
            ic->commitString(text);
            break;

        case VInputVInputCommandType::ShowCandidate:
            FCITX_DEBUG() << "ShowCandidate: " << text;
            // TODO: 显示候选词列表
            break;

        case VInputVInputCommandType::HideCandidate:
            FCITX_DEBUG() << "HideCandidate";
            // TODO: 隐藏候选词列表
            break;

        case VInputVInputCommandType::Error:
            FCITX_ERROR() << "Error: " << text;
            // 显示错误消息
            {
                auto& inputPanel = ic->inputPanel();
                inputPanel.setAuxUp(Text("❌ " + text));
                ic->updateUserInterface(UserInterfaceComponent::InputPanel);
            }
            break;

        default:
            FCITX_WARN() << "Unknown command type: "
                        << static_cast<int>(command->command_type);
            break;
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

                case VInputVInputCommandType::UndoText:
                    FCITX_INFO() << "UndoText: " << text;
                    // 删除指定长度的文本
                    for (size_t i = 0; i < text.length(); ++i) {
                        ic->forwardKey(Key(FcitxKey_BackSpace));
                    }
                    break;

                case VInputVInputCommandType::RedoText:
                    FCITX_INFO() << "RedoText: " << text;
                    // 重新提交文本
                    ic->commitString(text);
                    break;

                case VInputVInputCommandType::Error:
                    FCITX_ERROR() << "Error: " << text;
                    // 显示错误消息
                    {
                        auto& inputPanel = ic->inputPanel();
                        inputPanel.setAuxUp(Text("❌ " + text));
                        ic->updateUserInterface(UserInterfaceComponent::InputPanel);
                    }
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

void VInputEngine::requestUndo() {
    if (!vinput_core_initialized_) {
        FCITX_ERROR() << "V-Input Core 未初始化";
        return;
    }

    // 发送撤销请求事件
    VInputVInputEvent event;
    event.event_type = UndoRequest;
    event.data = nullptr;
    event.data_len = 0;

    VInputVInputFFIResult result = vinput_core_send_event(&event);
    if (result == VInputVInputFFIResult::Success) {
        FCITX_INFO() << "撤销请求已发送";

        // 处理撤销命令
        auto* ic = instance_->mostRecentInputContext();
        if (ic) {
            processCommands(ic);
        }
    } else {
        FCITX_ERROR() << "发送撤销请求失败: " << result;
    }
}

void VInputEngine::requestRedo() {
    if (!vinput_core_initialized_) {
        FCITX_ERROR() << "V-Input Core 未初始化";
        return;
    }

    // 发送重试请求事件
    VInputVInputEvent event;
    event.event_type = RedoRequest;
    event.data = nullptr;
    event.data_len = 0;

    VInputVInputFFIResult result = vinput_core_send_event(&event);
    if (result == VInputVInputFFIResult::Success) {
        FCITX_INFO() << "重试请求已发送";

        // 处理重试命令
        auto* ic = instance_->mostRecentInputContext();
        if (ic) {
            processCommands(ic);
        }
    } else {
        FCITX_ERROR() << "发送重试请求失败: " << result;
    }
}

} // namespace fcitx

// 注册插件
FCITX_ADDON_FACTORY(fcitx::VInputEngineFactory)
