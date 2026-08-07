# ADR 0006: Snapshot Loading Contract

- **Status:** Proposed
- **Date:** 2026-08-06

## Context

Tilewright needs to load RPG Maker MZ project data into memory for inspection and eventual mutation. The project data consists of multiple JSON files (e.g., `package.json`, `data/*.json`). The loading process must be safe, resource-bounded, and resilient to partial failures, while preserving the exact bytes of accepted documents according to [ADR 0004](0004-lossless-json-representation.md).

We need to define the contract for an experimental, read-only raw project snapshot loader. This loader will compose the existing capability-relative project inventory with `LosslessJsonDocument`.

## Decision

We will implement an experimental `ProjectSnapshot` loader with the following contract:

1. **Document Selection:**
   - Consider exact project-relative `package.json`.
   - Consider immediate `data/*.json` entries represented by the inventory's known standard-data, map-data, or `DataJson` extension-candidate classifications.
   - Do not parse `js/plugins.js` as JSON.
   - Do not parse arbitrary nested unknown JSON paths or binary/non-JSON files.
   - Select candidates from inventory classifications and exact evidenced paths; do not create a competing path-classification implementation.
   - A candidate path whose entry kind is symlink, directory, or other non-regular entry must not be opened. Return a per-document diagnostic and make the snapshot partial.
   - Non-candidate files remain represented only by the retained inventory.

2. **Filesystem Safety:**
   - Open document files relative to the supplied `cap_std::fs::Dir`.
   - Disable final-component symlink following using `cap-fs-ext` facilities (`OpenOptionsFollowExt::follow(FollowSymlinks::No)`).
   - Treat concurrent mutation as a documented non-atomic-snapshot limitation.
   - Use bounded reads so a growing file cannot cause unbounded allocation.

3. **Resource Limits:**
   - Add an explicit caller-supplied limits type covering: max document count, max bytes per document, and max aggregate bytes.
   - Require valid nonzero limits through construction or nonzero field types.
   - Specify deterministic limit behavior in inventory path order.
   - Every eligible document skipped or refused because of a limit must have a path-specific diagnostic; the result must be Partial, never Complete.
   - Avoid overflow when reading limit + 1 bytes.

4. **Result and Error Semantics:**
   - Return a fatal error only when no coherent snapshot can be established (e.g., inventory failure).
   - Collect individual open/read, unsupported-entry-kind, resource-limit, and `LosslessJsonDocument` parse failures in the successful snapshot.
   - Wrap or retain `LosslessJsonError` rather than duplicating its taxonomy.
   - Each document is either fully parsed and present or absent with a diagnostic.
   - Retain the `ProjectInventory` in the snapshot.
   - Store documents deterministically behind accessors over an ordered representation (`BTreeMap<PathBuf, LosslessJsonDocument>`).
   - Provide a single unambiguous completeness representation derived from diagnostics.
   - Keep public types documented and non-exhaustive.
   - Do not introduce a supposedly stable project/document ID abstraction yet; project-relative `PathBuf` is sufficient.

## Rationale

- **Partial Snapshots:** A damaged project (e.g., one malformed JSON file) should still yield a useful partial snapshot for the remaining valid files. Fatal errors are reserved for systemic failures like inability to read the directory.
- **Resource Limits:** Bounded reads and explicit limits prevent resource exhaustion attacks or accidental OOMs from malformed or excessively large files.
- **Symlink Policy:** Refusing to open symlinks prevents unexpected behavior and potential escapes, aligning with the conservative approach to filesystem safety.
- **Deterministic Order:** Processing in inventory path order ensures reproducible behavior, especially when resource limits are hit.

## Consequences

- The core library now provides a bounded, read-only project snapshot that can be used for inspection and validation.
- Callers must explicitly provide resource limits, forcing them to consider their environment's constraints.
- The snapshot is not atomic; concurrent modifications may result in an inconsistent snapshot.
- The snapshot does not provide typed views or semantic understanding of the loaded documents.
- The snapshot does not provide a stable project or document identifier beyond `PathBuf`.
- The snapshot does not support writing or mutating the loaded documents.

## Alternatives Considered

- **Fatal on any parse error:** Rejected because it prevents inspecting or recovering from partially damaged projects.
- **Implicit default limits:** Rejected because limits should be explicit and caller-defined based on their environment and use case.
- **Following symlinks:** Rejected due to security and complexity concerns; symlinks in project data are not standard and pose risks.

## Deferred Questions

- How to handle concurrent modifications during the snapshot process (currently documented as a limitation).
- The exact design of typed views over the loaded documents.
- Stable project and document identifiers beyond `PathBuf`.
- Write and mutation contracts.
