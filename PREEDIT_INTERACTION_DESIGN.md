# Fcitx5 Preedit 交互设计方案

## 当前方案分析

### 提议的交互流程
1. 录音 → 识别 → 显示灰色 preedit
2. 应用标点、ITN 处理
3. 一次性 CommitString 替换 preedit
4. 正式上屏

## 优缺点分析

### ✅ 优点
1. **符合传统输入法习惯** - 用户熟悉 preedit 预览机制
2. **可预览最终结果** - 上屏前看到完整处理后的文字
3. **有"反悔"机会** - preedit 阶段可以取消（ESC 键）
4. **视觉反馈清晰** - 灰色预览 → 黑色上屏，状态明确
5. **减少误操作** - 用户可以在上屏前检查结果

### ❌ 缺点
1. **延迟感明显** - 需要等待标点、ITN 处理（~100-200ms）
2. **流式识别体验差** - preedit 频繁更新会闪烁
3. **实现复杂度高** - 需要管理 preedit 状态、更新、清除
4. **撤销逻辑复杂** - 需要区分 preedit 和已上屏文字
5. **打断用户节奏** - 用户需要等待确认才能继续

## 改进方案

### 方案 A：双模式支持（推荐 ⭐）

提供两种模式，用户可在 GUI 设置中选择：

#### 1. 即时模式（默认，当前实现）
```
录音 → 识别 → 标点/ITN → 直接 CommitString
```

**特点：**
- ✅ 零延迟，体验流畅
- ✅ 适合快速输入、聊天、笔记
- ✅ 实现简单，已完成
- ❌ 无法预览最终结果

**适用场景：**
- 日常聊天
- 快速记录
- 熟练用户

#### 2. 预览模式（新增）
```
录音 → 识别 → Preedit 预览 → 标点/ITN → 自动/手动确认 → CommitString
```

**特点：**
- ✅ 可以预览和确认
- ✅ 适合重要文档、正式场合
- ✅ 减少错误上屏
- ❌ 有延迟感（可接受）

**适用场景：**
- 正式文档
- 重要邮件
- 需要高准确度的场合

### 方案 B：智能 Preedit 策略

根据不同阶段采用不同的显示策略：

#### 阶段 1：录音中
```
显示：🎤 录音中...（InputPanel AuxUp）
Preedit：空
```

#### 阶段 2：识别中（流式结果）
```
选项 2.1：不显示流式结果（推荐）
- 避免闪烁
- 用户专注于说话

选项 2.2：显示流式结果
- Preedit：实时更新识别文字（灰色）
- 问题：可能闪烁，分散注意力
```

#### 阶段 3：处理中
```
显示：🔵 识别中...（InputPanel AuxUp）
Preedit：原始识别结果（灰色）
```

#### 阶段 4：完成
```
Preedit：最终结果（标点 + ITN）（灰色）
延迟 300-500ms 后自动 CommitString
或：等待用户按回车确认
```

### 方案 C：渐进式上屏（创新 ⭐⭐）

结合即时模式和预览模式的优点：

```
录音 → 识别 → 立即 CommitString（原始结果）
                ↓
         后台处理标点/ITN
                ↓
         如果有变化：
           - 删除原始文字（Backspace）
           - CommitString（处理后结果）
```

**特点：**
- ✅ 零延迟感（立即上屏）
- ✅ 自动优化（后台处理）
- ✅ 用户无感知（如果没变化）
- ❌ 可能有"闪烁"（文字被替换）

**优化：**
- 只在变化较大时才替换（如添加了标点、转换了数字）
- 小变化（如空格调整）不替换

## 推荐实现方案

### 最佳方案：方案 A（双模式）+ 方案 C（可选）

#### 配置选项（GUI 基本设置）

```toml
[display]
mode = "instant"  # instant | preview | progressive

[display.preview]
auto_commit_delay = 500  # ms，自动上屏延迟
show_streaming = false   # 是否显示流式识别结果
confirm_key = "Return"   # 手动确认键（空表示自动）

[display.progressive]
min_change_threshold = 2  # 最小变化字符数才替换
```

#### 实现优先级

**Phase 1：保持当前即时模式**（已完成 ✅）
- 零延迟直接上屏
- 用户体验流畅

**Phase 2：添加预览模式**（可选）
- 实现 Preedit 显示
- 实现自动/手动确认
- 添加 GUI 配置选项

**Phase 3：添加渐进式模式**（实验性）
- 实现智能替换逻辑
- 测试用户体验

## 具体实现建议

### 预览模式实现

#### 1. Preedit 显示

```cpp
void VInputEngine::showPreedit(const std::string& text) {
    auto* ic = instance_->mostRecentInputContext();
    if (!ic) return;

    // 设置 preedit 文字
    Text preedit(text);
    preedit.setCursor(text.length());  // 光标在末尾

    // 设置为灰色
    preedit.append(text, TextFormatFlag::Underline);

    ic->inputPanel().setClientPreedit(preedit);
    ic->updatePreedit();
}
```

#### 2. 流程控制

```cpp
void VInputEngine::processRecognitionResult(const std::string& raw_text) {
    // 1. 显示原始结果到 preedit
    showPreedit(raw_text);

    // 2. 后台处理标点和 ITN
    std::string processed_text = applyPunctuationAndITN(raw_text);

    // 3. 更新 preedit 显示处理后的结果
    showPreedit(processed_text);

    // 4. 延迟后自动上屏（或等待用户确认）
    if (auto_commit_enabled) {
        scheduleAutoCommit(processed_text, 500);  // 500ms 后上屏
    } else {
        // 等待用户按回车
        waiting_for_confirm = true;
        pending_text = processed_text;
    }
}

void VInputEngine::commitPreedit() {
    auto* ic = instance_->mostRecentInputContext();
    if (!ic) return;

    // 清除 preedit
    ic->inputPanel().reset();
    ic->updatePreedit();

    // 上屏
    ic->commitString(pending_text);

    pending_text.clear();
    waiting_for_confirm = false;
}
```

#### 3. 按键处理

```cpp
void VInputEngine::keyEvent(KeyEvent& keyEvent) {
    // ... 现有代码 ...

    // 处理确认键
    if (waiting_for_confirm) {
        if (keyEvent.key().check(FcitxKey_Return)) {
            commitPreedit();
            keyEvent.filterAndAccept();
            return;
        }

        if (keyEvent.key().check(FcitxKey_Escape)) {
            cancelPreedit();
            keyEvent.filterAndAccept();
            return;
        }
    }
}
```

### 渐进式模式实现

```cpp
void VInputEngine::progressiveCommit(const std::string& raw_text) {
    // 1. 立即上屏原始结果
    auto* ic = instance_->mostRecentInputContext();
    ic->commitString(raw_text);

    // 2. 后台处理
    std::string processed_text = applyPunctuationAndITN(raw_text);

    // 3. 计算差异
    int changes = calculateDifference(raw_text, processed_text);

    // 4. 如果变化足够大，替换
    if (changes >= min_change_threshold) {
        // 删除原始文字
        for (size_t i = 0; i < raw_text.length(); ++i) {
            ic->forwardKey(Key(FcitxKey_BackSpace));
        }

        // 上屏处理后的文字
        ic->commitString(processed_text);
    }
}
```

## 用户体验对比

| 特性 | 即时模式 | 预览模式 | 渐进式模式 |
|------|---------|---------|-----------|
| 延迟感 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 可预览 | ❌ | ✅ | ⚠️ 部分 |
| 可确认 | ❌ | ✅ | ❌ |
| 流畅度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| 准确感知 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| 实现复杂度 | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

## 最终建议

### 短期（当前版本）
**保持即时模式**（已实现 ✅）
- 体验最流畅
- 实现最简单
- 适合大多数用户

### 中期（下一版本）
**添加预览模式作为可选项**
- 在 GUI 基本设置中添加"显示模式"选项
- 实现 Preedit 显示和自动确认
- 默认仍使用即时模式

### 长期（实验性）
**探索渐进式模式**
- 作为实验性功能
- 收集用户反馈
- 根据反馈决定是否正式采用

## 配置界面设计

在 GUI 基本设置页面添加：

```
显示模式：
  ○ 即时模式（推荐）- 识别结果直接上屏，零延迟
  ○ 预览模式 - 先显示预览，确认后上屏

预览模式设置：
  自动确认延迟：[500] ms
  □ 显示流式识别结果
  □ 需要手动按回车确认
```

---

**总结：当前的即时模式体验已经很好，建议保持。预览模式可以作为可选功能添加，满足不同用户需求。**
