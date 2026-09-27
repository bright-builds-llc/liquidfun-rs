---
phase: 30-periodic-hydraulic-fountain
verified: 2026-09-27T15:53:52Z
status: passed
score: 11/11 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 30-2026-09-27T14-01-51
generated_at: 2026-09-27T15:53:52Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 30: Periodic Hydraulic Fountain Verification Report

**Phase Goal:** Visitors can watch a timed piston squeeze a water reservoir so the same liquid travels through a throat into another chamber, without replacing the existing Fountain scene.
**Verified:** 2026-09-27T15:53:52Z
**Status:** passed
**Re-verification:** No — initial verification

This report is the phase goal check. It is not an independent review acknowledgment. Commit `9fec7ff` (`fix(30): draw the moving hydraulic piston`) is on `HEAD` and matches the current scene sources. It closes review warning WR-01.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | A new scene id `hydraulic-fountain` builds a dynamic piston that squeezes one plain water group through a floor throat. | ✓ VERIFIED | `SceneId::HydraulicFountain` parses `hydraulic-fountain` and calls `hydraulic_fountain::build`. The piston is `BodyType::Dynamic` on a limited prismatic joint. `on_advance` writes `set_prismatic_motor_speed` from elapsed time before `World::step`. One `ParticleGroupRecipe` uses the default `ParticleFlags::WATER`. |
| 2 | Within one 2 second period, at least one particle id that started in the piston chamber is on the fountain side, and the live count is unchanged. | ✓ VERIFIED | `piston_chamber_particles_cross_the_throat_within_one_period` snapshots ids, advances 120 steps in batches of 4, and asserts an original id past `x = 0.04` with an unchanged live count. Recorded `cargo test -p liquidfun-wasm` hydraulic fountain tests passed, including this test. |
| 3 | The captured frame draws the moving piston. | ✓ VERIFIED | `HydraulicFountainHooks` keeps the piston `BodyId`. `collect_segments` appends the four transformed local edges. `captured_frame_draws_the_moving_piston` reads `capture_frame` and asserts the right face moves toward the throat (32 floats: four walls and four piston edges). |
| 4 | The existing Fountain scene module is unchanged, and `MAX_ADVANCE_STEPS` stays 4. | ✓ VERIFIED | `fountain.rs` last commit is `a419152` (particle eviction), not this scene. `MAX_ADVANCE_STEPS: u32 = 4` and `MAX_STEPS_PER_FRAME = 4`. |
| 5 | The catalog lists twenty ready scenes, with `hydraulic-fountain` immediately after `sparky`, and the fountain record is unchanged. | ✓ VERIFIED | `SCENE_IDS` is 20 long and ends `sparky`, `hydraulic-fountain`. The `fountain` record is still the aimed stream with emission, launch speed, and aim. |
| 6 | Hydraulic Fountain is watch-first aside from the shared gravity slider, with a static preview of a piston, a throat, and two chambers. | ✓ VERIFIED | Record controls are `withGravitySlider([])` and the hint is `WATCH_FIRST_HINT`. `HydraulicFountainPreview` draws two chambers, a floor gap, a divider, a left piston, and water only on the left. `DemoNavigation` captions `Static preview`. |
| 7 | The phone frame contains the chamber walls and the piston stroke, and the README plan id is present with empty cues. | ✓ VERIFIED | Portrait and `viewBounds` are `minX -1.3`, `minY -0.15`, `maxX 1.3`, `maxY 1.6`. Wall endpoints through `y = 1.39` and piston faces `x = -0.86` and `x = -0.51` sit inside that rectangle. `maxY` is 1.6 rather than the plan's 1.55 so the 0.28 height fraction clears the camera inset. README plan `{ id: "hydraulic-fountain", cues: [] }` follows sparky. |
| 8 | Credits point at `hydraulic_fountain.rs` and the LiquidFun showcase link only. | ✓ VERIFIED | Implementation path is `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`, which `isAllowlistedScenePath` accepts as a single scene file. Inspiration is `[SHOWCASE]` (`https://google.github.io/liquidfun/`). |
| 9 | Chromium smoke opens, pauses, plays, and resets Hydraulic Fountain, and still opens the scenes that were already in the catalog. | ✓ VERIFIED | `hydraulic-fountain` is absent from `POINTER_CONTROL`, so `WATCH_FIRST_SCENE_IDS` includes it. Recorded `just web-player-smoke` exited 0 with 44 Chromium tests. |
| 10 | The shell reports twenty demos, twenty static previews, zero catalog cards, and a drawer whose last link is Hydraulic Fountain. | ✓ VERIFIED | `PAGE_SUMMARY` says `All twenty demos`. `shell.spec.ts` expects that phrase, 20 `Static preview` captions, 20 hidden SVGs, `.catalog-card` count 0, and a link named Hydraulic Fountain. |
| 11 | The implementing agent records evidence and does not approve its own work. | ✓ VERIFIED | `30-03-SUMMARY.md` records the smoke result and states the implementing agent did not approve the work. This verification file is the goal check only. |

**Score:** 11/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs` | Scene build, limited prismatic motor, schedule, and piston edges | ✓ VERIFIED | 287 lines. Dynamic piston, `set_prismatic_motor_speed`, water group, no kinematic pose write. |
| `crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs` | Crossing, rebuild, and draw tests | ✓ VERIFIED | Crossing, rebuild at translation 0, and moving piston face. |
| `crates/liquidfun-wasm/src/scene.rs` | `SceneId::HydraulicFountain` and parse token | ✓ VERIFIED | Enum, parse arm, and `build_scene` route. |
| `crates/liquidfun-wasm/src/session/tests.rs` | Allowlist token | ✓ VERIFIED | `("hydraulic-fountain", SceneId::HydraulicFountain)`. |
| `web/src/catalog/scene-records.ts` | Watch-first record | ✓ VERIFIED | Ready record after sparky. Fountain record still aimed. |
| `web/src/catalog/previews.tsx` | Static piston, throat, and two chambers | ✓ VERIFIED | `HydraulicFountainPreview` wired from the scene switch. |
| `web/src/catalog/portrait-bounds.ts` | Phone rectangle | ✓ VERIFIED | Matches the record `viewBounds` and contains the wall points. |
| `web/scripts/readme-svg/plans.ts` | README plan with `cues: []` | ✓ VERIFIED | Entry follows sparky. |
| `web/e2e/shell.spec.ts` | Twenty-demo shell asserts | ✓ VERIFIED | Preview counts and drawer last link. |
| `web/e2e/player.spec.ts` | Existing watch-first loop | ✓ VERIFIED | New id is absent from `POINTER_CONTROL`. |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `hydraulic_fountain.rs` | `World::set_prismatic_motor_speed` | `on_advance` before `World::step` | WIRED | `SessionCore::advance` calls `on_advance`, then `world.step`. |
| `session.rs` | `hydraulic_fountain.rs` | `build_scene` | WIRED | `SceneId::HydraulicFountain => hydraulic_fountain::build`. |
| `capture_frame` | piston edges | `collect_segments` | WIRED | Frame lane copies hook segments, including the transformed piston. |
| `scenes.ts` | `scene-records.ts` | shared id after sparky | WIRED | Both lists end with `hydraulic-fountain`. |
| `readme-svg/plans.ts` | `scenes.ts` | plan id coverage | WIRED | Empty-cue plan matches the last catalog id. |
| `scene-records.ts` | `hydraulic_fountain.rs` | implementation path | WIRED | Host-locked single-file scene path. |
| `player.spec.ts` | `scenes.ts` | `WATCH_FIRST_SCENE_IDS` | WIRED | Every id missing from `POINTER_CONTROL` is watch-first. |
| `shell.spec.ts` | `runtime.ts` | `All twenty demos` | WIRED | Footer phrase and smoke assert match. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `hydraulic_fountain.rs` particles | water group positions | `create_water_group` polygon fill, default water flags | Yes | ✓ FLOWING |
| `collect_segments` piston | body transform | `world.body_snapshot(self.piston)` | Yes | ✓ FLOWING |
| Catalog preview | static SVG | `HydraulicFountainPreview` | Artwork, not a live sim | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Particles cross and the piston is drawn | `cargo test -p liquidfun-wasm` hydraulic fountain tests | Recorded pass, including the crossing test and `captured_frame_draws_the_moving_piston` | ✓ PASS |
| Catalog play, pause, and reset | `just web-player-smoke` | Recorded exit 0, 44 Chromium tests | ✓ PASS |
| Workspace regression | `cargo test` at the repo root | Recorded exit 0 | ✓ PASS |

Spot-checks use the recorded runs named in the verification request. The current sources still contain those assertions.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| none | 30-01, 30-02, 30-03 | Phase requirement IDs are empty. Decisions D-01 through D-12 are the contract. | ✓ SATISFIED | No requirement IDs were invented. PLAY-03 stays on Phase 26. |

D-01 through D-12 hold in the sources: new id after sparky, two chambers and a throat, timed dynamic prismatic motor, watch-first controls, one water group, step cap 4, ready catalog record, portrait frame and empty-cue README plan, showcase-only credit, recorded Chromium smoke, and no self-approval in this file.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `hydraulic_fountain.rs` | 280 | `collect_circles` returns an empty list | ℹ️ Info | The piston is a box of segments. Empty circles match the plan. |
| none | — | `unwrap`, kinematic `set_transform`, extra particle flags, TODO/FIXME | — | Not present in the scene module. |

### Human Verification Required

None. D-11 names the recorded Chromium smoke as the browser gate, and the frame test covers the moving piston.

### Gaps Summary

No gaps. The scene visitors can open is a timed piston on one water group. The same particles can cross the throat within one period. The live frame draws that piston. Fountain remains the aimed emitter.

---

_Verified: 2026-09-27T15:53:52Z_
_Verifier: Claude (gsd-verifier)_
