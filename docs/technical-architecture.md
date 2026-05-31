# AuroraPulse Technical Architecture

## 1. Technical Goal

AuroraPulse should be a local-first assistant runtime that works well with a modest local model.

The architecture should assume:

- local LLM reasoning is limited
- tool execution can be made reliable
- retrieval can compensate for missing knowledge
- system design matters more than raw model intelligence

The main design idea is:

model for understanding, harness for control, tools for action, retrieval for context.

## 2. Runtime Constraints

Current constraints and assumptions:

- primary machine: personal Mac
- memory budget: about 24 GB RAM
- local model available: Gemma e4b through Ollama
- early interaction mode: CLI
- later interaction mode: voice

Architecturally, this means:

- avoid heavyweight multi-agent stacks
- avoid long autonomous chains
- prefer deterministic tool routing and short loops
- keep components inspectable and debuggable

## 3. Top-Level System

AuroraPulse should eventually follow this flow:

```text
Input (CLI / Voice)
  -> Preprocess
  -> Harness
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
  -> session context loader
  -> local LLM planner
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

## 5. Main Components

### 5.1 Interfaces

Purpose:
- capture user input
- render final responses
- surface clarification prompts

Expected modules:

- `aurorapulse/interfaces/cli.py`
- `aurorapulse/interfaces/voice.py`
- `aurorapulse/interfaces/daemon.py`

### 5.2 Harness

Purpose:
- orchestrate the full request lifecycle
- maintain session state
- route between reply, tool, and retrieval paths
- recover from common failures

Suggested future file:

- `aurorapulse/core/harness.py`

Responsibilities:

- load session context
- call planner
- validate planner output
- dispatch tool or retrieval request
- capture observation
- request final user-facing reply
- enforce retries, timeouts, and confirmations

### 5.3 Planner

Purpose:
- translate natural language into structured intent

Current basis:
- local Ollama model wrapper

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

### 5.4 Tool Registry

Purpose:
- provide a unified abstraction over executable capabilities

Suggested future file:

- `aurorapulse/core/tools.py`

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

### 5.5 Retrieval Layer

Purpose:
- provide relevant local context when the model lacks knowledge

Suggested future file:

- `aurorapulse/core/retriever.py`

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

### 5.6 Memory Layer

Purpose:
- preserve useful state without becoming a vague long-memory system

Suggested future file:

- `aurorapulse/core/memory.py`

Responsibilities:

- short-term session history
- user preferences
- small durable facts
- recent actions

Recommended split:

- short-term memory: in-session context
- durable profile memory: preferences and known facts
- retrieval memory: external indexed content

### 5.7 Response Composer

Purpose:
- convert execution results into concise user-facing language

This may use the local model, but the output should remain short and grounded in the observation returned by tools or retrieval.

## 6. Harness Loop

The default request loop should look like this:

1. Receive request
2. Load recent context and user profile
3. Ask planner for a structured decision
4. Validate the decision
5. If `clarify`, ask a short question
6. If `tool`, execute the tool
7. If `retrieve`, run retrieval and optionally ask the model for synthesis
8. Ask the model for a final response if needed
9. Persist session and memory updates
10. Return output

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

### Durable app data

- SQLite database for:
  - preferences
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
- planner decision logs
- tool invocation logs
- retrieval hit summaries
- execution timing
- failure reasons

This matters especially because small local models can fail quietly in confusing ways.

## 13. Suggested Project Shape

A likely next project shape is:

```text
aurorapulse/
  core/
    harness.py
    planner.py
    tools.py
    retriever.py
    memory.py
    session.py
    schemas.py
    settings.py
  integrations/
    llm/
    music/
    reminders/
    calendar/
    files/
    stt/
    tts/
    mcp/
  interfaces/
    cli.py
    voice.py
    daemon.py
docs/
  product.md
  technical-architecture.md
```

This keeps the core runtime distinct from tool implementations and interfaces.

## 14. Delivery Strategy

Recommended implementation order:

1. stabilize planner output schema
2. introduce harness loop
3. introduce tool registry
4. add one or two non-music tools
5. add retrieval for local docs
6. add session and memory
7. add voice pipeline
8. add MCP bridge

This order keeps complexity increasing only after the previous layer is understandable.

## 15. Technical Summary

AuroraPulse should be built as a small, custom, local assistant runtime.

Its architecture should rely on:

- a local model for intent understanding
- a custom harness for orchestration
- structured tools for execution
- retrieval for local knowledge
- narrow memory for continuity
- voice as an interface layer, not the system core

That combination is the right fit for a 24 GB local machine and a modest but useful local model.
