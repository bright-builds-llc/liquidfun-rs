---
phase: 39-place-then-play-tools-and-stars
fixed_at: 2026-10-13T02:41:00Z
review_path: .planning/phases/39-place-then-play-tools-and-stars/39-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 39: Code Review Fix Report

**Fixed at:** 2026-10-13T02:41:00Z
**Source review:** .planning/phases/39-place-then-play-tools-and-stars/39-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (fix scope critical_warning: R-WR-01, W-WR-01, W-WR-02, W-WR-03, W-WR-04)
- Fixed: 5
- Skipped: 0
- Info findings were left alone, except R-IN-02. It is the root cause of R-WR-01, so it was fixed in the same commit.

Each fix has an Arrange/Act/Assert regression test. Each test was run against the pre-fix source and failed there, then passed after the fix (details per finding below).

## Fixed Issues

### R-WR-01: `KindNotAllowed` can carry an all-zero outline, but the TS decoder treats status 2 as geometric

**Status:** fixed: requires human verification (wire-contract and precedence logic)
**Files modified:** `crates/liquidfun-wasm/src/puzzle/tools.rs`, `crates/liquidfun-wasm/src/puzzle/validate.rs`, `crates/liquidfun-wasm/src/puzzle/validate/tests.rs`
**Commit:** 0b5472f26
**Applied fix:**
- Added a single `decode_pose` helper that decodes kind, then angle, then position. This also fixes R-IN-02. `parse_one` now calls it after its kind and allowance checks, so the Phase 37 build-time error order is unchanged: kind, then allowed, then angle, then position, then build area. `pose_placement` became a thin test-only wrapper.
- `placement_statuses` decodes first. An undecodable pose now reports `UnknownKind`, `AngleOutOfRange` or `PositionOutOfRange` with no outline. Every other status comes from `parse_one` and carries the decoded outline. As a result, `isGeometricStatus` in `placement-status.ts` holds by construction.
- Updated the `Inspection::outlines` doc to state the contract.
- Regression test: `a_disallowed_kind_without_an_outline_reports_the_decoding_reason` runs on the `NO_TOOLS` spec with four poses: bad angle, out-of-range coordinate, kind 0, and the reference pose. It asserts the statuses `[4, 5, 1, 2]`, and that the outline is non-zero exactly when the status is geometric. On the pre-fix source it failed with `left: [2, 2, 1, 2]`, `right: [4, 5, 1, 2]`.
- `inspector_and_new_puzzle_agree` and both fixture replays still pass. The recorded outcomes are unchanged.

### W-WR-01: Play, Stop, Space and editor commands still work in the failure view

**Status:** fixed: requires human verification (state-handling logic)
**Files modified:** `web/src/player/puzzle-host.ts`, `web/src/player/puzzle-run.ts`, `web/src/player/puzzle-editing.ts`, `web/src/player/puzzle-lifecycle.ts`, `web/src/player/scene-lifecycle.ts`, `web/src/puzzle/hud-format.ts`, `web/src/components/PuzzleTransport.tsx`, `web/src/components/PuzzlePanel.tsx`, `web/tests/puzzle-run.test.ts`, `web/tests/puzzle-format-decision.test.ts`, `web/tests/worker-loop-fixture.ts`
**Commits:** a833050a4, f3a895627 (follow-up fixture fix, see below)
**Applied fix:**
- Added `ownsLiveSession(host)`. It is true only when the view is not `failure`, `host.puzzle.maybeOwned` is defined, and it equals `host.maybeSession`. These all check it: `playPuzzle`, `stopPuzzle`, `applyPuzzleEdit`, `applyKeyCommand`, `inspectNow`, `inspectLive`, and the Space `togglePlay` handler. `togglePlay` now returns `false` while failed, so Space keeps its default action.
- Added `releaseFailedPuzzle(host)`, which cancels any pending live inspection and clears `host.puzzle.maybeOwned`. The scene lifecycle's `failScene` calls it before disposing the session.
- `transportModel(state, failed = false)` now keeps the Play or Stop face but disables it, and disables Clear, while the level is failed. `PuzzleTransport` takes a `failed` prop, and `PuzzlePanel` passes `status === "failed"`.
- Regression tests in the `failure view` block of `tests/puzzle-run.test.ts`:
  - Play and Space are ignored when building fails.
  - Stop and Space are ignored when a run fails after a Win latch, and no stars are recorded.
  - Editor edits and the Escape key command are ignored.
  - `releaseFailedPuzzle` cancels the pending inspection and releases the owner.
- Two `transportModel` cases check the failed state.
- All four failure-view tests failed on the pre-fix source.
- Follow-up commit f3a895627: the worker loop test fixture cast a partial `SceneRuntime` with no `puzzle` field. Once `failScene` read `session.puzzle`, `scene-lifecycle-worker.test.ts > shows an actual initialization failure…` failed with `TypeError: Cannot read properties of undefined (reading 'maybeInspectionFrame')`. Production `SceneRuntime` always has a puzzle runtime (`playground-session.ts`), so I fixed the fixture to use `createPuzzleRuntime()` rather than adding a defensive check.

### W-WR-02: Space on a non-button control such as the "Failure details" `<summary>` is taken as Play/Stop

**Status:** fixed
**Files modified:** `web/src/puzzle/keyboard.ts`, `web/tests/puzzle-keyboard.test.ts`
**Commit:** 3805f1fe2
**Applied fix:**
- Added a pure `classifyKeyTarget(element)`. It reads only `tagName`, `isContentEditable` and `getAttribute`, so it can be tested without a DOM.
  - `<button>`, `<a>`, `<summary>` and the ARIA activatable roles (button, checkbox, link, menuitem*, option, radio, switch, tab) are classified as `"button"`, so native Space/Enter activation is left alone.
  - select, input, textarea and contenteditable elements are classified as `"formControl"`.
  - The canvas stays `"canvas"` even though it has `role="application"`.
- `keyTargetOf` now delegates to it.
- Regression tests: classification of each case, plus "leaves Space on the failure details disclosure to its native toggle".
- Limitation: vitest runs in a `node` environment, so the old `instanceof`-based path could not be exercised on a `<summary>` element. The new tests failed before the fix only because `classifyKeyTarget` did not exist yet.

### W-WR-03: Holding Space (key auto-repeat) toggles Play and Stop over and over

**Status:** fixed
**Files modified:** `web/src/puzzle/keyboard.ts`, `web/src/components/PuzzlePanel.tsx`, `web/tests/puzzle-keyboard.test.ts`
**Commit:** eff2a486d
**Applied fix:**
- Added a pure `isRepeatedPlayToggle(command, repeat)`. When it is true, the panel's `handleKeyDown` calls `preventDefault()` and does not dispatch the command, so only the first keydown toggles. Arrow-nudge repeats still pass through.
- Regression tests check three cases: a repeated toggle is swallowed, the first press is not, and a repeated nudge is not.

### W-WR-04: The editor pointer accepts any mouse button, so right-click places or moves tools

**Status:** fixed
**Files modified:** `web/src/player/puzzle-editor-input.ts`, `web/tests/puzzle-editor-input.test.ts`
**Commit:** fa7b71fed
**Applied fix:**
- Added `isEditingPress(event)`, which requires `isPrimary && button === 0`. It is checked before a press starts in `onDown`. It applies to every pointer type, so a pen barrel button or eraser also does not edit.
- The test harness `fire` now sends `button` and `isPrimary` (defaults `0` and `true`).
- Regression tests:
  - A right or middle click with a kind armed does not place.
  - A right press and drag on a tool neither selects, drags nor settles.
  - A non-primary pointer is ignored.
- All four tests failed on the pre-fix source.

## Verification

All commands were run after the last fix commit (HEAD f3a895627):

| Check | Result |
|---|---|
| `cargo fmt --all --check` | pass (exit 0) |
| `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | pass (exit 0) |
| `cargo test -p liquidfun-wasm` | pass: 489 passed, 0 failed, 3 ignored (lib, 184 s); doc-tests 0 |
| `bun scripts/web-build.ts wasm` | pass. The wasm-pack build prints a pre-existing `unreachable_code` warning in `crates/liquidfun/src/particle/solver/boundary/support.rs:98`; that file is untouched |
| `just puzzle-replay-wasm` | pass: `first-pour-solution.txt` won at step 201, final_count 689, peak_held 811, brim_step 262. `first-pour-empty.txt` lost at step 720, final_count 0. These match the recorded fixtures |
| `bun run test:unit` (web) | pass: 94 files, 1288 tests. The first run failed 1 test (`scene-lifecycle-worker.test.ts`, the partial-fixture TypeError above); it passed after commit f3a895627 |
| `bunx tsc --noEmit` (web) | pass (exit 0) |
| `bun scripts/bright-builds-check.ts all` | pass: findings=0 (file-lengths scanned=1476, lessons findings=0) |
| `bun scripts/web-build.ts build` then `CI=true bun run test:puzzle` (web) | pass: 29/29 (chromium + touch) |
| `just playground-scene-spot` (run because `scene-lifecycle.ts` changed) | pass: 25/25 fingerprints identical to `target/phase39/base.txt` (run dir `target/dam-break-perf/2026-10-13T02-40-52Z/`) |

Protected paths: these fix commits do not touch `crates/liquidfun`, the replay fixtures or `web/src/generated/`. `.planning/config.json` was not staged.

---

_Fixed: 2026-10-13T02:41:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
