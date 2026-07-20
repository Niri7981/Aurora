# AuroraPulse

[English](README.md) | **简体中文**

AuroraPulse 是面向 AI Agent 的本地个人记忆层。

> 把自己告诉 Aurora 一次，所有你授权的 AI 都能认识你。

Aurora 在本机维护用户的身份、当前关注、偏好和隐私策略。获得授权的 MCP 客户端只能按当前任务请求必要上下文。Aurora 不再是聊天 Agent、模型提供方或桌面自动化运行时。

## 当前产品

- 用户拥有的本地上下文文件
- 只读 stdio MCP Server
- 面向当前任务、带来源的 `ContextPack`
- 可配置隐私标记与最小必要披露
- 有界个人上下文搜索
- 审计失败即拒绝披露的本地访问日志
- 初始化、预览和审计管理命令

MCP Server 提供三个工具：

- `get_identity`
- `get_current_focus`
- `search_personal_context`

## 数据流

```text
本地文件
    -> Context Loader
    -> Disclosure Policy
    -> 当前任务的 ContextPack
    -> 获得授权的 MCP 客户端
    -> 本地审计日志
```

Aurora 每次调用都会重新读取本地上下文。它不会返回 `privacy-rules.json`，会在披露前删除带隐私标记的行、限制结果大小，并使用稳定的 `aurora://` 或 `workspace://` URI 代替绝对路径。

## 本地数据

Aurora 默认读取：

```text
~/.aurorapulse/identity-card.md
~/.aurorapulse/current-focus.md
~/.aurorapulse/preferences.json
~/.aurorapulse/privacy-rules.json
```

初始化缺失文件：

```bash
cargo run -- init
```

预览经过隐私过滤后可能离开 Aurora 的内容：

```bash
cargo run -- preview
```

## 运行 MCP Server

```bash
cargo build --release
./target/release/aurora serve .
```

将本地 release 二进制注册到 Codex：

```bash
codex mcp add aurora \
  --env AURORA_MCP_CLIENT=codex \
  -- "$(pwd)/target/release/aurora" serve "$(pwd)"
```

检查最近的访问：

```bash
./target/release/aurora audit .
```

审计事件保存在 `~/.aurorapulse/audit/mcp.jsonl`。

## 配置

可选路径覆盖：

```env
AURORA_HOME=/path/to/aurora-data
AURORA_IDENTITY_CARD=/path/to/identity-card.md
AURORA_CURRENT_FOCUS=/path/to/current-focus.md
AURORA_PREFERENCES=/path/to/preferences.json
AURORA_PRIVACY_RULES=/path/to/privacy-rules.json
AURORA_MCP_CLIENT=client-name
```

## 仓库结构

```text
src/
  main.rs
  cli.rs
  config.rs
  context/
    mod.rs
  mcp.rs
tests/
  context_loading.rs
  mcp_identity.rs
examples/
docs/
  product.md
  technical-architecture.md
  roadmap-cn.md
  adr/
```

## 下一步

下一阶段是建立带来源、可由用户纠正的长期记忆。之后通过 Source Adapter 导入聊天记录、笔记、邮件和文档，先生成可审阅的记忆候选，而不是默认把原始内容直接披露给 AI。

参见[产品定义](docs/product.md)、[技术架构](docs/technical-architecture.md)和[路线图](docs/roadmap-cn.md)。
