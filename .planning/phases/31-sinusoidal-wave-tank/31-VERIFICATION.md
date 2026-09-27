---
phase: 31-sinusoidal-wave-tank
verified: 2026-09-27T18:49:05Z
status: passed
score: 10/10 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-09-27T17-22-43
generated_at: 2026-09-27T18:49:05Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 31: Sinusoidal Wave Tank Verification Report

**Phase Goal:** Visitors can watch Wave Tank: a still pool whose end platform rises and falls and sends a wave toward the far wall, without changing Wave Machine.
**Verified:** 2026-09-27T18:49:05Z
**Status:** passed
**Re-verification:** No — initial verification

This report is the phase goal check. It is not an independent review acknowledgment. The phase detail in `ROADMAP.md` now states that goal; `31-CONTEXT.md` decisions D-01 through D-11 are the locked contract.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Scene id `wave-tank` builds one still water group and a dynamic end platform on a vertical prismatic joint. | ✓ VERIFIED | `SceneId::WaveTank` parses `"wave-tank"` and `build_scene` calls `wave_tank::build`. The platform is `BodyType::Dynamic`. The joint uses `PrismaticJointDef` with axis `(0.0, 1.0)`, limits outside the stroke, and motor force `1.0e6`. `on_advance` adds `1/60` and writes `PEAK_SPEED * sin(TAU * elapsed / PERIOD)` through `set_prismatic_motor_speed`. One `ParticleGroupRecipe` is created with radius `0.025` and damping `0.2`, and the module does not call `with_particle_flags`. |
| 2 | After a half period of `SessionCore::advance(4)` batches, platform translation is at least half the stroke, the live particle count is unchanged, and at least one particle id that began in the far-wall sample has moved up by one diameter. | ✓ VERIFIED | `far_wall_particles_rise_after_a_half_cycle` snapshots ids with start `x >= 1.20`, runs fifteen `advance(4)` calls (60 steps, period `2.0`), and asserts translation `>= 0.5 * 0.16`, an unchanged live count, and one original id up by `0.05`. `pool_starts_as_a_still_band` requires a non-empty far window, zero velocities, and a level surface. `motor_speed_follows_the_sine` checks the first sine sample and a near-zero crest speed. Recorded `cargo test -p liquidfun-wasm --lib` passed 226 tests, including that far-wall test. |
| 3 | `wave-machine.rs` and `hydraulic_fountain.rs` are unchanged, and `MAX_ADVANCE_STEPS` stays 4. | ✓ VERIFIED | Commits from `1700169` through `HEAD` do not touch those scene files or `wave_machine/drive.rs`. Their latest commits remain the Phase 30 piston work. `MAX_ADVANCE_STEPS: u32 = 4` and `MAX_STEPS_PER_FRAME = 4`. |
| 4 | The catalog lists twenty-one ready scenes, with `wave-tank` immediately after `hydraulic-fountain`, and the wave-machine record is unchanged. | ✓ VERIFIED | `SCENE_IDS` starts with `wave-machine` and ends `hydraulic-fountain`, `wave-tank`. The Wave Machine description is still "Watch a motorized tank rock and slosh the water inside." Wave Tank is `ready: true`. |
| 5 | Wave Tank is watch-first aside from the shared gravity slider, and its static preview shows a level pool with one end platform above the resting floor. | ✓ VERIFIED | Controls are `withGravitySlider([])` and the hint is `WATCH_FIRST_HINT`. Gravity recreates the session; `period`, `amplitude`, and `stroke` are rejected as `UnknownControl`. `WaveTankPreview` draws three walls, a raised slab, a vertical face, and a water rect, with no transform or animation. `Static preview` is not in the SVG. |
| 6 | The phone frame contains the pool walls and the platform at rest and at the crest, and the README plan id is `wave-tank` with empty cues. | ✓ VERIFIED | `viewBounds` and `PORTRAIT_VIEW_BOUNDS["wave-tank"]` are `{ minX: -0.12, minY: -0.28, maxX: 1.52, maxY: 1.02 }`. Portrait endpoints include walls through `y = 0.9` and `x = 1.44`, the slab and face at rest, and the crest points at `y + 0.16`, all inside that rectangle. `MIN_HEIGHT_FRACTION` is `0.20`. README plan `{ id: "wave-tank", cues: [] }` follows hydraulic-fountain. |
| 7 | Credits point at `wave_tank.rs` and the LiquidFun showcase link only. | ✓ VERIFIED | Implementation path is `crates/liquidfun-wasm/src/scene/wave_tank.rs`. Inspiration is `[SHOWCASE]` (`https://google.github.io/liquidfun/`). The record has no pinned LiquidFun test URL. |
| 8 | Chromium smoke opens, pauses, plays, and resets Wave Tank, and still opens the scenes that were already in the catalog. | ✓ VERIFIED | `wave-tank` is absent from `POINTER_CONTROL`, so `WATCH_FIRST_SCENE_IDS` includes it. That loop opens `SCENE_HASH_PATHS["wave-tank"]` (`#/scene/wave-tank`), then pause, play, and reset. Recorded `just web-player-smoke` exited 0 with 44 Chromium tests. |
| 9 | The shell reports twenty-one demos, twenty-one static previews, zero catalog cards, and a drawer whose last link is Wave Tank. | ✓ VERIFIED | `PAGE_SUMMARY` says `All twenty-one demos`. `shell.spec.ts` expects that phrase, 21 `Static preview` captions, 21 hidden SVGs, `.catalog-card` count 0, and a link named `/Wave Tank/`. The drawer uses 23 Tab and 23 Shift+Tab presses over 22 focusables. |
| 10 | The implementing agent records evidence and does not approve its own work. | ✓ VERIFIED | `31-01-SUMMARY.md`, `31-02-SUMMARY.md`, and `31-03-SUMMARY.md` each state that the implementing agent did not approve the work. This verification file is the goal check only. |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/liquidfun-wasm/src/scene/wave_tank.rs` | Scene build, vertical prismatic motor, and sine speed in `on_advance` | ✓ VERIFIED | 335 lines. Dynamic platform, `set_prismatic_motor_speed`, one water group, live platform segments. |
| `crates/liquidfun-wasm/src/scene/wave_tank/tests.rs` | Still-band and far-wall vertical motion tests | ✓ VERIFIED | Still pool, half-cycle rise, sine speed, rebuild at translation 0, rejected controls. |
| `crates/liquidfun-wasm/src/scene.rs` | `SceneId::WaveTank` and parse token `wave-tank` | ✓ VERIFIED | Enum, parse arm, `mod wave_tank`, and `build_scene` route. |
| `web/src/catalog/scene-records.ts` | Wave Tank record with watch-first controls | ✓ VERIFIED | Ready record after hydraulic-fountain. Wave Machine record still rocks the tank. |
| `web/src/catalog/previews.tsx` | Static SVG of a level pool and a raised end platform | ✓ VERIFIED | `WaveTankPreview` wired from the scene switch. |
| `web/src/catalog/portrait-bounds.ts` | Phone rectangle for `wave-tank` | ✓ VERIFIED | Matches the record `viewBounds` and contains rest and crest endpoints. |
| `web/scripts/readme-svg/plans.ts` | README plan coverage with `cues: []` | ✓ VERIFIED | Entry follows hydraulic-fountain. |
| `web/e2e/shell.spec.ts` | Twenty-one-demo shell asserts and drawer wrap onto Wave Tank | ✓ VERIFIED | Preview counts and `/Wave Tank/` last link. |
| `web/e2e/player.spec.ts` | Existing watch-first loop | ✓ VERIFIED | New id is absent from `POINTER_CONTROL`. |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `wave_tank.rs` | `World::set_prismatic_motor_speed` | `on_advance` before `World::step` | WIRED | `SessionCore::advance` calls `on_advance`, then `world.step`. The speed function is `prismatic.rs`. |
| `session.rs` | `wave_tank.rs` | `SessionCore::advance` | WIRED | Scene hooks stored by `build_scene` receive that call. Tests step only through `advance`. |
| `capture_frame` | platform edges | `collect_segments` | WIRED | Frame capture copies hook segments. The slab and face use `body_snapshot(platform).transform()`. |
| `scenes.ts` | `scene-records.ts` | shared id after hydraulic-fountain | WIRED | Both lists end with `wave-tank`. |
| `readme-svg/plans.ts` | `scenes.ts` | `assertReadmeSvgPlanCoverage` | WIRED | Empty-cue plan matches the last catalog id. |
| `scene-records.ts` | `wave_tank.rs` | implementation path | WIRED | Host-locked single-file scene path. |
| `player.spec.ts` | `scenes.ts` | `WATCH_FIRST_SCENE_IDS` | WIRED | Every id missing from `POINTER_CONTROL` is watch-first, including `wave-tank`. |
| `shell.spec.ts` | `runtime.ts` | `All twenty-one demos` | WIRED | Footer phrase and smoke assert match. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `wave_tank.rs` particles | water group positions | `create_water_group` polygon fill | Yes | ✓ FLOWING |
| `collect_segments` platform | body transform | `world.body_snapshot(self.platform)` | Yes | ✓ FLOWING |
| Catalog preview | static SVG | `WaveTankPreview` | Artwork, not a live sim | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Far-wall particles rise after a half cycle | `cargo test -p liquidfun-wasm --lib` | Recorded pass, 226 tests, including `far_wall_particles_rise_after_a_half_cycle` | ✓ PASS |
| Catalog play, pause, and reset | `just web-player-smoke` | Recorded exit 0, 44 Chromium tests | ✓ PASS |

Spot-checks use the recorded runs named in the verification request. The current sources still contain those assertions.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| none | 31-01, 31-02, 31-03 | Phase requirement IDs are empty. Decisions D-01 through D-11 are the contract. | ✓ SATISFIED | No requirement IDs were invented. REQUIREMENTS.md does not map this phase, and PLAY-03 stays on Phase 26. |

D-01 through D-11 hold in the sources: new id after hydraulic-fountain, still pool with a moving end and a fixed far wall, dynamic vertical prismatic sine motor, watch-first controls, one water group, step cap 4, ready catalog record, portrait frame and empty-cue README plan, showcase-only credit, recorded Chromium smoke, and no self-approval in this file.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `wave_tank.rs` | 330 | `collect_circles` returns an empty list | ℹ️ Info | The platform is drawn as box segments. Empty circles match the plan. |
| none | — | `unwrap`, kinematic `set_transform`, extra particle flags, TODO/FIXME, `testWaveMachine` | — | Not present in the scene module. |

### Human Verification Required

None. D-10 names the recorded Chromium smoke and the far-wall particle test as the gates for this scene.

### Gaps Summary

No gaps. The scene visitors can open is a still pool whose dynamic end platform is driven by a sine of simulation time. After a half cycle an original far-wall particle has risen by one diameter while the live count stays the same. Wave Machine remains the rocking tank.

---

_Verified: 2026-09-27T18:49:05Z_
_Verifier: Claude (gsd-verifier)_
