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
without treating them as editor validation. No typed map-content, mutation, or
persistence behavior is implied.

The next bounded slice is the selected-map summary accepted in
[ADR 0008](decisions/0008-experimental-selected-map-summary.md). Acceptance
defines its experimental read-only contract; it does not claim that the slice
is implemented or supported.

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
experimentally. Differential verification matched all 196 projected records
and map-document identities in the local four-project MZ 1.10.0 evidence
corpus. The next work is to implement the accepted, evidence-bounded
selected-map summary without widening the existing compatibility claim.
