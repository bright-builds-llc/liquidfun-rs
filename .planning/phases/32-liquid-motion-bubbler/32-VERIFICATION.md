---
phase: 32-liquid-motion-bubbler
verified: 2026-09-27T20:15:01Z
status: gaps_found
score: 11/12 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T20:15:01Z
lifecycle_validated: true
overrides_applied: 0
gaps:
  - truth: "A quiet return lifts liquid back to the upper reservoir so the drip continues for the whole play session."
    status: failed
    reason: "The plate rises on a slow vertical motor, but the shaft inlet at rest is narrower than one particle diameter, so liquid cannot rest on the plate and spill back over the divider."
    artifacts:
      - path: crates/liquidfun-wasm/src/scene/liquid_bubbler.rs
        issue: "PLATE_FLOOR_CLEARANCE raises the plate to y 0.02..0.10. The divider ends at y 0.14, leaving a 0.04 m opening. Particle diameter is 0.05 m. Polygon skin is another 0.01 m on each fixture. Horizontal clearance from the divider (x 0.56) to the plate (x 0.58) is 0.02 m. Particles cannot enter the shaft on top of the plate. Once the plate rises, the open path is under the plate, and the downward stroke pushes that liquid back out the floor inlet. The spill lip is the divider top at y 1.62; the plate top at full stroke is y 1.30."
    missing:
      - "Open the shaft inlet wider than one particle diameter so liquid that has already crossed the waist can sit on the plate."
      - "Carry that liquid above the divider spill lip and back into the upper reservoir without set_particle_position or a revolute motor."
      - "Add a SessionCore run past one dwell-rise cycle that shows an original particle id is back above the waist while live count stays constant."
---

# Phase 32: Liquid motion bubbler Verification Report

**Phase Goal:** Visitors can watch Liquid Bubbler: colored liquid drips through a narrow waist and turns a small wheel, without changing Water Wheel.
**Verified:** 2026-09-27T20:15:01Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

The opening spectacle is in the code and the native tests pass. The locked return (D-04) does not. The floor clearance that kept the plate at translation 0 also closed the shaft inlet.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Visitors can watch colored liquid drip through a narrow waist and turn a small wheel, without changing Water Wheel. | ✓ VERIFIED | `drip_crosses_the_waist_and_turns_the_wheel` passed on re-run. Amber `ParticleColor::new(242, 176, 64, 255)` is copied through `session.rs` color lanes into `canvas.ts`. `git diff 277710c^..HEAD` is empty for `water_wheel.rs`. |
| 2 | Catalog id `liquid-bubbler` is appended after `wave-tank`, titled Liquid Bubbler, and does not replace or retune Water Wheel. | ✓ VERIFIED | `SCENE_IDS` ends `wave-tank`, `liquid-bubbler`. Record is `ready: true`. Water Wheel still aims a jet. |
| 3 | Layout is an upper reservoir, one static waist, a lower chamber, and a wheel under the drip. | ✓ VERIFIED | Six static ground boxes. Lips leave a 0.14 m gap at y 0.90..0.98. Hub is `(0.0, 0.40)`. Water polygon is y 1.12..1.50. No jet and no shelf stack. |
| 4 | The wheel is a dynamic paddle wheel on a motor-off revolute joint, turned by particles. | ✓ VERIFIED | `RevoluteJointDef::new` plus `with_frame` only. `wheel_motor_stays_off_and_the_plate_is_beside_the_wheel` passed. No `set_revolute_motor_speed`. |
| 5 | A quiet return lifts liquid back to the upper reservoir so the drip continues for the whole play session, and reset restores the initial layout. | ✗ FAILED | The motor schedule exists (`dwell 3 s`, then `0.15` m/s), but the inlet is 0.04 m and the particle diameter is 0.05 m. Reset itself works: `rebuild_restores_the_reservoir` passed, and the player rebuilds a session on reset. |
| 6 | One plain water group uses one distinct color. Color Mixer is untouched. | ✓ VERIFIED | Default `ParticleFlags::WATER` plus one `with_color`. No mixing, tensile, elastic, or rigid flags. `color_mixer.rs` has no phase diff. |
| 7 | `MAX_ADVANCE_STEPS` stays 4, and radius and count were not cut. | ✓ VERIFIED | `MAX_ADVANCE_STEPS: u32 = 4` and `MAX_STEPS_PER_FRAME = 4`. Radius stays `0.025`. No `with_maximum_count`. No `playground adaptation` comment, which is correct because radius was not shrunk. |
| 8 | The catalog entry is ready, watch-first aside from gravity, hash-routed, and the existing shell stays free of `.catalog-card`. | ✓ VERIFIED | Description matches the one-behavior copy. Controls are `withGravitySlider([])`. `navigation.ts` builds `#/scene/${id}`. `DemoNavigation` captions `Static preview`. Shell spec still expects `.catalog-card` count 0. `PlaygroundShell.tsx` was not edited. |
| 9 | A portrait frame contains the vessel, and the README plan id is `liquid-bubbler` with empty cues. | ✓ VERIFIED | `PORTRAIT_VIEW_BOUNDS["liquid-bubbler"]` matches `viewBounds`. Portrait endpoints include walls, lips, hub, paddle tips, and the plate at rest and at stroke. `plans.ts` has `{ id: "liquid-bubbler", cues: [] }` after wave-tank. |
| 10 | Credits stay host-locked to this scene module and describe an original experimental scene, not a pinned LiquidFun test. | ✓ VERIFIED | `implementationPath` is `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs`, which `sceneBlobUrl` accepts. Inspiration is only the LiquidFun showcase URL. Description says original experimental scene. |
| 11 | A native test shows an original above-waist particle below the waist and a wheel angle change. Chromium smoke is wired to open, pause, play, and reset the scene with the rest of the catalog. | ✓ VERIFIED | Re-ran `cargo test -p liquidfun-wasm scene::liquid_bubbler`: 8 passed, including the crossing test (angle floor 0.05 rad, original ids, live count unchanged). `liquid-bubbler` is absent from `POINTER_CONTROL`, so `WATCH_FIRST_SCENE_IDS` includes it. Shell spec expects 22 previews and a drawer wrap onto `/Liquid Bubbler/`. Playwright was not re-run in this pass. |
| 12 | The implementing agent does not approve its own work. | ✓ VERIFIED | All three summaries state that they record evidence and do not approve. No review acknowledgment names the implementer as reviewer. |

**Score:** 11/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` | Static waist, motor-off wheel, delayed plate, one amber group | ✓ VERIFIED | 491 lines, substantive, built from `SceneId::LiquidBubbler`. Return inlet is too narrow (gap, not a stub). |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` | Waist crossing and wheel-angle tests | ✓ VERIFIED | Eight tests. Crossing, dwell, motor, and rebuild passed on re-run. `source_lifts_with_a_prismatic_plate` only reads source text. |
| `crates/liquidfun-wasm/src/scene.rs` | Parse token and `build_scene` route | ✓ VERIFIED | Enum, `"liquid-bubbler"` arm, and build arm are present. `lib.rs` `build_core` calls `parse_scene_id`. |
| `web/src/catalog/scene-records.ts` | Watch-first Liquid Bubbler record | ✓ VERIFIED | After wave-tank. Gravity slider only. |
| `web/src/catalog/previews.tsx` | Still SVG of a waist, amber drip, and wheel | ✓ VERIFIED | `LiquidBubblerPreview` uses `#F2B040`, a lip gap, a drip rect, and a wheel at `cx="69" cy="64"`. |
| `web/src/catalog/portrait-bounds.ts` | Phone rectangle | ✓ VERIFIED | Same rectangle as `viewBounds`. |
| `web/scripts/readme-svg/plans.ts` | `{ id: "liquid-bubbler", cues: [] }` | ✓ VERIFIED | Last plan, catalog order. |
| `web/e2e/shell.spec.ts` | Twenty-two demos and drawer wrap | ✓ VERIFIED | `All twenty-two demos`, preview counts 22, last link `/Liquid Bubbler/`. |
| `web/e2e/player.spec.ts` | Existing watch-first loop | ✓ VERIFIED | No `liquid-bubbler` pointer mapping, so the existing loop covers the hash. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `session.rs` `advance` | `liquid_bubbler.rs` `on_advance` | `on_advance` then `World::step` | ✓ WIRED | `SessionCore::advance` calls hooks before the step. |
| `on_advance` | prismatic motor | `set_prismatic_motor_speed` | ✓ WIRED | Speed is 0 for the first 3 s, then ±0.15 m/s. |
| `lib.rs` `build_core` | `liquid_bubbler::build` | `parse_scene_id("liquid-bubbler")` | ✓ WIRED | Player scene id reaches the native builder. |
| `scenes.ts` | `scene-records.ts` | shared id after wave-tank | ✓ WIRED | Hash href is `#/scene/liquid-bubbler`. |
| `scene-records.ts` | `liquid_bubbler.rs` | host-locked path | ✓ WIRED | Single file name under `scene/`. |
| particle recipe color | canvas fill | frame color lane | ✓ FLOWING | `with_color` values are copied in `capture` and drawn by `particleColor`. |
| side-shaft plate | upper reservoir | lift then spill over the divider | ✗ NOT_WIRED | Particles cannot reach the top of the plate, so the spill lip never receives them. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| Canvas particles | `particleColors` | `ParticleGroupRecipe::with_color` through `SessionCore` frame capture | Yes, amber `(242, 176, 64, 255)` | ✓ FLOWING |
| Canvas wheel | body snapshot transform | `collect_circles` / `collect_segments` on the live wheel body | Yes | ✓ FLOWING |
| Side-shaft return | plate translation | prismatic speed from elapsed time | Motor moves; liquid does not ride it back | ✗ DISCONNECTED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Original particle crosses the waist and the wheel angle leaves 0 | `cargo test -p liquidfun-wasm scene::liquid_bubbler -- --test-threads=1` | 8 passed, 0 failed, 0.76 s | ✓ PASS |
| Plate stays down and revolute motor stays off during the 2 s proof | same command | `plate_stays_down_during_the_proof` and `wheel_motor_stays_off_and_the_plate_is_beside_the_wheel` passed | ✓ PASS |
| Reset rebuilds a still reservoir and a resting wheel | same command | `rebuild_restores_the_reservoir` passed | ✓ PASS |
| Chromium opens, pauses, plays, and resets Liquid Bubbler | `just web-player-smoke` | Not re-run here (starts a preview server; prior claim was 44 passed in 38.5 s) | ? SKIP |
| Liquid returns above the waist after a full lift cycle | none | No test advances past the 3 s dwell. Geometry blocks the inlet. | ✗ FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| none | 32-01, 32-02, 32-03 | No PLAY-* id is assigned to Phase 32. PLAY-01, PLAY-02, and PLAY-03 are already complete on earlier phases. | ✓ SATISFIED | Empty `requirements:` on every plan. No orphaned Phase 32 id in `REQUIREMENTS.md`. |

Locked decisions D-01 through D-03 and D-05 through D-11 match the code. D-04 does not.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler/tests.rs` | `source_lifts_with_a_prismatic_plate` | Source-text assertions (`include_str!`) | ⚠️ Warning | The passing test checks that the file mentions `set_prismatic_motor_speed`. It does not show liquid returning to the reservoir. |
| `crates/liquidfun-wasm/src/scene/liquid_bubbler.rs` | `on_advance` | Reads `revolute_joint_angle` and discards the value | ℹ️ Info | Used only as a fallible read before writing plate speed. The revolute motor stays off. |

No TODO, FIXME, placeholder, or production `unwrap()` in the scene module.

### Human Verification Required

None for this verdict. The return gap is geometric and does not need a visual pass to confirm. A browser watch of the opening drip will not show whether liquid later returns to the reservoir.

### Gaps Summary

D-04 asks for a quiet return that lifts liquid back to the upper reservoir so the drip lasts the whole session. The scene does schedule a side-shaft plate: speed 0 for 3 seconds, then 0.15 m/s up and down, which is much quieter than Hydraulic Fountain. The rest pose that satisfied the dwell test closes the inlet. The plate occupies y 0.02..0.10 and the divider stops at y 0.14, so the opening is 0.04 m. Particles are 0.05 m across, and the project's own waist rule is that a gap narrower than one diameter stops the stream. Liquid therefore stays in the lower chamber or, after the plate rises, under the plate. The downward stroke pushes it back through the floor gap. It does not cross the spill lip at y 1.62. Reset of a fresh session still restores the reservoir, the wheel, and the particles.

The roadmap spectacle itself holds. Re-running the scene tests confirmed an original particle below the waist and a wheel angle of at least 0.05 rad within 2 seconds, with the revolute motor disabled and Water Wheel, Color Mixer, Liquid Timer, and Hydraulic Fountain unchanged. `lifecycle_mode` and `phase_lifecycle_id` match across `32-CONTEXT.md`, all three plans, and all three summaries. Research has no lifecycle frontmatter and is not marked `direct-fallback`.

---

_Verified: 2026-09-27T20:15:01Z_
_Verifier: Claude (gsd-verifier)_
