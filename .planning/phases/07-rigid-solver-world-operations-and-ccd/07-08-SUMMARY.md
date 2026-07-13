---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "08"
subsystem: rigid-continuous-collision
tags: [rust, ccd, toi, deterministic-order, transactional-rollback]
requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "04"
    provides: Transactional rigid body stepping, complete BodyState staging, and source-ordered islands
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "05"
    provides: Sleeping, waking, activation, bullet flags, and coherent world mutation controls
provides:
  - Private checked per-contact TOI cache and strict upstream sub-step counter lifecycle
  - Fresh-versus-resumed continuous-step state keyed by exact checked step configuration
  - Deterministic bounded manager-order CCD scanning with strict-less equal-alpha tie handling
  - Sweep equalization, checked TOI alpha conversion, tentative advance, complete rollback, and wake-on-accept
  - Semantic differential witnesses for selection, rejection, and eligibility exclusions without exposing CCD storage
affects: [07-09, 07-11, phase-7-rigid-evidence]
tech-stack:
  added: []
  patterns: [private checked state machine, manager-order strict-less selection, complete-state rollback, semantic-only differential diagnostics]
key-files:
  created:
    - crates/liquidfun/src/world/continuous.rs
    - crates/liquidfun/src/world/continuous/tests.rs
    - crates/liquidfun/tests/rigid_ccd_selection.rs
  modified:
    - crates/liquidfun/src/world.rs
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun/src/world/contact.rs
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/rigid_differential.rs
key-decisions:
  - "Scan the contact manager's existing source order directly and update the winner only on strict-less alpha, so equal-time contacts never need sorting or storage identity."
  - "Bound private CCD scanning at the reviewed 8,192-contact world limit while preserving the pinned strict `toi_count > MAX_SUB_STEPS` exclusion and one representable terminal count."
  - "Use feature-gated owned semantic diagnostics and named rejection controls to test CCD behavior without exposing cache flags, counts, indices, sweeps, or resume storage."
patterns-established:
  - "CCD cache lifecycle: public body/contact/fixture mutation invalidates relevant cached alpha, fresh steps reset cache and count, and only an exact pending step key retains them."
  - "CCD validation transaction: copy both complete BodyState values, advance and refresh, then either preserve and wake or restore the exact copies on rejection."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T06:22:22Z
duration: 32 min
completed: 2026-07-13
---

# Phase 7 Plan 08: CCD Candidate State Machine Summary

**Rigid worlds can now deterministically identify and transactionally validate the earliest eligible continuous-collision event through a bounded private TOI state machine.**

## Performance

- **Duration:** 32 min
- **Started:** 2026-07-13T05:50:17Z
- **Completed:** 2026-07-13T06:22:22Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added checked contact-local TOI alpha caching, a strict upstream-compatible sub-step counter bound, and invalidation across contact geometry, enabled/filter/sensor state, and participating body mutations.
- Added exact step-configuration identity for fresh versus matching pending continuous work; fresh work resets contact TOI cache/count state while matching pending work retains it.
- Scans contacts directly in manager order, skips the pinned disabled, budget, sensor, inactive, and non-bullet dynamic/dynamic cases, reuses valid cache entries, and computes missing values through the existing checked TOI kernel.
- Equalizes sweep start fractions, converts local TOI beta to absolute alpha with pinned arithmetic, preserves the first equal-alpha occurrence through strict-less selection, and rejects near-complete events at the pinned epsilon threshold.
- Tentatively advances complete body states and refreshes the candidate contact, restoring exact transforms, sweeps, snapshots, velocities, forces, torque, mass, and sleep state on rejection while preserving the accepted advance and waking both bodies.
- Added black-box semantic selection/exclusion witnesses plus a module-private complete-`BodyState` rollback regression guard.

## Task Commits

Each task was committed atomically after the exact ordered Rust gate passed:

1. **Task 1: Add bounded CCD state and contact TOI cache metadata** - `ed89072` (`feat`)
2. **Task 2: Select the earliest eligible TOI contact in manager order** - `d5dbfb6` (`feat`)

## Files Created/Modified

- `crates/liquidfun/src/world/continuous.rs` - Private bounded cache lifecycle, scan, selection, TOI conversion, validation, rollback, and pending-step state machine.
- `crates/liquidfun/src/world/continuous/tests.rs` - Cache/count/pending lifecycle and complete internal body-state rollback evidence.
- `crates/liquidfun/tests/rigid_ccd_selection.rs` - Semantic equal-time ordering, rejection, sensor/activity/bullet, and sub-step exclusion witnesses.
- `crates/liquidfun/src/world/contact.rs` - Checked cached alpha and TOI-count representation with mutation-driven invalidation.
- `crates/liquidfun/src/world/contact_manager.rs` - Manager-order mutable access, body cache invalidation, continuous refresh, and bounded diagnostic budget setup.
- `crates/liquidfun/src/world/body.rs` - Checked sweep-only equalization and synchronized tentative body advancement.
- `crates/liquidfun/src/world/object.rs` - World-owned continuous state plus invalidation at body, fixture, mass, transform, and control mutation boundaries.
- `crates/liquidfun/src/world.rs` - Private continuous module integration.
- `crates/liquidfun/src/rigid_differential.rs` - Feature-gated owned semantic CCD result, failure, and named rejection controls.

## Decisions Made

- Retained manager `Vec` order as the only selection order and used a strict `alpha < minimum_alpha` comparison. No pointer, hash, float sort, or extra stable-order layer enters CCD.
- Kept absolute cached alpha in a checked `0.0..=1.0` type and allowed the count to reach exactly `MAX_SUB_STEPS + 1`, which is required to represent the pinned strict-greater exclusion without overflow.
- Applied sweep alpha equalization to private body state before each uncached TOI query, matching the pinned shared-time-interval behavior for later contacts in the same scan.
- Capped one scan at the existing reviewed 8,192-contact world scale and checked rejection increments so adversarial contact sets cannot turn private diagnostics into unbounded work.
- Kept production CCD and pending state entirely private. The optional unpublished differential feature transports only owned semantic occurrence, alpha, contact, and named failure evidence.

## Test Evidence

- Task 1 RED exited 101 on the absent private continuous-step and contact TOI state symbols; both exact focused lifecycle tests then passed GREEN.
- Task 2 RED exited 101 on the absent semantic CCD diagnostic surface; all three planned integration witnesses then passed GREEN.
- The sensor exclusion witness initially exposed that a newly admitted contact's cached sensor flag had not yet been refreshed. The scanner was corrected to inspect the owning fixture definitions, exactly as the pinned source does, and the test passed.
- Focused final checks passed:
  - `cargo test -p liquidfun ccd_cache_is_invalidated_by_contact_and_sweep_changes --lib`
  - `cargo test -p liquidfun pending_ccd_state_survives_only_the_matching_step --lib`
  - `cargo test -p liquidfun rejected_ccd_candidate_restores_internal_body_state --lib`
  - `cargo test -p liquidfun --all-features --test rigid_ccd_selection` - 3/3
  - `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings`
- Before both retained task commits, the exact ordered gate passed with authoritative exit code 0:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`
- The final full test gate included 150 library tests, the new three-test CCD integration target, every existing integration target, and all 12 doctests.

## Simplification Review

- One deep `continuous` module owns lifecycle, scanning, selection, and validation; body and contact-manager additions are narrow cohesive primitives rather than alternate orchestration paths.
- Module-private tests moved to `continuous/tests.rs`, keeping the production state machine under 500 lines without fragmenting the algorithm into shallow modules.
- Existing checked `Sweep` and `time_of_impact` APIs perform all geometry and time arithmetic; no new collision kernel, dependency, unsafe path, or public continuation token was added.
- Exact copied `BodyState` rollback is smaller and safer than maintaining a field-by-field undo log, and the private equality regression proves every currently owned lane is restored.

## Deviations from Plan

- `crates/liquidfun/src/world/body.rs` was additionally modified because sweep-only alpha equalization and synchronized tentative advance must be cohesive operations over private `BodyState` fields.
- `crates/liquidfun/src/rigid_differential.rs` was additionally modified to follow the repository's established feature-gated semantic evidence pattern. It exposes no CCD cache, counter, index, sweep, or continuation representation.
- Module-private tests were moved to `crates/liquidfun/src/world/continuous/tests.rs` during the required simplification pass so the production implementation remains within the repository file-size guidance.

### Process adjustment: RED evidence was not committed

- Repository policy requires the complete ordered Rust gate before every commit. Both intentionally failing RED states were run and recorded but not committed; only verified GREEN task states were retained.

## Issues Encountered

- Task 1's first clippy gate correctly rejected private reader/increment methods that were not consumed until Task 2. Narrow temporary deferral annotations allowed the Task 1 checkpoint to pass; Task 2 then consumed those methods and removed every temporary annotation.
- The first combined sensor exclusion test selected a candidate because contact-local sensor state was stale before its first refresh. Reading fixture sensor definitions in the uncached eligibility path fixed the root cause and matches upstream `SolveTOI`.
- Long full-suite commands yielded through retained sessions. Their OS/session exit status was checked before each commit, and no commit was made while a gate process was active.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-09 can consume the private `ContinuousCandidate`, matching pending-step lifecycle, accepted body pair, refreshed touching contact, and retained TOI count to build and solve bounded TOI islands.
- Plan 07-11 can record semantic CCD occurrence, alpha, body, and contact evidence without depending on cache flags or storage coordinates.
- No production blocker remains. The deliberate residual boundary is that public `World::step` integration and TOI island solving belong to Plan 07-09.

## Self-Check: PASSED

- Task commits `ed89072` and `d5dbfb6` exist; all three declared created files exist.
- The nine-file implementation/test diff is represented above, and every focused and full verification command passed.
- Diff review found no unintended public CCD storage, continuation token, unsafe code, new dependency, pointer/hash ordering, unchecked counter, or rollback gap.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-13*
