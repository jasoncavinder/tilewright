# `tilewright-cli`

`tilewright-cli` is the human- and script-facing command-line adapter over the
[`tilewright`](../tilewright/README.md) library. Its installed executable name is
`tilewright`.

## Status

This crate is a scaffold. The executable currently prints the package version
and does not parse commands or inspect projects.

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
