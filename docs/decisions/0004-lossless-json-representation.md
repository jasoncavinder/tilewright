# ADR 0004: Lossless JSON Representation

- **Status:** Accepted
- **Date:** 2026-08-04

## Context

Tilewright needs to load, understand, validate, and safely modify RPG Maker MZ projects. These projects use JSON files that may contain undocumented fields, plugin-generated data, or structures Tilewright has not yet modeled.

According to [ADR 0003](0003-stock-authoring-data-parity.md), Tilewright must preserve unknown and plugin-defined content. Furthermore, the [Safety Model](../safety.md) requires that we do not silently lose unknown data.

The accompanying [representation study](../lossless-json-representation-study.md)
considers several JSON representation strategies:
1. Typed Serde deserialization with insertion-ordered extension storage.
2. An order-preserving `serde_json::Value` DOM with typed projections.
3. Concrete Syntax Trees (`jsonc-parser`).
4. Hybrid String Replacement (Raw String + Spans).

The tracked study now compares the first three strategies against the same
synthetic inputs. Typed extension storage and the ordered DOM preserve tested
unknown meaning, but normalize outer trivia and string escapes; typed storage
also moves known fields ahead of extensions. The CST retains all tested bytes,
including duplicate names, and permits explicit ambiguity refusal. A custom
raw-string-and-span mutation engine could be made safe, but doing so would
require Tilewright to own punctuation, trivia, and stale-span behavior that the
CST already provides.

## Decision

Tilewright will adopt a **Concrete Syntax Tree (CST)** architecture:

1. **Storage:** Parse the document into a CST (provisionally `jsonc-parser` 0.33.1) that retains all tokens admitted by the accepted strict-input contract, including whitespace and exact numeric/string lexemes. Comments are a backend capability, not currently accepted input.
2. **Typed Views:** Project typed domain models over the CST nodes.
3. **Mutation:** Place typed wrappers with type/shape checks around CST edits, then strictly
   revalidate the serialized result before accepting a mutation. Explicitly require staging, cloning, rollback, or equivalent isolation so failed validation cannot alter the caller-visible accepted document.

## Rationale

For accepted inputs—valid UTF-8, no BOM, strict JSON syntax, and at most 512 nested arrays (513 rejected)—the prototype provides byte identity when no
mutation occurs. Other resource/nesting behavior remains unestablished. This holds for the synthetic matrix and an authorized local
corpus of 252 MZ 1.10.0 JSON files. It also demonstrates a read-only typed view,
an exact tested replacement envelope, and structural punctuation
management while retaining tested lexemes outside each operation's measured
envelope. These are bounded observations, not a production support claim.

## Consequences

This decision means:
- **Untouched accepted documents** will remain byte-identical. A prototype byte
  boundary explicitly refuses invalid UTF-8 and BOM-prefixed input before CST
  construction, and version 0.33.1 also rejects a UTF-8 BOM. Whether BOM refusal
  becomes permanent production policy remains unresolved rather than being
  silently normalized.
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

- **Typed Serde with extension storage:** The executed candidate retains tested
  unknown meaning in an insertion-ordered extension map, but serialization
  moves its known field ahead of extensions and normalizes outer trivia and
  decoded string escapes.
- **Order-preserving value DOM:** The executed candidate retains tested input
  key order and arbitrary-precision numeric lexemes, but normalizes outer trivia
  and string escapes and collapses duplicate names to the last value.
- **Hybrid String Replacement (Raw String + Spans):** Not selected because it
  would require a custom structural mutation engine for punctuation, trivia,
  nested edits, and stale spans. This is an inferred maintenance tradeoff, not
  an experimentally proven impossibility.

## Validation

A tracked prototype exercises the following bounded behavior:
- **Comparative representation:** All three roadmap-level strategies run over
  the same synthetic cases with explicit byte-fidelity assertions. The CST is
  the only candidate that remains exact for every accepted comparison case and
  retains duplicates for wrapper-level refusal.
- **Authorized local no-op:** The strict gate and CST reproduce all 252 JSON
  files from four copied user-owned MZ 1.10.0 `data` directories byte-for-byte;
  only aggregate results and safe procedure are retained.
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
  numeric and raw-literal APIs require wrappers and output revalidation. A
  minimal typed view and an exact tested replacement envelope preserve tested neighbors exactly
  and refuse duplicate targets before mutation.
- **Byte boundary:** A prototype distinguishes explicit invalid-UTF-8, BOM, and
  strict-syntax refusal without normalization.
- **Diagnostics:** The parser exposes byte ranges plus 1-indexed line and column
  values. Unicode display width remains feature-dependent, and Tilewright's
  final diagnostic representation is unresolved.
- **Performance feasibility:** A reproducible, non-gating synthetic probe
  completes strict parse plus CST serialization of a 391 KB input in about 15
  ms on one documented arm64 macOS host. No production budget is claimed.
