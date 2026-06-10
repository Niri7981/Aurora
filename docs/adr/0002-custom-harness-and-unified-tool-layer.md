# Custom harness and unified tool layer

AuroraPulse will keep its request lifecycle, context engineering, validation, clarification, and tool dispatch inside a custom harness, while exposing capabilities through a unified native tool layer. We chose this because the product depends on tight control over local context, bounded single-step behavior, and predictable execution, which would be harder to preserve if the core runtime were handed over to a generic agent framework or treated as an MCP-shaped shell from the start.
