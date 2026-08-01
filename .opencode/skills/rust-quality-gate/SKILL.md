---
name: rust-quality-gate
description: Run bounded targeted or workspace-wide Rust formatting, compilation, test, lint, and documentation checks for Tilewright, then report exact results without hiding failures.
license: MPL-2.0
compatibility: opencode
metadata:
  audience: contributors
  workflow: verification
---

# Rust quality gate

Use this for implementation readiness, regression verification, or PR review.

## Rules

- Read `AGENTS.md` and inspect the requested scope.
- Do not run `cargo update`, install tools, publish crates, or modify source files.
- Run each unchanged command at most once unless the user explicitly requests repetition for a flaky test.
- Record failures instead of weakening checks.
- Prefer targeted checks during iteration and the full gate before declaring readiness.

## Targeted gate

For a single affected package, substitute its package name for `<package>`:

```sh
cargo check -p <package> --all-targets --all-features
cargo test -p <package> --all-targets --all-features
cargo clippy -p <package> --all-targets --all-features -- -D warnings
```

Run a named regression test directly when one exists. Add `cargo fmt --all --check` when Rust source changed.

## Full workspace gate

Run in this order and stop only when later checks would be meaningless:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

If the workspace does not yet define features, `--all-features` remains harmless. If a command is unsupported by the current Cargo version, report that fact rather than silently changing the intended gate.

## Report

Return:

```text
Rust quality gate
Scope: <targeted package/test or workspace>
Result: <pass|fail|blocked>

Checks
- <command>: <pass|fail|not run>

Failures / blockers
- <concise actionable detail or none>

Working tree
- <clean/dirty and whether checks generated tracked changes>
```
