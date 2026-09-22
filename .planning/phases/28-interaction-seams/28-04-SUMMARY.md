---
phase: 28-interaction-seams
plan: "04"
subsystem: wasm-scenes
tags: [wave-machine, revolute-motor, sim-time, liquidfun-wasm, tdd, ACT-04]

requires:
  - phase: 28-interaction-seams
    provides: Impulse SceneId factory patterns and Sequential TDD RED stub wiring
provides:
  - Wave Machine BuiltScene with motorized four-wall tank and on_advance cos(t) motor speed
  - SceneId::WaveMachine / wave-machine allowlist with watch-first hooks
  - Focused motor-speed-vs-sim-time unit tests
affects:
  - 28-06 catalog chrome
  - ACT-04 visitor watch-first rocking tank

tech-stack:
  added: []
  patterns:
    - Live revolute motor speed mutated only in on_advance from simulated t += 1/60
    - Water Wheel motor-off is the explicit anti-pattern for Wave Machine / Theo Jansen

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/wave_machine.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs

key-decisions:
  - "Minimal SceneId::WaveMachine wiring in Task 1 RED so construction tests compile under hooks"
  - "Motor formula uses PI to match pinned testWaveMachine.js — not Water Wheel motor-off"

patterns-established:
  - "Pattern: with_motor(true, 0.05*PI, 1e7) at create; set_revolute_motor_speed(0.05*cos(t)*PI) per on_advance"
  - "Pattern: watch-first apply_pointer no-op; unknown controls reject; MAX_ADVANCE_STEPS stays 4"

requirements-completed: [ACT-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:16:38Z

duration: 5min
completed: 2026-09-22
---

# Phase 28 Plan 04: Wave Machine Summary

**Wave Machine WASM scene with motorized four-wall tank rocking via `0.05 * cos(t) * π` from simulated session time**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-22T14:12:31Z
- **Completed:** 2026-09-22T14:16:38Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `SceneId::WaveMachine` / `"wave-machine"` constructs through the WASM factory with radius `0.025` and damping `0.2`
- Four-wall dynamic tank at `(0,1)` pinned by revolute with `enable_motor: true`, max torque `1e7`, initial speed `0.05 * π`
- Each `on_advance` does `t += 1/60` then `set_revolute_motor_speed(0.05 * cos(t) * π)`; pause freezes steps hence `t`; reset remounts with `t = 0`
- Watch-first: `apply_pointer` no-op; unknown controls reject; Water Wheel motor-off is not copied
- `MAX_ADVANCE_STEPS` stays 4; unit tests prove motor speed tracks sim-time advances

## Task Commits

Each task was committed atomically:

1. **Task 1: Failing Wave Machine allowlist and motor-time tests** - `377bd8e` (test)
2. **Task 2: Implement Wave Machine scene and factory wiring** - `ac66830` (feat)

**Plan metadata:** `2a612dd` (docs: complete plan)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/wave_machine.rs` - Wave Machine build, motorized hooks, and module tests
- `crates/liquidfun-wasm/src/scene.rs` - SceneId::WaveMachine allowlist + build_scene arm
- `crates/liquidfun-wasm/src/session/tests.rs` - `"wave-machine"` allowlist table entry

## Decisions Made

- Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks (same sequential-TDD pattern as Plans 01–03)
- Motor formula keeps `PI` to match pinned `testWaveMachine.js` Step; do not copy Water Wheel `enable_motor: false`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Compiling RED stubs instead of compile-fail-only**
- **Found during:** Task 1
- **Issue:** Plan text allowed compile-fail until enum exists, but Phase 27/28 decisions and hooks need compiling RED
- **Fix:** Stub `wave_machine::build` returning `SceneConstruction` with minimal `SceneId::WaveMachine` wiring
- **Files modified:** `wave_machine.rs`, `scene.rs`, `session/tests.rs`
- **Verification:** `cargo test -p liquidfun-wasm wave_machine` failed RED; allowlist parse passed
- **Committed in:** `377bd8e`

---

**Total deviations:** 1 auto-fixed (1× Rule 2)
**Impact on plan:** Necessary for correct TDD under hooks; no scope creep.

## Issues Encountered

None.

## Known Stubs

None — Wave Machine constructs fully with motorized revolute updates from simulated time.

## Threat Flags

None — `"wave-machine"` is exact-token allowlisted; motor writes stay inside `on_advance`; `MAX_ADVANCE_STEPS` remains 4.

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/wave_machine.rs`
- FOUND: `crates/liquidfun-wasm/src/scene.rs`
- FOUND: commit `377bd8e`
- FOUND: commit `ac66830`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ACT-04 Wave Machine native seam is ready for catalog chrome (28-06) and Theo Jansen (28-05)
- Do not raise `MAX_ADVANCE_STEPS` or silently change Wave Machine radius from `0.025`
- Do not copy Water Wheel motor-off into Theo Jansen

---
*Phase: 28-interaction-seams*
*Completed: 2026-09-22*
