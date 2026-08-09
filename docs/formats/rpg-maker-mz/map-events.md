# RPG Maker MZ selected-map event catalog contract

This document defines the evidence boundary for a small read-only projection of
event identity and placement from one evidenced `data/MapNNN.json` document. It
is a research result and accepted experimental contract, not a support claim.

## Question and scope

What is the smallest event projection that lets callers list the events on one
catalog-selected map without interpreting event pages, conditions, movement
routes, images, or commands?

Direct observations cover 1,555 event objects and 1,576 pages in 196 map
documents from four user-owned projects created by RPG Maker MZ 1.10.0. Official
MZ help separately documents map-scoped event IDs, editor-facing names and
memos, numbered event pages, and placement on a map. Later versions,
malformed-input editor behavior, and event mutation remain unknown.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| Map events have map-scoped automatically assigned IDs, editor names and memos, and numbered pages. | `MZ-HELP-MAP-EVENT-SETTINGS-2026-08-09` | Documented | High for the editor concepts | The help does not define their JSON encoding or malformed-input behavior. |
| Every audited event object has exactly the decoded keys `id`, `name`, `note`, `pages`, `x`, and `y`, with consistent scalar or array kinds. | `MZ-1.10.0-MAP-EVENT-SHAPE-AUDIT-2026-08-09` | Observed | High across 1,555 event objects | Plugins and later versions may add fields; decoded duplicate keys were not established absent. |
| Every observed event ID is a positive integer equal to its array index and unique within its map. | Shape audit | Observed | High across all audited events | Allocation after deletion and editor handling of mismatches remain unknown. |
| Event names are strings; all observed names are nonempty. | Shape audit plus official help | Documented and Observed | High in the audited scope | Empty-name editor behavior and whether names must be unique remain unknown. |
| Event `x` and `y` values are nonnegative integers and all observed coordinates are inside the containing map dimensions. | Shape audit plus documented placement behavior | Observed; placement meaning Documented | High in the audited scope | Coordinate limits, loop-map behavior, and editor handling of out-of-bounds values remain unknown. |
| Event `pages` values are nonempty arrays containing only objects; observed events have one through three pages. | Shape audit plus official help | Documented and Observed | High in the audited scope | Page-object fields, page semantics, commands, and editor handling of empty arrays remain outside this slice. |
| Tilewright's selected-map event catalog matches an independent direct extraction of every bounded field. | `MZ-1.10.0-MAP-EVENT-DIFFERENTIAL-2026-08-09` | Observed | High across 196 maps and 1,555 events | This does not test editor mutation, malformed-input behavior, later versions, or page semantics. |

## Aggregate shape audit

On 2026-08-09, a read-only aggregate audit inspected the same 196 three-digit
map documents authorized under `MZ-1.10.0-FRESH-4-2026-08-01`. The audit
verified the canonical ignored research root, source containment, and absence
of source symlinks before using `jq` 1.8.2.

The audit queried only decoded property names, JSON kinds, counts, integer
relationships and ranges, array lengths, and coordinate bounds. It emitted no
event names, notes, page contents, commands, project paths, raw documents,
excerpts, hashes, or per-project manifests.

The audit observed:

- 196 map documents containing 1,555 event objects and 1,576 page objects;
- one shared event key set: `id`, `name`, `note`, `pages`, `x`, and `y`;
- integer event IDs from 1 through 67, all equal to their array index and unique
  within the containing map;
- string names for every event, with no empty names;
- nonnegative integer coordinates, all within the containing map dimensions,
  with observed `x` values from 0 through 183 and `y` values from 0 through 176;
- nonempty page arrays containing one through three object entries; and
- a null entry at event-array index zero in every map, with all other array
  entries either null holes or event objects.

These are observations of MZ-generated states, not editor validation rules.

## Differential projection audit

On 2026-08-09, the selected-map event catalog was run read-only against the
same four authorized MZ 1.10.0 projects. For every coherent catalog record, an
independent `jq` extraction compared the map identity and dimensions, every
event ID, name, coordinate, and page count, the derived coordinate findings,
and the bounded CLI envelope.

All 196 selected maps, 1,555 event records, and 10,323 individual comparisons
matched exactly. Every snapshot was complete, every report had zero snapshot
diagnostics, and the independently derived and Tilewright coordinate-finding
lists were both empty. The audit emitted only aggregate counts and booleans.

This verifies one implementation against independently decoded MZ-generated
data. It does not establish editor validation, mutation, save/reopen fidelity,
page or command semantics, or behavior in later versions. No project path,
event name, note, page body, command, raw document, excerpt, field value, report,
hash, or per-project manifest was retained.

## Official documentation

The official *Map Event Settings* help page says that an event ID is unique
within its map and automatically assigned in creation order. It separately
describes the editor-facing Name, free-form Memo, and consecutively numbered
event pages. The official *Events* help page describes map events as events
placed and edited on a map.

- *Map Event Settings*, RPG Maker MZ Help,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_09_03.html>, accessed
  2026-08-09.
- *Events*, RPG Maker MZ Help,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_09.html>, accessed
  2026-08-09.

The help documents editor concepts, not the JSON property names or exact
serialization rules. Those remain bounded by direct observation.

## Accepted bounded typed contract

The experimental slice should accept an existing `ProjectSnapshot` and a
catalog-scoped `MapId`, then expose:

- a positive event ID scoped to the selected map;
- the decoded editor-facing event name;
- nonnegative `x` and `y` coordinate scalars; and
- the number of opaque event-page objects.

The result should include the selected map identity, exact project-relative map
path, map dimensions, records ordered by event ID, and deterministic contextual
findings for coordinates outside those dimensions. An out-of-bounds coordinate
is a Tilewright relationship finding, not a claim that MZ rejects the project.

The projection should require a coherent map catalog, a matching catalog
record, the evidenced three-digit map document, an object root, positive map
dimensions, and an `events` array. Each non-null entry should be an object with
unambiguous `id`, `name`, `x`, `y`, and `pages` fields. Event IDs should be
positive bounded integers equal to their array indexes. Coordinates should be
bounded unsigned integers. Page contents remain opaque; an empty page array can
be reported as a count of zero rather than treated as editor-invalid.

The `note` field, page bodies, commands, conditions, images, movement settings,
and every unknown field remain only in the untouched raw snapshot. The
projection introduces no mutation, serialization, persistence, runtime, or
editor-compatibility behavior.

## Fixture implications

Tests can generate minimal strict JSON in temporary project trees. No vendor
project or application asset is needed. The matrix should include:

- multiple event objects separated by null holes and returned in ID order;
- decoded names, zero coordinates, multiple opaque pages, and unknown fields;
- an empty page array without interpreting it as an editor-valid state;
- a deterministic out-of-bounds contextual finding;
- missing, duplicate, wrong-kind, malformed-string, and integer-boundary
  refusals for every required field;
- reserved index zero, index overflow, and ID/index mismatch refusals;
- catalog, document, root, dimensions, and event-array failures; and
- proof that unknown event and page fields remain byte-identical in the raw
  snapshot.

Generated values are Tilewright test inputs only. They do not establish that MZ
accepts or rejects malformed projects.

## Remaining unknowns and next experiments

- Create, move, rename, add a page to, and delete one event in separate
  disposable MZ 1.10.0 copies, saving and reopening after each action.
- Determine event-ID allocation and hole behavior after deletion and creation.
- Observe whether page deletion can produce an empty page array.
- Test how the editor handles out-of-bounds coordinates only if a future
  validation or write contract depends on that behavior.
- Audit page structure and command lists separately before typing either.
- Observe another named MZ version at or above 1.10.0 before generalizing the
  contract.
