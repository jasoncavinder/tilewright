# ADR 0001: Project boundaries

- **Status:** Accepted
- **Date:** 2026-08-01

## Context

Tilewright needs reusable project-format behavior plus interfaces for people,
scripts, and AI clients. Those components will evolve together, and a core API
change may require coordinated updates to adapters, tests, fixtures, and
documentation.

The maintainer also intends to build commercial products around the public core.
The public format implementation must remain useful independently and must not
become coupled to private product code.

## Decision

Tilewright uses one open-source Cargo workspace containing:

1. `tilewright`, the format-aware core library and primary public API;
2. `tilewright-cli`, a human- and script-facing adapter over the library; and
3. `tilewright-mcp`, a thin Model Context Protocol adapter over the library.

Dependency direction is inward: CLI and MCP may depend on `tilewright`;
`tilewright` must not depend on either adapter. Core behavior must not depend on
AI providers, agent hosts, GUI frameworks, terminal presentation, or commercial
Tilewright products.

Commercial applications are maintained in a separate private repository. The
public repository must not read from, copy from, or depend on that repository.

## Rationale

The library is the durable product. CLI, MCP, graphical interfaces, AI
integrations, and commercial applications are replaceable adapters around that
library. A monorepo still allows related open-source changes to be atomic and to
share one history, lockfile, CI policy, and documentation structure.

## Consequences

- Parsing, validation, domain modeling, serialization, compatibility, and safe
  mutations belong in the library.
- Adapters own protocol validation, translation, and presentation but must not
  duplicate domain behavior.
- The three packages may eventually be published independently and therefore
  require deliberate API and release coordination.
- Reusable format knowledge belongs in the public core even when a commercial
  product motivates it.
- Private product features and configuration cannot be used as undeclared
  implementation inputs to this repository.

The concrete raw/typed representation, loading API, error model, and write
transaction design are intentionally outside this ADR.
