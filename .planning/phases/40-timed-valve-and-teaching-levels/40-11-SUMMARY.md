---
phase: 40-timed-valve-and-teaching-levels
plan: "11"
subsystem: phase-gates
tags: [gates, D-24, D-23, fingerprints, fixtures, wasm-replay, playwright, step-cost, TOOL-05, LEVEL-01, LEVEL-02, LEVEL-03, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T09:37:47.076Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-01 baseline fingerprints (target/phase40/base.txt at phase base 65a37d35b); 40-02..40-10 valve, materials, hint, and the six registered levels"
provides:
  - "target/phase40/checks-11.{sh,results,log}: 27 D-24 gates, all exit 0 on HEAD 7353eaded"
  - "target/phase40/after-final.txt: 25 post-phase fingerprints, identical to base.txt"
  - "target/phase40/step-costs.txt: informational release ms/step for all 6 levels (D-23)"
  - "Phase 40 decision record: registry, per-level numbers, D-25 powder drop, D-09 buoyancy ship, D-26 valve design, D-11 pad"
affects:
  - "Phase 40 verification: D-24 evidence for TOOL-05 and LEVEL-01..04"
  - "Phase 41 level grid: PUZZLE_IDS has 6 levels, first-pour .. grand-finale"
tech-stack:
  added: []
  patterns:
    - "Final-gate script: one `run <name> <cmd>` per gate, each in its own errexit subshell, results appended as `<name> exit=<code> <UTC>`; every filtered cargo test gate greps for a non-zero passed count"
key-files:
  created:
    - target/phase40/checks-11.sh (gitignored evidence)
    - target/phase40/checks-11.results (gitignored evidence)
    - target/phase40/checks-11.log (gitignored evidence)
    - target/phase40/after-final.txt (gitignored evidence)
    - target/phase40/step-costs.txt (gitignored evidence)
    - target/phase40/release-puzzle.log (gitignored evidence)
    - target/phase40/particle-budget.log (gitignored evidence)
  modified: []
key-decisions:
  - "D-24 holds on HEAD 7353eaded: 27/27 gates exit 0 on the first run; no fix commit and no failed attempt files were needed"
  - "Final registry is 6 levels (first-pour, thick-syrup, jelly-pad, float-up, timed-gate, grand-finale); sand-chute dropped under D-25, float-up kept under D-09"
  - "Shipped valve design (D-26): the syrup-seal tent; the closed valve backstops a water jet and seals a strength-1.0 syrup tank. Window sweeps lose early and late (timed-gate 1-2 / 3-21 win / 22-24; grand-finale 1-9 / 10-27 win / 28-30)"
  - "first-pour keeps its Phase 38 portrait frame (about 34% phone height) because D-09 locks its geometry; every new level meets the 0.5 phone-height fill"
metrics:
  duration: 11min
  completed: 2026-10-13
  tasks: 2
  files: 0
---

# Phase 40 Plan 11: Final D-24 Gates and Decision Record Summary

**All 27 D-24 gates pass on the final tree (HEAD 7353eaded), on the first run. The scene-spot stamp `2026-10-13T09-27-14Z` gives 25/25 fingerprints identical to the phase base. `crates/liquidfun`, the first-pour fixtures and the protected tests are unchanged. All 14 fixtures replay exactly in wasm, and the puzzle e2e suite passes 57/57.** The informational step costs below run from 0.25 to 0.85 ms/step. No fix was needed, so this plan made no code commit.

## Task 1: D-24 gates

Script: `target/phase40/checks-11.sh`, which uses the Phase 39 Plan 14 format. Results are in `target/phase40/checks-11.results` and output is in `checks-11.log`. Phase base is `65a37d35b2d5f907f8fc08fa1c8c9aa90303895e`.

| # | Gate | Exit | Evidence |
| --- | --- | --- | --- |
| 1 | fingerprints | 0 | stamp `2026-10-13T09-27-14Z`, `diff base.txt after-final.txt` empty, 25 lines (**25/25 identical**) |
| 2 | engine-untouched | 0 | `git diff --stat $BASE -- crates/liquidfun` empty |
| 3 | first-pour-fixtures-unchanged | 0 | `first-pour-solution.txt` and `first-pour-empty.txt` have no diff and exist |
| 4 | protected-tests-unedited | 0 | hash, portrait-bounds and readme-svg tests plus hud-safe-frame spec have no diff |
| 5 | cargo-fmt | 0 | `cargo fmt --all --check` |
| 6 | cargo-clippy | 0 | `-D warnings` |
| 7 | cargo-clippy-wasm | 0 | `-p liquidfun-wasm`, `-D warnings` |
| 8 | cargo-build | 0 | `--all-targets --all-features` |
| 9 | cargo-test | 0 | 1,084 passed, 0 failed (summed over all test binaries) |
| 10 | wasm-crate-debug | 0 | `cargo test -p liquidfun-wasm --all-features`: 565 passed, 23 ignored |
| 11 | release-puzzle | 0 | `--release --lib puzzle::`: **257 passed**, 9 ignored (exact fixtures, every nudge sweep, both window sweeps, idea controls) |
| 12 | puzzle-replay-wasm | 0 | 14/14 `REPLAY … pass`, exact contract |
| 13 | typecheck | 0 | `bunx tsc --noEmit` clean |
| 14 | vitest | 0 | 97 files, 1,375 tests |
| 15 | bright-builds | 0 | `findings=0` (1,498 files scanned) |
| 16 | player-smoke | 0 | 45 passed |
| 17 | hud-safe-frame | 0 | fresh `build:app`, `CI=true`: 64 passed |
| 18 | puzzle-e2e | 0 | fresh `build:app`, `CI=true bun run test:puzzle`: 57 passed (valve browser replay, hint dismissal, every-level routes) |
| 19 | markdown | 0 | no non-GSD Markdown changed since the base |
| 20 | grep-no-wall-clock | 0 | no wall-clock APIs in the TS puzzle path or non-test puzzle Rust |
| 21 | grep-storage-only-in-hint-helper | 0 | the seam is only in `placements.ts`, no URL reads, and `localStorage` only in CanvasStage, first-hint-preference and playground-session |
| 22 | grep-no-innerhtml | 0 | puzzle UI, render and level metadata |
| 23 | grep-levels-out-of-scenes | 0 | none of the 6 ids appears in scene catalogs, frames, README plans, DemoNavigation or the scene Rust |
| 24 | grep-step-cap | 0 | the 4-step cap in session.ts, clock.ts and session.rs |
| 25 | grep-no-ts-geometry | 0 | no TS trig, and no sin/cos/sqrt in sat.rs, validate.rs or valve.rs |
| 26 | level-count | 0 | 6 levels (5–8), with the 5 required ids present |
| 27 | particle-budget | 0 | `step_zero_particles_fit_the_budget`: 1 passed |

Each filtered `cargo test` gate checks for a non-zero passed count: release-puzzle shows 257 and particle-budget shows 1. No failed attempt happened, so there are no `checks-11.attempt-<n>` files.

## Task 2: Step cost and phase decision record

### Step cost (D-23, informational, release-native, `target/phase40/step-costs.txt`)

```
STEPCOST first-pour particles 1989 ms_per_step 0.56
STEPCOST thick-syrup particles 1521 ms_per_step 0.44
STEPCOST jelly-pad particles 1365 ms_per_step 0.43
STEPCOST float-up particles 2394 ms_per_step 0.85
STEPCOST timed-gate particles 802 ms_per_step 0.25
STEPCOST grand-finale particles 1037 ms_per_step 0.36
```

These are one-off measurements and do not gate the phase. Every level is under the 3,000-particle step-0 budget, which gate 27 asserts.

### Final registry (6 levels, in order)

| # | Id | Idea | Caption | Target / full / Brim | Par | Step limit | Step-0 particles | ms/step |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | first-pour | plain water | "Water runs downhill. Tilt a plank to pour it into the cup." (the first-visit hint is "Tap the plank in the tray, then tap the canvas to place it.") | 540 / 900 / 810 | 1 | 720 | 1,989 | 0.56 |
| 2 | thick-syrup | viscous liquid needs a steep path | "Syrup is thick and slow. Give it a steep slide." | 610 / 944 / 705 | 2 | 720 | 1,521 | 0.44 |
| 3 | jelly-pad | uncounted elastic deflector | "Jelly never counts, but water slides across it. Drop the water onto the pad." | 420 / 700 / 630 | 1 | 720 | 1,365 | 0.43 |
| 4 | float-up | buoyant float plugs a drain | "The ball floats. Fill its well and it rises to plug the drain." | 233 / 389 / 272 | 1 | 720 | 2,394 | 0.85 |
| 5 | timed-gate | open time decides the outcome | "The valve opens on a timer. Catch the water first, then let the syrup out." | 502 / 800 / 582 | 1 | 720 | 802 | 0.25 |
| 6 | grand-finale | jelly pad + timed valve + syrup | "Finale: drop water on the jelly, catch it with the valve, then free the syrup." | 502 / 800 / 582 | 2 | 900 | 1,037 | 0.36 |

### Decisions

- **Powder (D-25): `sand-chute` dropped.** The 40-08 spike ran six water-versus-powder configurations: a coarse slit drain, a 15° ramp, viscous and jelly neighbours, plank scatter, and a slit-width sweep. One candidate came close: the r 0.08 slit drain, where powder drained 24–31% slower through 0.28–0.32 m slits. That effect fell to 15–22% at 0.24 and 0.40 m, and it reversed to +68% at 0.20 m. That leaves a band about two grid cells wide, which the ≥70% nudge rule cannot hold. No placement wins only for powder. Neither material piles up: both settled deposits are about 0.3 m high and 7.1–7.2 m wide. All the numbers are in 40-08-SUMMARY.
- **Buoyancy (D-09): `float-up` ships.** A circle float (r 0.35 m, density 0.4) rises 0.44 m and plugs the well's drain, so the pipe fills to the cup window. The control `float_up_needs_buoyancy` runs in every lane. With a density-2.0 ball, the recorded reference ends `Lost`: the ball rises less than 0.1 m and the peak is 160. With the real ball, the reference wins at step 550 (peak held 272, Brim 563). Nudges: 7/7 valid win. The level count is 6, inside 5–8.
- **Valve (D-26): the syrup-seal tent shipped.** The preferred diverter (drafts d1–d3) was monotone and could never lose by opening early. In the shipped design, the closed valve is a backstop for a water jet and also seals a tank of strength-1.0 syrup. Opening the valve frees the syrup. The 0.08 m clearance rule is unchanged, with no seat exemption. The clearance trickle (about 50 in timed-gate, 71 in grand-finale) does not decide the outcome. Open-tick sweeps (release):
  - timed-gate: ticks 1–2 lose, 3–21 win (reference tick 9) and 22–24 lose. The build area is the single valve slot, grid (2, 59, 4, 63).
  - grand-finale: ticks 1–9 lose, 10–27 win (reference tick 14) and 28–30 lose.
- **Wasm contract:** exact for all 14 fixtures, with no fallback. The viscous, elastic and circle-body solvers all match release-native.
- **Debug tolerance:** none. The debug-to-release spread is 0 for every new fixture, so no `debug_outcome_step_tolerance` was added.
- **Jelly pad (D-11): no change.** The pad stays level-authored and uncounted in both jelly-pad and grand-finale, so the placeable-tool fallback was not used.
- **Portrait frames:** first-pour keeps its Phase 38 portrait frame, which fills about 34% of the phone height, because D-09 locks its geometry. This is a known, recorded LEVEL-04 caveat. Every new level meets the 0.5 phone-height fill:

  | Level | Fill |
  | --- | --- |
  | thick-syrup | about 0.60 |
  | jelly-pad | about 0.66 |
  | float-up | about 0.66 |
  | timed-gate | above 0.5 (walls raised to 10.5 m in 40-07) |
  | grand-finale | about 0.60 |

- **Deferred:** the phone tray truncates the two tool names on grand-finale ("Pl… ×2", "Va… ×1"). This is still open in `deferred-items.md`. It is out of D-24 scope: the HUD overlap checks pass and the chips keep their accessible names.

## Verification

- Task 1 automated verify: `TASK1-VERIFY-OK`. That covers 27 `exit=0` lines with no other exit values, an empty fingerprint diff, 25 lines, and an empty engine diff.
- Task 2 automated verify: `VERIFY-OK`. There are 6 STEPCOST lines for the 6 registered levels.
- `git status --short` shows only the orchestrator's `.planning/config.json` (never staged). All evidence is under the gitignored `target/`.

## Deviations from Plan

None. The plan ran exactly as written. Two small additions to the gate script did not change any gate's intent:
- `no_match` replaces `! grep` inside multi-line gates, as in Phase 39, because errexit ignores `!` in the middle of a gate.
- `first-pour-fixtures-unchanged` also checks that both files exist.

## Known Stubs

None.

## Threat Flags

None. The threat register is covered by gates 1–4 and 16–17 (T-40-33), gate 21 (T-40-34), gate 22 (T-40-35), and gates 11, 12 and 20 (T-40-36). Gates passed on the first full run with no relabelled failures, so T-40-37 is covered too.

## Self-Check: PASSED

- FOUND: target/phase40/checks-11.sh, checks-11.results (27 exit=0), checks-11.log, after-final.txt (25 lines), step-costs.txt (6 lines)
- FOUND: target/dam-break-perf/2026-10-13T09-27-14Z/scene-spot.json
- FOUND: HEAD 7353eaded (no task commits; no fix was needed)
