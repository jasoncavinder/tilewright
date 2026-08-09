# RPG Maker MZ map-to-tileset validation contract

This document defines the evidence boundary for checking whether cataloged maps'
positive `tilesetId` values resolve to records in exact `data/Tilesets.json`.
It is a research result and proposed experimental contract, not a support claim.

## Question and scope

Can Tilewright compose its accepted map and tileset projections into a useful
project-wide reference check without implying general validity or interpreting
tileset behavior?

Direct observations cover 196 map references in four user-owned projects
created by RPG Maker MZ 1.10.0. Every observed positive value resolved to an
ID/index-consistent tileset record. Missing references, malformed input, editor
enforcement, later versions, assets, and runtime behavior remain unknown.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| Map Properties includes a tileset selection, and Tileset Settings describes configurations assigned to maps. | Official MZ help cited below | Documented | High for the editor concept | The help does not specify JSON encoding or malformed-reference behavior. |
| Every audited map's positive `tilesetId` resolves to an ID/index-consistent record in its project's tileset array. | `MZ-1.10.0-TILESET-SHAPE-AUDIT-2026-08-09` | Observed; relationship meaning Documented and Inferred | High across 196 maps in four projects | Missing references and editor enforcement were not tested. |
| Tilewright's validator matches an independent direct extraction of every map reference and bounded CLI-envelope field. | `MZ-1.10.0-MAP-TILESET-DIFFERENTIAL-2026-08-09` | Observed | High across all four projects and 196 maps | This does not test malformed references, editor behavior, mutation, or later versions. |

## Aggregate observation

The read-only aggregate audit recorded in the
[tileset-catalog contract](tileset-catalog.md) inspected 196 map documents and
24 tileset records from four authorized MZ 1.10.0 projects. It found positive
map references using IDs 1 through 4, all resolving within their containing
projects. The audit emitted only aggregate counts and relationships.

These are observations of MZ-generated states, not a rule that every accepted
project must satisfy the relationship or that the editor rejects a violation.

## Differential validation audit

On 2026-08-09, the validator and its JSON CLI adapter were run read-only against
the same four authorized projects. An independent `jq` extraction decoded every
cataloged map's `tilesetId` and checked it against the containing project's
tileset array and record ID.

All four projects and all 196 map references matched. The audit also compared
the versioned schema, validation scope, map count, finding count, snapshot
completeness, and diagnostic count for each project, for 220 comparisons in
total. All snapshots were complete, with zero diagnostics and zero unresolved
references. Only aggregate counts and booleans were emitted.

This verifies one implementation against independently decoded MZ-generated
data. It does not establish malformed-reference editor behavior, general
validity, assets, tile behavior, mutation, persistence, or later versions. No
project path, map or tileset name, raw document, excerpt, field value, report,
digest, or per-project manifest was retained.

## Official documentation

The official *Map Properties* help documents selecting a tileset for a map. The
official *Tileset Settings* help describes tilesets as configurations assigned
to maps.

- *Map Properties*, RPG Maker MZ Help,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_07_03.html>, accessed
  2026-08-09.
- *Tileset Settings*, RPG Maker MZ Help,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_08_10.html>, accessed
  2026-08-09.

The help documents editor concepts, not JSON property names or enforcement.

## Proposed bounded validation contract

The operation should accept an existing `ProjectSnapshot`, require coherent map
and tileset catalogs, summarize each cataloged map in ascending ID order, and
report a typed finding when a positive `tilesetId` lacks a matching record.

Structural prerequisite failures should remain typed errors. Findings should be
successful results and should not claim editor rejection. The operation should
not inspect modes, image slots, flags, asset existence, tile behavior, raw
events, runtime behavior, mutation, serialization, or persistence. Exact bytes
and unprojected fields remain in the snapshot.

## Fixture implications

Tests can generate minimal strict JSON in temporary project trees. They should
cover finding-free and missing-reference cases, map-ID ordering, malformed map
and tileset prerequisites, missing documents, resource limits, raw-byte
preservation, deterministic adapters, and absence of mutation.

Generated fixtures are Tilewright-owned test inputs only and do not establish
that MZ accepts or rejects malformed states.

## Remaining unknowns and next experiments

- Change one map's selected tileset and confirm the stored reference after save
  and reopen.
- Create a controlled missing reference and observe editor load, save, and test
  behavior without generalizing beyond that version and state.
- Determine whether partial validation is useful when one selected map is
  structurally unavailable.
- Observe another named MZ version at or above 1.10.0 before generalizing the
  contract.
