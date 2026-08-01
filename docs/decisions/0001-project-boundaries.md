# ADR 0001: Project boundaries

## Decision

Tilewright uses one open-source Cargo workspace containing:

1. The format-aware core library.
2. A CLI that depends on the library.
3. A thin MCP server that depends on the library.

Commercial applications are maintained in a separate private repository.

## Rationale

The library is the durable product. CLI, MCP, graphical interfaces, and AI
integrations are replaceable adapters around that library.
