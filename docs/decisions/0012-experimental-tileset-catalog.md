# ADR 0012: Experimental Tileset Catalog

- **Status:** Proposed
- **Date:** 2026-08-09

## Context

Tilewright's selected-map summary exposes a positive `tilesetId` scalar but
cannot resolve it to even an editor-facing name. The next database-table slice
should make tileset identity inspectable without prematurely interpreting tile
flags, image slots, modes, or notes.

The [tileset-catalog research](../formats/rpg-maker-mz/tileset-catalog.md)
combines official editor documentation with an aggregate audit of 24 tileset
records and 196 map references from four MZ 1.10.0 projects. Public identifier
scope, malformed-structure behavior, and unknown-field preservation require an
explicit decision.

## Decision

For the experimental tileset-catalog slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot`. Adapters will not reimplement tileset interpretation.
2. A positive `TilesetId` will be scoped to this experimental catalog. It is not
   yet a stable project-wide resource identifier.
3. The operation will select exact `data/Tilesets.json` and return an owned
   `TilesetCatalog` whose records are ordered by ID.
4. Each `TilesetRecord` will contain only its positive ID and decoded
   editor-facing name.
5. The root must be an array. Null entries are accepted as holes. Each non-null
   entry must be an object with unambiguous `id` and `name` fields; the ID must
   use the bounded unsigned-decimal form, be positive, and equal its array
   index.
6. Missing, duplicate, wrong-kind, malformed-string, unsupported-integer,
   reserved-index, index-range, and ID/index mismatch conditions are typed
   structural errors. A structural error prevents the catalog rather than
   returning a partial typed result.
7. `mode`, `tilesetNames`, `flags`, `note`, and unknown fields remain only in the
   untouched raw snapshot. The operation does not interpret or validate them.
8. Map-reference validation, assets, tile meaning, mutation, serialization,
   persistence, runtime behavior, and editor compatibility remain outside this
   decision.

## Rationale

Identity and name make the existing selected-map tileset scalar useful while
staying within strong evidence. An owned projection keeps the provisional CST
private and is straightforward for multiple adapters. Requiring ID/index
equality follows every observed database record and avoids inventing an
alternate identity when structure is inconsistent.

Deferring adjacent fields is material: observed integer modes do not by
themselves establish their enum mapping, and image-slot and flag arrays carry
substantially broader asset and tile-behavior semantics.

## Consequences

- Callers can list and resolve tileset IDs to decoded editor names.
- Unknown fields and exact bytes remain retained in `ProjectSnapshot`.
- A malformed required record prevents the typed catalog but not raw access.
- Cross-file reference validation remains a later compositional operation.
- Evidence currently covers MZ 1.10.0 only, so the capability remains
  Experimental even if implemented.

## Alternatives Considered

- **Expose mode, image names, and flags now:** Rejected because their exact
  encoding and semantic contracts need separate evidence.
- **Return CST nodes or generic JSON values:** Rejected because it leaks the
  provisional representation and makes adapters own domain behavior.
- **Reuse `MapId`:** Rejected because map and tileset identities are different
  resource domains.
- **Validate every map reference during catalog projection:** Deferred to a
  separate contextual validation operation so catalog structure remains useful
  independently.
- **Treat holes as fatal:** Rejected because indexed MZ database arrays commonly
  reserve index zero and may use null entries.

## Validation

Before implementation is ready for review, it must include generated synthetic
tests for success, holes, ordering, decoded strings, all structural refusals,
raw-byte preservation, and absence of mutation. Public Rustdoc and compatibility
documentation must state identifier scope and exact non-claims. Each adapter
must add deterministic output, error, and terminal-control tests.
