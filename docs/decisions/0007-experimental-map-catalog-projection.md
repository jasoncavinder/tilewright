# ADR 0007: Experimental Map Catalog Projection

- **Status:** Proposed
- **Date:** 2026-08-06

## Context

Tilewright now loads bounded, read-only raw project snapshots, but callers
cannot inspect any RPG Maker MZ concepts without reading raw JSON. The first
typed vertical slice should let callers list maps and understand their parent
relationships while preserving the lossless documents selected by ADR 0004.

The [map catalog research](../formats/rpg-maker-mz/map-catalog.md) supports a
narrow MZ 1.10.0 projection over `data/MapInfos.json`. Typed-view ownership,
stable identifiers, malformed-input behavior, and the distinction between
shape errors and project findings are consequential public API choices.

## Decision

For the experimental map-catalog slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot`. CLI and future MCP adapters will translate its result but
   will not reimplement map interpretation.
2. The operation will return an owned typed projection. It will not expose CST
   nodes, borrow adapter concepts, mutate the snapshot, or imply a production
   ownership model for future mutable views.
3. A public `MapId` will represent a positive `u32` map identifier. A record is
   accepted only when its decoded `id` equals its array index. `parentId == 0`
   maps to no parent; other accepted parent values map to `MapId`.
4. The first record exposes only `id`, decoded `name`, positive `order`, and
   optional parent ID. Other fields remain in the untouched raw document and do
   not make an otherwise understood record fail.
5. The operation returns a typed error instead of a partial typed catalog when
   the document is unavailable or a root, entry, or required field is missing,
   duplicated, the wrong JSON kind, or outside the bounded integer contract.
   The caller still retains the raw snapshot for inspection.
6. Once structural projection succeeds, deterministic contextual findings may
   report missing parents, cycles, duplicate order values, missing corresponding
   map documents, orphan evidenced map-document paths, and IDs whose document
   filename relationship is not evidenced. Classified map paths that do not
   encode a positive ID are also reported. Findings are observations about the
   project and Tilewright's evidence boundary, not claims that MZ rejects the
   project.
7. Records are addressable in map-ID order and separately enumerable in
   display order with the map ID as a deterministic tie-breaker.
8. No mutation, serialization, validation severity, or persistent write API is
   introduced by this decision.

## Rationale

An owned projection keeps the initial API small and avoids exposing the
provisional CST backend or committing to lifetime-heavy mutable views. Refusing
structurally ambiguous input prevents the typed catalog from silently omitting
or choosing among records. Contextual findings remain useful without conflating
Tilewright's consistency analysis with observed editor validation behavior.

The positive `MapId` newtype prevents root-parent sentinel zero from becoming a
map identifier. Requiring ID/index equality matches every observed record and
the controlled creation/deletion evidence while making mismatches explicit
rather than normalizing either value.

## Consequences

- Callers can build map lists and hierarchy displays without parsing JSON.
- The raw snapshot remains the authority for unknown fields and exact bytes.
- A malformed required record prevents the typed projection, but does not make
  the underlying raw snapshot unusable.
- This experimental owned projection does not resolve production ownership,
  threading, stale-view, or mutation design.
- `MapId` begins as an experimental resource identifier scoped to map catalog
  records; it is not yet a stable project-wide identity scheme.
- Relationship findings require explicit non-validation wording in adapters and
  compatibility documentation.

## Alternatives Considered

- **Deserialize through a second generic JSON DOM:** Rejected because it would
  collapse duplicate names and create a competing representation when the
  retained CST already supports read traversal.
- **Expose CST nodes publicly:** Rejected by ADR 0004 because it would leak a
  provisional dependency and make domain callers responsible for ambiguity.
- **Return partial typed records after shape errors:** Deferred because an
  incomplete catalog can make hierarchy and cross-file findings misleading.
  The raw snapshot already provides partial syntax-level inspection.
- **Use array index without checking the `id` field:** Rejected because it would
  hide a contradiction in a relationship observed to agree across all current
  evidence.
- **Treat every relationship finding as a fatal error:** Rejected because the
  editor's behavior for inconsistent parents, orders, and files is not yet
  established, and callers can still benefit from a structurally coherent
  catalog.

## Validation

Before implementation is ready for review, it must include synthetic tests for
all structural refusals, duplicate required names, unknown-field tolerance,
identifier/index handling, deterministic ordering, relationship findings, raw
byte preservation, and absence of mutation. Public Rustdoc, CLI documentation,
and the compatibility matrix must state the exact experimental non-claims.
