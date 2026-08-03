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
| Core `tilewright` package | Experimental | Exposes its package version and an experimental candidate discovery API. |
| `tilewright` CLI executable | Scaffold | Prints its package version; it does not parse commands. |
| `tilewright-mcp` server | Scaffold | Not yet an MCP server; it prints its package version. |
| RPG Maker MZ explicit-root candidate recognition | Experimental | Path-level recognition is based on the documented marker role and recorded MZ 1.10.0 observations from the tested macOS environment. CI on Ubuntu, macOS, and Windows is implementation regression coverage, not editor-compatibility evidence. Newer MZ versions and unobserved editor/filesystem combinations remain unknown. Discovery does not validate project contents, infer a version, parse JSON, or guarantee compatibility. It uses `std::fs` and does not provide race-free sandbox containment or complete root symlink rejection. |
| RPG Maker MZ versions before 1.10.0 | Unsupported | Outside the maintained target. Evidence-backed contributor proposals to expand the matrix are welcome. |
| Project-file parsing | Not implemented | No format models or parsers exist. |
| Project validation | Not implemented | No validation contract exists. |
| Lossless round trips | Not implemented | Representation and fidelity requirements remain open. |
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
