# ADR 0009: Experimental System Summary

- **Status:** Proposed
- **Date:** 2026-08-07

## Context

Tilewright can inventory, load, and inspect map-oriented project data, but a
caller still cannot obtain basic project-level orientation such as the game
title or player starting-position scalars without reading raw `System.json`.
The next typed slice should cover a small, officially documented subset without
turning the complete system database into a premature public model.

The [system-summary research](../formats/rpg-maker-mz/system-summary.md)
supports a bounded MZ 1.10.0 projection for the game title, currency unit,
locale, editor-map scalar, and player-start map/X/Y scalars. The representation
of zero and unset map references, cross-file validation, `versionId`, party
members, and the remaining system settings are not established sufficiently for
typed interpretation.

## Decision

For the experimental system-summary slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot`. CLI and future MCP adapters may render the result but will
   not reimplement `System.json` interpretation.
2. The operation will inspect only exact `data/System.json` and return an owned
   `SystemSummary` containing that exact project-relative path, decoded
   `gameTitle`, `currencyUnit`, and `locale` strings, plus `editMapId`,
   `startMapId`, `startX`, and `startY` as `u32` scalars.
3. String values will be decoded but not normalized or restricted to a grammar.
   Empty strings remain representable because editor validation and allowed
   contents are not established for every selected field.
4. Numeric values will use a bounded unsigned-decimal policy and accept zero.
   The map ID scalars will not become catalog-scoped `MapId` values until the
   editor's zero and unset-state representation is observed directly.
5. The operation will return a typed error rather than a partial summary when
   the document is absent, unavailable through snapshot diagnostics, has a
   non-object root, or has a missing, duplicate, wrong-kind, undecodable,
   negative, fractional, or out-of-`u32` required value.
6. The operation will not require `MapInfos.json`, load map documents, validate
   map references or coordinate bounds, or compare `gameTitle` with
   `package.json` and `index.html`. Those relationships belong to later
   contextual validation.
7. `partyMembers`, `versionId`, vehicles, audio, terms, type-name arrays,
   options, advanced settings, and every other system property remain opaque in
   the untouched raw snapshot.
8. No validation severity, stable project identity, mutation, serialization,
   persistence, or editor-compatibility claim is introduced by this decision.

## Rationale

These seven fields answer a concrete orientation question and are independently
documented by official sources. An owned projection follows the backend-neutral
pattern of the existing experimental typed slices while leaving the provisional
CST private and retaining unknown source data.

Keeping the map fields as nonnegative scalars avoids inventing semantics for an
editor-permitted missing player start. Separating projection from cross-file
validation also lets callers inspect a structurally coherent system document
even when a project is incomplete or internally inconsistent.

## Consequences

- Callers can inspect basic system metadata and player-start scalars without
  parsing JSON.
- Unknown properties and exact source bytes remain retained by
  `ProjectSnapshot`.
- A malformed required field prevents the summary but does not make the raw
  snapshot unusable.
- Map-reference coherence, coordinate bounds, and title consistency remain
  separate validation questions.
- The experimental API may later replace raw map scalars with a richer type
  after zero, unset, and dangling-reference behavior is evidenced.

## Alternatives Considered

- **Model all 58 observed top-level properties:** Rejected because most system
  settings have not completed their own evidence, identifier, and API work.
- **Return raw JSON or CST nodes:** Rejected because it leaks the provisional
  representation and makes adapters implement domain behavior.
- **Reuse positive `MapId` immediately:** Rejected for now because the editor
  permits deleting the player start and the resulting serialized state has not
  been observed.
- **Require coherent map-catalog references:** Deferred to project-wide
  validation; it is not required to report the stored system scalars.
- **Expose `partyMembers`:** Deferred until actor identifiers and database
  reference behavior have an evidence-bounded typed slice.
- **Expose `versionId`:** Rejected because controlled saves show that it changes
  across unrelated actions while its generation and stable meaning are unknown.

## Validation

Before implementation is ready for review, it must include generated synthetic
tests for successful projection, empty and escaped strings, zero and `u32`
numeric boundaries, every structural refusal, document availability, raw-byte
preservation, ignored unknown properties, and absence of mutation. Public
Rustdoc and the compatibility matrix must state the exact experimental
non-claims. Each adapter must add equivalent tests and non-claims when it
exposes the operation.
