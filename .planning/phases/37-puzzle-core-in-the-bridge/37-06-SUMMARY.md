---
phase: 37-puzzle-core-in-the-bridge
plan: "06"
subsystem: wasm-session
tags: [rust, session, puzzle, first-pour, margin, GOAL-02, GOAL-04]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-12T00:30:49.748Z"
requires:
  - crate::puzzle::{level_spec, builder::build_level, tools::Placements, first_pour::REFERENCE_PLANK_POSE} (Plan 05)
  - crate::puzzle::goal::{GoalTracker, Outcome, PuzzleStatus, fill_margin_count} (Plan 03)
  - SessionSource and EscapeLimits::SHARED (Plan 04)
provides:
  - SessionSource::Puzzle(LevelId, Placements) and SessionCore::create_puzzle
  - Puzzle rejection of apply_control (before the gravity preset) and set_gravity; restore_authored_gravity Ok(()) no-op
  - cfg(test) SessionCore::maybe_puzzle_status
  - cfg(test) goal::{maybe_margin_win_step, MARGIN_WIN_SLACK_STEPS} (raised-target margin rule)
  - puzzle::session_tests (6 default tests + 2 ignored tuning probes)
affects:
  - 37-07 (wires newPuzzle; removes the temporary dead_code markers, including the two lines on the SessionSource::Puzzle variant)
  - 37-08 (check_contract / checkReplay use the raised-target rule; replays record counts to step_limit - 120; 37-08-PLAN.md edited)
tech-stack:
  added: []
  patterns:
    - margin proven by replaying a count trace through a second, raised-target GoalTracker instead of a window minimum
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/session_tests.rs
  modified:
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/goal.rs
    - crates/liquidfun-wasm/src/puzzle/goal/tests.rs
    - .planning/phases/37-puzzle-core-in-the-bridge/37-08-PLAN.md
key-decisions:
  - "Won margin rule (Option A, coordinator decision): a second GoalTracker at target_count + full_count.div_ceil(20) with the same settle_steps must reach Won by step_limit - 120; outcome, outcome step and the lost rule are unchanged"
  - "first-pour needed no tuning: empty loses at 720 with max count 0, reference plank [8, 64, 23] wins at 201 and the raised tracker at 210, identical in debug and release"
  - "Puzzle sessions reject apply_control at the top (before the gravity preset) and set_gravity with UnknownControl; restore_authored_gravity is an Ok(()) no-op so the TS session is never disposed"
requirements: [GOAL-02, GOAL-04]
duration: 22min
completed: 2026-10-12
---

# Phase 37 Plan 06: Puzzle sessions from level and placements Summary

**`SessionCore` now builds puzzle sessions from `(LevelId, Placements)` at step 0 and rejects every scene-only input. First-pour meets D-07 natively in both profiles without any geometry change: the empty run loses with an empty cup, and the reference plank wins at step 201. It also wins with the target raised by the +5% margin, at step 210.**

## Performance

- **Duration:** about 22 min (2026-10-12T00:09Z to 00:31Z), including a stop for the margin-rule decision. Two full check runs took about 4.5 min each.
- **Tasks:** 3 of 3
- **Files:** 5 code files (1 created, 4 modified) plus the Plan 08 edit

## First-pour Tuning Result (D-07, D-03)

`first_pour.rs` is **unchanged** from Plan 05:

| Constant | Value |
| --- | --- |
| Walls | 6 quads, with a 0.4 m outlet at x -1.0..-0.6 under the reservoir |
| Cup walls | (1.4, 0)..(1.6, 1.8) and the backstop (3.4, 0)..(3.6, 3.4) |
| Cup interior | (1.6, 0)..(3.4, 1.8) |
| Liquid block | (-2.96, 4.84)..(-0.64, 7.9) at a 0.06 m stride |
| Counts | `target_count` 540, `full_count` 900 (margin 45) |
| Steps | `settle_steps` 30, `step_limit` 720 |
| Tools | `build_area` (-20, 44)..(40, 84), budget of 1 plank |
| `REFERENCE_PLANK_POSE` | `[8, 64, 23]`, a centre of (0.4, 3.2) m at -15° |

| Measure | Debug | Release |
| --- | --- | --- |
| Particle count | 1989 | 1989 |
| Empty run outcome | `Lost { step: 720 }` | `Lost { step: 720 }` |
| Empty run max count (ceiling 540 - 45 = 495) | **0** | **0** |
| Reference plank win step | **201** | **201** |
| Raised-target (585) tracker win step, deadline 600 | **210** | **210** |
| Lowest count in the latched settle window (steps 172..201) | 541 | 541 |
| Outcome after stepping to 600 plus two `advance(4)` calls | `Won { step: 201 }` | `Won { step: 201 }` |

The debug and release goal-count traces are identical through step 360. Both probes pass in both profiles with `--include-ignored`. The default suite reports 6 passed and 2 ignored.

## Fingerprint Gate (D-21)

- **Result:** `25/25 identical` (`target/phase37/gate-06.txt`).
- **Runs:**
  - Before the Task 1 commit: stamp `2026-10-12T00-16-16Z`.
  - Again before the Task 2 commit: stamp `2026-10-12T00-26-14Z`, logged in `target/phase37/spot-after-06.log` and compared through `target/phase37/after-06.txt`.

## Accomplishments

- **Session wiring:**
  - `SessionSource::Puzzle(LevelId, Placements)` builds through `build_level(spec, placements)` with `spec.particle_iterations` and `EscapeLimits::SHARED`. The Scene arm is unchanged.
  - `create_puzzle(level, placements)` is the constructor.
- **Input rejection:**
  - `is_puzzle()` guards `apply_control` (before `GRAVITY_CONTROL_NAME`) and `set_gravity`, which return `UnknownControl`.
  - `restore_authored_gravity` returns `Ok(())` for puzzles.
  - `apply_action` and `apply_pointer` reach `PuzzleHooks`, which reject them.
- **Tests:**
  - `cfg(test) maybe_puzzle_status` exposes the hooks' status.
  - Six AAA session tests cover:
    - creation at step 0
    - control and gravity-preset rejection
    - action and pointer rejection
    - live gravity rejection
    - the restore no-op
    - status step equal to `step_index` after three `advance(4)` batches (12)
- **Margin helper:** the pure `goal::maybe_margin_win_step(counts, target_count, full_count, settle_steps, step_limit) -> Option<u32>`, with `MARGIN_WIN_SLACK_STEPS = 120`. Three AAA tests cover the win, the raised threshold, and the deadline boundary on time vs one step late.
- **Probes:** two `#[ignore]`d tuning probes. The win probe continues on the same core that latched, without a second run from step 0.
- **Plan 08 handoff:** `37-08-PLAN.md` now uses the raised-target rule in the Rust `check_contract`, the TS `checkReplay` (via a new `marginWinStep` mirror and `MARGIN_WIN_SLACK_STEPS`), their tests and acceptance criteria. Replays also keep recording counts to `step_limit - 120` before the post-latch `advance(4)` pair.

## Task Commits

1. **Task 1: Puzzle source, `create_puzzle` and input rejections.** `b85ed6ec0` (`feat(37-06): build puzzle sessions from level and placements`): session.rs, puzzle.rs, puzzle/session_tests.rs.
1. **Task 2: First-pour probes with the raised-target margin rule.** `52feaaaee` (`test(37-06): prove first-pour outcomes with a raised-target margin rule`): goal.rs, goal/tests.rs, session_tests.rs.
1. **Task 3: Gate and checks.** I ran these before each commit and recorded the results above and below.

## Check Results (`target/phase37/checks-06.results`, final run)

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | exit=0 |
| `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit=0 |
| `cargo build -p liquidfun-wasm --all-targets --all-features` | exit=0 |
| `cargo test -p liquidfun-wasm --all-features` (397 passed, 3 ignored) | exit=0 |
| `bun scripts/bright-builds-check.ts all` | exit=0 |
| `bun scripts/web-build.ts wasm` | exit=0 |
| `git diff --stat 958d6fad7 -- crates/liquidfun` is empty | exit=0 |

The first run, before the Task 1 commit, also passed all seven (394 passed, 1 ignored).

## Deviations from Plan

### Decision-driven changes

**1. [Rule 4 - Decision, Option A] The won-margin rule is a raised-target tracker, not a settle-window minimum**
- **Found during:** Task 2
- **Issue:** The plan asked that every count in `counts[step - settle_steps .. step]` be at least `target + margin`. The hold starts at the first count at or above `target`, so the cup would have to gain more than the margin (45) in one step. No pour does that:
  - The measured single-step maximum was 12 with the original layout, 26 with a 1.6 m outlet, and 30 with the reservoir floor removed.
  - The gain at the target crossing was 4 to 9 with the original layout.
  - The only remaining lever was shrinking the cup until 5% of `full_count` fell to about 20, which would contort the level around a test artefact.
- **Decision:** The coordinator chose Option A. A second `GoalTracker` at `target_count + full_count.div_ceil(20)` with the same `settle_steps` must reach Won at a step `<= step_limit - 120`. The outcome, the outcome step and the loss rule stay unchanged.
- **Fix:**
  - Added `goal::maybe_margin_win_step` and `MARGIN_WIN_SLACK_STEPS`, with three tests.
  - The win probe continues the same core to step 600 and then runs the post-latch `advance(4)` pair.
  - Edited `37-08-PLAN.md` to carry the rule and the longer post-latch recording into `check_contract` and `checkReplay`.
- **Files modified:** goal.rs, goal/tests.rs, session_tests.rs, 37-08-PLAN.md
- **Commits:** 52feaaaee, plus the docs commit

**2. [Process] Two code commits instead of one**
- The plan asked for a single commit with the subject `feat(37-06): build puzzle sessions from level and placements`.
- I committed Task 1 under that subject before stopping for the decision. Task 2 is a separate `test(37-06)` commit, so Task 3's `git log -1` subject check does not match.
- The Task 2 commit contains three files, and the plan's four-file list does not apply: `first_pour.rs` needed no change, and the goal module gained the helper.

**3. [Formatting] Temporary marker comment on its own line**
- rustfmt moved the `// Temporary until Plan 37-07 wires newPuzzle; Plan 07 removes this line.` comment on `SessionSource::Puzzle` onto its own line.
- `grep -c` still prints 2. Plan 07 has to delete both lines there, the attribute and the comment.

### Other notes

- `generated_by` is `gsd-execute-phase`, as the orchestrator asked. The plan's output section says `gsd-executor`.
- I did not mark GOAL-02 or GOAL-04 complete in REQUIREMENTS.md, as instructed.
- The tuning experiments (wider outlet, no reservoir floor) were measured and then reverted. `first_pour.rs` has no diff.

## Known Stubs

- The two temporary `#[cfg_attr(not(test), allow(dead_code))]` markers in session.rs (`SessionSource::Puzzle` and `create_puzzle`) are intentional. Plan 37-07 removes them when `newPuzzle` wires the source.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/puzzle/session_tests.rs (204 lines), session.rs (515 lines), goal.rs (229), goal/tests.rs (451), all within 628.
- FOUND: commits b85ed6ec0 and 52feaaaee.
- FOUND: target/phase37/gate-06.txt (`25/25 identical`) and target/phase37/checks-06.results (7 lines, all exit=0).
