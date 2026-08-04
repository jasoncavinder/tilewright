# Workspace architecture

## Central idea

Tilewright's reusable Rust library is the project's brain. CLI, MCP, graphical
interfaces, AI integrations, and commercial applications are replaceable
interfaces around it.

This is an accepted boundary, recorded in
[ADR 0001](decisions/0001-project-boundaries.md). The library's stock
authoring-data parity target is defined by
[ADR 0003](decisions/0003-stock-authoring-data-parity.md), and its staged
development gates are in the [capability roadmap](capability-roadmap.md). The
details of project loading, errors, and writes remain
open until research and experiments support them.

## Workspace shape

The virtual Cargo workspace contains three independently versioned durable
product packages, plus internal tools:

```text
crates/tilewright/       durable, format-aware library
crates/tilewright-cli/   command-line adapter
crates/tilewright-mcp/   Model Context Protocol adapter
tools/jsonc-parser-study/ internal research and verification tool (non-publishable)
```

Allowed dependency direction:

```text
external application
        │
        ├──────────────┐
        v              v
tilewright-cli   tilewright-mcp
        │              │
        └──────┬───────┘
               v
          tilewright
```

`tilewright` must not depend on either adapter. All packages currently remain
unpublished as a deliberate safety measure while names, APIs, metadata, and
release practices are unsettled. Internal tools such as `jsonc-parser-study`
are explicitly marked `publish = false`.

## Core library responsibilities

`crates/tilewright/` owns reusable domain and format behavior:

- project discovery and loading;
- parsing and serialization;
- typed and lossless representations;
- project and resource identifiers;
- validation and contextual diagnostics;
- compatibility behavior;
- explicit mutation, diff, and transaction operations; and
- errors suitable for translation by different callers.

The final module and type structure for these capabilities has not been chosen.
Implementations should begin with a concrete, evidenced behavior rather than a
framework built ahead of format knowledge.

The library must remain independent of:

- MCP and any other transport protocol;
- AI models, providers, prompts, or agent hosts;
- OpenCode and Codex;
- terminal presentation and CLI argument parsing;
- GUI frameworks or editor automation;
- cloud services and account systems; and
- private or commercial Tilewright code.

## Adapter responsibilities

### Command-line interface

`crates/tilewright-cli/` maps command-line input into library operations and
renders their results for people and scripts. It may own argument parsing,
output selection, exit codes, and terminal presentation. It must not own a
second parser, validator, or mutation implementation.

### MCP server

`crates/tilewright-mcp/` validates protocol input, defines tools and resources,
invokes library operations, and serializes structured outcomes. It must remain
usable by different MCP clients rather than being designed only for one agent
host. Domain safety belongs primarily in reusable library operations, with the
adapter adding protocol-appropriate permission and confirmation controls.

## Separation of concerns inside the core

Parsing, validation, domain modeling, serialization, and persistence should be
separable even if early implementations keep them close together:

- **Parsing** establishes syntax without pretending that every value's meaning
  is known.
- **Representation** provides typed access while retaining unknown data needed
  for forward compatibility.
- **Validation** reports findings without conflating invalid input with parser
  crashes.
- **Serialization** makes preservation guarantees explicit.
- **Mutation** changes domain concepts rather than arbitrary paths whenever
  practical.
- **Persistence** handles filesystem scope, validation, backups, and atomic
  replacement independently from in-memory edits.

The exact raw-document and typed-view relationship uses an accepted CST/raw-storage-plus-typed-view direction ([ADR 0004](decisions/0004-lossless-json-representation.md)). Silent data
loss is not an acceptable answer.

## Public API principles

Public APIs should be small, typed, and domain-oriented. They should make error
behavior, ownership, identifiers, compatibility, and preservation guarantees
understandable to a caller. Lower-level access may still be necessary for
unknown or extension data.

Before stabilizing an API, establish:

- the evidenced domain contract;
- whether the operation is read-only, in-memory, or persistent;
- how unsupported fields and versions behave;
- diagnostic and failure semantics;
- tests for the promised behavior; and
- the cost of future extension under semantic versioning.

Avoid panics for recoverable input and I/O conditions. Public items require
Rustdoc, and behavior or architecture changes require matching documentation.

## Commercial boundary

The separate private Tilewright Studio repository may build applications around
the public library. Reusable project-format knowledge generally belongs here,
but commercial UX, hosted functionality, accounts, proprietary orchestration,
and private distribution do not. This workspace must never read or copy private
source as an implementation shortcut.
