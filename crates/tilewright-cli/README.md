# `tilewright-cli`

`tilewright-cli` is the human- and script-facing command-line adapter over the
[`tilewright`](../tilewright/README.md) library. Its installed executable name is
`tilewright`.

## Status

This crate is experimental. It provides help and version output plus read-only
`discover`, `inventory`, `snapshot`, `maps`, `map`, and `inspect-json` commands
over the core library's experimental RPG Maker MZ candidate-discovery,
capability-relative project inventory, raw snapshot loader, typed map catalog,
selected-map summary, and strict lossless JSON syntax APIs. It does not provide
general project understanding, editor validation, or modification.

## Install from a checkout

Install an optimized `tilewright` executable from the repository root:

```sh
cargo install --locked --path crates/tilewright-cli
tilewright --version
```

Cargo normally places the executable in its user binary directory, which must
be on `PATH`. After pulling changes that affect the CLI, replace the installed
copy explicitly:

```sh
cargo install --locked --force --path crates/tilewright-cli
```

Remove the checkout-installed package with:

```sh
cargo uninstall tilewright-cli
```

This is a local checkout workflow; the crate remains unpublished.

## Usage

```sh
# Show available commands and the package version.
cargo run -p tilewright-cli -- --help
cargo run -p tilewright-cli -- --version

# Inspect one explicit directory for an RPG Maker MZ project marker.
cargo run -p tilewright-cli -- discover path/to/project

# Emit a versioned, machine-readable result.
cargo run -p tilewright-cli -- discover path/to/project --format json

# Inventory all entries in an RPG Maker MZ project directory.
cargo run -p tilewright-cli -- inventory path/to/project
cargo run -p tilewright-cli -- inventory path/to/project --format json

# Load a bounded, read-only raw project snapshot.
cargo run -p tilewright-cli -- snapshot path/to/project
cargo run -p tilewright-cli -- snapshot path/to/project --format json

# List the typed map catalog in editor display order.
cargo run -p tilewright-cli -- maps path/to/project
cargo run -p tilewright-cli -- maps path/to/project --format json

# Summarize one catalog-selected map.
cargo run -p tilewright-cli -- map path/to/project 1
cargo run -p tilewright-cli -- map path/to/project 1 --format json

# Inspect a file for strict lossless JSON syntax.
cargo run -p tilewright-cli -- inspect-json path/to/file.json
cargo run -p tilewright-cli -- inspect-json path/to/file.json --format json
cargo run -p tilewright-cli -- inspect-json path/to/file.json --max-bytes 1048576
```

Candidate, negative, ambiguous, and non-regular marker findings exit with code
0 because discovery completed successfully. Operational failures exit with code
1. Argument errors use `clap`'s standard nonzero exit behavior.

The `snapshot` command reports complete and partial snapshots as successful
results. Per-document syntax, entry-kind, and resource-limit problems appear as
structured diagnostics. Its default limits are 1,024 attempted documents,
16 MiB per document, and 256 MiB across all documents. These are adjustable CLI
operational safeguards for selected-document processing, not bounds on the
initial inventory traversal or RPG Maker MZ format or compatibility limits:

```sh
cargo run -p tilewright-cli -- snapshot path/to/project \
  --max-documents 256 \
  --max-bytes-per-document 8388608 \
  --max-aggregate-bytes 134217728
```

Snapshot output includes document paths and byte lengths but never document
contents. Raw syntax loading does not establish project validity, semantic
understanding, MZ-version compatibility, or round-trip and write support.

The `maps` command loads the same bounded snapshot and delegates typed
projection to the core library. It reports map IDs, decoded editor-facing names,
positive display order, optional parent IDs, and deterministic contextual
findings. Map-catalog findings describe relationships or Tilewright's evidence
boundary; they are successful results and do not claim that RPG Maker MZ rejects
the project. A missing, unavailable, or structurally ambiguous
`data/MapInfos.json` prevents projection and exits with code 1. Unrelated
snapshot diagnostics remain separate and can accompany a successful map
catalog.

Human output escapes terminal control characters in names, paths, and
diagnostics. Versioned JSON output includes projected map names because they are
part of the command's requested result, but it does not emit source documents or
unprojected fields. The snapshot resource-limit options shown above are also
available on `maps`. The command does not interpret map contents or events,
validate editor compatibility, provide stable project-wide resource identities,
or establish mutation, round-trip, and write support.

The `map` command loads the same bounded snapshot and delegates selection and
projection to the core library. It requires a coherent map catalog and matching
positive ID, then reports the catalog and display names, exact evidenced
project-relative document path, positive dimensions and tileset ID scalar, and
the count of non-null opaque event objects. A selected map that is missing,
unavailable, outside the evidenced three-digit filename family, or structurally
ambiguous exits with code 1. Unrelated snapshot diagnostics remain separate and
can accompany a successful summary.

The `map` command neither emits raw documents or unprojected fields nor
interprets tile data, event bodies, or tileset relationships. The resource-limit
options shown above are available on both `maps` and `map`. Neither command
validates editor compatibility, provides stable project-wide resource identity,
or establishes mutation, round-trip, or write support.

JSON paths include an exact `utf8` value when one exists and a lossy `display`
value for presentation. Callers must not treat `display` as an exact encoding of
a non-UTF-8 path.

Note: While descendant symlink entries are reported without traversal, the
initial `open_ambient_dir` acquisition used by `inventory`, `snapshot`, `maps`,
and `map` may resolve root or ancestor symlinks and does not prove root identity.
The `inspect-json` command explicitly opens the provided path and makes no
project-containment claim.

## Responsibilities

The CLI may own:

- command-line argument and option parsing;
- selection of human-friendly or machine-readable output;
- translation from arguments into library operations;
- terminal presentation, exit codes, and actionable error messages; and
- future inspection, validation, preview, and batch workflows.

Project discovery, format parsing, validation, serialization, and mutation must
remain in the core library. The CLI must not become a second domain
implementation.

See the workspace [architecture](../../docs/architecture.md),
[compatibility status](../../docs/compatibility.md), and
[safety model](../../docs/safety.md).
