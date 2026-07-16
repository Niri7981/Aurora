# AuroraPulse Phase Architecture Diagrams

This directory keeps architecture images and their reproducible generation prompts together. Each diagram describes the system as it existed at the end of that phase; later capabilities must not be backported into an earlier phase image.

## Catalog

| Phase | Architecture focus | Image | Prompt | Status |
| --- | --- | --- | --- | --- |
| Phase 1 | User-owned Identity Card and auditable Context Bundle | [Phase 1 image](images/phase-1-identity-context-architecture.png) | [Phase 1 prompt](prompts/phase-1-identity-context-architecture.md) | Complete |
| Phase 2 | Provider-neutral model boundary and local/cloud context policies | [Phase 2 image](images/phase-2-model-provider-architecture.png) | [Phase 2 prompt](prompts/phase-2-model-provider-architecture.md) | Complete |
| Phase 3 | Custom Harness, unified Tool Registry, permissions and normalized results | [Phase 3 image](images/phase-3-harness-architecture.png) | [Phase 3 prompt](prompts/phase-3-harness-architecture.md) | Complete |

## Phase 1 Diagram

![AuroraPulse Phase 1 Identity and Context Architecture](images/phase-1-identity-context-architecture.png)

Phase 1 established the local source of truth:

```text
User-owned identity files + workspace context
  -> Context Loader
  -> Selective Filter
  -> Context Bundle Composer
  -> Preview & Audit
  -> identity-aware first reply
```

Identity remains editable local data. AuroraPulse reads known files, exposes the resulting bundle for inspection, and does not silently mutate memory.

## Phase 2 Diagram

![AuroraPulse Phase 2 Model Provider Architecture](images/phase-2-model-provider-architecture.png)

Phase 2 separated context ownership from model transport:

```text
Local Context Contract
  -> Provider-Aware Context Policy
  -> Prompt Composer
  -> ChatClient Interface
  -> OllamaProvider | OpenAIProvider
  -> normalized model content
```

Local and cloud providers share one application lifecycle while receiving different privacy policies. Runtime model switching does not move identity ownership into the provider.

## Phase 3 Diagram

![AuroraPulse Phase 3 Harness Architecture](images/phase-3-harness-architecture.png)

The Phase 3 diagram records the completed runtime path:

```text
CLI + Local Context
  -> App Controller
  -> Model Provider Adapter
  -> internal Planner JSON
  -> validated PlannerDecision
  -> Custom Harness
  -> Unified Tool Registry
  -> normalized ToolResult
  -> Session + user reply
```

The model proposes `chat`, `clarify`, `tool`, or `retrieve`. The Harness and Tool Registry retain execution authority. Phase 4 retrieval is shown as a dashed future executor because only its planner branch exists at the Phase 3 boundary.

## Generation Workflow

1. Open the matching file under `prompts/`.
2. Use the full `Imagine 2 Prompt` for the first generation.
3. Check every title, component name, arrow direction, phase boundary, and invariant.
4. Regenerate or edit until the text is correct and no future capability appears as complete.
5. Export a PNG without additional compression.
6. Save it under `images/` using the exact filename listed in the catalog.
7. Update the catalog status to `Complete`.

## Visual Contract

- Wide 16:10 off-white canvas.
- Hand-drawn Excalidraw-style black outlines and arrows.
- Large dashed containers for trust and ownership boundaries.
- Rounded pastel modules with a consistent color progression.
- English technical labels to reduce generated-text errors.
- Muted red only for prohibited or unsafe paths.
- Gray dashed boxes only for intentionally deferred capabilities.
- No decorative icons, people, gradients, 3D effects, or unrelated product names.

## Phase Boundaries

### Phase 1

Shows editable local identity sources, selective context loading, preview and first-turn personalization. It must not present cloud providers, native tools, retrieval, or voice as completed.

### Phase 2

Shows one provider-neutral context contract, Ollama and OpenAI-compatible adapters, provider-aware privacy policy, model catalog loading and session model switching. It must not present Harness-controlled tool execution as completed.

### Phase 3

Shows internal planner JSON, validated Rust decisions, centralized permissions, dynamic tool discovery, native dispatch, normalized results, bounded command execution and inspectable logs. Retrieval execution remains deferred to Phase 4.

## Naming Convention

```text
images/phase-N-topic-architecture.png
prompts/phase-N-topic-architecture.md
```

Keep generated images and their source prompts in the same phase directory so future architectural changes remain reviewable rather than becoming unexplained design artifacts.
