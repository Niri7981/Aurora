# ADR 0001: Aurora Is A Local MCP Personal Memory Layer

## Status

Accepted on 2026-07-20.

## Context

AuroraPulse began as a local assistant runtime with model providers, a planner, a custom harness, native tools, and an interactive CLI. The product direction became clearer after testing cross-agent personal context through MCP.

The differentiating problem is not running another chat loop. It is maintaining one user-owned personal source of truth that multiple authorized AI agents can access without repeated introductions or provider-owned memory.

## Decision

AuroraPulse will be a local personal memory authority exposed through MCP.

The primary runtime contains:

- local personal context and future durable memory
- context loading and retrieval
- disclosure policy
- MCP tools and Context Packs
- local access auditing
- small management commands

The primary runtime does not contain:

- model-provider clients
- an interactive chat experience
- a planner or agent harness
- native desktop or media tools
- voice interaction

External AI agents provide intelligence and conversation. Aurora provides memory ownership, provenance, policy, and disclosure.

## Consequences

- The old Python assistant and Rust chat-agent runtime are removed.
- MCP becomes the product boundary rather than an adapter attached to another assistant.
- Future chat and document support enters through Source Adapters and reviewable memory candidates.
- Agent-proposed writes require a separate authorization and approval design.
- The codebase becomes smaller, but useful behavior now depends on MCP-compatible hosts.
