---
phase: 40-timed-valve-and-teaching-levels
plan: "09"
subsystem: wasm-puzzle-levels
tags: [rust, wasm, puzzle, levels, buoyancy, float, fixtures, nudge-sweep, portrait-frames, D-09, D-12, D-28, LEVEL-01, LEVEL-02, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T08:39:55.187Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-03 FloatBody, collect_circles, probe/levels_tests harness; 40-06 level_proofs.rs; 40-07 fixture_table.rs and registry order; 40-08 sand-chute dropped"
provides:
  - "LevelId::FloatUp (float_up::FLOAT_UP), registered in LEVEL_IDS and PUZZLE_IDS between jelly-pad and timed-gate"
  - "float_up::{FLOAT, REFERENCE_PLANK_POSE, FLOAT_UP_DENSE} (FLOAT_UP_DENSE and the pose are test-only)"
  - "float-up-solution.txt and float-up-empty.txt fixtures"
  - "level_proofs: float_up_needs_buoyancy (every lane), float_up_nudges_mostly_win (release) and two fixture replays"
  - "LevelGeometry.maybeFloatTravel in puzzle-bounds.test.ts"
affects:
  - "40-10 grand-finale goes after timed-gate; registry is 5 levels now, 6 with the finale (within 5-8)"
  - "40-11 gates: 12 fixtures in just puzzle-replay-wasm; float-up step cost 0.83 ms/step"
tech-stack:
  added: []
  patterns:
    - "Float as a drain plug: a light circle floats up a well and seats in the drain hole, so a U-tube pipe can fill to a higher window"
    - "Compressed LiquidFun water is denser at depth: a density-2.0 control only sinks in a low, gently fed column"
    - "Trap baffle in front of an overflow window, and a thick roof where it meets a wall, keep jets and pooled water out of a closed cup"
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/float_up.rs
    - crates/liquidfun-wasm/src/puzzle/fixtures/float-up-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/float-up-empty.txt
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/fixture_table.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs
    - web/src/catalog/puzzles.ts
    - web/src/catalog/puzzle-levels.ts
    - web/src/catalog/puzzle-bounds.ts
    - web/tests/puzzle-bounds.test.ts
key-decisions:
  - "float-up ships (no D-09 drop). Mechanism (a), the plug, passed. A density-0.4 ball rises 0.44 m and seats in its well's drain hole, so the pipe fills to the cup window"
  - "Mechanism (b), the lifted deflector, was not drafted. With one stream, a dense ball's well fills and overflows just like a light one's, so buoyancy could not be necessary"
  - "Two leaks, not buoyancy, made the dense control win in early drafts: the funnel stream jetted through the window, and pooled roof water pushed through a thin roof's seam with the side wall. A trap baffle and a 0.64 m thick roof edge fixed both"
  - "A wide 1.0 m well, 0.15 m side clearance, a low weir (0.5 m below the window) and a 0.4 m reservoir outlet keep the column low enough that a density-2.0 ball stays on the floor"
  - "Caption: 'The ball floats. Fill its well and it rises to plug the drain.'"
metrics:
  duration: 49min
  completed: 2026-10-13
  tasks: 2
  files: 10
---

# Phase 40 Plan 09: Float Up Summary

**Decision: `float-up` ships.** In `float-up`, a level-authored circle float (r 0.35 m, density 0.4) floats up its well and plugs the well's drain. Water can then rise in a pipe to a window and spill into the cup. The asserted control `float_up_needs_buoyancy` runs the recorded reference with a density-2.0 ball, which stays on the well floor, and that run loses.

The level is registered between jelly-pad and timed-gate. Its fixtures replay identically in debug, release and wasm. All 7 valid nudges win. The browser draws the ball and replays the reference to the fixture's outcome.

## Task 1: Draft and tune (commit daafe2360)

### Mechanisms tried (probe output, authoring only, D-18)

The log is `target/phase40/probes/float-up.log`. The drafts, their outputs and the data-driven probe source are in `target/phase40/probes/fu-drafts/`: 91 draft runs, with the probe saved as `float_draft_probe.rs.keep`. The probe was never committed.

Mechanism (a) is the plug: the ball closes the well's waste route, so the level rises to the cup route.

| Drafts | Design | Result |
| --- | --- | --- |
| a1 | Side tube with a top neck; open well lip | The plank jet flew straight into the cup (won, 840). The float never moved. |
| a2-a3 | Cup roof added | The float sealed, but the cup got 0: the well's dead volume was about 1,000 particles. |
| a4-a6 | Skirt, capped slot, downpipe | Jets still entered the slot. Dense won 1,209, the same as light. |
| a7-a8 | Trap baffle; compact well | Light 140 vs dense 137, then 348 vs 228: the neck's waste capacity was too low. |
| a9 | r 0.35 ball, 0.05 m clearance | The ball never rose, because water could not get under it. |
| b1-b3 | U-tube: funnel, pipe, tube | Flow dragged the dense ball into the neck: dense won 390-486, light 459. |
| c1-c7 | Wide 0.9 m chamber, inlet above the ball | The dense ball stayed down (4 vs light 205). The plank's catch of the falling stream wasted 400-1,600 particles. |
| d1-d3 | Static gutter plus a bridging plank | The natural jet reached the funnel (empty run 226) until a deflector was added. The vertical drop still wasted about 800. |
| e1-e7 | Compact chamber, shifted funnel, inlet baffle | Delivery was fixed (light h600 up to 1,008). Dense 2.0 and 3.0 balls also won: the pressurized pipe lifted them. |
| f1-f7 | r 0.35 ball, 0.5 m drain hole, wide slots | The dense ball stayed down at most placements, but dense still won at three placements. |
| g1-g4 | Diagnosis and fix | The dense wins came from two leaks, the window jet-through and the roof seam. A trap baffle and a thick roof edge brought dense to 0 at all 10 placements. |
| h1-h3 | Tuning head, outlet and window | h3 is the final design. |

Static pool check (ball r 0.25 resting on the floor, final y after 300 steps; contact is y 0.25):

| Water depth | Density 1.5 | Density 2.0 | Density 3.0 |
| --- | --- | --- | --- |
| 1.0 m | 0.42 | 0.26 | 0.25 |
| 2.5 m | 1.14 | 0.35 | 0.25 |
| 3.0 m | 1.36 | 0.46 (hovers) | 0.25 |

LiquidFun's compressed water is denser at depth, so a density-2.0 control sinks reliably only in a low, gently fed column. The final well keeps that column low.

Mechanism (b), the lifted deflector, was not drafted. It cannot make buoyancy necessary with one stream, as recorded under key decisions. Mechanism (a) passed first.

### Final geometry and numbers

- Box: x -4..4 m, y -0.5..11 m.
- Reservoir floor at y 7.0, with a 0.4 m outlet at x -2.4..-2.0. Water block (-3.76, 7.24)–(-1.24, 10.66): 2,394 particles at step 0, under the 3,000 budget.
- Gutter: a 15° slab from x -2.9 down to -1.6, with a backstop. Without a plank, the stream drops off its end into the waste pit.
- Well: x 0.4..1.4, floor at y 2.0.
  - The ceiling rises to a 0.5 m drain hole at y 3.05.
  - The left wall's top, at y 3.25, is the weir.
  - The float starts at (0.9, 2.36) and seats at (0.9, 2.8), a rise of 0.44 m.
- Pipe: x 1.6..1.9. Two slots into the well, at y 2.2..2.45 and 2.6..2.8. A trap baffle down to y 3.55, and a 0.7 m window at y 3.75..4.45, 0.5 m above the weir.
- Funnel above the pipe. The cup roof drains into the funnel and is 0.64 m thick at the side wall.
- Cup: `cup_interior` is (2.4, 0)–(3.8, 1.0).
- Thresholds: full 389, target 233 (60%), Brim 272.
- Tools: 2 planks, par 1, step limit 720. Build area x -3.0..1.2, y 3.6..6.2. `no_place` covers the well, x 0.4..1.4, y 2.0..3.25.
- Reference: `placement plank -4 104 23`, which is centre (-0.2, 5.2) m at -15°.

Final Rust probe (`probe_float_up`, release):

```
PROBE float-up empty outcome 2 step 720 peak 0 brim none particles 2394
PROBE float-up reference outcome 1 step 550 peak 617 brim 563 particles 2394
PROBE float-up dense-float outcome 2 step 720 peak 160 brim none particles 2394
PROBE float-up nudges wins 7 valid 7 skipped 1 generated 8
```

The reference curve reads 513 at step 600. The margin-raised Brim is 292 and the margin-raised target is 253.

## Task 2: Registration, fixtures, proofs and frames (commit 2532a461b)

- Rust: `LEVEL_IDS = [FirstPour, ThickSyrup, JellyPad, FloatUp, TimedGate]`. TS: `PUZZLE_IDS` has the same order. No dead-code allowance exists in `float_up.rs` or `puzzle.rs`.
- Title "Float Up". Caption: "The ball floats. Fill its well and it rises to plug the drain."
- Frames: landscape (-4.2, -0.7, 4.2, 11.2) and portrait (-4.1, -0.6, 4.1, 11.1), the same as jelly-pad. The portrait frame fills about 66% of a 390×844 phone's height.
- `LEVEL_GEOMETRY` adds `maybeFloatTravel` (x 0.55..1.25, y 2.01..3.15), which covers the float from rest to its seat. Every geometry box is contained in both frames.
- The wasm cross-check is in `target/phase40/level-probe-float-up.txt`:
  - `cupInterior` [2.4, 0, 3.8, 1] and `buildArea` [-60, 72, 24, 124];
  - segments x -4..4, y -0.5..11;
  - `rigidCircles` [0.9, 2.36, 0.35];
  - 2,394 particles.

### RECORD lines

```
RECORD float-up-solution.txt outcome won outcome_step 550 final_count 317 peak_held 272 brim_step 563 profile release
RECORD float-up-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile release
```

Debug records identical values (`profile debug`), so the spread is 0 and no `debug_outcome_step_tolerance` was added.

### Proofs

- `float_up_needs_buoyancy` runs in every lane:
  - With the recorded reference, the density-2.0 copy (`FLOAT_UP_DENSE`) ends `Lost`, and its ball rises less than 0.1 m.
  - The real ball rises at least 0.4 m by the recorded win step, 550.
- Nudges: `NUDGES float-up-solution.txt wins 7 valid 7 skipped 1 generated 8`. The skipped variant is x + 2 cells. Its lower end comes within one particle diameter of the funnel's left lip (the pipe's left wall top), so the validator rejects it as `OverlapsWall`.

### Wasm contract

The contract is exact. `just puzzle-replay-wasm` passes all 12 fixtures under `WASM_REPLAY_CONTRACT = "exact"`, with the float-up values identical to release-native. The circle float kept runtime trig out of contact geometry (T-40-29), and no fallback was needed.

### Browser check

A temporary Playwright spec was run and then removed; it is kept as `target/phase40/tmp-float-up-screens.spec.ts.keep`.

- At 1280×800 it built the reference through the tray, canvas and arrow keys, played, and reached outcome step 550 with peak held 272, matching the fixture.
- Screenshots in `target/phase40/screens/`:
  - `float-up-building.png` shows the ball drawn in its well while building.
  - `float-up-building-390x844.png` is the phone frame.
  - `float-up-running.png` shows the ball seated in the drain while the pipe spills into the cup.
  - `float-up-won.png` is the result card.
- The puzzle draw path already draws `rigidCircles`, so no draw-path fix or new Vitest was needed.

### Step cost (D-23, informational)

```
STEPCOST float-up particles 2394 ms_per_step 0.83
```

The other levels are unchanged: first-pour 0.55, thick-syrup 0.43, jelly-pad 0.42 and timed-gate 0.24 ms/step.

## Verification

- Rust checks:
  - `cargo fmt --all --check` passes.
  - Both clippy commands pass with `-D warnings`.
  - `cargo build --all-targets --all-features` passes.
- Rust test lanes:
  - `cargo test -p liquidfun-wasm` (debug): 562 passed, 18 ignored.
  - `cargo test -p liquidfun-wasm --lib puzzle::` (debug): 241 passed.
  - `cargo test -p liquidfun-wasm --release --lib puzzle::`: 250 passed, 8 ignored. This includes all five nudge sweeps.
- `just puzzle-replay-wasm`: 12/12 pass, under the exact contract.
- Web:
  - `bunx tsc --noEmit` is clean.
  - `bun run test:unit` passes 97 files and 1,375 tests.
  - `bun run build:app` succeeded, and `CI=true bun run test:puzzle` passes 39/39.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- Unchanged files:
  - `crates/liquidfun` and the first-pour fixtures have no diff from the phase base.
  - The thick-syrup, jelly-pad and timed-gate fixtures are unchanged.
  - No scene, builder or session code changed, so the 25 scene fingerprints are unaffected. The Plan 11 gate re-measures them.

## Deviations from Plan

### Design decisions within plan discretion

**1. The plug is a drain at the top of the well, fed as a U-tube.** The plan's sketch had a side spill hole and an overflow lip. In the drafts, an open lip let plank jets reach the cup directly, so the cup is closed and fed only through a pipe window. A trap baffle and a thick roof edge stop the two leaks that let the dense control win.

**2. Mechanism (b) was not drafted.** Mechanism (a) passed, and the log records why (b) cannot make buoyancy necessary.

**3. Larger water and float values.**
- Water is 2,394 particles, just under the plan's 2,400. The well and pipe hold about 1,300 compressed particles.
- The float radius is 0.35 m, the top of the plan's 0.25-0.35 band. A larger ball allows a 0.5 m drain hole, which gives the dense case enough waste capacity.

### Path adjustments

**4. Test file.** The plan's acceptance greps `levels_tests.rs` for `fn float_up_needs_buoyancy`. As the orchestrator directed, per-level tests live in `levels_tests/level_proofs.rs`, and the test is there. The fixtures table is `levels_tests/fixture_table.rs`. Plan 06 moved these to keep `levels_tests.rs` under the 628-line limit.

**5. Bounds test extension.** `LevelGeometry` gained the optional `maybeFloatTravel`, so the "include the float's full travel" rule is checked rather than assumed.

### Auto-fixed Issues

None.

**Process notes:** LEVEL-01, LEVEL-02 and LEVEL-04 stay Pending in REQUIREMENTS.md, as instructed.

## Known Stubs

None.

## Threat Flags

None. The threat register is covered:

- T-40-29: the float is a circle with no rotation-dependent contacts, and the wasm replay is exact.
- T-40-30: the cup and `cup_interior` are static, and the float never touches the count region.

## Self-Check: PASSED
