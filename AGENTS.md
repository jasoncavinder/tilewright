# Tilewright AI Operating Guide

This file contains the small set of project rules that should be present in every agent session. Keep procedures in `.opencode/skills/` so they are loaded only when needed.

## Project mission

Tilewright is an open-source Rust toolkit for reading, validating, transforming, and eventually authoring tile-based RPG project data. The first supported target is RPG Maker MZ, but the core API must not unnecessarily hard-code one editor or one AI host.

## Architecture boundaries

- `crates/tilewright/` is the durable, format-aware library.
- `crates/tilewright-cli/` is a human- and script-facing adapter over the library.
- `crates/tilewright-mcp/` is a thin MCP adapter over the library.
- Dependency direction is inward: CLI and MCP may depend on `tilewright`; `tilewright` must not depend on either adapter.
- The core library must not depend on OpenCode, MCP concepts, model providers, UI frameworks, or commercial Tilewright products.
- Commercial code belongs in the separate `tilewright-studio` repository. Do not read from, copy from, or write to that sibling repository during work in this repository unless the user explicitly changes the scope.

## Evidence and format research

- RPG Maker MZ is proprietary. Do not guess undocumented file semantics or present inference as fact.
- Label format conclusions as observed, documented, inferred, or unknown.
- Prefer official documentation and direct observation of user-owned project files; use community material as corroboration, not unquestioned truth.
- Do not commit RPG Maker application code, bundled artwork, sample-game assets, or other proprietary material.
- Use minimal synthetic or user-created fixtures and record their provenance.
- Preserve unknown data by default. A parser must not silently discard fields it does not yet understand.

## Engineering rules

- Favor small, typed, testable changes over broad abstractions built ahead of evidence.
- Keep parsing, validation, domain modeling, and serialization separable.
- Make writes explicit and failure-safe; never leave a project half-written.
- Avoid panics in library code for recoverable input or I/O errors.
- Document public APIs and include regression tests for every format behavior or bug fix.
- When behavior or architecture changes, update the relevant README or ADR in the same change.

## Concurrent top-level OpenCode sessions

- The primary checkout is the integration checkout and is read-only for agent-authored changes. A top-level OpenCode session that may edit tracked files must be launched through `.opencode/bin/tilewright-session new <slug>` or `.opencode/bin/tilewright-session open <slug>`.
- Each concurrently writable top-level session owns exactly one linked Git worktree and one `work/<slug>` branch. Never run two top-level OpenCode processes in the same worktree.
- Before the first mutation, the coordinating agent must run `.opencode/bin/tilewright-session check --write`. If it does not report `write_isolation=ready`, do not edit; tell the user to relaunch through the session helper.
- The coordinator and all of its subagents share the coordinator's current worktree. Subagents must not create, switch, move, remove, lock, unlock, or prune worktrees and must not switch branches.
- Do not read from or modify another session's worktree. Cross-session coordination happens through the user, committed branches, diffs, or later integration—not shared mutable files.
- Worktree creation, retirement, merging, and branch deletion are human-controlled lifecycle operations. The helper refuses active, dirty, or unmerged worktrees during retirement.

## Verification

Run targeted checks while iterating. Before declaring a change ready, use the `rust-quality-gate` skill or `/quality` command. Do not claim a check passed unless it was actually run and its result inspected.

## Git and release safety

- Inspect `git status --short` and the current branch before edits.
- Do not commit, push, publish crates, create releases, delete branches, force-remove worktrees, or discard working-tree changes unless the user explicitly requests that operation.
- Never use `git reset --hard`, `git clean -f`, or force deletion as a convenience.

## Available project skills

- `rpg-maker-format-research`: evidence-first investigation of project data.
- `fixture-design`: safe, minimal fixture design and provenance.
- `public-api-evolution`: review changes to the durable library API.
- `rust-quality-gate`: targeted and workspace-wide Rust verification.
