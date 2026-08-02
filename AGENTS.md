# Tilewright AI Operating Guide

This file contains the small set of project rules that should be present in every agent session. Keep procedures in `.opencode/skills/` so they are loaded only when needed.

## Project mission

Tilewright is an open-source Rust toolkit for reading, validating, transforming, and eventually authoring tile-based RPG project data. The first supported target is RPG Maker MZ, but the core API must not unnecessarily hard-code one editor or one AI host.

## Required orientation

- Start with `README.md` for current maturity and the public project overview.
- Use `docs/README.md` as the canonical documentation index.
- Read `docs/vision.md`, `docs/architecture.md`, and `docs/safety.md` before changing scope, boundaries, or write behavior.
- Check `docs/compatibility.md` before claiming support and `docs/open-questions.md` before making a consequential design choice.
- Record proprietary-format evidence under `docs/formats/`; do not turn intent or memory into a format claim.

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

## Concurrent top-level agent sessions

- The primary checkout is the integration checkout and is read-only for agent-authored source changes.
- OpenCode may be launched normally from the primary checkout. No launcher, session lock, or pre-created worktree is required.
- Before its first source-file mutation, each top-level coordinator must create one uniquely named linked Git worktree under `.worktrees/` with a matching `agent/` branch. Worktree creation itself is the permitted lifecycle exception to the primary checkout's read-only rule.
- Use a collision-resistant identifier such as `<task-slug>-<UTC-timestamp>-<process-id>`. The branch is `agent/<identifier>` and the path is `.worktrees/<identifier>`.
- Unless the user specifies another base, create the worktree from the commit checked out when the session began. Uncommitted changes in the startup checkout are not inherited; do not copy, alter, or discard them.
- A coordinator owns only a worktree it created during the current session or a per-task worktree explicitly provisioned by the host, such as Codex Worktree mode. Never adopt, reuse, enter, modify, remove, or prune another session's worktree.
- Record the owned worktree's absolute path, branch, base commit, and purpose immediately. After creation, perform all reads related to the change, edits, write-capable commands, tests, reviews, and verification inside that worktree.
- Coordinators and their subagents share one worktree. Pass its absolute path and branch to every subagent. Subagents must not create, select, commit, remove, or otherwise manage worktrees or branches.
- A user request to change repository files implicitly authorizes the coordinator to create its worktree and make focused local commits on its owned `agent/` branch unless the user says not to commit. It does not authorize pushing, merging, rebasing shared branches, publishing, or modifying the primary checkout.
- Make coherent checkpoint commits during longer tasks when doing so materially protects recoverable progress.
- A coordinator may automatically remove its owned, agent-created worktree only after the requested work is complete, required verification has been inspected, all intended changes including untracked files are committed, and `git status --porcelain=v1 --untracked-files=all` is empty.
- Before removal, record the branch and final commit SHA. Use only normal `git worktree remove <path>` from outside the owned worktree. Never use `--force` or delete the directory manually.
- Removing a worktree preserves its branch and commits. Leave the branch in place for review or integration. Delete it only with explicit user authorization and only after proving it is merged into the intended target with `git merge-base --is-ancestor`; use normal `git branch -d`, never `-D`.
- If implementation, verification, committing, or cleanup is incomplete or fails, preserve the worktree and report its exact path, branch, status, completed checks, and required next action.
- Host-provisioned worktrees, including Codex Worktree mode, are cleaned up by their host unless the user explicitly authorizes another lifecycle action.
- Cross-session coordination happens through the user, committed branches, diffs, or later integration—not shared mutable files.
- Do not read from or modify another session's worktree, except for the narrow read-only local-research exception below.

## Shared local research evidence

The primary integration checkout's `.local-research/` directory is a
user-managed, ignored evidence store. It is a narrow exception to the rule
against reading another checkout.

An agent working in an isolated worktree may read from that directory only when:

- the user explicitly requests research using those materials;
- the exact research directory and its provenance are identified;
- the directory is ignored by Git;
- access is limited to the requested `.local-research/` subtree; and
- the filesystem sandbox grants read access to the directory.

This exception is strictly read-only. Agents must not:

- create, modify, rename, delete, move, execute, or change permissions on files
  in the shared research directory;
- stage ignored material with `git add -f`;
- copy proprietary contents into tracked paths, fixtures, patches, tool output,
  or documentation;
- inspect other untracked files or other areas of the primary checkout; or
- follow symlinks that resolve outside the authorized research root.

Agents should begin with path, file-type, size, and metadata manifests. File
contents may be inspected only when necessary for the bounded research question.
Committed findings must contain derived observations and exact provenance, not
proprietary source material.

Before access, verify that the requested material is ignored and that its
canonical path remains under the authorized `.local-research/` root. Treat
concurrent modification as a possible source of inconsistent observations.

The primary checkout remains read-only for all agent-authored changes. This
exception grants evidence access, not write authorization or general
cross-worktree access.

## Verification

Run targeted checks while iterating. Before declaring a change ready, use the `rust-quality-gate` skill or `/quality` command. Do not claim a check passed unless it was actually run and its result inspected.

## Git and release safety

- Inspect `git status --short`, the current branch, and the current commit before creating or changing a worktree.
- A request to change repository files authorizes focused local commits on the coordinator's owned `agent/` branch unless the user explicitly says not to commit.
- Do not push, merge, rebase shared branches, publish crates, create releases, or delete branches without explicit user authorization.
- Worktree removal is authorized only under the completed-and-clean conditions in the worktree policy above.
- Never use `git reset --hard`, `git clean -f`, force branch deletion, force worktree removal, or manual directory deletion as a convenience.

## Available project skills

- `rpg-maker-format-research`: evidence-first investigation of project data.
- `fixture-design`: safe, minimal fixture design and provenance.
- `public-api-evolution`: review changes to the durable library API.
- `rust-quality-gate`: targeted and workspace-wide Rust verification.
