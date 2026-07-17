# AuroraPulse

[English](README.md) | **简体中文**

AuroraPulse 是一个本地优先的个人上下文与助手运行时。

V1 聚焦于一个明确目标：

> 无论选择哪个模型，它都应该在第一次有效回复前知道自己正在帮助谁，同时个人记忆不归模型提供方所有。

当前实现是一个 Rust CLI，支持 Ollama 和 OpenAI 兼容提供方。本地上下文、Planner 决策、Harness 策略与原生工具执行彼此分离，因此模型提供方既不拥有记忆，也不能直接控制系统。

## V1 功能

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
/tools
/tools log
```

`/context init` 会创建尚不存在的本地上下文文件。

`/context preview` 会显示 AuroraPulse 在调用模型前注入的上下文包。

`/tools` 显示注入 Planner 提示词的准确工具目录；`/tools log` 显示当前进程中最近的标准化工具结果与执行耗时。

普通请求会在本地身份上下文之后发送给模型：

```text
我下一步应该做什么？
```

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

Phase 1 到 Phase 3 已全部完成。**Phase 4：Local Knowledge 计划于 2026-07-18 正式开始**，将从现有 `retrieve` 决策分支、经过授权的 Markdown/文本来源和带来源依据的回答开始。

## V1 不包含

- 全盘扫描
- 自动长期记忆
- 语音循环
- 将 MCP 作为核心运行时
- 由云端模型提供方拥有的记忆
