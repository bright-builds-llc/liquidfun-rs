---
phase: 12-performance-portability-and-release-hardening
plan: "07"
subsystem: testing
tags: [rust, jsonl, benchmarks, differential, protocol]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: reviewed Phase 12 performance policy, workload matrix, and native execution semantics
provides:
  - strict bounded benchmark request and result JSONL records
  - shared paired-run identity and request/result validation
  - separate performance, physics-mismatch, and harness-failure outcomes
affects: [paired-benchmark-runner, cpp-oracle-adapter, performance-reporting]
tech-stack:
  added: []
  patterns:
    - exact-byte request hashing before execution
    - authoritative unprofiled timing separated from optional profile diagnostics
key-files:
  created:
    - crates/liquidfun-test-protocol/src/performance/wire.rs
    - crates/liquidfun-test-protocol/tests/performance_wire.rs
  modified:
    - crates/liquidfun-test-protocol/src/performance.rs
    - crates/liquidfun-test-protocol/src/lib.rs
key-decisions:
  - "Echo one validated BenchmarkRunIdentity in request and result records, then validate the pair before accepting evidence."
  - "Represent performance, physics mismatch, and harness failure as mutually exclusive terminal outcomes; only performance carries authoritative unprofiled nanoseconds."
patterns-established:
  - "Paired-run identity: resolved hash, settings, workload, size, scalar mode, warmup, horizon, ordinal, policy, and profile mode travel together."
  - "Semantic checkpoint binding: timing evidence must carry a request- and resolved-hash-consistent checkpoint identity."
requirements-completed: [PERF-02, PERF-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T22:31:57Z
duration: 10min
completed: 2026-07-23
---

# Phase 12 Plan 07: Strict Benchmark Wire Contract Summary

**Bounded exact-byte benchmark records now give the Rust and C++ executors one identity-complete contract with unprofiled timing authority and distinct mismatch/failure outcomes.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-23T22:21:49Z
- **Completed:** 2026-07-23T22:31:57Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Added strict `benchmark_run_request` and `benchmark_run_result` JSONL records with reviewed byte, depth, horizon, sample, diagnostic-count, and policy bounds.
- Bound exact resolved bytes and SHA-256, solver settings, workload/size, scalar mode, warmup, horizon, sample ordinal, policy, profile mode, reset epoch, and semantic checkpoint identity.
- Kept authoritative unprofiled wall-clock measurements distinct from optional non-authoritative common-parent diagnostics.
- Kept successful performance results, semantic physics mismatches, and non-physics harness failures as mutually exclusive typed outcomes.
- Added six focused integration tests covering complete round trips, raw-value preservation, malformed/duplicate/unknown/missing/oversized input, identity contradictions, policy bounds, and every terminal outcome.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add strict benchmark request and result records** - `a273677` (feat)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/performance/wire.rs` - Strict benchmark identities, request/result records, codecs, bounds, and failure taxonomy.
- `crates/liquidfun-test-protocol/tests/performance_wire.rs` - Positive round-trip and negative framing, bounds, identity, and outcome coverage.
- `crates/liquidfun-test-protocol/src/performance.rs` - Declares and re-exports the reviewed wire API through the performance parent.
- `crates/liquidfun-test-protocol/src/lib.rs` - Makes the existing performance parent publicly reachable while preserving compatibility re-exports.

## Decisions Made

- Used one shared `BenchmarkRunIdentity` in both records so pair validation compares the complete execution identity instead of a subset of fields.
- Kept `unprofiled_nanoseconds` exclusively on the successful performance payload; profiled totals are rejected as unknown fields.
- Bound successful measurements and physics mismatches to semantic checkpoint identities while allowing pre-checkpoint harness failures to remain explicitly checkpoint-free.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exposed the existing performance parent module**

- **Found during:** Task 1 (Add strict benchmark request and result records)
- **Issue:** `lib.rs` declared `performance` privately, so integration tests and downstream executors could not use the plan-required `liquidfun_test_protocol::performance` import path.
- **Fix:** Changed the existing module declaration to `pub mod performance;` and retained the crate-root glob re-export for compatibility.
- **Files modified:** `crates/liquidfun-test-protocol/src/lib.rs`
- **Verification:** Focused parent-path integration tests and the complete ordered Rust gate passed.
- **Committed in:** `a273677`

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The one-line visibility fix was required to make the specified API path reachable; it introduced no new behavior or dependency.

## Issues Encountered

- Serde classifies an unknown member beside an adjacently tagged outcome as a malformed record, while forbidden fields inside the typed outcome payload retain the specific unknown-field category. Both forms fail before typed acceptance and are covered by tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Independent Rust and C++ benchmark executors can now consume and emit the same strict identity-complete records.
- The paired runner can validate exact request/result identity before incorporating raw samples into performance evidence.

## Known Stubs

None.

## Self-Check: PASSED

- Created protocol and focused-test files exist.
- Task commit `a273677` exists.
- Required request/result record names and parent-module exports are present.

***

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
