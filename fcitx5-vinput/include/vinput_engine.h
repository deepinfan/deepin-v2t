//! V-Input Fcitx5 输入法引擎

#ifndef VINPUT_ENGINE_H
#define VINPUT_ENGINE_H

#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <fcitx/addonmanager.h>

namespace fcitx {

class VInputState;

class VInputEngine : public InputMethodEngineV3 {
public:
    VInputEngine(Instance *instance);
    ~VInputEngine();

    // InputMethodEngine 接口实现
    void keyEvent(const InputMethodEntry &entry, KeyEvent &keyEvent) override;
    void activate(const InputMethodEntry &entry, InputContextEvent &event) override;
    void deactivate(const InputMethodEntry &entry, InputContextEvent &event) override;
    void reset(const InputMethodEntry &entry, InputContextEvent &event) override;

    // 配置相关
    void reloadConfig() override;
    const Configuration *getConfig() const override { return &config_; }
    void setConfig(const RawConfig &config) override;

    // 获取工厂实例
    FCITX_ADDON_FACTORY(VInputEngineFactory);

private:
    Instance *instance_;
    Configuration config_;

    // V-Input 核心状态
    VInputState *state_;

    // 音频捕获状态
    bool isRecording_;

    // 快捷键处理
    void handleVoiceInputKey(KeyEvent &keyEvent);
    void startRecording();
    void stopRecording();

    // 候选词处理
    void showCandidates(const std::string &text);
    void commitText(const std::string &text);
};

}  // namespace fcitx

#endif  // VINPUT_ENGINE_H
