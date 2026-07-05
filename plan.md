# AuroraPulse Plan

## Current Status

Phase 1 is functionally complete enough to move forward.

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
- The provider path is still Ollama-only at runtime.
- Cloud provider privacy filtering exists only as context rendering groundwork, not as a real API call path yet.

## Tomorrow: Phase 2 Cloud Provider Slice

Goal:

Let AuroraPulse run the same identity-aware first-message flow through a cloud API provider, starting with OpenAI/GPT, while keeping local context ownership unchanged.

Core principle:

The cloud model should receive a filtered context bundle. It should not own, store, or mutate the user's identity memory.

## Phase 2 Scope

Build:
- `ModelProvider` abstraction.
- `OllamaProvider` implementation using the current Ollama path.
- `OpenAIProvider` implementation for one GPT model.
- CLI provider selection, likely `--provider ollama|openai`.
- Provider-aware context rendering:
  - local providers may receive full local context.
  - cloud providers receive a shorter filtered bundle.
- Clear error messages when `OPENAI_API_KEY` is missing.
- Tests for provider selection and cloud redaction behavior.

Do not build yet:
- Anthropic/Gemini providers.
- Streaming.
- Tool execution through cloud providers.
- Long-term memory writes.
- Account sync.
- MCP adapter.

## Proposed Implementation Order

1. Add config fields:
   - `provider`
   - `openai_api_key`
   - `openai_model`
   - maybe `OPENAI_BASE_URL` for compatibility later.

2. Introduce provider interface:
   - move current `ChatClient` shape toward a provider-neutral trait.
   - keep messages/history support.
   - make provider receive already-prepared prompt text.

3. Wrap existing Ollama path:
   - preserve current behavior.
   - avoid changing planner/harness semantics.

4. Add OpenAI provider:
   - use HTTPS API call.
   - send system planner prompt plus context-aware user message.
   - parse response content using existing `PlannerDecision::parse`.

5. Add CLI/provider selection:
   - default remains Ollama.
   - allow env var first if command flags are too much for one day.

6. Validate behavior:
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

## Product Check

If the OpenAI path works, the key product claim becomes demonstrable:

> A stateless API model can feel account-aware because AuroraPulse locally supplies the user's editable identity context before the first reply.
