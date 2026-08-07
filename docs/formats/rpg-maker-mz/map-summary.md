# RPG Maker MZ selected-map summary contract

This document defines the evidence boundary for a small read-only projection of
one evidenced `data/MapNNN.json` document. It is a research result and accepted
experimental contract, not a support claim.

## Question and scope

What is the smallest selected-map summary that gives callers useful information
after map-catalog listing without parsing tile layers, event commands, or every
map setting?

Direct observations cover 196 map documents from four user-owned projects
created by RPG Maker MZ 1.10.0 on the recorded macOS environment. Existing
controlled experiments also establish the relationship between map-catalog IDs,
three-digit map filenames, blank map `displayName` values, and map event
creation. Later versions and malformed-input editor behavior remain unknown.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| Every audited map document has an object root and the same 25 decoded top-level property names. | `MZ-1.10.0-MAP-SUMMARY-SHAPE-AUDIT-2026-08-06` | Observed | High across all 196 documents | Duplicate decoded properties were not established absent; plugin and later-version fields may extend the object. |
| `displayName` is present and string-valued; all fresh-project values are empty. | Shape audit plus controlled map-creation records | Observed | High for presence and kind in the observed scope | Nonempty editor-authored values and malformed-input behavior were not directly audited here. |
| `width` and `height` are positive integer-valued numbers; observed ranges are 17–200 and 13–200. | Shape audit | Observed syntax; dimension meaning Inferred | High for observed kinds and ranges; high-confidence interpretation from names and tile-data relationship | Editor limits, zero or negative handling, and other versions remain unknown. |
| `tilesetId` is a positive integer-valued number from 1 through 4, and every value resolves to a matching `Tilesets.json` record in its project. | Shape audit and independent cross-file comparison | Observed; reference meaning Inferred | High for all 196 documents | Tileset zero, missing references, other identifiers, and editor enforcement remain unknown. |
| `events` is an array containing only null holes and objects; the corpus contains 1,555 object entries, and every observed object ID equals its array index. | Shape audit plus documented map-event role and controlled event creation | Documented and Observed | High across all 196 documents | Event fields, pages, commands, duplicate IDs, malformed entries, and editor enforcement remain outside this slice. |
| `data` is an integer array whose length equals `width * height * 6` in every audited document. | Shape audit | Observed; layer meaning Unknown | High for the arithmetic relationship in this corpus | Layer ordering, tile encoding, mutation rules, and other-version stability remain unestablished. |
| Tilewright's selected-map summary matches an independent direct extraction of every bounded field. | `MZ-1.10.0-MAP-SUMMARY-DIFFERENTIAL-2026-08-06` | Observed | High across all 196 summaries in the four-project MZ 1.10.0 corpus | This does not test malformed-input editor behavior, later versions, tile meaning, event meaning, or editor reopen behavior. |

## Aggregate shape audit

On 2026-08-06, a read-only aggregate audit inspected the 196 three-digit map
documents already authorized under `MZ-1.10.0-FRESH-4-2026-08-01`. The source
set contained 4,348,582 bytes; the largest document contained 703,471 bytes.
The audit verified the canonical ignored research root, regular file types and
sizes, and absence of source symlinks before using `jq` 1.8.2.

The audit queried only decoded property names, JSON kinds, counts, integer
relationships, numeric ranges, array lengths, and cross-file identifier
existence. It emitted no display names, notes, event contents, tile values,
project paths, raw documents, excerpts, or per-project manifests.

The audit observed:

- 196 object roots with one shared set of 25 decoded property names;
- string `displayName` in every document, empty in all 196 fresh-project maps;
- positive integer-valued `width`, `height`, and `tilesetId` in every document;
- width range 17–200, height range 13–200, and four distinct tileset IDs
  from 1 through 4;
- every tileset ID resolving to an ID/index-consistent `Tilesets.json` record in
  the same project;
- 196 `data` arrays containing only integer-valued numbers, each with exactly
  six values per map-area cell;
- 196 `events` arrays containing 1,555 objects and 290 null slots, with no other
  entry kinds; and
- event object IDs equal to their array indices in every observed entry.

These are observations of MZ-generated states, not editor validation rules.

## Differential projection audit

On 2026-08-06, the selected-map summary at implementation commit
`8dfa46c0dee0e8f8b229101c45428b0e066d9655` was run read-only against the same
four authorized MZ 1.10.0 projects. A temporary external Rust harness used
`serde_json` 1.0.151 to decode the source documents independently from
Tilewright's retained CST.

For every coherent catalog record, the harness compared catalog name, exact
three-digit project-relative path, map `displayName`, width, height, tileset ID,
and non-null object event count. Comparisons occurred in memory and emitted only
anonymized case numbers, counts, and boolean results.

All 196 selected-map summaries matched exactly. Every project snapshot was
complete, and all four projects produced zero snapshot diagnostics and zero
map-catalog findings. No project path, decoded name, raw document, excerpt,
field value, CLI report, digest, or per-project manifest was retained.

This verifies the implemented read-only projection against independently
decoded MZ-generated data. It is not an editor validation, mutation, save, or
reopen experiment and does not extend the version scope beyond MZ 1.10.0.

## Bounded typed contract

The experimental typed slice accepts an existing `ProjectSnapshot` and a
catalog-scoped `MapId`, then exposes only:

- the selected map ID and catalog name;
- the exact evidenced project-relative map-document path;
- the decoded map `displayName`;
- positive width and height;
- a positive tileset ID scalar; and
- the number of non-null event objects.

The operation requires a structurally coherent map catalog and a matching
catalog record, refuses IDs outside the evidenced three-digit filename family,
and distinguishes an absent document from a document unavailable through
snapshot diagnostics. It refuses missing, duplicate, wrong-kind, or unsupported
required values rather than silently choose or coerce them.

Unknown map fields and all event contents remain in the untouched raw snapshot.
The projection does not parse `data`, interpret layers or tiles, validate a
tileset reference, inspect events, or expose mutation and serialization.
Catalog findings unrelated to the selected document should not prevent a
summary once the required catalog and document structures are coherent.

## Fixture implications

Tests can generate minimal strict JSON in temporary project trees. No vendor
project is needed. The fixture matrix should include:

- one catalog record and one minimal matching map object;
- empty and nonempty `displayName` strings;
- null event holes and opaque event objects;
- unknown top-level and nested event fields retained in the raw document;
- every missing, duplicate, wrong-kind, and integer-boundary refusal;
- missing, unavailable, and non-file map-document cases;
- an unknown catalog ID and an ID above the evidenced three-digit range; and
- proof that unrelated map fields, tile data, and event bodies are not required
  or interpreted by the projection.

Generated fixture values are Tilewright test inputs only. They do not establish
that MZ accepts or rejects malformed projects.

## Remaining unknowns and next experiments

- Change only a map's Display Name and dimensions in a disposable authorized
  copy, then confirm the precise persisted fields after save and reopen.
- Change a map's tileset and confirm the cross-file identifier relationship.
- Determine whether zero dimensions or tileset ID zero can be produced or
  tolerated before describing them as editor-invalid.
- Establish tile-layer ordering before exposing typed tile data.
- Audit event structure separately before exposing event IDs, names, pages, or
  command counts.
- Observe another named MZ version at or above 1.10.0 before generalizing the
  contract.
