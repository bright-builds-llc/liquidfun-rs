---
phase: 01-oracle-provenance-and-repository-foundation
fixed_at: 2026-07-10T04:36:35Z
review_path: .planning/phases/01-oracle-provenance-and-repository-foundation/01-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 1: Code Review Fix Report

**Fixed at:** 2026-07-10T04:36:35Z  
**Source review:** `.planning/phases/01-oracle-provenance-and-repository-foundation/01-REVIEW.md`  
**Iteration:** 1

## Summary

- Findings in scope: 4
- Fixed: 4
- Skipped: 0

The fixes followed the repository's Rust, architecture, code-shape, testing,
and verification standards. Each finding has a focused regression and an
atomic `fix(01): ...` commit. The orchestrator-owned changes in
`.planning/config.json`, `.planning/STATE.md`, and `.planning/ROADMAP.md` were
preserved and excluded from every fix commit.

## Fixed Issues

### CR-01: Lexical path validation permits provenance records outside the repository

**Files modified:** `tools/xtask/src/provenance.rs`,
`tools/xtask/tests/provenance_cli.rs`  
**Commit:** `c6e9f35`  
**Applied fix:** Added one confined-path abstraction that canonicalizes the
repository and candidate, rejects every traversed symlink component, requires
the resolved file to remain beneath the canonical repository root, and is used
for source mappings, artifacts, and notice references. Artifact hashing now
receives the already-confined canonical file path.  
**Regression evidence:**
`cargo test -p xtask --test provenance_cli` passed 6 tests, including
`check_rejects_intermediate_symlink_escape_before_hashing`, which receives a
`provenance/path` failure for an ancestor symlink targeting an external file.

### WR-01: Windows discovery emits backslashes and cannot match the canonical snapshot

**Files modified:** `tools/xtask/src/inventory/discovery.rs`  
**Commit:** `85e96a2`  
**Applied fix:** Replaced platform path display with explicit
`Component::Normal` parsing and `/` joining, while rejecting empty,
non-normalized, or non-UTF-8 paths.  
**Regression evidence:** The cross-platform helper test
`path_text_uses_forward_slashes_between_components` passed, the inventory CLI
suite passed, and `cargo xtask inventory check` retained the canonical 177-row
snapshot. The existing Windows portability lane runs the same inventory check.

### WR-02: The publishable crate archive omits the MIT license text

**Files modified:** `crates/liquidfun/Cargo.toml`,
`crates/liquidfun/LICENSE`, `tools/xtask/src/package.rs`,
`tools/xtask/tests/package_cli.rs`  
**Commit:** `bd8ed95`  
**Applied fix:** Added a byte-identical crate-local MIT license to the package
allowlist. Package verification now requires `LICENSE` in the archive and
compares the unpacked bytes with the repository license before building.  
**Regression evidence:** `cmp LICENSE crates/liquidfun/LICENSE` passed;
`verify_accepts_archive_with_matching_license` passed; `cargo package -p
liquidfun --list --allow-dirty` listed `LICENSE`; and `cargo xtask package
verify` passed with 7 isolated archive entries under Rust 1.92.0.

### WR-03: Distinct IDs can duplicate one upstream mapping without failing coverage

**Files modified:** `tools/xtask/src/inventory/validation.rs`,
`tools/xtask/tests/inventory_cli.rs`  
**Commit:** `09047f2`  
**Applied fix:** Compatibility validation now indexes each coverage-relevant
`(kind, upstream_path, upstream_symbol)` tuple before set-based coverage
comparison and rejects a duplicate with both conflicting stable IDs in the
diagnostic. Aggregate subsystem rows remain outside one-to-one discovery
coverage by design.  
**Regression evidence:**
`check_rejects_distinct_ids_for_the_same_upstream_mapping` passed and asserted
the `inventory/duplicate-mapping` category plus both IDs. The full inventory
CLI suite and the canonical 177-row inventory check also passed.

## Full Verification

- `cargo fmt --all --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo build --workspace --all-targets --all-features` — passed.
- `cargo test --workspace --all-features` — passed: 29 tests plus doctests.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` — passed.
- `cargo +1.92.0 check -p liquidfun --all-targets --all-features` — passed.
- `cargo xtask package verify` — passed with 7 archive entries.
- `cargo xtask inventory check` — passed with 177 compatibility rows.
- `cargo xtask upstream verify` — passed at revision
  `7f20402173fd143a3988c921bc384459c6a858f2`; local CMake 3.27.9 and Apple
  Clang 21.0.0 were correctly labeled noncanonical.
- `cargo xtask provenance check` — passed with 0 artifact records.
- `cargo xtask check` — all applicable repository checks passed.
- `actionlint .github/workflows/ci.yml .github/workflows/oracle.yml` — passed.
- `git diff --check` — passed.

***

_Fixed: 2026-07-10T04:36:35Z_  
_Fixer: the agent (gsd-code-fixer)_  
_Iteration: 1_
