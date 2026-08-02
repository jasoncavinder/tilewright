# Security policy

## Supported versions

Tilewright is at the scaffold stage and has no supported release line yet. See
[`docs/compatibility.md`](../docs/compatibility.md) for current capability and
support claims. Security fixes are developed against the current `dev` branch
and promoted to `main` through the normal pull-request process.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability or include sensitive
project data in a report. Use GitHub's
[private vulnerability reporting](https://github.com/jasoncavinder/tilewright/security/advisories/new)
for this repository.

Include only the information needed to understand and reproduce the issue:

- the affected commit or version;
- the security impact and realistic threat model;
- minimal reproduction steps or synthetic data;
- any known mitigations; and
- whether the report includes embargo-sensitive information.

Never upload proprietary RPG Maker application code, bundled assets, commercial
plugins, user projects, credentials, or unrelated private data. If reproduction
appears to require such material, describe the constraint first so that a safe
evidence-sharing approach can be agreed upon.

The maintainer will acknowledge the report and coordinate next steps through the
private advisory. Response and remediation timing depends on severity,
reproducibility, and the project's early maturity; this policy does not promise a
fixed service-level agreement.

## Scope

Reports about repository automation, dependency compromise, parser or path
handling, unsafe project writes, and MCP security boundaries are in scope when
they affect implemented Tilewright behavior. General support questions and
unimplemented roadmap capabilities belong in the public issue tracker.
