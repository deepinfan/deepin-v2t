# 混合模式流式上屏实现

## 实现概述

实现了混合模式的流式上屏功能，结合了即时上屏和 Preedit 预览的优点。

## 核心原理

### 稳定性判断

保留最后 N 个字符在 Preedit（不稳定），其余部分立即上屏（稳定）。

```rust
// vinput-core/src/streaming/pipeline.rs
fn split_stable_unstable(&self, text: &str) -> (String, String) {
    const KEEP_LAST_CHARS: usize = 2; // 保留最后2个字符在 Preedit

    let chars: Vec<char> = text.chars().collect();

    if chars.len() <= KEEP_LAST_CHARS {
        // 全部不稳定
        return (String::new(), text.to_string());
    }

    let stable_count = chars.len() - KEEP_LAST_CHARS;
    let stable: String = chars[..stable_count].iter().collect();
    let unstable: String = chars[stable_count..].iter().collect();

    (stable, unstable)
}
```

### 流式识别结果

```rust
pub struct StreamingResult {
    pub partial_result: String,      // 完整的部分结果
    pub stable_text: String,          // 稳定的部分（可以上屏）
    pub unstable_text: String,        // 不稳定的部分（保留在 Preedit）
    pub should_add_comma: bool,       // 是否应该添加逗号
    pub is_final: bool,               // 是否最终结果
    // ...
}
```

## 实现流程

### 阶段 1：流式识别（边说边显示）

```
用户说话 → ASR 识别 → 分离稳定/不稳定文本

100ms: "今" → stable: "", unstable: "今"
       → Preedit 显示 "今"

200ms: "今天" → stable: "", unstable: "今天"
       → Preedit 更新为 "今天"

300ms: "今天天" → stable: "今", unstable: "天天"
       → 上屏 "今"
       → Preedit 显示 "天天"

400ms: "今天天气" → stable: "今天", unstable: "天气"
       → 上屏 "天"
       → Preedit 显示 "天气"
```

### 阶段 2：句子结束

```
检测到端点 → 清除 Preedit → 应用标点和 ITN → 上屏剩余文本

1. 清除 Preedit
2. 获取最终结果（含标点）："今天天气很好。"
3. 应用 ITN 处理
4. 计算剩余文本：final_result - last_committed_stable
5. 上屏剩余文本："天气很好。"
6. 重置状态，准备下一句
```

## 代码修改

### 1. Rust Core (vinput-core)

#### streaming/pipeline.rs
- 添加 `stable_text` 和 `unstable_text` 字段到 `StreamingResult`
- 实现 `split_stable_unstable()` 方法

#### ffi/types.rs
- 添加 `UpdatePreedit` 和 `ClearPreedit` 命令类型
- 添加 `update_preedit()` 和 `clear_preedit()` 辅助方法

#### ffi/exports.rs
- 修改音频处理循环，实现混合模式逻辑：
  - 识别过程中：上屏稳定文本，更新 Preedit 显示不稳定文本
  - 句子结束时：清除 Preedit，上屏剩余文本

### 2. C++ Plugin (fcitx5-vinput)

#### vinput_engine.cpp
- 添加 `UpdatePreedit` 命令处理：
  ```cpp
  case VInputVInputCommandType::UpdatePreedit:
      Text preedit(text);
      preedit.setCursor(text.length());
      auto& inputPanel = ic->inputPanel();
      inputPanel.setClientPreedit(preedit);
      ic->updatePreedit();
      break;
  ```

- 添加 `ClearPreedit` 命令处理：
  ```cpp
  case VInputVInputCommandType::ClearPreedit:
      auto& inputPanel = ic->inputPanel();
      inputPanel.reset();
      ic->updatePreedit();
      break;
  ```

## 用户体验

### 当前模式（优化前）
```
用户：今天天气很好
体验：[说话中...] → [等待 1-2秒] → "今天天气很好。"
感受：❌ 延迟明显，不知道识别了什么
```

### 混合模式（优化后）
```
用户：今天天气很好
体验：
  100ms: Preedit 显示 "今" (灰色)
  200ms: Preedit 显示 "今天" (灰色)
  300ms: 上屏 "今" (黑色)，Preedit 显示 "天天" (灰色)
  400ms: 上屏 "天" (黑色)，Preedit 显示 "天气" (灰色)
  600ms: 上屏 "天气" (黑色)，Preedit 显示 "很好" (灰色)
  1500ms: 清除 Preedit，上屏 "很好。" (黑色)

感受：✅ 流畅，有即时反馈，类似拼音输入法
```

## 配置参数

```rust
// 可调节参数（未来可在 GUI 中配置）
const KEEP_LAST_CHARS: usize = 2;  // 保留最后 N 个字符在 Preedit
```

## 优点

1. ✅ **即时反馈** - Preedit 实时显示识别结果
2. ✅ **渐进上屏** - 稳定的文字立即上屏，减少延迟感
3. ✅ **准确度高** - 不稳定的文字保留在 Preedit，避免错误上屏
4. ✅ **符合直觉** - 类似拼音输入法的体验
5. ✅ **标点和 ITN 准确** - 在最终阶段应用，不影响已上屏文字

## 局限性

1. ⚠️ **简单分词** - 当前按字符分割，可能在词中间切分
2. ⚠️ **固定阈值** - KEEP_LAST_CHARS 是固定值，未来可改为可配置
3. ⚠️ **ITN 限制** - 如果 ITN 改变了已上屏的文字，无法回退修改

## 未来改进

1. **智能分词** - 使用词典或 NLP 模型进行更准确的分词
2. **动态阈值** - 根据识别置信度动态调整稳定性判断
3. **停顿检测** - 实现 `should_add_comma` 逻辑，在停顿时添加逗号
4. **配置选项** - 在 GUI 中添加混合模式参数配置

## 测试方法

1. 编译并安装：
   ```bash
   cd vinput-core && cargo build --release
   cd ../fcitx5-vinput && mkdir -p build && cd build
   cmake .. && make
   sudo make install
   ```

2. 重启 Fcitx5：
   ```bash
   fcitx5 -r
   ```

3. 测试流式上屏：
   - 按空格键开始录音
   - 说话："今天天气很好"
   - 观察 Preedit 和上屏效果
   - 再次按空格键停止录音

## 日志输出

```
📝 上屏稳定文本: [今]
📝 上屏稳定文本: [天]
📝 上屏稳定文本: [天气]
🔔 检测到句子结束，处理最终结果
🎤 识别结果（含智能标点）: [今天天气很好。]
📝 开始 ITN 处理...
📋 ITN: 无需变更（输入已是规范格式）
✅ 最终结果: [今天天气很好。]
📝 上屏剩余文本: [很好。]
✨ 混合模式上屏完成
```

---

**实现时间**: 2026-02-16
**状态**: ✅ 已实现并编译通过
