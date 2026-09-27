---
phase: 32-liquid-motion-bubbler
plan: "01"
subsystem: simulation
tags: [liquidfun, particles, revolute, prismatic, wasm]

requires:
  - phase: 31-sinusoidal-wave-tank
    provides: Watch-first SceneId pattern and a vertical prismatic motor written from on_advance
provides:
  - Native liquid-bubbler scene with a static waist, a motor-off paddle wheel, and a delayed side-shaft plate
  - SessionCore proof that an original particle crosses the waist and the wheel angle leaves 0 within 2 seconds
affects: [32-02-catalog, 32-03-browser]

tech-stack:
  added: []
  patterns:
    - "Motor-off revolute wheel turned by particle hits, with plate speed written once per on_advance"
    - "Proof window stays inside a 3 second dwell so the return is not the spectacle"

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/liquid_bubbler.rs
    - crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Wheel density is 0.03 because 0.20 only turned the wheel about 0.007 rad in 2 seconds"
  - "The plate rests 0.02 m above the floor so polygon skin does not lift translation off 0 during the dwell"
  - "This summary records evidence and does not approve the work"

patterns-established:
  - "liquid-bubbler is parsed after wave-tank and mapped to gravity-slider index 21"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T19:48:54Z

duration: 20 min
completed: 2026-09-27
---

# Phase 32 Plan 01: Liquid Bubbler Scene Summary

**Amber water drips through a static waist, turns a motor-off paddle wheel within 2 seconds, and the side-shaft plate stays down during that proof**

## Performance

- **Duration:** 20 min
- **Started:** 2026-09-27T19:28:35Z
- **Completed:** 2026-09-27T19:48:54Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `SceneId::LiquidBubbler` builds one amber water group above a static waist, a dynamic four-paddle wheel on a motor-off revolute joint, and a dynamic side-shaft plate on an upward prismatic joint.
- After 30 calls of `SessionCore::advance(4)`, the live particle count is unchanged, an original above-waist id is in the lower chamber outside the hub, and the absolute wheel angle is at least 0.05 rad.
- At that sample the plate translation stays under 0.01 and the prismatic motor speed stays at 0. `MAX_ADVANCE_STEPS` remains 4. Water Wheel, Color Mixer, Liquid Timer, and Hydraulic Fountain are unchanged.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the waist, wheel, and side-shaft plate** - `277710c` (feat)
2. **Task 2: Prove the drip crosses and turns the wheel** - `b7cad7f` (feat)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` - Static waist, motor-off wheel, delayed plate, and one amber water group
- `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` - Still-reservoir, crossing, dwell, motor, rebuild, control, and source guards
- `crates/liquidfun-wasm/src/scene.rs` - `liquid-bubbler` parse token and `build_scene` route
- `crates/liquidfun-wasm/src/scene/gravity_slider.rs` - Variant index 21 and a 22-scene id list
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist pair after wave-tank

## Decisions Made

- Kept the starting waist gap `0.14`, radius `0.025`, damping `0.2`, dwell `3.0`, plate speed `0.15`, and four paddles. The crossing failed on angle, not on an empty chamber.
- Lowered `WHEEL_DENSITY` from `0.20` to `0.03`. At `0.20` the absolute angle after 2 seconds was `0.007`, under the `0.05` floor. The revolute motor stayed disabled.
- Rested the plate `0.02` m above the floor (`PLATE_CENTER` y `0.06`). A flush contact was separated by polygon skin to translation `0.015`, which is past the `0.01` dwell bound. The joint frame moved with the body, so translation stays `0` while the motor speed is `0`.
- Did not approve this work. D-11 leaves independent review eligible.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Lightened the wheel so particle hits clear the angle floor**
- **Found during:** Task 2 (Prove the drip crosses and turns the wheel)
- **Issue:** Original particles crossed the waist, but the absolute wheel angle was `0.007` rad at density `0.20`.
- **Fix:** Set `WHEEL_DENSITY` to `0.03`. Did not enable the revolute motor, change the radius, or raise the step cap.
- **Files modified:** `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`
- **Verification:** `drip_crosses_the_waist_and_turns_the_wheel` exits 0
- **Committed in:** `b7cad7f` (Task 2 commit)

**2. [Rule 1 - Bug] Cleared the plate off the floor skin**
- **Found during:** Task 2 (Prove the drip crosses and turns the wheel)
- **Issue:** During the dwell the plate translation was `0.015`, the gap two polygon skins leave after a flush contact. The commanded speed was still the dwell value `0`.
- **Fix:** Raised the rest pose by `0.02` m and matched the prismatic frame, so translation stays `0`. Kept `DWELL` at `3.0`, `PLATE_SPEED` at `0.15`, and the proof at 2 seconds.
- **Files modified:** `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`
- **Verification:** `plate_stays_down_during_the_proof` exits 0
- **Committed in:** `b7cad7f` (Task 2 commit)

**3. [Rule 3 - Blocking] Grew the gravity-slider coverage array to 22**
- **Found during:** Task 1 (Build the waist, wheel, and side-shaft plate)
- **Issue:** `every_scene_builds_downward_gravity_from_the_slider` hard-coded a 21-slot seen array, which cannot compile once `SceneId::LiquidBubbler` exists.
- **Fix:** Mapped the new variant to index `21` and sized both the id list and the seen array to 22.
- **Files modified:** `crates/liquidfun-wasm/src/scene/gravity_slider.rs`
- **Verification:** `cargo test -p liquidfun-wasm scene::gravity_slider -- --test-threads=1` exits 0
- **Committed in:** `277710c` (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (2 bug, 1 blocking)
**Impact on plan:** The retunes stay inside the allowed density and contact-clearance fixes. The waist, motor-off revolute, dwell, and 4-step cap are unchanged.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 32-02. The native scene builds and the crossing proof passes. Catalog, portrait frame, README plan, and Chromium smoke are still Plan 02 and Plan 03. Passing tests here are evidence only and are not an approval.

## Known Stubs

None.

---
*Phase: 32-liquid-motion-bubbler*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/liquid_bubbler.rs
- FOUND: crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs
- FOUND: .planning/phases/32-liquid-motion-bubbler/32-01-SUMMARY.md
- FOUND: 277710c
- FOUND: b7cad7f
