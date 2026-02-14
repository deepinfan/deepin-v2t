//! V-Input 状态实现

#include "vinput_state.h"
#include <iostream>

namespace fcitx {

VInputState::VInputState()
    : isRecording_(false),
      vinputCore_(nullptr) {
    // TODO: 初始化 vinput-core
    // vinputCore_ = vinput_core_new();
}

VInputState::~VInputState() {
    if (vinputCore_) {
        // TODO: 释放 vinput-core
        // vinput_core_free(vinputCore_);
    }
}

bool VInputState::startCapture() {
    if (isRecording_) {
        return false;
    }

    // TODO: 调用 vinput-core 开始音频捕获
    // vinput_core_start_capture(vinputCore_);

    isRecording_ = true;
    std::cout << "V-Input: 开始录音..." << std::endl;
    return true;
}

void VInputState::stopCapture() {
    if (!isRecording_) {
        return;
    }

    // TODO: 调用 vinput-core 停止音频捕获
    // vinput_core_stop_capture(vinputCore_);

    isRecording_ = false;
    std::cout << "V-Input: 停止录音" << std::endl;
}

std::string VInputState::getRecognitionResult() {
    if (!vinputCore_) {
        return "";
    }

    // TODO: 从 vinput-core 获取识别结果
    // const char* result = vinput_core_get_result(vinputCore_);
    // if (result) {
    //     return std::string(result);
    // }

    // 临时返回测试文本
    return "你好世界";
}

}  // namespace fcitx
