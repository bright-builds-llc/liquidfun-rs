---
phase: 19-interaction-polish-and-browser-verification
plan: "03"
subsystem: pointer-input
tags: [wasm, pointer-action, color-mixer, jelly-drop, water-wheel, tdd, rust]

requires:
  - phase: 19-interaction-polish-and-browser-verification
    provides: PointerKind parse and SceneHooks::apply_pointer from 19-02
provides:
  - Color Mixer pointer stir via Rust-internal per-particle forces
  - Jelly Drop poke at the click with per-id impulses
  - Water Wheel jet aim override from the fixed nozzle
affects: [19-04, wasm-pointer-action, canvas-pointer]

tech-stack:
  added: []
  patterns:
    - localized nearby-id forces use apply_particle_force / apply_particle_linear_impulse inside Rust
    - range APIs stay on contiguous labeled poke and full-system stir
    - cancel clears Color Mixer maybe_pointer and Water Wheel maybe_aim_velocity

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/color_mixer.rs
    - crates/liquidfun-wasm/src/scene/jelly_drop.rs
    - crates/liquidfun-wasm/src/scene/water_wheel.rs

key-decisions:
  - "Localized stir and poke loop per-particle engine methods; never pass scattered nearby ids to *_range APIs."
  - "Color Mixer Up/Cancel clear maybe_pointer so leftover tangent force cannot persist."
  - "Water Wheel pointer steers jet direction from JET_POSITION; Cancel restores (speed, -0.4); motor stays off."

patterns-established:
  - "Nearby particle selection copies positions and ids, then applies one engine call per matching id."
  - "Jelly pointer-down is one-shot; labeled poke-jelly keeps the first-third contiguous range."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:14:39Z

duration: 4min
completed: 2026-09-19
---

# Phase 19 Plan 03: Localized Stir, Poke, and Steered Jet Summary

**Color Mixer drag stirs nearby particles with per-id tangent forces, Jelly Drop pokes at the click, and Water Wheel aims the jet from the fixed nozzle, all inside Rust with cancel-cleared transients.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-09-19T02:10:30Z
- **Completed:** 2026-09-19T02:14:39Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Color Mixer stores `maybe_pointer` on Down/Move, applies `POINTER_STIR_FORCE` tangents through `apply_particle_force` per nearby id, and clears the point on Up/Cancel.
- Labeled `stir-speed` still applies global `(8, 0)` / `(18, 0)` through `apply_particle_force_range` on the full id list.
- Jelly pointer-down applies `POKE_IMPULSE` to group members within radius `1.0`; labeled `poke-jelly` still uses the first-third contiguous range.
- Water Wheel Down/Move aims from `JET_POSITION (-3.9, 3.15)` scaled by current jet speed; Cancel restores `Vec2::new(speed, -0.4)`; the revolute motor stays off.

## Task Commits

Each task was committed atomically:

1. **Task 1: Stir Color Mixer at a stored world point**
   - `a15caa7` (test): failing pointer-stir and source-scan tests
   - `163d6f1` (feat): `maybe_pointer` plus per-id tangent forces
2. **Task 2: Poke Jelly at the click and steer the Water Wheel jet**
   - `58de0db` (test): failing poke, aim, and cancel-restore tests
   - `3bd1cf7` (feat): per-id poke and `maybe_aim_velocity` jet override

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/color_mixer.rs` — pointer stir via Rust-internal per-particle forces
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` — pointer poke at world location
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — pointer jet aim override

## Decisions Made

- Use per-particle `apply_particle_force` / `apply_particle_linear_impulse` for scattered nearby ids; keep range APIs on contiguous labeled poke and full-system stir.
- Clear Color Mixer `maybe_pointer` on Up and Cancel so leftover local force cannot persist after the gesture ends.
- Steer the Water Wheel jet from the fixed nozzle toward the pointer; labeled Jet strength still sets speed; do not enable the revolute motor.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Jelly poke test compared gravity motion, not the poke**
- **Found during:** Task 2 RED
- **Issue:** After four steps, jelly particles already move under gravity, so a before/after position check passed on the no-op stub.
- **Fix:** Compare a pointer-down session against a no-pointer control world after the same four steps.
- **Files modified:** `crates/liquidfun-wasm/src/scene/jelly_drop.rs`
- **Commit:** `58de0db`

**2. [Rule 3 - Blocking] Motor-speed rg still matches the existing motor-off test**
- **Found during:** Task 2
- **Issue:** Plan acceptance asked `rg enable_motor|motor_speed` to be empty, but the pre-existing revolute test already reads `motor_speed()`.
- **Fix:** Added no motor writes. The implementation scan rejects `enable_motor` and `.motor_speed` assignments; the existing motor-off assertion remains.
- **Files modified:** none beyond the planned Water Wheel aim
- **Commit:** `3bd1cf7`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both keep the planned physics contract. WEB-05 stays pending until later plans wire canvas capture.

## Known Stubs

None — the 19-02 compile-only `apply_pointer` stubs in these three files are replaced with native mappings. Jelly Move/Up/Cancel and Water Wheel Up remain intentional no-ops.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 19-04 canvas Pointer Events pipeline. All six scene modules now handle the four pointer kinds with native physics. WEB-05 remains pending until canvas capture, instruction copy, and browser evidence land.

---

*Phase: 19-interaction-polish-and-browser-verification*
*Completed: 2026-09-19*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/color_mixer.rs
- FOUND: crates/liquidfun-wasm/src/scene/jelly_drop.rs
- FOUND: crates/liquidfun-wasm/src/scene/water_wheel.rs
- FOUND: a15caa7
- FOUND: 163d6f1
- FOUND: 58de0db
- FOUND: 3bd1cf7
