---
phase: 26-catalog-shell-and-basin-scenes
plan: "01"
subsystem: wasm-scenes
tags: [particles, wasm, basin, liquidfun-wasm, tdd]

requires:
  - phase: 18-native-scenes
    provides: Scene factory, SessionCore, basin fixture helpers
provides:
  - SceneId::Particles allowlist entry and build_scene wiring
  - Pinned Particles open-basin scene with water group and dynamic ball
  - Headless construction, advance, control-reject, and pointer no-op tests
affects:
  - 26-02 catalog/sidebar advertisement
  - 26-03 Liquid Timer scene
  - web player smoke once catalog lands

tech-stack:
  added: []
  patterns:
    - Watch-first scene hooks reject unknown controls/actions; pointer is no-op success
    - Pinned JS testbed geometry via attach_basin_fixture + ParticleGroupRecipe

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/particles.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session.rs

key-decisions:
  - "Particles matches testParticles.js floor, slanted walls, radius 0.035, water circle at (0,3) r=2, ball at (0,8) r=0.5 density 0.5"
  - "No Liquid Timer, catalog, web, or engine API changes in this plan"
  - "MAX_ADVANCE_STEPS remains 4"

patterns-established:
  - "Watch-first WASM scenes: empty presets or UnknownControl; apply_pointer Ok(()) without layout mutation"
  - "Three basin segments plus one collect_circles ball for rigid frame export"

requirements-completed: [BASIN-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T00:35:00Z

duration: 12min
completed: 2026-09-22
---

# Phase 26 Plan 01: Particles WASM Scene Summary

**Pinned Particles basin (open walls, falling water circle, dynamic ball) is constructible and advance-safe through the WASM session factory.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-09-22T00:23:27Z
- **Completed:** 2026-09-22T00:35:00Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Added `SceneId::Particles` / `"particles"` allowlist and `particles::build` factory arm.
- Ported open floor + slanted walls, water particle group (radius 0.035, color RGBA 255,0,0,255), and one dynamic ball from `testParticles.js`.
- Headless tests cover create, eight capped advances, unknown control/action rejection, and pointer no-op.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing Particles allowlist and construction tests** - `b505aca` (test)
2. **Task 2: Implement Particles scene and factory wiring** - `c40b684` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/particles.rs` - Particles scene build + hooks + unit tests
- `crates/liquidfun-wasm/src/scene.rs` - `mod particles`, enum/parse/build arms
- `crates/liquidfun-wasm/src/session.rs` - allowlist parse test includes `"particles"`

## Decisions Made

- Followed pinned JS geometry exactly; body at `(0, 8)` with local circle offset zero (equivalent to JS fixture offset).
- Watch-first: reject any presets/controls/actions; pointer succeeds without mutating particle count.
- Left `MAX_ADVANCE_STEPS` at 4; did not shrink radius or cut particle counts.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. Construction/advance tests are slower than Dam Break (~60–90s for the Particles filter set) because radius `0.035` yields a dense water circle; this is expected under D-12/D-13.

## Next Steps

- Plan 26-02: catalog/sidebar advertisement for Particles (and later Liquid Timer).
- Plan 26-03+: Liquid Timer scene; keep Particles factory-only until catalog plans wire the UI.

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/particles.rs`
- FOUND: `b505aca` (Task 1)
- FOUND: `c40b684` (Task 2)
- FOUND: `cargo test -p liquidfun-wasm particles` exit 0
- FOUND: `cargo test -p liquidfun-wasm parse_scene_id_maps_allowlisted_tokens` exit 0
- FOUND: `MAX_ADVANCE_STEPS: u32 = 4`
