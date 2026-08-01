---
name: public-api-evolution
description: Design and review changes to Tilewright's durable core-library API, keeping adapters thin, data transformations explicit, compatibility intentional, and error behavior testable.
license: MPL-2.0
compatibility: opencode
metadata:
  audience: library-maintainers
  workflow: api-design
---

# Public API evolution

Use this whenever a change adds, removes, or materially alters public types, traits, functions, errors, serialization behavior, or cross-crate contracts.

## Review sequence

1. Define the caller's real use case without assuming CLI, MCP, OpenCode, or GUI concepts belong in the core.
2. Identify the smallest domain operation that supports that use case.
3. Establish input, output, invariants, failure modes, and mutation boundaries.
4. Check whether unknown project data survives a read-modify-write round trip.
5. Keep I/O separate from pure transformation where that improves testability.
6. Prefer typed errors with useful context over strings or panics.
7. Consider forward compatibility and pre-1.0 migration cost without freezing premature abstractions.
8. Require examples, public docs, and focused tests for the contract.

## Boundary test

A public core API is suspect if its vocabulary includes:

- prompts, agents, models, MCP tool calls, or OpenCode sessions;
- terminal formatting or CLI argument parsing;
- GUI state or commercial-edition concepts.

Those belong in adapters unless they describe an editor-independent domain concept.

## Decision output

Return:

- proposed contract;
- alternatives considered;
- why the behavior belongs in the selected crate;
- data-preservation and error semantics;
- compatibility impact;
- required tests and docs;
- whether an ADR is warranted.
