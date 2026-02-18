# 体积优化总结

## 问题分析

原始体积：
- vinput-settings: 18 MB
- libvinput_core.so: 4.4 MB

## 优化措施

### 1. 移除无用依赖
- GUI 程序不需要依赖 `vinput-core`
- 移除 `vinput-gui/Cargo.toml` 中的 `vinput-core` 依赖

### 2. 添加 Release 优化配置
在 `Cargo.toml` 中添加：
```toml
[profile.release]
opt-level = "z"        # 优化体积（而不是速度）
lto = true             # 链接时优化
codegen-units = 1      # 单个代码生成单元（更好的优化）
strip = true           # 自动 strip 调试符号
panic = "abort"        # panic 时直接 abort（减小体积）
```

## 优化结果

优化后体积：
- vinput-settings: **6.7 MB** ⬇️ 减少 11.3 MB (62%)
- libvinput_core.so: **2.6 MB** ⬇️ 减少 1.8 MB (41%)

## 包大小更新

修订后的 deb 包大小：
- 核心库: 2.6 MB (原 4.4 MB)
- Fcitx5 插件: 78 KB
- 设置程序: 6.7 MB (原 18 MB)
- AI 模型: 227 MB
- Sherpa-ONNX 库: ~50 MB
- ONNX Runtime 库: ~100 MB

**新总计**: 约 **387 MB** (压缩后约 240-280 MB)

相比原计划减少了约 13 MB。

## 说明

egui 本身就比较大（包含字体、渲染引擎等），6.7 MB 对于一个完整的 GUI 应用来说是合理的。

进一步优化可能需要：
- 使用更轻量的 GUI 框架（如 GTK）
- 但这会增加开发复杂度和依赖
- 当前方案已经足够优化
