# AuroraPulse Product Thinking

## 1. Product Positioning

AuroraPulse is a local-first personal context and assistant runtime. Its first job is to let any model know who the user is before the first useful reply, without making that personal memory belong to a model provider.

Its goal is not to become a general super-agent on day one. Its goal is to provide an editable local identity card, assemble a minimal context bundle, and then use a selected local or API-backed model to help with frequent personal workflows.

The product direction is:

- local-first
- identity-aware
- provider-agnostic
- voice-ready
- tool-heavy
- retrieval-assisted
- execution-oriented

AuroraPulse should feel less like a blank chatbot and more like a practical assistant that already knows the user's basic identity, current focus, preferences, and local project context.

## 2. Core User

The initial target user is the builder themself:

- uses a personal Mac as the main environment
- already runs local models
- may also use GPT, Claude, Gemini, or other APIs directly
- values privacy and local control
- can tolerate an early-stage interface if the assistant is genuinely useful
- wants an assistant for work, notes, files, reminders, and lightweight system actions
- does not want to repeat the same personal background to every model or agent

This matters because the first versions do not need broad consumer polish. They need strong usefulness for one power user.

## 3. Product Goals

AuroraPulse should eventually help with these categories of tasks:

- identify: provide models with a short editable profile of who the user is
- contextualize: explain the user's current focus, active projects, preferences, and privacy boundaries
- capture: take notes, save reminders, record ideas
- retrieve: find files, search notes, summarize local content
- organize: check tasks, calendar, short plans, daily context
- execute: open apps, play music, run safe local actions
- answer: respond using local knowledge and personal context

The product should optimize for:

- low friction
- high trust
- predictable behavior
- private/local execution where practical

## 4. Non-Goals

AuroraPulse should not initially try to be:

- a fully autonomous general agent
- a heavy research agent that plans long chains on its own
- a broad web automation system
- a polished mass-market Siri replacement
- a memory system that stores everything forever
- a cloud-owned user profile that replaces local ownership

These are possible later, but they are not the right first target for a small local model.

## 5. Product Principles

### 5.1 Model should do less, system should do more

The local model is good enough for intent understanding, parameter extraction, short planning, and response generation. It should not be forced to carry the whole product.

### 5.2 Useful beats impressive

The assistant should succeed on real daily tasks more often than it produces flashy demos.

### 5.3 Constrained tools beat vague autonomy

Tool boundaries should be clear. The assistant should prefer narrow reliable actions over free-form behavior.

### 5.4 Clarify instead of guessing

When a request is ambiguous, the assistant should ask a short question instead of taking a risky action.

### 5.5 Local context is the product advantage

The strongest differentiator is not raw intelligence. It is access to local files, notes, preferences, routines, and system actions.

### 5.6 The user owns the identity layer

AuroraPulse should not depend on ChatGPT, Gemini, Claude, or any other provider account remembering the user. The durable identity card should live locally, be editable as plain user-owned data, and be shared with a model only as a minimal context bundle.

## 6. Core Use Cases

The first useful set of use cases should be:

1. Music and media
- play a song
- play an artist
- pause, resume, skip, volume

2. Notes and reminders
- save a note
- create a reminder
- list today's reminders

3. Files and local knowledge
- find a file
- summarize a folder or document
- answer questions from local notes or project docs

4. Lightweight daily coordination
- what is on my calendar today
- what should I focus on today
- summarize my current project context

5. Safe system actions
- open an app
- open a folder
- copy or surface useful local information

## 7. User Experience Shape

AuroraPulse will likely evolve through three interaction modes:

1. CLI
- best for early harness development
- easiest to debug
- easiest to inspect tool calls

2. Voice interface
- main daily-use interface
- speech in, response back as text and TTS

3. Background or menu bar assistant
- persistent wake-up flow
- lightweight ambient access

The user experience should remain simple:

- one request in
- one clear result out
- tool usage visible when useful
- low ceremony

## 8. Versioning Strategy

### V0: Tool demo

Purpose:
- prove local model plus one tool works end to end

Examples:
- Spotify control from natural language

### V1: Identity card and context bundle

Purpose:
- prove that a local or API-backed model can answer the first request with knowledge of who the user is

Scope:
- CLI
- editable identity card
- current focus file
- preferences and privacy rules
- current project context loading
- context bundle preview
- Ollama provider first, with provider abstraction for future OpenAI, Anthropic, and Gemini APIs

Success example:
- `aurora ask --provider ollama "我下一步应该做什么？"` answers using the user's identity, current focus, and current project context.
- The same request shape can later target `openai`, `anthropic`, or `gemini` without moving the user's memory into those providers.

### V2: Voice-first assistant

Purpose:
- make the product naturally usable every day

Scope:
- STT
- TTS
- wake flow without full daemon complexity

### V3: Personal context assistant

Purpose:
- become meaningfully useful because it knows the user's local world beyond the identity card

Scope:
- stronger RAG
- personal preferences
- project-aware retrieval
- better daily planning support

### V4: Tool ecosystem and MCP

Purpose:
- scale capabilities without turning the core into a mess

Scope:
- MCP tool bridge
- external integrations
- stricter permission model

## 9. Success Criteria

AuroraPulse is succeeding when:

- the user actually uses it multiple times a day
- the first model reply can reflect who the user is without platform account memory
- common requests work with low retries
- tool calls are predictable and easy to debug
- local context improves answers in a noticeable way
- the system remains understandable to its builder

## 10. Main Product Risks

- the model is asked to reason beyond its reliable range
- too many tools are added before the harness is stable
- voice complexity arrives before core usefulness
- RAG quality is poor, making answers feel random
- product scope expands faster than reliability improves
- identity context becomes too broad, stale, or hidden from the user
- cloud API support accidentally sends more personal context than needed

## 11. Near-Term Product Decisions

For the next stage, the product should commit to:

- one primary identity: local daily assistant
- one primary platform: macOS
- one primary runtime style: local context layer plus pluggable model providers
- one primary early interaction mode: CLI first, voice second
- one primary product edge: user-owned identity context for local and API-backed models

## 12. Product Summary

AuroraPulse is not trying to outsmart frontier models. It is trying to become the local identity and context layer that lets any model start from "I know who I am helping," while keeping that memory editable, inspectable, and owned by the user.
