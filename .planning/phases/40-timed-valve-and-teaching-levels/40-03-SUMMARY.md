---
phase: 40-timed-valve-and-teaching-levels
plan: "03"
subsystem: wasm-puzzle-core
tags: [rust, wasm, puzzle, materials, buoyancy, test-harness, nudge-sweep, LEVEL-01, LEVEL-02, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T05:07:04.757Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-01 stride-4 lanes, ToolKind::Valve, ValveGate, create_puzzle_from_spec, 5/6-token fixture grammar"
provides:
  - LiquidMaterial { Water, Viscous, Powder, Elastic } with flags() (Elastic = ELASTIC + SOLID group)
  - SystemTuning { maybe_viscous_strength, maybe_elastic_strength, maybe_powder_strength } with NONE
  - FloatBody { center, radius, density } and LevelSpec.floats (dynamic circles after geometry, before tools)
  - LevelSpec.system_tuning, LiquidBlock.material
  - hooks::LevelBodies { segments, valves, floats }; collect_circles reports floats in spec order
  - replay.rs shared helpers (pub(in crate::puzzle)) is_record_mode, assert_trace_shape, input_lines, record_fixture_file, replay_fixture_file
  - replay::fixture is pub(in crate::puzzle); fixture::placement_lanes
  - probe.rs probe, probe_spec, print_report, probe_nudges, probe_open_ticks
  - levels_tests.rs PARTICLE_BUDGET, FIXTURES, fixture_text, run_outcome, run_spec_outcome, nudge_variants, NudgeTally, tally_nudges, assert_nudges_mostly_win, open_tick_sweep, timing_window_ok
affects:
  - 40-05..40-10 (each level adds its FIXTURES pair, one-line replay/nudge tests and optional run_spec_outcome controls)
  - 40-07 (timed-gate uses open_tick_sweep + timing_window_ok)
tech-stack:
  added: []
  patterns:
    - water keeps the Phase 37 recipe chain exactly; other materials add particle flags and non-empty group flags
    - None strengths add no ParticleSystemDef builder call
    - harness iterates LEVEL_IDS so a registered level cannot skip coverage, three-star, empty or budget checks
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/materials_tests.rs
    - crates/liquidfun-wasm/src/puzzle/probe.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests.rs
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/spec.rs
    - crates/liquidfun-wasm/src/puzzle/first_pour.rs
    - crates/liquidfun-wasm/src/puzzle/builder.rs
    - crates/liquidfun-wasm/src/puzzle/builder/tests.rs
    - crates/liquidfun-wasm/src/puzzle/hooks.rs
    - crates/liquidfun-wasm/src/puzzle/replay.rs
    - crates/liquidfun-wasm/src/puzzle/replay/tests.rs
    - crates/liquidfun-wasm/src/puzzle/replay/fixture.rs
key-decisions:
  - "PuzzleHooks::new takes a LevelBodies bundle (segments, valves, floats) instead of an 8th parameter, which clippy pedantic too_many_arguments rejects"
  - "run_outcome validates with validate_for_build and builds with SessionCore::create_puzzle (build_puzzle_core minus the id allowlist), so draft levels not yet in LEVEL_IDS can be nudge-checked"
  - "Float bodies use friction 0.2, restitution 0 and the default filter, like the float_or_sink drop bodies, with no wake impulse"
  - "Probe peak is read at the step limit, so it can exceed the fixture peak_held recorded at the decision step (first-pour: 1361 vs 811)"
metrics:
  duration: 26min
  completed: 2026-10-13
  tasks: 2
  files: 12
---

# Phase 40 Plan 03: Level materials, floats and proof harness Summary

`LevelSpec` can now give each liquid block a material (water, viscous, powder, or elastic jelly in a `SOLID` group). It can also set optional particle-system strengths and add dynamic circle floats. One table-driven harness proves every registered level: it checks that both fixtures exist, that the solution earns three stars with margin, that the empty run loses, and that step 0 fits the 3,000-particle budget. A release-only nudge sweep and an open-tick window rule complete the harness. first-pour replays bit-identically.

## Task 1: Materials, tuning and floats (commit d72cba3df)

- `spec.rs`:
  - `LiquidMaterial::flags()` maps Water to (WATER, empty), Viscous to (VISCOUS, empty), Powder to (POWDER, empty) and Elastic to (ELASTIC, SOLID).
  - `SystemTuning::NONE` is the default.
  - `FloatBody` is new.
  - New fields: `LiquidBlock.material`, `LevelSpec.system_tuning` and `LevelSpec.floats`.
- `first_pour.rs`: adds `material: LiquidMaterial::Water`, `system_tuning: SystemTuning::NONE` and `floats: &[]`. Every other test spec uses struct update, so no other literal changed.
- `builder.rs`:
  - Creation order is now geometry, `create_floats`, tools, then liquid.
  - `system_definition` applies only the `Some` strengths.
  - For water, `create_block` keeps the exact Phase 37 chain. Other materials set their own particle flags, and also their group flags when those are non-empty.
  - `validated_trackers` rejects a float whose centre is not finite, or whose radius or density is not finite and positive (T-40-12).
- `hooks.rs`: `collect_circles` returns `(body position, radius)` for each float, mapping errors to `FrameCaptureFailed`.
- `materials_tests.rs` has 7 tests:
  - first-pour stays plain water with no tuning and no floats.
  - Jelly in the cup never counts at step 0: more than 100 pad particles are strictly inside, and goal 0.
  - Jelly still never counts after 60 steps: the goal count equals the counted water in the cup, which is 0.
  - A viscous reservoir at strength 1.25 runs 120 steps.
  - The float is a frame circle `(2.5, 0.3) r 0.25` at step 0.
  - The float rises at least 0.3 m in 180 steps.
  - A zero-radius float is rejected.
- `builder/tests.rs`: a new test checks the `LiquidMaterial::flags` mapping.

## Task 2: Probe and level harness (commit 59b39ba85)

- `replay.rs`: the five helpers moved here as `pub(in crate::puzzle)`, and `fixture` became `pub(in crate::puzzle) mod`. The first-pour `#[test]`s stay in `replay/tests.rs`. `fixture::placement_lanes` flattens placement lines.
- `probe.rs`: `probe`, `probe_spec`, `print_report`, `probe_nudges` and `probe_open_ticks`. They build through `validate_for_build` and never through `parse_level_id`.
- `levels_tests.rs`:
  - `PARTICLE_BUDGET = 3000` and the `FIXTURES` table, which holds first-pour's pair.
  - Harness helpers: `run_outcome` and `run_spec_outcome`, `nudge_variants`, `NudgeTally`, `tally_nudges`, `nudges_mostly_win` (`valid >= 4`, `2*valid >= generated`, `10*wins >= 7*valid`), `assert_nudges_mostly_win`, `open_tick_sweep` and `timing_window_ok`.
  - Tests over every `LEVEL_IDS` entry: coverage, three-star, the Brim rule (`brim*10 >= target*10 + full`, `brim <= full`), empty and `step_zero_particles_fit_the_budget`.
  - Synthetic generator, tally, rule and window tests.
  - Release-only `first_pour_nudges_mostly_win`, and the ignored `level_step_costs`.

### Probe line format

```
PROBE <label> outcome <0|1|2> step <n|none> peak <n> brim <n|none> particles <n> curve c30,c60,...
```

The authoring probe printed this for the first-pour reference, in release (authoring output only, not evidence):

```
PROBE first-pour reference outcome 1 step 201 peak 1361 brim 262 particles 1989 curve 0,0,123,251,427,580,733,843,949,1039,1112,1166,1200,1227,1249,1269,1291,1309,1331,1342,1348,1356,1361,1363
```

### First-pour nudge tally and runtime

- `NUDGES first-pour-solution.txt wins 8 valid 8 skipped 0 generated 8`
- Release `cargo test -p liquidfun-wasm --release --lib puzzle::levels_tests` reported 16 passed and 1 ignored, finishing in 1.02 s.
- `STEPCOST first-pour particles 1989 ms_per_step 0.58` (release-native, informational, D-23).

## Release first-pour RECORD lines (unchanged)

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 peak_held 811 brim_step 262 profile release
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile release
```

## Verification

- `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` all passed, as did `cargo build --all-targets --all-features`.
- `cargo test -p liquidfun-wasm` (debug): 550 passed, 6 ignored.
- `cargo test -p liquidfun-wasm --release --lib puzzle::`: 230 passed, 4 ignored. The first-pour replays are exact.
- `bun scripts/bright-builds-check.ts all` reported 0 findings.
- `git diff --quiet $(cat target/phase40/phase-base.txt) -- crates/liquidfun-wasm/src/puzzle/fixtures crates/liquidfun` exited 0.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `PuzzleHooks::new` would exceed clippy's argument limit**
- **Found during:** Task 1
- **Issue:** Adding `floats` as an 8th parameter triggers `clippy::too_many_arguments` under `-D warnings`.
- **Fix:** Added `hooks::LevelBodies { segments, valves, floats }`, which `new` destructures. Behavior is unchanged.
- **Files modified:** crates/liquidfun-wasm/src/puzzle/hooks.rs, crates/liquidfun-wasm/src/puzzle/builder.rs
- **Commit:** d72cba3df

**2. [Rule 3 - Blocking] Lints on the new types**
- **Issue:** `clippy::struct_field_names` fired on the plan-mandated `maybe_` fields, and the Viscous, Powder and Elastic variants were dead code in the non-test lib.
- **Fix:** Added `#[allow(clippy::struct_field_names, reason = …)]`, following `PreSolveControls`, and `#[cfg_attr(not(test), allow(dead_code))]` on `LiquidMaterial`. The later level plans construct the variants.
- **Commit:** d72cba3df

**3. [Rule 2 - Correctness] `run_outcome` builds without the id allowlist**
- **Issue:** The plan says `run_outcome` should use `crate::build_puzzle_core`. That function calls `parse_level_id`, so `probe_nudges` and `probe_open_ticks` would fail for a draft level that is not yet in `LEVEL_IDS`, even though the plan says level plans rely on probing drafts.
- **Fix:** `run_outcome` uses `validate_for_build` plus `SessionCore::create_puzzle`, which is `build_puzzle_core` without the id lookup. Rejected placements return `None`; any other build error panics. Fixture replays still go through `build_puzzle_core`.
- **Commit:** 59b39ba85

**4. [Rule 3 - Blocking] The acceptance grep did not match the rustfmt-wrapped attribute**
- **Issue:** rustfmt wrapped the long `cfg_attr(debug_assertions, ignore = "...; run with --release")` onto several lines, so `grep "cfg_attr(debug_assertions, ignore"` found nothing.
- **Fix:** The reason is now `release-only nudge sweep (D-16)`, short enough to stay on one line, and the note to run with `--release` moved to the doc comment.
- **Commit:** 168241584

**Other choices:**
- `materials_tests.rs` sorts in-cup particles into pad and water by `ParticleFlags::ELASTIC` instead of by group id. This gives the same split without depending on group creation order.
- To keep the `pub(super)` helpers covered without debug-lane simulations, cheap tests use placements the bridge rejects. A plank outside the build area makes every nudge skipped, and a valve on first-pour makes every tick invalid. The success paths run in the release nudge test and in the ignored probe test.
- TDD RED and GREEN were not committed separately. Each task's tests and implementation went into one commit, because the plan's acceptance is per task.

## Requirements status

LEVEL-01, LEVEL-02 and LEVEL-04 stay Pending in REQUIREMENTS.md. `requirements mark-complete` checked them off, and that change was reverted. This plan only delivers infrastructure for them: materials, the proof harness and the budget test. The teaching levels and the portrait frames come in later Phase 40 plans.

## Known Stubs

None.

## Threat Flags

None. T-40-09 is covered: strengths are `Result`-mapped, and the viscous test runs at 1.25. T-40-10 is covered by `step_zero_particles_fit_the_budget`. T-40-11 is covered: probe output is labeled authoring-only, and the sweep has a non-vacuity floor. T-40-12 is covered by float validation and a test.

## Self-Check: PASSED
