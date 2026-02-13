/*
 * Phase 1 集成测试
 * 测试 FFI 接口的基本功能
 */

#include <stdio.h>
#include <string.h>
#include "target/vinput_core.h"

void print_result(const char* test_name, VInputVInputFFIResult result) {
    printf("[%s] Result: %d ", test_name, result);
    if (result == Success) {
        printf("✓ Success\n");
    } else {
        printf("✗ Failed\n");
    }
}

int main() {
    printf("=== V-Input Phase 1 集成测试 ===\n\n");

    // 测试 1: 初始化
    printf("1. 测试初始化...\n");
    VInputVInputFFIResult result = vinput_core_init();
    print_result("init", result);
    if (result != Success) {
        return 1;
    }

    // 测试 2: 获取版本
    printf("\n2. 测试版本信息...\n");
    const char* version = vinput_core_version();
    printf("   Version: %s\n", version);

    // 测试 3: 发送 StartRecording 事件
    printf("\n3. 测试开始录音...\n");
    VInputVInputEvent start_event;
    start_event.event_type = StartRecording;
    start_event.data = NULL;
    start_event.data_len = 0;
    result = vinput_core_send_event(&start_event);
    print_result("start_recording", result);

    // 测试 4: 发送 StopRecording 事件
    printf("\n4. 测试停止录音...\n");
    VInputVInputEvent stop_event;
    stop_event.event_type = StopRecording;
    stop_event.data = NULL;
    stop_event.data_len = 0;
    result = vinput_core_send_event(&stop_event);
    print_result("stop_recording", result);

    // 测试 5: 尝试接收命令
    printf("\n5. 测试接收命令...\n");
    VInputVInputCommand command;
    result = vinput_core_try_recv_command(&command);
    
    if (result == Success) {
        printf("   ✓ 收到命令: type=%d\n", command.command_type);
        printf("   ✓ 文本: %.*s\n", (int)command.text_len, command.text);
        
        // 释放命令
        vinput_command_free(&command);
        printf("   ✓ 命令已释放\n");
    } else if (result == NoData) {
        printf("   ℹ 无命令数据 (NoData)\n");
    } else {
        printf("   ✗ 接收失败: %d\n", result);
    }

    // 测试 6: 关闭
    printf("\n6. 测试关闭...\n");
    result = vinput_core_shutdown();
    print_result("shutdown", result);

    printf("\n=== 测试完成 ===\n");
    return 0;
}
