---
phase: 48-fake-ferrofluid
plan: "03"
subsystem: playground-catalog
tags: [scene, fake-ferrofluid, catalog, hud-spinner, percent, portrait-frame, readme-plan, e2e, r14-sync]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: "2026-10-15T06:41:00.000Z"
requires:
  - "48-01: SceneId::FakeFerrofluid parse/build, FRAME_* and geometry"
  - "48-02: magnet-strength live control, magnet action, pointer seam, 3.0 m walls (FRAME_MAX_Y 3.6)"
provides:
  - "fake-ferrofluid last in Rust ALL_SCENE_IDS, SURVEY_SCENES (SurveyCue::Default) and web SCENE_IDS"
  - "web/src/catalog/fake-ferrofluid-strength.ts: MAGNET_STRENGTH_CONTROL (0..100, default 65, step 1, unit %, HUD spinner)"
  - "Catalog record with locked copy, controls withGravitySlider([MAGNET_STRENGTH_CONTROL, action(magnet, Magnet)]), SHOWCASE credit"
  - "FAKE_FERROFLUID_VIEW_BOUNDS (-3.6, -0.8, 3.6, 3.6), reused as the portrait frame"
  - "FakeFerrofluidPreview and FERRO #8A84AD in preview-kit"
  - "README plan (magnet cues at samples 120 and 160), demo-media plan, e2e hash path and POINTER_CONTROL drag + Magnet"
  - "formatRangeValueText speaks % as percent"
  - "scene/fake_ferrofluid/sync_tests.rs: 3 R14 include_str! tests"
affects: [48-04, 48-05]
tech-stack:
  added: []
  patterns:
    - "R14 sync test per scene: parse the TS constants and view bounds with include_str! and compare f32 bits"
key-files:
  created:
    - web/src/catalog/fake-ferrofluid-strength.ts
    - web/tests/fake-ferrofluid-controls.test.ts
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/sync_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs
    - web/src/catalog/scenes.ts
    - web/src/catalog/scene-view-bounds.ts
    - web/src/catalog/scene-records.ts
    - web/src/catalog/scene-records-curiosity.ts
    - web/src/catalog/preview-kit.tsx
    - web/src/catalog/previews.tsx
    - web/src/catalog/previews-curiosity.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/src/components/range-control.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts
    - web/tests/scenes.test.ts
    - web/tests/scene-catalog-controls.test.ts
    - web/tests/scene-catalog-credits.test.ts
    - web/tests/portrait-bounds.test.ts
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
key-decisions:
  - "FAKE_FERROFLUID_VIEW_BOUNDS maxY is 3.6 (48-02 FRAME_MAX_Y after the 3.0 m walls), superseding the plan's nominal 2.4; portrait wall tops use y 3.0"
  - "Fake Ferrofluid portrait MIN_HEIGHT_FRACTION 0.25 (measured fit 0.259; the frame is width-bound at 0.918 of the width)"
  - "README cue samples stay at 120 and 160 (48-02 native timing passed with margin)"
  - "Ferrofluid control tests live in the new web/tests/fake-ferrofluid-controls.test.ts; controls.test.ts is unchanged at 559 lines"
requirements-completed: []
requirements-advanced: [FERRO-01, FERRO-02, FERRO-03]
duration: 17min
completed: 2026-10-15
---

# Phase 48 Plan 03: Fake Ferrofluid Catalog Registration Summary

**Fake Ferrofluid now appears last in the playground catalog in Rust and on the web. It has the locked honest copy, a live `Magnet strength` HUD spinner that screen readers hear as "{n} percent", a `Magnet` panel action, the gravity slider, a seven-spike static preview and a phone frame that fits the 3.0 m dish. Rust tests compare the web constants and frame with the Rust values bit for bit, and the e2e player loop drives a drag plus the Magnet button.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-10-15T06:24:31Z
- **Completed:** 2026-10-15T06:41:00Z
- **Tasks:** 2
- **Files modified:** 23 (3 created, 20 modified)

## Accomplishments

- **Catalog in one commit (Task 1).**
  - Rust: `SceneId::FakeFerrofluid` is appended to `ALL_SCENE_IDS`, and `("fake-ferrofluid", SceneId::FakeFerrofluid, SurveyCue::Default)` to `SURVEY_SCENES`. Web: `"fake-ferrofluid"` is last in `SCENE_IDS`.
  - The record uses the exact UI-SPEC title, description, hint, controls and credits.
  - New pieces: the strength file (`MAGNET_STRENGTH_*`), `FAKE_FERROFLUID_VIEW_BOUNDS`, re-exported through `scene-records.ts` and `scenes.ts`, `FERRO`, `FakeFerrofluidPreview` (the exact UI-D79 SVG), the portrait entry, the README plan, the demo-media plan, and the e2e `SCENE_HASH_PATHS`.
  - The required test entries were added in the same commit.
- **Tests (Task 2).**
  - `formatRangeValueText` has a `%` branch that returns "{n} percent".
  - The new `fake-ferrofluid-controls.test.ts` checks the percent readout and spoken value, and the HUD/panel split. It also checks that the spinner steps 65 to 66 and clamps at 0 and 100.
  - `scenes.test.ts` gets the UI-D77 `FERRO_FORBIDDEN_CLAIMS` guard. `scene-catalog-controls.test.ts` checks the exact control list. `scene-catalog-credits.test.ts` checks the SHOWCASE credit. `portrait-bounds.test.ts` gets the eight wall points.
  - e2e `POINTER_CONTROL` is `{ gesture: "drag", control: "Magnet" }`.
  - The Rust R14 `sync_tests.rs` adds three tests.

## Frame and portrait numbers

- `FAKE_FERROFLUID_VIEW_BOUNDS` = `{ minX: -3.6, minY: -0.8, maxX: 3.6, maxY: 3.6 }`. It equals `FRAME_*` in `geometry.rs` (`FRAME_MAX_Y = WALL_TOP_Y + 0.6 = 3.6`).
- Fitted on a 390 x 844 canvas using the test's own camera fit: width 0.918, height **0.259**. `MIN_HEIGHT_FRACTION["fake-ferrofluid"] = 0.25`, with no `MIN_WIDTH_FRACTION` entry. As a cross-check, the same script reproduces the screw's recorded 0.313.
- Wall points: `(-3.3, 3.0)`, `(-3.0, 3.0)`, `(3.0, 3.0)`, `(3.3, 3.0)`, `(-3.3, -0.25)`, `(3.3, -0.25)`, `(-0.4, -0.45)` and `(0.4, -0.45)`. All are inside the portrait frame.

## Task Commits

1. **Task 1: Catalog registration across Rust and web**: `1f6aab234` (feat)
1. **Task 2: Honesty, controls, percent, frame and TS-sync tests plus e2e**: `f9d32df6f` (test). The percent test was written first and ran RED ("65 %" received), then turned GREEN.

## Decisions Made

See the key-decisions frontmatter.

## Deviations from Plan

### Superseded plan values (orchestrator instruction, from 48-02)

**1. View bounds and wall points follow the 3.0 m walls**
- **Found during:** Task 1
- **Issue:** The plan text uses the 48-01 nominal `maxY 2.4` and wall tops at y 1.8. 48-02 raised `WALL_TOP_Y` to 3.0 and `FRAME_MAX_Y` to 3.6.
- **Fix:** `FAKE_FERROFLUID_VIEW_BOUNDS.maxY` is 3.6, and the four wall-top points use y 3.0. The values were read from `geometry.rs`. The R14 test proves they match: a mutation back to 2.4 failed `web_view_bounds_match_the_frame` with `[-3.6, -0.8, 3.6, 2.4] vs [-3.6, -0.8, 3.6, 3.6]`, and the file was then restored.
- **Consequence:** the plan's nominal `MIN_HEIGHT_FRACTION` of about 0.18 became 0.25 (measured 0.259).

Otherwise the plan was executed as written. No auto-fixes were needed.

### Notes

- Caption: the copy says only "A scripted magnetic pull lifts the liquid into the spikes". It makes no claim about the `|E(h)|` sideways term or about any field, and the UI-D77 guard passes.
- E5: `ALL_SCENE_TIMEOUT_MS` stays at 380,000. The all-scene loops passed with the extra scene, so it was not raised.
- `docs/benchmarks/scene-fingerprints.json` and `crates/liquidfun` are unchanged.

## File line counts (limit 628)

`web/e2e/player-helpers.ts` 583, `web/e2e/player.spec.ts` 453, `web/tests/scenes.test.ts` 587 (under the 615 move threshold, so the guard stays there), `web/tests/portrait-bounds.test.ts` 498, `web/tests/controls.test.ts` 559 (unchanged), `web/tests/scene-catalog-controls.test.ts` 448, `web/tests/scene-catalog-credits.test.ts` 266, `web/tests/fake-ferrofluid-controls.test.ts` 56, `web/src/components/range-control.ts` 256, `fake_ferrofluid/sync_tests.rs` 111, `fake_ferrofluid.rs` 282.

## Verification

- `cargo fmt --all --check`, `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo build --all-targets --all-features` all pass.
- `cargo test -p liquidfun-wasm --all-features`: 744 passed, 81 ignored (741 after Task 1). `cargo test -p xtask --bin xtask playground::`: 104 passed.
- `cargo test -p liquidfun-wasm --all-features --lib fake_ferrofluid::sync_tests`: 3 passed.
- `bun run typecheck` passes. `bun run test:unit` gives 121 files and 1856 tests passed.
- `just web-build`, then `CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts e2e/shell.spec.ts --project chromium` gives **102 passed**. That count includes the four fake-ferrofluid HUD safe-frame cases (390x844, 1280x800, 844x390, 768x1024), and the interactive player loop now includes the drag and the Magnet button.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.

## Known Stubs

None.

## Next Phase Readiness

- Plan 04 still has to run `just readme-svg` for this scene, review the clip at cue samples 120 and 160, and do the four-mode color check.
- Plan 05 then runs the fingerprint gate, records scene-spot, and grows the baseline in its own commit.

## Self-Check: PASSED
