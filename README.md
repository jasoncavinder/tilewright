# Tilewright

[![CI](https://github.com/jasoncavinder/tilewright/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/jasoncavinder/tilewright/actions/workflows/ci.yml)

Tilewright is an open-source Rust toolkit for understanding, validating, and
eventually transforming project data used by tile-based role-playing game
development tools.

The first compatibility target is RPG Maker MZ 1.10.0 and newer. Tilewright is
designed around reusable project-data concepts rather than control of that
editor or assumptions about a particular AI host. Its long-term purpose is to
let people, scripts, and tools work through typed, high-level operations instead
of editing unfamiliar JSON blindly. This version range is a development target,
not a current support claim.

> [!IMPORTANT]
> Tilewright is experimental. It can inventory and load selected project JSON
> into bounded, read-only raw snapshots, then project map IDs, names, display
> order, and parent relationships into a typed catalog. It can also summarize
> one catalog-selected map's basic metadata and opaque event count, and selected
> project-level system settings, plus list tileset IDs and editor-facing names.
> Its first contextual validator checks only the stored player start. It does
> not yet provide broader semantic understanding, project validity, or
> modification. Do not rely on it for valuable workflows.

## What Tilewright aims to provide

- Project discovery and structured inspection.
- Typed models for supported project data.
- Contextual validation and diagnostics across files.
- Loss-aware parsing that preserves data Tilewright does not understand.
- Explicit, failure-safe mutation operations with preview and validation.
- Human-friendly and machine-readable command-line output.
- Safe, bounded domain operations through Model Context Protocol (MCP).
- A reusable library for third-party and commercial applications.

These are project goals, not claims about current functionality. See the
[project vision](docs/vision.md) and [compatibility status](docs/compatibility.md)
for the distinction between planned and supported behavior.

## Design principles

1. **Preserve unknown data.** Undocumented, version-specific, and extension
   fields must not be silently discarded.
2. **Require evidence for format claims.** Findings are labeled documented,
   observed, inferred, or unknown.
3. **Keep the library durable.** Format and domain behavior belongs in the core
   crate; CLI and MCP crates are adapters.
4. **Make writes explicit and failure-safe.** Read-only inspection comes before
   mutation, and writes must eventually support validation and atomicity.
5. **Avoid premature generalization.** Learn RPG Maker MZ concretely without
   hard-coding assumptions that needlessly prevent future format support.
6. **Keep AI optional.** The core library has no dependency on models, agent
   hosts, MCP, GUI frameworks, or commercial products.

## Workspace

| Package | Role | Current state |
| --- | --- | --- |
| [`tilewright`](crates/tilewright/README.md) | Format-aware domain library and primary public API | Experimental discovery, inventory, strict lossless JSON syntax, raw snapshot loading, typed map and tileset catalogs, selected-map and system summaries, and player-start validation |
| [`tilewright-cli`](crates/tilewright-cli/README.md) | Human- and script-facing adapter; installs the `tilewright` executable | Experimental discovery, inventory, raw snapshot, typed map/tileset projections, player-start validation, and JSON inspection adapter |
| [`tilewright-mcp`](crates/tilewright-mcp/README.md) | Thin MCP adapter over the library | Scaffold |

The dependency direction is inward:

```text
tilewright-cli ──┐
                 ├──> tilewright
tilewright-mcp ──┘
```

The `tilewright` library must never depend on the adapter crates. Commercial
applications live in a separate private repository and must not be copied into
or become a dependency of this workspace.

## Documentation

- [Documentation index](docs/README.md) — where each kind of project knowledge
  belongs.
- [Vision and scope](docs/vision.md) — goals, non-goals, and success criteria.
- [Capability roadmap](docs/capability-roadmap.md) — evidence-to-support
  sequence, parity dimensions, and readiness gates.
- [Architecture](docs/architecture.md) — crate boundaries and design
  constraints.
- [Safety model](docs/safety.md) — data preservation, write safety, and MCP
  boundaries.
- [Compatibility](docs/compatibility.md) — current status and compatibility
  terminology.
- [Open questions](docs/open-questions.md) — consequential decisions that have
  deliberately not been made.
- [RPG Maker MZ research](docs/formats/rpg-maker-mz/README.md) — evidence policy
  and research workflow.
- [Contributing](CONTRIBUTING.md) — development, fixture, and verification
  expectations.
- [Security policy](.github/SECURITY.md) — private vulnerability reporting and
  supported-version expectations.
- [Code of conduct](.github/CODE_OF_CONDUCT.md) — community participation and
  enforcement expectations.

Accepted architectural decisions are recorded under
[`docs/decisions/`](docs/decisions/README.md). Tests and source code remain the
authority for behavior that is actually implemented.

## Building and trying the CLI

Tilewright uses a Cargo workspace with a virtual root:

```sh
cargo build --workspace
cargo test --workspace
cargo run -p tilewright-cli -- --help
cargo run -p tilewright-cli -- discover path/to/project
cargo run -p tilewright-cli -- discover path/to/project --format json
cargo run -p tilewright-cli -- inventory path/to/project
cargo run -p tilewright-cli -- snapshot path/to/project
cargo run -p tilewright-cli -- maps path/to/project
cargo run -p tilewright-cli -- tilesets path/to/project
cargo run -p tilewright-cli -- map path/to/project 1
cargo run -p tilewright-cli -- system path/to/project
cargo run -p tilewright-cli -- validate path/to/project
cargo run -p tilewright-cli -- validate-tilesets path/to/project
cargo run -p tilewright-cli -- inspect-json path/to/file.json
```

To install the executable from the current checkout and run it without
`cargo run`:

```sh
cargo install --locked --path crates/tilewright-cli
tilewright --version
tilewright discover path/to/project
tilewright discover path/to/project --format json
tilewright inventory path/to/project
tilewright snapshot path/to/project
tilewright maps path/to/project
tilewright tilesets path/to/project
tilewright map path/to/project 1
tilewright system path/to/project
tilewright validate path/to/project
tilewright validate-tilesets path/to/project
tilewright inspect-json path/to/file.json
```

Re-run the install command with `--force` after pulling CLI changes. See the
[`tilewright-cli` README](crates/tilewright-cli/README.md#install-from-a-checkout)
for update and uninstall details.

The CLI currently exposes experimental candidate discovery, project inventory,
bounded raw snapshot loading, typed map-catalog inspection, selected-map
summaries, tileset identity/name catalogs, selected system-setting summaries,
bounded player-start and map-to-tileset validation, and strict lossless JSON
syntax inspection. The tileset command leaves modes, images, flags, and notes
opaque. The validation commands cover only the stored player start and
map-to-tileset references; they do not establish general project validity or
editor compatibility. No command establishes MZ-version compatibility,
round-trip behavior, or write support. Contributors should use the full
verification process described in
[CONTRIBUTING.md](CONTRIBUTING.md#verification).

## License

Tilewright is licensed under the [Mozilla Public License 2.0](LICENSE). This
README summarizes project intent, not legal advice; the license text controls.
