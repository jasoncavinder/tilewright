# `tilewright-cli`

`tilewright-cli` is the human- and script-facing command-line adapter over the
[`tilewright`](../tilewright/README.md) library. Its installed executable name is
`tilewright`.

## Status

This crate is experimental. It provides help and version output plus read-only
`discover`, `inventory`, and `inspect-json` commands over the core library's experimental RPG Maker MZ
candidate-discovery, capability-relative project inventory, and strict lossless JSON syntax APIs. It does not
load, validate, or modify projects.

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

# Inspect a file for strict lossless JSON syntax.
cargo run -p tilewright-cli -- inspect-json path/to/file.json
cargo run -p tilewright-cli -- inspect-json path/to/file.json --format json
cargo run -p tilewright-cli -- inspect-json path/to/file.json --max-bytes 1048576
```

Candidate, negative, ambiguous, and non-regular marker findings exit with code
0 because discovery completed successfully. Operational failures exit with code
1. Argument errors use `clap`'s standard nonzero exit behavior.

JSON paths include an exact `utf8` value when one exists and a lossy `display`
value for presentation. Callers must not treat `display` as an exact encoding of
a non-UTF-8 path.

Note: While descendant symlink entries are reported without traversal, the initial
`open_ambient_dir` acquisition may resolve root or ancestor symlinks and does not
prove root identity. The `inspect-json` command explicitly opens the provided path
and makes no project-containment claim.

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
