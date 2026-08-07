# ADR 0008: Experimental Selected-Map Summary

- **Status:** Accepted
- **Date:** 2026-08-06

## Context

Tilewright can load a raw project snapshot and project the `MapInfos.json`
catalog, but callers cannot inspect even basic properties of a selected
`MapNNN.json` document without reading raw JSON. The next vertical slice should
make one selected map useful from the CLI while avoiding premature tile-layer,
event-page, command, validation, or mutation contracts.

The [selected-map summary research](../formats/rpg-maker-mz/map-summary.md)
supports a narrow MZ 1.10.0 projection for the catalog name, map display name,
dimensions, tileset ID scalar, and count of opaque event objects. How this
operation composes with `MapId`, map-catalog findings, document availability,
and malformed required fields is a consequential public API choice.

## Decision

For the experimental selected-map summary slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot` and catalog-scoped `MapId`. CLI and future MCP adapters may
   render the result but will not reimplement map-document interpretation.
2. The operation will first require a structurally coherent map catalog and a
   matching catalog record. A catalog shape error or unknown requested map ID
   prevents a summary. Contextual catalog findings do not independently prevent
   the requested summary.
3. The evidenced document relationship is limited to positive IDs through 999
   and exact three-digit `data/MapNNN.json` paths. An ID outside that range is a
   typed refusal, not a guessed filename.
4. The operation will return an owned `MapSummary` containing the map ID,
   catalog name, exact project-relative document path, decoded `displayName`,
   positive `u32` width and height, positive `u32` tileset ID scalar, and count
   of non-null event objects.
5. The operation will return a typed error rather than a partial summary when
   the selected map document is absent, unavailable through snapshot
   diagnostics, has a non-object root, or has a missing, duplicate, wrong-kind,
   or unsupported required value. Required decoded names are `displayName`,
   `width`, `height`, `tilesetId`, and `events`. Numeric fields use the same
   bounded unsigned-decimal lexeme policy as the map catalog; width, height, and
   tileset ID additionally refuse zero.
6. Event counting will accept null holes and opaque object entries. It will not
   decode event IDs, names, pages, conditions, commands, or plugin-defined
   fields. A non-null, non-object entry is a structural refusal.
7. The operation will not require or interpret tile `data`, encounters,
   scrolling, audio, parallax, notes, or other map fields. All source bytes,
   unknown fields, and event bodies remain available only through the untouched
   raw snapshot.
8. The tileset ID is an observed positive scalar in this experimental result;
   the operation will not load `Tilesets.json`, validate the reference, or
   introduce a stable project-wide tileset identifier.
9. No validation severity, mutation, serialization, persistence, or editor
   compatibility claim is introduced by this decision.

## Rationale

Requiring an existing catalog record keeps selection tied to the map identity
contract already accepted in ADR 0007 and avoids turning orphan files into
implicitly valid maps. Returning one owned summary follows the first typed
slice's backend-neutral ownership model without settling future mutable views.

The selected fields answer a concrete inspection question while leaving tile
arrays and event bodies opaque. Treating event objects as countable but
uninterpreted gives callers useful scale information without converting a JSON
object into an understood event. Refusing unevidenced filenames and ambiguous
required names preserves the evidence boundary.

## Consequences

- Callers can inspect a selected map's basic identity, dimensions, display
  metadata, tileset scalar, and event count without parsing JSON.
- Unknown fields and exact bytes remain retained by `ProjectSnapshot`.
- A malformed required map field prevents the summary but does not make the raw
  snapshot unusable.
- Re-projecting the catalog is acceptable for this small experimental API; a
  future project model may compose typed slices differently.
- Positive width, height, and tileset bounds reflect the observed corpus and
  may require revision if later evidence establishes legitimate zero values.
- Event objects are counted, not semantically understood or validated.

## Alternatives Considered

- **Expose a generic typed map object now:** Rejected because tile layers,
  encounters, audio, parallax, and events have not completed their own evidence
  and API work.
- **Return raw JSON values or CST nodes:** Rejected because it leaks the
  provisional representation and makes adapters implement domain behavior.
- **Summarize orphan map files without a catalog record:** Rejected because the
  selected identity would bypass ADR 0007 and overstate an inconsistent path as
  an understood map.
- **Parse event IDs and pages while counting:** Deferred to a separate event
  vertical slice with its own evidence, diagnostics, and unknown-command policy.
- **Validate tile-data length and tileset existence:** Deferred to validation;
  neither is required to answer the selected summary question.
- **Accept IDs above 999 by widening the filename:** Rejected because the
  current evidence establishes only the three-digit family.

## Validation

Before implementation is ready for review, it must include generated synthetic
tests for successful projection, empty and nonempty display names, null event
holes, opaque unknown event content, every structural refusal, numeric
boundaries, catalog and document failures, unevidenced IDs, raw byte
preservation, and absence of mutation. Public Rustdoc, CLI documentation, and
the compatibility matrix must state the exact experimental non-claims.
