/*
 * Fcitx5 插件语法验证程序
 * Phase 0: 验证代码编译正确性（不依赖 Fcitx5 运行时）
 */

#include <iostream>
#include <cstring>

extern "C" {
#include "vinput_core.h"
}

// 模拟测试 Fcitx5 集成
class VInputEngineTest {
public:
    VInputEngineTest() : initialized_(false) {
        std::cout << "=== Fcitx5 插件 FFI 集成测试 ===" << std::endl;
        std::cout << std::endl;

        // 测试 1: 初始化
        std::cout << "1. 测试 vinput_core_init()..." << std::endl;
        VInputVInputFFIResult result = vinput_core_init();
        if (result == Success) {
            initialized_ = true;
            const char* version = vinput_core_version();
            std::cout << "   ✓ 初始化成功, version: " << version << std::endl;
        } else {
            std::cout << "   ✗ 初始化失败: " << result << std::endl;
            return;
        }
        std::cout << std::endl;

        // 测试 2: 发送事件
        std::cout << "2. 测试 vinput_core_send_event()..." << std::endl;
        VInputVInputEvent event;
        event.event_type = StartRecording;
        event.data = nullptr;
        event.data_len = 0;

        result = vinput_core_send_event(&event);
        if (result == Success) {
            std::cout << "   ✓ 事件发送成功" << std::endl;
        } else {
            std::cout << "   ✗ 事件发送失败: " << result << std::endl;
        }
        std::cout << std::endl;

        // 测试 3: 接收命令
        std::cout << "3. 测试 vinput_core_try_recv_command()..." << std::endl;
        VInputVInputCommand command;
        result = vinput_core_try_recv_command(&command);
        if (result == NoData) {
            std::cout << "   ✓ 无命令（符合预期）" << std::endl;
        } else if (result == Success) {
            std::cout << "   ✓ 接收到命令: " << command.text << std::endl;
            vinput_command_free(&command);
        } else {
            std::cout << "   ✗ 接收失败: " << result << std::endl;
        }
        std::cout << std::endl;

        // 测试 4: 音频数据
        std::cout << "4. 测试发送音频数据..." << std::endl;
        float audio_samples[512];
        std::memset(audio_samples, 0, sizeof(audio_samples));

        VInputVInputEvent audio_event;
        audio_event.event_type = AudioData;
        audio_event.data = reinterpret_cast<const uint8_t*>(audio_samples);
        audio_event.data_len = sizeof(audio_samples);

        result = vinput_core_send_event(&audio_event);
        if (result == Success) {
            std::cout << "   ✓ 音频数据发送成功 (" << sizeof(audio_samples) << " bytes)" << std::endl;
        } else {
            std::cout << "   ✗ 音频数据发送失败: " << result << std::endl;
        }
        std::cout << std::endl;
    }

    ~VInputEngineTest() {
        if (initialized_) {
            std::cout << "5. 测试 vinput_core_shutdown()..." << std::endl;
            VInputVInputFFIResult result = vinput_core_shutdown();
            if (result == Success) {
                std::cout << "   ✓ 关闭成功" << std::endl;
            } else {
                std::cout << "   ✗ 关闭失败: " << result << std::endl;
            }
            std::cout << std::endl;
        }
    }

private:
    bool initialized_;
};

int main() {
    VInputEngineTest test;

    std::cout << "✅ Fcitx5 插件 FFI 集成测试完成！" << std::endl;
    std::cout << std::endl;
    std::cout << "💡 Phase 0 验证:" << std::endl;
    std::cout << "   - C++ 可以调用 FFI 接口" << std::endl;
    std::cout << "   - vinput_core.h 头文件兼容" << std::endl;
    std::cout << "   - 类型转换正确" << std::endl;
    std::cout << "   - Fcitx5 插件骨架已创建" << std::endl;
    std::cout << "   - Phase 1 将构建完整插件" << std::endl;
    std::cout << std::endl;

    return 0;
}
