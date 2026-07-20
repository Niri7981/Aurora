# AuroraPulse Domain Context

AuroraPulse is a local personal memory layer exposed to authorized AI agents through MCP. It is not a chat agent, model provider, voice assistant, or general tool runtime.

## Product Language

**Personal Memory Layer**: The product category. Aurora owns local personal context and makes bounded portions available to authorized agents.

**Local Source Of Truth**: User-owned identity, preferences, focus, and durable memories stored on the local machine.

**MCP Client**: An external AI host or agent configured to call Aurora tools.

**Context Gateway**: The read-only MCP boundary that selects, filters, bounds, labels, and audits personal context.

**ContextPack**: A task-scoped response containing disclosed items, stable source URIs, and omission metadata.

**Disclosure Policy**: Local rules applied before context leaves Aurora. Privacy rules are policy input, never context output.

**Minimum-Necessary Disclosure**: Returning the smallest useful amount of personal context for the stated task purpose.

**Audit Event**: A local record of the client, tool, purpose, sources, omissions, status, and time of a context request.

**Durable Memory**: A user-correctable personal fact, preference, episode, relationship, goal, or open thread with provenance and validity.

**Memory Candidate**: Imported or agent-proposed information that has not yet become trusted durable memory.

**Provenance**: The source records and timestamps that explain why Aurora holds a memory.

**Source Adapter**: A format-specific importer that converts chats, notes, email, or documents into normalized local source records.

**Raw Source**: Imported evidence such as a transcript. It is not automatically a memory and is not disclosed by default.

**Authorization Grant**: A future Aurora-managed rule identifying which client may access which tools and data scopes.

## Invariants

- Personal data remains local by default.
- MCP disclosure is read-only and bounded.
- Privacy filtering happens before serialization to the client.
- Privacy rules are never returned as personal context.
- Every successful disclosure is audited before it is returned.
- Raw imported content does not automatically become trusted memory.
- Durable memory must be inspectable, correctable, and deletable.
