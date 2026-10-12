---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: 2026-10-12T22:36:41.391Z
---

# Phase 39: Place-then-Play Tools and Stars - Context

**Gathered:** 2026-10-12
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Turn the Phase 38 puzzle player into a place-then-play game. While building, the player places a limited set of fixed-size tools from a per-level tray. They can move, rotate (τ/24 steps) and remove tools with mouse, touch or keyboard. They press Play, and a winning run earns up to three stars: ★ Win, ★ Brim and ★ Thrifty. Deliverables:

- the authoritative Rust placement validator, with overlap and no-place checks, exposed to TS for live feedback
- the tool tray, the selection bar and the on-canvas tool overlay
- the editor state (place, move, rotate, nudge, remove, Clear) inside the puzzle phase machine
- Stop/Reset during a run, returning to building with placements kept
- the Brim and Thrifty data in `LevelSpec` and `levelInfo()`, plus the per-step peak-held fill in Rust
- `stars.ts` scoring, with the best result per star kept for the page session
- fast-forward at 1×, 2× and 4×
- keyboard parity
- a Playwright touch project and the edit-history fingerprint test

Out of scope here:

- the timed valve and its 4th pose value (TOOL-05, Phase 40)
- new levels, new tool kinds that only later levels need, and level tuning (Phase 40)
- localStorage persistence of stars and placements, `#/puzzles` and unlocks (Phase 41)
- sound (Phase 42) and GPU water (Phase 43)
- hints, two-finger rotate, drag-from-tray-only flows and the last-run trail ghost (deferred)

`crates/liquidfun` is not modified.

</domain>

<decisions>
## Implementation Decisions

### Validator: Rust is the single source of truth
- **D-01:** Rust owns all placement validation. Extend `puzzle/tools.rs` (or a sibling `puzzle/validate.rs`) with a pure validator that returns a per-placement status. Do not write a TS mirror of the geometry. The validator checks:
  - the existing Phase 37 checks: kind allowlist, per-kind budget, lane lengths, `angle_index < 24`, and the pose centre inside the build area
  - overlap of the tool polygon, inflated by one particle diameter (2 × `particle_radius`), with any level wall, any cup wall, the cup interior, any initial liquid block rectangle, or another placed tool

  The build-area rule stays "the pose centre is inside the build area", because that is what Phase 37 shipped and the recorded `first-pour` solution relies on it. The player-facing wording is "outside the build area".
- **D-02:** Expose one stateless wasm function, for example `inspectPuzzlePlacements(levelId, kinds, poses)`. It needs no session and no world, and it returns two lanes:
  - a status code per placement (valid, or a specific invalid reason)
  - the baked outline vertices per placement, which are the same τ/24-table vertices the builder uses for bodies

  TS calls it on every edit, at most once per animation frame while dragging. The answer is used for the red/valid tint, for Play enablement and for hit-testing and drawing. Parity therefore holds by construction, and the D-03 test only needs to confirm that `newPuzzle` agrees.
- **D-03:** `ProofSession.newPuzzle` runs the same validator and fails closed with a typed `SessionError` when any placement is invalid. Add a Rust unit test showing that every status the inspector reports as invalid makes `newPuzzle` fail, and every all-valid set builds. The recorded `first-pour-solution.txt` placement must stay valid; add a test for that.
- **D-04:** Overlap uses convex-polygon SAT on baked vertices, with no `sin`/`cos`. Tools and walls are convex quads. The cup interior and the liquid blocks are axis-aligned rectangles. Edges and touching count as overlap after inflation, so the result fails closed.
- **D-05:** These are the only `LevelSpec` additions in this phase: `brim_count` (u32), `par_tools` (u32), and an optional `no_place: &'static [GridRect or rect]` list for extra authored zones. `first-pour` uses an empty list. The cup interior and the liquid blocks are always implicit no-place zones. `levelInfo()` gains `brimCount` and `parTools` getters.

### Tools and the first level
- **D-06:** Phase 39 ships the existing `plank` kind only. The tray, validator, overlay and star code are data-driven over the level's tool budget (kinds and counts from `levelInfo()`), and Vitest covers the tray with a fake budget of two kinds. New kinds, such as a short plank or a pin, arrive in Phase 40 when a level needs them.
- **D-07:** `first-pour` changes its budget from 1 plank to **2 planks with `par_tools = 1`**, so Thrifty means something. Its geometry, liquid, target, full count and step limit stay unchanged, so the existing fixtures keep their outcomes. Choose `brim_count` so that it is at least 10 percentage points of `full_count` above target, and so the 1-plank reference solution clears it by at least 5% of `full_count` (the D-03 margin style from Phase 37). The reference solution is then a 3★ solution. Record the value and the measured peak held fill in the SUMMARY.

### Peak-held fill and when a run is decided (Rust, per step)
- **D-08:** Add a pure, step-based **peak held count** to `goal.rs`: the largest value `c` such that `count >= c` held for `settle_steps` consecutive steps at or before `step_limit`. In other words, it is the running maximum of the trailing-window minimum. Unit-test it in Arrange/Act/Assert form:
  - a splash shorter than the window does not raise it
  - it never decreases
  - it stops updating after `step_limit`

  Win is unchanged (`peak_held >= target_count`, equivalent to the Phase 37 latch). The existing `outcome` and `outcomeStep` semantics and both recorded fixtures must not change.
- **D-09:** Brim latches at the first step where `peak_held >= brim_count`, at or before `step_limit`. Thrifty is not a Rust concept; it is judged in TS from the placement count at Play.
- **D-10:** **When the run is decided:**
  - A lost run is decided at its loss latch (`step_limit`), as before.
  - A won run is decided at the Brim latch step if Brim is reached, otherwise at `step_limit`.

  Phase 38's 120-step settle tail and the result card start at the **decision** step, not at the win latch. After a win and before the decision, the fill meter lights its Win tick and the HUD shows a short non-blocking "Filled!" state in the result slot. This supersedes Phase 38 D-08/D-14 timing for won runs only. Lost runs behave exactly as in Phase 38.
- **D-11:** New optional frame scalars, present only on puzzle frames: `peakHeldCount`, `brimCount`, `brimStep` (`undefined` until Brim latches) and `decidedStep` (`undefined` until decided). Field names are at Claude's discretion. `web/src/physics/frame.ts` parses them into `maybePuzzle`, failing closed, with Vitest coverage. Scene frames are unchanged.
- **D-12:** Extend the replay fixtures and the Rust and Bun replay tests to assert `peak_held` and `brim_step` for the solution fixture. Add the new lines behind the existing explicit re-record flag. Existing `outcome`, `outcome_step` and `final_count` values must not change; if one does, stop and investigate.

### Stars
- **D-13:** Star rules, in a pure TS `web/src/puzzle/stars.ts`:
  - ★ Win: the run won.
  - ★ Brim: the run won and Brim latched.
  - ★ Thrifty: the run won with at most `parTools` placements at Play.

  Stars are earned only on a winning run. Thresholds come from `levelInfo()` and the frame, never from TS constants.
- **D-14:** "Best result per star across runs" is a per-level, per-criterion OR merge, kept in an in-memory store for the page session. It survives Retry, Edit, Stop and route changes within the page, but not a reload. Phase 41 persists this store to localStorage. Design its shape so Phase 41 can serialize it without changes, for example `{levelId → {win, brim, thrifty, bestPeakHeldPct}}`. Never lower a stored star.
- **D-15:** Stopping a run after the Win latch still records every star that has already latched (Win, Thrifty if eligible, and Brim if latched). Latches are permanent, so this is deterministic and never lowers a best result. Stopping before a win records nothing.
- **D-16:** Criteria are visible before Play. During building and running, the result slot (the Phase 38 fixed-size slot) shows a compact checklist: "★ Win 60%", "★ Brim N%" and "★ Thrifty ≤ 1 tool", each marked as already earned when it is in the best-of store. The fill meter and the glass cup gain the Brim tick through the Phase 38 data-driven tick list. At the end, the result card shows this run's three rows (earned or not) plus the best-of stars. The win text becomes "Filled! {peak held %} of {target %}", using the peak held fill rounded down, with the same pure formatters as Phase 38.

### Editor interactions
- **D-17:** Placing a tool uses **tap-to-arm, then tap the canvas**, for both mouse and touch:
  1. Tap or click a tray item to arm that kind. The item stays highlighted, and tapping it again disarms it.
  2. Tap or click the canvas to place one tool there, with its centre snapped to the 0.05 m grid and angle index 0. The new tool is selected, and the kind disarms after one placement.

  Tray items show the remaining count, and a kind at 0 is disabled. Drag-from-tray is not required (deferred).
- **D-18:** To move a tool, drag it. The grab offset is kept, and touch drags add a small upward lift so the finger does not hide the tool. The ghost snaps live to the grid during the drag. Hit-testing uses the outlines from D-02 in world space, with a screen-space slop so every tool's hit area is at least 44×44 CSS px. Tapping empty canvas deselects.
- **D-19:** Rotation and removal use a **selection bar**, a fixed HUD slot with buttons of at least 44 px: rotate counter-clockwise (+15°), rotate clockwise (−15°) and Remove. No on-canvas rotate handle and no two-finger twist. The bar is mounted for the whole puzzle route; its buttons are disabled when nothing is selected or outside building.
- **D-20:** Invalid placements are allowed to stay where they are. They are tinted red, the selection bar or a status line names the reason (for example "Overlaps the cup"), and Play is disabled with an accessible reason while any placement is invalid. A tool does not snap back on release. Zero placements is valid, so Play is enabled and the level loses naturally.
- **D-21:** The transport slot (Phase 38's Play slot) holds:
  - Play in building, Stop in running (Space toggles)
  - a speed control cycling 1× → 2× → 4×
  - Clear, which is secondary, enabled only in building with at least one placement, and removes every placement with no confirm dialog

  Stop/Reset in running returns to building on a fresh step-0 session with the placements kept. Retry and Edit from Phase 38 are unchanged.
- **D-22:** Tools are drawn by the TS overlay from placement state and the D-02 outlines in every phase: building, running and ended. Static tools never move, so this matches the physics bodies, which also appear in `rigidSegments` while running. Draw in this order: particles, then tools, then the glass cup overlay. Building tints: a normal tool color, a selected highlight, and red for invalid. Tools are locked and undimmed while running.

### Keyboard parity
- **D-23:** Keys, active while the puzzle canvas region or the puzzle HUD has focus:
  - Arrow keys nudge the selected tool by one grid cell (0.05 m), and Shift+Arrow by 10 cells.
  - R rotates +15° and Shift+R rotates −15°.
  - Delete and Backspace remove the selected tool.
  - Space toggles Play/Stop.
  - Escape deselects and disarms.

  Tray items are focusable buttons. Pressing Enter on one places a tool at the build-area centre, snapped, and selects it. The canvas region is focusable with an accessible label. Placed tools must be selectable without a pointer; Tab and Shift+Tab cycle the selection while the canvas region has focus (exact binding at Claude's discretion). Editing keys do nothing outside building.

### Fast-forward
- **D-24:** Speed changes only how many fixed steps run per presented frame: 1×, 2× or 4× the normal budget. It keeps the existing per-call cap (`MAX_ADVANCE_STEPS` stays 4) by calling advance several times per frame if needed. Debt is still dropped on slow devices. The outcome, the timer (step-based), the decision step and the settle tail are therefore identical at every speed. Speed persists across runs in the page session and starts at 1× on a new route. Scenes are unaffected.

### Puzzle phase machine
- **D-25:** Extend `reducePuzzle` and the puzzle store so that placements, the selection and the armed kind live in building state, and running and ended carry the placements used for that run. New events are arm, place, select, move, rotate, nudge, remove, clear, stop and setSpeed. The validation answer from D-02 is stored next to the placements. Edit events in non-building phases are ignored (return the same state). Unit-test the reducer separately from the DOM.

### HUD and camera rules
- **D-26:** New HUD leaves, the **tool tray** and the **selection bar**, carry `data-hud-obstacle` on the leaf. They are mounted when the puzzle route mounts and have fixed box sizes per layout that do not depend on phase, content or camera scale, so `safeFrame` never refits between building, running and ended (Phase 38 D-18). The tray goes at the bottom in portrait canvas-stage layout and at the side in landscape and desktop, at Claude's discretion within this rule. Extend the Phase 38 camera-stability e2e to cover placing, Play and Stop.

### Tests and gates
- **D-27:** **Edit-history fingerprint test**, at three levels:
  1. Vitest: a reducer sequence of place, remove, re-place and reset ×3 yields canonical placements equal to a single clean placement.
  2. Rust: placements given in permuted input order produce identical frame fingerprints after N steps.
  3. One e2e: performs that edit sequence in the UI, then compares a fixed-step frame fingerprint, read through the Phase 38 test seam or a debug attribute, with a clean placement.
- **D-28:** **Synthetic-clock e2e wins by placing a tool**: arm the plank, tap at the reference centre computed from the published camera transform, rotate clockwise once (angle index 23), press Play, and see the win card with stars. Add a Playwright `hasTouch` mobile project, iPhone-sized portrait, that runs the same flow with taps. Also add an e2e that runs the reference solution at 1×, 2× and 4× and asserts the same outcome step, decision step and peak held fill.
- **D-29:** Validator unit tests cover each invalid reason (wall, cup wall, cup interior, liquid, tool-tool, outside the build area, budget, angle, kind) and the valid reference pose. Add Vitest for the D-02 lane decoding (failing closed on malformed lanes), `stars.ts`, the best-of store, the speed control and the keyboard map.
- **D-30:** The phase is done only if all of these hold:
  - `just playground-scene-spot` reports 25/25 fingerprints identical to the pre-phase baseline. Record the baseline from phase-base HEAD.
  - `git diff --stat <phase-base> -- crates/liquidfun` is empty.
  - The Phase 37 and 38 fixtures replay unchanged: `cargo test -p liquidfun-wasm`, the `--release` replay and `just puzzle-replay-wasm`.
  - These pass: `just web-player-smoke`, the existing HUD safe-frame e2e and the puzzle e2e suite (updated only where D-10 intentionally changes won-run timing; any such edit is listed in the SUMMARY).
  - Rust fmt, clippy (`-D warnings`), build and tests pass, plus the touched Vitest suites, `npx tsc --noEmit` and `bun scripts/bright-builds-check.ts all` (file-length limits). Add new files rather than grow files near their limits (`puzzle-lifecycle.ts`, `PuzzlePanel.tsx`, `PuzzleHud.tsx`, `puzzle.css`, `replay.rs`).

### Claude's Discretion
- File and module split, for example `puzzle/validate.rs`, `web/src/puzzle/editor.ts`, `stars.ts`, `star-store.ts`, `keyboard.ts`, `components/PuzzleToolTray.tsx` and `PuzzleSelectionBar.tsx`.
- Exact wasm function name and lane layout for D-02, and the status code values.
- Exact `brim_count`, within D-07.
- Exact tray and selection-bar placement and styling per layout, within D-26, and the tool colors.
- Throttling details for inspector calls during a drag.
- The key that cycles selection among placed tools (D-23).
- Whether the "Filled!" in-progress state lives in the result slot or the fill meter chip, within D-10's fixed-slot rule.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope and requirements
- `.planning/ROADMAP.md` § "Phase 39: Place-then-Play Tools and Stars" and § "Shared acceptance rules": goal, five success criteria, the fingerprint gate and the construction-input rule
- `.planning/REQUIREMENTS.md`: TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-06, TOOL-07, STAR-01, STAR-02 (TOOL-05 is Phase 40)
- `.planning/research/v1.5-puzzle-mode/SUMMARY.md`: Phase 39 delivers and gate, the open-decision table (star merge, win hold) and the research flags (touch rotation, validator parity)
- `.planning/research/v1.5-puzzle-mode/FEATURES.md`: TOOL, PLACE, MODE and STAR table-stakes rows, the "Star rule" section and fast-forward
- `.planning/research/v1.5-puzzle-mode/ARCHITECTURE.md`: placement editor, ghosts with no wasm call per drag, static tools and the rigid lanes
- `.planning/research/v1.5-puzzle-mode/PITFALLS.md`: creation order and live edits (the permuted-history test), overlap at Play start (3-diameter thickness, no-place zones), HUD refit
- `PROJECT-SCOPE.md`: hobby-project scope policy
- `CLAUDE.md` § "Playground canvas framing" and § "Playground scale legend": the `data-hud-obstacle` leaf rule and the legend rule

### Prior phase contracts
- `.planning/phases/37-puzzle-core-in-the-bridge/37-CONTEXT.md`: D-03/D-22 margin and release-vs-wasm contract, D-08 placement lanes, D-09 canonical order, D-10 rebuild on Play/Reset, D-12 latch, D-18/D-19 fixtures and re-record flag
- `.planning/phases/38-puzzle-route-and-player-shell/38-CONTEXT.md`: D-06 phase machine, D-08 settle tail, D-13 tick list, D-14 result slot, D-16..D-19 interactions, chrome, fixed HUD slots and the camera-stability e2e, D-22 test seam
- `.planning/phases/38-puzzle-route-and-player-shell/38-UI-SPEC.md`: visual and copy contract for the puzzle HUD that this phase extends
- `.planning/phases/38-puzzle-route-and-player-shell/38-VERIFICATION.md`: what Phase 38 verified

### Bridge code
- `crates/liquidfun-wasm/src/puzzle/tools.rs` and `tools/tests.rs`: `ToolKind`, plank dimensions, `Placements::parse`, `PlacementError`, `GridRect`
- `crates/liquidfun-wasm/src/puzzle/angles.rs`: the τ/24 `(cos, sin)` table
- `crates/liquidfun-wasm/src/puzzle/spec.rs`, `first_pour.rs`: `LevelSpec` and the level data (`REFERENCE_PLANK_POSE`)
- `crates/liquidfun-wasm/src/puzzle/goal.rs`, `hooks.rs`: `GoalTracker`, `count_in_goal` and the per-step hook
- `crates/liquidfun-wasm/src/puzzle/info.rs`, `builder.rs`, `replay.rs`, `fixtures/`: `levelInfo()`, the builder and the replay harness
- `crates/liquidfun-wasm/src/lib.rs`, `frame.rs`, `session.rs`: `ProofSession`, frame scalars and `MAX_ADVANCE_STEPS`

### Web code
- `web/src/puzzle/reduce-puzzle.ts`, `placements.ts`, `format.ts`, `controller.ts`: phase machine, placement lanes, formatters
- `web/src/player/puzzle-lifecycle.ts`, `puzzle-loop.ts`, `puzzle-runtime.ts`: session lifecycle and stepping (fast-forward hook point)
- `web/src/physics/puzzle-session.ts`, `puzzle-loader.ts`, `frame.ts`: generated session wrapper and frame parsing
- `web/src/components/PuzzlePanel.tsx`, `PuzzleHud.tsx`, `PuzzleResultCard.tsx`, `PuzzleOptionsDrawer.tsx`, `web/src/styles/puzzle.css`: HUD slots
- `web/src/render/glass-cup.ts` (or the Phase 38 overlay module), `web/src/render/camera.ts` (`unprojectPoint`), `web/src/input/canvas-pointer.ts`: overlay and pointer mapping
- `web/src/catalog/puzzles.ts`, `puzzle-levels.ts`, `puzzle-bounds.ts`: registry, titles and bounds
- `web/e2e/puzzle.spec.ts`, `web/e2e/puzzle-helpers.ts`, `web/playwright.config.ts`, `web/scripts/puzzle-replay/`: e2e and replay patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Placements::parse` and `PlacementError` (Rust): the fail-closed checks that the validator extends. `angles.rs` gives exact rotated vertices with no trig.
- `encodePlacementLanes` / `PuzzlePlacement` (TS): integer-checked lane encoding, already used by the lifecycle to build sessions with placements.
- `reducePuzzle` with building, running and ended, `SETTLE_TAIL_STEPS`, and the Phase 38 formatters: extend these rather than replace them.
- The Phase 38 data-driven tick list on the fill meter and the glass cup: Brim plugs in as one more tick.
- The test seam `maybeReadTestPlacements` and the synthetic clock helpers in `web/e2e/puzzle-helpers.ts`.
- `unprojectPoint` and the camera frame for screen-to-world mapping, and the existing CSS-bound pointer mapping in `canvas-pointer.ts`.

### Established Patterns
- Pure helpers plus Vitest in Arrange/Act/Assert form. Typed fail-closed parsing of every wasm lane.
- Fixed HUD slots with content swapped inside them. `data-hud-obstacle` goes on leaves only.
- Rust unit tests stay inside the `liquidfun-wasm` lib test binary (no new integration-test binaries, because of the macOS launch-scan cost).
- File-length limits enforced by `bun scripts/bright-builds-check.ts all`.

### Integration Points
- `puzzle-lifecycle.ts`: Play, Retry, Edit and the new Stop build fresh sessions from the store's placements.
- `puzzle-loop.ts`: the per-frame advance budget, where the speed multiplier goes, and the decision step and settle tail.
- `PuzzlePanel.tsx` / `PuzzleHud.tsx`: mount the tray and selection bar as fixed slots, and route canvas pointer events to the editor in building.
- `info.rs` / `PuzzleLevelInfo`: add `brimCount` and `parTools`. `frame.rs`: add the D-11 scalars. `lib.rs`: export the D-02 inspector.

</code_context>

<specifics>
## Specific Ideas

- The genre references are The Incredible Machine (Stop keeps your parts), Cut the Rope (three independent, pre-visible stars) and Sugar Sugar (a cup full at N grains).
- "Peak held fill" is the single quantity behind Win, Brim and the win-card percentage, so a later slosh never lowers what the player earned.
- Reference win flow for e2e: plank at grid (8, 64), angle index 23 (−15°), which is one clockwise rotate press from 0.

</specifics>

<deferred>
## Deferred Ideas

- Drag-from-tray placement on desktop: optional polish, after tap-to-arm.
- Two-finger twist rotate: only if the selection-bar buttons test poorly on phones.
- Hint ladder (text nudge, then a ghost of one reference-solution tool): P2.
- Last-run water trail ghost: P3.
- Additional tool kinds (short plank, pin): Phase 40, as levels need them.
- localStorage persistence of stars and last placements: Phase 41 (PROG-01, PROG-04).
- Timed valve and its 4th pose value: Phase 40 (TOOL-05).

</deferred>

---

*Phase: 39-place-then-play-tools-and-stars*
*Context gathered: 2026-10-12*
