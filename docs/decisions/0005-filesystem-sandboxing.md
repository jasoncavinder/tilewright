# ADR 0005: Filesystem sandboxing and path resolution

- **Status:** Accepted
- **Date:** 2026-08-02

## Context

Tilewright reads untrusted RPG Maker MZ project directories. The initial
experimental candidate discovery API uses standard `std::fs` functions. This
approach is vulnerable to Time-of-Check to Time-of-Use (TOCTOU) race conditions
and ancestor symlink escapes. For example, if a symlink inside the project
directory points outside of it, standard path resolution might follow it,
allowing the library to read or eventually write files outside the intended
project boundary.

While the current discovery API attempts to reject a root path that is itself a
symlink, it does not provide race-free sandbox containment for subsequent
operations or nested paths, and ancestor components are still resolved
ambiently. As Tilewright moves toward full project loading and mutation, it
requires a robust mechanism to confine project operations beneath an authorized
directory.

Selecting that directory and confining later operations are separate trust
boundaries. `cap_std::fs::Dir::open_ambient_dir` is explicitly not sandboxed; it
may resolve root or ancestor symlinks using any authority available to the host
process. Capability-relative access cannot prove that an ambiently opened handle
identifies the directory the caller intended to authorize.

## Decision

Tilewright will use capability-based filesystem APIs (specifically the `cap-std`
crate) for project-bound access beneath an already-authorized directory handle
as a staged migration.

Once a project root has been authorized as a capability-based directory handle
(e.g., `cap_std::fs::Dir`), all subsequent project reads, writes, and path
resolutions will be performed relative to that handle.

This decision does not select the initial root-acquisition contract. A future
loading API might require its caller to provide an already-authorized directory
handle, or it might acquire one relative to a trusted parent with an explicit
no-follow and identity-validation policy. That choice requires separate design
and testing before Tilewright can claim that a path names the intended root.

The current experimental `discover_candidate` API uses `std::fs` as a temporary
exception. It does not claim race-free containment or complete root symlink
rejection. It will be migrated to `cap-std` or replaced by a capability-based
loader API once the project-loading boundary is fully designed.

## Rationale

- **Security:** Capability-relative `cap-std` methods reject path resolution
  outside the already-opened directory's subtree, including malicious `..`,
  absolute-path, and escaping-symlink inputs. This guarantee begins only after
  the authorized handle exists.
- **Correctness:** Capability-relative operations avoid the check-then-open
  pattern used by `std::fs`; they do not, by themselves, validate the identity
  of a directory selected through ambient authority.
- **Cross-Platform:** `cap-std` abstracts the complex platform-specific
  sandboxing logic, providing a consistent API across Unix and Windows.
- **Clarity:** It makes the descendant-access boundary explicit in the API.
  Functions that operate within an authorized project will take a `&Dir` rather
  than resolving each descendant from a process-wide path.

## Alternatives considered

- **Keeping `std::fs` and canonicalization:** Canonicalizing paths before access
  is prone to TOCTOU races. A symlink can be changed between the
  canonicalization check and the actual file open.
- **Platform-specific handle APIs:** Writing custom wrappers around `openat`
  (Unix) and `NtCreateFile` (Windows) would require significant unsafe code and
  maintenance overhead. `cap-std` provides a well-tested, cross-platform
  abstraction for this exact purpose.

## Consequences

- **Dependency:** The core depends on `cap-std`; the read-only inventory also
  uses `cap-fs-ext` to open each descendant directory with final-component
  symlink following disabled.
- **Refactoring:** Future loaders must be built around `cap_std::fs::Dir`
  instead of `std::path::Path` and `std::fs`.
- **API Design:** A path-taking API must not claim root identity merely because
  it immediately calls `Dir::open_ambient_dir`. The library must separately
  define how the initial capability is authorized.
- **Root Acquisition:** Whether callers provide an authorized `Dir` or the
  library acquires one relative to a trusted parent remains an open question.
- **Symlink Policy:** The read-only inventory reports every encountered symlink
  and does not follow it, including links whose targets may remain inside the
  capability. Policies for later loading and writes remain separate decisions;
  escaping the supplied capability is structurally prevented.
