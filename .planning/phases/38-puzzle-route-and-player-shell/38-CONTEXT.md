---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 38-2026-10-12T01-22-44
generated_at: 2026-10-12T01:26:53.041Z
---

# Phase 38: Puzzle Route and Player Shell - Context

**Gathered:** 2026-10-12
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Open a puzzle level at `#/puzzle/<id>` inside the existing SolidJS player. The level runs on the direct (main-thread) backend through the Phase 37 `ProofSession.newPuzzle` bridge. The player sees:

- the cup drawn as glass, with an etched target line and star ticks
- a fill meter, a simulation-step timer and a result card, each in a fixed HUD slot

Scene-only interactions and chrome are off on puzzle routes. An unknown level id shows a helpful fallback. The 25 scenes behave exactly as before.

Out of scope here:

- tool placement, the tool tray, Stop/Reset/Clear and stars (Phase 39)
- the timed valve and further levels (Phase 40)
- `#/puzzles`, level select, unlocks and progress (Phase 41)
- sound (Phase 42)
- GPU water (Phase 43)

</domain>

<decisions>
## Implementation Decisions

### Routing and lifecycle
- **D-01:** Leave `maybeParseSceneRoute` and `normalizeSceneRoute` in `web/src/routing/hash.ts` exactly as they are. Existing hash tests must pass without edits, and they assert that other paths are `unknown`. Add a separate wrapper parser, for example `maybeParsePlayableRoute`, that first recognizes `#/puzzle/<id>` and otherwise delegates to the scene parser. It returns a `PlayableRoute` union:
  - `{kind:"puzzle", id: PuzzleId}`
  - `{kind:"unknown-puzzle", maybeRaw}`
  - the existing scene route kinds

  Puzzle ids are validated against `PUZZLE_IDS` from `web/src/catalog/puzzles.ts`. Any extra segment is unknown, matching the scene rule.
- **D-02:** Replace `maybeReadySceneId` with a `PlayableTarget` union, `{kind:"scene", id} | {kind:"puzzle", id}`. The scene arm runs today's code path unchanged: the same `beginScene`, resize, controls, gravity, titles and worker choice. Puzzle code mounts only on the puzzle arm. Keep scene behavior unchanged by keeping the scene arm literally the old call path, not by re-implementing it.
- **D-03:** `#/puzzles` is not added in this phase and stays an unknown route (Phase 41). Levels do not appear in `SCENE_IDS`, `SCENES`, the scene sidebar (`DemoNavigation`), README plans, `PORTRAIT_VIEW_BOUNDS` or the scene survey.
- **D-04:** Puzzles always use the direct backend (`new ProofSession` path), never the worker. The worker path stays scene-shaped. Each run builds a fresh session with `ProofSession.newPuzzle(levelId, kinds, poses)`. In Phase 38 the placements are empty by default, but the lifecycle takes placements as an input so Phase 39 can supply them.
- **D-05:** The unknown-puzzle fallback reuses the `FallbackPanel` pattern with puzzle wording. It shows "Level not found", the raw id (text only, never HTML), a primary link to the first level (`#/puzzle/first-pour`) and a secondary link back to the scenes (`#/scene/wave-machine`). Page title and `data-*` route attributes follow the existing route-identity pattern.

### Puzzle phase machine (minimal for this phase)
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

### Timer
- **D-09:** The timer counts down the remaining simulation time, `(stepLimit − step) / 60`, shown as `0:12.3` with `tabular-nums`. It is always visible on puzzle routes:
  - In building, it shows the full limit, frozen.
  - In running, it counts down from the frame's step index.
  - In ended, it freezes at the outcome step.

  It reads only the frame step index and `stepLimit`, never wall-clock time. Its accessible name says it is simulation time, for example `aria-label="Simulation time left"`.
- **D-10:** The loss text follows the requirement: "Out of time: 54% of 80%". Both percentages are rounded down to whole percent of `full_count`. The fill is `goalCount / goalFull` at the latch, and the target is `goalTarget / goalFull`. Rounding down keeps a loss from ever displaying a fill equal to the target. Put this formatting in pure, unit-tested functions.

### Fill meter (HUD) and glass cup (canvas)
- **D-11:** The **HUD fill meter** is the authoritative readout. It is a fixed-slot chip with a short bar, a percentage (`54%`), a target marker and star ticks, and it marks the target as `/ 80%`. It updates from `RenderFrame.maybePuzzle` each presented frame.
- **D-12:** The **glass cup** is drawn on the canvas in world space from `levelInfo().cupInterior`. It has:
  - a faint translucent tint over the interior
  - brighter thin edge strokes with a highlight streak, so the existing physics cup walls read as glass
  - an etched (dashed or engraved-look) horizontal target line at the interior height that matches `targetCount / fullCount`
  - short star ticks on the cup wall

  Map the line height linearly from the interior bottom, as `fraction × interior height`. This is a visual aid, not a physics claim. Draw no fill-percentage text on the canvas. The overlay must stay visible in both the canvas and the shaded-blob (WebGL) render modes, so it draws above the particle layer. It is also repainted on held-frame repaints.
- **D-13:** In this phase the star ticks are the ★ Win target and a brim tick at 100% of `full_count`. The tick list is data-driven (`{fraction, label}[]`), so Phase 39 can add the ★ Brim threshold from Rust without restructuring the overlay or the meter.

### Result card
- **D-14:** The result card is a fixed HUD slot. The slot exists for the whole puzzle route, so the obstacle set never changes. Content swaps inside it:
  - In building and running, the slot is empty or a compact placeholder of the same box size.
  - In ended, it shows the outcome.
  - A win reads "Filled! 86% of 80%".
  - A loss reads "Out of time: 54% of 80%".

  Both outcomes show the actions Next, Retry and Edit.
- **D-15:** Next goes to the next entry in `PUZZLE_IDS`. When there is none (Phase 38 has only `first-pour`), Next is shown disabled with the accessible text "No next level yet", so the layout stays fixed. Retry and Edit behave as in D-06. Buttons use touch targets of at least 44 px.

### Interactions, chrome and camera
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

### Puzzle bounds registry
- **D-20:** Add `web/src/catalog/puzzle-bounds.ts`, separate from `portrait-bounds.ts`. Each `PuzzleId` gets a landscape world rectangle and a portrait world rectangle. `worldBoundsForTarget(target, viewport)` dispatches: scenes call the existing `worldBoundsForViewport` unchanged, and puzzles use the new registry by orientation. Both puzzle rectangles must contain the cup, the walls, the build area and the liquid spawn. The portrait rectangle is sized so the action fills an iPhone canvas (about 390 by 844 CSS pixels) with a small margin, following the repo's portrait framing guidance. Do not pad for HUD chrome; the safe frame handles that.
- **D-21:** Add a separate Vitest test, for example `web/tests/puzzle-bounds.test.ts`, keyed by every `PuzzleId`. It checks that each portrait frame fills the phone under the same rule as `portrait-bounds.test.ts`. It also checks that each rectangle contains the cup interior and the build area. Hardcode the level geometry the test compares against from the Rust `first-pour` spec, cross-checked once in the plan by reading `levelInfo()` from the wasm package. A Bun or Vitest check against the generated wasm is acceptable if cheap. The existing `portrait-bounds.test.ts` stays unedited.

### Test seam for the win path
- **D-22:** Phase 38 has no tool UI, so a natural run of `first-pour` always loses. The win path of the result card is covered two ways:
  - Vitest component and formatter tests with fake `PuzzleFrameStatus` values for won and lost.
  - One e2e that injects the recorded reference placements from `crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt` through a test-only seam, then sees the win card with Next, Retry and Edit.

  The seam is a window global read once at puzzle start, in the style of `__liquidfunSyntheticAnimationClock`. It must not be reachable from the URL. Use the synthetic clock or controlled refresh helpers where available so the run is not wall-clock bound. The natural empty-placement e2e asserts the loss card text pattern `Out of time: N% of M%`.

### Gates
- **D-23:** The phase is done only if all of these hold:
  - `just playground-scene-spot` reports 25/25 fingerprints identical to the pre-phase baseline (record the baseline from phase-base HEAD `589cb0fb9`).
  - `git diff --stat 589cb0fb9 -- crates/liquidfun` is empty.
  - `just web-player-smoke` passes.
  - These existing tests pass without edits: `tests/hash.test.ts`, `tests/portrait-bounds.test.ts`, `e2e/hud-safe-frame.spec.ts` and `tests/readme-svg.test.ts` (README plan coverage).
  - Rust fmt, clippy (`-D warnings`), build and tests pass, plus the touched Vitest suites, `npx tsc --noEmit` for web, and `bun scripts/bright-builds-check.ts all`, which enforces file-length limits. Watch the limits for `scene-records.ts`, `PlayerPanel.tsx` and `PlaygroundStage.tsx`, and add new files rather than grow those.

  Phase 38 should not need any Rust change. If a bridge change becomes necessary, it requires the fingerprint gate.

### Post-research clarifications (yolo, from 38-RESEARCH.md Open Questions)
- **D-24:** In canvas-stage mode, the GitHub source link and the build/version footer live only inside the scene controls drawer, which D-16 hides. Puzzle routes therefore get a small "Puzzle options" drawer. It holds only:
  - the render-mode option
  - the source link
  - the build/version footer

  Its toggle is a fixed HUD obstacle leaf, mounted for the whole puzzle route like the other slots. This preserves the repo's source and version visibility rule. It does not reintroduce any scene control.
- **D-25:** The portrait frame is strict. The `first-pour` portrait rectangle contains every wall: about `{minX:-6.1, minY:-0.6, maxX:6.1, maxY:9.1}`, which fills about 92% of the width and 34% of the height at 390×844. This matches the precedent of the existing tall-basin scenes, which use a 0.3 minimum height fraction. Do not crop the unreachable right wall. Re-authoring the geometry for a tighter phone fill is a Phase 40 (LEVEL-04) concern behind the fingerprint gate.
- **D-26:** A small TypeScript title table, keyed by `PuzzleId`, supplies the level title before wasm loads. Once the session loads, it is checked against `levelInfo().title`. A mismatch fails closed: the puzzle shows its failure overlay instead of silently showing the wrong title. The table must not be added to `web/src/catalog/puzzles.ts` if that would break its Rust `include_str!` allowlist test. A separate file is fine.

### Claude's Discretion
- Exact component and file split: for example `PuzzleHud.tsx`, `PuzzleResultCard.tsx`, `puzzle-lifecycle.ts`, `reduce-puzzle.ts`, `render/glass-cup.ts`.
- Exact HUD slot placement per layout, within D-18.
- Glass styling details: alpha, stroke widths, highlight placement, and the etched-line dash pattern.
- Whether the step-0 building frame comes from the same session that Play then advances, or from a second fresh session, within D-07.
- The exact landscape and portrait rectangles for `first-pour`, within D-20.
- Whether the Play button reuses existing `PlaybackButtons` styling primitives.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope and requirements
- `.planning/ROADMAP.md` § "Phase 38: Puzzle Route and Player Shell": goal and five success criteria
- `.planning/REQUIREMENTS.md`: GOAL-01, GOAL-03, PUZ-02..PUZ-06
- `.planning/research/v1.5-puzzle-mode/SUMMARY.md`: Phase 38 delivers and gate, and pitfalls (HUD refit, tilt re-apply, levels kept out of the catalog)
- `.planning/research/v1.5-puzzle-mode/ARCHITECTURE.md`: `PlayableTarget`, `reducePuzzle`, direct backend
- `.planning/research/v1.5-puzzle-mode/PITFALLS.md`: camera refit, tilt re-apply, catalog coupling
- `.planning/research/v1.5-puzzle-mode/FEATURES.md`: HUD, timer and result-card UX
- `PROJECT-SCOPE.md`: hobby-project scope policy
- `CLAUDE.md` § "Playground canvas framing", "Playground scale legend" and "README scene gallery": framing, `data-hud-obstacle` leaf rule, and legend rule

### Phase 37 bridge contract
- `.planning/phases/37-puzzle-core-in-the-bridge/37-CONTEXT.md`: D-08 placements, D-12 latch, D-14 stepping after the latch, D-16 `levelInfo`, D-17 frame scalars
- `.planning/phases/37-puzzle-core-in-the-bridge/37-02-SUMMARY.md`: `maybePuzzle` parse, fail closed, worker path left scene-shaped
- `.planning/phases/37-puzzle-core-in-the-bridge/37-VERIFICATION.md`: what was verified
- `crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt` and `first-pour-empty.txt`: recorded placements and outcomes for the D-22 seam

### Web code (integration points)
- `web/src/routing/hash.ts`: scene route parser (must stay unchanged)
- `web/src/player/runtime.ts`, `web/src/player/scene-lifecycle.ts`, `web/src/player/playground-session.ts`, `web/src/player/scene-runtime.ts`: lifecycle
- `web/src/components/PlaygroundStage.tsx`, `web/src/components/PlayerPanel.tsx`, `web/src/components/CanvasStage.tsx`, `web/src/components/FallbackPanel.tsx`: shell
- `web/src/player/scene-surface.ts`, `web/src/input/canvas-pointer.ts`, `web/src/input/tilt-binding.ts`, `web/src/input/gravity-slider.ts`: interactions
- `web/src/catalog/portrait-bounds.ts`, `web/src/render/safe-frame.ts`, `web/src/player/camera-frame.ts`, `web/src/player/hud-obstacles.ts`, `web/src/render/camera.ts`: camera
- `web/src/physics/frame.ts` (`maybePuzzle`), `web/src/physics/session.ts`, `web/src/physics/loader.ts`, `web/src/catalog/puzzles.ts`: session and frames
- `web/src/render/present-frame.ts`, `web/src/render/canvas.ts`: overlay hook point (`drawSceneFrame` in `scene-surface.ts`)
- `web/e2e/hud-safe-frame.spec.ts`, `web/e2e/player-helpers.ts`, `web/scripts/demo-media/capture.ts` (synthetic clock): e2e patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PUZZLE_IDS` / `PuzzleId` (`web/src/catalog/puzzles.ts`): the level allowlist already mirrored from Rust
- `RenderFrame.maybePuzzle` / `PuzzleFrameStatus` (`web/src/physics/frame.ts`): goal count, target, full, outcome, outcome step and step limit, parsed fail closed
- Generated `ProofSession.newPuzzle` and `levelInfo()` → `PuzzleLevelInfo` (`cupInterior`, `buildArea`, `fullCount`, `targetCount`, `settleSteps`, `stepLimit`, `title`, `toolKinds`, `toolCounts`, `gridMeters`). `GeneratedProofSession` in `session.ts` needs `levelInfo` added to its type.
- `FallbackPanel`, `ScaleLegend`, `safeFrame` + `hud-obstacles` measurement, `cameraFrame`, `projectPoint`: reuse as is
- `web/scripts/puzzle-replay/` has a TS fixture parser that can be reused for the D-22 seam's placements
- e2e helpers: `installControlledRefreshRate`, the synthetic animation clock, `SCENE_HASH_PATHS`

### Established Patterns
- SolidJS 1.9 with Kobalte and Tailwind v4, plus plain CSS in `web/src/styles/`. Theme tokens are in `src/styles/tailwind.css`.
- The mutable `SceneRuntime` bag plus Solid signals. Pure helpers are tested with Vitest using Arrange/Act/Assert.
- `data-hud-obstacle` goes on leaf elements only. Obstacles are measured only in canvas-stage mode.
- The direct backend's `nextFrame` = `advance` + `captureFrame` + `parseRenderFrame`. `MAX_ADVANCE_STEPS` is 4 and must not change.
- File-length limits are enforced by `bun scripts/bright-builds-check.ts all`. Several shell files are near their limits.

### Integration Points
- `PlaygroundStage`'s `<Show when={maybeCurrentSceneId()}>` branch becomes a target switch: scene → `PlayerPanel` as today, puzzle → a puzzle player that reuses the canvas surface, unknown → the fallbacks.
- `handleHashChange` in `scene-lifecycle.ts` gains a puzzle arm.
- The resize path in `frame-loop.ts` uses `maybeReadySceneId`. It must route through `PlayableTarget` / `worldBoundsForTarget` so puzzles also refit on resize.
- `drawSceneFrame` in `scene-surface.ts` is the hook for the glass-cup overlay after `presentSceneFrame`.

</code_context>

<specifics>
## Specific Ideas

- Copy text examples: loss "Out of time: 54% of 80%" (from the requirement); win "Filled! 86% of 80%".
- The timer is labeled as simulation time, because on slow devices steps run slower than wall time (research open question).
- Fixed HUD slots, with content swapped inside them, keep the camera from jumping. This is the single most important phone-UX rule for this phase.

</specifics>

<deferred>
## Deferred Ideas

- `#/puzzles` level grid and a sidebar Puzzles section: Phase 41
- Stop/Reset during a run, Clear, the tool tray and editor: Phase 39
- ★ Brim threshold data from Rust and star awarding: Phase 39 (the tick list is ready for it)
- Fast-forward 2×/4×: Phase 39
- Worker backend for puzzles: only if phone step cost needs it (research open question)
- `EXT_color_buffer_half_float` acceptance fix in `webgl-particles.ts`: recommended separate quick task (research SUMMARY), not part of this phase

</deferred>

---

*Phase: 38-puzzle-route-and-player-shell*
*Context gathered: 2026-10-12*
