---
phase: 48-fake-ferrofluid
plan: "01"
subsystem: playground-scene-rust
tags: [scene, fake-ferrofluid, particle-force, gravity-frame, polynomial-field, tensile, viscous, release-tests, spike, d-05]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: "2026-10-15T05:35:40.803Z"
requires:
  - "45-09: docs/adding-a-scene.md per-scene checklist (R1-R14)"
  - "47-01: build/build_parts split, release-only spike gating, preset rejection, push_loop dish outlines"
provides:
  - "scene/particle_force.rs: GravityFrame (up = -g/|g|, across = (up.y, -up.x)), ForceScale (density * (PARTICLE_STRIDE * 2r)^2 * iterations from the live definition), ParticleScratch, apply_accelerations, apply_group_acceleration (Phase 49)"
  - "SceneId::FakeFerrofluid registered for parse/build/variant_index/session token (NOT in ALL_SCENE_IDS or SURVEY_SCENES; Plan 03 adds those with the web SCENE_IDS entry)"
  - "scene/fake_ferrofluid.rs: build(), build_parts(), FakeFerrofluidHooks (magnet on at rest, strength 65 %), on_advance applying the scripted pull plus drag once per step through the helper"
  - "scene/fake_ferrofluid/field.rs: FieldParams, FIELD (cap 1.5), polynomial bump/bump_integral/columns, envelope, magnet_acceleration"
  - "scene/fake_ferrofluid/geometry.rs: dish quads and segments, WALL_TOP_Y 1.8, MAGNET_REST, puddle box, FRAME_* bounds"
  - "scene/fake_ferrofluid/spike_tests.rs: 11 release-only D-05 regression guards (P1-P5)"
  - "Candidate (a) verdict: PASS; candidate (b) never built"
affects: [48-02, 48-03, 48-04, 48-05, 49]
tech-stack:
  added: []
  patterns:
    - "Gravity-frame scripted force: accelerations built from GravityFrame (live world.gravity() every step) and converted with ForceScale = m * particle iterations, so F adds exactly a * dt per fixed step"
    - "Conservative pull: the lift and the sideways pull are the gradient of one bounded polynomial potential, capped at CAP * G * S, plus velocity drag, so a still magnet cannot pump energy into the liquid"
    - "Transcendental-free field math (+ - * /, abs, round, signum, length) so native and wasm compute identical bits"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/particle_force.rs
    - crates/liquidfun-wasm/src/scene/particle_force/tests.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/tests.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/test_support.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/spike_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs
key-decisions:
  - "Candidate (a), the polynomial potential pull, passes every written D-05 rule in the real scene; candidate (b) is not built"
  - "CAP 1.5 x G x S: the lowest measured cap (1.5, 2.0, 2.5) whose step-180 peak count at the default strength is at least 5 (it is exactly 5)"
  - "WALL_TOP_Y 1.8 m (FRAME_MAX_Y 2.4): 1.6 m walls, from the first measured worst top 1.21 m + 0.3, let the in-puddle column reach 1.42 m and failed P4, so the rule was reapplied to give 1.8 m, where the worst top is 1.21 m again"
  - "DEFAULT_STRENGTH_PERCENT stays 65 (5 peaks at step 180 at cap 1.5; 70 would give 6); no web constant changes"
  - "Drag (3 per second inside the envelope) is velocity damping independent of the gravity magnitude; every pulling term carries G (D-14)"
patterns-established:
  - "particle_force is the shared bridge for scripted per-particle or per-group accelerations; Phase 49 calls apply_group_acceleration"
requirements-completed: [FERRO-01, FERRO-03]
duration: 26min
completed: 2026-10-15
---

# Phase 48 Plan 01: Fake Ferrofluid Scene Core and D-05 Spike Summary

**A shared gravity-aware particle-force helper, calibrated to within 1e-3 at 2 and 3 particle iterations, now drives a 990-particle graphite-violet TENSILE | VISCOUS puddle. A capped polynomial potential pull lifts it into 5 separate spikes 12 steps (0.2 s) after load, and the scene passes every written D-05 stability rule at cap 1.5 with 1.8 m walls.**

## Performance

- **Duration:** 26 min
- **Started:** 2026-10-15T05:09:52Z
- **Completed:** 2026-10-15T05:35:40Z
- **Tasks:** 3 (Tasks 1 and 2 share one commit, as planned)
- **Files modified:** 11 (8 created, 3 modified)

## Accomplishments

- `particle_force.rs`: `GravityFrame::from_gravity` (None for zero, tiny or non-finite gravity), `ForceScale::for_system` reading radius and density from the live definition, `apply_accelerations` with reusable scratch lanes (zero results skipped), and `apply_group_acceleration` (one range call times the member count). Eight debug tests cover the frame, the scale, calibration at 2 and 3 iterations (measured relative error at most 1e-3), the group path and the zero-acceleration bit identity. No existing scene calls it.
- Fake Ferrofluid scene core: one static dish (floor slab plus 0.3 m walls overlapping it), one TENSILE | VISCOUS group (radius 0.04, viscous 0.25, `#8A84AD`) of 990 particles, and the magnet on at rest under the floor. `on_advance` reads `world.gravity()` every step and applies `magnet_acceleration - v * drag * R(s)` once per fixed step. Controls, actions and the pointer are inert until Plan 02.
- The release spike measured candidate (a) against the written rules, chose the cap and the wall height, and kept 11 release-only regression guards in `spike_tests.rs`.

## Spike Measurements (candidate (a), native release, gravity 10, 990 particles)

Appearance runs sample every 6 steps. "First >= 4" is the first sample with at least 4 peaks after load; "flat after off" counts steps after switching off at step 120; "re-on >= 4" counts steps after switching back on once flat. D-04 asks for spikes within about 1 s (60 steps).

| Iteration | Cap | Strength % | WALL_TOP_Y | First >= 4 (load) | Peaks @180 | Flat after off | Re-on >= 4 | Verdict |
|---|---|---|---|---|---|---|---|---|
| 0 | 1.5 | 65 | 1.8 | 12 (0.2 s, under 60) | 5 | 30 | 18 (0.3 s, under 60) | lowest cap with >= 5 |
| 0 | 2.0 | 65 | 1.8 | 12 | 7 | 36 | 12 | pass, not lowest |
| 0 | 2.5 | 65 | 1.8 | 12 | 7 | 42 | 12 | pass, not lowest |
| 0 (sensitivity) | 1.5 | 60 / 63 / 68 / 70 | 1.8 | 12 / 12 / 12 / 12 | 5 / 5 / 5 / 6 | 18 / 24 / 36 / 36 | 18 / 12 / 18 / 12 | all pass |
| 0 (sensitivity) | 2.0 | 60 / 63 / 68 / 70 | 1.8 | 12 each | 6 / 7 / 7 / 7 | 36 / 36 / 36 / 42 | 12 each | all pass |
| 1 | 1.5 | 65 | 1.6 | 12 | 5 | 30 | 18 | P4 FAIL (in-puddle top 1.417) |
| 2 (final) | 1.5 | 65 | 1.8 | 12 | 5 | 30 | 18 | PASS |

No load or re-on appearance step exceeded D-04's 60 steps, so there is no D-04 deviation.

60 s holds at strength 100 (3,600 steps). Max speed is measured every step; early and late are the 120..720 and 3,000..3,600 windows. KE is the mean of `sum 0.5|v|^2` over 120..1,860 (h1) and 1,860..3,600 (h2). Out, escaped and isolated are the maximum over samples every 30 steps. Top is the highest particle center over all steps.

| Cap / walls | Magnet | Max speed (all / early / late) m/s | KE h1 / h2 | Out | Escaped | Isolated | Top (m) |
|---|---|---|---|---|---|---|---|
| 1.5 / 1.8 (final) | under the dish (0, -0.35) | 1.924 / 0.604 / 0.603 | 2.496 / 2.499 | 0 | 0 | 0 | 0.885 |
| 1.5 / 1.8 (final) | above the puddle (0, 1.2) | 1.879 / 0.613 / 0.610 | 1.483 / 1.398 | 0 | 0 | 0 | 0.674 |
| 1.5 / 1.8 (final) | off-center (2, -0.35) | 2.008 / 0.609 / 0.604 | 2.295 / 2.121 | 0 | 0 | 0 | 0.890 |
| 1.5 / 1.8 (final) | inside the puddle (0, 0.3) | 1.335 / 0.611 / 0.604 | 1.898 / 1.892 | 0 | 0 | 0 | 1.145 |
| 1.5 / 1.8 (final) | in the puddle at the wall (2.7, 0.3) | 1.333 / 0.603 / 0.608 | 1.682 / 1.626 | 0 | 0 | 0 | **1.212** |
| 1.5 / 1.6 | under / above / off-center | 1.924, 1.879, 2.008 (late at most 0.618) | 2.452/2.383, 1.481/1.382, 2.381/2.164 | 0 | 0 | 0 | 0.875 / 0.674 / 0.901 |
| 1.5 / 1.6 | inside the puddle | 1.335 / 0.612 / 0.603 | 1.896 / 1.833 | 0 | 0 | 0 | **1.417** (P4 fails at 1.6) |
| 1.5 / 1.6 | at the wall | 1.333 / 0.606 / 0.612 | 1.685 / 1.614 | 0 | 0 | 0 | 1.145 |
| 2.0 / 1.8 | under / above / off / in / wall | 2.013, 1.900, 2.004, 1.648, 1.712 | 4.040/4.033, 2.136/2.063, 3.933/3.709, 3.704/3.758, 3.314/3.317 | 0 | 0 | 0 | 0.974 / 0.702 / 1.005 / 1.424 / 1.378 |
| 2.5 / 1.8 | under / above / off / in / wall | 1.957, 2.397, 2.298, 2.093, 2.155 | 5.517/5.464, 2.794/2.605, 5.322/5.071, 4.901/5.122, 4.456/4.579 | 0 | 0 | 0 | 1.084 / 0.754 / 1.097 / 1.537 / 1.540 |

Every hold at every cap met the P3 rules: overall max speed at most 2.40 m/s (rule 4.0), late at most early + 0.03 m/s (rule + 0.25), and KE h2 at most 1.25 * h1 + 0.5. The overall max speed is the load transient; the held speed stays near 0.6 m/s. P5: a 1,200-step session run evicted nothing. Each 60 s hold takes about 1.0-1.2 s native release including the O(n^2) isolation samples, which bounds the step at about 0.3 ms. The full release spike suite runs in about 5.8 s.

### CAP and wall-height decision

- **CAP = 1.5** (`const _: () = assert!(FIELD.cap <= 3.0)` holds). It is the lowest of 1.5, 2.0 and 2.5 whose step-180 count at the default strength is at least 5 (it is exactly 5, the low edge of D-04's 5..=9).
- **WALL_TOP_Y = 1.8 m.** The first cap-1.5 measurement (with the 1.8 m starting walls) gave a worst top of 1.212 m, so the rule gave 1.6 m. At 1.6 m the in-puddle hold reached 1.417 m (the column top is sensitive to small geometry changes), which fails P4. Reapplying the rule to that run gives roundup(1.717) = 1.8 m, where the worst top is 1.212 m again (deterministic), leaving 0.588 m of clearance. 1.7 m was not tried. P4 (`column_tops_stay_three_tenths_below_the_wall_tops`) guards this.
- **DEFAULT_STRENGTH_PERCENT = 65** (unchanged; inside 60..=70).

### Candidate verdict

Candidate (a) passed P1-P5 in tuning iteration 2 of 4 (the iterations changed only the cap and the wall height). Candidate (b) was not built (D-05 keeps the first candidate that passes). No rule was loosened.

## Final Constants for Plans 02 and 03

- `FIELD`: wavelength 0.65, envelope_half_width 2.6, reach 1.6, strength_scale 2.5, **cap 1.5**, drag_per_second 3.0.
- Strength: `DEFAULT_STRENGTH_PERCENT` 65, `MAX_STRENGTH_PERCENT` 100 (`#[cfg(test)]` until Plan 02).
- Liquid: radius 0.04, diameter 0.08, density 1.0, viscous 0.25, `ParticleColor::new(138, 132, 173, 255)`, `MAXIMUM_PARTICLE_COUNT` 3,000, **990 particles** built.
- Dish: inner faces x = +-3.0, outer faces x = +-3.3, floor top y 0.0, floor bottom y -0.25, walls 0.3 m thick, **WALL_TOP_Y 1.8**. Puddle box half width 2.96, half height 0.3, center (0, 0.34).
- Magnet: `MAGNET_REST` (0, -0.35), half length 0.4, half height 0.1 (`#[cfg(test)]` until Plan 02).
- **FRAME_\***: min x -3.6, max x 3.6, min y -0.8, max y 2.4 (= WALL_TOP_Y + 0.6). These match the UI-SPEC nominal `FAKE_FERROFLUID_VIEW_BOUNDS`.
- Portrait wall points: wall tops `(-3.3, 1.8)`, `(-3.0, 1.8)`, `(3.0, 1.8)`, `(3.3, 1.8)`; slab bottom corners `(-3.3, -0.25)`, `(3.3, -0.25)`; resting marker bottom corners `(-0.4, -0.45)`, `(0.4, -0.45)`.

## Decision Coverage

- D-01: (a) lift `E'(h)` along `up`, (b) periodic `b^4` column modulation centered on the magnet, (c) the sideways pull toward the magnet from the envelope slope. All three come from one potential's gradient.
- D-02: smooth bounded pieces, softened core, cap 1.5 x G x S, drag 3 per second times R(s). The holds show no oscillation growth.
- D-03: one TENSILE | VISCOUS group, viscous 0.25 (`const _: () = assert!(VISCOUS_STRENGTH <= 1.5)`).
- D-04: 5 peaks at default strength; spikes form 0.2 s after load and 0.3 s after re-on, and collapse in 0.5 s.
- D-05: the spike passed with recorded measurements, candidate (a).
- D-06: magnet on at rest under the floor center.
- D-08: force only in `on_advance`; the other hooks store nothing yet.
- D-11: strength 0 gives exactly zero pull while drag still applies (unit test on a grid).
- D-12, D-13: the shared helper, the calibration tests and the tilt test (`tilted_gravity_lifts_along_the_tilted_up`, `quarter_turn_of_gravity_turns_the_pull`).
- D-14: the field uses only `GravityFrame` axes and `G`, with no hard-coded -y. Drag is documented as gravity-independent damping.
- D-15: radius 0.04, 990 particles (cap 3,000), geometry within +-3.6 m.
- D-16: `#8A84AD`. The four-mode visual check is Plan 04.

## Task Commits

1. **Task 1: shared particle-force helper** and **Task 2: scene core and registration**, committed together as the plan specifies: `c5462cb40` (feat)
1. **Task 3: release spike, CAP and wall-height choice**: `05ec01f54` (test)

## Decisions Made

See the key-decisions frontmatter and "CAP and wall-height decision" above.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Workspace pedantic clippy findings in new test code**
- **Found during:** Task 2 pre-commit
- **Issue:** `clippy::pedantic` is enabled workspace-wide. It flagged int/float casts, strict float comparisons, an approximate `FRAC_1_SQRT_2` literal and constant-valued assertions in the new tests.
- **Fix:** I used `f32::from` on `u16`/`i16` ranges, `to_bits` comparisons and `std::f32::consts::FRAC_1_SQRT_2`. The test-only bin index uses one `#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = ...)]` (Sparky precedent). The `FRAME_MAX_Y >= WALL_TOP_Y + 0.6` headroom check became compile-time `const _: () = assert!(..)` items in geometry.rs instead of a runtime test.
- **Files modified:** particle_force/tests.rs, fake_ferrofluid/{tests,test_support,geometry}.rs
- **Commit:** c5462cb40

**2. [Rule 1 - Bug] Curl test stencil crossed into the capped region after the cap moved to 1.5**
- **Found during:** Task 3
- **Issue:** At cap 1.5 the 0.005 m finite-difference stencil at (s 0.65, h 1.0) reached the cap, so the test's own "cap inactive" guard failed.
- **Fix:** I narrowed the stencil step to 0.001 m. The 1e-2 curl tolerance and the sample points are unchanged.
- **Files modified:** fake_ferrofluid/tests.rs
- **Commit:** 05ec01f54

### Notes

- TDD: for Tasks 1 and 2 I wrote the tests and the implementation before the first run, so no failing (RED) run was recorded. Following the plan, the tests were committed with the implementation.
- During the spike, a temporary hooks `field` copy and an ignored `temp_measure` probe with `eprintln!` let one release binary measure all three caps. I removed both before the Task 3 commit (the grep for `eprintln!` and `println!` is empty).
- Two debug test names describe this plan's inert seam (`controls_and_actions_are_rejected_until_the_magnet_seam_lands`, `pointer_events_change_nothing_yet`). Plan 02 replaces them along with `collect_segments_returns_only_the_dish_loops`.
- Out of scope: the release build of `crates/liquidfun` prints the pre-existing `unreachable expression` warning at `particle/solver/boundary/support.rs:98`. It is already listed under "Left over from v1.4" in REQUIREMENTS.md. This plan does not touch the engine.

## Issues Encountered

None beyond the wall-height iteration above.

## Known Stubs

- `FakeFerrofluidHooks::apply_control`, `apply_action` (`Err(UnknownControl)`) and `apply_pointer` (`Ok(())`, no effect) are intentional placeholders. Plan 02 adds the `magnet-strength` control, the `magnet` action, pointer drag and the marker. These stubs do not block this plan's goal.

## Verification

- `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid`: 38 passed, 0 failed, 0 ignored (11 spike tests).
- `cargo test -p liquidfun-wasm --all-features --lib fake_ferrofluid`: 27 passed, 11 ignored (the spike tests are release-only).
- `cargo test -p liquidfun-wasm --all-features --lib particle_force`: 8 helper tests pass.
- `cargo fmt --all --check`, both clippy commands with `-D warnings`, `cargo build --all-targets --all-features` and `cargo test -p liquidfun-wasm --all-features` (721 passed, 63 ignored) all pass. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `git diff --quiet HEAD~2 -- crates/liquidfun web docs/benchmarks/scene-fingerprints.json` passes: no engine, web or baseline change.

## Next Phase Readiness

- Plan 02 can add the pointer, action and control seam, the marker and the D-23 release tests on top of `FakeFerrofluidHooks`. Remove `#[cfg(test)]` from `MAX_STRENGTH_PERCENT`, `MAGNET_HALF_*` and `FRAME_*` when production code uses them.
- Plan 03 mirrors `FRAME_*` (-3.6, -0.8, 3.6, 2.4) in `FAKE_FERROFLUID_VIEW_BOUNDS` and uses the eight wall points above. The strength default (65) and max (100) match the UI-SPEC.

## Self-Check: PASSED
