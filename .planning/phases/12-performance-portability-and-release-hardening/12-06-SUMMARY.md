---
phase: 12-performance-portability-and-release-hardening
plan: "06"
subsystem: performance-benchmarking
tags: [criterion, paired-benchmarks, semantic-validation, sealed-matrix, deterministic-timing]
requires:
  - phase: 12-04
    provides: reviewed fourteen-workload performance matrix and statistical policy
  - phase: 12-05
    provides: storage-neutral diagnostic profile schema and non-authoritative durations
provides:
  - complete Rust Criterion diagnostics over all reviewed performance matrix cases
  - sealed native benchmark executor that times only declared logical actions
  - exact resolved-byte and authoritative semantic validation before sample acceptance
  - alternating paired-engine caller order with bounded warmups, samples, and actions
affects: [phase-12-performance-runner, paired-oracle-benchmarks, benchmark-reporting, release-evidence]
tech-stack:
  added: []
  patterns: [sealed preconstructed benchmark cases, narrow measured regions, authoritative semantic checkpoint lanes]
key-files:
  created:
    - crates/liquidfun-benchmarks/src/paired.rs
    - crates/liquidfun-benchmarks/tests/performance_matrix.rs
    - crates/liquidfun-differential/src/performance.rs
    - crates/liquidfun-differential/src/performance/native.rs
  modified:
    - crates/liquidfun-benchmarks/benches/catalog.rs
    - crates/liquidfun-benchmarks/src/lib.rs
    - crates/liquidfun-test-protocol/src/performance/matrix.rs
    - protocol/benchmarks/phase12-v1.json
key-decisions:
  - "Keep Criterion Rust-only and diagnostic while the paired caller contract alternates native/oracle order independently."
  - "Accept benchmark samples only when authoritative non-visual checkpoint lanes match; renderer debug primitive order is diagnostic and non-authoritative."
  - "Resolve and validate exact catalog bytes before timing, then place only the declared logical action loop inside the clock interval."
patterns-established:
  - "Measured-region discipline: restart before clock start and capture, validation, and teardown after clock stop."
  - "Benchmark semantic authority: physics observations and occurrences decide acceptance while renderer-only primitives do not."
requirements-completed: [PERF-01, PERF-02, PERF-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T20:30:26Z
duration: 20m
completed: 2026-07-23
---

# Phase 12 Plan 06: Complete Paired Native Performance Benchmarks Summary

**The complete reviewed performance matrix now drives Rust-only Criterion diagnostics and a sealed native executor that times only declared actions and rejects identity or authoritative semantic drift.**

## Performance

- **Duration:** 20m
- **Started:** 2026-07-23T20:10:44Z
- **Completed:** 2026-07-23T20:30:26Z
- **Tasks:** 1
- **Files modified:** 12

## Accomplishments

- Replaced the seven-case benchmark constant with all 32 reviewed matrix rows covering exactly 14 workload kinds and every required size point.
- Added a reusable measured-region executor whose injected-clock tests prove restart, capture, validation, and teardown remain outside the timed action loop.
- Added exact resolved-byte/hash preparation, semantic prevalidation, and bounded warmup/sample/action enforcement before durations can be accepted.
- Kept Criterion explicitly diagnostic and Rust-only while exposing deterministic alternating native/oracle caller order for the later paired runner.
- Preserved the publishable `liquidfun` dependency boundary; its normal dependency tree remains limited to `bitflags`.

## TDD Evidence

- **RED:** The new matrix, measured-region, bound, and semantic-authority tests initially failed because the paired benchmark and differential performance APIs did not exist.
- **GREEN:** Typed matrix preparation, the native executor, Criterion bridge, and exact checkpoint validation made all 13 benchmark tests pass.
- **REFACTOR:** Consolidated benchmark preparation by resolved hash and simplified untimed semantic preparation without widening the measured region.
- The plan prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Execute the complete matrix with sealed timing boundaries** - `ec87a00` (feat)

## Files Created/Modified

- `crates/liquidfun-benchmarks/src/paired.rs` - Loads every reviewed matrix case, seals identity and bounds, alternates paired order, and exposes native diagnostic measurements.
- `crates/liquidfun-benchmarks/tests/performance_matrix.rs` - Covers all workloads and size points, clock boundaries, limits, determinism, and semantic authority.
- `crates/liquidfun-differential/src/performance/native.rs` - Implements injected-clock action timing and prepared native catalog execution.
- `crates/liquidfun-benchmarks/benches/catalog.rs` - Registers the complete matrix under an explicitly diagnostic Criterion group.
- `crates/liquidfun-test-protocol/src/performance/matrix.rs` - Corrects seven sealed scenario hashes to the canonical catalog byte identities.
- `protocol/benchmarks/phase12-v1.json` - Regenerates tracked matrix bytes from the typed performance contract.
- `Cargo.lock` and `crates/liquidfun-benchmarks/Cargo.toml` - Add test-only JSON mutation support without changing production dependencies.

## Decisions Made

- Criterion remains a Rust-only diagnostic consumer; it does not claim cross-engine comparison authority.
- The paired caller contract alternates engine order by sample index, while this plan implements only the sealed native half needed by the later paired runner.
- Exact bytes and hashes are mandatory identity checks. Benchmark semantic acceptance compares authoritative checkpoint metadata, observations, numerics, ordered occurrences, and unordered sets.
- Debug primitives and profile names remain diagnostic-only because their renderer ordering and instrumentation labels do not determine physics equivalence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired seven incorrect sealed performance-matrix hashes**

- **Found during:** Task 1 (Execute the complete matrix with sealed timing boundaries)
- **Issue:** Seven Plan 12-04 matrix bindings used hashes that did not identify their existing canonical Phase 11 catalog bytes, so exact case preparation failed closed.
- **Fix:** Replaced only those seven hashes with the reviewed catalog identities and regenerated the tracked JSON through the typed matrix renderer.
- **Files modified:** `crates/liquidfun-test-protocol/src/performance/matrix.rs`, `protocol/benchmarks/phase12-v1.json`
- **Verification:** `cargo test -p liquidfun-test-protocol --test performance_contract` passed all 10 tests, including exact tracked-byte equality.
- **Committed in:** `ec87a00`

**2. [Rule 3 - Blocking] Scoped semantic acceptance to authoritative non-visual lanes**

- **Found during:** Task 1 (Execute the complete matrix with sealed timing boundaries)
- **Issue:** Concurrent renderer diagnostics can vary debug-primitive ordering and ordinals between equivalent native runs even when every authoritative physics lane is identical.
- **Fix:** Added an explicit benchmark semantic comparator that ignores renderer-only debug primitives and diagnostic profile names while rejecting any authoritative observation or occurrence drift.
- **Files modified:** `crates/liquidfun-differential/src/performance/native.rs`, `crates/liquidfun-benchmarks/tests/performance_matrix.rs`
- **Verification:** Regression tests accept visual-only ordering drift, reject authoritative observation drift, and reproduce the same prepared physics semantics across two runs.
- **Committed in:** `ec87a00`

**Total deviations:** 2 auto-fixed (2 blocking)

**Impact on plan:** Both corrections were necessary to execute the sealed matrix against canonical inputs without granting renderer diagnostics physics authority. No consumer dependencies or production physics behavior changed.

## Verification

- `cargo test -p liquidfun-benchmarks --all-features` - 13 passed.
- `cargo bench -p liquidfun-benchmarks --bench catalog --no-run` - benchmark executable compiled.
- `cargo clippy -p liquidfun-benchmarks -p liquidfun-differential --all-targets --all-features -- -D warnings` - passed.
- `cargo test -p liquidfun-test-protocol --test performance_contract` - 10 passed.
- `cargo tree -p liquidfun --edges normal` - only `bitflags` remains in the production dependency tree.
- Repository pre-commit gate passed in required order: format, warning-denied Clippy, all-target build, and all-feature tests.
- `git diff --check` and staged-diff review found no whitespace errors or unintended task files.

## Known Stubs

None.

## Issues Encountered

- Concurrent Cargo work briefly held the shared build lock; verification was rerun against the isolated Phase 12 target directory after the lock cleared.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The complete native benchmark half is ready for the fixed-sample oracle runner and paired statistical report plans.
- Exact case identity, sample bounds, engine-order alternation, and semantic acceptance rules are exposed as reusable contracts.
- No blockers remain.

## Self-Check: PASSED

- All four created implementation/test files and this summary exist.
- Task commit `ec87a00` exists in repository history.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
