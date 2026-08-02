# Local format-research material

Everything in this directory except this README is intentionally ignored.

Use only authorized, user-owned research projects. Do not commit RPG Maker
runtime code, bundled assets, default databases, sample games, commercial
plugins, deployment output, or private project data.

## Layout

All paths below are relative to the canonical `.local-research/` directory in
the primary integration checkout, not to an agent's linked Git worktree.

- `sources/` contains user-managed source projects. Placement here grants agents
  standing permission to inspect them for relevant Tilewright research, but
  agents must not modify or execute these originals.
- `workspaces/<session-id>/` contains agent-owned experiment copies. Use a
  collision-resistant session identifier, create the directory only if it does
  not already exist, and add an ownership manifest containing the session ID,
  coordinator, creation time, canonical source path, provenance, purpose, and
  copy method.

Agents may freely create, modify, rename, and delete material inside their own
workspace, subject to the host's sandbox and confirmation controls. Project
scripts, plugins, binaries, runtimes, and generated games may be run only from
an experiment copy, never from `sources/`.

Treat executable project material as untrusted. Its working directory does not
confine it. Run it only in a host sandbox that restricts writes to the assigned
workspace, blocks unauthorized network access, and withholds credentials, or
after the user explicitly accepts unsandboxed risk for the named experiment. Do
not use symlinks that escape the workspace.

## Hard repository boundary

Raw or generated proprietary material must remain in this ignored directory.
Never force-add it, copy it into tracked paths, embed it in fixtures or
documentation, or retain it in patches, logs, screenshots, or commit messages.
Tracked research findings may contain only derived observations,
non-identifying provenance, and safe reproduction procedures.

Local contents are not backed up by Git. Agents may clean up only their own
completed experiment workspace after revalidating its canonical path, ownership
manifest, and symlinks. Preserve incomplete or ambiguously owned workspaces and
report their paths.

See `AGENTS.md` and the `rpg-maker-format-research` skill for the complete
research and evidence policy.
