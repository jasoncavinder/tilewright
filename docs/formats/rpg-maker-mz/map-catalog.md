# RPG Maker MZ map catalog contract

This document defines the evidence boundary for Tilewright's first typed
RPG Maker MZ project-data slice: read-only inspection of `data/MapInfos.json`
as a map catalog and hierarchy. It is a research result, not a support claim.

## Question and scope

What is the smallest typed map catalog that Tilewright can expose without
guessing undocumented semantics, collapsing unknown data, or implying that an
inconsistent project is rejected by RPG Maker MZ?

The maintained target begins at RPG Maker MZ 1.10.0. Direct observations in
this contract cover four user-owned projects created by MZ 1.10.0 on the
recorded macOS environment, plus controlled map creation, deletion, hole reuse,
reordering, child creation, and reparenting experiments already recorded in the
[research ledger](research-ledger.md).

## Evidence ledger

| Claim | Evidence | Classification | Confidence | Remaining uncertainty |
| --- | --- | --- | --- | --- |
| `data/MapInfos.json` stores map information and `data/MapNNN.json` stores each map's contents. | `MZ-SCRIPTREF-DB-1.0.0`, `MZ-1.10.0-FRESH-4-2026-08-01`, and the controlled map lifecycle records | Documented and Observed | High for the observed MZ 1.10.0 scope | Other versions, IDs above 999, damaged projects, and converted projects remain unobserved. |
| The root is an array whose index zero is `null`; non-null entries are objects. | `MZ-1.10.0-MAP-INFOS-SHAPE-AUDIT-2026-08-06` | Observed | High across all four audited projects | Editor acceptance of other root or entry shapes is unknown. |
| Every observed object has numeric `id`, numeric `order`, numeric `parentId`, and string `name`. | Shape audit plus controlled lifecycle records | Observed | High across 196 records | The editor's numeric range, tolerance for alternate numeric lexemes, and malformed-field behavior are unknown. |
| Every observed `id` equals its array index and is positive. | Shape audit plus creation, middle deletion, and hole-reuse records | Observed | High through observed ID 189 | Multiple-hole allocation, IDs above 999, manual mismatches, and enforcement remain unknown. |
| `order` is a positive, unique, compact display-order scalar independent of ID and array position. | Shape audit plus top-level reorder, child creation, and reparenting records | Observed and Inferred | High for the observed states; its display role is evidenced by controlled changes | Editor behavior for duplicate, missing, zero, negative, or gapped orders remains unknown. |
| `parentId` is zero for top-level maps and otherwise names another map ID. | Official script reference, shape audit, child creation, and reparenting records | Documented and Observed | High for the observed states | Missing parents, self-parenting, cycles, parent deletion, and deeper hierarchy remain untested. |
| Map-info objects are open to additional fields. | Shape audit found two key sets; five records contain an additional boolean `quick` field | Observed | High that uniform closed-object decoding would reject legitimate observed data | The meaning, version scope, and ownership of `quick` and future extra fields remain unknown. |
| A corresponding three-digit `MapNNN.json` exists for every observed non-null map-info entry. | Fresh-project audit and controlled map lifecycle records | Observed | High through ID 189 | Requiredness, orphan handling, IDs above 999, and editor behavior after manual inconsistency remain unknown. |

## Derived shape audit

On 2026-08-06, a read-only aggregate audit inspected only JSON types, decoded
property names, counts, numeric relationships, and reference existence in the
four authorized MZ 1.10.0 `MapInfos.json` files already identified by
`MZ-1.10.0-FRESH-4-2026-08-01`. It emitted no map names, project values, raw
documents, excerpts, or per-project manifests.

The audit observed:

- four array roots with lengths 2, 2, 190, and 6;
- four `null` entries, all at array index zero;
- 196 object entries;
- 191 objects with `expanded`, `id`, `name`, `order`, `parentId`, `scrollX`,
  and `scrollY`;
- five objects with the same fields plus boolean `quick`;
- consistent required-field JSON types for `id`, `name`, `order`, and
  `parentId`;
- no ID/index mismatches, duplicate IDs, duplicate orders, missing parent
  references, self-parent references, nonpositive IDs or orders, or noncompact
  per-document order sequences; and
- integer-valued `id`, `order`, and `parentId` fields, while some observed
  scroll coordinates were fractional.

The aggregate audit corroborates the controlled lifecycle evidence. It does
not establish editor validation rules for manually malformed input.

## Bounded typed contract

The first experimental typed slice exposes only:

- a positive map ID whose decoded numeric value equals its array index;
- the decoded map name string;
- a positive display order; and
- either a top-level parent marker (`parentId == 0`) or a positive parent map
  ID.

The typed projection must:

1. consume the already loaded lossless `data/MapInfos.json` document from a
   `ProjectSnapshot`;
2. leave the raw document and every unknown field untouched;
3. refuse a typed catalog when the root, an entry, or a required field is
   missing, ambiguous, has an unexpected JSON kind, or uses an unsupported
   integer form;
4. refuse duplicate decoded occurrences of a required field rather than use
   the CST backend's first-match behavior;
5. distinguish shape/refusal errors from contextual relationship findings;
6. report missing parents, parent cycles, duplicate display orders, missing
   corresponding map documents, orphan three-digit map documents, and
   classified map paths that do not encode a positive ID as deterministic
   findings rather than claims about editor rejection; and
7. expose no mutation or serialization operation.

Fields such as `expanded`, `scrollX`, `scrollY`, and `quick` remain retained in
the lossless raw document but outside the first typed API because they are not
needed to list maps or construct the parent relationship.

## Fixture implications

Tests should generate minimal strict JSON in temporary project trees. No vendor
project or default data is needed. Separate synthetic cases should cover:

- one top-level map and one child;
- a legal `null` hole;
- an unknown extra key retained by the untouched raw document;
- every root, entry, required-field, duplicate-key, and integer refusal;
- missing parents, cycles, duplicate order, missing map files, and orphan map
  files; and
- deterministic ID and display ordering.

These fixtures establish Tilewright behavior only. They do not establish that
MZ accepts or rejects the malformed cases.

## Remaining unknowns and next experiments

- Delete a parent map with a child in a disposable user-owned copy and observe
  the resulting hierarchy and files.
- Create multiple map-ID holes and observe the next allocation choice.
- Test deeper hierarchy and manually inconsistent parent/order values only when
  needed for a compatibility claim.
- Observe another named MZ version at or above 1.10.0 before claiming the
  contract is version-invariant.
- Determine the filename relationship for IDs above 999 before synthesizing
  paths outside the evidenced three-digit family.
