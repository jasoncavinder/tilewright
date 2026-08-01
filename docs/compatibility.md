# Compatibility and support status

Tilewright's first target is RPG Maker MZ, but no exact editor version or project
format version is supported yet. Compatibility claims must be backed by recorded
evidence, fixtures, and tests rather than inferred from the product name.

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
| Core `tilewright` package | Scaffold | Exposes only its package version. |
| `tilewright` CLI executable | Scaffold | Prints its package version; it does not parse commands. |
| `tilewright-mcp` server | Scaffold | Not yet an MCP server; it prints its package version. |
| RPG Maker MZ project detection | Not implemented | Detection criteria and a tested version matrix remain unknown. |
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

Reading, semantic interpretation, round-trip fidelity, and safe mutation are
separate compatibility claims. Support for one does not imply the others.

## Versioning and publishing

The workspace is versioned `0.1.0`, but all packages currently set
`publish = false`. No minimum supported Rust version, platform matrix, package
publication order, or pre-1.0 stability policy has been accepted.

Until those decisions are recorded:

- do not present crate versions as a format-support matrix;
- do not publish packages or create releases as an incidental task;
- treat public APIs as intentionally small and subject to explicit review; and
- record the toolchain and platform used for evidence and verification.

Relevant unresolved decisions are maintained in
[`open-questions.md`](open-questions.md).
