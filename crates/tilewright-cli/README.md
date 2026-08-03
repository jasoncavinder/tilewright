# `tilewright-cli`

`tilewright-cli` is the human- and script-facing command-line adapter over the
[`tilewright`](../tilewright/README.md) library. Its installed executable name is
`tilewright`.

## Status

This crate is experimental. It provides help and version output plus a
read-only `discover` command over the core library's experimental RPG Maker MZ
candidate-discovery API. It does not inventory, load, validate, or modify
projects.

## Usage

```sh
# Show available commands and the package version.
cargo run -p tilewright-cli -- --help
cargo run -p tilewright-cli -- --version

# Inspect one explicit directory for an RPG Maker MZ project marker.
cargo run -p tilewright-cli -- discover path/to/project

# Emit a versioned, machine-readable result.
cargo run -p tilewright-cli -- discover path/to/project --format json
```

Candidate, negative, ambiguous, and non-regular marker findings exit with code
0 because discovery completed successfully. Operational failures exit with code
1. Argument errors use `clap`'s standard nonzero exit behavior.

JSON paths include an exact `utf8` value when one exists and a lossy `display`
value for presentation. Callers must not treat `display` as an exact encoding of
a non-UTF-8 path.

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
