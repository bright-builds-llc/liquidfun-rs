---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T05:45:00Z
phase: 27-material-flag-groups
plan: "01"
subsystem: wasm-scenes
tags: [surface-tension, tensile, color-mixing, basin-family, liquidfun-wasm, tdd]

requires:
  - phase: 26-catalog-shell-and-basin-scenes
    provides: Watch-first SceneId factory, SessionCore, Particles/Liquid Timer patterns
provides:
  - Surface Tension WASM scene with TENSILE|COLOR_MIXING groups
  - Private vertical-wall basin_family helper for Elastic/Rigid reuse
  - surface-tension allowlist token in parse_scene_id
affects:
  - 27-02 elastic-particles
  - 27-03 rigid-particles
  - 27-04 catalog append

tech-stack:
  added: []
  patterns:
    - Private basin_family helper for shared vertical-wall basin + falling ball
    - Flag-recipe headless asserts for TENSILE|COLOR_MIXING after SessionCore::create

key-files:
  created:
    - crates/liquidfun-wasm/src/scene/surface_tension.rs
    - crates/liquidfun-wasm/src/scene/basin_family.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/session/tests.rs
    - crates/liquidfun-wasm/src/scene/particles.rs
    - crates/liquidfun-wasm/src/scene/liquid_timer.rs

key-decisions:
  - "Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks"
  - "Private basin_family helper with vertical walls ending at y=2 for Surface Tension reuse by Elastic/Rigid"

patterns-established:
  - "Pattern: basin_family attach_vertical_wall_basin + create_falling_ball for material flag-group scenes"
  - "Pattern: Surface Tension TENSILE|COLOR_MIXING on all three groups with damping 0.2 and radius 0.035"

requirements-completed: [MAT-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T05:40:40Z

duration: 5min
completed: 2026-09-22
---

# Phase 27 Plan 01: Surface Tension Scene Summary

**Pinned Surface Tension WASM scene with vertical-wall basin, three TENSILE|COLOR_MIXING color groups, falling ball, and private basin_family helper**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-22T05:35:21Z
- **Completed:** 2026-09-22T05:40:40Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Surface Tension constructs through `SceneId::SurfaceTension` / `"surface-tension"` with basin, three tensile color-mixing groups, and a falling dynamic ball
- Headless tests assert `TENSILE | COLOR_MIXING`, particle survival under capped advance, unknown-control rejection, and pointer no-op
- Private `basin_family` helper shares the vertical-wall basin (`y=2` tops) and ball spawn for later Elastic/Rigid plans
- `MAX_ADVANCE_STEPS` remains 4; no catalog/TS edits; no engine API changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing Surface Tension allowlist and flag tests** - `a596bdd` (test)
2. **Task 2: Implement basin helper, Surface Tension scene, and factory wiring** - `8ce29c9` (feat)

**Plan metadata:** `c172160` (docs: complete plan)

_Note: TDD tasks used RED stub then GREEN implementation_

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene/surface_tension.rs` - Pinned Surface Tension scene + headless flag/construction tests
- `crates/liquidfun-wasm/src/scene/basin_family.rs` - Shared vertical-wall basin + falling ball helpers
- `crates/liquidfun-wasm/src/scene.rs` - `SceneId::SurfaceTension`, parse, build arm, `mod basin_family`
- `crates/liquidfun-wasm/src/session/tests.rs` - Allowlist entry for `surface-tension`
- `crates/liquidfun-wasm/src/scene/particles.rs` - Clippy `doc_markdown` backtick fix (blocking)
- `crates/liquidfun-wasm/src/scene/liquid_timer.rs` - Clippy `doc_markdown` backtick fix (blocking)

## Decisions Made

- Wired a minimal `SceneId::SurfaceTension` stub in Task 1 so RED tests compile under sequential hooks instead of a compile-fail-only RED
- Extracted `basin_family` for the pinned vertical-wall geometry (Claude's Discretion / D-07) rather than duplicating basin polygons in Surface Tension alone

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Minimal SceneId wiring during RED**
- **Found during:** Task 1 (failing Surface Tension tests)
- **Issue:** Plan expected compile-fail until Task 2, but sequential commits with hooks need a compiling crate for RED verification
- **Fix:** Added enum variant, parse token, build arm, and stub `build` returning `SceneConstruction` so allowlist passes and behavioral tests fail
- **Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/scene/surface_tension.rs`
- **Verification:** Five `surface_tension` tests failed on construction; allowlist test passed
- **Committed in:** `a596bdd`

**2. [Rule 3 - Blocking] Clippy doc_markdown on Particles and Liquid Timer**
- **Found during:** Task 2 (clippy `-D warnings`)
- **Issue:** Pre-existing `LiquidFun` module docs without backticks blocked crate clippy
- **Fix:** Backticked `` `LiquidFun` `` in those two module docs (same fix applied to Surface Tension)
- **Files modified:** `crates/liquidfun-wasm/src/scene/particles.rs`, `crates/liquidfun-wasm/src/scene/liquid_timer.rs`
- **Verification:** `cargo clippy -p liquidfun-wasm --all-targets -- -D warnings` exits 0
- **Committed in:** `8ce29c9`

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Necessary for TDD under hooks and clippy gate; no scope creep into catalog or engine APIs

## Issues Encountered

None beyond the documented deviations.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for plan 27-02 (Elastic Particles) to reuse `basin_family`
- Catalog/TS advertising of Surface Tension remains deferred to later Phase 27 plans
- No sealed C++ parity claim

## Self-Check: PASSED

- FOUND: `crates/liquidfun-wasm/src/scene/surface_tension.rs`
- FOUND: `crates/liquidfun-wasm/src/scene/basin_family.rs`
- FOUND: commit `a596bdd`
- FOUND: commit `8ce29c9`

***
*Phase: 27-material-flag-groups*
*Completed: 2026-09-22*
