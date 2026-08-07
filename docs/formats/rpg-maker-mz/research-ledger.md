# RPG Maker MZ research ledger

This ledger indexes active and completed format investigations. Product intent
is not evidence of proprietary format behavior.

Maintainer-led research targets RPG Maker MZ 1.10.0 and newer under
[ADR 0002](../../decisions/0002-rpg-maker-mz-version-floor.md). Evidence remains
version-specific: the target range does not convert a 1.10.0 observation into a
claim about every later release. Older-version contributions may add separate
records and compatibility scope.

## Investigation index

| ID | Question | Status | Primary classification | Last updated |
| --- | --- | --- | --- | --- |
| [`mz-project-detection-001`](#mz-project-detection-001-what-minimum-evidence-identifies-an-rpg-maker-mz-project-root) | What minimum evidence identifies an RPG Maker MZ project root? | Active | Documented and observed for MZ 1.10.0; later target versions unresolved | 2026-08-01 |
| [`mz-project-layout-001`](#mz-project-layout-001-what-high-level-project-layout-and-file-roles-are-established) | What high-level project layout and file roles are established? | Active | Documented and observed for four MZ 1.10.0 templates | 2026-08-03 |
| [`mz-map-catalog-001`](#mz-map-catalog-001-what-is-the-smallest-evidenced-typed-map-catalog) | What is the smallest evidenced typed map catalog? | Active | Documented, observed, and inferred for MZ 1.10.0 | 2026-08-06 |
| [`mz-map-summary-001`](#mz-map-summary-001-what-is-the-smallest-useful-selected-map-summary) | What is the smallest useful selected-map summary? | Active | Documented, observed, and inferred for MZ 1.10.0 | 2026-08-06 |

The current synthesis and proposed read-only contract are in
[`project-layout.md`](project-layout.md); breadth and remaining gaps are in the
[`project-layout coverage matrix`](project-layout-coverage.md). Both
investigations remain active while required behaviors, later target versions,
and deployment targets lack direct observations.

## `mz-project-detection-001`: What minimum evidence identifies an RPG Maker MZ project root?

- **Status:** Active
- **Behavior depending on this:** Read-only project-root recognition and
  discovery diagnostics.
- **Scope:** RPG Maker MZ 1.10.0+ authoring-project roots; direct observations
  currently cover 1.10.0, supplemented by official online material and the
  version 1.0.0 script-reference suite. No claim about damaged projects,
  converted projects, deployment packages, or later target versions.
- **Last updated:** 2026-08-01

### Evidence ledger

| Claim | Evidence | Classification | Confidence and rationale | Unresolved alternatives |
| --- | --- | --- | --- | --- |
| The editor's Open Project flow selects a file presented as `Game` or `game.rmmzproject` from inside the project folder; four fresh MZ 1.10.0 projects contain exact lowercase `game.rmmzproject`. | `MZ-HELP-OPEN-2026-08-01`, `MZ-HELP-START-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for the marker's role and exact MZ 1.10.0 spelling because official procedure and four independent New Game outputs agree. | On-disk spelling in other versions and editor behavior after case changes remain unobserved. |
| The directory containing that selected marker is the project folder used by the editor. | `MZ-HELP-OPEN-2026-08-01`, `MZ-HELP-START-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for the documented workflow and observed MZ 1.10.0 roots. | Symlinked markers, aliases, nested roots, and converted/damaged projects are not addressed. |
| A project is managed and backed up as the whole contents of its chosen folder. | `MZ-HELP-START-2026-08-01` | Documented | High for the documented workflow. | It does not prove that every child is required or editor-authored. |
| The marker alone proves that every required project file is present and compatible. | No supporting evidence found. | Unknown | No confidence: the official help describes how to open a project, not validation or compatibility guarantees. | The marker may be empty, copied, stale, or accompanied by incomplete/version-incompatible data. |
| `data/System.json`, `data/MapInfos.json`, or standard directories are mandatory co-signals for recognizing a root. | `MZ-CONVERT-2026-08-01` documents their locations; `MZ-1.10.0-FRESH-4-2026-08-01` observes them in every template but does not test deletion. | Unknown | Medium as corroborating inventory, but requiredness remains untested. | The editor may reconstruct or tolerate missing files; plugins and versions may alter the inventory. |
| The MZ 1.10.0 marker contains the exact 12 ASCII bytes `RPGMZ 1.10.0` with no final newline. | `MZ-1.10.0-FRESH-4-2026-08-01` | Observed | High for the four observed templates because byte content and hashes are identical. | Other versions may use the same grammar, a different string, or different encoding; the editor may or may not validate the contents. |
| RPG Maker MZ 1.10.0 rejects a zero-byte regular `game.rmmzproject` and does not open the otherwise unchanged Basic project. | `MZ-1.10.0-EMPTY-MARKER-2026-08-01` | Observed | High for this exact malformed form: the experiment changes only marker contents, filesystem inspection confirms zero bytes, and the editor reports that it cannot read the marker. | Nonempty malformed contents, another version string, added whitespace, encoding changes, and other editor versions remain untested. |
| RPG Maker MZ 1.10.0 opens the otherwise unchanged Basic project when the marker is changed to same-length `RPGMZ 1.10.1`, and opening does not rewrite the marker. | `MZ-1.10.0-ALTERED-VERSION-MARKER-2026-08-01` | Observed | High for this exact alternate value: only the final byte differs from the generated form, the editor opens without error, and read-only inspection confirms the value remains unchanged afterward. | The editor may parse a version-shaped suffix, ignore some or all suffix bytes, or apply rules not distinguished by this experiment. |
| RPG Maker MZ 1.10.0 opens an otherwise unchanged Basic project whose marker is renamed on disk to capital-G `Game.rmmzproject`, without warning or rewriting it. | `MZ-1.10.0-CAPITAL-G-MARKER-2026-08-01` | Observed | High for this environment and spelling because the one-change result and post-open filesystem inspection agree. | Lowercase lookup resolves the capital-G entry on the observed volume, so editor case handling is not isolated from case-insensitive filesystem behavior. |
| Marker contents provide a stable cross-version version contract suitable for general detection. | Only one editor version has been observed. | Unknown | Low: the 1.10.0 value is promising but cannot establish a grammar or compatibility policy. | Controlled observations of later target versions and altered marker contents are required; pre-1.10.0 coverage is outside maintainer-led scope. |
| The default-option MZ 1.10.0 Basic Web deployment omits `game.rmmzproject` while retaining the authoring root's runtime-shaped files. | `MZ-HELP-DEPLOY-2026-08-01`, `MZ-1.10.0-BASIC-WEB-DEPLOY-2026-08-01` | Documented and Observed | High for this target and configuration: the deployment action and complete path/content comparison agree. | Other targets, deployment options, projects, and editor versions may include or transform different files. |
| Every deployment package can be distinguished from every authoring project solely by marker absence. | Only one target/configuration has been observed. | Unknown | Low: the Web observation supports a narrow negative test but not a universal deployment rule. | A copied marker, another target, or another editor version could invalidate the generalization. |
| A read-only detector may safely report an exact lowercase marker-bearing directory as an MZ **candidate** without claiming it is complete, parseable, compatible, or supported. | Inference from `MZ-HELP-OPEN-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01`, and Tilewright's compatibility vocabulary. | Inferred | High for a candidate-level result: documented behavior and direct observation agree while the status remains bounded. | False positives remain possible when a marker is copied or renamed. |

### Evidence records

#### `MZ-HELP-OPEN-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “Menu Bar Content” → “[File] Menu” → “Open Project,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_04.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** The Open Project procedure directs the user to select `Game` (or
  `game.rmmzproject`) inside the project folder.
- **Redistribution:** Only a short paraphrase and locator are committed.

#### `MZ-HELP-START-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “Getting Started Making Your Game” → “Creating a Project” and
  “Managing Projects,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_02.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** A project contains game data and assets, the editor reopens it by
  selecting `Game`/`game.rmmzproject`, and the documented backup unit is the
  entire chosen project folder.
- **Redistribution:** Only a short paraphrase and locator are committed.

#### `MZ-HELP-DEPLOY-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page; Windows, macOS, and browser targets are described.
- **Locator:** “Output Formats” → “Deployment,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_11_03.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Deployment exports a target-specific game folder or application;
  the page discusses optional exclusion of nested files under project `img`,
  `audio`, and `effects`, plus image/audio encryption that cannot be used inside
  the authoring project folder. It does not give a complete output inventory or
  encrypted filename contract.
- **Redistribution:** Only a short paraphrase and locator are committed.

#### `MZ-1.10.0-FRESH-4-2026-08-01`

- **Kind:** Observation
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  four user-created New Game projects labeled Basic, Tutorial, Map Set, and HD
  Layout; read-only inspection.
- **Locator:** Ignored local evidence directories
  `.local-research/sources/mz-1_10_0-fresh-basic/`,
  `.local-research/sources/mz-1_10_0-fresh-tutorial/`,
  `.local-research/sources/mz-1_10_0-fresh-map-set/`, and
  `.local-research/sources/mz-1_10_0-fresh-hd-layout/` in the primary integration
  checkout.
- **Accessed/observed:** 2026-08-01
- **Procedure:** Verified the canonical ignored root and absence of symlinks;
  compared sorted path/type/size manifests and non-content-revealing SHA-256
  digests; inspected marker bytes; checked selected media types; validated every
  `data/*.json` file with `jq`; scanned JSON byte properties; and compared
  `MapInfos.json` indices with map filenames. Later aggregate inspection counted
  extensions and compared basename-stem sets without inspecting asset or
  companion-file contents. No scripts or plugins were executed.
- **Summary:** All four roots share exact lowercase `game.rmmzproject`, identical
  marker bytes `RPGMZ 1.10.0`, the same 12 immediate root entries and 30 child
  directories, plural standard filenames, and `js/plugins.js`. Their shared
  runtime shell includes `index.html`, `package.json`, `css/game.css`,
  `js/main.js`, six `js/rmmz_*.js` files, and JavaScript/WebAssembly dependencies
  under `js/libs`. All 252 JSON files across the four projects parse as valid
  UTF-8 JSON without BOM, carriage returns, or final newlines. Observed map files
  use three digits; all non-null map-info IDs equal their array indices and have
  corresponding files through ID 189 in the Map Set project.
- **Redistribution:** The user explicitly authorized read-only research. No
  project files, vendor assets, runtime code, default data, or raw manifests are
  committed; only derived observations are recorded.

#### `MZ-1.10.0-CST-NOOP-2026-08-03`

- **Kind:** Controlled local representation experiment.
- **Question:** Does the proposed strict-gate-plus-CST representation reproduce
  the observed MZ 1.10.0 `data/*.json` corpus byte-for-byte without mutation?
- **Source/provenance:** Ignored workspace copies containing only the `data`
  directories from the same four user-owned fresh MZ 1.10.0 projects recorded
  by `MZ-1.10.0-FRESH-4-2026-08-01`.
- **Environment:** Tilewright `jsonc-parser-study` at the 2026-08-03 experiment
  branch, `jsonc-parser` 0.33.1, Rust 1.97.1, arm64 macOS 26.6.
- **Procedure:** Verified that the four selected source `data` directories had
  no symlinks, copied only those directories into a unique ignored workspace,
  and ran `cargo run --release -p jsonc-parser-study --example corpus_noop
  --locked -- <owned-workspace>/corpus`. The checker emits aggregate counts
  only and neither prints nor retains file paths or contents.
- **Observed:** All 252 JSON files were valid UTF-8 without a BOM, passed the
  prototype strict JSON gate, and serialized from the CST byte-identically.
  There were zero strict rejections, changed outputs, symlinks, or read errors.
- **Confidence:** High for no-op representation of these exact authorized MZ
  1.10.0 files with `jsonc-parser` 0.33.1. This is not a mutation, loader,
  cross-version, plugin-data, or editor-reopen claim.
- **Redistribution:** No source file, filename, path manifest, excerpt, digest,
  or per-file diagnostic is committed. Only this aggregate observation and the
  safe reproduction procedure are retained.

#### `MZ-1.10.0-BASIC-WEB-DEPLOY-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  user-created Basic project; Web Browsers deployment with unused-file exclusion
  and encryption disabled as requested.
- **Locator:** Source
  `.local-research/sources/mz-1_10_0-fresh-basic/`; deployment output
  `.local-research/mz-1_10_0-deploy-basic-web-default/mz-1_10_0-fresh-basic/`
  in the primary integration checkout.
- **Accessed/observed:** Deployed and inspected 2026-08-01.
- **Procedure:** The user performed the editor deployment. Read-only inspection
  verified ignored canonical paths and absence of symlinks, then compared sorted
  relative path manifests, file types, modes, sizes, and every common file byte
  for byte. The single changed JSON file was checked structurally without
  recording its proprietary runtime configuration value. No output was
  executed.
- **Summary:** The source has 1,312 files and 30 child directories; output has
  1,311 files and 29 child directories. Output omits only
  `game.rmmzproject` and the empty `movies/` directory. All output files retain
  their source-relative paths. Of 1,311 common files, 1,310 are byte-identical;
  `package.json` remains valid JSON with the same top-level keys and changes
  only the string-valued `chromium-args` field.
- **Redistribution:** No deployment file, runtime code, asset, raw manifest, or
  configuration value is committed; only derived observations are recorded.

#### `MZ-1.10.0-EMPTY-MARKER-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the user-created Basic project.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-marker-empty/` in the primary integration
  checkout.
- **Accessed/observed:** Changed and tested by the user, then inspected
  read-only, 2026-08-01.
- **Procedure:** The marker alone was truncated to zero bytes. The user selected
  it through the editor's Open Project flow. Read-only inspection verified the
  ignored canonical path, regular-file type, zero-byte size, absence of
  symlinks, identical non-marker path manifests, and byte-identical non-marker
  files.
- **Summary:** The editor displayed “Unable to read file game.rmmzproject.” and
  did not open the project. The editor did not establish any behavior for
  nonempty malformed marker forms in this experiment.
- **Redistribution:** Only the short diagnostic and derived observation are
  committed; no project files or vendor contents are included.

#### `MZ-1.10.0-ALTERED-VERSION-MARKER-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  the same disposable Basic-project copy used for the empty-marker experiment.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-marker-empty/` in the primary integration
  checkout.
- **Accessed/observed:** Changed and tested by the user, then inspected
  read-only, 2026-08-01.
- **Procedure:** The marker was changed from zero bytes to the 12 ASCII bytes
  `RPGMZ 1.10.1` without a final newline, differing from the generated baseline
  only in the final digit. The user selected it through Open Project and closed
  without saving. Read-only inspection afterward verified the exact bytes,
  regular-file type, and absence of a final newline.
- **Summary:** RPG Maker MZ 1.10.0 opened the project without an error and did
  not rewrite the marker. This disproves using equality with the running editor
  version as a prerequisite for candidate recognition, but it does not reveal
  the complete accepted marker grammar.
- **Redistribution:** Only the user-created marker variation and derived
  observation are described; no project or vendor contents are committed.

#### `MZ-1.10.0-CAPITAL-G-MARKER-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the user-created Basic project. On the containing volume,
  a lowercase path lookup resolves the capital-G entry.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-marker-case-capital-g/` in the primary integration
  checkout.
- **Accessed/observed:** Changed and tested by the user, then inspected
  read-only, 2026-08-01.
- **Procedure:** Starting from the generated marker contents, only the on-disk
  filename changed from `game.rmmzproject` to `Game.rmmzproject` through a
  two-step rename. The user selected it through Open Project and closed without
  saving. Read-only inspection verified the exact capital-G spelling, normal
  12-byte contents, no final newline, absence of symlinks, and byte-identical
  non-marker files. A lowercase lookup was also observed to resolve the entry.
- **Summary:** MZ 1.10.0 opened the project without warning or error and did not
  rewrite the filename. Because the filesystem resolves a lowercase lookup to
  the capital-G entry, the result does not establish behavior on a case-sensitive
  filesystem.
- **Redistribution:** Only the path variation and derived observation are
  committed; no project or vendor contents are included.

### Established findings

- **Documented and observed:** The direct root-selection signal is
  `game.rmmzproject`; exact lowercase spelling is established for the four MZ
  1.10.0 New Game projects.
- **Documented:** The project is folder-scoped; its full contents are the unit
  the official help tells users to copy for backup.
- **Observed:** The MZ 1.10.0 marker form is the exact ASCII text
  `RPGMZ 1.10.0` without a final newline.
- **Observed:** MZ 1.10.0 rejects a zero-byte marker and does not open the
  otherwise unchanged project.
- **Observed:** MZ 1.10.0 accepts and preserves the same-length altered marker
  `RPGMZ 1.10.1`; exact equality with the running editor version is not required.
- **Observed:** MZ 1.10.0 accepts and preserves `Game.rmmzproject` on the
  observed volume, where lowercase lookup resolves that capital-G entry.
- **Observed:** The default-option MZ 1.10.0 Basic Web deployment omits the
  marker while retaining the project's runtime-shaped files.
- **Unknown:** Required companion files, the cross-version marker grammar, and
  validation of nonempty malformed marker contents remain unestablished.

### Inferences

- **Inferred, high confidence:** Exact lowercase marker presence can justify a bounded
  `candidate` result, but not `valid`, `compatible`, or `supported`.
- **Inferred, medium confidence:** Standard data files may be reported as
  corroborating inventory or missing-file warnings after recognition, but they
  should not yet decide identity.
- **Inferred, high confidence:** Marker version text is not trustworthy evidence
  of the running editor version or project compatibility because the user can
  alter it to an accepted unequal value.
- **Inferred, high confidence:** Discovery should enumerate and retain actual
  marker spelling rather than relying on a lowercase path lookup whose behavior
  varies with filesystem case sensitivity.

### Risks and unknowns

- **Unknown fields:** No additional marker fields exist in the observed 12-byte
  form, and the zero-byte form is rejected by 1.10.0. Which bytes or fields the
  editor validates remains unknown because `RPGMZ 1.10.1` is accepted unchanged;
  other versions may extend or change the form.
- **Ordering/encoding:** ASCII bytes and no final newline are observed for
  1.10.0. Filesystem enumeration order remains irrelevant to recognition.
- **Identifiers/references:** No identifier or cross-file reference is needed
  for candidate recognition; using one would overstate the evidence.
- **Version/plugin differences:** Exact 1.10.0 marker casing is observed, and
  the observed Web deployment omits it. Cross-version stability,
  case-sensitive-filesystem behavior, other altered-marker forms, and
  other-target deployment inclusion remain unknown. Plugins may add unrelated
  root entries.

### Next experiment

Repeat the capital-G marker experiment on a case-sensitive filesystem to
separate editor behavior from pathname lookup behavior. Restore the exact marker
before removing one possible companion at a time. Deploy the unchanged Basic
project to the other available targets and repeat the untouched-project manifest
on at least one other named MZ version at or above 1.10.0 before defining a
cross-version grammar for the maintained range.
Do not commit generated projects or vendor assets.

### Implementation implications

- **Safe now:** Implement a read-only candidate recognizer scoped to an explicit
  directory, recognizing exact lowercase `game.rmmzproject` and reporting a
  separately labeled case-variant candidate when enumeration finds another
  spelling. Return the exact path and an evidence-bounded status; optionally
  report the generated 1.10.0 bytes, editor-accepted altered-version form, or
  zero-byte rejected form. Treat marker version text as untrusted and handle
  ambiguity, non-regular entries, and symlinks explicitly.
- **Not justified:** Parsing marker contents, identifying an editor version,
  requiring standard JSON files for identity, recursing for projects, or
  calling a candidate valid/supported.
- **Fixtures/tests needed:** Synthetic temporary directories for marker present,
  absent, wrong extension, non-regular marker, ambiguous casing, deployment-like
  data without a marker, and extra unknown/plugin-created entries. Fixture names
  test discovery only and must not claim editor-generated contents.

## `mz-project-layout-001`: What high-level project layout and file roles are established?

- **Status:** Active
- **Behavior depending on this:** Read-only project inventory, classification of
  known versus unknown entries, and later loader scoping.
- **Scope:** High-level directories and standard database/map filenames named by
  official sources. JSON shapes, requiredness, serialization, assets, core
  scripts, save data, and deployment internals are out of scope except where
  needed to classify their roles.
- **Last updated:** 2026-08-03

### Implementation status

The core implements an **Experimental**, capability-relative inventory from this
evidence boundary. It reports exact project-relative native paths and entry
kinds, does not follow symlinks or read contents, recognizes only the evidenced
standard immediate-root and immediate `data` filename families, labels other
immediate `data/*.json` paths as extension candidates rather than proven plugin
content, and leaves everything else unknown. This implementation status adds no
new proprietary-format evidence or compatibility claim.

### Evidence ledger

| Claim | Evidence | Classification | Confidence and rationale | Unresolved alternatives |
| --- | --- | --- | --- | --- |
| MZ project data locations include `audio`, `img`, `icon`, `fonts`, `data`, `effects`, `css`, `movies`, and `js`. | `MZ-CONVERT-2026-08-01`, `MZ-HELP-ASSETS-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for MZ 1.10.0: all four roots share the same directory manifest. | Editor-requiredness and other-version layouts remain unknown. |
| `data/MapInfos.json` and three-digit `data/MapNNN.json` files hold map index and map/event data respectively in the observed projects. | `MZ-CONVERT-2026-08-01`, `MZ-SCRIPTREF-DB-1.0.0`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for MZ 1.10.0 syntax: 196 observed map files use three digits, including a contiguous range through 189, and each observed map-info ID has a corresponding file. | Width beyond ID 999, gaps after deletion, other versions, and editor enforcement remain unknown. |
| Creating one top-level map from the MZ 1.10.0 Basic project adds `data/Map002.json`, appends an index-2/ID-2 entry to `MapInfos.json`, and changes only `editMapId` and `versionId` in `System.json`. | `MZ-1.10.0-ADD-MAP-2026-08-01` | Observed | High for this action and starting state because the persisted map was confirmed after restart and a complete before/after comparison found only those three path/content changes. | Allocation after gaps, deletion/reuse, child maps, reordering, IDs above 999, other settings, and other versions remain unknown. |
| Deleting that trailing ID-2 map removes `data/Map002.json`, shortens `MapInfos.json` from three elements to two with no trailing hole, and changes only `editMapId` and `versionId` in `System.json`; the resulting map-info bytes equal the original Basic project. | `MZ-1.10.0-DELETE-TRAILING-MAP-2026-08-01` | Observed | High for deletion of the highest allocated map in this state because the user confirmed absence after restart and a complete comparison agrees. | Deleting a non-trailing map may leave a null/hole; ID reuse, child handling, reordering, undo/recovery behavior, and other versions remain unknown. |
| Adding a second top-level default-settings map appends `Map003.json` and index/ID/order 3 while preserving `Map002.json`; the two map files are byte-identical despite different editor Names stored in `MapInfos.json`. | `MZ-1.10.0-ADD-SECOND-MAP-2026-08-01` | Observed | High for sequential allocation and external naming in this state because a complete comparison and restart confirmation agree. | Allocation after a hole, modified map contents, duplicated maps, child maps, and other versions may behave differently. |
| Deleting middle ID-2 removes `Map002.json`, retains a length-4 `MapInfos.json` with `null` at index 2, preserves ID/index 3 and `Map003.json`, and compacts the surviving entry's `order` from 3 to 2. | `MZ-1.10.0-DELETE-MIDDLE-MAP-2026-08-01` | Observed | High for this state because the user confirmed the intended maps after restart and a complete comparison isolates the removal, null hole, and order change. | Whether the next creation reuses index/ID/file 2, how multiple holes are selected, child-map behavior, and other versions remain unknown. |
| Creating a map after that deletion offers default Name `MAP002`, recreates `Map002.json`, fills array index/ID 2, assigns logical `order` 3, and preserves the existing index/ID 3 entry at `order` 2. | `MZ-1.10.0-REUSE-MAP-HOLE-2026-08-01` | Observed | High for reuse of the single available hole because the user-reported default Name and complete disk comparison agree. | Multiple-hole selection, whether a maximum or deleted trailing slot behaves the same, allocation limits, child maps, and other versions remain unknown. |
| Reordering two top-level maps swaps only their `MapInfos.json` `order` values while preserving array positions, IDs, `parentId` 0, filenames, and map contents; the moved/selected map becomes `System.editMapId`. | `MZ-1.10.0-REORDER-TOP-LEVEL-MAPS-2026-08-01` | Observed | High for this pair because the user confirmed both UI orders after restart and the inverse-order folders differ only in `MapInfos.json` and `System.json`. | Ordering with children, collapsed nodes, more maps, equal/invalid order values, manual edits, and other versions remain unknown. |
| Creating a child map under ID 3 offers default Name `MAP004`, creates file/index/ID 4 with `parentId` 3 and `order` 3, sets the parent `expanded` true, and shifts the following top-level map's `order` from 3 to 4 without changing existing map files. | `MZ-1.10.0-ADD-CHILD-MAP-2026-08-01` | Observed | High for this hierarchy because the user confirmed indentation after restart and the complete delta isolates the new file and map-info changes. | Multiple children, collapsed-state persistence, reparenting, nested depth, parent deletion, and other versions remain unknown. |
| Reparenting child ID 4 from parent ID 3 to ID 2 changes the child's `parentId`/`order` and the new parent's `order`; IDs, filenames, map contents, and serialized `expanded` values remain fixed. | `MZ-1.10.0-REPARENT-CHILD-MAP-2026-08-01` | Observed with locator limitation | Medium-high: the pre-action structure was inspected immediately, the user confirmed the post-action hierarchy after restart, and both current copies match it; however, the pre-action directory was later overwritten. | Deeper hierarchy, reparenting across collapsed subtrees, sibling placement choices, cycles, parent deletion, and other versions remain unknown. |
| In a controlled exact-copy title edit, Save Project updates the expected files while MZ remains open; quitting without a prompt makes no subsequent content, size, hash, or modification-time change. Earlier apparent close-required behavior is not reproduced. | `MZ-HELP-PROJECT-LIFECYCLE-2026-08-01`, `MZ-1.10.0-SAVE-WHILE-OPEN-CLOSE-2026-08-01`, `MZ-1.10.0-SAVE-CLOSE-PATTERN-2026-08-01` | Documented and Observed; earlier cause Unknown | High for this title-edit workflow because complete tree fingerprints were identical before and after close. | Other edit types, dirty close, project switching, delayed or failed I/O, platforms, and versions remain untested. |
| The standard database inventory includes actors, classes, skills, items, weapons, armors, enemies, troops, states, animations, tilesets, common events, and system data. | `MZ-HELP-DATABASE-2026-08-01`, `MZ-SCRIPTREF-DB-1.0.0`, `MZ-CONVERT-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for MZ 1.10.0 because every template contains the plural filenames named by the script reference. | Exact other-version coverage and editor-requiredness remain unknown. |
| The conversion page's singular `Skill.json`/`Item.json` spellings conflict with the script reference, but MZ 1.10.0 New Game output uses `Skills.json`/`Items.json` exclusively. | `MZ-SCRIPTREF-DB-1.0.0`, `MZ-CONVERT-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented conflict and Observed resolution | High for MZ 1.10.0: all four projects use plural forms and none contains a singular form. | The conversion page may contain typographical errors or reflect unobserved conversion behavior. |
| `Animations.json` is MZ database data; an imported MV-compatible `img/animations` directory is deprecated and is absent from all four fresh MZ 1.10.0 projects. | `MZ-SCRIPTREF-DB-1.0.0`, `MZ-CONVERT-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for fresh 1.10.0 output. | Converted projects and later versions may contain the directory. |
| `js/plugins` contains plugin files and `js/plugins.js` is the registration/configuration file present in every observed project. | `MZ-CONVERT-2026-08-01`, `MZ-HELP-PLUGINS-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for MZ 1.10.0 filename/layout; the HD Layout template also demonstrates template-dependent plugin inventory. | Other-version representation and plugin-defined additions remain unknown. |
| Optional plugin-defined JSON files may coexist in `data`. | `MZ-HELP-UNIQUE-DATA-2026-08-01` | Documented | High: the official `UniqueDataLoading` plugin instructions explicitly allow parseable JSON files there. | Other plugins may use non-JSON files, nested directories, or external paths. |
| Users may add browsable subfolders under asset folders and import or delete project resources through Resource Manager. | `MZ-HELP-ASSETS-2026-08-01`, `MZ-HELP-RESOURCE-LIFECYCLE-2026-08-01` | Documented | High for the stated editor capabilities. | Exact eligible roots, nesting depth, collision behavior, validation, and empty-directory cleanup are not stated. |
| In a controlled MZ 1.10.0 Resource Manager import, selecting a contributor-created PNG for `img/pictures` immediately added a same-named, byte-identical copy before Save Project; its size, mode, and modification time matched the source, and no existing file content changed. | `MZ-1.10.0-RESOURCE-IMPORT-PICTURE-2026-08-01` | Observed | High for this asset, destination, editor version, and platform because complete before/after path and content comparisons were made while the editor remained open. | Import validation, filename normalization, collision behavior, other formats/destinations, nested paths, permissions on other platforms, and whether timestamp preservation is contractual remain unknown. |
| Saving that imported-resource project changed only `System.versionId` in existing file content; the imported PNG remained byte-identical, and quitting without a prompt caused no later content change. | `MZ-1.10.0-RESOURCE-IMPORT-PICTURE-2026-08-01` | Observed | High for content behavior in this workflow because comparisons were made before save, after save while open, and after clean quit. | Pre-save metadata was not captured in this import experiment; later deletion evidence isolates core-file metadata behavior for one separate save, not a universal contract. |
| In a controlled MZ 1.10.0 Resource Manager deletion, confirming deletion immediately removed the selected PNG before Save Project, retained the existing `img/pictures` directory, and changed no remaining file content. | `MZ-1.10.0-RESOURCE-DELETE-PICTURE-2026-08-01` | Observed | High for this unreferenced asset, directory, version, and platform because a complete comparison was made while the editor remained open. | Other reference types, directory depths, recovery/undo, validation, destinations, platforms, and versions remain unknown. |
| Save Project after that deletion changed only `System.versionId` in file content but advanced modification and change times on `game.rmmzproject`, `data/System.json`, `package.json`, and `index.html`; their modes and birth times remained fixed, and clean quit caused no later change. | `MZ-1.10.0-RESOURCE-DELETE-PICTURE-2026-08-01` | Observed | High for this save because hashes and metadata were captured immediately before save, after save while open, and after no-prompt quit. | Whether the same four files are touched for every save, exact write mechanics/atomicity, dirty close, failures, filesystems, and other versions remain unknown. |
| During deletion-experiment setup, `package.json` modification time advanced without a byte change in both the previously closed source project and the newly opened copy. | `MZ-1.10.0-RESOURCE-DELETE-PICTURE-2026-08-01` | Observed timing; cause Unknown | High that the metadata-only changes occurred; low for attribution because no snapshot was taken before project launch/copy/open. | Editor launch, automatic reopening, explicit open, copying behavior, host software, or another process may have caused the changes. |
| Resource Manager in MZ 1.10.0 discovers a manually created one-level subdirectory beneath `img/pictures` and can import a PNG into it; the resulting project-relative path preserves the directory and filename exactly, and the copied bytes and observed file metadata match the source. | `MZ-1.10.0-RESOURCE-IMPORT-NESTED-2026-08-01` | Observed | High for this path, asset, version, and platform because the closed pre-open tree, user-observed Resource Manager UI, and three disk boundaries agree. | Greater depth, other asset roots/formats, case/Unicode normalization, invalid names, collisions, symlinked folders, platforms, and versions remain unknown. |
| With a closed pre-open snapshot, launching/opening the nested experiment advanced `package.json` modification/change times and the marker's change time without changing their bytes; other selected core-file content and modification times stayed fixed. | `MZ-1.10.0-RESOURCE-IMPORT-NESTED-2026-08-01` | Observed workflow effect | High for the launch/open sequence because the same copy was inspected immediately before and while open. | The experiment does not separate application launch, automatic reopen, explicit project open, or lower-level filesystem metadata mechanisms. |
| Save Project after nested import again changed only `System.versionId` in standard file content, advanced modification/change times on four selected core files, preserved the nested asset, and was followed by a no-change clean quit. | `MZ-1.10.0-RESOURCE-IMPORT-NESTED-2026-08-01` | Observed | High for this workflow because pre-save, post-save-open, and post-close hashes and metadata were captured. | Other actions, directories, write mechanics/atomicity, dirty close, failures, filesystems, and versions remain unknown. |
| For one MZ 1.10.0 Show Picture event command selecting a nested PNG, the serialized asset reference is the forward-slash string `tilewright-nested/tilewright-resource-probe` relative to `img/pictures`, with no `.png` extension. | `MZ-1.10.0-REFERENCE-NESTED-PICTURE-2026-08-01` | Observed | High for this command, asset, and version because an exact closed baseline, user-reported selection UI, and structural before/after comparison agree. | Other commands/fields, asset roots/formats, separators, extension rules, case/Unicode, platforms, versions, and editor tolerance of altered references remain unknown. |
| Creating and saving that one event changed only `data/Map001.json` and `data/System.json` content; the referenced asset remained byte-identical and clean close made no later filesystem change. | `MZ-1.10.0-REFERENCE-NESTED-PICTURE-2026-08-01` | Observed | High for this controlled action because complete standard-content comparisons and selected metadata were captured before open, after save while open, and after close. | Prompt status at close was not reported; runtime resolution, manually altered references, and other event edits remain unknown. |
| Given a controlled Show Picture event matching the previously editor-created reference, MZ 1.10.0 Resource Manager deletes the referenced nested PNG immediately without warning or refusal, retains the now-empty nested directory, and leaves the serialized reference unchanged before Save Project. | `MZ-1.10.0-DELETE-REFERENCED-PICTURE-2026-08-01` | Observed, with controlled-input limitation | High for the deletion UI and disk effects because the exact owned workspace was verified, MZ reloaded and indexed the controlled event, and the pre-save path/hash/reference state was captured. Medium for generalization because this copy's event was injected from the earlier observed structure rather than recreated through Event Editor. | Other commands/reference types, runtime behavior, editor-created dangling references in the same copy, recovery, platforms, and versions remain unknown. |
| Saving after that referenced deletion leaves the dangling Show Picture string and `Map001.json` bytes unchanged, changes only `System.versionId` in standard file content, and closing the saved project makes no later content change. | `MZ-1.10.0-DELETE-REFERENCED-PICTURE-2026-08-01` | Observed, with controlled-input limitation | High for the captured pre-save, post-save-open, and post-close content boundaries. | Runtime resolution, diagnostic behavior in other editor surfaces, metadata write mechanics, other references, platforms, and versions remain unknown. |
| Official asset roles include PNG images, Ogg Vorbis audio, and WebM/MP4 movies, with platform-dependent movie pairing guidance. | `MZ-HELP-ASSETS-2026-08-01` | Documented | High for the documented asset standards. | Editor acceptance of malformed files, case/Unicode behavior, and plugin-defined formats remain unknown. |
| Every fresh MZ 1.10.0 template contains 120 `.efkefc`, seven `.efkmodel`, and 48 PNG files under `effects`, plus 31 PNG and 31 stem-paired `.txt` files under `img/tilesets`. | `MZ-1.10.0-FRESH-4-2026-08-01` | Observed | High for counts, extensions, and one-to-one basename pairing because all four manifests agree. | Effect-file semantics and tileset text-companion purpose remain unknown; contents were intentionally not inspected. |
| The editor's System 2 settings select main, number, and fallback font filenames; all four fresh projects contain the same two-file WOFF `fonts` inventory and one PNG under `icon`. | `MZ-HELP-SYSTEM2-2026-08-01`, `MZ-CONVERT-2026-08-01`, `MZ-1.10.0-FRESH-4-2026-08-01` | Documented and Observed | High for the font role and fresh 1.10.0 inventory. | Other supported font extensions, requiredness, subfolders, filename rules, and deployment behavior remain unknown. |
| Fresh MZ 1.10.0 projects contain an observed runtime shell comprising `index.html`, `package.json`, `css/game.css`, `js/main.js`, `js/plugins.js`, `js/rmmz_*.js`, and JavaScript/WebAssembly dependencies under `js/libs`. | `MZ-1.10.0-FRESH-4-2026-08-01`, `MZ-HELP-PROJECT-LIFECYCLE-2026-08-01` | Observed, with documented update capability | High for the four fresh projects; official help separately documents Update Core Script. | Requiredness, file ownership, replacement behavior, and cross-version inventories remain unknown. |
| Changing only the Game Title in the MZ 1.10.0 System 1 editor and performing a persistent save changes `data/System.json` at `gameTitle` and `versionId`, `package.json` at `window.title`, and the HTML `title` element in `index.html`, without changing the path manifest or marker bytes. | `MZ-1.10.0-SAVE-GAME-TITLE-2026-08-01` | Observed | High for this action and version because a complete before/after comparison found exactly three changed files and the user confirmed the title survived restart. | Other settings, save operations, versions, and failure paths may touch different files; the meaning and generation rule of `versionId` remain unknown. |
| `img`, `audio`, and `effects` can contribute files to deployment and may be filtered by the deployment option. | `MZ-HELP-DEPLOY-2026-08-01` | Documented | High for these named project directories and option behavior; low for completeness. | The source does not enumerate all deployed or authoring directories. |
| Image/audio encryption is a deployment transformation, and encrypted files cannot be used within the authoring project folder. | `MZ-HELP-DEPLOY-2026-08-01` | Documented | High for the stated boundary. | Output filenames, extensions, headers, key representation, and per-target behavior remain unknown. |
| The default-option MZ 1.10.0 Basic Web deployment preserves every non-marker file path, omits `game.rmmzproject` and the empty `movies` directory, and changes only `package.json` content. | `MZ-1.10.0-BASIC-WEB-DEPLOY-2026-08-01` | Observed | High for this project, target, and option set because complete manifests and all common file contents were compared. | Other projects, nonempty movies, options, targets, and versions may deploy differently. |
| All 252 `data/*.json` files across the four observed projects are valid UTF-8 JSON without BOM, carriage returns, or final newlines. | `MZ-1.10.0-FRESH-4-2026-08-01` | Observed | High for the inspected MZ 1.10.0 templates because every file was checked. | Other versions, edited projects, plugin files, and serializer tolerance may differ. |
| The proposed strict-gate-plus-CST representation no-op serializes all 252 JSON files from the four observed fresh MZ 1.10.0 projects byte-identically. | `MZ-1.10.0-CST-NOOP-2026-08-03` | Observed | High for the copied authorized corpus and `jsonc-parser` 0.33.1 because every file passed the strict gate and exact-byte comparison. | Mutations, edited/plugin data, other versions, loading diagnostics, and editor reopen behavior remain untested. |
| The project tree can be separated cleanly into “editor-only files” and “runtime-only files” by directory name. | Official sources show project data/assets being edited and also deployed or loaded at runtime. | Inferred false | High confidence that the binary split is unsafe at directory level: `data` and assets serve both authoring and game execution. | Individual files may still be editor-only, runtime templates, or generated, but this needs observation. |
| Unknown root entries or unrecognized files prove the directory is not an MZ project. | `MZ-HELP-UNIQUE-DATA-2026-08-01` establishes sanctioned extension data. | Inferred false | High: rejecting additions would conflict with documented plugin extensibility. | A future strict health check may diagnose particular conflicts without changing root identity. |
| Incidental unknown root metadata may appear after project creation without changing marker-based identity. | `MZ-1.10.0-BASIC-LATER-METADATA-2026-08-01` | Observed presence; origin Inferred | High that the file appeared after the original baseline; only medium that it is host-OS metadata because no controlled creator action was observed. | Editors, backup tools, sync tools, plugins, or other host software can add different unknown entries. |
| JSON key order, whitespace, encoding, filename case, array order, IDs, and cross-file invariants may be normalized during discovery. | No supporting evidence found. | Unknown | No confidence; discovery does not need normalization. | Controlled round-trip and before/after experiments are required for each behavior. |

### Evidence records

#### `MZ-CONVERT-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ official Japanese conversion course;
  page/editor version is not stated; describes conversion from MV to MZ.
- **Locator:** “MVデータのコンバート” → conversion destination table and
  cautions, <https://rpgmakerofficial.com/product/mz/course/convert/convert.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Names MZ project-relative locations for audio, images, icons,
  fonts, maps, database categories, system data, and plugins. It also states that
  MV-compatible animation images require an added directory because MZ lacks it.
- **Conflict:** Uses singular `Skill.json` and `Item.json`, while the official
  script reference and all four observed MZ 1.10.0 projects use plural
  filenames. It also prints singular `js/plugin.js`, while all four observed
  MZ 1.10.0 projects use plural `js/plugins.js`. Conversion behavior and other
  versions remain unobserved.
- **Redistribution:** Only path names, a paraphrased inventory, and a locator are
  committed.

#### `MZ-SCRIPTREF-DB-1.0.0`

- **Kind:** Official documentation
- **Version/environment:** “RPGツクールMZ スクリプトリファレンス” suite,
  version 1.0.0 as stated by the companion `first.pdf`; the database PDF itself
  does not restate an editor version.
- **Locator:** Database reference,
  <https://rpgmakerofficial.com/product/mz/plugin/javascript/script_reference/database.pdf>;
  suite introduction and version,
  <https://rpgmakerofficial.com/product/mz/plugin/javascript/script_reference/first.pdf>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Associates database globals and categories with `Actors.json`,
  `Classes.json`, `Skills.json`, `Items.json`, `Weapons.json`, `Armors.json`,
  `Enemies.json`, `Troops.json`, `States.json`, `Animations.json`,
  `Tilesets.json`, `CommonEvents.json`, `System.json`, example `Map001.json`,
  and `MapInfos.json`.
- **Redistribution:** No PDF or extracted table is committed; only a summarized
  filename/category inventory and exact locator are recorded.

#### `MZ-HELP-DATABASE-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “Database” → “What is the Database?”,
  <https://rpgmakerofficial.com/product/MZ_help-en/01_08.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Describes the database categories and their high-level purposes,
  distinct from maps and map events.
- **Redistribution:** Only a paraphrase and locator are committed.

#### `MZ-HELP-ASSETS-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “Asset Standards,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_11_01.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Documents project-relative `img` subdirectories and their asset
  purposes; permits browsable subfolders under asset folders; and documents PNG
  images, Ogg Vorbis audio, and WebM/MP4 movie roles. It does not provide a
  complete project-root inventory or exhaustive editor-tolerance rules.
- **Redistribution:** Only a summarized directory inventory and locator are
  committed.

#### `MZ-HELP-PLUGINS-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “How to Use Aid Tools” → “Plugin Manager,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_05.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** The editor manages official and user-created plugins, their
  enabled state, and parameters.
- **Redistribution:** Only a paraphrase and locator are committed.

#### `MZ-HELP-UNIQUE-DATA-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page; official `UniqueDataLoading` plugin.
- **Locator:** “Using Official Plugins” → “UniqueDataLoading,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_11_05.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Allows optional parseable JSON files in the project's `data`
  folder and explicitly mentions data added by proprietary plugins.
- **Redistribution:** Only a paraphrase and locator are committed.

#### `MZ-HELP-RESOURCE-LIFECYCLE-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “How to Use Aid Tools” → “Resource Manager,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_05.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Resource Manager displays project resource folders/files and can
  import, export, or delete resources. Export leaves the project copy in place;
  deletion is described as unrecoverable. Exact filesystem validation and
  reference behavior are not specified.
- **Redistribution:** Only a paraphrase and locator are committed.

#### `MZ-HELP-PROJECT-LIFECYCLE-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “Menu Bar Content” → “Save Project,” “Open Folder,” and “Update
  Core Script,” <https://rpgmakerofficial.com/product/MZ_help-en/01_04.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** Save Project overwrites project contents, Open Folder exposes the
  project directory for manual file operations, and Update Core Script updates
  the core-script version. The page does not enumerate files touched by either
  write operation.
- **Redistribution:** Only a paraphrase and locator are committed.

#### `MZ-HELP-SYSTEM2-2026-08-01`

- **Kind:** Official documentation
- **Version/environment:** RPG Maker MZ online English help; editor/help version
  is not stated on the page.
- **Locator:** “Database” → “System 2 Settings” → “Advanced Settings,”
  <https://rpgmakerofficial.com/product/MZ_help-en/01_08_12_02.html>.
- **Accessed/observed:** 2026-08-01
- **Summary:** System 2 selects main and number font filenames plus a fallback
  font. The page does not specify the `fonts` directory schema or accepted file
  formats.
- **Redistribution:** Only a paraphrase and locator are committed.

#### `MZ-1.10.0-SAVE-GAME-TITLE-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the user-created Basic project.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-save-title/` in the primary integration checkout.
- **Accessed/observed:** Edited, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user changed only Database → System 1 → Game Title to the
  contributor-created string `Tilewright Layout Research 001`, saved the
  project, closed the editor, reopened the project, and confirmed the title
  persisted. Read-only inspection verified ignored canonical paths and absence
  of symlinks, compared complete sorted path manifests and every corresponding
  file byte for byte, parsed the two changed JSON files structurally, and
  compared the HTML title element. Changed scalar values were redacted for
  lexical comparisons; vendor/default values were not recorded.
- **Summary:** Baseline and experiment each contain 1,313 files and 31
  directories with identical relative paths. Exactly three files differ:
  `data/System.json`, `package.json`, and `index.html`. `System.json` changes
  only string `gameTitle` and numeric `versionId`; `package.json` changes only
  string `window.title`; and `index.html` changes only its `title` element. The
  synthetic title occurs once in each changed file. After those changed scalars
  are redacted, each file is byte-identical to its baseline counterpart. The
  JSON byte properties remain valid UTF-8 with no BOM, carriage returns, or
  final newline; `index.html` retains its final newline. File modes are
  unchanged, and `game.rmmzproject` remains byte-identical.
- **Conflict/history:** An earlier apparent edit did not survive restart and
  produced no verified persisted delta. It is excluded from the format claim;
  only the subsequently saved, restart-surviving state is recorded here.
- **Redistribution:** Only the contributor-created title, project-relative
  paths, JSON paths/types, HTML element name, counts, and derived comparisons
  are committed. No project files, default values, runtime code, or vendor
  assets are included.

#### `MZ-1.10.0-ADD-MAP-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the user-created Basic project.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-map-add-one/` in the primary integration checkout.
- **Accessed/observed:** Created, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user created exactly one top-level map, changed only its
  editor Name from the offered `MAP002` to the contributor-created
  `Tilewright Map Add 001`, left Display Name blank and all other New Map
  settings at their editor-provided defaults, saved, closed, reopened the exact
  experiment marker, and confirmed persistence. Read-only inspection verified
  the ignored canonical path and absence of symlinks, compared complete path
  manifests and every corresponding file byte for byte, and inspected only
  structural JSON differences and the contributor-created name.
- **Summary:** The baseline has 1,313 files and the experiment 1,314; both have
  31 directories. The only added path is `data/Map002.json`; only
  `data/MapInfos.json` and `data/System.json` otherwise differ. `MapInfos.json`
  grows from two to three array elements, preserves both existing elements
  structurally and their serialized bytes, and appends an object at array index
  2 whose numeric `id` and `order` are 2, numeric `parentId` is 0, and `name`
  is the synthetic name. The new entry has a boolean `quick` field that the
  pre-existing map-info entry lacks, demonstrating non-uniform object keys in
  this observed array. `Map002.json` is a JSON object with an empty string
  `displayName`, an empty `events` array, and no top-level `id`; the synthetic
  editor Name occurs only in `MapInfos.json`. `System.json` changes only numeric
  `editMapId` and `versionId`, with `editMapId` becoming 2; after redacting
  those scalars it is byte-identical to baseline. All three affected JSON files
  are valid UTF-8 without BOM, carriage returns, or final newlines. The new map
  file was created with mode 0644 while the copied `Map001.json` retained mode
  0755; this is filesystem metadata observed in this environment, not a format
  requirement. The marker remains byte-identical.
- **Conflict/history:** The first reported creation attempt produced no disk
  delta and no occurrence of the synthetic name. It is excluded from the
  format claim; only the later restart-surviving state is recorded.
- **Redistribution:** Only the contributor-created name, project-relative
  paths, JSON paths/types, counts, and derived structural observations are
  committed. No map data, default values beyond identifiers needed to describe
  the allocation, runtime code, or vendor assets are included.

#### `MZ-1.10.0-DELETE-TRAILING-MAP-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the successful one-map-addition project.
- **Locator:** Baseline `.local-research/mz-1_10_0-map-add-one/`; experiment
  `.local-research/mz-1_10_0-map-delete-one/`; original comparison point
  `.local-research/sources/mz-1_10_0-fresh-basic/` in the primary integration
  checkout.
- **Accessed/observed:** Deleted, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user deleted only the contributor-created
  `Tilewright Map Add 001`, explicitly saved the copied project, closed and
  reopened its exact marker, and confirmed the map remained absent. Read-only
  inspection verified the ignored canonical path and absence of symlinks,
  compared complete manifests and all corresponding files byte for byte, and
  inspected only structural JSON differences and the contributor-created name.
- **Summary:** The baseline has 1,314 files and the deletion project 1,313; both
  have 31 directories. The only removed path is `data/Map002.json`; only
  `data/MapInfos.json` and `data/System.json` otherwise differ. `MapInfos.json`
  shrinks from three elements to two, exactly equals the retained prefix
  structurally, contains no trailing null, and is byte-for-byte identical to
  the original fresh Basic `MapInfos.json`. `System.json` changes only numeric
  `editMapId` and `versionId`; `editMapId` becomes 1. After those scalars are
  redacted, it is byte-identical to the addition baseline; compared with the
  original Basic project it differs only at `versionId`. The deleted synthetic
  name no longer occurs anywhere in the experiment. The affected JSON retains
  valid UTF-8, no BOM, no carriage returns, no final newline, and the existing
  file modes. The marker remains byte-identical.
- **Evidence limit:** This was deletion of the last/highest map entry. It does
  not establish how MZ represents deletion from the middle of an allocated
  sequence, whether a freed ID is reused, or how children affect deletion.
- **Redistribution:** Only the contributor-created name, project-relative
  paths, JSON paths/types, counts, and derived comparisons are committed. No
  project file contents, runtime code, or vendor assets are included.

#### `MZ-1.10.0-ADD-SECOND-MAP-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the successful one-map-addition project.
- **Locator:** Baseline `.local-research/mz-1_10_0-map-add-one/`; experiment
  `.local-research/mz-1_10_0-map-add-two/` in the primary integration checkout.
- **Accessed/observed:** Created, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user created one additional top-level map, changed only its
  editor Name to the contributor-created `Tilewright Map Add 002`, left Display
  Name blank and all other settings at their editor-provided defaults,
  explicitly saved, closed, reopened the exact experiment marker, and confirmed
  both added maps persisted. Read-only inspection verified the ignored
  canonical path and absence of symlinks, compared complete manifests and all
  corresponding files byte for byte, and inspected only structural JSON
  differences and the contributor-created name.
- **Summary:** The baseline has 1,314 files and the experiment 1,315; both have
  31 directories. The only added path is `data/Map003.json`; only
  `data/MapInfos.json` and `data/System.json` otherwise differ. `MapInfos.json`
  grows from three elements to four, preserves the existing elements
  structurally and their serialized bytes, and appends index 3 with numeric
  `id` and `order` 3, numeric `parentId` 0, and the synthetic Name.
  `System.json` changes only numeric `editMapId` and `versionId`, with
  `editMapId` becoming 3; redacting those scalars makes it byte-identical to
  baseline. `Map002.json` remains byte-identical, and new `Map003.json` is
  byte-identical to `Map002.json` despite their different editor Names. Both
  map files have blank `displayName`, no top-level `id`, and empty event arrays.
  The second synthetic Name occurs only in `MapInfos.json`. Affected JSON
  remains valid UTF-8 without BOM, carriage returns, or final newlines; the new
  map file has mode 0644 in this environment. The marker remains byte-identical.
- **Evidence limit:** The byte equality applies to two maps created with the
  same observed defaults. It does not prove that filename and `MapInfos.json`
  are the only identity sources for modified, imported, child, plugin-extended,
  or other-version maps.
- **Redistribution:** Only contributor-created names, project-relative paths,
  JSON paths/types, counts, and derived comparisons are committed. No map data,
  runtime code, default database contents, or vendor assets are included.

#### `MZ-1.10.0-DELETE-MIDDLE-MAP-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the successful two-added-map project.
- **Locator:** Baseline `.local-research/mz-1_10_0-map-add-two/`; experiment
  `.local-research/mz-1_10_0-map-delete-middle/` in the primary integration
  checkout.
- **Accessed/observed:** Deleted, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user deleted only `Tilewright Map Add 001` (index/ID 2),
  left `Tilewright Map Add 002` (index/ID 3) intact, explicitly saved, closed,
  reopened the exact experiment marker, and confirmed the intended absence and
  presence. Read-only inspection verified the ignored canonical path and
  absence of symlinks, compared complete manifests and all corresponding files
  byte for byte, and inspected only structural JSON differences and the two
  contributor-created names.
- **Summary:** The baseline has 1,315 files and the experiment 1,314; both have
  31 directories. The only removed path is `data/Map002.json`; only
  `data/MapInfos.json` and `data/System.json` otherwise differ. `MapInfos.json`
  retains length 4, replaces its index-2 object with JSON `null`, and preserves
  the index-3 object's `id` 3 and all its fields except numeric `order`, which
  changes from 3 to 2. Its non-null map indices are therefore 1 and 3.
  `Map003.json` remains byte-identical. `System.editMapId` remains 3 and only
  numeric `versionId` changes; redacting that scalar makes `System.json`
  byte-identical to baseline. The deleted synthetic Name is absent, while the
  surviving Name occurs only in `MapInfos.json`. Affected JSON retains valid
  UTF-8, no BOM, no carriage returns, no final newline, and existing file modes.
  The marker remains byte-identical.
- **Inference:** For this controlled state, `order` represents a compact map-tree
  ordering distinct from stable array index/`id`; this interpretation is high
  confidence for the observed edit but not a cross-version or full semantic
  contract.
- **Evidence limit:** The experiment establishes one middle hole but not
  subsequent ID allocation, multiple-hole selection, reparenting, child
  deletion, or editor behavior with hand-edited inconsistent data.
- **Redistribution:** Only contributor-created names, project-relative paths,
  JSON paths/types, counts, and derived comparisons are committed. No project
  file contents, runtime code, or vendor assets are included.

#### `MZ-1.10.0-REUSE-MAP-HOLE-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the middle-deletion project with a single index-2 hole.
- **Locator:** Baseline `.local-research/mz-1_10_0-map-delete-middle/`;
  experiment `.local-research/mz-1_10_0-map-reuse-hole/` in the primary
  integration checkout.
- **Accessed/observed:** Created, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user began creating one new top-level map and recorded the
  editor-offered default Name `MAP002`, then changed only the editor Name to the
  contributor-created `Tilewright Map Reuse 001`, left Display Name blank and
  all other settings at their editor-provided defaults, saved, closed, reopened
  the exact experiment marker, and confirmed both surviving maps. Read-only
  inspection verified the ignored canonical path and absence of symlinks,
  compared complete manifests and all corresponding files byte for byte, and
  inspected only structural JSON differences and contributor-created names.
- **Summary:** The baseline has 1,314 files and the experiment 1,315; both have
  31 directories. The only added path is `data/Map002.json`; only
  `data/MapInfos.json` and `data/System.json` otherwise differ. `MapInfos.json`
  retains length 4 and replaces the index-2 null with an object whose numeric
  `id` is 2, numeric `order` is 3, numeric `parentId` is 0, and Name is the new
  synthetic value. The existing index-3/ID-3 entry remains structurally
  unchanged at `order` 2. `Map003.json` remains byte-identical. The recreated
  `Map002.json` is byte-identical to `Map003.json`, has blank `displayName`, no
  top-level ID, and an empty events array. `System.json` changes only numeric
  `editMapId` and `versionId`, with `editMapId` becoming 2; redacting those
  scalars makes it byte-identical to baseline. Affected JSON remains valid UTF-8
  without BOM, carriage returns, or final newlines; both added maps have mode
  0644 in this environment. The marker remains byte-identical.
- **Inference:** MZ 1.10.0 reuses the single available map allocation hole in
  this controlled state while appending the new map to the end of the compact
  logical order. Confidence is high for this state, but no rule for choosing
  among multiple holes is established.
- **Locator history:** The experiment directory was inspected immediately after
  hole reuse, then was later used accidentally during the reorder experiment
  and no longer preserves the originally observed order values. The original
  reuse delta and derived structure were recorded before that mutation; the
  later inverse-reorder folder independently returns to the same order/ID shape.
  This history limits raw re-checkability of that directory and is not hidden.
- **Redistribution:** Only contributor-created names, the user-reported default
  Name, project-relative paths, JSON paths/types, counts, and derived
  comparisons are committed. No project file contents, runtime code, or vendor
  assets are included.

#### `MZ-1.10.0-REORDER-TOP-LEVEL-MAPS-2026-08-01`

- **Kind:** Controlled experiment with locator correction
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copies of the three-top-level-map state.
- **Locator:** Reuse-first state
  `.local-research/mz-1_10_0-map-reorder-one/`; Add-first state
  `.local-research/mz-1_10_0-map-reorder-swap/` in the primary integration
  checkout. The latter was created from the identically ordered, later-mutated
  `.local-research/mz-1_10_0-map-reuse-hole/` state.
- **Accessed/observed:** Reordered, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The initial instruction asked the user to place
  `Tilewright Map Reuse 001` above `Tilewright Map Add 002`. The source folder
  was also changed to that order, so comparing it with the first copy showed no
  map delta and only a save-generated `System.versionId` change. The user then
  confirmed the visible Reuse-first order. For the inverse experiment, the user
  copied that state, moved `Tilewright Map Add 002` immediately above
  `Tilewright Map Reuse 001` without changing indentation or content, saved,
  closed, reopened, and confirmed completion. Read-only inspection compared the
  preserved Reuse-first and Add-first copies completely.
- **Summary:** Both order states contain 1,315 files and 31 directories with
  identical path manifests. Only `data/MapInfos.json` and `data/System.json`
  differ. In the Reuse-first state, index/ID 2 has `order` 2 and index/ID 3 has
  `order` 3. In the Add-first state, their `order` values are 3 and 2
  respectively. For both entries, `order` is the only changed map-info field;
  IDs, array indices, Names, and `parentId` 0 remain stable. `Map002.json` and
  `Map003.json` remain byte-identical across states. `System.editMapId` changes
  from 2 to the moved/selected ID 3, and `System.versionId` changes; no other
  System scalar differs. The affected JSON retains valid UTF-8, no BOM, no
  carriage returns, and no final newline. The marker remains byte-identical.
- **Inference:** For this top-level pair, smaller positive `order` appears
  earlier in the confirmed UI order, while ID/index/file identity remains
  stable. Confidence is high for this pair, but hierarchy traversal and invalid
  or duplicate order values remain unknown.
- **Evidence limit:** Because the first source directory was later mutated, the
  record relies on the preserved first-copy state, the inverse-copy comparison,
  the user's UI confirmations, and the immediately recorded earlier snapshot.
  It does not establish behavior for child maps or arbitrary ordering data.
- **Redistribution:** Only contributor-created names, project-relative paths,
  JSON paths/types, counts, and derived comparisons are committed. No project
  file contents, runtime code, or vendor assets are included.

#### `MZ-1.10.0-ADD-CHILD-MAP-2026-08-01`

- **Kind:** Controlled experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the successful Add-first top-level-order project.
- **Locator:** Baseline `.local-research/mz-1_10_0-map-reorder-swap/`;
  experiment `.local-research/mz-1_10_0-map-add-child-one/` in the primary
  integration checkout.
- **Accessed/observed:** Created, saved, closed, reopened, and confirmed by the
  user; inspected read-only on 2026-08-01.
- **Procedure:** The user invoked New Map from parent
  `Tilewright Map Add 002` (index/ID 3), recorded the offered default Name
  `MAP004`, changed only the Name to contributor-created
  `Tilewright Map Child 001`, left Display Name blank and all other settings at
  editor defaults, confirmed one-level indentation beneath that parent, saved,
  closed, reopened the exact experiment marker, and confirmed the hierarchy.
  Read-only inspection verified the ignored canonical path and absence of
  symlinks, compared complete manifests and all corresponding files byte for
  byte, and inspected only structural JSON differences and contributor names.
- **Summary:** The baseline has 1,315 files and the experiment 1,316; both have
  31 directories. The only added path is `data/Map004.json`; only
  `data/MapInfos.json` and `data/System.json` otherwise differ. `MapInfos.json`
  grows from four elements to five and appends index/ID 4 with numeric
  `parentId` 3 and numeric `order` 3. Parent index/ID 3 remains at `order` 2 and
  changes only boolean `expanded` from false to true. The following top-level
  index/ID 2 remains `parentId` 0 and changes only `order` from 3 to 4. The new
  traversal orders are therefore ID 1 at 1, parent ID 3 at 2, child ID 4 at 3,
  and later top-level ID 2 at 4. Existing `Map002.json` and `Map003.json` remain
  byte-identical. New `Map004.json` is byte-identical to those default-settings
  maps, with blank `displayName`, no top-level ID, and an empty events array.
  `System.json` changes only numeric `editMapId` and `versionId`, with
  `editMapId` becoming 4; redacting those scalars makes it byte-identical to
  baseline. Affected JSON remains valid UTF-8 without BOM, carriage returns, or
  final newlines; the new map file has mode 0644. The marker remains
  byte-identical.
- **Inference:** In this observed tree, `parentId` identifies hierarchy while
  compact `order` follows the visible depth-first sequence; adding a child also
  expands its parent in editor metadata. Confidence is high for this one child,
  but deeper hierarchy and collapsed-state behavior remain untested.
- **Locator history:** The experiment directory was inspected immediately after
  child creation, then was later modified accidentally during the reparent
  experiment and now contains the reparented state. The intended reparent copy
  also contains that post-action hierarchy, so no immutable pre-action folder
  remains. The immediately recorded pre-action structure is retained in this
  ledger, and the limitation is recorded rather than hidden.
- **Redistribution:** Only contributor-created names, the user-reported default
  Name, project-relative paths, JSON paths/types, counts, and derived
  comparisons are committed. No map data, runtime code, or vendor assets are
  included.

#### `MZ-1.10.0-REPARENT-CHILD-MAP-2026-08-01`

- **Kind:** Controlled action with overwritten baseline locator
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copies of the one-child project.
- **Locator:** Intended baseline
  `.local-research/mz-1_10_0-map-add-child-one/` and intended experiment
  `.local-research/mz-1_10_0-map-reparent-child-one/` in the primary integration
  checkout. Both currently contain the post-reparent hierarchy.
- **Accessed/observed:** Reparented, saved, closed, reopened, and confirmed by
  the user; inspected read-only on 2026-08-01.
- **Procedure:** The user was instructed to copy the child project and move
  `Tilewright Map Child 001` from parent `Tilewright Map Add 002` (ID 3) to
  parent `Tilewright Map Reuse 001` (ID 2), preserving one-level indentation
  and all content. The source rather than intended copy received the move; the
  user confirmed persistence after restart. Read-only inspection discovered
  overwritten locator state, verified ignored canonical paths and absence of
  symlinks, and compared current copies with the immediately recorded
  pre-action structure without reconstructing evidence files.
- **Summary:** The recorded pre-action state had child index/ID 4 at `parentId`
  3/`order` 3 and new parent index/ID 2 at `order` 4. Both current post-action
  copies have child ID 4 at `parentId` 2/`order` 4 and parent ID 2 at `order` 3.
  Old parent ID 3 remains top-level at `order` 2. Its boolean `expanded` remains
  true, while the new parent's remains false; no expansion scalar is normalized
  by the move. `Map002.json`, `Map003.json`, and `Map004.json` remain
  byte-identical, `System.editMapId` remains 4, and marker bytes remain fixed.
- **Inference:** `parentId` and compact `order` carry the persisted hierarchy
  move in this state, while `expanded` is independently persisted editor state
  rather than a value derived solely from having children. Confidence is high
  for the observed values; exact UI collapse semantics and other versions remain
  unestablished.
- **Evidence limit:** No complete immutable pre-action directory remains. The
  field-level direction is supported by the immediately recorded pre-action
  snapshot, two matching post-action copies, timestamps, requested action, and
  user confirmation, so confidence is lower than for intact before/after pairs.
  Cycles, deeper nesting, and parent deletion are untested.
- **Redistribution:** Only contributor-created names, project-relative paths,
  JSON paths/types, counts, and derived comparisons are committed. No project
  file contents, runtime code, or vendor assets are included.

#### `MZ-1.10.0-SAVE-CLOSE-PATTERN-2026-08-01`

- **Kind:** Repeated procedural observation; controlled experiment pending
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  several disposable project copies operated by the user.
- **Locator:** Earlier title and map experiments under the authorized
  `.local-research/` root, including the failed-first/successful-later histories
  in `MZ-1.10.0-SAVE-GAME-TITLE-2026-08-01` and
  `MZ-1.10.0-ADD-MAP-2026-08-01`, plus the reorder/reparent locator histories.
- **Accessed/observed:** Pattern accumulated and explicitly reported by the user
  on 2026-08-01.
- **Summary:** On multiple attempts, the user reported invoking Save Project
  while the editor remained open, but read-only inspection found no intended
  disk delta. Repeating the edit and closing the project produced the persisted
  delta. Later source directories also acquired active-project changes during
  close-oriented reorder and reparent workflows. The user concluded that
  closing after saving was necessary in their workflow.
- **Conflict:** Official help describes Save Project itself as overwriting
  project contents. The observations do not isolate whether Save returned
  before a delayed flush, close or project switching performed an additional
  write, the wrong copy remained active, or another workflow detail explains
  the result.
- **Next evidence needed:** From a fully closed editor, change one synthetic
  title in an exact disposable copy, invoke Save Project, keep the editor open
  for a disk comparison, then close without another edit for a second
  comparison.
- **Resolution:** `MZ-1.10.0-SAVE-WHILE-OPEN-CLOSE-2026-08-01` performed that
  experiment and did not reproduce a close requirement. The earlier pattern is
  retained as workflow history but cannot support an editor persistence rule.
- **Redistribution:** Only procedural timing, synthetic-edit intent, and derived
  disk observations are recorded. No project contents are committed.

#### `MZ-1.10.0-SAVE-WHILE-OPEN-CLOSE-2026-08-01`

- **Kind:** Controlled two-phase experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable Basic-project copy; editor fully quit before setup.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-save-close-probe/` in the primary integration
  checkout.
- **Accessed/observed:** Saved and inspected while open, then quit and inspected
  again on 2026-08-01.
- **Procedure:** With MZ initially quit, the user copied the exact baseline,
  opened only the experiment marker, changed Game Title to contributor-created
  `Tilewright Save Close Probe 001`, invoked Save Project, and kept MZ and that
  project open without switching, playtesting, closing, or saving again.
  Read-only phase-one inspection compared complete manifests and contents. The
  user then quit MZ without another save or edit; no prompt appeared. Phase two
  repeated a whole-tree content fingerprint plus selected file hashes, sizes,
  and modification times.
- **Summary:** While MZ remained open, the experiment already differed from
  baseline at exactly `data/System.json`, `index.html`, and `package.json`, with
  no path-count change. Structural changes matched the prior title experiment:
  `System.gameTitle`, numeric `System.versionId`, `package.window.title`, and
  the HTML title. The marker remained unchanged. The phase-one experiment tree
  contained 1,313 files and had content fingerprint
  `85791960807895f875061075b5b727203a0cd5e9184380477a72af60ad57434f`.
  After quitting without a prompt, the file count and fingerprint were exactly
  the same; hashes, sizes, and modification times for the three changed files
  and marker were also unchanged. Closing performed no observable filesystem
  write in this controlled workflow.
- **Inference:** The earlier apparent need to close was caused by an unresolved
  workflow or active-copy issue, not a general close requirement. This is high
  confidence as a rejection of that hypothesis for the controlled title edit,
  but it does not establish transaction guarantees for every editor action.
- **Redistribution:** Only the contributor-created title, project-relative
  paths, structural fields, counts, derived hashes, timing, and behavior are
  committed. No project contents or vendor assets are included.

#### `MZ-1.10.0-RESOURCE-IMPORT-PICTURE-2026-08-01`

- **Kind:** Controlled three-phase experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable Basic-project copy; editor fully quit before setup.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-resource-import-picture/`; contributor source
  `.local-research/contributor-assets/tilewright-resource-probe.png` in the
  primary integration checkout. These ignored, user-owned paths are locators,
  not committed fixtures.
- **Accessed/observed:** Imported and inspected before save, inspected after
  Save Project while open, then quit and inspected again on 2026-08-01.
- **Procedure:** With MZ initially quit, the user duplicated the exact Basic
  baseline, opened only the experiment marker, opened Tools → Resource Manager,
  selected `img/pictures`, imported the contributor-created probe, closed
  Resource Manager, and kept the project open without invoking Save Project.
  Read-only inspection compared complete paths and contents. The user then
  invoked Save Project and kept the project open for a second comparison, then
  quit without another edit or save for the final comparison.
- **Synthetic source provenance:** The source is a contributor-created 68-byte
  PNG containing one 1×1, 8-bit grayscale-plus-alpha pixel (PNG color type 4),
  with only `IHDR`, `IDAT`, and `IEND` chunks. Its SHA-256 is
  `431ced6916a2a21a156e38701afe55bbd7f88969fbbfc56d7fe099d47f265460`.
  It was made solely for this experiment and contains no vendor or third-party
  project material.
- **Pre-save summary:** The experiment contained 1,314 files and 30
  project-relative directories, versus 1,313 files and the same 30 directories
  in the baseline. The only added path was
  `img/pictures/tilewright-resource-probe.png`; no existing file content
  differed. Source and destination had identical bytes, SHA-256, 68-byte size,
  mode `0644`, and modification time `2026-08-01T17:43:03-10:00`. The copied
  destination had its own later creation/change time, so Resource Manager did
  not merely reference the external path.
- **Post-save summary:** The only additional content delta was numeric
  `data/System.json` `/versionId`, from `34133583` to `40814977`. The imported
  PNG remained unchanged. `game.rmmzproject`, `data/System.json`,
  `package.json`, and `index.html` all had modification time
  `2026-08-01T17:46:36-10:00`; only `System.json` differed in bytes from the
  baseline. Because pre-save metadata for those unchanged-byte files was not
  captured, Save Project rewriting or touching them is an inference rather
  than an established content contract.
- **Post-close summary:** Quitting without a prompt left 1,314 files and 30
  project-relative directories. The complete relative-path/content fingerprint
  (SHA-256 over sorted `relative-path NUL file-SHA-256 LF` records) was
  `b597b593901ac5e539eaf1564fe17ba744731eb1193cf3a1f017c755edde85ac`
  both immediately after Save Project and after quit; selected hashes, sizes,
  modes, and modification times also remained fixed. Clean close made no
  observable follow-up write.
- **Evidence limits:** This establishes copying, placement, byte fidelity, and
  timing for one valid PNG imported to one existing standard directory. It does
  not establish accepted formats, validation, collision/overwrite behavior,
  case or Unicode normalization, nested placement, reference updates, deletion,
  deployment handling, cross-platform metadata, or other-version behavior.
- **Redistribution:** Only the contributor-created filename and asset
  characteristics, project-relative destination, scalar delta, counts, hashes,
  timestamps, procedure, and derived comparisons are committed. Neither the
  research project nor the PNG is added as fixture data.

#### `MZ-1.10.0-RESOURCE-DELETE-PICTURE-2026-08-01`

- **Kind:** Controlled three-phase experiment
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the completed contributor-asset import experiment; editor
  fully quit before setup.
- **Locator:** Source
  `.local-research/mz-1_10_0-resource-import-picture/`; experiment
  `.local-research/mz-1_10_0-resource-delete-picture/` in the primary
  integration checkout. Both are ignored, user-owned evidence locators rather
  than committed fixtures.
- **Accessed/observed:** Deleted and inspected before save, inspected after Save
  Project while open, then quit without a prompt and inspected again on
  2026-08-01.
- **Procedure:** With MZ initially quit, the user duplicated the completed import
  project, opened only the experiment marker, opened Tools → Resource Manager →
  `img/pictures`, selected `tilewright-resource-probe.png`, confirmed deletion,
  closed Resource Manager, and kept the project open without invoking Save
  Project. Read-only inspection captured complete paths, contents, selected
  core-file metadata, and a whole-tree fingerprint. The user then invoked Save
  Project and kept the project open for the second comparison, then quit without
  another edit or save for the final comparison.
- **Pre-save summary:** Relative to the 1,314-file source, the experiment had
  1,313 files and the same 30 project-relative directories. The sole path/content
  difference was absence of
  `img/pictures/tilewright-resource-probe.png`; all remaining files were
  byte-identical. The pre-save relative-path/content fingerprint was
  `e8bb6fa3404af4cdc2e4a6ac71bb08fa897e71330b1f525c4798f42ff002c992`.
- **Post-save content summary:** The asset remained absent. The only existing
  content delta was numeric `data/System.json` `/versionId`, from `40814977` to
  `44010632`; marker, `package.json`, and `index.html` hashes and bytes remained
  fixed. The post-save relative-path/content fingerprint was
  `08338c5cea9b997370a41b05e812deb262f4fb7eb20e6dbadbc101480b148c12`.
- **Post-save metadata summary:** Immediately before save, modification times
  were `2026-08-01T17:46:36-10:00` for the marker, `System.json`, and
  `index.html`, and `2026-08-01T17:54:55-10:00` for `package.json`. Save Project
  advanced both modification and change times on all four to
  `2026-08-01T18:00:34-10:00`, including the three unchanged-byte files. Modes,
  sizes where content stayed fixed, and birth times did not change. This
  establishes a same-byte save touch/rewrite for those files in this workflow;
  it does not reveal whether writes were atomic or in-place.
- **Post-close summary:** The user quit without a save prompt. File/directory
  counts, complete fingerprint, selected hashes, sizes, modes, birth times,
  modification times, and change times all matched the saved-but-open state.
  Clean close made no observable follow-up write.
- **Metadata confound:** Before Save Project, `package.json` had already acquired
  a later same-byte modification time in both the previously closed source
  project (`17:54:16`) and new experiment (`17:54:55`). The sequence suggests a
  launch, automatic-reopen, or explicit-open side effect, but no snapshot exists
  immediately before those actions, so the cause is **unknown** and is not
  attributed to Resource Manager deletion.
- **Evidence limits:** The PNG was not referenced by project data. This does not
  establish reference cleanup or warnings, nested-folder cleanup, undo/recovery,
  collision behavior, deletion of other resource types, platform behavior, or
  other editor versions.
- **Redistribution:** Only the contributor-created filename, project-relative
  path, scalar delta, counts, hashes, timestamps, procedure, and derived
  comparisons are committed. Neither project contents nor the PNG are added as
  fixture data.

#### `MZ-1.10.0-RESOURCE-IMPORT-NESTED-2026-08-01`

- **Kind:** Controlled four-boundary experiment plus user-observed editor UI
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable Basic-project copy; editor fully quit before setup; the same
  contributor-created 68-byte PNG used by the flat import experiment.
- **Locator:** Baseline `.local-research/sources/mz-1_10_0-fresh-basic/`; experiment
  `.local-research/mz-1_10_0-resource-import-nested/`; contributor source
  `.local-research/contributor-assets/tilewright-resource-probe.png` in the
  primary integration checkout. These ignored, user-owned paths are evidence
  locators, not committed fixtures.
- **Accessed/observed:** Prepared and inspected while closed, opened and imported
  without saving, inspected after Save Project while open, then quit without a
  prompt and inspected again on 2026-08-01.
- **Procedure:** With MZ closed, the user duplicated the Basic baseline and
  manually created empty `img/pictures/tilewright-nested/`. Read-only inspection
  captured the pre-open path/content and core-file metadata state. The user
  opened only the experiment marker, opened Tools → Resource Manager, observed
  `img/pictures/tilewright-nested` beneath `img/pictures` in the alphabetically
  presented left-side list, selected it, imported the contributor probe,
  closed Resource Manager, and paused without saving. After inspection, the
  user invoked Save Project and paused while open, then quit without another
  edit or save.
- **Pre-open host metadata:** Finder created or updated `.DS_Store` at the root,
  `img`, and `img/pictures` while the user created the directory. The experiment
  had 1,315 files and 31 project-relative directories versus the source's 1,313
  files and 30 directories. Excluding `.DS_Store`, the new empty directory was
  the only path difference and all standard project files were byte-identical.
  `.DS_Store` contents were not inspected or attributed to MZ.
- **Open-workflow metadata:** Before launch, marker and `package.json`
  modification times were `2026-08-01T13:02:59-10:00` and
  `2026-08-01T14:01:40-10:00`. While the project was open but unsaved,
  `package.json` modification/change times were
  `2026-08-01T18:19:13-10:00`; its bytes, size, and mode remained fixed. The
  marker's change time also advanced to `18:19:13` while its bytes, size, mode,
  and `13:02:59` modification time stayed fixed. Selected `System.json` and
  `index.html` bytes and modification times did not change. This isolates a
  launch/open workflow effect but not which substep or filesystem operation
  causes it.
- **Pre-save import summary:** Resource Manager displayed the manually created
  nested directory and imported the PNG successfully. Disk inspection found the
  exact path
  `img/pictures/tilewright-nested/tilewright-resource-probe.png`. Source and
  destination shared SHA-256
  `431ced6916a2a21a156e38701afe55bbd7f88969fbbfc56d7fe099d47f265460`,
  68-byte size, mode `0644`, modification time
  `2026-08-01T17:43:03-10:00`, and macOS birth time; the destination had its own
  later change time. Excluding `.DS_Store`, the imported asset was the sole
  standard path/content addition. The experiment had 1,316 files and 31
  project-relative directories. Its pre-save standard-content fingerprint was
  `4ec7858600e20d4bd4ca4ac26c0f477389f877b186f1fb9881386cf4334ee194`.
- **Post-save summary:** The nested asset remained byte- and metadata-identical
  to its pre-save state. The only standard existing-file content delta was
  numeric `data/System.json` `/versionId`, from `34133583` to `11445413`.
  Marker, `System.json`, `package.json`, and `index.html` modification/change
  times all advanced to `2026-08-01T18:46:42-10:00`; the three non-System files
  retained identical bytes, and all four retained their modes and birth times.
  The standard-content fingerprint excluding `.DS_Store` became
  `a603da33d59296bef5867d91e53ca089849f21ffbcd99c582edd44a0155dd15d`.
- **Post-close summary:** The user quit without a prompt. File/directory counts,
  standard-content fingerprint, selected hashes, sizes, modes, birth times,
  modification times, and change times all matched the saved-but-open state.
  Clean close made no observable follow-up write.
- **Evidence limits:** The UI list appeared alphabetical in this observation,
  but that is presentation behavior, not an on-disk order or discovery
  requirement. Only one manually created subdirectory level, one standard root,
  and one valid ASCII-named PNG were tested. One later controlled-input
  experiment covers this PNG's Show Picture reference and deletion; greater
  depth, invalid names, symlinks, other formats/references, platforms, and
  versions remain unknown.
- **Redistribution:** Only the contributor-created directory/filename and asset
  characteristics, project-relative path, scalar delta, counts, hashes,
  timestamps, procedure, and derived comparisons are committed. Neither project
  contents, `.DS_Store`, nor the PNG are added as fixture data.

#### `MZ-1.10.0-REFERENCE-NESTED-PICTURE-2026-08-01`

- **Kind:** Controlled reference-creation experiment plus user-observed editor
  UI
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  disposable copy of the completed nested-import experiment; contributor-owned
  event name and synthetic PNG.
- **Locator:** Source `.local-research/mz-1_10_0-resource-import-nested/`;
  experiment
  `.local-research/mz-1_10_0-resource-reference-nested-picture/` in the primary
  integration checkout. Both are ignored, user-owned evidence locators rather
  than committed fixtures.
- **Accessed/observed:** Prepared and inspected while closed, edited and saved
  while open, then closed and inspected again on 2026-08-01. Whether a close
  prompt appeared was not reported.
- **Procedure:** With MZ closed, the user duplicated the completed nested-import
  project. Read-only inspection confirmed an exact 1,316-file,
  31-project-relative-directory copy with no symlinks and standard-content
  fingerprint
  `a603da33d59296bef5867d91e53ca089849f21ffbcd99c582edd44a0155dd15d`
  excluding `.DS_Store`. The user opened only the experiment marker, created one
  event on `Map001` named `Tilewright Picture Reference 001`, added one Show
  Picture command for picture number 1, selected the nested synthetic PNG, left
  other event/command values at editor defaults, invoked Save Project, and kept
  the editor open for inspection before closing without further edits.
- **Selection UI observation:** In “Select an Image,” immediately below
  `(None)`, the left-side picture list showed an expandable
  `tilewright-nested`; expanding it showed `tilewright-resource-probe`. The user
  reported no warning or unexpected behavior. This is UI presentation evidence,
  not a filesystem-order contract.
- **Serialized map summary:** The source `Map001.json` `events` array was empty.
  The saved copy contains `[null, event]`, with the new event at array index and
  numeric `id` 1, contributor name, editor-selected coordinates `(2, 2)`, and
  one page. Its command list contains the user-created Show Picture command
  (`code` 231, `indent` 0) followed by the terminating empty command (`code` 0).
  The Show Picture parameters are
  `[1,"tilewright-nested/tilewright-resource-probe",0,0,0,0,100,100,255,0]`.
  The nested asset string is parameter index 1, uses `/`, is relative beneath
  `img/pictures`, and omits `.png`. The user action establishes the command-code
  association in this record; it does not establish a general command schema.
- **Complete content delta:** Excluding `.DS_Store`, exactly
  `data/Map001.json` and `data/System.json` changed. `System.json` changed only
  numeric `/versionId`, from `11445413` to `55115205`. The marker,
  `package.json`, `index.html`, nested PNG, and every other standard path/content
  remained fixed. The resulting standard-content fingerprint was
  `c6e9f453818548dfce8d19df2c90e859026ae2c273c578875734419704cd57af`.
- **Save/close metadata:** Save Project assigned modification/change time
  `2026-08-01T18:59:38-10:00` to the marker, `System.json`, changed
  `Map001.json`, `package.json`, and `index.html`; modes and birth times stayed
  fixed. Closing made no subsequent change to the fingerprint, serialized
  reference, selected hashes, sizes, modes, birth times, modification times, or
  change times.
- **Evidence limits:** This proves only one Show Picture reference representation
  and event allocation from an empty `events` array. It does not prove schemas,
  reference bases, separators, extension omission, ID allocation, runtime
  resolution, or broken-reference behavior for other fields, commands, assets,
  maps, platforms, or versions.
- **Redistribution:** Only the contributor-created names, project-relative path,
  structural JSON paths/types/scalars, command parameters, counts, hashes,
  timestamps, procedure, and derived comparisons are committed. Neither project
  contents nor the PNG are added as fixture data.

#### `MZ-1.10.0-DELETE-REFERENCED-PICTURE-2026-08-01`

- **Kind:** Controlled Resource Manager experiment against a contributor-owned
  synthetic PNG and controlled event record
- **Version/environment:** RPG Maker MZ 1.10.0 on macOS 26.6 build 25G72;
  Codex-managed isolated project copy; no playtest, plugin execution, or network
  access.
- **Locator:** Immutable input
  `.local-research/sources/mz-1_10_0-fresh-basic/`; owned experiment
  `.local-research/workspaces/tilewright-mz-layout-20260801T061033Z-9907/project/`;
  provenance and ownership manifest at sibling `OWNERSHIP.md`. These ignored
  paths are local evidence locators, not committed fixtures.
- **Accessed/observed:** Workspace created, edited, exercised, saved, closed, and
  inspected on 2026-08-01.
- **Controlled input:** The source was copied with `cp -a` after canonical-path,
  regular-directory, and no-symlink checks. A contributor-authored base64 recipe
  produced a 68-byte 1×1 grayscale-plus-alpha PNG with SHA-256
  `431ced6916a2a21a156e38701afe55bbd7f88969fbbfc56d7fe099d47f265460`
  at `img/pictures/tilewright-nested/tilewright-resource-probe.png`. The
  pre-open standard-content fingerprint excluding `.DS_Store` was
  `4ec7858600e20d4bd4ca4ac26c0f477389f877b186f1fb9881386cf4334ee194`.
- **Reference setup and limitation:** The map event was inserted as controlled
  JSON using the exact field/value shape and Show Picture parameters established
  by `MZ-1.10.0-REFERENCE-NESTED-PICTURE-2026-08-01`, with only its
  contributor-created event name changed. The editor noticed the external
  modification, the operator accepted Reload, and Event Searcher found event ID
  1 at `(2,2)`. This demonstrates that MZ accepted and indexed the controlled
  input, but it is not a second independent Event Editor serialization sample.
- **Deletion observation:** Resource Manager showed
  `img/pictures/tilewright-nested` and the selected
  `tilewright-resource-probe.png`. Activating Delete removed the PNG immediately
  with no warning, confirmation, or refusal visible. Before Save Project, the
  file was absent, the empty nested directory remained, and the Show Picture
  parameter still equaled `tilewright-nested/tilewright-resource-probe`.
  `Map001.json` retained SHA-256
  `c62efc441762f4556e707adda45b06fdd51f4f2287af894c99430da837718c6d`.
- **Save/close boundary:** Save Project left that map hash and dangling string
  unchanged. Standard file content changed only at `System.versionId`, from
  `34133583` to `54461537`; the resulting `System.json` SHA-256 was
  `15b4c1383756e206ddae490873eb7f0e848eaee1604503347913f3ece23c52a1`.
  The marker and `package.json` retained SHA-256 values
  `d999ef7df90cc897cccd4bfa8f86811224e61a8c61b6fe7e13f8321c7932e97d`
  and `37f1c9fc562abf7c2ca9620e35adc15b13cab52209f5e29c7627be2d6d508eaf`.
  Closing the saved window presented no accessible prompt and made no later
  content change. A complete final comparison with the immutable input differed
  only in `data/Map001.json`, `data/System.json`, and the additional empty
  nested directory.
- **Confidence:** High for this selected resource, command string, editor
  version, platform, and three captured boundaries. Medium for broader
  reference-lifecycle semantics because the event in this copy was controlled
  input corroborated by an earlier editor-created event, not independently
  authored through Event Editor during this experiment.
- **Evidence limits:** The result does not establish runtime missing-resource
  behavior, Event Editor diagnostics after reopening the command, automatic
  cleanup for other command/reference types, recovery or undo semantics,
  directory cleanup for other depths, or behavior on other platforms/versions.
- **Redistribution:** Only the contributor-authored recipe description, hashes,
  relative paths, structural values, procedure, UI outcome, and derived
  comparisons are committed. No project, vendor content, or PNG is committed.

#### `MZ-1.10.0-BASIC-LATER-METADATA-2026-08-01`

- **Kind:** Observation
- **Version/environment:** The authorized Basic MZ 1.10.0 research project on
  macOS 26.6 build 25G72, observed after the original fresh-project manifest and
  subsequent user-driven copy experiments.
- **Locator:**
  `.local-research/sources/mz-1_10_0-fresh-basic/.DS_Store` in the primary
  integration checkout.
- **Accessed/observed:** 2026-08-01; filesystem modification timestamp observed
  as 2026-08-01T14:11:18-10:00.
- **Procedure:** Compared the current immediate-entry manifest with the earlier
  recorded fresh baseline and checked presence across the four fresh research
  roots. Contents were not inspected.
- **Summary:** `.DS_Store` is now present only in the Basic root and was absent
  from its original fresh baseline. Host-OS creation is a plausible inference,
  not a controlled or documented MZ behavior.
- **Redistribution:** Only the incidental filename, metadata, and uncertainty are
  recorded; no file contents are committed.

### Established findings

- **Documented:** Official sources identify a useful but not necessarily
  complete high-level inventory: `data`, `audio`, `img`, `icon`, `fonts`,
  `js/plugins`, and `effects`, plus the standard database/map filename families
  summarized in [`project-layout.md`](project-layout.md).
- **Observed:** Four MZ 1.10.0 New Game choices share the same 12 immediate root
  names and 30 child directories, while their map counts and selected template
  data/plugin contents differ.
- **Observed:** MZ 1.10.0 uses plural `Skills.json`, `Items.json`, and
  `plugins.js`; the official singular spellings are not observed.
- **Documented:** `data` is extension-capable; it is not a closed set of stock
  filenames.
- **Documented:** Asset subfolders and user-imported resources make the asset
  trees open-world; PNG, Ogg Vorbis, and WebM/MP4 are official standard roles.
- **Observed:** Resource Manager copied one contributor-created PNG immediately
  into `img/pictures` before Save Project, preserving its filename, bytes, size,
  mode, and modification time. Save Project later changed only
  `System.versionId` in existing file content; a clean quit made no further
  change.
- **Observed:** Resource Manager immediately deleted that unreferenced PNG in a
  separate copy before Save Project, retaining the existing directory and every
  other file byte. The later save changed only `System.versionId` content but
  advanced metadata on four core files; a no-prompt quit made no further change.
- **Observed:** Resource Manager discovered one manually created directory under
  `img/pictures` and imported the contributor PNG at the exact nested relative
  path. The copy preserved bytes, filename, size, mode, modification time, and
  macOS birth time. The UI list appeared alphabetical, but no disk-order
  implication follows.
- **Observed:** One Show Picture command references that PNG as
  `tilewright-nested/tilewright-resource-probe`: a forward-slash path relative to
  `img/pictures`, without the `.png` extension. This is command-specific evidence,
  not a universal asset-reference rule.
- **Observed, controlled-input limitation:** Resource Manager deletes that
  selected PNG without warning, keeps the empty nested directory, and leaves
  the Show Picture string dangling before and after save and close. Runtime
  behavior and other reference types remain unknown.
- **Observed:** Generated resource trees also include effect-specific extensions
  and stem-paired tileset `.txt` companions; discovery cannot treat an asset
  directory as containing only the media format named by its high-level role.
- **Documented and observed:** Fonts are selected by project data; all four
  fresh 1.10.0 projects contain the same two WOFF files and one PNG under
  `icon`, but requiredness and broader accepted formats remain unknown.
- **Observed:** Every inspected data file is valid UTF-8 JSON with the same
  no-BOM, no-CR, no-final-newline byte properties.
- **Observed:** The default-option Basic Web deployment preserves every
  non-marker file path, omits the marker and empty `movies` directory, and
  changes only one string-valued runtime configuration field in `package.json`.
- **Observed:** A persistent Game Title edit changes three existing files but
  no paths: `data/System.json` (`gameTitle` and `versionId`), `package.json`
  (`window.title`), and the HTML title in `index.html`. The marker is unchanged.
- **Observed:** Adding one top-level map to the Basic project creates
  `Map002.json`, appends index/ID 2 to `MapInfos.json`, and changes only
  `System.editMapId` and `System.versionId` elsewhere. The editor Name is in
  `MapInfos.json`; the map file has a separate blank `displayName` and no
  embedded top-level ID.
- **Observed:** Deleting that highest map reverses the map-file and map-info
  additions, trims the array rather than leaving a trailing hole, resets
  `System.editMapId` to 1, and changes `System.versionId`.
- **Observed:** Sequential creation allocates `Map003.json` with index/ID/order
  3. The two default-settings added map files are byte-identical despite their
  distinct Names in `MapInfos.json`.
- **Observed:** Deleting middle ID 2 leaves JSON `null` at array index 2,
  preserves file/index/ID 3, and compacts the surviving entry's `order` to 2.
- **Observed:** The next creation reuses that only hole as file/index/ID 2 while
  assigning it `order` 3; the preserved ID 3 remains at `order` 2.
- **Observed:** Reordering the two top-level maps swaps only their `order`
  values; array indices, IDs, parent IDs, filenames, and map bytes remain fixed.
- **Observed:** Creating one child allocates ID/file 4, sets `parentId` to the
  parent ID 3, inserts its `order` immediately after the parent, expands that
  parent, and shifts the following top-level order without rewriting map files.
- **Observed:** Reparenting that child changes `parentId` and compact order but
  not IDs, files, contents, or either parent's existing `expanded` value.
- **Observed:** In a controlled exact-copy title edit, Save Project persisted
  all expected changes while MZ remained open, and quitting without a prompt
  made no additional filesystem change. Earlier apparent close-required behavior
  was not reproduced and remains unexplained workflow history.
- **Observed, origin inferred:** Incidental `.DS_Store` appeared later in the
  Basic root; unknown root entries cannot be attributed or rejected by name
  alone.

### Inferences

- **Inferred, high confidence:** Discovery should recognize known entries while
  retaining unknown entries as unknown, not rejecting or deleting them.
- **Inferred, medium-high confidence:** Deployment output and authoring projects
  can share data/assets, so runtime-shaped files are not root identity signals.
- **Inferred, high confidence:** The observed standard inventory is useful for
  MZ 1.10.0 diagnostics after candidate recognition, but presence in all four
  templates still does not prove editor-requiredness.

### Risks and unknowns

- **Unknown fields:** All JSON extension fields and plugin-defined data remain
  unbounded; standard filenames do not imply standard-only contents.
- **Ordering/encoding:** UTF-8 validity and absent BOM/CR/final newline are
  observed. The controlled title save preserved all other bytes in the two
  changed JSON files after the changed scalars were redacted, but broader JSON
  key order, array order, whitespace preservation, Unicode normalization, and
  serializer tolerance remain unknown.
- **Ordering:** For one confirmed top-level pair, smaller positive `order`
  appears first in the editor tree, and swapping visible order swaps only those
  scalar values. One child creation inserts the child immediately after its
  parent in compact order and shifts the following top-level entry. Deeper
  traversal, invalid/duplicate values, and other versions remain unknown.
- **Hierarchy:** One creation and one reparent show a child immediately after
  its parent in compact order. `parentId` changes with reparenting, while the
  observed `expanded` values do not necessarily follow whether a map currently
  has a child. Cycles, depth limits, and parent deletion remain unknown.
- **Save timing:** One controlled title edit confirms immediate observable disk
  state after Save Project and no additional clean-close write. One resource
  import also appears on disk before Save Project, while the subsequent save
  changes `System.versionId` content and clean close changes nothing further.
  One resource deletion behaves the same at content boundaries and additionally
  establishes that Save Project can advance modification/change times on four
  core files whose bytes do not all change. Metadata-only `package.json` changes
  also occurred during later project setup but cannot yet be attributed to
  launch, reopen, open, or copying. Dirty close, project switching, other edit
  types, delayed/failed I/O, and other versions remain unknown.
- **Open timing:** A closed pre-open snapshot followed by the nested import
  isolates same-byte metadata changes during the launch/open workflow:
  `package.json` modification/change times and marker change time advanced before
  Save Project. Exact attribution among launch, automatic reopen, explicit open,
  editor logic, and filesystem mechanisms remains unknown.
- **Identifiers/references:** Three-digit map filenames, index-matching non-null
  map-info IDs, and corresponding files are observed through ID 189. The
  controlled addition independently produced filename/index/ID 2 while the map
  file itself lacked an embedded ID; deletion of that highest ID trimmed the
  array, while middle deletion left a null hole and preserved later ID 3.
  `order` compacted independently. The next creation reused the single ID-2
  hole and placed it last by `order`. Multiple-hole selection, larger IDs,
  enforcement, and other references remain unknown. Separately, one Show
  Picture command stores a nested picture reference without the `.png`
  extension. Deleting that selected resource through Resource Manager produces
  no warning and leaves the exact string dangling through Save Project and
  close in one controlled-input copy. Runtime resolution, other command/field
  reference bases, cleanup behavior, and normalization remain unknown.
- **Version/plugin differences:** One fresh-project version matrix and one Web
  deployment now exist for 1.10.0. Other editor versions, targets, deployment
  options, migration residue, and arbitrary plugins remain unknown.

### Next experiment

The referenced-picture deletion question is resolved for one controlled MZ
1.10.0 input: Resource Manager silently removes the file and preserves both the
empty nested directory and dangling Show Picture string. The next smallest
layout experiment should use a fresh owned copy to test one deeper asset
subdirectory or one parent-map deletion, whichever is needed before the next
loader boundary. Also test multiple map holes, marker casing on a case-sensitive
filesystem, remaining deployment targets, and another named MZ version at or
above 1.10.0. Never commit vendor material.

### Implementation implications

- **Safe now and implemented experimentally:** Inventory known immediate-root
  names and known `data` filename families relative to a caller-authorized
  directory capability; classify other immediate `data/*.json` paths only as
  extension candidates; report all other entries as unknown without reading,
  executing, following symlinks, moving, or deleting them.
- **Not justified:** Treating the known inventory as exhaustive; loading every
  JSON file as stock data; generalizing observed casing, padding, encoding,
  ordering, identifier equality, or reference validity to other versions;
  assuming the observed Web deployment layout applies to other targets,
  projects, options, or versions.
- **Fixtures/tests needed:** Prefer generated temporary trees for path discovery.
  If committed, use filename-only synthetic fixtures with the provenance record
  required by [`fixtures/README.md`](../../../fixtures/README.md). Add an
  editor-created fixture only after the contributor documents creation steps,
  version, redistribution rights, sanitization, and expected fidelity.

Use stable, descriptive IDs such as `mz-project-detection-001`. A completed
investigation may move to a focused sibling document; keep its index row here.

## `mz-map-catalog-001`: What is the smallest evidenced typed map catalog?

- **Status:** Active
- **Behavior depending on this:** Read-only typed map listing, display order,
  parent hierarchy, and contextual project findings.
- **Scope:** `data/MapInfos.json` in RPG Maker MZ 1.10.0. Later target
  versions, malformed-input editor behavior, mutation, and persistence remain
  outside this investigation.
- **Last updated:** 2026-08-06

### Evidence ledger

The claim-level ledger, bounded typed contract, fixture implications, and next
experiments are maintained in
[`map-catalog.md`](map-catalog.md#evidence-ledger). That document composes the
existing official script-reference record and controlled map lifecycle records
with `MZ-1.10.0-MAP-INFOS-SHAPE-AUDIT-2026-08-06`.

### Evidence record: `MZ-1.10.0-MAP-INFOS-SHAPE-AUDIT-2026-08-06`

- **Kind:** Read-only aggregate shape audit.
- **Version/environment:** Four authorized, user-owned projects created with
  RPG Maker MZ 1.10.0 on the macOS environment recorded by
  `MZ-1.10.0-FRESH-4-2026-08-01`; audited with `jq` 1.8.1 on 2026-08-06.
- **Procedure:** Verified the canonical ignored research root, regular-file
  types, sizes, and absence of source symlinks. Queried only root/field JSON
  kinds, decoded property-name sets, counts, integer relationships, ID/index
  equality, order uniqueness/compactness, and parent-reference existence.
- **Observed:** Four array roots contain 196 object records and four null index
  zero entries. Every record has numeric `id`, `order`, and `parentId` plus
  string `name`; IDs equal indices, positive orders are unique and compact per
  document, and every nonzero parent resolves. Five records have an additional
  boolean `quick` field. Some scroll coordinates are fractional.
- **Limits:** The audit does not establish editor validation, alternate numeric
  lexeme tolerance, field semantics beyond existing controlled evidence,
  behavior above ID 999, or other MZ versions. Duplicate decoded properties
  were not asserted absent and remain an explicit typed-view refusal case.
- **Redistribution:** No project value, map name, raw document, excerpt,
  per-project manifest, or proprietary content is retained. Only aggregate
  derived observations and the safe procedure are recorded.

### Implementation implications

The first typed projection may use `id`, `name`, `order`, and `parentId` while
retaining every other field in the raw lossless document. It must refuse
ambiguous or malformed required fields, distinguish structural refusal from
contextual relationship findings, and make no mutation or editor-validation
claim. The accepted architecture is recorded in
[ADR 0007](../../decisions/0007-experimental-map-catalog-projection.md).

### Next experiment

Test parent deletion or multiple ID holes in a disposable authorized copy when
one of those behaviors is needed for a compatibility claim. Observe another
named MZ version at or above 1.10.0 before generalizing the contract.

## `mz-map-summary-001`: What is the smallest useful selected-map summary?

- **Status:** Active
- **Behavior depending on this:** Read-only inspection of one catalog-selected
  map's basic metadata, dimensions, tileset scalar, and opaque event count.
- **Scope:** Three-digit `data/MapNNN.json` documents in four RPG Maker MZ
  1.10.0 projects. Tile semantics, event contents, validation, mutation,
  persistence, malformed-input editor behavior, and later versions remain
  outside this investigation.
- **Last updated:** 2026-08-06

### Evidence ledger

The claim-level ledger, bounded contract, fixture implications, and
remaining experiments are maintained in
[`map-summary.md`](map-summary.md#evidence-ledger).

### Evidence record: `MZ-1.10.0-MAP-SUMMARY-SHAPE-AUDIT-2026-08-06`

- **Kind:** Read-only aggregate shape and cross-file audit.
- **Version/environment:** The four authorized, user-owned MZ 1.10.0 projects
  recorded by `MZ-1.10.0-FRESH-4-2026-08-01`; `jq` 1.8.2 on arm64 macOS 26.6
  build 25G72.
- **Procedure:** Verified the canonical ignored research root, regular map-file
  types and sizes, and absence of source symlinks. Queried decoded root property
  names, JSON kinds, counts, integer relationships and ranges, array lengths,
  event entry kinds and ID/index equality, and map-to-tileset reference
  existence. The comparison emitted aggregate values only.
- **Observed:** All 196 map documents have object roots and one shared set of 25
  decoded top-level property names. `displayName` is always a string and is
  empty in this fresh-project corpus. Widths and heights are positive integers
  in ranges 17–200 and 13–200. Positive tileset IDs range from 1 through 4 and
  all resolve to ID/index-consistent `Tilesets.json` records. Every integer tile
  array contains exactly six values per map-area cell. Event arrays contain
  1,555 objects and 290 null slots, no other entry kinds, and every object ID
  equals its array index.
- **Limits:** Decoded duplicate properties were not established absent. The
  audit does not establish editor validation, dimension or tileset limits,
  tile-layer meaning, event semantics, map mutation fidelity, or other-version
  behavior.
- **Redistribution:** No project path, display name, note, event body, tile
  value, raw document, excerpt, digest, or per-project manifest is retained.
  Only aggregate derived observations and the safe procedure are recorded.

### Implementation implications

The accepted next typed slice will expose a catalog-selected map's two decoded
names, evidenced path, positive dimensions, positive tileset scalar, and opaque
event object count. It must retain every other field in the raw snapshot,
refuse ambiguous required structure, avoid interpreting tile arrays or event
bodies, and make no validation or editor-compatibility claim. The accepted
architecture is recorded in
[ADR 0008](../../decisions/0008-experimental-selected-map-summary.md).

### Next experiment

Change only one map's Display Name and dimensions in a disposable authorized
copy, save and reopen it, and compare the exact persisted fields. A separate
tileset change can then isolate that reference. Event structure should remain a
separate investigation before any event fields are typed.

## Investigation template

Copy this section for each bounded question.

```markdown
## <ID>: <question>

- **Status:** Proposed | Active | Complete | Blocked
- **Behavior depending on this:** <parser, validator, API, or writer behavior>
- **Scope:** <file, field, editor/runtime versions, and explicit exclusions>
- **Last updated:** YYYY-MM-DD

### Evidence ledger

| Claim | Evidence | Classification | Confidence and rationale | Unresolved alternatives |
| --- | --- | --- | --- | --- |
| <one falsifiable claim> | <source or observation ID> | Documented / Observed / Inferred / Unknown | <high/medium/low plus why> | <what else could explain it> |

### Evidence records

#### <source-or-observation-id>

- **Kind:** Official documentation | Observation | Controlled experiment | Runtime behavior | Community corroboration
- **Version/environment:** <exact version and relevant platform/tooling>
- **Locator:** <URL, document section, fixture path, or reproducible procedure>
- **Accessed/observed:** YYYY-MM-DD
- **Summary:** <paraphrase or minimal original observation>
- **Redistribution:** <why any committed material is safe>

### Established findings

- <documented or observed conclusion and its limits>

### Inferences

- <inference, reasoning, confidence, and what would falsify it>

### Risks and unknowns

- **Unknown fields:** <risk>
- **Ordering/encoding:** <risk>
- **Identifiers/references:** <risk>
- **Version/plugin differences:** <risk>

### Next experiment

<The smallest controlled change or observation that resolves the most important unknown.>

### Implementation implications

- **Safe now:** <read, parse, validate, expose raw data, etc.>
- **Not justified:** <types, normalization, or writes that evidence does not support>
- **Fixtures/tests needed:** <minimal fixture and assertions>
```

## Ledger quality rules

- One row states one falsifiable claim.
- A classification describes evidence strength, not implementation priority.
- Confidence includes a reason and the important limitation.
- Evidence locators must let another maintainer reproduce or re-check the claim.
- Observations record the relevant version and the controlled action.
- Conflicts and unknowns remain visible.
- A parser accepting one example does not establish semantics or compatibility.
- Writer behavior requires stronger evidence and preservation tests than
  read-only inspection.
