# AuroraPulse Phase 2 Architecture Image Prompt

这份提示词用于生成 Phase 2 的 Model Provider 架构图。正文使用英文，突出 provider-neutral runtime 与本地/云端隐私边界。

## Diagram Content

### Title

`AuroraPulse Phase 2 — Model Provider Architecture`

### Subtitle

`One local context contract → provider-aware filtering → isolated model adapters`

### Section 1

Dashed container label:

`Local Context Ownership (trusted source boundary)`

Three horizontal boxes:

1. `Identity Context`
   - `Identity Card + Current Focus`
   - `user-owned local files`
2. `Project Context`
   - `CONTEXT.md / AGENTS.md`
   - `current workspace only`
3. `Runtime Configuration`
   - `provider + model + endpoint`
   - `.env / process environment`

Draw downward arrows into the provider-neutral pipeline.

### Section 2

Dashed container label:

`AuroraPulse Provider-Neutral Runtime (memory stays outside the model)`

Six boxes connected left to right:

1. `App Controller`
   - `request + session history`
   - `runtime model selection`
2. `Context Policy`
   - `local policy / cloud policy`
   - `minimum necessary disclosure`
3. `Prompt Composer`
   - `filtered context + user request`
   - `provider-neutral messages`
4. `ChatClient Interface`
   - `provider_name`
   - `chat + list_models`
5. `Configured Provider Router`
   - `ollama | openai`
   - `no identity storage`
6. `Response Normalizer`
   - `provider envelope → content`
   - `clear API errors`

Add this note above the pipeline:

`The provider receives a prepared prompt; it never owns AuroraPulse memory.`

### Section 3

Dashed container label:

`Provider Adapters (same contract, different transport)`

Four horizontal boxes:

1. `OllamaProvider`
   - `local full context policy`
   - `/api/chat`
   - `local model catalog`
2. `OpenAIProvider`
   - `cloud-filtered context policy`
   - `/v1/chat/completions`
   - `/v1/models`
3. `/model Selector`
   - `read provider catalog`
   - `switch model for current session`
4. `Shared Session Contract`
   - `system + history + user`
   - `same App lifecycle`

Draw arrows from `Configured Provider Router` down to both provider boxes. Draw arrows from both providers to `/model Selector` and `Response Normalizer`. Connect `Shared Session Contract` upward to `ChatClient Interface`.

### Section 4

Dashed container label:

`External / Explicit Boundaries (provider-owned transport only)`

Four boxes:

1. `Ollama Local Runtime`
   - `127.0.0.1 model service`
2. `OpenAI-Compatible API`
   - `HTTPS JSON request / response`
3. `Credentials & Endpoints`
   - `OPENAI_API_KEY`
   - `OPENAI_BASE_URL + model IDs`
4. `No Provider-Owned Memory`
   - `no account memory dependency`
   - `no context mutation`

Use light gray for the first three boxes and muted red for `No Provider-Owned Memory`.

### Footer

Add this exact footer:

`Invariants: INV-001 one provider-neutral context contract · INV-002 local gets full policy, cloud gets filtered policy · INV-003 credentials stay outside prompts · INV-004 provider errors stay visible · INV-005 switching models does not move memory ownership`

## Imagine 2 Prompt

Create a wide, high-resolution hand-drawn software architecture diagram on a clean off-white background, matching a polished Excalidraw engineering diagram. Use slightly imperfect black marker outlines, rounded rectangular pastel boxes, bold hand-drawn arrows, large dashed boundary containers, and a casual handwritten technical font. The layout should be technically precise, spacious, and easy to scan. Canvas ratio approximately 16:10, around 2000 × 1250 pixels. Keep all labels horizontal and legible. Do not crop any content. Do not add icons, people, decorative illustrations, gradients, shadows, logos, 3D effects, or corporate flowchart styling.

At the top write the exact title:

`AuroraPulse Phase 2 — Model Provider Architecture`

Directly beneath it write:

`One local context contract → provider-aware filtering → isolated model adapters`

Build four large stacked dashed-outline sections.

SECTION 1 label: `Local Context Ownership (trusted source boundary)`

Place three pastel blue boxes:

- `Identity Context` / `Identity Card + Current Focus` / `user-owned local files`
- `Project Context` / `CONTEXT.md / AGENTS.md` / `current workspace only`
- `Runtime Configuration` / `provider + model + endpoint` / `.env / process environment`

Draw downward arrows into Section 2.

SECTION 2 label: `AuroraPulse Provider-Neutral Runtime (memory stays outside the model)`

Place six connected boxes in a left-to-right pipeline:

- light cyan: `App Controller` / `request + session history` / `runtime model selection`
- light green: `Context Policy` / `local policy / cloud policy` / `minimum necessary disclosure`
- pale yellow: `Prompt Composer` / `filtered context + user request` / `provider-neutral messages`
- pale orange: `ChatClient Interface` / `provider_name` / `chat + list_models`
- lavender: `Configured Provider Router` / `ollama | openai` / `no identity storage`
- pale purple: `Response Normalizer` / `provider envelope → content` / `clear API errors`

Above the pipeline write: `The provider receives a prepared prompt; it never owns AuroraPulse memory.`

SECTION 3 label: `Provider Adapters (same contract, different transport)`

Place four boxes in one row:

- pale peach: `OllamaProvider` / `local full context policy` / `/api/chat` / `local model catalog`
- pale mint: `OpenAIProvider` / `cloud-filtered context policy` / `/v1/chat/completions` / `/v1/models`
- pale yellow: `/model Selector` / `read provider catalog` / `switch model for current session`
- pale purple: `Shared Session Contract` / `system + history + user` / `same App lifecycle`

Draw arrows from `Configured Provider Router` to both provider adapters. Draw arrows from both providers to `/model Selector` and `Response Normalizer`. Connect `Shared Session Contract` upward to `ChatClient Interface`.

SECTION 4 label: `External / Explicit Boundaries (provider-owned transport only)`

Place four horizontal boxes:

- light gray: `Ollama Local Runtime` / `127.0.0.1 model service`
- light gray: `OpenAI-Compatible API` / `HTTPS JSON request / response`
- light gray: `Credentials & Endpoints` / `OPENAI_API_KEY` / `OPENAI_BASE_URL + model IDs`
- muted red: `No Provider-Owned Memory` / `no account memory dependency` / `no context mutation`

Connect `OllamaProvider` down to `Ollama Local Runtime`. Connect `OpenAIProvider` down to `OpenAI-Compatible API` and `Credentials & Endpoints`.

At the bottom add this exact footer:

`Invariants: INV-001 one provider-neutral context contract · INV-002 local gets full policy, cloud gets filtered policy · INV-003 credentials stay outside prompts · INV-004 provider errors stay visible · INV-005 switching models does not move memory ownership`

Preserve all spelling exactly. The title must say `AuroraPulse`, `Phase 2`, and `Model Provider Architecture`. Do not mention Phase 3, ProofForge, Riptide, tools, Harness execution, MCP, voice, retrieval, or autonomous agents. Do not invent additional providers.

## Short Prompt Variant

Generate a polished Excalidraw-style architecture diagram titled `AuroraPulse Phase 2 — Model Provider Architecture`. Use an off-white background, hand-drawn black outlines, dashed containers, rounded pastel boxes, handwritten technical lettering, and clear arrows. Show local Identity and Project Context plus Runtime Configuration flowing through `App Controller → Context Policy → Prompt Composer → ChatClient Interface → Configured Provider Router → Response Normalizer`. Below, show `OllamaProvider`, `OpenAIProvider`, `/model Selector`, and `Shared Session Contract`; then explicit external boundaries for Ollama, the OpenAI-compatible HTTPS API, credentials, and the rule `No Provider-Owned Memory`. Emphasize local-full versus cloud-filtered context, one provider-neutral contract, visible errors, and session-only model switching. Keep spelling exact and do not add Phase 3 tool execution.
