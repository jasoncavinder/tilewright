# Evidence-to-support capability roadmap

This roadmap translates Tilewright's long-term
[stock authoring-data parity](decisions/0003-stock-authoring-data-parity.md)
goal into bounded library work. It describes intended sequencing and readiness
gates, not current support. The current capability matrix remains authoritative
in [`compatibility.md`](compatibility.md).

## Operating model

Tilewright does not move from one monolithic research phase into one monolithic
implementation phase. Each capability follows the same feedback loop:

```text
evidence -> bounded contract -> legal fixture -> implementation
    ^                                             |
    |                                             v
compatibility update <- differential verification and tests
```

A conflict or surprise during implementation or editor verification returns the
capability to research. A successful experiment is evidence, not support.

The unit of compatibility is a tuple such as:

```text
operation × project-data area × named MZ version
          × relevant platform/filesystem constraints × preservation guarantee
```

This prevents “supports MZ” from hiding materially different claims about
discovery, loading, interpretation, round trips, validation, or writes.

## Capability dimensions

| Dimension | Question answered | Does not imply |
| --- | --- | --- |
| Discovery | Is this explicit directory an MZ authoring-project candidate? | Completeness, compatibility, or validity |
| Inventory | Which exact project-relative entries exist, and how are they classified? | That known-looking files are parseable or required |
| Syntax loading | Can the relevant bytes and document syntax be represented? | That fields or references are semantically understood |
| Typed understanding | Which evidenced concepts and relationships can callers inspect? | That all unknown content is understood or writable |
| Validation | Which syntax, shape, reference, compatibility, or safety findings can be established? | That a finding is necessarily rejected by the editor |
| Round trip | What lexical, structural, or semantic information survives serialization? | That arbitrary edits are safe |
| In-memory mutation | Can a bounded domain operation produce a valid proposed change set? | Permission or safety to persist it |
| Persistence | Can a validated change be committed and recovered safely? | Support for unrelated mutations or versions |

Compatibility documentation must state these dimensions independently.

## Cross-cutting exit gate

Before a capability is marked **Supported**, its change must provide:

1. a specific caller use case and the smallest domain operation that satisfies
   it;
2. a version-scoped format contract backed by the research ledger;
3. legal synthetic, generated, or contributor-created fixtures with provenance;
4. positive, negative, boundary, diagnostic, and preservation tests appropriate
   to the contract;
5. structured failure behavior without recoverable-input panics;
6. explicit treatment of unknown, plugin-defined, and future-version content;
7. differential verification against the named MZ editor version when editor
   interoperability is part of the claim;
8. public documentation and examples for any public API;
9. an update to [`compatibility.md`](compatibility.md) that states the exact
   support and remaining unknowns; and
10. the repository's complete quality gate.

Experimental spikes may deliberately stop earlier, but they must not expose a
premature durable API or change the compatibility status to Supported.

## Sequenced milestones

### 1. Explicit-root candidate discovery

Implement the smallest behavior already supported by the
[project-layout research](formats/rpg-maker-mz/project-layout.md): inspect only
an explicitly supplied directory, classify immediate marker-entry symlinks
without following them, retain exact path spelling, and report a marker-bearing
directory as a candidate rather than a valid or compatible project.
Capability-based root containment is future work.

Use programmatically generated temporary directory trees for lowercase,
case-variant, missing, non-regular, symlinked, ambiguous, deployment-shaped, and
unknown-entry cases. This milestone must not parse project JSON or infer a
version from marker text.

### 2. Read-only project inventory

Add exact-path inventory and conservative known/extension/unknown
classification within the selected root. Preserve actual filenames and casing;
do not normalize, execute, delete, or reinterpret unknown entries. Files that
look like stock data remain observations until their syntax and role are
separately established.

**Current status: Experimental.** The core now inventories descendants relative
to a caller-authorized `cap_std::fs::Dir`, reports exact native `PathBuf` values
and entry kinds, and does not follow symlinks or read file contents. Exact
evidenced immediate-root names and immediate standard `data` filename families
are known; other immediate `data/*.json` paths are extension candidates, not
proven plugin data; all remaining paths are unknown. Initial capability
acquisition remains unresolved under ADR 0005.

### 3. Lossless representation decision

[ADR 0004](decisions/0004-lossless-json-representation.md) selects a CST architecture. The executed comparison evaluated:

- typed deserialization with extension storage;
- an order-preserving document model with typed views; and
- a lossless syntax or targeted-edit representation retaining original bytes.

The evaluation covered no-op byte preservation, unknown keys and nested values, ordering,
numeric lexemes, string escapes, duplicate keys, invalid UTF-8 or BOM handling,
and a controlled typed scalar edit.

The accepted decision requires untouched documents to remain byte-identical. The guarantee for a
touched document must be explicit, tested, and allowed to refuse unsafe edits.

### 4. Raw project loading and diagnostics

Load the relevant project bytes into the selected representation without
pretending every document or value is understood. Separate fatal project-scope
I/O failures from per-document syntax diagnostics so a damaged project can
produce a useful partial snapshot when doing so is safe.

The loader must retain source paths, document identity, and unknown contents.
Strict versus permissive behavior, error types, and source-location contracts
must be decided from concrete use cases rather than adapter needs.

**Current status: Experimental.** The core can load an experimental, read-only
raw project snapshot that composes authorized inventory with per-document
identity and syntax diagnostics. It enforces caller-supplied resource limits,
preserves exact accepted bytes, and returns a partial snapshot if some documents
fail to load. It does not turn path classification or syntax acceptance into an
implicit domain or compatibility claim.

### 5. Typed vertical slices

Grow typed understanding one useful end-to-end slice at a time. The initial
order should follow existing evidence and user value:

1. project and system metadata, map inventory, and map hierarchy;
2. map contents, events, pages, and conditions;
3. stock database tables and stable identifiers;
4. event command codes and parameter arrays, incrementally;
5. assets and serialized resource references; and
6. plugin registration and parameters without executing plugin code.

Each slice includes format evidence, raw-data survival, typed inspection,
contextual diagnostics, reference behavior, and its own compatibility scope.
Unknown event commands, extra object keys, and plugin-defined values must remain
representable rather than becoming unconditional parse failures.

The first implemented slice is the bounded read-only
[`MapInfos.json` map catalog](formats/rpg-maker-mz/map-catalog.md), with its
experimental projection contract accepted in
[ADR 0007](decisions/0007-experimental-map-catalog-projection.md).

**Current status: Experimental.** The core projects a structurally coherent
loaded map-info document into map IDs, decoded names, display order, and parent
relationships. It retains raw bytes and unknown fields in the snapshot, refuses
ambiguous required structure, and reports contextual relationship findings
without treating them as editor validation. A second bounded projection can
summarize one catalog-selected map's exact document path, decoded display name,
positive dimensions and tileset scalar, and opaque event-object count. It
deliberately does not interpret tile arrays or event contents, and neither
projection implies mutation or persistence behavior.

The selected-map summary contract is accepted in
[ADR 0008](decisions/0008-experimental-selected-map-summary.md) and implemented
experimentally. Neither acceptance nor implementation makes the capability
Supported.

A bounded selected-map event catalog is implemented experimentally under
accepted [ADR 0011](decisions/0011-experimental-map-event-catalog.md). It
projects map-scoped IDs, names, coordinates, and opaque page counts, while page
bodies, commands, notes, and unknown fields remain in the raw snapshot. Its
aggregate evidence covers 1,555 events across 196 MZ 1.10.0 map documents.
An independent differential audit matched all 10,323 bounded comparisons across
that corpus. Neither acceptance nor implementation makes the capability
Supported.

The bounded `System.json` orientation summary is accepted in
[ADR 0009](decisions/0009-experimental-system-summary.md) and implemented
experimentally. It projects only the seven accepted string and nonnegative
integer fields, retains raw bytes and unknown settings in the snapshot, and
does not validate map references or coordinate bounds. A controlled MZ 1.10.0
save preserved negative player-start coordinates, exposing a known gap in that
unsigned contract. Accepted
[ADR 0014](decisions/0014-signed-player-start-coordinates.md) defines the
correction. Neither acceptance nor implementation makes the capability
Supported.

A bounded tileset identity/name catalog is implemented experimentally under
accepted [ADR 0012](decisions/0012-experimental-tileset-catalog.md). It leaves
modes, image slots, tile flags, notes, and unknown fields in the raw snapshot.
Its evidence covers 24 records and 196 resolving map references across four MZ
1.10.0 projects, and an independent differential audit matched all 72 bounded
comparisons. Neither acceptance nor implementation makes the capability
Supported.

An independent differential audit matched all seven projected fields and the
bounded CLI envelope across the four-project MZ 1.10.0 corpus. This closes the
initial implementation-verification step without generalizing to later
versions, malformed inputs, editor validation, or broader system semantics.

### 6. Project-wide validation

Compose parsers and typed views into distinct validation layers:

- syntax and encoding;
- expected structural shape;
- identifiers and cross-file references;
- resource existence and path behavior;
- named-version compatibility and Tilewright support boundaries; and
- safety or project-health advice.

Diagnostics must make clear whether a condition is observed to be incompatible
with the editor, outside Tilewright's support, internally inconsistent, or
merely advisory. Exact public severity and diagnostic types remain an API
decision.

**Current status: Experimental first slice.** The player-start contract accepted
in [ADR 0010](decisions/0010-experimental-player-start-validation.md) composes
the system summary, coherent map catalog, and selected-map dimensions. It
reports only the evidenced zero triplet, a distinct zero-map/mixed-coordinate
state, a missing positive catalog record, and coordinates outside the map
rectangle.
Structural projection failures remain errors. A finding-free result is not a
general project-validity or compatibility claim, and the general severity and
diagnostic model remains open.

A separate independent differential audit matched all nine player-start output
and relationship comparisons across each of four authorized MZ 1.10.0
projects: 36 of 36 comparisons matched, all snapshots were complete, and all
observed states were finding-free. Negative categories remain synthetic
implementation coverage rather than editor-acceptance evidence.

A second bounded slice is implemented under accepted
[ADR 0013](decisions/0013-experimental-map-tileset-validation.md). It composes
the map and tileset catalogs with every selected-map summary and reports only
positive tileset references with no catalog record. Structural prerequisites
remain errors, and finding-free output is not a general validity claim. An
independent differential audit matched all 196 references and all 24 bounded
CLI-envelope comparisons in the four-project MZ 1.10.0 corpus.

### 7. In-memory domain mutation and semantic diffs

Add one bounded mutation at a time. A mutation changes an evidenced domain
concept rather than exposing arbitrary path or JSON writes. It produces a
reviewable change set, preserves unknown content according to the selected
fidelity contract, validates the proposed state, and performs no persistence by
default.

Start with operations whose editor behavior and cross-file effects have been
controlled experimentally. Each operation requires positive, refusal,
preservation, and editor-reopen tests.

### 8. Failure-safe persistence

Only after in-memory changes are reliable should the library persist them. The
eventual flow is:

1. load a scoped snapshot and record concurrency preconditions;
2. apply a bounded operation in memory;
3. produce semantic and file-level previews;
4. validate the proposed project;
5. refuse when compatibility or preservation is not established;
6. prepare temporary replacements and recovery data;
7. commit through a documented, failure-safe multi-file protocol; and
8. report exactly what changed and any warnings.

Locking, editor concurrency, backups, interruption recovery, filesystem
atomicity, and project-open behavior require research and an ADR before this
milestone can be Supported.

### 9. Adapter and version expansion

Expose stable library capabilities through scriptable CLI output and bounded
MCP operations. Adapters translate arguments, protocols, presentation, and
permissions; they do not reimplement format behavior or weaken core safety.

Repeat the evidence and regression loop for each later MZ version and relevant
platform condition. Contributor-led support for older releases follows
[ADR 0002](decisions/0002-rpg-maker-mz-version-floor.md) and must not weaken the
maintained 1.10.0+ contract.

## Test and evidence layers

The project uses complementary evidence without confusing private observations
with redistributable regression inputs:

| Layer | Purpose | Repository status |
| --- | --- | --- |
| Synthetic unit and property tests | Boundaries, malformed inputs, preservation, deterministic behavior | Committed with source and generation details |
| Minimal contributor-created project fixtures | Legal end-to-end relationships that are impractical to express as isolated files | Committed only with provenance and redistribution permission |
| Local compatibility corpus | Direct observations of legitimate user-owned editor projects and outputs | Ignored under `.local-research/`; never copied into Git |
| Differential editor experiments | Compare Tilewright behavior and mutations with a named MZ version | Derived observations recorded; proprietary inputs and outputs remain local |

Fuzzing should be added as syntax and path surfaces mature. It complements
evidence-backed semantics; it does not establish field meaning.

## Contributor and agent workflow

Keep each research or implementation task bounded to one question or capability
slice. A useful contribution states:

- the caller outcome being advanced;
- the evidence and version scope;
- what remains unknown;
- the preservation and failure contract;
- fixtures and tests added or proposed; and
- the exact compatibility status before and after the change.

Research, representation spikes, and implementation may use separate pull
requests when that makes uncertainty easier to review. Do not hide an
architectural decision in a parser dependency or generated model, and do not
expand an implementation merely because adjacent fields look familiar.

## Immediate next work

Explicit-root candidate discovery, capability-relative project inventory, an
immutable single-document lossless syntax representation, and a read-only raw
project snapshot loader are implemented experimentally. The first typed
map-catalog projection and its `maps` CLI adapter are also implemented
experimentally. A tileset identity/name projection and its `tilesets` CLI
adapter begin the database-table stage without interpreting tile behavior or
assets. The selected-map summary and its `map` CLI adapter are implemented
experimentally without interpreting tile contents. The selected-map event
catalog and its `events` CLI adapter expose only bounded event identity,
placement, and opaque page counts. The system summary and its `system` CLI
adapter are implemented experimentally without interpreting other system
settings. A first player-start validation slice and its `validate` CLI adapter
compose those projections experimentally. A separate `validate-tilesets` slice
checks map-to-tileset references. Neither establishes general project validity
or editor compatibility.
Differential verification matched all 196 catalog records and all 196
selected-map summaries in the local four-project MZ 1.10.0 evidence corpus. It
also matched all 28 field comparisons and all four output envelopes for the
system summary. Controlled MZ 1.10.0 experiments also establish same-map start
relocation, Delete-generated zero-triplet serialization, and preservation of
mixed-zero, exact-boundary, dangling-map, negative-coordinate, and upper
out-of-bounds states. The negative case returns the system and player-start
contracts to the contract stage under accepted ADR 0014. Independent
differential verification also matched the bounded tileset and event
projections and the map-to-tileset validation relationship against the
four-project corpus. A controlled tileset rename changed only the name field;
the next editor evidence steps are tileset mode and map assignment plus event
creation, movement, renaming, page lifecycle, and deletion. Tile behavior,
event page bodies, and commands remain later separate slices.
