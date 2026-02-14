/*
 * V-Input Engine for Fcitx5
 * Complete implementation with FFI integration
 */

#ifndef VINPUT_ENGINE_H
#define VINPUT_ENGINE_H

#include <fcitx/inputmethodengine.h>
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/instance.h>
#include <fcitx/inputcontext.h>
#include <memory>

extern "C" {
#include "vinput_core.h"
}

namespace fcitx {

/**
 * V-Input 输入法引擎
 *
 * 完整实现：VAD + ASR + ITN + 候选词
 */
class VInputEngine : public InputMethodEngine {
public:
    VInputEngine(Instance* instance);
    ~VInputEngine() override;

    // 输入法生命周期
    void activate(const InputMethodEntry& entry, InputContextEvent& event) override;
    void deactivate(const InputMethodEntry& entry, InputContextEvent& event) override;
    void reset(const InputMethodEntry& entry, InputContextEvent& event) override;

    // 按键处理
    void keyEvent(const InputMethodEntry& entry, KeyEvent& keyEvent) override;

    // 获取子配置
    const Configuration* getConfig() const override { return nullptr; }
    void setConfig(const RawConfig&) override {}

private:
    Instance* instance_;
    bool vinput_core_initialized_;
    bool is_recording_;

    void startRecording();
    void stopRecording();
    void processCommands(InputContext* ic);
};

/**
 * Fcitx5 插件工厂
 */
class VInputEngineFactory : public AddonFactory {
public:
    AddonInstance* create(AddonManager* manager) override {
        return new VInputEngine(manager->instance());
    }
};

} // namespace fcitx

#endif // VINPUT_ENGINE_H
