# AuroraPulse

[English](README.md) | **简体中文**

AuroraPulse 是面向 AI Agent 的本地个人身份与记忆层。

它的产品北极星是：

> 把自己告诉 Aurora 一次，所有你授权的 AI 都能认识你。

Aurora 在用户本机维护一份由用户拥有的个人事实源，并且只向获得授权的 AI 提供当前任务真正需要的上下文。身份、当前关注、偏好、记忆和披露策略属于 Aurora 与用户，而不属于任何模型提供方。

当前实现是这个产品方向的 Rust 基础：一个具备可编辑本地上下文、Ollama 与 OpenAI 兼容提供方、结构化 Planner、自定义 Harness 和权限受控原生工具的 CLI。Phase 4 已加入第一个面向外部 Agent 的产品边界：只读本地 MCP Server，让获得授权的 Agent 请求有范围限制的 Context Pack。

## 当前基础

- 可编辑的身份卡
- 当前关注事项文件
- 稳定偏好设置
- 隐私规则
- 从 `CONTEXT.md`、`AGENTS.md` 或 `CLAUDE.md` 读取当前项目上下文
- 上下文包预览
- Ollama 与 OpenAI 兼容模型提供方
- 结构化 Planner 决策与自定义 Harness
- 带集中式风险策略的统一原生工具注册表
- 可检查的标准化工具结果
- 提供结构化 Context Pack 的只读本地 MCP 身份服务
- 动态脱敏标记与本地 MCP 访问审计日志

## MCP 身份服务

Phase 4 已将现有本地上下文层变成可供其他 Agent 使用的 MCP 身份服务。这一切片刻意保持很小：

- 本地 stdio MCP Server
- 只读的身份、当前关注和个人上下文工具
- 带来源信息、针对当前任务生成的 Context Pack
- 最小必要披露与明确的敏感信息边界
- 与 Codex 跑通端到端集成，证明全新任务无需重复介绍也能认识用户

这条端到端链路已于 2026-07-18 在全新 Codex 任务中验证。Codex 自动发现并调用 `get_identity` 与 `get_current_focus`，随后仅根据 `aurora://identity-card.md` 和 `aurora://current-focus.md` 回答，没有读取工作区文件。

长期记忆写入、泛文档摄取和语音能力继续延后，先让只读跨 Agent 身份链路接受更多真实使用。

## 本地身份文件

AuroraPulse 默认读取：

```text
~/.aurorapulse/identity-card.md
~/.aurorapulse/current-focus.md
~/.aurorapulse/preferences.json
~/.aurorapulse/privacy-rules.json
```

可以从仓库中的示例开始：

```bash
mkdir -p ~/.aurorapulse
cp examples/identity-card.md ~/.aurorapulse/identity-card.md
cp examples/current-focus.md ~/.aurorapulse/current-focus.md
cp examples/preferences.json ~/.aurorapulse/preferences.json
cp examples/privacy-rules.json ~/.aurorapulse/privacy-rules.json
```

这些文件是完全由用户拥有的普通数据。身份、当前目标、偏好或隐私边界发生变化时，可以直接打开并编辑。

## 运行

```bash
cargo run -- .
```

CLI 内可使用：

```text
/context init
/context preview
/model
/mcp log
/tools
/tools log
```

`/context init` 会创建尚不存在的本地上下文文件。

`/context preview` 会显示 AuroraPulse 在调用模型前注入的上下文包。

`/tools` 显示注入 Planner 提示词的准确工具目录；`/tools log` 显示当前进程中最近的标准化工具结果与执行耗时。

`/mcp log` 显示外部 Agent 最近的上下文访问，包括客户端、工具、返回的来源 URI 和脱敏行数。

普通请求会在本地身份上下文之后发送给模型：

```text
我下一步应该做什么？
```

以本地 MCP Server 运行 Aurora：

```bash
cargo build --release
./target/release/aurora serve .
```

将 release 二进制注册到 Codex：

```bash
codex mcp add aurora \
  --env AURORA_MCP_CLIENT=codex \
  -- "$(pwd)/target/release/aurora" serve "$(pwd)"
```

stdio Server 提供三个只读工具：`get_identity`、`get_current_focus` 和 `search_personal_context`。

## 环境配置

```env
AURORA_PROVIDER=ollama
OLLAMA_MODEL=gemma4:e4b
OLLAMA_URL=http://127.0.0.1:11434
```

使用 OpenAI 兼容的云端提供方：

```env
AURORA_PROVIDER=openai
OPENAI_API_KEY=...
OPENAI_BASE_URL=https://api.openai.com
OPENAI_MODEL=gpt-4o-mini
```

`OPENAI_BASE_URL` 也可以指向兼容网关。除非基础 URL 已经以 `/v1` 结尾，否则 AuroraPulse 会追加 `/v1/chat/completions`。

可选路径覆盖：

```env
AURORA_HOME=/path/to/local/context
AURORA_IDENTITY_CARD=/path/to/identity-card.md
AURORA_CURRENT_FOCUS=/path/to/current-focus.md
AURORA_PREFERENCES=/path/to/preferences.json
AURORA_PRIVACY_RULES=/path/to/privacy-rules.json
```

## 当前 Rust 结构

```text
src/
  main.rs
  app.rs
  cli.rs
  config.rs
  context/
    mod.rs
  harness.rs
  model/
    mod.rs
    ollama.rs
    openai.rs
  planner.rs
  session.rs
  startup_animation.rs
  theme.rs
  tools/
    mod.rs
tests/
  app_runtime.rs
  context_loading.rs
  harness_runtime.rs
  planner_schema.rs
  startup_cli.rs
```

## 架构图

各阶段架构图与可复现的 Imagine 2 提示词统一维护在 [docs/architecture/phase-diagrams](docs/architecture/phase-diagrams/README.md)。

### Phase 1：身份与上下文

![AuroraPulse Phase 1 身份与上下文架构](docs/architecture/phase-diagrams/images/phase-1-identity-context-architecture.png)

Phase 1 建立由用户拥有的身份文件、选择性本地上下文加载、可审计的 Context Bundle，以及理解用户身份的首次回复。

### Phase 2：模型提供方

![AuroraPulse Phase 2 模型提供方架构](docs/architecture/phase-diagrams/images/phase-2-model-provider-architecture.png)

Phase 2 加入提供方中立的模型边界、按提供方过滤上下文的策略、Ollama 与 OpenAI 兼容适配器，以及运行时模型选择。

### Phase 3：Harness 运行时

![AuroraPulse Phase 3 Harness 架构](docs/architecture/phase-diagrams/images/phase-3-harness-architecture.png)

Phase 3 完成结构化 Planner、自定义 Harness、统一 Tool Registry、集中式权限策略、标准化工具结果、有界执行与可检查日志。

Phase 1 到 Phase 4 已全部完成。**Phase 4：MCP Identity Server 已于 2026-07-18 通过 Codex 真实验证。**下一阶段是持久、可由用户纠正的个人记忆。

## V1 不包含

- 全盘扫描
- 自动长期记忆
- 语音循环
- 由云端模型提供方拥有的记忆
- 向 Agent 无限制倾倒全部上下文
