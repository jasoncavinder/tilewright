# ADR 0011: Experimental Selected-Map Event Catalog

- **Status:** Proposed
- **Date:** 2026-08-09

## Context

Tilewright can select and summarize one map, but its events remain opaque beyond
their count. The next useful read-only slice is a compact list of event IDs,
editor names, positions, and page counts. Event-page contents and commands must
remain opaque until their own evidence and contracts exist.

The [map-event research](../formats/rpg-maker-mz/map-events.md) combines
official editor documentation with an aggregate audit of 1,555 events across
196 MZ 1.10.0 map documents. The public ownership, identifier scope,
malformed-structure behavior, coordinate findings, and adapter boundary require
an explicit decision.

## Decision

For the experimental selected-map event-catalog slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot` and catalog-scoped `MapId`. Adapters will not reimplement
   event-document interpretation.
2. A positive `MapEventId` will be scoped to the selected map. It is not a
   stable project-wide resource identifier.
3. The operation will require a coherent map catalog, matching record, and
   evidenced three-digit map document. It will parse only positive map
   dimensions and the `events` array from the map root.
4. A successful `MapEventCatalog` will own the selected map ID and catalog name,
   exact project-relative document path, dimensions, event records in ascending
   ID order, and deterministic contextual findings.
5. Each `MapEventRecord` will contain its map-scoped positive ID, decoded name,
   nonnegative `u32` coordinates, and opaque page count. It will not expose the
   memo, page objects, conditions, movement, images, commands, or extension
   fields.
6. Null array entries are accepted as holes. A non-null entry must be an object
   with unambiguous `id`, `name`, `x`, `y`, and `pages` fields. IDs must use the
   bounded unsigned-decimal form, be positive, and equal their array indexes.
   Coordinates use the same bounded form but may be zero. `pages` must be an
   array; its contents remain uninterpreted and an empty array is reported as
   count zero rather than declared invalid.
7. Coordinates outside the positive map dimensions produce ordered contextual
   findings. They do not prevent other event records from being returned and do
   not claim that RPG Maker MZ rejects the state.
8. Missing, duplicate, wrong-kind, malformed-string, unsupported-integer,
   reserved-zero, index-range, and ID/index mismatch conditions are typed
   structural errors. A structural error prevents the catalog rather than
   returning a partial typed result.
9. The raw snapshot retains all source bytes and unknown data. This operation
   performs no I/O, mutation, serialization, persistence, runtime execution, or
   editor compatibility check.

## Rationale

Event identity, names, and placement provide immediate inspection value without
requiring page or command semantics. A map-scoped identifier reflects the
official documentation and observed array relationship without prematurely
inventing project-wide identity. Separating coordinate findings from structural
errors follows the existing map-catalog pattern and avoids converting an
observed relationship into an editor-validity claim.

An owned projection keeps the provisional CST private and is straightforward
for CLI, MCP, and application callers. Leaving page bodies and memos in the raw
snapshot minimizes exposure and preserves unknown or plugin-defined content.

## Consequences

- Callers can list bounded event identity and placement for one selected map.
- Page count is useful scale information, not semantic understanding of a page.
- Unknown event and page content remains byte-identical in the snapshot.
- Malformed one-event structure prevents the typed catalog but not raw access.
- Future page, command, validation, or mutation slices require separate
  evidence and contracts.
- Evidence currently covers MZ 1.10.0 only, so the capability remains
  Experimental even if implemented.

## Alternatives Considered

- **Expose complete event and page models:** Rejected because page fields and
  commands have not completed evidence or preservation design.
- **Return CST nodes or generic JSON values:** Rejected because it leaks the
  provisional representation and makes adapters own domain behavior.
- **Use one project-wide event identifier:** Rejected because official and
  observed evidence scopes event identity to a map.
- **Treat out-of-bounds coordinates as a fatal parse error:** Rejected because
  the relationship can be reported without claiming editor invalidity.
- **Require nonempty page arrays:** Rejected because the corpus observation does
  not establish malformed-input editor behavior.
- **Expose event memos immediately:** Deferred because the first CLI use case
  does not require free-form memo content.

## Validation

Before implementation is ready for review, it must include generated synthetic
tests for success, holes, ordering, decoded names, coordinate findings, opaque
page counts, every structural refusal, raw-byte preservation, and absence of
mutation. Public Rustdoc and compatibility documentation must state identifier
scope and exact non-claims. Each adapter must add equivalent output, error, and
terminal-control tests when it exposes the operation.
