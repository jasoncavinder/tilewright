# jsonc-parser-study

This is an internal, non-publishable research and verification tool for
Tilewright. It compares three representation strategies and records bounded
observations about `jsonc-parser` as a possible Concrete Syntax Tree (CST)
backend for Tilewright's lossless JSON representation.

## Purpose

- Exercise a strict AST validation pass before CST construction.
- Verify exact round trips for the documented synthetic input matrix.
- Compare typed Serde extension storage, an order-preserving value DOM, and a
  CST against the same synthetic inputs.
- Prototype a read-only typed CST view and an exact scalar-replacement envelope.
- Demonstrate explicit invalid-UTF-8 and BOM refusal at a byte boundary.
- Record string escaping and the risks of unchecked raw literal APIs.
- Measure the lexical envelope of representative structural edits.
- Prototype refusal of ambiguous duplicate-key mutations.
- Provide non-gating synthetic timing and privacy-safe aggregate corpus tools.
- Provide executable evidence for the bounded claims in proposed ADR 0004.

## Scope and Provenance

All committed tests and timing inputs are minimal synthetic JSON created for
this study and redistributable under the repository license. They contain no
vendor or third-party project data.

The optional `corpus_noop` example may be run only against an authorized copy
inside the ignored local-research sandbox. It emits aggregate counts only. No
project file, filename, path manifest, excerpt, digest, or per-file diagnostic
may be copied into Git or retained in logs.

## Reproduction

To run the study tests and verify the claims:

```bash
cargo test -p jsonc-parser-study --all-targets --all-features --locked
```

Run the non-gating synthetic timing probe in an optimized build:

```bash
cargo run --release -p jsonc-parser-study --example measure --locked
```

Run the aggregate no-op checker only against a uniquely owned, ignored copy of
authorized project data:

```bash
cargo run --release -p jsonc-parser-study --example corpus_noop --locked -- \
  <owned-workspace>/corpus
```

## Expected Result

All tests should pass. The timing probe prints machine-specific CSV and has no
pass/fail threshold. The corpus checker fails on read errors or byte changes and
prints only aggregate counts. These results demonstrate only the behaviors
named by the tests and accompanying study; they do not accept ADR 0004 or
establish a production implementation.

Remaining gates include production typed-view design, the final diagnostic API,
dependency-maintenance and performance budgets, production mutation wrappers,
and later stale-snapshot/persistence safety.
