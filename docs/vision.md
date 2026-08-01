# Project vision and scope

## Mission

Tilewright makes structured project data used by tile-based RPG development
tools understandable and safely operable through reusable Rust APIs. It aims to
replace blind raw-JSON editing with explicit discovery, inspection, validation,
and eventually controlled transformation.

RPG Maker MZ is the first compatibility target. The durable product is not an
editor automation script or a single integration: it is a clean, format-aware
library that can support people, command-line automation, AI systems, editor
integrations, and applications.

## Who Tilewright is for

- Developers building tools around RPG project data.
- Game creators who need inspection, validation, migration, or bulk editing.
- Scripts and continuous-integration jobs that need structured, deterministic
  results.
- AI and MCP clients that require safer operations than arbitrary filesystem
  writes.
- Open-source and commercial applications that need a reusable domain library.

## Long-term goals

Tilewright intends to support:

- discovering projects and resources such as maps, events, database entries,
  tilesets, actors, items, skills, and enemies;
- parsing supported files into typed Rust representations;
- validating references and invariants across files;
- producing contextual diagnostics and human- or machine-readable summaries;
- preserving fields and extension data it does not understand;
- comparing projects or versions semantically;
- previewing and applying focused domain mutations;
- creating or editing selected maps, metadata, events, pages, conditions, and
  dialogue through high-level operations;
- exposing stable operations through a CLI and a thin MCP server; and
- remaining useful to consumers that do not use AI, MCP, or the original editor.

This list is directional. Only capabilities listed as supported in
[`compatibility.md`](compatibility.md) are current product behavior.

## Non-goals and boundaries

### Not editor GUI automation

Tilewright works with project data directly. It must not require GUI automation
or control of a closed-source editor to provide its core functionality.

### Not a generalized engine framework yet

The architecture should avoid needless RPG Maker-specific coupling, but the
project will learn one concrete format before attempting a universal abstraction
for unrelated engines.

### Not an AI framework

The library does not know whether its caller is a person, shell script, GUI,
agent, or model. Provider-specific orchestration belongs outside the core.

### Not the commercial product repository

This public repository owns reusable format knowledge, domain APIs, adapters,
tests, fixtures, and documentation. Proprietary interfaces, hosted services,
commercial AI workflows, account functionality, and distribution mechanisms
belong in a separate private Tilewright Studio repository. The public repository
may not use that private repository as an undeclared source dependency or source
of copied implementation.

### Not the speculative runtime-NPC project

Generated NPC conversations, runtime inference, personality systems, and local
image generation have different runtime and product constraints. They are not
part of the foundational authoring toolkit unless a concrete reusable component
is identified later.

## Development sequence

The sequence is deliberately risk-ordered; research may change priorities.

1. **Foundation and format orientation:** document boundaries, establish
   research and fixture practices, choose a supported Rust version, and add CI.
2. **Read-only inspection:** detect projects, load basic project files, list
   maps and events, summarize data, and validate file presence and basic shape.
3. **Typed models and diagnostics:** add stable identifiers, incremental domain
   types, source context, and project-wide validation reports.
4. **Lossless round trips:** establish the raw/typed layering and unknown-field
   strategy, then verify semantic and formatting preservation.
5. **Controlled mutation:** add previewable, validated, atomic operations for
   selected domain changes.
6. **Adapter expansion:** expose stable library behavior through scriptable CLI
   output and bounded MCP tools with explicit write controls.

## Definition of success

The open-source foundation succeeds when:

- it can safely identify and load legitimate projects from supported versions;
- it presents useful structured information and clear validation failures;
- supported round trips do not destroy unsupported data;
- core behavior is reusable independently of CLI, MCP, and AI tooling;
- CLI and MCP interfaces expose the same domain behavior without reimplementing
  it;
- fixtures are minimal, legal, and understandable;
- format conclusions are evidence-backed and uncertainty remains visible;
- contributors can work without private model or product configuration; and
- commercial and third-party applications can build around the public core
  without weakening its independence.

## Decision discipline

Important design questions are tracked in [`open-questions.md`](open-questions.md).
This vision does not resolve them. Once accepted, a consequential resolution
belongs in an ADR and should be reflected in architecture, compatibility, tests,
and user documentation as applicable.
