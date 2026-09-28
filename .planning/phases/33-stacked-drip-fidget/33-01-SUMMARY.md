---
phase: 33-stacked-drip-fidget
plan: 01
subsystem: wasm-scenes
tags: [liquidfun, particles, revolute, prismatic, wasm]

requires:
  - phase: 32-liquid-motion-bubbler
    provides: Side-shaft prismatic plate schedule and motor-off revolute pattern
provides:
  - stacked-drip scene with three limited motor-off trays and one teal water group
  - Cascade proof that the starting cohort finishes below the bottom tray in top-to-bottom order while the plate stays down
affects: [33-02 catalog, 33-03 playground framing, 33-04 return stroke]

tech-stack:
  added: []
  patterns:
    - "Tray pours are limited revolute joints with the motor left off"
    - "Plate speed is written from elapsed time in on_advance before World::step"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/stacked_drip.rs
    - crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs
    - crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Counterweight density is 1.05 so a loaded tray stays poured through the 5 second proof and an empty tray still rests on the lower limit."
  - "Angular damping is 20 so that poured pose has not crept back under 0.05 rad by the proof sample."
  - "This summary records evidence and does not approve the work."

patterns-established:
  - "Stacked Drip trays pour toward negative local x, away from the right-hand shaft."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-09-27T23-41-40
generated_at: 2026-09-28T00:32:07Z

duration: 21min
completed: 2026-09-28
---

# Phase 33 Plan 01: Stacked Drip Scene Summary

**Native stacked-drip scene: three motor-off limited trays, one teal water group, and a side-shaft plate that stays down while that cohort drains below the bottom tray**

## Performance

- **Duration:** 21 min
- **Started:** 2026-09-28T00:11:30Z
- **Completed:** 2026-09-28T00:32:07Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `stacked-drip` builds three dynamic trays on revolute joints limited from 0 to tau/8, with the motors left off, plus one teal water group above the top tray.
- The side-shaft plate uses a world-up prismatic joint. During the 6 second dwell its commanded speed is 0.
- After 75 calls of `advance(4)`, the starting particle count is unchanged, every original id that began above the top tray is below the bottom tray and left of the divider, and the upper tray moves before the middle tray, which moves before the lower tray.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build three limited trays and a delayed side-shaft plate** - `16f193e` (feat)
2. **Task 2: Prove the cohort drains top to bottom while the plate stays down** - `2efe891` (feat)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/stacked_drip.rs` - Scene build, dwell schedule, and hooks
- `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs` - Static walls and tray fixtures
- `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs` - Cascade, order, plate, and rebuild proofs
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::StackedDrip`, token `stacked-drip`, and `build_scene`
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` - Variant index 22 and a 23-scene list
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist pair for `stacked-drip`

## Decisions Made

- Counterweight density is **1.05**, not the starting 8. Density 8 held the empty trays, and the loaded trays returned to the lower limit before the 5 second sample. 1.05 still rests an empty tray on the lower limit and leaves a loaded tray poured through the proof.
- Angular damping is **20**, not the starting 0.4. With the lighter counterweight, 0.4 and 4 both let the trays creep back under 0.05 rad by the sample. Damping 20 keeps each tray at least 0.05 rad from rest at that sample.
- Dwell stays **6 seconds**, the proof stays **5 seconds** (75 batches of `advance(4)`), plate speed stays **0.15**, and particle radius stays **0.025**. `MAX_ADVANCE_STEPS` stays 4.
- This summary records that evidence. It does not approve the work.

## Deviations from Plan

None - plan executed exactly as written. The density and damping changes are the retune the plan allows when a named cascade test fails.

## Issues Encountered

The first build used counterweight density 8 and angular damping 0.4. Liquid reached the floor, but the trays had returned to the lower limit before 5 seconds, so the final-angle check failed. Density 1.05 and damping 20 keep the pour past 0.05 rad at that same sample while the cohort is already below the bottom tray. No particles were destroyed or respawned, and `MAX_ADVANCE_STEPS` was not raised.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 33-02. The scene id exists. Catalog, portrait bounds, and README SVG plans are still later plans.
- Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, and Liquid Tumbler were not modified.
- Independent review is still required. This summary does not approve the implementation.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/stacked_drip.rs
- FOUND: crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs
- FOUND: crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs
- FOUND: 16f193e
- FOUND: 2efe891

## Test commands

- `cargo test -p liquidfun-wasm --lib scene::stacked_drip -- --test-threads=1`
- `cargo test -p liquidfun-wasm --lib cohort_passes_below_the_bottom_tray -- --test-threads=1`
- `cargo test -p liquidfun-wasm --lib upper_tray_moves_before_the_lower_trays -- --test-threads=1`
- `cargo test -p liquidfun-wasm --lib plate_stays_down_during_the_cascade -- --test-threads=1`
- `cargo test -p liquidfun-wasm --lib parse_scene_id_maps_allowlisted_tokens -- --test-threads=1`
- `cargo test -p liquidfun-wasm scene::gravity_slider -- --test-threads=1`

---

*Phase: 33-stacked-drip-fidget*
*Completed: 2026-09-28*
