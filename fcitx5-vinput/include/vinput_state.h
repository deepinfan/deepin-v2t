//! V-Input 状态管理

#ifndef VINPUT_STATE_H
#define VINPUT_STATE_H

#include <string>
#include <memory>

// 导入 vinput-core C API
extern "C" {
    #include "vinput_core.h"
}

namespace fcitx {

class VInputState {
public:
    VInputState();
    ~VInputState();

    // 启动/停止语音输入
    bool startCapture();
    void stopCapture();

    // 获取识别结果
    std::string getRecognitionResult();

    // 检查是否正在录音
    bool isRecording() const { return isRecording_; }

private:
    bool isRecording_;
    void* vinputCore_;  // vinput-core 实例句柄
};

}  // namespace fcitx

#endif  // VINPUT_STATE_H
