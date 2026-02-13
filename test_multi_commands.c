/*
 * 多命令处理测试
 */

#include <stdio.h>
#include <string.h>
#include "target/vinput_core.h"

const char* command_type_name(VInputVInputCommandType type) {
    switch(type) {
        case CommitText: return "CommitText";
        case ShowCandidate: return "ShowCandidate";
        case HideCandidate: return "HideCandidate";
        case Error: return "Error";
        default: return "Unknown";
    }
}

int main() {
    printf("=== V-Input 多命令处理测试 ===\n\n");

    // 初始化
    printf("1. 初始化 Core...\n");
    if (vinput_core_init() != Success) {
        printf("   ✗ 初始化失败\n");
        return 1;
    }
    printf("   ✓ 初始化成功\n");

    // 模拟录音流程
    printf("\n2. 开始录音...\n");
    VInputVInputEvent start_event = {StartRecording, NULL, 0};
    vinput_core_send_event(&start_event);
    printf("   ✓ 录音已开始\n");

    printf("\n3. 停止录音...\n");
    VInputVInputEvent stop_event = {StopRecording, NULL, 0};
    vinput_core_send_event(&stop_event);
    printf("   ✓ 录音已停止\n");

    // 接收所有命令
    printf("\n4. 接收命令序列:\n");
    int cmd_count = 0;
    while (1) {
        VInputVInputCommand command;
        VInputVInputFFIResult result = vinput_core_try_recv_command(&command);
        
        if (result == Success) {
            cmd_count++;
            printf("   [命令 #%d]\n", cmd_count);
            printf("      类型: %s\n", command_type_name(command.command_type));
            
            if (command.text != NULL && command.text_len > 0) {
                printf("      文本: %.*s\n", (int)command.text_len, command.text);
            } else {
                printf("      文本: (无)\n");
            }
            
            vinput_command_free(&command);
        } else if (result == NoData) {
            printf("   ✓ 所有命令已接收 (共 %d 个)\n", cmd_count);
            break;
        } else {
            printf("   ✗ 接收失败: %d\n", result);
            break;
        }
    }

    // 关闭
    printf("\n5. 关闭 Core...\n");
    vinput_core_shutdown();
    printf("   ✓ 关闭成功\n");

    printf("\n=== 测试完成 ===\n");
    printf("验证结果: %s\n", cmd_count == 3 ? "✓ PASS" : "✗ FAIL");
    
    return cmd_count == 3 ? 0 : 1;
}
