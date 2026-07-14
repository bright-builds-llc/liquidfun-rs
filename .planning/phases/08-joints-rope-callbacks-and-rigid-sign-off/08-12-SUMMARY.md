---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "12"
subsystem: differential-adapter
tags: [rust, joints, rope, callbacks, lifecycle, reconstruction]
requires:
  - phase: 08-11
    provides: closed bounded phase8-v1 request and observation protocol
provides:
  - native execution of all nineteen retained and Phase 8 rigid witness families
  - public-API mappings for all eleven joint kinds, standalone rope, callbacks, lifecycle, reconstruction, and diagnostics
  - deterministic semantic observations with checked private identity and occurrence mappings
affects: [phase-8-cpp-adapter, differential-evidence, rigid-sign-off]
tech-stack:
  added: []
  patterns: [thin public-API adapter, checked semantic identity maps, authoritative lifecycle projection, fail-closed deferred backend gate]
key-files:
  created:
    - crates/liquidfun-differential/src/rigid_world/phase8.rs
    - crates/liquidfun-differential/src/rigid_evidence/phase8.rs
    - crates/liquidfun-differential/tests/rigid_world_phase8.rs
  modified:
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_world/evidence.rs
    - crates/liquidfun-differential/src/rigid_world/phase7.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
    - protocol/fixtures/accepted/rigid-world-request.jsonl
key-decisions:
  - "Phase 8 native execution uses only public liquidfun APIs; scenario identities, native handles, occurrence ordinals, and reconstruction coordinates remain adapter-private."
  - "Every step routes through one directive hook, and authoritative lifecycle records retain their engine effect order."
  - "Phase 8 C++ execution fails closed before process spawn with phase8_cpp_adapter_pending_plan_08_13 until Plan 08-13 implements that backend."
patterns-established:
  - "Exhaustive adapter dispatch: all closed joint definitions and mutations are total matches that return typed harness failures."
  - "Semantic reconstruction maps newest-first public output back to checked per-body fixture and joint identities without exposing storage coordinates."
requirements-completed: [RIGD-11, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T02:30:57Z
duration: 27min
completed: 2026-07-14
---

# Phase 8 Plan 12: Native Phase 8 Adapter Summary

**The complete nineteen-family phase8-v1 corpus now executes deterministically through native Rust, producing bounded joint, rope, callback, lifecycle, reconstruction, and diagnostic evidence.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-07-14T02:03:28Z
- **Completed:** 2026-07-14T02:30:57Z
- **Tasks:** 1
- **Files modified:** 19

## Accomplishments

- Mapped all eleven joint definitions and every closed mutation to checked public `liquidfun` APIs, including gear dependency snapshots and reactions.
- Added standalone rope creation, mutation, stepping, inspection, teardown, deterministic semantic snapshots, and complete reset proof.
- Routed ordinary and configured world steps through validated contact-filter and pre-solve directives, then emitted authoritative lifecycle observations in effect order with checked occurrence ordinals.
- Added public reconstruction and diagnostics projections with semantic body, fixture, joint, rope, and dependency identities while keeping native handles and output-local coordinates private.
- Preserved every Phase 6 and Phase 7 family, including destruction and continuous-collision projections, through the new Phase 8 executor path.
- Added focused coverage for the full corpus, all eleven joint kinds, all seventeen mutation branches, callback timing, reconstruction, diagnostics, retained families, deterministic reset, and the explicit C++ backend gate.

## Task Commits

Each task was committed atomically:

1. **Task 08-12-01: Implement native joint/rope/callback/reconstruction execution and observations** - `d95d8d7` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase8.rs` - Executes Phase 8 actions and maps joint, rope, callback, lifecycle, reconstruction, and diagnostic evidence.
- `crates/liquidfun-differential/src/rigid_evidence/phase8.rs` - Converts public joint snapshots into bounded semantic protocol observations.
- `crates/liquidfun-differential/tests/rigid_world_phase8.rs` - Covers deterministic full-corpus execution, all joint kinds and mutations, hook timing, reconstruction, diagnostics, and C++ gating.
- `crates/liquidfun-differential/src/rigid_world.rs` - Integrates Phase 8 dispatch, checked identity ownership, hooks, reset proof, and retained timeline execution.
- `crates/liquidfun-differential/src/rigid_world/evidence.rs` - Preserves destruction and continuous-collision evidence under authoritative lifecycle collection.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs` - Accepts all nineteen result timelines and validates multi-record reconstruction observations.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Aligns rope cardinality and gear topology with the public native API contract.
- `protocol/fixtures/accepted/rigid-world-request.jsonl` - Corrects canonical rope and gear declarations for native execution.

## Decisions Made

- Stored validated native pre-solve directives rather than re-validating inside the infallible hook callback.
- Tracked fixture ownership separately so reconstruction preserves newest-first order within each body rather than relying on a global fixture reversal.
- Added a separate lifecycle occurrence set and checked monotonic ordinal so retained legacy event collection cannot affect authoritative Phase 8 timing.
- Explicitly gated Phase 8 C++ execution before child spawn because that backend and cross-engine comparator belong to Plan 08-13.

## Deviations from Plan

### Automatically Fixed Issues

**1. Closed Phase 8 request and result limits from Plan 08-11 did not map to the public native contracts**

- **Found during:** Native full-corpus execution
- **Issue:** Results allowed only nine timelines, rope declarations allowed two vertices although `RopeDef` requires three, gear sources could share one moving endpoint, and only one reconstruction observation was accepted.
- **Fix:** Raised the result bound to the closed nineteen-family registry, aligned rope/schema minimums, validated gear moving endpoints, accepted checked contiguous reconstruction records, and corrected the canonical fixture.
- **Files modified:** Protocol result/validation/schema sources and accepted request/schema presentations.
- **Verification:** Protocol Phase 8 tests, schema byte-stability test, and native nineteen-family execution all pass.

**2. Retained Phase 6/7 evidence needed adaptation to authoritative lifecycle collection**

- **Found during:** Retained rigid-world regression suite
- **Issue:** Direct destruction and continuous-collision projections could duplicate or lose legacy evidence after lifecycle reports became authoritative.
- **Fix:** Collected lifecycle before removing semantic mappings and limited synthetic continuous pre-solve projection to partial-progress evidence.
- **Files modified:** Native rigid-world executor, Phase 7 step path, and evidence projection.
- **Verification:** All 45 retained rigid-world integration tests pass unchanged.

## Issues Encountered

- A focused callback test confirmed that disabled pre-solve retains three legacy report events while authoritative lifecycle correctly ends at `PreSolve` without a `PostSolve`; the test now pins both contracts.
- The Phase 8 C++ adapter is intentionally unavailable until Plan 08-13 and fails closed with the stable reason `phase8_cpp_adapter_pending_plan_08_13` before any process is spawned.

## Verification

- `cargo test -p liquidfun-differential --test rigid_world_phase8 --all-features` - 6 passed.
- `cargo test -p liquidfun-differential --test rigid_world --all-features` - 45 passed.
- `cargo test -p liquidfun-test-protocol rigid_world_phase8 --all-features` - 7 passed.
- Schema presentation byte-stability test - passed.
- Ordered Rust gate: format, Clippy with denied warnings, all-target build, and all-feature tests - passed.
- `git diff --check` - passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-13 can implement the C++ Phase 8 adapter and comparator against the now-green native semantic result surface.
- The explicit pre-spawn gate prevents Phase 8 requests from accidentally running against the older C++ protocol implementation.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*
