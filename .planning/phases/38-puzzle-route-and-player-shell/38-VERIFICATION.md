---
phase: 38-puzzle-route-and-player-shell
verified: "2026-10-12T04:07:36.303Z"
status: passed
human_approved_at: "2026-10-12T15:20:04Z"
human_approved_by: "user (repository owner) via chat: approved, push it"
score: 5/5 roadmap success criteria verified (52/52 plan must-have truths); 2 visual/product-judgement items need a human
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: "2026-10-12T04:07:36.303Z"
lifecycle_validated: true
overrides_applied: 0
human_verification:
  - test: "Open #/puzzle/first-pour on a desktop browser and a phone, once with the canvas render mode and once with the shaded-blob (WebGL) mode from Puzzle options. Watch building, press Play, and let the run end."
    expected: "The cup reads as glass: faint tint, bright thin edges, a highlight streak, an etched amber dashed target line at 60%, a star tick at the target and a plain tick at the brim. The overlay stays above the water in both modes, and no percentage text is drawn on the canvas."
    why_human: "Whether the cup 'reads as glass' is a visual judgement. The AI verifier inspected screenshots captured earlier in the phase (target/phase38/screens/*.png) and the CSS stacking. It cannot confirm the look on real devices or in the WebGL mode."
  - test: "Judge whether the fixed HUD fill chip (left rail on desktop, top band on phone), together with the on-canvas glass target line and ticks, satisfies 'a fill meter at the cup' (SC-1, PUZ-04)."
    expected: "The owner accepts the D-11/D-12 split: the HUD chip is the authoritative numeric meter, and the glass cup carries the target line and ticks. Otherwise, the owner records a follow-up to move the readout next to the cup."
    why_human: "This is an interpretation of requirement wording, locked in CONTEXT D-11/D-12, that only the product owner can confirm. The chip is a fixed HUD obstacle leaf, not a canvas label beside the cup."
---

# Phase 38: Puzzle Route and Player Shell Verification Report

**Phase Goal:** Players can open a level at `#/puzzle/<id>` inside the existing player and watch a run with a glass cup, a live fill meter, a step timer and a result card, while the 25 scenes behave exactly as before.
**Verified:** 2026-10-12T04:07:36Z, at HEAD `3f55978ee` (after the review fixes `a3b5e3214`, `6fceb3ae3` and `96c10546b`)
**Status:** passed (automated checks by AI verifier; the 2 human items were approved by the repository owner in chat at 2026-10-12T15:20:04Z; see 38-HUMAN-UAT.md)
**Re-verification:** No. This is the initial verification.
**Verifier:** Claude (gsd-verifier), an AI verifier. This is not a human review.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| - | ----- | ------ | -------- |
| 1 | Opening `#/puzzle/<first-level>` shows the level with a glass cup, an etched target line, star ticks and a fill meter whose percentage rises as that level's water settles in the cup. `#/puzzle/<unknown>` shows a helpful fallback with a way back. | ✓ VERIFIED (visual look needs human) | `playable-route.ts` parses against `PUZZLE_IDS`. `PlaygroundStage.tsx` `<Match>` mounts `PuzzlePanel`, or `PuzzleFallbackPanel` for unknown ids. `scene-surface.ts:72-76` calls `drawGlassCup` after `presentSceneFrame` on puzzle routes only. `glass-cup.ts` draws the tint, edges, highlight, the dashed amber target at `minY + (target/full) x height`, and the ticks. The scene canvas (z-index 1) sits above the WebGL particle surface (z-index 0), so the overlay shows in both modes. The meter reads `RenderFrame.maybePuzzle.goalCount`, counted in Rust (Phase 37). The e2e `wins with the recorded solution` polls `aria-valuenow > 0` while running and ends at `76`. The e2e `shows a helpful fallback` checks "Level not found", the raw id as text, and both links. |
| 2 | The timer runs on simulation time and only during a run. A loss shows "Out of time: N% of M%". A win shows the result card with Next, Retry and Edit. | ✓ VERIFIED | `remainingSimulationSteps` (`reduce-puzzle.ts`) shows the full limit in building, `stepLimit - stepIndex` while running, and freezes at the latch. `formatSimulationTime` uses integer step math, and the puzzle code makes no wall-clock calls (grep is clean). The e2e asserts `0:12.0` in building, `Out of time: 0% of 60%` with `0:00.0` on a natural run, and `Filled! 76% of 60%` with `0:08.6` through the seam. Next is disabled with `Next: No next level yet`; Retry and Edit are present. |
| 3 | On puzzle routes, pointer poke and drag, tilt, the gravity slider, zoom and pan are inactive, scene-only chrome is hidden, and an e2e shows the camera scale stays constant between building and running. | ✓ VERIFIED | `assignPuzzleCanvas` attaches no pointer handler. `syncCanvasInteractive` is gated on `route().kind === "scene"`. `beginPuzzleRoute` stops tilt, turns pan off, resets the camera view and clears construction values. `PuzzlePanel` imports no ViewportTools, tilt, slider or scene chrome (checked by `puzzle-chrome.test.ts`). The e2e `turns scene interactions and chrome off` keeps `data-pointer-accepted` at 0. `keeps the camera fixed across building, running and ended` and the new `keeps the camera fixed from the first building frame after in-app entry` pass at 1280x800 and 390x844. |
| 4 | The fill meter, timer and result card are `data-hud-obstacle` leaves in fixed slots, the scale legend shows, and the portrait frame comes from a separate puzzle bounds registry with its own fill test. | ✓ VERIFIED | `PuzzleHud.tsx` and `PuzzleResultCard.tsx` put `data-hud-obstacle` on each leaf. All leaves mount for the whole route, and `puzzle.css` gives them fixed boxes. `ScaleLegend` renders in `.puzzle-legend-slot` in both layouts (e2e). `catalog/puzzle-bounds.ts` is separate from `portrait-bounds.ts` (unchanged), and `tests/puzzle-bounds.test.ts` checks fill and containment for each `PuzzleId`. The e2e `keeps every HUD leaf clear of the world` passes at both viewports. |
| 5 | The 25 scenes behave exactly as before: 25/25 fingerprints unchanged, player smoke and the protected tests pass without edits, and no level appears in `SCENE_IDS` or the scene sidebar. | ✓ VERIFIED | `git diff 589cb0fb9 -- crates` is empty (0 lines), as is the diff for `tools`, `Cargo.toml` and `Cargo.lock`. `just playground-scene-spot` is a Rust `cargo xtask`, so the 25/25 evidence in `target/phase38/after-final.txt` still applies: its diff against `base.txt` is empty, with 25 lines. The protected files `hash.test.ts`, `portrait-bounds.test.ts`, `hud-safe-frame.spec.ts` and `readme-svg.test.ts` are byte-identical to `589cb0fb9`, and so are `hash.ts`, `puzzles.ts`, `scenes.ts`, `portrait-bounds.ts`, `plans.ts`, `DemoNavigation.tsx`, `PlayerPanel.tsx` and `FallbackPanel.tsx`. Re-run on HEAD: HUD safe-frame 64/64 and player smoke 45/45. `first-pour` is absent from scene catalogs, README plans, portrait bounds and DemoNavigation. The e2e `keeps levels out of the scene catalog routes` and `returns to scenes cleanly` pass. |

**Score:** 5/5 roadmap truths verified. All 52 plan frontmatter truths across 38-01 to 38-09 are covered by the code inspection and the passing suites below.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `web/src/routing/playable-route.ts` | PlayableRoute parser delegating to the scene parser | ✓ VERIFIED | Calls `maybeParseSceneRoute(hash)` for non-puzzle hashes, so `#/puzzles` stays a scene `unknown` |
| `web/src/player/playable-target.ts` | PlayableTarget union and helpers | ✓ VERIFIED | Used by `frame-loop.ts`, `playground-session.ts` and `scene-lifecycle.ts` |
| `web/src/catalog/puzzle-levels.ts` | Titles, next id, `puzzleHash` | ✓ VERIFIED | `maybeNextPuzzleId('first-pour')` is undefined; `format.ts` now uses `puzzleHash` (IN-02) |
| `web/src/puzzle/reduce-puzzle.ts` | Pure phase machine, 120-step tail | ✓ VERIFIED | Latch only at `maybeOutcomeStep === stepIndex`; an outcome flip throws; holds at `tailEndStep` |
| `web/src/puzzle/format.ts` | Timer, fill and result view models | ✓ VERIFIED | Percentages are floored; the tick list is data-driven `[★ Win, Full]` |
| `web/src/puzzle/placements.ts` | Lane encoder and test seam | ✓ VERIFIED | Reads only the `__liquidfunPuzzleTestPlacements` window global; nothing reads the URL or storage |
| `web/src/catalog/puzzle-bounds.ts` | Separate landscape and portrait registry | ✓ VERIFIED | `worldBoundsForTarget` passes scenes to `worldBoundsForViewport` unchanged |
| `web/src/render/glass-cup.ts` | Glass overlay | ✓ VERIFIED | Wrapped in `save`/`restore` and draws no text |
| `web/src/physics/puzzle-session.ts`, `puzzle-loader.ts` | Direct-backend adapter | ✓ VERIFIED | Step-0 `captureFrame` does not advance; scene input throws `Puzzle sessions accept no scene input`; uses `ProofSession.newPuzzle` |
| `web/src/player/puzzle-runtime.ts`, `puzzle-loop.ts`, `puzzle-lifecycle.ts` | Runtime, loop and lifecycle | ✓ VERIFIED | Single steps with the 4-step cap; title check fails closed; the WR-01 guard is `maybeLevelId() === id` |
| `web/src/components/Puzzle*.tsx`, `web/src/styles/puzzle.css` | HUD, result card, options drawer, fallback | ✓ VERIFIED | Fixed slots; `aria-live="polite"` on the result |
| `web/e2e/puzzle.spec.ts`, `puzzle-helpers.ts`, `test:puzzle` script | Puzzle e2e | ✓ VERIFIED | 16 tests, including the WR-02 in-app entry case at both viewports |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `PlaygroundStage.tsx` | `PuzzlePanel` | `<Match when={maybePuzzleId()}>` | ✓ WIRED | Passes `assignCanvas={props.assignPuzzleCanvas}` and the controller |
| `scene-surface.ts drawSceneFrame` | `drawGlassCup` | after `presentSceneFrame`, puzzle route only | ✓ WIRED | Held-frame and resize repaints go through the same `drawSceneFrame` |
| `frame-loop.ts connectResizeObserver` | `worldBoundsForTarget` / `maybeStartPuzzle` | target dispatch | ✓ WIRED | Scenes still call `startScene` |
| `scene-lifecycle.ts handleHashChange` | `beginPuzzleRoute` / `leavePuzzle` | puzzle arm before the scene lines | ✓ WIRED | The scene lines below it are unchanged |
| `puzzle-lifecycle.ts` | `loadPuzzleSession`, `changeTiltGravity(false)`, `puzzleTitleForId` | entry and build | ✓ WIRED | |
| `puzzle-loop.ts` | `reducePuzzle` via `dispatch` and `puzzleStepBudget` | one stepped event per step | ✓ WIRED | |
| `PuzzleHud` / `PuzzleResultCard` | `fillMeterModel`, `timerText`, `resultCardModel`, `resultActionsModel` | pure view models | ✓ WIRED | |
| `PuzzlePanel` | `createHudObstacles` | `onMount` | ✓ WIRED | Since WR-02, the first measurement is synchronous |
| `puzzle-helpers.ts` | `first-pour-solution.txt` | `parseFixture` and `addInitScript` | ✓ WIRED | The seam win e2e passes |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `PuzzleFillMeter` | `state.status.goalCount` / `state.latch` | `RenderFrame.maybePuzzle` from the wasm `captureFrame` (Rust goal counter) | Yes: rises to 76 with the solution, 0 on the natural run | ✓ FLOWING |
| `PuzzleTimer` | `stepIndex`, `stepLimit` | frame step index and Rust `stepLimit` | Yes: `0:12.0`, then `0:08.6` or `0:00.0` | ✓ FLOWING |
| `PuzzleResultCard` | `latch` | reducer latch at the Rust outcome step | Yes: win and loss copy asserted in e2e | ✓ FLOWING |
| glass overlay | `puzzle.maybeGlass` | `levelInfo().cupInterior`, `targetCount`, `fullCount` | Yes, parsed from wasm in `buildPuzzle` | ✓ FLOWING |

### Behavioral Spot-Checks (re-run on HEAD `3f55978ee`, 2026-10-12 about 04:03Z to 04:07Z)

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Web typecheck and unit tests | `cd web && bun run typecheck && bun run test:unit` | tsc clean; 74 files, 789 tests passed | ✓ PASS |
| Managed checks | `bun scripts/bright-builds-check.ts all` | 1415 files scanned, 0 findings | ✓ PASS |
| Protected files and engine untouched | `git diff --quiet 589cb0fb9 -- crates/liquidfun web/tests/hash.test.ts web/tests/portrait-bounds.test.ts web/e2e/hud-safe-frame.spec.ts web/tests/readme-svg.test.ts` | exit 0; `git diff 589cb0fb9 -- crates` has 0 lines | ✓ PASS |
| Fresh build | `just web-build` | First attempt exit 1: one failure in `tests/readme-svg-webp.test.ts`, see Anti-Patterns. Second attempt exit 0: 789/789, vite build OK | ✓ PASS (after rerun) |
| Puzzle e2e | `CI=true bun run test:puzzle` | 16 passed | ✓ PASS |
| HUD safe frame (unedited) | `CI=true npx playwright test e2e/hud-safe-frame.spec.ts` | 64 passed | ✓ PASS |
| Player smoke | `just web-player-smoke` | 789 unit tests plus 45 Playwright tests passed, including demo-media-clock | ✓ PASS |
| Scope and security greps | wall-clock calls in `web/src/puzzle` and `Puzzle*.tsx`; URL or storage use in puzzle code; `innerHTML`; `first-pour` in scene catalogs, plans, portrait bounds and DemoNavigation; `MAX_STEPS_PER_FRAME = 4` | no matches (grep rc=1) where none are expected; the cap is 4 in `clock.ts`, `session.ts` and `puzzle-session.ts` | ✓ PASS |
| 25/25 fingerprints | cited: `diff target/phase38/base.txt target/phase38/after-final.txt` | empty, 25 lines each (scene-spot run 2026-10-12T03:33:15Z). Not re-run, because nothing changed in crates, tools or the Cargo manifests since `589cb0fb9` | ✓ PASS (cited) |
| Rust fmt, clippy, build, test | cited: `target/phase38/checks-09.results` | all exit=0 (03:33Z to 03:47Z). Not re-run, because nothing Rust changed after that run | ✓ PASS (cited) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| GOAL-01 | 02, 04, 05, 06, 08, 09 | Live fill % of the level's water in the cup against a fixed capacity | ✓ SATISFIED | Meter fed from the Rust `goalCount`/`goalFull`; the e2e meter rises then latches at 76% |
| GOAL-03 | 02, 04, 05, 06, 08, 09 | Simulation-time timer, only during a run; "Out of time: N% of M%" | ✓ SATISFIED | `remainingSimulationSteps` and `formatSimulationTime`; e2e `Out of time: 0% of 60%` and `0:00.0` |
| PUZ-02 | 01, 06, 07, 08, 09 | Direct `#/puzzle/<id>`; unknown id gets a helpful fallback | ✓ SATISFIED | Parser, `Switch`, fallback panel; e2e at both viewports |
| PUZ-03 | 03, 05, 07, 08, 09 | Scene interactions off; camera does not jump between building and running | ✓ SATISFIED | Chrome-off e2e plus the two camera e2e tests, including in-app entry after WR-02 |
| PUZ-04 | 02, 03, 06, 08, 09 | HUD follows chrome rules: meter with target and ticks, timer, result card with Next, Retry and Edit | ✓ SATISFIED (placement wording needs human) | Leaves carry `data-hud-obstacle`, the legend shows, e2e overlap check passes. "Meter at the cup" relies on the D-11/D-12 split (human item 2) |
| PUZ-05 | 03, 07, 08, 09 | Cup drawn as glass with an etched target line and star ticks | ✓ SATISFIED (look needs human) | `glass-cup.ts`, hook in `drawSceneFrame`, `glass-cup.test.ts` |
| PUZ-06 | 01, 07, 09 | 25 scenes unchanged | ✓ SATISFIED | 25/25 fingerprints (cited, no Rust change); protected tests unedited and passing; smoke 45/45 |

No orphaned requirements. REQUIREMENTS.md maps exactly these seven IDs to Phase 38, and every one is claimed by at least one plan.

### CONTEXT decision spot-checks

- D-01 and D-03: `hash.ts` and `puzzles.ts` are byte-identical to the base, and `#/puzzles` falls through to the scene parser.
- D-04: puzzles use `loadPuzzleSession` and `ProofSession.newPuzzle` only, never the worker.
- D-07: `buildPuzzle` captures step 0 without advancing; Play steps the same session.
- D-08: the latch is observed per step; the card appears at the latch (`ended`, `held:false`), then holds after 120 steps.
- D-15: Next is disabled with the accessible name `Next: No next level yet`.
- D-16: `changeTiltGravity(false)` on entry; no `reapplyStoredTiltGravity` on the puzzle path; pan is off.
- D-24: the options drawer holds only the render mode, the source link and the footer.
- D-26: a title mismatch throws into `failScene`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `web/tests/readme-svg-webp.test.ts` | 97-116 | Timing race: a 512-byte stdin write can resolve before the spawn ENOENT error, so `writeFrame` does not reject | ⚠️ Warning (pre-existing, outside Phase 38) | Failed once inside the full `just web-build` unit run, then passed 8/8 in isolation (8 runs) and on the rebuild. The file and `web/scripts/readme-svg/` are unchanged since `589cb0fb9`. It is a flaky test in README export tooling, not a Phase 38 regression. Worth a separate quick fix. |
| `web/src/player/puzzle-lifecycle.ts` | `leavePuzzle` | Teardown depends on the scene branch (review IN-01) | ℹ️ Info | Latent; it needs `getContext("2d")` to return null. Left open by the review fix scope. |
| `web/tests/puzzle-chrome.test.ts` | | Source-text assertions (review IN-03) | ℹ️ Info | Brittle but backed by the Playwright geometry and chrome checks |
| `web/src/components/PuzzlePanel.tsx`, `puzzle.css` | | Failure overlay sits below the HUD layer (review IN-05) | ℹ️ Info | Cosmetic, failure state only |
| `web/e2e/puzzle.spec.ts` | | No e2e leaves a puzzle mid-run (review IN-04, partly covered by the WR-02 case) | ℹ️ Info | Unit lifecycle tests cover the teardown |

No TODO, FIXME or placeholder stubs were found in the Phase 38 source files. Loading placeholders (`--%`, `-:--.-`) are intentional fixed-box content that real frames replace.

### Human Verification Required

### 1. Glass cup look in both render modes

**Test:** Open `#/puzzle/first-pour` on desktop and on a phone. Switch Puzzle options between canvas and shaded-blob (WebGL) modes, then watch building, a run and the end state.
**Expected:** A translucent glass cup with bright edges and a highlight streak, an etched amber dashed target line at 60% with a star tick, and a brim tick. The overlay stays above the water in both modes, with no text on the canvas.
**Why human:** This is a visual judgement. The AI verifier saw the earlier screenshots (`target/phase38/screens/desktop-won.png`, `phone-lost.png`), which match the design, but cannot judge the look on real devices or in WebGL mode.

### 2. "Fill meter at the cup" wording

**Test:** Compare the HUD fill chip (desktop left rail, phone top band) and the on-canvas target line and ticks with the SC-1/PUZ-04 wording "a fill meter at the cup".
**Expected:** The owner accepts the locked D-11/D-12 split, or records a follow-up.
**Why human:** This interprets requirement wording.

### Gaps Summary

No blocking gaps. Every roadmap success criterion and plan must-have is backed by code that exists, is substantive and is wired, with real data flowing from the Rust puzzle frame. All relevant gates were re-run on the current HEAD and pass. The exception is the 25/25 fingerprint and Rust check evidence, which is cited, because nothing in Rust, wasm, tools or the Cargo manifests changed since `589cb0fb9`.

The review fixes hold:
- WR-01 has a lifecycle unit test.
- WR-02 has a new in-app entry camera e2e at both viewports, and HUD safe-frame (64) and player smoke (45) still pass for scenes after the shared `hud-obstacles.ts` change.
- IN-02 now uses `puzzleHash`.

Status is `human_needed` only because the glass appearance and the meter placement wording need a human to confirm.

---

_Verified: 2026-10-12T04:07:36Z_
_Verifier: Claude (gsd-verifier). This is AI verification, not a human review._
