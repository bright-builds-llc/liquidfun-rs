---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "01"
subsystem: differential-testing-foundation
tags: [rust, cargo, protocol, differential-testing, isolation]

requires:
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Cargo-first workspace, sole default consumer crate, reviewed private dependencies, and package-isolation checks
provides:
  - Unpublished engine-neutral protocol package boundary
  - Unpublished differential runner with inward-only dependencies on protocol and liquidfun
  - Cargo-verified isolation from the published liquidfun dependency graph
affects: [02-02, 02-03, 02-04, protocol, differential-runner, packaging]

tech-stack:
  added: [thiserror 2.0.18]
  patterns: [private non-default workspace crates, pure protocol versus effectful runner separation]

key-files:
  created:
    - crates/liquidfun-test-protocol/Cargo.toml
    - crates/liquidfun-test-protocol/src/lib.rs
    - crates/liquidfun-differential/Cargo.toml
    - crates/liquidfun-differential/src/lib.rs
  modified:
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "Separate engine-neutral protocol contracts from the effectful differential runner so parsing and comparison do not depend on orchestration."
  - "Keep both harness crates unpublished and outside default-members while preserving liquidfun as the unchanged sole default consumer package."

patterns-established:
  - "Dependency direction: liquidfun-differential depends inward on liquidfun-test-protocol and liquidfun; neither private crate is reachable from liquidfun."
  - "Private tooling manifests inherit workspace edition, MSRV, lints, and reviewed dependency pins while forbidding unsafe code at crate roots."

requirements-completed:
  - COMP-03
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T06:31:22Z

duration: 5 min
completed: 2026-07-10
---

# Phase 2 Plan 01: Private Protocol and Differential Workspace Foundation Summary

**Two unpublished Rust crates now isolate semantic protocol contracts from differential orchestration while the published `liquidfun` crate remains the sole dependency-free default member.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-07-10T06:26:34Z
- **Completed:** 2026-07-10T06:31:22Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added `liquidfun-test-protocol` as the unpublished, engine-neutral package boundary for later strict scenario and trace contracts.
- Added `liquidfun-differential` as the unpublished runner package with explicit inward path dependencies on the protocol crate and native Rust engine.
- Centralized `thiserror` alongside the existing reviewed private serialization, hash, and metadata dependencies, then regenerated the shared lockfile through Cargo.
- Proved `liquidfun` remains the sole default member with no harness, Serde, C++, feature, or build-script edge.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add isolated private workspace packages** - `7cd8fa0` (`feat`)

## Files Created/Modified

- `Cargo.toml` - Registers both non-default private crates and centralizes `thiserror` without changing `default-members`.
- `Cargo.lock` - Cargo-generated resolution for the two workspace packages and `thiserror`.
- `crates/liquidfun-test-protocol/Cargo.toml` - Unpublished protocol manifest with only reviewed private serialization, hash, metadata, and error dependencies.
- `crates/liquidfun-test-protocol/src/lib.rs` - Truthful crate-level protocol-boundary documentation with unsafe code forbidden.
- `crates/liquidfun-differential/Cargo.toml` - Unpublished runner manifest with inward path dependencies and reviewed private tooling dependencies.
- `crates/liquidfun-differential/src/lib.rs` - Truthful crate-level runner-boundary documentation with unsafe code forbidden.

## Decisions Made

- Kept protocol/domain concerns in a cohesive private crate and process/comparison orchestration in a separate private runner, preserving the functional-core/imperative-shell direction established by phase research.
- Reused workspace dependency pins instead of adding parallel versions or unreviewed libraries; no Tokio, approximate-equality, schema, FFI, RNG, feature, or build dependency was introduced.

## Verification Evidence

- `cargo check -p liquidfun-test-protocol -p liquidfun-differential --all-targets --all-features` passed.
- Plan-scoped Clippy and build checks for both new crates passed with warnings denied.
- `cargo build -p liquidfun`, `cargo test -p liquidfun`, and `cargo xtask package verify` passed.
- The required ordered sequence passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `cargo tree -p liquidfun --edges normal` contains only `liquidfun`; it contains neither private crate, `serde`, nor `serde_json`.
- Static acceptance checks found exactly one unchanged `default-members = ["crates/liquidfun"]` line, `publish = false` in both new manifests, and no build script, feature, CMake, or harness reference in `crates/liquidfun/Cargo.toml`.
- Scoped diff review confirmed no file under `crates/liquidfun` changed, and `git diff --check` passed.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for `02-02-PLAN.md` to add invariant-bearing protocol versions, IDs, exact float bits, limits, provenance, scenarios, and trace types inside the isolated protocol crate.
- No consumer-boundary, dependency-direction, compilation, or packaging blocker remains.

## Self-Check: PASSED

- All six implementation files listed in this summary exist.
- Task commit `7cd8fa0` exists and contains only the scoped workspace, lockfile, and private-crate changes.
- Summary lifecycle metadata matches Plan 02-01, and all three requirement IDs are copied verbatim.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
