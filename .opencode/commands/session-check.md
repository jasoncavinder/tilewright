---
description: Check whether this top-level OpenCode session is isolated for writes
agent: tilewright
---

Current session isolation:

!`.opencode/bin/tilewright-session check --write 2>&1 || true`

Explain whether this session may safely modify files. Do not modify files. If isolation is not ready, provide the exact launcher command pattern needed to create or reopen a managed worktree session.
