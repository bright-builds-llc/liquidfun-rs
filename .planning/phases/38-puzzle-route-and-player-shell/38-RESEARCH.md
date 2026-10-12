# Phase 38: Puzzle Route and Player Shell - Research

**Researched:** 2026-10-11
**Domain:** SolidJS 1.9 player shell integration (routing, lifecycle, frame loop, canvas overlay, HUD safe frame) on top of the Phase 37 `ProofSession.newPuzzle` bridge
**Confidence:** HIGH for integration points. Every file, line and behavior below was read from HEAD `3014ffb5d`, whose web and Rust code equal phase base `589cb0fb9`. The first-pour geometry was checked in the generated wasm through Bun. MEDIUM for Solid effect ordering details and the final HUD slot pixel layout.

## Summary

Phase 38 needs no new dependency and no Rust change. Every capability already exists in the repo. You need:

- a wrapper hash parser
- a `PlayableTarget` dispatch at five call sites
- a puzzle session adapter that can capture the step-0 frame without advancing (the direct `SceneSession.nextFrame` always advances at least one step)
- a small puzzle frame loop that clamps steps to the 120-step settle tail
- a glass-cup draw call appended inside `drawSceneFrame`
- a new puzzle panel that reuses the canvas markup, `createHudObstacles`, `ScaleLegend` and the drawer primitive

The scene arm can stay literally the old call path. The scene code paths that must change are the route type (widened), the `bindNavigation` route effect, the resize-observer start and bounds dispatch, the `handleHashChange` puzzle arm, and the overlay hook. Each change is additive, and for scene routes it is a no-op by construction.

Four findings change how you should plan:

1. **The canvas-stage shell is the default on every screen, including desktop.** See `screenShellFromSearch` in `web/src/player/screen-shell.ts:15-22` and `canvasStageActive` in `fullscreen-capability.ts:58-64`. So the D-19 desktop viewport (1280×800) measures HUD obstacles exactly like a phone. The "desktop viewport" layout that does not measure obstacles exists only with `?shell=form-factor` on a fullscreen-capable browser. D-18 asks that both layouts be supported.
1. **In canvas-stage mode, provenance and the GitHub source link exist only inside the controls drawer.** `SiteHeader` hides `GitHubSourceLink` when `canvasStage` is set (`SiteHeader.tsx:42`), and `SiteFooter` is rendered inside `SceneControlsSheet` (`PlayerPanel.tsx:341`). If D-17 ("hide the `SceneControlsSheet` drawer") is applied literally, puzzle routes break the Bright Builds provenance and source-disclosure rule. Keep a puzzle-specific options drawer that holds only the render-mode option and `SiteFooter`, as `FallbackPanel` already does.
1. **Vitest runs in the `node` environment with no DOM library** (`web/vitest.config.ts`). The repo's "component tests" are pure view-model tests or source-text inspection tests (`tests/scale-legend.test.ts:178-191`, `tests/hud-sliders-toggle.test.ts`). Satisfy D-22's "Vitest component tests" with pure view-model functions, such as `resultCardModel(status)`, plus source-inspection tests. Do not add jsdom or a testing library.
1. **Tilt would actively break a puzzle unless it is stopped.** A tilt binding enabled in a scene keeps calling `session.maybeSession.setGravity` on every accelerometer sample (`tilt-binding.ts:28-65`). On a puzzle session that throws: the Rust side returns `UnknownControl`, and the recommended adapter throws as well. The failure path then calls `failScene`. Stop the binding synchronously when entering a puzzle.

**Primary recommendation:** Build puzzle code in new files (`routing/playable-route.ts`, `player/playable-target.ts`, `puzzle/*`, `physics/puzzle-session.ts`, `player/puzzle-lifecycle.ts`, `player/puzzle-loop.ts`, `catalog/puzzle-bounds.ts`, `render/glass-cup.ts`, `components/Puzzle*.tsx`, `styles/puzzle.css`). Touch shared shell files only at the six documented dispatch points. Keep `PlayerPanel.tsx`, `CanvasStage.tsx`, `hash.ts`, `portrait-bounds.ts`, `puzzles.ts` and `canvas.ts` byte-unchanged.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Routing and lifecycle

- **D-01:** Leave `maybeParseSceneRoute` and `normalizeSceneRoute` in `web/src/routing/hash.ts` exactly as they are. Existing hash tests must pass without edits, and they assert that other paths are `unknown`. Add a separate wrapper parser, for example `maybeParsePlayableRoute`, that first recognizes `#/puzzle/<id>` and otherwise delegates to the scene parser. It returns a `PlayableRoute` union:
  - `{kind:"puzzle", id: PuzzleId}`
  - `{kind:"unknown-puzzle", maybeRaw}`
  - the existing scene route kinds

  Puzzle ids are validated against `PUZZLE_IDS` from `web/src/catalog/puzzles.ts`. Any extra segment is unknown, matching the scene rule.
- **D-02:** Replace `maybeReadySceneId` with a `PlayableTarget` union, `{kind:"scene", id} | {kind:"puzzle", id}`. The scene arm runs today's code path unchanged: the same `beginScene`, resize, controls, gravity, titles and worker choice. Puzzle code mounts only on the puzzle arm. Keep scene behavior unchanged by keeping the scene arm literally the old call path, not by re-implementing it.
- **D-03:** `#/puzzles` is not added in this phase and stays an unknown route (Phase 41). Levels do not appear in `SCENE_IDS`, `SCENES`, the scene sidebar (`DemoNavigation`), README plans, `PORTRAIT_VIEW_BOUNDS` or the scene survey.
- **D-04:** Puzzles always use the direct backend (`new ProofSession` path), never the worker. The worker path stays scene-shaped. Each run builds a fresh session with `ProofSession.newPuzzle(levelId, kinds, poses)`. In Phase 38 the placements are empty by default, but the lifecycle takes placements as an input so Phase 39 can supply them.
- **D-05:** The unknown-puzzle fallback reuses the `FallbackPanel` pattern with puzzle wording. It shows "Level not found", the raw id (text only, never HTML), a primary link to the first level (`#/puzzle/first-pour`) and a secondary link back to the scenes (`#/scene/wave-machine`). Page title and `data-*` route attributes follow the existing route-identity pattern.

#### Puzzle phase machine (minimal for this phase)

- **D-06:** Add a pure `reducePuzzle` state machine. Its phases are:
  - `loading`
  - `building`: step-0 frame held, untimed
  - `running`
  - `ended`: outcome latched, with a short settle tail, then held

  In Phase 38 the only transitions are:
  - Play (building → running)
  - Retry (ended → running, with a fresh session and the same placements)
  - Edit (ended → building, with a fresh step-0 session and the same placements)
  - route change (dispose)

  Stop/Reset during a run, Clear and the tool editor are Phase 39. Unit-test the reducer separately from the DOM.
- **D-07:** Building shows the level's step-0 frame from a fresh session that has not been advanced. Play starts stepping that session, or an equivalent fresh one. It must not edit a live world.
- **D-08:** After the Rust outcome latches (`outcome` 1 or 2), the simulation keeps stepping for a fixed 120-step (2 s simulated) settle tail so the water visibly settles. Then playback holds the last frame. The timer and fill shown in the result use the latched values (`outcomeStep` and the count at the latch), not the values after the tail. The latch step is visible in the frame. The result card appears at the latch, without waiting for the tail.

#### Timer

- **D-09:** The timer counts down the remaining simulation time, `(stepLimit − step) / 60`, shown as `0:12.3` with `tabular-nums`. It is always visible on puzzle routes:
  - In building, it shows the full limit, frozen.
  - In running, it counts down from the frame's step index.
  - In ended, it freezes at the outcome step.

  It reads only the frame step index and `stepLimit`, never wall-clock time. Its accessible name says it is simulation time, for example `aria-label="Simulation time left"`.
- **D-10:** The loss text follows the requirement: "Out of time: 54% of 80%". Both percentages are rounded down to whole percent of `full_count`. The fill is `goalCount / goalFull` at the latch, and the target is `goalTarget / goalFull`. Rounding down keeps a loss from ever displaying a fill equal to the target. Put this formatting in pure, unit-tested functions.

#### Fill meter (HUD) and glass cup (canvas)

- **D-11:** The **HUD fill meter** is the authoritative readout. It is a fixed-slot chip with a short bar, a percentage (`54%`), a target marker and star ticks, and it marks the target as `/ 80%`. It updates from `RenderFrame.maybePuzzle` each presented frame.
- **D-12:** The **glass cup** is drawn on the canvas in world space from `levelInfo().cupInterior`. It has:
  - a faint translucent tint over the interior
  - brighter thin edge strokes with a highlight streak, so the existing physics cup walls read as glass
  - an etched (dashed or engraved-look) horizontal target line at the interior height that matches `targetCount / fullCount`
  - short star ticks on the cup wall

  Map the line height linearly from the interior bottom, as `fraction × interior height`. This is a visual aid, not a physics claim. Draw no fill-percentage text on the canvas. The overlay must stay visible in both the canvas and the shaded-blob (WebGL) render modes, so it draws above the particle layer. It is also repainted on held-frame repaints.
- **D-13:** In this phase the star ticks are the ★ Win target and a brim tick at 100% of `full_count`. The tick list is data-driven (`{fraction, label}[]`), so Phase 39 can add the ★ Brim threshold from Rust without restructuring the overlay or the meter.

#### Result card

- **D-14:** The result card is a fixed HUD slot. The slot exists for the whole puzzle route, so the obstacle set never changes. Content swaps inside it:
  - In building and running, the slot is empty or a compact placeholder of the same box size.
  - In ended, it shows the outcome.
  - A win reads "Filled! 86% of 80%".
  - A loss reads "Out of time: 54% of 80%".

  Both outcomes show the actions Next, Retry and Edit.
- **D-15:** Next goes to the next entry in `PUZZLE_IDS`. When there is none (Phase 38 has only `first-pour`), Next is shown disabled with the accessible text "No next level yet", so the layout stays fixed. Retry and Edit behave as in D-06. Buttons use touch targets of at least 44 px.

#### Interactions, chrome and camera

- **D-16:** On puzzle routes:
  - Pointer poke and drag are not attached, or not forwarded.
  - Pan and zoom are off: no `ViewportTools` zoom or pan buttons, and the camera view stays at reset.
  - Tilt is off: no `AccelerometerToggle` or `TiltPane`, and `reapplyStoredTiltGravity` is never called for puzzle sessions.
  - The gravity slider is not appended or rendered.
  - The interaction hint is hidden.

  Rust already rejects controls and gravity for puzzle sessions with `UnknownControl`. TS must not send them.
- **D-17:** Hide this scene-only chrome on puzzle routes:
  - `SceneControls` and `SceneCredits` (`PlayerSceneChrome`)
  - `AnimatedSvgPane` and export
  - the scene `PlaybackButtons`, which the puzzle transport replaces
  - `HudControlSlider` and the sliders toggle
  - `DebugReadout`
  - the `SceneControlsSheet` drawer

  Keep this chrome: site header, sidebar navigation (scenes only), the render-mode option and `ScaleLegend`.
- **D-18:** Puzzle HUD leaves are the fill meter, timer, puzzle transport (the Play button; Phase 39 extends this slot), result card and level title. Each carries `data-hud-obstacle` on its leaf element. Each has a box size that is fixed per viewport layout and does not depend on puzzle phase, text content or camera scale. All of them mount when the puzzle route mounts. None appear or disappear when switching between building, running and ended, so `safeFrame` never refits. Puzzle HUD chrome is laid out for both the desktop viewport and the canvas-stage layout, and both show `ScaleLegend`.
- **D-19:** Add an e2e check (a new spec, or a new block in a new puzzle spec) that opens `#/puzzle/first-pour`. It records `data-world-frame` and the published pixels-per-meter in building, presses Play, waits for running steps, then waits for the ended state, and asserts that the frame and scale are identical across all three. Cover a desktop viewport and a phone portrait viewport.

#### Puzzle bounds registry

- **D-20:** Add `web/src/catalog/puzzle-bounds.ts`, separate from `portrait-bounds.ts`. Each `PuzzleId` gets a landscape world rectangle and a portrait world rectangle. `worldBoundsForTarget(target, viewport)` dispatches: scenes call the existing `worldBoundsForViewport` unchanged, and puzzles use the new registry by orientation. Both puzzle rectangles must contain the cup, the walls, the build area and the liquid spawn. The portrait rectangle is sized so the action fills an iPhone canvas (about 390 by 844 CSS pixels) with a small margin, following the repo's portrait framing guidance. Do not pad for HUD chrome; the safe frame handles that.
- **D-21:** Add a separate Vitest test, for example `web/tests/puzzle-bounds.test.ts`, keyed by every `PuzzleId`. It checks that each portrait frame fills the phone under the same rule as `portrait-bounds.test.ts`. It also checks that each rectangle contains the cup interior and the build area. Hardcode the level geometry the test compares against from the Rust `first-pour` spec, cross-checked once in the plan by reading `levelInfo()` from the wasm package. A Bun or Vitest check against the generated wasm is acceptable if cheap. The existing `portrait-bounds.test.ts` stays unedited.

#### Test seam for the win path

- **D-22:** Phase 38 has no tool UI, so a natural run of `first-pour` always loses. The win path of the result card is covered two ways:
  - Vitest component and formatter tests with fake `PuzzleFrameStatus` values for won and lost.
  - One e2e that injects the recorded reference placements from `crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt` through a test-only seam, then sees the win card with Next, Retry and Edit.

  The seam is a window global read once at puzzle start, in the style of `__liquidfunSyntheticAnimationClock`. It must not be reachable from the URL. Use the synthetic clock or controlled refresh helpers where available so the run is not wall-clock bound. The natural empty-placement e2e asserts the loss card text pattern `Out of time: N% of M%`.

#### Gates

- **D-23:** The phase is done only if all of these hold:
  - `just playground-scene-spot` reports 25/25 fingerprints identical to the pre-phase baseline (record the baseline from phase-base HEAD `589cb0fb9`).
  - `git diff --stat 589cb0fb9 -- crates/liquidfun` is empty.
  - `just web-player-smoke` passes.
  - These existing tests pass without edits: `tests/hash.test.ts`, `tests/portrait-bounds.test.ts`, `e2e/hud-safe-frame.spec.ts` and `tests/readme-svg.test.ts` (README plan coverage).
  - Rust fmt, clippy (`-D warnings`), build and tests pass, plus the touched Vitest suites, `npx tsc --noEmit` for web, and `bun scripts/bright-builds-check.ts all`, which enforces file-length limits. Watch the limits for `scene-records.ts`, `PlayerPanel.tsx` and `PlaygroundStage.tsx`, and add new files rather than grow those.

  Phase 38 should not need any Rust change. If a bridge change becomes necessary, it requires the fingerprint gate.

### Claude's Discretion

- Exact component and file split: for example `PuzzleHud.tsx`, `PuzzleResultCard.tsx`, `puzzle-lifecycle.ts`, `reduce-puzzle.ts`, `render/glass-cup.ts`.
- Exact HUD slot placement per layout, within D-18.
- Glass styling details: alpha, stroke widths, highlight placement, and the etched-line dash pattern.
- Whether the step-0 building frame comes from the same session that Play then advances, or from a second fresh session, within D-07.
- The exact landscape and portrait rectangles for `first-pour`, within D-20.
- Whether the Play button reuses existing `PlaybackButtons` styling primitives.

### Deferred Ideas (OUT OF SCOPE)

- `#/puzzles` level grid and a sidebar Puzzles section: Phase 41
- Stop/Reset during a run, Clear, the tool tray and editor: Phase 39
- ★ Brim threshold data from Rust and star awarding: Phase 39 (the tick list is ready for it)
- Fast-forward 2×/4×: Phase 39
- Worker backend for puzzles: only if phone step cost needs it (research open question)
- `EXT_color_buffer_half_float` acceptance fix in `webgl-particles.ts`: recommended separate quick task (research SUMMARY), not part of this phase

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| GOAL-01 | Live fill % for the cup, counting only the level's water inside it, against a fixed capacity | Rust already counts per step and publishes `RenderFrame.maybePuzzle.goalCount/goalFull` (parsed fail-closed, `frame.ts:36-63`). The HUD meter shows `floorPercent(goalCount, goalFull)`, clamped to 100. The glass cup comes from `levelInfo().cupInterior` = `[1.6, 0, 3.4, 1.8]` m (verified in wasm). |
| GOAL-03 | Timer on simulation time, only during a run; loss message "Out of time: N% of M%" | `formatSimulationTime(stepLimit − stepIndex)` reads only `frame.stepIndex` and `stepLimit` = 720. The latch count is read from the per-step frame at `stepIndex === outcomeStep`, which the puzzle loop sees because it steps one at a time (see Pattern 4). Empty run: `Out of time: 0% of 60%`. |
| PUZ-02 | `#/puzzle/<id>` opens the level; an unknown id shows a fallback | `maybeParsePlayableRoute` wraps `maybeParseSceneRoute` (D-01) and renders `PuzzleFallbackPanel`. `hash.ts` is unchanged. |
| PUZ-03 | Scene interactions off; the camera does not jump between building and running | No canvas pointer attach (`assignPuzzleCanvas`), tilt stopped, no gravity slider, no ViewportTools, `IDENTITY_CAMERA_VIEW`. All HUD leaves are fixed-size and always mounted. The D-19 e2e reads `data-world-frame` (`camera-frame.ts:38-53`) and a published `data-pixels-per-meter`. |
| PUZ-04 | Puzzle HUD follows chrome rules: `data-hud-obstacle` leaves, scale legend, meter, timer, result card | `createHudObstacles` (`hud-obstacles.ts`) is reused inside `PuzzlePanel`, and `ScaleLegend` sits in a fixed-width leaf. `tests/puzzle-chrome.test.ts` inspects the source. |
| PUZ-05 | Cup drawn as glass with an etched target line and star ticks | `render/glass-cup.ts` is drawn after `presentSceneFrame` inside `drawSceneFrame` (`scene-surface.ts:54-71`). The 2D canvas is z-index 1, above the WebGL particle surface at z-index 0 (`player-viewport.css:47-65`). |
| PUZ-06 | 25 scenes unchanged | No Rust change, so fingerprints hold by construction. The gate reruns `just playground-scene-spot`. The scene arm keeps the old call path. The listed tests are verified unchanged with `git diff --quiet 589cb0fb9 -- <files>`. |

</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md / Bright Builds)

- **GSD workflow:** edits happen inside `/gsd-execute-phase`. `.planning/**` is never formatted with mdformat.
- **File length:** `bun scripts/bright-builds-check.ts all` fails any tracked source file over **628** physical lines. Functions over about 161 lines should trigger a refactor.
- **Naming:** use the `maybe` prefix for optional or nullable internal names and fields. Use `camelCase`. Make illegal states unrepresentable. Parse boundary data into domain types (use this for `levelInfo` and the test seam).
- **Errors:** propagate errors instead of swallowing them. Avoid `catch {}` (it already appears in `session.ts:145` for gravity; do not copy that pattern into puzzle code). Fail closed.
- **Tests:** test behavior, with one concern per test and explicit `// Arrange`, `// Act`, `// Assert` comments.
- **Functional core, imperative shell:** the reducer, formatters, bounds and glass geometry are pure. The lifecycle and loop form the shell.
- **Frontend:** SolidJS. Dark mode is the default. Public open-source pages must expose the source link and the version, commit and build provenance in visible product chrome (this drives the options-drawer recommendation).
- **Playground canvas framing** (CLAUDE.md): the portrait frame is sized to the action and must include the walls. Do not pad for chrome; the safe frame handles that. New on-canvas chrome carries `data-hud-obstacle` on the **leaf**, and its box size must not depend on camera scale.
- **Playground scale legend** (CLAUDE.md): every playground surface shows `ScaleLegend`. Zoom and viewport changes republish `maybePixelsPerMeter`. World units are meters.
- **README scene gallery and portrait bounds** apply to catalog scenes only. Levels must not be added (D-03).
- **Dependencies:** prefer the standard library and existing dependencies. Add no npm package in this phase.
- **Package manager:** Bun 1.4.2 (`web/bun.lock`). Install with `bun install --frozen-lockfile`.
- **Standing authorization:** autonomous fixes and commits are allowed. Package publication is not.
- **Markdown:** if non-GSD Markdown is touched (none expected), run `just markdown-check` (mdformat 1.0.0 on Python 3.13).

## Standard Stack

### Core (all existing; verified in `web/package.json` and `node_modules`)

| Library | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| solid-js | 1.9.15 | UI, signals, `<Switch>/<Match>` for the target switch | Repo framework [VERIFIED: web/package.json] |
| @corvu/drawer (via `components/ui/drawer.tsx`) | ^0.2.4 | Options drawer (render mode and `SiteFooter`) on canvas-stage puzzle routes | Already used by `FallbackPanel` and `PlayerPanel` [VERIFIED] |
| Tailwind v4 plus plain CSS in `src/styles/*.css` | ^4.3.3 | Puzzle HUD styling in a new `styles/puzzle.css` | Player chrome is plain CSS [VERIFIED] |
| Generated `liquidfun-wasm` (`wasm-pack 0.15.0`, gitignored) | built locally | `ProofSession.newPuzzle(id, Uint8Array, Int32Array)`, `levelInfo()`, `captureFrame()` | Phase 37 bridge [VERIFIED: generated d.ts lines 126-249] |
| Vitest | 5.0.1, `environment: "node"` | Pure-module and source-inspection tests | [VERIFIED: web/vitest.config.ts] |
| @playwright/test | 1.63.0, Chromium only | e2e against `vite preview` of `dist` at `127.0.0.1:4173/liquidfun-rs/` | [VERIFIED: playwright.config.ts] |
| TypeScript | 7.0.2 (`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`) | Type checking | [VERIFIED: tsconfig.json] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Pure view-model and source-inspection tests | jsdom with @solidjs/testing-library | That adds two dev dependencies and a second test environment, against the "minimize dependencies" rule. The repo has no DOM test precedent. |
| A new `puzzle-loop.ts` | Reusing `scheduleFrame` in `frame-loop.ts` | `scheduleFrame` cannot clamp to the tail end or hold. Editing it touches the scene loop that D-02 protects. |
| `GeneratedPuzzleSession extends GeneratedProofSession` | Adding `levelInfo` to `GeneratedProofSession` | `tests/session.test.ts:70` has `class FakeGeneratedProofSession implements GeneratedProofSession`, so a new required method breaks that test's typecheck. |

**Installation:** none. Prerequisite for typecheck, Vitest and e2e: `bun scripts/web-build.ts wasm` (about 3 s when cached). Before any Playwright run: `cd web && bun run build:app`, or use `just web-build`.

## Architecture Patterns

### Recommended Project Structure (new files unless marked MOD)

```
web/src/
├── routing/playable-route.ts      # PlayableRoute union, maybeParsePlayableRoute, normalizePlayableRoute (wraps hash.ts)
├── player/playable-target.ts      # PlayableTarget, maybePlayableTarget, samePlayableTarget, puzzle title helpers
├── catalog/puzzle-bounds.ts       # PUZZLE_VIEW_BOUNDS / PUZZLE_PORTRAIT_BOUNDS: Record<PuzzleId, WorldBounds>, worldBoundsForPuzzle, worldBoundsForTarget
├── puzzle/reduce-puzzle.ts        # pure phase machine (D-06, D-08)
├── puzzle/format.ts               # formatSimulationTime, floorPercent, resultCardModel, fillMeterModel, goal ticks
├── puzzle/placements.ts           # Placement type, encodePlacementLanes (WR-01 fix), maybeReadTestPlacements (D-22 seam)
├── physics/puzzle-session.ts      # GeneratedPuzzleSession, parsePuzzleLevelInfo, createPuzzleSession, loadPuzzleSession
├── player/puzzle-lifecycle.ts     # beginPuzzle / playPuzzle / retryPuzzle / editPuzzle / leavePuzzle
├── player/puzzle-loop.ts          # schedulePuzzleFrame with tail clamp and hold
├── render/glass-cup.ts            # glassCupGeometry (pure) + drawGlassCup(context, camera, glass)
├── components/PuzzlePanel.tsx     # canvas markup + obstacles + both layouts
├── components/PuzzleHud.tsx       # title, fill meter, timer, transport, legend leaves
├── components/PuzzleResultCard.tsx
├── components/PuzzleFallbackPanel.tsx
├── styles/puzzle.css
├── player/scene-runtime.ts        # MOD: route: PlayableRoute; puzzle fields
├── player/runtime.ts              # MOD: widen route params to PlayableRoute (bodies unchanged)
├── input/gravity-slider.ts        # MOD: widen route param (body already guards kind === "scene")
├── player/playground-session.ts   # MOD: normalizePlayableRoute; target-based initial view and route effect
├── player/scene-lifecycle.ts      # MOD: handleHashChange puzzle arm; optional startPuzzle in frameDeps
├── player/frame-loop.ts           # MOD: resize observer bounds/start via target
├── player/scene-surface.ts        # MOD: glass overlay after presentSceneFrame; assignPuzzleCanvas
├── components/PlaygroundStage.tsx # MOD: Switch scene | puzzle | unknown-puzzle | fallback
└── App.tsx                        # MOD: pass puzzle props
web/tests/  playable-route, playable-target, reduce-puzzle, puzzle-format, puzzle-placements,
            puzzle-bounds, puzzle-session, puzzle-loop, glass-cup, puzzle-chrome (all new)
web/e2e/    puzzle.spec.ts, puzzle-helpers.ts (new)
```

### Current line counts and limit (628) for files this phase touches

| File | Lines | Planned change | Headroom |
| --- | --- | --- | --- |
| `components/PlaygroundStage.tsx` | 199 | `<Switch>` plus puzzle props (about +45) | OK |
| `components/PlayerPanel.tsx` | 493 | **none** (keep byte-identical) | 135 |
| `components/CanvasStage.tsx` | 413 | **none** | 215 |
| `player/scene-lifecycle.ts` | 311 | puzzle arm, about +20 | OK |
| `player/scene-surface.ts` | 408 | overlay hook and `assignPuzzleCanvas`, about +25 | OK |
| `player/frame-loop.ts` | 305 | resize dispatch, about +10 | OK |
| `player/playground-session.ts` | 176 | puzzle signals and effect, about +25 | OK |
| `player/scene-runtime.ts` | 93 | puzzle fields, about +15 | OK |
| `player/runtime.ts` | 94 | type widening only | OK |
| `routing/hash.ts` | 64 | **none** (D-01) | n/a |
| `App.tsx` | 50 | puzzle props, about +8 | OK |
| `render/canvas.ts` | 399 | **none** (the overlay lives in `glass-cup.ts`) | 229 |
| `catalog/scene-records.ts` | 617 | **none** | 11 |
| `styles/canvas-stage.css` | 417 | **none** (new `puzzle.css`) | 211 |
| `physics/session.ts` | 181 | **none** | n/a |
| `e2e/player-helpers.ts` | 577 | **none** (new `puzzle-helpers.ts`) | 51 |

### Pattern 1: Wrapper route parser (D-01)

**What:** `maybeParsePlayableRoute(hash)` splits exactly like `hash.ts:42-45`. If `parts[0] === "puzzle"`:

- `parts.length === 2` and the id is in `PUZZLE_IDS` gives `{kind:"puzzle", id}`.
- Anything else gives `{kind:"unknown-puzzle", maybeRaw: parts.slice(1).join("/")}`. This covers `#/puzzle` alone, extra segments and unknown ids.

Any other first segment delegates to `maybeParseSceneRoute(hash)`. `normalizePlayableRoute` delegates to `normalizeSceneRoute` unless the parse is a puzzle kind; puzzle kinds never get a replacement hash.

```typescript
// web/src/routing/playable-route.ts
import { PUZZLE_IDS, type PuzzleId } from "../catalog/puzzles";
import { maybeParseSceneRoute, normalizeSceneRoute, type SceneRoute } from "./hash";

export type PlayableRoute =
  | SceneRoute
  | { readonly kind: "puzzle"; readonly id: PuzzleId }
  | { readonly kind: "unknown-puzzle"; readonly maybeRaw: string };
// #/puzzles → parts[0] === "puzzles" → delegates → {kind:"unknown"} (D-03, unchanged scene fallback)
// "https://evil.example/#/puzzle/first-pour" → parts[0] === "https:" → delegates → unknown
```

**Do not** import `PUZZLE_IDS` into `hash.ts`. Keep `puzzles.ts` unchanged too: the Rust mirror test parses `export const PUZZLE_IDS = [` from it with `include_str!` (`crates/liquidfun-wasm/src/puzzle/tests.rs:5-22`).

### Pattern 2: `PlayableTarget` dispatch with the scene arm unchanged (D-02)

Widen the route *type*, not the scene logic. `maybeReadySceneId`, `isReadySceneRoute`, `titleForRoute` (`runtime.ts:16-41`) and `maybeRouteGravitySliderMagnitude` (`gravity-slider.ts:37-45`) already return `undefined`, `false` or `DEFAULT_TITLE` for any `kind !== "scene"`. Changing their parameter type from `SceneRoute` to `PlayableRoute` keeps scene behavior byte-identical, and every existing caller (`scene-surface.ts:213,327`, `worker-controls.ts:28`, `frame-loop.ts:57,266`) keeps compiling.

Exact dispatch points:

| Site | Today | Change |
| --- | --- | --- |
| `playground-session.ts:28,39` | `normalizeSceneRoute`, `isReadySceneRoute` for the initial view | `normalizePlayableRoute`; `maybePlayableTarget(route) !== undefined ? loading : fallback` |
| `playground-session.ts:140-151` route effect | non-scene route calls `session.abandonScene()` | non-target calls `abandonScene()`. **This must change:** the effect runs after the puzzle canvas `ref` is assigned during `setRoute`, so the old code would wipe `maybeCanvas` and disconnect the resize observer. |
| `scene-lifecycle.ts:245-275` `handleHashChange` | `maybeReadySceneId` | Use `normalizePlayableRoute` and `maybePlayableTarget`. If the target is unchanged, return. If it is a puzzle, call `session.beginPuzzleRoute(id)` and return. **Before** the unchanged scene lines, call `session.leavePuzzle()` (a no-op unless the previous target was a puzzle). |
| `frame-loop.ts:266-293` resize observer | `maybeReadySceneId(deps.route())` gives `worldBoundsForViewport` plus `startScene` | `worldBoundsForTarget(target, w, h)`. The scene branch still calls `worldBoundsForViewport(id, w, h)` and `deps.startScene(id)`. Add an **optional** `maybeStartPuzzle?: (id) => void` to `FrameLoopDeps`, because `tests/worker-loop-fixture.ts:94` builds a typed `FrameLoopDeps` literal and a new required key would break it. |
| `scene-surface.ts:54-71` `drawSceneFrame` | `presentSceneFrame(...)` | Add `drawGlassCup(...)` afterwards when `session.maybePuzzleGlass !== undefined && session.route().kind === "puzzle"`. |
| `PlaygroundStage.tsx:134-195` | `<Show when={maybeCurrentSceneId()}>` | `<Switch fallback={<FallbackPanel/>}>`: a `<Match>` for the scene holding the **identical** `PlayerPanel` JSX, plus `<Match>`es for puzzle and unknown-puzzle. Update `routeIdentity` with `puzzle:<id>` and `unknown-puzzle:<raw>`. Add `data-puzzle` and `data-puzzle-phase` attributes on `<main>`. |

`recreateScene`, `applySceneControl`, `createSvgExportRequest` and worker controls all go through `maybeReadySceneId`, so they are already no-ops on puzzle routes. Their chrome is not rendered either.

### Pattern 3: Puzzle session adapter (capture without advance)

`createSceneSession.nextFrame` always runs `advance(stepCount ≥ 1)` before capturing (`session.ts:67-89`). Building needs the step-0 frame (D-07), so add a puzzle adapter:

```typescript
// web/src/physics/puzzle-session.ts (shape)
export interface GeneratedPuzzleSession extends GeneratedProofSession {
  levelInfo(): RawPuzzleLevelInfo;           // move RawPuzzleLevelInfo from scripts/puzzle-replay/wasm.ts
}
export type PuzzleLevelInfo = {               // parsed once, wasm object freed in finally
  readonly levelId: PuzzleId; readonly title: string;
  readonly targetCount: number; readonly fullCount: number;
  readonly settleSteps: number; readonly stepLimit: number;
  readonly cupInterior: WorldBounds;          // meters
  readonly buildArea: WorldBounds;            // meters = grid cells × gridMeters
};
export interface PuzzleSession extends SceneSession {
  captureFrame(): RenderFrame;                // captureFrame + parseRenderFrame + free, no advance
  readonly levelInfo: PuzzleLevelInfo;
}
export async function loadPuzzleSession(id: PuzzleId, lanes: PlacementLanes): Promise<GeneratedPuzzleSession> {
  await init({ module_or_path: wasmUrl });    // idempotent: __wbg_init returns early (generated js:819-820)
  return ProofSession.newPuzzle(id, lanes.kinds, lanes.poses);
}
```

- Store the adapter in `session.maybeSession`. Then `disposeOwnedSession`, `beginScene` and `abandonScene` dispose it with no new code, and `isWorkerSession` is false.
- Implement `applyControl`, `applyAction`, `pointerAction`, `setGravity` and `restoreAuthoredGravity` as methods that **throw** `"Puzzle sessions accept no scene input"` and never call wasm. That is fail-fast, and unit tests assert that the generated methods are never called.
- Put `loadPuzzleSession` in the new module, **not** `physics/loader.ts`. `tests/scene-lifecycle-worker.test.ts:10` mocks `../src/physics/loader` with only `loadSceneSession`. Keeping the puzzle loader out of `loader.ts` keeps that mock complete.
- Optionally re-export `RawPuzzleLevelInfo` and `GeneratedPuzzleSession` from `scripts/puzzle-replay/wasm.ts` to the new src module so the type is not duplicated.

### Pattern 4: Puzzle frame loop with exact latch, settle tail and hold (D-06, D-08, D-09)

- Write a dedicated `schedulePuzzleFrame(clock, context, deps)` modeled on `scheduleFrame` (`frame-loop.ts:105-199`).
- Store the rAF id in `clock.maybeAnimationFrameId`. Then `cancelPendingFrame`, which `beginScene`, `abandonScene` and route changes all call, cancels the puzzle loop too.
- Keep the same `accumulateStepTime`, `FRAME_STEP_BUDGET_MS` and `restoreUnrunSteps` (`clock.ts`) handling. Never raise the cap of 4.
- Step **one at a time**, as the scene loop already does (`nextFrame(1)` inside a loop), so the frame at `stepIndex === outcomeStep` is observed. The first frame whose `maybePuzzle.outcome !== "running"` has `stepIndex === outcomeStep`, because Rust latches in `on_after_step` and the capture follows that step. Its `goalCount` is the latched count. Fixtures confirm the values: won 201 / 689 and lost 720 / 0.
- Clamp the per-rAF step count to `tailEndStep − stepIndex` once latched (`tailEndStep = outcomeStep + 120`). When `stepIndex === tailEndStep`, stop scheduling and dispatch `held`. Set the view to `paused` with the held frame.
- Feed every per-step status into `reducePuzzle`, but call the Solid `setPuzzleState` and `setView` once per presented frame (D-11).
- Set `clock.maybePreviousFrame = frame` for every presented frame. Resize (`frame-loop.ts:296-299`), HUD refit and `paintHeldFrame` then repaint the puzzle frame and its overlay.
- Visibility: copy the `document.hidden` branch. `handleVisibilityChange` already resets `maybeLastTimestamp`.
- Total run for the empty fixture: 720 + 120 = 840 steps. At about 1 ms per step in wasm (`bun scripts/puzzle-replay.ts replay` replays both fixtures in 1.15 s wall time), that is about 210 rAFs at 4 steps per frame.

Building, Play, Retry and Edit:

- **Building** uses `session.captureFrame()` on the fresh session. Draw it, set `clock.maybePreviousFrame`, set view `{kind:"paused", frame: observeFrame(frame, undefined, 0)}` and reducer `loaded`.
- **Play** starts the loop on the **same** never-advanced session (discretion choice). It is identical to a fresh session because the world was never stepped, and it avoids a second build.
- **Retry** rebuilds through `beginPuzzle(id, placements, {run: true})`. **Edit** rebuilds through `beginPuzzle(id, placements, {run: false})`.
- After the first `init`, `newPuzzle` is synchronous. The only async gap is one microtask, so no loading overlay is painted. The HUD slots stay mounted either way, so the obstacle set does not change.

### Pattern 5: Glass overlay above particles in every mode (D-12)

- Layering is `canvas.particle-surface` (WebGL shaded blob) at z-index 0, under `canvas.scene-canvas` (2D) at z-index 1 (`player-viewport.css:47-65`).
- In shaded-blob mode, `drawRenderFrame` clears the 2D canvas to transparent (`canvas.ts:292-304`), then draws segments and circles. In canvas modes it fills the background, draws particles, then segments.
- So `drawGlassCup` called **after** `presentSceneFrame` inside `drawSceneFrame` is above particles in both modes.
- Every repaint path calls `session.drawSceneFrame`: the direct loop, `presentOwnedFrame`, the resize observer, `paintHeldFrame` (HUD refit, render-mode change via `appearance` → `paintHeldFrame`, particle-limit change) and the worker loop.
- The 2D context transform is already DPR-scaled to CSS pixels (`canvas.ts:99`), so project with `projectPoint(camera, …)` (`camera.ts:171`).

```typescript
// web/src/render/glass-cup.ts (shape)
export type GoalTick = { readonly fraction: number; readonly label: string };
export type GlassCup = { readonly interior: WorldBounds; readonly targetFraction: number; readonly ticks: readonly GoalTick[] };
export function glassCupGeometry(info: PuzzleLevelInfo): GlassCup { /* targetFraction = target/full; ticks [★ Win @0.6, Full @1] */ }
export function targetLineY(glass: GlassCup): number { return glass.interior.minY + glass.targetFraction * (glass.interior.maxY - glass.interior.minY); } // 1.08 m
export function drawGlassCup(context: CanvasRenderingContext2D, camera: Camera, glass: GlassCup): void {
  context.save();
  try { /* tint fillRect (alpha ~0.08), edge strokes + highlight, setLineDash for etched line, ticks */ }
  finally { context.restore(); }            // MUST: renderer never resets lineDash/globalAlpha (static-layer.ts:53 note)
}
```

### Pattern 6: Fixed HUD slots (D-14, D-18) in both layouts

- **Canvas stage (the default on desktop and phone):** `PuzzlePanel` calls `createHudObstacles(viewportFrame, canvas)` in `onMount` exactly like `PlayerPanel.tsx:146-157`, and wires it to `onHudObstaclesChange`.
  - Leaves, each with `data-hud-obstacle` and a fixed CSS `width`/`height` (or `min-width` plus `tabular-nums`): the title `h2`, the fill-meter chip, the timer `output`, the transport (Play), the result-card slot, the legend wrapper and the options trigger.
  - The scale-legend leaf must be a **wrapper that is always present** with a fixed width, like `.canvas-hud-trailing .scale-legend { width: 100px }`. `ScaleLegend` renders nothing until `maybePixelsPerMeter` is set (`ScaleLegend.tsx:21`), and zero-size elements are filtered out of the obstacle set (`hud-obstacles.ts:23`).
  - Place leaves in the top and bottom bands so the wide first-pour band (aspect about 1.26) keeps the full phone width. On a 390×844 canvas the world band is about 285 px tall, which leaves about 500 px for chrome.
- **Form-factor desktop (`?shell=form-factor`):** obstacles are not measured (`PlayerPanel.tsx:147-153`). Render the HUD in normal flow above and below the canvas, like `status-row` and `control-row`, plus a `ScaleLegend` overlay at the canvas bottom-right (`.viewport-frame > .scale-legend`, `player-viewport.css:75-80`). The canvas is `width: 100%` with `aspect-ratio: 16 / 9`, so content swaps below it cannot resize it.
- **Result-card placeholder:** in building and running, show the goal ("Fill the cup to 60% before time runs out") at the same box size, then swap to the outcome content.
- **Options drawer:** reuse the `Drawer`, `DrawerContent` and `DrawerTrigger` primitives with a puzzle trigger labeled "Puzzle options". `SceneControlsTrigger` hardcodes "Scene controls" (`CanvasStage.tsx:281-290`). The drawer holds only the render-mode `<select>` (`RENDER_MODE_GROUPS`, `maybeParseRenderMode`), `GitHubSourceLink` and `SiteFooter`. No scene controls, credits, tilt, export or debug.

### Anti-Patterns to Avoid

- **Editing `PlayerPanel.tsx`, `CanvasStage.tsx`, `hash.ts`, `portrait-bounds.ts` or `puzzles.ts`.** This risks the scene arm and the tests that must stay unedited. The source-inspection tests (`scale-legend.test.ts`, `hud-sliders-toggle.test.ts`) read `PlayerPanel.tsx` and `CanvasStage.tsx`.
- **Calling `presentOwnedFrame` for puzzles.** It advances one step, then auto-plays (or pauses under reduced motion). Building must not advance.
- **Adding a required key to `FrameLoopDeps`, `GeneratedProofSession` or `SceneSession`.** This breaks the typed fakes in `tests/worker-loop-fixture.ts:94` and `tests/session.test.ts:70`.
- **A wall-clock timer, or deciding the outcome from rendered frames in TS.** TS only displays the Rust latch.
- **Content-sized HUD chips,** or chrome that mounts and unmounts per phase. Either one refits the camera.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Win, loss and fill counting | TS counting over `particlePositions` | `RenderFrame.maybePuzzle` (Rust per-step latch) | Determinism (GOAL-04); the parse is already fail-closed |
| Camera fit around chrome | Custom layout math | `safeFrame`, `createHudObstacles`, `cameraFrame`, `publishWorldFrame` | Existing tie-break and fallback rules; e2e reads `data-world-frame` |
| World projection | Manual transform | `projectPoint`, `createCamera` | Y inversion and offsets |
| Scale bar | New legend | `ScaleLegend` / `scaleLegend` | Repo rule; snap 1/2/5 × 10ⁿ |
| Fixture parsing in e2e | New parser | `parseFixture` + `fixtureLanes`/placements from `web/scripts/puzzle-replay/fixture.ts` | Mirrors the Rust grammar one to one |
| Step accounting | New clock | `accumulateStepTime`, `restoreUnrunSteps`, `FRAME_STEP_BUDGET_MS` | 4-step cap, hidden-tab rules |
| Drawer | New modal | `components/ui/drawer.tsx` | Focus handling, mobile behavior |
| Stale async guard | Ad-hoc flags | `nextGeneration` / `isStaleGeneration` (`player/generation.ts`) plus `session.generation` | Same pattern as `beginScene` |

**Key insight:** each piece of the camera and safe-frame system has fixed rules that the existing e2e enforces (fixed leaf size, sub-pixel equality, fallback ratio). Reusing them unchanged is what makes "no camera jump" provable.

## Runtime State Inventory

Not a rename or migration phase, but the route and state handoff between scene and puzzle has runtime state that must be reset:

| Category | Items Found | Action Required |
| --- | --- | --- |
| Stored data | `localStorage` keys `liquidfun.render-mode` (via `appearance`) and the HUD-sliders preference. Puzzles read the render mode; nothing new is stored in Phase 38 | None |
| Live in-memory state carried across a route change | `tiltBinding` (live listener), `panEnabled`, `clock.cameraView` (zoom/pan), `constructionValues`, `maybeCanvasPointer`, `clock.hudObstacles`, `maybePuzzleGlass` (new) | `beginPuzzle`: stop tilt (`changeTiltGravity(binding, false, () => undefined, …)`), `setPanEnabled(false)`, `cameraView = IDENTITY_CAMERA_VIEW`, `constructionValues = {}`. `assignPuzzleCanvas`: detach the old pointer. `leavePuzzle`: clear the glass and puzzle state before the scene arm runs. |
| OS-registered state | None, verified: browser-only change | None |
| Secrets and env vars | None | None |
| Build artifacts | `web/src/generated/liquidfun-wasm` (gitignored; must exist for typecheck, Vitest and build). `web/dist` (served by e2e `vite preview`). A **stale preview server** on port 4173 is reused locally (`reuseExistingServer: !CI`) | Rebuild `dist` before e2e. Run Playwright with `CI=true` or make sure no stale server is running. |

## Common Pitfalls

### Pitfall 1: The route effect wipes the puzzle canvas

**What goes wrong:** On scene → puzzle, `setRoute` synchronously mounts `PuzzlePanel`, whose canvas `ref` assigns `maybeCanvas`. Then the `bindNavigation` effect sees a non-scene route and calls `abandonScene()`, which clears the canvas and context and disconnects the resize observer. The puzzle never starts.
**How to avoid:** Make the effect target-based (`maybePlayableTarget`).
**Warning signs:** The puzzle sticks at "loading" after navigating from a scene, even though a direct load works.

### Pitfall 2: Building advances the world

**What goes wrong:** Calling `nextFrame()` or `presentOwnedFrame` shows step 1, not step 0, and auto-plays.
**How to avoid:** Use the adapter's `captureFrame()`. Unit test: building `stepIndex === 0` and `advance` not called.

### Pitfall 3: Latched values taken from a batch boundary

**What goes wrong:** If several steps run before a single capture, the card shows the count at the batch end instead of at `outcomeStep`.
**How to avoid:** Step and capture one step at a time (the scene loop already does). Record `latchedCount` from the first latched frame. Assert `frame.stepIndex === status.maybeOutcomeStep` at the latch, and fail closed otherwise.

### Pitfall 4: Overshooting the settle tail, or never holding

**How to avoid:** Clamp the steps per rAF to `tailEndStep − stepIndex`. Stop scheduling at the tail end. Unit-test with a fake session that latches at a known step.

### Pitfall 5: Canvas state leaks into later frames

**What goes wrong:** `setLineDash`, `globalAlpha` or `lineWidth` set by the glass overlay persist. The next frame's segments and circles then draw dashed or faded, because `drawRenderFrame` never resets them and `static-layer.ts` strokes cached paths with the current dash state.
**How to avoid:** `save()`/`restore()` in `try/finally`. Unit-test with a recording fake context that checks `save` and `restore` are balanced and `setLineDash([])` is restored.

### Pitfall 6: Glass overlay leaks onto scenes

**How to avoid:** `leavePuzzle` clears `maybePuzzleGlass`, and `drawSceneFrame` also checks `session.route().kind === "puzzle"`. e2e: open the puzzle, then the dam-break sidebar link, and expect Playing with a pointer accepted.

### Pitfall 7: Tilt or pointer reach the puzzle session

**What goes wrong:** Live tilt calls `setGravity`. Pointer forwarding calls `pointerAction`, which Rust rejects with `UnknownControl`, and that leads to `failScene`.
**How to avoid:** Stop tilt in `beginPuzzle` before creating the session. `assignPuzzleCanvas` never attaches `attachCanvasPointer`. The adapter methods throw, which a unit test asserts. Also neutralize the `canvas-interactive` crosshair cursor with puzzle CSS, or guard `syncCanvasInteractive`.

### Pitfall 8: HUD obstacle set or box size changes between phases

**What goes wrong:** `createHudObstacles` re-measures on every DOM mutation (`hud-obstacles.ts:63-66`). A box that grows (for example a wrapping result text, or the timer going from "0:12.0" to a different width) or a leaf that appears late (`ScaleLegend` before the scale is published) refits the camera.
**How to avoid:** Fixed CSS sizes on every leaf, `font-variant-numeric: tabular-nums`, an always-present legend wrapper, and the same placeholder size in building and running.
**Warning signs:** The D-19 e2e frame differs, or `data-world-frame-fallback="true"` appears at 390×844.

### Pitfall 9: Typed-array coercion in the test seam (Phase 37 WR-01)

**What goes wrong:** `Int32Array.from([8.7, NaN])` silently produces `[8, 0]`.
**How to avoid:** `encodePlacementLanes` checks safe integers, the i32 range and known kind codes, and throws otherwise. If the seam global is present but invalid, the puzzle fails visibly (failure view). Phase 39 reuses the same encoder.

### Pitfall 10: The test seam becomes reachable from the URL

**How to avoid:** Read only `Reflect.get(window, "__liquidfunPuzzleTestPlacements")` in `beginPuzzle`. Never parse `location`, `hash`, `search` or storage for placements. Add a grep check in the gate. The global ships in the production bundle, as `__liquidfunSyntheticAnimationClock` already does, because e2e tests the built `dist`.

### Pitfall 11: Loss with a fill at or above the target

**What goes wrong:** D-10 says rounding down prevents a loss from showing a fill equal to the target. That holds only when the latched count is below the target. A run can reach the step limit with count ≥ target but fewer than 30 held steps, which gives "Out of time: 61% of 60%".
**How to avoid:** Unit-test this edge case and keep the honest numbers. The copy is still truthful. Flag it for UX review in Phase 39 or 40.

### Pitfall 12: Optional-property typing

**What goes wrong:** `exactOptionalPropertyTypes` rejects `{maybeX: undefined}` for `maybeX?: T`.
**How to avoid:** Use the conditional spread pattern (`frame.ts`), or declare `T | undefined`. `noUncheckedIndexedAccess` means typed-array reads in `parsePuzzleLevelInfo` need guards.

## Code Examples

### Formatting (pure; D-09, D-10)

```typescript
// web/src/puzzle/format.ts
/** Remaining simulation time as m:ss.t, floored to tenths; integer math only. */
export function formatSimulationTime(remainingSteps: number): string {
  const tenths = Math.floor(Math.max(0, remainingSteps) / 6); // 60 steps/s → 6 steps per tenth
  const minutes = Math.floor(tenths / 600);
  const seconds = Math.floor((tenths % 600) / 10);
  return `${minutes}:${String(seconds).padStart(2, "0")}.${tenths % 10}`;
}
// 720 → "0:12.0"; 519 (win at 201) → "0:08.6"; 0 → "0:00.0"
export function floorPercent(count: number, full: number): number {
  return Math.min(100, Math.floor((count * 100) / full));
}
// win 689/900 → 76, target 540/900 → 60 → "Filled! 76% of 60%"; empty → "Out of time: 0% of 60%"
```

### Reading the frame and scale in e2e (D-19)

```typescript
// web/e2e/puzzle-helpers.ts (copy the readHudGeometry / stableWorldFrame pattern from hud-safe-frame.spec.ts:49-147; do not import or edit that spec)
await installControlledRefreshRate(page, 15);                 // 66.7 ms per rAF → 4 steps per frame, not wall-clock bound
await page.setViewportSize({ width: 390, height: 844 });
await page.goto("/liquidfun-rs/#/puzzle/first-pour");
await expect(page.locator('main[data-puzzle-phase="building"]')).toBeVisible();
const building = { frame: await stableWorldFrame(page), ppm: await page.locator(".puzzle-panel").getAttribute("data-pixels-per-meter") };
await page.getByRole("button", { name: "Play" }).click();
await expect.poll(() => numericAttribute(page.locator("main"), "data-step-index")).toBeGreaterThan(0);
// ... running read, then await main[data-puzzle-phase="ended"] (and data-puzzle-held="true") → read again → expect equal
```

Publish `data-pixels-per-meter={props.maybePixelsPerMeter}` on the puzzle panel root. The scale is not exposed in the DOM today, only the snapped legend label and bar width.

### Seam injection in e2e (D-22)

```typescript
import { readFileSync } from "node:fs";
import { parseFixture } from "../scripts/puzzle-replay/fixture";
const fixture = parseFixture(readFileSync(resolve(repoRoot, "crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt"), "utf8"));
await page.addInitScript((value) => {
  Object.defineProperty(window, "__liquidfunPuzzleTestPlacements", { value, configurable: true });
}, { level: fixture.level, placements: fixture.placements.map((p) => ({ kind: p.kind, x: p.x, y: p.y, angle: p.angle })) });
// plain arrays/objects only (no typed arrays across the Playwright boundary)
// expect text /Filled! \d+% of \d+%/ (exactly "Filled! 76% of 60%"), Next disabled with name "No next level yet", Retry, Edit
```

## first-pour Geometry (verified in the generated wasm, 2026-10-11)

| Item | Value | Source |
| --- | --- | --- |
| levelInfo | title "First Pour", target 540, full 900, settle 30, stepLimit 720, gridMeters 0.05, toolKinds [1], toolCounts [1] | Bun probe of `ProofSession.newPuzzle("first-pour", [], [])` |
| cupInterior | `[1.6, 0, 3.4, 1.8]` m (f32: 1.600000023841858, 1.7999999523162842) | `levelInfo().cupInterior()` |
| Build area (placement centres) | grid `[-20, 44, 40, 84]` → x −1.0..2.0 m, y 2.2..4.2 m | `levelInfo().buildArea()` |
| Plank footprint (Phase 39 relevance) | 3.0 m × 0.25 m; any angle reaches x −2.5..3.5, y 0.7..5.7 | `tools.rs:20-23` |
| Rigid segments extent (all walls) | x −6..6, y −0.5..9 | step-0 frame `rigidSegments` |
| Cup walls | left x 1.4..1.6, y 0..1.8; backstop x 3.4..3.6, y 0..3.4 | `first_pour.rs:38` |
| Reservoir walls / liquid spawn | x −3.2..−0.4, y 4.6..8.2 / particles x −2.94..−0.66, y 4.86..7.86 (1989 particles) | frame and spec |
| Empty run at step 720 | water spreads to x −5.79 (the left outer wall), max x 1.38; outcome lost @720, count 0 | Bun probe |
| Target line height | 0 + 0.6 × 1.8 = **1.08 m** | D-12 formula |

### Recommended bounds (discretion, within D-20)

- **Landscape:** `{minX: -6.2, minY: -0.7, maxX: 6.2, maxY: 9.2}`. It contains every wall, the spawn, the cup and the build area, with a 0.2 m stroke margin.
- **Portrait:** `{minX: -6.1, minY: -0.6, maxX: 6.1, maxY: 9.1}`. Fit at 390×844 with the 16 px inset and no obstacles: scale 29.3 px/m, width fraction **0.918**, height fraction **0.337**. That matches the `TALL_BASIN` scenes, which use a 0.3 minimum.
  - Suggested test thresholds: width > 0.85 (the default rule), height > 0.33.
  - A tighter action frame (x −6.1..3.8, y −0.6..8.4) reaches height 0.386, but it clips the right outer wall and the wall tops, which breaks the D-20 "walls" clause. The left wall cannot be cropped because the empty-run water reaches it.
  - See Open Question 1.

`tests/puzzle-bounds.test.ts` mirrors `fittedFrameFraction` (`portrait-bounds.test.ts:84-101`) for every `PuzzleId` and asserts containment of the hardcoded cup interior `[1.6, 0, 3.4, 1.8]`, the build area `[-1, 2.2, 2, 4.2]`, the walls `[-6, -0.5, 6, 9]` and the spawn `[-2.96, 4.84, -0.64, 7.9]`. The landscape-vs-portrait selection rule mirrors `worldBoundsForViewport` (`portrait-bounds.ts:98-108`).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| "Desktop" meant the scrolling page | Canvas-stage shell is the default at every size; `?shell=form-factor` restores the old page | Before v1.5 (`screen-shell.ts`) | D-19 desktop runs through obstacle measurement |
| Rust frame had no puzzle data | Optional `maybePuzzle` scalars, fail-closed | Phase 37-02/07 | The HUD reads Rust only |
| `GeneratedProofSession` covered all sessions | Puzzle sessions add `levelInfo` | Phase 37-07 | Use an `extends` interface |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | Solid 1.9 runs JSX render work (refs) before user `createEffect`s within one synchronous `setRoute` update, so the route effect can wipe the new canvas | Pitfall 1 | Low: the recommended target-based effect is correct under either ordering [ASSUMED: Solid internals from training] |
| A2 | Chromium wasm step cost for first-pour is about 1 ms per step, like Bun/JSC, so a full e2e run takes about 3-5 s under 15 Hz controlled refresh | Pattern 4, Validation | Medium: a slower step only lengthens the test, and the 30 s timeout still has margin [ASSUMED from the Bun timing] |
| A3 | Playwright `addInitScript` serializes plain objects with arrays | Code Examples | Low: documented behavior [ASSUMED] |
| A4 | Interpreting D-17 "hide the SceneControlsSheet drawer" as "hide the scene drawer content" is acceptable, while keeping a puzzle options drawer for render mode, source and provenance | Pattern 6 | Medium: needs owner or UI review confirmation |
| A5 | The first-pour portrait height fraction of about 0.34 is acceptable for "fills the phone" in Phase 38 | Bounds | Medium: LEVEL-04 (Phase 40) may want re-authored geometry, which is a Rust change behind the fingerprint gate |

## Open Questions (RESOLVED)

1. **Portrait frame strictness vs. phone fill**
   - What we know: the authored level is 12 m wide, the walls reach y = 9, and empty-run water reaches the left wall.
   - What's unclear: whether the owner prefers strict wall containment (about 34% height fill) or a tighter action frame (about 39%) that crops the unreachable right wall.
   - RESOLVED: use the strict frame, which follows D-20 literally. Revisit when Phase 40 tunes levels for LEVEL-04.
1. **Puzzle options drawer vs. D-17**
   - What we know: canvas-stage provenance and source live only in the drawer, which the Bright Builds rule requires.
   - RESOLVED: keep a minimal "Puzzle options" drawer (render mode, source link, footer). Treat D-17 as excluding the scene drawer *content*. Confirm in the UI spec.
1. **Level title before wasm loads**
   - What we know: the title is authored in Rust (`levelInfo().title`). `puzzles.ts` is parsed by a Rust mirror test and holds ids only.
   - RESOLVED: keep a TS `PUZZLE_TITLES: Record<PuzzleId, string>` in a new `catalog/puzzle-levels.ts`, so the synchronous route title and the loading heading follow the existing pattern. Assert `levelInfo.title === PUZZLE_TITLES[id]` at load (fail closed) and in e2e.
1. **Add `e2e/puzzle.spec.ts` to `test:player`?**
   - RESOLVED: no. Keep `test:player` (and therefore `just web-player-smoke`) as the unchanged scene gate. Add a `test:puzzle` script and run it as its own gate.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Bun | scripts, Vitest, build | ✓ | 1.4.2 | — |
| Node | Playwright runner | ✓ | 24.13.0 | — |
| wasm-pack + rustc | generated package | ✓ | 0.15.0 / 1.97.0 | — |
| Playwright Chromium | e2e | ✓ | 1.63.0 (browsers cached) | `bun run browser:install` |
| jq | fingerprint baseline extraction | ✓ | 1.7.1 | — |
| Python 3.13 + mdformat | only if non-GSD Markdown is touched | ✓ | 3.13.12 | — |

**Missing dependencies:** none.

**Fingerprint baseline:** the Phase 37 stamp `target/dam-break-perf/2026-10-12T00-58-37Z` (git `410e2111b`) is valid for `589cb0fb9`, because `git diff --stat 410e2111b 589cb0fb9 -- crates tools Cargo.toml Cargo.lock` is empty (verified). `target/phase37/after-final.txt` (25 lines) can be copied as the baseline, with that evidence recorded. Rerunning `just playground-scene-spot` is cheap (about 25 s with the release binary cached). Extract with `jq -r '.scenes[] | "\(.scene) \(.fingerprint)"' <stamp>/scene-spot.json | sort`.

## Validation Architecture

(Included at the orchestrator's request. `workflow.nyquist_validation` is `false` in config.)

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Vitest 5.0.1 (node env) + Playwright 1.63.0 (Chromium, `vite preview` of `dist`) |
| Config file | `web/vitest.config.ts`, `web/playwright.config.ts` |
| Quick run command | `cd web && bunx vitest run tests/<file>.test.ts` |
| Full suite command | `cd web && bun run typecheck && bun run test:unit` (~550 tests, seconds) ; `cd web && bun run build:app && CI=true bunx playwright test e2e/puzzle.spec.ts` |
| Scene regression gates | `just web-player-smoke` (44 tests, about 40 s plus build) ; `cd web && CI=true bunx playwright test e2e/hud-safe-frame.spec.ts` ; `just playground-scene-spot` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| PUZ-02 | parse `#/puzzle/first-pour`, unknown ids, extra segments, `#/puzzle`, `#/puzzles` → scene unknown, full URL → unknown | unit | `bunx vitest run tests/playable-route.test.ts` | ❌ Wave 1 |
| PUZ-02 | fallback shows "Level not found", raw id as text, both links | e2e | `bunx playwright test e2e/puzzle.spec.ts -g "unknown level"` | ❌ |
| GOAL-03 | `formatSimulationTime`, `floorPercent`, loss/win copy, latch values, edge case count ≥ target on loss | unit | `bunx vitest run tests/puzzle-format.test.ts` | ❌ Wave 1 |
| GOAL-03 / D-06 / D-08 | reducer transitions: loaded, play, latch, tail, held, retry, edit; timer frozen in building and ended | unit | `bunx vitest run tests/reduce-puzzle.test.ts` | ❌ Wave 1 |
| D-08 | loop steps one at a time, clamps to the tail, holds, never exceeds 4 per frame | unit (stubbed rAF like `worker-loop-fixture.ts`) | `bunx vitest run tests/puzzle-loop.test.ts` | ❌ Wave 2 |
| D-07 | building frame at step 0 (no advance); scene inputs throw without calling wasm; levelInfo parsed and freed | unit | `bunx vitest run tests/puzzle-session.test.ts` | ❌ Wave 1 |
| D-22 / WR-01 | encoder rejects non-integers, out-of-range values and unknown kinds; seam absent → empty; invalid seam → error | unit | `bunx vitest run tests/puzzle-placements.test.ts` | ❌ Wave 1 |
| GOAL-03 | natural run shows `Out of time: \d+% of \d+%` (exactly 0% of 60%) | e2e | `bunx playwright test e2e/puzzle.spec.ts -g "loses"` | ❌ |
| GOAL-01 / D-22 | seam solution run shows `Filled! 76% of 60%`, Next disabled with name "No next level yet", Retry, Edit | e2e | `… -g "wins"` | ❌ |
| PUZ-03 / D-19 | frame and pixels-per-meter equal in building, running and ended at 1280×800 and 390×844 | e2e | `… -g "camera"` | ❌ |
| PUZ-03 | no zoom, pan, accelerometer, gravity slider, "Scene controls" or "Pause scene"; canvas drag leaves `data-pointer-accepted` at 0 with no failure | e2e | `… -g "interactions"` | ❌ |
| PUZ-04 | every puzzle HUD leaf has `data-hud-obstacle`, ScaleLegend in both layouts, no tilt/ViewportTools/DebugReadout imports | unit (source inspection) | `bunx vitest run tests/puzzle-chrome.test.ts` | ❌ Wave 2 |
| PUZ-04 | portrait and landscape frames fill and contain the cup, build area, walls and spawn | unit | `bunx vitest run tests/puzzle-bounds.test.ts` | ❌ Wave 1 |
| PUZ-05 | glass geometry (target line 1.08 m, ticks), save/restore balanced, dash reset | unit (recording fake context) | `bunx vitest run tests/glass-cup.test.ts` | ❌ Wave 1 |
| PUZ-06 | 25/25 fingerprints; listed tests unedited; smoke and HUD e2e pass; puzzle → scene works with no overlay or tilt leak | gate + e2e | `just playground-scene-spot` ; `git diff --quiet 589cb0fb9 -- web/tests/hash.test.ts web/tests/portrait-bounds.test.ts web/e2e/hud-safe-frame.spec.ts web/tests/readme-svg.test.ts` ; `just web-player-smoke` | partial (existing) |

### Sampling Rate

- **Per task commit:** the touched Vitest file(s), `cd web && bun run typecheck`, `bun scripts/bright-builds-check.ts all`.
- **Per wave merge:** `cd web && bun run test:unit`. Starting with the UI wave, also build and run `e2e/puzzle.spec.ts`.
- **Phase gate:** everything in D-23, plus `e2e/hud-safe-frame.spec.ts`, the `e2e/puzzle.spec.ts` suite, and the grep checks: no `performance.now`/`Date.now`/`setTimeout` in `src/puzzle/**` or in the timer and outcome paths, no `location` reads near the seam, and no `first-pour` in `scenes.ts`, `scene-records.ts`, `portrait-bounds.ts`, `readme-svg/plans.ts`, `scene.rs`, `scene_spot.rs` or xtask.

### Wave 0 Gaps

None for the framework. All test files listed above are new and belong to their plans. Prerequisite: `bun scripts/web-build.ts wasm` has run.

## Security Domain

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Allowlisted route ids (`PUZZLE_IDS`). The raw id is rendered only through Solid text bindings and truncated for display (for example to 80 chars). The seam is strictly validated (`encodePlacementLanes`). Rust validates placements again. |
| V6 Cryptography | no | — |

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| HTML injection through `#/puzzle/<raw>` | Tampering | Text binding only, never `innerHTML`; allowlist parse |
| Test seam abused by URL | Elevation | The seam reads only a JS global set by `addInitScript`, never URL or storage; a grep gate checks this |
| Non-finite or coerced placement values reaching wasm | Tampering | Integer and range checks before `Int32Array` (WR-01); Rust `Placements::parse` fails closed |

## Recommended Plan Breakdown (waves, minimal file overlap)

| Wave | Plan | Files (exclusive) | Depends on |
| --- | --- | --- | --- |
| 1 | **38-01 Routing and targets** | `routing/playable-route.ts`, `player/playable-target.ts`, `catalog/puzzle-levels.ts` (titles), tests | — |
| 1 | **38-02 Puzzle pure core** | `puzzle/reduce-puzzle.ts`, `puzzle/format.ts`, `puzzle/placements.ts`, `puzzle/controller.ts` (the `PuzzleController` / props contract type used by 38-06), tests | — |
| 1 | **38-03 Bounds and glass** | `catalog/puzzle-bounds.ts`, `render/glass-cup.ts`, `tests/puzzle-bounds.test.ts`, `tests/glass-cup.test.ts`; one-time wasm cross-check recorded in the SUMMARY | — |
| 1 | **38-04 Puzzle session adapter** | `physics/puzzle-session.ts`, optional type re-export in `scripts/puzzle-replay/wasm.ts`, `tests/puzzle-session.test.ts` | — |
| 2 | **38-05 Lifecycle and loop wiring** | MOD `scene-runtime.ts`, `runtime.ts`, `gravity-slider.ts`, `playground-session.ts`, `scene-lifecycle.ts`, `frame-loop.ts`, `scene-surface.ts`; NEW `player/puzzle-lifecycle.ts`, `player/puzzle-loop.ts`, `tests/puzzle-loop.test.ts` | 38-01..04 |
| 2 | **38-06 Puzzle UI** | NEW `PuzzlePanel.tsx`, `PuzzleHud.tsx`, `PuzzleResultCard.tsx`, `PuzzleFallbackPanel.tsx`, `styles/puzzle.css`, `tests/puzzle-chrome.test.ts`; MOD `PlaygroundStage.tsx`, `App.tsx` | 38-01, 38-02 (contract); integrates with 38-05 via the `SceneRuntime` fields named in the contract. Run after 38-05 if the contract is not fixed in 38-02. |
| 3 | **38-07 Puzzle e2e** | `e2e/puzzle.spec.ts`, `e2e/puzzle-helpers.ts`, `package.json` `test:puzzle` | 38-05, 38-06 |
| 4 | **38-08 Gates and evidence** | `target/phase38/*` only: baseline, scene-spot 25/25, crates diff, unchanged-tests diff, smoke, HUD e2e, Rust fmt/clippy/build/test, tsc, Vitest, bright-builds, grep gates | all |

The plans share no files within a wave. 38-05 and 38-06 overlap only through the type contract, so only one of them edits each shared file. `PlayerPanel.tsx`, `CanvasStage.tsx`, `hash.ts`, `portrait-bounds.ts`, `puzzles.ts`, `canvas.ts` and `scene-records.ts` are never touched.

## Sources

### Primary (HIGH confidence, read in this session)

- `web/src/{routing/hash.ts, player/{runtime,scene-runtime,scene-lifecycle,playground-session,frame-loop,scene-surface,camera-frame,hud-obstacles,view,observe,fullscreen-capability,screen-shell}.ts, physics/{session,loader,live-session,clock,frame}.ts, input/{canvas-pointer,tilt-binding,gravity-slider}.ts, render/{camera,canvas,present-frame,safe-frame,static-layer}.ts, catalog/{puzzles,portrait-bounds}.ts, components/{PlaygroundStage,PlayerPanel,CanvasStage,FallbackPanel,PlaygroundShell,ScaleLegend,SiteHeader,SiteFooter}.tsx}`
- `web/src/styles/{canvas-stage,player-viewport}.css`, `web/{package.json,tsconfig.json,vitest.config.ts,playwright.config.ts,vite.config.ts}`
- `web/e2e/{hud-safe-frame.spec.ts,player-helpers.ts,demo-media-clock.spec.ts}`, `web/scripts/demo-media/capture.ts`, `web/scripts/puzzle-replay/{fixture,wasm}.ts`
- `web/tests/{portrait-bounds,hash,session,scene-surface,scene-lifecycle-worker,worker-loop-fixture,scale-legend,hud-sliders-toggle}.test.ts|.ts`
- `crates/liquidfun-wasm/src/{lib.rs, frame.rs, puzzle/{first_pour,info,tools,tests}.rs, puzzle/fixtures/*.txt}`; generated `liquidfun_wasm.{d.ts,js}`
- `justfile`, `scripts/web-build.ts`, `scripts/bright-builds-check.ts --help`, `.github/workflows/pages.yml`
- Bun probe of the generated wasm (levelInfo, step-0 frame extents, empty run at step 720); `bun scripts/puzzle-replay.ts replay` timing (1.15 s)
- `.planning/phases/37-*/{37-CONTEXT,37-02-SUMMARY,37-VERIFICATION}.md`, `.planning/research/v1.5-puzzle-mode/{SUMMARY,ARCHITECTURE,PITFALLS,FEATURES}.md`

### Tertiary (LOW, training knowledge)

- Solid 1.9 update and effect ordering (A1); Playwright init-script argument serialization (A3)

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH. No additions; versions read from `package.json` and the tool CLIs.
- Architecture: HIGH. Every dispatch point was located by line, and the typed-fake constraints were checked.
- Pitfalls: HIGH for code-grounded items (effect wipe, capture without advance, dash leak, tilt, typed fakes, stale preview server). MEDIUM for UX copy edge cases.
- Bounds: HIGH for the geometry (wasm-verified). MEDIUM for the strict-vs-tight choice, which is the owner's taste.

**Research date:** 2026-10-11
**Valid until:** 2026-11-10 (stable internal codebase; re-check if Phase 39 work lands first)
