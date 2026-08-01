---
description: Review current Tilewright changes without modifying files
agent: reviewer
subtask: true
---

Review this scope:

$ARGUMENTS

If no scope is supplied, inspect all staged and unstaged changes and relevant untracked source or test files from `git status --short`.

Begin with `git status --short`, `git diff --stat`, `git diff`, and `git diff --cached`. Read enough surrounding code and tests to review in context. Do not modify files or run fixes. Return the structured findings required by the reviewer agent.
