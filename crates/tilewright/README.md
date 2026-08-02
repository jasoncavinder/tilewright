# `tilewright`

`tilewright` is the format-aware core library and the primary public API of the
Tilewright workspace.

## Status

This crate is a scaffold. It currently exposes only the package version and does
not load, parse, validate, or write project data.

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

The raw/typed representation, loading API, and write transaction design remain
open. See the workspace [architecture](../../docs/architecture.md),
[safety model](../../docs/safety.md), and
[open questions](../../docs/open-questions.md) before adding public APIs.
