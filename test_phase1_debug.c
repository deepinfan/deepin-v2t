/*
 * Phase 1 集成测试 - 详细调试版本
 */

#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include "target/vinput_core.h"

int main() {
    printf("=== V-Input Phase 1 调试测试 ===\n\n");

    // 初始化
    printf("1. 初始化...\n");
    VInputVInputFFIResult result = vinput_core_init();
    printf("   Result: %d\n", result);
    if (result != Success) {
        return 1;
    }

    // 开始录音
    printf("\n2. 开始录音...\n");
    VInputVInputEvent start_event;
    start_event.event_type = StartRecording;
    start_event.data = NULL;
    start_event.data_len = 0;
    result = vinput_core_send_event(&start_event);
    printf("   Result: %d\n", result);

    printf("   等待 100ms...\n");
    usleep(100000);  // 100ms

    // 停止录音
    printf("\n3. 停止录音...\n");
    VInputVInputEvent stop_event;
    stop_event.event_type = StopRecording;
    stop_event.data = NULL;
    stop_event.data_len = 0;
    result = vinput_core_send_event(&stop_event);
    printf("   Result: %d\n", result);

    printf("   等待 100ms...\n");
    usleep(100000);  // 100ms

    // 尝试多次接收命令
    printf("\n4. 尝试接收命令 (多次尝试)...\n");
    for (int i = 0; i < 5; i++) {
        printf("   尝试 #%d: ", i + 1);
        VInputVInputCommand command;
        memset(&command, 0, sizeof(command));
        
        result = vinput_core_try_recv_command(&command);
        printf("result=%d ", result);
        
        if (result == Success) {
            printf("SUCCESS!\n");
            printf("      command_type=%d\n", command.command_type);
            printf("      text_len=%zu\n", command.text_len);
            if (command.text != NULL && command.text_len > 0) {
                printf("      text='%.*s'\n", (int)command.text_len, command.text);
            } else {
                printf("      text=NULL\n");
            }
            vinput_command_free(&command);
            break;
        } else if (result == NoData) {
            printf("NoData\n");
        } else {
            printf("Error %d\n", result);
        }
        
        usleep(50000);  // 50ms
    }

    // 关闭
    printf("\n5. 关闭...\n");
    result = vinput_core_shutdown();
    printf("   Result: %d\n", result);

    printf("\n=== 测试完成 ===\n");
    return 0;
}
