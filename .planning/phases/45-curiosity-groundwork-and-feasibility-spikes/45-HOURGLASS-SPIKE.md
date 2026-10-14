# Phase 45 Sand Hourglass Feasibility Spike (PREP-03, D-14..D-18, D-23..D-27)

## Status

Complete; owner confirmation pending (Plan 08). The rules below were recorded before measuring and committed together with the spike harness in commit A (`09ab08395`) before the first measured run, so the geometry, metrics, pass rule, sweeps and decision rule could not be tuned to a result. Results, flip cost and the decision by the pre-recorded rule were added after the single measured run; no rule, geometry or sweep changed afterwards.

## Geometry

Toy meter scale (D-15, D-23): particle radius r = 0.05 m (diameter 0.1), fill spacing 0.075 m (= 0.75 × 2r), walls 0.3 m thick (3 diameters), gravity (0, -10). Every wall is a convex quad on one static body via `attach_basin_fixture` (density 0, friction 0.2). Walls never move. The glass is point-symmetric about c = (0, 0): the lower half is written out and the upper half is the point reflection q → −q of every lower quad. A point reflection is a half turn, so counter-clockwise quads stay counter-clockwise.

Lower half (as built, counter-clockwise):

| Part | Vertices |
| --- | --- |
| Floor | box x -1.8..1.8, y -3.7..-3.4 |
| Left wall (overlaps the funnel) | box x -1.8..-1.5, y -3.7..-1.3 |
| Right wall | box x 1.5..1.8, y -3.7..-1.3 |
| Left funnel (lower-bulb ceiling) | (-1.5, -1.6), (-0.25, 0.0), (-0.4865, 0.1847), (-1.7365, -1.4153) |
| Right funnel (re-ordered counter-clockwise) | (1.7365, -1.4153), (0.4865, 0.1847), (0.25, 0.0), (1.5, -1.6) |

Upper half (point reflection, as built):

| Part | Vertices |
| --- | --- |
| Ceiling | box x -1.8..1.8, y 3.4..3.7 |
| Right wall | box x 1.5..1.8, y 1.3..3.7 |
| Left wall | box x -1.8..-1.5, y 1.3..3.7 |
| Right funnel (upper-bulb floor) | (1.5, 1.6), (0.25, 0.0), (0.4865, -0.1847), (1.7365, 1.4153) |
| Left funnel (upper-bulb floor) | (-1.7365, 1.4153), (-0.4865, -0.1847), (-0.25, 0.0), (-1.5, 1.6) |

- Funnels slope about 52° from horizontal (inner face from (±0.25, 0) to (±1.5, ±1.6)). The neck is the gap between (-0.25, 0) and (0.25, 0): 0.5 m = 5 diameters (D-23). Lower bulb interior: box x -1.5..1.5, y -3.4..-1.6, plus the funnel region up to the neck.
- Sand fill: one 0.075 m grid symmetric about x = 0 (x = (i + 0.5)·0.075, y = (j + 0.5)·0.075), keeping points inside the upper fill region (the funnel region |x| < 0.25 + 1.25·y/1.6 for 0 < y < 1.6, plus the box |x| < 1.5 for 1.6 ≤ y ≤ 1.8) and at least r from every wall quad. **564 particles**, all above the neck at step 0, within the 3,000 budget (D-15). `maximum_count` 4,000. Checked by the non-ignored `sand_fill_fits_the_budget_and_starts_in_the_upper_bulb` and `reflected_walls_build_a_world` tests (no stepping).
- System: `ParticleSystemDef::default()` with density 1.0 and radius 0.05 (default powder strength). Step 1/60 s, 8 velocity / 3 position iterations, particle iterations 2.

Pre-measurement geometry note: the plan estimated about 600 particles; the r clearance from the slanted funnels gives 564. The right funnel's vertex list is re-ordered counter-clockwise (same quad).

## Metrics and pass rule

Fixed before measuring. Sample every 15 steps = 0.25 s; gravity (0, -10); run 30 s = 1,800 steps; then flip and run 10 s more.

- `upper` = count(y > 0), `lower` = count(y < 0); flow = Δlower per sample.
- Jam (D-25): `upper > 0` and `lower` unchanged for more than 60 consecutive steps (checked every step, any time before the flip). Implemented as a per-step tracker: a step counts as stalled when `lower` equals the previous step's value and `upper > 0`; otherwise the count resets. A jam is recorded at the first step where the count exceeds 60.
- Deposit = lower-bulb particles with y < -0.2 and speed < 0.5 m/s (excludes the falling stream).
- Bins: 15 bins of width 0.2 m across x -1.5..1.5; a bin is occupied with at least 3 deposit particles; bin top = max y of its deposit particles (the x of that top particle is kept as well).
- `edge` = mean of the tops of the leftmost and rightmost occupied bins; `base_width` = right edge of the rightmost occupied bin − left edge of the leftmost occupied bin; `peak` = nearest-rank p95 of the occupied bin tops.
- Nearest-rank percentile p of N values = the value at 1-based rank ⌈p·N⌉ of the sorted values (so for 19 or fewer occupied bins the p95 bin top is the highest bin top; the median of 21 samples is the 11th).
- `slope_deg = atan((peak − edge) / half_run)` in degrees, where `half_run` = (x of the rightmost occupied bin's top particle − x of the leftmost occupied bin's top particle) / 2 (edge-relative reading of D-25's "p95 height against base width"; research Pitfall 3 / A4; see the correction below). No occupied bin, or `half_run` ≤ 0 (one occupied bin), reads 0° (no pile).
- Informational, also logged: `literal_d25_deg = atan((p95 of deposit particle y − (−3.4)) / (base_width / 2))` and `side_fit_deg` = mean |atan| of least-squares slopes of bin tops (at bin centres) left and right of the highest bin (each side includes the highest bin; a side with fewer than 2 bins is skipped; no side reads 0°).
- **Pass (D-25):** over the last 5 s (25-30 s, 21 samples) the median `m` of `slope_deg` is ≥ 20°, every sample lies in [m − 5°, m + 5°] (±5°), no jam, AND `m` exceeds the water control's median by at least 15° (guard against a confinement artifact). The water control's margin is 0° by construction, so the water control cannot pass.
- Literal-D-25 verdict (informational, research A4): the same rule applied to `literal_d25_deg` (median ≥ 20°, all within ±5°, no jam), without the water guard. The decision uses only the edge-relative verdict.
- Recorded per configuration: particles, mean native release ms/step (hook + `world.step` timed together), slope samples, final `upper`/`lower`, jam yes/no with time, frozen count and freezes per step (candidate 2), max speed of frozen grains over every step (candidate 2, assumption A3).

Pre-measurement metric corrections from the plan's `<facts>` (made before commit A, before any stepping):

1. **Horizontal run.** The plan's `atan((peak − edge) / (base_width / 2))` reads a true 20° pile low, because the bin tops are sampled near each bin's inner side while `base_width` runs to the outer edges of the outermost bins. For a 20° particle triangle spanning the 3 m bulb (columns on the fill grid, each topped exactly on the 20° surface), the plan formula gives 17.2° (and 16.2° if p95 picked the second-highest bin), so the plan's own required unit test (20° ± 1°) could not pass. The run is now measured between the x positions of the top particles of the two outermost occupied bins; the same triangle reads 19.5° (unit test `symmetric_twenty_degree_triangle_gives_twenty_degrees`), and `plan_formula_reads_the_twenty_degree_triangle_low` keeps the plan formula's reading below 19° as a record. `base_width` is unchanged and still drives `literal_d25_deg`. Both corrections make passing no easier for a flat fill: a flat deposit has `peak − edge` ≈ 0 whatever the run.
1. **Nearest rank.** "Nearest-rank p95" uses the textbook rank ⌈0.95·N⌉.
1. **Support median.** For the support rule, the median weight is taken over the non-`WALL` lower-bulb particles (frozen grains excluded), so frozen walls do not shift the threshold.
1. **Grain key.** `ParticleId` has no `Ord`, so the per-grain slow-step counters live in a `BTreeMap` keyed by a deterministic `u64` hash of the particle id (`DefaultHasher::new()`), which is also the tie-break for "lowest y first, ties by particle id". A key collision is a harness error.

Unit tests (non-ignored, pure functions, Arrange/Act/Assert): flat bin tops give 0°; a symmetric 20° triangle gives 20° ± 1°; the plan formula reads that triangle below 19°; a stalled `lower` series with `upper > 0` for 61 steps is a jam and for 60 steps is not; an empty upper bulb never jams; a steady 24° window passes against 2° water and fails against 12° water; a reading 5.5° from the median fails.

## Flip measurement

D-26, after each configuration's 30 s run. Never flip by inverting gravity.

1. **Batched** (timed with `Instant`, whole sequence): read ids, positions and velocities of ALL particles including frozen `WALL` grains; `mark_particle_for_destruction` each, ONE `compact_pending_particles`; ONE `create_particle_group` at −p with the configuration's sand flags (this releases frozen grains); then `set_particle_velocity(member_i, −v_i)` per member. Candidate 3 rebuilds one `RIGID` group per old clump (grouped by old group id in first-appearance order); that variant is reported with its group count. After the timed section (untimed) the harness checks that the particle count is unchanged and that member positions equal −p in order (A1, tolerance 1e-6). If the order differs, `order_ok false` is recorded and velocities are re-assigned by nearest position.
1. Run 10 s after the flip with the same rule hook; record `lower` at 10 s post-flip and whether ≥ 90% of the sand passed the neck again (informational for SAND-02). Progress is printed once per second as `POST` lines.
1. **Per-particle comparison** (timed): `set_particle_position(id, −p)` for every non-`WALL` particle, then `set_particle_velocity(id, −v)`, on the state at the end of the 10 s rerun. Cost only. If a `WALL` particle exists then, one `set_particle_position` on it is tried untimed and reported as `wall_edit ok|error` (`none` when there is no `WALL` particle).

Candidate 2 ordering: the release proof below runs before the flip, so the batched flip normally sees 0 frozen grains for candidate 2 (`frozen_rebuilt` is logged); any `WALL` grain present is rebuilt with the sand flags by the same code path.

## Candidates and sweeps

Measured in this order (D-24); the earliest passing configuration is the chosen sand rule. Each control and candidate is one `#[test] #[ignore = "D-24 hourglass spike"]` running its whole fixed sweep; with `--test-threads=1` the test names (`c0_control_a_water`, `c0_control_b_powder`, `c1_stick_slip`, `c2_freeze_to_wall`, `c3_rigid_clumps`) run in this order.

- Controls: (W) water, flags empty, label `w-water`; (P) `POWDER`, default powder strength, label `p-powder`. The water median over 25-30 s is computed once and shared by every later configuration's water guard.
1. **Stick-slip** (`POWDER` base, per-step hook before `world.step`): lower-bulb particles (y < 0) with speed < v_slip and weight ≥ 0.5 × median weight of lower-bulb particles that step get `set_particle_velocity(v · (1 − k))`. Sweep (v_slip, k) ∈ {0.2, 0.5} × {0.5, 1.0}. Labels `c1-stick-slip-v0.2-k0.5`, `c1-stick-slip-v0.2-k1.0`, `c1-stick-slip-v0.5-k0.5`, `c1-stick-slip-v0.5-k1.0`.
1. **Freeze to `WALL`** (`POWDER` base, per-step hook): a non-`WALL` grain with y < −0.2, speed < 0.05 for 30 consecutive steps and the same support rule is a candidate; each step freeze at most K candidates (lowest y first, ties by grain key), K ∈ {8, 32}: mark them, ONE compaction, re-create each as `WALL` at the same position and color with zero velocity. No force call is made anywhere in this spike, so frozen grains never reach a force call. Release proof: after the 30 s run, change gravity to (0, −10.5), release every frozen grain (mark, compact once, re-create as `POWDER` with zero velocity), assert the frozen count is 0, restore (0, −10) (also a release point), then flip. Never permanent (D-24). Labels `c2-freeze-k8`, `c2-freeze-k32`.
1. **Rigid clumps:** the same fill split into 2 × 2 clumps by grid index (column div 2, row div 2), each its own group with `ParticleGroupFlags::RIGID`, particle flags empty. Where the slanted funnel clips a block, the remaining 1-3 points form a smaller rigid group, so the particle count stays 564 like every other configuration; the `BUILD` line reports full and partial clump counts. One configuration, label `c3-rigid-clumps`.

Rigid circle-body grains are out (48-circle frame cap, D-24). A configuration whose world construction, hook, step or flip fails is recorded as `RESULT hourglass <label> error <message> verdict FAIL`; the sweep continues.

## D-18 trigger

If candidate 2's re-created `WALL` grains do not stay still (max frozen speed > 1e-3 m/s over the run) or the destroy-and-recreate path errors, add `pub fn set_particle_flags(&mut self, particle: ParticleId, flags: ParticleFlags) -> Result<(), ParticleEditError>` to `World` beside `set_particle_velocity`, routed through `set_particle_flags_internal` (widened to `pub(crate)`), matching upstream `b2ParticleSystem::SetParticleFlags` semantics, with focused AAA tests in `crates/liquidfun/src/world/particle_object/particle_flags_tests.rs`; rerun candidate 2 with flag toggling as configuration `2b`; then run `just scene-fingerprint-gate`, which must report 25/25. If the trigger does not fire, `crates/liquidfun` stays untouched.

## Decision rule

D-27 (pre-recorded owner rule of 2026-10-14), verbatim: "Decision: **go** if any candidate passes, **drop** otherwise. By the owner's 2026-10-14 decision, a failed spike returns SAND-01 and SAND-02 to a seed (record that seed update in this phase if the outcome is drop). Owner confirmation: after the spike, the orchestrator presents the measured result and asks the owner to confirm. If the owner is unavailable, the pre-recorded owner rule is applied and recorded as such, with the confirmation left as an open human item — never record an invented approval."

Applied here: go if any candidate configuration (1-3) passes the edge-relative rule, naming the earliest passing one as the chosen sand rule; drop otherwise. The controls are not candidates. Owner confirmation happens in Plan 08; never record an invented approval. No REQUIREMENTS, ROADMAP, PROJECT or seed edits in this plan.

## Run command

```
cargo test -p liquidfun-wasm --release --lib hourglass_spike -- --ignored --nocapture --test-threads=1 2>&1 | tee target/phase45/hourglass-spike.log
```

Output lines (the log is gitignored):

- `BUILD hourglass <label> particles <n> groups <n> full_clumps <n> partial_clumps <n>` once per world.
- `SPIKE hourglass <label> t <s> upper <n> lower <n> slope <deg> literal <deg> side <deg> frozen <n>` every 0.25 s.
- `RESULT hourglass <label> particles <n> ms_per_step <f> median_slope <deg> min <deg> max <deg> jam <no|yes@t> water_margin <deg> verdict <PASS|FAIL>` and `DETAIL hourglass <label> literal_median .. literal_verdict .. side_median .. upper_end .. lower_end .. frozen_end .. freezes_total .. freezes_per_step .. freezes_max_step .. max_frozen_speed ..` per configuration.
- `RELEASE hourglass <label> released <n> frozen_after 0 restore_released <n>` (candidate 2).
- `POST hourglass <label> t <s> upper <n> lower <n> frozen <n>` once per second after the flip.
- `FLIP hourglass <label> batched_ms <f> per_particle_ms <f> count_ok <bool> order_ok <bool> rerun_lower_10s <n>/<total> rerun_90pct <bool> groups <n> frozen_rebuilt <n> moved <n> wall_edit <none|ok|error>`.

## Results

One release run on 2026-10-14 from commit A (`09ab08395`), log `target/phase45/hourglass-spike.log` (1,241 lines, gitignored): 5 ignored tests passed (harness), 9 `RESULT hourglass` and 9 `FLIP hourglass` lines, no harness error, no fix commit, no rerun. Every configuration was tried; none is "not tried". Configuration `2b` was not tried because the D-18 trigger did not fire (below).

These are authoring measurements for the go or drop decision, not evidence for any shipped claim (D-16).

Window = 25-30 s (21 samples). Slopes in degrees. Water margin = edge-relative median − water median (0.76°). Every configuration has 564 particles and ends with `upper` 0 / `lower` 564 at 30 s.

| # | Label | Particles | ms/step | Median slope | Min | Max | literal_d25_deg median (min-max) | side_fit_deg median | Water margin | Jam | Frozen (end / total freezes / per step / max per step / max speed m/s) | Edge-relative verdict | Literal-D-25 verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| W | w-water | 564 | 0.128 | 0.76 | 0.75 | 0.77 | 25.88 (25.88-25.91) | 0.48 | 0.00 | no | n/a | FAIL | PASS |
| P | p-powder | 564 | 0.127 | 2.87 | 2.82 | 3.03 | 25.92 (25.87-25.96) | 8.49 | 2.11 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.2-k0.5 | 564 | 0.141 | 3.85 | 3.83 | 3.87 | 26.00 (25.98-26.01) | 11.92 | 3.09 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.2-k1.0 | 564 | 0.138 | 3.23 | 3.22 | 3.25 | 26.06 (26.04-26.06) | 2.93 | 2.47 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.5-k0.5 | 564 | 0.145 | 0.24 | 0.21 | 0.26 | 26.09 (26.07-26.11) | 0.48 | -0.52 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.5-k1.0 | 564 | 0.140 | 2.97 | 2.97 | 2.98 | 25.97 (25.95-25.98) | 4.05 | 2.21 | no | n/a | FAIL | PASS |
| 2 | c2-freeze-k8 | 564 | 0.136 | 0.34 | 0.34 | 0.34 | 26.13 (26.13-26.13) | 1.53 | -0.42 | no | 448 / 448 / 0.249 / 8 / 0.000000 | FAIL | PASS |
| 2 | c2-freeze-k32 | 564 | 0.137 | 0.33 | 0.33 | 0.33 | 26.15 (26.15-26.15) | 1.51 | -0.43 | no | 463 / 463 / 0.257 / 13 / 0.000000 | FAIL | PASS |
| 3 | c3-rigid-clumps | 564 | 0.480 | 2.73 | 2.69 | 2.83 | 26.46 (26.38-26.50) | 5.99 | 1.97 | no | n/a | FAIL | PASS |

Candidate 3 builds 152 rigid groups: 130 full 2 × 2 clumps and 22 partial clumps where the funnel clips a block.

Informational, from the `SPIKE` lines (not part of the rule): the largest edge-relative slope at any sample of the 30 s run was W 11.22° at 3.75 s, P 19.19° at 1.50 s, `c1-stick-slip-v0.2-k0.5` 27.98° at 1.50 s, `-v0.2-k1.0` 11.42° at 2.00 s, `-v0.5-k0.5` 16.03° at 1.50 s, `-v0.5-k1.0` 28.50° at 1.00 s, both `c2-freeze` 19.19° at 1.50 s (identical to P until the first freeze), `c3-rigid-clumps` 12.99° at 1.50 s. These peaks fall during the pour, when only a few bins are occupied, and they slump within a second or two.

What the numbers show:

1. **No configuration piles.** Every edge-relative median over 25-30 s is between 0.24° and 3.85°, far below 20°, and every water margin is below 3.1°, far below the 15° guard. All nine rows fail by the pre-recorded rule.
1. **The literal D-25 reading is a confinement artifact, as research Pitfall 3 predicted.** The water control itself reads 25.9° on `literal_d25_deg`, and every configuration reads 25.9-26.5°. A flat layer whose p95 particle height is about 0.73 m above the floor in the 3 m bulb gives atan(0.73 / 1.5) ≈ 25.9°, whatever the material. The literal verdict would PASS plain water, so it cannot distinguish sand from water; the decision uses only the edge-relative verdict, as recorded before measuring.
1. **The glass drains fast and never jams.** The 0.5 m neck passes all 564 grains in about 3 s for every configuration (`upper` 0 by 3.0 s; for the clumps, 2 grains left at 3.0 s and 0 at 4.0 s); no jam anywhere.
1. **Stick-slip (1):** velocity damping before `world.step` does not hold a slope. The edits happen before the step, and the in-step pressure solve moves the grains again, so short heaps formed during the pour (up to 28.5°) slump to 0.2-3.9°.
1. **Freeze to `WALL` (2):** frozen grains stay perfectly still (max frozen speed 0.000000 m/s, no path error), but the 30-step rest test (speed < 0.05 m/s) is met only after the powder deposit has already flattened (38 frozen at 5 s, about 420 at 10 s), so the rule freezes a flat layer (0.33-0.34°). K = 8 and K = 32 give the same picture (at most 8 and 13 freezes per step). The release proof passed in both: the gravity change to 10.5 released 448 and 463 grains, 0 stayed frozen, and restoring gravity 10 released 0 more.
1. **Rigid clumps (3):** 2 × 2 rigid clumps flow through the 5-diameter neck without jamming and settle at 2.7°, with ms/step about 3.8 times the powder control (0.480 against 0.127).
1. **D-18 trigger did not fire:** the re-created `WALL` grains did not move and destroy-and-recreate raised no error, so no `World::set_particle_flags` was added and `crates/liquidfun` is unchanged (configuration `2b` not tried).
1. Native release cost is 0.13-0.15 ms/step for configurations W, P, 1 and 2, and 0.48 ms/step for the rigid clumps, at 564 particles.

## Flip cost

Authoring measurements (D-16), native release, 564 particles, one flip per configuration after its 30 s run (after the release proof for candidate 2). `batched` times the whole sequence (read all, mark all, one compaction, group rebuild at −p, per-member −v). `per-particle` times `set_particle_position(id, −p)` then `set_particle_velocity(id, −v)` over every non-`WALL` particle, on the state at the end of the 10 s rerun.

| Label | batched ms | per-particle ms | count_ok | order_ok | Groups rebuilt | Frozen rebuilt | lower at 10 s post-flip | ≥ 90% passed again | per-particle moved | WALL edit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| w-water | 2.172 | 0.130 | true | true | 1 | 0 | 564/564 | yes | 564 | none |
| p-powder | 2.172 | 0.130 | true | true | 1 | 0 | 564/564 | yes | 564 | none |
| c1-stick-slip-v0.2-k0.5 | 2.165 | 0.130 | true | true | 1 | 0 | 564/564 | yes | 564 | none |
| c1-stick-slip-v0.2-k1.0 | 2.280 | 0.141 | true | true | 1 | 0 | 564/564 | yes | 564 | none |
| c1-stick-slip-v0.5-k0.5 | 2.062 | 0.133 | true | true | 1 | 0 | 564/564 | yes | 564 | none |
| c1-stick-slip-v0.5-k1.0 | 1.958 | 0.130 | true | true | 1 | 0 | 564/564 | yes | 564 | none |
| c2-freeze-k8 | 2.095 | 0.044 | true | true | 1 | 0 | 564/564 | yes | 174 | ok |
| c2-freeze-k32 | 2.019 | 0.036 | true | true | 1 | 0 | 564/564 | yes | 173 | ok |
| c3-rigid-clumps (one group per clump) | 36.451 | 0.185 | true | true | 152 | 0 | 564/564 | yes | 564 | none |

- Assumption A1 holds: rebuilt group members come back in source order in every configuration (`order_ok true`), and the particle count is unchanged (`count_ok true`). No duplicate sand (Pitfall 1 avoided).
- After the flip the sand runs through the neck again in about 3 s (`upper` 0 by 3.0 s post-flip for W, P, 1 and 2; 4.0 s for the clumps), so 100% has passed again at 10 s.
- Candidate 2's batched flip rebuilt 0 frozen grains because the release proof ran first (by the recorded order). During the 10 s rerun the freeze hook froze 390 and 391 grains again, so the per-particle comparison moved only the 174 and 173 non-`WALL` grains, and one untimed `set_particle_position` on a `WALL` grain succeeded (`wall_edit ok`).
- At this size the per-particle path is cheaper than the batched one (about 0.13 ms against 2.1 ms). Each `set_particle_position` rebuilds the proxy list and clears contacts, pairs and triads, which is O(n) per call with a small constant, while the batched path pays for a system clone in the compaction plus group planning. The per-particle path is still O(n²) overall and drops pairs and triads; the batched path keeps O(n) and, for the clumps, scales with the group count (36.5 ms for 152 groups).

## Decision

Decision by pre-recorded rule: drop; owner confirmation: pending (Plan 08).

By D-27, no candidate configuration (1-3) passes the edge-relative rule: all 7 candidate configurations (4 stick-slip, 2 freeze, 1 clumps) and both controls fail, with medians of 0.24-3.85° against the 20° bar and water margins below 3.1° against the 15° guard.

On confirmation (or, if the owner is unavailable, by the pre-recorded owner rule of 2026-10-14), Plan 08 returns SAND-01 and SAND-02 to a seed.

Context for that checkpoint (observations, not decisions): the flip itself is cheap and correct (2 ms batched at 564 particles, order and count preserved), frozen `WALL` grains stay still without any engine change, and short heaps of up to about 28° appear during the pour but slump within a second or two. Rules that act within the pour, such as freezing on a shorter rest test, were outside this time box (D-17) and were not run.

## Commits

- Commit A (add spike and rules): `09ab08395ee8ab8ad7adc8a2460d1c1f22f34d72` `test(45-07): add the hourglass feasibility spike (D-24)`
- Commit B (remove spike code): `effc9c98f60062a98eebb38ed65587ede7b34f36` `chore(45-07): remove the hourglass spike code`
- No D-18 commit: the trigger did not fire.
- Restore the code with `git show 09ab08395ee8ab8ad7adc8a2460d1c1f22f34d72 -- crates/liquidfun-wasm/src/hourglass_spike.rs` (plus `crates/liquidfun-wasm/src/hourglass_spike/flip.rs`, `geometry.rs`, `hooks.rs` and `metrics.rs`, and the `#[cfg(test)] mod hourglass_spike;` line in `crates/liquidfun-wasm/src/lib.rs`).
- `crates/liquidfun` is unchanged (`git diff --stat b151838be -- crates/liquidfun` is empty); no scene, session or shared builder file was edited.
