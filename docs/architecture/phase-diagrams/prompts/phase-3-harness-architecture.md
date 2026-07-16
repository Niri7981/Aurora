# AuroraPulse Phase 3 Architecture Image Prompt

这份提示词用于生成一张与参考图相同信息设计语言的 Phase 3 架构图。图中正文使用英文，避免图像模型生成中文时出现错字。

## Diagram Content

### Title

`AuroraPulse Phase 3 — Harness Architecture`

### Subtitle

`Local context → model-planned JSON → harness-controlled tools → normalized reality`

### Section 1

Dashed container label:

`User & Local Context (trusted input boundary)`

Three horizontal boxes:

1. `CLI Input`
   - `natural language requests`
   - `slash commands`
   - `Yes / Always / No`
2. `Editable Local Context`
   - `Identity Card + Current Focus`
   - `Preferences + Privacy Rules`
   - `Project Context`
3. `Runtime Selection`
   - `provider + model`
   - `Ollama / OpenAI-compatible`

Draw downward arrows from all three boxes into the runtime pipeline below.

### Section 2

Dashed container label:

`AuroraPulse Runtime (custom harness, not an autonomous agent)`

Six boxes in one left-to-right pipeline:

1. `App Controller`
   - `local command routing`
   - `context composition`
   - `session history`
2. `Model Provider Adapter`
   - `system prompt + context`
   - `Ollama / OpenAI-compatible`
3. `Planner Parser`
   - `internal JSON → validated enum`
   - `chat | clarify | tool | retrieve`
4. `Custom Harness`
   - `decision routing`
   - `pending action + permissions`
   - `single-step control`
5. `Unified Tool Registry`
   - `dynamic tool catalog`
   - `name + argument validation`
   - `risk policy + dispatch`
6. `Normalized Tool Result`
   - `succeeded | failed | denied`
   - `data + error + elapsed_ms`
   - `last 32 audit entries`

Connect all six boxes with bold black right-pointing arrows.

Add a thin return arrow from `Normalized Tool Result` back toward `Custom Harness`, labeled:

`verified result → session → user reply`

Add one small note above the pipeline:

`The model proposes an action; it never executes one directly.`

### Section 3

Dashed container label:

`Native Tool Targets (system-owned execution)`

Four horizontal boxes:

1. `local_launch.open_app`
   - `macOS open -a`
   - `medium risk`
   - `confirmation required`
2. `spotify.play_artist`
   - `artist query → playback helper`
   - `low risk`
3. `spotify.play_track`
   - `track query → playback helper`
   - `low risk`
4. `retrieve`
   - `decision branch exists`
   - `executor starts in Phase 4`

The fourth box must be light gray with a dashed outline to show that it is intentionally not implemented in Phase 3.

Draw downward arrows from `Unified Tool Registry` to the three active tool boxes. Draw a dashed downward arrow from `Custom Harness` to the gray `retrieve` box.

Draw upward return arrows from the active tool boxes to `Normalized Tool Result`, labeled:

`ToolResult`

### Section 4

Dashed container label:

`External / Explicit Boundaries (never implicit)`

Four horizontal boxes:

1. `Ollama Local Runtime`
   - `local model process`
2. `OpenAI-compatible API`
   - `JSON request / response envelope`
3. `OS + Python Helpers`
   - `bounded command execution`
   - `20 second timeout`
4. `No Direct Model Control`
   - `no shell access`
   - `no tool bypass`

Use light gray for the first three boxes. Use muted red for `No Direct Model Control`.

Connect `Model Provider Adapter` down to `Ollama Local Runtime` and `OpenAI-compatible API`. Connect native tool targets down to `OS + Python Helpers`. Do not draw any arrow from a model directly to a native tool.

### Footer

Add a single handwritten footer line:

`Invariants: INV-001 model proposes, harness decides · INV-002 registry validates every action · INV-003 low=direct, medium=session approval, high=confirm every time · INV-004 tool reality overrides model claims · INV-005 bounded execution + inspectable logs`

## Imagine 2 Prompt

Create a wide, high-resolution hand-drawn software architecture diagram on a clean off-white background, matching the visual language of a polished Excalidraw engineering diagram. Use slightly imperfect black marker outlines, rounded rectangular boxes, bold hand-drawn arrows, large dashed boundary containers, and a casual handwritten technical font. The composition should feel precise, spacious, and authored by a senior systems engineer, not like a corporate flowchart.

Canvas ratio approximately 16:10, around 2000 × 1250 pixels. Keep every label horizontal and highly legible. Use consistent spacing and alignment. Do not crop any section or footer. Do not add decorative illustrations, icons, people, gradients, shadows, logos, or 3D effects.

At the top, write the exact title:

`AuroraPulse Phase 3 — Harness Architecture`

Directly beneath it, write the exact subtitle:

`Local context → model-planned JSON → harness-controlled tools → normalized reality`

Build four large stacked dashed-outline sections.

SECTION 1 label: `User & Local Context (trusted input boundary)`

Place three pastel blue boxes in one row:

- `CLI Input` / `natural language requests` / `slash commands` / `Yes / Always / No`
- `Editable Local Context` / `Identity Card + Current Focus` / `Preferences + Privacy Rules` / `Project Context`
- `Runtime Selection` / `provider + model` / `Ollama / OpenAI-compatible`

Draw downward arrows from these boxes into Section 2.

SECTION 2 label: `AuroraPulse Runtime (custom harness, not an autonomous agent)`

Place six rounded boxes in a clean left-to-right pipeline, connected by bold black arrows. Use these exact labels and text:

- light cyan box: `App Controller` / `local command routing` / `context composition` / `session history`
- light green box: `Model Provider Adapter` / `system prompt + context` / `Ollama / OpenAI-compatible`
- pale yellow box: `Planner Parser` / `internal JSON → validated enum` / `chat | clarify | tool | retrieve`
- light orange box: `Custom Harness` / `decision routing` / `pending action + permissions` / `single-step control`
- lavender box: `Unified Tool Registry` / `dynamic tool catalog` / `name + argument validation` / `risk policy + dispatch`
- pale purple box: `Normalized Tool Result` / `succeeded | failed | denied` / `data + error + elapsed_ms` / `last 32 audit entries`

Above this pipeline add the exact note: `The model proposes an action; it never executes one directly.`

Add a thin return arrow from `Normalized Tool Result` back toward `Custom Harness`, labeled `verified result → session → user reply`.

SECTION 3 label: `Native Tool Targets (system-owned execution)`

Place four boxes in one row:

- pale peach: `local_launch.open_app` / `macOS open -a` / `medium risk` / `confirmation required`
- pale mint: `spotify.play_artist` / `artist query → playback helper` / `low risk`
- pale pink: `spotify.play_track` / `track query → playback helper` / `low risk`
- light gray dashed box: `retrieve` / `decision branch exists` / `executor starts in Phase 4`

Draw downward arrows from `Unified Tool Registry` to the three active native tool boxes. Draw a dashed arrow from `Custom Harness` to the gray `retrieve` box. Draw upward return arrows from active tools toward `Normalized Tool Result`, labeled `ToolResult`.

SECTION 4 label: `External / Explicit Boundaries (never implicit)`

Place four boxes in one row:

- light gray: `Ollama Local Runtime` / `local model process`
- light gray: `OpenAI-compatible API` / `JSON request / response envelope`
- light gray: `OS + Python Helpers` / `bounded command execution` / `20 second timeout`
- muted red: `No Direct Model Control` / `no shell access` / `no tool bypass`

Connect `Model Provider Adapter` downward to both model runtime boxes. Connect the active native tools downward to `OS + Python Helpers`. Never draw a direct arrow from any model runtime to any native tool.

At the bottom, add this exact single-line footer in small handwritten text:

`Invariants: INV-001 model proposes, harness decides · INV-002 registry validates every action · INV-003 low=direct, medium=session approval, high=confirm every time · INV-004 tool reality overrides model claims · INV-005 bounded execution + inspectable logs`

Visual priorities: readable architecture first, then hand-drawn charm. Preserve all spelling exactly. The word `AuroraPulse` must be spelled correctly. Do not write ProofForge, Riptide, MCP, Phase 4 Complete, autonomous execution, or any unlisted product name. Do not invent extra components.

## Short Prompt Variant

Generate a polished Excalidraw-style architecture diagram titled `AuroraPulse Phase 3 — Harness Architecture`, on an off-white background with hand-drawn black outlines, dashed section containers, rounded pastel boxes, handwritten technical lettering, and bold directional arrows. Show four stacked sections: `User & Local Context`, `AuroraPulse Runtime`, `Native Tool Targets`, and `External / Explicit Boundaries`. The main pipeline must be `App Controller → Model Provider Adapter → Planner Parser → Custom Harness → Unified Tool Registry → Normalized Tool Result`. Show internal planner JSON becoming `chat | clarify | tool | retrieve`; centralized tool validation and risk policy; `succeeded | failed | denied` results; macOS app launch and Spotify tools below; Phase 4 retrieve as a gray dashed future box; Ollama/OpenAI and OS helpers as explicit external boundaries. Emphasize `The model proposes an action; it never executes one directly.` Add the footer invariants: model proposes/harness decides, registry validates, low-medium-high confirmation policy, tool reality wins, bounded execution and inspectable logs. Keep all labels crisp and correctly spelled, with no extra products or components.
