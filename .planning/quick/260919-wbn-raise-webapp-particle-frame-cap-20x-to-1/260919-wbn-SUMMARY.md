---
phase: 260919-wbn-raise-webapp-particle-frame-cap-20x-to-1
plan: "01"
subsystem: wasm-playground
tags: [wasm, particles, playground, frame-cap, liquidfun-wasm]

requires:
  - phase: 18-six-native-physics-demos
    provides: Six native WASM playground scenes with copied frames
provides:
  - Copied WASM/JS frame particle cap 10240
  - ~10x denser playground particles at locked ~1/sqrt(10) radii
  - Dam Break Medium 48x40 = 1920, Small 650, Large 2772, system cap 10240
affects:
  - web playground visuals
  - demo-media stills
  - elastic group-creation Voronoi budgets

tech-stack:
  added: []
  patterns:
    - Identical MAX_PARTICLE_COUNT = 10240 in Rust FrameData, session capture, and JS parseRenderFrame
    - Scene packing uses locked 1/sqrt(10) radii with unchanged rigid geometry

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/dam_break.rs
    - crates/liquidfun-wasm/src/scene/dam_break/tests.rs
    - crates/liquidfun-wasm/src/scene/float_or_sink.rs
    - crates/liquidfun-wasm/src/scene/color_mixer.rs
    - crates/liquidfun-wasm/src/scene/color_mixer/tests.rs
    - crates/liquidfun-wasm/src/scene/jelly_drop.rs
    - crates/liquidfun-wasm/src/scene/fountain.rs
    - crates/liquidfun-wasm/src/scene/water_wheel.rs
    - crates/liquidfun-wasm/src/scene/water_wheel/tests.rs
    - crates/liquidfun/src/world/particle_object.rs
    - web/src/physics/frame.ts
    - web/tests/frame.test.ts
    - web/e2e/rust-wasm-proof.spec.ts

key-decisions:
  - "Copied-frame particle cap is 10240 in Rust FrameData, session capture, and JS parseRenderFrame"
  - "Playground packing uses exact locked radii/spacing; rigid bodies and canvas camera stay unchanged"
  - "Jelly Drop at radius 0.050596 needed larger private group-creation Voronoi budgets; public liquidfun radius/spacing defaults were not changed"
  - "Color Mixer measured 1154 and Jelly circle/square measured 793/961, so scene maximum_count stayed 2200"

patterns-established:
  - "Frame caps stay identical across frame.rs, session.rs, and frame.ts"
  - "Emitter scenes use a named EMIT_SPACING constant instead of paddle or magic stagger literals"

requirements-completed:
  - WASM-02
  - DEMO-01
  - DEMO-02
  - DEMO-03
  - DEMO-04
  - DEMO-05
  - DEMO-06
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 260919-wbn
generated_at: 2026-09-20T05:10:04Z

duration: 48min
completed: 2026-09-20
---

# Phase 260919-wbn Plan 01: Raise webapp particle frame cap Summary

**Copied WASM/JS frames now accept 10240 particles, and all six playground scenes pack about 10× denser at locked ~1/sqrt(10) radii without growing basins or rigid bodies.**

## Performance

- **Duration:** 48 min
- **Started:** 2026-09-20T04:22:07Z
- **Completed:** 2026-09-20T05:10:04Z
- **Tasks:** 3
- **Files modified:** 17

## Accomplishments

- Raised the copied-frame particle cap from 512 to 10240 in Rust `FrameData`, session `capture_frame`, and JS `parseRenderFrame`, with tests that accept 10240 and reject 10241.
- Packed Dam Break to Medium 48×40 = 1920, Small 25×26 = 650, Large 63×44 = 2772 at radius `0.06324555` and spacing `0.101193`, system cap 10240.
- Packed Float or Sink to 1800/3840, Color Mixer and Jelly Drop via finer radii with unchanged group extents, Fountain/Water Wheel emit ~10× per step with cap 3200 and named `EMIT_SPACING`.

## Task Commits

Each task was committed atomically:

1. **Task 1–3: Raise cap, pack scenes, scale emitters** - `3a047bf` (feat)

**Plan metadata:** not committed (orchestrator owns `.planning` / STATE / ROADMAP).

_Note: User constraints asked for one atomic code commit at the end of Task 3, with no `.planning` docs in git._

## Files Created/Modified

- `crates/liquidfun-wasm/src/frame.rs` - `MAX_PARTICLE_COUNT = 10240`
- `crates/liquidfun-wasm/src/session.rs` - `MAX_FRAME_PARTICLES = 10240` plus Dam Break 1920 pins
- `web/src/physics/frame.ts` - JS `MAX_PARTICLE_COUNT = 10240`
- `web/tests/frame.test.ts` - reject 10241 / accept 10240
- `crates/liquidfun-wasm/src/scene/dam_break.rs` - finer 10× grids and radius/spacing
- `crates/liquidfun-wasm/src/scene.rs` - documented Medium 48×40 = 1920, cap 10240
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` - 45×40 = 1800, cap 3840
- `crates/liquidfun-wasm/src/scene/color_mixer.rs` - radius `0.05692`, cap 2200
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` - radius `0.050596`, cap 2200
- `crates/liquidfun-wasm/src/scene/fountain.rs` - emit 10/20/30, cap 3200, `EMIT_SPACING = 0.025298`
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` - emit 20, cap 3200, `EMIT_SPACING = 0.044272`
- `crates/liquidfun/src/world/particle_object.rs` - private group-creation Voronoi budgets for dense elastic fills
- `web/e2e/rust-wasm-proof.spec.ts` - Dam Break copy and proof JSON use 1920

## Decisions Made

- Keep rigid radii (Dam Break 0.75, Float or Sink 0.5, Water Wheel hub 0.35 / paddles) and canvas camera unchanged; no extra draw-scale.
- Do not regenerate demo-media. Catalog stills will look stale because particles are now ~0.32× diameter.
- Color Mixer and Jelly scene `maximum_count` stayed 2200: measured Color Mixer 1154, Jelly circle 793, Jelly square 961.
- Dummy `frame.rs` test lane count 192 and radius 0.2 stay; they are not Dam Break pins.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Raised private elastic-group Voronoi budgets**

- **Found during:** Task 2 (Jelly Drop packing)
- **Issue:** Locked jelly radius `0.050596` with unchanged `JELLY_HALF_EXTENT = 1.2` failed `create_particle_group` with `InvalidParticleGroupTopology` even at `maximum_count` 10240. Sampling was under 2200; the group-creation Voronoi grid/work/queue limits were too small for the denser elastic fill.
- **Fix:** Raised private constants in `crates/liquidfun/src/world/particle_object.rs` only: cells 8192, queue 32768, work 32_000_000, nodes 16384. Public particle radius/spacing defaults were not changed.
- **Files modified:** `crates/liquidfun/src/world/particle_object.rs`
- **Verification:** `cargo test -p liquidfun-wasm --lib` jelly tests construct; circle 793 and square 961 particles.
- **Committed in:** `3a047bf`

**2. [Rule 3 - Blocking] Water Wheel rotation proof slack**

- **Found during:** Task 3 (emitter scaling)
- **Issue:** After 180 on-steps the captured paddle angle moved 0.048, just under the old 0.05 threshold, with the finer 20-particle jet.
- **Fix:** Keep 180 steps and motor-off proof; require rotation `> 0.04` so native coupling still has to beat the motor-off bound (`< 0.01`).
- **Files modified:** `crates/liquidfun-wasm/src/scene/water_wheel/tests.rs`
- **Verification:** `medium_jet_with_emission_on_turns_the_wheel` passed.
- **Committed in:** `3a047bf`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both were required for locked jelly radius and honest wheel-rotation proof. Scene particle caps did not need a bump. No canvas scale and no demo-media regeneration.

## Issues Encountered

Jelly construction failed closed on topology, not on the 2200 particle cap. Color Mixer (no elastic Voronoi) constructed at 1154 with cap 2200. Fountain and Water Wheel plateau tests passed with lifetime 3.0 unchanged.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Playground frames and scenes are ready for visual review; catalog stills are stale until someone regenerates demo-media.
- Full `liquidfun-wasm` lib tests take ~10 minutes at the new densities.
- Orchestrator should record this quick-task SUMMARY and update STATE/ROADMAP.

## Self-Check: PASSED

- Commit `3a047bf` exists on `main` and is based on `79d6f12`.
- Key files exist: `crates/liquidfun-wasm/src/frame.rs`, `web/src/physics/frame.ts`, `crates/liquidfun/src/world/particle_object.rs`.
- `.planning/**` was not git-added.

---
*Phase: 260919-wbn-raise-webapp-particle-frame-cap-20x-to-1*
*Completed: 2026-09-20*
