---
phase: 11-examples-headless-tooling-and-testbed
plan: "17"
subsystem: catalog-benchmarks
tags: [criterion, benchmark, catalog, semantic-checkpoint, cargo-isolation]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "06"
    provides: Closed typed scenario registry and benchmark eligibility mappings
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "14"
    provides: Native resolved-scenario execution and canonical semantic checkpoints
provides:
  - Private non-default Criterion 0.8.2 package over the typed catalog and native backend
  - Seven fixed representative benchmark cases with exact hashes, settings, warmups, and logical horizons
  - Manual measured-region boundary that excludes resolution, restart, checkpoint capture, and validation
affects: [phase11-evidence, phase12-performance, benchmark-smoke, package-isolation]
tech-stack:
  added: [criterion 0.8.2]
  patterns:
    - Resolve and semantically validate benchmark authority before Criterion registration
    - Return only accumulated declared logical-tick duration from iter_custom
key-files:
  created:
    - crates/liquidfun-benchmarks/src/lib.rs
    - crates/liquidfun-benchmarks/benches/catalog.rs
    - crates/liquidfun-benchmarks/tests/catalog_equivalence.rs
    - crates/liquidfun-benchmarks/Cargo.toml
  modified:
    - Cargo.toml
    - Cargo.lock
key-decisions:
  - "Confine Criterion exactly 0.8.2 to the dev-dependencies of one unpublished package outside default-members."
  - "Use a manual timer inside a sealed case method so only the declared logical actions contribute to Criterion's returned duration."
  - "Reject fixed-identity or semantic checkpoint drift before registration or before a duration can become an accepted sample."
patterns-established:
  - "Catalog benchmark case: fixed slug/version/hash/settings/warmup/horizon plus one restart factory and exact semantic validator."
  - "Measured region: fresh session before Instant::now, exact StepOnce horizon under the timer, checkpoint capture and validation after elapsed time."
requirements-completed: [EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T04:50:50Z
duration: 18 min
completed: 2026-07-21
---

# Phase 11 Plan 17: Canonical Catalog Benchmarks Summary

**A private Criterion bridge now benchmarks seven fixed canonical catalog runs while excluding resolution, restart, and semantic validation from measured logical-tick time.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-22T04:32:30Z
- **Completed:** 2026-07-22T04:50:50Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added `liquidfun-benchmarks` as an unpublished workspace member outside the unchanged `default-members = ["crates/liquidfun"]` boundary and pinned Criterion exactly to 0.8.2 only in that package's dev-dependencies.
- Added fixed rigid, joint, particle, group, particle/body mixed, AABB query, and ray callback cases carrying exact scenario version, resolved SHA-256, solver settings, warmup count, and measured logical horizon.
- Prepared every case through the reviewed catalog mapping, exact native restart factory, repeated semantic checkpoint validation, and fixed identity checks before Criterion registration.
- Used `iter_custom` with a sealed manual-timing method: fresh session construction occurs before `Instant::now`, only exact `StepOnce` commands execute under the timer, and canonical checkpoint capture and validation occur after elapsed time is accumulated.
- Added behavior and source-boundary tests proving representative equivalence, semantic mismatch rejection, exact horizons, measured-region delegation, and Cargo/package isolation.

## TDD Evidence

- **RED:** `cargo test -p liquidfun-benchmarks --test catalog_equivalence` failed at compile time because `BenchmarkCaseErrorKind` and `representative_catalog_benchmarks` did not exist.
- **GREEN:** The focused equivalence suite passes 5/5, focused deny-warnings Clippy passes, and `cargo bench -p liquidfun-benchmarks --bench catalog --no-run` compiles the optimized Criterion target.
- **REFACTOR:** Fixed hashes, settings, and horizons replaced temporary discovery output; preparation, restart, logical execution, capture, and validation remain separate cohesive functions.

The intentionally failing RED state was not committed because repository policy requires every commit to follow a completely passing ordered Rust gate.

## Task Commits

1. **Task 1: Create the private benchmark bridge with measured-region discipline** - `84b913f` (feat)
2. **Task 1 refinement: Keep benchmark smoke threshold-free** - `eeecc4e` (test)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-benchmarks/Cargo.toml` - Unpublished package manifest with Criterion confined to an exact private dev-dependency.
- `crates/liquidfun-benchmarks/src/lib.rs` - Fixed case preparation, native restart, semantic validation, and manual logical-tick timing boundary.
- `crates/liquidfun-benchmarks/benches/catalog.rs` - Criterion `iter_custom` bridge over prevalidated cases.
- `crates/liquidfun-benchmarks/tests/catalog_equivalence.rs` - Identity, mismatch, horizon, measured-region, and dependency-isolation coverage.
- `Cargo.toml` - Adds the benchmark package as a workspace member without changing the sole default member.
- `Cargo.lock` - Locks Criterion 0.8.2 and its private benchmark dependency graph.

## Decisions Made

- Used each definition's reviewed default settings only after checking them against fixed `1/60` timestep, `8/3` rigid iterations, and explicit per-family particle iterations.
- Fixed the measured horizon to the complete declared checkpoint schedule for each smoke case rather than inventing broad workload budgets.
- Derived the expected checkpoint from one untimed native run and required a second complete untimed warmup run to match exactly before returning the case.
- Kept sample timing diagnostic-only: no thresholds, platform authority, optimization claim, oracle-performance comparison, or Phase 12 sign-off was added.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo xtask package verify` still reaches the documented pre-existing Phase 10 packaged-test isolation failure because several `liquidfun` unit tests use repository-relative `include_str!` artifact paths absent from the unpacked package. This plan did not modify those tests. The narrower package boundary checks pass: `cargo tree -p liquidfun --edges normal` contains only `bitflags`, and `cargo package -p liquidfun --list --allow-dirty` contains no benchmark, Criterion, differential, or protocol paths.

## Security Verification

- Every case resolves through the reviewed typed catalog and requires benchmark eligibility before native effects.
- Fixed slug, version, SHA-256, settings, horizon, and final semantic checkpoint disagreements fail with bounded categories and never echo raw records or private engine state.
- Fresh native session construction and canonical checkpoint capture remain outside the measured duration, while a post-run mismatch returns an error before Criterion receives the sample.
- Criterion, differential, and protocol dependencies remain outside the publishable `liquidfun` normal dependency graph and package archive.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-17's `EXMP-03` mapping is implemented in the canonical benchmark consumer and retained in summary frontmatter. Its global requirement checkbox remains unchanged until the remaining Phase 11 consumers prove the complete shared-scenario scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 12 can add controlled-hardware methodology and performance budgets over these fixed cases without changing scenario authority or measured-region semantics.
- Remaining Phase 11 evidence and testbed plans can reuse the same resolved hashes and semantic checkpoints.
- No blocker remains for the next incomplete Phase 11 plan.

## Self-Check: PASSED

- Confirmed all four created benchmark-package files exist and implementation commits `84b913f` and `eeecc4e` are present.
- Confirmed the focused equivalence tests pass 5/5, focused deny-warnings Clippy passes, and the Criterion benchmark target compiles with `--no-run`.
- Confirmed the exact ordered `cargo fmt --all`, full-workspace deny-warnings Clippy, all-targets build, and all-features test gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-17`.
- Confirmed `cargo tree -p liquidfun --edges normal` contains no Criterion or private harness package and the publishable package listing contains no private benchmark paths.
- Confirmed the four fenced pre-existing edits remain unstaged and uncommitted.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
