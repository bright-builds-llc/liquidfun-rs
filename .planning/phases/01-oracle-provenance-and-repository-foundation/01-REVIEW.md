---
phase: 01-oracle-provenance-and-repository-foundation
reviewed: 2026-07-10T04:40:04Z
status: clean
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
  critical: 0
  warning: 0
  info: 0
  total: 0
---

# Phase 1 Code Review

## Summary

The original 39-file Phase 1 scope was re-reviewed at standard depth after fix
commits `c6e9f35`, `85e96a2`, `bd8ed95`, and `09047f2`. All four prior findings
are resolved, their regression coverage passes, and no new correctness,
security, supply-chain, packaging, CI, or cross-platform issue was found in the
fixes.

The review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, and the local architecture, code-shape, verification,
testing, and Rust standards.

## Prior Finding Resolution

- **CR-01 resolved:** `ConfinedPaths` validates every component with
  `symlink_metadata`, rejects symlinks, canonicalizes the final file, and requires
  it to remain beneath the canonical repository root. Source mappings, artifact
  files, and notice references all use the shared confinement path. The new
  intermediate-symlink escape test passes.
- **WR-01 resolved:** inventory discovery now serializes normal path components
  with `/` separators independently of the host platform. The normalization test
  passes and the committed discovery bytes remain current.
- **WR-02 resolved:** `liquidfun` now packages a crate-local `LICENSE`, and
  package verification requires the entry and compares its bytes with the root
  MIT license before building. The real archive contains seven entries including
  `LICENSE`.
- **WR-03 resolved:** compatibility validation rejects duplicate
  `(kind, upstream_path, upstream_symbol)` mappings before coverage set
  comparison and names both conflicting stable IDs. The focused duplicate
  mapping test passes.

## Verification Evidence

- `cargo fmt --all --check` — passed
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed
- `cargo build --workspace --all-targets --all-features` — passed
- `cargo test --workspace --all-features` — passed (29 tests)
- `cargo test -p xtask --test provenance_cli --test inventory_cli --test package_cli` — passed (17 focused tests)
- `cargo package -p liquidfun --list --allow-dirty` — passed; `LICENSE` is present
- `cargo xtask inventory check` — passed with 177 rows
- `cargo xtask provenance check` — passed with zero artifact records
- `cargo xtask package verify` — passed with a seven-entry isolated package
- `actionlint .github/workflows/ci.yml .github/workflows/oracle.yml` — passed

## Clean Evidence

- Provenance paths are lexically validated, symlink-safe, canonically confined,
  and required to be regular files.
- Inventory path output is deterministic across platform separators, and
  compatibility mappings are one-to-one at the declared discovery granularity.
- Package isolation still rejects traversal, links, CMake/build-script/native
  leakage, and now also requires the exact project license.
- Structured command arguments, allowlisted presets, full-SHA action pins, and
  checksummed tool downloads remain intact.
- Default Cargo membership and package contents continue to isolate the native
  Rust consumer crate from private xtask and C++ oracle tooling.

_Reviewer: gsd-code-reviewer_
_Depth: standard_
