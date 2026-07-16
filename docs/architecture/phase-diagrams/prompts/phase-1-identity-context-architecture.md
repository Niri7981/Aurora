# AuroraPulse Phase 1 Architecture Image Prompt

这份提示词用于生成 Phase 1 的 Identity Card 与 Context Bundle 架构图。正文使用英文，以减少图像模型生成技术文字时的错字。

## Diagram Content

### Title

`AuroraPulse Phase 1 — Identity & Context Architecture`

### Subtitle

`User-owned files → selective loading → auditable context bundle → identity-aware first reply`

### Section 1

Dashed container label:

`User-Owned Identity Sources (editable local truth)`

Five horizontal boxes:

1. `identity-card.md`
   - `who the user is`
   - `roles + stable background`
2. `current-focus.md`
   - `what matters now`
   - `active goals + priorities`
3. `preferences.json`
   - `interaction preferences`
   - `stable defaults`
4. `privacy-rules.json`
   - `sharing boundaries`
   - `local-only + private markers`
5. `Workspace Context`
   - `CONTEXT.md`
   - `AGENTS.md / CLAUDE.md`

Draw downward arrows from all five source boxes into the context pipeline.

### Section 2

Dashed container label:

`AuroraPulse Context Layer (local, inspectable, read-only by default)`

Six boxes connected left to right:

1. `Context Init`
   - `/context init`
   - `create missing templates`
   - `never overwrite user files`
2. `Context Loader`
   - `read known local paths`
   - `graceful missing-file fallback`
3. `Project Detector`
   - `inspect current workspace`
   - `load explicit context files only`
4. `Selective Filter`
   - `minimum necessary context`
   - `apply privacy markers`
5. `Context Bundle Composer`
   - `identity + focus + preferences`
   - `project + runtime metadata`
6. `Preview & Audit`
   - `/context preview`
   - `show sources + exclusions`

Use bold black arrows between all boxes. Add a thin feedback arrow from `Preview & Audit` back to the user-owned files, labeled:

`user edits the source of truth`

Add the note above the pipeline:

`AuroraPulse reads context; it does not silently invent or own identity.`

### Section 3

Dashed container label:

`First-Turn Personalization (bounded consumer)`

Four boxes connected left to right:

1. `Natural Language Request`
   - `first user message`
2. `Prompt Composition`
   - `Context Bundle + request`
   - `clear source boundaries`
3. `Local Model Path`
   - `Ollama provider`
   - `identity-aware request`
4. `Personalized Reply`
   - `knows user + current focus`
   - `short, grounded answer`

Draw one downward arrow from `Context Bundle Composer` to `Prompt Composition`.

Add a light gray dashed box to the right, not connected as an active Phase 1 component:

`Future Provider Adapters`

- `same context contract`
- `provider must not own memory`

### Section 4

Dashed container label:

`External / Explicit Boundaries (never implicit)`

Four horizontal boxes:

1. `~/.aurorapulse/`
   - `plain user-editable files`
2. `Current Workspace`
   - `explicit project context files`
3. `Ollama Local Runtime`
   - `model process, not memory owner`
4. `No Automatic Memory Writes`
   - `no full-disk scan`
   - `no hidden profile mutation`

Use light gray for the first three boxes and muted red for `No Automatic Memory Writes`.

### Footer

Add this exact footer:

`Invariants: INV-001 identity remains user-owned · INV-002 known files only · INV-003 missing files degrade gracefully · INV-004 context is previewable · INV-005 no automatic memory mutation`

## Imagine 2 Prompt

Create a wide, high-resolution hand-drawn software architecture diagram on a clean off-white background, matching a polished Excalidraw engineering diagram. Use slightly imperfect black marker outlines, rounded rectangular pastel boxes, bold hand-drawn arrows, large dashed boundary containers, and a casual handwritten technical font. Keep the composition precise, evenly spaced, and easy to scan. Canvas ratio approximately 16:10, around 2000 × 1250 pixels. Do not crop the title, sections, arrows, or footer. Do not add icons, people, decorative illustrations, gradients, 3D effects, or corporate diagram styling.

At the top write the exact title:

`AuroraPulse Phase 1 — Identity & Context Architecture`

Below it write the exact subtitle:

`User-owned files → selective loading → auditable context bundle → identity-aware first reply`

Build four large stacked dashed-outline sections.

SECTION 1 label: `User-Owned Identity Sources (editable local truth)`

Place five pastel blue boxes in one row:

- `identity-card.md` / `who the user is` / `roles + stable background`
- `current-focus.md` / `what matters now` / `active goals + priorities`
- `preferences.json` / `interaction preferences` / `stable defaults`
- `privacy-rules.json` / `sharing boundaries` / `local-only + private markers`
- `Workspace Context` / `CONTEXT.md` / `AGENTS.md / CLAUDE.md`

Draw downward arrows from all source boxes into Section 2.

SECTION 2 label: `AuroraPulse Context Layer (local, inspectable, read-only by default)`

Place six boxes in a left-to-right pipeline:

- light cyan: `Context Init` / `/context init` / `create missing templates` / `never overwrite user files`
- light green: `Context Loader` / `read known local paths` / `graceful missing-file fallback`
- pale yellow: `Project Detector` / `inspect current workspace` / `load explicit context files only`
- pale orange: `Selective Filter` / `minimum necessary context` / `apply privacy markers`
- lavender: `Context Bundle Composer` / `identity + focus + preferences` / `project + runtime metadata`
- pale purple: `Preview & Audit` / `/context preview` / `show sources + exclusions`

Connect the six boxes with bold black arrows. Above them write: `AuroraPulse reads context; it does not silently invent or own identity.` Add a thin return arrow from `Preview & Audit` to the source files labeled `user edits the source of truth`.

SECTION 3 label: `First-Turn Personalization (bounded consumer)`

Place four connected boxes:

- pale cyan: `Natural Language Request` / `first user message`
- pale green: `Prompt Composition` / `Context Bundle + request` / `clear source boundaries`
- pale yellow: `Local Model Path` / `Ollama provider` / `identity-aware request`
- pale purple: `Personalized Reply` / `knows user + current focus` / `short, grounded answer`

Draw a downward arrow from `Context Bundle Composer` to `Prompt Composition`. To the right add a light gray dashed future box: `Future Provider Adapters` / `same context contract` / `provider must not own memory`.

SECTION 4 label: `External / Explicit Boundaries (never implicit)`

Place four boxes:

- light gray: `~/.aurorapulse/` / `plain user-editable files`
- light gray: `Current Workspace` / `explicit project context files`
- light gray: `Ollama Local Runtime` / `model process, not memory owner`
- muted red: `No Automatic Memory Writes` / `no full-disk scan` / `no hidden profile mutation`

At the bottom add this exact footer:

`Invariants: INV-001 identity remains user-owned · INV-002 known files only · INV-003 missing files degrade gracefully · INV-004 context is previewable · INV-005 no automatic memory mutation`

Preserve every label and filename exactly. The title must say `AuroraPulse`, `Phase 1`, and `Identity & Context Architecture`. Do not mention Phase 2, Phase 3, ProofForge, Riptide, tools, MCP, autonomous agents, voice, or vector databases. Do not invent components.

## Short Prompt Variant

Generate a polished Excalidraw-style architecture diagram titled `AuroraPulse Phase 1 — Identity & Context Architecture`. Use an off-white background, hand-drawn black outlines, dashed section containers, rounded pastel boxes, handwritten technical lettering, and clear arrows. Show `identity-card.md`, `current-focus.md`, `preferences.json`, `privacy-rules.json`, and workspace context flowing through `Context Init → Context Loader → Project Detector → Selective Filter → Context Bundle Composer → Preview & Audit`, then into `Natural Language Request → Prompt Composition → Local Model Path → Personalized Reply`. Emphasize user-owned editable files, selective loading, previewability, graceful missing-file behavior, no full-disk scan, and no automatic memory writes. Keep all spelling exact and do not add Phase 2 or Phase 3 components.
