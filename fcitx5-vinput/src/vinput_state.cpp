//! V-Input 状态实现

#include "vinput_state.h"
#include <iostream>

namespace fcitx {

VInputState::VInputState()
    : isRecording_(false),
      vinputCore_(nullptr) {
    // 初始化 vinput-core
    auto result = vinput_core_init();
    if (result == VInputVInputFFIResult::Success) {
        std::cout << "V-Input Core 初始化成功" << std::endl;
        std::cout << "V-Input Core 版本: " << vinput_core_version() << std::endl;
    } else {
        std::cerr << "V-Input Core 初始化失败: " << static_cast<int>(result) << std::endl;
    }
}

VInputState::~VInputState() {
    // 释放 vinput-core
    vinput_core_shutdown();
}

bool VInputState::startCapture() {
    if (isRecording_) {
        return false;
    }

    // 发送 StartRecording 事件
    VInputVInputEvent event = {
        .event_type = VInputVInputEventType::StartRecording,
        .data = nullptr,
        .data_len = 0
    };

    auto result = vinput_core_send_event(&event);
    if (result == VInputVInputFFIResult::Success) {
        isRecording_ = true;
        std::cout << "V-Input: 开始录音..." << std::endl;
        return true;
    } else {
        std::cerr << "V-Input: 启动录音失败: " << static_cast<int>(result) << std::endl;
        return false;
    }
}

void VInputState::stopCapture() {
    if (!isRecording_) {
        return;
    }

    // 发送 StopRecording 事件
    VInputVInputEvent event = {
        .event_type = VInputVInputEventType::StopRecording,
        .data = nullptr,
        .data_len = 0
    };

    auto result = vinput_core_send_event(&event);
    if (result == VInputVInputFFIResult::Success) {
        isRecording_ = false;
        std::cout << "V-Input: 停止录音" << std::endl;
    } else {
        std::cerr << "V-Input: 停止录音失败: " << static_cast<int>(result) << std::endl;
    }
}

std::string VInputState::getRecognitionResult() {
    // 轮询命令队列，获取识别结果
    VInputVInputCommand command;
    std::string result;

    while (true) {
        auto res = vinput_core_try_recv_command(&command);

        if (res == VInputVInputFFIResult::NoData) {
            // 没有更多命令
            break;
        }

        if (res != VInputVInputFFIResult::Success) {
            std::cerr << "V-Input: 获取命令失败: " << static_cast<int>(res) << std::endl;
            break;
        }

        // 处理命令
        switch (command.command_type) {
            case VInputVInputCommandType::CommitText:
                if (command.text != nullptr) {
                    result = std::string(command.text);
                    std::cout << "V-Input: 收到文本: " << result << std::endl;
                }
                break;

            case VInputVInputCommandType::ShowCandidate:
                // 暂时忽略（未来用于候选词显示）
                break;

            case VInputVInputCommandType::HideCandidate:
                // 暂时忽略
                break;

            case VInputVInputCommandType::Error:
                if (command.text != nullptr) {
                    std::cerr << "V-Input: 错误: " << command.text << std::endl;
                }
                break;
        }

        // 释放命令资源
        vinput_command_free(&command);
    }

    return result;
}

}  // namespace fcitx
