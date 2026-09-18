---
phase: 18-six-native-physics-demos
plan: "06"
subsystem: wasm-scene
tags: [liquidfun-wasm, dam-break, fountain, tdd, bounded-emission]

requires:
  - phase: 18-six-native-physics-demos
    provides: SceneId factory, Dam Break Medium/Normal basin, Recreated preset bag, Water Wheel lifetime/capacity emit pattern
provides:
  - Evolved Dam Break with water-amount and gravity construction presets
  - Live drop-obstacle and reset-obstacle on the existing circle
  - Bounded Fountain stream that plateaus at ≤ 320 particles
affects: [18-07, 18-09, catalog-ready, player-controls]

tech-stack:
  added: []
  patterns:
    - construction water/gravity presets return ControlEffect::Recreated
    - obstacle drop/reset uses set_body_transform plus a wake impulse
    - fountain emit is live with with_lifetime + with_maximum_count + destroy_by_age

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/dam_break.rs
    - crates/liquidfun-wasm/src/scene/fountain.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Keep Dam Break Medium 16×12=192 and Normal gravity (0,-10) as the documented default; Small is 8×8=64 and Large is 20×14=280."
  - "Drop-obstacle raises the existing circle to (2.5, 7.2) and wakes it; reset-obstacle restores (2.5, 5.5); neither recreates the world."
  - "Fountain defaults to Medium/Up/Medium, emits in on_advance with lifetime 3s and maximum_count 320, and treats a full system as a no-op."

patterns-established:
  - "Dam Break construction names recreate through the Plan 01 bag; obstacle actions stay live on the same body id."
  - "Fountain/jet scenes seed one particle so capture has a color buffer before the first emit."

requirements-completed: [DEMO-01, DEMO-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T05:10:11Z

duration: 12min
completed: 2026-09-18
---

# Phase 18 Plan 06: Dam Break Controls and Bounded Fountain Summary

**Dam Break gained locked water-amount/gravity recreates plus drop/reset on the existing circle, and Fountain replaced its stub with a TAU-aimed stream that plateaus at ≤ 320.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-09-18T04:58:38Z
- **Completed:** 2026-09-18T05:10:11Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `apply_control("water-amount", "small"|"medium"|"large")` and `apply_control("gravity", "low"|"normal"|"high")` return `ControlEffect::Recreated`. Medium/Normal remains 192 particles and gravity `(0, -10)`.
- `drop-obstacle` moves the same circle to `(2.5, 7.2)` and applies wake impulse `(0, -0.1)`. `reset-obstacle` restores `(2.5, 5.5)` bits without rebuilding the world.
- `SessionCore::create(SceneId::Fountain)` builds a teal `WATER` bowl. Default Medium/Up/Medium emission plateaus at ≤ 320 after 240 steps. `emission-rate`, `launch-speed`, and `aim-angle` return `ControlEffect::Live`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Evolve Dam Break water, gravity, and obstacle**
   - `18a6805` (test): failing water/gravity recreate and obstacle action tests
   - `3401a83` (feat): locked presets, drop/reset on the existing circle
2. **Task 2: Build a plateauing Fountain stream**
   - `b85302c` (test): failing create, plateau, off, and live-control tests
   - `5f29e1d` (feat): lifetime-capped emit, TAU aim, stub-list update

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/dam_break.rs` — water-amount/gravity recreate, drop/reset obstacle, Medium/Normal locked
- `crates/liquidfun-wasm/src/scene/fountain.rs` — bounded bowl emitter with live rate/speed/aim
- `crates/liquidfun-wasm/src/session.rs` — Fountain removed from the unimplemented stub list

## Decisions Made

- Keep `build(presets)` and read water-amount/gravity from the bag; missing keys stay Medium/Normal.
- Keep the existing Dam Break circle. Drop uses public `set_body_transform` plus `apply_body_linear_impulse_to_center(..., WakePolicy::Wake)`.
- Fountain ignores the construction bag. Aim is `-τ/8` / `0` / `+τ/8` from world up via `std::f32::consts::TAU`. Seed one particle so capture has a color buffer when emission is off.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fountain left the unimplemented stub list**
- **Found during:** Task 2
- **Issue:** `create_stub_scenes_fail_closed_without_a_live_world` still expected `SceneId::Fountain` to return `SceneUnimplemented`.
- **Fix:** Removed Fountain from that stub list. Color Mixer remains the last stub.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib -- --test-threads=1` exits 0
- **Committed in:** `5f29e1d`

**2. [Rule 3 - Blocking] Empty Fountain capture has no color buffer**
- **Found during:** Task 2 (`emission_rate_off_does_not_increase_count`)
- **Issue:** A zero-particle system made `capture_frame` return `FrameCaptureFailed` after `emission-rate=off`.
- **Fix:** Seed one teal `WATER` particle with lifetime 3s, matching the Water Wheel pattern, so capture works before the first emit.
- **Files modified:** `crates/liquidfun-wasm/src/scene/fountain.rs`
- **Verification:** Off-emission test and full `liquidfun-wasm` lib suite exit 0
- **Committed in:** `5f29e1d`

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Required so Fountain can construct and capture without poisoning the session. No unbounded emission and no `session.rs` factory change.

## Issues Encountered

- Clippy `wrong_self_convention` rejected `AimAngle::from_world_up`; renamed to `angle_from_up`.
- Plan said not to edit `session.rs`; the stub-list update was required for the full lib suite, as in Plans 03–05.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-07 Color Mixer or the next incomplete Wave-2/3 scene. Dam Break Medium/Normal remains the documented basin. Fountain population plateaus at ≤ 320. Do not mark catalog scenes ready until later chrome plans wire them. Color Mixer `build(presets)` still returns `SceneUnimplemented`.

***
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/dam_break.rs
- FOUND: crates/liquidfun-wasm/src/scene/fountain.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: 18-06-SUMMARY.md
- FOUND: 18a6805
- FOUND: 3401a83
- FOUND: b85302c
- FOUND: 5f29e1d
