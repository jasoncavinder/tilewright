# RPG Maker MZ project-layout research coverage

This document answers how comprehensively Tilewright understands RPG Maker MZ
project files and folders as of 2026-08-01. It measures research coverage, not
implemented or supported behavior. Claim-level evidence remains in the
[research ledger](research-ledger.md), and the proposed discovery contract is in
[project-layout.md](project-layout.md).

The maintained target begins at MZ 1.10.0 under
[ADR 0002](../../decisions/0002-rpg-maker-mz-version-floor.md). Pre-1.10.0 gaps
are intentionally outside maintainer-led scope; gaps in later target versions
remain relevant before forward compatibility can be claimed.

## Overall assessment

The minimum read-only discovery contract is well evidenced for fresh RPG Maker
MZ 1.10.0 projects on the observed macOS environment. The wider project layout
is **partially characterized**, not comprehensive across all legitimate MZ
projects.

A closed exhaustive path list is neither established nor appropriate:

- **Documented:** Users may add browsable subfolders under asset folders and
  import their own resources.
- **Documented:** Plugins may add optional JSON data, reference additional
  resources, and change game behavior.
- **Documented:** MV conversion can add deprecated MZ-nondefault paths such as
  `img/animations` and copied plugin contents.
- **Observed:** New Game templates vary in map, database, asset, and plugin
  contents while retaining the same standard root shape.
- **Observed:** Unrelated host metadata can appear later in a legitimate root;
  `.DS_Store` appeared in the Basic research copy after the original fresh
  manifest was recorded.

Comprehensive discovery therefore means recognizing evidenced standard families
while preserving and reporting everything else as unknown. It does not mean
enumerating every path a project may legally contain.

## Coverage matrix

| Area | Coverage | Evidence-backed knowledge | Important remaining unknowns |
| --- | --- | --- | --- |
| Fresh MZ 1.10.0 root identity | **Strong for observed scope** | Official Open Project procedure plus four generated projects establish the marker role and lowercase generated spelling. | Other editor versions, case-sensitive filesystems, damaged/copied markers, and converted roots. |
| Marker contents | **Partial** | Generated projects use `RPGMZ 1.10.0`; empty content is rejected; MZ 1.10.0 accepts and preserves `RPGMZ 1.10.1`. | Complete grammar, prefix validation, whitespace/encoding tolerance, and other versions. |
| Fresh-project root directories | **Strong for observed scope** | All four templates originally shared 12 immediate entries and 30 child directories. | Other platforms, versions, editor editions, and post-creation lifecycle additions. |
| Standard database filenames and purposes | **Strong at filename/purpose level** | Official references and all templates agree on 14 non-map JSON names, including plural `Skills.json` and `Items.json`. | Requiredness, detailed schemas, edited-project variants, extension fields, and other versions. |
| Map file family | **Partial, including catalog lifecycle and aggregate map shape** | `MapInfos.json` plus three-digit `MapNNN.json` is documented and observed for 196 maps through ID 189. Controlled lifecycle actions distinguish ID, order, hierarchy, and file identity. All 196 map documents share an object shape; basic metadata kinds, positive dimensions, tileset references, tile-array length relationships, and opaque event entry counts have been audited. | Deeper hierarchy, parent deletion, multiple-hole selection, IDs above 999, malformed-input enforcement, tile-layer meaning, event semantics, mutation, and other versions. |
| Asset directory families | **Strong at high level** | Official help documents image, audio, and movie purposes; fresh projects establish their default directories. Generated `img/tilesets` also contains 31 stem-paired PNG/`.txt` pairs, whose text purpose is unknown. | Requiredness, companion semantics, filename normalization, collisions, invalid assets, and all plugin-specific resource paths. |
| Asset nesting and formats | **One nested import observed; broader behavior partial** | Official help permits browsable subfolders and documents PNG images, Ogg Vorbis audio, and WebM/MP4 movie roles. Resource Manager discovered one manually created directory beneath `img/pictures` and preserved the nested path during PNG import. Fresh trees additionally contain `.efkefc`, `.efkmodel`, WebAssembly, WOFF, and tileset `.txt` files. | Other roots, depth limits, references, extension semantics, Unicode/case behavior, invalid names, symlinks, platforms, versions, and deployment preservation. |
| Asset references | **One command-specific representation and deletion observed** | A Show Picture command stores a nested PNG as `tilewright-nested/tilewright-resource-probe`, relative to `img/pictures`, with `/` and no extension. In a controlled-input copy, Resource Manager deleted the PNG silently, retained the empty nested directory, and preserved the dangling string through save and close. | Other commands/fields, roots, formats, separators, extension rules, normalization, runtime resolution, editor-created replication of the deletion input, platforms, and versions. |
| Runtime shell and core scripts | **Observed, semantics partial** | Fresh 1.10.0 projects contain `index.html`, `package.json`, `css/game.css`, `js/main.js`, `js/plugins.js`, `js/rmmz_*.js`, and `js/libs` runtime dependencies. The editor documents a core-script update operation. | Cross-version file sets, update diffs, requiredness, ownership, and whether projects may intentionally replace these files. |
| Plugins and extension data | **Open-world by design** | Plugin JavaScript, enabled state/parameters, optional `data` JSON, and plugin resource interactions are documented. | Arbitrary plugin-created paths, schemas, side effects, external files, and compatibility. |
| Fonts and icons | **Partial** | Fresh projects contain the same two WOFF files under `fonts` and one PNG under `icon`; System 2 officially selects main, number, and fallback fonts. | Other supported font extensions, nested folders, required filenames, and deployment behavior. |
| Converted MV projects | **Documented only** | Official conversion instructions describe copied database/assets/fonts/plugins and deprecated `img/animations`. | No authorized converted project has been observed; singular filename/plugin-table conflicts remain version-limited. |
| Resource import/delete lifecycle | **Partial for flat/nested import and referenced/unreferenced deletion** | Resource Manager copied a contributor-created PNG into `img/pictures`, discovered a manually created nested directory and imported there, and deleted both an unreferenced flat copy and a referenced nested copy before Save Project. The referenced deletion was silent, retained the empty nested directory, and did not rewrite the Show Picture command. | Validation, collisions, filename normalization, other formats/roots, greater nesting, recovery, runtime missing-resource behavior, other reference types, deployment behavior, platforms, and versions. |
| Save Project and core-script update lifecycle | **Partial, with four controlled open/close boundaries** | Controlled title, flat/nested resource-import, and resource-delete workflows were observable at their relevant disk boundaries; Save Project persisted expected state while open, and no-prompt quitting made no later change. Resource saves show same-byte metadata updates to the marker, `System.json`, `package.json`, and `index.html`. A closed pre-open snapshot also isolates same-byte package/marker metadata changes during launch/open, though the exact substep remains unknown. Earlier apparent close-required behavior was not reproduced. Core-script update is documented only. | Other edit types, exact project-open cause, dirty close, project switching, atomicity, delayed/failed I/O, `versionId` generation, core-script diffs, filesystems, and other versions. |
| Playtest and save data | **Unknown at path level** | Official help documents playtesting and in-game saving but not a complete project-relative artifact inventory. | Save/config locations, temporary files, caches, logs, crash residue, and platform differences. |
| Web deployment without filtering/encryption | **Strong for one configuration** | The observed Basic deployment omitted the marker and empty `movies`, preserved every non-marker file path, and changed only one `package.json` field. | Nonempty movies, other projects, plugins, and version differences. |
| Windows/macOS deployment | **Documented only** | Official help names Windows folders and macOS folders/`Game.app`. | Complete manifests, marker inclusion, packaging, permissions, symlinks, and signing metadata. |
| Exclude-unused deployment | **Documented only** | Official help limits filtering to nested files under `img`, `audio`, and `effects`, with plugin-related caveats. | Reachability analysis, false exclusions, directory cleanup, and exact output diffs. |
| Encrypted deployment | **Documented only** | Official help treats image/audio encryption as deployment behavior and says encrypted files cannot be used in the project folder. | Output extensions, headers, filenames, key storage, per-target differences, and plugin behavior. |
| Encoding and serialization | **Narrow observation plus one save delta** | All 252 initially observed `data/*.json` files were valid UTF-8 without BOM, CR bytes, or final newlines. The title-save experiment retained those properties and was byte-identical after redacting the changed scalars. | Editor tolerance, other edit types, key/array reordering, numeric forms, Unicode normalization, and writer fidelity. |
| Unknown and host-created entries | **Behavioral policy established** | Official extensibility and later `.DS_Store` appearance demonstrate that unknown entries cannot invalidate identity. | Classification heuristics should remain advisory; ownership and safe deletion are not established. |

## Observed MZ 1.10.0 standard families

The four original New Game manifests establish these generated root entries:

```text
audio/  css/  data/  effects/  fonts/  icon/  img/  js/  movies/
game.rmmzproject  index.html  package.json
```

The Basic project also establishes these non-asset runtime families:

```text
css/game.css
js/main.js
js/plugins.js
js/rmmz_*.js
js/libs/*.{js,wasm}
```

These are **observed** 1.10.0 paths, not a required-file schema. The editor's
documented core-script update operation means runtime file sets can be
version-dependent even within an otherwise recognizable project.

## What should be learned before broader loading

Project discovery can be implemented from the current evidence without waiting
for every row above. A loader claiming broad MZ project understanding should
wait for controlled observations in this order:

1. **Resource lifecycle:** flat and one-level nested PNG imports, referenced and
   unreferenced deletion, and one serialized Show Picture reference are
   characterized; next test greater nesting, another reference-bearing command,
   or runtime handling of the known dangling string in a separately authorized
   experiment.
2. **Map lifecycle:** creation, deletion positions, single-hole reuse,
   top-level reorder, child creation, and reparenting are characterized; later
   test deeper hierarchy, parent deletion, and multiple holes.
3. **Plugin lifecycle:** add a minimal contributor-authored plugin, toggle it,
   change one parameter, and add one optional JSON document.
4. **Playtest lifecycle:** compare before/after playtest without saving, then
   perform one in-game save and locate only newly created artifacts.
5. **Deployment matrix:** repeat Web with unused-file filtering and controlled
   contributor assets, then inspect Windows and macOS default outputs; test
   encryption only with contributor-created media.
6. **Version and conversion matrix:** observe another named MZ version at or
   above 1.10.0 and one authorized user-owned MV-to-MZ conversion whose
   resulting MZ project falls inside the maintained range.
7. **Filesystem matrix:** repeat marker-casing and filename-collision tests on a
   case-sensitive filesystem.

Each experiment must change one concept, use disposable user-owned material,
avoid executing unknown plugins, and record exact versions and options. No
vendor project contents should become fixtures.

## Implementation-readiness boundary

- **Implemented experimentally:** explicit-root candidate discovery,
  capability-relative exact-path inventory, bounded raw snapshot loading, and a
  typed `MapInfos.json` catalog for map ID, name, display order, and parent
  relationships. These retain unknown entries and raw fields without implying
  broader semantics.
- **Not ready for a broad compatibility claim:** production project loading,
  typed map contents, plugin-aware interpretation, deployment recognition
  across targets, edited-project validation, or lossless writes.
- **Fundamentally open-ended:** enumerating every plugin-created, user-created,
  host-created, or future-version path. Tilewright must preserve unknown content
  rather than wait for an impossible exhaustive list.

Compatibility remains **Experimental** for discovery, inventory, raw snapshot
loading, and the typed map catalog in
[`compatibility.md`](../../compatibility.md).
