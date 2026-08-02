# ADR 0003: Stock authoring-data parity

- **Status:** Accepted
- **Date:** 2026-08-01

## Context

Tilewright's long-term goal is to load, understand, validate, and safely modify
RPG Maker MZ projects. “Parity with RPG Maker MZ” needs a bounded meaning so it
does not become an unsupported promise to reproduce the editor GUI, game
runtime, deployment system, or arbitrary plugin behavior.

The format is proprietary and extensible. Official documentation, controlled
editor observations, and legitimate project files can establish stock behavior,
but plugins may execute arbitrary JavaScript and create arbitrary data. Project
files may also contain unknown fields, future-version structures, user-created
resources, and host-created entries. Tilewright therefore cannot equate parity
with a closed list of paths or complete knowledge of every project entry.

Compatibility is also multidimensional. Recognizing a project does not prove it
can be loaded; loading one file does not establish its semantics; understanding
a value does not prove that Tilewright can rewrite it losslessly; and one safe
mutation does not establish general write support.

## Decision

Tilewright will pursue **stock authoring-data parity** for explicitly supported
RPG Maker MZ versions and capabilities. For each stated scope, that means
working toward all of the following independently testable properties:

1. recognizing legitimate stock authoring project roots;
2. loading the relevant stock project data without silently losing unknown or
   extension content;
3. representing and interpreting evidenced stock concepts and relationships;
4. reporting syntax, structural, referential, compatibility, and safety
   findings with useful context;
5. preserving the documented fidelity guarantee through supported round trips;
6. applying supported domain mutations with the same relevant project semantics
   as the editor; and
7. producing results that the named editor version can reopen and continue to
   edit, playtest, and deploy within the tested scope.

Each property is a separate compatibility claim. Support must be bounded by the
operation, project-data area, named MZ version, relevant platform or filesystem
conditions, and preservation guarantee. The maintained version floor in
[ADR 0002](0002-rpg-maker-mz-version-floor.md) does not make every later version
or capability supported automatically.

RPG Maker MZ is the behavioral oracle for editor interoperability, but not the
only standard for Tilewright diagnostics. Tilewright may report integrity or
safety problems that the editor tolerates. Documentation and diagnostics should
distinguish, without prematurely fixing public enum names:

- input observed to be rejected by the editor;
- behavior outside Tilewright's evidenced or safely writable scope;
- internal project inconsistencies or broken references; and
- advisory findings that may be intentional.

Unknown and plugin-defined content remains open-world:

- discovery must permit and report it;
- reading and supported edits must preserve it according to the stated fidelity
  guarantee;
- the core library must not execute plugin code to discover its meaning; and
- semantic support for a particular plugin may be added later as an explicit,
  independently tested compatibility profile.

This parity target concerns authoring project data. Reproducing the editor GUI,
the JavaScript game runtime, arbitrary plugin execution, or every deployment
artifact is not required. Deployment and runtime files may still be classified,
inspected, or validated when a separately evidenced capability needs them.

Capabilities advance through an evidence-to-support loop: format evidence,
bounded contract, legal fixtures or generated test data, implementation,
differential editor testing where applicable, and an explicit compatibility
matrix update. The [capability roadmap](../capability-roadmap.md) defines the
development sequence and exit gates. Research or a successful one-off parse
alone does not establish support.

## Rationale

Stock authoring-data parity expresses the user's useful outcome without
requiring Tilewright to clone unrelated proprietary application behavior. It
keeps the library reusable and domain-oriented while making editor
interoperability observable and testable.

Independent capability claims prevent partial progress from being mistaken for
general compatibility. The open-world plugin policy makes comprehensive loading
possible through preservation and explicit uncertainty rather than an
impossible attempt to enumerate every extension.

## Consequences

- The compatibility matrix must remain granular. A capability may be supported
  for one data area and version while later stages remain experimental, not
  implemented, unsupported, or unknown.
- Typed APIs must coexist with a preservation strategy for unknown data. The
  exact representation remains an open decision and needs a focused experiment
  before broad parser APIs are committed.
- Validation must not collapse editor incompatibility, Tilewright limitations,
  project errors, and advice into one undifferentiated failure.
- Mutation support requires editor-interoperability observations, preservation
  tests, preview and validation behavior, and eventually failure-safe
  persistence.
- Plugin-specific semantics require explicit profiles or contracts; generic
  loading must remain useful without them.
- Local user-owned projects may serve as private differential-test inputs, but
  committed regression data must satisfy the repository's fixture and
  intellectual-property policy.
- Current compatibility remains unchanged. No capability becomes supported by
  accepting this goal or roadmap.

## Alternatives considered

- **Claim parity after parsing all known stock JSON files:** rejected because
  syntax acceptance does not establish semantics, preservation, validation, or
  safe mutation.
- **Delay implementation until the entire format is exhaustively researched:**
  rejected because plugins and future versions make exhaustive enumeration
  impossible; bounded vertical capabilities can advance safely.
- **Execute plugins to discover all extension behavior:** rejected because it
  is unsafe, non-deterministic, and incompatible with a reusable data library.
- **Require byte-for-byte reproduction of every editor save:** rejected as a
  blanket rule. Exact preservation is valuable for untouched data, but the
  appropriate lexical, structural, or semantic guarantee for edited documents
  remains to be decided and stated per capability.
