---
phase: 36-re-survey-and-wrap-up
plan: "02"
subsystem: performance-evidence
tags: [scene-survey, abba, before-after, docs, PERF-10]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-10-11T02-24-50
generated_at: "2026-10-11T03:33:43.755Z"
requires:
  - 36-01 scene-spot stamps that record scene_filter and full_catalog
  - target/phase35/bin/spot-before, before-full.jsonl, abba-gated.sh and keep_rule.py (Phase 35)
provides:
  - docs/benchmarks/scene-survey.md "v1.4 before/after" table for all 25 scenes (pre-registered, gated ABBA)
  - refreshed "Current ranked survey (v1.4 final)" table and the unchanged Phase 34 survey under "Original survey (Phase 34)"
  - user-facing pointers (spot-check page, README, BENCHMARKING) to the survey page with no new numbers
affects:
  - 36-03 wrap-up (records future work and marks PERF-10 complete)
tech-stack:
  added: []
  patterns:
    - pre-registered protocol plus a frozen, selftested table script before the first timed leg
    - table rows pasted directly from the frozen script output, then mdformat
key-files:
  created: []
  modified:
    - docs/benchmarks/scene-survey.md
    - docs/playground-scene-spot-check.md
    - README.md
    - BENCHMARKING.md
key-decisions:
  - "v1.4 before/after (spot-before 809582431 vs spot-v14-final, crates/ = edcafebc0; one gated ABBA set, --runs 5, all 25 scenes): faster 24, within noise 1 (drawing-particles), slower 0; 25/25 fingerprints in all six legs"
  - "Current ranking source is v14-rank-b1.jsonl (spot-v14-final --runs 3, gated); a1 is a fingerprint-only repeat"
  - "Named-workload reports that cite ms/step (tesla-valve README, native-performance-audit, playground-dam-break-timing, simulation-performance) stay unchanged"
requirements: [PERF-10]
duration: 15min
completed: 2026-10-11
---

# Phase 36 Plan 02: v1.4 whole-catalog before/after scene survey Summary

**A pre-registered, gated ABBA run of all 25 catalog scenes found the final v1.4 engine faster than the pre-Phase-35 engine in 24 scenes. Drawing-particles was within noise and no scene was slower. Fingerprints were 25/25 equal in every leg. The table, a refreshed current ranking and the original Phase 34 survey are now on `docs/benchmarks/scene-survey.md`.**

## Performance

- **Duration:** about 15 min (03:19:50Z to 03:34Z)
- **Tasks:** 3 of 3
- **Files modified:** 4 tracked docs. Local evidence is under `target/phase35/`, which is gitignored.

## Evidence (local, gitignored, under target/phase35/)

- **Binaries**
  - before: `bin/spot-before`, SHA-256 `5bfc7b614d0701b6c9ba0606dd3b1b0937c5fc931676dff55c294cb6467e0742`, source `809582431faecfdfa13b87f0d8376527bea7f0b4`.
  - after: `bin/spot-v14-final`, SHA-256 `a7d837b1f7e2aafba61b4c392e3db3a7b027e0baf68344372f644b78d15eb10f`, inode 552852888, link count 1.
    - Built with `cargo build --release -p liquidfun-wasm --bin playground-scene-spot` from HEAD `5c664d20c5ce76e3b3a1396abd6c60b12844cbe4`. Cargo found the build up to date (0.05 s).
    - `git diff edcafebc0 HEAD -- crates/ Cargo.toml Cargo.lock rust-toolchain.toml` was empty, and `cmp` shows the binary is byte-identical to `bin/spot-A2r`.
    - RUSTFLAGS was unset.
- **Script:** `v14_table.py`, SHA-256 `ec4923836298296c1c08e37923028fa13794963f7edfcb50b7462cb80449dc89`. I froze it before timing and re-checked the hash before writing the tables. All four selftests matched expectations:
  - S: Faster, Slower +5.0%, Within noise +3.0%, counts 1/23/1, exit 0.
  - F: `PUBLICATION: blocked (a2: particles)`, exit 4.
  - R: row 1 liquid-tumbler 25.429, exit 0.
  - nosuch: `INPUT ERROR`, exit 2.
- **Protocol:** `v14-protocol.txt` has written_utc 2026-10-11T03:21:56Z. The first timed leg started at 03:22:04Z. Both warm launches exited 0 in 0.09 s.
- **ABBA set `v14`:** the first attempt counted. Span 03:22:04 to 03:27:12 UTC, 1-minute load 5.31 to 6.42. GATE-WAITED 2 (120 s, 30 s); VOID, VOID-IGNORED, GATE-TIMEOUT and PRE-WAIT 0. The keep_rule listing (information only) was none in both pairs.
- **Ranking set `v14-rank`:**
  - The first attempt was void at the end of a1 (`cargo-or-rustc-running`). Another repository, open-bitcoin, was running `cargo llvm-cov`/`cargo test`. I moved its keep.txt into `v14-rank.void-1791689300/` unread and re-ran the command once.
  - The counted run spans 03:29:06 to 03:30:18 UTC, with 1-minute load 5.63 to 6.37. GATE-WAITED 2 (30 s each); GATE-TIMEOUT, PRE-WAIT and VOID-IGNORED 0.
  - The A/A keep_rule listing (same binary, information only) flagged elastic-particles, particles and soup above their b1 max.
- **D-04:** all six legs had 25/25 fingerprints equal to `before-full.jsonl`, and rank b1 equaled a1. Both tables print `PUBLICATION: ok`. I ran no diagnostic, so no `v14-diag-*` file exists.

## Before/after results (v14-table.txt, ranked by mean after median)

| Rank | Scene | Change (pair 1 / pair 2) | Result |
| --- | --- | --- | --- |
| 1 | liquid-tumbler | -21.5% / -20.9% | Faster |
| 2 | tesla-valve | -15.4% / -19.9% | Faster |
| 3 | stacked-drip | -14.0% / -13.6% | Faster |
| 4 | particles | -5.2% / -5.6% | Faster |
| 5 | washing-machine | -27.9% / -30.1% | Faster |
| 6 | liquid-bubbler | -19.9% / -19.8% | Faster |
| 7 | soup | -14.4% / -14.2% | Faster |
| 8 | soup-stirrer | -17.0% / -17.4% | Faster |
| 9 | wave-tank | -9.7% / -9.9% | Faster |
| 10 | wave-machine | -17.0% / -11.6% | Faster |
| 11 | hydraulic-fountain | -4.4% / -4.4% | Faster |
| 12 | water-wheel | -42.6% / -43.9% | Faster |
| 13 | impulse | -5.7% / -4.2% | Faster |
| 14 | liquid-timer | -9.9% / -9.3% | Faster |
| 15 | fountain | -44.8% / -43.7% | Faster |
| 16 | elastic-particles | -5.3% / -5.0% | Faster |
| 17 | dam-break | -14.2% / -12.1% | Faster |
| 18 | float-or-sink | -8.3% / -10.1% | Faster |
| 19 | color-mixer | -11.1% / -9.5% | Faster |
| 20 | surface-tension | -10.1% / -10.3% | Faster |
| 21 | rigid-particles | -14.8% / -14.5% | Faster |
| 22 | jelly-drop | -4.3% / -5.7% | Faster |
| 23 | theo-jansen | -29.2% / -26.6% | Faster |
| 24 | sparky | -9.4% / -9.4% | Faster |
| 25 | drawing-particles | -5.1% / -1.9% | Within noise |

`counts: faster 24, within noise 1, slower 0`; `slower: none`. No Slower scene, so Plan 03 has no slower scene to record as future work. Particles is still faster than the pre-Phase-35 engine in both pairs (-5.2% / -5.6%), even after A2r.

## Task Commits

1. **Task 1:** built spot-v14-final, wrote, selftested and froze v14_table.py, and pre-registered v14-protocol.txt. Every artifact is local and gitignored, so there is no commit.
1. **Task 2:** ran the gated ABBA set and the ranking set, then wrote both tables. Local only, no commit.
1. **Task 3:** commit `e5640ea88`, `docs(36): publish v1.4 whole-catalog before/after scene survey`, with the four doc files.

## Checks

- `mdformat` on the 4 files, then `just markdown-check`: exit 0.
- `cargo xtask docs check`: exit 0.
- `cargo test -p xtask --test docs_contract`: 43 passed.
- `bun scripts/bright-builds-check.ts all`: findings 0.
- Claim scan (`fps|times faster|[0-9]+x faster`): one match, the required "no FPS promise" disclaimer at scene-survey.md:35.
- The Phase 34 ranked table is identical to commit `9d8718bc1`, and the moved Recorded run and Ranked table sections are byte-identical apart from the demoted headings.

## ms/step inventory (`git grep -n "ms/step" -- '*.md' ':!.planning'`)

- `docs/benchmarks/scene-survey.md`: the survey page itself.
- `docs/benchmarks/tesla-valve/README.md`: unchanged. It is a separate named-workload report for the Tesla valve fixed-trajectory stages.
- `docs/native-performance-audit.md`: unchanged. It is a separate named-workload report (SHA-bound native audit).
- `docs/playground-dam-break-timing.md`: unchanged. It is a separate named-workload report (Dam Break pair).
- `docs/simulation-performance.md`: unchanged. It is a separate named-workload report (particle-contact scan sample).

## Decisions Made

- Every ms value for drawing-particles shows 3 decimals plus 4 in parentheses, including its min-max cells. I applied the plan's formatting rule to every ms cell.
- The current ranked table uses the Phase 34 table's right-aligned numeric columns so the two ranked tables look the same. The cell values come straight from v14-rank-table.txt.

## Deviations from Plan

- **How to read, item 1:** besides putting the last sentence in the past tense, I reworded the first sentence from "across the 3 fresh runs" to cover both run counts (5 per leg in the before/after table, 3 in the ranked tables). The old wording would have been wrong for the new 5-run table. This was wording only and adds no number.
- **BENCHMARKING.md:** "that local sample is not the Dam Break 3× gate" became "those local samples are not the Dam Break 3× gate", because the sentence now names two pages. The 3× and manifest warnings are kept.
- **Ranking set:** one void and one re-run under the protocol's void rules. This was not a deviation, but I record it here for transparency.

## Issues Encountered

- Another repository's cargo run voided the first ranking attempt. The gate handled it as designed: I re-ran the set once, and the re-run counted without a PRE-WAIT or NO_VOID.

## Threat Flags

None. T-36-04 to T-36-07 were mitigated as planned: the protocol and frozen hash came before timing, the gate and void rules applied, all six legs had 25/25 fingerprints, and the "Local observations, not public speed claims" framing stays with no number added to README or BENCHMARKING.

## Next Phase Readiness

- Plan 36-03 can record the future work (D-09) and mark PERF-10 complete. This plan does not mark PERF-10 complete.
- No `v1.4` tag exists and no milestone archive was run (D-10). Nothing was pushed.

## Self-Check: PASSED
