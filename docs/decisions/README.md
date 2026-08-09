# Architecture decision records

ADRs record consequential decisions that should remain understandable after the
implementation context has changed.

## Proposed decisions

| ADR | Status | Summary |
| --- | --- | --- |
| [0011: Experimental selected-map event catalog](0011-experimental-map-event-catalog.md) | Proposed | Add a read-only owned projection of map-scoped event IDs, names, positions, and opaque page counts for one catalog-selected map. |

## Accepted decisions

| ADR | Status | Summary |
| --- | --- | --- |
| [0001: Project boundaries](0001-project-boundaries.md) | Accepted | One open-source workspace with a durable core library and thin CLI/MCP adapters; commercial applications remain separate. |
| [0002: RPG Maker MZ version floor](0002-rpg-maker-mz-version-floor.md) | Accepted | Maintainer-led compatibility work targets RPG Maker MZ 1.10.0 and newer; older-version contributions remain welcome. |
| [0003: Stock authoring-data parity](0003-stock-authoring-data-parity.md) | Accepted | Tilewright pursues version-scoped, independently tested parity for stock MZ authoring data while preserving unknown and plugin-defined content. |
| [0004: Lossless JSON representation](0004-lossless-json-representation.md) | Accepted | Adopt a Concrete Syntax Tree (CST) architectural direction to retain source bytes, project typed views, and apply operation-specific edit envelopes. |
| [0005: Filesystem sandboxing](0005-filesystem-sandboxing.md) | Accepted | Tilewright will use capability-based filesystem APIs (`cap-std`) to confine project operations beneath an already-authorized directory handle. Initial root acquisition remains unresolved, and the current experimental discovery API uses `std::fs` as a temporary exception. |
| [0006: Snapshot loading contract](0006-snapshot-loading-contract.md) | Accepted | Adopt an experimental, capability-relative, bounded, read-only raw snapshot loader with deterministic partial diagnostics. |
| [0007: Experimental map catalog projection](0007-experimental-map-catalog-projection.md) | Accepted | Adopt a read-only, owned typed projection for map identity, display order, hierarchy, and contextual findings over a raw project snapshot. |
| [0008: Experimental selected-map summary](0008-experimental-selected-map-summary.md) | Accepted | Adopt a read-only owned summary for one catalog-selected map's basic metadata, dimensions, tileset scalar, and opaque event count. |
| [0009: Experimental system summary](0009-experimental-system-summary.md) | Accepted | Add a read-only owned summary for selected `System.json` metadata and map-position scalars. |
| [0010: Experimental player-start validation](0010-experimental-player-start-validation.md) | Accepted | Compose the bounded system, catalog, and selected-map projections into deterministic player-start findings without implying general project validity. |

## Adding or changing a decision

An ADR should state its status, context, decision, rationale, consequences, and
important alternatives. Link evidence or experiments where they matter. Do not
mark a proposal accepted without maintainer agreement.

Prefer a new superseding ADR when reversing an accepted decision so that the
reasoning history remains visible. Update the architecture and user-facing docs
in the same change.
