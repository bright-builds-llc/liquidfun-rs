---
phase: 39-place-then-play-tools-and-stars
plan: "08"
subsystem: puzzle-replay
tags: [rust, wasm, bun, replay, fixtures, peak-held, brim, D-12]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T01:13:35Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: "39-05 ProofFrame peakHeldCount/brimCount/brimStep/decidedStep; 39-06 rebuilt wasm package with those getters"
provides:
  - crates/liquidfun-wasm/src/puzzle/replay/fixture.rs grammar with peak_held and brim_step (shared with web/scripts/puzzle-replay/fixture.ts)
  - Native and Bun wasm replays that record and check peak held and Brim step at the decision step
  - Fixtures with peak_held 811 / brim_step 262 (solution) and peak_held 0 / brim_step none (empty)
affects:
  - 39 gate plan (rebuilds wasm and reruns `bun scripts/puzzle-replay.ts replay`)
tech-stack:
  added: []
  patterns:
    - record mode replays only level and placement lines, so a grammar extension can be recorded without a lenient parser
    - decision captured from the frame whose step_index equals decidedStep; decided step must never change afterwards
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/replay/fixture.rs
    - crates/liquidfun-wasm/src/puzzle/replay/fixture/tests.rs
    - web/tests/puzzle-replay-stars.test.ts
  modified:
    - crates/liquidfun-wasm/src/puzzle/replay.rs
    - crates/liquidfun-wasm/src/puzzle/replay/tests.rs
    - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt
    - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-empty.txt
    - web/scripts/puzzle-replay/fixture.ts
    - web/scripts/puzzle-replay.ts
    - web/tests/puzzle-replay.test.ts
key-decisions:
  - "A recorded Brim step must hold at brim_count + fill margin by step_limit - 120 in both the exact and the debug contract, matching how the win margin already applies to both"
  - "Record mode (Rust and Bun) parses only the level and placement lines, so recording does not depend on the expectation grammar being current"
  - "Grammar tests moved with the grammar into replay/fixture/tests.rs; replay/tests.rs keeps contract and fixture-replay tests"
requirements-completed: []
duration: 14min
completed: 2026-10-13
---

# Phase 39 Plan 08: Peak Held and Brim Step Replay Fixtures Summary

**Both replay harnesses now record and check the peak held count and the Brim step at the D-10 decision step. The solution fixture records `peak_held 811` and `brim_step 262`, and the empty fixture records `peak_held 0` and `brim_step none`. Release-native and Bun wasm agree exactly. The existing `outcome`, `outcome_step` and `final_count` lines are byte-unchanged.**

## Performance

- **Duration:** about 14 min
- **Started:** 2026-10-13T01:00:07Z
- **Completed:** 2026-10-13T01:13:35Z
- **Tasks:** 2 of 2
- **Files:** 10 (3 created, 7 modified)

## Accomplishments

- The grammar moved verbatim from `replay.rs` into `replay/fixture.rs` and was then extended:
  - `KNOWN_KEYS` gains `peak_held` and `brim_step`.
  - After `final_count`, the grammar requires `peak_held <u32>` and then `brim_step <u32|none>`, with nothing after them.
  - `FixtureExpectation` gains `peak_held` and `maybe_brim_step`.
- `replay.rs` (379 lines, down from 487) now holds only the harness.
  - `ReplayTrace` gains `peak_held`, `maybe_brim_step` and `decided_step`.
  - `replay` keeps stepping until it has observed the decision and reached `step_limit − 120`. It reads the peak and Brim values from the frame whose step equals `decidedStep`. It asserts that the decided step first appears on its own frame and never changes afterwards, including through the post-latch `advance(4)` calls.
- `check_contract`:
  - The exact form also compares the peak held count and the Brim step.
  - The debug form allows the Brim step to differ by `DEBUG_OUTCOME_STEP_TOLERANCE` (5) when both runs reach Brim. Otherwise both runs must have no Brim.
  - In both forms, a recorded Brim requires `maybe_margin_win_step(counts, brim_count, …)`, so the Brim count raised by 45 must still be held by step 600.
- `record_line` appends `peak_held N brim_step N|none` before `profile`.
- The TS `fixture.ts` mirrors the grammar, the error texts and the contract. `marginWinStep` now takes the count to raise, the target or the Brim count. `ReplayLevel` gains `brimCount`.
- The Bun `replaySession` mirrors `observe_decision`. Its RECORD and REPLAY lines now include `peak_held` and `brim_step`.
- Tests:
  - Rust grammar tests in `replay/fixture/tests.rs` (273 lines) check the exact error text for these cases: missing `peak_held`, missing `brim_step`, `brim_step` before `peak_held`, `brim_step -1`, `brim_step nope`, `peak_held none`, and a line after `brim_step`.
  - Rust contract tests in `replay/tests.rs` (377 lines) cover: exact peak held, exact Brim step, a debug Brim step at tolerance 5 versus 6, Brim present exactly when recorded, the raised-Brim hold, and the record line format.
  - The fixture replay test also checks the trace shape: the decision step is the Brim step for a Brim win and the outcome step for a loss, and the counts run through the later of the decision and step 600.
  - `web/tests/puzzle-replay-stars.test.ts` mirrors the same cases with identical messages.

## RECORD Lines

Release native (`LIQUIDFUN_PUZZLE_RECORD=1 cargo test -p liquidfun-wasm --release --lib puzzle::replay -- --nocapture`):

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 peak_held 811 brim_step 262 profile release
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile release
```

Debug native (same command without `--release`) printed identical values with `profile debug`.

Bun wasm (`bun scripts/puzzle-replay.ts replay --record`, using the package Plan 06 built; not rebuilt here):

```
RECORD first-pour-solution.txt outcome won outcome_step 201 final_count 689 peak_held 811 brim_step 262 profile wasm
RECORD first-pour-empty.txt outcome lost outcome_step 720 final_count 0 peak_held 0 brim_step none profile wasm
```

These match the 39-05 measurements: peak 811 and Brim at step 262, with the run decided at step 262.

## Final Fixture Text

`first-pour-solution.txt`:

```
level first-pour
placement plank 8 64 23
outcome won
outcome_step 201
final_count 689
peak_held 811
brim_step 262
```

`first-pour-empty.txt`:

```
level first-pour
outcome lost
outcome_step 720
final_count 0
peak_held 0
brim_step none
```

`git diff 424735672beffd5989a89f12a321f429e55bd8b5 -- crates/liquidfun-wasm/src/puzzle/fixtures | grep '^-[a-z]'` prints nothing, so the fixture diff contains only additions.

## Task Commits

1. **Task 1: Rust grammar split, peak_held and brim_step capture and checks** - `973ecd0c6` (feat)
1. **Task 2: TS grammar and Bun wasm replay parity** - `b01cdebbf` (feat)

TDD note: in Task 1 the new tests failed first, because the checked-in fixtures lacked `peak_held` (`fixture line 6: missing \`peak_held\` line`). The fixtures were then recorded and the tests went green. Each task landed as one feat commit with its tests, as in the earlier Phase 39 plans.

## Verification

- `cargo fmt --all --check`: clean.
- `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`: exit 0 after the fixes listed under Deviations.
- `cargo test -p liquidfun-wasm --lib puzzle::replay` (debug): 27 passed. The `--release` run: 27 passed.
- `cargo test -p liquidfun-wasm` (debug, full): 488 passed, 0 failed, 3 ignored. The 3 ignored tests existed before this plan. This run happened before the two lint-only fixes; the replay tests were rerun after them.
- `bunx vitest run tests/puzzle-replay.test.ts tests/puzzle-replay-stars.test.ts`: 35 passed. The full `bunx vitest run`: 88 files and 1162 tests passed.
- `bunx tsc --noEmit`: exit 0.
- `bun scripts/puzzle-replay.ts replay`: exit 0, with `REPLAY first-pour-solution.txt pass (outcome won step 201 final_count 689 peak_held 811 brim_step 262)` and `REPLAY first-pour-empty.txt pass (outcome lost step 720 final_count 0 peak_held 0 brim_step none)`.
- `bun scripts/bright-builds-check.ts all`: `findings=0`.
- Acceptance greps: the solution fixture has 2 matching new lines, the empty fixture has `peak_held 0` and `brim_step none`, `wc -l replay.rs` is 379 (below 400), and `"peak_held"` and `"brim_step"` appear in `fixture.ts`.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Record mode could not parse the old fixtures**
- **Found during:** Task 1, step 4.
- **Issue:** The new strict grammar rejects the checked-in fixtures until they gain the two new lines, but record mode parsed the full fixture before replaying. Recording the lines therefore needed the lines to exist already.
- **Fix:** Record mode now parses only the level and placement lines (`input_lines` in Rust, `fixtureInputText` in TS). Normal replay still uses the strict full grammar. Record mode still prints and never writes.
- **Commits:** `973ecd0c6`, `b01cdebbf`

**2. [Rule 3 - Blocking] Clippy `private_interfaces` and `format_collect`**
- **Found during:** Task 1.
- **Issue:** The `pub(super)` items in `fixture.rs` were less visible than the `pub(super)` replay functions that use them. `input_lines` built its string with `format!` inside `collect`.
- **Fix:** Fixture items are now `pub(in crate::puzzle)`, and `input_lines` uses `join`.
- **Commit:** `973ecd0c6`

**3. [Rule 2 - Consistency] The Brim margin applies to the exact contract too**
- The plan named the raised-Brim hold only for the debug contract. It is now required in both forms, as the win margin already was (Phase 37 D-22: "Both forms require the recorded outcome and the D-03 margin"). Both current fixtures pass it: the peak at step 600 is 1331, against a requirement of 855.

**4. [Plan-sanctioned] Grammar tests moved to `replay/fixture/tests.rs`**
- The existing grammar tests moved together with the grammar, so `replay/tests.rs` covers only the contract and the replays.

## Issues Encountered

None.

## Known Stubs

None.

## Threat Flags

None. T-39-24 is mitigated: the outcome lines are byte-identical, the fixture diff contains only additions, and record mode prints and never writes. T-39-25 is mitigated: both runtimes have mirrored accept and reject tests that check the exact same error text.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/puzzle/replay/fixture.rs
- FOUND: crates/liquidfun-wasm/src/puzzle/replay/fixture/tests.rs
- FOUND: web/tests/puzzle-replay-stars.test.ts
- FOUND: commit 973ecd0c6
- FOUND: commit b01cdebbf
