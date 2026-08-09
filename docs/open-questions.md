# Open architectural and product questions

The questions below are intentionally unresolved. They are recorded so that a
prototype, dependency choice, or AI-generated type does not accidentally become
project policy.

A proposed answer should include evidence, alternatives, compatibility effects,
data-loss risks, and the smallest experiment needed to validate it. Accepted
consequential decisions should be recorded in an ADR.

## Toolchain and releases

- What minimum supported Rust version will Tilewright use?
- What supported platform matrix should supplement the current stable-Rust
  Ubuntu, macOS, and Windows CI development gate?
- When should packages become publishable, and in what order?
- What stability and semantic-versioning promises apply before and after 1.0?
- What repository, documentation, and release metadata must precede publishing?
- What response and remediation commitments should accompany future supported
  releases beyond the current private vulnerability-reporting channel?

All packages currently use `publish = false`, and no minimum Rust version is
declared.

## Format compatibility

The initial version range is resolved by
[ADR 0002](decisions/0002-rpg-maker-mz-version-floor.md): maintainer-led work
targets RPG Maker MZ 1.10.0 and newer. The long-term compatibility goal is
resolved by [ADR 0003](decisions/0003-stock-authoring-data-parity.md):
version-scoped stock authoring-data parity with open-world preservation for
unknown and plugin-defined content. The following details remain open:

- Which named versions at or above 1.10.0 need independent observations and
  regression coverage for each behavior?
- What project and runtime artifacts are authoritative for each behavior?
- How will version differences and plugin-defined fields be represented?
- Is parsing strict, permissive, or configurable?
- How much lexical formatting, key ordering, and numeric representation must a
  serializer preserve beyond semantic equivalence?
- Under what conditions may original minimal editor-created projects be
  redistributed as fixtures?
- What compatibility and trademark wording is appropriate?

## Data representation and APIs

- What production typed-view ownership and API design should sit over the CST?
  [ADR 0007](decisions/0007-experimental-map-catalog-projection.md)
  selects an owned projection for the experimental map catalog, and
  [ADR 0009](decisions/0009-experimental-system-summary.md) does the same for
  the bounded system summary. They deliberately do not settle mutable or
  production views.
- What `Send`/`Sync` and cross-thread ownership guarantees should raw documents
  and typed views provide?
- How should raw extension data relate to typed mutation APIs?
- What are the production operation-specific mutation envelopes?
- How should stale-document or stale-node detection be handled?
- What constitutes a stable project or resource identifier? ADR 0007
  introduces an experimental positive `MapId` scoped to map-catalog records,
  not a stable project-wide identity scheme. ADR 0009 deliberately retains
  system map fields as unvalidated `u32` scalars.
- How should event-command parameter arrays be typed incrementally?
- How should validation findings, severities, source locations, and related
  diagnostics be represented?
  [ADR 0010](decisions/0010-experimental-player-start-validation.md) establishes
  one severity-free contextual finding slice without settling the general
  design.
- Which error library, if any, is appropriate for the core?
- How should semantic project diffs be represented?
- Should BOM-prefixed input be rejected permanently or preserved by a separate
  byte-level envelope?
- What are the production document size, nesting, lexeme, document-count, and
  aggregate resource limits? (The experimental snapshot loader requires explicit
  caller-supplied limits.)
- What performance and dependency budgets are acceptable?

## Loading and writes

- What is the production project-loading API and what proves a path is a
  compatible project? (The experimental snapshot loader uses a
  capability-relative `Dir` and does not prove compatibility.)
- How will Tilewright acquire and verify the initial project-root capability
  without ambient root or ancestor symlink substitution or TOCTOU ambiguity?
- When may symlinks that resolve within an authorized project capability be
  followed, rejected, or reported? (The experimental snapshot loader rejects
  them.)
- What backup, locking, temporary-file, and atomic-replacement strategy is safe
  across supported platforms?
- When must a write be refused because lossless preservation is not established?
- How will dry runs, validation, and semantic diffs compose into a transaction?

## Adapters

- Which CLI framework and output-schema conventions should be used?
- Which Rust MCP SDK and protocol versions should be supported?
- Which MCP operations require confirmation, capability negotiation, or an
  explicit write mode?
- How should stable library errors map to CLI exit codes and MCP failures?

## Project identity

- Is the name “Tilewright” available for the intended legal and commercial use?

Tracking a question here does not prevent experimentation. It prevents an
experiment from being documented as a promise before the evidence and decision
exist.
