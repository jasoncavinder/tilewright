# Architecture decision records

ADRs record consequential decisions that should remain understandable after the
implementation context has changed.

## Accepted decisions

| ADR | Status | Summary |
| --- | --- | --- |
| [0001: Project boundaries](0001-project-boundaries.md) | Accepted | One open-source workspace with a durable core library and thin CLI/MCP adapters; commercial applications remain separate. |

## Adding or changing a decision

An ADR should state its status, context, decision, rationale, consequences, and
important alternatives. Link evidence or experiments where they matter. Do not
mark a proposal accepted without maintainer agreement.

Prefer a new superseding ADR when reversing an accepted decision so that the
reasoning history remains visible. Update the architecture and user-facing docs
in the same change.
