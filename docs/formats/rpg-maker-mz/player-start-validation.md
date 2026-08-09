# RPG Maker MZ player-start validation contract

This document defines the evidence boundary for Tilewright's first contextual
project validation. It is a research result and experimental contract, not a
general project-validity or compatibility claim.

## Question and scope

Can Tilewright use the bounded system summary, map catalog, and selected-map
summary to report useful player-start relationships without guessing editor
acceptance rules or exposing a broad validation framework prematurely?

Direct observations cover one disposable copy of a user-created RPG Maker MZ
1.10.0 project on the recorded macOS environment. A separate read-only
differential audit covers four authorized MZ 1.10.0 projects. Official
documentation describes deletion and the runtime consequence of an unset
player start. Later versions, converted projects, malformed editor behavior,
vehicle starts, passability, event placement, and write behavior remain outside
this contract.

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| The player start can be moved by selecting a map position, and the editor shows the selected location. | `MZ-HELP-SYSTEM1-2026-08-07` | Documented | High for the editor concept | The help does not name serialized fields or numeric bounds. |
| The player-start icon can be deleted, and the game cannot start without a player starting position. | `MZ-HELP-SYSTEM1-2026-08-07` | Documented | High for the documented editor and runtime behavior | The help does not specify serialized unset values. |
| Moving the player start within the same map from displayed coordinate `(8, 6)` to `(2, 3)` changed `startX`, `startY`, and save bookkeeping, while `startMapId` remained fixed; reopening showed `(2, 3)`. | `MZ-1.10.0-PLAYER-START-RELOCATION-2026-08-09` | Observed | High for this controlled action and version | Cross-map relocation, boundary positions, and later versions remain unobserved. |
| MZ 1.10.0 presents the exact stored triplet `startMapId = 0`, `startX = 0`, `startY = 0` as `None`, preserves it across Save, and presents `None` again after reopen. | `MZ-1.10.0-PLAYER-START-ZERO-TRIPLET-2026-08-09` | Observed tolerance and preservation | High for recognizing and preserving the exact triplet | The triplet was prepared outside the editor because automation could not activate the main map canvas; this does not prove the Delete gesture emits it. |
| A positive `startMapId` absent from a coherent `MapInfos.json` catalog is an unresolved cross-file reference. | Existing typed contracts | Inferred | High as an internal relationship finding | Editor acceptance and repair behavior remain unknown. |
| A nonnegative coordinate at or beyond a selected map's positive width or height is outside that map's zero-origin rectangular index range. | Relocation observation plus typed map dimensions | Inferred | High as arithmetic contextual validation | Editor acceptance, clamping, passability, and runtime behavior remain unknown. |
| A zero map ID with nonzero coordinates has established set or unset semantics. | Incomplete mixed-zero experiment | Unknown | No conclusion | The Mac locked before the editor could open the prepared state. |
| Tilewright's merged player-start validator matches an independent reconstruction of the bounded relationship across the four-project MZ 1.10.0 corpus. | `MZ-1.10.0-PLAYER-START-DIFFERENTIAL-2026-08-09` | Observed implementation behavior | High for the exact implementation and corpus | All observed source states were finding-free; synthetic tests cover negative categories, while editor behavior for those states remains unknown. |

## Controlled editor observations

### `MZ-1.10.0-PLAYER-START-RELOCATION-2026-08-09`

- **Kind:** Controlled editor experiment.
- **Version/environment:** RPG Maker MZ 1.10.0 on arm64 macOS 26.6 build
  25G72; disposable copy of a user-created Basic project.
- **Procedure:** Copied one immutable authorized source into a unique ignored
  research workspace. In Database → System 1, opened the player starting
  position selector, changed the displayed location from map 1 coordinate
  `(8, 6)` to `(2, 3)`, accepted, saved, quit, and reopened. Compared the
  complete project copy with the baseline using derived property-level results.
- **Observed:** Only `data/System.json` differed in file content. Its changed
  decoded root properties were `startX`, `startY`, and `versionId`;
  `startMapId` and the complete root property set remained fixed. After reopen,
  the editor displayed map 1 coordinate `(2, 3)`.
- **Limits:** This isolates same-map relocation, not cross-map identity,
  coordinate limits, the `versionId` generation rule, or write atomicity.
- **Redistribution:** No project file, value unrelated to the controlled
  position, screenshot, raw diff, asset, or proprietary content is retained.

### `MZ-1.10.0-PLAYER-START-ZERO-TRIPLET-2026-08-09`

- **Kind:** Controlled malformed/tolerance experiment.
- **Version/environment:** The same editor, platform, and disposable project.
- **Procedure:** With the editor closed, changed only `startMapId`, `startX`,
  and `startY` to zero in the owned copy. Opened Database → System 1, observed
  the displayed player start, saved without changing it, compared the input and
  saved state, then quit and reopened to observe the setting again.
- **Observed:** The editor displayed `None` before Save and after reopen. Save
  preserved all three zero values; the only decoded semantic property changed
  by that save was `versionId`, and the root property set remained fixed.
- **Limits:** This directly establishes recognition and preservation of the
  exact zero triplet, not that editor deletion generates it. A Playtest attempt
  was confounded by an unrelated NW.js profile warning and supplies no runtime
  evidence. The documented cannot-start consequence remains the authority.
- **Redistribution:** Only derived property names, equality results, controlled
  values, and editor presentation are retained.

## Differential implementation audit

### `MZ-1.10.0-PLAYER-START-DIFFERENTIAL-2026-08-09`

- **Kind:** Read-only differential implementation audit.
- **Version/environment:** Tilewright commit `01d2c2c`; four authorized,
  user-owned MZ 1.10.0 projects; `jq` 1.8.2 on arm64 macOS 26.6 build 25G72.
- **Procedure:** Built the merged CLI with Cargo's locked dependency graph. For
  each source, invoked `tilewright validate --format json` and independently
  reconstructed the bounded result from `System.json`, the indexed
  `MapInfos.json` record, and selected map dimensions. Compared schema version,
  scope, snapshot completeness, diagnostic count, three stored scalars,
  finding-free state, and ordered finding categories. Verified that no source
  tree contained a symbolic link before reading it.
- **Observed:** All four cases matched all nine comparisons, for 36 of 36 total
  matches. Every CLI snapshot was complete with no diagnostics, and every
  independently reconstructed result was finding-free.
- **Limits:** The corpus does not exercise the four finding categories. Their
  implementation behavior is covered by generated synthetic tests, not editor
  acceptance evidence. This audit does not establish later-version behavior,
  general project validity, runtime success, mutation, or persistence.
- **Redistribution:** The procedure emitted only anonymized case numbers,
  pass/fail booleans, and aggregate counts. No project path, title, map ID,
  coordinate, dimension, raw document, excerpt, digest, or CLI report is
  retained.

## Bounded validation contract

The experimental operation accepts an existing `ProjectSnapshot` and composes
the existing system-summary, map-catalog, and selected-map projections. It:

- reports the exact zero triplet as a missing player start;
- reports a zero map ID with nonzero coordinates as unevidenced, without
  deciding whether it is set or unset;
- reports a positive map ID missing from a coherent catalog;
- checks a catalog-selected map using `x < width` and `y < height`; and
- preserves all raw documents and unknown fields without filesystem I/O.

An unavailable structural projection is an operation error, not a contextual
finding. Findings have no public severity and do not generally claim editor
rejection. The missing-player-start message may state the documented
cannot-start consequence. A finding-free report means only that this bounded
check found no issue; it is not a project-validity or compatibility result.

## Fixture implications

Tests generate minimal strict JSON in temporary project trees. No vendor
project is needed. The matrix covers the exact unset triplet, a zero map ID
with nonzero coordinates, a missing positive catalog record, each rectangular
boundary, positive in-bounds coordinates, structural projection errors,
raw-byte preservation, deterministic findings, and separate snapshot
diagnostics.

Generated values are Tilewright test inputs only. They do not establish that
RPG Maker MZ accepts or rejects the synthetic states.

## Remaining unknowns and next experiments

- Use the editor's Delete gesture on the player-start icon and compare the
  generated state with the observed zero triplet.
- Complete the zero-map/nonzero-coordinate tolerance experiment after the host
  is unlocked.
- Test exact coordinate boundaries and dangling positive map IDs in a
  disposable copy before making editor-rejection claims.
- Repeat the relocation and unset observations on another named MZ version at
  or above 1.10.0 before broadening compatibility scope.
- Keep vehicle starts, passability, map transfer events, and broader diagnostic
  severity design in separate investigations.
