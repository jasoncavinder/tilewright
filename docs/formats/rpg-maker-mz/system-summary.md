# RPG Maker MZ system-summary contract

This document defines the evidence boundary for a small read-only projection of
`data/System.json`. It is a research result and implemented experimental
contract, not a support claim.

## Question and scope

What is the smallest system summary that gives callers useful project-level
orientation without interpreting the complete database, validating references,
or treating editor state as stable project identity?

Direct observations cover four user-owned projects created by RPG Maker MZ
1.10.0 on the recorded macOS environment. Official documentation describes the
roles of the selected settings. Later versions, converted projects, malformed
input, unset starting-position serialization, and plugin-defined extensions
remain outside the established scope.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| `System.json` stores system data including initial game settings. | `MZ-HELP-SYSTEM1-2026-08-07` and `MZ-SCRIPTREF-DB-1.0.0` | Documented | High for the official role and filename | Editor-requiredness and other-version behavior remain unknown. |
| `gameTitle`, `currencyUnit`, and `locale` identify the game title, currency unit, and language setting. | Official System 1 help and script reference | Documented | High for the named roles | Allowed string contents, locale grammar, normalization, and editor validation are unknown. |
| `editMapId` is the map being edited; `startMapId`, `startX`, and `startY` describe the player's initial position. | `MZ-SCRIPTREF-DB-1.0.0`; controlled map lifecycle records; official System 1 help | Documented and Observed | High for the field roles in the observed MZ 1.10.0 scope | The serialized unset-start state, malformed references, numeric limits, and other versions remain unknown. |
| All four audited documents have object roots and one shared set of 58 decoded top-level property names. | `MZ-1.10.0-SYSTEM-SUMMARY-SHAPE-AUDIT-2026-08-07` | Observed | High across the four fresh projects | Duplicate decoded properties were not established absent; converted, plugin-extended, and later-version shapes may differ. |
| Every selected string field is present and string-valued; every selected numeric field is present and integer-valued. | Shape audit | Observed syntax | High across the four fresh projects | Requiredness and editor behavior for missing, duplicate, alternate-kind, or alternate numeric forms remain unknown. |
| Observed edit and start map IDs are positive and resolve to map-catalog records; observed start coordinates are nonnegative and within the referenced map dimensions. | Shape and cross-file audit | Observed | High across all four projects | This is not evidence that the editor rejects zero, missing, dangling, or out-of-bounds values. |
| `versionId` changes during several otherwise unrelated saves. | Existing controlled save and map lifecycle records | Observed | High for those workflows | Its generation rule and stable meaning are unknown, so it is excluded from the summary. |

## Aggregate shape audit

On 2026-08-07, a read-only aggregate audit inspected the four authorized
`System.json` documents recorded by `MZ-1.10.0-FRESH-4-2026-08-01`. The files
were regular files between 6,544 and 6,636 bytes, and the source trees contained
no symbolic links. The audit used `jq` 1.8.2 and emitted only decoded property
names, JSON kinds, counts, integer relationships, ranges, and boolean
cross-file results.

The audit observed:

- four object roots with one shared set of 58 decoded top-level names;
- string-valued `gameTitle`, `currencyUnit`, and `locale` in every document;
- integer-valued `editMapId`, `startMapId`, `startX`, and `startY` in every
  document;
- positive edit and start map IDs resolving to `MapInfos.json` records in every
  project; and
- nonnegative player-start coordinates within the referenced map's positive
  width and height in every project.

No title, currency text, locale value, path, raw document, excerpt, digest, or
per-project manifest was retained. These observations describe MZ-generated
states; they do not establish editor validation rules.

## Bounded typed contract

The experimental slice accepts an existing `ProjectSnapshot` and reads only
exact `data/System.json`. It exposes:

- the exact project-relative document path;
- decoded game-title, currency-unit, and locale strings;
- the nonnegative editor-map ID scalar; and
- the nonnegative player-start map ID, X, and Y scalars.

The projection refuses an absent or unavailable document, a non-object root,
and missing, duplicate, wrong-kind, undecodable, negative, fractional, or
out-of-`u32` required values. Strings remain unnormalized and may be empty.
Numeric map fields remain `u32` scalars rather than catalog-scoped `MapId`
values because zero and unset-state behavior have not been observed directly.

Unknown properties and all exact source bytes remain in the untouched raw
snapshot. The operation does not parse party members or other system settings,
interpret `versionId`, require a map catalog, validate map references or
coordinates, compare the title with `package.json` or `index.html`, infer an MZ
version, or expose mutation and serialization.

## Fixture implications

Tests can generate minimal strict JSON in temporary project trees. No vendor
project is needed. The fixture matrix should cover:

- all seven selected fields with zero and positive numeric boundaries;
- empty, escaped, Unicode, and ordinary strings;
- unknown top-level and nested fields retained in the raw document;
- missing, duplicate decoded, wrong-kind, negative, fractional, and overflow
  values for every required field family;
- missing, unavailable, and non-file `System.json` cases; and
- proof that party members, `versionId`, map catalogs, and unrelated settings
  are not required or interpreted.

Generated fixture values are Tilewright test inputs only. They do not establish
that MZ accepts or rejects malformed projects.

## Remaining unknowns and next experiments

- Delete the player's starting position in a disposable MZ 1.10.0 project,
  save, reopen, and observe the exact `startMapId`/X/Y representation.
- Change only the player starting position and confirm its persisted fields and
  map-coordinate relationship.
- Change locale and currency independently before describing their accepted
  grammars or cross-file effects.
- Determine whether `editMapId` zero or a dangling ID is an editor-produced or
  tolerated authoring state.
- Observe another named MZ version at or above 1.10.0 before generalizing the
  contract.
