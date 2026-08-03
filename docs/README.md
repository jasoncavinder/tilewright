# Tilewright documentation

This directory is the durable home for Tilewright's project intent,
architecture, format evidence, and proposed or accepted decisions.

## Where knowledge belongs

| Information | Canonical location |
| --- | --- |
| Public introduction and current maturity | Root [`README.md`](../README.md) |
| Goals, non-goals, and success criteria | [`vision.md`](vision.md) |
| Evidence-to-support development sequence and gates | [`capability-roadmap.md`](capability-roadmap.md) |
| Crate boundaries and engineering constraints | [`architecture.md`](architecture.md) |
| Data-preservation, write, and MCP safety requirements | [`safety.md`](safety.md) |
| Current support and compatibility terminology | [`compatibility.md`](compatibility.md) |
| Deliberately unresolved decisions | [`open-questions.md`](open-questions.md) |
| Architectural decisions (Proposed and Accepted) | [`decisions/`](decisions/README.md) |
| Lossless JSON representation study | [`lossless-json-representation-study.md`](lossless-json-representation-study.md) |
| Evidence about proprietary formats | [`formats/`](formats/rpg-maker-mz/README.md) |
| Fixture provenance and expected behavior | [`../fixtures/README.md`](../fixtures/README.md) |
| Contributor process and quality gates | [`../CONTRIBUTING.md`](../CONTRIBUTING.md) |
| GitHub branch, merge, automation, and security policy | [`maintainers/github.md`](maintainers/github.md) |
| AI-agent operating constraints | [`../AGENTS.md`](../AGENTS.md) |

Crate READMEs explain the local purpose of each package and link back here.
Implemented behavior is ultimately defined by source code and tests; design
documents must not claim support that those artifacts do not demonstrate.

## Document types

- **Intent documents** describe goals and constraints. They may discuss future
  behavior but must distinguish it from current support.
- **ADRs** record proposed or accepted consequential decisions and their
  tradeoffs. Proposed decisions are not accepted architecture.
- **Format research** records claims with evidence, classification, confidence,
  and unresolved alternatives.
- **Compatibility documentation** states what has actually been tested and what
  promises users may rely on.

When these conflict, current source and tests govern implemented behavior, an
accepted ADR governs architecture, and evidence records govern format claims.
Update the relevant documents in the same change when behavior or architecture
changes.
