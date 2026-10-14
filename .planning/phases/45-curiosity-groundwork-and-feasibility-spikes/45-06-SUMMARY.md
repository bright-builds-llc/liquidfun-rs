---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "06"
subsystem: wasm-spike
tags: [rust, wasm, spike, siphon, prep-03, d-20, d-21, d-22]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:33:12.899Z"
requires:
  - "45-01: pinned scene fingerprint baseline (the spike edits no scene, session or engine file)"
provides:
  - "45-SIPHON-SPIKE.md: pre-recorded geometry, metrics, D-21 pass rule, D-20 sweeps, 14 measured rows and the D-22 decision (drop)"
  - "Spike harness preserved in git history at dcf7992b3 (crates/liquidfun-wasm/src/siphon_spike.rs + siphon_spike/{geometry,tests}.rs), removed in 6366fe335"
affects: [45-08, 50]
tech-stack:
  added: []
  patterns:
    - "Self-contained World-driven spike: test-only module registered by one #[cfg(test)] line in lib.rs, SceneHooks-shaped SpikeHook trait, rules committed with the harness before the first run"
    - "Free-surface estimate = median over 2-diameter x-bins of each bin's highest particle (p95 of all particles reads 0.95·H for a column)"
key-files:
  created:
    - .planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-SIPHON-SPIKE.md
  modified: []
key-decisions:
  - "Siphon spike decision by pre-recorded D-22 rule: drop. All 14 configurations (plain, viscous 0.5/1.0/1.5, tensile+viscous, fake atmosphere α 0.5/1/2, weighted lids β 0.5/1/2, tube pull α 0.25/0.5/1) FAIL; 0 leaks, 0 lost, 0.39-0.49 ms/step at 1,543 particles"
  - "Dominant failure: LiquidFun water at r 0.05, g 10 compresses strongly over metre-scale heads (the primed 2 m source column settles to about 1.2 m within 0.5 s), so the crest empties before a siphon can form; every t* is the 2 s floor"
  - "Viscous rule A passes are the primed outlet leg draining slowly into D (count_s does not fall); tube pull α 1.0 ends level (diff 0.003) but fails rule A on a first-window dip (554 to 546) and acts as a gated pump"
  - "Pre-measurement rule fixes before commit A: surface = median of bin tops instead of p95 of all particles, t* not taken before 2 s, mouth gaps counted as interior"
  - "No REQUIREMENTS, ROADMAP, PROJECT or seed edits; Plan 08 shows the drop to the owner with the hourglass result"
requirements-completed: []
duration: 11min
completed: 2026-10-14
---

# Phase 45 Plan 06: Siphon Feasibility Spike Summary

**Decision: drop.** By the pre-recorded D-22 rule, all 14 measured siphon configurations fail. The primed 2 m LiquidFun water column compresses to about 1.2 m within 0.5 s, so the tube crest empties before any siphon can form.

These numbers are authoring measurements for the decision, not evidence for any shipped claim (D-16). The full record, including the pre-registered rules, is in `45-SIPHON-SPIKE.md`. The raw log is `target/phase45/siphon-spike.log` (1,747 lines, gitignored).

## Performance

- **Duration:** about 11 min
- **Started:** 2026-10-14T16:21:58Z
- **Completed:** 2026-10-14T16:33:12Z
- **Tasks:** 2
- **Files:** 1 doc created; 4 spike files added then removed; `lib.rs` back to its pre-spike state

## Accomplishments

- Fixed the geometry (1,543-particle primed inverted U, 0.3 m walls, 0.5 m tube), metric formulas, D-21 pass rule, D-20 sweeps and D-22 decision rule in `45-SIPHON-SPIKE.md`, and committed them together with the harness in commit A before the first measured run.
- Ran all six candidates (14 configurations) once in native release, with no harness error and no rerun.
- Recorded every row in order, the rule A window detail, and the candidate 3 head-difference check. Then applied D-22: **drop**.
- Removed the spike code in commit B. It can be restored from commit A.

## Results

| # | Label | ms/step | t* (s) | Rule A | Rule B | diff_end (m) | Leaks | Lost | count_d 2 / 5 / 10 / 20 / 30 s | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | c1-plain-water | 0.389 | 2.00 | fail | fail | 1.496 | 0 | 0 | 448 / 396 / 402 / 402 / 402 | FAIL |
| 2 | c2-viscous-0.5 | 0.445 | 2.00 | pass | fail | 1.499 | 0 | 0 | 327 / 364 / 378 / 386 / 391 | FAIL |
| 2 | c2-viscous-1.0 | 0.432 | 2.00 | pass | fail | 1.501 | 0 | 0 | 269 / 337 / 364 / 377 / 384 | FAIL |
| 2 | c2-viscous-1.5 | 0.433 | 2.00 | pass | fail | 1.501 | 0 | 0 | 239 / 304 / 350 / 373 / 379 | FAIL |
| 3 | c3-tensile-viscous | 0.454 | 2.00 | fail | fail | 1.662 | 0 | 0 | 375 / 375 / 377 / 377 / 377 | FAIL |
| 4 | c4-fake-atmosphere-0.5 | 0.409 | 2.00 | fail | fail | 1.491 | 0 | 0 | 438 / 392 / 401 / 401 / 401 | FAIL |
| 4 | c4-fake-atmosphere-1.0 | 0.411 | 2.00 | fail | fail | 1.503 | 0 | 0 | 428 / 390 / 404 / 404 / 403 | FAIL |
| 4 | c4-fake-atmosphere-2.0 | 0.415 | 2.00 | fail | fail | 1.495 | 0 | 0 | 414 / 381 / 403 / 404 / 403 | FAIL |
| 5 | c5-weighted-lids-0.5 | 0.443 | 2.00 | fail | fail | 1.489 | 0 | 0 | 406 / 383 / 386 / 386 / 385 | FAIL |
| 5 | c5-weighted-lids-1.0 | 0.461 | 2.00 | fail | fail | 1.475 | 0 | 0 | 375 / 365 / 368 / 368 / 368 | FAIL |
| 5 | c5-weighted-lids-2.0 | 0.486 | 2.00 | fail | fail | 1.409 | 0 | 0 | 332 / 331 / 332 / 332 / 332 | FAIL |
| 6 | c6-tube-pull-0.25 | 0.401 | 2.00 | fail | fail | 1.234 | 0 | 0 | 524 / 502 / 506 / 506 / 506 | FAIL |
| 6 | c6-tube-pull-0.50 | 0.409 | 2.00 | fail | fail | 1.082 | 0 | 0 | 580 / 527 / 541 / 541 / 541 | FAIL |
| 6 | c6-tube-pull-1.00 | 0.458 | 2.00 | fail | pass | 0.003 | 0 | 0 | 554 / 898 / 923 / 942 / 950 | FAIL |

Every configuration starts with 1,543 particles, and count_d is 201 at step 0.

Candidate 3 head-difference check: from 0 to 2 s, count_d rises by 174 for tensile+viscous water and by 247 for plain water (70% as much). Downhill flow still works with TENSILE. It still forms no siphon.

## Decision

**drop**: by the pre-recorded D-22 rule, no natural candidate (1-3) and no stand-in (4-6) passes. The owner sees this outcome with the hourglass result in Plan 08. Requirement and roadmap bookkeeping is deferred to the Plan 08 owner checkpoint.

## Task Commits

1. **Task 1: rules, harness and candidates (commit A):** `dcf7992b3081d2575623930a8386f51adf03ae93` (test)
1. **Task 2: remove the spike code (commit B):** `6366fe335983a23403eb0b67b51a517eac7737f5` (chore)
1. **Task 2: results and decision (commit C):** `3641871fa` (docs)

Restore with `git show dcf7992b3 -- crates/liquidfun-wasm/src/siphon_spike.rs crates/liquidfun-wasm/src/siphon_spike/geometry.rs crates/liquidfun-wasm/src/siphon_spike/tests.rs`.

## Files Created/Modified

- `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-SIPHON-SPIKE.md`: rules, results, decision and commits.
- `crates/liquidfun-wasm/src/siphon_spike.rs`, `siphon_spike/geometry.rs`, `siphon_spike/tests.rs`: added in A and deleted in B.
- `crates/liquidfun-wasm/src/lib.rs`: added the `#[cfg(test)] mod siphon_spike;` line in A and removed it in B. `git diff dcf7992b3^ -- crates/liquidfun-wasm/src/lib.rs` is empty.

## Decisions Made

- These are listed in `key-decisions` above. The siphon outcome is **drop**. No requirement, roadmap, project or seed file was edited.

## Deviations from Plan

### Auto-fixed Issues

All of these were made before commit A and before any stepping, and they are recorded in the spike doc's rule sections.

**1. [Rule 1 - Bug] Surface estimator read 1.9 at step 0**
- **Found during:** Task 1, metric design
- **Issue:** The plan defined `source_surface` as the p95 of y over all particles in S. For a uniform column of height H, that gives about 0.95·H. The primed 2 m column would therefore read 1.9 (crest - 0.3) before anything moved, so t* would fire at once.
- **Fix:** Use the median, over occupied 0.2 m x-bins, of each bin's highest particle. A unit test checks that a flat column with top row 2.0 reads 2.0.
- **Files modified:** `siphon_spike.rs`, `45-SIPHON-SPIKE.md`
- **Commit:** dcf7992b3

**2. [Rule 1 - Bug] Rule A could pass on the priming drain**
- **Found during:** Task 1, after the geometry count test showed a source top row of 1.975, one row above crest - 0.3
- **Issue:** Compression settling could cross 1.9 within the first second. Meanwhile the 3.2 m outlet leg drains its priming water into D. Together, these could let plain water pass rule A with no siphon.
- **Fix:** t* is never taken before 2 s, and rule B uses the same t*. A unit test covers this (`rule_a_ignores_a_priming_drain_in_the_first_two_seconds`). The measured run confirmed the concern, since the source column did compress to about 1.2 m.
- **Files modified:** `siphon_spike.rs`, `siphon_spike/tests.rs`, `45-SIPHON-SPIKE.md`
- **Commit:** dcf7992b3

**3. [Rule 1 - Bug] Mouth gaps missing from the interior regions**
- **Found during:** Task 1, geometry
- **Issue:** The inlet mouth gap (x -2.1..-1.8) lay outside both expanded S and the expanded inlet leg. Particles there would have counted as leaks.
- **Fix:** Both mouth gaps are now interior regions and are filled at step 0. A test checks that the step-0 fill has 0 leaks.
- **Commit:** dcf7992b3

**4. [Rule 2 - Correctness] Lids are frictionless and fixed-rotation**
- **Issue:** The plan did not say how the lids should move. A tilting lid with 0.05 m wall clearance could wedge in the tank and stop measuring top load at all.
- **Fix:** The lids are pistons with friction 0 and fixed rotation. This is recorded in the candidate list before the run.
- **Commit:** dcf7992b3

**5. [Note] Fill count and sweep size**
- One global 0.075 m grid gives 1,543 particles, against the plan's estimate of about 1,800.
- The fixed sweep has 14 configurations (1 + 3 + 1 + 3 + 3 + 3). The plan's "at most 13" was an arithmetic slip. The doc's Results table has 14 rows, matching the 14 `RESULT siphon` log lines.
- `count_s` (particles in S) and a `DETAIL` line were added as informational output. The SPIKE and RESULT line formats are otherwise as specified.

---

**Total deviations:** 4 auto-fixed (3 Rule 1, 1 Rule 2) and 1 note, all made before measurement.
**Impact on plan:** No rule, geometry or sweep changed after the first measured run.

## Issues Encountered

None in the harness. All 6 ignored tests completed and the run finished in 11 s.

## Known Stubs

None. The spike code is removed.

## Next Phase Readiness

- Plan 08 shows the owner the siphon **drop** together with the hourglass result. The SIPHON seed and REQUIREMENTS/ROADMAP bookkeeping happen there only if the owner confirms.
- For that checkpoint, the doc notes an untried, different experiment: a much shorter head with a pre-settled fill. It falls outside this time box (D-17).

## Self-Check: PASSED

- FOUND: `.planning/phases/45-curiosity-groundwork-and-feasibility-spikes/45-SIPHON-SPIKE.md`
- FOUND: `target/phase45/siphon-spike.log` (14 `RESULT siphon` lines)
- MISSING as intended: `crates/liquidfun-wasm/src/siphon_spike.rs`; `lib.rs` has no `siphon_spike` line
- FOUND commits: dcf7992b3, 6366fe335, 3641871fa
- `git diff --stat b151838be -- crates/liquidfun`: empty. Commits A, B and C touch none of REQUIREMENTS.md, ROADMAP.md or seeds. The final metadata commit changes ROADMAP.md only for orchestrator plan-progress bookkeeping (the 45-06 checkbox and 6/10); it records no siphon outcome. PREP-03 is not ticked.
