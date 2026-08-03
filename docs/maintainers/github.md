# GitHub repository governance

This document records the intended collaboration and automation settings for
the public repository at <https://github.com/jasoncavinder/tilewright>. GitHub
repository settings and active rulesets enforce the policy; this file explains
the expected state so that configuration drift can be reviewed.

## Branch model

Tilewright has two long-lived branches:

- `dev` is the default integration branch. Normal contributions and automated
  dependency updates target it.
- `main` is the stable promotion branch. It accepts only pull requests whose
  head branch is `dev`.

No permanent release, staging, or hotfix branch exists. Add one only when a
concrete release or support process requires it.

Short-lived human branches use one of these prefixes followed by a concise
kebab-case description:

- `feat/`
- `fix/`
- `docs/`
- `research/`
- `refactor/`
- `test/`
- `chore/`

Coordinator-created worktree branches follow the collision-resistant `agent/`
convention in `AGENTS.md`. Host-provisioned Codex Worktree mode may use
`codex/`; the PR policy accepts both prefixes. Dependabot owns `dependabot/`
branches.

## Pull requests and merges

All updates to `dev` and `main`, including maintainer updates, go through pull
requests after the initial repository bootstrap.

- Pull requests into `dev` use squash merge so that integration history remains
  linear.
- Promotion pull requests from `dev` into `main` use a merge commit so that
  development commit identities and promotion boundaries remain visible.
- Rebase merges are disabled.
- Pull-request titles use a conventional prefix such as `feat:`, `fix:`,
  `docs:`, `research:`, `refactor:`, `test:`, `chore:`, `ci:`, or `build:`.

Both long-lived branches require the `Rust quality`, `Dependency review`,
`PR policy`, and `CodeQL (Rust)` checks and require review conversations to be
resolved. Required approval count is zero while the project has only one
maintainer because GitHub does not allow authors to approve their own pull
requests. Increase it when a second active maintainer is available.

CodeQL check enforcement uses a staged rollout to avoid locking the no-bypass
rulesets: first establish a successful `CodeQL (Rust)` pull-request run, then
add that exact check context to both rulesets before merging the workflow
change.

Pull requests into `dev` must be tested with the latest `dev`. The `main`
ruleset does not require strict synchronization: after a merge-commit promotion,
`main` contains a promotion commit that is intentionally absent from `dev`, so
requiring `dev` to contain the latest `main` would deadlock the next promotion.
The promotion checks still test GitHub's proposed merge commit.

Force pushes and branch deletion are blocked. `dev` additionally requires a
linear history. There is no routine owner bypass; if a broken rule or GitHub
outage makes recovery impossible, change the narrowest repository setting
needed, document the reason, restore the rule, and verify the resulting history.

## Continuous integration

CI runs the contributor quality gate on current stable Rust across Ubuntu, macOS,
and Windows runners. This is a development gate, not a
minimum-supported-Rust-version or platform-support promise. Those compatibility
decisions remain open.

Pull requests also receive dependency review, policy, and CodeQL checks. Clippy
provides the compiler-integrated Rust lint gate, while CodeQL advanced setup
analyzes Rust in `none` build mode. GitHub's default-setup REST endpoint does
not accept Rust for this repository, so the pinned advanced workflow supplies
the supported path instead. Dependabot monitors Cargo and GitHub Actions, and
all tracked action references are pinned to full commit SHAs.

No workflow publishes crates, creates GitHub releases, or distributes binaries.
Release automation should be added only after the versioning, platform, and
publication questions in `docs/open-questions.md` are resolved.

## Repository security settings

The intended repository settings are:

- Actions default token permissions are read-only and workflows may not approve
  pull requests.
- Dependabot alerts and security updates are enabled.
- Secret scanning and push protection are enabled.
- CodeQL advanced setup scans Rust; default setup remains disabled because its
  REST endpoint does not accept Rust for this repository.
- Private vulnerability reporting is enabled and documented in
  [the security policy](../../.github/SECURITY.md).
- Issues are enabled; wiki, Projects, and Discussions are disabled initially.

Security or automation changes should preserve least privilege, avoid secrets in
fork-triggered workflows, and prefer GitHub-owned actions pinned to immutable
commit SHAs.
