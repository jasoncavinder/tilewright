---
description: Implement an approved Tilewright Rust task
agent: rust-implementer
---

Current writable-session preflight:

!`.opencode/bin/tilewright-session check --write 2>&1 || true`

Implement this approved task:

$ARGUMENTS

If the preflight does not report `write_isolation=ready`, stop without editing and tell the user to relaunch from the primary checkout with `.opencode/bin/tilewright-session new <slug>`.

Verify important assumptions against the repository. Follow `AGENTS.md`, use relevant project skills, add focused tests, and run applicable checks. Do not commit, push, publish, or expand the scope without explaining why.
