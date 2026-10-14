---
phase: 46-cartesian-diver
plan: "02"
subsystem: playground-catalog
tags: [scene, cartesian-diver, catalog, preview, portrait-frame, readme-plan, e2e]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: "2026-10-14T19:20:00Z"
requires:
  - "46-01: SceneId::CartesianDiver scene module, geometry and README cue samples 20/110"
provides:
  - "cartesian-diver last in Rust ALL_SCENE_IDS, SURVEY_SCENES (SurveyCue::Action(\"squeeze\")) and web SCENE_IDS"
  - "web/src/catalog/scene-records-curiosity.ts CURIOSITY_SCENE_RECORDS (new curiosity family)"
  - "web/src/catalog/previews-curiosity.tsx CartesianDiverPreview"
  - "CARTESIAN_DIVER_VIEW_BOUNDS, reused as the portrait frame"
  - "README plan (squeeze at samples 20 and 110) and demo-media capture plan"
  - "Unit and e2e coverage: honest caption, wall points, cues, controls, credits, pointer loop, HUD safe frame"
affects: [46-03, 46-04, 46-05]
tech-stack:
  added: []
  patterns:
    - "New scene family module appended to the SCENES spread (curiosity after machines)"
key-files:
  created:
    - web/src/catalog/scene-records-curiosity.ts
    - web/src/catalog/previews-curiosity.tsx
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/scene_spot/tests.rs
    - web/src/catalog/scenes.ts
    - web/src/catalog/scene-view-bounds.ts
    - web/src/catalog/scene-records.ts
    - web/src/catalog/previews.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts
    - web/tests/scenes.test.ts
    - web/tests/scene-catalog-controls.test.ts
    - web/tests/scene-catalog-credits.test.ts
    - web/tests/portrait-bounds.test.ts
    - web/tests/readme-svg.test.ts
    - web/e2e/player-helpers.ts
    - web/e2e/player.spec.ts
key-decisions:
  - "46-02: the demo-media capture plan for cartesian-diver uses a center canvas click at (0.5, 0.5), a brief squeeze, because SceneCaptureAction has only click/drag kinds (D-17 deviation); README plan and survey keep the named squeeze action"
  - "46-02: UI-SPEC copy strings kept verbatim; the caption describes default behavior and makes no promise about where the diver floats at high gravity"
requirements-completed: [SCENE-01, SCENE-03, DIVER-01]
duration: about 30 min
completed: 2026-10-14
---

# Phase 46 Plan 02: Cartesian Diver Catalog Registration Summary

**Cartesian Diver is now the last playground scene in both the Rust and web catalogs. It ships with the locked UI-SPEC copy, a Squeeze action plus the gravity slider, a static preview of the squeezed bottle, a bottle-sized landscape and portrait frame, and README cues at samples 20 and 110. Unit and Chromium e2e tests cover the honest caption, phone frame, controls, credits, pointer loop and HUD safe frame.**

## Performance

- **Duration:** about 30 min
- **Started:** 2026-10-14T18:47Z
- **Completed:** 2026-10-14T19:20Z
- **Tasks:** 2 of 2
- **Files:** 2 created, 17 modified

## Accomplishments

- **Task 1 (843cbd152):** One green commit registers the scene everywhere. Rust: `SceneId::CartesianDiver` is appended to `ALL_SCENE_IDS`, the survey entry uses `SurveyCue::Action("squeeze")`, and `only_idle_scenes_are_scripted` now includes `cartesian-diver`. Web: the id is appended to `SCENE_IDS`, along with the new curiosity family record, preview, `CARTESIAN_DIVER_VIEW_BOUNDS` (`-1.6, -0.4, 1.6, 6.0`), which is re-exported and reused as the portrait frame, the README and demo-media plans, and the required test and e2e route entries.
- **Task 2 (5cbb05f48):** Added the UI-D58 honesty test and the `"Squeeze"` locked label. Added the six portrait wall points, the README squeeze-cue test, the Squeeze-before-gravity control test (`recreates: false`), the SHOWCASE inspiration assertion, and the `POINTER_CONTROL` entry so the e2e interactive loop clicks the canvas and activates Squeeze.

## Verification

- `cargo fmt --all`; `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` was clean.
- `cargo test -p liquidfun-wasm --all-features`: 634 passed, 36 ignored. This includes the catalog-order, survey and gravity-slider tests.
- `cargo test -p xtask --bin xtask playground::`: 104 passed.
- `bun run typecheck` passed. `bun run test:unit` reached 1847 passed after Task 2 (1844 after Task 1).
- `just web-build`, then `CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts e2e/shell.spec.ts --project chromium`: 94 passed in 2.3 min. This includes `cartesian-diver at 390x844` and `at 1280x800 keeps the world clear of HUD chrome`, and the interactive-scene loop with the (0.5, 0.45) canvas click.
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- `docs/benchmarks/scene-fingerprints.json` and `crates/liquidfun` are unchanged (`git diff --quiet HEAD~2`).

## E5 and file lengths

- `ALL_SCENE_TIMEOUT_MS` is unchanged at 380,000. The all-scene loops passed with the extra scene.
- Line counts: `web/e2e/player-helpers.ts` 581, `web/tests/scenes.test.ts` 516, `web/tests/portrait-bounds.test.ts` 471. All are within the 628 limit.

## Decisions Made

- The curiosity family lives in its own module and is spread after `...MACHINE_SCENE_RECORDS`, as UI-SPEC specifies.
- The diver control test is a separate `it` block, so each test checks one concern, rather than an extra assertion in the large UI-SPEC controls test.

## Deviations from Plan

### Documented deviation (D-17 demo media)

**1. D-17 demo media uses a canvas click instead of the named `squeeze` action**
- D-17 says the demo-media `SCENE_CAPTURE_PLANS` entry uses the same `squeeze` action. However, `SceneCaptureAction` supports only `click` and `drag` kinds, and adding a new kind is out of scope.
- The diver capture plan therefore uses a center canvas click at (0.5, 0.5). This point is inside the bottle hit region and produces a brief squeeze and release.
- The README plan (`squeeze` at samples 20 and 110) and the survey (`SurveyCue::Action("squeeze")`) still use the named action, as D-17 specifies.
- Commit: 843cbd152

Otherwise the plan was executed as written. No auto-fixes were needed, and the UI-SPEC copy strings are unchanged.

## Known Stubs

None.

## Threat Flags

None. No new network, storage or input surface was added. The hash route is still matched against the static `SCENE_IDS` allowlist.

## Next Phase Readiness

- S2 (`just readme-svg` for `docs/assets/readme/cartesian-diver-10s.svg` and WebP) and the scene-fingerprint baseline are left to later plans. The fingerprint baseline grows only in its own commit.

## Self-Check: PASSED

- FOUND: web/src/catalog/scene-records-curiosity.ts, web/src/catalog/previews-curiosity.tsx
- FOUND: commit 843cbd152 (Task 1), commit 5cbb05f48 (Task 2)
