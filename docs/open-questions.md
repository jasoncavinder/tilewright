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
  Ubuntu CI development gate?
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
targets RPG Maker MZ 1.10.0 and newer. The following details remain open:

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

- What lossless representation will preserve unknown JSON fields?
- Should the library expose raw documents, typed views, or both?
- How should raw extension data relate to typed mutation APIs?
- What constitutes a stable project or resource identifier?
- How should event-command parameter arrays be typed incrementally?
- How should validation findings, severities, source locations, and related
  diagnostics be represented?
- Which error library, if any, is appropriate for the core?
- How should semantic project diffs be represented?

## Loading and writes

- What is the project-loading API and what proves a path is a compatible
  project?
- How are project roots canonicalized and filesystem access scoped?
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
