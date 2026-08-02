# ADR 0002: RPG Maker MZ version floor

- **Status:** Accepted
- **Date:** 2026-08-01

## Context

Tilewright needs a bounded compatibility target while its format knowledge and
library are still being established. Investigating every historical RPG Maker
MZ release would multiply the research, fixture, implementation, and regression
matrix before the project has implemented its first read-only capability.

The maintainer's available editor and direct project evidence establish RPG
Maker MZ 1.10.0 as the current baseline. The project marker is not sufficient
to enforce a version floor: MZ 1.10.0 was observed opening and preserving a
marker changed from `RPGMZ 1.10.0` to `RPGMZ 1.10.1`.

## Decision

Tilewright's maintained RPG Maker MZ target is editor version **1.10.0 and
newer**.

- Maintainer-led research, implementation, fixtures, and compatibility testing
  will focus on named MZ versions at or above 1.10.0.
- Versions older than 1.10.0 are outside the maintained compatibility target.
- A newer version is not automatically supported merely because it is above the
  floor. Each supported behavior still needs a stated version scope, evidence,
  fixtures or generated test data, and tests.
- Candidate discovery must not claim that marker contents prove an editor
  version. Until version identification is evidenced, discovery may recognize a
  candidate without establishing that it falls inside the maintained range.
- Contributions adding evidence and support for older versions are welcome, but
  they must preserve the existing 1.10.0+ contract and include the research,
  fixtures, tests, and documentation needed to define their own scope. Accepting
  such a contribution changes the maintained compatibility matrix and requires
  explicit review.

## Rationale

A concrete lower bound keeps the initial workload proportional to the project's
maturity and aligns implementation with the direct evidence already available.
Keeping newer releases inside the intended range lets the project investigate
forward as releases appear, while the evidence requirement prevents the range
from becoming an unsupported blanket compatibility claim.

## Consequences

- Compatibility documentation distinguishes the maintained target range from
  behavior actually implemented and tested.
- Research gaps for pre-1.10.0 releases are out of scope rather than blockers for
  initial discovery and loading work.
- Unknown behavior in newer target releases remains visible and may require
  version-specific diagnostics or refusal.
- Project/version detection cannot rely on the observed marker string alone.
- Older-version contributions may expand the matrix later without forcing the
  initial maintainers to research those releases now.

This ADR does not make any RPG Maker MZ capability supported. Current capability
status remains governed by [`compatibility.md`](../compatibility.md).
