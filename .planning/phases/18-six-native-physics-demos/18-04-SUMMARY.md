---
phase: 18-six-native-physics-demos
plan: "04"
subsystem: wasm-scene
tags: [liquidfun-wasm, jelly-drop, elastic, tdd, particle-group]

requires:
  - phase: 18-six-native-physics-demos
    provides: SceneId factory routing JellyDrop to jelly_drop::build(presets)
provides:
  - Native Jelly Drop elastic group on two overlapping rigid bars
  - Circle/Square and Soft/Medium/Firm construction presets that recreate the world
  - Labeled poke-jelly impulse that deforms the group without reset
affects: [18-05, 18-08, 18-09, catalog-ready, player-controls]

tech-stack:
  added: []
  patterns:
    - construction presets return ControlEffect::Recreated
    - poke uses apply_particle_linear_impulse_range on a contiguous group range
    - native bbox test after poke plus recovery steps

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/jelly_drop.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Keep build(presets) and parse shape/softness from the bag; defaults are circle and medium 1.0."
  - "Poke impulse is (0, -8) on a contiguous first-third of group member ids."
  - "Overlapping bars near y=1 plus system elastic/spring 0.75 keep the blob inside the camera box after poke."

patterns-established:
  - "Wave-2 construction presets recreate via SessionCore; live poke does not."
  - "If an elastic group shreds, retune count, shelf, or poke range instead of expanding the engine API."

requirements-completed: [DEMO-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T04:44:54Z

duration: 13min
completed: 2026-09-18
---

# Phase 18 Plan 04: Jelly Drop Native Spike Summary

**Native pink `ELASTIC|SPRING` group on two overlapping bars, with Circle/Square and Soft/Medium/Firm construction resets and a labeled downward poke that deforms in-engine.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-09-18T04:31:34Z
- **Completed:** 2026-09-18T04:44:54Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `SessionCore::create(SceneId::JellyDrop)` builds a pink `ELASTIC|SPRING` filled group of 8..=220 particles over two static bars near `y=1`.
- `apply_control("shape", "circle"|"square")` and `apply_control("softness", "soft"|"medium"|"firm")` return `ControlEffect::Recreated`; `build(presets)` rereads the bag.
- `apply_action("poke-jelly")` applies impulse `(0, -8)` to a contiguous first-third of group members. After poke plus 60 steps, particle count is unchanged and positions stay finite inside `(-7,-2)..(7,9)`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Construct a coherent elastic group on two bars**
   - `b50cf5e` (test): failing factory, flag, and constant-count tests
   - `066b5fb` (feat): `ParticleGroupRecipe` builder, two bars, stub-list update
2. **Task 2: Recreate on shape/softness and poke without reset**
   - `4db4e42` (test): failing recreate, poke, bbox, and allowlist tests
   - `1bc6691` (feat): Recreated presets, poke impulse, bounded recovery retune

**Plan metadata:** docs commit on this SUMMARY / STATE / ROADMAP / REQUIREMENTS update

_Note: TDD tasks produced test then feat commits._

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` — elastic group builder, shape/softness allowlist, poke action, native honesty tests
- `crates/liquidfun-wasm/src/session.rs` — Jelly Drop removed from unimplemented stub-scene list

## Decisions Made

- Keep `build(presets)` from Plan 01. Missing keys default to circle and medium strength `1.0`; Soft `0.4`, Firm `2.0`.
- Documented poke is downward `(0, -8)` on a contiguous first-third of `particle_group_view` member ids, not JavaScript position writes.
- Overlap the two bars at `y=1` and raise system elastic/spring strength to `0.75` with damping `1.2` so the blob stays coherent after poke. No public `liquidfun` API change.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Jelly Drop left the unimplemented stub list**
- **Found during:** Task 1 (native group implementation)
- **Issue:** `create_stub_scenes_fail_closed_without_a_live_world` still expected `SceneId::JellyDrop` to return `SceneUnimplemented`.
- **Fix:** Removed Jelly Drop from that stub list so Dam Break, Float or Sink, and remaining stubs still fail closed.
- **Files modified:** `crates/liquidfun-wasm/src/session.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib -- --test-threads=1` exits 0
- **Committed in:** `066b5fb`

**2. [Rule 1 - Bug] Whole-group poke launched particles past the camera box**
- **Found during:** Task 2 (poke plus 60-step bbox test)
- **Issue:** Impulse `(0, -8)` on every member plus a gap between bars sent a particle to about `(10.8, -4.8)`.
- **Fix:** Overlapped the two bars, raised system elastic/spring strength, and applied the same documented impulse to a contiguous first-third of group members so the blob deforms without shredding.
- **Files modified:** `crates/liquidfun-wasm/src/scene/jelly_drop.rs`
- **Verification:** `poke_then_sixty_steps_keep_particles_inside_the_camera_box` passes; full `liquidfun-wasm` lib tests exit 0
- **Committed in:** `1bc6691`

***

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Required so the factory can construct Jelly Drop and the poke honesty test stays inside the documented box. No engine API expansion and no fake deformation.

## Issues Encountered

- First whole-group poke shredded through the bar gap. Retuned shelf overlap, material strengths, and poke range per Assumption A2 instead of adding a custom spring or expanding `liquidfun`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 18-05 Water Wheel spike. Jelly Drop is a native world with construction shape/softness and a labeled poke. Do not mark the catalog scene ready until later chrome plans wire it. Do not add pointer input.

***
*Phase: 18-six-native-physics-demos*
*Completed: 2026-09-18*

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/jelly_drop.rs
- FOUND: crates/liquidfun-wasm/src/session.rs
- FOUND: 18-04-SUMMARY.md
- FOUND: b50cf5e
- FOUND: 066b5fb
- FOUND: 4db4e42
- FOUND: 1bc6691
