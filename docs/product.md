# AuroraPulse Product Thinking

## 1. Product Positioning

AuroraPulse is a local-first personal assistant built around a local LLM.

Its goal is not to become a general super-agent on day one. Its goal is to become a dependable daily assistant that can understand natural language, use a small set of tools well, and help with frequent personal workflows on a local machine.

The product direction is:

- local-first
- voice-ready
- tool-heavy
- retrieval-assisted
- execution-oriented

AuroraPulse should feel less like a chatbot and more like a practical assistant that can help the user do small useful things every day.

## 2. Core User

The initial target user is the builder themself:

- uses a personal Mac as the main environment
- already runs local models
- values privacy and local control
- can tolerate an early-stage interface if the assistant is genuinely useful
- wants an assistant for work, notes, files, reminders, and lightweight system actions

This matters because the first versions do not need broad consumer polish. They need strong usefulness for one power user.

## 3. Product Goals

AuroraPulse should eventually help with these categories of tasks:

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

### V1: Daily assistant skeleton

Purpose:
- prove a reusable assistant runtime exists

Scope:
- CLI
- tool registry
- basic memory
- local file retrieval
- reminders or notes
- one or two system actions

### V2: Voice-first assistant

Purpose:
- make the product naturally usable every day

Scope:
- STT
- TTS
- wake flow without full daemon complexity

### V3: Personal context assistant

Purpose:
- become meaningfully useful because it knows the user's local world

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

## 11. Near-Term Product Decisions

For the next stage, the product should commit to:

- one primary identity: local daily assistant
- one primary platform: macOS
- one primary runtime style: local model plus explicit tools
- one primary early interaction mode: CLI first, voice second
- one primary product edge: private access to local context

## 12. Product Summary

AuroraPulse is not trying to outsmart frontier models. It is trying to become a small, reliable, local assistant that understands requests, uses tools safely, and gets more useful as it gains access to the user's real local context.
