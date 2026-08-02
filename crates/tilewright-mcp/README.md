# `tilewright-mcp`

`tilewright-mcp` is intended to be a thin Model Context Protocol adapter over
the [`tilewright`](../tilewright/README.md) library.

## Status

This crate is a scaffold, not a functioning MCP server. The executable currently
prints the package version.

## Responsibilities

The adapter may own:

- MCP tool and resource definitions;
- protocol input validation and structured result serialization;
- translation between MCP requests and library operations;
- agent-usable error reporting; and
- protocol-level capabilities, confirmation, and permission controls.

It must remain usable by different MCP clients rather than being tied to one
agent host. Project parsing, validation, mutation, data preservation, and
transaction safety belong in the core library. Arbitrary filesystem writes are
not an acceptable substitute for bounded domain tools.

See the workspace [architecture](../../docs/architecture.md),
[MCP safety requirements](../../docs/safety.md#mcp-safety-boundary), and
[compatibility status](../../docs/compatibility.md).
