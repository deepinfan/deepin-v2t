# Claude Code 增强配置 (CCG Enhanced)
## 一、核心原则
### 1.1 调研优先（强制）
修改代码前必须：
1. **检索相关代码** - 使用 `mcp__ace-tool__search_context` 或 Grep/Glob
2. **识别复用机会** - 查找已有相似功能，优先复用而非重写
3. **追踪调用链** - 使用 Grep 分析影响范围
### 1.2 修改前三问
1. 这是真问题还是臆想？（拒绝过度设计）
2. 有现成代码可复用吗？（优先复用）
3. 会破坏什么调用关系？（保护依赖链）
### 1.3 红线原则
- 禁止 copy-paste 重复代码
- 禁止破坏现有功能
- 禁止对错误方案妥协
- 禁止盲目执行不加思考
- 禁止基于假设回答（必须检索验证）
- 关键路径必须有错误处理
### 1.4 知识获取（强制）
遇到不熟悉的知识，必须联网搜索，严禁猜测：
- 通用搜索：`WebSearch` / `mcp__exa__web_search_exa`
- 库文档：`mcp___upstash_context7-mcp__resolve-library-id` → `query-docs`
- 开源项目：`mcp__mcp-deepwiki__deepwiki_fetch`

---

## 四、任务分级

| 级别 | 判断标准 | 处理方式 |
|------|----------|----------|
| 简单 | 单文件、明确需求、少于 20 行 | 直接执行 |
| 中等 | 2-5 个文件、需要调研 | 简要说明方案 → 执行 |
| 复杂 | 架构变更、多模块、不确定性高 | 完整规划流程 |

### 4.1 复杂任务流程

1. **RESEARCH** - 调研代码，不提建议
2. **PLAN** - 列出方案，等待用户确认
3. **EXECUTE** - 严格按计划执行
4. **REVIEW** - 完成后自检
触发：用户说"进入X模式"或任务符合复杂标准时自动启用

### 4.2 复杂问题深度思考

触发场景：多步骤推理、架构设计、疑难调试、方案对比
强制工具：`mcp__sequential-thinking__sequentialthinking`

---

## 五、工具速查

| 场景 | 推荐工具 |
|------|----------|
| 代码语义检索 | `mcp__ace-tool__search_context` |
| 精确字符串/正则 | `Grep` |
| 文件名匹配 | `Glob` |
| 代码库探索 | `Task` + `subagent_type=Explore` |
| 技术方案规划 | `EnterPlanMode` 或 `Task` + `subagent_type=Plan` |
| 库官方文档 | `mcp___upstash_context7-mcp__query-docs` |
| 开源项目文档 | `mcp__mcp-deepwiki__deepwiki_fetch` |
| 联网搜索 | `WebSearch` / `mcp__exa__web_search_exa` |
| 深度推理 | `mcp__sequential-thinking__sequentialthinking` |
| PDF 读取 | `mcp__pdf-reader__read_pdf` |
| 跨会话记忆 | `mcp__server-memory__*`（仅用户要求时） |
| 快捷操作 | Skill（`/commit`、`/debug`、`/review` 等） |

**选择原则**：语义理解用 `ace-tool`，精确匹配用 `Grep`

---

## 六、Git 规范

- 不主动提交/push，除非用户明确要求
- Commit 格式：`<type>(<scope>): <description>`
- 不添加 Claude 署名标记
- 提交前：`git diff` 确认改动范围
- 禁止 `--force` 推送到 main/master

---

## 七、安全检查
- 禁止硬编码密钥/密码/token
- 不提交 .env / credentials 等敏感文件
- 用户输入在系统边界必须验证

---

## 八、代码风格
- **KISS** - 能简单就不复杂
- **DRY** - 零容忍重复，必须复用
- **保护调用链** - 修改函数签名时同步更新所有调用点
完成后清理：临时文件、废弃代码、未使用导入、调试日志
---

## 九、交互规范

### 何时询问用户
- 存在多个合理方案时
- 需求不明确或有歧义时
- 改动范围超出预期时
- 发现潜在风险时

### 何时直接执行
- 需求明确且方案唯一
- 小范围修改（少于 20 行）
- 用户已确认过类似操作

### 敢于说不
发现问题直接指出，不妥协于错误方案

### 无需确认
- 如需执行sudo，采用SUDO_ASKPASS=/usr/libexec/gcr-ssh-askpass sudo 一次性密码弹窗方式
- file,cat,grep,sed,tail,tee等常用shell命令读取项目目录和之外的目录或文件，无需确认
- 允许所有在项目目录之内对目录和文件的读取、编辑和创建文件操作，无需确认
- 允许所有访问外包网站获取信息的操作

### 文档保存
所有在过程中生成的项目文档，均在项目目录的docs子目录中创建和保存

---

## 十、环境特定（Windows / PowerShell）
- 不支持 `&&`，使用 `;` 分隔命令
- 中文路径用引号包裹

---
## 输出设置
- 中文响应
- 禁用表情符号
- 禁止截断输出