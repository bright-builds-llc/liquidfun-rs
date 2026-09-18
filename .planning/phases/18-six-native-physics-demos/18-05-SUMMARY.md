---
phase: 18-six-native-physics-demos
plan: "05"
subsystem: wasm-scene
tags: [liquidfun-wasm, water-wheel, revolute, tdd, particle-body]

requires:
  - phase: 18-six-native-physics-demos
    provides: SceneId factory routing WaterWheel to water_wheel::build(presets)
provides:
  - Native Water Wheel with a motor-off revolute hub and four transformed paddles
  - Live jet-strength and emission controls that emit bounded water in on_advance
  - Angle proofs that rotation comes from particle-body coupling, not a motor
affects: [18-06, 18-09, catalog-ready, player-controls]

tech-stack:
  added: []
  patterns:
    - RevoluteJointDef without a motor; paddles from BodySnapshot::transform().apply
    - live jet/emission ControlEffect::Live
    - full-system emit is a no-op, not a session poison

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/water_wheel.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Keep build(presets) and ignore the bag; jet strength and emission are live."
  - "Hub at (0, 3) uses RevoluteJointDef::new without a motor; paddles are local fixtures plus transformed segments."
  - "Medium jet at 8 m/s from the left plus two WATER particles per step rotates the wheel natively; no D-11 pinwheel fallback."

patterns-established:
  - "Wave-2 runtime jet/emission stay Live; construction bag remains unused."
  - "If a motor-off wheel does not turn, retune jet aim, paddle area, or radius instead of enabling the motor."

requirements-completed: [DEMO-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T04:54:13Z

duration: 8min
completed: 2026-09-18
---

# Phase 18 Plan 05: Water Wheel Native Spike Summary

**Motor-off revolute paddle wheel at `(0, 3)` turned by a bounded left-side water jet, with live Weak/Med/Strong and On/Off controls and transformed paddle capture.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-09-18T04:45:56Z
- **Completed:** 2026-09-18T04:54:13Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `SessionCore::create(SceneId::WaterWheel)` builds a static trough, a dynamic hub circle, four paddle fixtures, and one motor-off revolute pin. `collect_segments` maps `PADDLE_LOCAL_SEGMENTS` through `BodySnapshot::transform().apply`.
- After 180 steps with emission On and medium jet (8 m/s), captured paddle angle changes by more than 0.05 rad. After 180 Off steps the angle stays within 0.01 rad. No revolute motor and no production `set_body_transform` spin.
- Particle population uses `with_lifetime(3.0)`, `with_maximum_count(320)`, and `destroy_by_age(true)`. After 240 On steps the count is ≤ 320 and no longer strictly increasing. A full system emit is a no-op.

## Task Commits

Each task was committed atomically:

1. **Task 1: Pin a motor-off wheel and capture transformed paddles**
   - `4769bf4` (test): failing create, motor-off, and live-pose capture tests
   - `fa327a4` (feat): trough, hub, four paddles, `RevoluteJointDef::new` without a motor
2. **Task 2: Prove the jet turns the wheel and emission stays bounded**
   - `346f904` (test): failing On/Off angle, plateau, and live-control tests
   - `9e1b6f7` (feat): `on_advance` jet, live jet/emission allowlist, stub-list update

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — native Water Wheel world, live jet/emission, motor-off proofs
- `crates/liquidfun-wasm/src/session.rs` — drop Water Wheel from the unimplemented stub list

## Decisions Made

- Keep `build(presets)` and ignore the bag; jet strength and emission are live session policy.
- Pin the hub at `(0, 3)` with `RevoluteJointDef::new` plus `with_frame`; never call `with_motor`.
- Medium jet (8 m/s) from `(-3.9, 3.15)` emitting two `WATER` particles per step rotates the wheel through native coupling. Assumption A3 held; no D-11 pinwheel-and-balls revision.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Water Wheel left the unimplemented stub list**
- **Found during:** Task 2
- **Issue:** `session.rs` still expected `SceneId::WaterWheel` to return `SceneUnimplemented`, so the full lib suite would fail once create succeeded.
- **Fix:** Removed Water Wheel from `create_stub_scenes_fail_closed_without_a_live_world`. Fountain and Color Mixer remain stubs.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib` exits 0
- **Committed in:** `9e1b6f7`

**2. [Rule 1 - Bug] Clippy `unnecessary_wraps` on emit**
- **Found during:** Task 2
- **Issue:** `emit_jet` always returned `Ok(())`, which clippy denied under `-D warnings`.
- **Fix:** Emit returns `()` and treats any create failure as a no-op.
- **Files modified:** `crates/liquidfun-wasm/src/scene/water_wheel.rs`
- **Verification:** `cargo clippy -p liquidfun-wasm --all-targets -- -D warnings` exits 0
- **Committed in:** `9e1b6f7`

***

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Required for the full lib suite and clippy. No motor, no D-11 fallback, no `session.rs` factory change.

## Issues Encountered

None. The motor-off wheel rotated from the jet on the first tuned emit (two particles/step, radius 0.16, wheel density 0.45).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-06 Dam Break controls or the next incomplete Wave-2 scene. Water Wheel physics honesty is proven natively. Catalog ready-flag and player chrome remain later plans. Fountain and Color Mixer `build(presets)` still return `SceneUnimplemented`.

***
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/water_wheel.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: 4769bf4
- FOUND: fa327a4
- FOUND: 346f904
- FOUND: 9e1b6f7
