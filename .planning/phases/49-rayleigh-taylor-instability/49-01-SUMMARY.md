---
phase: 49-rayleigh-taylor-instability
plan: "01"
subsystem: playground-scene-rust
tags: [scene, rayleigh-taylor, particle-force, gravity-frame, polynomial-seed, spike, d-06, d-06a, release-tests]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: "2026-10-19T02:09:12Z"
requires:
  - "48-01: particle_force::{GravityFrame, ForceScale, apply_group_acceleration}"
  - "45-09: docs/adding-a-scene.md per-scene checklist (R1-R14)"
  - "47-01 / 48-01: build/build_parts split, release-only spike gating, preset rejection, push_loop outlines"
provides:
  - "scene/rayleigh_taylor.rs: WeightRatio (exact tokens 1.2/1.5/2/3, default 2), Tuning + TUNING, build(), build_parts(), RayleighTaylorHooks (settle 36, ramp 24, extra weight along -GravityFrame::up through apply_group_acceleration)"
  - "scene/rayleigh_taylor/geometry.rs: 2.8 x 1.6 m closed tank (lid 0.1 m above the fill), polynomial wave()/interface_y(), lattice, split_layers, FRAME_* {-2.0, -0.6, 2.0, 2.2}"
  - "scene/rayleigh_taylor/test_support.rs: D-07 finger count, overturn, settled-interface depth line, fluid top, ripple depth, max speed, out-of-tank, rain share"
  - "scene/rayleigh_taylor/spike_tests.rs: 10 release-only D-06/D-06a regression guards (S0-S6, rain veto, settle frame)"
  - "SceneId::RayleighTaylor registered for parse/build/variant_index/session token (NOT in ALL_SCENE_IDS or SURVEY_SCENES; Plan 03 adds those with the web SCENE_IDS entry)"
affects: [49-02, 49-03, 49-04, 49-05]
tech-stack:
  added: []
  patterns:
    - "Measure from the settled state: depth lines and ripple growth are read against the interface at the end of the settle window, so a slumping column cannot satisfy a finger rule"
    - "Stiff-pressure settle: pressure strength 2.0 keeps a 1.5 m column within 0.08 m of its lattice height while the extra weight is off"
key-files:
  created:
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/geometry.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/tests.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/hook_tests.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/test_support.rs
    - crates/liquidfun-wasm/src/scene/rayleigh_taylor/spike_tests.rs
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene/particle_force.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/session/tests.rs
    - .planning/phases/49-rayleigh-taylor-instability/49-UI-SPEC.md
key-decisions:
  - "D-06a rerun: D-05 candidate 1 (plain WATER) passes; candidates 2 and 3 were not needed"
  - "TUNING: r 0.035, WATER, viscous 0.25, repulsive 1.0 (inert), pressure strength 2.0, damping 0.2, seed C (5 troughs @ 0.10 m + 9 @ 0.03 m), settle 36, ramp 24"
  - "Tank 2.8 x 1.6 m inner (fill 1.5 m, lid 1.6 m, interface 0.75 m), 1,484 particles; FRAME_* {-2.0, -0.6, 2.0, 2.2}"
  - "Seed C 36/24 chosen over seed A 36/24 (earlier S1, 84 vs 114) because A 36/24 fails S4 at ratio 3 (6.24 m/s > 6.0)"
  - "S6 reads each frame's ripple depth against its own interface (step 0 against the step-0 interface, step 60 against the settled interface) so the reference never cancels and a slump never counts as growth"
patterns-established:
  - "Rule-first spike with an amendment: when a sweep passes for the wrong reason, stop, amend the written rules, then rerun; never loosen a deadline"
requirements-completed: []
duration: "~45 min of execution (two executors, excluding the checkpoint wait)"
completed: 2026-10-18
---

# Phase 49 Plan 01: Rayleigh–Taylor Scene Core and D-06 Spike Summary

**A closed 2.8 × 1.6 m tank holds 1,484 water particles in two groups, with orange above blue. After a 0.6 s settle and a 0.4 s ramp, the orange group gets extra weight along live gravity through `apply_group_acceleration`. At ratio 2 and g 10, three separate fingers pass the settled 20% line by step 114 (1.9 s) and the layers overturn by step 156 (2.6 s), and the tuning passes every D-06a rule.**

## Performance

- **Duration:** about 45 min of execution, split between two executors around the D-06a decision checkpoint
- **Started:** 2026-10-18T20:26:16-05:00 (phase execution start)
- **Completed:** 2026-10-18T21:09:05-05:00 (tuning commit)
- **Tasks:** 3 of 3
- **Files modified:** 11 (6 created, 5 modified)

## Accomplishments

- Scene core, from Tasks 1 and 2 (the previous executor): a tank built from four overlapping static quads with a lid, a polynomial seed with no libm, a bit-reproducible two-group lattice, the exact-token `weight-ratio` parser, fail-closed hooks (`Recreated` for listed ratios, `Ok` for pointers), and the Rust registration. The `expect(dead_code)` attribute on `apply_group_acceleration` was removed (numstat `0 7`).
- D-06a rule update, written before any rerun measurement: S0 (no slump), a depth line and S6 measured from the settled interface, fingers counted only after the settle, and fewer than 3 fingers in the settle frame. Every deadline is unchanged (step 240, 600 and 1,200), along with the rain veto, S4, S5 and the g80 allowance.
- The rerun sweep (23 configurations, within the ~30-run time box) found a passing candidate-1 configuration. It is committed as `TUNING`, and its 10 rules remain as release-only regression guards (`cargo test -p liquidfun-wasm --release --lib rayleigh_taylor`: 62 passed, 0 ignored).

## Task Commits

1. **Tasks 1 and 2: tank geometry, seed, lattice, WeightRatio, hooks, build and Rust registration**: `fa11f9b00` (feat)
1. **Task 3 (part 1): D-06 metrics, metric unit tests and release spike rules**: `6b66b6c00` (test), from the previous executor, which stopped at the decision checkpoint
1. **D-06a amendment**: `5756a5c29` (docs, orchestrator)
1. **Task 3 (part 2): D-06a rules, fluid-filled tank, sweep and locked TUNING, plus UI-SPEC frame values**: `c22ee16dd` (test)

**Plan metadata:** the commit that follows this SUMMARY (docs)

## Sweep 1 (invalidated by the settle collapse)

The original rules were measured from the fixed `INTERFACE_MEAN_Y = 1.5` and a fixed depth line at 0.86 m. The setup was the 2.8 × 3.2 m tank with a 3.0 m fill, r 0.04 (2,300 particles), engine-default pressure strength 0.05 and damping 1.0, candidate 1 (WATER), viscous 0.25 and repulsive 1.0. I regenerated the table from the committed `6b66b6c00` code and the previous executor's scratch harness, then restored my work. Each cell gives the first step with at least 3 fingers / the first overturned step / the rain share at step 240. The table is invalid: the column collapses to about 1.5 m during the settle, so the fixed line is crossed before any extra weight applies. Several "finger" steps (24, 48, 54) fall inside or just after the settle window.

| Seed | Settle/ramp | 1.2: fingers / overturn | 2: fingers / overturn / rain | 3: fingers / overturn | S6 depth 0 -> 60 (gain) | max speed 2 / 3 | Verdict (old rules) |
|------|-------------|-------------------------|------------------------------|-----------------------|-------------------------|-----------------|---------------------|
| A | 30/18 | 54 / 372 | 60 / 150 / 0.000 | 84 / 102 | 0.020 -> 0.897 (0.877) | 5.00 / 5.85 | "PASS" |
| A | 18/12 | 54 / 318 | 48 / 126 / 0.001 | 48 / 114 | 0.020 -> 0.935 (0.915) | 8.02 / 9.19 | "PASS" |
| A | 36/24 | 54 / 348 | 54 / 174 / 0.001 | 84 / 108 | 0.020 -> 0.818 (0.798) | 6.88 / 8.03 | "PASS" |
| B | 30/18 | 24 / 366 | 24 / 150 / 0.000 | 24 / 102 | 0.020 -> 0.870 (0.850) | 5.00 / 5.86 | "PASS" |
| B | 18/12 | 24 / 366 | 24 / 126 / 0.001 | 24 / 108 | 0.020 -> 0.872 (0.852) | 8.03 / 8.33 | "PASS" |
| B | 36/24 | 24 / 342 | 24 / 150 / 0.000 | 24 / 108 | 0.020 -> 0.837 (0.817) | 6.42 / 6.33 | "PASS" |
| C | 30/18 | 54 / 396 | - / 156 / 0.000 | - / 108 | 0.020 -> 0.899 (0.879) | 5.00 / 5.80 | fail (S1) |
| C | 18/12 | 54 / 306 | 48 / 144 / 0.001 | 48 / 102 | 0.020 -> 0.799 (0.779) | 7.83 / 7.17 | "PASS" |
| C | 36/24 | 54 / 360 | 54 / 138 / 0.002 | 90 / 108 | 0.020 -> 0.789 (0.769) | 5.08 / 7.86 | "PASS" |

The previous executor also ran two probes. At pressure strength 0.25 the 3.0 m lattice rests at about 2.11 m. At r 0.06 with pressure 0.25 it rests at 2.43 m, but then shows only 2 fingers at ratio 2 and no overturn at 1.2 within 1,200 steps.

## Sweep 2 (D-06a rerun, rules fixed before measuring)

Tank: 2.8 × 1.6 m inner (fill 1.5 m, interface 0.75 m, lid 1.6 m). Depth line = settled interface − 0.32 m. S0 limit: 0.16 m. Candidate 1 (WATER), viscous 0.25 and repulsive 1.0 throughout. "-" means never within the window. Fingers count only after the settle, and the settle frame showed 0 fingers in every row. Rain share at step 240 was ≤ 0.023 in every full row, so the rain veto never decided a result.

Lever probes (seed A unless stated):

| # | r | Pressure | Damping | Seed, settle/ramp | n | S0 drop (m) | 1.2: fingers / overturn | 2: fingers / overturn | 3: fingers / overturn | S6 gain | Verdict |
|---|---|----------|---------|-------------------|---|-------------|-------------------------|-----------------------|-----------------------|---------|---------|
| 1 | 0.04 | 0.25 | 1.0 | A 30/18 | 1150 | 0.317 | - / - | - / 162 | 84 / 102 | 0.140 | fail S0, S1, S3 |
| 2 | 0.04 | 0.5 | 1.0 | A 30/18 | 1150 | 0.205 | - / - | 222 / 228 | 90 / 102 | 0.208 | fail S0, S3 |
| 3 | 0.04 | 0.5 | 0.2 | A 30/18 | 1150 | 0.212 | - / - | 114 / 156 | 60 / 96 | 0.180 | fail S0, S3 |
| 4 | 0.04 | 1.0 | 1.0 | A 30/18 | 1150 | 0.072 | - / - | - / 180 | 60 / 96 | 0.226 | fail S1, S3 |
| 5 | 0.04 | 1.0 | 0.2 | A 30/18 | 1150 | 0.079 | - / - | 108 / 144 | 60 / 90 | 0.225 | fail S3 |

The ratio 1.2 interface locks like a jammed lattice. The probes below ran ratio 1.2 for 2,400 steps and recorded ratio 1.5:

| # | r | Pressure | Damping | Seed, settle/ramp | n | S0 drop (m) | 1.2: fingers / overturn | 1.5: fingers / overturn |
|---|---|----------|---------|-------------------|---|-------------|-------------------------|-------------------------|
| 6 | 0.04 | 1.0 | 0.0 | A 30/18 | 1150 | 0.077 | - / - | - / 210 |
| 7 | 0.04 | 0.5 | 0.0 | A 30/18 | 1150 | 0.204 | - / - | - / 228 |
| 8 | 0.04 | 1.0 | 0.2 | B 30/18 | 1150 | 0.079 | - / - | - / 246 |
| 9 | 0.04 | 1.0 | 0.2 | C 30/18 | 1150 | 0.079 | - / - | - / 246 |
| 10 | 0.035 | 1.0 | 0.2 | A 30/18 | 1484 | 0.225 | 462 / 480 | - / 234 |
| 11 | 0.035 | 1.5 | 0.2 | A 30/18 | 1484 | 0.180 | 462 / 522 | 270 / 228 |
| 12 | 0.035 | 2.0 | 0.2 | A 30/18 | 1484 | 0.143 | - / 786 | 216 / 228 |
| 13 | 0.035 | 2.0 | 1.0 | A 30/18 | 1484 | 0.147 | - / - | 330 / 336 |
| 14 | 0.035 | 4.0 | 0.2 | A 30/18 | 1484 | 0.071 | - / - | - / 294 |

Grid at r 0.035, pressure 2.0, damping 0.2 (1,484 particles):

| # | Seed | Settle/ramp | S0 drop (m) | Settled interface | 1.2: overturn | 2: fingers / overturn / rain | 3: fingers / overturn | S6 depth 0 -> 60 (gain) | max speed 2 / 3 (600 steps) | Verdict |
|---|------|-------------|-------------|-------------------|---------------|------------------------------|-----------------------|-------------------------|-----------------------------|---------|
| 15 | A | 30/18 | 0.143 | 0.571 | 786 | 120 / 132 / 0.000 | 60 / 90 | 0.053 -> 0.246 (0.194) | 3.44 / 4.31 | PASS |
| 16 | A | 18/12 | 0.055 | 0.578 | - | 90 / 132 / 0.000 | 42 / 72 | 0.053 -> 0.231 (0.179) | 3.03 / 3.91 | fail S3 |
| 17 | A | 36/24 | 0.075 | 0.600 | 426 | 84 / 132 / 0.000 | 66 / 96 | 0.053 -> 0.160 (0.107) | 3.60 / 6.24 | PASS, then S4 fail |
| 18 | B | 30/18 | 0.143 | 0.575 | - | 108 / 138 / 0.000 | 60 / 84 | 0.053 -> 0.244 (0.192) | 3.90 / 4.75 | fail S3 |
| 19 | B | 18/12 | 0.055 | 0.585 | - | 90 / 132 / 0.000 | 42 / 72 | 0.053 -> 0.221 (0.169) | 3.04 / 4.54 | fail S3 |
| 20 | B | 36/24 | 0.075 | 0.606 | - | 78 / 126 / 0.000 | 66 / 90 | 0.053 -> 0.138 (0.085) | 3.80 / 4.25 | fail S3 |
| 21 | C | 30/18 | 0.143 | 0.571 | - | 108 / 150 / 0.000 | 60 / 90 | 0.053 -> 0.242 (0.189) | 2.75 / 4.59 | fail S3 |
| 22 | C | 18/12 | 0.055 | 0.580 | - | 102 / 156 / 0.000 | 42 / 78 | 0.053 -> 0.217 (0.165) | 3.09 / 4.25 | fail S3 |
| 23 | **C** | **36/24** | 0.075 | 0.602 | 378 | 114 / 156 / 0.000 | 66 / 96 | 0.053 -> 0.133 (0.080) | 3.82 / 4.29 | **PASS (chosen)** |

## Chosen TUNING and Why

D-05 order: candidate 1 (plain WATER) has passing configurations (15, 17 and 23), so candidates 2 and 3 were not run. All three have rain share 0.000, so the tie-break is the earlier S1 step: 17 (84), then 23 (114), then 15 (120). Configuration 17 fails S4 at ratio 3 (60 s max speed 6.24 m/s > 6.0), so the rule moves to configuration 23, which passes S4 and S5.

```rust
const TUNING: Tuning = Tuning {
    radius: 0.035, particle_flags: ParticleFlags::WATER,
    viscous_strength: 0.25, repulsive_strength: 1.0,
    pressure_strength: 2.0, damping: 0.2,
    modes: [SeedMode { troughs: 5, amplitude: 0.10 }, SeedMode { troughs: 9, amplitude: 0.03 }],
    settle_steps: 36, ramp_steps: 24,
};
```

Final measurements (the committed release guards, ratio 2 and g 10 unless stated):

| Rule | Measured | Limit |
|------|----------|-------|
| S0 fluid top drop over the settle | 1.4525 -> 1.3772 m (0.075 m) | ≤ 0.16 m |
| Settle frame fingers | 0 | < 3 |
| Settled interface / depth line | 0.602 m / 0.282 m | n/a |
| S1 first ≥ 3 fingers | step 114 (1.9 s) | ≤ 240 |
| S2 first overturn | step 156 (2.6 s) | ≤ 600 |
| S3 ratio 1.2 overturn | step 378 (6.3 s); never ≥ 3 fingers | ≤ 1,200 |
| Ratio 1.5 (record) | no ≥ 3 finger sample; overturn step 216 | record only |
| Ratio 3 (record) | fingers step 66; overturn step 96 | record only |
| Rain share at step 240 | 0.000 | ≤ 0.5 |
| S6 ripple depth | 0.0525 m at step 0 -> 0.1329 m at step 60 (gain 0.080 m) | ≥ 0.06 m |
| S4 ratio 2, 60 s | max speed early 3.82, late 2.36, overall 3.82 m/s; out of tank 0; every step Ok | ≤ 6.0, late ≤ early, 0 |
| S4 ratio 3, 60 s | max speed early 4.29, late 2.36, overall 4.29 m/s; out of tank 0; every step Ok | ≤ 6.0, late ≤ early, 0 |
| S5 two 600-step runs | bit-identical positions and velocities | identical |

At r 0.035, the critical velocity is 0.07 × 120 = 8.4 m/s, so the 6.0 m/s bound still sits below it (noted in the spike doc; the rule is unchanged).

Step-240 ASCII map at ratio 2 (40 columns across the inner tank, 24 rows from the floor to the lid; `#` = heavy majority, `.` = light majority). By step 240 the orange is mostly at the bottom with mixed pockets. The fingers are clearest at step 120, which is shown second.

```text
step 240 (fingers 1, top 1.284, interface 0.512)
|                                        |
|                                        |
|                                        |
|      . .                 .             |
|..... ....             .....            |
|..... ......        . . ...... ..    .  |
|....#.. #.......  .........#. . ....... |
|......... ...... ...........#........#..|
|.........#.....####.###.......#.#.#.... |
|#....... .#.#.####..#.....#..#.#.#.....#|
|.........#..##.####...#...#..#....#.##..|
|#...........####.####.####.####.#.#.#..#|
|.#....##...######.######.#######....###.|
|...#####.#..##...####.####..###..#...##.|
|#.###.....#...####.#.....##....#.#...#.#|
|##.....###.############...###.#.....##..|
|..#......###.#######..##.#.####.#.##.##.|
|...#####.#######..##..###.########....#.|
|###################.#.#####..##.#.##.##.|
|###################..########..#.##.#.#.|
|###.###############.#########.##.#.##..#|
|#.#.#.######.########################.##|
|.....###########.#######################|
|....############.##.##################.#|

step 120 (fingers 3, top 1.240, interface 0.462)
|                                        |
|                                        |
|                .                       |
|             . .. #                     |
|             ## .#..                    |
|           #...#.# ..     #             |
|          ###........       #           |
|        ###......... # ### ##  #   #### |
|######### ..........################### |
|###########........#####################|
|##############...#############.########.|
|##############..##.######.###.#########.|
|###############..#.###.#...#.#########..|
|##############....####.......#########..|
|######...######...####.....###########..|
|#####....#####....####....############..|
|#####.....#####..####.....############..|
|.#####..#######.####.....########..###..|
|.####....#####...###...#####......####..|
|...##.#...............######.....####...|
|..######..............######.....#####..|
|...######..............####.....#.####..|
|..#.#####..........................#....|
|........#...............................|
```

## Final Geometry, Frame and Wall Points (for Plan 03)

| Constant | Value |
|----------|-------|
| `TANK_HALF_WIDTH` | 1.4 (inner width 2.8 m, unchanged) |
| `FLOOR_TOP_Y` | 0.0 |
| `LID_BOTTOM_Y` | **1.6** (was 3.2) |
| `WALL_THICKNESS` | 0.3 (≥ 6 × r = 0.21) |
| `FILL_TOP_Y` | **1.5** (was 3.0) |
| `INTERFACE_MEAN_Y` | **0.75** (was 1.5) |
| `TANK_INNER_HEIGHT` (test only) | 1.6 |
| Outer box | x ±1.7, y −0.3 … 1.9 |
| Particles | 1,484 (53 columns × 28 rows at stride 0.0525) |
| `FRAME_MIN_X` / `FRAME_MIN_Y` / `FRAME_MAX_X` / `FRAME_MAX_Y` | −2.0 / −0.6 / 2.0 / **2.2** (was 3.8) |

- `RAYLEIGH_TAYLOR_VIEW_BOUNDS = { minX: -2.0, minY: -0.6, maxX: 2.0, maxY: 2.2 }`, also used for the portrait frame.
- `MIN_HEIGHT_FRACTION["rayleigh-taylor"]`: fitted height share = 2.8 / 4.0 × 390 / 844 ≈ 0.323, so use **0.3** (the Archimedes Screw value). Plan 03 still lists the nominal 0.48; 0.48 would fail.
- Portrait wall points (8): `(-1.7, -0.3)`, `(1.7, -0.3)`, `(-1.7, 1.9)`, `(1.7, 1.9)`, `(-1.4, 0.0)`, `(1.4, 0.0)`, `(-1.4, 1.6)`, `(1.4, 1.6)`. **Plan 03 still lists the nominal `3.5` and `3.2` values, so use these instead.**
- 49-UI-SPEC.md was updated in the same commit (`c22ee16dd`): the frame, `MIN_HEIGHT_FRACTION`, the wall points, the nominal geometry table, the UI-D94 portrait comment (the tank is now wide, not portrait-friendly) and the static preview composition, redrawn at 25 units/m for the 2.8 × 1.6 m tank.

## D-07 Interpretation and Decision Coverage

- D-07, as written before measuring: 17 bins of 0.16 m across the inner tank, with the two wall bins skipped. A bin is past the line when at least 4 heavy particles in it lie below the D-06a settled depth line, and a finger is a maximal run of adjacent past bins.
- Settled interface: the particle height below which as many particles lie as the light group holds. Fluid top: the 99th-percentile particle height. Both were defined before the rerun.
- D-01 one system, two groups, no `COLOR_MIXING`/`TENSILE`/`SOLID`. D-02 ramped extra weight `(ratio − 1)·|g|` along `-GravityFrame::up`, read every step. D-03 settle 0.6 s, ramp 0.4 s. D-04 polynomial seed with no RNG or time. D-05 candidate 1. D-06/D-06a all rules pass. D-08 exact tokens. D-10 hooks fail closed and ignore pointers. D-11 as amended by D-06a (2.8 m wide, wider than tall, 1,484 particles, within 10 m). D-12 colors unchanged (`#FB923C` / `#3B82F6`). D-09 and D-13 (web) belong to Plan 03.

## Deviations from Plan

### Amendment

**1. [D-06a, orchestrator-recorded decision] Rules amended after the settle-collapse checkpoint**
- **Found during:** Task 3 (previous executor)
- **Issue:** Sweep 1 passed only because the 3.0 m column collapsed during the settle.
- **Fix:** Added S0 and the settle-frame rule, measured the depth line and S6 from the settled interface, added pressure strength and damping to `Tuning`, and made the tank wider than tall so the fluid fills it. Deadlines, the rain veto, S4, S5 and g80 are unchanged.
- **Files modified:** rayleigh_taylor.rs, geometry.rs, test_support.rs, spike_tests.rs, tests.rs, 49-UI-SPEC.md
- **Commit:** `c22ee16dd`

### Auto-fixed Issues

**2. [Rule 1 - Bug] S6 literal reading would have counted the slump**
- **Found during:** Task 3, while writing the D-06a rules
- **Issue:** If both frames are measured against one fixed reference, the reference cancels, so the S6 gain equals the raw drop of the trough particle, and that drop includes the slump D-06a wants to exclude.
- **Fix:** Each frame's ripple depth is read against that frame's interface: the step-0 interface for step 0 and the settled interface for step 60. This was written into the spike doc before measuring.
- **Commit:** `c22ee16dd`

**3. [Rule 3 - Blocking] Geometry tests tied to the old tank**
- **Issue:** `tank_is_as_tall_as_wide_or_a_little_taller` (aspect 1.0 to 1.3) and `MIN_LATTICE_POINTS = 1_500` contradict the D-06a tank (aspect 0.57, 1,484 particles).
- **Fix:** Replaced the aspect test with `tank_inner_width_stays_in_the_toy_range` (2.4 to 3.2 m, D-11) and `lid_sits_just_above_the_fill` (headspace ≤ 10% of the inner height, D-06a). Lowered the lattice floor to 1,200 (the budget cap of 3,000 is unchanged). Moved the synthetic finger tests to `depth_line(INTERFACE_MEAN_Y)`.
- **Commit:** `c22ee16dd`

**4. [Structure] Extra `hook_tests.rs` file**
- The previous executor split the build, hook and session tests out of `tests.rs` into `rayleigh_taylor/hook_tests.rs` (435 lines) to stay under the 628-line limit. The file is not listed in the plan's `files_modified`; it lives in the same lib test binary, so no new test binary is added.
- **Commit:** `fa11f9b00`

## Known Stubs

None.

## Notes for Later Plans

- Ratio 1.2 is the fragile rule. Only 3 of the 9 grid configurations overturn at 1.2; elsewhere the lattice locks. The committed tuning overturns at step 378 and is deterministic. Any later change to radius, pressure, damping, seed or timing must rerun the release guards.
- The column compresses further once the extra weight is on (fluid top about 1.24 to 1.34 m during the run against a lid at 1.6 m), so the README clip will show a gap under the lid after the first second.
- At r 0.035 the pressure cap is about 0.25 × 8.4² ≈ 17.6, so at g 80 the column will squash hard (the D-18 g80 allowance applies). Plan 02's g2/g80 smokes and tilt test reuse `test_support` (`interface_height`, `depth_line`, `finger_count(.., depth_line)`).
- `crates/liquidfun`, `web/` and `docs/benchmarks/scene-fingerprints.json` are untouched.

## Self-Check: PASSED

- All six `rayleigh_taylor` source files exist; commits `fa11f9b00`, `6b66b6c00`, `5756a5c29` and `c22ee16dd` are in `git log`.
- Release: `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor` passed 62 with 0 ignored; `cargo test -p liquidfun-wasm --all-features --release` passed 880.
- Debug: `cargo test -p liquidfun-wasm --all-features` passed 798 with 93 ignored (release-only guards).
- `cargo fmt --all --check`, both clippy `-D warnings` commands, `cargo build --all-targets --all-features` and `bun scripts/bright-builds-check.ts all` (0 findings) pass.
- `cfg_attr(debug_assertions, ignore` count (10) equals the `#[test]` count (10). There is no `eprintln!` or `println!`, and the temporary sweep harness is deleted.
