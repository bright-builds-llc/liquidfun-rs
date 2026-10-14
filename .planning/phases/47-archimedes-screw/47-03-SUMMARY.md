---
phase: 47-archimedes-screw
plan: "03"
subsystem: playground-catalog
tags: [scene, archimedes-screw, catalog, hud-spinner, preview, portrait-frame, ts-sync, e2e]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: "2026-10-14T22:17:42.822Z"
requires:
  - "47-01: SceneId::ArchimedesScrew, geometry.rs FRAME_* and wall points"
  - "47-02: live screw-speed / screw-direction controls, DEFAULT 30 / MAX 60 rpm"
provides:
  - "archimedes-screw last in Rust ALL_SCENE_IDS, SURVEY_SCENES (SurveyCue::Default) and web SCENE_IDS"
  - "web/src/catalog/archimedes-screw-speed.ts SCREW_SPEED_CONTROL (HUD spinner, 0-60 rpm, default 30)"
  - "Catalog record with locked copy, Direction preset, gravity slider, SHOWCASE credit"
  - "ArchimedesScrewPreview static SVG and its ScenePreview case"
  - "ARCHIMEDES_SCREW_VIEW_BOUNDS (-6.1, -2.0, 2.65, 4.45), reused as the portrait frame"
  - "Watch-first README plan and idle-click demo-media plan"
  - "Rust TS-sync tests web_speed_constants_match_rust and web_view_bounds_match_the_frame"
affects: [47-04, 47-05]
tech-stack:
  added: []
  patterns:
    - "Rust test reads a TS source with include_str! and compares parsed f32 bits against cfg(test) geometry constants"
key-files:
  created:
    - web/src/catalog/archimedes-screw-speed.ts
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs
    - web/src/catalog/scenes.ts
    - web/src/catalog/scene-view-bounds.ts
    - web/src/catalog/scene-records.ts
    - web/src/catalog/scene-records-curiosity.ts
    - web/src/catalog/previews.tsx
    - web/src/catalog/previews-curiosity.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts
    - web/tests/scenes.test.ts
    - web/tests/scene-catalog-controls.test.ts
    - web/tests/scene-catalog-credits.test.ts
    - web/tests/portrait-bounds.test.ts
    - web/tests/controls.test.ts
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
key-decisions:
  - "MIN_HEIGHT_FRACTION[archimedes-screw] = 0.3: the final frame is width-bound on a 390 x 844 canvas and fits 0.3127 of its height (nominal 0.23 assumed the larger research frame)"
  - "The view-bounds sync test parses each TS number and compares f32 bits, not Display substrings, so -2.0 cannot falsely match -2.5"
  - "ALL_SCENE_TIMEOUT_MS unchanged (E5 not needed): the all-scene player loops passed with the extra scene"
requirements-completed: []
duration: 19min
completed: 2026-10-14
---

# Phase 47 Plan 03: Archimedes Screw Catalog Registration Summary

**Archimedes Screw is now the last scene in the playground, in both the Rust and web catalogs. It has the locked honest copy, a live `Screw speed` HUD spinner (0 to 60 rpm, default 30), a `Direction` Forward/Reverse preset and the gravity slider, plus a static preview and a portrait frame that matches the landscape one. Rust tests now fail if the web speed constants or view bounds drift from the Rust constants.**

## Performance

- **Duration:** about 19 min
- **Started:** 2026-10-14T21:59Z
- **Completed:** 2026-10-14T22:17Z
- **Tasks:** 2/2
- **Files:** 1 created, 19 modified

## Accomplishments

- **Rust:** `SceneId::ArchimedesScrew` is appended to `ALL_SCENE_IDS`, and `("archimedes-screw", SceneId::ArchimedesScrew, SurveyCue::Default)` to `SURVEY_SCENES`. The order test, the survey test and the gravity-slider test pass.
- **Web catalog:**
  - `SCENE_IDS` ends with `"archimedes-screw"`.
  - The record follows the diver and uses `withGravitySlider([SCREW_SPEED_CONTROL, runtimePreset("screw-direction", "Direction", ...)])`, `sceneSource("archimedes_screw.rs")` and `[SHOWCASE]`.
  - The UI-SPEC title, description and hint are used verbatim.
- **Speed file:** `archimedes-screw-speed.ts` is a copy of the Drum speed control with `SCREW_SPEED_*` names and the UI-SPEC doc comments.
- **Preview:** `ArchimedesScrewPreview` uses the exact UI-D67 SVG, and `previews.tsx` has a case for it before the `never` guard.
- **Frames:**
  - `ARCHIMEDES_SCREW_VIEW_BOUNDS = { minX: -6.1, minY: -2.0, maxX: 2.65, maxY: 4.45 }` carries the "Keep in sync with `FRAME_*`" docstring. It is re-exported through `scene-records.ts` and `scenes.ts`.
  - `PORTRAIT_VIEW_BOUNDS` reuses the same constant (UI-D72).
- **Scripts:** the README plan is `{ id: "archimedes-screw", cues: [] }`, and the demo-media plan uses the idle-scene click at step 180.
- **Tests:**
  - Locked description, hint, id and title entries.
  - A `"gravity"` recreating id.
  - The credits path, plus a `[SHOWCASE_HREF]` inspiration check.
  - `MIN_HEIGHT_FRACTION` 0.3.
  - Nine portrait wall points.
  - A caption honesty test against the UI-D66 claims.
  - A controls-equality test.
  - A HUD spinner step test: `30 -> 31`, `0 -> 0`. HUD ids are `["screw-speed"]`; panel ids are `["screw-direction", "gravity"]`.
- **e2e:** the screw has a `SCENE_HASH_PATHS` route, and `POINTER_CONTROL` uses `{ gesture: "click", control: "Screw speed" }`.

## Measurements

- **Portrait fit:** at 390 x 844 the fitted frame covers 0.918 of the canvas width and 0.3127 of its height, measured with the test's own `createCamera`/`projectPoint` math. `MIN_HEIGHT_FRACTION["archimedes-screw"] = 0.3`; there is no `MIN_WIDTH_FRACTION` entry.
- **Wall points** (world m), from the final 47-01/47-02 geometry:
  - Reservoir left wall top: (-5.9, 1.6).
  - Reservoir floor outer corner (-5.9, -1.8) and ramp far bottom (2.45, 0.55).
  - Bottom housing far corners (-4.9117, -1.2305) and (-5.1704, 0.2367).
  - Top housing far corners (2.0165, 2.7695) and (1.7578, 4.2367).
  - Weir lip (1.7, 2.05).
  - Chute outer wall top (2.45, 2.8).
- **E5:** `ALL_SCENE_TIMEOUT_MS` is unchanged at 380,000. Both all-scene player loops passed.
- **Line counts** (limit 628):

| File | Lines |
|---|---|
| `web/e2e/player-helpers.ts` | 582 |
| `web/tests/scenes.test.ts` | 550 |
| `web/tests/portrait-bounds.test.ts` | 485 |
| `web/tests/controls.test.ts` | 559 |
| `web/tests/scene-catalog-controls.test.ts` | 430 |
| `web/tests/scene-catalog-credits.test.ts` | 264 |
| `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs` | 283 |

`controls.test.ts` stayed under 628 lines, so it was not split.

## Task Commits

1. **Task 1:** `31cbbc522` feat(47-03): add Archimedes Screw to the playground catalog
1. **Task 2:** `9013df312` test(47-03): cover Archimedes Screw copy, controls, frame and player loop

## Decisions Made

- `MIN_HEIGHT_FRACTION` is 0.3, just below the measured 0.3127. The UI-SPEC's nominal 0.23 came from the research frame; the final frame is shorter relative to its width.
- The view-bounds sync test does not check for `minX: {FRAME_MIN_X}` substrings, because `f32` Display prints `-2` for `-2.0`, and `minY: -2` would also match `-2.5`. Instead, `ts_field` parses each number and the test compares `f32::to_bits`. A mutation check confirmed the test catches drift: changing `maxY` to 4.5 made it fail with `[.., 4.5] vs [.., 4.45]`.
- The speed sync test matches `SCREW_SPEED_DEFAULT = 30;`, including the semicolon, so `300` cannot match.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Float comparison in the view-bounds sync test**

- **Found during:** Task 2, clippy.
- **Issue:** The plan's substring approach is ambiguous for whole-number floats, and a direct `assert_eq!` of `f32` arrays trips `clippy::float_cmp`.
- **Fix:** Parse each field and compare bit patterns.
- **Commit:** `9013df312`.

**2. [Minor] Wall-point choice for "both reservoir floor corners"**

- The outer bottom-left corner (-5.9, -1.8) and the ramp's far bottom corner (2.45, 0.55) mark the floor's extremes. The ramp's far bottom corner replaces the inner ramp-top point listed in 47-01.

No other deviations. The UI-SPEC copy, controls, preview SVG, plans and frames were used exactly as specified.

## Known Stubs

None.

## Deferred Issues

None new. The pre-existing release-only warning in `crates/liquidfun/src/particle/solver/boundary/support.rs:98` is still in `deferred-items.md`.

## Verification

- `cargo fmt --all` passes. `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo build --all-targets --all-features` are clean.
- `cargo test -p liquidfun-wasm --all-features`:
  - Task 1: 679 passed, 0 failed, 52 ignored.
  - Task 2: 681 passed, 0 failed, 52 ignored. This includes the two new sync tests.
- `cargo test -p xtask --bin xtask playground::`: 104 passed.
- `cd web && bun run typecheck && bun run test:unit`: 120 files and 1,852 tests passed.
- `just web-build`, then `CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts e2e/shell.spec.ts --project chromium`: 98 passed. This includes the archimedes-screw HUD safe-frame cases at 390x844, 1280x800, 844x390 and 768x1024.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `git diff --quiet HEAD~2 -- docs/benchmarks/scene-fingerprints.json crates/liquidfun`: unchanged.

## Self-Check: PASSED

The speed file, this SUMMARY, and commits 31cbbc522 and 9013df312 exist.
