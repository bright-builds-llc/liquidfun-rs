---
phase: 40-timed-valve-and-teaching-levels
plan: "10"
subsystem: wasm-puzzle-levels
tags: [rust, wasm, puzzle, levels, finale, valve, jelly, viscous, fixtures, window-sweep, playwright, LEVEL-01, LEVEL-02, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T09:23:56.440Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-03 probe/levels_tests harness; 40-06 jelly pad and level_proofs.rs; 40-07 Timed Gate tent and fixture_table.rs; 40-08 sand-chute dropped; 40-09 float-up registered"
provides:
  - "LevelId::GrandFinale (grand_finale::GRAND_FINALE), the last entry of LEVEL_IDS and PUZZLE_IDS"
  - "grand_finale::{REFERENCE_PLANK_POSE, REFERENCE_VALVE_POSE, GRAND_FINALE_WITHOUT_PAD, GRAND_FINALE_WITH_PLANKS} (test-only)"
  - "grand-finale-solution.txt (6-token valve line) and grand-finale-empty.txt fixtures"
  - "level_proofs: grand_finale_needs_the_valve, grand_finale_valve_time_matters, grand_finale_needs_the_pad, grand_finale_nudges_mostly_win and two fixture replays"
  - "levels_tests: level_count_is_within_the_teaching_range"
  - "web/e2e/puzzle-levels.spec.ts: every registered level at desktop and phone, Next walk, and the last level's disabled Next"
affects:
  - "40-11 gates: 14 fixtures in just puzzle-replay-wasm; 6 registered levels; grand-finale step cost 0.36 ms/step"
  - "Phase 41 level grid: PUZZLE_IDS has 6 levels ending with grand-finale"
tech-stack:
  added: []
  patterns:
    - "Build area as a narrow column: when one GridRect must hold two tool zones, place them so the rectangle between them holds no liquid"
    - "Ridged roof: the untouched stream lands on the waste side of a ridge; a plank carries it over to the delivery side"
    - "Margin budget for a tent level: reference minus closed-valve run is the syrup delivered after the open tick (about 185), so loss margin plus Brim margin is fixed at about 65; the water amount splits it"
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/grand_finale.rs
    - crates/liquidfun-wasm/src/puzzle/fixtures/grand-finale-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/grand-finale-empty.txt
    - web/e2e/puzzle-levels.spec.ts
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/fixture_table.rs
    - crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs
    - web/src/catalog/puzzles.ts
    - web/src/catalog/puzzle-levels.ts
    - web/src/catalog/puzzle-bounds.ts
    - web/tests/puzzle-bounds.test.ts
    - web/package.json
key-decisions:
  - "Combination: jelly pad (Elastic) + timed valve + syrup (Viscous 1.0), three ideas including the valve. The plank steers the water, so the syrup needs no plank path"
  - "The lower half reuses the Timed Gate tent unchanged (cup, valve slot, syrup tank, ramp and lip). The jelly pad feeds the ramp: its nozzle output free-falls onto the ramp, which a direct pad-to-ramp hand-off could not make fast enough"
  - "Build area is the column x 0.1..0.6 m above the valve slot, so it holds no liquid (builder invariant). A ridged roof sends the untouched stream to waste, and the plank carries it over the ridge"
  - "Reference: plank 10 172 1 (centre 0.5, 8.6 m at 15°) and valve 4 61 6 14 (opens at 7.0 s); budget Plank 2 + Valve 1, par 2, step limit 900"
  - "Caption: 'Finale: drop water on the jelly, catch it with the valve, then free the syrup.' (78 characters, two-line clamp safe)"
  - "A plank cannot stand in the valve slot (the cup ceiling blocks it), so the plank-instead control is recorded as invalid and the never-opening valve (tick 30) stands in for a permanent gate"
metrics:
  duration: 43min
  completed: 2026-10-13
  tasks: 3
  files: 13
---

# Phase 40 Plan 10: Grand Finale Summary

**`grand-finale` combines a jelly pad, a timed valve and syrup.** The player's plank drops a reservoir stream onto a roof that feeds a chimney. An uncounted jelly pad turns the stream onto a ramp, and the ramp launches a jet. While closed, the valve stops the jet so the water drops into the cup, and it also seals a strength-1.0 syrup tank. Opening it frees the syrup.

The level is registered last, making 6 levels. Its fixtures replay identically in debug, release and wasm, and these release tests pass: `grand_finale_needs_the_valve`, `grand_finale_valve_time_matters`, `grand_finale_needs_the_pad`, and the nudge sweep (16/16). A new Playwright spec opens every registered level at 1280×800 and 390×844. It also walks Next through the registry and shows the finale's Next disabled with "Next: No next level yet".

## Task 1: Draft and tune (commit 86d16bdef)

Tuning used the uncommitted data-driven draft probe from 40-09, with elastic blocks added. Its source is kept at `target/phase40/probes/gf-drafts/gf_draft_probe.rs.keep`, beside every draft input and output. The log is `target/phase40/probes/grand-finale.log`. Probe output is authoring data only (D-18).

| Draft | Change | Result |
| --- | --- | --- |
| a1 | Timed Gate tent fed by the pad nozzle at the ramp top; reservoir upper right; plank onto a 14° roof | Winning ticks 9–14 only; peak 504–525 against target 502; water arrives about step 240 |
| a2 | 21° roof, 2.0 m cup (full 1000), 790 water, limit 900 | No-valve run won (633): the weak jet falls into the cup |
| a3 | 573 water | No-valve 592. The flow map shows the jet leaving the lip slowly and landing inside the cup |
| b1 | Pad lifted so its output free-falls onto the ramp; 2.0 m cup | No-valve won (672): the stream landed at the ramp foot |
| b2 | Timed Gate cup (1.6 m), cradle moved left, ramp extended up-left, 0.5 m outlet, 602 water | Strong jet, no-valve 335; but a never-opening valve won (544) |
| b3–b4 | 470 water, 0.6 m outlet | Window ticks 9–27; reference holds 656 by step 780. This draft was committed |

## Task 2: Registration retune, fixtures, proofs and frames (commits 67f3c943b, 31dec5b1e)

### Why the geometry changed

The b4 build area (x 0.1..4.2 m) covered the syrup block, so the builder invariant `liquid_starts_outside_the_cup_and_the_build_area` failed in the release lane. The fix keeps that test and moves the plank zone into a column directly above the valve slot (x 0.1..0.6 m):

| Draft | Change | Result |
| --- | --- | --- |
| c1 | Ridge roof (ridge at x 0.8); reservoir across the top right with a tilted floor; plank column | Never-opens 367; reference holds only 557 (spray lost at the plank's end) |
| c2 | c1 with 660 water | Never-opens won (510) |
| c3–c4 | Ridge at x 0.6; outlet at x 1.05..1.45; 528 and 581 water | Reference holds 599 and 614 by step 780, short of 622 |
| c5 | 0.6 m outlet (x 1.05..1.65); plank centre at x 0.5 | Much better catch; never-opens won (551) |
| c6 (final) | c5 with 480 water | Results below |

### Final geometry and numbers

- Box: x -4.5..4.5 m, y -0.5..11.2 m (9 m by 11.7 m).
- Lower half: the Timed Gate tent, unchanged.
  - Cup interior (-1.2, 0)–(0.6, 1.6).
  - Valve slot at grid x 2..4, y 59..63.
  - Syrup tank opening at x 0.41, y 3.12..3.5, behind a sill dam.
  - 45° ramp from (-3.0, 4.95) down to (-1.3, 3.25), then the launch lip.
  - A ceiling over the cup at y 3.9 and a wall beside the ramp close every other route into the cup.
- Jelly pad:
  - Elastic, 0.5 strength, uncounted, about 150 particles.
  - It sits in a slotted cradle at x -4.3..-2.7, top y 5.62, under a chimney. The baffle leaves a 0.28 m nozzle.
- Roof: ridged at x 0.6. The left roof falls to the chimney and the right roof falls to the waste shaft.
- Reservoir: x 1.05..4.3 with a tilted floor and a 0.6 m outlet at x 1.05..1.65. Water block (1.09, 10.14)–(4.26, 10.68).
- Tools: `Plank 2, Valve 1`, par 2, step limit 900 (max open tick 30).
- Build area: grid (2, 59, 12, 190). Three no-place zones leave only the plank column above the roof and the valve slot.
- Thresholds: full 800, target 502 (63%), Brim 582. The empty-run ceiling is 462 and the margin-raised Brim is 622.
- Step-0 particles: 1,037 (budget 3,000).

### Final committed probe (`probe_grand_finale`, release)

```
PROBE grand-finale empty outcome 2 step 900 peak 419 brim none particles 1037
PROBE grand-finale reference outcome 1 step 574 peak 708 brim 670 particles 1037
PROBE grand-finale no-valve outcome 2 step 900 peak 352 brim none particles 1037
PROBE grand-finale plank-instead invalid InvalidPlacement(OverlapsWall)
PROBE grand-finale valve-never-opens outcome 2 step 900 peak 470 brim none particles 1037
PROBE grand-finale without-pad outcome 2 step 900 peak 351 brim none particles 876
PROBE grand-finale nudges wins 16 valid 16 skipped 2 generated 18
```

Draft component counts with the final geometry: the valve catches 399 water when the syrup is removed, held syrup leaks 71, and syrup alone gives 419. The reference holds 655 by step 780, against the raised Brim of 622.

### RECORD lines (release; debug is identical)

```
RECORD grand-finale-solution.txt outcome won outcome_step 574 final_count 527 peak_held 582 brim_step 670 profile release
RECORD grand-finale-empty.txt outcome lost outcome_step 900 final_count 421 peak_held 419 brim_step none profile release
```

The debug spread is 0, so no `debug_outcome_step_tolerance` was added. The empty run's maximum count is 421, which is 41 under the 462 ceiling.

### Proofs (release lane)

| Test | Assertion | Result |
| --- | --- | --- |
| `grand_finale_needs_the_valve` | The reference plank without the valve is `Lost` | pass |
| `grand_finale_valve_time_matters` | At least one open tick in 1..=30 is not `Won` | pass: ticks 1–9 and 28–30 lose |
| `grand_finale_needs_the_pad` | The reference on `GRAND_FINALE_WITHOUT_PAD` is `Lost` | pass |
| `grand_finale_nudges_mostly_win` | ≥ 70% of valid nudges win | `NUDGES grand-finale-solution.txt wins 16 valid 16 skipped 2 generated 18` |
| `level_count_is_within_the_teaching_range` (every lane) | 5–8 levels, the required ids present, GrandFinale last | pass (6 levels) |

The skipped nudges are valve x+1 and x+2, which leave the slot. Open-tick sweep (release `SWEEP` lines):

| Ticks | Outcome |
| --- | --- |
| 1–9 | lost (opened before the water arrives; the jet flies past) |
| 10–27 | won (the reference tick 14 is inside) |
| 28–30 | lost (the syrup cannot fill the cup in time) |

### Registry, title, caption and frames

- Rust `LEVEL_IDS` and TS `PUZZLE_IDS` both list: first-pour, thick-syrup, jelly-pad, float-up, timed-gate, grand-finale.
- Title "Grand Finale". Caption: "Finale: drop water on the jelly, catch it with the valve, then free the syrup."
- Frames: landscape (-4.7, -0.7, 4.7, 11.4) and portrait (-4.6, -0.6, 4.6, 11.3). The portrait frame fills about 60% of a 390×844 phone's height.
- `LEVEL_GEOMETRY` was cross-checked against the generated wasm (`target/phase40/level-probe-grand-finale.txt`):
  - `cupInterior` [-1.2, 0, 0.6, 1.6] and `buildArea` [2, 59, 12, 190];
  - segments x -4.5..4.5, y -0.5..11.2;
  - 1,037 particles at x -4.26..4.26, y 4.5..10.62.

### Wasm contract

The contract is exact. `just puzzle-replay-wasm` passes all 14 fixtures under `WASM_REPLAY_CONTRACT = "exact"`. The grand-finale values equal release-native (won 574, peak held 582, Brim 670; empty lost at 900 with peak 419). No fallback was used (`target/phase40/replay-wasm-10.log`).

### Step cost (D-23, informational)

```
STEPCOST grand-finale particles 1037 ms_per_step 0.36
```

The other levels are unchanged: first-pour 0.56, thick-syrup 0.44, jelly-pad 0.43, float-up 0.84 and timed-gate 0.25 ms/step.

## Task 3: Every-level e2e (commit 322d3b61d)

`web/e2e/puzzle-levels.spec.ts` (chromium project) iterates `PUZZLE_IDS`:

- At 1280×800 and 390×844, each level opens at `#/puzzle/<id>` into building. The test checks:
  - the title equals `PUZZLE_TITLES[id]`;
  - the caption leaf shows `PUZZLE_CAPTIONS[id]`, or `FIRST_LEVEL_HINT` on first-pour with fresh storage;
  - the failure overlay has `data-failed="false"`;
  - the camera did not fall back;
  - `overlappingLeaves` and `overlappingObstacles` are both empty.

  Screenshots are `target/phase40/screens/level-<id>-<WxH>.png`, 12 files.
- At 1280×800, an empty run at 4× ends `lost`. For every level except the last, Next links to `puzzleHash(next)`. On grand-finale, the button "Next: No next level yet" is disabled and no "Next level" link exists (`level-grand-finale-last-next.png`).
- The spec is added to `test:puzzle`, and `CI=true bun run test:puzzle` passes 57/57: the 39 earlier tests plus 18 new ones.

## Verification

- Rust checks:
  - `cargo fmt --all --check` passes.
  - Both clippy commands pass with `-D warnings`.
  - `cargo build --all-targets --all-features` passes.
- Rust test lanes:
  - `cargo test -p liquidfun-wasm` (debug): 565 passed, 23 ignored.
  - `cargo test -p liquidfun-wasm --release --lib puzzle::`: 257 passed, 9 ignored. This includes every nudge sweep, the open-tick sweeps and the idea controls.
- `just puzzle-replay-wasm`: 14/14 pass, under the exact contract.
- Web:
  - `bunx tsc --noEmit` is clean.
  - `bun run test:unit` passes 97 files and 1,375 tests.
  - `bun run build:app` succeeded, and `CI=true bun run test:puzzle` passes 57/57.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- Unchanged files:
  - `crates/liquidfun` has no diff from the phase base.
  - The only fixture changes are the two new grand-finale files.
  - No scene, builder or session code changed, so the 25 scene fingerprints are unaffected. The Plan 11 gate re-measures them.
- `grand_finale.rs` and `puzzle.rs` carry no `allow(dead_code)`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The Task 1 build area covered the syrup block**
- **Found during:** Task 2 (release `puzzle::` lane)
- **Issue:** `liquid_starts_outside_the_cup_and_the_build_area` failed for grand-finale. The b4 build rectangle spanned the valve slot and an upper-right plank zone, so it covered the syrup tank.
- **Fix:** The plank zone moved into a column above the valve slot. A ridged roof sends the untouched stream to waste, and the reservoir was re-tuned (c1–c6 above). The invariant test was not changed.
- **Files modified:** crates/liquidfun-wasm/src/puzzle/grand_finale.rs
- **Commit:** 67f3c943b

### Design decisions within plan discretion

**2. The plank steers the water, not the syrup.** The plan's default sketch had the syrup on a steep plank path. The proven Timed Gate tent drops the syrup straight into the cup, so the plank's job is to deliver the water onto the jelly. The level still uses three ideas: Elastic, Viscous and Valve.

**3. Par 2, not 3.** The reference needs one plank and one valve, and the plan allows fewer than 3.

**4. Plank-instead control.** No valid plank fits the valve slot, because the 3 m plank hits the cup ceiling. The probe records that placement as invalid (`OverlapsWall`), and the never-opening valve (tick 30, which loses with peak 470) stands in for a permanent gate. `grand_finale_needs_the_valve` asserts the plan's control (the reference without the valve).

**5. Extra control `grand_finale_needs_the_pad`.** It is cheap and shows that the jelly pad matters (without it the run loses with peak 351).

**6. Caption.** The default caption describes a syrup plank path, which this design does not have. The shipped caption names what the level really combines.

### Path adjustments

**7. Test file.** The plan's acceptance greps `levels_tests.rs` for the two valve tests. As the orchestrator directed, per-level tests live in `levels_tests/level_proofs.rs` (lines 399 and 417). `level_count_is_within_the_teaching_range` is in `levels_tests.rs` as planned, which is now 620 lines, under the 628 limit.

**Process notes:**
- The draft probe module was never committed. Its source is in `target/phase40/probes/gf-drafts/`.
- LEVEL-01, LEVEL-02 and LEVEL-04 stay Pending in REQUIREMENTS.md, as instructed.

## Deferred Issues

- Tool-tray name truncation in portrait when a level has two tool kinds ("Pl… ×2", "Va… ×1"). This is logged in `deferred-items.md`. The icons, counts and accessible names are intact.

## Known Stubs

None.

## Threat Flags

None. The threat register is covered:

- T-40-31: grand-finale is allowlisted in both mirrors, and the `include_str!` mirror test passes. The e2e opens only `PUZZLE_IDS`, and the unknown-id fallback test still passes.
- T-40-32: 1,037 particles at step 0. Full fixture runs pass, and the every-level e2e checks that no failure overlay appears.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/puzzle/grand_finale.rs, both grand-finale fixtures, web/e2e/puzzle-levels.spec.ts, target/phase40/probes/grand-finale.log, target/phase40/level-probe-grand-finale.txt, target/phase40/screens/level-grand-finale-390x844.png and level-grand-finale-last-next.png
- FOUND: commits 86d16bdef, 67f3c943b, 31dec5b1e and 322d3b61d
