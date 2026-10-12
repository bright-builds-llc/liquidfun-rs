---
phase: 37-puzzle-core-in-the-bridge
plan: "08"
subsystem: puzzle-replay
tags: [rust, typescript, wasm, replay, fixtures, determinism, gates, GOAL-04]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-12T00:58:51.077Z"
requires:
  - crate::build_puzzle_core, SessionCore::{advance, capture_frame, step_index, level_info} (Plan 07)
  - ProofFrame goal getters and PuzzleLevelInfo (Plan 07)
  - goal::{maybe_margin_win_step, MARGIN_WIN_SLACK_STEPS, fill_margin_count} (Plans 03, 06)
  - parseRenderFrame(...).maybePuzzle (Plan 02)
  - loadGeneratedWasm, repoRoot and spike_contract exact (Plan 01)
provides:
  - test-only puzzle::replay (parse_fixture, lanes, replay, check_contract, DEBUG_OUTCOME_STEP_TOLERANCE = 5)
  - recorded fixtures first-pour-solution.txt (won 201 / 689) and first-pour-empty.txt (lost 720 / 0)
  - web/scripts/puzzle-replay/fixture.ts (parseFixture, fixtureLanes, marginWinStep, checkReplay, WASM_REPLAY_CONTRACT exact)
  - "`bun scripts/puzzle-replay.ts replay [--record]` and `just puzzle-replay-wasm`"
  - final Phase 37 gate evidence (target/phase37/checks-08.results, gate-08.txt)
affects:
  - 38 (player shell can reuse fixture.ts contract helpers and the replay recipe as a regression guard)
tech-stack:
  added: []
  patterns:
    - one strict line grammar implemented in Rust and TS with shared test vectors
    - print-only re-record paths (LIQUIDFUN_PUZZLE_RECORD=1, --record)
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/replay.rs
    - crates/liquidfun-wasm/src/puzzle/replay/tests.rs
    - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-empty.txt
    - web/scripts/puzzle-replay/fixture.ts
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - web/scripts/puzzle-replay.ts
    - web/scripts/puzzle-replay/wasm.ts
    - web/tests/puzzle-replay.test.ts
    - justfile
key-decisions:
  - "Release-native, debug-native and wasm all replay first-pour identically (won 201 / 689, lost 720 / 0); release and wasm keep the exact contract, debug asserts the fallback with DEBUG_OUTCOME_STEP_TOLERANCE = max(5, 2 * 0) = 5"
  - "WASM_REPLAY_CONTRACT stays exact with tolerance 0, confirmed by wasm RECORD lines equal to release"
  - "Both parsers check i32 / u32 ranges after the regex check, so Rust and TS accept exactly the same fixture text"
requirements: [GOAL-04]
duration: 16min
completed: 2026-10-12
---

# Phase 37 Plan 08: Reference-solution replay harness and final gates Summary

**Two recorded first-pour fixtures (won at step 201 with 689 particles in the cup, lost at the 720-step limit with 0) replay exactly in release native and in the generated wasm, and identically in debug native. All fourteen final gates pass, including 25/25 scene fingerprints and an untouched `crates/liquidfun`.**

## Performance

- **Duration:** about 16 min (2026-10-12T00:43Z to 00:59Z), including two full check runs (about 4.5 min each)
- **Tasks:** 3 of 3
- **Files:** 10 (5 created, 5 modified)

## Phase 37 Spike Result and Replay Contract

Spike table from 37-01-SUMMARY.md (HEAD `958d6fad7`, 300 steps of `advance(1)` + `captureFrame()`, FNV-1a 64 over particle positions, rigid segments, rigid circles):

| Scene | release-vs-wasm | debug-vs-release | debug-vs-wasm |
| --- | --- | --- | --- |
| dam-break | identical | diverges at step 91 | diverges at step 91 |
| water-wheel | identical | diverges at step 39 | diverges at step 39 |
| theo-jansen | diverges at step 70 | diverges at step 2 | diverges at step 2 |

Final contracts:

| Runtime | Contract | Evidence |
| --- | --- | --- |
| Release native (`cargo test -p liquidfun-wasm --release --lib puzzle::replay`) | **exact**: same outcome, outcome step and final count, plus the margin | 14 passed |
| Debug native (`cargo test -p liquidfun-wasm --lib puzzle::replay`) | **fallback**: same outcome, margin, outcome step within `DEBUG_OUTCOME_STEP_TOLERANCE = 5` | 14 passed; measured spread 0 |
| Wasm (`just puzzle-replay-wasm`) | **exact** (`WASM_REPLAY_CONTRACT = "exact"`, tolerance 0) | both fixtures `REPLAY ... pass` |

The margin rule in every contract is the Plan 06 raised-target rule: a won trace must still win in a second `GoalTracker` at `target + full.div_ceil(20)` = 540 + 45 = 585 with the same 30 settle steps, by step 720 - 120 = 600. A lost trace must latch at the step limit with every count at or below 540 - 45 = 495.

The debug tolerance (and any future wasm fallback tolerance) is measured on first-pour, not taken from the spike scenes. First-pour is the only level and the fixture being replayed. Its tools are static, at body angle 0, with τ/24 baked vertices, so no trig runs during the replay. The spike scenes have dynamic rotating bodies (water-wheel, theo-jansen's motorized linkage) and other particle counts. Their first divergence steps describe the frame hash of a different system, not the outcome-step spread of a level. On first-pour the debug and release traces latch at the same step with the same count. The tolerance is therefore the floor max(5, 2 × 0) = 5.

## Fixtures (verbatim)

`crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt`:

```
level first-pour
placement plank 8 64 23
outcome won
outcome_step 201
final_count 689
```

`crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-empty.txt`:

```
level first-pour
outcome lost
outcome_step 720
final_count 0
```

## RECORD and REPLAY Lines

`target/phase37/record-release.txt`:

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 profile release
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 profile release
```

`target/phase37/record-debug.txt`:

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 profile debug
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 profile debug
```

`target/phase37/record-wasm.txt`:

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 profile wasm
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 profile wasm
```

`target/puzzle-replay/replay.log` (from `just puzzle-replay-wasm`):

```
REPLAY first-pour-solution.txt pass (outcome won step 201 final_count 689)
REPLAY first-pour-empty.txt pass (outcome lost step 720 final_count 0)
```

Measured spreads: debug vs release 0 steps (both fixtures), wasm vs release 0 steps (both fixtures).

Negative checks (run, then reverted): changing the solution's `outcome_step` to 202 fails the release test with `outcome step 201 differs from recorded 202`. Changing `final_count` to 688 makes the wasm replay print `REPLAY first-pour-solution.txt fail: final count 689 differs from recorded 688` and exit 1.

## Accomplishments

- **Native harness (`puzzle/replay.rs`, cfg(test)):**
  - A strict D-18 parser with `fixture line N: ...` errors. It rejects blank lines, `\r`, unknown keys, out-of-order expectations, unknown levels and tools, and malformed or out-of-range integers.
  - `lanes` flattens placements in fixture order.
  - `replay` drives `build_puzzle_core` with `advance(1)` and `ProofFrame` getters. It checks every frame's target, full count, step limit and step index against `level_info`. After the latch it keeps the same session to step 600, then runs two `advance(4)` calls, and panics if the latch changes.
  - `check_contract` implements the exact and fallback forms.
- **Bun harness:**
  - `fixture.ts` mirrors the parser, the lanes, `marginWinStep` (a `GoalTracker` mirror) and `checkReplay`.
  - `puzzle-replay.ts replay [--record]` uses `ProofSession.newPuzzle`, reads `levelInfo()` and frees it, then parses every frame through `parseRenderFrame(...).maybePuzzle` and frees it.
  - Every printed line is logged to `target/puzzle-replay/replay.log`. The script exits 1 on any failure.
- **Re-record paths:** `LIQUIDFUN_PUZZLE_RECORD=1` (native) and `--record` (Bun) only print `RECORD` lines. Fixtures were unchanged after every record run.
- **Recipe:** `just puzzle-replay-wasm` rebuilds wasm, then replays.

## Task Commits

1. **Task 1: Native parser, replay, fixtures and contract.** `410e2111b` (`test(37-08): replay recorded first-pour fixtures natively`): puzzle.rs, puzzle/replay.rs, puzzle/replay/tests.rs, both fixtures.
1. **Task 2: TS parser, Bun wasm replay and recipe.** No separate commit (the plan commits these files in Task 3).
1. **Task 3: Final gates and commit.** `74b85a09e` (`test(37-08): replay puzzle fixtures through wasm and close phase gates`): justfile, web/scripts/puzzle-replay.ts, web/scripts/puzzle-replay/{fixture,wasm}.ts, web/tests/puzzle-replay.test.ts.

## Final Gates (`target/phase37/checks-08.results`)

| # | Check | Result |
| --- | --- | --- |
| 1 | `cargo fmt --all --check` | exit=0 |
| 2 | `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit=0 |
| 3 | `cargo build -p liquidfun-wasm --all-targets --all-features` | exit=0 |
| 4 | `cargo test -p liquidfun-wasm --all-features` (425 passed, 3 ignored) | exit=0 |
| 5 | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` (14 passed) | exit=0 |
| 6 | `just puzzle-replay-wasm` (2 REPLAY pass) | exit=0 |
| 7 | `cd web && bun run typecheck && bun run test:unit` (62 files, 561 tests) | exit=0 |
| 8 | `bun scripts/bright-builds-check.ts all` | exit=0 |
| 9 | 25/25 fingerprints: stamp `2026-10-12T00-58-37Z`, `diff base.txt after-final.txt` empty, 25 lines | exit=0 |
| 10 | `git diff --stat 958d6fad7 -- crates/liquidfun` empty | exit=0 |
| 11 | no `first-pour` in scenes.ts, scene-records.ts, portrait-bounds.ts, readme-svg plans, scene.rs, scene_spot.rs | exit=0 |
| 12 | `MAX_ADVANCE_STEPS: u32 = 4` | exit=0 |
| 13 | no `Temporary until Plan 37-` markers and no `crates/liquidfun-wasm/tests` | exit=0 |
| 14 | no non-GSD Markdown changed since the phase base | exit=0 |

**25/25 result:** `25/25 identical` (`target/phase37/gate-08.txt`).

Before the Task 1 commit, `checks-08a.results` also recorded fmt, clippy, build, test (425 passed, 3 ignored) and bright-builds-check at exit=0. `cargo test -p liquidfun-wasm --lib puzzle::session_tests` still reports 6 passed and 2 ignored.

File lengths: replay.rs 487, replay/tests.rs 308, puzzle-replay.ts 285, fixture.ts 379, puzzle-replay.test.ts 400 lines, all within the checker's limit.

## Deviations from Plan

- **Range checks in both parsers.** The plan's TS rule says integers are "safe integers". To keep both parsers accepting exactly the same text, the TS parser also bounds placement integers to i32 and expectation integers to u32, matching Rust's `str::parse` targets.
- **Task 2 RED/GREEN not committed separately.** The plan places all five web/justfile files in the Task 3 commit, so the RED test run (module not found) and the GREEN run (20 passed) were verified but not committed on their own.
- **Vitest count.** `tests/puzzle-replay.test.ts` has 20 cases: 7 from Plan 01 (the plan's text counts 6) plus 13 here.
- `generated_by` is `gsd-execute-phase`, as the orchestrator asked. The plan's output section says `gsd-executor`.
- GOAL-04 is not marked complete in REQUIREMENTS.md, as instructed. Phase verification owns that step.

Otherwise the plan was executed as written.

## Known Stubs

None.

## Threat Flags

None. The new surfaces are test-only fixtures and developer scripts covered by T-37-31 to T-37-37. Each mitigation is implemented: print-only record paths, a strict grammar, a contract that fails closed, the latch checks, a rebuild before replay, and the fingerprint and engine gates.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/puzzle/replay.rs, replay/tests.rs, fixtures/first-pour-solution.txt, fixtures/first-pour-empty.txt, web/scripts/puzzle-replay/fixture.ts
- FOUND: commits 410e2111b and 74b85a09e
- FOUND: target/phase37/checks-08.results (14 lines, all exit=0), gate-08.txt, after-final.txt (25 lines), record-release.txt, record-debug.txt, record-wasm.txt, target/puzzle-replay/replay.log (2 pass lines)
