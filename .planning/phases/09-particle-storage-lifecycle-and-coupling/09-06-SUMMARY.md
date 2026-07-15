---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "06"
subsystem: particle-lifecycle
tags: [rust, particles, lifetime-clock, stable-identity, transactional-compaction]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "03"
    provides: authoritative particle storage, pending snapshots, and stable public identity
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "04"
    provides: unified owned lanes and allocation-preserving permutations
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "05"
    provides: safe stable-ID inspection and derived-state repair
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "15"
    provides: pinned equal-expiration oracle witness and exact provenance
provides:
  - checked 32.32 lifetime clock with source-equivalent truncation and i32 expiration values
  - canonical finite-before-infinite stable-ID oldest ordering and immediate destroy-by-age eviction
  - ascending-old-row zombie compaction with owned requested-listener occurrences before invalidation
  - independent lifecycle property model spanning creation, ticking, expiry, marking, eviction, compaction, pause markers, and optional lanes
affects: [09-07, 09-08, 09-09, 09-13, phase-10]
tech-stack:
  added: []
  patterns: [pure lifetime coordinator, source-timed owned effect, one survivor permutation]
key-files:
  created:
    - crates/liquidfun/src/particle/lifetime.rs
    - crates/liquidfun/src/particle/lifetime/tests.rs
    - crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs
    - crates/liquidfun/tests/particle_lifetimes.rs
  modified:
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/storage/properties.rs
key-decisions:
  - "Represent elapsed lifetime time as checked 32.32 fixed point and preserve the pinned truncation-toward-zero conversions without inheriting signed overflow."
  - "Lock equal finite expirations to the pinned witness: later insertion is selected first by oldest eviction, independent of Rust sort or hash behavior."
  - "Emit stable-ID destruction-listener occurrences during the ascending old-row scan, then invalidate all pending identities through one authoritative permutation."
patterns-established:
  - "Lifetime state prepares pure clock/order decisions while ParticleStorage remains the authority that marks, snapshots, invalidates, and permutes rows."
  - "Immediate capacity eviction uses the ordinary pending compactor and adds no listener request of its own."
requirements-completed: [PART-08, PART-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T06:57:01Z
duration: 25 min
completed: 2026-07-15
---

# Phase 9 Plan 06: Particle Lifetime and Zombie Lifecycle Summary

**Pinned lifetime quantization, oracle-derived oldest selection, and snapshot-before-invalidation zombie compaction now operate over stable particle identity.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-07-15T06:32:24Z
- **Completed:** 2026-07-15T06:57:01Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added a checked public 32.32 lifetime clock and stable-ID ordering reachable from both curated particle and crate-root exports.
- Encoded the pinned witness's equal-expiration selection order, finite-before-infinite eviction, expiry marking, and immediate destroy-by-age compaction.
- Produced requested destruction-listener occurrences in ascending old-row order before one total survivor permutation invalidates identities.
- Extended the independent property model across the complete lifecycle operation vocabulary while preserving optional-lane and pause-marker behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Port the lifetime clock and canonical oldest ordering** - 037a39d (feat)
1. **Task 2: Prove zombie snapshot and compaction sequencing** - 528bdf8 (feat)

## Files Created/Modified

- crates/liquidfun/src/particle/lifetime.rs - Checked clock, stable-ID ordering, lifecycle coordinator, and compaction effects.
- crates/liquidfun/src/particle/lifetime/tests.rs - Focused expiry, capacity, pending-oldest, and occurrence-order regressions.
- crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs - Independent randomized lifecycle state machine.
- crates/liquidfun/tests/particle_lifetimes.rs - External reachability, clock quantization, invalid-input, and canonical witness tests.
- crates/liquidfun/src/particle/storage.rs - Narrow lifetime-lane, pending-snapshot, and present-row adapters.
- crates/liquidfun/src/particle/storage/permutation.rs - Particle-module visibility for the single permutation authority.
- crates/liquidfun/src/particle/storage/properties.rs - Lifecycle property child-module wiring.
- crates/liquidfun/src/particle.rs - Public lifetime and destruction-occurrence exports.
- crates/liquidfun/src/lib.rs - Curated crate-root re-exports.

## Decisions Made

- Checked overflow is a typed no-effect error; only successful conversions preserve the pinned C++ truncation order.
- Equal finite expiration ties select the newest inserted stable identity first, exactly matching witness SHA 08d41d25f3766b9bf4bef51fb10713b7f925c074399b9642ad5cb4ce933fc8e3.
- Listener occurrences carry stable semantic particle identity and are collected before invalidation; later World integration can append them directly to the unified journal.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exposed narrow storage-to-lifecycle seams**

- **Found during:** Task 1
- **Issue:** The planned lifetime child module could not update private expiration lanes or invoke the existing private permutation authority.
- **Fix:** Added narrow particle-module adapters and widened only apply_permutation to particle-module visibility.
- **Files modified:** crates/liquidfun/src/particle/storage.rs, crates/liquidfun/src/particle/storage/permutation.rs
- **Verification:** Full Rust gate sequence and the production single-permutation-authority regression passed.
- **Committed in:** 037a39d, 528bdf8

**2. [Rule 1 - Bug] Allowed dirty lifetime ordering to retain pending rows**

- **Found during:** Task 2 property testing
- **Issue:** A minimized create-mark-destroy-oldest sequence found that dirty ordering used live-only resolution and rejected an already-pending stable identity.
- **Fix:** Added present-row resolution for live or pending expiration entries and a focused readable regression.
- **Files modified:** crates/liquidfun/src/particle/storage.rs, crates/liquidfun/src/particle/lifetime/tests.rs
- **Verification:** The minimized regression and 128-case lifecycle property suite pass.
- **Committed in:** 528bdf8

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug). **Impact:** Both changes were necessary to connect the planned deep module and preserve source-compatible pending lifecycle behavior; no solver or World-step scope was added.

## Issues Encountered

- A second minimized property sequence exposed an independent-model error: it reconstructed equal ties from dense creation order instead of retaining the prior stable expiration queue. The model now owns and stable-sorts its own order. No production change was required for that case.
- TDD RED signals were captured before implementation, but RED-only commits were not created because repository instructions require the full format, lint, build, and test sequence to pass before every commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 09-07 World-step lifecycle integration. The lifecycle coordinator, ordered occurrences, and one compaction transaction are available without adding Phase 10 solver behavior.

## Self-Check: PASSED

- All key created files exist.
- Both task commits are present with 09-06 in their messages.
- Focused lifetime/property verification, cargo check -p liquidfun --all-features, and all four required Rust gates pass.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
