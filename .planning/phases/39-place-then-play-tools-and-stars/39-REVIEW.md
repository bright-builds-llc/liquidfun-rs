---
phase: 39-place-then-play-tools-and-stars
reviewed: 2026-10-13T02:24:14Z
depth: standard
diff_base: 424735672beffd5989a89f12a321f429e55bd8b5
reviewers: gsd-code-reviewer (AI) x2 — Rust half and web half, merged by the orchestrator
files_reviewed: 66
files_reviewed_list:
  - crates/liquidfun-wasm/src/frame.rs
  - crates/liquidfun-wasm/src/frame/puzzle.rs
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/puzzle.rs
  - crates/liquidfun-wasm/src/puzzle/builder.rs
  - crates/liquidfun-wasm/src/puzzle/first_pour.rs
  - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-empty.txt
  - crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt
  - crates/liquidfun-wasm/src/puzzle/goal.rs
  - crates/liquidfun-wasm/src/puzzle/hooks.rs
  - crates/liquidfun-wasm/src/puzzle/info.rs
  - crates/liquidfun-wasm/src/puzzle/inspect.rs
  - crates/liquidfun-wasm/src/puzzle/peak.rs
  - crates/liquidfun-wasm/src/puzzle/replay.rs
  - crates/liquidfun-wasm/src/puzzle/replay/fixture.rs
  - crates/liquidfun-wasm/src/puzzle/sat.rs
  - crates/liquidfun-wasm/src/puzzle/spec.rs
  - crates/liquidfun-wasm/src/puzzle/tools.rs
  - crates/liquidfun-wasm/src/puzzle/validate.rs
  - web/package.json
  - web/playwright.config.ts
  - web/scripts/puzzle-replay.ts
  - web/scripts/puzzle-replay/fixture.ts
  - web/scripts/puzzle-replay/hash.ts
  - web/src/components/PlaygroundStage.tsx
  - web/src/components/PuzzleHud.tsx
  - web/src/components/PuzzleIcons.tsx
  - web/src/components/PuzzlePanel.tsx
  - web/src/components/PuzzleResultCard.tsx
  - web/src/components/PuzzleSelectionBar.tsx
  - web/src/components/PuzzleStarList.tsx
  - web/src/components/PuzzleToolTray.tsx
  - web/src/components/PuzzleTransport.tsx
  - web/src/physics/frame-puzzle.ts
  - web/src/physics/frame-scalars.ts
  - web/src/physics/frame.ts
  - web/src/physics/puzzle-inspector.ts
  - web/src/physics/puzzle-loader.ts
  - web/src/physics/puzzle-session.ts
  - web/src/player/puzzle-editing.ts
  - web/src/player/puzzle-editor-input.ts
  - web/src/player/puzzle-host.ts
  - web/src/player/puzzle-lifecycle.ts
  - web/src/player/puzzle-loop.ts
  - web/src/player/puzzle-run.ts
  - web/src/player/puzzle-runtime.ts
  - web/src/player/scene-surface.ts
  - web/src/puzzle/controller.ts
  - web/src/puzzle/debug-attributes.ts
  - web/src/puzzle/edit-event.ts
  - web/src/puzzle/editor-announce.ts
  - web/src/puzzle/editor-format.ts
  - web/src/puzzle/editor.ts
  - web/src/puzzle/format.ts
  - web/src/puzzle/frame-hash.ts
  - web/src/puzzle/hud-format.ts
  - web/src/puzzle/keyboard.ts
  - web/src/puzzle/placement-status.ts
  - web/src/puzzle/reduce-puzzle.ts
  - web/src/puzzle/speed.ts
  - web/src/puzzle/star-format.ts
  - web/src/puzzle/star-store.ts
  - web/src/puzzle/stars.ts
  - web/src/render/puzzle-tools.ts
  - web/src/styles/puzzle-editor.css
  - web/src/styles/puzzle.css
findings:
  critical: 0
  warning: 5
  info: 12
  total: 17
status: issues_found
---

# Phase 39: Code Review Report

Two independent AI reviewers covered the phase diff from `424735672`: one for the 19 Rust files in `crates/liquidfun-wasm`, one for the 47 web files. Finding IDs are prefixed `R-` (Rust) and `W-` (web).

| Severity | Rust | Web | Total |
|---|---|---|---|
| Critical | 0 | 0 | 0 |
| Warning | 1 | 4 | 5 |
| Info | 5 | 7 | 12 |

## Part A: Rust bridge

## Phase 39: Code Review Report (Rust half)

**Reviewed:** 2026-10-13T02:19:57Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

### Summary

This review covers the Phase 39 Rust changes in `crates/liquidfun-wasm` from
`424735672` to HEAD: the SAT clearance test, the shared placement validator and
the `inspectPuzzlePlacements` export, the peak-held, Brim and decision
tracking, the new frame scalars and their consistency rules, and the D-12
replay fixtures. The sibling `tests.rs` files were read for context. The web
files are covered by a separate review.

Overall the code is correct and fails closed. Specific checks:

- **Inspector and `newPuzzle` agree.** A status is invalid exactly when
  `validate_for_build` fails. Parse errors and overlaps show up in both.
  Budget overruns are caught by `check_budget` in one and `mark_over_budget`
  in the other, and both count every placement of a kind, including duplicate
  allowances. `inspector_and_new_puzzle_agree` covers this.
- **SAT.** Checking projections on edge normals gives a lower bound on the
  true distance. It never reports a false "separated" result. Non-finite
  input fails closed. It has no trig and no square root. Rust does not fuse
  operations into FMA, so native and wasm builds compute the same bits.
- **Peak tracker against `GoalTracker`.** The trailing-window minimum
  reaching the target is equivalent to the consecutive-hold counter. The
  xorshift test confirms this. Because `brim_count > target_count` is
  enforced in both `validated_trackers` and `counts_are_consistent`, Brim can
  never latch before Win. So the `stars_are_consistent` rules cannot reject a
  status the hooks actually produce.
- **Lanes at the wasm boundary.** `check_lanes` limits the count to 16 and
  uses `checked_mul` for the stride. Integer narrowing uses
  `i16::try_from`/`u8::try_from`, so nothing can overflow. Grid values are
  `i16`, so every vertex is finite.
- **Fixture parser.** It is panic-free on malformed input. `tokens[0]` is
  always present because empty lines are rejected first.
- **Repo rules.** No `unwrap()` in production code, `maybe_` names are used
  for optional values, and errors are typed.

`cargo test -p liquidfun-wasm --lib -- puzzle:: frame::` passed: 177 passed,
0 failed, 2 ignored. That includes both fixture replays and
`reference_peak_clears_brim_with_margin`.

The one warning is a latent wire-contract mismatch: Rust can return status
code 2 with an all-zero outline, and the TS decoder does not expect that.

### Warnings

#### R-WR-01: `KindNotAllowed` can carry an all-zero outline, but the TS decoder treats status 2 as geometric

**File:** `crates/liquidfun-wasm/src/puzzle/validate.rs:185-192` (with `crates/liquidfun-wasm/src/puzzle/tools.rs:238-274`)

**Issue:** The status comes from `parse_one`, which checks rules in this order:
kind → allowed → angle → position → build area. The outline comes separately
from `pose_placement`, which returns `None` when the angle or position is out
of range. Consider a known kind that is missing from the level budget and also
has a bad angle (for example 24) or a coordinate outside `i16`. Rust reports
`KindNotAllowed` (code 2) with eight zeros in the outline lane.

`web/src/puzzle/placement-status.ts` `isGeometricStatus` treats only codes
1, 4 and 5 as having no outline. So `puzzle-inspector.ts:79` would decode the
zeros as a real degenerate quad at the world origin, and that quad would be
used for drawing and hit-testing. This can't happen today because `first-pour`
allows the only kind, `plank`. It becomes reachable as soon as Phase 40 adds a
kind that some level does not allow. The `Inspection::outlines` doc ("eight
zeros when the pose has no outline") describes the behavior, but the TS side
infers whether an outline exists from the status code.

**Fix:** Make the status determine the geometry on the Rust side, so the TS
rule stays true. In `placement_statuses`, when `parse_one` returns
`ToolNotAllowed` but `pose_placement` is `None`, report the geometric reason
instead:

```rust
let maybe_placement = pose_placement(code, pose);
let status = match parse_one(code, pose, spec.tool_budget, spec.build_area) {
    Ok(_placement) => PlacementStatus::Valid,
    Err(PlacementError::ToolNotAllowed) if maybe_placement.is_none() => {
        undecodable_pose_status(pose) // AngleOutOfRange or PositionOutOfRange
    }
    Err(error) => PlacementStatus::from_error(error).ok_or(error)?,
};
quads.push(maybe_placement.map(Placement::vertices));
```

Then add a test asserting that every non-geometric code has a zero outline and
every geometric code has a non-zero outline, using the `NO_TOOLS` spec. An
alternative is an explicit per-placement `has_outline` lane that TS reads
instead of inferring from the code.

### Info

#### R-IN-01: SAT on edge normals over-rejects corner-to-corner clearances

**File:** `crates/liquidfun-wasm/src/puzzle/sat.rs:14-31`, `crates/liquidfun-wasm/src/puzzle/validate.rs:235-244`

**Issue:** When two shapes face each other corner to corner, the true distance
lies along the line joining the corners, not along an edge normal. So the
largest edge-normal separation underestimates it. For axis-aligned shapes the
test reports overlap up to a Euclidean clearance of `gap·√2`, about 0.113 m
for `first-pour` instead of 0.08 m. This errs on the safe side, as D-04 asks,
and both entry points share it, so parity holds. However, the validate module
doc says outlines "must clear … by one particle diameter", which suggests an
exact distance test.

**Fix:** Document that this is effectively a square (Chebyshev-style)
inflation rather than a rounded one. For example, add to the
`separated_by_more_than` doc: "conservative: corner-to-corner clearances up
to about gap·√2 also count as overlap." No code change is needed unless
players find it too strict.

#### R-IN-02: Pose decoding is duplicated between `pose_placement` and `parse_one`

**File:** `crates/liquidfun-wasm/src/puzzle/tools.rs:238-274`

**Issue:** The kind, `i16` and angle decoding is written twice with different
rule orders. R-WR-01 comes from exactly this split. If a new pose rule (such as
the Phase 40 valve time) is added to only one of the two functions, the status
lane and the outline lane will drift apart again.

**Fix:** Have `parse_one` call `pose_placement` for the decoding (or share a
small `decode_pose` that returns `Result<Placement, PlacementError>`). Keep the
budget and build-area checks in `parse_one`, and keep the Phase 37 error
precedence by checking `ToolNotAllowed` against the decoded kind first.

#### R-IN-03: The frame's `peakHeldCount` keeps rising after `decidedStep`, and the fixture pins the decision-step value

**File:** `crates/liquidfun-wasm/src/puzzle/peak.rs:50-68`, `crates/liquidfun-wasm/src/puzzle/replay.rs:113-135`, `crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt:6`

**Issue:** Following D-08, the tracker freezes only after `step_limit`. On a
Brim win it is decided at the Brim latch (step 262). At that step the peak is
always just at or above `brim_count`; the fixture records `peak_held 811`
against a Brim count of 810. During the settle tail the peak keeps rising.
So the "peak held" the replay records is not the measured peak D-07 asks the
SUMMARY to report. The D-07 margin is actually proven by the counts-based
`check_brim_margin` and `reference_peak_clears_brim_with_margin`, and those
tests pass. Anything that reads `peakHeldCount` at a later frame, such as the
win card and `bestPeakHeldPct` (web half), will show a different value from
the replay's.

**Fix:** No Rust change is needed. Add a doc line on
`PuzzleStatus::peak_held_count` and the `peakHeldCount` getter: "keeps rising
until `step_limit`; sample at `decidedStep` for the D-12 value". Make sure the
SUMMARY reports the peak measured at `step_limit - MARGIN_WIN_SLACK_STEPS` or
at `step_limit`, not the 811 decision-step value, as "the measured peak held
fill".

#### R-IN-04: The "before allocating" claim does not hold at the wasm boundary

**File:** `crates/liquidfun-wasm/src/puzzle/validate.rs:112-117`, `crates/liquidfun-wasm/src/puzzle/inspect.rs:40-55`

**Issue:** `check_lanes` runs before any Rust `Vec` is allocated. But for
`&[u8]`/`&[i32]` parameters, wasm-bindgen glue has already copied the whole
JS typed array into linear memory. An oversized lane is still bounded only by
the caller. The caller is the page itself, so this is not a security issue,
but the doc overstates the guarantee.

**Fix:** Change the wording to "before the validator allocates", or note that
the 16-placement cap bounds work, not the glue copy.

#### R-IN-05: Count-ordering spec errors are reported as `SceneError::Geometry`

**File:** `crates/liquidfun-wasm/src/puzzle/builder.rs:66-80`

**Issue:** `validated_trackers` returns `SceneError::Geometry` when any of
`target < brim <= full` or `par_tools <= total_budget` fails. That label is
misleading when debugging a mis-authored level. The error becomes
`SessionError::SceneConstruction` anyway, so the effect on users is nil.
Separately, `total_budget` uses `.sum()` on `u32`, which panics on overflow in
debug builds. That is unreachable with `u8` counts and a handful of
allowances.

**Fix:** If you want clearer diagnostics, add a `SceneError::LevelSpec`
variant, or mention the count rules in a comment on `validated_trackers`.
Optionally use `fold(0_u32, u32::saturating_add)` in `total_budget`.

---

_Reviewed: 2026-10-13T02:19:57Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_

## Part B: Web player

## Phase 39: Code Review Report (web half)

**Reviewed:** 2026-10-13T02:22:51Z
**Depth:** standard
**Files Reviewed:** 47
**Status:** issues_found

### Summary

This review covers the Phase 39 web diff from `424735672` to HEAD. In scope were the editor state machine, the live and committed inspection split, the pointer editor, keyboard routing, the fast-forward loop, stars and the best-of store, the HUD components, wasm boundary parsing, debug attributes and the replay script. The `hit-test.ts` module was also read because the pointer editor depends on it. The Rust files are covered by a separate reviewer.

Overall the code is careful. The reducer is pure, ignores edit events outside building and throws on contract violations. Inspector answers are tagged with a revision, and stale or mis-sized answers are dropped. The frame and level-info parsers fail closed and cross-check outcome, Brim and decided-step consistency. The loop advances one `nextFrame(1)` per capture at every speed, so the outcome, decided step and settle tail do not depend on speed. `performance.now()` only bounds work per frame; it never decides an outcome. No `innerHTML`, `eval`, `Date.now` or `Math.random` appears in scope. All player text is set as Solid text nodes.

The main issue (W-WR-01) is a regression from Phase 38. `playPuzzle` no longer checks that the puzzle owns the live session, and `stopPuzzle` never did. `failScene` disposes the session but leaves `host.puzzle.current` and `host.puzzle.maybeOwned` in place. The HUD also sits above the non-blocking failure overlay. Together these let Play, Stop, Space and editor commands leave the failure view without using "Try again". The other warnings cover keyboard handling (Space on `<summary>`, key auto-repeat) and unfiltered mouse buttons.

### Warnings

#### W-WR-01: Play, Stop, Space and editor commands still work in the failure view, so a failed level can be left without "Try again"

**File:** `web/src/player/puzzle-run.ts:174-224` (also `web/src/player/puzzle-lifecycle.ts:147-166`, `web/src/player/puzzle-editing.ts:136-158`, `web/src/puzzle/hud-format.ts:39-58`)

**Issue:** Phase 38's `playPuzzle` returned early unless `host.maybeSession === host.puzzle.maybeOwned`. Phase 39 replaced that with `host.puzzle.maybeOwned === undefined` (line 181). `stopPuzzle` (line 205) has no ownership or view check at all.

The failure path is `failScene` in `web/src/player/scene-lifecycle.ts:77-86`. It clears `host.maybeSession` but not `host.puzzle.maybeOwned`, and it leaves `host.puzzle.current` in `building` or `running`. It also does not cancel `host.puzzle.maybeInspectionFrame`.

In canvas-stage layout, `.puzzle-hud` (z-index 3) sits above the failure overlay. That overlay is an `.empty-state` with z-index 2 and `pointer-events: none`. The transport therefore stays clickable, and `transportModel` still reports Play or Stop as enabled because it does not know about the failure view. The result:

- After an inspector failure in building, clicking Play or pressing Space builds a new run session, calls `presentStepZero` and sets the view to `playing`. The failure view disappears without "Try again".
- After a loop failure in running (for example the reducer's "Puzzle outcome changed after the latch"), Stop or Space calls `recordStop`. That records stars from the failed run, builds a preview and sets the view to `paused`.
- Canvas pointer edits and key edits keep changing the hidden editor and running the inspector under the failure overlay.

This defeats the fail-closed handling that the reducer and frame parser are built around. No test covers Play or Stop in the failure view.

**Fix:** Add one guard and use it at every command entry point. Also make `failScene`'s puzzle cleanup release the puzzle owner.

```ts
// puzzle-host.ts
export function ownsLiveSession(host: PuzzleHost): boolean {
  const maybeOwned = host.puzzle.maybeOwned;
  return (
    host.view().kind !== "failure" &&
    maybeOwned !== undefined &&
    host.maybeSession === maybeOwned
  );
}

// puzzle-run.ts: playPuzzle and stopPuzzle
if (!ownsLiveSession(host)) {
  return;
}

// puzzle-editing.ts: applyPuzzleEdit and applyKeyCommand
if (before.kind !== "building" || !ownsLiveSession(host)) {
  return;
}
```

Then:

1. In the failure path (or `disposeCurrentSession`), call `cancelPendingInspection(host)` and set `host.puzzle.maybeOwned = undefined`.
1. Pass the failed status into `transportModel` so Play and Stop render `aria-disabled` while failed.
1. Add a Vitest test: put the host in a failure view in building and in running, call `playPuzzle`, `stopPuzzle` and `applyPuzzleCommand({kind: "togglePlay"})`, and assert that the view stays `failure` and nothing is recorded in the star store.

#### W-WR-02: Space on a non-button control such as the "Failure details" `<summary>` is taken as Play/Stop and its default is prevented

**File:** `web/src/puzzle/keyboard.ts:63-64, 77-95`; `web/src/components/PuzzlePanel.tsx:166-181`

**Issue:** `keyTargetOf` returns `"other"` for any element that is not a canvas, button, anchor, select, input or textarea. That includes the `<summary>` of the failure overlay's "Failure details" disclosure (`PuzzlePanel.tsx:383-389`), which is inside the panel `<section>` that owns the key handler. Space then maps to `togglePlay`. In building or running, `togglePlay` returns `true`, so `preventDefault()` runs and the disclosure cannot be opened with Space (WCAG 2.1.1). Combined with W-WR-01, Space can also start or stop a run behind the failure overlay. The same applies to Arrow, R, Delete and Escape on any other focusable element that is not a button, which may be added to the panel later.

**Fix:** Classify natively activatable and ARIA-activatable elements as buttons, and only route Space and Enter from the canvas or HUD controls.

```ts
if (
  target instanceof HTMLButtonElement ||
  target instanceof HTMLAnchorElement ||
  target.tagName === "SUMMARY" ||
  target.getAttribute("role") === "button"
) {
  return "button";
}
if (target.isContentEditable || /* select, input, textarea */ ...) {
  return "formControl";
}
```

Alternatively, skip the panel key map for events whose target is inside `.puzzle-failure`.

#### W-WR-03: Holding Space (key auto-repeat) toggles Play and Stop over and over

**File:** `web/src/components/PuzzlePanel.tsx:166-181`; `web/src/player/puzzle-lifecycle.ts:147-161`

**Issue:** `handleKeyDown` does not check `event.repeat`. If Space is held past the OS repeat delay (about 500 ms), the first keydown builds a run session and the repeats alternate Stop and Play. Each repeat disposes and rebuilds a wasm session, records any latched stars and announces "Stopped…". A user who presses Space a little too long, or who uses a long key-press (common with motor impairments), sees the run they just started stopped at once. Escape and Delete repeats are harmless, but the Play/Stop toggle is not.

**Fix:** Ignore repeats for the transport toggle. Arrow-nudge repeats can stay, because they are useful.

```ts
const maybeCommand = puzzleKeyCommand({ ... });
if (maybeCommand?.kind === "togglePlay" && event.repeat) {
  event.preventDefault();
  return;
}
```

You could also add a `repeat` field to `PuzzleKeyInput` and handle this inside the pure map, with a unit test.

#### W-WR-04: The editor pointer accepts any mouse button, so right-click places or moves tools

**File:** `web/src/player/puzzle-editor-input.ts:170-196`

**Issue:** `onDown` does not filter `event.button` or `event.isPrimary`. With a kind armed, a right-click (button 2) or middle-click (button 1) places a tool and also opens the context menu or starts auto-scroll. Right-dragging a tool moves it, and right-clicking empty canvas deselects. None of these match the tap-to-arm and drag contract in D-17 and D-18.

**Fix:**

```ts
function onDown(event: PointerEvent): void {
  if (maybePress !== undefined || !event.isPrimary) {
    return;
  }
  if (event.pointerType === "mouse" && event.button !== 0) {
    return;
  }
  ...
}
```

### Info

#### W-IN-01: Validity changes after a pointer drag are never announced; the `move` announcement branch cannot be reached

**File:** `web/src/puzzle/editor-announce.ts:137-138`; `web/src/player/puzzle-editing.ts:83-93`; `web/src/player/puzzle-editor-input.ts:163-168`

**Issue:** Pointer moves are always sent as `"live"`, and the drag commits through `settle → inspectNow → commitNow`, which never calls `announce`. The `case "move"` in `editAnnouncement` is therefore dead code. The UI-SPEC copy "Now overlaps the cup." and "This tool fits again." is never announced for a drag, even though pointer-up counts as a committed edit (UI-D14). The polite result message still shows the status line, so the information does reach the user.

**Fix:** In `inspectNow`, take `before.editor` from the editor at press start (or from the last committed state). After the commit, call `editAnnouncement(before, after, { kind: "move", id, x, y })`. Alternatively, remove the dead branch.

#### W-IN-02: The Play button's enabled state disagrees with `canPlay` during drags

**File:** `web/src/puzzle/hud-format.ts:39-46` vs `web/src/puzzle/reduce-puzzle.ts:105-111`

**Issue:** `playControl` uses `committedPlacementsValid`, which ignores the revision, so the button does not flicker. `canPlay` requires a fresh inspection (`allPlacementsValid`). During a drag, Play looks enabled (accent, no `aria-disabled`), but clicking it with a second finger or pressing Space does nothing. `data-puzzle-play-enabled` then reads `false` while the button looks enabled. The UI-SPEC accent rule says "A disabled Play (invalid placements, stale inspection) uses the Phase 38 disabled look." The divergence is intentional per the code comment, so it should be recorded in the SUMMARY, or the button should be disabled only while a pointer press is active.

#### W-IN-03: The Brim count and peak-held count in the frame are not checked against level info or across frames

**File:** `web/src/physics/frame-puzzle.ts:196-203`; `web/src/puzzle/reduce-puzzle.ts:269-328`; `web/src/player/puzzle-run.ts:154-159`

**Issue:** The glass-cup Brim tick uses `levelInfo.brimCount` (`puzzle-host.ts:115`). The meter, star rows and pending text use the frame's `status.brimCount`. Nothing checks that the two are equal. The Bun replay script does check `puzzle.brimCount !== level.brimCount`, but the player does not. D-08 says the peak never decreases, but `stepRunning` does not check that across frames. A Rust regression in either value would display inconsistently instead of failing closed.

**Fix:** In `buildPuzzle` and `playPuzzle`, require `status.brimCount === info.brimCount`. In `stepRunning`, throw if `status.brimCount !== state.status.brimCount` or `status.peakHeldCount < state.status.peakHeldCount`.

#### W-IN-04: The inspector decoder accepts a degenerate outline, which `distanceToQuad` treats as containing every point

**File:** `web/src/physics/puzzle-inspector.ts:52-60, 75-81`; `web/src/puzzle/hit-test.ts:35-43`

**Issue:** For geometric statuses, `decodeOutline` only checks that coordinates are finite. If a quad has all four corners equal (for example the eight-zero "no outline" lane sent for a geometric status by mistake), every `cross` is 0. The `sides.every(side >= 0)` branch then returns distance 0 for every point, so that tool would win every press on the canvas. The current Rust code emits outlines for exactly the statuses `isGeometricStatus` expects, so this is defensive.

**Fix:** In `decodeOutline`, reject a geometric outline with zero area (shoelace area of 0) using `invalidInspection()`, and add a Vitest case.

#### W-IN-05: An inspector failure shows the build-failure copy, not the UI-SPEC "could not continue" copy

**File:** `web/src/components/PuzzlePanel.tsx:52-54, 294-298`

**Issue:** The UI-SPEC Copywriting Contract specifies "This level could not continue" and "Something went wrong while checking your tools. Try again, or open a scene from the list." for inspector failures. The panel always shows the Phase 38 "This level could not start" and "…while building the level…" text.

**Fix:** Have the failure carry a kind, for example `failScene(error, "inspect")`, or pick the copy from the puzzle phase when the failure happened.

#### W-IN-06: An in-flight pointer press survives phase changes, and `detach()` runs a commit inspection during teardown

**File:** `web/src/player/puzzle-editor-input.ts:163-168, 255-265, 311-323`

**Issue:**

1. `detach()` calls `cancel()`, which calls `finishPress`, which calls `deps.settle()`. On route leave (`leavePuzzle`) or canvas reassignment, this runs the wasm inspector, redraws and publishes while the route is being torn down.
1. A press is not cancelled when the phase changes. For example: arm, press the canvas, Space (Play), Space (Stop), release. The release still runs `tap()` with the stale `{ kind: "place", toolKind }` captured at press time, so it places a tool even though Play disarmed the kind.

**Fix:** In `detach()`, call `abandonPress` instead of `finishPress`. In `onUp` and `onMove`, abandon the press if `deps.state()` left building since the press started, for example by storing the `revision` and phase at press time. Alternatively, call `maybeEditorInput?.cancel()` from `playPuzzle`, `stopPuzzle` and `beginPuzzleRoute`.

#### W-IN-07: The `KeyTarget` `"control"` variant is never produced

**File:** `web/src/puzzle/keyboard.ts:2, 77-95`

**Issue:** `keyTargetOf` never returns `"control"`, and `puzzleKeyCommand` never treats it differently from `"other"`. This dead variant suggests a distinction between HUD controls and other elements that the code does not implement. That missing distinction is related to W-WR-02.

**Fix:** Either remove the variant, or return `"control"` for elements inside `[data-hud-obstacle]` and only route Space/Tab when the target is `canvas` or `control`.

---

_Reviewed: 2026-10-13T02:22:51Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
