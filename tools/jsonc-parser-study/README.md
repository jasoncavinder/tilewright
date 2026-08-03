# jsonc-parser-study

This is an internal, non-publishable research and verification tool for
Tilewright. It records bounded observations about `jsonc-parser` as a possible
Concrete Syntax Tree (CST) backend for Tilewright's lossless JSON
representation.

## Purpose

- Exercise a strict AST validation pass before CST construction.
- Verify exact round trips for the documented synthetic input matrix.
- Record string escaping and the risks of unchecked raw literal APIs.
- Measure the lexical envelope of representative structural edits.
- Prototype refusal of ambiguous duplicate-key mutations.
- Provide executable evidence for the bounded claims in proposed ADR 0004.

## Scope and Provenance

All tests use minimal, synthetic JSON fixtures generated directly in the test code.
No proprietary RPG Maker MZ data is included or required.

## Reproduction

To run the study tests and verify the claims:

```bash
cargo test -p jsonc-parser-study --all-targets --all-features --locked
```

## Expected Result

All tests should pass. Passing demonstrates only the behaviors named by the
tests and the accompanying study; it does not accept ADR 0004 or establish a
production implementation.

Remaining gates include typed-view design, stale-snapshot refusal, the final
diagnostic API, dependency-maintenance policy, performance evaluation,
production mutation wrappers, and persistence/write safety.
