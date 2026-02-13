// FFI 测试程序 (C)
//
// 编译: gcc -o test_ffi test_ffi.c -L../target/release -lvinput_core
// 运行: LD_LIBRARY_PATH=../target/release ./test_ffi

#include "target/vinput_core.h"
#include <stdio.h>
#include <string.h>

int main() {
    printf("=== V-Input FFI 测试 ===\n\n");

    // 获取版本
    const char* version = vinput_core_version();
    printf("Version: %s\n\n", version);

    // 初始化
    printf("1. 初始化 Core...\n");
    enum VInputVInputFFIResult result = vinput_core_init();
    if (result == Success) {
        printf("   ✓ 初始化成功\n\n");
    } else {
        printf("   ✗ 初始化失败: %d\n", result);
        return 1;
    }

    // 发送事件
    printf("2. 发送 StartRecording 事件...\n");
    struct VInputVInputEvent event;
    event.event_type = StartRecording;
    event.data = NULL;
    event.data_len = 0;

    result = vinput_core_send_event(&event);
    if (result == Success) {
        printf("   ✓ 事件已发送\n\n");
    } else {
        printf("   ✗ 发送失败: %d\n", result);
    }

    // 尝试接收命令
    printf("3. 尝试接收命令...\n");
    struct VInputVInputCommand command;
    result = vinput_core_try_recv_command(&command);
    if (result == NoData) {
        printf("   ✓ 无命令（符合预期）\n\n");
    } else if (result == Success) {
        printf("   ✓ 接收到命令: %s\n", command.text);
        vinput_command_free(&command);
    } else {
        printf("   ✗ 接收失败: %d\n", result);
    }

    // 发送音频数据事件
    printf("4. 发送 AudioData 事件...\n");
    float audio_samples[512];
    memset(audio_samples, 0, sizeof(audio_samples));

    struct VInputVInputEvent audio_event;
    audio_event.event_type = AudioData;
    audio_event.data = (const uint8_t*)audio_samples;
    audio_event.data_len = sizeof(audio_samples);

    result = vinput_core_send_event(&audio_event);
    if (result == Success) {
        printf("   ✓ 音频事件已发送 (%zu bytes)\n\n", sizeof(audio_samples));
    } else {
        printf("   ✗ 发送失败: %d\n", result);
    }

    // 关闭
    printf("5. 关闭 Core...\n");
    result = vinput_core_shutdown();
    if (result == Success) {
        printf("   ✓ 关闭成功\n\n");
    } else {
        printf("   ✗ 关闭失败: %d\n", result);
        return 1;
    }

    printf("✅ FFI 测试完成！\n");
    printf("\n💡 Phase 0 验证:\n");
    printf("   - C 可以 #include vinput_core.h\n");
    printf("   - FFI 函数可正常调用\n");
    printf("   - 类型定义兼容\n");
    printf("   - Phase 1 将实现完整功能\n");

    return 0;
}
