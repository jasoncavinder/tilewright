# Tilewright OpenCode setup

This directory keeps project-local OpenCode behavior portable and reviewable.

## Design

- `agents/`: narrow roles with different mutation privileges.
- `commands/`: short entry points for recurring workflows.
- `skills/`: detailed procedures loaded on demand instead of consuming every prompt.
- `opencode.jsonc`: project safety, LSP, watcher, and context settings.
- Root `AGENTS.md`: lean, always-loaded policy and architecture boundaries.

The setup intentionally does **not** duplicate globally configured Context7 or GitHub MCP servers. OpenCode merges global and project configuration, so keeping credentials and general-purpose integrations global avoids drift and accidental secret commits.

Project intent and architecture live in `docs/`, indexed by `docs/README.md`.
Agent definitions should link to those canonical documents rather than restating
or silently extending project decisions.

It also does not add plugins or custom tools yet. Add one only after a workflow repeats enough that a command or skill cannot express it cleanly. The future `tilewright-mcp` binary should remain product code, not an OpenCode-only helper hidden in this directory.

## Commands

- `/orient <task>` — inspect and plan without changing files.
- `/research-format <question>` — investigate RPG Maker MZ data with an evidence ledger.
- `/api-change <proposal>` — design or review a public core-library API change.
- `/implement <approved task>` — implement through the Rust specialist lane.
- `/review [scope]` — read-only review of current changes.
- `/verify [scope]` — run targeted, read-only verification.
- `/quality [scope]` — run the full Rust quality gate.

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

## Agent-managed Git worktrees

OpenCode may be launched normally from the primary checkout:

```sh
cd ~/Projects/tilewright-dev/tilewright
opencode
```

When a task requires repository changes, the top-level coordinator creates a unique linked worktree under the ignored `.worktrees/` directory and a matching `agent/` branch. No wrapper, launcher lock, or pre-created worktree is required.

A representative lifecycle is:

```sh
mkdir -p .worktrees

session_id="mapinfos-parser-$(date -u +%Y%m%d-%H%M%S)-$$"
base_commit="$(git rev-parse HEAD)"

git worktree add \
  -b "agent/$session_id" \
  ".worktrees/$session_id" \
  "$base_commit"
```

After creation, the coordinator records the absolute path, branch, base commit, and task purpose. All implementation, subagent work, review, tests, and verification use that worktree. The primary checkout remains read-only for agent-authored source changes.

The OpenCode process remains rooted at the directory where it was launched, so the coordinator must use the owned worktree's absolute path for file tools and the worktree as the working directory for commands. Every subagent receives that path and branch explicitly.

When work is complete, verified, committed, and clean, the coordinator records the final commit SHA and removes only the worktree:

```sh
git -C ".worktrees/$session_id" \
  status --porcelain=v1 --untracked-files=all

git worktree remove ".worktrees/$session_id"
```

Normal `git worktree remove` refuses a dirty worktree. Agents must never use `--force` or manually delete a worktree directory. Removing a worktree preserves its `agent/` branch and commits for review and integration.

Branches are separate from worktrees. An `agent/` branch may be deleted only with explicit user authorization after proving it is merged into the intended target:

```sh
git merge-base --is-ancestor "agent/$session_id" dev
git branch -d "agent/$session_id"
```

If work, verification, committing, or cleanup is incomplete, the coordinator preserves the worktree and reports its path, branch, status, and next required action. See `AGENTS.md` for the complete ownership and safety policy.

## Codex desktop interoperability

A Codex desktop task created in per-chat Worktree mode already owns an isolated worktree and must not create a nested one. Its host manages that worktree's lifecycle unless the user explicitly authorizes otherwise. Codex Local mode does not establish isolated ownership by itself. See `AGENTS.md` for the common policy.
