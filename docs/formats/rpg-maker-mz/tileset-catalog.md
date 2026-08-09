# RPG Maker MZ tileset catalog contract

This document defines the evidence boundary for a small read-only projection of
tileset identity and editor-facing names from exact `data/Tilesets.json`. It is
a research result and accepted experimental contract, not a support claim.

## Question and scope

What is the smallest tileset projection that lets callers resolve the positive
`tilesetId` scalar already exposed by selected-map summaries without typing tile
flags, image slots, modes, or notes?

Direct observations cover 24 tileset records in four user-owned projects
created by RPG Maker MZ 1.10.0 and 196 map references. Official MZ help
separately documents tilesets, their editor-facing names, modes, images, tile
settings, and assignment to maps. Later versions, record lifecycle, malformed
input, and editor persistence remain unknown.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| Tilesets are database records used to design maps, with editor-only names and separately configured modes, images, tile settings, and notes. | `MZ-HELP-TILESET-SETTINGS-2026-08-09` | Documented | High for editor concepts | The help does not define JSON encoding or malformed-input behavior. |
| Each audited `Tilesets.json` root is a seven-entry array with null at index zero and six object records. | `MZ-1.10.0-TILESET-SHAPE-AUDIT-2026-08-09` | Observed | High across all four projects | Record-count changes and lifecycle operations were not observed. |
| Every observed record has exactly `flags`, `id`, `mode`, `name`, `note`, and `tilesetNames`; IDs are positive integers equal to array indexes and names are nonempty strings. | Shape audit | Observed | High across 24 records | Plugins or later versions may add fields; decoded duplicate keys were not established absent. |
| Every audited map's positive `tilesetId` resolves to an ID/index-consistent record in its project's tileset array. | Shape audit and 196-map cross-file comparison | Observed; reference meaning Documented and Inferred | High in the audited scope | Missing-reference editor behavior and tileset changes remain unknown. |
| Observed `mode` values are integers 0 or 1, `tilesetNames` has nine strings, and `flags` has 8,192 integers. | Shape audit | Observed syntax only | High in the audited scope | Numeric mode mapping, slot meaning, flag encoding, allowed lengths, and mutation rules remain unestablished. |
| Tilewright's tileset catalog matches an independent direct extraction of every bounded record and CLI-envelope field. | `MZ-1.10.0-TILESET-DIFFERENTIAL-2026-08-09` | Observed | High across all four projects and 24 records | This does not test editor mutation, malformed input, adjacent fields, or later versions. |

## Aggregate shape audit

On 2026-08-09, a read-only aggregate audit inspected exact
`data/Tilesets.json` and 196 map documents from the four authorized projects
recorded under `MZ-1.10.0-FRESH-4-2026-08-01`. It verified source containment
and absence of source symlinks before using `jq` 1.8.2.

The audit queried only decoded property names, JSON kinds, counts, integer
relationships and ranges, array lengths, and cross-file identifier existence.
It emitted no tileset names, notes, asset names, flags, map paths, project paths,
raw documents, excerpts, hashes, or per-project manifests.

The audit observed:

- four seven-entry arrays, each with one null slot and six object records;
- 24 records with one shared key set: `flags`, `id`, `mode`, `name`, `note`, and
  `tilesetNames`;
- positive integer IDs 1 through 6, all equal to their array indexes;
- nonempty string names and empty string notes in all 24 records;
- integer modes using two distinct values, without establishing their mapping;
- nine-string `tilesetNames` arrays and 8,192-integer `flags` arrays; and
- 196 positive map references using IDs 1 through 4, all resolving to records
  in their containing projects.

These are observations of MZ-generated states, not editor validation rules.

## Differential projection audit

On 2026-08-09, the tileset catalog and its JSON CLI adapter were run read-only
against the same four authorized projects. An independent `jq` extraction
compared every record ID and decoded name plus the schema, snapshot completeness,
diagnostic, and record-count envelope.

All four projects, all 24 records, and all 72 individual comparisons matched
exactly. Every snapshot was complete and every report had zero diagnostics. The
audit emitted only aggregate counts and booleans.

This verifies one implementation against independently decoded MZ-generated
data. It does not establish mode, image-slot, flag, note, lifecycle, editor
validation, mutation, persistence, or later-version behavior. No project path,
tileset name, note, asset name, flag, raw document, excerpt, field value, report,
hash, or per-project manifest was retained.

## Official documentation

The official *Tileset Settings* help page describes tilesets as the tile-image
and behavior configurations assigned to maps. It documents an editor-only Name
separately from Mode, Images, tile settings, and Notes. The official *Map
Properties* help page documents selection of a tileset for a map.

- *Tileset Settings*, RPG Maker MZ Help,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_08_10.html>, accessed
  2026-08-09.
- *Map Properties*, RPG Maker MZ Help,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_07_03.html>, accessed
  2026-08-09.

The help documents concepts, not JSON property names or exact serialization.

## Accepted bounded typed contract

The experimental slice should accept an existing `ProjectSnapshot` and project
exact `data/Tilesets.json` into positive catalog-scoped IDs and decoded names,
ordered by ID. It should accept null holes, require each non-null record's ID to
equal its array index, and refuse missing, duplicate, wrong-kind, malformed, or
unsupported required values.

The operation should not require or interpret `mode`, `tilesetNames`, `flags`,
`note`, or unknown fields. Those values and exact bytes remain only in the raw
snapshot. A `TilesetId` is an experimental tileset-catalog identifier, not yet
a stable project-wide resource identity.

Map-to-tileset reference validation remains a separate operation. The catalog
introduces no mutation, serialization, persistence, asset validation, runtime,
or editor-compatibility behavior.

## Fixture implications

Tests can generate minimal strict JSON in temporary project trees. The matrix
should include null holes, ID ordering, decoded names, unknown fields, missing
and unavailable documents, non-array roots, non-object entries, every required
field ambiguity, integer and index boundaries, ID/index mismatch, raw-byte
preservation, and absence of mutation.

Generated fixtures are Tilewright-owned test inputs only and do not establish
that MZ accepts or rejects malformed states.

## Remaining unknowns and next experiments

- Change only one tileset name and confirm the exact persisted field after save
  and reopen.
- Toggle one tileset mode and establish the numeric mapping without inferring it
  from stock names.
- Change one map's selected tileset and confirm the cross-file identifier
  relationship after save and reopen.
- Audit tileset record creation, deletion, maximum changes, and hole reuse only
  when those behaviors are needed.
- Investigate image slots and flags as separate capabilities before typing them.
- Observe another named MZ version at or above 1.10.0 before generalizing the
  contract.
