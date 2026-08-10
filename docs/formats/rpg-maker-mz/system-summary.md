# RPG Maker MZ system-summary contract

This document defines the evidence boundary for a small read-only projection of
`data/System.json`. It is a research result and implemented experimental
contract, not a support claim.

## Question and scope

What is the smallest system summary that gives callers useful project-level
orientation without interpreting the complete database, validating references,
or treating editor state as stable project identity?

Direct observations cover four user-owned projects created by RPG Maker MZ
1.10.0 plus controlled player-start experiments in disposable copies on the
recorded macOS environment. Official documentation describes the roles of the
selected settings. Later versions, converted projects, runtime behavior,
general malformed-input behavior, and plugin-defined extensions remain outside
the established scope.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| `System.json` stores system data including initial game settings. | `MZ-HELP-SYSTEM1-2026-08-07` and `MZ-SCRIPTREF-DB-1.0.0` | Documented | High for the official role and filename | Editor-requiredness and other-version behavior remain unknown. |
| `gameTitle`, `currencyUnit`, and `locale` identify the game title, currency unit, and language setting. | Official System 1 help and script reference | Documented | High for the named roles | Allowed string contents, locale grammar, normalization, and editor validation are unknown. |
| `editMapId` is the map being edited; `startMapId`, `startX`, and `startY` describe the player's initial position. | `MZ-SCRIPTREF-DB-1.0.0`; controlled map lifecycle and player-start records; official System 1 help | Documented and Observed | High for the field roles in the observed MZ 1.10.0 scope | Malformed references, numeric limits, and other versions remain unknown. |
| MZ 1.10.0 presents and preserves the exact `startMapId = 0`, `startX = 0`, `startY = 0` triplet as `None`, and the editor's Delete gesture generated that triplet in a separate copy. | `MZ-1.10.0-PLAYER-START-ZERO-TRIPLET-2026-08-09` and `MZ-1.10.0-PLAYER-START-DELETE-2026-08-09` | Observed generation, tolerance, and preservation | High for these controlled actions and version | Runtime behavior and later versions remain unobserved. |
| MZ 1.10.0 saved prepared mixed-zero, exact-boundary, dangling-map, negative-coordinate, and upper-out-of-bounds player-start states without normalizing the three scalars. | `MZ-1.10.0-PLAYER-START-TOLERANCE-MATRIX-2026-08-09` | Observed persistence tolerance | High for the five exact states and version | Save tolerance does not establish validity, runtime behavior, UI meaning, or broader numeric limits. |
| All four audited documents have object roots and one shared set of 58 decoded top-level property names. | `MZ-1.10.0-SYSTEM-SUMMARY-SHAPE-AUDIT-2026-08-07` | Observed | High across the four fresh projects | Duplicate decoded properties were not established absent; converted, plugin-extended, and later-version shapes may differ. |
| Every selected string field is present and string-valued; every selected numeric field is present and integer-valued. | Shape audit | Observed syntax | High across the four fresh projects | Requiredness and editor behavior for missing, duplicate, alternate-kind, or alternate numeric forms remain unknown. |
| Observed edit and start map IDs are positive and resolve to map-catalog records; observed start coordinates are nonnegative and within the referenced map dimensions. | Shape and cross-file audit | Observed | High across all four projects | This is not evidence that the editor rejects zero, missing, dangling, or out-of-bounds values. |
| `versionId` changes during several otherwise unrelated saves. | Existing controlled save and map lifecycle records | Observed | High for those workflows | Its generation rule and stable meaning are unknown, so it is excluded from the summary. |
| Tilewright's merged system-summary implementation matches an independent projection of all seven bounded fields across the four-project corpus. | `MZ-1.10.0-SYSTEM-SUMMARY-DIFFERENTIAL-2026-08-07` | Observed implementation behavior | High for the exact implementation and corpus | Later versions, converted projects, malformed inputs, and editor acceptance remain untested. |
| Tilewright's signed-coordinate implementation reproduces all six retained controlled triplets and emits schema version 2. | `MZ-1.10.0-SIGNED-PLAYER-START-DIFFERENTIAL-2026-08-09` | Observed implementation behavior | High for the exact implementation and controlled cases | This does not broaden editor-validity, runtime, or version claims. |

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

## Differential implementation audit

On 2026-08-07, Tilewright commit `f1742a0` was built with Cargo's locked
dependency graph. For each of the same four authorized MZ 1.10.0 projects, the
audit invoked `tilewright system --format json` and separately used `jq` 1.8.2
to project the seven source fields directly from `data/System.json`.

All 28 field comparisons matched: three decoded strings and four nonnegative
integer scalars in each of four projects. All four CLI results also reported
schema version 1, exact `data/System.json` identity, complete snapshots, and no
snapshot diagnostics. The procedure emitted only anonymized pass/fail booleans
and aggregate counts; it retained no projected value or project path.

This verifies the merged implementation against this local corpus. It does not
establish behavior for malformed or unavailable documents beyond synthetic
tests, human-output presentation beyond adapter tests, editor acceptance,
converted projects, later MZ versions, or broader system semantics.

## Signed-coordinate differential audit

On 2026-08-09, the signed-coordinate implementation based on `13bec1b` was run
read-only against the retained Delete case and five-case tolerance matrix. A
separate `jq` projection supplied each stored triplet. Tilewright reproduced all
six triplets exactly, including `-1, -1`, and every `system` and `validate` JSON
report emitted schema version 2. All six expected finding-category arrays also
matched. Only per-case pass/fail results and the aggregate `6/6` result were
retained; no project content or path was retained in tracked material.

This verifies the correction against the controlled evidence. It does not
establish semantic validity, runtime behavior, broader numeric limits, or
later-version compatibility.

## Controlled player-start tolerance observations

### `MZ-1.10.0-PLAYER-START-DELETE-2026-08-09`

- **Kind:** Controlled editor action.
- **Version/environment:** RPG Maker MZ 1.10.0 on arm64 macOS 26.6 build
  25G72; unique disposable copy of the authorized Basic project.
- **Procedure:** Deleted the player-start marker through the editor, saved,
  closed, and inspected only derived property-level differences against the
  immutable baseline.
- **Observed:** The saved triplet was exactly `startMapId = 0`, `startX = 0`,
  and `startY = 0`. Those three properties and unrelated `versionId` were the
  only decoded `System.json` values that differed from the baseline.
- **Limits:** This establishes one editor-generated unset representation, not
  runtime behavior, other versions, or every deletion context.
- **Redistribution:** Only the controlled values, changed property names, and
  equality results are retained.

### `MZ-1.10.0-PLAYER-START-TOLERANCE-MATRIX-2026-08-09`

- **Kind:** Controlled malformed/tolerance matrix.
- **Version/environment:** The same editor and platform; five separately copied
  disposable projects with a `17 x 13` selected map.
- **Procedure:** With the editor closed, prepared one triplet per copy: mixed
  zero `(0, 8, 6)`, exact upper in-bounds `(1, 16, 12)`, dangling positive map
  `(999, 8, 6)`, negative coordinates `(1, -1, -1)`, and exact upper
  out-of-bounds `(1, 17, 13)`. Opened and saved each project in MZ, then
  compared only the selected scalars, decoded changed-property names, and root
  key equality with its baseline.
- **Observed:** Every save retained the prepared triplet exactly. The changed
  properties were limited to the prepared scalar or scalars plus unrelated
  `versionId`; the decoded root property set remained fixed. In particular,
  MZ saved signed negative `startX` and `startY` values.
- **Limits:** Persistence tolerance is not evidence of editor validity, intended
  UI meaning, runtime success, passability, clamping rules, or numeric bounds
  beyond these exact values. No playtest was performed.
- **Redistribution:** Only the controlled triplets, map dimensions, changed
  property names, and equality results are retained.

## Bounded typed contract

The experimental slice accepts an existing `ProjectSnapshot` and reads only
exact `data/System.json`. It exposes:

- the exact project-relative document path;
- decoded game-title, currency-unit, and locale strings;
- the nonnegative editor-map ID scalar;
- the nonnegative player-start map ID scalar; and
- the signed player-start X and Y scalars.

The projection refuses an absent or unavailable document, a non-object root,
and missing, duplicate, wrong-kind, undecodable, fractional, exponent-form, or
field-range-exceeding required values. Strings remain unnormalized and may be
empty. Numeric map fields remain `u32` scalars rather than catalog-scoped
`MapId` values, while coordinates use the `i64` representation bound accepted
in [ADR 0014](../../decisions/0014-signed-player-start-coordinates.md). These
types report stored values without contextual validation.

Unknown properties and all exact source bytes remain in the untouched raw
snapshot. The operation does not parse party members or other system settings,
interpret `versionId`, require a map catalog, validate map references or
coordinates, compare the title with `package.json` or `index.html`, infer an MZ
version, or expose mutation and serialization.

## Fixture implications

Tests can generate minimal strict JSON in temporary project trees. No vendor
project is needed. The fixture matrix should cover:

- all seven selected fields with unsigned map-ID and signed-coordinate
  boundaries;
- empty, escaped, Unicode, and ordinary strings;
- unknown top-level and nested fields retained in the raw document;
- missing, duplicate decoded, wrong-kind, fractional, exponent-form, and
  field-specific overflow values;
- missing, unavailable, and non-file `System.json` cases; and
- proof that party members, `versionId`, map catalogs, and unrelated settings
  are not required or interpreted.

Generated fixture values are Tilewright test inputs only. They do not establish
that MZ accepts or rejects malformed projects.

## Remaining unknowns and next experiments

- Establish the editor presentation and runtime consequences of mixed-zero,
  dangling, negative, and upper-out-of-bounds states separately before assigning
  validity semantics.
- Change locale and currency independently before describing their accepted
  grammars or cross-file effects.
- Determine whether `editMapId` zero or a dangling ID is an editor-produced or
  tolerated authoring state.
- Observe another named MZ version at or above 1.10.0 before generalizing the
  contract.
