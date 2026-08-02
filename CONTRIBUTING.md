# Contributing to Tilewright

Tilewright is early in its development, and careful evidence and narrow changes
are more valuable than broad implementations built on assumptions. Contributions
should make the supported behavior clearer, safer, or better tested.

## Start here

Before contributing, read:

1. the [project vision](docs/vision.md);
2. the [architecture](docs/architecture.md);
3. the [safety model](docs/safety.md); and
4. any applicable [architecture decisions](docs/decisions/README.md) and format
   research.

`AGENTS.md` contains the always-loaded operating rules for AI-assisted work.
Detailed OpenCode workflows live under `.opencode/` so they can be loaded only
when needed.

## Choose the right kind of change

- **Format research:** record evidence and uncertainty before promising format
  behavior. Follow the [RPG Maker MZ research workflow](docs/formats/rpg-maker-mz/README.md).
- **Core behavior:** implement parsing, validation, domain modeling,
  serialization, and mutation in `crates/tilewright/`.
- **Adapters:** keep the CLI and MCP crates focused on input validation,
  translation, presentation, and protocol concerns.
- **Public API changes:** describe the intended contract, error behavior,
  preservation implications, and compatibility cost before implementation.
- **Fixtures:** use only minimal, redistributable data with explicit provenance.
- **Architecture changes:** update an existing ADR or propose a new one rather
  than hiding a long-lived decision inside an implementation.

Do not decide items listed in [open questions](docs/open-questions.md) implicitly.
A contribution may propose a resolution, but consequential decisions require
maintainer agreement and normally an ADR.

## Format evidence

RPG Maker MZ is proprietary, so familiarity or memory is not sufficient evidence
for a format contract. Classify material format claims as:

- **Documented:** supported by official documentation or another authoritative
  contract.
- **Observed:** directly verified in legitimate, user-owned project files or
  generated output.
- **Inferred:** a reasoned conclusion from evidence that is not definitive.
- **Unknown:** unresolved.

Separate a field's JSON shape from its meaning and invariants. Preserve unknown
fields by default and do not commit proprietary application code, assets, or
sample-game content as evidence.

Maintainer-led RPG Maker MZ work targets version 1.10.0 and newer under
[ADR 0002](docs/decisions/0002-rpg-maker-mz-version-floor.md). Contributions
for older versions are welcome when they supply a bounded version contract,
evidence, legal fixtures or generated data, regression tests, and matching
compatibility documentation; they must not weaken the established 1.10.0+
behavior.

## Fixtures

Follow [`fixtures/README.md`](fixtures/README.md). Every nontrivial fixture must
state its purpose, origin, creation method, redistribution status, removed
sensitive or proprietary content, and expected behavior. Prefer one small
fixture per behavior.

Parser and writer changes should include round-trip and unknown-field
preservation tests when relevant. Never update expected output without inspecting
and explaining the change.

## Development workflow

Inspect the worktree before editing:

```sh
git status --short
git branch --show-current
```

The primary checkout is reserved for integration and remains read-only for agent-authored source changes. A top-level OpenCode coordinator launched from the primary checkout creates a unique linked worktree under `.worktrees/` before making changes. Host-provisioned per-task worktrees, such as Codex Worktree mode, already satisfy this requirement. The complete ownership and cleanup policy is in `AGENTS.md` and the [OpenCode setup](.opencode/README.md).

Keep changes focused. Do not reformat unrelated files, upgrade dependencies incidentally, or mix research conclusions with unrelated refactoring. Avoid panics for recoverable library input or I/O errors, and document public APIs.

### Branches and pull requests

Create short-lived branches from `dev` using `feat/`, `fix/`, `docs/`,
`research/`, `refactor/`, `test/`, or `chore/` followed by a concise kebab-case
description. Agent-owned branches follow the `agent/` convention in `AGENTS.md`.
Host-provisioned Codex Worktree mode may instead supply a `codex/` branch; that
host-managed prefix is accepted without renaming.

Open normal contributions against `dev`. Pull-request titles use a conventional
prefix such as `feat:`, `fix:`, `docs:`, `research:`, `refactor:`, `test:`,
`chore:`, `ci:`, or `build:` because squash-merged titles become repository
history. Only a promotion pull request from `dev` targets `main`.

Both long-lived branches require pull requests, passing checks, and resolved
review conversations. Changes into `dev` are squash merged; promotions into
`main` use merge commits. See the complete
[GitHub repository governance](docs/maintainers/github.md) policy.

## Verification

Run targeted checks while iterating. Before marking a change ready, run:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets --all-features
cargo test --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --all-features --no-deps
```

Report commands actually run and their results. Do not weaken a gate to hide a
failure or describe an unrun check as passing.

GitHub Actions runs the same gate on current stable Rust and an Ubuntu runner.
That CI environment is not yet a minimum-supported-Rust-version or platform
support promise.

## Licensing and project boundaries

Contributions are made under the repository's [MPL-2.0 license](LICENSE). Do not
contribute material you lack the right to redistribute.

This repository contains only the open-source foundation. Do not read from,
copy from, write to, or depend on the separate commercial Tilewright Studio
repository as part of work here.
