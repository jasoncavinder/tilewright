# ADR 0013: Experimental Map-Tileset Reference Validation

- **Status:** Proposed
- **Date:** 2026-08-09

## Context

Tilewright can now project cataloged maps, selected map summaries, and tileset
identity independently. Four-project evidence found that all 196 audited maps'
positive `tilesetId` values resolve to ID/index-consistent tileset records, but
the library does not yet expose that relationship as a reusable operation.

The smallest useful next validation slice is to identify unresolved
map-to-tileset references without interpreting tileset modes, images, flags,
tile behavior, or editor rejection.

## Decision

For the experimental map-to-tileset validation slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot`. Adapters will not reimplement reference validation.
2. The operation will require coherent map and tileset catalogs, then summarize
   every cataloged map in ascending map-ID order.
3. A map whose positive `tilesetId` has no matching `TilesetRecord` will produce
   a deterministic `MissingTileset` finding containing typed map and tileset
   IDs.
4. Missing or ambiguous prerequisite catalogs and structurally unavailable map
   summaries will be typed errors. They will not be converted into findings.
5. Findings are successful results. A finding-free result means only that this
   bounded relationship check found no unresolved reference.
6. The operation will preserve the raw snapshot and will not perform
   filesystem I/O, inspect tileset assets or behavior, assign severity, mutate,
   serialize, persist, execute a project, or claim editor acceptance.

## Rationale

Composition keeps each projection independently useful and prevents the
tileset catalog from acquiring map-specific behavior. Reusing the selected-map
summary also avoids a second parser for the same map structure. Consequently,
malformed required summary fields unrelated to `tilesetId` can prevent this
initial validator; that is preferable to two typed interpretations drifting.

Separating structural errors from contextual findings lets callers distinguish
“the reference is absent from a coherent catalog” from “Tilewright could not
establish the prerequisite structure.”

## Consequences

- Callers can check all cataloged map-to-tileset references in one operation.
- Results are deterministic and preserve typed resource domains.
- One malformed selected-map summary prevents the project-wide report.
- Tileset assets, modes, image slots, flags, and tile meaning remain opaque.
- Evidence covers MZ 1.10.0 only, so the capability remains Experimental even
  if implemented.

## Alternatives Considered

- **Validate references while building either catalog:** Rejected because it
  couples otherwise independent projections.
- **Parse only `tilesetId` again:** Rejected because a second map parser could
  drift from the accepted selected-map projection.
- **Return partial findings after structural errors:** Deferred until a caller
  need and deterministic partial-result contract justify the added complexity.
- **Treat a missing reference as an error:** Rejected because the structural
  documents can be coherent even when their relationship is not.
- **Call a finding-free result a valid project:** Rejected because this check
  covers only one bounded cross-file relationship.

## Validation

Implementation must include generated synthetic tests for finding-free and
missing-reference cases, deterministic ordering, each structural error layer,
raw-byte preservation, and absence of mutation. The CLI must include versioned
JSON, human output, resource-limit, terminal-control, and stream-separation
tests. Compatibility documentation must state the exact scope and non-claims.
