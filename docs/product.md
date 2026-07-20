# AuroraPulse Product

## Product Definition

AuroraPulse is a local personal memory layer for AI agents.

Its north star is:

> Tell Aurora once. Every AI you authorize can know you.

Aurora owns neither the conversation nor the intelligence. It owns the user's local source of truth and the boundary that decides what an external AI may know for one task.

## Problem

Personal context is fragmented across chat products, model accounts, notes, files, and repeated introductions. Provider memory is incomplete, difficult to inspect, and unavailable to other agents. The user should not have to rebuild their identity for every new model or conversation.

## Product Promise

- Personal memory remains on the user's machine.
- The user can inspect, edit, correct, and delete it.
- Compatible AI agents access it through MCP.
- Each request receives a small, task-relevant Context Pack rather than a full memory dump.
- Every disclosure has a source and a local audit event.
- Sensitive material is filtered before it crosses the process boundary.

## Current Product Boundary

Aurora currently provides:

- editable identity, current-focus, preference, and privacy files
- optional project context from the active workspace
- read-only MCP tools for identity and personal-context retrieval
- marker-based privacy filtering
- bounded lexical retrieval
- stable source URIs and omission metadata
- fail-closed local disclosure auditing

Aurora currently does not provide:

- an embedded chat experience
- model-provider integrations
- desktop or media automation
- autonomous tool execution
- automatic long-term memory writes
- broad document ingestion
- cloud synchronization

## Product Layers

### 1. Personal Source Of Truth

User-owned identity, preferences, current focus, and later durable memories. Every derived memory must retain provenance and correction history.

### 2. Context Gateway

The MCP boundary through which authorized agents request personal context. The gateway selects, filters, bounds, labels, and audits every response.

### 3. Ingestion And Memory Curation

The next product layer. Source adapters will import chats, notes, email, and documents into a local raw archive. Imported material becomes reviewable memory candidates before it becomes trusted personal memory.

## Trust Principles

### Local ownership

Aurora data is stored locally in user-controlled files or databases.

### Minimum necessary disclosure

The default response is the smallest useful Context Pack for the stated purpose.

### Provenance before inference

A memory without a source, time, and subject is not trustworthy enough to become durable personal context.

### Correction over silent learning

The user must be able to reject, revise, expire, or delete inferred memories.

### Raw sources are not memories

A chat transcript can contain other people's facts, jokes, quotations, temporary emotions, and stale statements. Raw imports remain evidence, not automatic identity.

### Agents do not own memory

Agents may eventually propose memories, but Aurora and the user decide what becomes durable and what may be disclosed.

## Initial User

The first user is a person who uses several AI agents on one computer and wants those agents to share accurate personal context without handing durable identity ownership to a model provider.

## Success

Aurora succeeds when a fresh authorized agent can help immediately using current, relevant personal context, while the user can answer all three questions:

1. What does Aurora know about me?
2. Why does it believe that?
3. Which agent received it, and why?
