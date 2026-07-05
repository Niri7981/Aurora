# AuroraPulse

AuroraPulse is a local-first personal context and assistant runtime.

V1 focuses on one concrete promise:

> Any selected model should know who it is helping before the first useful reply, without making that personal memory belong to the model provider.

The first implementation path is Rust CLI + Ollama, with the local context layer kept separate so future GPT, Claude, Gemini, or other API providers can receive a filtered context bundle later.

## V1 Shape

- Editable identity card
- Current focus file
- Stable preferences
- Privacy rules
- Current project context from `CONTEXT.md`, `AGENTS.md`, or `CLAUDE.md`
- Context bundle preview
- Ollama as the first provider

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
```

`/context init` creates the local context files if they do not exist.

`/context preview` shows the context bundle AuroraPulse will inject before calling the model.

Any normal request is sent to the model with local identity context prepended:

```text
我下一步应该做什么？
```

## Environment

```env
OLLAMA_MODEL=gemma4:e4b
OLLAMA_URL=http://127.0.0.1:11434
```

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
  context.rs
  harness.rs
  ollama.rs
  planner.rs
  session.rs
tests/
  app_runtime.rs
  context_loading.rs
  harness_runtime.rs
  planner_schema.rs
  startup_cli.rs
```

## Not V1

- Full-disk scanning
- Automatic long-term memory
- Voice loop
- MCP as the core runtime
- Cloud-provider-owned memory
