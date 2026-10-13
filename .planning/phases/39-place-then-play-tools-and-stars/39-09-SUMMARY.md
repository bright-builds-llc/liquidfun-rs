---
phase: 39-place-then-play-tools-and-stars
plan: "09"
subsystem: web-puzzle-shell-sessions-and-editing
tags: [typescript, puzzle, shell, preview-session, run-session, stars, inspector, announcements, keyboard, D-02, D-14, D-15, D-20, D-21, D-23, D-24, D-27, UI-D14]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T01:27:05Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: editor.ts and editor-format.ts (39-02); stars.ts, star-store.ts, keyboard.ts, speed.ts (39-03); buildPuzzleSessionSync, inspectPlacementLanes, inspectPlacements, PuzzleLevelInfo.toolBudget/brimCount/parTools/buildAreaCells (39-06); reducer editor/stop/setSpeed/inspected events, canPlay, goalTicks, runStarsForState (39-07)
provides:
  - web/src/player/puzzle-host.ts PuzzleHost, dispatch, publishState, disposeCurrentSession, viewKindFor, adoptPuzzleSession, presentStepZero
  - web/src/player/puzzle-run.ts buildPuzzle, playPuzzle, stopPuzzle, retryPuzzle, editPuzzle, cycleSpeed, recordDecision, recordStop, publishHeldHash
  - web/src/player/puzzle-editing.ts applyPuzzleEdit, inspectNow, commitInspection, redrawBuilding, cancelPendingInspection, toggleArm, placeAtCenter, applyKeyCommand, EditMode, EditorKeyCommand
  - web/src/player/puzzle-lifecycle.ts applyPuzzleCommand (Space toggle plus editor keys) and the final PuzzleController
  - web/src/puzzle/editor-announce.ts editAnnouncement
  - web/src/puzzle/frame-hash.ts FNV_OFFSET, fnv1a64Bytes, fnv1a64Lanes, hex16 (replay hash.ts re-exports it)
  - PuzzleRuntime.starStore, maybeLevelInfo, maybeBestStars, maybeHeldHash, announcement, maybeInspectionFrame
affects:
  - 39-10 (loop multiplier reads state.speed; cycleSpeed already dispatches setSpeed)
  - 39-11 and 39-12 (canvas overlay, tray, transport, selection bar, star list and keydown attach to the controller)
  - 39-13 (e2e D-10 literals in web/e2e/puzzle.spec.ts)
tech-stack:
  added: []
  patterns:
    - building shows an empty-lane preview session; Play and Stop build sessions synchronously after wasm init (Research Pattern 3)
    - host primitives live in puzzle-host.ts so the editing, run and lifecycle modules import in one direction (host ← editing ← run ← lifecycle)
    - star recording hooks the loop's dispatch closure, so it fires exactly once on the running → ended transition
    - live edits coalesce through one requestAnimationFrame id in PuzzleRuntime.maybeInspectionFrame; commits cancel it
key-files:
  created:
    - web/src/player/puzzle-host.ts
    - web/src/player/puzzle-run.ts
    - web/src/player/puzzle-editing.ts
    - web/src/puzzle/editor-announce.ts
    - web/src/puzzle/frame-hash.ts
    - web/tests/puzzle-host-fixture.ts
    - web/tests/puzzle-run.test.ts
    - web/tests/puzzle-editing.test.ts
    - web/tests/puzzle-editor-announce.test.ts
  modified:
    - web/src/player/puzzle-lifecycle.ts
    - web/src/player/puzzle-runtime.ts
    - web/src/puzzle/controller.ts
    - web/scripts/puzzle-replay/hash.ts
    - web/tests/puzzle-lifecycle.test.ts
key-decisions:
  - "Host primitives moved to a new puzzle-host.ts and buildPuzzle, Retry and Edit moved to puzzle-run.ts, so no module imports in a cycle; puzzle-lifecycle.ts re-exports PuzzleHost and the run commands for existing callers"
  - "Stars are recorded in the loop's dispatch closure on running → ended with a win, and by Stop before the preview rebuild, so a failing rebuild cannot lose already-latched stars (D-15)"
  - "Every state-changing committed edit, including select and arm, runs a commit inspection; a no-op edit (same state object) skips inspection and announcement"
  - "A live inspection publishes its state too; the HUD reads only the committed snapshot, so copy and Play do not change (UI-D14)"
  - "Space (togglePlay) is a transport command routed by applyPuzzleCommand in the lifecycle; applyKeyCommand takes only editor keys (EditorKeyCommand)"
  - "When editAnnouncement returns undefined, the previous announcement text stays; route entry, Try again and leavePuzzle reset it to empty"
  - "A tool with no committed status counts as valid for announcements, so a newly invalid tool is still announced"
requirements-completed: []
duration: 9min
completed: 2026-10-13
---

# Phase 39 Plan 09: Puzzle Shell Sessions, Stop, Stars, Editing and Key Commands Summary

**In building, the puzzle shows an empty-lane preview. Play builds the run session synchronously from canonical placements. Stop returns to a fresh preview with the tools kept. Won runs and Stop after a win record stars into a page-session best-of store, and the held frame publishes its FNV-1a 64 hash. Every committed edit runs the Rust inspector, redraws, publishes and announces. Live drags coalesce to one live inspection per animation frame. The final `PuzzleController` exposes every editor and transport command, including D-23 key dispatch.**

## Performance

- **Duration:** about 9 min
- **Started:** 2026-10-13T01:18:27Z
- **Completed:** 2026-10-13T01:27:05Z
- **Tasks:** 2 of 2
- **Files:** 9 created, 5 modified

## Accomplishments

### Run side (Task 1)

- `puzzle-run.ts` (261 lines):
  - `buildPuzzle` builds the run's canonical placements only when Retry left `loading` with `pendingRun`. Every other build gets empty lanes.
  - After `loaded`, it publishes `maybeLevelInfo` and the stored best.
  - `playPuzzle` requires `canPlay`. It calls `buildPuzzleSessionSync` with `canonicalPlacements` lanes, frees the preview, presents the run's step-0 frame and starts the loop.
  - `stopPuzzle` records latched stars (D-15) and builds an empty preview synchronously. It disposes the run session, cancels the pending frame and dispatches `stop {stepIndex, status}`. The editor is kept.
  - All bodies are wrapped in try/catch that routes errors to `failScene`, and the generation guard supersedes stale builds and loops.
- Stars:
  - `recordDecision` fires once on running → ended with a win. It records `runStarsForState(...)` and `floorPercent(peakHeldCount, goalFull)`.
  - `recordStop` records only when the running status is won.
  - `maybeBestStars` updates after each record and on route entry.
  - The store lives in `PuzzleRuntime.starStore`. It survives `leavePuzzle` and new route entries, and has no web storage.
- Held hash: once held, `maybeHeldHash()` is `hex16(fnv1a64Lanes([particlePositions, rigidSegments, rigidCircles]))`. Play, Stop, Retry, Edit, route entry, Try again and leave reset it.
- `frame-hash.ts` holds the moved FNV code verbatim. `scripts/puzzle-replay/hash.ts` re-exports it, so the replay tests and script are unchanged; `bun -e` prints `cbf29ce484222325` for the offset.
- `puzzle-lifecycle.ts` (188 lines):
  - Route entry reads the seam into a local and resets speed to 1×.
  - Try again resets with `placementsInOrder(current.editor)` and keeps the current speed.
  - `leavePuzzle` clears level info, the held hash, the announcement and any pending inspection, and keeps the store.

### Editing side (Task 2)

- `puzzle-editing.ts` (221 lines), `applyPuzzleEdit(host, event, mode)`:
  - **Commit:** dispatches the edit, then cancels any pending live frame. It inspects `placementsInOrder(editor)` through `inspectPlacements(inspectPlacementLanes, …)` and dispatches `inspected {revision, mode: "commit"}`. It then redraws the last frame once, publishes and announces.
  - **Live:** dispatches and publishes at once, and schedules at most one `requestAnimationFrame`. That callback dispatches `inspected` in live mode, redraws and publishes. It never announces.
  - Edits outside building are no-ops.
- `commitInspection` runs in `buildPuzzle` before the first building publish, and in `stopPuzzle` before presenting. Stop announces "Stopped. Your tools are kept." or "Stopped. Stars kept: …".
- `editAnnouncement` (pure) composes the UI-SPEC copy from committed snapshots only:
  - armed and canceled;
  - placed, plus the reason when invalid;
  - "Plank N of M selected.", plus the reason when invalid;
  - "Rotated to −15°.", plus any validity change;
  - nudge and move announce only when validity changes;
  - removed and cleared.
- Controller:
  - New members: `announcement`, `clear`, `toggleArm`, `placeAtCenter` (build-area centre (10, 64) for first-pour, angle 0, selected), `rotateSelected`, `removeSelected`, `command`.
  - `command` routes Space to Play when `canPlay` (still handled when Play is blocked) or to Stop in running. All other keys go to `applyKeyCommand`.
  - Nudge, rotate and remove need a selection. Escape disarms and deselects. `cycleSelection` returns false past the last tool. Every command returns false in loading and ended.

## Task Commits

1. **Task 1 RED:** `e22470490` (test). Failing tests for preview and run sessions, Stop, stars and the held hash.
1. **Task 1 GREEN:** `33df0f78e` (feat). Preview and run sessions, Stop, star recording and the held hash.
1. **Task 2 RED:** `0da2fe7b1` (test). Failing tests for committed and live editing, announcements and key commands.
1. **Task 2 GREEN:** `22f850a8b` (feat). Committed and live editing with Rust inspection, announcements and key commands.

No refactor commits were needed.

## Verification

- RED runs: both test files failed to load, because `puzzle-run`/`puzzle-host` in Task 1 and `puzzle-editing`/`editor-announce` in Task 2 did not exist yet.
- Task 1 check: `bunx vitest run tests/puzzle-run.test.ts tests/puzzle-lifecycle.test.ts tests/puzzle-replay.test.ts tests/puzzle-shell-wiring.test.ts` passed 69 tests, and `bunx tsc --noEmit` exited 0.
- Task 2 check: `bunx vitest run tests/puzzle-editing.test.ts tests/puzzle-editor-announce.test.ts tests/puzzle-run.test.ts tests/puzzle-lifecycle.test.ts` passed 76 tests, and `bunx tsc --noEmit` exited 0.
- Full Vitest: 91 files, 1217 tests passed.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `bun run build:app`: built.
- Acceptance greps:
  - These matched: `export function stopPuzzle` and `buildPuzzleSessionSync` in `puzzle-run.ts`; the `frame-hash` re-export in `hash.ts`; `export function applyPuzzleEdit` and `inspectPlacementLanes` in `puzzle-editing.ts`; `command(command: PuzzleKeyCommand): boolean` in `controller.ts`.
  - These printed nothing: `placements:` in `puzzle-runtime.ts`; `localStorage|sessionStorage|indexedDB`; `setTimeout|setInterval|Date.now` in `puzzle-editing.ts` and `puzzle-run.ts`.
- Line counts: `puzzle-lifecycle.ts` 188 (< 360), `tests/puzzle-lifecycle.test.ts` 486 (≤ 628).
- The web package has no lint or format script.

### e2e: `CI=true bunx playwright test e2e/puzzle.spec.ts --project=chromium`

**15 passed, 1 failed.**

- **Still failing:** "wins with the recorded solution through the test seam" fails on its D-10 literal:
  - Expected `"Filled! 76% of 60%"`.
  - Received `"Filled! 90% of 60%"`.
- **What now works:** the 39-07 gap is closed. The seeded tool gets a committed inspection before building is first published, so Play is enabled, the run session is built from canonical lanes, the fill rises and the run wins.
- **Why it still fails:** the remaining mismatches are the pre-D-10 literals that 39-07 assigned to Plan 13: "Filled! 76% of 60%", "0:08.6", held step 321 and meter 76.
- **Scratch check:** I patched those literals to the D-10 values ("Filled! 90% of 60%", "0:07.6", 382, 90) and ran the same test alone. The whole flow passed: win, deterministic Retry replay, and Edit back to building with Play enabled at 0:12.0. The spec was then restored with `git checkout`, and `git diff --quiet` confirmed it. Plan 13 still owns the literal edit.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] New `puzzle-host.ts`; `buildPuzzle`, Retry and Edit live in `puzzle-run.ts`**
- **Found during:** Task 1
- **Issue:** As written, the plan has the lifecycle import `puzzle-run.ts` (controller, `recordDecision`, `publishHeldHash`). It also has `puzzle-run.ts` call lifecycle internals: `startLoop`, `dispatch`, and `buildPuzzle` for Retry and Edit. Task 2 adds `puzzle-editing.ts`, which both sides need. That is an import cycle.
- **Fix:**
  - The host type and primitives moved to `puzzle-host.ts`. `buildPuzzle`, `startLoop`, Retry and Edit moved to `puzzle-run.ts`.
  - The lifecycle keeps route entry, Try again, leave, the controller and command routing. It re-exports `PuzzleHost` and the run commands.
  - `recordDecision` runs from the loop's dispatch closure instead of a lifecycle dispatch wrapper. Behavior is the same: it records once on running → ended.
- **Files:** `web/src/player/puzzle-host.ts` (new, not in `files_modified`), `puzzle-run.ts`, `puzzle-lifecycle.ts`
- **Commit:** `33df0f78e`

**2. [Rule 3 - Blocking] Shared test fixture**
- **Found during:** Task 1
- **Issue:** Three test files need the same host, the generated-session fakes with scripted win/Brim/loss frames, inspector fakes and rAF stubs.
- **Fix:** Added `web/tests/puzzle-host-fixture.ts`, following the repo's `*-fixture.ts` convention. `puzzle-lifecycle.test.ts` now uses it, which brought it from 601 to 486 lines.
- **Commit:** `e22470490`, `33df0f78e`

**3. [Rule 2 - Correctness] Run step 0 is checked**
- **Found during:** Task 1
- **Issue:** The `play` reducer event does not validate the new run session's first frame.
- **Fix:** `captureStepZero` throws "Puzzle level did not start at step 0" unless the frame is a running step 0. The error goes to `failScene`.
- **Commit:** `33df0f78e`

**4. Lifecycle test updates (Phase 38 expectations)**
- "builds with the test seam's placements" is now "seeds the editor from the test seam and still builds an empty preview": the preview lanes are empty and the editor holds the seam placement.
- The Retry and Edit tests compare `state.editor.tools` instead of the removed `host.puzzle.placements`.
- The leave test checks that the editor is empty.
- The loader mock now provides `buildPuzzleSessionSync` and `inspectPlacementLanes`.

## Known Stubs

None. The controller's new members are not rendered yet. Plans 11 and 12 attach the canvas overlay, tray, transport, selection bar, star list, live region and panel keydown.

## Threat Flags

None. Mitigations are in place and tested:

- **T-39-26:** the seam is read once at route entry through `maybeReadTestPlacements` and only seeds the editor. Play still requires a fresh, all-valid committed Rust inspection.
- **T-39-27:** Play and Stop increment the generation. Async builds free stale sessions, and every failure reaches `failScene`.
- **T-39-28:** live edits schedule at most one rAF inspection, and a test shows a second live edit does not schedule another.
- **T-39-29:** stars are recorded once on running → ended, plus on Stop after a win, through the OR/max store.
- **T-39-30:** accepted. The held hash is a deterministic fingerprint.

## Requirements

TOOL-02, TOOL-04, TOOL-06 and STAR-02 now have their session-side behavior: editing, Play/Stop/Clear, key dispatch, and star recording with the best kept. Each still needs the Plan 11/12 UI (pointer input, transport, keydown, star list), so none is marked complete in REQUIREMENTS.md. TOOL-03 was already Complete.

## Self-Check: PASSED

- FOUND: web/src/player/puzzle-host.ts, web/src/player/puzzle-run.ts, web/src/player/puzzle-editing.ts, web/src/puzzle/editor-announce.ts, web/src/puzzle/frame-hash.ts, web/tests/puzzle-host-fixture.ts, web/tests/puzzle-run.test.ts, web/tests/puzzle-editing.test.ts, web/tests/puzzle-editor-announce.test.ts
- FOUND commits: e22470490, 33df0f78e, 0da2fe7b1, 22f850a8b
