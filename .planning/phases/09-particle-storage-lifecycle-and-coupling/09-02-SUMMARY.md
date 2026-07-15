---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "02"
subsystem: particle-storage
tags: [rust, particles, soa, stable-identity, permutation, proptest]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "01"
    provides: checked particle definitions and closed production lane inventory
  - phase: 03-rust-object-model-and-storage-architecture
    provides: system-scoped generational particle identity and transactional storage evidence
provides:
  - authoritative production particle row lanes with lazy optional allocation
  - one validate-prepare-commit permutation authority for rotation and survivor compaction
  - independent randomized semantic evidence for stable IDs, references, invalid maps, and retirement
affects: [09-03, 09-04, 09-05, 09-06, 09-07, 09-08, 09-09, phase-10]
tech-stack:
  added: []
  patterns: [candidate-first row creation, validate-prepare-commit permutation, stable-ID semantic model]
key-files:
  created:
    - crates/liquidfun/src/particle/storage/lanes.rs
    - crates/liquidfun/src/particle/storage/permutation/tests.rs
    - crates/liquidfun/src/particle/storage/properties/permutation_model.rs
    - crates/liquidfun/src/particle/storage/validation.rs
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/identity.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/storage/properties.rs
key-decisions:
  - "Route rotation and survivor compaction through one permutation module that validates and prepares every lane, identity, reference, and group range before one commit."
  - "Preserve row-owned forces and stuck counters while clearing derived weights and transient stuck candidates for regeneration after a permutation."
  - "Compare randomized permutations through stable ParticleId semantics rather than reproducing the production dense-index algorithm in the model."
patterns-established:
  - "Failed particle creation and failed permutation validation leave identities, lanes, and derived references unchanged."
  - "Derived records survive compaction only when every referenced stable particle survives."
requirements-completed: [PART-03, PART-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T05:12:27Z
duration: 30 min
completed: 2026-07-15
---

# Phase 9 Plan 02: Production Particle Storage and Atomic Permutations Summary

**Complete production particle lanes with candidate-first creation and one atomic permutation authority proven against stable-ID semantic models.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-07-15T04:42:16Z
- **Completed:** 2026-07-15T05:12:27Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Replaced representative integer buffers with required `Vec2`, flag, group, weight, force, proxy, contact, pair, triad, and body-contact state plus lazy color, user-association, stuck, and expiration lanes.
- Kept dense rows private and made creation candidate-first, so capacity, identity, group-range, or lane validation failures cannot partially allocate an identity or append only part of a row.
- Centralized all supported row rotation and survivor compaction in a single validate-prepare-commit transaction that remaps identities, row lanes, references, expiration order, and group ranges before one authoritative mutation.
- Added 128-case property tests that compare stable semantic snapshots through optional-lane allocation, grouped permutations, pending deletion, contacts, body contacts, pairs, triads, invalid mappings, and terminal generation retirement.
- Added a source-scan regression that rejects a second permutation authority or direct rotation, retention, and swap-removal of authoritative storage lanes.

## Task Commits

Each task was committed atomically:

1. **Replace representative storage with production lanes** - `67fb31c` (feat)
1. **Prove atomic total permutations against an independent model** - `3c99e76` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/storage.rs` - Cohesive particle storage owner, stable identity access, candidate-first creation, and delegation to the exclusive permutation authority.
- `crates/liquidfun/src/particle/storage/lanes.rs` - Production required, optional, derived, stuck, lifetime, and owned-buffer lane vocabulary.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Total mapping validation, complete candidate preparation, reference remapping, identity retirement, and single commit.
- `crates/liquidfun/src/particle/storage/permutation/tests.rs` - Focused atomicity, payload, group-range, reference, and authority-scan regressions.
- `crates/liquidfun/src/particle/storage/properties.rs` - Existing bounded operation model wired to production particle inputs and optional lanes.
- `crates/liquidfun/src/particle/storage/properties/permutation_model.rs` - Independent stable-ID semantic model for valid and invalid permutations plus terminal retirement.
- `crates/liquidfun/src/particle/storage/validation.rs` - Shared derived-reference and contiguous group-range validation boundary.
- `crates/liquidfun/src/particle/storage/identity.rs` - Production-input identity and generation-retirement fixtures.

## Decisions Made

- The permutation module owns the only `apply_permutation` implementation. Publicly relevant stable semantics are checked before dense coordinates are rewritten, and storage entrypoints only construct mappings and delegate.
- Derived weights and transient stuck-candidate lists are regeneration state, so a permutation clears them. Row-owned forces and stuck counters move with the particle row.
- Group ranges are rebuilt from candidate groups before commit. A permutation that would split one group into disjoint ranges is rejected without mutation.
- Pair and triad records are remapped as references only. Their generation and solver behavior remain Phase 10 work.

## Deviations from Plan

None - the plan was executed as written.

## Issues Encountered

- The first warning-denied Clippy pass identified one production wildcard import and test-only precision-loss casts. Explicit imports and checked integer conversion resolved both before the ordered gate sequence was restarted.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 09-03 can give `World` authoritative particle-system ownership on top of production storage whose identity, capacity, and deferred-destruction behavior are executable invariants.
- Particle contact generation, solver coupling, and pair/triad topology generation remain deliberately outside this plan and retain their Phase 10 boundary.
- `storage.rs` is 628 lines after extracting permutation and validation responsibilities; it remains the cohesive storage owner while each specialized module stays below the repository's practical file-size trigger.

## Self-Check: PASSED

- All eight created or modified storage files exist.
- Task commits `67fb31c` and `3c99e76` are present.
- Focused lane, permutation, and property suites pass.
- The ordered format, warning-denied Clippy, all-target/all-feature build, and all-feature test gates pass.
- The authoritative source scan finds exactly one `apply_permutation` implementation and no direct row rotation, retention, or swap-removal in storage or lane ownership modules.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
