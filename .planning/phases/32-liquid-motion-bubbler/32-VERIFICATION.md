---
phase: 32-liquid-motion-bubbler
verified: 2026-09-27T20:48:35Z
status: passed
score: 12/12 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T20:48:35Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 11/12
  gaps_closed:
    - "A quiet return lifts liquid back to the upper reservoir so the drip continues for the whole play session."
  gaps_remaining: []
  regressions: []
---

# Phase 32: Liquid motion bubbler Verification Report

**Phase Goal:** Visitors can watch Liquid Bubbler: colored liquid drips through a narrow waist and turns a small wheel, without changing Water Wheel.
**Verified:** 2026-09-27T20:48:35Z
**Status:** passed
**Re-verification:** Yes — after gap closure

## Goal Achievement

The opening spectacle still holds, and the locked return now holds too. Liquid boards the plate through a vertical slot taller than one particle, the plate top rises above the divider spill lip, and an original particle is back in the upper reservoir after one dwell-plus-rise.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Visitors can watch colored liquid drip through a narrow waist and turn a small wheel, without changing Water Wheel. | ✓ VERIFIED | `drip_crosses_the_waist_and_turns_the_wheel` passed on re-run. Amber `ParticleColor::new(242, 176, 64, 255)` is still copied through `session.rs` color lanes into `canvas.ts`. `git diff 277710c^..HEAD` is empty for `water_wheel.rs`. |
| 2 | Catalog id `liquid-bubbler` is appended after `wave-tank`, titled Liquid Bubbler, and does not replace or retune Water Wheel. | ✓ VERIFIED | `SCENE_IDS` still ends `wave-tank`, `liquid-bubbler`. Record is `ready: true`. Water Wheel was not in the 32-04 diff. |
| 3 | Layout is an upper reservoir, one static waist, a lower chamber, and a wheel under the drip. | ✓ VERIFIED | Six static ground boxes remain. Lips still leave a 0.14 m gap at y 0.90..0.98. Hub is `(0.0, 0.40)`. Water polygon is y 1.12..1.50. |
| 4 | The wheel is a dynamic paddle wheel on a motor-off revolute joint, turned by particles. | ✓ VERIFIED | `RevoluteJointDef::new` plus `with_frame` only. `wheel_motor_stays_off_and_the_plate_is_beside_the_wheel` passed. No `set_revolute_motor_speed`. |
| 5 | A quiet return lifts liquid back to the upper reservoir so the drip continues for the whole play session, and reset restores the initial layout. | ✓ VERIFIED | Inlet slot is 0.16 m between shape faces (0.14 m after 0.01 m polygon skin), above the 0.05 m diameter. Plate top at stroke is y 1.86, above the y 1.62 spill lip. `return_lifts_an_original_particle_above_the_waist` passed. `rebuild_restores_the_reservoir` passed. Speed stays 0.15 m/s after a 3 s dwell, and `scheduled_plate_speed` repeats on `CYCLE`. |
| 6 | One plain water group uses one distinct color. Color Mixer is untouched. | ✓ VERIFIED | Default water group plus one `with_color`. No mixing, tensile, elastic, or rigid flags. `color_mixer.rs` has no diff since the scene landed. |
| 7 | `MAX_ADVANCE_STEPS` stays 4, and radius and count were not cut. | ✓ VERIFIED | `MAX_ADVANCE_STEPS: u32 = 4` in `session.rs`. Radius stays `0.025`. No `with_maximum_count`. |
| 8 | The catalog entry is ready, watch-first aside from gravity, hash-routed, and the existing shell stays free of `.catalog-card`. | ✓ VERIFIED | Description still matches the one-behavior copy. Controls are `withGravitySlider([])`. `navigation.ts` still builds `#/scene/${id}`. `PlaygroundShell.tsx` was not edited by plan 32-04. |
| 9 | A portrait frame contains the vessel, and the README plan id is `liquid-bubbler` with empty cues. | ✓ VERIFIED | `PORTRAIT_VIEW_BOUNDS["liquid-bubbler"]` is still minX `-0.71`, minY `-0.16`, maxX `1.12`, maxY `1.98`, which contains the raised plate at y 1.86. Portrait points now include divider bottom y 0.22 and plate corners `(0.58, 0.02)..(0.94, 1.86)`. `plans.ts` still has `{ id: "liquid-bubbler", cues: [] }` after wave-tank. |
| 10 | Credits stay host-locked to this scene module and describe an original experimental scene, not a pinned LiquidFun test. | ✓ VERIFIED | `implementationPath` is still `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`. Inspiration is only the showcase URL. Description says original experimental scene. |
| 11 | A native test shows an original above-waist particle below the waist and a wheel angle change. Chromium smoke is wired to open, pause, play, and reset the scene with the rest of the catalog. | ✓ VERIFIED | Re-ran `cargo test -p liquidfun-wasm scene::liquid_bubbler -- --test-threads=1`: 9 passed, including the crossing test. `liquid-bubbler` is still absent from pointer-control wiring, so the watch-first loop still covers it. Playwright was not re-run in this pass. |
| 12 | The implementing agent does not approve its own work. | ✓ VERIFIED | Plan 32-04 summary states that it records evidence and does not approve. This re-verification is a separate verifier pass. |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` | Static waist, motor-off wheel, delayed plate, one amber group | ✓ VERIFIED | Divider y 0.22..1.62, plate x 0.58..0.94 and y 0.02..0.06 at rest, stroke 1.80 m. Return path is open. |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` | Waist crossing, wheel-angle, and return tests | ✓ VERIFIED | Nine tests. Crossing, dwell, motor, rebuild, and `return_lifts_an_original_particle_above_the_waist` passed on re-run. |
| `crates/liquidfun-wasm/src/scene.rs` | Parse token and `build_scene` route | ✓ VERIFIED | Unchanged by plan 32-04. Enum, `"liquid-bubbler"` arm, and build arm remain. |
| `web/src/catalog/scene-records.ts` | Watch-first Liquid Bubbler record | ✓ VERIFIED | Still after wave-tank. Gravity slider only. |
| `web/src/catalog/previews.tsx` | Still SVG of a waist, amber drip, and wheel | ✓ VERIFIED | Unchanged by plan 32-04. `LiquidBubblerPreview` remains wired for the catalog id. |
| `web/src/catalog/portrait-bounds.ts` | Phone rectangle | ✓ VERIFIED | Same rectangle as `viewBounds`. Raised plate y 1.86 stays inside maxY 1.98. |
| `web/scripts/readme-svg/plans.ts` | `{ id: "liquid-bubbler", cues: [] }` | ✓ VERIFIED | Still last plan, catalog order. |
| `web/e2e/shell.spec.ts` | Twenty-two demos and drawer wrap | ✓ VERIFIED | Not edited by plan 32-04. Prior wiring remains. |
| `web/e2e/player.spec.ts` | Existing watch-first loop | ✓ VERIFIED | Not edited by plan 32-04. No pointer mapping, so the existing loop covers the hash. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `session.rs` `advance` | `liquid_bubbler.rs` `on_advance` | `on_advance` then `World::step` | ✓ WIRED | `SessionCore::advance` still calls hooks before the step. |
| `on_advance` | prismatic motor | `set_prismatic_motor_speed` | ✓ WIRED | Speed is 0 for the first 3 s, then ±0.15 m/s, repeating on `CYCLE`. |
| `lib.rs` `build_core` | `liquid_bubbler::build` | `parse_scene_id("liquid-bubbler")` | ✓ WIRED | Player scene id still reaches the native builder. |
| `scenes.ts` | `scene-records.ts` | shared id after wave-tank | ✓ WIRED | Hash href is `#/scene/liquid-bubbler`. |
| `scene-records.ts` | `liquid_bubbler.rs` | host-locked path | ✓ WIRED | Single file name under `scene/`. |
| particle recipe color | canvas fill | frame color lane | ✓ FLOWING | `with_color` values are copied in `capture` and drawn by `particleColor`. |
| side-shaft plate | upper reservoir | lift then spill over the divider | ✓ WIRED | After dwell plus rise plus 1 s, an original id is at y > 0.98 and x < 0.48. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| Canvas particles | `particleColors` | `ParticleGroupRecipe::with_color` through `SessionCore` frame capture | Yes, amber `(242, 176, 64, 255)` | ✓ FLOWING |
| Canvas wheel | body snapshot transform | `collect_circles` / `collect_segments` on the live wheel body | Yes | ✓ FLOWING |
| Side-shaft return | plate translation and live particle positions | prismatic speed from elapsed time, then particle positions after `World::step` | Yes. The return test reads live positions of the original ids. | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Original particle crosses the waist and the wheel angle leaves 0 | `cargo test -p liquidfun-wasm scene::liquid_bubbler -- --test-threads=1` | 9 passed, 0 failed, 2.63 s, including `drip_crosses_the_waist_and_turns_the_wheel` | ✓ PASS |
| Plate stays down and revolute motor stays off during the 2 s proof | same command | `plate_stays_down_during_the_proof` and `wheel_motor_stays_off_and_the_plate_is_beside_the_wheel` passed | ✓ PASS |
| Reset rebuilds a still reservoir and a resting wheel | same command | `rebuild_restores_the_reservoir` passed | ✓ PASS |
| An original particle is back above the waist and left of the divider after one dwell-plus-rise | same command | `return_lifts_an_original_particle_above_the_waist` passed | ✓ PASS |
| Chromium opens, pauses, plays, and resets Liquid Bubbler | `just web-player-smoke` | Not re-run here. Plan 32-04 did not edit the player smoke spec. | ? SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| none | 32-01, 32-02, 32-03, 32-04 | No PLAY-* id is assigned to Phase 32. PLAY-01, PLAY-02, and PLAY-03 are already complete on earlier phases. | ✓ SATISFIED | Empty `requirements:` on every plan, including 32-04. No orphaned Phase 32 id in `REQUIREMENTS.md`. |

Locked decisions D-01 through D-11 match the code, including D-04.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` | `source_lifts_with_a_prismatic_plate` | Source-text assertions (`include_str!`) | ⚠️ Warning | This test still only checks that the file mentions `set_prismatic_motor_speed` and does not teleport particles. The return behavior is now covered by `return_lifts_an_original_particle_above_the_waist`. |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` | `on_advance` | Reads `revolute_joint_angle` and discards the value | ℹ️ Info | Used only as a fallible read before writing plate speed. The revolute motor stays off. |

No TODO, FIXME, placeholder, or production `unwrap()` in the scene module. The return test does not call `set_particle_position`.

### Human Verification Required

None. The return is geometric and covered by the native dwell-plus-rise test. A browser watch is not required to confirm that an original particle crosses back into the upper reservoir.

### Gaps Summary

The previous gap is closed. Divider faces run y 0.22 to 1.62 and x 0.48 to 0.56. The plate at rest is x 0.58 to 0.94 and y 0.02 to 0.06, so the boarding slot under the divider is 0.16 m between shape faces and 0.14 m after the 0.01 m polygon skin on each fixture. Particle diameter is 0.05 m. The 0.02 m side clearances meet that skin, so liquid stays on the plate instead of draining beside it. Stroke is 1.80 m, so the plate top reaches y 1.86, above the spill lip at y 1.62. One second into the descent the top is still at y 1.71, and the space above the lip is open from the plate's left edge into x < 0.48.

`cargo test -p liquidfun-wasm scene::liquid_bubbler -- --test-threads=1` exited 0: 9 passed, 0 failed, 2.63 s. That includes `return_lifts_an_original_particle_above_the_waist` and `drip_crosses_the_waist_and_turns_the_wheel`. `lifecycle_mode` and `phase_lifecycle_id` match across `32-CONTEXT.md`, plans 32-01 through 32-04, and their summaries. Research has no lifecycle frontmatter and is not marked `direct-fallback`.

---

_Verified: 2026-09-27T20:48:35Z_
_Verifier: Claude (gsd-verifier)_
