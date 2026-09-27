---
phase: 30-periodic-hydraulic-fountain
plan: "01"
subsystem: playground-scene
tags: [prismatic, particles, wasm, hydraulic-fountain]

requires:
  - phase: 28-interaction-seams
    provides: Live joint motor writes from simulation time, and a prismatic joint with collide-connected rails
provides:
  - SceneId::HydraulicFountain builds a dynamic limited piston and one default water group
  - A SessionCore test shows an original particle id on the fountain side after 120 steps with the live count unchanged
affects: [30-02-catalog, 30-03-browser-proof]

tech-stack:
  added: []
  patterns:
    - "Prismatic motor speed is a square wave rewritten in on_advance before World::step"
    - "Watch-first scene hooks reject period, stroke, and aim names"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs
    - crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Kept the plan starting numbers: speed 0.6, stroke 0.35, period 2.0, max force 1.0e6, throat below y = 0.12"
  - "Moved scene tests to hydraulic_fountain/tests.rs so the scene module stays under 500 lines"

patterns-established:
  - "Hydraulic fountain drives a dynamic piston with set_prismatic_motor_speed and does not edit fountain.rs"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 30-2026-09-27T14-01-51
generated_at: 2026-09-27T15:19:15Z

duration: 41min
completed: 2026-09-27
---

# Phase 30 Plan 01: Native Hydraulic Piston Summary

**A dynamic piston on a limited prismatic motor squeezes one water group through a floor throat, and an original particle id is on the fountain side after 120 steps**

## Performance

- **Duration:** 41 min
- **Started:** 2026-09-27T14:37:36Z
- **Completed:** 2026-09-27T15:19:15Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `hydraulic-fountain` builds a dynamic piston on a limited prismatic joint and one default water group in the piston chamber.
- `on_advance` adds `1/60` and writes motor speed `0.6` for the first half of the 2 second period and `-0.6` for the second half.
- After 120 `SessionCore::advance` steps the live count is unchanged and at least one starting `ParticleId` is past `x = 0.04`. A rebuilt session starts at prismatic translation `0` with every particle on the piston side.
- `crates/liquidfun-wasm/src/scene/fountain.rs` is unchanged and `MAX_ADVANCE_STEPS` stays 4.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the dynamic piston scene** - `d4fb07d` (test) and `0aac3ad` (feat)
2. **Task 2: Prove the original particles cross within one period** - `c1ffbc6` (test)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` - Scene build, wall boxes, water group, and half-period motor schedule
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs` - Layout, motor schedule, crossing, rebuild, and watch-first hook tests
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::HydraulicFountain`, parse token, and `build_scene` route
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` - Variant index 19 and a 20-entry scene list
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist pair for `hydraulic-fountain`

## Decisions Made

- Kept the plan's starting numbers. The crossing test passed without changing stroke, speed, period, throat, max force, fill box, or piston start x.
- Split tests into `hydraulic_fountain/tests.rs` once the module plus tests would pass 500 physical lines. The scene implementation stays in the single credit-path file.

## Deviations from Plan

### Auto-fixed Issues

None.

Task 2's crossing tests passed on the first run because Task 1 already built the piston, the water group, and the motor schedule at the planned starting numbers. There was no failing-test-only implementation gap and no retune commit.

**Total deviations:** 0 auto-fixed
**Impact on plan:** The recommended starting geometry already moves the original particles through the throat within one period.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 30-02. The native scene id parses and builds. Catalog, portrait frame, and README plan entries are still absent, which is Plan 02's job.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs
- FOUND: crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs
- FOUND: d4fb07d
- FOUND: 0aac3ad
- FOUND: c1ffbc6

---
*Phase: 30-periodic-hydraulic-fountain*
*Completed: 2026-09-27*
