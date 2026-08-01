---
name: fixture-design
description: Create or review minimal, legally safe Tilewright test fixtures with explicit provenance, one behavioral purpose per fixture, and strong round-trip regression coverage.
license: MPL-2.0
compatibility: opencode
metadata:
  audience: contributors
  workflow: testing
---

# Fixture design

Use this whenever tests need RPG project data.

## Safety and provenance

- Use synthetic data or files created by the contributor in a project they are authorized to use.
- Do not copy vendor sample projects, bundled art, audio, fonts, application code, or third-party game content.
- Record how the fixture was created and what may be redistributed.
- Strip unrelated content and personal paths or identifiers.

## Design rules

- One fixture should demonstrate one format behavior or regression.
- Keep fixtures human-reviewable and as small as the format permits.
- Preserve realistic cross-file references when the behavior depends on them.
- Pair a fixture with assertions that explain why each important field exists.
- For parsers and writers, include round-trip tests and explicit unknown-field preservation tests.
- Do not update expected output blindly. Inspect and explain every change.

## Suggested provenance entry

Add or update `fixtures/README.md` with:

```text
Fixture: <path>
Purpose: <single behavior>
Origin: <synthetic or user-created>
Creation method: <minimal steps>
Redistribution status: <why it is safe to commit>
Sensitive/proprietary content removed: <yes and what>
```

## Review output

State the fixture's purpose, provenance, minimality, expected invariants, and tests that consume it.
