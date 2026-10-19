---
phase: 49-rayleigh-taylor-instability
plan: "03"
subsystem: playground-catalog
tags: [scene, rayleigh-taylor, catalog, preset-control, portrait-frame, r14-sync, e2e]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: "2026-10-19T02:45:48Z"
requires:
  - "49-01: SceneId::RayleighTaylor, WEIGHT_RATIO_CONTROL, WeightRatio::parse, WEIGHT_RATIO_TOKENS, DEFAULT_WEIGHT_RATIO_TOKEN, FRAME_* {-2.0, -0.6, 2.0, 2.2}"
  - "49-02: D-18 outcome tests (TUNING unchanged)"
provides:
  - "rayleigh-taylor last in ALL_SCENE_IDS, SURVEY_SCENES (SurveyCue::Default) and web SCENE_IDS"
  - "web/src/catalog/rayleigh-taylor-ratio.ts: WEIGHT_RATIO_CONTROL (recreating preset 1.2x/1.5x/2x/3x), DEFAULT_PRESET_VALUES weight-ratio = 2"
  - "Curiosity record with locked UI-SPEC copy, withGravitySlider([WEIGHT_RATIO_CONTROL]), SHOWCASE credits"
  - "RAYLEIGH_TAYLOR_VIEW_BOUNDS reused as the portrait frame; RayleighTaylorPreview with RT_HEAVY/RT_LIGHT"
  - "Watch-first README plan (no cues) and demo-media plan; e2e SCENE_HASH_PATHS entry, no POINTER_CONTROL"
  - "web/tests/rayleigh-taylor.test.ts honesty guard and Weight ratio shape; rayleigh_taylor/sync_tests.rs R14 sync"
affects: [49-04, 49-05]
tech-stack:
  added: []
  patterns:
    - "Measure MIN_HEIGHT_FRACTION with the real camera (viewport inset) instead of the bounds aspect"
key-files:
  created:
    - web/src/catalog/rayleigh-taylor-ratio.ts
    - web/tests/rayleigh-taylor.test.ts
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/sync_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor.rs
    - web/src/catalog/scenes.ts
    - web/src/catalog/scene-view-bounds.ts
    - web/src/catalog/scene-records.ts
    - web/src/catalog/scene-records-curiosity.ts
    - web/src/catalog/preview-kit.tsx
    - web/src/catalog/previews.tsx
    - web/src/catalog/previews-curiosity.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/src/components/scene-controls.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts
    - web/tests/scenes.test.ts
    - web/tests/scene-catalog-controls.test.ts
    - web/tests/scene-catalog-credits.test.ts
    - web/tests/portrait-bounds.test.ts
    - web/e2e/player-helpers.ts
key-decisions:
  - "MIN_HEIGHT_FRACTION[rayleigh-taylor] = 0.29: the camera-fitted share on 390x844 is 0.2969 (viewport inset), so the 0.3 from 49-01-SUMMARY would fail"
  - "Static preview follows the 49-01-updated UI-SPEC composition (tank x 45-115, y 25-65), not the plan's older 2.8 x 3.2 m coordinates"
  - "Portrait comment uses the updated UI-D94 wording: the wide closed tank sets the width"
patterns-established:
  - "R14 option-id sync: parse every option(\"<id>\", ...) call in the TS control file and compare in order with the Rust token list"
requirements-completed: []
duration: "~14 min"
completed: 2026-10-18
---

# Phase 49 Plan 03: Rayleigh–Taylor Catalog Registration Summary

**Rayleigh–Taylor Instability now appears last in both catalogs. It has the locked, honest caption, a recreating `Weight ratio` preset (1.2×, 1.5×, 2×, 3×; default 2×) before the gravity slider, a closed-tank preview in orange and blue, and a landscape and portrait frame that matches Rust `FRAME_*`. A Rust `include_str!` test fails if the web frame, ratio tokens or default drift from Rust.**

## Performance

- **Duration:** about 14 min
- **Started:** 2026-10-19T02:31:31Z
- **Completed:** 2026-10-19T02:45:48Z
- **Tasks:** 2 of 2
- **Files modified:** 22 (3 created, 19 modified)

## Accomplishments

- Rust: `SceneId::RayleighTaylor` was appended to `ALL_SCENE_IDS`, and `("rayleigh-taylor", SceneId::RayleighTaylor, SurveyCue::Default)` to `SURVEY_SCENES`. The order test, the survey test and the gravity-slider tests pass.
- Web catalog: the id was added to `SCENE_IDS`, and the record follows the ferrofluid with the exact UI-SPEC title, description, hint and credits (`sceneSource("rayleigh_taylor.rs")`, `[SHOWCASE]`). The new control file holds `WEIGHT_RATIO_CONTROL`, and `DEFAULT_PRESET_VALUES["weight-ratio"]` is `"2"`. `RAYLEIGH_TAYLOR_VIEW_BOUNDS` `{ -2.0, -0.6, 2.0, 2.2 }` is re-exported through `scene-records.ts` and `scenes.ts`, and `PORTRAIT_VIEW_BOUNDS` uses it too.
- Preview: `RT_HEAVY` `#FB923C` and `RT_LIGHT` `#3B82F6` were added to `preview-kit.tsx`. `RayleighTaylorPreview` draws the UI-SPEC composition: the wide tank, three fingers with caps and two rising bubbles. Its case comes last in the `ScenePreview` switch.
- Scripted media: the README plan is `{ id: "rayleigh-taylor", cues: [] }`, and the demo-media plan uses the idle click at step 180. Both come last. The e2e hash path is added, with no `POINTER_CONTROL` entry, so the scene runs in the watch-first loop.
- Tests: `rayleigh-taylor.test.ts` has the UI-D90 honesty guard and the Weight ratio shape test (panel `["weight-ratio", "gravity"]`, HUD `[]`). The controls test checks `[WEIGHT_RATIO_CONTROL, GRAVITY_CONTROL]`, and the credits test checks the SHOWCASE inspiration. The portrait test checks the eight tank wall points. The Rust `sync_tests.rs` has three R14 tests.

## Task Commits

1. **Task 1: catalog registration across Rust and web**: `2cc80ef90` (feat)
1. **Task 2: honesty, Weight ratio shape, credits, frame and TS-sync tests, plus the e2e runs**: `9fdd8ccee` (test)

**Plan metadata:** the docs commit that follows this SUMMARY

## Recorded Values

| Item | Value |
|------|-------|
| `MIN_HEIGHT_FRACTION["rayleigh-taylor"]` | **0.29**; measured fitted share on 390 × 844 is 0.2969 (width share 0.918) from `worldBoundsForViewport` + `createCamera` + `projectPoint`, the same path as `fittedFrameFraction` |
| E5 `ALL_SCENE_TIMEOUT_MS` | unchanged; the watch-first and interactive all-scene loops passed within the existing timeout |
| Portrait wall points | `(±1.7, −0.3)`, `(±1.7, 1.9)`, `(±1.4, 0)`, `(±1.4, 1.6)`, all inside the fitted frame |
| Line counts | `e2e/player-helpers.ts` 584, `tests/scenes.test.ts` 593, `tests/portrait-bounds.test.ts` 511, `tests/scene-catalog-controls.test.ts` 463, `tests/rayleigh-taylor.test.ts` 60 (all ≤ 628) |

## Verification

- `cargo fmt --all --check`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo build --all-targets --all-features` all pass.
- `cargo test -p liquidfun-wasm --all-features`: 807 passed, 0 failed, 102 ignored (release-only guards). `cargo test -p xtask --bin xtask playground::`: 104 passed.
- `rayleigh_taylor::sync_tests`: 3 passed. As a red check, I temporarily set the TS default to `"1.5"` and `maxY` to `2.5`. `web_default_weight_ratio_matches_rust` and `web_view_bounds_match_the_frame` both failed, and I restored the files before committing.
- `bun run typecheck` passes. `bun run test:unit`: 122 files, 1,859 tests passed.
- `just web-build`, then `CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts e2e/shell.spec.ts --project chromium`: 104 passed. That includes both `rayleigh-taylor` HUD safe-frame cases (390×844 and 1280×800, the two the spec generates for a scene without HUD controls), plus the watch-first pause, play and reset loop.
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- `git diff --quiet HEAD~2 -- docs/benchmarks/scene-fingerprints.json crates/liquidfun` is clean. README SVG/WebP were not regenerated (that is Plan 04).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `MIN_HEIGHT_FRACTION` 0.3 would fail**
- **Found during:** Task 1
- **Issue:** 49-01-SUMMARY and the orchestrator note give 0.3 (from 2.8 / 4.0 × 390 / 844 ≈ 0.323). The camera's viewport inset brings the real fitted share down to 0.2969, which is below 0.3.
- **Fix:** I measured it with the test's own camera path and set 0.29, the value rounded down to two decimals that the plan's rule calls for. The test comment records 0.297.
- **Files modified:** web/tests/portrait-bounds.test.ts
- **Commit:** `2cc80ef90`

**2. [Superseded nominal] Preview coordinates and portrait comment follow the updated UI-SPEC**
- **Issue:** The plan's `<facts>` preview SVG (tank `x="52" y="12" width="56" height="64"`, caps at `cy="61"`) and its "portrait-friendly" comment describe the old 2.8 × 3.2 m tank. Plan 01 redrew both in 49-UI-SPEC.md for the 2.8 × 1.6 m tank.
- **Fix:** I used the UI-SPEC composition exactly (tank `x="43.5" y="23.5" width="73" height="43"`, caps `cy="55"/"57"`, bubbles `cx="73"/"87"`) and the UI-D94 comment "The wide closed tank sets the width; a tighter frame would clip a wall." The plan's 3-count preview grep, run against the UI-SPEC coordinates, also returns 3.
- **Commit:** `2cc80ef90`

**3. [TDD note] RED phase**
- Task 1 had to land the implementation in one green commit, so Task 2's tests passed on their first run. In place of a RED commit, the mutation check above shows the Rust sync tests are meaningful.

## Known Stubs

None.

## Threat Flags

None. The hash route uses the static `SCENE_IDS` allowlist, the select sends only the four declared ids, and the credits path goes through `links.ts`. All of these match the plan's threat register (T-49-09..T-49-12).

## Self-Check: PASSED

- Created files exist: `web/src/catalog/rayleigh-taylor-ratio.ts`, `web/tests/rayleigh-taylor.test.ts`, `crates/liquidfun-wasm/src/scene/rayleigh_taylor/sync_tests.rs`.
- Commits `2cc80ef90` and `9fdd8ccee` are in `git log`.
