# RPG Maker MZ project detection and high-level layout

This document defines the smallest project-discovery contract supported by the
evidence collected on 2026-08-01. It is a research result, and the current
implementation is **Experimental**. Claim-level sources, versions, confidence,
conflicts, and experiments are recorded in the
[research ledger](research-ledger.md). The breadth and remaining gaps are
summarized separately in the
[project-layout coverage matrix](project-layout-coverage.md).

## Scope and evidence boundary

Tilewright's maintained target begins at RPG Maker MZ 1.10.0 under
[ADR 0002](../../decisions/0002-rpg-maker-mz-version-floor.md). Older releases
are outside maintainer-led research and are not blockers for this contract.
Newer releases remain inside the intended range but require named evidence and
tests before any behavior is called supported.

- **Documented:** RPG Maker MZ's official help tells a user to open a project by
  selecting `Game` (or `game.rmmzproject`) inside the project folder.
- **Documented:** Official conversion and reference material names a useful
  subset of project-relative directories and data files.
- **Observed:** Four user-authorized projects created with RPG Maker MZ 1.10.0
  on macOS 26.6 cover the Basic, Tutorial, Map Set, and HD Layout choices in the
  New Game flow. They were inspected read-only on 2026-08-01.
- **Unknown:** Most official online help pages do not state an editor version.
  The official script-reference suite identifies itself as version 1.0.0, but
  that does not establish compatibility with every editor version.

The documents below therefore distinguish a candidate authoring root from a
complete, parseable, compatible, or supported project.

## Minimum root-recognition contract

### Detection signals

| Signal | Evidence status | Contract status | Discovery role |
| --- | --- | --- | --- |
| An immediate regular file named `game.rmmzproject` | Role **documented**; exact lowercase spelling **observed** in all four generated MZ 1.10.0 projects | **Required** by the portable candidate contract; other versions remain **unknown** | The only portable positive authoring-root signal currently justified. Its containing directory may be reported as an **MZ project candidate**. |
| On-disk `Game.rmmzproject` with capital `G` | MZ 1.10.0 editor acceptance **observed** on a macOS volume where lowercase lookup resolves the same entry | **Platform-limited case variant**, not a portable spelling rule | Report the exact spelling and a case-variant candidate diagnostic. Do not infer editor acceptance on a case-sensitive filesystem or silently normalize the path. |
| Marker contents `RPGMZ 1.10.0` as 12 ASCII bytes without a final newline | Generated form **observed** identically in all four MZ 1.10.0 projects; MZ 1.10.0 also **observed** opening unrevised `RPGMZ 1.10.1` | **Optional form corroboration**, not trusted version identity | May report the generated 1.10.0 form, but must not infer the running editor version, compatibility, or a cross-version grammar. |
| A zero-byte `game.rmmzproject` | Editor rejection **observed** in a one-change MZ 1.10.0 experiment | **Known rejected form for MZ 1.10.0**, not a general marker grammar | Preserve the path-level candidate evidence but report that the observed editor version rejected the marker; do not call the project openable or valid. |
| `data/System.json` or `data/MapInfos.json` | Location/purpose **documented** and presence **observed** in all four projects; editor-requiredness **unknown** | **Optional corroboration** | Corroborating inventory or health diagnostics only, not identity requirements. |
| `data`, `img`, `audio`, `fonts`, `icon`, `js`, `effects`, `css`, or `movies` | Presence **observed** in all four projects; editor-requiredness **unknown** | **Optional corroboration** | Never sufficient alone; deployed games and incomplete/copied trees may contain them. |
| `index.html`, executable/application bundles, package metadata, or other runtime-shaped files | Presence in authoring roots and the MZ 1.10.0 Web deployment **observed**; no complete official contract found | **Not accepted** as an authoring signal | Do not use as MZ authoring-root signals. The observed deployment retained these runtime-shaped paths while omitting the marker. |
| Extra files or directories | Plugin extension data is **documented** | **Permitted**, with contents **unknown** | Do not reject a candidate because unknown entries exist. |

No detection signal has yet been proved version-dependent or version-invariant.
Version dependence among the maintained 1.10.0+ range remains **unknown** until
additional named editor versions are observed. The floor cannot be enforced from
the marker string alone.

### Smallest safe behavior to implement next

A first library change can be a read-only **candidate recognizer**, not a loader
or validator:

1. Accept an explicit directory supplied by the caller; do not search unrelated
   ancestors, descendants, home directories, or sibling projects.
2. Enumerate only immediate entries needed to locate the documented marker.
   Classify immediate marker-entry symlinks without following them. Preserve and
   report the spelling actually found. Capability-based root containment is
   future work.
3. Recognize the exact lowercase `game.rmmzproject` spelling documented by the
   help and observed in all four generated MZ 1.10.0 projects. Enumerate entries
   so the actual spelling is retained even on a case-insensitive filesystem. A
   case variant such as observed `Game.rmmzproject` may produce a case-variant
   candidate diagnostic, not silent normalization or a portable acceptance
   claim. Simultaneous variants remain ambiguous.
4. Require exactly one regular marker candidate. Report missing, non-regular,
   symlinked, or ambiguous marker entries distinctly.
5. Return `candidate`, not `valid`, `compatible`, or `supported`. An exact
   12-byte `RPGMZ 1.10.0` marker may be reported as matching the observed 1.10.0
   form. A zero-byte marker may be reported as a form rejected by the observed
   1.10.0 editor. Because that editor also opens and preserves
   `RPGMZ 1.10.1`, marker text must not be treated as trusted editor-version or
   compatibility metadata. These results do not establish a general grammar.
6. Optionally inventory documented companion paths after recognition, but make
   missing or conflicting paths diagnostics rather than identity failures.
7. Leave every unknown entry untouched and unexecuted. Discovery is read-only.

This behavior deliberately permits false-positive candidates when a marker was
copied. Validation and version support require additional observations, parsers,
and tests.

### Implemented inventory boundary

The core's separate capability-relative inventory is **Experimental**. It takes
a caller-authorized directory handle, recursively reports exact native
project-relative paths and entry kinds, and reports symlinks without following
them. It does not acquire the initial capability, inspect marker or file
contents, parse JSON, or infer validity or compatibility.

Path classification is intentionally narrow and case-sensitive:

- the 12 exact immediate-root names observed across the four MZ 1.10.0 New Game
  projects are known standard root entries;
- the exact standard immediate `data` filenames in this research and the
  three-ASCII-digit `data/MapNNN.json` family are known data paths;
- other immediate `data/*.json` paths are extension candidates because that
  location is documented as extension-capable, but are not called plugin data;
  and
- every other descendant remains unknown.

Known classification is a pathname observation only. It does not assert that an
entry has the expected filesystem kind, contains valid data, is required, or is
compatible with any editor version.

## Evidence-backed high-level layout

The tree is an inventory of documented locations, not a required-file schema.
`?` means presence or exact naming remains unknown.

```text
<project root>/
├── game.rmmzproject              observed 1.10.0 marker: `RPGMZ 1.10.0`
├── index.html                    runtime entry document present in authoring root
├── package.json                  runtime/package metadata present in authoring root
├── css/
│   └── game.css                  observed runtime stylesheet
├── data/                         maps and database/game data; extension-capable
│   ├── Actors.json
│   ├── Classes.json
│   ├── Skills.json               plural spelling observed in all four projects
│   ├── Items.json                plural spelling observed in all four projects
│   ├── Weapons.json
│   ├── Armors.json
│   ├── Enemies.json
│   ├── Troops.json
│   ├── States.json
│   ├── Animations.json
│   ├── Tilesets.json
│   ├── CommonEvents.json
│   ├── System.json
│   ├── MapInfos.json
│   ├── MapNNN.json               three digits observed for IDs 1 through 189
│   └── <plugin-defined>.json ?   officially supported extension point
├── audio/                        bgm, bgs, me, and se asset subdirectories
├── img/                          image assets in purpose-specific subdirectories
│   └── tilesets/                 PNGs plus stem-paired `.txt` files observed
├── icon/                         one PNG observed in each fresh project
├── fonts/                        two identical WOFF files observed per template
├── js/
│   ├── main.js                   observed runtime entry script
│   ├── rmmz_*.js                 observed core-script family
│   ├── libs/                     observed JavaScript/WebAssembly dependencies
│   ├── plugins/                  plugin programs
│   └── plugins.js                observed plugin registration/configuration file
├── effects/                      `.efkefc`, Model/`.efkmodel`, Texture/PNG observed
├── movies/                       observed movie asset directory
└── <unknown entries> ?           preserve and report without rejection
```

### Standard data-file purposes

| Project-relative path or family | High-level purpose | Evidence limit |
| --- | --- | --- |
| `data/Actors.json` | **Documented:** actors controlled by the player | Requiredness and JSON contract are unknown. |
| `data/Classes.json` | **Documented:** actor class characteristics and growth | Requiredness and JSON contract are unknown. |
| `data/Skills.json` | **Documented and observed:** skills and battle/special actions | Plural spelling appears in all four MZ 1.10.0 projects. The conversion page's singular spelling is not observed in this version. |
| `data/Items.json` | **Documented and observed:** possessed/usable items | Plural spelling appears in all four MZ 1.10.0 projects. The conversion page's singular spelling is not observed in this version. |
| `data/Weapons.json` | **Documented:** weapons | Requiredness and JSON contract are unknown. |
| `data/Armors.json` | **Documented:** armor and defensive equipment | Requiredness and JSON contract are unknown. |
| `data/Enemies.json` | **Documented:** enemy characters | Requiredness and JSON contract are unknown. |
| `data/Troops.json` | **Documented:** enemy groups and battle events | Detailed cross-file references are unknown. |
| `data/States.json` | **Documented:** character/enemy states | Requiredness and JSON contract are unknown. |
| `data/Animations.json` | **Documented:** animation database data | MZ-native versus imported MV-compatible records and related assets are not classified here. |
| `data/Tilesets.json` | **Documented and observed:** tileset composition used by maps | All 24 observed records use positive ID/index equality and nonempty editor names; all 196 observed map references resolve. Modes, image slots, flags, notes, lifecycle behavior, and editor enforcement remain outside the bounded catalog. |
| `data/CommonEvents.json` | **Documented:** common event processing | Command encoding and references are outside this investigation. |
| `data/System.json` | **Documented:** system settings plus terms, types, switch names, and variable names | It must not be treated as a proven version/magic file. |
| `data/MapInfos.json` | **Documented and observed:** map information including IDs, names, ordering, and parent relationships in the version 1.0.0 reference; an array layout in all four MZ 1.10.0 projects | Controlled additions, deletion, reuse, and reorder distinguish ID/index from compact `order`. Child creation sets `parentId`, inserts after the parent, and initially expands that parent. Reparenting changes `parentId` and compact order without changing IDs/files or either parent's existing `expanded` value. Multiple children, deeper hierarchy, multiple-hole selection, inconsistent inputs, and other versions remain unknown. Objects in the observed array do not all have identical keys. |
| `data/MapNNN.json` | **Documented and observed:** a map and its map events | All observed map filenames use exactly three digits. The Map Set project contains `Map001.json` through `Map189.json`; every non-null `MapInfos.json` entry has an ID equal to its array index and a corresponding file. Controlled Basic-project additions created `Map002.json` then `Map003.json`; middle deletion removed 2 while preserving 3, and the next creation reused `Map002.json`. Default-settings map files are byte-identical and contain no top-level ID or editor Name. These are observations, not cross-version invariants. |
| Other `data/*.json` | **Documented extension possibility:** official plugin support can load optional JSON | Treat as unknown/plugin-defined, not malformed stock data. Do not execute or reinterpret it. |

### Asset and extension directories

- **Documented:** The official conversion table names `audio`, `fonts`, `icon`,
  `img/system`, and the principal `img` subdirectories for battlebacks,
  characters, enemies, faces, parallaxes, pictures, side-view actors/enemies,
  tilesets, and title images.
- **Documented:** The official asset help gives the purposes and formats of
  those image families. It permits browsable subfolders under asset folders and
  documents PNG image, Ogg Vorbis audio, and paired WebM/MP4 movie roles.
- **Documented:** The deployment help names project `img`, `audio`, and
  `effects` as resource trees whose unused files may be excluded.
- **Observed:** All four MZ 1.10.0 roots have the same 30-child-directory manifest:
  `audio/{bgm,bgs,me,se}`, `css`, `data`, `effects/{Model,Texture}`, `fonts`,
  `icon`, the documented `img` subdirectories, `js/{libs,plugins}`, and
  `movies`.
- **Observed:** Each fresh project contains 120 `.efkefc`, seven `.efkmodel`,
  and 48 PNG files under `effects`. Each also contains 31 PNG and 31 `.txt`
  files under `img/tilesets`; the PNG and text basename stems pair one-to-one.
  The text-file purpose and effect-file semantics were not inspected and remain
  **unknown**.
- **Observed:** In one controlled MZ 1.10.0 Resource Manager import, a
  contributor-created PNG selected for `img/pictures` appeared at the
  same-named project-relative path before Save Project. The copied file retained
  the source bytes, size, mode, and modification time. This is one import
  observation, not a general filename, validation, or metadata contract.
- **Observed:** In a separate controlled copy, Resource Manager immediately
  removed that unreferenced PNG before Save Project without removing the
  existing `img/pictures` directory or changing any remaining file bytes.
  A later controlled-input copy establishes one referenced nested deletion;
  other reference types and cleanup depths remain unknown.
- **Observed:** Resource Manager discovered one contributor-named directory
  manually created beneath `img/pictures` while MZ was closed. It displayed the
  nested path under its parent and imported the synthetic PNG at the exact
  project-relative path. The copied file preserved bytes, filename, size, mode,
  modification time, and macOS birth time. Only one level and one ASCII name
  were tested.
- **Documented but version-limited:** The conversion guide says MZ does not have
  `img/animations` by default; that folder is copied only for deprecated
  MV-compatible animation data in the described migration.
- **Unknown:** These observations establish the fresh-project directory list
  only for the four MZ 1.10.0 New Game choices on the observed platform. They do
  not prove editor-requiredness, cross-version stability, or exhaustive allowed
  contents.

Asset folders are open-world. Discovery must allow nested user-created folders,
retain exact relative paths, and avoid treating a documented format or naming
recommendation as proof that every encountered asset is valid. Conversely, it
must not reject non-image companion files merely because official image prose
names PNG: the generated tileset directory itself contains `.txt` files.

One controlled Show Picture command stores its selected nested PNG as
`tilewright-nested/tilewright-resource-probe`: a `/`-separated string relative
to `img/pictures`, without `.png`. This is **observed** for that command and MZ
1.10.0 only. Discovery must preserve the exact path and filename; broader
loading must not generalize its base directory, separator, extension omission,
case, or normalization rules to other asset-reference fields without evidence.
In a later controlled-input copy, Resource Manager deleted that selected PNG
without warning or refusal, retained the now-empty nested directory, and left
the exact Show Picture string unchanged before Save Project, after save, and
after close. This is **observed** dangling-reference behavior for one MZ 1.10.0
command only; it does not establish runtime resolution or cleanup rules for
other references.

### Project lifecycle and incidental contents

- **Documented:** Resource Manager can import and delete project resource files;
  exports leave the project copy in place.
- **Observed:** One Resource Manager PNG import wrote the copied asset while the
  editor remained open and before Save Project. The later save changed only
  `data/System.json`'s `versionId` in existing file content; quitting without a
  prompt made no further content change.
- **Observed:** Deleting that unreferenced PNG in a separate copy also took
  effect before Save Project. The later save changed only `System.versionId` in
  file content but advanced modification and change times on
  `game.rmmzproject`, `data/System.json`, `package.json`, and `index.html`,
  including three unchanged-byte files. Clean quit produced no later write.
  Modification times therefore cannot serve as content-change or authorship
  evidence.
- **Observed, controlled-input limitation:** Deleting a nested PNG selected by
  one Show Picture command also took effect immediately, without a warning or
  refusal. The empty user-created directory remained and the serialized string
  remained dangling through Save Project and close. The editor accepted and
  indexed the controlled event on reload, but this copy's event was injected
  from a previously editor-created structure rather than independently created
  through Event Editor.
- **Observed, cause unknown:** During later setup, `package.json` acquired a new
  modification time without changing bytes in both the prior project and newly
  opened copy. Launch, automatic reopen, explicit open, copying, and host
  software were not isolated.
- **Observed:** A subsequent closed pre-open snapshot narrowed that uncertainty:
  during the launch/open workflow, same-byte `package.json` modification/change
  times and the marker's change time advanced before Save Project. Which launch,
  automatic-reopen, explicit-open, editor, or filesystem substep caused each
  update remains **unknown**.
- **Documented:** Save Project overwrites project contents, and Update Core
  Script can change the runtime script set. Exact touched files depend on the
  edit and remain broadly unknown.
- **Observed:** In a controlled exact-copy title edit, Save Project wrote all
  expected changes while MZ remained open. Quitting without a prompt produced
  no subsequent content, hash, size, or modification-time change. Earlier
  apparent close-required behavior was not reproduced and remains unexplained
  workflow history, not an editor persistence contract.
- **Observed:** In one controlled MZ 1.10.0 save, changing only System 1's Game
  Title and confirming it after restart preserved the complete path manifest
  and marker bytes. It changed only `data/System.json` (`gameTitle` and
  `versionId`), `package.json` (`window.title`), and the HTML title in
  `index.html`. After redacting those scalar changes, all three files retain
  their baseline bytes. This establishes duplicated title metadata for that
  version and action, not a general save or synchronization contract.
- **Observed:** Creating one top-level map in a controlled MZ 1.10.0 Basic copy
  adds `data/Map002.json`, appends index/ID 2 to `MapInfos.json`, and changes
  only `System.editMapId` and `System.versionId` elsewhere. The map's editor
  Name is stored in `MapInfos.json`; the new map file contains the separately
  blank Display Name and no top-level ID. Existing map-info elements and marker
  bytes remain unchanged. Deletion, reuse, children, and reordering are still
  unknown.
- **Observed:** Deleting that added trailing map removes `Map002.json`, shortens
  `MapInfos.json` from three elements to two without a trailing null, restores
  the original Basic map-info bytes, resets `System.editMapId` to 1, and changes
  `System.versionId`. Middle deletion, ID reuse, and child behavior remain
  unknown.
- **Observed:** Adding a second default-settings map sequentially creates
  `Map003.json` and index/ID/order 3 while preserving the first added map. The
  two added map files are byte-identical despite different editor Names stored
  in `MapInfos.json`; both lack a top-level embedded ID. This is a narrow
  external-identity observation, not a universal reference contract.
- **Observed:** Deleting middle ID 2 removes `Map002.json`, retains a length-4
  map-info array with `null` at index 2, preserves `Map003.json` and ID/index 3,
  and changes the surviving entry's `order` from 3 to 2. `System.editMapId`
  remains 3; only `System.versionId` changes. This demonstrates a gap and
  separates observed ordering from identity.
- **Observed:** Creating the next map in that one-hole state offers default Name
  `MAP002`, recreates `Map002.json`, fills array index/ID 2, and assigns it
  `order` 3 while leaving ID 3 at `order` 2. This establishes single-hole reuse
  and logical-order append behavior only for the observed state.
- **Observed:** Reordering the two contributor-named top-level maps swaps only
  their `order` values. Their array indices, IDs, `parentId` 0, filenames, and
  map bytes remain unchanged; `System.editMapId` tracks the moved/selected map.
  For this confirmed pair, the smaller order value appears earlier in the UI.
- **Observed:** Creating child ID/file 4 under parent ID 3 sets `parentId` 3 and
  inserts the child immediately after its parent in compact order. It changes
  the parent's `expanded` flag to true and shifts the following top-level map's
  order, while all existing map files remain byte-identical.
- **Observed:** Reparenting that child to ID 2 changes the child's `parentId`
  from 3 to 2 and reflows compact order, but preserves every map ID, filename,
  and map byte. The old parent's `expanded` stays true and the new parent's
  stays false in the serialized data, so expansion is not derived solely from
  current child presence in this observation.
- **Documented:** Deployment can target Windows, macOS, and Web. Unused-file
  filtering applies to nested files under `img`, `audio`, and `effects`, with
  plugin caveats. Image/audio encryption is deployment-only; official help says
  encrypted files cannot be used in the project folder.
- **Documented:** MV conversion can add copied content and the deprecated,
  nondefault `img/animations` directory.
- **Observed later, origin inferred:** `.DS_Store` appeared in the Basic research
  root after its fresh manifest was recorded. Its likely host-OS origin is not
  an MZ format claim, but its presence demonstrates that incidental unknown
  entries must not invalidate discovery.

## Authoring data, runtime inputs, and deployment output

| Category | Evidence-backed treatment |
| --- | --- |
| Project marker | **Documented authoring role and observed Web-deployment omission:** selected by the editor to open the project and absent from the observed MZ 1.10.0 Web output. Other deployment targets and options remain unknown. |
| `data` JSON | **Documented/inferred dual role:** edited project/database/map information is associated with runtime data globals in the official script reference. Do not classify the directory as editor-only. |
| Assets (`img`, `audio`, `effects`, fonts, movies) | **Documented dual-use direction:** stored in the project for editor selection and/or copied into deployment. Exact per-file deployment behavior can vary with options and plugins. |
| Core/plugin JavaScript | **Documented project/runtime role at a high level:** the editor manages plugins and core-script updates; plugins alter game behavior. Exact files and ownership are not established here. |
| `index.html`, `package.json`, and `css` | **Observed runtime-shaped inputs in every authoring root:** their presence cannot identify deployment output or replace the project marker. A controlled title save also shows the editor synchronizing the title into `index.html` and `package.json` alongside `data/System.json`. |
| Web deployment folder | **Documented generated output and observed for MZ 1.10.0:** the default-option Basic Web deployment retained all 1,311 non-marker files at the same relative paths, omitted `game.rmmzproject` and the empty `movies/` directory, and changed one runtime configuration field in `package.json`. Other targets, options, projects, and versions remain unknown. |
| Other deployment folders or `Game.app` | **Documented generated output:** the Deployment command exports a target-specific game folder or application. Their complete layouts and marker inclusion remain unknown. |
| Playtest/save/cache files | **Unknown:** no path, lifetime, or ownership contract was established in this investigation. Do not detect or delete them by guessed names. |
| Host/editor incidental files | **Open-world treatment:** later `.DS_Store` presence is observed but not attributed to MZ. Unknown metadata, backups, logs, or plugin files must be reported without becoming identity requirements. |

A `data` directory or runtime entry point cannot distinguish an authoring root
from a deployment. The marker distinguishes the four observed MZ 1.10.0
authoring roots from the observed default-option Web deployment, but other
targets and versions remain untested; marker presence still yields only a
candidate result.

## Assumptions discovery must not make

- **Filename and casing:** Exact lowercase `game.rmmzproject`, plural
  `Skills.json`/`Items.json`, plural `plugins.js`, and three-digit map filenames
  are generated by MZ 1.10.0. The editor also opens capital-G
  `Game.rmmzproject` in the observed macOS environment, where lowercase lookup
  resolves that entry. Do not generalize acceptance to case-sensitive
  filesystems or other versions, silently case-fold paths, or assume every
  `.json` under `data` is stock MZ data.
- **Marker version text:** MZ 1.10.0 opens and preserves a marker changed to
  `RPGMZ 1.10.1`. Do not treat marker text as trusted evidence of the editor
  version, project compatibility, or format support.
- **Identifiers:** Do not assume an array index equals an embedded ID, IDs are
  contiguous or positive, map filename digits equal a map ID, or an ID is stable
  enough to expose publicly. The controlled addition aligned filename/index/ID
  2, but the new map file itself had no top-level ID. Middle deletion produced
  a null index-2 gap while preserving ID/index 3.
- **Ordering:** Do not reorder directory entries, JSON objects, arrays, maps, or
  database records. `MapInfos.json` has an `order` property in the version 1.0.0
  reference. Middle deletion preserved ID/index 3 while compacting its `order`
  from 3 to 2, so observed `order` is not interchangeable with ID or array
  index. A direct top-level reorder swaps only the affected order values, with
  the smaller value appearing first for that pair. Hierarchical traversal,
  invalid/duplicate values, and serialization rules remain unknown.
- **Encoding:** All 252 observed `data/*.json` files parse as JSON and valid
  UTF-8, with no BOM, carriage return, or final newline. Do not normalize these
  properties or generalize them beyond the observed projects; Unicode
  normalization, numeric lexical form, and preservation requirements remain
  unknown.
- **Cross-file references:** Do not infer referential validity from names such
  as `tilesetId`, `parentId`, or other `...Id` fields. Existence, null/sentinel
  conventions, bounds, and version/plugin semantics need separate research. A
  controlled child provides one valid `parentId` linkage to an existing
  map-info ID, but orphan/cycle/depth behavior remains unknown. One Show Picture
  string survives deletion of its selected PNG without an editor warning; do
  not assume discovery can prove all asset references resolve or that the editor
  cleans dangling values.
- **Duplicated metadata:** The observed editor writes one Game Title concept to
  three files and also changes `System.versionId`. Do not assume
  `data/System.json` is the sole authority, that all three are always present or
  synchronized, or that `versionId` is a stable format/editor version.
- **Completeness:** Do not reject a marker-bearing candidate because a standard
  path is missing, and do not declare it healthy because all known names exist.
- **Execution:** Do not load JavaScript, instantiate plugins, or evaluate data
  during discovery.

## Plugin-created and unknown contents

The official `UniqueDataLoading` plugin explicitly permits optional JSON files
in `data` and mentions data added by proprietary plugins. Tilewright discovery
must therefore use an open-world inventory:

- recognize only evidenced standard names and classify everything else as
  unknown or extension data;
- preserve exact paths and spelling in any reported inventory;
- do not parse an unknown JSON file as a known database solely because it is in
  `data`;
- do not execute plugin code to discover its files;
- do not delete, move, rename, normalize, or write unknown entries;
- keep filesystem access under the caller-selected root and report, rather than
  follow, immediate marker symlinks until a separate policy is accepted; and
- let later loaders opt into specific extension knowledge without weakening the
  default preservation rule.

## Proposed fixtures and tests

The inventory implementation uses programmatically generated temporary directory
trees and adds no persistent project fixture data. Any future persistent fixture
must follow [`fixtures/README.md`](../../../fixtures/README.md).

Minimum path-level cases:

1. A filename-only synthetic candidate with an empty marker. Its provenance must
   identify it as the zero-byte form observed to be rejected by MZ 1.10.0; the
   path may be discovered, but diagnostics must not call it editor-openable.
2. No marker; wrong `.rmmvproject` extension; marker path that is a directory;
   and a symlinked marker.
3. Exact lowercase marker, differently cased marker, altered marker version
   text including the observed editor-accepted `RPGMZ 1.10.1`, and simultaneous
   variants, with deterministic diagnostics rather than platform-dependent
   selection.
4. A deployment-shaped synthetic tree containing `data`, `img`, and runtime
   names but no marker; it must not be recognized as an authoring candidate.
5. A candidate containing missing standard paths, conflicting singular/plural
   database names, additional plugin JSON, and arbitrary unknown root entries;
   identity remains candidate-level and unknowns remain untouched.
6. Scope tests proving discovery enumerates only immediate entries and
   classifies immediate marker-entry symlinks without following them. Root-path
   tests must retain the documented ambient-resolution and race limitations.

An editor-created fixture should wait for contributor authorization and a named
version experiment. Prefer committing a sanitized manifest or purpose-built
contributor data over an untouched default project, and record redistribution
rights, removed vendor content, and expected fidelity explicitly.

For future resource and deployment tests, a separately authorized synthetic
fixture may be a single 1×1 PNG with no ancillary chunks. Its provenance should
state the pixel/color type, exact creation recipe, contributor ownership,
redistribution permission, byte hash, and the one behavior under test. Do not
copy the ignored research asset into fixtures merely because it was observed.

## Required next experiment

The four-template manifest comparison, Basic Web deployment, persistent Game
Title saves, controlled map lifecycle actions, save/close probe, flat and nested
Resource Manager import/deletion experiments, one serialized Show Picture
reference, and one silent referenced-resource deletion establish a strong MZ
1.10.0 local baseline. Separate owned copies should next test deeper asset/map
hierarchy, parent deletion, and multiple-hole selection. The capital-G marker
test still needs a case-sensitive filesystem. Other deployment targets and
another named editor version at or above 1.10.0 should follow. This will resolve
or extend:

- reference cleanup and runtime behavior beyond the one observed dangling Show
  Picture string;
- case sensitivity, marker-content validation, and version dependence;
- which observed standard paths the editor actually requires;
- which files are authoring-only, runtime inputs, playtest-generated, or
  deployment output; and
- the strongest evidence-backed positive and negative detection tests.

Until those observations exist, current compatibility remains **Experimental** as stated in [`compatibility.md`](../../compatibility.md).
