# Tilewright OpenCode setup

This directory keeps project-local OpenCode behavior portable and reviewable.

## Design

- `agents/`: narrow roles with different mutation privileges.
- `commands/`: short entry points for recurring workflows.
- `skills/`: detailed procedures loaded on demand instead of consuming every prompt.
- `opencode.jsonc`: project safety, LSP, watcher, and context settings.
- Root `AGENTS.md`: lean, always-loaded policy and architecture boundaries.

The setup intentionally does **not** duplicate globally configured Context7 or GitHub MCP servers. OpenCode merges global and project configuration, so keeping credentials and general-purpose integrations global avoids drift and accidental secret commits.

It also does not add plugins or custom tools yet. Add one only after a workflow repeats enough that a command or skill cannot express it cleanly. The future `tilewright-mcp` binary should remain product code, not an OpenCode-only helper hidden in this directory.

## Commands

- `/orient <task>` — inspect and plan without changing files.
- `/research-format <question>` — investigate RPG Maker MZ data with an evidence ledger.
- `/api-change <proposal>` — design or review a public core-library API change.
- `/implement <approved task>` — implement through the Rust specialist lane.
- `/review [scope]` — read-only review of current changes.
- `/verify [scope]` — run targeted, read-only verification.
- `/quality [scope]` — run the full Rust quality gate.
- `/session-check` — report whether the current top-level session is isolated for writes.

## Optional global model binding

Models are deliberately not committed so contributors can use their own providers. A user may bind the generic global `rust-implementer` agent in `~/.config/opencode/agents/rust-implementer.md`; the project-local agent supplies Tilewright-specific instructions without naming a provider or model.

For Jason's machine, the global agent can include:

```yaml
model: lmstudio/strand-rust-coder-14b-v1
```

Keep that binding outside this repository. A model should be bound to a role only when it is consistently better for that role; commands and skills remain provider-independent.

## LSP note

`lsp: true` enables OpenCode's LSP integration where supported. Ensure `rust-analyzer` is installed and on `PATH`:

```sh
rustup component add rust-analyzer
rust-analyzer --version
```

Some OpenCode builds still treat the callable LSP tool as experimental. In those builds, launch with:

```sh
OPENCODE_EXPERIMENTAL_LSP_TOOL=true opencode
```

If a V2 build reports LSP as configured but unavailable, that is currently a runtime limitation rather than a Tilewright configuration error.

## Concurrent top-level sessions with Git worktrees

Writable top-level OpenCode sessions must run in separate linked worktrees. The primary checkout is reserved for integration, inspection, and human-controlled lifecycle operations. Subagents stay inside their coordinator's worktree; they do not receive separate worktrees.

Commit this OpenCode setup before creating the first session so new worktrees inherit the helper and rules. From the primary checkout:

```sh
cd ~/Projects/tilewright-dev/tilewright

# Terminal 1: create branch work/mapinfos-parser and launch OpenCode.
.opencode/bin/tilewright-session new mapinfos-parser

# Terminal 2: create an independent branch and launch another OpenCode process.
.opencode/bin/tilewright-session new event-model
```

The default worktree location is the sibling directory:

```text
~/Projects/tilewright-dev/tilewright-worktrees/<slug>/
```

Override it with `TILEWRIGHT_WORKTREE_ROOT` when needed. Session slugs use lowercase letters, digits, and hyphens; branches use `work/<slug>`.

Useful lifecycle commands:

```sh
# Show worktrees and active/stale launcher locks.
.opencode/bin/tilewright-session list

# Verify the current OpenCode process is safe to write.
.opencode/bin/tilewright-session check --write

# Reopen an existing worktree after its prior OpenCode process exits.
.opencode/bin/tilewright-session open mapinfos-parser

# After work/mapinfos-parser has been integrated into main, safely remove it.
.opencode/bin/tilewright-session retire mapinfos-parser
```

The launcher uses a lock under the repository's shared Git directory so a second launcher cannot open the same session worktree concurrently. `retire` refuses to proceed while the session is active, while the worktree is dirty, or while its branch is not fully merged into the selected base. It never force-removes a worktree.

Starting `opencode` manually inside a linked worktree does not establish the launcher lock and therefore fails the write preflight. Use `tilewright-session open <slug>` instead.
