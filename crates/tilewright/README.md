# `tilewright`

`tilewright` is the format-aware core library and the primary public API of the
Tilewright workspace.

## Status

This crate is in early development. It currently exposes its package version,
experimental candidate discovery, experimental capability-relative project
inventory, an experimental immutable strict-JSON syntax representation, and an
experimental read-only raw project snapshot loader. Its first typed projection
can inspect map IDs, names, display order, and parent relationships. It does not
yet provide broader understanding, general project validity, or write support. A second
experimental projection can summarize one catalog-selected map's display name,
dimensions, tileset ID scalar, and opaque event count.
An additional experimental projection reports selected `System.json` strings
and stored map-position scalars without validating their relationships.
The first experimental contextual validator composes those scalars with the map
catalog and selected-map dimensions to inspect only the player start.

### Example: Candidate Discovery

Candidate discovery identifies directories that appear to be RPG Maker MZ
projects based on marker files. It does not validate the project contents.

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
but by itself provides no typed RPG Maker view or mutation API. Backend
ownership, cross-thread use, resource limits, and final diagnostic policy remain
subject to change.

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
        println!("Loaded partial snapshot with {} errors", snapshot.diagnostics().len());
    }

    println!("Loaded {} documents", snapshot.documents().len());
    Ok(())
}
```

### Example: Typed Map Catalog

The experimental map catalog projects a structurally coherent
`data/MapInfos.json` document from an existing snapshot. Unknown fields and
exact source bytes remain untouched in the raw snapshot.

```rust
use tilewright::rpg_maker_mz::map_catalog::map_catalog;
use tilewright::rpg_maker_mz::snapshot::ProjectSnapshot;

fn print_maps(snapshot: &ProjectSnapshot) {
    match map_catalog(snapshot) {
        Ok(catalog) => {
            for map in catalog.records_in_display_order() {
                println!("{}: {}", map.id(), map.name());
            }
            for finding in catalog.findings() {
                println!("map catalog finding: {finding:?}");
            }
        }
        Err(error) => eprintln!("map catalog unavailable: {error}"),
    }
}
```

The projection refuses ambiguous or malformed required fields. Its contextual
findings identify relationships that Tilewright cannot reconcile; they do not
claim that RPG Maker MZ rejects the project. No map mutation or serialization
API exists.

### Example: Selected-Map Summary

The experimental selected-map summary reuses a catalog-scoped `MapId` and reads
only bounded fields from the evidenced matching map document. Tile data and
event bodies remain opaque and untouched.

```rust
use tilewright::rpg_maker_mz::map_catalog::MapId;
use tilewright::rpg_maker_mz::map_summary::map_summary;
use tilewright::rpg_maker_mz::snapshot::ProjectSnapshot;

fn print_map_summary(snapshot: &ProjectSnapshot, id: u32) {
    let Some(id) = MapId::new(id) else {
        eprintln!("map ID must be positive");
        return;
    };

    match map_summary(snapshot, id) {
        Ok(summary) => println!(
            "{}: {}x{}, tileset {}, {} events",
            summary.catalog_name(),
            summary.width(),
            summary.height(),
            summary.tileset_id(),
            summary.event_count()
        ),
        Err(error) => eprintln!("map summary unavailable: {error}"),
    }
}
```

The operation requires a coherent map catalog and a matching document in the
evidenced three-digit filename family. It does not validate tileset references,
interpret events or tile layers, establish editor compatibility, or expose
mutation and serialization.

### Example: System Summary

The experimental system summary reads seven bounded fields from exact
`data/System.json` in an existing snapshot. Unknown settings and exact source
bytes remain untouched.

```rust
use tilewright::rpg_maker_mz::snapshot::ProjectSnapshot;
use tilewright::rpg_maker_mz::system_summary::system_summary;

fn print_system_summary(snapshot: &ProjectSnapshot) {
    match system_summary(snapshot) {
        Ok(summary) => println!(
            "{} ({}) starts on map {} at ({}, {})",
            summary.game_title(),
            summary.locale(),
            summary.start_map_id(),
            summary.start_x(),
            summary.start_y()
        ),
        Err(error) => eprintln!("system summary unavailable: {error}"),
    }
}
```

The map ID and coordinate values are nonnegative scalars, not validated map
references. The operation does not interpret other system settings, compare
titles across files, establish editor compatibility, or expose mutation and
serialization.

### Example: Player-Start Validation

The experimental validator composes existing projections over one snapshot. It
returns contextual findings separately from structural projection errors.

```rust
use tilewright::rpg_maker_mz::player_start_validation::validate_player_start;
use tilewright::rpg_maker_mz::snapshot::ProjectSnapshot;

fn print_player_start_findings(snapshot: &ProjectSnapshot) {
    match validate_player_start(snapshot) {
        Ok(validation) => {
            for finding in validation.findings() {
                println!("player-start finding: {finding}");
            }
        }
        Err(error) => eprintln!("player start could not be validated: {error}"),
    }
}
```

The operation recognizes the observed exact zero triplet, reports a missing
positive catalog record, and checks coordinates against selected-map
dimensions. A finding-free report is not general project validity or editor
compatibility. The operation does not inspect passability, events, vehicles, or
write behavior.

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

Initial root acquisition, production project loading, production typed-view
ownership, and write transaction design remain open. See the workspace
[architecture](../../docs/architecture.md),
[safety model](../../docs/safety.md), and
[open questions](../../docs/open-questions.md) before adding public APIs.
