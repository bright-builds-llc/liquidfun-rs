---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "07"
subsystem: wasm-spike
tags: [rust, wasm, spike, hourglass, sand, prep-03, d-24, d-25, d-26, d-27]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:51:30.835Z"
requires:
  - "45-01: pinned scene fingerprint baseline (the spike edits no scene, session or engine file)"
  - "45-06: siphon spike harness pattern and measurement lessons (rules fixed before commit A)"
provides:
  - "45-HOURGLASS-SPIKE.md: pre-recorded geometry, edge-relative slope metric with water guard, jam rule, D-26 flip procedure, D-24 sweeps, 9 measured rows, flip cost and the D-27 decision by rule (drop, owner confirmation pending for Plan 08)"
  - "Spike harness preserved in git history at 09ab08395 (crates/liquidfun-wasm/src/hourglass_spike.rs + hourglass_spike/{flip,geometry,hooks,metrics}.rs), removed in effc9c98f"
affects: [45-08, 51]
tech-stack:
  added: []
  patterns:
    - "Edge-relative pile slope: rise = p95 bin top minus mean outermost bin tops; run measured between the outermost bins' top particles; same-geometry water control and a 15 degree margin guard"
    - "Batched point-reflection flip: mark all, one compaction, one group rebuild at -p, per-member -v; member order equals source order (A1 holds)"
key-files:
  created:
    - .planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-HOURGLASS-SPIKE.md
  modified: []
key-decisions:
  - "Hourglass spike decision by pre-recorded D-27 rule: drop; owner confirmation pending (Plan 08). All 9 configurations (water, powder, stick-slip v_slip {0.2,0.5} x k {0.5,1.0}, freeze-to-WALL K {8,32}, rigid 2x2 clumps) FAIL the edge-relative rule: 25-30 s medians 0.24-3.85 degrees, water margins below 3.1 degrees"
  - "Literal D-25 reading (p95 height over half base width) reads 25.9-26.5 degrees for every row including plain water, so it is a confinement artifact; the decision uses only the edge-relative verdict as recorded before measuring"
  - "D-18 did not fire: re-created WALL grains stayed at 0 m/s and destroy-and-recreate raised no error; crates/liquidfun unchanged, no World::set_particle_flags, configuration 2b not tried"
  - "Flip cost at 564 particles: batched rebuild about 2 ms (36.5 ms for 152 rigid clump groups), per-particle set_particle_position about 0.13 ms (O(n) per call, clears contacts/pairs/triads); order and count preserved, 100% passes the neck again within about 3 s"
  - "Pre-measurement fix before commit A: the run is measured between the outermost bins' top particles; the plan formula read a 20 degree particle triangle as 17.2 degrees and could not pass its own 20 +/- 1 unit test"
  - "No REQUIREMENTS, ROADMAP, PROJECT or seed edits in 45-07; Plan 08 shows the drop to the owner and, on confirmation, returns SAND-01 and SAND-02 to a seed"
requirements-completed: []
duration: 12min
completed: 2026-10-14
---

# Phase 45 Plan 07: Sand Hourglass Feasibility Spike Summary

**Decision by pre-recorded rule: drop; owner confirmation: pending (Plan 08).** None of the 9 measured configurations reaches a 20° pile. Each one drains the glass in about 3 s and settles to an almost flat deposit (0.24-3.85° median over 25-30 s). That is within 3.1° of plain water, against the required 15° margin.

These numbers are authoring measurements for the decision, not evidence for any shipped claim (D-16). The full record, including the rules that were committed before measuring, is in `45-HOURGLASS-SPIKE.md`. The raw log is `target/phase45/hourglass-spike.log` (1,241 lines, gitignored).

## Performance

- **Duration:** about 12 min
- **Started:** 2026-10-14T16:40:14Z
- **Completed:** 2026-10-14T16:51:30Z
- **Tasks:** 2
- **Files:** 1 doc created. 5 spike files were added and then removed, and `lib.rs` is back to its pre-spike state.

## Accomplishments

- Committed the hourglass rules in `45-HOURGLASS-SPIKE.md` together with the harness in commit A, before the first measured run. The rules cover:
  - the point-symmetric glass (564-particle upper fill, 0.5 m neck, 52° funnels, the upper half built by point reflection)
  - the edge-relative slope metric and the water guard
  - the jam rule
  - the D-26 flip procedure
  - the D-24 sweeps, the D-18 trigger and the D-27 rule
- Ran the controls and all three sand rules (9 configurations) once in native release. There was no harness error, no fix commit and no rerun.
- Measured the flip cost in both ways, batched and per-particle, for every configuration. Never inverted gravity.
- Applied D-27 by rule: **drop**. Owner confirmation is left to Plan 08.
- Removed the spike code in commit B. It can be restored from commit A.

## Results

| # | Label | ms/step | Median / min / max slope (°) | literal_d25 median (°) | side_fit median (°) | Water margin (°) | Jam | Frozen end / max speed | Edge-relative | Literal-D-25 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| W | w-water | 0.128 | 0.76 / 0.75 / 0.77 | 25.88 | 0.48 | 0.00 | no | n/a | FAIL | PASS |
| P | p-powder | 0.127 | 2.87 / 2.82 / 3.03 | 25.92 | 8.49 | 2.11 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.2-k0.5 | 0.141 | 3.85 / 3.83 / 3.87 | 26.00 | 11.92 | 3.09 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.2-k1.0 | 0.138 | 3.23 / 3.22 / 3.25 | 26.06 | 2.93 | 2.47 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.5-k0.5 | 0.145 | 0.24 / 0.21 / 0.26 | 26.09 | 0.48 | -0.52 | no | n/a | FAIL | PASS |
| 1 | c1-stick-slip-v0.5-k1.0 | 0.140 | 2.97 / 2.97 / 2.98 | 25.97 | 4.05 | 2.21 | no | n/a | FAIL | PASS |
| 2 | c2-freeze-k8 | 0.136 | 0.34 / 0.34 / 0.34 | 26.13 | 1.53 | -0.42 | no | 448 / 0.000000 | FAIL | PASS |
| 2 | c2-freeze-k32 | 0.137 | 0.33 / 0.33 / 0.33 | 26.15 | 1.51 | -0.43 | no | 463 / 0.000000 | FAIL | PASS |
| 3 | c3-rigid-clumps | 0.480 | 2.73 / 2.69 / 2.83 | 26.46 | 5.99 | 1.97 | no | n/a | FAIL | PASS |

Every configuration has 564 particles and ends at 30 s with `upper` 0 and `lower` 564. The literal-D-25 column passes plain water too, so it cannot tell sand from water, which is research Pitfall 3. The decision uses only the edge-relative verdict.

## Flip cost

| Label | batched ms | per-particle ms | count_ok | order_ok | Groups | lower 10 s post-flip |
| --- | --- | --- | --- | --- | --- | --- |
| w-water | 2.172 | 0.130 | true | true | 1 | 564/564 |
| p-powder | 2.172 | 0.130 | true | true | 1 | 564/564 |
| c1-stick-slip-v0.2-k0.5 | 2.165 | 0.130 | true | true | 1 | 564/564 |
| c1-stick-slip-v0.2-k1.0 | 2.280 | 0.141 | true | true | 1 | 564/564 |
| c1-stick-slip-v0.5-k0.5 | 2.062 | 0.133 | true | true | 1 | 564/564 |
| c1-stick-slip-v0.5-k1.0 | 1.958 | 0.130 | true | true | 1 | 564/564 |
| c2-freeze-k8 | 2.095 | 0.044 (174 non-WALL) | true | true | 1 | 564/564 |
| c2-freeze-k32 | 2.019 | 0.036 (173 non-WALL) | true | true | 1 | 564/564 |
| c3-rigid-clumps (per clump) | 36.451 | 0.185 | true | true | 152 | 564/564 |

## Decision

Decision by pre-recorded rule: drop; owner confirmation: pending (Plan 08).

On confirmation (or, if the owner is unavailable, by the pre-recorded owner rule of 2026-10-14), Plan 08 returns SAND-01 and SAND-02 to a seed.

**D-18 outcome:** The trigger did not fire. Frozen `WALL` grains stayed at 0.000000 m/s, and destroy-and-recreate raised no error. No `World::set_particle_flags` was added, `crates/liquidfun` is unchanged, and configuration `2b` was not tried.

## Task Commits

1. **Task 1: rules, harness, controls and candidates (commit A):** `09ab08395ee8ab8ad7adc8a2460d1c1f22f34d72` (test)
1. **Task 2: remove the spike code (commit B):** `effc9c98f60062a98eebb38ed65587ede7b34f36` (chore)
1. **Task 2: results and decision (commit C):** `c304ed183` (docs)

To restore the harness, run `git show 09ab08395 -- crates/liquidfun-wasm/src/hourglass_spike.rs crates/liquidfun-wasm/src/hourglass_spike/flip.rs crates/liquidfun-wasm/src/hourglass_spike/geometry.rs crates/liquidfun-wasm/src/hourglass_spike/hooks.rs crates/liquidfun-wasm/src/hourglass_spike/metrics.rs`.

## Files Created/Modified

- `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-HOURGLASS-SPIKE.md`: the rules, results, flip cost, decision and commits.
- `crates/liquidfun-wasm/src/hourglass_spike.rs` and `hourglass_spike/{flip,geometry,hooks,metrics}.rs`: added in A and deleted in B.
- `crates/liquidfun-wasm/src/lib.rs`: A added the `#[cfg(test)] mod hourglass_spike;` line and B removed it. `git diff 09ab08395^ -- crates/liquidfun-wasm/src/lib.rs` is empty.

## Decisions Made

See `key-decisions` above. The hourglass outcome by rule is **drop**, and owner confirmation is still pending. No requirement, roadmap, project or seed file was edited.

## Deviations from Plan

### Auto-fixed Issues

All of these were made before commit A and before any stepping. They are recorded in the spike doc's rule sections.

**1. [Rule 1 - Bug] The plan's slope formula could not pass its own unit test**
- **Found during:** Task 1, metric design
- **Issue:** `atan((peak − edge) / (base_width / 2))` takes its heights from the top particles, which sit near each bin's inner side, while `base_width` runs to the outer edges of the outermost bins. A true 20° particle triangle spanning the 3 m bulb therefore reads 17.2°, or 16.2° if p95 picks the second-highest bin. The required test is "20° triangle gives 20° ± 1°".
- **Fix:** The run is now measured between the x positions of the top particles in the two outermost occupied bins, and the same triangle reads 19.5°. `base_width` is unchanged and still feeds `literal_d25_deg`. A unit test (`plan_formula_reads_the_twenty_degree_triangle_low`) records the plan formula's reading. A flat deposit still reads about 0° whatever the run, so passing is no easier.
- **Files modified:** `hourglass_spike/metrics.rs`, `45-HOURGLASS-SPIKE.md`
- **Commit:** 09ab08395

**2. [Rule 1 - Bug] `ParticleId` has no `Ord`**
- **Found during:** Task 1
- **Issue:** The plan asks for a `BTreeMap` keyed by particle id, with ties broken by particle id. `ParticleId` derives only `Eq`/`Hash`.
- **Fix:** The key is a deterministic `u64` hash of the id (`DefaultHasher::new()`), which also serves as the tie-break. A key collision is treated as a harness error.
- **Commit:** 09ab08395

**3. [Rule 3 - Blocking] The file-length limit forced extra submodules**
- **Issue:** With the hooks and the flip included, the main spike file was 737 lines, over the 628-line limit.
- **Fix:** Added `hourglass_spike/hooks.rs` and `hourglass_spike/flip.rs` alongside the planned `geometry.rs` and `metrics.rs`. All were removed in commit B.
- **Commit:** 09ab08395

**4. [Note] Definitions the plan left open, fixed before commit A**
- "Nearest-rank" uses the textbook rank ⌈p·N⌉.
- The support median is taken over non-`WALL` lower-bulb particles.
- Partial 2 × 2 clumps at the slanted funnel become smaller rigid groups, so the count stays 564. That gives 152 groups: 130 full and 22 partial.
- The fill is 564 particles, against the plan's estimate of about 600.
- `BUILD`, `DETAIL`, `RELEASE` and `POST` lines were added as informational output. The `SPIKE`/`RESULT`/`FLIP` formats are as specified; `FLIP` has extra trailing fields.

---

**Total deviations:** 3 auto-fixed (2 Rule 1, 1 Rule 3) and 1 note, all made before measurement.
**Impact on plan:** No rule, geometry or sweep changed after the first measured run.

## Issues Encountered

The harness had no issues. All 5 ignored tests completed, and the run finished in 4 s.

## Known Stubs

None. The spike code is removed.

## Next Phase Readiness

- Plan 08 shows the owner both the hourglass **drop** and the siphon drop. Only after confirmation, or under the pre-recorded owner rule if the owner is unavailable, does it return SAND-01 and SAND-02 to a seed and mark Phase 51 accordingly.
- The checkpoint gets this context (observations, not decisions):
  - The flip is cheap and correct.
  - Frozen `WALL` grains stay still without any engine change.
  - Heaps of up to about 28° form briefly during the pour but slump within a second or two.
  - Rules that act during the pour were outside this time box (D-17).

## Self-Check: PASSED

- FOUND: `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-HOURGLASS-SPIKE.md`
- FOUND: `target/phase45/hourglass-spike.log` (9 `RESULT hourglass` and 9 `FLIP hourglass` lines)
- MISSING as intended: `crates/liquidfun-wasm/src/hourglass_spike.rs`. `lib.rs` has no `hourglass_spike` line.
- FOUND commits: 09ab08395, effc9c98f, c304ed183
- `git diff --stat b151838be -- crates/liquidfun` is empty. No 45-07 commit touches REQUIREMENTS.md, ROADMAP.md, PROJECT.md or seeds, and PREP-03 is not ticked.
