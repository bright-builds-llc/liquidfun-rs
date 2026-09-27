---
phase: 29-sparky-drawing-and-full-catalog
plan: "02"
subsystem: wasm-scenes
tags: [particles, drawing, elastic, wasm]

requires:
  - phase: 28-interaction-seams
    provides: Session pointer routing and World::destroy_particles_in_shape
provides:
  - Empty Drawing Particles vessel with water and elastic pointer paint
affects: [29-sparky-drawing-and-full-catalog]

tech-stack:
  added: []
  patterns:
    - "Brush samples destroy inside a circle, then create or append a particle group"
    - "Strokes join only while the live group's flags still match the current material"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/drawing_particles.rs
    - crates/liquidfun-wasm/src/scene/drawing_particles/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Material is live-only: water has empty group flags, elastic is ELASTIC plus SOLID, and construction presets are rejected."
  - "Destruction-by-age is set false because the particle-system default is on."
  - "A create that hits the 10240 cap returns success after the destroy and does not keep a stale join."

patterns-established:
  - "Pointer down and move stamp; pointer up clears the join; an up with no open stroke leaves one stamp and then clears."

requirements-completed: [FX-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T06:19:47Z

duration: 52min
completed: 2026-09-27
---

# Phase 29 Plan 02: Drawing Particles Paint Summary

**Empty Drawing Particles vessel with destroy-then-create water and elastic paint that joins only while group flags still match**

## Performance

- **Duration:** 52 min
- **Started:** 2026-09-27T05:27:26Z
- **Completed:** 2026-09-27T06:19:47Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `drawing-particles` constructs with zero particles inside the four-wall vessel from `DrawingParticles.h`.
- Pointer down and move erase a 0.2 circle, then paint. Water flows with empty group flags. Elastic uses `ELASTIC` particles in a `SOLID` group and does not recreate the world.
- Pointer up clears the join. An unpaired pointer up leaves one stamp and then clears. A brush that empties the joined group drops that id before the next sample.

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing Drawing allowlist and paint tests** - `a3ccdb6` (test)
2. **Task 2: Implement empty vessel and water or elastic paint** - `5ce0343` (feat)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/drawing_particles.rs` - Empty vessel, water and elastic brush, join and clear
- `crates/liquidfun-wasm/src/scene/drawing_particles/tests.rs` - Parse, vessel, join, elastic, erase, and cap tests
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::DrawingParticles` parse and build arm
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` - Eighteen-id allowlist ending in Drawing Particles
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist token for `drawing-particles`
- `crates/liquidfun-wasm/src/session.rs` - Empty frames succeed before any colored particle allocates a color lane

## Decisions Made

- Material stays on the hooks. `apply_control("material", ...)` returns live, and a non-empty construction preset is `UnknownControl`, so the material is not stored in the session preset bag.
- Particle-system destruction-by-age defaults on, so this scene sets it false. The 10240 maximum stays in place. A failed create after the brush destroy is a successful no-op.
- The module sentence wraps `LiquidFun` in backticks so the doc lint accepts the required wording.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Empty vessel frames need no color lane**
- **Found during:** Task 2 (Implement empty vessel and water or elastic paint)
- **Issue:** `capture_frame` required a color lane. A new particle system allocates that lane only after the first colored particle, so the empty vessel could not export its walls.
- **Fix:** When the system has no particles and no color lane, capture uses an empty color slice and still copies the rigid segments.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`
- **Verification:** `construction_starts_empty_inside_the_vessel_walls` passes, including the four opening edges.
- **Committed in:** `5ce0343` (Task 2 commit)

**2. [Rule 2 - Missing Critical] Gravity coverage array grew with the new scene id**
- **Found during:** Task 1 (Failing Drawing allowlist and paint tests)
- **Issue:** `all_scene_ids` became 18 ids, but the existing gravity test still marked only 17 slots. Index 17 would panic once Drawing Particles built.
- **Fix:** The seen array and the "every variant" assertion are now length 18.
- **Files modified:** `crates/liquidfun-wasm/src/scene/gravity_slider.rs`
- **Verification:** `every_scene_builds_downward_gravity_from_the_slider` passes.
- **Committed in:** `a3ccdb6` (Task 1 commit)

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both keep the empty vessel visible and the existing gravity coverage test valid. No Sparky id, no web edits, and `MAX_ADVANCE_STEPS` stays 4.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 29-03. Drawing Particles is a native scene id with water and elastic paint. Catalog, portrait, and Sparky wiring are still ahead.

## Self-Check: PASSED

## Known Stubs

None.

---
*Phase: 29-sparky-drawing-and-full-catalog*
*Completed: 2026-09-27*
