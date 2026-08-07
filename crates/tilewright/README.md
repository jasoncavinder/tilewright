# `tilewright`

`tilewright` is the format-aware core library and the primary public API of the
Tilewright workspace.

## Status

This crate is in early development. It currently exposes its package version,
experimental candidate discovery, experimental capability-relative project
inventory, an experimental immutable strict-JSON syntax representation, and an
experimental read-only raw project snapshot loader. It does not yet understand,
validate, or write RPG Maker project-file contents.

### Example: Candidate Discovery

Candidate discovery identifies directories that appear to be RPG Maker MZ projects based on marker files. It does not validate the project contents.

Discovery results, marker observations, marker kinds, and errors are
non-exhaustive while this API is experimental. Downstream matches must retain a
fallback arm and use `..` in record patterns.

```rust
use std::path::Path;
use tilewright::rpg_maker_mz::discovery::{discover_candidate, CandidateDiscovery};

let path = Path::new("path/to/project");
match discover_candidate(path) {
    Ok(CandidateDiscovery::Candidate { marker, .. }) => {
        println!("Found candidate marker at: {}", marker.path.display());
    }
    Ok(CandidateDiscovery::NoMarker) => {
        println!("No marker found.");
    }
    Ok(other) => {
        println!("Other discovery result: {:?}", other);
    }
    Err(e) => {
        eprintln!("Discovery failed: {}", e);
    }
}
```

### Example: Capability-Relative Inventory

Inventory recursively reports exact project-relative paths, entry kinds, and
conservative pathname classifications. The caller must provide an already
authorized `cap_std::fs::Dir`; Tilewright does not acquire ambient authority or
prove the identity of the selected root.

Inventory output and errors are non-exhaustive while this API is experimental.
The operation does not read file contents or follow symbolic links.

```rust
use cap_std::fs::Dir;
use tilewright::rpg_maker_mz::inventory::{
    InventoryError, ProjectInventory, inventory_project,
};

fn inspect_authorized_project(root: &Dir) -> Result<ProjectInventory, InventoryError> {
    let inventory = inventory_project(root)?;
    for entry in &inventory.entries {
        println!("{:?} {}", entry.kind, entry.path.display());
    }
    Ok(inventory)
}
```

### Example: Strict Lossless JSON Syntax

`LosslessJsonDocument` parses caller-supplied bytes without filesystem I/O or
domain interpretation. Accepted source text and bytes remain available exactly,
and the concrete CST backend is not exposed.

```rust
use tilewright::json::LosslessJsonDocument;

let input = b"{\r\n  \"unknown\": 1e+02\r\n}\r\n";
let document = LosslessJsonDocument::parse(input)?;

assert_eq!(document.source_bytes(), input);
assert_eq!(document.to_string().as_bytes(), input);
# Ok::<(), tilewright::json::LosslessJsonError>(())
```

The current experimental contract accepts valid UTF-8 strict JSON without a
leading UTF-8 byte-order mark. It retains duplicate names and lexical details,
but provides no typed RPG Maker views or mutation API. Backend ownership,
cross-thread use, resource limits, and final diagnostic policy remain subject to
change.

### Example: Read-Only Project Snapshot

The snapshot loader composes inventory with `LosslessJsonDocument` to load
candidate project files into memory. It enforces caller-supplied resource limits
and returns a partial snapshot if some documents fail to load.

```rust
use cap_std::fs::Dir;
use std::num::NonZeroUsize;
use tilewright::rpg_maker_mz::snapshot::{
    load_snapshot, SnapshotLimits, SnapshotCompleteness, SnapshotError
};

fn load_authorized_project(root: &Dir) -> Result<(), SnapshotError> {
    let limits = SnapshotLimits {
        max_documents: NonZeroUsize::new(100).unwrap(),
        max_bytes_per_document: NonZeroUsize::new(1024 * 1024).unwrap(),
        max_aggregate_bytes: NonZeroUsize::new(10 * 1024 * 1024).unwrap(),
    };

    let snapshot = load_snapshot(root, limits)?;

    if snapshot.completeness() == SnapshotCompleteness::Partial {
        println!("Loaded partial snapshot with {} errors", snapshot.diagnostics.len());
    }

    println!("Loaded {} documents", snapshot.documents.len());
    Ok(())
}
```

## Responsibilities

As the project develops, this crate owns reusable:

- project discovery and loading;
- loss-aware parsing and serialization;
- typed and raw project-data representations;
- validation and contextual diagnostics;
- compatibility behavior;
- domain-level mutation, diff, and transaction operations; and
- error types suitable for different adapters.

It must remain independent of MCP, AI models and agent hosts, CLI presentation,
GUI frameworks, cloud services, and commercial Tilewright code. Recoverable
input and I/O errors must not become panics, and unsupported fields must not be
silently discarded.

Initial root acquisition, production project loading, typed-view ownership, and write
transaction design remain open. See the workspace
[architecture](../../docs/architecture.md),
[safety model](../../docs/safety.md), and
[open questions](../../docs/open-questions.md) before adding public APIs.
