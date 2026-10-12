---
phase: 37-puzzle-core-in-the-bridge
verified: 2026-10-12T01:07:19Z
status: passed
score: 44/44 must-haves verified (5 roadmap SCs + 39 plan truths)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: 2026-10-12T01:07:19Z
lifecycle_validated: true
overrides_applied: 0
deferred:
  - truth: "GOAL-04 browser half: a recorded solution replays to the same result in a real browser (Phase 37 replays the generated wasm through Bun)"
    addressed_in: "Phase 40"
    evidence: "Phase 40 SC3: 'At least one solution replays to the same result in the browser as natively.' 37-CONTEXT.md Deferred: 'Playwright replay of a solution in a real browser: Phase 40'"
---

# Phase 37: Puzzle Core in the Bridge Verification Report

**Phase Goal:** A level can be built in the bridge from a level id and a placement list, and its cup fill, win and timeout are decided per fixed step in Rust, with the same result natively and in wasm.
**Verified:** 2026-10-12T01:07:19Z
**Status:** passed
**Re-verification:** No (initial verification)

**Final HEAD:** `f11e7bd30`. The last code commit is `74b85a09e`. `git diff 74b85a09e..HEAD` touches only `.planning/` files.

## Goal Achievement

### Roadmap Success Criteria

| # | Success criterion | Status | Evidence |
| --- | --- | --- | --- |
| SC1 | A native test builds first-pour from its level id and a placement list (empty allowed), steps it, and reads count, target, full, outcome, outcome step and step limit from each frame and from `levelInfo()` | VERIFIED | `puzzle::api_tests::first_pour_frames_and_level_info_report_the_goal_contract` builds through `build_puzzle_core("first-pour", &[], &[])`, the same helper `ProofSession::newPuzzle` calls (lib.rs:202-212). It checks every `PuzzleLevelInfo` field against `level_spec(FirstPour)` and checks the running scalars on frames 0 to 10. `puzzle::replay::replay` (replay.rs:276-369) reads all six `ProofFrame` getters on every frame through the latch and after it, for both the empty fixture and the one-plank fixture. It asserts that `outcomeStep` is present exactly when the outcome is latched, and that target, full and limit equal `levelInfo()`. I reran `cargo test -p liquidfun-wasm --lib puzzle`: 104 passed, 2 ignored. |
| SC2 | Wins only after fill holds at or above target for the settle window (30 steps) before the limit. A splash does not win, a run that never holds loses at the limit, and a later slosh does not undo a win | VERIFIED | `GoalTracker::observe` (goal.rs:186-203) resets `held` whenever the count drops below target, wins at `held >= settle_steps && step <= step_limit`, latches `Lost { step: step_limit }` otherwise, and returns early once latched. Unit tests: `splash_shorter_than_settle_does_not_win`, `dip_resets_the_hold`, `never_holding_loses_at_the_step_limit`, `win_survives_a_later_slosh`, `loss_survives_a_later_hold`, `win_completing_at_the_step_limit_wins`. The tracker runs in `PuzzleHooks::on_after_step` after `world.step` (session.rs:215-245), using its own step counter. `capture_frame` fails closed if that counter differs from `step_index` (frame.rs:130). The replay keeps stepping on the same session after the latch, to step 600 and then `advance(4)` twice, and asserts the latch never changes. FIRST_POUR uses `settle_steps: 30` and `step_limit: 720`. |
| SC3 | AAA counting tests: only counted particles with centres inside the interior count. They cover rim, outside-wall, empty, evicted and non-counted-group cases, and fill uses the authored full count clamped to 100% | VERIFIED | goal/tests.rs has 29 tests, each with `// Arrange`, `// Act` and `// Assert`. They include `excludes_particles_on_each_interior_edge`, `excludes_a_particle_just_above_the_rim`, `excludes_a_particle_outside_the_wall`, `empty_lanes_count_zero`, `evicted_particles_lower_the_count_but_not_full`, `excludes_non_counted_groups`, `excludes_ungrouped_particles`, `excludes_non_finite_positions`, `fill_ratio_is_count_over_full` and `fill_ratio_clamps_to_one`. `count_in_goal` uses strict containment and counts only listed groups. Counted groups come from `group_ids()`, read-only. |
| SC4 | The spike runs first and its result is recorded. A reference-solution harness replays a recorded `(level, placements)` fixture to the same outcome and outcome step natively. Angle-snapping and fill-margin policy follow from the spike | VERIFIED | The spike was the first code commit (`d9711bf7e`, before any bridge change). The spike table and the contract are in 37-01-SUMMARY.md and again in 37-08-SUMMARY.md under "Phase 37 Spike Result and Replay Contract". The project has no separate phase-level SUMMARY file, so the final plan summary serves as one. Release native and wasm match on dam-break and water-wheel, and theo-jansen (trig) diverges at step 70. Policy: the contract is exact between release native and wasm, debug native uses the fallback, and static tools use the literal `TAU_24` table with body angle 0. A grep finds no `sin`, `cos`, `from_angle` or `oriented_box` outside tests in `puzzle/`. Fixtures: `first-pour-solution.txt` (won 201 / 689) and `first-pour-empty.txt` (lost 720 / 0). I reran `cargo test -p liquidfun-wasm --release --lib puzzle::replay` (14 passed) and `just puzzle-replay-wasm` (both `REPLAY ... pass`, same outcome, step and final count). |
| SC5 | `just playground-scene-spot` 25/25 unchanged, `crates/liquidfun` untouched, level ids on a separate Rust allowlist outside `SCENE_IDS` | VERIFIED | `target/phase37/gate-08.txt` says `25/25 identical`. Stamp `2026-10-12T00-58-37Z` has `git_head` `410e2111b`, 25 scenes and `full_catalog: true`. `git diff --stat 410e2111b..HEAD -- crates tools Cargo.toml Cargo.lock` is empty, so the stamp covers final HEAD's Rust code. `diff base.txt after-final.txt` is empty, both files have 25 lines, and base comes from phase base `958d6fad7`. `git diff --stat 958d6fad7 -- crates/liquidfun` is empty (rerun). `LevelId`/`LEVEL_IDS`/`parse_level_id` live in puzzle.rs. Disjointness tests run in both directions, plus an `include_str!` mirror test against `web/src/catalog/puzzles.ts`. `git grep first-pour` finds no hits in scenes.ts, scene-records.ts, portrait-bounds.ts, readme-svg plans, scene.rs, scene_spot.rs or tools/xtask. |

### Plan Must-Have Truths (merged)

| Plan | Truths | Status | Notes |
| --- | --- | --- | --- |
| 37-01 | 4 | VERIFIED | `spot-base.jsonl` and `base.txt` have 25 lines, from `phase-base.txt` = `958d6fad7`. `just puzzle-spike` and `spike-report.txt` show 9 COMPARE lines. The shared hash vectors appear in both the native and Bun tests. `crates/liquidfun` is unchanged. |
| 37-02 | 4 | VERIFIED | `parsePuzzleStatus` uses the conditional spread `maybePuzzle === undefined ? {} : { maybePuzzle }`. I reran Vitest on frame-puzzle, frame, session and puzzle-replay: 82 passed. |
| 37-03 | 4 | VERIFIED | The goal core and latch are covered by the SC2 and SC3 evidence. `div_ceil(20)` margin. LevelId allowlist and mirror tests (puzzle/tests.rs, 6 tests). |
| 37-04 | 3 | VERIFIED | `SessionSource` and `escape_limits` replace `scene_id`. `create(SceneId)` keeps its signature. `from_source(self.source.clone()` appears on both recreation branches. Gates 04, 06 and 08 are all `25/25 identical`. |
| 37-05 | 7 | VERIFIED | `Placements::parse` returns typed `PlacementError`s and sorts canonically. Plank vertices index `TAU_24[`. FIRST_POUR is a `const` with target 540, full 900, settle 30, limit 720, one plank and a build area. `build_level` follows the D-09 order: 1989 particles, initial count 0. `PuzzleHooks` rejects controls, actions and pointer input. The default `maybe_puzzle_status` returns `None` for scenes. |
| 37-06 | 5 | VERIFIED (truth 4 amended, see below) | `SessionSource::Puzzle(LevelId, Placements)` and `create_puzzle`. The `is_puzzle()` guards reject controls, presets and gravity. Restore is an `Ok(())` no-op. The status step equals `step_index` after `advance(4)` batches. I reran the ignored probes with `--include-ignored`: 8 passed. Empty loses at 720 with max count 0. The plank wins at 201, and the raised-target tracker wins at 210. |
| 37-07 | 6 | VERIFIED | `newPuzzle` and `levelInfo` appear in the generated d.ts. Six `Option` getters in frame/puzzle.rs. `PuzzleLevelInfo` is re-exported. `set_puzzle_status` checks `observed_step == step_index`. `git grep "Temporary until Plan 37-"` is empty. The remaining `allow(dead_code)` attributes all predate this phase. |
| 37-08 | 6 | VERIFIED | Both fixtures and the strict parsers in Rust and TS. Release exact and debug fallback (`DEBUG_OUTCOME_STEP_TOLERANCE = 5`, measured spread 0). Wasm exact. Post-latch stepping. The record paths only print: Bun writes only `target/puzzle-replay/replay.log`. All 14 final gates are `exit=0` in `checks-08.results`. |

**Score:** 44/44 (5 roadmap SCs, 39 plan truths).

### The Plan 06 won-margin rule change (D-03)

- **Original plan wording (37-06 truth 4):** every count in the latched 30-step window must be at or above target plus the margin (540 + 45 = 585).
- **Why it was replaced:** the window starts at the first count at or above 540, so the original rule would need a one-step inflow larger than 45. The measured one-step maximum was 12 to 30. As written, the rule could not be met. Literally, it still fails: the lowest count in the window is 541.
- **Replacement (Option A, recorded in 37-06-SUMMARY.md and carried into 37-08-PLAN.md truth 2):** a second `GoalTracker` with target 585 and the same 30-step settle window must reach Won by step `720 - 120 = 600`. Measured: it wins at step 210, identically in debug, release and wasm.
- **Judgment: it meets SC4 and D-03.**
  - D-03's fallback asks that the solution win "by a fill margin of at least +5% of `full_count` over target". The raised-target rule shows exactly that: the run holds at least 585 for the full settle window before the limit. Equivalently, a run whose count was 45 lower at every step would still win, with 120 steps to spare. That is the robustness D-03 is after.
  - The loss side is unchanged and is met with a wide margin: the empty run's maximum count is 0, below the ceiling of 495.
  - The margin only adds a check. The spike gave the "identical" branch, so the binding release-vs-wasm contract is exact outcome, step and final count, and that contract holds.
- **Override:** none needed. The roadmap SC and the locked decision D-03 are both met, and the plan truth was amended during the phase by a documented decision. If the owner wants the superseded plan wording on record anyway, an `overrides:` entry naming 37-06 truth 4 would do it.

### Required Artifacts

| Artifact | Status | Details |
| --- | --- | --- |
| `crates/liquidfun-wasm/src/puzzle.rs` + `puzzle/{angles,builder,first_pour,goal,hooks,info,spec,tools}.rs` | VERIFIED | Exist, substantive (55-305 lines each), wired from session.rs and lib.rs |
| `puzzle/{api_tests,session_tests,tests,replay}.rs`, `goal/tests.rs`, `tools/tests.rs`, `builder/tests.rs`, `replay/tests.rs` | VERIFIED | `cfg(test)` modules inside the lib test binary; no `crates/liquidfun-wasm/tests` directory (D-20) |
| `puzzle/fixtures/first-pour-{solution,empty}.txt` | VERIFIED | D-18 grammar, read by the Rust tests and the Bun script |
| `crates/liquidfun-wasm/src/frame/puzzle.rs`, `frame.rs`, `session.rs`, `session/escape.rs`, `lib.rs` | VERIFIED | Wired; all files ≤ 545 lines |
| `crates/liquidfun-wasm/src/scene_spot/frame_hash.rs` | VERIFIED | Test-only spike hash |
| `web/src/physics/frame.ts`, `web/tests/frame-puzzle.test.ts` | VERIFIED | `maybePuzzle` parsing with Vitest coverage |
| `web/src/catalog/puzzles.ts` | VERIFIED | `PUZZLE_IDS = ["first-pour"]`. Not imported by web code yet, as intended; Phase 38 owns routing. Read by the Rust mirror test. |
| `web/scripts/puzzle-replay.ts`, `puzzle-replay/{fixture,hash,spike,wasm}.ts`, `web/tests/puzzle-replay.test.ts` | VERIFIED | Wired through `just puzzle-replay-wasm` and `just puzzle-spike` |
| `scripts/puzzle-spike.sh`, `justfile` recipes | VERIFIED | `set -euo pipefail`; the recipes exist |

gsd-tools `verify artifacts` passed for every plan (29/29).

### Key Link Verification

gsd-tools could not resolve the symbol-style `from` fields, so I grepped every pattern by hand. All 17 links are WIRED:

| From | To | Status |
| --- | --- | --- |
| puzzle-replay.ts spike / replay | generated wasm `ProofSession` / `newPuzzle` | WIRED (`loadGeneratedWasm` ×3, `newPuzzle` at :220) |
| puzzle-spike.sh | `spike_prints_per_step_frame_hashes` | WIRED |
| parseRenderFrame | parsePuzzleStatus | WIRED (conditional spread) |
| advance / advance_profiled | evict_escaped_particles(`self.escape_limits`) | WIRED (×2) |
| apply_control | `from_source(self.source.clone()` | WIRED (×2) |
| Placement::vertices | `TAU_24[` | WIRED |
| PuzzleHooks::on_after_step | `count_in_goal` + `GoalTracker::observe` | WIRED |
| build_level | `vertices()` | WIRED |
| from_source Puzzle arm | `build_level(spec, placements)` | WIRED |
| apply_control / set_gravity | `is_puzzle` → `UnknownControl` | WIRED |
| capture_frame | `set_puzzle_status` | WIRED |
| new_puzzle | `build_puzzle_core` → `create_puzzle` | WIRED |
| replay.rs replay | `build_puzzle_core` + ProofFrame getters | WIRED |
| puzzle-replay.ts replay | fixtures directory / `maybePuzzle` | WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Real data | Status |
| --- | --- | --- | --- | --- |
| ProofFrame puzzle getters | `maybe_puzzle: Option<PuzzleStatus>` | `hooks.maybe_puzzle_status()`, from `count_in_goal` over the live `particle_system_view` lanes each step | Yes. The count rises to 689 on the plank fixture and stays at 0 on the empty fixture. | FLOWING |
| PuzzleLevelInfo | `&'static LevelSpec` | `level_spec(LevelId)` const | Yes. Checked field by field against FIRST_POUR. | FLOWING |
| TS `maybePuzzle` | six getters | the wasm `ProofFrame` | Yes. The Bun replay parses every frame. | FLOWING |

### Behavioral Spot-Checks (run by me at HEAD)

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Puzzle unit and API tests (debug) | `cargo test -p liquidfun-wasm --lib puzzle` | 104 passed, 2 ignored | PASS |
| Release replay, exact contract | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` | 14 passed | PASS |
| Wasm replay, exact contract | `just puzzle-replay-wasm` | `REPLAY first-pour-solution.txt pass (outcome won step 201 final_count 689)`, `REPLAY first-pour-empty.txt pass (outcome lost step 720 final_count 0)` | PASS |
| First-pour tuning probes | `cargo test -p liquidfun-wasm --lib puzzle::session_tests -- --include-ignored` | 8 passed | PASS |
| Web frame and replay tests | `bunx vitest run tests/frame-puzzle.test.ts tests/puzzle-replay.test.ts tests/frame.test.ts tests/session.test.ts` | 82 passed | PASS |
| Engine untouched | `git diff --stat 958d6fad7 -- crates/liquidfun` | empty | PASS |
| 25/25 fingerprints | recorded gate (stamp at `410e2111b`; no Rust or tools diff to HEAD) | `25/25 identical`, `diff` empty | PASS (recorded; valid for final code) |
| Registry isolation | `git grep first-pour` over the scene registries, scene.rs, scene_spot.rs and xtask | no hits | PASS |
| Step cap | grep `MAX_ADVANCE_STEPS: u32 = 4` | present | PASS |

The full fmt, clippy, build, test and bright-builds-check gates are recorded as `exit=0` in `target/phase37/checks-08.results`, which was stamped against the final code. I did not rerun them.

### Requirements Coverage

| Requirement | Source plans | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| GOAL-02 | 37-02, 37-03, 37-05, 37-06, 37-07 | Win when fill holds at or above target for about 0.5 s before the limit; a splash does not win | SATISFIED (bridge core) | SC2 evidence: the Rust latch, its unit tests, the frame scalars and TS parsing. The on-screen result is Phase 38 (GOAL-03, PUZ-*). |
| GOAL-04 | 37-01, 37-04, 37-05, 37-06, 37-08 | Same outcome regardless of device speed; a recorded solution replays to the same result natively and in the browser | SATISFIED for native and wasm; real-browser replay deferred | Fixed-step decision with no wall clock and `MAX_ADVANCE_STEPS` 4. Exact replay in release native and in wasm (Bun, the same generated package the browser loads). The Playwright replay in a browser is Phase 40 SC3 (see deferred). |

Every requirement ID in the plan frontmatter is accounted for. REQUIREMENTS.md maps only GOAL-02 and GOAL-04 to Phase 37, so there are no orphans. Both are still `[ ]` / `Pending` in REQUIREMENTS.md. The plans leave marking them complete to phase verification or the orchestrator, and I did not edit that file.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| (all changed files) | n/a | TODO, FIXME or placeholder | none found | n/a |
| session.rs:35,82,533; scene.rs:141; frame.rs:37 | n/a | `allow(dead_code)` | Info | All predate this phase (present at `958d6fad7`); none added |

### Code Review (37-REVIEW.md) Disposition

None of the findings blocks the phase goal.

- **WR-01 (JS typed-array coercion before the Rust checks):**
  - **Why it does not block:** there is no production caller yet. The only caller, the fixture path, validates integers before building lanes. Rust fails closed on everything it receives.
  - **Where it belongs:** the TS placement encoder and validator parity belong to Phase 39 (SC2, "The Rust validator rejects the same placements"), and Phase 38 adds the first player caller.
  - **Recommendation:** carry WR-01 into Phase 38/39 planning as a shared `encodePlacementLanes` encoder.
- **IN-01 (different parser diagnostics):** cosmetic. Rust and TS accept and reject the same inputs.
- **IN-02 (Rust does not require `Lost` at `step_limit`):** the tracker always produces it, so this is only defense in depth.
- **IN-03 (a mid-batch failure leaves the counter ahead):** it fails closed, and D-10 already requires a rebuild after any failure.
- **IN-04 (zip truncation):** engine lanes are aligned by construction.
- **IN-05 and IN-06:** tooling and message robustness only.

### Human Verification Required

None. The phase delivers the bridge core and tooling, with no UI. Every criterion is checked by deterministic native, release, wasm and fingerprint commands. The real-browser replay is deferred to Phase 40 by the roadmap.

### Gaps Summary

There are no gaps. A level is built from `(level id, placements)` through `newPuzzle` / `build_puzzle_core`. Cup count, settle-window win and timeout are latched per fixed step in Rust. The recorded solution and empty fixtures replay to the same outcome, outcome step and final count in release native and in wasm, and within the fallback contract in debug, where the measured spread is 0. The 25 scenes are unchanged and `crates/liquidfun` is untouched.

Two notes for later phases:

1. The Plan 06 margin rule now uses a raised target. It meets D-03.
1. WR-01 should feed Phase 38/39's TS placement encoder.

---

_Verified: 2026-10-12T01:07:19Z_
_Verifier: Claude (gsd-verifier)_
