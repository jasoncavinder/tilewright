# Architecture decision records

ADRs record consequential decisions that should remain understandable after the
implementation context has changed.

## Accepted decisions

| ADR | Status | Summary |
| --- | --- | --- |
| [0001: Project boundaries](0001-project-boundaries.md) | Accepted | One open-source workspace with a durable core library and thin CLI/MCP adapters; commercial applications remain separate. |
| [0002: RPG Maker MZ version floor](0002-rpg-maker-mz-version-floor.md) | Accepted | Maintainer-led compatibility work targets RPG Maker MZ 1.10.0 and newer; older-version contributions remain welcome. |
| [0003: Stock authoring-data parity](0003-stock-authoring-data-parity.md) | Accepted | Tilewright pursues version-scoped, independently tested parity for stock MZ authoring data while preserving unknown and plugin-defined content. |
| [0004: Filesystem sandboxing](0004-filesystem-sandboxing.md) | Accepted | Tilewright will use capability-based filesystem APIs (`cap-std`) to confine project operations beneath an already-authorized directory handle. Initial root acquisition remains unresolved, and the current experimental discovery API uses `std::fs` as a temporary exception. |

## Adding or changing a decision

An ADR should state its status, context, decision, rationale, consequences, and
important alternatives. Link evidence or experiments where they matter. Do not
mark a proposal accepted without maintainer agreement.

Prefer a new superseding ADR when reversing an accepted decision so that the
reasoning history remains visible. Update the architecture and user-facing docs
in the same change.
