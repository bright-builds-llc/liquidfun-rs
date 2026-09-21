---
phase: 24-shared-hot-path-waves-through-3
plan: "01"
subsystem: particle-contacts
tags: [particle-rows, ParticleIndex, pair_rows, resolve_live, scalar]

requires:
  - phase: 23-baseline-pair-and-named-audit
    provides: named extra work particle_rows as algorithm/shape vs FindContacts Proxy.index
provides:
  - neighborhood Proxy.row ParticleIndex plus private aligned pair_rows
  - O(1) SoA indexing in generate/validate_pairs/listener_effects
  - maybe_live_row over resolve_live for previous public ParticleId contacts
affects:
  - 24-02 unprofiled Dam Break re-pair
  - leftover scalar waves after samply retarget

tech-stack:
  added: []
  patterns:
    - carry dense ParticleIndex on neighborhood Proxy; public handles stay ParticleId
    - previous contacts use existing resolve_live, not a HashMap cache
    - fail-closed MissingParticle on stale IDs and pair_rows length mismatch

key-files:
  created: []
  modified:
    - crates/liquidfun/src/particle/proxy.rs
    - crates/liquidfun/src/particle/contact.rs
    - crates/liquidfun/src/particle/view.rs
    - crates/liquidfun/src/particle/storage/lifecycle.rs
    - crates/liquidfun/tests/particle_contacts.rs

key-decisions:
  - "Carry dense ParticleIndex on neighborhood Proxy and a private pair_rows lane aligned with public ParticleNeighborPair ParticleId pairs."
  - "Previous public ParticleContact IDs go through maybe_live_row to resolve_live; do not store last-step rows on public contacts or add a HashMap cache."
  - "liquidfun stays scalar and safe: no rayon, std::simd, unsafe indexing, or workspace unsafe_code change."

patterns-established:
  - "Neighborhood candidates index positions/flags from carried rows; previous IDs use the generational identity map."
  - "pair_rows is pub(in crate::particle); rustdoc compile_fail locks the public API."

requirements-completed: [PERF-SHARED, PERF-BASELINE]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T15:46:11Z

duration: 15min
completed: 2026-09-21
---

# Phase 24 Plan 01: Index-Preserving Neighborhood Contacts Summary

**Shared neighborhood/contact generation now indexes SoA lanes in O(1) from carried `ParticleIndex` (candidates) or `resolve_live` (previous public IDs), matching C++ `FindContacts_Reference` `Proxy.index` in safe scalar Rust.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-09-21T15:31:07Z
- **Completed:** 2026-09-21T15:46:11Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Restored the C++ index-preserving shape: each neighborhood `Proxy` keeps the dense SoA row that produced it, and `ParticleNeighborhood` stores a private `pair_rows` lane aligned with public `ParticleNeighborPair` IDs.
- Deleted `particle_rows` linear `particle_ids().iter().position` scans from `generate`, `validate_pairs`, and `listener_effects`.
- Kept public `ParticleContact` / `ParticleNeighborPair` on `ParticleId` only; `ParticleIndex` is not exported from `lib.rs` or `particle.rs`.
- Proved destroyed previous contact IDs still return `ParticleContactError::MissingParticle` via `maybe_live_row` → `resolve_live`.

## Task Commits

Each task was committed atomically:

1. **Task 1: RED — failing tests for carried rows and MissingParticle** - `b6c9367` (test)
2. **Task 2: GREEN — carry ParticleIndex and delete particle_rows scans** - `b704314` (feat)

**Plan metadata:** pending `docs(24-01): complete index-preserving neighborhood contacts plan`

_Note: TDD tasks used RED then GREEN commits._

## Files Created/Modified

- `crates/liquidfun/src/particle/proxy.rs` — `Proxy.row`, private `pair_rows`, crate-internal `pair_rows()`, compile_fail lock, row-carry unit test
- `crates/liquidfun/src/particle/contact.rs` — zip `pairs()` with `pair_rows`; previous IDs via `maybe_live_row`; no `particle_rows`
- `crates/liquidfun/src/particle/view.rs` — `maybe_live_row` over `resolve_live`
- `crates/liquidfun/src/particle/storage/lifecycle.rs` — `resolve_live` visibility `pub(in crate::particle)`
- `crates/liquidfun/tests/particle_contacts.rs` — MissingParticle after `destroy_particle`

## Decisions Made

- Carry dense `ParticleIndex` on neighborhood `Proxy` and a private `pair_rows` lane aligned with public `ParticleNeighborPair` ParticleId pairs.
- Previous public `ParticleContact` IDs go through `maybe_live_row` to `resolve_live`; do not store last-step rows on public contacts or add a HashMap cache.
- `liquidfun` stays scalar and safe: no rayon, `std::simd`, unsafe indexing, or workspace `unsafe_code` change.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Bound must-use particle creation receipt in the unit test**
- **Found during:** Task 2 (GREEN compile of `particle::proxy::row_carry`)
- **Issue:** `create_particle_with_def` returns `ParticleCreationReceipt`; `-D unused-must-use` rejected dropping it, then rejected unused `created_particle()`.
- **Fix:** Bind with `let _ = ...created_particle();` so destruction_occurrences stay acknowledged without changing Arrange particles.
- **Files modified:** `crates/liquidfun/src/particle/proxy.rs`
- **Verification:** `cargo test -p liquidfun --lib particle::proxy::` passed
- **Committed in:** `b704314` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Needed for the crate-internal RED test to compile under warning-deny; no production behavior change.

## Issues Encountered

None beyond the must-use receipt in the new unit test.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 24-02 unprofiled Dam Break re-pair into a new exclusive stamp. This plan does not claim PERF-GATE; leftover named frames remain possible after samply retarget. Independent AI review remains a later phase step — this implementing agent does not self-approve.

---
*Phase: 24-shared-hot-path-waves-through-3*
*Completed: 2026-09-21*

## Self-Check: PASSED
