# AuroraPulse

**English** | [简体中文](README.zh-CN.md)

AuroraPulse is a local personal memory layer for AI agents.

> Tell Aurora once. Every AI you authorize can know you.

Aurora keeps the user's identity, current focus, preferences, and privacy policy on the local machine. Authorized MCP clients request only the context needed for the current task. Aurora is not a chat agent, model provider, or desktop automation runtime.

## Current Product

- User-owned local context files
- A read-only stdio MCP server
- Task-scoped, source-aware `ContextPack` responses
- Configurable privacy markers and minimum-necessary disclosure
- Bounded personal-context search
- Fail-closed local access auditing
- Management commands for initialization, preview, and audit inspection

The MCP server exposes:

- `get_identity`
- `get_current_focus`
- `search_personal_context`

## Data Flow

```text
Local files
    -> Context Loader
    -> Disclosure Policy
    -> task-scoped ContextPack
    -> authorized MCP client
    -> local audit log
```

Aurora reloads local context for every request. It never sends `privacy-rules.json`, removes marked lines before disclosure, limits result size, and replaces absolute paths with stable `aurora://` or `workspace://` source URIs.

## Local Data

By default Aurora reads:

```text
~/.aurorapulse/identity-card.md
~/.aurorapulse/current-focus.md
~/.aurorapulse/preferences.json
~/.aurorapulse/privacy-rules.json
```

Initialize missing files:

```bash
cargo run -- init
```

Preview exactly what may leave Aurora after privacy filtering:

```bash
cargo run -- preview
```

Start the local PostgreSQL database:

```bash
docker compose up -d postgres
```

Copy `.env.example` to `.env` and adjust any local paths before starting Aurora.

## Run The MCP Server

```bash
cargo build --release
./target/release/aurora serve .
```

Register the local release binary with Codex:

```bash
codex mcp add aurora \
  --env AURORA_MCP_CLIENT=codex \
  -- "$(pwd)/target/release/aurora" serve "$(pwd)"
```

Inspect recent access:

```bash
./target/release/aurora audit .
```

Audit events are stored in `~/.aurorapulse/audit/mcp.jsonl`.

## Configuration

Optional path overrides:

```env
AURORA_HOME=/path/to/aurora-data
AURORA_IDENTITY_CARD=/path/to/identity-card.md
AURORA_CURRENT_FOCUS=/path/to/current-focus.md
AURORA_PREFERENCES=/path/to/preferences.json
AURORA_PRIVACY_RULES=/path/to/privacy-rules.json
AURORA_MCP_CLIENT=client-name
```

## Repository Shape

```text
backend/
  Cargo.toml
  src/
    api/
      cli.rs
      mcp.rs
    application/
      context_gateway.rs
    domain/
      context.rs
      context_pack.rs
      conversation.rs
      disclosure.rs
      import_batch.rs
    infrastructure/
      audit_log.rs
      local_context.rs
      database/
        mod.rs
    main.rs
    lib.rs
    config.rs
  migrations/
    202607270001_create_import_batches.sql
    202607270002_create_conversations.sql
  tests/
    context_loading.rs
    context_gateway.rs
    conversations.rs
    import_batches.rs
examples/
docs/
  product.md
  technical-architecture.md
  roadmap-cn.md
  adr/
```

## Next

The next product slice imports a bounded chat source, lets the user authorize a conversation and time range, and lets Codex analyze it through MCP with message-level citations. Durable memory follows as a second loop.

See [Product](docs/product.md), [Technical Architecture](docs/technical-architecture.md), and [Roadmap](docs/roadmap-cn.md).
