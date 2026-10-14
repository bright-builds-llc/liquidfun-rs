---
phase: 46-cartesian-diver
plan: "01"
subsystem: playground-scene-rust
tags: [scene, cartesian-diver, buoyancy, particles, release-tests, tuning, d-07a]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: "2026-10-14T18:47:00Z"
requires:
  - "45-09: docs/adding-a-scene.md per-scene checklist (R1-R12)"
provides:
  - "SceneId::CartesianDiver registered for parse/build/variant_index/session token (NOT in ALL_SCENE_IDS or SURVEY_SCENES; Plan 02 adds those with the web SCENE_IDS entry)"
  - "scene/cartesian_diver.rs: build(), CartesianDiverHooks (squeeze intent, 15-step ramp, density switch, draw-only dent and bubble)"
  - "scene/cartesian_diver/motion_tests.rs: 11 release-only sink/rise/README/containment tests"
  - "Locked tuning constants (candidate P6) and README cue samples 20 and 110 for Plan 02"
affects: [46-02, 46-03, 46-04, 46-05]
tech-stack:
  added: []
  patterns:
    - "Release-only timing tests gated with cfg_attr(debug_assertions, ignore) inside the existing lib test binary"
    - "Rise target derived from a measured particle-surface percentile (pure test helper) instead of a hard-coded height"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/cartesian_diver.rs
    - crates/liquidfun-wasm/src/scene/cartesian_diver/geometry.rs
    - crates/liquidfun-wasm/src/scene/cartesian_diver/tests.rs
    - crates/liquidfun-wasm/src/scene/cartesian_diver/motion_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs
key-decisions:
  - "46-01: diver tuning locked at candidate P6 (D-07a): particle radius 0.05, 2 particle iterations (no R6 arm), particle damping 0.2, pressure strength 0.25, released/squeezed density 0.3/8.0, body linear damping 0.5"
  - "46-01: the gravity-80 rise is judged against the top third of the water column (95th-percentile particle y measured at release), per D-07a; gravity 2 and 10 keep the bottle top-third target"
  - "46-01: README cues stay at sample 20 (step 60) and sample 110 (step 330); release at 330 passes, so the sample-100 fallback is not needed"
  - "46-01: geometry unchanged from UI-D60 nominal; D-04 '6 particle diameters' applied as diver radius 0.6 m = 6 x 0.1 m diameter"
requirements-completed: [DIVER-01, DIVER-02, DIVER-03, SCENE-02]
duration: about 35 min of execution plus the decision checkpoint pause
completed: 2026-10-14
---

# Phase 46 Plan 01: Cartesian Diver Bridge Scene Summary

**The Cartesian Diver scene now builds natively: a closed bottle of 1,936 water particles holds one sleepless circle diver. A squeeze ramps the diver's fixture density from 0.3 to 8.0 over 15 steps, and the dent and bubble are drawn only. Eleven release-mode tests show it sinking and rising at gravity 2, 10 and 80, with the gravity-80 rise judged against the measured water column (D-07a).**

## Performance

- **Duration:** about 35 min of execution, plus the pause for the Task 2 decision checkpoint
- **Started:** 2026-10-14T18:13Z (after planning)
- **Completed:** 2026-10-14T18:47Z
- **Tasks:** 2 of 2
- **Files:** 4 created, 3 modified

## Accomplishments

- **Task 1 (ce1683223):** Scene module, geometry and hooks. The input model follows D-01 to D-03: Down inside the hit region squeezes, Up or Cancel anywhere releases, Move is ignored, and the `squeeze` action toggles. A 15-step integer ramp drives `set_fixture_density` plus `reset_body_mass_data`, and only runs when the ramp value changes. The draw-only outline has 14 segments with a TAU-based bow, and the drawn bubble shrinks from 0.6R to 0.3R. Rust registration covers the variant, parse, build, `variant_index => 25` and the session token. Debug-safe unit tests are included.
- **Task 2 (f7add9b9a):** Locked candidate P6 and added `motion_tests.rs` with 11 tests, each gated by `#[cfg_attr(debug_assertions, ignore = "release-only diver timing (D-07)")]`. Added pure helpers `water_surface_y` (95th-percentile particle height) and `column_top_third_y` with 3 debug unit tests.

## Final constants

| Constant | Value | Where |
|---|---|---|
| PARTICLE_RADIUS | 0.05 m (stride 0.075) | geometry.rs |
| Grid | 31 x 70 from (-1.15, 0.05); 1,936 particles after the diver disk is skipped (budget 3,000) | geometry.rs |
| PRESSURE_STRENGTH | 0.25 | cartesian_diver.rs |
| PARTICLE_DAMPING (ParticleSystemDef damping) | 0.2 (engine default 1.0) | cartesian_diver.rs |
| RELEASED_DENSITY / SQUEEZED_DENSITY | 0.3 / 8.0 | cartesian_diver.rs |
| DIVER_LINEAR_DAMPING | 0.5 | cartesian_diver.rs |
| Particle iterations | default 2 (no `particle_iterations` arm, D-13) | scene.rs unchanged |
| RAMP_STEPS | 15 (0.25 s) | cartesian_diver.rs |
| DIVER_RADIUS / DIVER_START | 0.6 m / (0, 4.6) | geometry.rs |

**Geometry for Plan 02 (unchanged from UI-D60):**
- Interior: x in [-1.2, 1.2], y in [0, 5.6].
- Outer wall rectangle: x in [-1.6, 1.6], y in [-0.4, 6.0].
- Drawn outline rectangle: x = ±1.45, y from -0.25 to 5.85.
- Hit region: x in [-1.9, 1.9], y in [-0.7, 6.3].

**README cues:** `squeeze` at sample 20 (step 60) and again at sample 110 (step 330). The clip ends at step 600. Measured: the diver reaches the bottom third at step 128 and the top third at step 465.

## Tuning table

The table records sink steps / rise steps. The diver holds for 30 steps after reaching the bottom third, and each phase is capped at 900 steps. "–" means the target was never reached. Windows: g 80 180/240, g 10 180/240, g 2 420/540. The rise target in the sweep was the bottle's top third (y ≥ 3.733) at every gravity.

| Cand. | r | press. | rel/sq | damp | iters | g 80 | g 10 | g 2 | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| A (nominal) | .05 | .2 | .5/5 | 1 | 2 | 14/– (rest 2.43) | 68/214 (rest 3.20) | 184/783 | fail g80, g2 rise, rest |
| B | .05 | .25 | .5/8 | 1 | 2 | 13/– | 64/129 | 164/– | fail |
| C | .05 | .25 | .3/8 | .5 | 2 | 14/– | 67/111 | 146/– (floor) | fail |
| G | .05 | .25 | .3/5 | .5 | 2 | 14/– | 70/206 | 177/617 | fail |
| D | .05 | .25 | .3/8 | .5 | 3 | 23/154 | 64/162 | 184/– (floor) | fail g2, README |
| E | .05 | .25 | .5/8 | 1 | 3 | 21/349 | 76/184 | 231/– (stall 1.12) | fail |
| F | .05 | .2 | .5/5 | 1 | 3 | 21/689 | 89/215 | 257/– | fail |
| H | .05 | .25 | .3/5 | .5 | 3 | 24/93 | 71/163 | 217/– | fail g2 |
| I | .05 | .25 | .3/5 | 1 | 3 | 22/123 | 97/143 | 262/– | fail g2 |
| J | .05 | .2 | .3/5 | 1 | 3 | 20/90 | 93/146 | 264/– (1.93) | fail g2 |
| K | .05 | .25 | .3/3 | .5 | 3 | 24/211 | 106/– (floor) | 339/– | fail |
| L | .05 | .25 | .3/8 | 1 | 3 | 22/154 | 78/167 | 219/– | fail g2 |
| Q3 / Q4 | .05 | .25 | .3/8 | .25 / 0 | 3 | 22/59, 21/61 | 60/154, 57/160 | 169/–, 154/– | fail g2 |
| R6a | .06 | .2 | .5/5 | 1 | 2 | 17/– | 76/274 | 210/– | fail |
| R6d | .06 | .25 | .3/8 | .5 | 3 | 29/58 | 64/160 | 184/– (floor) | fail g2 |
| R6e | .06 | .2 | .3/5 | 1 | 3 | 29/57 | 96/170 | 280/– | fail g2 |
| Q5 (outside list) | .05 | .25 | .3/8 | .5 | 4 | 26/54 | 63/155 | 178/– (floor) | fail |
| 6 or 8 iters (outside list) | .05 | .2 | .5/5 | 1 | 6/8 | released diver stays on floor | stays on floor | stalls | fail |
| P6 (+ particle damping 0.2, outside list) | .05 | .25 | .3/8 | .5 | 2 | 15/– | 68/114 | 142/390 | full suite 9/10, fails only g80 rise |
| D + two 0.4 m floor posts (outside list) | .05 | .25 | .3/8 | .5 | 3 | 23/185 | 68/153 | 191/– (1.55) | full suite 9/10, fails only g2 rise |

**Root cause of the g 80 failure:** The engine caps particle pressure at 0.25 × critical pressure (`crates/liquidfun/src/particle/solver/pressure.rs`). The cap does not depend on `pressure_strength`, so at gravity 80 with 2 iterations the water column collapses to a surface of about 1.7 to 2.9 m. More iterations raise the column, but then a diver that touches the flat floor stays stuck at low gravity.

### Final P6 results under the D-07a targets (release, measured with a temporary probe that was removed before the commit)

| Gravity | Rest y after settle | Sink to y < 1.867 | Surface at release (p95) | Rise target | Diver y at release | Rise | Window | Result |
|---|---|---|---|---|---|---|---|---|
| 80 | 2.413 | 15 steps | 2.573 | column top third 1.715 | 0.607 | 20 steps (max y 3.03 in the next 240) | 180 / 240 | pass |
| 10 | 3.443 (≥ 3.733 after 240 more steps) | 68 steps | 3.254 | bottle top third 3.733 | 0.604 | 114 steps | 180 / 240 | pass |
| 2 | 4.158 | 142 steps | 5.140 | bottle top third 3.733 | 1.211 | 390 steps | 420 / 540 | pass |
| 10, 300-step hold | – | 68 steps | 3.308 | bottle top third 3.733 | 0.605 | 146 steps | – / 240 | pass, no floor stick |
| README (g 10, no settle) | – | bottom third at step 128 (< 330) | – | bottle top third 3.733 | – | top third at step 465 (≤ 597) | – | pass |

Release run: `cargo test -p liquidfun-wasm --release --lib cartesian_diver` gave 44 passed, 0 failed, 0 ignored (11 motion tests plus 33 unit tests). The debug run lists all 11 motion tests as ignored.

## Decisions Made

- Candidate P6 is locked, as recorded in D-07a. The yolo checkpoint auto-selected option 1.
- The gravity-80 rise target is `column_top_third_y(water_surface_y(heights at release))`, computed in the test, not a hard-coded height. `assert_rise_after_hold` also asserts that the diver starts below the target, so the check is never vacuous.
- An extra test, `released_diver_leaves_the_floor_after_a_long_hold_at_gravity_ten`, holds the squeeze for 5 s and then requires a rise within 240 steps. It guards against the floor-stick failure seen with 3 or more iterations.
- D-04 "6 particle diameters" is applied as diver radius 0.6 m = 6 × 0.1 m particle diameter.

## Deviations from Plan

### D-07a amendment (decision checkpoint, not an auto-fix)

**1. [Rule 4 - Decision] Gravity-80 rise judged against the water column**
- **Found during:** Task 2 sweep (previous executor)
- **Issue:** No allowed lever passes D-07 at gravity 2, 10 and 80 together. The engine pressure cap collapses the gravity-80 column below the bottle's top third.
- **Resolution:** Yolo auto-selected option 1, recorded as D-07a in 46-CONTEXT.md (commit 0a2ec3463). This executor implemented it: candidate P6, with the gravity-80 rise measured against the top third of the settled water column within the 240-step window.
- **Files modified:** cartesian_diver.rs, cartesian_diver/geometry.rs, cartesian_diver/motion_tests.rs, cartesian_diver/tests.rs
- **Commit:** f7add9b9a

**2. [Rule 2 - Missing check] Rise tests assert a below-target start**
- **Found during:** Task 2
- **Issue:** At gravity 80, the column top-third target (1.715) lies below the bottle's bottom-third line (1.867). A rise check could therefore pass without the diver moving.
- **Fix:** `assert_rise_after_hold` asserts that the diver y at release is below the rise target before it measures the rise. Measured: 0.607 against 1.715.
- **Commit:** f7add9b9a

### Other notes

- Particle damping (`ParticleSystemDef::with_damping(0.2)`) was not in the plan's lever list. D-07a accepted it explicitly as a scene-local lever.
- The plan's motion-test names are kept, except the gravity-80 rise test, which is named `released_diver_rises_in_the_water_column_at_gravity_eighty` to reflect D-07a.
- `generated_by` follows the plan's `<output>` value (`gsd-execute-phase`), matching earlier SUMMARYs.

## Issues Encountered

- Honest limit: at gravity 80 the water column only reaches about 2.6 m in the 5.6 m bottle, so the diver floats around mid-bottle (rest y 2.41) rather than near the cap. Plan 02's caption and hint should not promise that the diver floats at the top at high gravity.

## Known Stubs

None.

## Next Phase Readiness

- Plan 02 can add `cartesian-diver` to `ALL_SCENE_IDS`, `SURVEY_SCENES` and the web `SCENE_IDS` together. It can copy the geometry rectangles above into the view bounds, portrait walls and hit-region docs, and use README cues at samples 20 and 110.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene/cartesian_diver/motion_tests.rs
- FOUND: crates/liquidfun-wasm/src/scene/cartesian_diver.rs, geometry.rs, tests.rs
- FOUND: commit ce1683223 (Task 1), commit f7add9b9a (Task 2)
- Untouched: `git diff --quiet HEAD~2 -- crates/liquidfun docs/benchmarks/scene-fingerprints.json web`
- Gates: cargo fmt; clippy for `-p liquidfun-wasm` and the workspace (all targets, all features, `-D warnings`); `cargo build --all-targets --all-features`; `cargo test -p liquidfun-wasm --all-features` (634 passed, 36 ignored); the release diver suite (44 passed); `bun scripts/bright-builds-check.ts all` (0 findings)
