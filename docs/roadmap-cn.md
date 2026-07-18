# AuroraPulse 执行计划表

## 1. 当前共识

基于 Phase 0-3 的实现与最新产品判断，AuroraPulse 当前已经定下来的方向是：

> 把自己告诉 Aurora 一次，所有你授权的 AI 都能认识你。

- 项目身份是面向 AI Agent 的 `Local Identity and Memory Layer`，不是另一个通用聊天 Agent
- Aurora 是本地个人事实、当前关注、偏好、记忆和披露策略的所有者与唯一事实源
- Codex、GPT、Claude、Gemini 或其他 Agent 都只是经过授权的上下文消费者
- MCP 是第一条跨 Agent 产品接口，Aurora Core 仍然负责权限、来源、过滤和审计
- V1 默认只读、最小必要披露，不允许 Agent 获取与当前任务无关的完整个人档案
- `Rust` 是主运行时，现有 `Context Layer + Harness + Tool Registry` 继续作为可信执行基础
- 第一条必须跑顺的新闭环是 `全新 Agent 任务 -> 调用 Aurora MCP -> 获得受限 Context Pack -> 无需重复介绍即可回答`
- 第一版总优先级是：`准确 -> 授权 -> 可审计 -> 跨 Agent -> 再谈自动记忆和陪伴界面`

## 2. 第一版成功定义

AuroraPulse 下一版成功，不是“自己更会聊天”，而是下面这条跨 Agent 链路稳定可用：

1. 用户只在 Aurora 中维护一次本地、可编辑的个人身份与当前上下文
2. 用户在一个全新的 Codex 任务中提出需要个人背景的问题
3. Codex 通过本地 MCP 调用 Aurora，而不是要求用户重新介绍自己
4. Aurora 根据 Agent、任务目的和隐私策略选择最小必要上下文
5. Aurora 返回带来源信息、可预览和可审计的 `Context Pack`
6. Codex 使用这些上下文准确回答，同时看不到未授权或无关的信息

一句话说，就是：

`一次告诉 Aurora -> 本地个人事实源 -> 权限过滤 -> Aurora MCP -> 任意授权 Agent`

## 3. 分阶段计划表

| 阶段 | 目标 | 关键产物 | 当前不做 | 成功标准 |
| --- | --- | --- | --- | --- |
| `Phase 0` | 稳住 Rust CLI 主干 | Rust CLI、Ollama 接入、基础 session、清晰错误提示 | 不做语音、不做多工具扩张 | CLI 可以稳定完成文本输入到短回复闭环 |
| `Phase 1` | 落地 `Identity Card` 和 `Context Bundle` | identity-card、current-focus、preferences、privacy-rules、context preview | 不做自动长期记忆、不做全盘扫描 | 模型第一句话能知道用户是谁 |
| `Phase 2` | 抽出 `Model Provider` 层 | Ollama provider、provider 选择参数、未来 OpenAI/Anthropic/Gemini 接口边界 | 不把云端 API 当记忆源 | 同一份 context bundle 可送给不同 provider |
| `Phase 3` | 落地 `Custom Harness` 和 `Unified Tool Layer` | planner schema、action validation、clarify 分支、tool registry、tool result normalization | 不继续把业务逻辑写死在 CLI 分支里 | 新增工具不需要重写主循环 |
| `Phase 4` | 建立 `MCP Identity Server` | 本地 stdio MCP、只读身份工具、任务级 Context Pack、调用审计 | 不做自动记忆写入、不做泛文档检索 | 全新 Codex 任务可经授权认识用户，无需重复介绍 |
| `Phase 5` | 建立 `Durable Personal Memory` | 持久会话、事实与经历、来源、纠正与遗忘、相关记忆召回 | 不让模型静默改写身份 | 换会话、重启或换模型后仍能准确召回相关个人信息 |
| `Phase 6` | 接入 Agent 记忆提议 | `propose_memory`、冲突检测、用户审批、Agent 级写权限 | 不开放任意直接写入 | Agent 可以建议记忆，但只有用户能批准身份和长期记忆变更 |
| `Phase 7` | 扩展 Companion 与连接入口 | 本地陪伴界面、菜单栏、语音、更多 MCP Agent 或平台适配器 | 不做全天监听、不追求通用超级 Agent | 同一份 Aurora 记忆可在多个授权入口中保持连续体验 |

### Phase 3 完成状态（2026-07-16）

`Phase 3` 已完成，当前主链路为：

`CLI -> Context Bundle -> Model Provider -> PlannerDecision -> Harness -> ToolRegistry -> ToolResult`

完成项：

- planner 内部 JSON 已收束为 `chat / clarify / tool / retrieve` 四种 Rust 枚举分支
- 工具目录由 `ToolRegistry` 动态生成并注入模型，不再在 provider prompt 中写死
- Registry 统一校验工具名、必填参数和参数类型
- 风险策略统一为：Low 直接执行、Medium 支持本次或会话放行、High 每次确认
- 工具结果统一为 `succeeded / failed / denied`，保留结构化 data 和 error
- Harness 保存最近 32 条调用结果及耗时，可通过 `/tools log` 检查
- 外部命令型工具具有 20 秒执行边界
- 新增 native tool 不需要修改 CLI 主循环或 provider 实现

### Phase 4 完成状态（2026-07-18）

`Phase 4 MCP Identity Server` 已完成，真实链路为：

`Fresh Codex Task -> Aurora MCP -> DisclosurePolicy -> ContextPack -> Codex Answer -> Local Audit Log`

完成项：

- `aurora serve [workspace]` 启动本地 stdio MCP Server
- 提供 `get_identity`、`get_current_focus`、`search_personal_context` 三个只读工具
- 工具声明只读、非破坏、幂等和封闭世界安全提示
- Context Pack 返回稳定的 `aurora://` / `workspace://` 来源 URI，不暴露本机绝对路径
- MCP 使用 `privacy-rules.json` 中的动态 `redaction_markers` 过滤敏感行
- 每次成功或失败调用写入 `~/.aurorapulse/audit/mcp.jsonl`
- CLI 通过 `/mcp log` 检查最近的外部 Agent 上下文访问
- release 二进制已注册为 Codex 全局 MCP Server `aurora`
- 全新、空白、只读 Codex 任务已自动调用身份与当前关注工具，并引用两个 Aurora 来源完成回答

下一阶段正式进入 `Phase 5 Durable Personal Memory`，重点解决事实源更新、持久会话、纠正、遗忘和相关记忆召回。

## 4. 历史执行表（Phase 0-3 已完成）

下面是已经完成的早期落地顺序，保留用于回顾架构演进。

| 顺序 | 工作项 | 目的 | 交付物 |
| --- | --- | --- | --- |
| `1` | 定义本地 identity 文件 | 把“你是谁”变成可编辑对象 | `identity-card.md`、`current-focus.md`、`preferences.json`、`privacy-rules.json` |
| `2` | 实现 context loader | 从本机读取身份、focus、偏好和当前项目上下文 | `src/context/` 雏形 |
| `3` | 实现 context bundle preview | 让用户看到要发给模型的内容 | `aurora context preview` |
| `4` | 抽象 model provider | 为本地模型和未来云端 API 铺路 | `OllamaProvider` + provider trait |
| `5` | 串进 `aurora ask` | 第一问就带身份上下文 | `aurora ask --provider ollama "..."` |
| `6` | 加 provider 隐私规则 | 云端 API 默认只拿最小 identity card | provider-aware privacy filtering |
| `7` | 加项目上下文探测 | 当前 repo 自动读取 `CONTEXT.md` / `AGENTS.md` / `CLAUDE.md` | 项目层上下文进入 bundle |
| `8` | 加结构化 Planner 与 Harness | 让模型只能提出可校验决策 | `chat / clarify / tool / retrieve` 决策边界 |
| `9` | 加统一工具与权限层 | 让执行权留在本地 runtime | Tool Registry、风险策略、标准化结果 |
| `10` | 补测试、日志和架构资料 | 固化 Phase 0-3 的可信基础 | runtime tests、tool logs、phase diagrams |

## 5. 每阶段都要守住的约束

- 模型只负责 `Model Role`，不直接拥有系统控制权
- 所有模型输出都要满足 `Validated Model Output`
- 所有执行结果都以 `Tool Reality Precedence` 为准
- 默认交互形态是 `Single-Step Action`
- 有歧义先 `Clarification`
- 文件类操作遵守 `Read-First File Handling`
- 高风险动作遵守 `Action Risk Confirmation`
- 上下文遵守 `Selective Context Loading`
- MCP 输出遵守 `Minimum Necessary Disclosure`
- Agent 默认只有只读权限，任何长期记忆写入都需要用户批准
- 每个 Context Pack 必须保留来源并支持本地审计
- 语音反馈遵守 `Short Voice Feedback`

## 6. 已完成基础与下一里程碑

### Milestone A：本地身份文件成型

- 创建 `identity-card.md`
- 创建 `current-focus.md`
- 创建 `preferences.json`
- 创建 `privacy-rules.json`
- 文件必须是用户可直接打开、修改、删除的明文资料

### Milestone B：Context Layer 成型

- 能加载身份、focus、偏好和隐私规则
- 能探测当前项目里的 `CONTEXT.md` / `AGENTS.md` / `CLAUDE.md`
- 能按 provider 生成最小 `context bundle`
- 能预览本次会发给模型的上下文

### Milestone C：Provider Layer 成型

- 把当前 Ollama 调用包成 `OllamaProvider`
- CLI 支持选择 provider
- provider 接口为未来 OpenAI / Anthropic / Gemini API 预留
- 云端 provider 默认走更严格的隐私过滤

### Milestone D：第一问体验跑通

- `aurora ask "我下一步应该做什么？"` 能读取 identity card 和 current focus
- 回答明显体现“知道用户是谁”
- 同一套上下文逻辑不绑定某个模型 provider
- context bundle 可审计、可删减、可重新生成

### Milestone E：跨 Agent 身份闭环（已完成）

- 新增本地 `aurora serve` MCP 入口
- 首版只提供 `get_identity`、`get_current_focus`、`search_personal_context`
- 每次调用按 Agent、purpose 和隐私策略生成最小 Context Pack
- 用户可以预览返回内容和查看 MCP 调用记录
- 在全新 Codex 任务中完成一次无需重复身份介绍的真实调用

## 7. 当前最推荐的马上开工项

如果只选接下来最该做的 3 件事，建议按这个顺序：

1. 定义 `Fact / Episode / Open Thread` 三类个人记忆记录，以及来源和更新时间
2. 让用户查看、纠正、遗忘个人记忆，先不开放 Agent 直接写入
3. 将相关记忆召回接入现有 Context Pack，并验证换会话后无需重复提供精确信息

这三步构成 `Phase 5 Durable Personal Memory` 的第一个可验证闭环。PDF、网页剪藏、向量数据库、语音和 Agent 自动写入仍然顺延。

## 8. 历史 7 天计划（已完成）

下面这份计划默认按“连续 7 个工作日、每天有完整开发时间”来排。

### Day 1：定身份数据结构

- 设计 `identity-card.md`
- 设计 `current-focus.md`
- 设计 `preferences.json`
- 设计 `privacy-rules.json`
- 加初始化命令或样例文件

当天目标：
用户能打开本地文件，清楚地修改“我是谁”和“我最近在做什么”。

### Day 2：实现 Context Loader

- 新增 `src/context/`
- 读取 identity、focus、preferences、privacy rules
- 读取当前项目的 `CONTEXT.md` / `AGENTS.md` / `CLAUDE.md`
- 对缺失文件给出温和 fallback

当天目标：
AuroraPulse 能在模型调用前拿到本机身份上下文。

### Day 3：实现 Context Bundle Preview

- 新增 `aurora context preview`
- 输出本次将注入模型的 identity、focus、project context
- 标出来源文件
- 标出因隐私规则被排除的字段

当天目标：
用户能审计“模型这次会知道我什么”。

### Day 4：抽象 Model Provider

- 新增 provider trait / enum
- 把现有 Ollama 调用包成 `OllamaProvider`
- CLI 支持 `--provider ollama`
- 预留 `openai`、`anthropic`、`gemini` 配置位，但不急着实现联网调用

当天目标：
模型不再直接等于 Ollama，AuroraPulse 的记忆不绑定 provider。

### Day 5：串起 Identity-Aware Ask

- 新增或调整 `aurora ask`
- 在用户请求前拼入 context bundle
- 保持 planner 输出结构化和可校验
- 回答时体现身份、focus、当前项目语境

当天目标：
第一句话体验成立：模型上来就知道用户是谁。

### Day 6：加隐私和 provider 策略

- 区分 local provider 和 cloud provider
- 云端 provider 默认只拿短 identity card
- 支持 privacy rule 阻止某些字段进入 bundle
- 测试未授权目录不会被读取

当天目标：
未来接 GPT / Claude / Gemini API 时，不会把本机个人信息无脑发送出去。

### Day 7：联调、测试、样例

- 补 context loader 测试
- 补 bundle preview 测试
- 补 provider 选择测试
- 写一个真实 `identity-card` 样例
- 对比裸模型回答和 AuroraPulse 注入后的回答

当天目标：
V1 可以演示“无账号 API / 本地模型也有账号级身份体验”。

## 9. 如果每天时间不完整

如果不是全职推进，可以这样理解：

- `1 天` 的工作量大约等于 `1 个完整开发日`
- 如果每天只能投入 `2-4 小时`，上面的 7 天计划更接近 `2 周左右`
- 优先级不要变，宁可拉长时间，也不要提前并行太多模块

## 10. 任务表：按文件和模块拆

这一版任务表默认以当前 Rust 结构为基础：

- 已有文件：`src/main.rs`、`src/cli.rs`、`src/config.rs`、`src/session.rs`
- 当前新增结构：`src/context/`、`src/model/`

### A. 现有文件要做什么

| 文件 | 当前职责 | 要做的任务 | 完成标准 |
| --- | --- | --- | --- |
| `src/main.rs` | 入口，加载 config 后进入 CLI | 改成更薄的启动入口，只负责组装 app 和启动 mode | `main.rs` 不再承载业务流程 |
| `src/cli.rs` | Banner + 文本 REPL | 增加 `ask` 和 `context preview` 的入口 | CLI 可以展示将注入模型的上下文 |
| `src/config.rs` | 读取 workspace、dotenv、模型配置 | 扩展 identity 文件路径、provider 配置、隐私策略配置 | 配置结构能支撑本地和未来云端 provider |
| `src/model/ollama.rs` | Ollama provider | 封装本地 Ollama 调用 | 模型调用接口不再和 CLI 会话逻辑耦合 |
| `src/session.rs` | 简单聊天历史 | 收紧成 working session，只保留最近上下文；不要让它承担长期记忆 | 只服务短时上下文和最近 turn |

### B. 需要新增的核心模块

| 模块 | 作用 | 主要任务 | 完成标准 |
| --- | --- | --- | --- |
| `src/context/mod.rs` | context 门面 | 读取本地身份文件、项目上下文，生成 preview 和 bundle | 上层不需要知道具体文件布局 |
| `src/model/mod.rs` | 模型 provider 抽象 | 定义 provider trait / enum | Ollama 和未来 API 共用同一入口 |
| `src/model/ollama.rs` | Ollama provider | 调本地 Ollama chat API | 本地 provider 路径稳定 |
| `src/model/openai.rs` | OpenAI-compatible provider | 调 `/v1/chat/completions` | 云端 provider 路径可验证 |

### C. Day 1 到 Day 7 的具体任务表

| 天数 | 主要文件 | 具体任务 | 当天产出 |
| --- | --- | --- | --- |
| `Day 1` | `src/config.rs` 样例本地文件 | 定身份文件格式和默认路径 | 可编辑 identity 文件 |
| `Day 2` | `src/context/mod.rs` | 实现 context loader | 能读取身份和 focus |
| `Day 3` | `src/context/mod.rs` `src/cli.rs` | 实现 context preview | 可审计 context bundle |
| `Day 4` | `src/model/mod.rs` `src/model/ollama.rs` | 抽象 provider 并接入 Ollama | provider 可替换 |
| `Day 5` | `src/app.rs` `src/harness.rs` `src/cli.rs` | 串起 identity-aware ask | 第一问知道用户是谁 |
| `Day 6` | `src/context/mod.rs` tests | 加 provider 隐私过滤 | 云端 API 铺路 |
| `Day 7` | tests docs examples | 联调、补测试、写样例 | V1 可演示 |

### D. 每天的最小交付检查

每天结束前至少检查这 4 件事：

1. 当天新增代码是否还符合 `Single-Step Action`
2. context bundle 是否可预览、可审计
3. 未授权目录和隐私字段是否没有进入 bundle
4. provider 抽象是否没有把记忆绑定到某个模型厂商

### E. 写代码的推荐顺序

如果你每天只想盯最关键的一组文件，可以按这个节奏：

1. 先盯 `config + identity-card + current-focus`
2. 再盯 `context loader + bundle preview`
3. 然后盯 `model/mod.rs + model/ollama.rs + model/openai.rs`
4. 最后盯 `ask + privacy filtering + tests`

也就是说：

先让模型知道用户是谁，再考虑工具和语音入口。
