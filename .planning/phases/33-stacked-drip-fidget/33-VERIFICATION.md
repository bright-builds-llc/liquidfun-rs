---
phase: 33-stacked-drip-fidget
verified: 2026-09-28T02:22:28Z
status: passed
score: 16/16 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-09-27T23-41-40
generated_at: 2026-09-28T02:22:28Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 33: Stacked drip fidget Verification Report

**Phase Goal:** Visitors can watch Stacked Drip: liquid drains through a vertical stack of three moving trays, and each tray tips when the drip reaches it. Original scene. Catalog id `stacked-drip` after `liquid-bubbler`. Watch-first. One water group. Motor-off revolute trays. Quiet physical return. `MAX_ADVANCE_STEPS` stays 4.
**Verified:** 2026-09-28T02:22:28Z
**Status:** passed
**Re-verification:** No — initial verification

This report is goal-backward evidence. It is not an independent review acknowledgment, and it does not describe the implementing agent as having approved the phase.

## Goal Achievement

Stacked Drip is a real catalog scene. One teal water group starts above three dynamic trays. Each tray is a motor-off revolute joint limited from rest `0` to `tau/8`. After a 5 second dwell window of `advance(4)`, the starting particles are below the bottom tray, the upper tray moves before the middle tray, and the middle tray moves before the lower tray, while the side-shaft plate is still down. After one dwell plus one rise plus one second, an original particle id is back above the top tray and left of the divider, with the live count unchanged.

Plan 01 sealed the divider to the floor and required every cascade particle to stay left of the divider. Plan 04 opened the inlet so the plate can lift the same particles. The cascade test now requires every original id below the bottom tray, and treats a particle right of the divider as a bypass only while that particle is still at or above the bottom tray. That matches the opened shaft. The sealed-divider wording is not the finished scene.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | `stacked-drip` builds one teal water group above three dynamic trays, each on a motor-off revolute joint limited from rest 0 to a pour angle, with a catch-side counterweight, and a dynamic side-shaft plate whose speed stays 0 through the cascade proof. | ✓ VERIFIED | `ParticleColor::new(64, 196, 196, 255)`, default water flags, `RevoluteJointDef` `with_limits(true, 0.0, POUR_ANGLE)` and no `with_motor`. Counterweight is on local `+x` at density `1.05`. Plate speed is `0` while `phase < DWELL` (`DWELL` is `6.0`, proof is `5.0` s). `plate_stays_down_during_the_cascade` passed. |
| 2 | After a bounded `SessionCore::advance(4)` run that finishes inside the dwell, the live count is unchanged, every original particle that began above the top tray is below the bottom tray, and each tray angle has left 0 by at least 0.05 rad. | ✓ VERIFIED | `cohort_passes_below_the_bottom_tray` and `upper_tray_moves_before_the_lower_trays` passed on re-run (`PROOF_BATCHES` is 75 calls of `advance(4)`). See the shaft note above for the plan-01 left-of-divider clause. |
| 3 | The upper tray's first sample past 0.05 rad is strictly earlier than the middle tray's, which is strictly earlier than the lower tray's. | ✓ VERIFIED | `upper_tray_moves_before_the_lower_trays` ranks revolute joints by pivot `y` and asserts `upper_batch < middle_batch < lower_batch`. Passed on re-run. |
| 4 | At that sample the plate translation and prismatic speed are still about 0, and every revolute motor stays disabled. | ✓ VERIFIED | Translation absolute value under `0.01` and speed within `1.0e-5` of `0` in `plate_stays_down_during_the_cascade`. `is_motor_enabled()` is false for all three revolute joints in `tray_motors_stay_off_and_the_plate_is_beside_the_stack`. Source has `set_prismatic_motor_speed` and no `set_revolute_motor_speed`. |
| 5 | Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, and Liquid Tumbler are unchanged, and `MAX_ADVANCE_STEPS` stays 4. | ✓ VERIFIED | Those five scene files have no commits from `16f193e` through `HEAD`. Latest touch is still `76cdd58` (phase 32). `MAX_ADVANCE_STEPS` is `4` in `session.rs`. `MAX_STEPS_PER_FRAME` is `4` in `web/src/physics/clock.ts`. `PARTICLE_RADIUS` stays `0.025`. |
| 6 | The catalog lists twenty-three ready scenes, with `stacked-drip` immediately after `liquid-bubbler`, and the Liquid Timer and Liquid Bubbler records are unchanged. | ✓ VERIFIED | `SCENE_IDS` ends `liquid-bubbler`, `stacked-drip` (23 ids). Stacked Drip record is `ready: true`. Liquid Timer still credits the pinned timer sources. Liquid Bubbler still describes the waist and wheel. |
| 7 | Stacked Drip is watch-first aside from the shared gravity slider, and its static preview shows a vertical stack of trays under a colored drip, captioned `Static preview` by the shell. | ✓ VERIFIED | `controls: withGravitySlider([])` and `WATCH_FIRST_HINT`. Vitest expects only `GRAVITY_CONTROL` and rejects `tray`, `color`, and `speed`. `StackedDripPreview` draws three tilted lines under `#40C4C4` with no wheel circle. Caption stays in `DemoNavigation.tsx`, not in the SVG. |
| 8 | The phone frame contains the reservoir, all three trays at rest and at the pour angle, the walls, the divider, and the plate at rest and at the top of the stroke, and the README plan id is `stacked-drip` with empty cues. | ✓ VERIFIED | `viewBounds` and `PORTRAIT_VIEW_BOUNDS["stacked-drip"]` are `{ minX: -0.86, minY: -0.16, maxX: 1.68, maxY: 2.18 }`. Portrait wall points include divider `y` `0.22`..`1.70`, plate rest `y` `0.02`..`0.06` at `x` `1.00`..`1.50`, and raised plate `y` `1.92`..`1.96`. `portrait-bounds.test.ts` passed 5 tests. README plan is `{ id: "stacked-drip", cues: [] }` after `liquid-bubbler`. Height floor is `0.39` because the fitted phone fraction is under `0.40`. |
| 9 | Credits point at `stacked_drip.rs` and the LiquidFun showcase link only. | ✓ VERIFIED | `implementationPath` is `crates/liquidfun-wasm/src/scene/stacked_drip.rs`. `inspiration: [SHOWCASE]`. Credits test expects only `SHOWCASE_HREF`. No `github.com/google/liquidfun/blob` URL in the stacked-drip scene module. |
| 10 | Chromium smoke opens, pauses, plays, and resets Stacked Drip, and still opens the scenes that were already in the catalog. | ✓ VERIFIED | `WATCH_FIRST_SCENE_IDS` is every `SCENE_IDS` entry absent from `POINTER_CONTROL`. `stacked-drip` is not in `POINTER_CONTROL`, and its hash is `#/scene/stacked-drip`. `33-03-SUMMARY.md` records `just web-player-smoke` at 44 passed in 38.8s. This verification did not re-run that browser suite. |
| 11 | The shell reports twenty-three demos, twenty-three static previews, zero catalog cards, and a drawer whose last link is Stacked Drip. | ✓ VERIFIED | `shell.spec.ts` expects `All twenty-three demos`, `toHaveCount(23)` for sidebar previews and hidden SVGs, `.catalog-card` `toHaveCount(0)`, and `/Stacked Drip\b/`. Fountain stays `/Static preview Fountain\b/`. |
| 12 | Smoke does not assert that a particle has returned above the top tray. | ✓ VERIFIED | `player.spec.ts` and `shell.spec.ts` have no stacked-drip particle-return assertion. The return proof is the native test in plan 04. |
| 13 | After one dwell plus one rise and a short settle, an original particle id that started above the top tray is above the top tray again and left of the divider, and the live count is unchanged. | ✓ VERIFIED | `return_lifts_an_original_particle_above_the_top_tray` advances `DWELL + (STROKE / PLATE_SPEED) + 1.0` seconds with `advance(4)` only. Passed on re-run. |
| 14 | The three revolute motors stay disabled, the plate stays in the shaft beside the trays, and the scene does not destroy or rewrite particle positions to fake the loop. | ✓ VERIFIED | Return test asserts every revolute `is_motor_enabled()` is false. Plate `x` is `1.00`..`1.50`, right of divider `0.98`. No `set_particle_position`, `create_particle_with_def`, or `set_particle_velocity`. `with_destruction_by_age(false)` is set. `source_lifts_with_a_prismatic_plate` passed. |
| 15 | The plan 01 cascade tests still pass, including the plate staying down through the proof window. | ✓ VERIFIED | Re-run of `scene::stacked_drip` under `CARGO_TARGET_DIR=target/verify-33`: 9 passed, 0 failed, in 1.13s. That includes the cohort, order, and plate-down tests. |
| 16 | The phone wall list matches the opened divider and the plate at rest and at full stroke. | ✓ VERIFIED | Portrait points match divider bottom `y = 0.22`, spill lip `y = 1.70`, rest plate `(1.00, 0.02)`..`(1.50, 0.06)`, and raised plate `y` `1.92`..`1.96`. `bun run test:unit -- tests/portrait-bounds.test.ts` passed 5 tests. |

**Score:** 16/16 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/stacked_drip.rs` | Three limited revolute trays and a delayed prismatic plate | ✓ VERIFIED | Build, dwell schedule, and `on_advance` call `set_prismatic_motor_speed`. 456 lines. |
| `crates/liquidfun-wasm/src/scene/stacked_drip/vessel.rs` | Walls, sloped floor, and tray fixtures | ✓ VERIFIED | Four wall boxes, floor wedge, deck plus counterweight. |
| `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs` | Cascade order and return tests | ✓ VERIFIED | Nine tests, all passed on re-run. Trays ranked by pivot `y`. |
| `crates/liquidfun-wasm/src/scene.rs` | `SceneId::StackedDrip` and token `stacked-drip` | ✓ VERIFIED | Enum variant, parse arm, and `build_scene` route. |
| `web/src/catalog/scene-records.ts` | Watch-first Stacked Drip record | ✓ VERIFIED | Appended after Liquid Bubbler. |
| `web/src/catalog/previews.tsx` | Static SVG of three trays under a teal drip | ✓ VERIFIED | Local `DRIP = "#40C4C4"`. Shared gold `DRIP` is unchanged. |
| `web/src/catalog/portrait-bounds.ts` | Phone rectangle | ✓ VERIFIED | Same rectangle as `viewBounds`. |
| `web/scripts/readme-svg/plans.ts` | README plan with empty cues | ✓ VERIFIED | `{ id: "stacked-drip", cues: [] }` after `liquid-bubbler`. |
| `web/e2e/shell.spec.ts` | Twenty-three-demo shell and drawer wrap | ✓ VERIFIED | Last link is `/Stacked Drip\b/`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `stacked_drip.rs` | revolute joint defs | `with_limits`, no revolute `with_motor` | ✓ WIRED | Tray joints use `with_limits(true, 0.0, POUR_ANGLE)` only. |
| `stacked_drip.rs` | prismatic motor | `set_prismatic_motor_speed` in `on_advance` | ✓ WIRED | `SessionCore::advance` calls `on_advance` before `World::step`. |
| `scenes.ts` | `scene-records.ts` | shared `stacked-drip` id | ✓ WIRED | Id is last in `SCENE_IDS` and last in `SCENES`. |
| `readme-svg/plans.ts` | `scenes.ts` | `assertReadmeSvgPlanCoverage` id | ✓ WIRED | Plan id matches catalog order with `cues: []`. |
| `scene-records.ts` | `stacked_drip.rs` | single-file `sceneSource` path | ✓ WIRED | Path is `crates/liquidfun-wasm/src/scene/stacked_drip.rs`. |
| `player.spec.ts` | `scenes.ts` | watch-first ids absent from `POINTER_CONTROL` | ✓ WIRED | Stacked Drip is included by omission from the pointer map. |
| `shell.spec.ts` | `runtime.ts` | footer `All twenty-three demos` | ✓ WIRED | Both strings match. |
| return test | scene | `advance(4)` through dwell plus rise | ✓ WIRED | Test name `return_lifts_an_original_particle_above_the_top_tray` calls `session.advance(4)` only. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `stacked_drip/tests.rs` | particle ids and positions | `session.read_particles` after `SessionCore::advance` | Yes. Positions come from the stepped particle system, not a hardcoded end pose. | ✓ FLOWING |
| `StackedDripPreview` | SVG geometry | static catalog illustration | The preview is intentionally static. Motion is the WASM scene. | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Cascade, order, plate-down, and return proofs | `CARGO_TARGET_DIR=target/verify-33 cargo test -p liquidfun-wasm --lib scene::stacked_drip -- --test-threads=1` | 9 passed, 0 failed, 1.13s | ✓ PASS |
| Phone frame contains the finished shaft | `cd web && bun run test:unit -- tests/portrait-bounds.test.ts` | 5 passed | ✓ PASS |
| Chromium open, pause, play, and reset | `just web-player-smoke` | Not re-run. Plan 03 summary records 44 passed in 38.8s. Spec still routes `stacked-drip` through the watch-first loop. | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| none | 33-01 through 33-04 | Phase requirement field is empty. `REQUIREMENTS.md` has no Phase 33 mapping. | ✓ SATISFIED | No orphaned requirement ids. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/stacked_drip/tests.rs` | 84 | Bypass check `x >= divider && y >= bottom` cannot fail once every particle is already `y < bottom` | ℹ️ Info | The live check is that every original particle is below the bottom tray. Particles already below the tray may sit in the shaft, which is the return boarding path. |
| `crates/liquidfun-wasm/src/scene/stacked_drip.rs` | 29 | `ANGULAR_DAMPING` is `20` and counterweight density is `1.05` | ℹ️ Info | Plan 01 allowed retune when a named cascade test failed. `33-01-SUMMARY.md` records both values so a poured tray stays past `0.05` rad at 5 s while an empty tray still starts at rest. |

### Human Verification Required

None. The phase gate is the native cascade and return tests, the portrait-bounds unit test, and the already recorded Chromium smoke spec. This verification re-ran the native and portrait tests.

### Gaps Summary

No blocking gaps. Plan 04's opened shaft supersedes plan 01's sealed-divider sentence: at the end of the 5 second proof, a particle may be right of the divider only after it is below the bottom tray. The return test still requires an original id back above the top tray and left of the divider, with the live count unchanged and the revolute motors off.

Lifecycle provenance matches across `33-CONTEXT.md`, all four plans, and all four summaries: `lifecycle_mode: yolo` and `phase_lifecycle_id: 33-2026-09-27T23-41-40`. None of those artifacts are marked `direct-fallback`.

---

_Verified: 2026-09-28T02:22:28Z_
_Verifier: Claude (gsd-verifier)_
