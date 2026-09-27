---
phase: 31-sinusoidal-wave-tank
plan: "01"
subsystem: playground-scene
tags: [prismatic, particles, wasm, wave-tank]

requires:
  - phase: 30-periodic-hydraulic-fountain
    provides: A dynamic prismatic motor rewritten from simulation time inside on_advance
provides:
  - SceneId::WaveTank builds one still water group and a dynamic end platform on a vertical prismatic joint
  - After 60 SessionCore::advance steps an original far-wall particle has risen by one diameter and the live count is unchanged
affects: [31-02-catalog, 31-03-browser-proof]

tech-stack:
  added: []
  patterns:
    - "Prismatic motor speed is PEAK_SPEED * sin(TAU * elapsed / PERIOD), written in on_advance before World::step"
    - "Watch-first scene hooks reject period, amplitude, and stroke names"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/wave_tank.rs
    - crates/liquidfun-wasm/src/scene/wave_tank/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Kept the plan starting numbers: stroke 0.16, period 2.0, channel 0.90, depth 0.32, damping 0.2, radius 0.025, max force 1.0e6"
  - "Passing native tests are evidence only. This implementing agent did not approve the work."

patterns-established:
  - "Wave tank drives a dynamic vertical platform with set_prismatic_motor_speed and does not edit wave_machine.rs"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-09-27T17-22-43
generated_at: 2026-09-27T18:21:09Z

duration: 13min
completed: 2026-09-27
---

# Phase 31 Plan 01: Native Wave Tank Summary

**A dynamic end platform on a vertical prismatic sine motor lifts one end of a still pool, and an original far-wall particle rises by one diameter after 60 steps**

## Performance

- **Duration:** 13 min
- **Started:** 2026-09-27T18:08:10Z
- **Completed:** 2026-09-27T18:21:09Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `wave-tank` builds one default water group and one dynamic platform on a vertical prismatic joint with limits outside the sine.
- `on_advance` adds `1/60` and writes `PEAK_SPEED * sin(TAU * elapsed / PERIOD)`, with peak speed `stroke * TAU / (2 * period)`.
- After fifteen `SessionCore::advance(4)` calls the platform translation is at least half the stroke, the live count is unchanged, and at least one particle that started at `x >= 1.20` has moved up by `0.05`.
- `wave_machine.rs`, `hydraulic_fountain.rs`, and `MAX_ADVANCE_STEPS` (still 4) are unchanged.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the dynamic end-platform scene** - `1700169` (test) and `5bd0eae` (feat)
2. **Task 2: Prove the still pool and the far-wall rise** - `de0e62e` (test)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/wave_tank.rs` - Scene build, vertical prismatic motor, and sine speed in `on_advance`
- `crates/liquidfun-wasm/src/scene/wave_tank/tests.rs` - Still-band, far-wall rise, sine speed, rebuild, and source tests
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::WaveTank`, parse token `wave-tank`, and `build_scene` route
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` - Variant index 20 and a 21-entry scene list
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist pair for `wave-tank`

## Decisions Made

- Kept the plan starting numbers. `far_wall_particles_rise_after_a_half_cycle` passed at stroke `0.16`, period `2.0`, channel `0.90`, depth `0.32`, damping `0.2`, radius `0.025`, and max force `1.0e6`, so those values were not retuned.
- Passing native tests are evidence only. Per D-11 this implementing agent did not approve the work.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Grew the gravity-slider seen buffers with the new scene id**
- **Found during:** Task 1 (Build the dynamic end-platform scene)
- **Issue:** `every_scene_builds_downward_gravity_from_the_slider` stored coverage in `[false; 20]`. Index 20 for `SceneId::WaveTank` would panic before the scene list could be proven complete.
- **Fix:** Grew both seen buffers to 21 beside `all_scene_ids() -> [SceneId; 21]` and `SceneId::WaveTank => 20`.
- **Files modified:** `crates/liquidfun-wasm/src/scene/gravity_slider.rs`
- **Verification:** `cargo test -p liquidfun-wasm scene::gravity_slider -- --test-threads=1` passed
- **Committed in:** `1700169`

Task 2's far-wall test passed on the first run because Task 1 already built the platform, the water group, and the sine at the planned starting numbers. There was no retune commit.

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The extra buffer growth is the same scene-list bookkeeping the plan already required for index 20. No geometry, engine API, or step-cap change.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 31-02. The native scene id parses and builds. Catalog, portrait frame, and README plan entries are still absent, which is Plan 02's job.

This summary records evidence and does not approve the work.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/wave_tank.rs
- FOUND: crates/liquidfun-wasm/src/scene/wave_tank/tests.rs
- FOUND: 1700169
- FOUND: 5bd0eae
- FOUND: de0e62e
