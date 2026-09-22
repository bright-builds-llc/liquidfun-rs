---
phase: 26-catalog-shell-and-basin-scenes
verified: 2026-09-22T01:56:00Z
status: passed
score: 5/5 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-09-22T00-02-36
generated_at: 2026-09-22T01:56:00Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 26: Catalog shell and basin scenes Verification Report

**Phase Goal:** Visitors can use the shared catalog and player with the first watch-first testbed ports—Particles and Liquid Timer—without losing the six existing scenes.
**Verified:** 2026-09-22T01:56:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

The phase goal holds in the live tree. `SCENE_IDS` is eight ready entries (six originals plus `particles` and `liquid-timer`). Both new WASM scenes construct pinned basin/drain layouts on the public `liquidfun` API, expose play/pause/reset only, credit the pinned LiquidFun tests at `7f204021…`, and ship static `Static preview` SVGs. Chromium `just web-player-smoke` completed on HEAD with 38 passed / 0 failed; Vitest contract subset and `liquidfun-wasm` scene unit tests re-ran green during this verification.

Objective UAT (catalog order, credits, factory allowlist, empty-controls hide, e2e watch-first vs interactive split, smoke log) was agent-verified. No checkpoint required human judgment under D-14 / hobby scope (recognizable ports gated by Chromium smoke, not headed visual review).

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Visitor can play, pause, and reset Particles and Liquid Timer, and reset restores each scene's initial layout. | ✓ VERIFIED | `web/e2e/player.spec.ts` watch-first loop over `controls.length === 0` scenes: Pause/Play/Reset via `resetNearZero` (step index restarts below ceiling after dispose remount). Empty `controls: []` in `scenes.ts`; `SceneControls` returns `null` when empty. WASM `apply_control` / `apply_action` reject unknowns; pointer is no-op. Fresh smoke: `target/web-build/web-build.log` ends `complete player-smoke`; `web/test-results/.last-run.json` `"status": "passed"`. |
| 2 | Particles and Liquid Timer each credit the pinned LiquidFun test they port. | ✓ VERIFIED | `scenes.ts` inspiration: `testParticles.js` / `Particles.h` and `testLiquidTimer.js` / `LiquidTimer.h` at commit `7f20402173fd143a3988c921bc384459c6a858f2`; implementation paths `crates/liquidfun-wasm/src/scene/particles.rs` and `liquid_timer.rs` via host-locked `sceneSource`. Locked by `web/tests/scenes.test.ts`. |
| 3 | Visitor can watch Particles: water falls in an open basin and a ball drops into it. | ✓ VERIFIED | `particles.rs` builds floor + slanted walls matching pinned endpoints, water circle `(0,3) r=2`, dynamic ball `(0,8) r=0.5`, radius `0.035`, gravity `(0,-10)`. Unit tests: `create_builds_open_basin_water_and_ball`, `eight_advances_keep_particles_alive` (re-ran ok). Factory: `SceneId::Particles => particles::build`. E2E opens `#/scene/particles` and advances `data-step-index`. |
| 4 | Visitor can watch Liquid Timer: tensile, viscous liquid drains through shelves into bottom columns. | ✓ VERIFIED | `liquid_timer.rs` closed bowl chain, `TENSILE \| VISCOUS` slab, ten shelf/column edges including four bottom columns. Tests: `create_builds_bowl_slab_and_drain_geometry`, `constructed_group_flags_include_tensile_and_viscous`, advances keep particles (re-ran ok). Factory: `SceneId::LiquidTimer => liquid_timer::build`. E2E opens `#/scene/liquid-timer`. |
| 5 | Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel remain available and runnable in the same player. | ✓ VERIFIED | `SCENE_IDS` keeps the six ids first, all `ready: true` with labeled controls. `player.spec.ts` `INTERACTIVE_SCENE_IDS` still gesture + labeled-control path; `POINTER_CONTROL` covers those six. Smoke suite still includes them. |

**Score:** 5/5 truths verified (roadmap success criteria)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/liquidfun-wasm/src/scene/particles.rs` | Open basin + water + ball | ✓ VERIFIED | 307 lines; substantive build + hooks + 4 unit tests |
| `crates/liquidfun-wasm/src/scene/liquid_timer.rs` | Tensile/viscous drain | ✓ VERIFIED | 316 lines; bowl/shelves/slab + 5 unit tests |
| `crates/liquidfun-wasm/src/scene.rs` | SceneId + build_scene arms | ✓ VERIFIED | `Particles` / `LiquidTimer` parse + `particles::build` / `liquid_timer::build` |
| `crates/liquidfun-wasm/src/session.rs` | Allowlist + cap 4 | ✓ VERIFIED | `MAX_ADVANCE_STEPS = 4`; parse tests include both new ids |
| `web/src/catalog/scenes.ts` | Eight-scene catalog + credits | ✓ VERIFIED | Order, `ready: true`, empty controls, pinned credits |
| `web/src/catalog/previews.tsx` | Static SVG previews | ✓ VERIFIED | `case "particles"` / `case "liquid-timer"`; wired via `DemoNavigation` |
| `web/src/player/runtime.ts` | Eight-demo PAGE_SUMMARY | ✓ VERIFIED | "All eight demos run…" |
| `web/src/components/SceneControls.tsx` | Hide when empty | ✓ VERIFIED | `controls.length === 0` → `return null` |
| `web/tests/scenes.test.ts` | Eight-scene Vitest contract | ✓ VERIFIED | Order, hints, empty controls, pinned labels/hrefs |
| `web/scripts/demo-media/model.ts` | Capture plans sync | ✓ VERIFIED | `SCENE_CAPTURE_PLANS` appends both; `assertSceneCapturePlanCoverage` |
| `web/e2e/player.spec.ts` | Interactive vs watch-first | ✓ VERIFIED | Split coverage; watch-first play/pause/reset |
| `web/e2e/player-helpers.ts` | Eight hash paths | ✓ VERIFIED | `SCENE_HASH_PATHS` + `ALL_SCENE_TIMEOUT_MS = 160_000` |
| `web/e2e/shell.spec.ts` | Eight Static previews | ✓ VERIFIED | Desktop `.demo-sidebar` count 8; `.catalog-card` 0 |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `scene.rs` | `particles.rs` | `SceneId::Particles => particles::build` | ✓ WIRED | Pattern in source |
| `particles.rs` | liquidfun API | basin / circle / particle group | ✓ WIRED | `attach_basin_fixture`, `create_particle_group` |
| `scene.rs` | `liquid_timer.rs` | `SceneId::LiquidTimer => liquid_timer::build` | ✓ WIRED | Pattern in source |
| `liquid_timer.rs` | ParticleFlags | `TENSILE \| VISCOUS` | ✓ WIRED | On group recipe |
| `scenes.ts` | WASM SceneId tokens | `particles` / `liquid-timer` | ✓ WIRED | Matches `parse_scene_id` (gsd-tools false-negative: cross-crate) |
| `previews.tsx` | DemoNavigation | `ScenePreview` switch | ✓ WIRED | `DemoNavigation` imports and renders `ScenePreview` |
| `scenes.test.ts` | `scenes.ts` | credit / order asserts | ✓ WIRED | Pattern found |
| `demo-media/model.ts` | `SCENE_IDS` | `assertSceneCapturePlanCoverage` | ✓ WIRED | Import-time assert |
| `player.spec.ts` | catalog controls | `controls.length === 0` | ✓ WIRED | Watch-first branch |
| `justfile` `web-player-smoke` | e2e suite | `bun scripts/web-build.ts player-smoke` | ✓ WIRED | Recipe present; log complete (gsd-tools false-negative: `from` is not a file path) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Catalog / player | `SCENES` / `SCENE_IDS` | Static catalog records | Real metadata + ready flags | ✓ FLOWING |
| Particles WASM | particle count / rigid circles | `create_water_group` + dynamic ball | Unit tests assert count > 0 and circles exported | ✓ FLOWING |
| Liquid Timer WASM | flags + segments | `TENSILE\|VISCOUS` recipe + `SHELF_ENDPOINTS` | Flag and ≥10 segment asserts | ✓ FLOWING |
| Watch-first e2e | `data-step-index` | Live WASM session advance | Polls > 0 then pause/play/reset | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Catalog/Vitest contract | `bunx vitest run tests/scenes.test.ts tests/demo-media-model.test.ts tests/navigation.test.ts` | 21 passed | ✓ PASS |
| Particles WASM tests | `cargo test -p liquidfun-wasm --lib -- particles` | 7 matched tests ok (incl. Particles suite) | ✓ PASS |
| Liquid Timer WASM tests | `cargo test -p liquidfun-wasm --lib -- liquid_timer` | 5 passed | ✓ PASS |
| parse_scene_id allowlist | `cargo test -p liquidfun-wasm --lib -- parse_scene_id` | 2 passed | ✓ PASS |
| Chromium player-smoke (existing evidence) | `just web-player-smoke` log + `.last-run.json` | `complete player-smoke`; `"status":"passed"`; SUMMARY 38/0; Vitest 186 in smoke path | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PLAY-02 | 26-03, 26-04, 26-05 | Play, pause, reset; reset restores initial layout | ✓ SATISFIED | Watch-first e2e + empty controls + remount reset |
| PLAY-03 | 26-03, 26-04 | Credit pinned LiquidFun tests | ✓ SATISFIED | Pinned JS/C++ hrefs + Vitest credit asserts |
| BASIN-01 | 26-01, 26-05 | Particles open basin + water + ball | ✓ SATISFIED | `particles.rs` + unit/e2e/smoke |
| BASIN-02 | 26-02, 26-05 | Liquid Timer tensile/viscous drain | ✓ SATISFIED | `liquid_timer.rs` + unit/e2e/smoke |

No orphaned Phase 26 requirements: REQUIREMENTS.md maps only PLAY-02, PLAY-03, BASIN-01, BASIN-02 to this phase; all appear in plan frontmatter.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TODO/FIXME/placeholder stubs in phase key files | — | None blocking |

Notes (non-blocking): `SIX_SCENE_TIMEOUT_MS` remains a deprecated alias of `ALL_SCENE_TIMEOUT_MS` (intentional compatibility). Watch-first capture plans use a center-click `SceneAction` stub (documented; WASM pointer is no-op).

### Human Verification Required

None. D-14 names Chromium `just web-player-smoke` as the browser gate; construction/recognition are locked by pinned geometry unit tests. Headed aesthetic judgment is out of scope for this hobby phase.

### Gaps Summary

No gaps. All five roadmap success criteria and supporting plan must-haves are present, substantive, wired, and covered by unit + Vitest + recorded smoke evidence.

### Confirmation-bias notes (informational)

1. Reset e2e asserts step-index remount rather than particle-layout checksums; construction unit tests and dispose-on-reset still satisfy PLAY-02 under the Phase 20 remount rule.
2. “Watch” criteria are proven by pinned geometry + advancing sessions, not pixel diffs — intentional per hobby scope / D-14.
3. Automated `verify key-links` reported false negatives for cross-crate / justfile paths; manual wiring checks passed.

---

_Verified: 2026-09-22T01:56:00Z_
_Verifier: Claude (gsd-verifier)_
