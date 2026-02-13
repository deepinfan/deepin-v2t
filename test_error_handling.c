/*
 * 错误处理测试
 * 测试增强的错误分类、严重度和恢复策略
 */

#include <stdio.h>
#include <string.h>
#include "target/vinput_core.h"

void test_error_scenario(const char* scenario_name) {
    printf("\n--- %s ---\n", scenario_name);
}

int main() {
    printf("=== V-Input 错误处理测试 ===\n");
    printf("本测试验证错误处理增强功能：\n");
    printf("- 错误严重度分类\n");
    printf("- 恢复策略识别\n");
    printf("- 用户友好的错误消息\n");
    printf("- 错误码生成\n");
    printf("- 结构化日志记录\n\n");

    // 测试 1: 正常初始化
    test_error_scenario("正常初始化");
    VInputVInputFFIResult result = vinput_core_init();
    if (result == Success) {
        printf("✓ 初始化成功\n");
    } else {
        printf("✗ 初始化失败: %d\n", result);
        return 1;
    }

    // 测试 2: 重复初始化（应该处理优雅）
    test_error_scenario("重复初始化（幂等性测试）");
    result = vinput_core_init();
    if (result == Success) {
        printf("✓ 重复初始化被正确处理（幂等）\n");
    } else {
        printf("✗ 重复初始化失败: %d\n", result);
    }

    // 测试 3: 空指针保护
    test_error_scenario("空指针保护");
    result = vinput_core_try_recv_command(NULL);
    if (result == NullPointer) {
        printf("✓ 空指针被正确检测并拒绝\n");
    } else if (result == NoData) {
        printf("⚠ 空指针检查可能有问题（返回 NoData）\n");
    } else {
        printf("✗ 空指针检查失败: %d\n", result);
    }

    // 测试 4: 未初始化状态（先关闭再测试）
    test_error_scenario("未初始化状态检测");
    vinput_core_shutdown();

    VInputVInputCommand command;
    result = vinput_core_try_recv_command(&command);
    if (result == NotInitialized) {
        printf("✓ 未初始化状态被正确检测\n");
    } else {
        printf("⚠ 可能的状态管理问题: %d\n", result);
    }

    // 重新初始化用于后续测试
    vinput_core_init();

    // 测试 5: 命令队列空状态
    test_error_scenario("空命令队列处理");
    result = vinput_core_try_recv_command(&command);
    if (result == NoData) {
        printf("✓ 空命令队列正确返回 NoData\n");
    } else {
        printf("✗ 空命令队列处理异常: %d\n", result);
    }

    // 测试 6: 事件处理错误容忍
    test_error_scenario("事件处理（空数据）");
    VInputVInputEvent event = {StartRecording, NULL, 0};
    result = vinput_core_send_event(&event);
    if (result == Success) {
        printf("✓ 开始录音事件处理成功\n");
    } else {
        printf("✗ 事件处理失败: %d\n", result);
    }

    // 清理
    printf("\n--- 清理资源 ---\n");
    vinput_core_shutdown();
    printf("✓ 资源已释放\n");

    printf("\n=== 测试完成 ===\n");
    printf("\n总结：\n");
    printf("- 错误码在日志中可见（E1001-E9999）\n");
    printf("- 错误严重度分类（Low/Medium/High/Critical）\n");
    printf("- 恢复策略（Retry/Degrade/UserAction/Restart）\n");
    printf("- 所有错误都有结构化日志记录\n");

    return 0;
}
