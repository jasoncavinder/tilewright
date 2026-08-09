# ADR 0010: Experimental Player-Start Validation

- **Status:** Proposed
- **Date:** 2026-08-09

## Context

Tilewright can load a bounded snapshot and project selected system and map
fields, but callers must still combine those projections to determine whether
the stored player start names a catalog map and falls within its dimensions.
The first validation slice should prove the composition pattern without
claiming general project validity or fixing the final diagnostic framework.

The [player-start research](../formats/rpg-maker-mz/player-start-validation.md)
records official documentation, a controlled relocation, and direct MZ 1.10.0
recognition and preservation of the exact zero triplet as `None`. The editor's
Delete-generated serialization, mixed zero/nonzero states, malformed-reference
acceptance, and general severity policy remain unresolved.

## Decision

For the experimental player-start validation slice:

1. The core library will expose a pure, read-only operation over an existing
   `ProjectSnapshot`. Adapters may render its result but will not reimplement
   system, catalog, selected-map, or coordinate behavior.
2. The operation will return an owned report containing the stored start map,
   X, and Y scalars plus deterministic contextual findings.
3. The exact `0, 0, 0` triplet will produce a missing-player-start finding. Its
   message may include the official documentation's cannot-start consequence.
4. A zero map ID with either nonzero coordinate will produce a distinct
   unevidenced-state finding. Tilewright will not classify that state as set or
   unset until the incomplete editor experiment is resolved.
5. A positive map ID absent from a coherent map catalog will produce a missing
   map-record finding. A present map will be summarized and checked using the
   zero-origin rectangular relation `x < width` and `y < height`.
6. System, catalog, and selected-map structural failures will remain typed
   operation errors. They will not be flattened into contextual findings.
7. A finding-free report means only that this bounded operation found no
   player-start issue. It will not imply project validity, editor acceptance,
   named-version compatibility, passability, or runtime success.
8. Findings will not introduce severities, source spans, repair suggestions, or
   a stable general diagnostic hierarchy. Those remain broader API questions.
9. The initial `tilewright validate <PATH>` adapter will report this one scope
   in human or versioned JSON output. Completed validation, including reports
   with findings, exits successfully; acquisition, loading, and structural
   projection failures exit with code 1.
10. No filesystem writes, mutation, serialization, persistence, vehicle-start
    interpretation, passability checks, or event analysis are introduced.

## Rationale

This slice crosses three already bounded projections and produces immediate
user value through the CLI. Keeping structural errors separate tells callers
whether validation actually ran. Treating findings as successful report data
matches the existing map-catalog adapter and avoids prematurely equating every
internal inconsistency with editor rejection.

The exact zero-triplet rule preserves the strongest observation available. A
separate mixed-zero finding makes uncertainty visible instead of generalizing
from an incomplete experiment.

## Consequences

- Callers can inspect the stored player start without manually joining three
  JSON documents.
- The first project-wide contextual validation pattern remains pure,
  deterministic, capability-relative through its input snapshot, and
  backend-neutral.
- Raw documents, unknown fields, and source bytes remain untouched.
- CLI scripts can distinguish completed findings from operational failures.
- General diagnostic types, severity, repair policy, compatibility validation,
  and write-time validation remain open.

## Alternatives Considered

- **Return every condition as an error:** Rejected because cross-file findings
  are completed inspection results, unlike unavailable structural inputs.
- **Treat any zero map ID as unset:** Rejected because only the exact zero
  triplet has been observed in MZ 1.10.0.
- **Claim finding-free means valid:** Rejected because the operation covers one
  relationship and no complete project-validity contract exists.
- **Add a general severity hierarchy now:** Deferred until more independent
  validation slices establish shared requirements.
- **Place the join in the CLI:** Rejected because reusable format behavior
  belongs in the core library.
- **Validate passability and event occupancy:** Deferred because those concepts
  require separate evidence and typed map understanding.

## Validation

Implementation includes generated synthetic tests for the exact zero triplet,
mixed zero/nonzero scalars, missing positive map records, coordinate boundaries,
structural errors, raw-byte preservation, deterministic output, stream
separation, resource-limit forwarding, and human and schema-version-1 JSON
adapters. The complete locked Rust quality gate must pass before merge.
