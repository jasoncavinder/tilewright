# ADR 0004: Lossless JSON Representation

- **Status:** Proposed
- **Date:** 2026-08-02

## Context

Tilewright needs to load, understand, validate, and safely modify RPG Maker MZ projects. These projects use JSON files that may contain undocumented fields, plugin-generated data, or structures Tilewright has not yet modeled.

According to [ADR 0003](0003-stock-authoring-data-parity.md), Tilewright must preserve unknown and plugin-defined content. Furthermore, the [Safety Model](../safety.md) requires that we do not silently lose unknown data.

The accompanying [representation study](../lossless-json-representation-study.md)
considers several JSON representation strategies:
1. Typed Serde deserialization (`serde_json::Value`).
2. Order-preserving JSON DOMs (`jstrict`, `hifijson`).
3. Concrete Syntax Trees (`jsonc-parser`).
4. Hybrid String Replacement (Raw String + Spans).

The tracked prototype supplies positive evidence for the CST option. Earlier
comparative DOM observations are not retained as decision-grade evidence, and
the proposal does not rely on them. A custom raw-string-and-span mutation engine
could be made safe, but doing so would require Tilewright to own punctuation,
trivia, and stale-span behavior that the CST already provides.

## Decision

We propose that Tilewright adopt a **Concrete Syntax Tree (CST)** architecture:

1. **Storage:** Parse the document into a CST (provisionally `jsonc-parser` 0.33.1) that retains all tokens, including whitespace, comments, and exact numeric/string lexemes.
2. **Typed Views:** Project typed domain models over the CST nodes.
3. **Mutation:** Place validated, typed wrappers around CST edits, then strictly
   revalidate the serialized result before accepting a mutation.

## Rationale

For accepted inputs—valid UTF-8, no BOM, strict JSON syntax, and at most the
parser's observed nesting limit—the prototype provides byte identity when no
mutation occurs. It also demonstrates that CST operations can manage structural
punctuation while retaining lexemes outside an operation's measured mutation
envelope. These are bounded observations, not a production support claim.

## Consequences

If accepted, this proposal would mean:
- **Untouched accepted documents** will remain byte-identical. Invalid UTF-8 is
  rejected before the string parser, and version 0.33.1 rejects a UTF-8 BOM.
  BOM preservation remains unresolved rather than being silently normalized.
- **Touched documents** have operation-specific mutation envelopes. Structural
  edits may change the target container's punctuation, separators, indentation,
  newline layout, and adjacent or owned trivia. Exact preservation outside that
  envelope must be backed by operation-level tests; it is not guaranteed for
  every nominally untouched CST node.
- Strict input excludes comments, so comment-adjacent mutation behavior is not
  part of the currently accepted input domain.
- Mutations must be refused if the target node is ambiguous (for example,
  duplicate decoded keys). `CstObject::get` returns the first match, so wrapper
  operations must enumerate and compare decoded names first.
- Stale snapshot/hash refusal is a future Tilewright wrapper requirement.
- The core library will not expose public JSON DOM types; typed views will be the primary API.
- `CstInputValue::String` escapes unescaped input in the tested version.
  Unchecked surfaces such as `CstInputValue::Number(String)`,
  `CstNumberLit::set_raw_value`, and `CstStringLit::set_raw_value` must remain
  behind validated Tilewright wrappers.
- Every proposed serialized mutation must pass the same strict validation used
  for loading before it can be accepted.

## Alternatives Considered

- **Typed or order-preserving DOMs:** Not selected by this proposal because
  their value-oriented models do not themselves provide the token/trivia
  retention demonstrated by the CST prototype. The tracked study does not claim
  a reproducible comparative serialization result for `serde_json`, `jstrict`,
  or `hifijson`.
- **Hybrid String Replacement (Raw String + Spans):** Not selected because it
  would require a custom structural mutation engine for punctuation, trivia,
  nested edits, and stale spans. This is an inferred maintenance tradeoff, not
  an experimentally proven impossibility.

## Validation

A tracked prototype exercises the following bounded behavior:
- **Structural edits:** Representative object and array insertions/deletions
  remain valid and have exact output assertions. A combined mutation case also
  records exact preservation of tested unknown nested data, numeric lexemes,
  string escapes, and array contents outside the measured edit envelope.
- **Strict syntax:** A strict AST pass with every extension disabled runs before
  CST construction. A lexical preflight rejects non-JSON whitespace and raw C0
  controls that the scanner otherwise accepts. CST construction itself preserves
  comments despite `allow_comments = false`, so it cannot be the strict gate.
- **Mutation safety:** The string constructor quotes names and values, escapes
  required punctuation and controls, and preserves tested Unicode; unchecked
  numeric and raw-literal APIs require wrappers and output revalidation.
- **Diagnostics:** The parser exposes byte ranges plus 1-indexed line and column
  values. Unicode display width remains feature-dependent, and Tilewright's
  final diagnostic representation is unresolved.
