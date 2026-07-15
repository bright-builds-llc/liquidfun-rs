---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "03"
subsystem: particle-object-lifecycle
tags: [rust, particles, world-ownership, stable-identity, destruction]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "02"
    provides: authoritative production particle lanes and atomic stable-ID compaction
  - phase: 03-rust-object-model-and-storage-architecture
    provides: world- and system-scoped handles, destruction records, and association cleanup
provides:
  - one authoritative ParticleStorage owner per live particle system
  - public multi-system creation, configuration, inspection, pending deletion, and teardown APIs
  - complete system and particle destruction evidence with distinct scoped handle failures
affects: [09-04, 09-05, 09-06, 09-07, 09-08, 09-09, phase-10]
tech-stack:
  added: []
  patterns: [system-owned particle storage, preflight-before-identity allocation, snapshot-before-cascade]
key-files:
  created:
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun/tests/particle_objects.rs
  modified:
    - crates/liquidfun/src/arena.rs
    - crates/liquidfun/src/error.rs
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/identity.rs
    - crates/liquidfun/src/particle/storage/properties.rs
    - crates/liquidfun/src/world.rs
    - crates/liquidfun/src/world/object.rs
key-decisions:
  - "Store all particle rows, membership, pending state, and diagnostic identity in the owning system's ParticleStorage; World keeps no parallel particle arena or membership vectors."
  - "Preflight particle creation before allocating a world diagnostic identity, then commit immediately into the already-validated storage candidate."
  - "Snapshot group and particle membership before system teardown, emit owned records in pinned category and occurrence order, and drop the complete storage owner in one cascade."
patterns-established:
  - "Public particle resolution validates world, system, generation, and pending state through one storage authority."
  - "Pending particles retain identity until compaction produces the record applications use for explicit AssociationMap cleanup."
requirements-completed: [PART-01, PART-02, PART-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T05:39:12Z
duration: 22 min
completed: 2026-07-15
---

# Phase 9 Plan 03: Authoritative Particle Object Lifecycle Summary

**Converged the public particle object model on one stable, system-owned storage authority with transactional pending deletion and complete teardown evidence.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-07-15T05:17:11Z
- **Completed:** 2026-07-15T05:39:12Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Removed the placeholder world-level particle arena and redundant system/group particle membership vectors; each live particle system now owns the only authoritative `ParticleStorage` for its rows and lifecycle state.
- Added public system and particle snapshots, newest-first system enumeration, checked pause configuration, definition-backed creation, distinct pending access, deferred compaction, and immediate or cascading destruction.
- Preserved world- and system-scoped stable identity across dense-row compaction and slot reuse while separating wrong-world, wrong-system, pending, and stale outcomes.
- Made failed fixed-capacity creation atomic with respect to both particle state and world diagnostic identity allocation.
- Added black-box coverage for multi-system isolation, association cleanup timing, complete group/system snapshots, pending rows during system teardown, and removal of all resolvable group and particle references.

## Task Commits

Each task was committed atomically:

1. **Make particle systems own authoritative storage** - `b94d07f` (feat)
1. **Protect multi-system identity and destruction behavior** - `3a8c69d` (test)

## Files Created/Modified

- `crates/liquidfun/src/world/particle_object.rs` - Public particle-system and particle lifecycle adapter over authoritative per-system storage.
- `crates/liquidfun/tests/particle_objects.rs` - Black-box identity, ordering, compaction, capacity, association, and teardown regressions.
- `crates/liquidfun/src/world/object.rs` - Particle systems own storage; placeholder particle authority and redundant membership lists are removed from world objects.
- `crates/liquidfun/src/particle/storage.rs` - Narrow lifecycle, snapshot, preflight, diagnostic-identity, group-clear, and targeted-compaction adapters on the existing storage owner.
- `crates/liquidfun/src/particle/storage/identity.rs` - Stable identity entries retain their world-local diagnostic identity.
- `crates/liquidfun/src/particle/storage/properties.rs` - Existing property fixtures supply the new diagnostic field.
- `crates/liquidfun/src/arena.rs` - Read-only next-handle preparation supports invariant-valid system storage before atomic arena insertion.
- `crates/liquidfun/src/error.rs` - Public handle vocabulary distinguishes pending deletion from stale destruction.
- `crates/liquidfun/src/world.rs` - Declares the particle lifecycle module.
- `crates/liquidfun/src/particle.rs` - Re-exports supported particle snapshots through the particle namespace.
- `crates/liquidfun/src/lib.rs` - Re-exports supported particle snapshots from the crate root.

## Decisions Made

- `ParticleStorage` is the sole particle authority. Public world methods locate the owning system from the scoped handle and delegate row state, pending state, group membership, compaction, and stable identity validation to that storage.
- Creation validates owner scope, group scope, and storage capacity before consuming a diagnostic identity. The immediate commit after preflight has no intervening mutation point.
- System destruction captures authoritative group and particle snapshots first, then emits groups, particles, and the system in the established cascade order. Pending particles are included and become stale with the rest of the removed storage.
- Application associations remain application-owned. Destruction and compaction records provide the exact stable identities needed for explicit, ordered `AssociationMap` cleanup.

## Deviations from Plan

### Auto-fixed Supporting Files

**1. Extended the private arena with read-only next-handle preparation**

- **Found during:** Task 1
- **Issue:** A system's `ParticleStorage` requires its final system-scoped identity at construction, but the generic arena previously exposed that identity only after consuming the value.
- **Fix:** Added `Arena::next_handle` with a focused regression proving the prepared handle equals the next atomic insertion. No insertion, removal, generation, or reuse semantics changed.
- **Files:** `crates/liquidfun/src/arena.rs`
- **Verification:** Warning-denied Clippy, all-target/all-feature build, all-feature tests, and the focused prepared-handle unit test pass.

**2. Added narrow storage and error adapters required to remove duplicate authority**

- **Found during:** Task 1
- **Issue:** The existing Phase 09-02 storage had no public-world adapter for diagnostic identity, initial-versus-declared capacity, preflight, snapshots, group clearing, or single-particle compaction, and the handle vocabulary could not distinguish pending access.
- **Fix:** Added only the storage-owner operations needed by the lifecycle boundary plus `HandleError::PendingDelete`; no solver or Phase 10 behavior was introduced.
- **Files:** `crates/liquidfun/src/error.rs`, `crates/liquidfun/src/particle/storage.rs`, `crates/liquidfun/src/particle/storage/identity.rs`, `crates/liquidfun/src/particle/storage/properties.rs`
- **Verification:** Existing storage properties, object-model tests, particle-identity tests, and the complete all-feature suite pass.

## Issues Encountered

- The first warning-denied focused pass found one unnecessary mutable test binding. It was removed before restarting and passing the ordered commit gate.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later Phase 9 plans can attach body contacts, lifetime policy, and query behavior to one unambiguous system-owned particle state and stable identity source.
- Phase 10 solver work remains outside this plan; no particle contact generation, pair/triad topology generation, or solver behavior was added.
- The new lifecycle module is 480 lines and remains a cohesive deep boundary over private storage rather than fragmenting creation, lookup, and teardown into shallow modules.

## Self-Check: PASSED

- Both task commits are present and the summary's 11 implementation/test files exist.
- `cargo check -p liquidfun --all-features` passes.
- `cargo test -p liquidfun --test particle_objects` passes all 11 black-box tests.
- Existing `object_model` and `particle_identity` integrations pass.
- The ordered format, warning-denied Clippy, all-target/all-feature build, and all-feature test gates pass.
- The base-to-head source scan finds no world-level particle arena or parallel `self.particles` authority.
- `.planning/STATE.md` and `.planning/ROADMAP.md` are unchanged.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
