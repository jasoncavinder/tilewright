# Tilewright

Open-source tooling for understanding, validating, and transforming
tile-based RPG project data.

## Workspace packages

- `tilewright`: Format-aware domain library.
- `tilewright-cli`: Human- and script-friendly command-line interface.
- `tilewright-mcp`: Thin Model Context Protocol adapter.

## Architectural rule

The `tilewright` library must not depend on MCP, AI models, OpenCode,
a graphical interface, or commercial Tilewright products.
