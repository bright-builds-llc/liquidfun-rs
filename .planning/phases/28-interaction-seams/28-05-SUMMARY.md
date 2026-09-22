---
phase: 28-interaction-seams
plan: "05"
subsystem: wasm-scenes
tags: [theo-jansen, soft-distance, revolute-motor, motor-direction, liquidfun-wasm, tdd, ACT-05]

requires:
  - phase: 28-interaction-seams
    provides: Wave Machine SceneId factory patterns and Sequential TDD RED stub wiring
provides:
  - Theo Jansen BuiltScene with soft CreateLeg distance legs and live motor reverse
  - SceneId::TheoJansen / theo-jansen allowlist completing the five Phase 28 scene ids
  - Focused soft-joint and motor-direction unit tests
affects:
  - 28-06 catalog chrome
  - ACT-05 visitor walk-under-load reverse control

tech-stack:
  added: []
  patterns:
    - Soft DistanceJointDef frequency 10.0 damping 0.5 for CreateLeg suspension
    - Live motor-direction forward/reverse via set_revolute_motor_speed without world recreate

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/theo_jansen.rs
    - crates/liquidfun-wasm/src/scene/theo_jansen/tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Minimal SceneId::TheoJansen wiring in Task 1 RED so construction tests compile under hooks"
  - "Direction-only motor-direction forward/reverse; no speed-magnitude or limit-toggle"
  - "Split tests under theo_jansen/ to stay under Bright Builds file-length limit"

patterns-established:
  - "Pattern: CreateLeg soft distance set with_frequency(10.0)/with_damping_ratio(0.5) plus FilterData groupIndex -1"
  - "Pattern: motor-direction Live preset flips set_revolute_motor_speed sign; Water Wheel motor-off is anti-pattern"

requirements-completed: [ACT-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:25:53Z

duration: 7min
completed: 2026-09-22
---

# Phase 28 Plan 05: Theo Jansen Summary

**Theo Jansen WASM walker with soft distance legs, groupIndex -1 filtering, particle load, and live motor-direction reverse**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-22T14:18:06Z
- **Completed:** 2026-09-22T14:25:53Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- `SceneId::TheoJansen` / `"theo-jansen"` constructs through the WASM factory with radius `0.2` and damping `0.2`
- Ground, end walls, 40 balls, chassis+wheel with `FilterData` groupIndex `-1`, motorized revolute at `+2.0` / max torque `400`
- Six CreateLeg soft distance sets (`with_frequency(10.0)`, `with_damping_ratio(0.5)`); legs are not welded
- Live `motor-direction` `forward`/`reverse` flips motor speed sign via `set_revolute_motor_speed` without recreating the world; Reset remounts forward
- All five Phase 28 scene ids parse through the factory; `MAX_ADVANCE_STEPS` stays 4

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing Theo Jansen allowlist and reverse tests** - `29cfc0b` (test)
2. **Task 2: Implement Theo Jansen scene and factory wiring** - `ba13bd4` (feat)

**Plan metadata:** `fbd08b6` (docs: complete plan)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/theo_jansen.rs` - Theo Jansen build, soft legs, live motor reverse, and hooks
- `crates/liquidfun-wasm/src/scene/theo_jansen/tests.rs` - Soft-joint and motor-direction unit tests
- `crates/liquidfun-wasm/src/scene.rs` - SceneId::TheoJansen allowlist + build_scene arm
- `crates/liquidfun-wasm/src/session/tests.rs` - `"theo-jansen"` allowlist table entry

## Decisions Made

- Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks (same sequential-TDD pattern as Plans 01–04)
- Direction-only reverse control; no speed magnitude or limit-toggle chrome
- Split tests under `theo_jansen/` after implementation exceeded the 628-line file-length check

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Compiling RED stubs instead of compile-fail-only**
- **Found during:** Task 1
- **Issue:** Plan text allowed compile-fail until enum exists, but Phase 27/28 decisions and hooks need compiling RED
- **Fix:** Stub `theo_jansen::build` returning `SceneConstruction` with minimal `SceneId::TheoJansen` wiring
- **Files modified:** `theo_jansen.rs`, `scene.rs`, `session/tests.rs`
- **Verification:** `cargo test -p liquidfun-wasm theo_jansen` failed RED; allowlist parse passed
- **Committed in:** `29cfc0b`

**2. [Rule 3 - Blocking] Split tests for file-length limit**
- **Found during:** Task 2
- **Issue:** Combined implementation+tests reached 701 lines; Bright Builds file-lengths limit is 628
- **Fix:** Moved unit tests to `theo_jansen/tests.rs` and left `#[cfg(test)] mod tests;`
- **Files modified:** `theo_jansen.rs`, `theo_jansen/tests.rs`
- **Verification:** `cargo test -p liquidfun-wasm theo_jansen` exits 0; main file 512 lines
- **Committed in:** `ba13bd4`

---

**Total deviations:** 2 auto-fixed (1× Rule 2, 1× Rule 3)
**Impact on plan:** Necessary for correct TDD under hooks and managed file-length policy; no scope creep.

## Issues Encountered

None.

## Known Stubs

None — Theo Jansen constructs fully with soft legs, particle load, and live motor reverse.

## Threat Flags

None — `"motor-direction"` allowlists exact `"forward"` / `"reverse"` only; soft-leg honesty covered by unit tests; `MAX_ADVANCE_STEPS` remains 4.

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/theo_jansen.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/theo_jansen/tests.rs`
- FOUND: `crates/liquidfun-wasm/src/scene.rs`
- FOUND: commit `29cfc0b`
- FOUND: commit `ba13bd4`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ACT-05 Theo Jansen native seam is ready for catalog chrome (28-06)
- Do not raise `MAX_ADVANCE_STEPS` or weld Theo legs into rigid polygons
- Do not copy Water Wheel motor-off into Theo Jansen or Wave Machine

---
*Phase: 28-interaction-seams*
*Completed: 2026-09-22*
