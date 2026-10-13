---
phase: 40-timed-valve-and-teaching-levels
plan: "07"
subsystem: wasm-puzzle-levels
tags: [rust, wasm, puzzle, levels, valve, timing, viscous, fixtures, window-sweep, playwright, TOOL-05, LEVEL-01, LEVEL-02, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T07:30:55.883Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-02/40-04 valve tool, stride-4 lanes and stepper; 40-03 probe/levels_tests harness; 40-06 level_proofs.rs and D-09 registry order"
provides:
  - "LevelId::TimedGate (timed_gate::TIMED_GATE), registered in LEVEL_IDS and PUZZLE_IDS after jelly-pad"
  - "timed_gate::{REFERENCE_VALVE_POSE, TIMED_GATE_WITH_PLANK, PERMANENT_PLANK_POSE} (test-only)"
  - "Four recorded fixtures: timed-gate solution (6-token valve line), empty, early (tick 1), late (tick 24)"
  - "level_proofs: timed_gate_open_time_decides_the_outcome, timed_gate_is_reproducible, timed_gate_needs_the_valve_to_open, nudges and four fixture replays"
  - "levels_tests/fixture_table.rs: the FIXTURES table (moved out of levels_tests.rs)"
  - "web/e2e/puzzle-valve.spec.ts: browser replay of the valve reference through the stepper"
affects:
  - "40-08/40-09 insert sand-chute and float-up before timed-gate in D-09 order; add fixtures to levels_tests/fixture_table.rs"
  - "40-10 grand-finale can reuse the tent mechanism (valve as jet backstop plus syrup seal)"
tech-stack:
  added: []
  patterns:
    - "Timing window as a tent: the closed valve keeps a prefix of one stream (jet backstop) and a suffix of another (syrup seal); early loses the first, late loses the second"
    - "A one-slot build area when any other valid position would bypass the lesson"
    - "An uncommitted data-driven draft probe (geometry from a text file, parallel tick sweeps, ASCII flow maps) for fast tuning without recompiles"
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/timed_gate.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/fixture_table.rs
    - crates/liquidfun-wasm/src/puzzle/fixtures/timed-gate-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/timed-gate-empty.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/timed-gate-early.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/timed-gate-late.txt
    - web/e2e/puzzle-valve.spec.ts
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs
    - web/scripts/puzzle-replay/fixture.ts
    - web/src/catalog/puzzles.ts
    - web/src/catalog/puzzle-levels.ts
    - web/src/catalog/puzzle-bounds.ts
    - web/tests/puzzle-bounds.test.ts
    - web/package.json
key-decisions:
  - "Shipped design is the D-26 syrup fallback (strength exactly 1.0) in a tent layout: an upright valve stops a water jet into the cup and seals a syrup tank's side opening. The preferred diverter was spiked first and gave a monotone sweep in every variant"
  - "The valve has a one-slot build area (grid x 2..4, y 59..63). A lower valve still catches the jet but uncovers the syrup opening, which would win at any tick and bypass timing"
  - "Reference open tick 9 (4.5 s), not the band centre 12: only ticks 5-10 hold the margin-raised Brim by step 600"
  - "Caption describes the real mechanism: 'The valve opens on a timer. Catch the water first, then let the syrup out.'"
  - "Side walls rise to 10.5 m (no physics effect) so the portrait frame fills more than half a phone's height"
  - "FIXTURES moved to levels_tests/fixture_table.rs; levels_tests.rs is now 598 lines"
metrics:
  duration: 66min
  completed: 2026-10-13
  tasks: 3
  files: 16
---

# Phase 40 Plan 07: Timed Gate Summary

`timed-gate` is registered as level 4 of the current registry, after jelly-pad. An upright valve in a fixed slot first stops a water jet so it drops into the cup. It also seals a strength-1.0 syrup tank. Opening it frees the syrup into the cup and lets any remaining jet fly to waste.

The release open-tick sweep shows the window: ticks 1-2 lose, 3-21 win and 22-24 lose. All four fixtures replay identically in debug, release and wasm under the exact contract. A reproducibility test proves equal frame hashes and outcomes. A Playwright test builds the reference through the tray, canvas and valve stepper, and reaches the fixture's outcome step 429 and peak held 582 with the Brim star.

## Task 1: Valve design spike, draft and tuning (commit 7340a1924)

### Design search (probe output, authoring only; log `target/phase40/probes/timed-gate.log`)

Tuning used an uncommitted, data-driven draft probe. It read the geometry from a text file and ran each tick sweep in parallel threads, about 2 s per sweep, and printed ASCII flow maps. Its source is kept at `target/phase40/probes/tg-drafts/draft_probe.rs.keep`, with every draft input and output beside it.

1. **Preferred diverter (D-26), drafts d1-d3.** Two water tanks shared a funnel over the cup. The valve deflected tank B to the waste floor while closed. Every variant was monotone: opening earliest was best.
   - d2: tick 1 peaked at 1178, falling to 472 at tick 24.
   - d3: tick 1 peaked at 1036, falling to 581 at tick 24.

   The reason is structural. A diverted stream is wasted whether or not the funnel overflows, so an early opening never loses more than an on-time one. No losing early band is possible.
1. **Uncounted "muddy" water diverter, m1-m3.** Rejected for two reasons:
   - Suffix order: the newest water flushes the cup. The m1 empty run counted 890 clean.
   - Prefix order: the win latches while the clean water is still filling, before the muddy water can displace it.
1. **Syrup seal plus funnel or tray overflow (the D-13 classic), s1-s15 and the tray and column families.**
   - Hole and neck rates rise with head, so funnels absorbed every overlap. When they did spill, each stream alone spilled as much.
   - The best early harm was only s14: tick 1 at 536 against 617.
   - Side findings:
     - A held syrup plug leaks with water head, not with plug thickness (about 40 at 0.6 m, about 190 at 1.6 m).
     - 0.2 m flat holes jam at about 25/s.
     - Syrup does not flow across a flat floor.
1. **Shipped tent design, t1 to the g-grid.** The closed valve keeps a prefix of water A (its jet backstop) and a suffix of syrup B (its seal).
   - A dam at the opening's sill halved the leak, from 85 to 49.
   - Raising A's launch lip moved the jet to mid-valve.
   - A threshold evaluator chose the counts.

### Geometry and numbers

- Box: x -4.5..4.5 m, y -0.5..10.5 m.
- Water tank: floor `quad(-4.3, 5.2, -2.4, 5.4)` with a 0.5 m outlet (x -2.4..-1.9). A 45° ramp and a launch lip turn the stream into a jet aimed right.
- Syrup tank:
  - A 45° floor from (0.41, 3.0) up to (1.81, 4.4), with a dam `quad(0.41, 3.0, 0.5, 3.12)`.
  - Left wall `quad(0.41, 3.5, 0.61, 6.6)`, which leaves the side opening at x 0.41 between y 3.12 and 3.5.
  - Right wall `quad(1.81, 4.2, 2.01, 6.8)`.
- Cup: walls at x -1.4..-1.2 and 0.6..0.8, 1.6 m tall. `cup_interior` is (-1.2, 0)–(0.6, 1.6).
- Liquids:
  - Water: block (-4.26, 5.44)–(-2.44, 6.22).
  - Syrup: block (0.65, 4.45)–(1.77, 5.71), `LiquidMaterial::Viscous`, counted.
  - Viscous strength is exactly `Some(1.0)`.
- Thresholds: full 800 (2.88 m² / 0.0036), target 502 (63%), Brim 582 (target + 10% of full). Ceiling is 462 and the margin-raised Brim is 622.
- Tools: `Valve 1`, par 1, step limit 720. The build area is grid (2, 59, 4, 63), the valve slot.
- Reference: `placement valve 4 61 6 9`, an upright valve centred at (0.2, 3.05) m that opens at step 270.

### Final probe (`probe_timed_gate`, release)

```
PROBE timed-gate empty outcome 2 step 720 peak 443 brim none particles 802
PROBE timed-gate reference outcome 1 step 429 peak 692 brim 519 particles 802
PROBE timed-gate permanent-plank outcome 2 step 720 peak 441 brim none particles 802
PROBE timed-gate nudges wins 8 valid 8 skipped 2 generated 10
```

The two skipped nudges are x+1 and x+2, which are outside the slot.

Draft measurements with the final geometry:

| Run | Count |
| --- | --- |
| Valve-caught water alone | 388 |
| Syrup alone, opened at tick 1 | 378 |
| Syrup leak while held for the whole run | 52 |

## Task 2: Registration, fixtures, sweep, reproducibility (commit f0f82543e)

### Registry, title, caption and frames

- Rust: `LEVEL_IDS = [FirstPour, ThickSyrup, JellyPad, TimedGate]`. TS: `PUZZLE_IDS = ["first-pour", "thick-syrup", "jelly-pad", "timed-gate"]`. The draft dead-code allowance is gone.
- Title: "Timed Gate".
- Caption: "The valve opens on a timer. Catch the water first, then let the syrup out." jelly-pad's Next now links to timed-gate.

| Frame | Rectangle |
| --- | --- |
| Landscape | -4.7, -0.7, 4.7, 10.7 |
| Portrait | -4.6, -0.6, 4.6, 10.6 (fills more than half a 390×844 phone) |

`LEVEL_GEOMETRY` is cross-checked against the generated wasm in `target/phase40/level-probe-timed-gate.txt`:

- `cupInterior` [-1.2, 0, 0.6, 1.6] and `buildArea` [2, 59, 4, 63].
- Segment extents x -4.5..4.5, y -0.5..10.5.
- 802 particles at x -4.26..1.74, y 4.5..6.18.

### Open-tick window sweep (release, `target/phase40/timed-gate-sweep.txt`)

| Ticks | Outcome |
| --- | --- |
| 1, 2 | lost |
| 3–21 | won (one contiguous band of 19; reference 9 inside) |
| 22, 23, 24 | lost |

`timed_gate_open_time_decides_the_outcome` asserts `timing_window_ok` and the reference tick's win. `timed_gate_needs_the_valve_to_open` asserts that a permanent upright plank in the slot ends `Lost`. That uses `run_spec_outcome` on the test-only spec `TIMED_GATE_WITH_PLANK`.

### RECORD lines (release; debug is identical)

```
RECORD timed-gate-solution.txt outcome won outcome_step 429 final_count 531 peak_held 582 brim_step 519
RECORD timed-gate-empty.txt outcome lost outcome_step 720 final_count 447 peak_held 443 brim_step none
RECORD timed-gate-early.txt outcome lost outcome_step 720 final_count 447 peak_held 443 brim_step none
RECORD timed-gate-late.txt outcome lost outcome_step 720 final_count 444 peak_held 440 brim_step none
```

The loss margins are 15 for empty and early (max count 447 against the 462 ceiling) and 18 for late (444). The debug-to-release spread is 0, so no `debug_outcome_step_tolerance` was added.

### Determinism and wasm (D-08)

- `timed_gate_is_reproducible` runs in every lane. Two sessions built from the solution lanes have equal `frame_hash` after 360 steps, which is past the open step of 270. Both latch the same `Won` outcome.
- `just puzzle-replay-wasm` passes all 10 fixtures under `WASM_REPLAY_CONTRACT = "exact"`, including all four timed-gate fixtures with values identical to release-native. No fallback was used.

### Nudge tally and budgets

- `NUDGES timed-gate-solution.txt wins 8 valid 8 skipped 2 generated 10`.
- The other levels are unchanged: first-pour 8/8, thick-syrup 15/15, jelly-pad 8/8.
- `STEPCOST timed-gate particles 802 ms_per_step 0.24`. That is well under the 3,000-particle step-0 budget.

## Task 3: Browser replay through the stepper (commit f1718b024)

`web/e2e/puzzle-valve.spec.ts` runs at 1280×800 in the chromium project only. The touch project matches only `puzzle-touch.spec.ts`. The test:

1. Parses `timed-gate-solution.txt` and arms "Valve, 1 left".
1. Taps `gridToPage(4, 61, "timed-gate")`, corrects the cell with arrow keys, and presses `r` six times to reach angle 6.
1. Clicks "Open the valve 0.5 s later" five times, from tick 4 to tick 9, and asserts the readout "4.5 s".
1. Asserts `data-puzzle-placements` is `valve:4,61,6,9`.
1. After Escape, asserts both stepper buttons are `aria-disabled="true"`. It reselects the valve on the canvas and checks that `]` reads "5.0 s" and `[` returns to "4.5 s".
1. Plays and checks the browser-observed values: `data-puzzle-outcome-step` 429 and `data-puzzle-peak-held` 582, both equal to the fixture, plus a "Filled!" card and the earned Brim star row.

Screenshots: `target/phase40/screens/valve-running.png` (the jet hitting the closed valve, with the "2.8 s" countdown label) and `valve-won.png`.

## Verification

- `cargo fmt --all` passed.
- Both clippy commands pass with `-D warnings`, and `cargo build --all-targets --all-features` passed.
- `cargo test -p liquidfun-wasm` (debug): 559 passed, 16 ignored.
- `cargo test -p liquidfun-wasm --release --lib puzzle::`: 246 passed, 7 ignored. This includes the sweep, the nudges and the plank control.
- `just puzzle-replay-wasm`: 10/10 pass, with an exact contract.
- Web:
  - `bunx tsc --noEmit` is clean.
  - `bun run test:unit` passes 97 files and 1,375 tests.
  - `bun run build:app` succeeded, and `CI=true bun run test:puzzle` passes 39/39.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `crates/liquidfun` and the first-pour fixtures have no diff from the phase base. The thick-syrup and jelly-pad fixtures are unchanged. No scene, builder or session code changed, so the 25 scene fingerprints are unaffected (the Plan 11 gate re-measures them).

## Deviations from Plan

### Design decisions within plan discretion

**1. The diverter design could not show an early losing band, so the D-26 syrup fallback shipped in a tent layout.**
- The plan's spike was run first (d1-d3) and was monotone, as described under Task 1.
- The plan's fallback sketch (a viscous reservoir behind the valve feeding a shared chute) was also tuned at length (s1-s15). It never produced a decisive early band, because syrup crawls through any constriction that water saturates.
- The shipped layout keeps every D-26 rule:
  - There is no valve-seat exemption, and the 0.08 m clearance applies.
  - The valve blocks a stream (the jet).
  - The syrup is exactly strength 1.0.
  - The trickle of about 50 does not decide the outcome. The margins are 13 or more.

**2. One-slot build area.**
- The nudge sweep exposed a bypass. A valve 2 cells lower still catches the jet but uncovers the top of the syrup opening, which wins at every tick.
- The build area is therefore the valve slot, grid x 2..4 and y 59..63. Every position in it both catches the jet and seals the syrup, and all 8 valid nudges win.

**3. Reference tick 9 rather than the band centre.**
- The winning band is ticks 3-21, so its centre is 12.
- The margin-raised Brim (622) holds by step 600 only for ticks 5-10. At tick 12 the draft holds about 600 by step 600, which misses the raised Brim.
- Tick 9 is the most central tick with margin, and ticks 8 and 10 also win with Brim.

**4. Caption.** The plan's default ("Pick the moment the water goes through") does not describe this mechanism. The caption names the two steps the player times.

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `levels_tests.rs` would exceed the 628-line limit**
- **Found during:** Task 2
- **Issue:** Four more `FIXTURES` entries (16 lines) would take `levels_tests.rs` past 628 lines, from 626.
- **Fix:** The table moved to the child module `levels_tests/fixture_table.rs`, using `include_str!("../fixtures/…")`. `levels_tests.rs` re-imports it and is now 598 lines. It is still the same lib test binary.
- **Files modified:** crates/liquidfun-wasm/src/puzzle/levels_tests.rs, crates/liquidfun-wasm/src/puzzle/levels_tests/fixture_table.rs
- **Commit:** f0f82543e

**2. [Rule 1 - Bounds] The portrait frame filled 49% of a phone**
- **Found during:** Task 2 (`puzzle-bounds.test.ts`)
- **Issue:** With 10 m walls the timed-gate portrait frame filled 0.493 of an iPhone's height, below the 0.5 rule for portrait-authored levels.
- **Fix:** The side walls rise to 10.5 m. Nothing in the level is above 8 m, and the RECORD lines did not change.
- **Commit:** f0f82543e

**Process notes:**
- The draft probe module was never committed. Task 1 moved it to `target/phase40/probes/tg-drafts/draft_probe.rs.keep` before the commit.
- LEVEL-01, LEVEL-02 and LEVEL-04 stay Pending in REQUIREMENTS.md, as instructed. TOOL-05 was already marked complete in REQUIREMENTS.md, so `requirements mark-complete TOOL-05` changed nothing.

## Known Stubs

None.

## Threat Flags

None. The threat register is covered:

- T-40-24: the open step comes only from the integer tick and the session step counter. The reproducibility test and the exact wasm replay prove it.
- T-40-25: the Rust range check still runs on every build, and the e2e uses only the stepper.
- T-40-26: the sweep is an assertion, and the losing bands are reported in full above.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/puzzle/timed_gate.rs, levels_tests/fixture_table.rs, the four timed-gate fixtures, web/e2e/puzzle-valve.spec.ts, target/phase40/probes/timed-gate.log, target/phase40/timed-gate-sweep.txt, target/phase40/screens/valve-running.png
- FOUND: commits 7340a1924, f0f82543e and f1718b024
