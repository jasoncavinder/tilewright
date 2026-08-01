# RPG Maker MZ research ledger

This ledger indexes active and completed format investigations. It begins empty
because product intent is not evidence of proprietary format behavior.

## Investigation index

| ID | Question | Status | Primary classification | Last updated |
| --- | --- | --- | --- | --- |
| _None yet_ | | | | |

Use stable, descriptive IDs such as `mz-project-detection-001`. A completed
investigation may move to a focused sibling document; keep its index row here.

## Investigation template

Copy this section for each bounded question.

```markdown
## <ID>: <question>

- **Status:** Proposed | Active | Complete | Blocked
- **Behavior depending on this:** <parser, validator, API, or writer behavior>
- **Scope:** <file, field, editor/runtime versions, and explicit exclusions>
- **Last updated:** YYYY-MM-DD

### Evidence ledger

| Claim | Evidence | Classification | Confidence and rationale | Unresolved alternatives |
| --- | --- | --- | --- | --- |
| <one falsifiable claim> | <source or observation ID> | Documented / Observed / Inferred / Unknown | <high/medium/low plus why> | <what else could explain it> |

### Evidence records

#### <source-or-observation-id>

- **Kind:** Official documentation | Observation | Controlled experiment | Runtime behavior | Community corroboration
- **Version/environment:** <exact version and relevant platform/tooling>
- **Locator:** <URL, document section, fixture path, or reproducible procedure>
- **Accessed/observed:** YYYY-MM-DD
- **Summary:** <paraphrase or minimal original observation>
- **Redistribution:** <why any committed material is safe>

### Established findings

- <documented or observed conclusion and its limits>

### Inferences

- <inference, reasoning, confidence, and what would falsify it>

### Risks and unknowns

- **Unknown fields:** <risk>
- **Ordering/encoding:** <risk>
- **Identifiers/references:** <risk>
- **Version/plugin differences:** <risk>

### Next experiment

<The smallest controlled change or observation that resolves the most important unknown.>

### Implementation implications

- **Safe now:** <read, parse, validate, expose raw data, etc.>
- **Not justified:** <types, normalization, or writes that evidence does not support>
- **Fixtures/tests needed:** <minimal fixture and assertions>
```

## Ledger quality rules

- One row states one falsifiable claim.
- A classification describes evidence strength, not implementation priority.
- Confidence includes a reason and the important limitation.
- Evidence locators must let another maintainer reproduce or re-check the claim.
- Observations record the relevant version and the controlled action.
- Conflicts and unknowns remain visible.
- A parser accepting one example does not establish semantics or compatibility.
- Writer behavior requires stronger evidence and preservation tests than
  read-only inspection.
