---
phase: 40-timed-valve-and-teaching-levels
plan: "06"
subsystem: wasm-puzzle-levels
tags: [rust, wasm, puzzle, levels, viscous, elastic, jelly-pad, fixtures, nudge-sweep, portrait-frames, LEVEL-01, LEVEL-02, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T06:23:33.280Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-03 LiquidMaterial, SystemTuning, probe and levels_tests harness; 40-05 PUZZLE_CAPTIONS"
provides:
  - "LevelId::ThickSyrup (thick_syrup::THICK_SYRUP) and LevelId::JellyPad (jelly_pad::JELLY_PAD), registered in LEVEL_IDS and PUZZLE_IDS"
  - "thick_syrup::{REFERENCE_PLANK_POSES, SHALLOW_ANGLE}; jelly_pad::{REFERENCE_PLANK_POSE, JELLY_PAD_WITHOUT_PAD, pad_centroid} (test-only)"
  - "Four recorded fixtures: thick-syrup and jelly-pad solution and empty"
  - "levels_tests/level_proofs.rs: per-level fixture replays, release nudge sweeps and idea controls"
  - "MIN_TALL_LEVEL_HEIGHT_FRACTION = 0.5 phone-fill rule for portrait-authored levels"
affects:
  - "40-07..40-10 (insert levels in D-09 order; add per-level tests to levels_tests/level_proofs.rs, since levels_tests.rs is at 626 of 628 allowed lines)"
  - "40-10 (the last level keeps the disabled Next form)"
tech-stack:
  added: []
  patterns:
    - "Idea controls are release-only asserted tests over the recorded reference (re-angled planks, or a const spec copy with the idea removed)"
    - "A fixed level-authored pad can be made necessary by putting it on every path to the cup: a roofed raised cup reached only along a channel that starts at a drained cradle"
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/thick_syrup.rs
    - crates/liquidfun-wasm/src/puzzle/jelly_pad.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs
    - crates/liquidfun-wasm/src/puzzle/fixtures/thick-syrup-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/thick-syrup-empty.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/jelly-pad-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/jelly-pad-empty.txt
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests.rs
    - web/src/catalog/puzzles.ts
    - web/src/catalog/puzzle-levels.ts
    - web/src/catalog/puzzle-bounds.ts
    - web/tests/puzzle-bounds.test.ts
    - web/tests/playable-target.test.ts
    - web/tests/puzzle-replay.test.ts
    - web/e2e/puzzle.spec.ts
key-decisions:
  - "thick-syrup uses viscous strength 1.5, the top of the D-29 band: at 1.25 the shallow -15 degree control came too close to the target"
  - "thick-syrup reference planks are -30 and -45 degrees; the control re-angles both to -15 degrees, where water wins by step 207 but syrup peaks at 577 of target 610"
  - "jelly-pad keeps a fixed, level-authored pad (no D-11 tool change). An open divider design let streams arc into the cup with or without the pad, so the cup is raised and roofed. The only way in is a channel that starts at a slotted cradle the pad fills"
  - "The jelly-pad caption says what really happens (water slides across the pad), not 'bounce': the probes show no rebound"
  - "Per-level proof tests moved to levels_tests/level_proofs.rs to keep levels_tests.rs under the 628-line limit"
metrics:
  duration: 57min
  completed: 2026-10-13
  tasks: 3
  files: 16
---

# Phase 40 Plan 06: Thick Syrup and Jelly Pad Summary

Two teaching levels are now registered after first-pour, in D-09 order. In `thick-syrup`, viscous syrup reaches the cup in time only down steep planks. In `jelly-pad`, an uncounted elastic pad is the only floor across a drain on the way to a roofed cup. Each level has recorded three-star and empty fixtures. Both replay identically in debug, release and wasm, pass their release nudge sweeps, and have an asserted test proving their idea is necessary. first-pour's Next now links to `#/puzzle/thick-syrup`.

## Task 1: thick-syrup draft and tuning (commit d195f0991)

### Geometry and numbers

- Box: x -4..4 m, y -0.5..10 m. Floor `quad(-4, -0.5, 4, 0)`, side walls up to y 10.
- Reservoir: floor `quad(-3.8, 7.0, -2.0, 7.2)`, right wall `quad(-1.4, 7.0, -1.2, 9.8)`. The 0.6 m outlet runs from x -2.0 to -1.4.
- Syrup: block (-3.76, 7.24)–(-1.44, 9.6) at spacing 0.06, `LiquidMaterial::Viscous`, counted. That is 1,521 particles at step 0.
- Cup: left wall `quad(1.6, 0, 1.8, 1.7)`, and the right side wall is the backstop. `cup_interior` is (1.8, 0)–(3.8, 1.7).
- Build area: grid (-36, 36, 32, 136), which is x -1.8..1.6 m and y 1.8..6.8 m.
- Viscous strength: `maybe_viscous_strength: Some(1.5)`.
- Thresholds: target 610 (65%), full 944 (3.4 m² / 0.0036), Brim 705.
- Tools: budget 2 planks, par 2, step limit 720.
- Reference: plank (-18, 100, 22) and plank (18, 60, 21). These are centres (-0.9, 5.0) m at -30° and (0.9, 3.0) m at -45°.

### Tuning notes (probe output, authoring only)

- Flow on a slope is limited by the outlet, and the shallow path mostly delays the pour. So the target sits between what the shallow run reaches by step 690 and what the steep run reaches by step 570.
- A sweep of 60 layouts over outlets of 0.5, 0.6 and 0.7 m and strengths of 1.25 and 1.5 found the best headroom at a 0.6 m outlet, strength 1.5, and planks at -30° then -45°. The sweep log is `target/phase40/probes/thick-syrup-sweep.log`.
- Water control on the same -15° planks: water wins at step 207 with 1,184 particles. Syrup peaks at 577 against a target of 610. So "needs a steep slide" is honest.

## Task 2: jelly-pad draft and tuning (commit 7fcbaf24a)

### Geometry and numbers

- Box: x -4..4 m, y -0.5..11 m.
- Reservoir: floor `quad(-3.8, 8.0, -2.4, 8.2)`, right wall `quad(-2.0, 8.0, -1.8, 10.8)`. The 0.4 m outlet runs from x -2.4 to -2.0.
- Water: block (-3.76, 8.24)–(-2.04, 10.6), 1,131 particles.
- Cradle: three base slabs at y 3.2–3.4 leave two 0.16 m drain slots, at x -0.7..-0.54 and 0.3..0.46. The chimney's left wall is `quad(-1.4, 3.4, -1.2, 4.6)`. The channel block `quad(1.2, 3.2, 2.0, 3.8)` sits on the right.
- Roof: tilted literal quad (0, 4.4), (3.8, 4.8), (3.8, 5.0), (0, 4.6). The chimney baffle `quad(0, 4.1, 0.2, 6.4)` leaves a 0.28 m gap above the pad.
- Cup: raised, with floor `quad(1.8, 1.8, 3.8, 2.0)` and left wall `quad(1.8, 2.0, 2.0, 3.2)`. `cup_interior` is (2.0, 2.0)–(3.8, 3.4).
- Pad: block (-1.18, 3.42)–(1.18, 3.82), `LiquidMaterial::Elastic` (which adds the SOLID group flag), `counted: false`, color (236, 112, 178). That is 234 particles, so 1,365 particles at step 0.
- Elastic strength: `maybe_elastic_strength: Some(0.5)`.
- Thresholds: target 420 (60%), full 700 (2.52 m² / 0.0036), Brim 630 (90%).
- Tools: budget 2 planks, par 1, step limit 720.
- `no_place`: grid (-28, 64, 0, 92), the cradle and the foot of the chimney.
- Reference: one plank at (-40, 120, 22), which is centre (-2.0, 6.0) m at -30°.

### How the design evolved (probe output, authoring only)

1. **Open divider with the pad next to it.** Streams arced straight over the divider, so runs with and without the pad were identical. Rejected.
1. **Roof with a side window.** Plank streams threaded the window directly. In a 585-placement grid, several placements won without the pad (best peak 1,012). Rejected.
1. **Deep pit with a 650-particle pad.** The pad sagged by 0.34 m, and water squeezed down its sides. Rejected.
1. **Final design.** A shallow drained cradle, a chimney inlet and a baffle. Across a 544-placement grid (many invalid), the best run without the pad peaked at 194 against a target of 420.
   - The top 15 placements with the pad all win. Their peaks are 830–910 and their pad drift is 0.13–0.18 m.
   - The pad never counts, and the no-pad copy loses.

The sweep log is `target/phase40/probes/jelly-pad-sweep.log`.

### D-11 decision

The pad stays fixed and level-authored, so the plan did not need D-11's placeable-tool fallback.

## Task 3: Registration, fixtures, frames and proofs (commit 2aa5fa03a)

### Registry, captions and Next

- Rust: `LEVEL_IDS = [FirstPour, ThickSyrup, JellyPad]`. TS: `PUZZLE_IDS = ["first-pour", "thick-syrup", "jelly-pad"]`. The draft `allow(dead_code)` attributes are gone.
- Titles: "Thick Syrup" and "Jelly Pad".
- Captions:
  - thick-syrup: "Syrup is thick and slow. Give it a steep slide."
  - jelly-pad: "Jelly never counts, but water slides across it. Drop the water onto the pad."
- `web/e2e/puzzle.spec.ts` now expects the first-pour result card's "Next level" link to point at `#/puzzle/thick-syrup`. The disabled last-level form is left to Plan 10.

### Frames

| Level | Landscape | Portrait | Fill at 390×844 (width / height) |
| --- | --- | --- | --- |
| thick-syrup | -4.2, -0.7, 4.2, 10.2 | -4.1, -0.6, 4.1, 10.1 | 1.0 / about 0.60 |
| jelly-pad | -4.2, -0.7, 4.2, 11.2 | -4.1, -0.6, 4.1, 11.1 | 1.0 / about 0.66 |

`LEVEL_GEOMETRY` values come from the Rust specs. They were cross-checked against the generated wasm `levelInfo()` and frame extents, recorded in `target/phase40/level-probe-thick-syrup.txt` and `target/phase40/level-probe-jelly-pad.txt` (script: `target/phase40/probe-level.ts`).

### Proofs (in `levels_tests/level_proofs.rs`)

- `thick_syrup_needs_a_steep_path`: the recorded reference with both angles set to step 23 (-15°) is valid and ends `Lost`.
- `jelly_pad_needs_the_pad`: `run_spec_outcome(&JELLY_PAD_WITHOUT_PAD, reference)` ends `Lost`. On the real reference session, the pad centroid stays within 0.3 m of its step-0 position at every step up to the Brim decision step (447). The probe measured a 0.138 m maximum over the full 720 steps.
- Both controls and both nudge sweeps are release-only and pass in the release `puzzle::` lane.

### RECORD lines

Release:

```
RECORD thick-syrup-solution.txt outcome won outcome_step 440 final_count 660 peak_held 705 brim_step 501 profile release
RECORD thick-syrup-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile release
RECORD jelly-pad-solution.txt outcome won outcome_step 351 final_count 500 peak_held 630 brim_step 447 profile release
RECORD jelly-pad-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile release
```

Debug: all four lines are identical to release (`profile debug`). The spread is 0, so the shared tolerance of 5 applies and no `debug_outcome_step_tolerance` was added. first-pour still records won 201 / 689 / 811 / 262 and lost 720 / 0 / 0 / none.

### Nudge tallies (release)

- `NUDGES thick-syrup-solution.txt wins 15 valid 15 skipped 1 generated 16`
- `NUDGES jelly-pad-solution.txt wins 8 valid 8 skipped 0 generated 8`
- `NUDGES first-pour-solution.txt wins 8 valid 8 skipped 0 generated 8`

### Wasm contract

The contract is exact. `just puzzle-replay-wasm` passes all 6 fixtures with values identical to release-native. This resolves research Pitfall 12 for the viscous and elastic solvers; no fallback was needed.

### Budgets and step cost (D-23, informational)

```
STEPCOST first-pour particles 1989 ms_per_step 0.58
STEPCOST thick-syrup particles 1521 ms_per_step 0.44
STEPCOST jelly-pad particles 1365 ms_per_step 0.43
```

Both new levels are well under the 3,000-particle step-0 budget.

## Verification

- `cargo fmt --all --check` passed. `cargo clippy --all-targets --all-features -- -D warnings` and `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` both exit 0. `cargo build --all-targets --all-features` passed.
- `cargo test -p liquidfun-wasm` (debug): 554 passed, 12 ignored.
- `cargo test -p liquidfun-wasm --release --lib puzzle::`: 238 passed, 6 ignored.
- `just puzzle-replay-wasm`: 6/6 pass.
- Web:
  - `bunx tsc --noEmit` is clean.
  - `bun run test:unit` passes 97 files and 1,375 tests.
  - `bun run build:app` succeeded, and `CI=true bun run test:puzzle` passes 38/38.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `git diff --quiet $(cat target/phase40/phase-base.txt) -- first-pour fixtures crates/liquidfun` exits 0. No scene, builder or session code changed, so the 25 scene fingerprints are unaffected (the Plan 11 gate re-measures them).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `levels_tests.rs` exceeded the file-length limit**
- **Found during:** Task 3
- **Issue:** The new tests took `levels_tests.rs` to 735 lines. The limit is 628, so `bright-builds-check` failed.
- **Fix:** Moved the per-level proofs into the child module `levels_tests/level_proofs.rs`. These are the fixture replays, nudge sweeps (including first-pour's) and idea controls. It is still the same lib test binary, and the release `puzzle::` count is unchanged at 238.
- **Files modified:** crates/liquidfun-wasm/src/puzzle/levels_tests.rs, crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs
- **Commit:** 2aa5fa03a

**2. [Rule 1 - Stale test] Two unit tests assumed a one-level registry**
- **Found during:** Task 3
- **Issue:**
  - `playable-target.test.ts` "has no next level after the only level" expected first-pour to have no Next.
  - `puzzle-replay.test.ts` "checked-in fixtures parse" expected exactly `["won", "lost"]`.
- **Fix:**
  - first-pour's Next is now `"thick-syrup"`, and a new test checks that the last registry id has no Next.
  - The fixture-outcome expectation is now derived from the file suffix, so solution files expect a win and empty files a loss.
- **Files modified:** web/tests/playable-target.test.ts, web/tests/puzzle-replay.test.ts
- **Commit:** 2aa5fa03a

**3. [Design within plan discretion] The jelly-pad layout departs from the starting concept**
- The plan's concept was a divider with a cradle that rises above it. In that layout the pad was irrelevant, as described under Task 2. The final layout keeps every D-11 element: a fixed, level-authored, uncounted elastic SOLID pad seated in static geometry, which the reference stream must use. Its caption describes the measured behavior; the plan's default "bounce" caption would have been untrue.

**Process notes:**
- Tuning used a temporary probe that read geometry from a text file, so iterations did not need recompiles. It was never committed, and its draft files are under `target/phase40/probes/drafts/`. The committed `#[ignore]` probes `probe_thick_syrup` and `probe_jelly_pad` reproduce the final numbers, logged in `target/phase40/probes/{thick-syrup,jelly-pad}.log`.
- LEVEL-01, LEVEL-02 and LEVEL-04 stay Pending in REQUIREMENTS.md, as instructed. Only the phase completion marks them.

## Known Stubs

None.

## Threat Flags

None.
- T-40-21: the draft variants were unreachable from wasm until this registration, and `parse_level_id` plus the TS/Rust mirror test guard the allowlist.
- T-40-22: step-0 counts are 1,521 and 1,365, both under 3,000. Viscous strength 1.5 is inside the stable band, and full runs are exercised in every lane.
- T-40-23: fixtures were recorded only from the release replay harness, and the wasm replay is exact.

## Self-Check: PASSED
