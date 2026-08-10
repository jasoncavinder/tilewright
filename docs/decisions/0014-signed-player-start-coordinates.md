# ADR 0014: Signed Player-Start Coordinates

- **Status:** Accepted
- **Date:** 2026-08-09
- **Supersedes in part:**
  [ADR 0009](0009-experimental-system-summary.md) and
  [ADR 0010](0010-experimental-player-start-validation.md)

## Context

ADR 0009 selected `u32` for all four projected map-position scalars because the
initial four-project corpus contained only nonnegative values. The resulting
system summary therefore rejects a negative `startX` or `startY` before the
player-start validator can report its relationship to the selected map.

A controlled RPG Maker MZ 1.10.0 tolerance matrix has now contradicted that
assumption. In an owned disposable project, the editor saved prepared
`startX = -1` and `startY = -1` values without normalizing them. The same matrix
also preserved exact upper in-bounds coordinates, upper out-of-bounds
coordinates, a dangling positive start-map ID, and a zero map ID with nonzero
coordinates. These observations establish persistence tolerance in that
version, not editor validity or runtime success.

The stored coordinate domain must therefore be representable independently
from coordinate validation. Map IDs remain nonnegative in all available
evidence and retain their existing zero and positive-reference behavior.

## Decision

The experimental system-summary and player-start contracts will change as
follows:

1. `editMapId` and `startMapId` remain stored `u32` scalars. Their zero,
   dangling, and catalog-reference semantics remain separate validation
   questions.
2. `startX` and `startY` become stored `i64` scalars in `SystemSummary`,
   `PlayerStartValidation`, and their associated findings. The `i64` boundary
   is an explicit Tilewright representation limit, not a claimed MZ limit.
3. Coordinate projection accepts strict base-ten JSON integer lexemes in the
   inclusive `i64` range. It continues to refuse fractional, exponent-form,
   or out-of-range coordinates through the existing typed unsupported-integer
   error. Map ID projection retains its unsigned-decimal `u32` policy.
4. The exact `startMapId = 0`, `startX = 0`, `startY = 0` triplet continues to
   produce `MissingPlayerStart`.
5. A zero map ID with either nonzero signed coordinate continues to produce
   `ZeroMapIdWithCoordinates`, but documentation and human output describe it
   as an editor-preserved ambiguous state rather than an unobserved state.
6. For a positive catalog-selected map, a negative coordinate or a coordinate
   greater than or equal to the corresponding positive dimension produces the
   existing `OutOfBounds` finding. This remains a contextual geometric finding,
   not an editor-rejection claim.
7. The core operations remain pure and read-only. Exact source bytes, unknown
   properties, and every unprojected setting remain untouched in the snapshot.
8. Because successful `system --format json` and `validate --format json`
   reports can newly contain negative coordinate values, the CLI output schema
   version advances from 1 to 2. The other command shapes remain unchanged,
   but the current executable-wide schema number advances consistently.

## Rationale

Signed storage reflects observed project data without conflating tolerance with
validity. Keeping map IDs unsigned avoids expanding an unrelated domain. An
`i64` bound is wide, directly serializable by the Rust and JSON adapters, and
keeps recoverable range failure explicit without exposing the provisional CST.

Reusing `OutOfBounds` preserves the caller's real question: whether the stored
coordinate lies in the selected map's zero-origin rectangle. A separate
negative-coordinate category would encode a representation detail without
changing the relationship or available remediation.

Advancing the JSON schema version makes the successful value-domain change
visible to script consumers. Tilewright is pre-1.0 and these APIs are
Experimental, but versioned output should still avoid silent contract changes.

## Consequences

- Tilewright can summarize and validate the newly observed negative-coordinate
  state instead of failing structural projection.
- Public Rust coordinate accessors and finding fields change from `u32` to
  `i64`; downstream experimental callers must update explicit types.
- Existing nonnegative reports retain the same values and finding categories.
- CLI consumers must accept schema version 2 before processing new reports.
- Values outside `i64`, exponent notation, and fractional forms remain refused
  without implying that MZ rejects them.
- General project validity, runtime behavior, mutation, and persistence remain
  outside the capability.

## Alternatives Considered

- **Keep `u32` and classify negative values as malformed:** Rejected because it
  contradicts a directly observed MZ 1.10.0 saved state and prevents contextual
  validation from running.
- **Use `i32`:** Rejected because no editor evidence establishes a 32-bit
  coordinate limit; the narrower bound would add an unsupported assumption.
- **Expose the raw numeric lexeme:** Rejected because callers need a small
  domain value and the raw document already preserves exact syntax.
- **Add a distinct negative-coordinate finding:** Rejected because negative and
  upper-bound values violate the same evidenced rectangular relationship.
- **Keep CLI schema version 1:** Rejected because successful coordinate fields
  change from a nonnegative to a signed value domain.
- **Interpret editor preservation as validity:** Rejected because Save tolerance
  does not establish runtime behavior, passability, or editor intent.

## Validation

Implementation requires generated synthetic tests for `-1`, `i64::MIN`,
`i64::MAX`, both overflow directions, nonnegative regression cases, zero-map
mixed coordinates, negative and upper out-of-bounds findings, raw-byte
preservation, schema-version-2 human/JSON adapters, and typed structural errors.
The retained local controlled cases must be checked differentially without
copying project data into the repository. The complete Rust quality gate must
pass before merge.
