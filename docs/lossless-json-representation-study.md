# Lossless JSON Representation Study

## Executive Recommendation

The completed bounded comparison supports selecting a **Concrete Syntax Tree
(CST)** architecture, provisionally using `jsonc-parser` 0.33.1. Typed Serde
extension storage and an order-preserving value DOM retain unknown meaning but
do not retain all source bytes. The CST retains the complete tested lexical
form, supports a read-only typed projection, and performs a controlled typed
scalar replacement within an exact measured envelope. ADR 0004 is now Accepted,
but this study does not establish production support.

The accepted architectural direction was evaluated through this provisional
layering:

1. Apply a transient strict AST syntax pass before CST construction without
   interpreting numeric lexemes as Rust numeric values.
2. Store accepted text as a CST and project typed domain views over its nodes.
3. Expose only validated mutation wrappers, strictly revalidating serialized
   output before accepting an edit.

## Caller Use Case and Scope

Tilewright needs to load, understand, validate, and eventually modify RPG Maker
MZ project data without silently losing unknown or extension content. This study
evaluates representation mechanics with independently created synthetic JSON
and one aggregate no-op observation over authorized local MZ 1.10.0 data. No
proprietary input is committed or reproduced.

## Non-Goals

- Implementing production parsing, typed views, persistence, or transactions.
- Exposing public JSON-manipulation APIs.
- Selecting Tilewright's final diagnostic representation.
- Establishing stale-snapshot refusal, a performance budget, or production
  readiness.

## Evaluation Criteria and Status

| Criterion | Weight | Status | Current evidence | Remaining work |
| --- | ---: | --- | --- | --- |
| Preservation safety | 30% | Observed | Synthetic no-op matrix, duplicate refusal, and `MZ-1.10.0-CST-NOOP-2026-08-03` over 252 authorized local files | Production integration, edited/plugin data, and later versions |
| Mutation precision | 25% | Partially observed | String and numeric wrappers, exact scalar replacement/insertion/deletion envelopes, adversarial-neighbor preservation, and duplicate refusal | Production wrappers and operation-specific expansion |
| Typed-view integration | 15% | Partially observed | Minimal typed string projections over ordered DOM and retained CST nodes | Design production domain projections and error types |
| Source spans and diagnostics | 10% | Observed | `direct_parser_reports_exact_diagnostic_location` | Select Tilewright's diagnostic API and Unicode-width policy |
| Malformed-input behavior | 5% | Observed for tested boundary | Strict rejection, explicit invalid-UTF-8/BOM refusal, and nesting-limit tests | Production diagnostics and broader resource limits |
| License | 2.5% | Documented | MIT license verified via immutable upstream 0.33.1 [`Cargo.toml`](https://github.com/dprint/jsonc-parser/blob/041f112d0dd6ffb7e181a471c2de5a15e9420b69/Cargo.toml) and [`LICENSE`](https://github.com/dprint/jsonc-parser/blob/041f112d0dd6ffb7e181a471c2de5a15e9420b69/LICENSE) | Ongoing dependency policy |
| Maintenance health | 2.5% | Unknown | Pinned 0.33.1 dependency | Ongoing dependency policy, backend conformance tests, upgrade review, and an exit strategy |
| Dependency footprint | 5% | Partially observed | Workspace lockfile | Set an accepted dependency budget |
| Performance feasibility | 5% | Observed, non-gating | Reproducible synthetic release-mode timing probe on one documented host | Representative production workloads and an accepted budget |

Weights describe decision importance; they are not combined into a readiness
score while criteria remain Unknown or Partially observed.

## Candidates Considered

The executed comparison uses the same synthetic inputs for three roadmap-level
strategies:

1. **Typed deserialization with extension storage:** a typed `known: String`
   field plus flattened insertion-ordered `IndexMap<String,
   serde_json::Value>` extensions.
2. **Order-preserving document model with typed views:**
   `serde_json::Value` with `preserve_order` and `arbitrary_precision`, plus a
   typed string projection.
3. **Lossless syntax representation:** the existing strict gate and
   `jsonc-parser` 0.33.1 CST, plus a unique-property typed projection.

The first two deliberately use the same current JSON implementation so the
comparison isolates representation strategy rather than unrelated parser
behavior. Typed extension storage moves the known field ahead of extensions
when serialized and both value models decode string escapes and normalize outer
trivia. The ordered DOM preserves input key order, and arbitrary-precision
numbers retain the tested numeric lexemes. Both value models collapse duplicate
object keys unless typed deserialization rejects a duplicate known field. The
CST retains all tested bytes and lets a wrapper refuse ambiguous decoded names.

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

## Executed Comparative Matrix

`three_required_strategies_have_comparable_no_op_results` runs each accepted
input through all three strategies. “Exact” means the serialized UTF-8 bytes
equal the input; every non-exact output is also passed through the strict gate.

| Synthetic case | Typed extensions | Ordered DOM | CST |
| --- | --- | --- | --- |
| Compact, known field first, nested numeric extension | Exact | Exact | Exact |
| Unknown field before known field | Reordered | Exact | Exact |
| Pretty outer whitespace and final newline | Normalized | Normalized | Exact |
| Escaped known string | Escape decoded | Escape decoded | Exact |
| Numeric and escaped-string lexemes in unknown nested data | String escape decoded | String escape decoded | Exact |
| CRLF outer layout and trailing newline | Normalized | Normalized | Exact |

The typed and ordered-DOM views decode the same known string value in every
case. Separate mutation assertions show that both value models retain the
unknown data's JSON meaning while changing lexical form. A duplicate known name
is rejected by typed deserialization, collapsed to the last value by the DOM,
and retained by the CST; the CST typed wrapper then refuses the ambiguous
lookup without changing bytes.

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

## Authorized Local No-Op Observation

`MZ-1.10.0-CST-NOOP-2026-08-03` copied only the `data` directories from the
four user-owned fresh MZ 1.10.0 projects already recorded by
`MZ-1.10.0-FRESH-4-2026-08-01` into a unique ignored workspace. The aggregate
corpus tool inspected 252 JSON files: all 252 were valid UTF-8, had no BOM,
passed the strict gate, and serialized from the CST byte-identically. There
were no strict rejections, changed outputs, symlinks, or read errors. Note: The
`corpus_noop` process success alone does not prove the complete recorded
invariant; every printed counter must be inspected.

This bridges the synthetic no-op result to those exact authorized files only.
It does not establish mutation fidelity, edited or plugin-generated data,
other MZ versions, a production loader, or editor reopen compatibility. Raw
inputs, paths, filenames, excerpts, and per-file results remain ignored and are
not retained.

## Strict Input Domain

The currently accepted prototype domain is:

- valid UTF-8 presented as `&str`;
- no UTF-8 BOM;
- JSON whitespace limited to space, tab, LF, and CR;
- no unescaped U+0000 through U+001F inside strings;
- every `ParseOptions` extension disabled; and
- at most 512 nested arrays in the observed boundary test.

`byte_boundary_can_refuse_invalid_utf8_and_bom_without_normalizing` demonstrates
a byte-level wrapper that explicitly distinguishes invalid UTF-8, a leading
UTF-8 BOM, and strict syntax rejection before CST construction. Version 0.33.1
also directly rejects a leading UTF-8 BOM. This establishes that refusal is
implementable without normalization; whether permanent production policy
should reject or separately preserve BOM-prefixed input remains a maintainer
decision.

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
- a minimal typed CST string view reads one uniquely named property without
  changing bytes;
- replacing that uniquely named string changes only its literal in the tested
  compact and CRLF layouts, while an ambiguous duplicate target is refused
  without changing bytes;
- scalar or raw-literal replacement changes the selected literal and requires
  output revalidation;
- insertion/deletion may change punctuation, separators, indentation, newline
  layout, and adjacent or owned trivia in the target container; and
- exact preservation outside that envelope is claimed only where an
  operation-level test establishes it.

Representative first, middle, last, and sole deletions for objects and arrays
have exact output assertions in addition to strict semantic validation.
Insertions have exact output assertions across beginning, middle, and end
positions, compact and multiline layouts, LF/CRLF, and unusual legal
indentation. A combined removal/insertion case confirms exact survival of
neighboring unknown nested data, numeric lexemes, string escapes, and array
contents outside the measured edit envelope.
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
but the string replacement prototype does not perform type checking or
validation-before-acceptance; the test performs revalidation after mutation.
Every proposed output must pass the strict gate again.

Duplicate decoded property names make name-based mutation ambiguous. A
prototype wrapper enumerates decoded names and refuses missing or ambiguous
targets before changing the CST. Tests include literal, nested, and
escape-equivalent duplicate names and confirm byte identity after refusal.
The same unique-property lookup now backs a minimal typed string view and an
exact tested replacement envelope using `CstInputValue::String`; exact compact and CRLF
outputs preserve tested neighboring lexemes. Stale snapshot/hash refusal
remains future wrapper work for later loading and persistence capabilities, not
a representation-selection experiment.

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

`cargo run --release -p jsonc-parser-study --example measure --locked` generates
the same deterministic object-shaped inputs for all three strategies, performs
parse plus serialization, and emits CSV. One arm64 macOS 26.6 run with Rust
1.97.1 measured:

| Input bytes | Typed extensions | Ordered DOM | Strict gate + CST |
| ---: | ---: | ---: | ---: |
| 808 | 18.75 µs | 16.57 µs | 68.33 µs |
| 77,356 | 1.12 ms | 0.78 ms | 3.13 ms |
| 390,956 | 3.81 ms | 3.07 ms | 15.41 ms |

The CST path includes both the transient strict AST pass and CST construction,
and its output remains the full input size; value-model outputs are smaller
because they normalize lexical form. These are single-run per-iteration
averages, not statistical medians; the simple probe does not isolate allocation
or I/O, and no threshold is enforced. The result establishes bounded
feasibility only. Representative production workloads, repeated statistical
benchmarking, and an accepted budget remain future work.

## Unresolved Questions

- What production typed-view and error API should sit over the CST?
- How should stale snapshots and semantic project diffs be represented?
- What constitutes a stable project or resource identifier?
- What is Tilewright's final diagnostic representation and Unicode-width policy?
- Should BOM-prefixed input be rejected permanently or preserved by a separate
  byte-level envelope? Explicit refusal is demonstrated but not yet accepted as
  policy.
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
| Typed extension storage and an ordered DOM preserve tested unknown meaning but normalize documented lexical forms | `three_required_strategies_have_comparable_no_op_results`; `typed_mutations_preserve_unknown_meaning_but_not_all_lexemes` | Observed | High for the synthetic comparison corpus and selected crate features |
| The ordered DOM collapses duplicate names while typed deserialization rejects a duplicate known field; CST retains both and permits explicit refusal | `duplicate_names_distinguish_refusal_from_silent_collapse` | Observed | High for the tested duplicate form |
| A minimal typed CST view reads a decoded string without changing source bytes | `cst_typed_view_reads_without_changing_bytes` | Observed | High for the tested unique string field |
| An exact tested replacement envelope changes only the selected literal in tested compact and CRLF layouts and refuses duplicate targets unchanged | `cst_typed_scalar_replacement_has_an_exact_envelope` | Observed | High for the tested envelopes |
| A byte wrapper can distinguish and refuse invalid UTF-8, BOM-prefixed input, and strict syntax failure before CST construction | `byte_boundary_can_refuse_invalid_utf8_and_bom_without_normalizing` | Observed | High for the tested byte sequences; production policy remains undecided |
| All 252 authorized local MZ 1.10.0 JSON files pass the strict gate and no-op serialize from CST byte-identically | `MZ-1.10.0-CST-NOOP-2026-08-03`; `corpus_noop` | Observed | High for the exact ignored corpus and dependency version |
| String constructor escapes required characters and preserves tested Unicode names and values | `string_constructor_escapes_required_characters_and_preserves_unicode` | Observed | High |
| Raw number/string literal surfaces can produce invalid JSON | `unchecked_raw_literal_apis_can_create_invalid_json` | Observed | High |
| Validated number wrapper refuses before mutation | `validated_number_refuses_invalid_input_before_mutation` | Observed | High |
| Representative structural deletions have exact valid output envelopes | `object_deletion_envelope_is_exact_across_positions`; `array_deletion_envelope_is_exact_across_positions` | Observed | High |
| Representative insertion envelopes preserve exact documented invariants | `insertion_envelope_covers_positions_and_layouts` | Observed | High |
| Removal and insertion preserve tested adversarial neighbors outside the envelope | `mutations_preserve_adversarial_neighbors_outside_the_envelope` | Observed | High |
| Duplicate targets can be refused without changing bytes | `duplicate_targets_are_refused_without_changing_bytes` | Observed | High |
| Strict gate plus CST is slower than the two value models but completes the tested 391 KB parse/serialize probe in about 15 ms on one documented host | `measure` release-mode example | Observed | Medium: single-host, non-statistical feasibility probe only |
| A custom span editor would add maintenance burden | Architectural analysis | Inferred | Medium |
