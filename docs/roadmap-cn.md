# AuroraPulse 执行计划表

## 1. 当前共识

基于 `CONTEXT.md` 和现有 ADR，AuroraPulse 当前已经定下来的方向是：

- 项目身份是 `Local Daily Assistant`，不是音乐专用助手，也不是通用超级 agent
- `Rust` 是主运行时，Python 保留为原型参考
- 核心架构是 `Custom Harness + Unified Tool Layer`
- 第一版交互主线是 `Global Shortcut Entry -> Voice-First Interaction -> Short Spoken Reply Loop`
- 第一条必须跑顺的闭环是 `Music-First Voice Loop`
- 第一版总优先级是：`稳 -> 准 -> 自然 -> 再谈更炫的能力`

## 2. 第一版成功定义

AuroraPulse 第一版成功，不是“功能很多”，而是下面这条链路稳定可用：

1. 用户按下全局快捷键唤起 AuroraPulse
2. 用户说一句自然语言请求
3. 系统完成一次 `Single-Step Action`
4. 工具返回真实结果，满足 `Observed Task Completion`
5. AuroraPulse 用很短的 TTS 给出反馈

一句话说，就是：

`唤起 -> 语音输入 -> 正确执行 -> 短语音反馈`

## 3. 分阶段计划表

| 阶段 | 目标 | 关键产物 | 当前不做 | 成功标准 |
| --- | --- | --- | --- | --- |
| `Phase 0` | 稳住 Rust CLI 主干 | Rust CLI、Ollama 接入、基础 session、清晰错误提示 | 不做语音、不做多工具扩张 | CLI 可以稳定完成文本输入到短回复闭环 |
| `Phase 1` | 落地 `Custom Harness` 和 `Unified Tool Layer` | planner schema、action validation、clarify 分支、tool registry、tool result normalization | 不继续把业务逻辑写死在 CLI 分支里 | 新增工具不需要重写主循环 |
| `Phase 2` | 跑通 `Music-First Voice Loop` | 全局快捷键、录音、STT、Spotify tool、TTS 短反馈、当前任务状态 | 不做全天监听、不做长语音对话 | 用户可按快捷键说“播放周杰伦”，系统能播歌并短语音回答 |
| `Phase 3` | 补齐第一批本地日常能力 | `Local Note`、`Local Launch`、read-first 文件操作、基础偏好设置 | 不做提醒常驻、不做复杂 GUI 自动化 | 每天可完成几类真实小事，且行为稳定 |
| `Phase 4` | 接入 `Local Knowledge` | Markdown/文档/文本检索、source-aware answering、轻量 provenance | 不急着吞 PDF、网页剪藏、数据库 | 能基于本地资料做简短回答，并在需要时指出来源 |
| `Phase 5` | 做 `Resident Runtime` | 后台常驻进程、提醒触发、前后台状态管理 | 不做自主长链执行、不做 always-listening | 提醒真正能主动冒出来，助理无需每次手动开 CLI |
| `Phase 6` | 扩展工具生态 | MCP adapter、更多本地或外部工具接入 | 不把 MCP 当产品核心 runtime | 新工具接入成本明显下降，核心 harness 仍然清晰可控 |

## 4. 近期执行表

这是最适合现在直接开干的顺序。

| 顺序 | 工作项 | 目的 | 交付物 |
| --- | --- | --- | --- |
| `1` | 清理 Rust CLI 主入口 | 让当前主线只保留 Rust 运行时思路 | `src/` 结构清晰、错误提示统一、基础 session 可用 |
| `2` | 设计 planner 输出 schema | 为 `Validated Model Output` 打底 | 明确的 action/mode schema 和参数校验规则 |
| `3` | 实现 harness 主循环 | 把模型理解、校验、澄清、工具调用拆开 | `harness`、`session`、`tool registry` 雏形 |
| `4` | 把 Spotify 迁进统一工具层 | 让第一条工具链路走正确架构 | `music` 工具注册、执行、结果归一化 |
| `5` | 增加 clarify 和失败引导 | 对齐 `Clarification` 和 `Guided Failure Feedback` | 歧义追问、短失败反馈、小步恢复 |
| `6` | 接入全局快捷键和录音 | 为语音主线做入口 | 唤起模块、录音模块、可调用的语音入口 |
| `7` | 接入本地 STT + TTS | 跑通 `Wake-to-Voice Success Loop` | 语音转文本、短语音反馈 |
| `8` | 增加当前任务状态 | 支撑“这首歌为什么好听”这类短追问 | 当前歌曲、最近工具动作等短时状态 |
| `9` | 做音乐相关短解释 | 让它“不像个傻子”但不跑题 | 围绕当前播放上下文的短解释能力 |
| `10` | 再扩到 notes / launch / files | 让它从音乐 demo 变成最小日常助理 | 本地笔记、本地启动、read-first 文件能力 |

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

## 6. 第一阶段里程碑拆分

### Milestone A：Rust 骨架稳定

- 跑通 CLI 输入输出
- 稳定调用本地模型
- 保留最小 session
- 统一错误信息

### Milestone B：Harness 成型

- planner 输出结构化
- action 校验落地
- clarify 分支落地
- tool registry 能注册和调用 Spotify

### Milestone C：音乐语音闭环跑通

- 全局快捷键可唤起
- 录音和 STT 可用
- Spotify 工具执行稳定
- TTS 给出简短反馈

### Milestone D：短追问自然可用

- 维护 `Current Task State`
- 支持围绕当前播放内容的短解释
- 保持语音短反馈，不演变成长聊天

## 7. 当前最推荐的马上开工项

如果只选接下来最该做的 3 件事，建议按这个顺序：

1. 先把 Rust 里的 `harness + schema + tool layer` 打出来
2. 再把 Spotify 迁进统一工具层，作为第一条标准工具链
3. 最后接全局快捷键、STT、TTS，跑通音乐语音闭环

这三步做完，AuroraPulse 才算真正从“本地聊天壳”进入“本地日常助理”的主轨道。

## 8. 按天拆分的 10 天计划

下面这份计划默认按“连续 10 个工作日、每天有完整开发时间”来排。

### Day 1：收 Rust 主线，清理入口

- 盘一遍 `src/` 当前结构
- 明确哪些 Python 行为只是参考，哪些必须迁到 Rust
- 清理 `main / cli / config / session / ollama` 的边界
- 把当前 CLI 跑通并统一错误输出

当天目标：
Rust CLI 成为唯一明确主线，最小文本对话壳稳定可跑。

### Day 2：定 planner schema

- 设计第一版 action schema
- 明确 `tool / clarify / reply` 这几种模式
- 定参数校验规则
- 定失败时的回退路径

当天目标：
把 `Validated Model Output` 从理念变成可编码的数据结构。

### Day 3：搭 harness 雏形

- 写第一版 harness 主循环
- 把“读输入 -> 调模型 -> 校验 -> 分发”串起来
- 把 session 和当前任务状态的最小结构补出来
- 先不接复杂工具，只把骨架跑顺

当天目标：
主循环不再写死在 CLI 分支里，而是开始变成真正的 assistant runtime。

### Day 4：做 unified tool layer

- 设计 tool trait / registry / result shape
- 定义工具输入输出的统一格式
- 把工具返回值和用户回复拆开
- 把“工具真实结果优先于模型判断”落地到接口里

当天目标：
后面加音乐、笔记、文件、启动都能走同一种调用方式。

### Day 5：迁 Spotify 到标准工具链

- 把 Spotify 相关能力迁成正式工具
- 支持播放歌曲、播放歌手、暂停、继续、切歌、音量
- 对齐错误提示和观察结果
- 补最基本的歧义处理分支

当天目标：
第一条真实工具链路通过统一架构跑通。

### Day 6：补 clarify 和短失败反馈

- 处理“放周杰伦”这类模糊输入
- 加上澄清问题分支
- 把失败反馈压短
- 让失败时默认带一个小的下一步提示

当天目标：
AuroraPulse 开始像“靠谱助理”，而不是“要么猜、要么报错”的壳。

### Day 7：接全局快捷键和录音

- 选定 macOS 下的快捷键接入方案
- 接录音模块
- 做按下唤起、说完结束的基础体验
- 保留文本入口作为后备

当天目标：
从纯 CLI 进入真正的 voice entry 阶段。

### Day 8：接本地 STT 和短 TTS

- 接本地 STT
- 接短 TTS 反馈
- 串起“快捷键 -> 录音 -> 转文字 -> 执行 -> 语音回一句”
- 优化语音链路里的超时和失败提示

当天目标：
`Wake-to-Voice Success Loop` 第一次完整跑通。

### Day 9：补 current task state 和音乐短解释

- 保存当前播放歌曲和最近一次动作
- 支持“这首歌为什么好听”这类短追问
- 让解释围绕当前上下文，不发散成长聊天
- 收紧语音反馈长度

当天目标：
它不只是会播歌，还能围绕当前任务给出一点聪明但短的解释。

### Day 10：联调、修边角、定下一阶段

- 把整条音乐语音闭环反复走一遍
- 修掉最影响体验的错误和卡顿
- 记录还缺什么才能进 `Local Note / Local Launch / Local Knowledge`
- 整理下一阶段任务列表

当天目标：
拿到一个可以真实试用的音乐优先语音助理雏形。

## 9. 如果每天时间不完整

如果不是全职推进，可以这样理解：

- `1 天` 的工作量大约等于 `1 个完整开发日`
- 如果每天只能投入 `2-4 小时`，上面的 10 天计划更接近 `3 周左右`
- 优先级不要变，宁可拉长时间，也不要提前并行太多模块

## 10. 任务表：按文件和模块拆

这一版任务表默认以当前 Rust 结构为基础：

- 已有文件：`src/main.rs`、`src/cli.rs`、`src/config.rs`、`src/ollama.rs`、`src/session.rs`
- 接下来会新增：`src/harness.rs`、`src/schema.rs`、`src/tools/`、`src/audio/`、`src/state.rs`

### A. 现有文件要做什么

| 文件 | 当前职责 | 要做的任务 | 完成标准 |
| --- | --- | --- | --- |
| `src/main.rs` | 入口，加载 config 后进入 CLI | 改成更薄的启动入口，只负责组装 app 和启动 mode | `main.rs` 不再承载业务流程 |
| `src/cli.rs` | Banner + 文本 REPL | 从“主逻辑”降级为纯接口层，只负责文本输入输出和用户可见反馈 | CLI 只做界面，不做 planner 和 tool dispatch |
| `src/config.rs` | 读取 workspace、dotenv、模型配置 | 扩展语音和后台需要的配置项，比如快捷键、音频、TTS/STT 路径 | 配置结构能支撑文本和语音两种入口 |
| `src/ollama.rs` | 直接 chat 请求 | 从“直接聊天”抽成更清晰的模型客户端，支持 planner / reply 两类调用 | 模型调用接口不再和 CLI 会话逻辑耦合 |
| `src/session.rs` | 简单聊天历史 | 收紧成 working session，只保留最近上下文；不要让它承担长期记忆 | 只服务短时上下文和最近 turn |

### B. 需要新增的核心模块

| 模块 | 作用 | 主要任务 | 完成标准 |
| --- | --- | --- | --- |
| `src/harness.rs` | 自研主循环 | 接收请求、加载上下文、调 planner、校验、分发、整合回复 | Rust 主循环从这里统一流转 |
| `src/schema.rs` | planner 输出结构 | 定义 `reply / tool / clarify` 模式和参数结构 | 模型输出可以被严格解析和校验 |
| `src/state.rs` | 当前任务状态 | 保存当前播放歌曲、最近工具结果、最近动作 | 能支持“这首歌为什么好听”这类追问 |
| `src/tools/mod.rs` | 工具抽象层 | 定义 tool trait、registry、统一 result shape | 所有工具可通过一致接口注册和调用 |
| `src/tools/music.rs` | 音乐工具 | 把 Spotify 行为接成标准工具 | 音乐能力不再写死在 agent/CLI 分支里 |
| `src/tools/notes.rs` | 本地笔记工具 | 创建、追加、读取本地笔记 | 能面向本地笔记目录稳定工作 |
| `src/tools/files.rs` | 文件读取工具 | 搜索、打开、总结本地文本内容 | 默认 read-first，不擅自改文件 |
| `src/tools/launch.rs` | 本地启动工具 | 打开 app、文件夹、文件 | 只做轻启动，不做 GUI 自动化 |
| `src/audio/input.rs` | 录音入口 | 快捷键唤起后的录音控制 | 能稳定开始/结束录音 |
| `src/audio/stt.rs` | 本地转写 | 把语音转成文本请求 | STT 结果可送进 harness |
| `src/audio/tts.rs` | 语音反馈 | 播放很短的反馈语音 | 成功、失败、澄清都能短反馈 |
| `src/audio/hotkey.rs` | 全局快捷键 | 监听唤起动作 | 语音入口不依赖手动开 CLI |

### C. Day 1 到 Day 10 的具体任务表

| 天数 | 主要文件 | 具体任务 | 当天产出 |
| --- | --- | --- | --- |
| `Day 1` | `src/main.rs` `src/cli.rs` `src/config.rs` | 清理启动流程，收 Rust 主线，统一错误输出，确认配置入口 | 一个干净可跑的 Rust CLI 壳 |
| `Day 2` | `src/schema.rs` `src/ollama.rs` | 新增 schema，拆 planner / reply 调用，明确 action 结构 | 第一版结构化 planner 输出 |
| `Day 3` | `src/harness.rs` `src/session.rs` | 新增 harness，改 session 为短时 working session | 主循环和 CLI 解耦 |
| `Day 4` | `src/tools/mod.rs` `src/state.rs` | 新增 tool registry 和当前任务状态结构 | 统一工具层雏形 + 短时状态 |
| `Day 5` | `src/tools/music.rs` `src/harness.rs` | 迁 Spotify 链路进统一工具层 | 第一条标准工具链跑通 |
| `Day 6` | `src/schema.rs` `src/harness.rs` `src/cli.rs` | 增 clarify 分支、短失败反馈、参数兜底 | 更像可靠助理的文本版流程 |
| `Day 7` | `src/audio/hotkey.rs` `src/audio/input.rs` `src/config.rs` | 接全局快捷键和录音控制 | 可唤起的语音入口 |
| `Day 8` | `src/audio/stt.rs` `src/audio/tts.rs` `src/harness.rs` | 接 STT、TTS，串起语音主环路 | 第一版 wake-to-voice 闭环 |
| `Day 9` | `src/state.rs` `src/tools/music.rs` `src/ollama.rs` | 加当前歌曲状态和音乐短解释 | 能接“这首歌为啥好听”这类追问 |
| `Day 10` | 全部相关文件 | 联调、修错误、测体验、整理下一阶段任务 | 可真实试用的音乐语音雏形 |

### D. 每天的最小交付检查

每天结束前至少检查这 4 件事：

1. 当天新增代码是否还符合 `Single-Step Action`
2. 模型输出是否经过了 `Validated Model Output`
3. 结果判断是不是仍以 `Tool Reality Precedence` 为准
4. 用户反馈是不是仍然短，而不是开始变成长解释

### E. 写代码的推荐顺序

如果你每天只想盯最关键的一组文件，可以按这个节奏：

1. 先盯 `main / cli / config / ollama / session`
2. 再盯 `harness / schema / state`
3. 然后盯 `tools/mod + tools/music`
4. 最后盯 `audio/hotkey + input + stt + tts`

也就是说：

先把骨架做对，再把第一条工具链做对，最后把语音入口接上。
