# AuroraPulse

AuroraPulse is a local-first personal assistant context centered on helping one user get useful help from any model without repeatedly reintroducing themself. It exists to give local models and API-backed models a user-owned identity context, then turn natural-language requests into dependable assistance grounded in local context and explicit tools.

## Language

**Local Daily Assistant**:
The primary identity of AuroraPulse. It is a personal assistant for frequent everyday tasks on a local machine, not a single-purpose music helper or a general autonomous agent.
_Avoid_: Music assistant, chatbot, super-agent, Siri clone

**Core Task Set**:
The first bounded set of tasks AuroraPulse is expected to do well: music control, local notes, files and local knowledge, and opening apps. Local reminders remain in scope for the product, but depend on a resident runtime before they become a true first-class daily capability.
_Avoid_: General productivity suite, web automation platform, everything assistant

**Local-First**:
AuroraPulse prefers a local model as its primary intelligence layer and stores its own user data locally by default. External services may still be used for specific capabilities, but the assistant's control loop and core user context should remain anchored on the user's machine.
_Avoid_: Cloud-first, fully offline, remote-first

**Identity Card**:
The first durable personal context object in AuroraPulse. It is a short, user-editable local file that tells a model who the user is, what they are currently focused on, how they prefer assistance, and which privacy boundaries matter. It should be injected before the first model reply when useful, whether the provider is local or API-backed.
_Avoid_: Hidden profile, inferred personality model, account-owned memory

**Account-Like Memory Experience**:
The V1 product promise that users of local models or stateless API calls can still get a first-message experience where the model knows who they are. AuroraPulse provides this by owning the identity card locally and sending only a minimal context bundle to the selected model.
_Avoid_: Platform account memory, provider-owned personalization, automatic cloud profile

**Provider-Agnostic Context**:
AuroraPulse should prepare personal and project context before choosing or calling a model provider. The same local context layer should work with Ollama, OpenAI API, Anthropic API, Gemini API, or future providers, with provider-specific privacy rules deciding what can be sent.
_Avoid_: Provider-locked memory, model-specific profile format, cloud-first personalization

**Context Bundle**:
A small, auditable package of local context assembled for one request or one external assistant. It may include identity card fields, current focus, project context, selected preferences, and source notes, but it should exclude private data that is not needed for the task or is blocked by privacy rules.
_Avoid_: Full memory dump, whole-vault prompt, invisible context injection

**Local User Data**:
The assistant-owned data that should live on the user's computer by default, such as session history, preferences, notes, reminders, and indexed local knowledge. This does not include external service state that AuroraPulse may read or act on through an integration.
_Avoid_: Cloud memory, hosted profile, external account state

**Single-Step Action**:
The default interaction shape where one user request leads to one clear, verifiable action or answer. AuroraPulse should prefer completing one bounded task well over planning or executing long autonomous chains.
_Avoid_: Multi-step autonomy, open-ended agent runs, long self-directed workflows

**Clarification**:
When a request is ambiguous, AuroraPulse should ask a short narrowing question before acting. Clarification is preferred over guessing because the assistant is expected to be dependable in bounded daily tasks.
_Avoid_: Best-guess execution, silent assumption, over-eager action

**Local Knowledge**:
The subset of the user's files, notes, and other machine-resident content that AuroraPulse can search, summarize, and answer questions from. It is part of the assistant's local user data, not a general web knowledge source.
_Avoid_: Web search corpus, cloud knowledge base, generic internet answers

**Read-First File Handling**:
For file-related requests, AuroraPulse should default to reading, summarizing, locating, or answering from local content before making changes. File modification, movement, or deletion is a separate class of action and should not be inferred from a read-oriented request.
_Avoid_: Implicit file edits, auto-organizing files, write-on-read behavior

**Action Risk Confirmation**:
AuroraPulse should classify side-effecting actions by risk and require confirmation for higher-risk operations before executing them. Low-risk actions may run directly, but destructive or broad changes should never happen silently.
_Avoid_: Flat trust model, silent destructive action, ungraded tool execution

**Local Note**:
Notes are machine-resident user files that AuroraPulse can create, read, summarize, and help organize locally. A note is not defined by a specific editor such as Typora; the editor is separate from the note itself.
_Avoid_: Cloud note, editor-owned note, hosted document

**Local Reminder**:
A reminder is a local prompt tied to a specific time or moment to surface one thing the user should remember. It only becomes a fully useful capability when AuroraPulse has an active or resident way to surface that prompt, and it is not a full task-management system.
_Avoid_: Full task manager, project planner, cloud reminder object

**Local Launch**:
A lightweight local action that opens an app, folder, or file on the user's machine without complex GUI automation. It is a bounded convenience capability, not a general desktop-control system.
_Avoid_: GUI automation, robotic desktop control, workflow macro platform

**Resident Runtime**:
A lightweight background-running form of AuroraPulse that stays available on the user's machine and can proactively surface time-based or event-based assistance. This capability is a prerequisite for reminders to feel real rather than merely stored, and it is meant for availability rather than autonomy.
_Avoid_: CLI-only session, one-shot script, full autonomous daemon

**Global Shortcut Entry**:
The first preferred wake mechanism for the resident runtime, where the user invokes AuroraPulse explicitly through a system-wide shortcut before speaking or typing a request. It is favored over always-listening wake words in the early stages.
_Avoid_: Always-listening wake word, passive microphone standby, ambient hotword loop

**Voice-First Interaction**:
For the resident runtime, voice is the primary interaction mode after explicit wake-up, with text kept as a fallback channel. Continuous always-listening conversation may be a later evolution, but it is not the first-stage interaction model.
_Avoid_: Text-only assistant, ambient always-on voice as the first release

**Short Voice Feedback**:
AuroraPulse should respond in very short spoken feedback by default, especially for successful single-step actions and simple failures. Voice interaction should optimize for quick acknowledgement rather than extended explanation.
_Avoid_: Verbose spoken explanations, narrated reasoning, long assistant monologues

**Guided Failure Feedback**:
When an action fails, AuroraPulse should still keep the response short and, when useful, add one small next-step prompt to help the user recover. Failure handling should unblock the next turn without turning into a technical explanation dump.
_Avoid_: Raw error dumps, long diagnostics by default, dead-end failure responses

**Model Role**:
The local model is AuroraPulse's primary understanding layer: it interprets requests, extracts action parameters, and helps produce short user-facing replies. It is not the whole system and should not own long execution chains or system control by itself.
_Avoid_: Autonomous operator, full control plane, self-directed workflow engine

**Validated Model Output**:
Model decisions must be checked against explicit action and parameter rules before AuroraPulse acts on them. When the output is invalid, the assistant should fall back to clarification or a bounded failure response rather than executing a guess.
_Avoid_: Blind model execution, schema-free actioning, trust-the-LLM control flow

**Unified Tool Layer**:
AuroraPulse executes capabilities through one consistent tool abstraction, so music, notes, file reading, local knowledge access, and local launch all follow the same registration and invocation shape. It should feel protocol-like and extensible, but remain owned by AuroraPulse rather than delegated to an external runtime model.
_Avoid_: Ad-hoc feature branches, tool-specific control loops, direct feature wiring

**Custom Harness**:
The request lifecycle, context loading, validation, clarification, tool dispatch, and reply shaping are owned by AuroraPulse itself. External protocols may be integrated later, but the core control loop and context engineering remain first-party.
_Avoid_: Framework-owned runtime, protocol-as-product-core, black-box agent loop

**Selective Context Loading**:
AuroraPulse should load only the small amount of local context that is genuinely relevant to the current request. Context engineering should favor precise, demand-driven grounding over dumping large histories or document sets into the model.
_Avoid_: Full-context stuffing, whole-vault prompting, oversized prompt assembly

**Working Context Sources**:
In the near term, AuroraPulse should ground requests mainly from recent session context, a small set of user preferences, and retrieved local content that matches the current need. It should not depend on an expansive vague long-term memory to feel useful.
_Avoid_: Total recall memory, giant personal archive prompts, fuzzy lifelong memory

**Stable Preference**:
A small, explicit, and durable user setting that meaningfully shapes AuroraPulse behavior, such as response style, default note location, or preferred local launch targets. Preferences should be easy to inspect and edit, not inferred as broad personality claims.
_Avoid_: Personality model, implicit psych profile, sprawling inferred memory

**Initial Local Knowledge Sources**:
The first local knowledge corpus should stay focused on text-forward personal and project content, especially Markdown notes, project documents, and plain text files. Broader source types can come later after retrieval quality is dependable.
_Avoid_: Everything-indexed vault, mixed-media ingestion first, broad source sprawl

**Source-Aware Answering**:
When AuroraPulse answers from local knowledge, it should preserve awareness of where the answer came from and surface that provenance when useful. Local knowledge responses should feel grounded rather than guessed.
_Avoid_: Source-free synthesis, unsupported summary, contextless knowledge claims

**Lightweight Provenance Delivery**:
AuroraPulse should keep provenance lightweight in voice-first interaction, giving the answer first and surfacing source detail only when useful or explicitly requested. Trust should come from retrievability without turning every response into a citation recital.
_Avoid_: Mandatory spoken citations, source overload, provenance-first voice responses

**Assistant Personality**:
AuroraPulse should have a noticeable personal style so it feels alive and distinctive, but that personality must stay subordinate to reliability, brevity, and task completion. Personality shapes tone, not whether the assistant executes safely or answers clearly.
_Avoid_: Pure utility robot, companion-first character, personality-over-correctness

**Expressive Delivery**:
AuroraPulse expresses personality through tone, phrasing, and feedback style rather than through unauthorized initiative or off-task behavior. The assistant can feel lively without changing the execution contract.
_Avoid_: Personality through disobedience, improvisational execution, chatty derailment

**Default Assistant Style**:
AuroraPulse should begin with one strong default personality and only a small amount of user-adjustable style variation, such as calmer, playier, or more concise delivery. Early customization should tune the same core assistant rather than split it into many separate personas.
_Avoid_: Persona catalog, many character modes, fully dynamic identity switching

**Wake-to-Voice Success Loop**:
In the voice stage, an AuroraPulse interaction is successful when the user can invoke it with a global shortcut, speak a request, receive a short spoken reply, and have the requested task complete correctly. This remains a core later loop, but it no longer displaces the V1 identity-card loop.
_Avoid_: Text-only success criteria, partial demo success, wake without completion

**Observed Task Completion**:
AuroraPulse should treat a task as complete only when the external result is actually observed through the relevant tool or system response. A model's claim that something is done is not enough on its own.
_Avoid_: Claimed completion, model-declared success, pretend execution

**Tool Reality Precedence**:
When the model's expectation conflicts with the observed tool result, AuroraPulse must trust the tool-observed reality and respond from there. Recovery should come through retry, clarification, or short failure guidance rather than pretending the model was right.
_Avoid_: Model-over-reality, hallucinated success, reasoning-over-observation

**Short Spoken Reply Loop**:
In the voice stage, AuroraPulse should use brief TTS responses as part of the wake-to-voice interaction loop rather than trying to sustain long spoken conversations. Spoken output is there to confirm, clarify, or lightly guide the next turn.
_Avoid_: Long-form voice chat, extended spoken dialogue, narrated assistant sessions

**Music-First Voice Loop**:
The first fully dependable wake-to-voice workflow can be music control, because it is easy to verify, feels immediately useful, and exercises the assistant loop well. Music is the first proving ground for voice interaction, not the V1 product identity.
_Avoid_: Tool-order sprawl, notes-first voice demo, music-as-product-identity

**Contextual Short Explanation**:
AuroraPulse may give short, context-aware explanations tied to the current task or active object, such as the music that is currently playing. These explanations should stay brief and grounded in the immediate context rather than turning into broad open-ended commentary.
_Avoid_: Long topical digressions, generic info-dumps, freeform companion chatter

**Current Task State**:
AuroraPulse may keep a small amount of short-lived structured state about the current or very recent task, such as the currently playing track or the last executed tool action. This state exists to support natural follow-up requests, not to model the whole world.
_Avoid_: World model, giant session graph, indefinite live state

**Short-Horizon State**:
In the first version, AuroraPulse should keep only a narrow, short-lived state horizon that helps with immediate follow-up turns. This state should remain simple and bounded rather than evolving into a complex persistent state machine.
_Avoid_: Long-horizon state tracking, deep conversational memory graph, persistent orchestration state
