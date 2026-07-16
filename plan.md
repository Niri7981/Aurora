# AuroraPulse Plan

## Current Status

Phases 1 through 3 are complete. The project is ready to begin the Phase 4 local-knowledge slice.

Done:
- Editable local identity files are defined and initialized with `/context init`.
- AuroraPulse loads `identity-card.md`, `current-focus.md`, `preferences.json`, and `privacy-rules.json` from `~/.aurorapulse/`.
- AuroraPulse loads project context from `CONTEXT.md`, `AGENTS.md`, or `CLAUDE.md` in the active workspace.
- `/context preview` shows the context bundle before model calls.
- Normal questions are sent to the model with identity context prepended.
- Planner parsing now tolerates fenced JSON, extra model chatter, and plain text replies.
- Tests cover context loading, context initialization, cloud redaction markers, prompt injection, and parser fallback behavior.

Phase 1 caveat:
- The identity card is currently user-edited text, not generated through onboarding questions.
- Phase 1 originally used the Ollama-only runtime path; Phase 2 now adds the OpenAI-compatible provider path.
- Cloud provider privacy filtering now applies to preview and model context rendering, but the policy is still marker-based and should be hardened.

## Phase 2 Cloud Provider Slice

Goal:

Let AuroraPulse run the same identity-aware first-message flow through a cloud API provider, starting with OpenAI/GPT, while keeping local context ownership unchanged.

Core principle:

The cloud model should receive a filtered context bundle. It should not own, store, or mutate the user's identity memory.

Current progress:
- Env-based provider selection is implemented through `AURORA_PROVIDER=ollama|openai`.
- `OllamaProvider` preserves the existing local path.
- `OpenAIProvider` calls an OpenAI-compatible `/v1/chat/completions` endpoint.
- Local `.env` has been verified against a compatible gateway.
- `OPENAI_MODEL=gpt-5.4-mini` worked with the configured gateway.
- GPT-compatible first-answer behavior was validated with `我是谁？我最近在做什么？`.
- Cloud context preview uses the cloud policy and redacts `local-only:` / `private:` lines.
- Missing `OPENAI_API_KEY` is covered by a clear error and test.

## Phase 2 Scope

Built:
- `ModelProvider` abstraction.
- `OllamaProvider` implementation using the current Ollama path.
- `OpenAIProvider` implementation for one GPT-compatible model.
- Env-based provider selection through `AURORA_PROVIDER=ollama|openai`.
- Provider-aware context rendering:
  - local providers may receive full local context.
  - cloud providers receive a shorter filtered bundle.
- Clear error messages when `OPENAI_API_KEY` is missing.
- Tests for provider selection and cloud redaction behavior.

Do not build yet:
- Anthropic/Gemini providers.
- Streaming.
- Long-term memory writes.
- Account sync.
- MCP adapter.

## Remaining Implementation Order

1. Decide whether env-only provider selection is enough for V2, or add CLI flags:
   - `--provider ollama`
   - `--provider openai`

2. Improve cloud prompt boundaries:
   - shrink project context for cloud calls
   - show a concise cloud bundle by default
   - keep full preview available for audit

3. Replace the temporary `curl` implementation when the provider layer stabilizes:
   - evaluate a small blocking Rust HTTP client
   - keep tests provider-neutral

4. Add one more cloud provider only after OpenAI-compatible behavior is stable:
   - Anthropic or Gemini, not both at once.

5. Keep validating behavior:
   - `/context preview` for local provider.
   - `/context preview openai` or equivalent for cloud-filtered preview.
   - Ask GPT: `我是谁？我最近在做什么？`
   - Compare against bare GPT call behavior.

## Acceptance Criteria

Phase 2 is done when:
- `aurora` can answer through Ollama as before.
- `aurora` can answer through OpenAI API with the same identity card flow.
- GPT's first answer reflects the local identity card and current focus.
- Cloud provider prompts exclude `local-only:` and `private:` lines.
- Missing API key produces a clear user-facing error.
- `cargo test` passes.

Current result:
- All criteria above are met for the OpenAI-compatible gateway path.
- Remaining work is hardening, UX, and deciding whether to keep `curl` or move to a Rust HTTP client.

## Phase 3 Custom Harness And Unified Tool Layer

Status: complete.

Built:
- `App` composes local context, calls the selected provider, parses the internal planner JSON, and hands a validated `PlannerDecision` to the harness.
- `PlannerDecision` supports bounded `chat`, `clarify`, `tool`, and `retrieve` branches.
- `ToolRegistry` owns tool discovery, required-argument validation, risk classification, dispatch, and result normalization.
- The planner tool catalog is generated from `ToolRegistry::specs()` for every model call; tool names and argument schemas are no longer duplicated in a provider prompt.
- Risk policy is centralized: low-risk tools execute directly, medium-risk tools support one-time or session approval, and high-risk tools require confirmation every time.
- Tool execution returns a normalized `ToolResult` with `succeeded`, `failed`, or `denied` status, structured data, and an optional error.
- The harness records the latest 32 tool results with execution timing. `/tools` and `/tools log` expose the catalog and audit trail.
- External command-backed tools have a bounded 20-second execution window and restore control on timeout.
- Tests cover dynamic prompt injection, action validation, unknown tools, clarification, confirmation, session approval, high-risk re-confirmation, normalized failures, denials, and successful results.

Acceptance result:
- Adding a native tool only requires registering a `ToolDefinition`; the CLI loop, provider implementations, and planner prompt do not need tool-specific branches.
- The model proposes actions through internal JSON but cannot execute tools directly.
- Harness and tool reality remain authoritative over model claims.

## Next: Phase 4 Local Knowledge

The next slice is the `retrieve` branch: authorized Markdown/text discovery, source metadata, bounded retrieval, and short source-aware synthesis. PDF ingestion, web clipping, and a broad vector database remain out of scope for the first slice.

## Product Check

If the OpenAI path works, the key product claim becomes demonstrable:

> A stateless API model can feel account-aware because AuroraPulse locally supplies the user's editable identity context before the first reply.
