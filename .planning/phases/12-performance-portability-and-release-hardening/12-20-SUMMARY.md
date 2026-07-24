---
phase: 12-performance-portability-and-release-hardening
plan: "20"
subsystem: performance-benchmarking
tags: [rust, cpp, jsonl, supervisor, interleaving, raw-evidence]
requires:
  - phase: 12-19
    provides: strict long-lived C++ benchmark endpoint with reset epochs and semantic checkpoint identity
  - phase: 12-07
    provides: bounded benchmark wire records and disjoint typed outcomes
  - phase: 12-06
    provides: sealed native measured-region execution
provides:
  - exact Rust/C++ paired sample interleaving across five independent baseline runs
  - immutable raw reports retaining durations, engine order, process generations, and reset epochs
  - concrete bounded native and supervised C++ benchmark adapters
  - fail-closed disjoint harness, physics-mismatch, and performance outcomes
affects: [performance-analysis, release-evidence, compatibility-reporting]
tech-stack:
  added: []
  patterns:
    - request-aware native checkpoint identity outside the authoritative timer
    - bounded supervisor operation returning only validated typed benchmark results
    - process-generation and reset-epoch pairs as raw lifecycle identity
key-files:
  created:
    - crates/liquidfun-differential/src/performance/oracle.rs
    - crates/liquidfun-differential/src/performance/report.rs
    - crates/liquidfun-differential/src/performance/report/runner.rs
    - crates/liquidfun-differential/src/supervisor/catalog/benchmark.rs
    - crates/liquidfun-differential/tests/paired_performance.rs
  modified:
    - crates/liquidfun-differential/src/performance.rs
    - crates/liquidfun-differential/src/performance/native.rs
    - crates/liquidfun-differential/src/supervisor/catalog.rs
    - crates/liquidfun-differential/tests/fixtures/fake_oracle.rs
key-decisions:
  - "Keep raw process generation and reset epoch together so reviewed child cycling cannot be mistaken for reset rollback."
  - "Hash canonical checkpoint JSON without its JSONL newline on the native side to match the pinned C++ checkpoint-record identity."
  - "Keep unprofiled totals as the sole authority while common-parent and Rust-child durations remain diagnostic-only fields."
patterns-established:
  - "Paired ordering: odd sample ordinals run Rust then C++; even ordinals run C++ then Rust, yielding Rust,C++,C++,Rust across adjacent pairs."
  - "Failure boundary: a harness failure or semantic mismatch terminates at the first stable location and can never contribute a duration."
requirements-completed: [PERF-02, PERF-03, PERF-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T01:04:39Z
duration: 28m
completed: 2026-07-24
---

# Phase 12 Plan 20: Paired Performance Reporting Summary

**Exact same-host Rust/C++ interleaving now produces immutable raw timing evidence with request-bound semantic checkpoints, lifecycle identity, diagnostic-only profiles, and disjoint terminal outcomes.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-07-24T00:36:43Z
- **Completed:** 2026-07-24T01:04:39Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Executed five independent baseline runs with 30 pairs each and exact `Rust,C++,C++,Rust` interleaving across adjacent sample ordinals.
- Preserved every authoritative raw nanosecond value together with baseline run, sample ordinal, engine order, process generation, reset epoch, report identity, policy, compatibility status, and semantic checkpoint identity.
- Added concrete native and pinned-oracle adapters without exposing child handles, pipes, or engine-private state outside their reviewed boundaries.
- Kept common parent profiles and optional Rust-only children explicitly diagnostic while unprofiled wall-clock totals remain the only performance authority.
- Proved stable first-failure handling for timeout, crash, malformed schema, oversized output, provenance mismatch, sanitizer output, reset drift, and physics mismatch.

## TDD Evidence

- **RED:** The focused paired test initially failed because the reviewed adapter/report surface did not exist. The failing state was not committed.
- **GREEN:** Ten focused integration tests now cover exact interleaving, raw retention, concrete native identity, supervised C++ recovery and bounds, profiles, reset lifecycle, and disjoint outcomes.
- **REFACTOR:** Split immutable report models from paired execution assembly at the repository file-length trigger and reused existing native measurement and supervisor lifecycle logic.

## Task Commits

Each task was committed atomically:

1. **Task 1: Interleave paired samples and build immutable raw reports** - `221869e` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/performance.rs` - Declares and re-exports the reviewed paired performance surface.
- `crates/liquidfun-differential/src/performance/native.rs` - Adds request-aware native measurement and canonical checkpoint identity outside the timer.
- `crates/liquidfun-differential/src/performance/oracle.rs` - Defines the adapter boundary plus concrete native Rust and supervised C++ adapters.
- `crates/liquidfun-differential/src/performance/report.rs` - Defines immutable plans, raw samples, reports, diagnostics, and disjoint outcomes.
- `crates/liquidfun-differential/src/performance/report/runner.rs` - Executes exact interleaving and fail-closed report assembly.
- `crates/liquidfun-differential/src/supervisor/catalog.rs` - Routes benchmark requests through the existing child lifecycle.
- `crates/liquidfun-differential/src/supervisor/catalog/benchmark.rs` - Owns bounded benchmark write/read/drain/reap behavior and strict identity validation.
- `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs` - Supplies typed benchmark success and adversarial child behaviors.
- `crates/liquidfun-differential/tests/paired_performance.rs` - Proves paired order, identity, reset, recovery, diagnostic, and terminal-outcome contracts.

## Decisions Made

- Stored `(process_generation, reset_epoch)` rather than treating reset epochs as globally monotonic, because the reviewed reuse supervisor intentionally cycles after 100 requests while a complete session collects 150 samples per engine.
- Reused the native measured-action helper through a checkpoint-returning internal seam so setup, warmup, capture, validation, hashing, and teardown remain outside the authoritative timer.
- Kept the synchronous supervisor responsible for all child I/O, bounded drains, sanitizer inspection, shutdown, and reaping; adapters receive only a validated typed result or a closed failure category.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added a bounded benchmark operation to the existing supervisor**

- **Found during:** Task 1 (concrete C++ adapter)
- **Issue:** The plan required concrete C++ execution but the existing supervisor exposed only catalog runs; bypassing it would duplicate process safety or expose raw process state.
- **Fix:** Added one crate-private typed operation that owns request framing, bounded output, deadlines, sanitizer inspection, identity validation, shutdown, and reaping.
- **Files modified:** `crates/liquidfun-differential/src/supervisor/catalog.rs`, `crates/liquidfun-differential/src/supervisor/catalog/benchmark.rs`
- **Verification:** Focused poison/recovery, oversized-output, sanitizer, crash, and provenance tests pass; full Rust gate passes.
- **Committed in:** `221869e`

**2. [Rule 2 - Missing Critical] Added request-aware native semantic identity**

- **Found during:** Task 1 (concrete native adapter)
- **Issue:** The existing native measurement returned only a duration and captured a default request ID, so it could not prove identity equivalence with the paired C++ request.
- **Fix:** Added a request-aware measured sample returning only authoritative duration and canonical semantic checkpoint identity while preserving the existing public method and timing boundary.
- **Files modified:** `crates/liquidfun-differential/src/performance/native.rs`, `crates/liquidfun-differential/src/performance/oracle.rs`
- **Verification:** Direct native measurement and concrete adapter produce identical request-bound checkpoint identities in the focused suite.
- **Committed in:** `221869e`

**3. [Rule 2 - Missing Critical] Extended the fake oracle with bounded benchmark behaviors**

- **Found during:** Task 1 (supervisor integration proof)
- **Issue:** Existing fixtures could not exercise benchmark-specific typed results, mid-session poisoning, output bounds, or sanitizer classification.
- **Fix:** Added strict benchmark request decoding plus success, malformed, second-malformed, oversized, sanitizer, and crash behaviors.
- **Files modified:** `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs`
- **Verification:** All concrete supervised-oracle integration cases pass.
- **Committed in:** `221869e`

**4. [Rule 1 - Bug] Accepted reviewed process cycling without losing reset identity**

- **Found during:** Task 1 (complete 150-sample session)
- **Issue:** A global reset-epoch expectation would reject the supervisor's reviewed 100-request child cycle even though the new child correctly restarts at epoch one.
- **Fix:** Persisted and validated the raw `(process_generation, reset_epoch)` pair, requiring monotonic epochs within a generation and exact generation-plus-one/epoch-one transitions.
- **Files modified:** `crates/liquidfun-differential/src/performance/report.rs`, `crates/liquidfun-differential/src/performance/report/runner.rs`, `crates/liquidfun-differential/tests/paired_performance.rs`
- **Verification:** The focused suite completes 150 samples per engine and proves the exact transition at sample 101.
- **Committed in:** `221869e`

**Total deviations:** 4 auto-fixed (1 bug, 3 missing critical)
**Impact on plan:** All deviations were narrow correctness and verification seams required to execute the specified concrete adapters safely; no public production-engine dependency or foreign runtime was added.

## Issues Encountered

- The immutable report model initially crossed the repository's refactor trigger. It was split into a 526-line model and a 343-line execution child without changing the reviewed public surface.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p liquidfun-differential --test paired_performance` - 10 passed.
- `cargo fmt --all` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `cargo build --all-targets --all-features` - passed.
- `cargo test --all-features` - passed.

## Next Phase Readiness

- The immutable raw session report is ready for Plan 12-08/next statistical aggregation and release-evidence consumers.
- No authentication gates, deferred implementation stubs, or new unmodeled threat surfaces remain.

## Self-Check: PASSED

All listed implementation files, the summary, and task commit `221869e` exist.
