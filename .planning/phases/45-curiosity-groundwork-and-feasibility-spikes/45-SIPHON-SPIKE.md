# Phase 45 Siphon Feasibility Spike (PREP-03, D-14..D-22)

## Status

Rules recorded before measuring; results pending. This file is committed together with the spike harness (commit A) before the first measured run, so the geometry, metrics, pass rule, sweeps and decision rule below cannot be tuned to a result.

## Geometry

Toy meter scale (D-15, D-19): particle radius r = 0.05 m (diameter 0.1), fill spacing 0.075 m (= 0.75 × 2r), every wall 0.3 m thick (3 diameters), tube inner width 0.5 m (5 diameters), gravity (0, -10). Axis-aligned quads on one static body via `attach_basin_fixture` (density 0, friction 0.2), overlapping at joints.

| Part | x range | y range |
| --- | --- | --- |
| Source floor | -4.6 .. -0.7 | -0.3 .. 0.0 |
| Source left wall | -4.6 .. -4.3 | -0.3 .. 3.3 |
| Inlet divider (tube inner wall; mouth gap below it) | -2.1 .. -1.8 | 0.5 .. 3.0 |
| Source right wall = inlet leg outer wall | -1.3 .. -1.0 | -0.3 .. 2.2 |
| Crest lower wall | -1.3 .. 1.3 | 1.9 .. 2.2 |
| Crest upper wall | -2.1 .. 2.1 | 2.7 .. 3.0 |
| Dest left wall = outlet leg outer wall | 1.0 .. 1.3 | -1.3 .. 2.2 |
| Outlet divider (mouth gap below it) | 1.8 .. 2.1 | -0.5 .. 3.0 |
| Dest right wall | 4.3 .. 4.6 | -1.3 .. 3.0 |
| Dest floor | 0.7 .. 4.6 | -1.3 .. -1.0 |

Interior regions (closed boxes):

| Region | x range | y range |
| --- | --- | --- |
| S (source open tank) | -4.3 .. -2.1 | 0.0 .. 3.3 |
| Inlet mouth (gap under the inlet divider) | -2.1 .. -1.8 | 0.0 .. 0.5 |
| Inlet leg | -1.8 .. -1.3 | 0.0 .. 2.2 |
| Crest channel | -1.8 .. 1.8 | 2.2 .. 2.7 |
| Outlet leg | 1.3 .. 1.8 | -1.0 .. 2.2 |
| Outlet mouth (gap under the outlet divider) | 1.8 .. 2.1 | -1.0 .. -0.5 |
| D (dest open tank) | 2.1 .. 4.3 | -1.0 .. 3.0 |

- Crest = 2.2 (inner lower wall height at the apex); inner upper = 2.7. Inlet mouth top = 0.5.
- Primed fill (D-19): S up to y 2.0, inlet mouth, inlet leg, crest channel, outlet leg and outlet mouth completely full, D up to y -0.4 (the outlet mouth top is -0.5, so it starts submerged). Points lie on one global 0.075 m grid with origin (-4.25, -0.95), kept when inside a fill box and at least r from every wall quad. On that grid the source top row is y 1.975 and the D top row is y -0.425.
- Particle count at step 0: **1,543** (707 in S), within the 3,000 budget (D-15); `maximum_count` 4,000. Checked by the non-ignored `primed_fill_fits_the_budget_and_starts_inside` test (no stepping).
- The open water reaches D only through the tube: S's walls rise to 3.0/3.3, more than 1 m above the source surface, so there is no overflow path (D-22 "never call plain overflow a siphon").
- Ideal equalized level with the tube still full: about y 0.8 in both tanks (2.2 m wide each), well below crest - 0.3 = 1.9.

Pre-measurement geometry adjustments from the plan's `<facts>` (made before commit A):

1. The two mouth gaps are added as interior regions and filled. The plan's region list omitted them, so a particle in the inlet mouth (x -2.1 .. -1.8) between expanded S (to -2.0) and the expanded inlet leg (from -1.9) would have been counted as a leak.
1. The fill uses one global grid rather than per-region grids, so the expected count is 1,543 rather than the plan's estimate of about 1,800.

## Metrics and pass rule

Sampling: every 15 steps = 0.25 s at 60 Hz, gravity (0, -10), 8 velocity / 3 position iterations, particle iterations 2, run 30 s = 1,800 steps. Native release build. No eviction pass; leaks and lost particles are measured instead.

- `source_surface` = free-surface height of the particles inside S: bin S by x into 0.2 m (2-diameter) bins, take each occupied bin's highest particle y, and report the nearest-rank median of those bin tops. An empty region reads as its floor. `dest_surface` = the same over D.
- `count_d` = particles inside D. `count_s` = particles inside S (informational only).
- `t*` = first sample **at or after 2 s** with `source_surface <= crest - 0.3` (= 1.9). If never, rule A fails.
- Rule A (D-21 transfer): over [t*, t* + 2 s], every 0.5 s window has positive Δcount_d and the total Δcount_d >= 10. If the run ends before t* + 2 s, rule A fails.
- Rule B (D-21 equal levels): `|source_surface - dest_surface| < 0.15` at 30 s, or at the first time after t* when count_d has not changed for 2 s (8 consecutive 0.25 s intervals).
- Leaks: particles outside the union of interior regions expanded by 0.1 m. Lost: live count below the start count. Pass needs 0 leaks and 0 lost at every sample (no leaks, no evictions).
- **PASS = rule A and rule B and no leaks and no lost.**
- Also recorded: particle count at step 0, mean native release ms/step over the run (hook + `world.step` timed together), count_d at 2, 5, 10, 20, 30 s, and Δcount_d over 0..2 s.

Pre-measurement metric deviations from the plan's `<facts>` (made before commit A, before any stepping):

1. **Surface estimator.** The plan defined `source_surface` as the nearest-rank p95 of y over all particles in S. For a uniform column of height H that reads about 0.95·H, not H: the primed 2.0 m source column would read 1.9 = crest - 0.3 at step 0, so t* would fire before anything moved. The median of per-bin tops reads the actual free surface (unit test `surface_of_a_flat_column_is_its_top_row`: a 27-row column with top row 2.0 reads 2.0) and still ignores isolated splashes.
1. **t* floor of 2 s.** The grid puts the source top row at 1.975, only 0.075 m (one row) above crest - 0.3, and LiquidFun water compresses noticeably at depth (Phase 40-09). Settling alone could cross 1.9 in the first second while the 3.2 m outlet leg drains its priming water into D, letting plain water pass rule A with no siphon. Free fall down the outlet leg takes about 0.7 s, so t* is not taken before 2 s (unit test `rule_a_ignores_a_priming_drain_in_the_first_two_seconds`). Rule B uses the same t*.

Unit tests (non-ignored, pure functions, Arrange/Act/Assert): rule A passes a steadily rising series, fails when transfer stops after t*, fails when the source never reaches crest - 0.3, ignores a priming drain in the first 2 s; rule B passes a final difference of 0.1 and fails 0.2.

## Candidates and sweeps

Measured in this order (D-20); the earliest passing configuration is picked. Each candidate is one `#[test] #[ignore = "D-20 siphon spike"]` running its whole fixed sweep.

1. **Plain water** (flags empty), default system (`ParticleSystemDef::default()` with r 0.05, density 1.0). Control, expected to fail. Label `c1-plain-water`.
1. **VISCOUS**, `with_viscous_strength` ∈ {0.5, 1.0, 1.5}. Labels `c2-viscous-0.5`, `c2-viscous-1.0`, `c2-viscous-1.5`.
1. **TENSILE | VISCOUS** with the liquid_timer system values. `scene/liquid_timer.rs` sets only the radius and maximum count, so this is the default system (viscous 0.25, surface tension pressure 0.2, normal 0.2) at the spike's r 0.05. Extra: transfer in the first 2 s (Δcount_d over 0..2 s) versus candidate 1, to report whether head difference still drives flow (TENSILE zeroes its own weight pressure). Label `c3-tensile-viscous`.
1. **Fake atmosphere** (stand-in): per-step hook; surface particles = non-wall particles inside S ∪ D with `weight < 0.5 × median weight of particles in S ∪ D` that step; each gets an extra acceleration α·|g| along the `world.gravity()` direction, α ∈ {0.5, 1.0, 2.0}, applied with `apply_particle_force` using F = m·a·particle_iterations, m = density·(0.75·2r)² = 0.005625 kg. Labels `c4-fake-atmosphere-0.5`, `-1.0`, `-2.0`.
1. **Weighted lids** (stand-in): one dynamic box lid per open tank, width = tank width - 0.1 = 2.1 m, height 0.2 m, centred on the tank, bottom resting on the initial surface (top fill row + r); lid density = β × 1.0 × 0.5 / 0.2 = 2.5·β kg/m², so its weight per meter equals β × the water weight per meter of a 0.5 m column, β ∈ {0.5, 1.0, 2.0}. Lids are frictionless and fixed-rotation (pistons), so a tilted lid cannot wedge against the tank walls. Labels `c5-weighted-lids-0.5`, `-1.0`, `-2.0`.
1. **Modeled tube pull** (stand-in, last resort): only while `source_surface` is above the inlet mouth top (0.5), non-wall particles inside the inlet leg get acceleration α·|g| upward (against gravity), inside the crest channel α·|g| toward +x, inside the outlet leg α·|g| downward (along gravity), α ∈ {0.25, 0.5, 1.0}. Labels `c6-tube-pull-0.25`, `-0.50`, `-1.00`.

At most 13 configurations × 1,800 steps. A configuration whose world construction or step fails is recorded as `RESULT siphon <label> error <message> verdict FAIL`; the sweep continues.

## Decision rule

D-22 (pre-recorded): **go** if a natural candidate (1-3) passes; **reshape** if only a stand-in (4-6) passes, and name that stand-in plainly for the Phase 50 caption (never "suction" or "atmospheric pressure" unless modeled); **drop** if none passes. Pick the earliest passing candidate in order. Never call plain overflow a siphon.

The siphon has no owner checkpoint of its own. No requirement or roadmap edits in this plan; Plan 08 shows the outcome to the owner together with the hourglass result.

## Run command

```
cargo test -p liquidfun-wasm --release --lib siphon_spike -- --ignored --nocapture --test-threads=1 2>&1 | tee target/phase45/siphon-spike.log
```

Output: `SPIKE siphon <label> t <s> src <m> dst <m> count_d <n> leaks <n> lost <n> count_s <n>` every 0.25 s; one `RESULT siphon <label> particles <n> ms_per_step <f> t_star <s|none> rule_a <pass|fail> rule_b <pass|fail> diff_end <m> leaks_max <n> lost_max <n> verdict <PASS|FAIL>` and one `DETAIL siphon <label> count_d_2s .. count_d_30s transfer_0_2s src_end dst_end` line per configuration. The log is gitignored.

## Results

(pending)

## Decision

(pending)

## Commits

(pending)
