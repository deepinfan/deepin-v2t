//! V-Input Fcitx5 引擎实现

#include "vinput_engine.h"
#include "vinput_state.h"
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/statusarea.h>
#include <fcitx/text.h>

namespace fcitx {

VInputEngine::VInputEngine(Instance *instance)
    : instance_(instance),
      state_(new VInputState()),
      isRecording_(false) {
    reloadConfig();
}

VInputEngine::~VInputEngine() {
    delete state_;
}

void VInputEngine::activate(const InputMethodEntry &entry, InputContextEvent &event) {
    // 激活输入法时的处理
    // 重置状态
    isRecording_ = false;
}

void VInputEngine::deactivate(const InputMethodEntry &entry, InputContextEvent &event) {
    // 失活时停止录音
    if (isRecording_) {
        stopRecording();
    }
}

void VInputEngine::reset(const InputMethodEntry &entry, InputContextEvent &event) {
    // 重置输入法状态
    if (isRecording_) {
        stopRecording();
    }
}

void VInputEngine::keyEvent(const InputMethodEntry &entry, KeyEvent &keyEvent) {
    // 处理键盘事件

    // Ctrl+Space 长按触发语音输入
    if (keyEvent.key().check(FcitxKey_space, KeyState::Ctrl)) {
        if (keyEvent.isRelease()) {
            // 按键释放 - 停止录音
            if (isRecording_) {
                stopRecording();
                keyEvent.filterAndAccept();
                return;
            }
        } else {
            // 按键按下 - 开始录音
            if (!isRecording_) {
                startRecording();
                keyEvent.filterAndAccept();
                return;
            }
        }
    }

    // 候选词选择 (1-9)
    if (keyEvent.key().isDigit() && !isRecording_) {
        // TODO: 处理候选词选择
        return;
    }
}

void VInputEngine::startRecording() {
    if (state_->startCapture()) {
        isRecording_ = true;

        // TODO: 显示录音指示器
        // 可以在状态栏显示 "🎤 Recording..."
    }
}

void VInputEngine::stopRecording() {
    if (!isRecording_) {
        return;
    }

    state_->stopCapture();
    isRecording_ = false;

    // 获取识别结果
    std::string result = state_->getRecognitionResult();

    if (!result.empty()) {
        // 显示候选词
        showCandidates(result);
    }
}

void VInputEngine::showCandidates(const std::string &text) {
    // TODO: 实现候选词显示
    // 暂时直接提交文本
    commitText(text);
}

void VInputEngine::commitText(const std::string &text) {
    auto *ic = instance_->mostRecentInputContext();
    if (ic) {
        ic->commitString(text);
    }
}

void VInputEngine::reloadConfig() {
    // TODO: 从配置文件加载设置
    readAsIni(config_, "conf/vinput.conf");
}

void VInputEngine::setConfig(const RawConfig &config) {
    config_.load(config);
    safeSaveAsIni(config_, "conf/vinput.conf");
}

// 工厂实现
class VInputEngineFactory : public AddonFactory {
public:
    AddonInstance *create(AddonManager *manager) override {
        return new VInputEngine(manager->instance());
    }
};

}  // namespace fcitx

// 注册插件
FCITX_ADDON_FACTORY_V2(vinput, fcitx::VInputEngineFactory)
