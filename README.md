# AuroraPulse

AuroraPulse is a local-first personal context and assistant runtime.

V1 focuses on one concrete promise:

> Any selected model should know who it is helping before the first useful reply, without making that personal memory belong to the model provider.

The current implementation is a Rust CLI with Ollama and OpenAI-compatible providers. Local context, planner decisions, harness policy, and native tool execution remain separate so model providers never own memory or direct system control.

## V1 Shape

- Editable identity card
- Current focus file
- Stable preferences
- Privacy rules
- Current project context from `CONTEXT.md`, `AGENTS.md`, or `CLAUDE.md`
- Context bundle preview
- Ollama and OpenAI-compatible model providers
- Structured planner decisions and a custom harness
- Unified native tool registry with centralized risk policy
- Inspectable normalized tool results

## Local Identity Files

By default AuroraPulse looks in:

```text
~/.aurorapulse/identity-card.md
~/.aurorapulse/current-focus.md
~/.aurorapulse/preferences.json
~/.aurorapulse/privacy-rules.json
```

You can start from the examples in this repo:

```bash
mkdir -p ~/.aurorapulse
cp examples/identity-card.md ~/.aurorapulse/identity-card.md
cp examples/current-focus.md ~/.aurorapulse/current-focus.md
cp examples/preferences.json ~/.aurorapulse/preferences.json
cp examples/privacy-rules.json ~/.aurorapulse/privacy-rules.json
```

These files are plain user-owned data. Open them directly and edit them whenever your identity, focus, preferences, or privacy boundaries change.

## Run

```bash
cargo run -- .
```

Inside the CLI:

```text
/context init
/context preview
/model
/tools
/tools log
```

`/context init` creates the local context files if they do not exist.

`/context preview` shows the context bundle AuroraPulse will inject before calling the model.

`/tools` shows the exact tool catalog injected into the planner prompt. `/tools log` shows the
most recent normalized tool results and execution timing for the current process.

Any normal request is sent to the model with local identity context prepended:

```text
我下一步应该做什么？
```

## Environment

```env
AURORA_PROVIDER=ollama
OLLAMA_MODEL=gemma4:e4b
OLLAMA_URL=http://127.0.0.1:11434
```

For an OpenAI-compatible cloud provider:

```env
AURORA_PROVIDER=openai
OPENAI_API_KEY=...
OPENAI_BASE_URL=https://api.openai.com
OPENAI_MODEL=gpt-4o-mini
```

`OPENAI_BASE_URL` may point at a compatible gateway. AuroraPulse appends `/v1/chat/completions` unless the base URL already ends in `/v1`.

Optional path overrides:

```env
AURORA_HOME=/path/to/local/context
AURORA_IDENTITY_CARD=/path/to/identity-card.md
AURORA_CURRENT_FOCUS=/path/to/current-focus.md
AURORA_PREFERENCES=/path/to/preferences.json
AURORA_PRIVACY_RULES=/path/to/privacy-rules.json
```

## Current Rust Structure

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

## Architecture Diagrams

Phase-by-phase architecture images and reproducible Imagine 2 prompts are maintained in [docs/architecture/phase-diagrams](docs/architecture/phase-diagrams/README.md).

Phase 1 through Phase 3 are complete and documented there. **Phase 4: Local Knowledge is scheduled to begin on 2026-07-18**, starting with the existing `retrieve` decision branch, authorized Markdown/text sources, and source-aware answers.

## Not V1

- Full-disk scanning
- Automatic long-term memory
- Voice loop
- MCP as the core runtime
- Cloud-provider-owned memory
