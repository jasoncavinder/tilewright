# Safety and data-preservation model

This document records project-level safety requirements. It does not claim that
planned protections are already implemented; see
[`compatibility.md`](compatibility.md) for current support.

## Primary invariant: do not silently lose unknown data

Legitimate project files may contain undocumented fields, fields introduced by
different versions, plugin-generated data, or structures Tilewright has not yet
modeled. A successful parse must not be treated as authorization to discard that
information on serialization.

Any representation and writer design must answer:

- which unknown keys, values, ordering, and encodings are preserved;
- whether the guarantee is lexical, structural, or semantic;
- how typed access and raw extension data interact;
- what happens when a typed edit conflicts with preserved raw data; and
- when Tilewright refuses a round trip because it cannot meet its guarantee.

Potential designs include extension maps, preserved raw documents, typed views
over lossless storage, or explicit strict and permissive modes. None has been
selected yet. The requirement to avoid silent loss is established; the mechanism
is an [open decision](open-questions.md#data-representation-and-apis).

## Evidence limits writes

Tilewright must not normalize, renumber, reorder, or reinterpret data merely
because a structure appears familiar. Format claims require the evidence process
described in [`formats/rpg-maker-mz/`](formats/rpg-maker-mz/README.md).

When evidence is incomplete, safe behavior may include preserving raw data,
exposing it as unknown, limiting an operation to read-only inspection, or
refusing a write. Guessing a format invariant is not a safety strategy.

## Planned write lifecycle

Persistent mutation is a later development phase. Before Tilewright presents a
write path as supported, the reusable library should provide a coherent flow:

1. scope all access to an explicitly selected project root;
2. load with known compatibility and preservation behavior;
3. apply a bounded domain mutation in memory;
4. produce a human- and machine-readable preview or semantic diff;
5. validate the proposed result;
6. persist through failure-safe temporary and atomic replacement behavior;
7. provide backup or recovery behavior appropriate to the platform; and
8. report exactly what changed and any remaining warnings.

The detailed transaction and backup design remains unresolved. No current
Tilewright binary implements project writes.

## Filesystem safety

Project operations should eventually enforce:

- an explicit project root rather than arbitrary process-wide filesystem access;
- path normalization and traversal prevention;
- clear behavior for symlinks and paths outside the root;
- bounded file selection based on domain operations;
- no partially written project after a recoverable failure; and
- deterministic diagnostics that identify affected paths without leaking
  unrelated sensitive data.

## MCP safety boundary

MCP exposes project operations to agents and therefore needs a narrower contract
than a generic file-editing tool. Preferred tools are high-level and bounded,
for example project inspection, map or event listing, validation, change
preview, and selected domain updates.

Write-capable tools should eventually include:

- explicit project-root scoping and path checks;
- dry-run or preview support;
- validation before persistence;
- clear write capability or confirmation controls;
- protection from malformed model-generated input;
- semantic results suitable for review; and
- the same unknown-field and atomic-write guarantees as direct library callers.

The MCP crate translates the protocol. It must not implement a less safe second
version of core mutations or expose arbitrary filesystem writes as a substitute
for missing domain operations.

## Fixture and intellectual-property safety

Repository safety includes respecting proprietary and third-party material. Do
not commit RPG Maker application code, bundled artwork or audio, vendor sample
games, commercial plugin data, or project files without clear redistribution
rights. Use minimal synthetic or contributor-created fixtures and document
provenance according to [`fixtures/README.md`](../fixtures/README.md).

### Local research sandbox

The ignored `.local-research/` store may contain authorized, user-owned projects
that cannot be redistributed. Agents may inspect immutable projects under
`sources/` and perform destructive or executable experiments only in uniquely
owned copies under `workspaces/`, as defined by `AGENTS.md` and the tracked local
research README.

Executable project material is untrusted. A process working directory does not
confine filesystem, network, or credential access, so execution requires a host
sandbox that enforces those boundaries or explicit user acceptance of the
unsandboxed risk for a named experiment. Repository policy is not a process
sandbox.

This experimentation is not a supported Tilewright write path and does not
weaken the repository boundary. Proprietary inputs and generated outputs remain
ignored and local. Only derived observations, non-identifying provenance, safe
procedures, and separately validated minimal synthetic fixtures may enter the
tracked repository.

## Security reporting

A private vulnerability-reporting channel has not yet been selected. It must be
established before users are encouraged to rely on write-capable or
network-facing releases. Until then, this document records design requirements
but is not a substitute for a published security policy.
