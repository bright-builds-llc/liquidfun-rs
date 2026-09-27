---
phase: 29-sparky-drawing-and-full-catalog
verified: 2026-09-27T08:05:34Z
status: passed
score: 18/18 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-09-27T04-14-40
generated_at: 2026-09-27T08:05:34Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 29: Sparky, Drawing, and full catalog Verification Report

**Phase Goal:** Visitors can watch Sparky sparks and paint Drawing Particles, and can open all twelve missing testbed scenes from the catalog while the original six remain.
**Verified:** 2026-09-27T08:05:34Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

The live tree matches the phase goal. `SCENE_IDS` is nineteen ready entries. The twelve v1.3 testbed scenes (Drawing Particles, Elastic Particles, Impulse, Liquid Timer, Particles, Rigid Particles, Soup, Soup Stirrer, Sparky, Surface Tension, Theo Jansen, Wave Machine) are in that list, and the original six (Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel) remain, with Liquid Tumbler still between Theo Jansen and Drawing Particles.

Sparky builds six dynamic circles in the tall chamber and spawns one powder group per sparkable `ContactTransitionKind::Begin` in `SceneHooks::on_after_step`, after `World::step` returns with `NoDecisionHook`. Bursts fade through `World::set_particle_colors` and are destroyed at 0.75 s or when the 16-slot ring reuses a slot. Drawing Particles constructs an empty vessel (radius 0.05, maximum count 10240) and paints on pointer down, move, and an unpaired up: destroy inside a 0.2 circle, then create a water or elastic group. Elastic is `ParticleFlags::ELASTIC` plus `ParticleGroupFlags::SOLID` and does not recreate the world.

`MAX_ADVANCE_STEPS` and `MAX_STEPS_PER_FRAME` stay 4. Catalog and scene module docs describe experimental recognizable ports. Chromium `just web-player-smoke` passed (Playwright 44). Code review `29-REVIEW.md` is clean. Doctests were not executed in the collected evidence.

### Observable Truths

Roadmap success criteria. All four hold.

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Visitor can open Drawing Particles, Elastic Particles, Impulse, Liquid Timer, Particles, Rigid Particles, Soup, Soup Stirrer, Sparky, Surface Tension, Theo Jansen, and Wave Machine from the catalog, and the existing six scenes remain available. | ✓ VERIFIED | `web/src/catalog/scenes.ts` `SCENE_IDS` has all twelve plus `dam-break`, `fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, `water-wheel`, and `liquid-tumbler` (19 ids). New records are `ready: true`. Factory arms `SceneId::DrawingParticles` and `SceneId::Sparky` call `drawing_particles::build` and `sparky::build`. Gravity is split off before those builders. Smoke opens the catalog (Playwright 44 passed). |
| 2 | Visitor can watch Sparky: colliding circles throw fading particle sparks (post-step contact observation; no mid-step world mutation; no FFI expansion). | ✓ VERIFIED | `sparky.rs` six circles, sparkable set excludes walls, powder on Begin only, `set_particle_colors` fade, lifetime 0.75, ring 16. `session.rs` steps with `NoDecisionHook` then `on_after_step(contact_transitions)`. `WorldCommand` stays destroy-body and destroy-fixture. No `set_particle_colors` WASM export. |
| 3 | Visitor can paint Drawing Particles into an empty vessel, and at least one non-water material looks different from plain water. | ✓ VERIFIED | Construction creates no groups. Brush destroy-then-create. Water is `ParticleFlags::WATER` with empty group flags and blue `ParticleColor`. Elastic is `ELASTIC` + `SOLID` and green. Catalog Material preset `recreates: false`. E2E drag asserts `data-particle-count` goes from 0 to greater than 0. |
| 4 | Developer can confirm scene docs and catalog wording describe recognizable ports, not sealed C++ parity, and do not cut particle counts or raise the 4-step catch-up cap to fake smoothness. | ✓ VERIFIED | Drawing module doc names experimental native ports and static catalog previews. Sparky module doc says recognizable behavior. `PAGE_SUMMARY` says nineteen demos on this repository's engine. No sealed-parity or bit-exact wording under `web/src/catalog` or `web/src/player/runtime.ts`. Both scenes use maximum count 10240 and `with_destruction_by_age(false)`. `MAX_ADVANCE_STEPS: u32 = 4`. `MAX_STEPS_PER_FRAME = 4`. Sparky ring comment records the playground adaptation (16 versus upstream `c_maxVFX` 50). |

**Score:** 4/4 roadmap success criteria verified. 18/18 plan frontmatter truths verified below.

### Supporting plan truths (also verified)

| Truth | Status | Evidence |
| --- | --- | --- |
| A non-zero particle color can be replaced for many particles in one Rust call, and the copied color lane shows that color. | ✓ | `World::set_particle_colors` writes `set_particle_colors_internal` after every id resolves. `particle_color_batch.rs` covers the replace. |
| A group created with `ParticleColor::ZERO` does not gain a color lane from the batch writer. | ✓ | `MissingColorLane` when `maybe_colors()` is `None`; Display string `particle color lane is not allocated`. |
| The batch writer does not add a network call, an FFI export, or a per-particle WASM write. | ✓ | Writer lives in `liquidfun`. WASM `lib.rs` has no color-batch export. |
| Drawing Particles starts as an empty open vessel the visitor can recognize. | ✓ | Four pinned walls, radius 0.05, zero groups. Catalog description and static preview case `drawing-particles`. |
| Pointer down or move destroys particles in a 0.2 circle, then creates a stroke. Pointer up clears the join. An up with no open stroke leaves one stamp and then clears. | ✓ | `should_stamp` / `stamp` / Up and Cancel clear `maybe_last_group` and `stroke_open`. |
| Water flows with no special group flags. Elastic clumps because particles are `ELASTIC` and the group is `SOLID`. Strokes join only while group flags still match. | ✓ | `Material::particle_flags` / `group_flags`. `join_destination` uses `AppendTo` only when group flags equal the current material. |
| Material does not recreate the world. Reset drops the join and returns water. Particle count is not cut and destruction-by-age stays off. | ✓ | `apply_control` returns `ControlEffect::Live`. Session reset rebuilds from presets; default material is water and `material: "water"`. Maximum count 10240. Destruction-by-age false. |
| Sparky opens as six dynamic circles in the tall chamber. Walls are not sparkable. | ✓ | `CIRCLE_COUNT = 6`, staggered centers, `sparkable_bodies` only. Four wall polygons via `attach_basin_fixture`. |
| After a step unlocks, each new sparkable Begin throws one powder burst. Persist and End do not. Other scenes still step with `NoDecisionHook`. | ✓ | `spawn_from_begins` filters `Begin` and sparkable bodies. Default `on_after_step` is `Ok(())`. Both advance paths use `NoDecisionHook`. |
| Bursts fade in the Rust color lane, die at 0.75 s by destroying their particles, and a 16-slot ring destroys the previous group before reuse. | ✓ | `SPARK_LIFETIME_SECONDS = 0.75`, `write_fade` → `set_particle_colors`, `release_slot` destroys then compacts before overwrite. `SPARK_RING_LEN = 16`. |
| Pointer input does nothing. Reset drops every burst by rebuilding the session. Rigid capture caps stay 64 and 48. | ✓ | Sparky `apply_pointer` returns `Ok(())`. Reset is session rebuild. `MAX_RIGID_SEGMENTS = 64`, `MAX_RIGID_CIRCLES = 48`. |
| The catalog lists nineteen ready scenes, ending liquid-tumbler, drawing-particles, sparky, and the original six stay in place. | ✓ | `SCENE_IDS` order matches. No `ready: false` in `scene-records.ts`. |
| Drawing Particles offers Water and Elastic. Elastic does not recreate the world. Sparky is watch-first aside from Gravity. | ✓ | Material preset options water/elastic, `runtimePreset` sets `recreates: false`. Sparky hint is `WATCH_FIRST_HINT` and controls are `withGravitySlider([])`. |
| Each new scene has a captioned static preview, a phone frame that includes its walls, and a README plan id. | ✓ | `previews.tsx` cases. Portrait rectangles `minX: -4.2` and `minX: -22`. README plans: drawing pointer-up at `(0, 2)`, sparky `cues: []`. Captions stay `Static preview`. |
| Visitor copy says nineteen experimental demos and does not claim sealed parity. | ✓ | `PAGE_SUMMARY` in `runtime.ts`. Shell spec asserts `All nineteen demos`. |
| Chromium smoke opens, plays, pauses, and resets both new scenes and the scenes already in the catalog. | ✓ | Orchestrator: `just web-player-smoke` exit 0, Playwright 44 passed. Hash paths for both new ids are in `player-helpers.ts`. |
| One Drawing drag leaves particles. Sparky stays on the watch-first loop. | ✓ | `POINTER_CONTROL` maps only `drawing-particles` among the new ids. Spec asserts particle count 0 then greater than 0, and watch-first ids have no pointer mapping. |
| Gravity is not treated as a pointer gesture. The shell still has zero catalog cards, and the drawer last link is Sparky. | ✓ | Interactive ids are `POINTER_CONTROL` keys, not `controls.length`. `shell.spec.ts` keeps `.catalog-card` count 0 and last drawer link `/Sparky/`. |

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun/src/world/particle_object/system.rs` | `World::set_particle_colors` | ✓ VERIFIED | Public method; empty slice no-op; missing lane does not allocate |
| `crates/liquidfun/src/world/object/tests/particle_color_batch.rs` | Focused batch color tests | ✓ VERIFIED | Replace, empty slice, zero-color lane, stale id |
| `crates/liquidfun-wasm/src/scene/drawing_particles.rs` | Empty vessel plus water and elastic paint | ✓ VERIFIED | `build`; brush 0.2; flags and join |
| `crates/liquidfun-wasm/src/scene.rs` | Parse and build arms | ✓ VERIFIED | `"drawing-particles"` and `"sparky"` |
| `crates/liquidfun-wasm/src/scene/sparky.rs` | Chamber, sparkable circles, bounded bursts | ✓ VERIFIED | `build` plus `on_after_step` override |
| `crates/liquidfun-wasm/src/session.rs` | `on_after_step` after `NoDecisionHook` | ✓ VERIFIED | `advance` and `advance_profiled` |
| `web/src/catalog/scene-records.ts` | `SCENES` including the two new records | ✓ VERIFIED | Appended after Liquid Tumbler |
| `web/src/catalog/previews.tsx` | Static Drawing and Sparky SVGs | ✓ VERIFIED | Exhaustive switch cases |
| `web/src/catalog/portrait-bounds.ts` | Phone rectangles | ✓ VERIFIED | Both ids keyed |
| `web/scripts/readme-svg/plans.ts` | README plan coverage | ✓ VERIFIED | Both ids; `assertReadmeSvgPlanCoverage` |
| `web/src/components/PlaygroundStage.tsx` | `data-particle-count` | ✓ VERIFIED | `maybeFrame()?.particleCount ?? 0` |
| `web/e2e/player.spec.ts` | Pointer versus watch-first split | ✓ VERIFIED | Drawing drag mapping |
| `web/e2e/shell.spec.ts` | Nineteen-demo shell asserts | ✓ VERIFIED | Cards stay 0; last link Sparky |

gsd-tools `verify artifacts` and `verify key-links` returned `all_passed` / `all_verified` for plans 29-01 through 29-05.

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `system.rs` | `storage/runtime.rs` | validated indices into `maybe_colors` | ✓ WIRED | `set_particle_colors` |
| `scene.rs` | `drawing_particles.rs` | `build_scene(SceneId::DrawingParticles)` | ✓ WIRED | `drawing_particles::build` |
| `drawing_particles.rs` | `World::destroy_particles_in_shape` | brush before `create_particle_group` | ✓ WIRED | radius 0.2 |
| `session.rs` | `sparky.rs` | `SceneHooks::on_after_step` | ✓ WIRED | after step unlock |
| `sparky.rs` | `World::set_particle_colors` | one batch write per live burst | ✓ WIRED | `write_fade` |
| `scenes.ts` | `scene-records.ts` | `SCENE_IDS` and `SCENES` order | ✓ WIRED | `drawing-particles` then `sparky` |
| `plans.ts` | `scenes.ts` | `assertReadmeSvgPlanCoverage` | ✓ WIRED | both plan ids |
| `player.spec.ts` | `PlaygroundStage.tsx` | Drawing drag reads `data-particle-count` | ✓ WIRED | attribute on `<main>` |
| `player.spec.ts` | `scenes.ts` | interactive ids are `POINTER_CONTROL` keys | ✓ WIRED | Sparky excluded |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `PlaygroundStage.tsx` | `maybeFrame()?.particleCount` | WASM frame `particleCount` | Yes — copied session particle count | ✓ FLOWING |
| `sparky.rs` fade | color lane | `set_particle_colors` from stored original color | Yes — per-slot coefficient, alpha 255 | ✓ FLOWING |
| `drawing_particles.rs` | particle groups | pointer stamp `create_particle_group` | Yes — brush samples, not a fixed preload | ✓ FLOWING |
| Catalog `SCENES` | scene records | static `scene-records.ts` | Yes — authored catalog contract, every id `ready: true` | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Chromium catalog smoke, including Drawing drag and Sparky watch-first | `just web-player-smoke` | Exit 0. Playwright 44 passed (orchestrator, plan 29-05). | ✓ PASS |
| Native and WASM suites for color write, paint, and sparks | `cargo test -p liquidfun -p liquidfun-wasm` | 74 suites `test result: ok`. `liquidfun-wasm` lib tests 209 passed (orchestrator). | ✓ PASS |
| Doctests | `cargo test --doc` | Compile was stopped while rustc sat in uninterruptible wait. | ? SKIP — not executed; not treated as a code failure |
| Batch writer, scene factory, catalog ids, step cap | source inspection | Patterns above present. `WorldCommand` has no create or color variant. | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| PLAY-01 | 29-04, 29-05 | Visitor can open the twelve named scenes, and the existing six remain available. | ✓ SATISFIED | Nineteen-id catalog and factory; original six still present; Chromium smoke. Tracker checkbox in `REQUIREMENTS.md` is still Pending. |
| FX-01 | 29-01, 29-03, 29-04, 29-05 | Visitor can watch Sparky: colliding circles throw fading particle sparks. | ✓ SATISFIED | Post-step powder bursts, color fade, watch-first catalog. Tracker checkbox still Pending. |
| FX-02 | 29-02, 29-04, 29-05 | Visitor can paint Drawing Particles into an empty vessel, and at least one non-water material looks different from plain water. | ✓ SATISFIED | Empty vessel, water versus elastic flags and colors, paint drag. Tracker checkbox still Pending. |

No phase-29 requirement ID is missing from PLAN frontmatter. `DRAW-02`, `PRESET-01`, `PARITY-01`, and `BOX2D-01` stay future requirements and are not claimed by this phase.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| — | — | None | — | No TODO, FIXME, or placeholder in the scene, color-writer, or catalog record files. A failed paint create at the 10240 cap returns `Ok(())` after the brush destroy; plan 29-02 specifies that. |

### Human Verification Required

None. Objective catalog order, factory allowlist, paint and spark wiring, step-cap checks, and the recorded Chromium smoke are agent-verified. That matches prior playground phases in this repo: recognizable ports are gated by smoke plus focused tests.

### Gaps Summary

No gaps. Phase 30 does not own any unmet Phase 29 criterion.

---

_Verified: 2026-09-27T08:05:34Z_
_Verifier: Claude (gsd-verifier)_
