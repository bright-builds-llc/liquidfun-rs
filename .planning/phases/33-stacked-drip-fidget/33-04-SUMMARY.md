---
phase: 33-stacked-drip-fidget
plan: "04"
subsystem: testing
tags: [stacked-drip, particles, side-shaft, prismatic]

requires:
  - phase: 33-stacked-drip-fidget
    provides: Native stacked-drip scene with three motor-off trays and a delayed side-shaft plate
provides:
  - A sealed side shaft whose plate boards liquid and spills it back above the top tray
  - A dwell-plus-rise test that puts an original particle back above the top tray with the live count unchanged and the revolute motors off
affects: [phase-33-verification]

tech-stack:
  added: []
  patterns:
    - "The plate spans the shaft so liquid stays on it, and the vertical slot under the divider stays wider than one particle"
    - "The left floor slopes into that slot so drained liquid reaches the plate before the rise"
    - "The cascade proof allows a particle right of the divider only after it is below the bottom tray"

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/stacked_drip.rs
    - crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs
    - crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs
    - web/tests/portrait-bounds.test.ts

key-decisions:
  - "Divider runs y 0.22 to 1.70 at x 0.90 to 0.98. The plate spans x 1.00 to 1.50, y 0.02 to 0.06 at rest, so the side gap is 0.02 m"
  - "Stroke stays 1.90 m at 0.15 m/s. One second into the descent the plate top is still above the y 1.70 spill lip"
  - "The left floor falls from (-0.70, 0.16) to (0.90, 0.08) so liquid slides onto the plate"
  - "This implementing agent did not approve the work"

patterns-established:
  - "Return identity follows original particle ids that started above the top tray"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-09-27T23-41-40
generated_at: 2026-09-28T02:15:49Z

duration: 25 min
completed: 2026-09-28
---

# Phase 33 Plan 04: Open the Return Shaft Summary

**The sealed side shaft lifts an original particle back above the top tray after one dwell-plus-rise, with the tray motors still off**

## Performance

- **Duration:** 25 min
- **Started:** 2026-09-28T01:50:00Z
- **Completed:** 2026-09-28T02:15:49Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- The divider runs from y 0.22 to y 1.70, so the slot above the resting plate is 0.16 m, wider than the 0.05 m particle diameter.
- The plate spans x 1.00 to 1.50 and is 0.04 m thick. Each side gap is 0.02 m. It rises 1.90 m at 0.15 m/s, so one second into the descent its top is still above the y 1.70 spill lip.
- `return_lifts_an_original_particle_above_the_top_tray` passes with the cascade order tests. Dwell stays 6 s, the 5 s proof still sees the plate down, and `MAX_ADVANCE_STEPS` stays 4.

## Task Commits

Each task was committed atomically:

1. **Task 1: Seal the shaft and slope the floor into the inlet** - `0235692` (feat)
2. **Task 2: Prove an original particle returns above the top tray** - `cb850cd` (test)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/stacked_drip.rs` - Divider y 0.22 to 1.70, plate x 1.00 to 1.50 and y 0.02 to 0.06 at rest, stroke 1.90 m, speed 0.15 m/s. The left floor wedge meets the shaft at y 0.08. Revolute motors are still never enabled.
- `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs` - Divider box and shaft floor use those constants. The wedge polygon is the sloped left floor.
- `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs` - One dwell-plus-rise run checks live count, an original particle id back above the top tray and left of the divider, and disabled revolute motors. The cascade sample still requires every original particle below the bottom tray, and treats a particle right of the divider as a bypass only when that particle is still at or above the bottom tray.
- `web/tests/portrait-bounds.test.ts` - Divider, resting plate, raised plate, slope, and shaft floor corners match that geometry. The phone frame stays minX -0.86, minY -0.16, maxX 1.68, maxY 2.18.

## Decisions Made

- Followed the bubbler seal instead of the plan's 0.10 m side gap. Phase 32 recorded that a gap that wide drains to the floor. The plate spans the shaft, and the vertical slot stays open for a particle.
- Stroke stays 1.90 m and speed stays 0.15 m/s. At the sample, one second into the descent, the plate top is about y 1.81, above the y 1.70 spill lip.
- The flat left floor left most of the water sitting away from the shaft. The left floor now falls from y 0.16 at x -0.70 to y 0.08 at the divider, which is above the resting plate top, so liquid slides onto the plate.
- Independent AI review remains eligible under the 2026-09-16 owner policy. This implementing agent did not approve the work. Passing tests are evidence for the orchestrator to re-verify, not a review acknowledgment.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Sealed the shaft and sloped the floor into the inlet**
- **Found during:** Task 2 (return an original particle above the top tray)
- **Issue:** The plan's first inlet opened a 0.10 m gap beside the plate. A spanning plate can lift particles, but on a flat floor they stayed in the shaft or never boarded. Sampling while the plate top was still above the spill lip was not enough by itself: three particles sat on the plate near x 1.14 to 1.23, and the rest sat on the floor.
- **Fix:** Plate spans x 1.00 to 1.50 with half-height 0.02 and a 0.02 m side gap. Divider bottom is y 0.22 and the spill lip is y 1.70. Stroke stays 1.90 m at 0.15 m/s. The left floor slopes down to y 0.08 at the divider so liquid boards the plate. The cascade assertion accepts a particle right of the divider only when it is also below the bottom tray. Portrait corners match those faces, including the raised plate at y 1.92 to 1.96.
- **Files modified:** `crates/liquidfun-wasm/src/scene/stacked_drip.rs`, `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs`, `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs`, `web/tests/portrait-bounds.test.ts`
- **Verification:** `CARGO_TARGET_DIR=target/return-check cargo test -p liquidfun-wasm --lib scene::stacked_drip -- --test-threads=1` exited 0 (9 passed). `cd web && bun run test:unit -- tests/portrait-bounds.test.ts` exited 0 (5 passed). `target/debug/deps` directory reads hung in this workspace, so the same cargo test used a fresh target directory.
- **Committed in:** `0235692` (Task 1) and `cb850cd` (Task 2)

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The dwell, particle radius, step cap, and revolute motors stay as locked. The inlet, plate span, and left floor changed so both proofs can pass together.

## Issues Encountered

`target/debug/deps` directory listing hung on `getdirentries` / `getattrlistbulk`, so `cargo test` against the default target directory did not finish. The same test command with `CARGO_TARGET_DIR=target/return-check` compiled and passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

The native return proof and the portrait frame test pass. This summary does not approve the work.

## Finished geometry

- Divider: x 0.90 to 0.98, y 0.22 to 1.70. Center (0.94, 0.96), half-height 0.74.
- Plate at rest: x 1.00 to 1.50, y 0.02 to 0.06. Center (1.25, 0.04), half-width 0.25, half-height 0.02.
- Side gap: 0.02 m, narrower than the 0.05 m particle diameter. Boarding slot: 0.16 m.
- Stroke 1.90 m, speed 0.15 m/s, dwell 6 s. Raised plate: x 1.00 to 1.50, y 1.92 to 1.96. Sample plate top is about y 1.81, above the y 1.70 spill lip.
- Left floor wedge: (-0.78, -0.08), (0.90, -0.08), (0.90, 0.08), (-0.70, 0.16). Shaft floor: x 0.90 to 1.60, y -0.08 to 0.

## Test commands

- `CARGO_TARGET_DIR=target/return-check cargo test -p liquidfun-wasm --lib scene::stacked_drip -- --test-threads=1` — 9 passed
- `cd web && bun run test:unit -- tests/portrait-bounds.test.ts` — 5 passed

*Phase: 33-stacked-drip-fidget*
*Completed: 2026-09-28*

## Self-Check: PASSED

- FOUND: `.planning/phases/33-stacked-drip-fidget/33-04-SUMMARY.md`
- FOUND: `0235692`
- FOUND: `cb850cd`
- FOUND: `crates/liquidfun-wasm/src/scene/stacked_drip.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs`
- FOUND: `web/tests/portrait-bounds.test.ts`
