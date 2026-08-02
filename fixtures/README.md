# Test fixtures

Fixtures make format behavior reviewable and reproducible without importing
proprietary or private project material.

This directory currently contains no data fixtures.

## Safety policy

Use only:

- small synthetic data written specifically for Tilewright;
- minimal files created by a contributor in a project they are authorized to
  use and redistribute;
- sanitized data with a clearly documented origin; or
- appropriately licensed third-party material whose license is recorded and
  compatible with repository use.

Do not commit RPG Maker application files or code, bundled artwork, audio,
fonts, default databases, vendor sample-game assets, commercial plugin content,
or third-party projects without explicit redistribution rights. Remove personal
paths, names, identifiers, and unrelated content.

If redistribution status is uncertain, do not add the fixture. Record the
research procedure or generate equivalent synthetic data instead.

## Design rules

- Give each fixture one primary format behavior or regression.
- Keep it small enough to understand during review.
- Preserve realistic cross-file references only when the behavior depends on
  them.
- Explain why every unusual field exists through the provenance record or test
  assertions.
- Pair parser/writer fixtures with round-trip tests and explicit unknown-field
  preservation tests when relevant.
- Do not update expected output blindly; inspect and explain every difference.
- Keep malformed fixtures narrowly malformed and state the expected diagnostic.

## Fixture index

Add one row for every nontrivial fixture or cohesive fixture directory.

| Fixture | Purpose | Origin | Consuming tests |
| --- | --- | --- | --- |
| _None yet_ | | | |

## Required provenance record

Each fixture needs a record here or in a README beside a cohesive fixture set:

```text
Fixture: <relative path>
Purpose: <single behavior or regression>
Origin: <synthetic, contributor-created, or licensed source>
Creation method: <minimal reproducible steps>
Relevant version/environment: <editor/runtime/tool versions or not applicable>
Redistribution status: <why it is safe to commit>
Sensitive/proprietary content removed: <yes/no and what>
Expected behavior: <parse, reject, validate, or round-trip expectation>
Expected fidelity: <exact, structural, semantic, or not applicable>
Intentionally malformed data: <none or exact fields and expected diagnostics>
Consuming tests: <test modules or names>
```

For a controlled editor experiment, document the action and resulting minimal
observation in the [research ledger](../docs/formats/rpg-maker-mz/research-ledger.md)
without copying proprietary application content.

## Review checklist

A fixture review should confirm:

- provenance and redistribution rights are explicit;
- the data is minimal for its stated purpose;
- expected invariants and intentional failures are documented;
- tests actually consume it;
- personal and proprietary content is absent; and
- round-trip and unknown-field assertions match the claimed fidelity.
