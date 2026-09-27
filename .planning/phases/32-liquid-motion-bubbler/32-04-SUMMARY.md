---
phase: 32-liquid-motion-bubbler
plan: "04"
subsystem: testing
tags: [liquid-bubbler, particles, side-shaft, prismatic]

requires:
  - phase: 32-liquid-motion-bubbler
    provides: Native liquid-bubbler scene with a static waist, a motor-off wheel, and a delayed side-shaft plate
provides:
  - A side-shaft inlet wide enough for one particle to sit on the plate
  - A dwell-plus-rise test that puts an original particle back above the waist with the live count unchanged and the revolute motor off
affects: [phase-32-verification]

tech-stack:
  added: []
  patterns:
    - "The plate spans the shaft so liquid stays on it, and the vertical slot under the divider stays wider than one particle"
    - "The return proof uses the same particle ids as the two-second crossing test"

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/liquid_bubbler.rs
    - crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs
    - web/tests/portrait-bounds.test.ts

key-decisions:
  - "The plate spans x 0.58 to 0.94 with half-height 0.02, and the divider runs y 0.22 to 1.62, so liquid can board the plate and the side gap is narrower than one particle"
  - "Stroke is 1.80 m at 0.15 m/s so the plate top is still above the y 1.62 spill lip one second into the descent"
  - "Dwell stays 3 s, particle radius stays 0.025, MAX_ADVANCE_STEPS stays 4, and the revolute motor stays off"
  - "This implementing agent did not approve the work"

patterns-established:
  - "Return identity follows live particle ids, the same identity the waist-crossing test already uses"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T20:43:31Z

duration: 24 min
completed: 2026-09-27
---

# Phase 32 Plan 04: Open the Return Shaft Summary

**The side shaft now lifts an original particle back above the waist after one dwell-plus-rise, with the wheel motor still off**

## Performance

- **Duration:** 24 min
- **Started:** 2026-09-27T20:19:20Z
- **Completed:** 2026-09-27T20:43:31Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- The divider runs from y 0.22 to y 1.62, so the slot above the plate is wider than the 0.05 m particle diameter.
- The plate spans x 0.58 to 0.94, is 0.04 m thick, and rises 1.80 m. Its top stays above the y 1.62 spill lip through the extra second after the rise.
- `return_lifts_an_original_particle_above_the_waist` passes with the existing crossing test. Dwell stays 3 s, speed stays 0.15 m/s, radius stays 0.025, and `MAX_ADVANCE_STEPS` stays 4.

## Task Commits

Each task was committed atomically:

1. **Task 1: Open the return shaft** - `2c79d63` (fix)
2. **Task 2: Prove an original particle returns above the waist** - `d7fe3bc` (fix)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` - Divider bottom at y 0.22, plate from x 0.58 to 0.94 and y 0.02 to 0.06 at rest, stroke 1.80 m. The revolute motor is still never enabled.
- `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` - One dwell-plus-rise run checks live count, an original particle id back above the waist and left of the divider, and a disabled revolute motor.
- `web/tests/portrait-bounds.test.ts` - Divider and plate corners match that geometry. The phone frame stays minX -0.71, minY -0.16, maxX 1.12, maxY 1.98.

## Decisions Made

- Followed the plan's first constants, then changed them after the return test failed. A 0.10 m gap beside the plate let liquid drain to the floor. The plate now spans the shaft, and the vertical slot above it stays open for a particle.
- Stroke is 1.80 m rather than 1.64 m. At the sample, one second into the descent, the plate top is still above the spill lip.
- Plate half-height is 0.02 so the step off the floor is low enough for liquid to board during the 3 s dwell.
- Independent AI review remains eligible under the 2026-09-16 owner policy. This implementing agent did not approve the work. Passing tests are evidence for the orchestrator to re-verify, not a review acknowledgment.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Sealed the shaft instead of widening the side gap**
- **Found during:** Task 2 (return an original particle above the waist)
- **Issue:** The plan's first constants opened a 0.10 m gap beside the plate. Liquid sat on the floor of that gap. The plate reached translation 1.49 m with every particle still below y 0.18. Widening that gap, the plan's listed plate tune, would have made the drain larger. Raising the divider bottom would have left the same drain.
- **Fix:** Plate spans x 0.58 to 0.94 with half-height 0.02. Divider bottom is y 0.22 and the top stays y 1.62. Stroke is 1.80 m. Portrait corners match those faces.
- **Files modified:** `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`, `web/tests/portrait-bounds.test.ts`
- **Verification:** `cargo test -p liquidfun-wasm scene::liquid_bubbler -- --test-threads=1` exited 0 (9 passed). `bun run test:unit -- tests/portrait-bounds.test.ts` exited 0 (5 passed).
- **Committed in:** `d7fe3bc` (Task 2 commit)

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The dwell, particle radius, step cap, and revolute motor stay as locked. The inlet and stroke changed so the return test can pass.

## Issues Encountered

The first geometry run did not return a particle. The plate moved, and the liquid did not ride it. The tuned inlet and stroke above are what the return test passed with.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

The native return proof and the portrait frame test pass. Re-verification set `32-VERIFICATION.md` to passed. This summary does not approve the phase.

*Phase: 32-liquid-motion-bubbler*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: `.planning/phases/32-liquid-motion-bubbler/32-04-SUMMARY.md`
- FOUND: `2c79d63`
- FOUND: `d7fe3bc`
- FOUND: `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs`
