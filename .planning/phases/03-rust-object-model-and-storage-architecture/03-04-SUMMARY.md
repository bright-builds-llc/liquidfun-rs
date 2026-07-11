---
phase: 03-rust-object-model-and-storage-architecture
plan: "04"
subsystem: particle-storage
tags: [rust, stable-identity, soa, transactional-permutation, proptest, owned-buffers]
requires:
  - phase: 03-rust-object-model-and-storage-architecture
    plan: "01"
    provides: Opaque world-scoped handles and checked generational identity
  - phase: 03-rust-object-model-and-storage-architecture
    plan: "02"
    provides: World-owned particle systems, destruction records, and typed associations
provides:
  - Stable particle identity separated from private ephemeral dense positions
  - One validate-then-commit permutation for representative SoA lanes and derived indices
  - Explicit live, pending-delete, vacant, and retired identity lifecycles
  - Bounded model-based state-machine evidence and private owned-buffer capacity semantics
affects: [03-05, phase-9-particles, particle-identity, particle-groups, external-buffers]
tech-stack:
  added: [proptest 1.11.0]
  patterns: [stable-id-to-dense-map, transactional-permutation, owned-lane-bundle, bounded-model-testing]
key-files:
  created:
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/identity.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/particle/storage/properties.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/liquidfun/Cargo.toml
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "ParticleId remains stable and public while every dense ParticleIndex stays private, ephemeral, and particle-system validated before lookup."
  - "One authoritative validate-then-commit mapping updates all representative lanes, identity entries, proxies, contacts, pairs, triads, lifetime order, and group ranges."
  - "Pending deletion preserves an owned row snapshot and rejects mutation; compaction advances or retires identity generations and makes removed IDs stale."
  - "Future external-buffer behavior is represented only by a private owned lane bundle whose declared capacity controls growth and whose buffers return on teardown."
requirements-completed: [API-01, API-02, API-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-07-11T01-23-59
generated_at: 2026-07-11T03:12:06Z
duration: 7 min
completed: 2026-07-11
---

# Phase 3 Plan 04: Dense Particle Identity and Permutation Spike Summary

**A bounded private particle-storage spike now proves stable identity across dense SoA movement, transactional derived-index remapping, explicit deletion lifecycle, and fixed owned-buffer capacity semantics.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-07-11T03:05:27Z
- **Completed:** 2026-07-11T03:12:06Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Added a private group-contiguous representative SoA whose stable `ParticleId` values remain separate from private dense positions and reject wrong-world, cross-system, pending-delete, stale, capacity, and generation-exhaustion access explicitly.
- Centralized reorder and compaction behind one validate-then-commit transaction that remaps required and optional lanes, both identity directions, proxies, contacts, pairs, triads, deterministic lifetime order, and contiguous group ranges.
- Added 128-case bounded `proptest` state-machine coverage against an independent semantic model, with readable operation-prefix diagnostics and baseline coverage for every planned operation class.
- Added a private owned-lane-bundle constructor and teardown that distinguish declared fixed capacity from spare allocation capacity without exposing raw pointers, solver passes, or Phase-9 bulk APIs.

## Task Commits

Each task was committed atomically after its focused checks and the complete Rust gate:

1. **Task 1: Implement stable particle identity and lifecycle state mapping** - `419db6a` (feat)
2. **Task 2: Centralize transactional lane permutation and derived remapping** - `73514ff` (feat)
3. **Task 3: Property-test storage state machines and lock future buffer semantics** - `721cb87` (test)

## Files Created/Modified

- `crates/liquidfun/src/particle.rs` - Private particle architecture boundary; no solver or bulk public API.
- `crates/liquidfun/src/particle/storage.rs` - Stable identity lifecycle, representative lanes, authoritative transaction, invariant checker, and owned-lane bundle.
- `crates/liquidfun/src/particle/storage/identity.rs` - Focused stable, cross-scope, pending, stale, capacity, and retirement tests.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Focused all-lane remapping and no-partial-commit tests.
- `crates/liquidfun/src/particle/storage/properties.rs` - Bounded independent-model state machine plus owned-buffer construction and teardown evidence.
- `Cargo.toml`, `crates/liquidfun/Cargo.toml`, and `Cargo.lock` - Reviewed centralized `proptest = "1.11.0"` dev dependency and Cargo-generated resolution.
- `crates/liquidfun/src/lib.rs` - Private particle module wiring; public exports remain limited to the already established stable `ParticleId`.

## Decisions Made

- Particle-system scope is checked before the dense map is consulted, and the spike uses disjoint private identity-slot ranges supplied by the owning world architecture.
- Removed-row derived references are discarded deterministically during compaction; out-of-range references fail validation before any authoritative storage changes.
- Once an optional lane is materialized, earlier or absent row values receive deterministic defaults so all present lanes remain exactly aligned.
- Solver-visible traversal uses vectors and explicit order only; no hash iteration participates in storage transactions.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first optional-lane rotation assertion treated a row-level absent value as if the SoA lane itself were absent. The test was corrected to reflect the invariant that a present optional lane stores deterministic defaults for every row.
- Strict Clippy initially flagged range-index loops and `&Option<T>` helper signatures in Task 1; both were simplified before the task commit and the complete gate passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 03-05 can document and integrate the object/storage boundary with executable particle permutation evidence now available.
- Full particle solver passes, public bulk mutation, and external-buffer APIs remain deliberately deferred to Phase 9.

## Self-Check: PASSED

- Task commits `419db6a`, `73514ff`, and `721cb87` exist in history.
- All five created particle source/test files and four modified integration/dependency files exist.
- Focused identity, permutation, invalid-transaction, capacity, teardown, and 128-case state-machine tests pass.
- The exact full Rust gate passes in required order with no unsafe code, `unwrap()`, raw pointers, solver-visible hash iteration, solver passes, or new public Phase-9 API.

***

_Phase: 03-rust-object-model-and-storage-architecture_
_Completed: 2026-07-11_
