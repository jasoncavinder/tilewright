# `tilewright`

`tilewright` is the format-aware core library and the primary public API of the
Tilewright workspace.

## Status

This crate is in early development. It currently exposes its package version,
experimental candidate discovery, and experimental capability-relative project
inventory, plus an experimental immutable strict-JSON syntax representation. It
does not yet load, understand, validate, or write RPG Maker project-file
contents.

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

Initial root acquisition, project loading, typed-view ownership, and write
transaction design remain open. See the workspace
[architecture](../../docs/architecture.md),
[safety model](../../docs/safety.md), and
[open questions](../../docs/open-questions.md) before adding public APIs.
