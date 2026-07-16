---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "19"
subsystem: particle-storage-lifecycle
tags: [rust, particles, creation-receipt, eviction, permutation, weights]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "06"
    provides: particle lifetime, destroy-by-age, and source-ordered compaction outcomes
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "05"
    provides: borrow-scoped aggregate particle views including contacts and weights
provides:
  - one owned public particle-creation receipt carrying the live created identity and synchronous committed eviction occurrences
  - contact-derived candidate weight recomputation inside the single transactional particle permutation authority
  - full-system eviction and stable-contact permutation regressions
affects: [09-20, 09-22, 09-23, phase-10]
tech-stack:
  added: []
  patterns: [owned synchronous outcome receipt, remap-then-derive candidate preparation, stable-ID contact assertions]
key-files:
  created:
    - crates/liquidfun/tests/particle_creation_eviction.rs
    - crates/liquidfun/tests/particle_permutation_coherence.rs
  modified:
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/storage/permutation/tests.rs
    - crates/liquidfun/src/particle/storage/properties/permutation_model.rs
key-decisions:
  - "Return committed capacity-eviction occurrences only through one owned must-use ParticleCreationReceipt; never reconstruct, queue, or redeliver them."
  - "Recompute permutation candidate weights from remapped body contacts followed by remapped particle contacts before the no-fail commit."
patterns-established:
  - "Public particle creation projects identity from a receipt so synchronous lifecycle outcomes cannot be silently discarded."
  - "Derived particle weights are prepared from the candidate contact graph, not copied or zero-filled independently of retained contacts."
requirements-completed: [API-09, PART-02, PART-04, PART-05, PART-08, PART-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-16T04:57:09Z
duration: 3h 37m
completed: 2026-07-16
---

# Phase 9 Plan 19: Eviction Receipts and Permutation Weight Coherence Summary

**Particle creation now returns committed capacity-eviction evidence synchronously, and every contact-bearing compaction or rotation publishes weights derived from its retained remapped contacts.**

## Performance

- **Duration:** 3h 37m
- **Started:** 2026-07-16T01:19:00Z
- **Completed:** 2026-07-16T04:56:08Z
- **Tasks:** 2
- **Files modified:** 27

## Accomplishments

- Added the single public, must-use `ParticleCreationReceipt` contract and migrated both creation entrypoints, exports, differential execution, doctests, unit tests, and integration callers to project the created identity explicitly.
- Preserved only the real commit's source-ordered destruction occurrences, proving a listener-flagged capacity victim appears synchronously exactly once while unrequested and failed replacement paths publish none.
- Recomputed candidate weights after contact remapping in the existing validate-prepare-commit permutation transaction, preserving the source body-contact-then-particle-contact accumulation order.
- Added compaction, rotation, invalid-map atomicity, property-model, and black-box stable-ID regressions that prevent contacts from being exposed beside stale or all-zero derived weights.

## Task Commits

Each task was committed atomically:

1. **Task 1: Carry immediate eviction occurrences through particle creation** - `431a862` (fix)
1. **Task 2: Recompute weights from remapped contacts before permutation commit** - `f6330f4` (fix)

## Files Created/Modified

- `crates/liquidfun/src/world/particle_object.rs` - Defines the owned creation receipt and carries the real committed compaction outcome into it.
- `crates/liquidfun/src/world/object.rs` - Returns the same receipt from the default-definition creation wrapper.
- `crates/liquidfun/src/particle.rs` and `crates/liquidfun/src/lib.rs` - Export the single receipt definition through the curated public surfaces.
- `crates/liquidfun/src/particle/storage.rs` - Shares the authoritative source-ordered contact-weight accumulator with permutation candidate preparation.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Remaps contacts, derives candidate weights, and commits the coherent candidate atomically.
- `crates/liquidfun/src/particle/storage/permutation/tests.rs` - Covers contact-bearing middle-row compaction, full rotation, and invalid-map no-effect behavior.
- `crates/liquidfun/src/particle/storage/properties/permutation_model.rs` - Independently models retained body and particle contact contributions by stable identity.
- `crates/liquidfun/tests/particle_creation_eviction.rs` - Proves requested and unrequested capacity-eviction receipt semantics.
- `crates/liquidfun/tests/particle_permutation_coherence.rs` - Proves public aggregate contacts and weights remain coherent immediately after compaction.

## Decisions Made

- Kept destruction occurrences owned by the successful creation return value rather than adding mutable world queues, callback registration, or later-step delivery.
- Reused the production `recompute_weights` addition order through one internal helper so ordinary contact refresh and permutation candidates cannot drift.
- Compared retained contacts by stable public particle identities; dense rows remain private and acquire no new compatibility contract.
- Applied the repository-local Rust, architecture, code-shape, verification, and testing standards to keep the domain transaction in storage, expose an invariant-carrying receipt, and test pure candidate behavior independently.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected the stale zero-weight property oracle**

- **Found during:** Task 2 (permutation weight coherence)
- **Issue:** The existing independent permutation model asserted that every successful permutation published all-zero weights, encoding the defect that this plan closes.
- **Fix:** Replaced that expectation with an independent stable-ID model that accumulates retained body-contact contributions before retained particle-contact contributions.
- **Files modified:** `crates/liquidfun/src/particle/storage/properties/permutation_model.rs`
- **Verification:** The 128-case total-permutation property, invalid-map atomicity property, focused permutation tests, and full all-feature suite pass.
- **Committed in:** `f6330f4`

***

**Total deviations:** 1 auto-fixed (1 Rule 1 bug).
**Impact on plan:** The independent oracle now verifies the planned behavior instead of preserving the known defect; no Phase 10 topology generation or solver behavior was added.

## Issues Encountered

- A previous Task 2 `cargo test --all-features` process was interrupted by the user turn and was discarded as evidence. A fresh uninterrupted four-command repository gate was run immediately before the Task 2 commit.
- Under concurrent machine load, macOS delayed several test binaries at dynamic-loader startup. The complete suite was retained and finished successfully; no target was skipped or retried as a substitute for a failure.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- G09-EVICTION-OCCURRENCE/WR-03 and G09-PERMUTATION-WEIGHTS/WR-04 now have implementation, focused regression, property-model, and full-suite closure.
- Plan 09-20 can validate the receipt-backed creation result and complete protocol lifecycle parity without inventing an asynchronous occurrence channel.
- Plans 09-22 through 09-24 can exercise the repaired runtime in the exact-ref evidence corpus; this plan makes no compatibility promotion claim.

## Self-Check: PASSED

- Both created regression files exist.
- Commits `431a862` and `f6330f4` contain plan ID `09-19`.
- Focused eviction, creation, permutation, property, and black-box coherence tests pass.
- Mandatory format, warning-denied Clippy, all-target/all-feature build, all-feature tests, and doctests pass.
- No shrink artifact, hidden occurrence queue, delayed callback, Phase 10 pair/triad generation, or solver behavior was added.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-16*
