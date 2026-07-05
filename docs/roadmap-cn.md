# AuroraPulse 执行计划表

## 1. 当前共识

基于 `CONTEXT.md` 和现有 ADR，AuroraPulse 当前已经定下来的方向是：

- 项目身份是 `Local Daily Assistant`，不是音乐专用助手，也不是通用超级 agent
- V1 的核心对象是可编辑的 `Identity Card`，让模型第一句话之前就知道用户是谁
- AuroraPulse 的个人记忆归本机所有，不依赖 GPT / Claude / Gemini 的账号级记忆
- `Rust` 是主运行时，Python 保留为原型参考
- 核心架构是 `Custom Harness + Unified Tool Layer`
- 第一版交互主线先回到 `CLI -> Identity Card -> Context Bundle -> Model Provider`
- 第一条必须跑顺的闭环是 `首次提问 -> 模型知道用户是谁 -> 回答体现当前 focus`
- 第一版总优先级是：`稳 -> 准 -> 自然 -> 再谈更炫的能力`

## 2. 第一版成功定义

AuroraPulse 第一版成功，不是“功能很多”，而是下面这条链路稳定可用：

1. 用户维护一张本地、可编辑的 `identity-card`
2. 用户发出第一句自然语言请求
3. AuroraPulse 本地读取 identity、current focus、偏好和当前项目上下文
4. AuroraPulse 生成可预览、可审计的 `context bundle`
5. 所选模型 provider 在第一次回答前就拿到最小必要上下文
6. 回答能体现“知道用户是谁”和“用户最近在做什么”

一句话说，就是：

`第一句话 -> 本地身份上下文 -> 任意模型 provider -> 个性化回答`

## 3. 分阶段计划表

| 阶段 | 目标 | 关键产物 | 当前不做 | 成功标准 |
| --- | --- | --- | --- | --- |
| `Phase 0` | 稳住 Rust CLI 主干 | Rust CLI、Ollama 接入、基础 session、清晰错误提示 | 不做语音、不做多工具扩张 | CLI 可以稳定完成文本输入到短回复闭环 |
| `Phase 1` | 落地 `Identity Card` 和 `Context Bundle` | identity-card、current-focus、preferences、privacy-rules、context preview | 不做自动长期记忆、不做全盘扫描 | 模型第一句话能知道用户是谁 |
| `Phase 2` | 抽出 `Model Provider` 层 | Ollama provider、provider 选择参数、未来 OpenAI/Anthropic/Gemini 接口边界 | 不把云端 API 当记忆源 | 同一份 context bundle 可送给不同 provider |
| `Phase 3` | 落地 `Custom Harness` 和 `Unified Tool Layer` | planner schema、action validation、clarify 分支、tool registry、tool result normalization | 不继续把业务逻辑写死在 CLI 分支里 | 新增工具不需要重写主循环 |
| `Phase 4` | 接入 `Local Knowledge` | Markdown/文档/文本检索、source-aware answering、轻量 provenance | 不急着吞 PDF、网页剪藏、数据库 | 能基于本地资料做简短回答，并在需要时指出来源 |
| `Phase 5` | 跑通 `Voice Loop` | 全局快捷键、录音、STT、TTS 短反馈、当前任务状态 | 不做全天监听、不做长语音对话 | 用户可按快捷键提问，系统基于身份上下文短语音回答 |
| `Phase 6` | 扩展工具生态 | MCP adapter、更多本地或外部工具接入 | 不把 MCP 当产品核心 runtime | 新工具接入成本明显下降，核心 harness 仍然清晰可控 |

## 4. 近期执行表

这是最适合现在直接开干的顺序。

| 顺序 | 工作项 | 目的 | 交付物 |
| --- | --- | --- | --- |
| `1` | 定义本地 identity 文件 | 把“你是谁”变成可编辑对象 | `identity-card.md`、`current-focus.md`、`preferences.json`、`privacy-rules.json` |
| `2` | 实现 context loader | 从本机读取身份、focus、偏好和当前项目上下文 | `src/context/` 雏形 |
| `3` | 实现 context bundle preview | 让用户看到要发给模型的内容 | `aurora context preview` |
| `4` | 抽象 model provider | 为本地模型和未来云端 API 铺路 | `OllamaProvider` + provider trait |
| `5` | 串进 `aurora ask` | 第一问就带身份上下文 | `aurora ask --provider ollama "..."` |
| `6` | 加 provider 隐私规则 | 云端 API 默认只拿最小 identity card | provider-aware privacy filtering |
| `7` | 加项目上下文探测 | 当前 repo 自动读取 `CONTEXT.md` / `AGENTS.md` / `CLAUDE.md` | 项目层上下文进入 bundle |
| `8` | 加最小本地检索 | 只扫授权目录里的 Markdown / txt | 简单关键词检索 + 来源 |
| `9` | 补测试和样例 | 确认不会读未授权内容 | context / provider / preview 测试 |
| `10` | 再扩到 tools / voice | 在身份上下文稳定后接日常能力 | 本地笔记、启动、语音入口 |

## 5. 每阶段都要守住的约束

- 模型只负责 `Model Role`，不直接拥有系统控制权
- 所有模型输出都要满足 `Validated Model Output`
- 所有执行结果都以 `Tool Reality Precedence` 为准
- 默认交互形态是 `Single-Step Action`
- 有歧义先 `Clarification`
- 文件类操作遵守 `Read-First File Handling`
- 高风险动作遵守 `Action Risk Confirmation`
- 上下文遵守 `Selective Context Loading`
- 语音反馈遵守 `Short Voice Feedback`

## 6. V1 里程碑拆分

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

## 7. 当前最推荐的马上开工项

如果只选接下来最该做的 3 件事，建议按这个顺序：

1. 先把本地 `identity-card` 和 `current-focus` 文件格式定下来
2. 再实现 `context loader + context bundle preview`
3. 最后把 `aurora ask` 改成先构建上下文，再调用 `OllamaProvider`

这三步做完，AuroraPulse 才真正解决“API / 本地模型第一句话不知道你是谁”的问题。

## 8. 按天拆分的 7 天计划

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

- 已有文件：`src/main.rs`、`src/cli.rs`、`src/config.rs`、`src/ollama.rs`、`src/session.rs`
- 接下来会新增：`src/context/`、`src/model_provider.rs`、`src/context.rs`

### A. 现有文件要做什么

| 文件 | 当前职责 | 要做的任务 | 完成标准 |
| --- | --- | --- | --- |
| `src/main.rs` | 入口，加载 config 后进入 CLI | 改成更薄的启动入口，只负责组装 app 和启动 mode | `main.rs` 不再承载业务流程 |
| `src/cli.rs` | Banner + 文本 REPL | 增加 `ask` 和 `context preview` 的入口 | CLI 可以展示将注入模型的上下文 |
| `src/config.rs` | 读取 workspace、dotenv、模型配置 | 扩展 identity 文件路径、provider 配置、隐私策略配置 | 配置结构能支撑本地和未来云端 provider |
| `src/ollama.rs` | 直接 chat 请求 | 收进 `OllamaProvider`，不再让上层直接依赖 Ollama | 模型调用接口不再和 CLI 会话逻辑耦合 |
| `src/session.rs` | 简单聊天历史 | 收紧成 working session，只保留最近上下文；不要让它承担长期记忆 | 只服务短时上下文和最近 turn |

### B. 需要新增的核心模块

| 模块 | 作用 | 主要任务 | 完成标准 |
| --- | --- | --- | --- |
| `src/context.rs` | context 门面 | 对外暴露加载和 bundle 生成能力 | 上层不需要知道具体文件布局 |
| `src/context/identity.rs` | 身份卡读取 | 读取 `identity-card.md` | 模型可获得“你是谁”的短上下文 |
| `src/context/focus.rs` | 当前 focus | 读取 `current-focus.md` | 模型可知道用户最近在做什么 |
| `src/context/preferences.rs` | 偏好读取 | 读取稳定偏好 | 回答风格和工作偏好可进入 bundle |
| `src/context/privacy.rs` | 隐私规则 | 根据 provider 过滤字段 | 云端 API 不会默认拿到过宽上下文 |
| `src/context/project.rs` | 项目上下文 | 探测 `CONTEXT.md` / `AGENTS.md` / `CLAUDE.md` | 项目层信息不混进个人长期记忆 |
| `src/context/bundle.rs` | bundle 生成 | 组装、预览、标注来源 | context bundle 可审计 |
| `src/model_provider.rs` | 模型 provider 抽象 | 定义 provider trait / enum | Ollama 和未来 API 共用同一入口 |

### C. Day 1 到 Day 7 的具体任务表

| 天数 | 主要文件 | 具体任务 | 当天产出 |
| --- | --- | --- | --- |
| `Day 1` | `src/config.rs` 样例本地文件 | 定身份文件格式和默认路径 | 可编辑 identity 文件 |
| `Day 2` | `src/context.rs` `src/context/*.rs` | 实现 context loader | 能读取身份和 focus |
| `Day 3` | `src/context/bundle.rs` `src/cli.rs` | 实现 context preview | 可审计 context bundle |
| `Day 4` | `src/model_provider.rs` `src/ollama.rs` | 抽象 provider 并接入 Ollama | provider 可替换 |
| `Day 5` | `src/app.rs` `src/harness.rs` `src/cli.rs` | 串起 identity-aware ask | 第一问知道用户是谁 |
| `Day 6` | `src/context/privacy.rs` tests | 加 provider 隐私过滤 | 云端 API 铺路 |
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
3. 然后盯 `model_provider + ollama`
4. 最后盯 `ask + privacy filtering + tests`

也就是说：

先让模型知道用户是谁，再考虑工具和语音入口。
