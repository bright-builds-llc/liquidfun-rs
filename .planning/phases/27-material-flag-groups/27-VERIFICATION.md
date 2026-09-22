---
phase: 27-material-flag-groups
verified: 2026-09-22T06:21:06Z
status: passed
score: 4/4 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-09-22T05-14-55
generated_at: 2026-09-22T06:21:06Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 27: Material flag groups Verification Report

**Phase Goal:** Visitors can watch the three material showcase scenes—Surface Tension, Elastic Particles, and Rigid Particles—as recognizable flag-group ports beside the existing demos.
**Verified:** 2026-09-22T06:21:06Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

The phase goal holds in the live tree. `SCENE_IDS` is eleven ready entries (eight prior scenes plus `surface-tension`, `elastic-particles`, `rigid-particles`). Each new WASM scene builds a vertical-wall basin, three colored flag-group clumps, and a falling ball on the public `liquidfun` API; catalog records are `ready: true` with empty watch-first controls, pinned credits at `7f204021…`, and `Static preview` SVGs. Chromium `just web-player-smoke` completed during plan 27-06 with 38 passed / 0 failed; focused WASM material unit tests (19) and Vitest catalog subset (17) re-ran green during this verification.

Objective UAT (catalog order, credits, factory allowlist, empty-controls hide, e2e watch-first vs interactive split, smoke log) was agent-verified. No checkpoint required human judgment under D-16 / hobby scope (recognizable ports gated by Chromium smoke + flag-recipe unit tests, not headed visual review). Precedence: Phase 26 VERIFICATION used the same gate for BASIN watch criteria.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Visitor can open, play, pause, and reset Surface Tension, Elastic Particles, and Rigid Particles from the catalog, with credits to the pinned LiquidFun tests. | ✓ VERIFIED | Catalog: three `ready: true` records with `controls: []`, `WATCH_FIRST_HINT`, host-locked `sceneSource(*.rs)`, inspiration JS+H at commit `7f20402173fd143a3988c921bc384459c6a858f2`. E2E: `WATCH_FIRST_SCENE_IDS` includes all `controls.length === 0` scenes; play/pause/reset without pointer. Shell: eleven Static preview captions; `.catalog-card` 0; drawer last link Rigid Particles. Smoke evidence: `target/web-build/web-build.log` ends `complete player-smoke`; `web/test-results/.last-run.json` `"status": "passed"`; SUMMARY 38/0. |
| 2 | Visitor can watch Surface Tension: three colored tensile groups bead and bleed color when a ball hits them. | ✓ VERIFIED | `surface_tension.rs` (302 lines): shared basin + red/green circles + blue box with `TENSILE \| COLOR_MIXING`, dynamic ball. Tests: construction, flag recipe, capped advances keep particles, watch-first control/pointer rejection. Factory: `SceneId::SurfaceTension => surface_tension::build`. E2E opens `#/scene/surface-tension`. |
| 3 | Visitor can watch Elastic Particles: three soft clumps deform when a ball falls on them. | ✓ VERIFIED | `elastic_particles.rs` (382 lines): SPRING red + ELASTIC green + ELASTIC spinning box (`angular_velocity` 2.0), all `ParticleGroupFlags::SOLID`; asserts do not collapse into Jelly Drop's `ELASTIC\|SPRING`. Factory: `SceneId::ElasticParticles => elastic_particles::build`. E2E opens `#/scene/elastic-particles`. |
| 4 | Visitor can watch Rigid Particles: three colored clumps stay solid and do not stretch like jelly when a ball hits them. | ✓ VERIFIED | `rigid_particles.rs` (368 lines): three groups with `ParticleGroupFlags::RIGID \| SOLID`; tests assert no `ELASTIC`/`SPRING` particle flags; spinning box pose matches Elastic layout. Factory: `SceneId::RigidParticles => rigid_particles::build`. E2E opens `#/scene/rigid-particles`. |

**Score:** 4/4 truths verified (roadmap success criteria)

### Supporting plan truths (also verified)

| Truth | Status | Evidence |
| ----- | ------ | -------- |
| SCENE_IDS order is eight existing then the three material ids; all three `ready: true` with empty controls | ✓ | `scenes.ts` order + `controls: []` |
| PAGE_SUMMARY says eleven demos; Static preview SVG cases for all three; captions remain Static preview | ✓ | `runtime.ts` "All eleven demos"; `previews.tsx` cases; `DemoNavigation` caption |
| Vitest locks eleven SCENE_IDS + credits; SCENE_CAPTURE_PLANS matches SCENE_IDS | ✓ | `scenes.test.ts` + `demo-media/model.ts`; Vitest 17 passed this verification |
| Chromium smoke covers material play/pause/reset; eight prior scenes still run; eleven previews | ✓ | `player.spec.ts` / `shell.spec.ts` / recorded smoke 38/0 |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/liquidfun-wasm/src/scene/surface_tension.rs` | Tensile color-mixing basin scene | ✓ VERIFIED | 302 lines; build + hooks + 5 unit tests |
| `crates/liquidfun-wasm/src/scene/basin_family.rs` | Shared vertical-wall basin + ball | ✓ VERIFIED | 83 lines; `attach_vertical_wall_basin` + `create_falling_ball` |
| `crates/liquidfun-wasm/src/scene/elastic_particles.rs` | Soft SPRING/ELASTIC SOLID clumps | ✓ VERIFIED | 382 lines; distinct flags + spin + 5 tests |
| `crates/liquidfun-wasm/src/scene/rigid_particles.rs` | Rigid SOLID clumps | ✓ VERIFIED | 368 lines; RIGID\|SOLID without elastic flags + 5 tests |
| `crates/liquidfun-wasm/src/scene.rs` | SceneId + build_scene arms | ✓ VERIFIED | Three parse tokens + three `=> *_::build` arms |
| `web/src/catalog/scenes.ts` | Eleven-scene catalog + credits | ✓ VERIFIED | Order, ready, empty controls, pinned hrefs |
| `web/src/catalog/previews.tsx` | Three static SVG previews | ✓ VERIFIED | Exhaustive switch cases; wired via DemoNavigation |
| `web/src/player/runtime.ts` | Eleven-demo PAGE_SUMMARY | ✓ VERIFIED | "All eleven demos run…" |
| `web/tests/scenes.test.ts` | Eleven-scene Vitest contract | ✓ VERIFIED | Order, hints, empty controls, pinned labels/hrefs |
| `web/scripts/demo-media/model.ts` | Capture plans sync | ✓ VERIFIED | Three material plans + `assertSceneCapturePlanCoverage` |
| `web/e2e/player.spec.ts` | Watch-first play/pause/reset | ✓ VERIFIED | `controls.length === 0` branch covers five watch-first scenes |
| `web/e2e/player-helpers.ts` | Eleven hash paths + timeout | ✓ VERIFIED | 11 `SCENE_HASH_PATHS`; `ALL_SCENE_TIMEOUT_MS = 220_000` |
| `web/e2e/shell.spec.ts` | Eleven Static previews | ✓ VERIFIED | Desktop count 11; Rigid Particles last focus; `.catalog-card` 0 |

gsd-tools `verify artifacts` returned all_passed for plans 27-01 through 27-06.

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `scene.rs` | `surface_tension.rs` | `SceneId::SurfaceTension => surface_tension::build` | ✓ WIRED | Pattern in source |
| `surface_tension.rs` | liquidfun API | `TENSILE \| COLOR_MIXING` | ✓ WIRED | `GROUP_FLAGS` on all three recipes |
| `scene.rs` | `elastic_particles.rs` | `SceneId::ElasticParticles => elastic_particles::build` | ✓ WIRED | Pattern in source |
| `elastic_particles.rs` | liquidfun API | SOLID + SPRING/ELASTIC + `with_angular_velocity(2.0)` | ✓ WIRED | Manual verify (gsd-tools regex false-negative on escaped pattern) |
| `scene.rs` | `rigid_particles.rs` | `SceneId::RigidParticles => rigid_particles::build` | ✓ WIRED | Pattern in source |
| `rigid_particles.rs` | liquidfun API | `RIGID \| SOLID` without elastic particle flags | ✓ WIRED | Pattern + unit asserts |
| `scenes.ts` | WASM SceneId tokens | `surface-tension` / `elastic-particles` / `rigid-particles` | ✓ WIRED | Matches `parse_scene_id` (gsd-tools false-negative: cross-crate target) |
| `previews.tsx` | DemoNavigation | `ScenePreview` switch | ✓ WIRED | Import + render + Static preview caption |
| `scenes.test.ts` | `scenes.ts` | credit / order asserts | ✓ WIRED | Pattern found |
| `demo-media/model.ts` | `SCENE_IDS` | `assertSceneCapturePlanCoverage` | ✓ WIRED | Import-time assert |
| `player.spec.ts` | catalog controls | `controls.length === 0` | ✓ WIRED | Watch-first branch (gsd-tools missed escaped regex) |
| `justfile` `web-player-smoke` | e2e suite | `bun scripts/web-build.ts player-smoke` | ✓ WIRED | Recipe present; log complete (gsd-tools false-negative: `from` is not a file path) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Catalog / player | `SCENES` / `SCENE_IDS` | Static catalog records | Real metadata + ready flags | ✓ FLOWING |
| Surface Tension WASM | flags + particle count | `TENSILE\|COLOR_MIXING` recipes + ball | Unit tests assert flags and count > 0 | ✓ FLOWING |
| Elastic Particles WASM | distinct soft flags + spin | SPRING / ELASTIC + SOLID | Flag contrast + angle/ω asserts | ✓ FLOWING |
| Rigid Particles WASM | rigid group flags | `RIGID\|SOLID` recipes | Asserts three rigid groups, no elastic particle flags | ✓ FLOWING |
| Watch-first e2e | `data-step-index` | Live WASM session advance | Polls > 0 then pause/play/reset | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Material WASM unit tests | `cargo test -p liquidfun-wasm --lib -- surface_tension elastic_particles rigid_particles parse_scene_id` | 19 passed | ✓ PASS |
| Catalog/Vitest contract | `bunx vitest run tests/scenes.test.ts tests/demo-media-model.test.ts` | 17 passed | ✓ PASS |
| Chromium player-smoke (recorded 27-06) | `just web-player-smoke` log + `.last-run.json` | `complete player-smoke`; `"status":"passed"`; SUMMARY 38/0 | ✓ PASS |

Note: Full `cargo test -p liquidfun-wasm` can still report 1 failure in `scene::float_or_sink::tests::cork_finishes_above_stone_after_the_same_native_steps` (cork y ≈ 0.8832429). That failure reproduces at pre-Phase-27 commit `8a8bdf6`; Phase 27 did not modify `float_or_sink.rs` or the liquidfun engine. Treated as pre-existing, not a Phase 27 regression.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| MAT-01 | 27-01, 27-04, 27-05, 27-06 | Surface Tension tensile bead + color bleed | ✓ SATISFIED | `surface_tension.rs` + catalog + Vitest + smoke |
| MAT-02 | 27-02, 27-04, 27-05, 27-06 | Elastic Particles soft clump deformation | ✓ SATISFIED | `elastic_particles.rs` + catalog + Vitest + smoke |
| MAT-03 | 27-03, 27-04, 27-05, 27-06 | Rigid Particles solid clumps (not jelly) | ✓ SATISFIED | `rigid_particles.rs` + catalog + Vitest + smoke |

No orphaned Phase 27 requirements: REQUIREMENTS.md maps only MAT-01, MAT-02, MAT-03 to this phase; all appear in plan frontmatter. REQUIREMENTS.md marks all three Complete.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TODO/FIXME/placeholder stubs in phase key scene/catalog/e2e files | — | None blocking |

Notes (non-blocking): `SIX_SCENE_TIMEOUT_MS` remains a deprecated alias of `ALL_SCENE_TIMEOUT_MS`. Watch-first capture plans may still use a center-click `SceneAction` stub (documented; WASM pointer is no-op). `MAX_ADVANCE_STEPS` remains 4.

### Human Verification Required

None. D-16 names Chromium `just web-player-smoke` as the browser gate; construction/recognition are locked by pinned flag-recipe and geometry unit tests. Headed aesthetic judgment is out of scope for this hobby phase (same stance as Phase 26 VERIFICATION for BASIN watch criteria).

### Gaps Summary

No gaps. All four roadmap success criteria and supporting plan must-haves are present, substantive, wired, and covered by unit + Vitest + recorded smoke evidence. Later Phase 28 owns interaction-heavy scenes (ACT-*); Phase 29 owns Sparky/Drawing and the full twelve-scene catalog claim — not deferred gaps for MAT-01/02/03.

### Confirmation-bias notes (informational)

1. Unit tests lock flag recipes and construction/advance survival, not a measured post-impact color-histogram or AABB stretch metric; under D-07–D-09 / D-16 that is intentional (recognizable ports, not sealed parity).
2. Reset e2e asserts step-index remount rather than particle-layout checksums; construction unit tests and dispose-on-reset still satisfy play/pause/reset under the remount rule.
3. gsd-tools key-link checks produced several false negatives (cross-crate targets, escaped regex, justfile-as-source); manual wiring verification closed those.

### Lifecycle provenance

| Artifact | lifecycle_mode | phase_lifecycle_id |
| -------- | -------------- | ------------------ |
| `27-CONTEXT.md` | yolo | `27-2026-09-22T05-14-55` |
| Plans 27-01…27-06 | yolo | `27-2026-09-22T05-14-55` |
| Summaries 27-01…27-06 | yolo | `27-2026-09-22T05-14-55` |
| This VERIFICATION.md | yolo | `27-2026-09-22T05-14-55` |

`lifecycle_validated: true` — CONTEXT, PLANs, SUMMARYs, and VERIFICATION share compliant provenance (not `direct-fallback`).

---

_Verified: 2026-09-22T06:21:06Z_
_Verifier: Claude (gsd-verifier)_
