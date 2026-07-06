# AuroraPulse Technical Architecture

## 1. Technical Goal

AuroraPulse should be a local-first personal context and assistant runtime that works well with a modest local model while leaving room for API-backed providers.

The architecture should assume:

- local LLM reasoning is limited
- API-backed LLM calls are often stateless and do not know who the user is
- tool execution can be made reliable
- retrieval can compensate for missing knowledge
- system design matters more than raw model intelligence

The main design idea is:

local context for identity, model for understanding, harness for control, tools for action, retrieval for evidence.

## 2. Runtime Constraints

Current constraints and assumptions:

- primary machine: personal Mac
- memory budget: about 24 GB RAM
- local model available: Gemma e4b through Ollama
- future providers: OpenAI API, Anthropic API, Gemini API, or other compatible APIs
- early interaction mode: CLI
- later interaction mode: voice

Architecturally, this means:

- avoid heavyweight multi-agent stacks
- avoid long autonomous chains
- prefer deterministic tool routing and short loops
- keep components inspectable and debuggable
- prepare context locally before calling any model provider
- preview or audit any personal context sent to cloud APIs

## 3. Top-Level System

AuroraPulse should eventually follow this flow:

```text
Input (CLI / Voice)
  -> Preprocess
  -> Local Context Layer
  -> Harness
  -> Model Provider
  -> Planner / Router
  -> Tool Call or Retrieval
  -> Observation
  -> Response Composer
  -> Output (Text / TTS)
```

More concretely:

```text
User request
  -> input interface
  -> identity card and current focus loader
  -> project context loader
  -> context bundle builder
  -> selected model provider
  -> planner
  -> one of:
     - direct reply
     - tool call
     - retrieval
     - clarification
  -> execution layer
  -> result normalization
  -> local LLM final response
  -> UI / voice output
```

## 4. Core Design Principles

### 4.1 Keep the harness custom

The runtime loop should be owned by the project. Do not hide the core control path behind a large agent framework early on.

### 4.2 Strong structure around model outputs

The planner should emit constrained structured data, not free-form paragraphs.

### 4.3 Small tools with explicit contracts

Each tool should do one thing clearly and return structured results.

### 4.4 Retrieval is a subsystem, not a magic layer

RAG should be intentionally scoped to local content that is worth searching.

### 4.5 Unsafe actions require policy

System actions, shell actions, and external side effects need confirmation and guardrails.

### 4.6 Identity context must be editable and auditable

The V1 identity card should be a user-editable local file. AuroraPulse should be able to preview the context bundle that will be sent to a local model or cloud API before relying on it for important flows.

### 4.7 Providers are replaceable

Ollama is the first provider, but the local context layer should not be tied to Ollama. OpenAI, Anthropic, Gemini, or future providers should receive the same kind of minimal context bundle through provider-specific adapters.

## 5. Main Components

### 5.1 Interfaces

Purpose:
- capture user input
- render final responses
- surface clarification prompts

Expected modules:

- `src/cli.rs`
- `src/audio/` later
- resident runtime module later

### 5.2 Harness

Purpose:
- orchestrate the full request lifecycle
- maintain session state
- route between reply, tool, and retrieval paths
- recover from common failures

Suggested future file:

- `src/harness.rs`

Responsibilities:

- load session context
- call planner
- validate planner output
- dispatch tool or retrieval request
- capture observation
- request final user-facing reply
- enforce retries, timeouts, and confirmations

### 5.2a Local Context Layer

Purpose:
- own the user's identity context before any model call
- assemble the smallest useful context bundle for the current request
- keep local memory separate from provider-owned account memory

Current and future files:

- `src/context/mod.rs`
- future split candidates: `identity.rs`, `project.rs`, `bundle.rs`, `privacy.rs`

V1 responsibilities:

- load an editable identity card
- load current focus
- load stable preferences
- load privacy rules
- detect current project context from `CONTEXT.md`, `AGENTS.md`, or `CLAUDE.md`
- build a previewable context bundle for a selected provider
- exclude blocked personal fields when the selected provider is cloud-backed

V1 local data examples:

```text
~/.aurorapulse/identity-card.md
~/.aurorapulse/current-focus.md
~/.aurorapulse/preferences.json
~/.aurorapulse/privacy-rules.json
```

### 5.3 Model Provider

Purpose:
- call the selected model without tying AuroraPulse memory to that provider

Current files:

- `src/model/mod.rs`
- `src/model/ollama.rs`
- `src/model/openai.rs`

Expected providers:

- `OllamaProvider` for the first local model path
- `OpenAIProvider` later
- `AnthropicProvider` later
- `GeminiProvider` later

The provider interface should receive a prepared prompt or message list from the harness. It should not own identity, memory, retrieval, or permission policy.

### 5.4 Planner

Purpose:
- translate natural language into structured intent

Current basis:
- selected model provider, with Ollama first

Suggested behavior:
- emit JSON with a mode and arguments

Example shape:

```json
{
  "mode": "chat|tool|retrieve|clarify",
  "tool_name": "spotify.play_track",
  "arguments": {
    "query": "Jay Chou Sunny Day"
  },
  "retrieve_query": "",
  "clarify_question": "",
  "reply": ""
}
```

The planner should not directly execute anything.

### 5.5 Tool Registry

Purpose:
- provide a unified abstraction over executable capabilities

Suggested future file:

- `src/tools/mod.rs`

Responsibilities:

- register tools
- expose tool descriptions
- validate arguments
- call tools
- normalize results

Early tool categories:

- music
- reminders
- notes
- files
- calendar
- apps
- shell

### 5.6 Retrieval Layer

Purpose:
- provide relevant local context when the model lacks knowledge

Suggested future file:

- `src/retriever.rs`

Responsibilities:

- define retrievable corpora
- chunk and index documents
- run semantic and keyword retrieval
- return compact evidence sets

Typical sources:

- personal notes
- project docs
- Markdown files
- text files
- PDFs

### 5.7 Memory Layer

Purpose:
- preserve useful state without becoming a vague long-memory system

Suggested future file:

- `src/memory.rs`

Responsibilities:

- short-term session history
- user preferences
- small durable facts
- recent actions

Recommended split:

- short-term memory: in-session context
- durable identity memory: editable identity card, preferences, current focus, and known facts
- retrieval memory: external indexed content

### 5.8 Response Composer

Purpose:
- convert execution results into concise user-facing language

This may use the local model, but the output should remain short and grounded in the observation returned by tools or retrieval.

## 6. Harness Loop

The default request loop should look like this:

1. Receive request
2. Load editable identity card and current focus
3. Load current project context when available
4. Apply provider-specific privacy rules
5. Build a minimal context bundle
6. Ask the selected model provider for a structured planner decision
7. Validate the decision
8. If `clarify`, ask a short question
9. If `tool`, execute the tool
10. If `retrieve`, run retrieval and optionally ask the model for synthesis
11. Ask the model for a final response if needed
12. Persist session and approved memory updates
13. Return output

This is intentionally short. The system should prefer one clear loop over long autonomous reasoning chains.

## 7. Tool Strategy

Tool support should evolve in two stages.

### Stage 1: Native tools

Implement a local `ToolRegistry` first.

Why:

- easier to debug
- simpler contracts
- lower moving-part count
- better fit for a small local assistant

### Stage 2: MCP adapter

Add MCP once the internal tool model is stable.

Why:

- standardize external capabilities
- integrate browser, filesystem, calendar, GitHub, and other ecosystems
- keep core harness separate from vendor-specific integrations

The harness should treat MCP as one backend for tools, not as the entire architecture.

## 8. Retrieval Strategy

The retrieval subsystem should start narrow.

### First retrieval targets

- project repositories
- personal notes
- selected reference folders

### Retrieval pipeline

1. ingest documents
2. chunk with source metadata
3. embed and index
4. retrieve top candidates
5. optionally rerank
6. inject short evidence into model prompt

### Recommendations

- begin with one local vector store
- keep documents and metadata inspectable
- store source path and snippet boundaries
- prefer small focused context blocks

Candidate technologies:

- embeddings: local embedding model
- vector store: Chroma or FAISS
- metadata store: SQLite

## 9. Voice Pipeline

Voice should remain a separate pipeline around the same harness.

Expected flow:

```text
Microphone
  -> STT
  -> text request
  -> harness
  -> text response
  -> TTS
  -> speaker output
```

Recommended first choices:

- STT: `faster-whisper`
- TTS: macOS `say`

Do not tightly couple voice logic with tool or planning logic.

## 10. Data and Storage

Storage should be intentionally split by purpose.

### Configuration

- `.env`
- model settings
- feature flags
- identity card path
- current focus path
- provider settings
- privacy rule settings

### Durable app data

- plain local files for:
  - identity card
  - current focus
  - stable preferences
  - privacy rules
- SQLite database for:
  - reminders metadata
  - memory summaries
  - session pointers

### Tokens and credentials

- local cache file or SQLite
- Spotify tokens
- future integration credentials

### Retrieval artifacts

- vector index
- source metadata
- ingestion state

## 11. Safety and Trust

The assistant will only feel good if it is predictable.

Needed safety rules:

- confirmations for destructive actions
- allowlists for shell-like capabilities
- structured tool validation
- timeouts and retries
- visible error messages
- logging of tool calls and results

Suggested risk levels:

- low risk: read-only retrieval, listing, summarization
- medium risk: opening apps, creating reminders, playback control
- high risk: shell execution, file modification, external posting

High-risk actions should require stronger confirmation or remain disabled in early versions.

## 12. Observability

The system should be easy to inspect while being built.

Need:

- request logs
- context bundle previews
- planner decision logs
- tool invocation logs
- retrieval hit summaries
- execution timing
- failure reasons

This matters especially because small local models can fail quietly in confusing ways.

## 13. Suggested Project Shape

A likely next project shape is:

```text
src/
  main.rs
  app.rs
  cli.rs
  config.rs
  context/
    mod.rs
  model/
    mod.rs
    ollama.rs
    openai.rs
  harness.rs
  planner.rs
  session.rs
  tools/
  audio/
docs/
  product.md
  technical-architecture.md
  roadmap-cn.md
```

This keeps the core runtime distinct from tool implementations and interfaces.

## 14. Delivery Strategy

Recommended implementation order:

1. define editable identity card and current focus files
2. implement local context loading
3. implement context bundle preview
4. introduce model provider abstraction with Ollama first
5. route `ask` through context bundle plus provider
6. add provider-specific privacy filtering
7. add lightweight local retrieval for approved folders
8. add tools, voice, and adapters after identity-aware answering works

This order keeps complexity increasing only after the previous layer is understandable.

## 15. Technical Summary

AuroraPulse should be built as a small, custom, local personal context and assistant runtime.

Its architecture should rely on:

- editable local identity context before the first model reply
- pluggable model providers, with Ollama first
- a custom harness for orchestration
- structured tools for execution
- retrieval for local knowledge
- narrow memory for continuity
- voice as an interface layer, not the system core

That combination is the right fit for a 24 GB local machine, a modest but useful local model, and future API-backed models that should borrow context without owning the user's memory.
