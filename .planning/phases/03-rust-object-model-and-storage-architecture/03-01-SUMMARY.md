---
phase: 03-rust-object-model-and-storage-architecture
plan: "01"
subsystem: object-model
tags: [rust, typed-handles, generational-arena, identity, property-testing]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Engine-neutral semantic identities isolated from implementation handles
provides:
  - Six distinct opaque world-scoped public handle types
  - Checked process-unique world-key allocation and typed identity failures
  - Deterministic bounded generational arena with permanent slot retirement
  - Seeded model-based coverage for reuse, stale access, and cross-world rejection
affects: [03-02, 03-03, 03-04, 03-05, world-storage, particle-identity]

tech-stack:
  added: []
  patterns: [opaque typed handles, checked generations, retired slots, deterministic free list]

key-files:
  created:
    - crates/liquidfun/src/identity.rs
    - crates/liquidfun/src/error.rs
    - crates/liquidfun/src/arena.rs
  modified:
    - crates/liquidfun/src/lib.rs

key-decisions:
  - "Complete handle identity is a private world key, slot, and u64 generation; equality and hashing cover all three."
  - "Arena reuse is deterministic LIFO while public iteration is explicit ascending slot order."
  - "A generation that cannot advance permanently retires its slot and can never wrap."

patterns-established:
  - "Typed identity: each object kind is a distinct opaque newtype with no raw constructor or stable layout promise."
  - "Lookup validation: internal kind and world checks precede slot and generation resolution."
  - "Arena retirement: removal invalidates first, then either advances and frees or permanently retires."

requirements-completed: [API-01, API-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-07-11T01-23-59
generated_at: 2026-07-11T02:27:06Z

duration: 8 min
completed: 2026-07-11
---

# Phase 3 Plan 01: Typed Identity and Generational Arena Foundation Summary

**Six opaque world-scoped handles backed by checked generational identity and a deterministic arena that permanently retires exhausted slots**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-11T02:18:59Z
- **Completed:** 2026-07-11T02:27:06Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added distinct `BodyId`, `FixtureId`, `JointId`, `ParticleSystemId`, `ParticleGroupId`, and stable `ParticleId` types with complete private identity equality and hashing.
- Added checked process-unique world keys plus explicit wrong-world, stale/destroyed, internal wrong-kind, capacity, generation, and world-key exhaustion errors.
- Added a deterministic bounded arena whose stale handles never resolve after reuse and whose `u64::MAX` generations retire permanently.
- Added focused unit coverage and 128 seeded bounded model-based operation sequences that retain stale handles and probe cross-world access with failure seed and operation-prefix diagnostics.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define opaque typed world identities and errors** - `d0baf7c` (feat)
1. **Task 2: Implement checked generational arena reuse and retirement** - `75cc198` (feat)

## Files Created/Modified

- `crates/liquidfun/src/identity.rs` - Private world/slot/generation identity and six curated public handle types.
- `crates/liquidfun/src/error.rs` - Typed lookup, arena insertion, and world-key exhaustion failures.
- `crates/liquidfun/src/arena.rs` - Private deterministic generational storage, retirement behavior, and model-based tests.
- `crates/liquidfun/src/lib.rs` - Private module wiring and curated public identity/error re-exports.

## Decisions Made

- Used a checked monotonic process-local `u64` world key; exhaustion fails rather than wrapping.
- Kept slot coordinates private and architecture-sized because handles have no serialization or layout contract.
- Used LIFO vacant-slot reuse for simple deterministic allocation and ascending-slot iteration for an explicit hash-independent traversal order.
- Distinguished a full live arena from a capacity made unusable by retired generations so insertion reports `CapacityExceeded` or `GenerationExhausted` precisely.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later Phase 3 plans can build destruction cascades, commands, associations, and stable particle mapping on a checked world-scoped identity substrate.
- The private arena intentionally remains unused by a public `World` until the next storage plan establishes the owning facade.

## Self-Check: PASSED

- Task commits `d0baf7c` and `75cc198` exist in history.
- All three created source files and the modified crate root exist.
- Focused identity and arena tests pass, including seeded operation-prefix diagnostics.
- The exact full Rust gate passes in required order with `#![forbid(unsafe_code)]` intact and no forbidden identity escape hatch.

***

_Phase: 03-rust-object-model-and-storage-architecture_
_Completed: 2026-07-11_
