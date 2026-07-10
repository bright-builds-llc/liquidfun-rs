---
phase: 01-oracle-provenance-and-repository-foundation
reviewed: 2026-07-10T04:27:53Z
status: issues_found
depth: standard
files_reviewed: 39
files_reviewed_list:
  - .cargo/config.toml
  - .github/workflows/ci.yml
  - .github/workflows/oracle.yml
  - .gitignore
  - .gitmodules
  - ARCHITECTURE.md
  - COMPATIBILITY.md
  - CONTRIBUTING.md
  - Cargo.toml
  - README.md
  - TESTING.md
  - THIRD_PARTY_NOTICES.md
  - UPSTREAM.md
  - crates/liquidfun/Cargo.toml
  - crates/liquidfun/src/lib.rs
  - docs/decisions/0001-oracle-selection.md
  - justfile
  - reference/artifacts/manifest.toml
  - reference/compatibility.json
  - reference/discovery.json
  - reference/source-map.toml
  - reference/upstream-lock.toml
  - rust-toolchain.toml
  - tools/reference/CMakeLists.txt
  - tools/reference/CMakePresets.json
  - tools/xtask/Cargo.toml
  - tools/xtask/src/inventory.rs
  - tools/xtask/src/inventory/discovery.rs
  - tools/xtask/src/inventory/report.rs
  - tools/xtask/src/inventory/validation.rs
  - tools/xtask/src/main.rs
  - tools/xtask/src/package.rs
  - tools/xtask/src/provenance.rs
  - tools/xtask/src/upstream.rs
  - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  - tools/xtask/tests/inventory_cli.rs
  - tools/xtask/tests/package_cli.rs
  - tools/xtask/tests/provenance_cli.rs
  - tools/xtask/tests/upstream_cli.rs
finding_counts:
  critical: 1
  warning: 3
  info: 0
  total: 4
---

# Phase 1 Code Review

## Summary

Phase 1 has a sound Cargo/oracle separation and the checked-in evidence currently
passes its local gates, but four issues remain. The highest-severity issue lets
provenance paths escape the repository through a symlinked ancestor. The other
findings cover a deterministic Windows inventory failure, a publishable package
that omits its MIT license text, and duplicate compatibility mappings that are
silently collapsed during coverage validation.

The review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, and the local architecture, code-shape, verification,
testing, and Rust standards.

## Critical Issues

### CR-01: Lexical path validation permits provenance records outside the repository

**File:** `tools/xtask/src/provenance.rs:367-380`

**Issue:** `validate_relative_path` only rejects absolute and `..` components
(`tools/xtask/src/provenance.rs:540-559`). The subsequent
`repository_root.join(...)` operations follow symlinks in ancestor components.
For artifact paths, `symlink_metadata` checks only the final resolved entry, so a
normal-looking path such as `reference/artifacts/link/file.bin`, where `link` is
a symlink outside the checkout, is accepted as a regular file and hashed. The
same ancestor-symlink escape affects source-map paths at lines 255-260 and notice
references at lines 339-351. A reviewed artifact or provenance mapping can
therefore attest to a file that is not in the repository, violating the stated
confinement and supply-chain boundary.

**Fix:** Canonicalize the repository root and every existing candidate, require
the canonical candidate to remain beneath the canonical root, and reject
symlinks in every traversed component (not only the final component). Apply one
shared confined-path helper to source mappings, artifacts, and notice references.
Add a regression fixture whose intermediate directory is a symlink outside the
fixture root and assert `provenance/path` failure before hashing.

## Warnings

### WR-01: Windows discovery emits backslashes and cannot match the canonical snapshot

**File:** `tools/xtask/src/inventory/discovery.rs:217-237`

**Issue:** `path_text` converts filesystem-derived `Path` values directly with
`to_str()`. On Windows, paths returned by `read_dir` serialize with `\`
separators. The checked-in discovery ledger uses `/`, while
`validate_relative_path` explicitly rejects backslashes
(`tools/xtask/src/inventory.rs:487-503`). Consequently, the Windows portability
job's `cargo xtask inventory check` invocation
(`.github/workflows/oracle.yml:183-185`) will compare a backslash-based scan with
the forward-slash snapshot and report stale bytes. `inventory discover` on
Windows would also write a snapshot that its own next validation rejects.

**Fix:** Build repository-relative ledger paths from `Component::Normal` values
joined with `/` instead of displaying the platform `Path`. Add a Windows test or
a platform-independent unit test for the normalization helper, then exercise
the inventory checker in Windows CI before treating that portability lane as
usable.

### WR-02: The publishable crate archive omits the MIT license text

**File:** `crates/liquidfun/Cargo.toml:10`

**Issue:** The explicit `include` allowlist packages only the manifest, Rust
sources, and README. `cargo package -p liquidfun --list --allow-dirty` confirms a
six-entry archive with no `LICENSE`. This leaves the packaged README's
`./LICENSE` link broken (`README.md:7-8`) and distributes the crate without the
license notice referenced by its `license = "MIT"` metadata and documentation.
The package verifier checks forbidden content but never requires the license,
so CI currently reports the incomplete archive as verified.

**Fix:** Arrange for the root MIT license to be copied into the crate package
(for example through supported `license-file` metadata or a crate-local license
file included explicitly), and make `package verify` require that exact file.
Add a positive package-content test and rerun `cargo package --list` to confirm
the license is present without admitting repository-only material.

### WR-03: Distinct IDs can duplicate one upstream mapping without failing coverage

**File:** `tools/xtask/src/inventory/validation.rs:227-253`

**Issue:** Compatibility validation enforces unique stable IDs at lines 30-56,
but it does not enforce uniqueness of `(kind, upstream_path, upstream_symbol)`.
`coverage` immediately collects mappings into a `BTreeSet`, which silently
collapses duplicate tuples. Two rows with different IDs can therefore claim the
same discovered surface, inflate inventory/evidence counts, and still pass both
set-difference checks.

**Fix:** Reject duplicate mapping tuples while validating compatibility entries,
with an error that names both conflicting IDs. Keep the set comparison for
coverage only after one-to-one mapping uniqueness is established, and add a
fixture with two distinct IDs targeting the same upstream path/symbol.

## Verification Evidence

- `cargo fmt --all --check` — passed
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed
- `cargo test --workspace --all-features` — passed (25 tests)
- `actionlint .github/workflows/ci.yml .github/workflows/oracle.yml` — passed
- `cargo xtask inventory check` — passed with 177 rows on macOS
- `cargo xtask upstream verify` — passed; local noncanonical CMake/compiler were correctly labeled
- `cargo xtask provenance check` — passed with zero artifact records
- `cargo xtask package verify` — passed, but its six-entry archive demonstrates WR-02

## Clean Evidence

- External GitHub Actions are pinned by full commit SHA, and downloaded CMake,
  Ninja, and LLVM installer inputs have explicit SHA-256 checks.
- Process construction uses structured `Command` arguments and preset
  allowlists; no shell evaluation or interpolated command string was found.
- Default Cargo membership isolates `liquidfun`, while `xtask` is unpublished
  and absent from the consumer dependency graph.
- Archive inspection rejects absolute paths, parent traversal, links, native
  source extensions, CMake files, and `build.rs` before extraction.
- Current oracle revision, gitlink, checkout, origin URL, clean worktree, source
  mappings, discovery ledger, generated report, and documentation agree.

_Reviewer: gsd-code-reviewer_
_Depth: standard_
