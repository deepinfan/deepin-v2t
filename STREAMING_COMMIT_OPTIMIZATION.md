# V-Input 流式上屏优化方案

## 当前问题分析

### 现状
```
用户说话 → 等待停顿 → 端点检测 → 标点/ITN → 一次性上屏
         ↑_____________等待时间_____________↑
```

**问题：**
- ❌ 必须等到整句话说完才上屏
- ❌ 没有即时反馈，用户不知道识别了什么
- ❌ 延迟感明显（~1-2秒）
- ❌ 不符合"流式识别"的预期

## 优化方案

### 方案 1：真正的流式上屏（推荐 ⭐⭐⭐⭐⭐）

#### 核心思路
**边识别边上屏，标点和 ITN 实时应用**

#### 实现流程

```
时间轴：
0ms:   用户开始说话
100ms: 识别到 "今天" → 立即上屏 "今天"
300ms: 识别到 "天气" → 立即上屏 "天气"
800ms: 检测到停顿 → 上屏 "，"（逗号）
1000ms: 识别到 "很好" → 立即上屏 "很好"
1500ms: 检测到句子结束 → 上屏 "。"（句号）
```

#### 技术实现

##### 1. 使用 Preedit 显示流式结果

```cpp
void VInputEngine::onPartialResult(const std::string& partial_text) {
    auto* ic = instance_->mostRecentInputContext();
    if (!ic) return;

    // 计算新增的文字
    std::string new_text = getNewText(partial_text, last_committed_text);

    if (!new_text.empty()) {
        // 应用 ITN（仅数字转换，快速）
        std::string itn_text = applyQuickITN(new_text);

        // 立即上屏新增部分
        ic->commitString(itn_text);

        last_committed_text += itn_text;
    }

    // 更新 preedit 显示未确定的部分
    std::string uncommitted = getUncommittedText(partial_text);
    if (!uncommitted.empty()) {
        Text preedit(uncommitted);
        ic->inputPanel().setClientPreedit(preedit);
        ic->updatePreedit();
    }
}

void VInputEngine::onFinalResult(const std::string& final_text) {
    auto* ic = instance_->mostRecentInputContext();
    if (!ic) return;

    // 清除 preedit
    ic->inputPanel().reset();
    ic->updatePreedit();

    // 计算剩余未上屏的文字
    std::string remaining = getRemainingText(final_text, last_committed_text);

    if (!remaining.empty()) {
        // 应用完整的标点和 ITN
        std::string processed = applyFullProcessing(remaining);
        ic->commitString(processed);
    }

    // 重置状态
    last_committed_text.clear();
}
```

##### 2. 分级 ITN 处理

**快速 ITN（流式阶段）：**
- 仅转换数字（"一千二百" → "1200"）
- 延迟 <10ms
- 用于流式上屏

**完整 ITN（最终阶段）：**
- 转换货币、日期、百分比
- 应用标点规则
- 延迟 ~100ms
- 用于最终确认

```rust
// 在 ITNEngine 中添加
impl ITNEngine {
    /// 快速 ITN（仅数字）
    pub fn process_quick(&self, text: &str) -> ITNResult {
        // 只转换数字，跳过其他规则
        // 速度优先
    }

    /// 完整 ITN（所有规则）
    pub fn process_full(&self, text: &str) -> ITNResult {
        // 应用所有规则
        // 准确度优先
    }
}
```

##### 3. 智能分词上屏

```rust
pub struct StreamingCommitter {
    /// 已上屏的文字
    committed_text: String,
    /// 待确认的文字
    pending_text: String,
    /// 上屏阈值（token 数量）
    commit_threshold: usize,
}

impl StreamingCommitter {
    /// 处理流式结果
    pub fn process_partial(&mut self, partial_text: &str) -> Vec<String> {
        let mut to_commit = Vec::new();

        // 分词
        let tokens = tokenize(partial_text);

        // 如果有足够的稳定 token，上屏
        if tokens.len() >= self.commit_threshold {
            let stable_tokens = &tokens[..tokens.len() - 1];  // 保留最后一个
            let stable_text = stable_tokens.join("");

            if stable_text != self.committed_text {
                let new_text = &stable_text[self.committed_text.len()..];
                to_commit.push(new_text.to_string());
                self.committed_text = stable_text;
            }
        }

        to_commit
    }
}
```

### 方案 2：Preedit 流式预览 + 延迟上屏

#### 核心思路
**使用 Preedit 显示流式结果，稳定后逐步上屏**

#### 实现流程

```
时间轴：
0ms:   用户开始说话
100ms: Preedit 显示 "今天"（灰色）
300ms: Preedit 更新为 "今天天气"（灰色）
500ms: "今天" 稳定 → CommitString "今天"，Preedit 显示 "天气"
800ms: 检测到停顿 → CommitString "天气，"，Preedit 清空
1000ms: Preedit 显示 "很好"（灰色）
1500ms: 句子结束 → CommitString "很好。"
```

**特点：**
- ✅ 有即时视觉反馈（Preedit）
- ✅ 稳定的词逐步上屏
- ✅ 减少延迟感
- ⚠️ Preedit 会更新（但可接受）

### 方案 3：混合模式（最佳 ⭐⭐⭐⭐⭐）

#### 核心思路
**结合流式上屏和 Preedit 预览的优点**

#### 实现策略

```
阶段 1：流式识别（边说边显示）
- Preedit 实时显示识别结果（灰色）
- 每识别到 2-3 个稳定的词，立即 CommitString
- 保留最后 1-2 个不稳定的词在 Preedit

阶段 2：停顿检测
- 检测到停顿 → 添加逗号
- CommitString 剩余文字 + 逗号
- 清空 Preedit

阶段 3：句子结束
- 应用完整的标点和 ITN
- CommitString 最终结果
```

#### 详细实现

```cpp
class VInputEngine {
private:
    std::string committed_text_;      // 已上屏的文字
    std::string preedit_text_;        // Preedit 中的文字
    int stable_token_count_;          // 稳定的 token 数量

    void onStreamingResult(const std::string& partial_text) {
        auto* ic = instance_->mostRecentInputContext();
        if (!ic) return;

        // 1. 分词
        auto tokens = tokenize(partial_text);

        // 2. 判断哪些 token 是稳定的
        int stable_count = tokens.size() - 2;  // 保留最后 2 个 token
        if (stable_count > stable_token_count_) {
            // 有新的稳定 token，上屏
            std::string stable_text = joinTokens(tokens, 0, stable_count);
            std::string new_text = stable_text.substr(committed_text_.length());

            if (!new_text.empty()) {
                // 快速 ITN（仅数字）
                std::string itn_text = applyQuickITN(new_text);

                // 立即上屏
                ic->commitString(itn_text);

                committed_text_ += itn_text;
                stable_token_count_ = stable_count;
            }
        }

        // 3. 更新 Preedit 显示不稳定的部分
        if (stable_count < tokens.size()) {
            std::string unstable_text = joinTokens(tokens, stable_count, tokens.size());
            updatePreedit(ic, unstable_text);
        }
    }

    void onPauseDetected() {
        auto* ic = instance_->mostRecentInputContext();
        if (!ic) return;

        // 停顿 → 添加逗号
        if (!preedit_text_.empty()) {
            ic->commitString(preedit_text_ + "，");
            clearPreedit(ic);
        } else {
            ic->commitString("，");
        }

        committed_text_.clear();
        stable_token_count_ = 0;
    }

    void onFinalResult(const std::string& final_text) {
        auto* ic = instance_->mostRecentInputContext();
        if (!ic) return;

        // 清除 Preedit
        clearPreedit(ic);

        // 计算剩余文字
        std::string remaining = final_text.substr(committed_text_.length());

        if (!remaining.empty()) {
            // 应用完整的标点和 ITN
            std::string processed = applyFullProcessing(remaining);

            // 添加句号
            if (!processed.empty() && !isPunctuation(processed.back())) {
                processed += "。";
            }

            ic->commitString(processed);
        }

        // 重置状态
        committed_text_.clear();
        preedit_text_.clear();
        stable_token_count_ = 0;
    }
};
```

### 方案 4：渐进式上屏（激进 ⭐⭐⭐⭐）

#### 核心思路
**完全抛弃 Preedit，所有文字立即上屏，错误时回退修正**

```
时间轴：
0ms:   用户开始说话
100ms: 识别到 "今" → 立即上屏 "今"
200ms: 识别到 "天" → 立即上屏 "天"
300ms: 识别到 "天气" → 发现 "今天" 更合理
       → Backspace 删除 "今天"
       → 上屏 "今天天气"
800ms: 检测到停顿 → 上屏 "，"
```

**特点：**
- ✅ 极致流畅，零延迟
- ✅ 类似打字的即时感
- ❌ 可能有"闪烁"（文字被修正）
- ❌ 实现非常复杂

## 推荐方案对比

| 方案 | 流畅度 | 延迟感 | 实现难度 | 推荐度 |
|------|--------|--------|----------|--------|
| 当前方案 | ⭐⭐ | ⭐⭐ | ⭐ | ⭐⭐ |
| 方案 1：真正流式 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| 方案 2：Preedit 流式 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 方案 3：混合模式 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 方案 4：渐进式 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐⭐ | ⭐⭐⭐ |

## 最佳实践建议

### 推荐：方案 3（混合模式）

**为什么？**
1. **即时反馈** - Preedit 实时显示，用户知道识别了什么
2. **渐进上屏** - 稳定的词立即上屏，减少延迟感
3. **准确度高** - 不稳定的词保留在 Preedit，避免错误上屏
4. **符合直觉** - 类似拼音输入法的体验

### 实现步骤

#### Step 1: 修改 Rust Core 返回流式结果

```rust
// 在 StreamingPipeline 中添加
pub struct StreamingResult {
    pub partial_text: String,        // 完整的部分结果
    pub stable_text: String,         // 稳定的部分（可以上屏）
    pub unstable_text: String,       // 不稳定的部分（保留在 preedit）
    pub is_final: bool,              // 是否最终结果
    pub should_add_comma: bool,      // 是否应该添加逗号
}

impl StreamingPipeline {
    pub fn get_streaming_result(&self) -> StreamingResult {
        let partial = self.get_partial_result();
        let tokens = tokenize(&partial);

        // 保留最后 2 个 token 在 preedit
        let stable_count = tokens.len().saturating_sub(2);

        let stable_text = tokens[..stable_count].join("");
        let unstable_text = tokens[stable_count..].join("");

        StreamingResult {
            partial_text: partial,
            stable_text: applyQuickITN(stable_text),  // 快速 ITN
            unstable_text,
            is_final: self.pipeline_state == PipelineState::Completed,
            should_add_comma: self.detect_pause(),
        }
    }
}
```

#### Step 2: 修改 FFI 接口

```rust
// 添加新的命令类型
pub enum VInputCommandType {
    CommitText = 1,
    UpdatePreedit = 2,      // 新增：更新 preedit
    ClearPreedit = 3,       // 新增：清除 preedit
    CommitAndClear = 4,     // 新增：上屏并清除 preedit
    // ...
}

// 添加流式结果回调
pub type VInputStreamingCallback = extern "C" fn(*const VInputStreamingResult);

#[repr(C)]
pub struct VInputStreamingResult {
    pub stable_text: *mut c_char,      // 可以上屏的文字
    pub unstable_text: *mut c_char,    // 保留在 preedit 的文字
    pub should_add_comma: bool,
    pub is_final: bool,
}
```

#### Step 3: 修改 C++ 插件

```cpp
void VInputEngine::onStreamingCallback(const VInputStreamingResult* result) {
    auto* ic = instance_->mostRecentInputContext();
    if (!ic) return;

    std::string stable(result->stable_text);
    std::string unstable(result->unstable_text);

    // 1. 上屏稳定的文字
    if (!stable.empty()) {
        // 计算新增部分
        std::string new_text = stable.substr(last_committed_.length());
        if (!new_text.empty()) {
            ic->commitString(new_text);
            last_committed_ = stable;
        }
    }

    // 2. 更新 Preedit 显示不稳定的文字
    if (!unstable.empty()) {
        Text preedit(unstable);
        preedit.setCursor(unstable.length());
        ic->inputPanel().setClientPreedit(preedit);
        ic->updatePreedit();
    } else {
        // 清空 Preedit
        ic->inputPanel().reset();
        ic->updatePreedit();
    }

    // 3. 如果检测到停顿，添加逗号
    if (result->should_add_comma) {
        ic->commitString("，");
    }

    // 4. 如果是最终结果，应用完整处理
    if (result->is_final) {
        // 清空 Preedit
        ic->inputPanel().reset();
        ic->updatePreedit();

        // 上屏剩余文字 + 句号
        if (!unstable.empty()) {
            std::string final_text = applyFullProcessing(unstable);
            ic->commitString(final_text + "。");
        }

        // 重置状态
        last_committed_.clear();
    }
}
```

## 用户体验对比

### 当前方案
```
用户：今天天气很好
体验：[说话中...] → [等待 1-2秒] → "今天天气很好。"
感受：❌ 延迟明显，不知道识别了什么
```

### 优化后（方案 3）
```
用户：今天天气很好
体验：
  100ms: "今天" 上屏
  300ms: "天气" 上屏
  800ms: "，" 上屏（检测到停顿）
  1000ms: "很好" 在 Preedit 显示（灰色）
  1500ms: "很好。" 上屏，Preedit 清空

感受：✅ 流畅，有即时反馈，类似打字
```

## 配置选项设计

在 GUI 基本设置中添加：

```
流式上屏设置：
  ● 混合模式（推荐）- 稳定的词立即上屏，不稳定的词显示在 Preedit
  ○ 完全流式 - 所有识别结果立即上屏（可能有修正）
  ○ 延迟上屏 - 等待整句完成后一次性上屏（当前模式）

混合模式参数：
  稳定阈值：[2] 个 token（保留最后 N 个 token 在 Preedit）
  快速 ITN：[✓] 启用（流式阶段仅转换数字）
```

## 实现优先级

### Phase 1：添加流式结果支持（高优先级）
1. 修改 `StreamingPipeline` 返回流式结果
2. 添加 FFI 流式回调接口
3. 修改 C++ 插件支持 Preedit 更新

### Phase 2：实现混合模式（推荐）
1. 实现稳定性判断逻辑
2. 实现渐进上屏
3. 实现 Preedit 管理

### Phase 3：添加配置选项
1. 在 GUI 中添加流式上屏设置
2. 支持多种模式切换
3. 可调节参数

## 性能考虑

### 快速 ITN 优化

```rust
impl ITNEngine {
    /// 快速 ITN（仅数字，<10ms）
    pub fn process_quick(&self, text: &str) -> String {
        // 只转换中文数字，跳过其他规则
        let mut result = text.to_string();

        // 仅应用数字转换
        for block in tokenize(text) {
            if block.is_chinese_number() {
                let converted = ChineseNumberConverter::convert(&block.content);
                result = result.replace(&block.content, &converted);
            }
        }

        result
    }

    /// 完整 ITN（所有规则，~100ms）
    pub fn process_full(&self, text: &str) -> ITNResult {
        // 当前的完整实现
        self.process(text)
    }
}
```

### Preedit 更新频率控制

```cpp
// 限制 Preedit 更新频率，避免闪烁
class PreeditThrottler {
    std::chrono::steady_clock::time_point last_update_;
    const int min_interval_ms_ = 50;  // 最小更新间隔 50ms

public:
    bool shouldUpdate() {
        auto now = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
            now - last_update_).count();

        if (elapsed >= min_interval_ms_) {
            last_update_ = now;
            return true;
        }
        return false;
    }
};
```

## 总结

### 当前问题
❌ 必须等到整句话说完才上屏，延迟感明显

### 推荐解决方案
✅ **方案 3：混合模式**
- Preedit 实时显示识别结果
- 稳定的词立即上屏
- 不稳定的词保留在 Preedit
- 停顿时添加逗号
- 句子结束时应用完整处理

### 实现建议
1. **短期**：添加流式结果支持和 Preedit 显示
2. **中期**：实现混合模式
3. **长期**：提供多种模式供用户选择

---

**需要我帮您实现混合模式吗？这将显著提升用户体验！**
