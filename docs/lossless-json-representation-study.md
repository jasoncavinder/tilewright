# Lossless JSON Representation Study

## Executive Recommendation

Tilewright should continue evaluating a **Concrete Syntax Tree (CST)**
architecture, provisionally using `jsonc-parser` 0.33.1. The tracked prototype
provides positive evidence for strict syntax gating, no-op lexical preservation,
bounded structural edits, and source diagnostics. ADR 0004 remains Proposed;
the study does not establish production support.

The proposed layering is:

1. Apply a transient strict AST syntax pass before CST construction without
   interpreting numeric lexemes as Rust numeric values.
2. Store accepted text as a CST and project typed domain views over its nodes.
3. Expose only validated mutation wrappers, strictly revalidating serialized
   output before accepting an edit.

## Caller Use Case and Scope

Tilewright needs to load, understand, validate, and eventually modify RPG Maker
MZ project data without silently losing unknown or extension content. This study
evaluates representation mechanics with independently created synthetic JSON;
it makes no new proprietary-format claim.

## Non-Goals

- Implementing production parsing, typed views, persistence, or transactions.
- Exposing public JSON-manipulation APIs.
- Selecting Tilewright's final diagnostic representation.
- Establishing stale-snapshot refusal or performance readiness.

## Evaluation Criteria and Status

| Criterion | Weight | Status | Current evidence | Remaining work |
| --- | ---: | --- | --- | --- |
| Preservation safety | 30% | Observed | `no_op_round_trip_preserves_every_declared_matrix_case`; `duplicate_targets_are_refused_without_changing_bytes` | Production integration and broader format fixtures |
| Mutation precision | 25% | Partially observed | String, numeric-wrapper, exact insertion/deletion envelopes, adversarial-neighbor preservation, and duplicate-refusal tests | Scalar replacement contracts and stale-snapshot refusal |
| Typed-view integration | 15% | Unknown | None | Design typed projections over retained CST nodes |
| Source spans and diagnostics | 10% | Observed | `direct_parser_reports_exact_diagnostic_location` | Select Tilewright's diagnostic API and Unicode-width policy |
| Malformed-input behavior | 5% | Partially observed | Strict rejection, direct BOM, and nesting-limit tests | Define the byte-decoding boundary and broader resource limits |
| Maintenance health and license | 5% | Documented | Pinned 0.33.1 dependency and upstream MIT metadata | Ongoing dependency policy and upgrade tests |
| Dependency footprint | 5% | Partially observed | Workspace lockfile | Set an accepted dependency budget |
| Performance feasibility | 5% | Unknown | None | Reproducible representative benchmarks |

Weights describe decision importance; they are not combined into a readiness
score while criteria remain Unknown or Partially observed.

## Candidates Considered

The design space included typed Serde deserialization, order-preserving DOMs,
CSTs, and a custom raw-string-plus-span editor. The tracked harness evaluates
only `jsonc-parser` 0.33.1. Earlier observations about `serde_json`, `jstrict`,
and `hifijson` are not retained as decision-grade comparative evidence, so this
study does not claim that their serializers were reproducibly rejected.

The CST proposal is based on the candidate's demonstrated positive behavior.
A custom span editor remains possible, but requiring Tilewright to own
punctuation, trivia, nested-edit, and stale-span behavior is an inferred
maintenance disadvantage.

## Experimental Method

The workspace package [`jsonc-parser-study`](../tools/jsonc-parser-study/README.md)
contains deterministic Rust tests using only synthetic inputs. The canonical
command is:

```bash
cargo test -p jsonc-parser-study --all-targets --all-features --locked
```

Each Observed claim below names the test that reaches the claimed API. Passing
the harness demonstrates only these bounded observations.

## Executed Synthetic No-Op Matrix

The `NO_OP_CASES` table used by
`no_op_round_trip_preserves_every_declared_matrix_case` contains exactly:

- `compact_no_newline`
- `pretty_whitespace`
- `unusual_whitespace`
- `key_order_stable`
- `unknown_top_level`
- `unknown_nested_deep`
- `array_with_nulls`
- `mixed_nested_array`
- `number_forms`
- `unicode_literal`
- `unicode_escaped`
- `string_escapes`
- `duplicate_keys`
- `crlf_endings`
- `final_newline`
- `no_final_newline`
- `empty_object`
- `empty_array`
- `deeply_nested`

The test parses each case through the strict gate, constructs a CST, serializes
without mutation, and compares the resulting UTF-8 string exactly.

## Strict Input Domain

The currently accepted prototype domain is:

- valid UTF-8 presented as `&str`;
- no UTF-8 BOM;
- JSON whitespace limited to space, tab, LF, and CR;
- no unescaped U+0000 through U+001F inside strings;
- every `ParseOptions` extension disabled; and
- at most 512 nested arrays in the observed boundary test.

Invalid UTF-8 is rejected before this Rust string API and therefore remains a
separate loading-boundary design question. Version 0.33.1 directly rejects a
leading UTF-8 BOM; the proposal does not silently strip it or claim BOM
preservation.

Strict validation uses `parse_to_ast` with comment collection disabled and all
extensions disabled. A lexical preflight closes scanner gaps for non-JSON
whitespace and raw C0 controls. This pass does not deserialize numbers into a
Rust numeric type: tests accept and preserve a 2,000-digit integer and a large
exponent lexeme.

`CstRootNode::parse` is not itself the strict gate. It asks for comments as
tokens and consequently preserves comments even when `allow_comments` is false.
Comments are outside the accepted strict-input domain.

## Preservation and Mutation Envelope

Untouched accepted documents in the executed matrix serialize byte-identically.
That observation does not extend to rejected bytes, BOM-prefixed input, or
untested syntax.

Touched documents have operation-specific envelopes:

- string construction escapes required property-name/value characters and
  preserves tested Unicode;
- scalar or raw-literal replacement changes the selected literal and requires
  output revalidation;
- insertion/deletion may change punctuation, separators, indentation, newline
  layout, and adjacent or owned trivia in the target container; and
- exact preservation outside that envelope is claimed only where an
  operation-level test establishes it.

Representative first, middle, last, and sole deletions for objects and arrays
have exact output assertions in addition to strict semantic validation.
Insertions exercise beginning, middle, and end positions, compact and multiline
layouts, LF/CRLF, and unusual legal indentation. A combined removal/insertion
case confirms exact survival of neighboring unknown nested data, numeric
lexemes, string escapes, and array contents outside the measured edit envelope.
Because comments are outside the accepted domain, comment-adjacent
mutation behavior is not currently promised.

## Mutation and Refusal Implications

`CstInputValue::String` accepts an unescaped Rust string and, in 0.33.1, quotes
names and values, escapes required quotes, backslashes, and controls, and
preserves the tested Unicode and non-BMP characters.

`CstInputValue::Number(String)`, `CstNumberLit::set_raw_value`, and
`CstStringLit::set_raw_value` accept unchecked lexical text and can create
invalid output. Production APIs must hide these surfaces behind typed or
validated wrappers. The prototype's numeric wrapper validates before mutation,
and every proposed output must pass the strict gate again.

Duplicate decoded property names make name-based mutation ambiguous. A
prototype wrapper enumerates decoded names and refuses missing or ambiguous
targets before changing the CST. Tests include literal, nested, and
escape-equivalent duplicate names and confirm byte identity after refusal.
Stale snapshot/hash refusal remains future wrapper work.

## Diagnostics

The direct diagnostic test confirms an ASCII trailing-comma error's exact byte
range, error kind/message, and 1-indexed line and column. Column calculation for
Unicode text depends on jsonc-parser's `error_unicode_width` feature, which this
harness does not select. Tilewright's final diagnostic representation remains
unresolved.

## Dependency and Performance Risks

The harness pins `jsonc-parser` 0.33.1 through `Cargo.lock`. Dependency
maintenance must include rerunning these regression tests on upgrade. If the
crate becomes unsuitable, Tilewright would need another CST implementation or a
custom editor.

No performance conclusion is retained. Representative project-size benchmarks,
methodology, and an acceptance budget are still required.

## Unresolved Questions

- What typed-view API should sit over the CST?
- How should stale snapshots and semantic project diffs be represented?
- What constitutes a stable project or resource identifier?
- What is Tilewright's final diagnostic representation and Unicode-width policy?
- Should BOM-prefixed input be rejected permanently or preserved by a separate
  byte-level envelope?
- What performance and dependency budgets are acceptable?

## Evidence Ledger

| Claim | Evidence | Classification | Confidence |
| --- | --- | --- | --- |
| Strict wrapper rejects documented extensions and lexical gaps | `strict_parser_rejects_json_extensions_and_non_json_lexemes` | Observed | High |
| Validation preserves large numeric lexemes without numeric interpretation | `strict_parser_accepts_large_lexemes_without_numeric_interpretation` | Observed | High |
| Version 0.33.1 accepts 512 nested arrays and rejects 513 | `strict_parser_enforces_the_observed_nesting_limit` | Observed | High |
| Default CST parsing accepts trailing commas | `direct_parser_default_accepts_trailing_commas` | Observed | High |
| Explicit strict options reject trailing commas | `direct_parser_strict_options_reject_trailing_commas` | Observed | High |
| CST construction preserves comments despite `allow_comments = false` | `direct_cst_parse_preserves_comments_despite_strict_comment_option` | Observed | High |
| Direct CST parsing rejects a UTF-8 BOM | `direct_parser_rejects_utf8_bom` | Observed | High |
| Parser diagnostics expose the tested range, kind, line, and column | `direct_parser_reports_exact_diagnostic_location` | Observed | High |
| Executed no-op matrix is byte-identical | `no_op_round_trip_preserves_every_declared_matrix_case` | Observed | High |
| String constructor escapes required characters and preserves tested Unicode names and values | `string_constructor_escapes_required_characters_and_preserves_unicode` | Observed | High |
| Raw number/string literal surfaces can produce invalid JSON | `unchecked_raw_literal_apis_can_create_invalid_json` | Observed | High |
| Validated number wrapper refuses before mutation | `validated_number_refuses_invalid_input_before_mutation` | Observed | High |
| Representative structural deletions have exact valid output envelopes | `object_deletion_envelope_is_exact_across_positions`; `array_deletion_envelope_is_exact_across_positions` | Observed | High |
| Representative insertion envelopes preserve exact documented invariants | `insertion_envelope_covers_positions_and_layouts` | Observed | High |
| Removal and insertion preserve tested adversarial neighbors outside the envelope | `mutations_preserve_adversarial_neighbors_outside_the_envelope` | Observed | High |
| Duplicate targets can be refused without changing bytes | `duplicate_targets_are_refused_without_changing_bytes` | Observed | High |
| A custom span editor would add maintenance burden | Architectural analysis | Inferred | Medium |
