# Compatibility and support status

Tilewright's maintained RPG Maker MZ target is editor version 1.10.0 and newer,
as recorded in [ADR 0002](decisions/0002-rpg-maker-mz-version-floor.md).
Its long-term compatibility target is version-scoped stock authoring-data
parity, as recorded in
[ADR 0003](decisions/0003-stock-authoring-data-parity.md).
No project-format capability is supported yet. The version floor bounds planned
work; it does not make 1.10.0 or any newer release automatically compatible.
Compatibility claims must be backed by recorded evidence, fixtures, and tests
rather than inferred from the product name or marker contents.

## Status vocabulary

- **Scaffold:** package or interface exists but does not perform the described
  project operation.
- **Experimental:** implemented for exploration; its contract may change and it
  is not safe to rely on for valuable data.
- **Supported:** documented, tested behavior with stated input versions and
  preservation guarantees.
- **Not implemented:** planned or under consideration, but no usable behavior
  exists yet.
- **Unsupported:** intentionally rejected or outside the stated contract.
- **Unknown:** not researched or not established by sufficient evidence.

Use these terms in release notes, format documents, and user-facing output. Do
not use “supported” to mean only that one file happened to parse.

## Current matrix

| Capability | Status | Notes |
| --- | --- | --- |
| Core `tilewright` package | Experimental | Exposes its package version, candidate discovery, capability-relative read-only project inventory, immutable strict lossless JSON syntax representation, read-only raw project snapshot loader, typed map catalog projection, selected-map summary, and system summary. |
| `tilewright` CLI executable | Experimental | Provides help/version output and human- or versioned JSON-formatted access to the core library's experimental explicit-root candidate discovery, capability-relative project inventory, bounded raw snapshot loader, typed map catalog, selected-map summary, system summary, and strict lossless JSON syntax inspection. Initial `open_ambient_dir` acquisition for inventory, snapshot loading, map inspection, and system inspection may resolve root or ancestor symlinks and does not prove root identity. The `inspect-json` command explicitly opens the provided path and makes no project-containment claim. Raw snapshot output omits document contents; typed output includes only the requested projections, contextual information, and separate snapshot diagnostics. The CLI does not establish general semantic understanding, editor validation, MZ-version compatibility, round-trip support, or writes. |
| `tilewright-mcp` server | Scaffold | Not yet an MCP server; it prints its package version. |
| RPG Maker MZ explicit-root candidate recognition | Experimental | Path-level recognition is based on the documented marker role and recorded MZ 1.10.0 observations from the tested macOS environment. CI on Ubuntu, macOS, and Windows is implementation regression coverage, not editor-compatibility evidence. Newer MZ versions and unobserved editor/filesystem combinations remain unknown. Discovery does not validate project contents, infer a version, parse JSON, or guarantee compatibility. It uses `std::fs` and does not provide race-free sandbox containment or complete root symlink rejection. |
| RPG Maker MZ capability-relative project inventory | Experimental | Recursively reports deterministically ordered, exact project-relative native paths, entry kinds, and conservative known/extension-candidate/unknown pathname classifications beneath a caller-authorized `cap_std::fs::Dir`. Symlinks are reported without traversal, and file contents are not read. Known paths are limited to evidenced immediate-root names and immediate standard `data` filename families; other immediate `data/*.json` names are extension candidates, not proven plugin content. The API does not acquire or verify the initial root capability, validate entry kinds or contents, infer compatibility, provide an atomic snapshot during concurrent mutation, or return partial results after an I/O error. |
| Strict lossless JSON syntax representation | Experimental | `LosslessJsonDocument` parses one caller-supplied byte slice into a backend-hidden CST and retains exact accepted source text and bytes. It accepts valid UTF-8 strict JSON without a leading UTF-8 BOM, rejects comments and parser extensions, retains duplicate names and lexical details, and reports syntax failures through a backend-neutral diagnostic type containing a message and byte range. It performs no filesystem I/O, domain interpretation, project identification, typed viewing, mutation, or persistence. Backend ownership and threading behavior, resource limits, BOM policy, and final diagnostic types are not stable. |
| RPG Maker MZ versions before 1.10.0 | Unsupported | Outside the maintained target. Evidence-backed contributor proposals to expand the matrix are welcome. |
| Project-file parsing | Experimental | The core can load an experimental, read-only raw project snapshot that composes authorized inventory with per-document identity and syntax diagnostics. It enforces caller-supplied resource limits, preserves exact accepted bytes, and returns a partial snapshot if some documents fail to load. It does not turn path classification or syntax acceptance into an implicit domain or compatibility claim. |
| RPG Maker MZ typed map catalog | Experimental | Projects a structurally coherent loaded `data/MapInfos.json` into map IDs, decoded names, positive display order, and optional parent IDs while retaining the untouched raw document. It refuses ambiguous or malformed required structure and reports deterministic contextual parent, order, and map-document findings without claiming editor rejection. Direct evidence covers four MZ 1.10.0 projects and controlled map lifecycle experiments; an independent differential audit matched all 196 projected records and map-document identities in that corpus with no findings or snapshot diagnostics. Later versions, malformed-input editor behavior, IDs above 999, map contents, mutation, and persistence remain unknown or unimplemented, so the capability remains Experimental. |
| RPG Maker MZ selected-map summary | Experimental | Requires a coherent typed map catalog and matching catalog-scoped ID, then projects one evidenced three-digit map document into its exact path, catalog and display names, positive dimensions, positive tileset ID scalar, and count of opaque event objects. Unknown fields and exact bytes remain in the raw snapshot. The operation refuses ambiguous required structure and does not interpret tile data or event bodies, validate tileset references or editor compatibility, support IDs above 999, mutate, serialize, or persist data. Direct shape evidence covers 196 map documents from four MZ 1.10.0 projects, and an independent differential audit matched every bounded summary field across the corpus. Later versions and malformed-input editor behavior remain unknown, so the capability remains Experimental. |
| RPG Maker MZ system summary | Experimental | Projects exact `data/System.json` from an existing snapshot into its exact path, decoded game title, currency unit, and locale, plus nonnegative editor-map and player-start map/X/Y scalars. Unknown settings and exact bytes remain in the raw snapshot. The operation refuses ambiguous required structure and does not interpret other system settings, validate map references or coordinate bounds, compare titles across files, mutate, serialize, or persist data. Aggregate shape evidence covers four MZ 1.10.0 projects, and an independent differential audit matched all 28 field comparisons and all four output envelopes across that corpus. Later versions and malformed-input editor behavior remain unknown, so the capability remains Experimental. |
| Project validation | Not implemented | No validation contract exists. |
| Lossless project round trips | Not implemented | The immutable syntax representation has an exact accepted-input no-op contract, but no project round-trip, typed edit, or supported mutation exists. |
| Persistent project writes | Not implemented | No write operations or transaction safety exist. |

This table should change in the same pull request that implements and tests a
capability.

## Establishing compatibility

A supported behavior needs:

1. a specific input scope, including observed editor or format versions;
2. evidence recorded through the
   [format-research process](formats/rpg-maker-mz/README.md);
3. minimal, redistributable fixtures or generated test data;
4. positive, negative, and preservation tests appropriate to the behavior;
5. documented error and unknown-data behavior; and
6. a statement of what remains unsupported or unknown.

The lower bound is not a version-detection rule. Current evidence shows that MZ
1.10.0 accepts an altered marker string, so marker contents cannot yet prove
whether a candidate falls inside the maintained range.

Reading, semantic interpretation, round-trip fidelity, and safe mutation are
separate compatibility claims. Support for one does not imply the others.
The [capability roadmap](capability-roadmap.md) defines the more detailed
dimensions and the evidence-to-support exit gate.

## Versioning and publishing

The workspace is versioned `0.1.0`, but all packages currently set
`publish = false`. No minimum supported Rust version, platform matrix, package
publication order, or pre-1.0 stability policy has been accepted.

Until those decisions are recorded:

- do not present crate versions as a format-support matrix;
- do not publish packages or create releases as an incidental task;
- treat public APIs as intentionally small and subject to explicit review; and
- record the toolchain and platform used for evidence and verification.

Repository CI currently runs the contributor quality gate on current stable
Rust across Ubuntu, macOS, and Windows runners. This detects regressions in the
development environment; it does not establish an MSRV or a supported
operating-system matrix.

Relevant unresolved decisions are maintained in
[`open-questions.md`](open-questions.md).
