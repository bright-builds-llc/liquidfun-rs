---
phase: 34-scene-timing-survey
verified: 2026-10-07T09:08:50Z
status: passed
score: 4/4 roadmap success criteria verified (11/11 merged must-haves)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 34-2026-10-07T05-23-23
generated_at: 2026-10-07T09:08:50Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 34: Scene Timing Survey Verification Report

**Phase Goal:** Maintainers can rank every playground scene by native simulation cost with one command.
**Verified:** 2026-10-07T09:20:00Z
**Status:** passed
**Re-verification:** No. This is the initial verification.

Lifecycle note: CONTEXT.md and both PLAN.md files share `lifecycle_mode: yolo` and `phase_lifecycle_id: 34-2026-10-07T05-23-23`. Neither SUMMARY.md has `generated_by`, `lifecycle_mode` or `phase_lifecycle_id` frontmatter. Because the provenance chain is incomplete, `lifecycle_validated` is false. This is a bookkeeping gap, not a goal gap.

Orchestrator follow-up (2026-10-07T09:10Z): the missing SUMMARY.md provenance fields were added with their true values (`generated_by: gsd-executor`, this lifecycle id, `generated_at` taken from each file's commit time). The verifier's `verified`/`generated_at` timestamp was corrected to its actual commit time. `gsd-tools verify lifecycle 34 --require-plans --require-verification` then accepted context, plans, summaries and verification, so `lifecycle_validated` is now true. The goal verdict is unchanged.

## Goal Achievement

### Observable Truths (roadmap success criteria)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | `just playground-scene-spot` steps every catalog scene, and a test fails when a catalog scene is missing from its list. | VERIFIED | The justfile recipe (lines 171-172) is still the one line `cargo xtask playground scene-spot`. `spot.rs` runs the `playground-scene-spot` bin with `--warmup/--steps/--runs`. `SURVEY_SCENES` lists all 25 ids in catalog order. I ran the release bin with `--warmup 0 --steps 1 --runs 1` and it printed 25 scene lines, with wave-machine first. `survey_scenes_match_catalog_scene_ids` compares SURVEY_SCENES with `SCENE_IDS` read through `include_str!` from `web/src/catalog/scenes.ts`. I simulated drift on a scratch copy with an extra `fake-scene`: the same comparison gives unequal, so the test would fail. xtask also rejects bin output whose set differs from the parsed catalog, listing missing and unexpected ids. |
| 2 | The output reports, per scene, median ms/step over repeated runs and the live particle count. | VERIFIED | Each run builds a fresh `SessionCore`, and `summarize` computes median/min/max over `runs` (default 3). Particle counts come from `live_particle_count()`, both start and end. The xtask stdout table has the columns Rank, Scene, Median/Min/Max ms/step, Start particles, End particles and Interaction. The stamp JSON records runs=3. |
| 3 | Scenes that are idle by default get a small scripted interaction so their timing reflects real use, and the table says which. | VERIFIED | `SurveyCue` applies only to float-or-sink (`drop-body`), impulse (pointer up at 1, 2) and drawing-particles (pointer up at 0, 2), after the start snapshot and before warmup. The cues live only in scene_spot.rs; nothing under `src/scene/` or in session.rs changed since 338b6632e. The Interaction column labels these three `scripted`. The roadmap names Sparky as an example, but D-10 deliberately kept it `default`, and the recorded run supports that: Sparky goes from 0 to 550 particles with no input, so it is not idle. `only_idle_scenes_are_scripted` and `survey_cues_apply_to_fresh_sessions` pass. |
| 4 | `docs/benchmarks/scene-survey.md` records the ranked table with commit, machine and command. | VERIFIED | The doc has a 25-row ranked table plus commit `96a3ac6a6…`, macos/aarch64, Apple M4 Max, 16 cores, rustc 1.97.0, `just playground-scene-spot` and the expanded cargo command with 60/120/3. I compared every row with `target/dam-break-perf/2026-10-07T06-20-54Z/scene-spot.json` (rank, values to 3 decimals, particles, interaction): 0 mismatches. The JSON `git_head` equals the recorded commit. Since 96a3ac6a6, `crates tools web justfile` changed only in c79b211ad (new error variants, a test, and CLI-test assertions), and none of those changes affect timing. |

**Score:** 4/4 roadmap truths verified. The plan-level must-haves (7 in Plan 01, 5 in Plan 02) all hold, as covered above and in the tables below.

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene_spot.rs` | SURVEY_SCENES, SurveyCue, per-run timing, spread, JSON line | VERIFIED | 381 lines. Exported through lib.rs and called by the bin. No unwrap or TODO. |
| `crates/liquidfun-wasm/src/scene_spot/tests.rs` | Catalog coverage test plus cue, summarize and JSON tests | VERIFIED | 10 tests pass, including `survey_scenes_match_catalog_scene_ids`. |
| `crates/liquidfun-wasm/src/bin/playground_scene_spot.rs` | `--runs` flag | VERIFIED | Parses `--runs` (default `DEFAULT_RUNS`) and passes it to `run_scene_spot`. |
| `tools/xtask/src/playground/spot.rs` | Catalog-backed validation, `--runs` passthrough, ranked stdout | VERIFIED | `CATALOG_SCENES_TS` comes from `include_str!`. `REQUIRED_SCENES` is gone. `--runs` is parsed, zero is rejected, and the value is passed to the bin. |
| `tools/xtask/src/playground/spot/survey_table.rs` | catalog_scene_ids, rank_scene_samples, render_ranked_table | VERIFIED | Pure core. Orders slowest first with a scene-id tiebreak. |
| `tools/xtask/tests/fixtures/fake_upstream_tool.rs` | Fake output for every catalog scene | VERIFIED | Reads the catalog through `include_str!` and emits `median_ms_per_step`. |
| `docs/benchmarks/scene-survey.md` | Ranked table with provenance | VERIFIED | Matches the stamp exactly. |
| `docs/playground-scene-spot-check.md` | Pointer to the survey, five-scene table marked historical | VERIFIED | Links `(benchmarks/scene-survey.md)` and labels the sample and the reproduce text historical. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| scene_spot/tests.rs | web/src/catalog/scenes.ts | `include_str!` coverage test | WIRED | The test passes. Drift simulation shows it would fail. |
| xtask spot.rs | web/src/catalog/scenes.ts | `include_str!` plus `catalog_scene_ids` | WIRED | `run()` passes the parsed catalog to `validate_scene_samples`. |
| xtask spot.rs | playground_scene_spot bin | `cargo run --release --bin playground-scene-spot -- --warmup --steps --runs` | WIRED | Args are built in `run_spot_bin`. The CLI test with `--runs 1` asserts `report["runs"] == 1`. |
| scene_spot.rs | session.rs | create, apply_action, apply_pointer, advance(1), live_particle_count | WIRED | `time_run` and `apply_cue` call each of them. |
| justfile | xtask | `cargo xtask playground scene-spot` | WIRED | `justfile_keeps_the_one_line_playground_scene_spot_alias` passes. |
| scene-survey.md | scene-spot.json stamp | values copied from the stamp | WIRED | Row-by-row comparison found 0 mismatches. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| xtask ranked table | `ranked` | Bin stdout JSON, from real `SessionCore` timing | Yes. The bin smoke run printed real ms values and particle counts. | FLOWING |
| scene-survey.md table | Rows | Stamp 2026-10-07T06-20-54Z | Yes. The values match the stamp, and the stamp `git_head` is the clean committed source. | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Bin times every catalog scene | `cargo run -p liquidfun-wasm --release --quiet --bin playground-scene-spot -- --warmup 0 --steps 1 --runs 1` | 25 lines. First is wave-machine. Only float-or-sink, impulse and drawing-particles are `scripted`. | PASS |
| Coverage, cue and spread unit tests | `cargo test -p liquidfun-wasm --lib scene_spot` | 10 passed | PASS |
| xtask CLI (justfile pin, stamp, ranked stdout) | `cargo test -p xtask --test playground_cli spot` | 2 passed | PASS |
| xtask validation and parse unit tests | `cargo test -p xtask --bin xtask spot` | 15 passed | PASS |
| Catalog drift detection | Scratch copy of scenes.ts with an extra id, compared with the SURVEY_SCENES ids | real catalog equal=True, catalog with fake id equal=False | PASS |
| Doc matches the recorded stamp | Python row-by-row comparison of the doc against scene-spot.json | 25 rows, 0 mismatches | PASS |

The orchestrator also reports that the full local suite (fmt, clippy, build, tests, markdown-check, bright-builds checks) passed on bbc2ec990, and that the targeted checks passed after c79b211ad.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| PERF-07 | 34-01, 34-02 | Maintainers can time every playground catalog scene natively with one command and see the scenes ranked by median ms/step, with a check that fails when a catalog scene is missing. | SATISFIED | Truths 1, 2 and 4. REQUIREMENTS.md maps PERF-07 to Phase 34 and marks it Complete. No orphaned IDs. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| (none) | - | No TODO, FIXME, unwrap() or placeholder in the phase source files | - | - |
| tools/xtask/src/playground/spot.rs | 21-23 | Defaults 60/120/3 are duplicated with scene_spot.rs (review IN-03, open) | Info | No behavior impact today, because xtask always passes explicit flags. |
| survey_table.rs, scene_spot/tests.rs, fake_upstream_tool.rs | - | The SCENE_IDS parser exists in three copies (review IN-04, open, deliberate) | Info | A catalog format change needs all three updated. The wasm test fails loudly if that is missed. |
| tools/xtask/src/playground/spot.rs | validate_scene_sample | Per-sample runs/steps are not compared with the requested counts (review IN-06, open) | Info | Low risk, because `cargo run` rebuilds the bin. |

### Human Verification Required

None. Every success criterion can be checked from repo artifacts, the committed stamp and non-destructive commands.

### Gaps Summary

There are no gaps. One command (`just playground-scene-spot`) times all 25 catalog scenes over 3 fresh runs each and prints a slowest-first table with median/min/max ms/step, live start and end particle counts, and a `default`/`scripted` label. A unit test, plus xtask validation, fails on catalog drift. The committed survey page matches a real stamp from clean committed source. Sparky stays `default`, unlike the roadmap's illustrative example, by locked decision D-10, and the data shows it is not idle. Review info items IN-03, IN-04 and IN-06 are still open as advisory notes only.

---

_Verified: 2026-10-07T09:20:00Z_
_Verifier: Claude (gsd-verifier)_
