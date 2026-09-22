---
phase: 26-catalog-shell-and-basin-scenes
plan: "02"
subsystem: wasm-scenes
tags: [liquid-timer, wasm, basin, tensile, viscous, liquidfun-wasm, tdd]

requires:
  - phase: 26-catalog-shell-and-basin-scenes
    provides: Particles factory pattern, SceneId allowlist, SessionCore watch-first hooks
provides:
  - SceneId::LiquidTimer allowlist entry and build_scene wiring
  - Pinned Liquid Timer closed-bowl scene with TENSILE|VISCOUS slab and shelf edges
  - Headless construction, flag, segment, advance, control-reject, and pointer no-op tests
affects:
  - 26-03 catalog/sidebar advertisement
  - web player smoke once catalog lands

tech-stack:
  added: []
  patterns:
    - Watch-first scene hooks reject unknown controls/actions; pointer is no-op success
    - ChainShape::closed bowl plus EdgeShape shelves exported via static collect_segments
    - ParticleGroupRecipe with ParticleFlags::TENSILE | ParticleFlags::VISCOUS

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/liquid_timer.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Liquid Timer matches testLiquidTimer.js bowl, radius 0.025, TENSILE|VISCOUS slab, and ten shelf/column edges"
  - "Default ParticleSystemDef viscous/surface-tension strengths; no strength tweaks required"
  - "MAX_ADVANCE_STEPS remains 4; Particles scene unchanged"

patterns-established:
  - "Shelf/column EdgeShape endpoints mirrored in a static RigidSegment list for drawable drains"
  - "Flag honesty asserted via particle_system_view flags buffer like Jelly Drop"

requirements-completed: [BASIN-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T00:39:14Z

duration: 4min
completed: 2026-09-22
---

# Phase 26 Plan 02: Liquid Timer WASM Scene Summary

**Pinned Liquid Timer bowl (tensile/viscous slab, gap/shelves/columns) is constructible and advance-safe through the WASM session factory.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-09-22T00:35:34Z
- **Completed:** 2026-09-22T00:39:14Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Added `SceneId::LiquidTimer` / `"liquid-timer"` allowlist and `liquid_timer::build` factory arm.
- Ported closed `ChainShape` bowl, radius `0.025`, `TENSILE | VISCOUS` filled box at `(0, 3.6)`, and all ten shelf/column `EdgeShape` fixtures from `testLiquidTimer.js`.
- Headless tests cover create, flag bits, ≥10 drawable segments, eight capped advances, unknown control/action rejection, and pointer no-op.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing Liquid Timer allowlist and flag tests** - `a2fd888` (test)
2. **Task 2: Implement Liquid Timer scene and factory wiring** - `7cf7b0e` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/liquid_timer.rs` - Liquid Timer scene build + hooks + unit tests
- `crates/liquidfun-wasm/src/scene.rs` - `mod liquid_timer`, enum/parse/build arms
- `crates/liquidfun-wasm/src/session.rs` - allowlist parse test includes `"liquid-timer"`

## Decisions Made

- Followed pinned JS geometry and flags exactly; left default viscous/surface-tension strengths (drain recognizable without tweaks).
- Watch-first: reject any presets/controls/actions; pointer succeeds without mutating particle count.
- Left `MAX_ADVANCE_STEPS` at 4; did not change Particles or cut particle counts.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Liquid Timer construction/advance is faster than Particles (~13s for the five-test filter) because the tensile/viscous slab is smaller than the Particles water circle.

## Next Steps

- Plan 26-03: catalog/sidebar advertisement for Particles and Liquid Timer.
- Keep both scenes factory-only until catalog plans wire the UI; do not edit `web/**` yet.

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/liquid_timer.rs`
- FOUND: `a2fd888` (Task 1)
- FOUND: `7cf7b0e` (Task 2)
- FOUND: `cargo test -p liquidfun-wasm liquid_timer` exit 0
- FOUND: `cargo test -p liquidfun-wasm parse_scene_id_maps_allowlisted_tokens` exit 0
- FOUND: `MAX_ADVANCE_STEPS: u32 = 4`
