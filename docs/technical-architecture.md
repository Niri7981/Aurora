# AuroraPulse Technical Architecture

## Goal

AuroraPulse is a local personal memory authority exposed through MCP. The runtime loads user-owned context, applies disclosure policy, returns bounded Context Packs, and records every access locally.

It does not embed a model, run a chat loop, or execute general agent actions.

## Current Components

```text
src/main.rs          process entrypoint
src/cli.rs           serve/init/preview/audit commands
src/config.rs        workspace and local data paths
src/context/mod.rs   source loading, validation, templates, disclosure policy
src/mcp.rs           MCP tools, retrieval, ContextPack assembly, audit
```

## Runtime Flow

```text
MCP host launches `aurora serve [workspace]`
    -> rmcp performs the stdio handshake
    -> client discovers read-only Aurora tools
    -> client calls a tool with a narrow purpose
    -> Aurora reloads local context
    -> retrieval selects a bounded set of documents
    -> DisclosurePolicy removes marked lines
    -> Aurora builds a source-aware ContextPack
    -> audit event is appended locally
    -> ContextPack is returned to the client
```

Audit is fail-closed. If Aurora cannot record the disclosure, it returns an error instead of the personal context.

## Local Sources

Global personal sources live under `AURORA_HOME`, which defaults to `~/.aurorapulse`:

```text
identity-card.md
current-focus.md
preferences.json
privacy-rules.json
audit/mcp.jsonl
```

The active workspace may contribute `CONTEXT.md`, `AGENTS.md`, or `CLAUDE.md`. These sources are considered only by personal-context search and are labeled with `workspace://` URIs.

## MCP Contract

### Tools

- `get_identity(purpose)`
- `get_current_focus(purpose)`
- `search_personal_context(query, purpose, max_items?)`

All tools are annotated as read-only, non-destructive, idempotent, and closed-world.

### ContextPack

```text
purpose
query?
client
access
items[]
  category
  label
  source
  content
  truncated
omissions[]
  source
  reason
  line_count
```

Each item is limited to 4,000 characters. Search returns at most six items. Absolute local paths are replaced by `aurora://`, `workspace://`, or filename-only `local://` URIs.

## Retrieval

Current retrieval is intentionally small and deterministic. It tokenizes ASCII terms and Chinese text, uses Chinese bigrams, scores term overlap, and gives stable base priority to identity, current focus, and preferences.

There is no vector database in the current runtime. Durable-memory retrieval will add a storage and indexing boundary without changing the MCP disclosure contract.

## Disclosure Policy

`privacy-rules.json` defines case-insensitive redaction markers. Lines containing a marker are omitted before external disclosure. Defaults are `private:` and `local-only:`.

The privacy rules document is used as policy input and is never returned as personal context.

The `preview` command uses the same external filtering path so the user can inspect the effective boundary.

## Authorization Boundary

The current release treats MCP registration in the host as authorization. `AURORA_MCP_CLIENT` labels the configured client in audit records. The `purpose` field is client-declared, not cryptographically verified.

Future authorization must add client identities, explicit scopes, grants, revocation, and per-source policy. Until then Aurora exposes only bounded read-only tools.

## Management CLI

```text
aurora serve [workspace]
aurora init [workspace]
aurora preview [workspace]
aurora audit [workspace]
```

No argument displays help. There is no interactive chat mode.

## Future Memory And Import Boundary

The planned ingestion path is separate from disclosure:

```text
Source Adapter
    -> immutable local source record
    -> normalized messages/documents
    -> memory candidates with provenance
    -> user review and correction
    -> durable memory store
    -> task retrieval
    -> existing DisclosurePolicy and ContextPack boundary
```

A source adapter parses a format; it does not decide that every statement is true. Candidate extraction may be performed by an authorized agent later, but promotion into durable memory requires Aurora policy and user control.
