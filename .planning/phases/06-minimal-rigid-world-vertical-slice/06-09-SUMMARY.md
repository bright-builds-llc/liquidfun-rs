---
phase: 06-minimal-rigid-world-vertical-slice
plan: "09"
subsystem: rigid-world-differential-evidence
tags: [rust, rigid-world, differential, first-divergence, supervisor, minimization]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "07"
    provides: Closed 57-path phase6-v1 exact-first policy and rigid result schema
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "08"
    provides: Declaration-validated native rigid-world result adapter
provides:
  - Declaration-first native/oracle rigid comparison across all structural, ordered, and float observables
  - Stable action/checkpoint/field first-divergence signatures retained in failure bundles and reduction
  - Bounded rigid oracle supervision with handshake, concurrent drains, provenance, terminal, reset, kill, and reap enforcement
  - Validity-preserving timeline-action reduction and fail-closed D1 promotion authority
affects: [06-12-rigid-evidence-workflow, rigid-world-regressions, compatibility-reporting]
tech-stack:
  added: []
  patterns: [declaration-first comparison, exact semantic-path signatures, typed reduction revalidation, shared bounded supervisor machinery]
key-files:
  created:
    - crates/liquidfun-differential/src/rigid_evidence.rs
    - crates/liquidfun-differential/src/supervisor/rigid_world.rs
  modified:
    - crates/liquidfun-differential/src/failure_bundle.rs
    - crates/liquidfun-differential/src/minimizer.rs
    - crates/liquidfun-differential/src/supervisor.rs
    - crates/liquidfun-differential/tests/rigid_world.rs
key-decisions:
  - "Validate native and oracle results independently against request declarations before reading any cross-engine physics field."
  - "Preserve manager, report, manifold-point, and destruction sequences exactly; no rigid collection is sorted or canonicalized."
  - "Bind replay and reduction identity to witness family, preceding action, checkpoint, semantic path, mismatch kind, and phase6-v1 profile hash."
  - "Gate generic rigid fixture staging and promotion on BuildIdentity D1 authority so local D2 output cannot enter accepted reference paths."
patterns-established:
  - "Rigid comparison transaction: declaration validation for both engines, exact semantic-path alignment, then one first-divergence traversal."
  - "Rigid reducer boundary: edit ordered actions, decode the full candidate through the strict protocol, then evaluate only valid candidates."
requirements-completed: [RIGD-01, RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T06:49:22Z
duration: 14 min
completed: 2026-07-12
---

# Phase 6 Plan 09: Rigid Comparison and Supervision Summary

**Declaration-first rigid comparison with exact ordered semantics, stable action/checkpoint/field signatures, bounded C++ supervision, and validity-preserving reduction**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-12T06:35:19Z
- **Completed:** 2026-07-12T06:49:22Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Independently validates native and oracle root identity, timeline family, checkpoint identity/phase/counts, and declaration order before comparing physics fields.
- Traverses body, fixture, contact, manifold, event, and destruction evidence in engine-provided order and applies only the exact named `phase6-v1` float policy after alignment.
- Captures real rigid oracle output through the existing handshake, bounded concurrent-drain, timeout, terminal, reset, kill, and reap machinery while retaining `HarnessFailureKind` separation.
- Preserves exact first-divergence identity in machine reports, failure bundles, replay evaluation, and typed candidate reduction.
- Rejects local D2 rigid evidence before the generic fixture stage/review/promote lifecycle can target accepted reference paths.

## Task Commits

Each task was committed atomically:

1. **Task 1: Compare, diagnose, reduce, and supervise rigid traces** - `ac6452c` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_evidence.rs` - Declaration validation, exact-path comparison, stable mismatch reports, and D1 promotion guard.
- `crates/liquidfun-differential/src/supervisor/rigid_world.rs` - One-shot rigid oracle capture on shared bounded process primitives.
- `crates/liquidfun-differential/src/minimizer.rs` - Typed rigid timeline-action transforms with strict candidate revalidation.
- `crates/liquidfun-differential/src/failure_bundle.rs` - Optional exact failure-signature artifact and manifest digest.
- `crates/liquidfun-differential/src/main.rs` - Existing bundle call site explicitly opts out of a rigid signature.
- `crates/liquidfun-differential/src/lib.rs` - Exports the rigid evidence surface.
- `crates/liquidfun-differential/src/supervisor.rs` - Registers and exports rigid supervision.
- `crates/liquidfun-differential/tests/rigid_world.rs` - Structural, numeric, order, process, reset, reduction, bundle, and promotion-authority coverage.

## Decisions Made

- Declaration disagreements are boundary failures with engine-side attribution, never physics mismatches between two invalid results.
- Full ordered semantic records are compared directly; no manager, event, manifold-point, or destruction order is normalized.
- The reducer changes only timeline actions and relies on the strict request decoder to preserve both required families, every witness, lifecycle validity, checkpoint references, and bounds.
- Promotion authority is checked before generic fixture lifecycle effects and requires `BuildIdentity::can_promote_canonical_evidence()`.

## Deviations from Plan

None - plan executed exactly as written. The existing CLI bundle construction was updated to populate the new optional signature field so the planned failure-bundle extension remained source compatible.

## Issues Encountered

- The first reducer fixture used a redundant physics step, which correctly changed later declared event counts. The test was replaced with an idempotent duplicate custom-mass action after its checkpoint, preserving semantics while still proving valid action reduction.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p liquidfun-differential --test rigid_world comparison --all-features` — 4 passed.
- `cargo test -p liquidfun-differential --test rigid_world supervisor --all-features` — 2 passed against the real reviewed oracle when present.
- `cargo test -p liquidfun-differential --test rigid_world reduction --all-features` — 1 passed.
- `cargo test -p liquidfun-differential --test rigid_world --all-features` — 12 passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `cargo build --all-targets --all-features` — passed.
- `cargo test --all-features` — passed.
- `git diff --check` — passed.

## Next Phase Readiness

- Plan 06-12 can connect these comparison, capture, bundle, reduction, and authority primitives to contributor commands and canonical promotion workflow.
- No known blockers remain for the rigid evidence lifecycle.

## Self-Check: PASSED

- Created comparator and rigid supervisor files exist.
- Task commit `ac6452c` exists and contains the implementation and tests.
- Required comparison, supervisor, reduction, strict Clippy, build, test, source-identity, and diff checks pass.

***

*Phase: 06-minimal-rigid-world-vertical-slice*
*Completed: 2026-07-12*
