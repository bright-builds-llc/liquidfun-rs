---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "20"
subsystem: rigid-native-evidence
tags: [rust, joints, callbacks, destruction, differential-evidence]
requires:
  - phase: 08-19
    provides: validator-backed step-bearing Phase 8 corpus
provides:
  - public-API native execution for all strengthened Phase 8 witnesses
  - exact callback and destruction lifecycle projection
  - deterministic post-step evidence for every joint kind and gear topology
affects: [08-21, 08-22, differential-adapters, rigid-sign-off]
tech-stack:
  added: []
  patterns: [public snapshot projection, authoritative lifecycle copying, typed-family contact isolation]
key-files:
  created: []
  modified:
    - crates/liquidfun-differential/src/rigid_world/evidence.rs
    - crates/liquidfun-differential/src/rigid_world/phase8.rs
    - crates/liquidfun-differential/tests/rigid_world_phase8.rs
    - protocol/fixtures/accepted/rigid-world-request.jsonl
key-decisions:
  - "Solver-only Phase 8 timelines reject incidental fixture contacts through the public collision hook so joint evidence remains isolated."
  - "Phase 8 lifecycle evidence is copied only for callback and destruction families from authoritative owned reports."
  - "Legacy Phase 7 staging tests use a test-only Phase 7 identity view of the complete valid typed corpus until production migration in Plan 08-22."
requirements-completed: [RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T05:35:00Z
duration: 40min
completed: 2026-07-14
---

# Phase 8 Plan 20: Native Step-Bearing Adapter and Evidence Summary

**The native adapter now executes every strengthened Phase 8 witness through public engine APIs and emits deterministic, bounded post-step and lifecycle evidence.**

## Performance

- **Duration:** 40 minutes
- **Started:** 2026-07-14T04:55:00Z
- **Completed:** 2026-07-14T05:35:00Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Executed all nineteen rigid families deterministically and emitted nontrivial live records for all eleven joint kinds, all four RR/RP/PR/PP gear source combinations, standalone rope, reconstruction, and diagnostics.
- Preserved callback filter, admission, PreSolve material/disable, repeated occurrence, and no-PostSolve behavior as exact ordered lifecycle arrays.
- Preserved explicit-versus-implicit destruction semantics, dependent-gear-first cascades, contact teardown, fixture goodbye, and body destruction order from authoritative reports.
- Isolated solver-only joint timelines from incidental fixture contacts while retaining declared mixed-joint collision suppression and callback/destruction contact behavior.
- Corrected the mixed-joint fixture so the surviving connected joint admits collision after the separate suppressing joint is destroyed, with a protocol regression protecting the sequence.
- Kept comparator production logic and C++ production code unchanged; tests locate semantic observations by kind and require the C++ adapter to fail closed until Plan 08-21.

## Task Commits

1. **Task 08-20-01: Execute and observe every strengthened witness through native APIs** - `bf2d79c` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase8.rs` - Refreshes filter pairs, isolates solver-only contacts, and projects only authoritative lifecycle families.
- `crates/liquidfun-differential/src/rigid_world/evidence.rs` - Keeps legacy Phase 6/7 event projection separate from typed Phase 8 lifecycle evidence.
- `crates/liquidfun-differential/tests/rigid_world_phase8.rs` - Pins deterministic solver, gear, mutation, callback, destruction, rope, and diagnostic behavior.
- `crates/liquidfun-differential/tests/rigid_world.rs` - Requires the pre-08-21 C++ adapter to reject strengthened execution cleanly.
- `crates/liquidfun-differential/tests/phase8_comparator.rs` - Finds strengthened mutation observations by semantic kind.
- `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs` - Validates the checked-in Phase 8 profile and preserves legacy staging control-flow coverage through a test-only identity view.
- `crates/liquidfun-differential/tests/round_trip.rs` - Requires the exact unsupported-Phase-8 CTest boundary until Plan 08-21.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs` - Protects mixed-joint collision restoration order.
- `protocol/fixtures/accepted/rigid-world-request.jsonl` - Permits the surviving mixed connected joint to collide after suppressor destruction.

## Decisions Made

- Used the public collision hook to suppress undeclared solver-fixture contact pairs instead of inspecting or mutating private contact state.
- Re-filtered both fixtures when a callback directive changes so reject-to-admit transitions are reconsidered on the next public world step.
- Kept ordinary Phase 6/7 report collection unchanged and treated typed Phase 8 lifecycle observations as the only Phase 8 event authority.
- Kept all nineteen valid typed timelines in the legacy staging test view because the current closed validator requires every family; changed only the temp copy's request/scenario identity and policy hash.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected contradictory mixed-joint collision declaration**

- **Found during:** Native mixed-joint execution
- **Issue:** Both connected joints suppressed collision, but the witness expected collision to return after destroying only one joint.
- **Fix:** Set `joint-mixed-connected.collide_connected` to true and added a protocol ordering regression.
- **Verification:** Focused protocol and native Phase 8 suites passed.

**2. [Rule 3 - Blocking] Preserved legacy staging coverage under the closed Phase 8 validator**

- **Found during:** Full differential workflow verification
- **Issue:** The checked-in request now has Phase 8 identity while production staging remains Phase 7 until Plan 08-22; a nine-timeline view is also rejected because all nineteen families are mandatory.
- **Fix:** Construct a test-only Phase 7 identity/profile view of all nineteen typed timelines solely for legacy transaction control-flow tests and increased the deterministic minimizer attempt budget for the larger corpus.
- **Verification:** `rigid_fixture_workflow` passed 15/15 and the full differential crate passed.

**3. [Rule 3 - Blocking] Updated the pre-08-21 CTest expectation**

- **Found during:** Full differential crate verification
- **Issue:** The C++ protocol self-test still expected the old adapter to execute the strengthened fixture.
- **Fix:** Require the exact fail-closed `unsupported Phase 8 execution action` result until Plan 08-21 implements the C++ side.
- **Verification:** `round_trip` passed 13/13 without production C++ changes.

## Issues Encountered

- The first callback admission step remained filtered because changing the adapter directive alone did not touch broad-phase proxies; round-tripping the public fixture filter correctly schedules reconsideration.
- Incidental overlapping fixtures inflated joint-family contact and lifecycle evidence; typed-family collision-hook isolation removed that unrelated solver input.
- The expanded corpus requires 16,384 bounded minimizer attempts rather than the legacy 4,096-attempt ceiling.

## Verification

- Focused Phase 8 protocol suite: 13 tests passed.
- Focused native Phase 8 suite: 10 tests passed.
- Rigid-world integration suite: 45 tests passed.
- Phase 8 comparator suite: 4 tests passed.
- Rigid fixture workflow: 15 tests passed.
- Full `liquidfun-differential` crate, including 13 round-trip and 10 supervisor tests: passed.
- `cargo fmt --all`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo build --all-targets --all-features`: passed.
- `cargo test --all-features`: 185 library tests, every integration target, and 13 doctests passed.
- `git diff --check`: passed before the implementation commit.

## User Setup Required

None.

## Next Phase Readiness

- Native Phase 8 evidence is ready for independent C++ adapter implementation in Plan 08-21.
- Plan 08-22 still owns production staging migration and native/C++ differential comparison.
- The C++ adapter remains intentionally fail closed for strengthened Phase 8 execution until Plan 08-21.
