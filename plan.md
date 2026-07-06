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
- Tool execution through cloud providers.
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

## Product Check

If the OpenAI path works, the key product claim becomes demonstrable:

> A stateless API model can feel account-aware because AuroraPulse locally supplies the user's editable identity context before the first reply.
