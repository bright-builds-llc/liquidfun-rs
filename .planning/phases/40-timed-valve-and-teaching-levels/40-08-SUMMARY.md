---
phase: 40-timed-valve-and-teaching-levels
plan: "08"
subsystem: wasm-puzzle-levels
tags: [rust, wasm, puzzle, levels, powder, spike, D-25, LEVEL-01, LEVEL-02, LEVEL-04]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: "2026-10-13T07:46:14.685Z"
requires:
  - phase: 40-timed-valve-and-teaching-levels
    provides: "40-03 LiquidMaterial::Powder and create_puzzle_from_spec; 40-06/40-07 registry order first-pour, thick-syrup, jelly-pad, timed-gate"
provides:
  - "D-25 powder decision: sand-chute dropped, with the spike numbers recorded below"
  - "Spike source preserved in git history at e79597945 (crates/liquidfun-wasm/src/puzzle/powder_spike.rs), removed in 2879193cb"
affects:
  - "40-09 float-up inserts after jelly-pad; D-09 order no longer has a sand-chute slot"
  - "40-10 grand-finale should not rely on powder behaving differently from water"
  - "40-11 level-count check: registry is 4 levels now, 6 if float-up and grand-finale ship (within 5-8)"
tech-stack:
  added: []
  patterns:
    - "Material spike: the same leaked test-only spec built twice (water, then powder) with snapshot metrics at fixed seconds and a relative delta per metric"
key-files:
  created: []
  modified: []
key-decisions:
  - "sand-chute dropped under D-25: no spike metric was both at least 25% distinct from same-geometry water and outcome-changing"
  - "The only metric past 25% (coarse r 0.08 grains through a 0.28-0.32 m slit, about 30% slower drain) reverses at 0.20 m, where powder drains 68% more than water, and fades to -1% at 0.56 m. So 'coarse sand jams in narrow gaps' would be false"
  - "Spike code was committed (e79597945) and then removed (2879193cb). It is not a useful regression, and keeping the source in history makes the numbers reproducible"
metrics:
  duration: 12min
  completed: 2026-10-13
  tasks: 2
  files: 2
---

# Phase 40 Plan 08: Sand Chute Powder Spike Summary

**Decision: `sand-chute` dropped under D-25.** Powder was compared with water in the same geometry and at the same radius in six spike configurations. No player-visible metric was both at least 25% distinct and able to change which placements win. No level code, fixture, catalog entry or caption was added. The registry is still `first-pour`, `thick-syrup`, `jelly-pad`, `timed-gate` in D-09 order.

The spike numbers below are authoring measurements (D-18). They are the reason for the decision, not evidence for any shipped claim.

## Task 1: Time-boxed powder spike (commit e79597945)

The spike lived in `crates/liquidfun-wasm/src/puzzle/powder_spike.rs` as six `#[ignore = "D-25 powder spike"]` tests in the existing lib test binary:

- Each test builds a test-only `LevelSpec` twice through `create_puzzle_from_spec`, once with a `Water` block and once with a `Powder` block.
- The geometry, radius and fill spacing (1.5 × radius) are identical in the two runs.
- Default powder strength. The research already found that 1.0 changes nothing.
- Metrics are read from particle positions at whole seconds (60 steps per second, release).

Log: `target/phase40/probes/sand-chute-spike.log` (116 lines, kept out of git).

### (a) Coarse grains

Slit drain: a 3.1 m by 2.2 m reservoir above a centred slit 1.75 diameters wide. "Drained" counts particles below the reservoir floor.

| Radius, slit | Metric | 2 s | 4 s | 8 s |
| --- | --- | --- | --- | --- |
| 0.06, 0.21 m (805 particles) | drained, water / powder | 92 / 90 (-2%) | 180 / 165 (-8%) | 328 / 276 (-16%) |
| 0.06, 0.21 m | deposit p95 height | 2.49 / 2.50 | 1.88 / 1.91 | 0.30 / 0.28 (-7%) |
| 0.06, 0.21 m | deposit p5-p95 width | 6.86 / 7.50 | 7.50 / 7.53 | 7.12 / 7.20 (+1%) |
| 0.08, 0.28 m (468 particles) | drained | 54 / 46 (-15%) | 106 / 75 (-29%) | 189 / 130 (-31%) |
| 0.08, 0.28 m | deposit p95 height | 1.62 / 2.64 (+63%, the falling stream) | 1.85 / 1.88 | 0.34 / 0.30 (-13%) |
| 0.08, 0.28 m | deposit p5-p95 width | 6.57 / 7.34 | 7.40 / 7.28 | 7.07 / 7.13 (+1%) |

Neither radius piles up. At 8 s both deposits lie flat across the box, about 0.3 m deep and 7.1-7.2 m wide.

15° ramp: a 1.2 m by 1.0 m block released at the top of a 6 m ramp, followed by a run-out.

| Radius | Metric | 1 s | 2 s | 4 s |
| --- | --- | --- | --- | --- |
| 0.06 (143) | mean x, water / powder | -1.10 / -1.20 | 2.95 / 2.73 (-7%) | 7.10 / 7.16 (+1%) |
| 0.06 | front p95 x | 1.73 / 1.49 (-14%) | 7.88 / 7.90 | 7.91 / 7.91 |
| 0.08 (80) | mean x | -1.05 / -1.19 | 2.97 / 2.70 (-9%) | 7.08 / 7.16 (+1%) |
| 0.08 | front p95 x | 1.77 / 1.39 (-22%) | 7.86 / 7.65 (-3%) | 7.86 / 7.88 |
| 0.08 | tail p5 x | -2.89 / -2.85 | -1.60 / -1.23 (+23%) | 6.16 / 6.27 (+2%) |

### (b) Powder beside a neighbour

Viscous neighbour (strength 1.5, r 0.04, 1,200 particles): a viscous block with the free block beside it, collapsing right across a flat floor.

| Metric | 1 s | 2 s | 4 s |
| --- | --- | --- | --- |
| free mean x, water / powder | 0.10 / 0.02 (0.09 m apart, near-zero base) | 2.08 / 2.01 (-3%) | 3.36 / 3.27 (-3%) |
| free front p95 x | 2.82 / 2.39 (-15%) | 4.86 / 4.86 | 4.84 / 4.85 |
| free mean y | 0.23 / 0.27 | 0.21 / 0.19 | 0.21 / 0.20 |
| viscous mean x | -2.21 / -2.21 | -1.94 / -1.95 | -1.51 / -1.56 (-3%) |
| free particles behind the viscous front | 0 / 1 | 6 / 7 | 10 / 11 |

The powder neither separates from, nor rides on, nor flows differently beside the syrup.

Jelly neighbour (r 0.04, 499 particles): a block dropped from 1.6 m onto a solid elastic slab in a shallow cradle.

| Metric | 1 s | 2 s | 4 s |
| --- | --- | --- | --- |
| free particles in the pad box, water / powder | 21 / 11 | 77 / 69 (-10%) | 92 / 77 (-16%) |
| free p5-p95 width | 2.69 / 2.48 (-8%) | 7.49 / 7.55 | 7.28 / 7.36 |
| free p99 height | 1.21 / 1.35 | 0.55 / 0.60 | 0.40 / 0.47 |
| pad mean y | 0.20 / 0.22 | 0.14 / 0.15 | 0.15 / 0.14 |

### (c) Scatter off a -30° plank

A 1.2 m square block dropped from 1 m onto a static plank.

| Radius | Metric | 1 s | 2 s |
| --- | --- | --- | --- |
| 0.04 (420) | x std dev, water / powder | 1.28 / 1.20 (-6%) | 1.87 / 1.63 (-13%) |
| 0.04 | p5-p95 width | 3.96 / 3.86 (-3%) | 4.11 / 3.73 (-9%) |
| 0.04 | particles above the plank top | 18 / 29 | 8 / 21 |
| 0.08 (110) | x std dev | 1.46 / 1.34 (-8%) | 2.05 / 1.61 (-22%) |
| 0.08 | p5-p95 width | 4.61 / 4.04 (-13%) | 7.15 / 3.72 (-48%) |
| 0.08 | particles above the plank top | 8 / 6 | 2 / 1 |

### (d) Robustness of the one candidate: slit-width sweep at r 0.08

Drained count, water / powder (468 particles each).

| Slit | 4 s | 8 s | 12 s |
| --- | --- | --- | --- |
| 0.20 m (1.25 d) | 32 / 34 (+6%) | 47 / 60 (+28%) | 50 / 84 (**+68%**) |
| 0.24 m (1.5 d) | 63 / 59 (-6%) | 115 / 99 (-14%) | 157 / 122 (-22%) |
| 0.28 m (1.75 d) | 106 / 75 (-29%) | 189 / 130 (-31%) | 254 / 176 (-31%) |
| 0.32 m (2 d) | 141 / 104 (-26%) | 246 / 187 (-24%) | 317 / 242 (-24%) |
| 0.40 m (2.5 d) | 216 / 169 (-22%) | 353 / 283 (-20%) | 420 / 358 (-15%) |
| 0.56 m (3.5 d) | 332 / 279 (-16%) | 449 / 422 (-6%) | 463 / 459 (-1%) |
| 0.28 m shifted +0.05 m | 103 / 81 (-21%) | 184 / 131 (-29%) | 240 / 175 (-27%) |

### Why nothing qualifies

The "honestly distinct" test has two parts. A metric must differ from water by at least 25%, and that difference must change which placements win. Several rows pass the first part. None passes both:

- **Coarse slit drain (r 0.08), the only real candidate.**
  - Powder drains 24-31% slower only through 0.28-0.32 m slits. The effect is about 15-22% at 0.24 m and 0.40 m.
  - At 0.20 m the sign flips: water jams harder, and powder drains 68% more.
  - A level built on it would depend on a band about two grid cells wide (0.05 m per cell), and a one-cell nudge leaves that band. The ≥70% nudge rule could not hold.
  - The lesson would not be real either. A wider path speeds water and powder alike, so no placement wins only for powder, and a water-style solution loses only through step-limit tuning.
  - The plan's example caption ("Coarse sand jams in narrow gaps") is false: the narrowest slit jams water, not sand.
- **Small-count or near-zero-base rows.** These cross 25% in relative terms, but a player could not see them or use them:
  - jelly pad-box count at 1 s: 21 vs 11 of about 290 free particles;
  - particles above the plank at r 0.04: 18 vs 29 of 420;
  - plank width at r 0.08 and 2 s: a handful of water particles at the far wall, out of 110;
  - viscous free mean x at 1 s: 0.09 m apart, against a mean near 0;
  - deposit p95 height at 2 s: the falling stream, not a pile. At 8 s both are flat at about 0.3 m.

  The steadier metrics for the same runs, such as x std dev, front x and settled widths, stay at or below 22%.
- **No piling anywhere.** Settled deposits are 0.28-0.34 m high and 7.1-7.2 m wide for both materials, which agrees with research Pitfall 2.

## Task 2: Drop path (commit 2879193cb)

- Removed `powder_spike.rs` and its `#[cfg(test)] mod powder_spike;` line. `crates/liquidfun-wasm/src/puzzle.rs` is byte-identical to its state before the plan.
- `grep -rn "SandChute\|sand-chute\|sand_chute" crates/liquidfun-wasm/src web/src web/tests` prints nothing (exit 1).
- No `LevelId` variant, spec file, fixture, `FIXTURES` entry, `PUZZLE_IDS` entry, title, caption, bounds or `LEVEL_GEOMETRY` entry was created.
- Level count: 4 are registered now. The minimum set after drops (D-25) is first-pour, viscous, jelly pad, valve and finale. Plans 09 (float-up) and 10 (grand-finale) take the count to 5-6, which is within the 5-8 range.

## Verification

- `cargo fmt --all --check` passes, and so does `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`.
- Workspace clippy and `cargo build --all-targets --all-features` were run at the Task 1 commit.
- With the spike present, `cargo test -p liquidfun-wasm` (debug) passed: 559 passed, 22 ignored, 6 of them spike tests.
- After the drop:
  - `cargo test -p liquidfun-wasm --lib puzzle::` (debug): 238 passed, 15 ignored.
  - `cargo test -p liquidfun-wasm --release --lib puzzle::`: 246 passed, 7 ignored.
- `just puzzle-replay-wasm`: 10/10 fixtures pass. The timed-gate values are unchanged (solution won at step 429, peak held 582).
- Web: `bunx tsc --noEmit` is clean, and `bun run test:unit` passes 97 files and 1,375 tests. The puzzle e2e was not run, because no web file changed.
- `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `crates/liquidfun` is untouched, and no fixture, scene, builder or session code changed, so the 25 scene fingerprints are unaffected.

## Deviations from Plan

### Design decisions within plan discretion

**1. A sixth configuration tested the robustness of the one candidate.** The plan allowed up to six configurations. Five covered candidates a-c, and the sixth was a slit-width sweep with a shifted-slit variant. Without it, the r 0.08 slit result (-31%) would have looked like a usable effect. With it, the effect is a narrow band that reverses sign.

**2. The spike code was committed and then removed in its own commit.** Plan Task 2 expects a `chore(40-08): drop sand-chute after the D-25 spike` commit for removing spike tests. Committing the spike first (e79597945) keeps its source reproducible from history and gives each task an atomic commit.

### Auto-fixed Issues

None. A clippy cast-lint failure in the spike module was fixed before its first commit with a module-level `allow` and a stated reason. That module has since been removed.

**Process notes:** LEVEL-01, LEVEL-02 and LEVEL-04 stay Pending in REQUIREMENTS.md, as instructed.

## Known Stubs

None.

## Threat Flags

None. T-40-27 is mitigated: no caption was shipped. T-40-28 does not apply, because no level was added.

## Self-Check: PASSED

- FOUND: target/phase40/probes/sand-chute-spike.log (116 spike lines plus a DECISION line)
- FOUND: commits e79597945 and 2879193cb
- CONFIRMED ABSENT: crates/liquidfun-wasm/src/puzzle/sand_chute.rs and every SandChute, sand-chute or sand_chute reference in crates/liquidfun-wasm/src, web/src and web/tests
