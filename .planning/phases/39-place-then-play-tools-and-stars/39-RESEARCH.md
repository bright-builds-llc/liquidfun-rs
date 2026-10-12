# Phase 39: Place-then-Play Tools and Stars - Research

**Researched:** 2026-10-12
**Domain:** Rust/WASM placement validation (convex SAT on baked τ/24 vertices), per-step peak-held fill, SolidJS place-then-play editor (pointer, touch, keyboard), star scoring, Playwright touch e2e
**Confidence:** HIGH for the code integration points, geometry and measured numbers (read from source and run against the generated wasm at HEAD `ee177a478`); MEDIUM for UI layout fit in phone landscape (computed from the Phase 38 UI-SPEC numbers, not rendered)

## Summary

Phases 37 and 38 left clean seams for everything this phase needs. On the Rust side, `Placements::parse` (`puzzle/tools.rs`) already fails closed for kind, budget, lane length, angle and build-area centre, and `Placement::vertices()` already bakes plank corners from the literal `TAU_24` table. The validator therefore adds one pure SAT routine over `[Vec2; 4]` quads and a per-placement status pass. The status pass serves a stateless wasm inspector and is also called by `build_puzzle_core`, so the browser and `newPuzzle` run the same code (D-02, D-03). The peak-held fill is a small integer tracker next to `GoalTracker` in `PuzzleHooks::on_after_step`. Its four new scalars ride on the existing `PuzzleStatus` / `ProofFrame` optional-getter pattern. None of this touches `crates/liquidfun` or any scene path, so the 25 fingerprints stay unchanged by construction. The gate still runs.

On the web side, the puzzle loop already steps **one fixed step per capture** (`puzzle-loop.ts` `stepOnce` → `nextFrame(1)`). The reducer observes every step, so moving the "ended" transition from the win latch to the D-10 decision step is a local reducer change. Fast-forward is a pure multiplier on the per-frame step budget, with no change to `MAX_ADVANCE_STEPS`. The biggest structural decision is how the building phase shows tools. The Phase 38 Play path advances the *same* step-0 session (D-07). Once placements change while building, that session's `rigidSegments` go stale. Recommended: the building phase always shows a **no-placement preview session**, tools are drawn only by the TS overlay from the inspector outlines, and Play builds a **fresh run session** from the canonical placements. D-07 allows "an equivalent fresh one", and this matches the roadmap rule "rebuilt at step 0 from (level, canonical placements) on Play and Reset".

Measurements against the generated wasm (release, exact with native release per Phase 37): the reference plank `(8, 64, 23)` clears every wall, cup wall, the cup interior and the water by at least **0.891 m** under face-normal SAT, far above the 0.08 m inflation. Its peak held count reaches **541 at step 201**, which exactly reproduces the Phase 37 win latch. It reaches **1331 by step 600** and **1361 by step 720**, against `full_count` 900. **Recommended `brim_count = 810` (90%)**: Brim latches at step **262** with peak 811, so the decision step is 262, the held step is 382, and the win card reads "Filled! 90% of 60%".

**Primary recommendation:** Build the Rust validator, inspector, peak tracker and frame scalars first, behind the fingerprint baseline. In parallel, build the pure TS core (editor reducer, stars, store, keyboard map, speed helper) against agreed field names. Then wire the UI with a no-placement preview session in building and a fresh run session on Play.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Validator: Rust is the single source of truth
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

#### Tools and the first level
- **D-06:** Phase 39 ships the existing `plank` kind only. The tray, validator, overlay and star code are data-driven over the level's tool budget (kinds and counts from `levelInfo()`), and Vitest covers the tray with a fake budget of two kinds. New kinds, such as a short plank or a pin, arrive in Phase 40 when a level needs them.
- **D-07:** `first-pour` changes its budget from 1 plank to **2 planks with `par_tools = 1`**, so Thrifty means something. Its geometry, liquid, target, full count and step limit stay unchanged, so the existing fixtures keep their outcomes. Choose `brim_count` so that it is at least 10 percentage points of `full_count` above target, and so the 1-plank reference solution clears it by at least 5% of `full_count` (the D-03 margin style from Phase 37). The reference solution is then a 3★ solution. Record the value and the measured peak held fill in the SUMMARY.

#### Peak-held fill and when a run is decided (Rust, per step)
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

#### Stars
- **D-13:** Star rules, in a pure TS `web/src/puzzle/stars.ts`:
  - ★ Win: the run won.
  - ★ Brim: the run won and Brim latched.
  - ★ Thrifty: the run won with at most `parTools` placements at Play.

  Stars are earned only on a winning run. Thresholds come from `levelInfo()` and the frame, never from TS constants.
- **D-14:** "Best result per star across runs" is a per-level, per-criterion OR merge, kept in an in-memory store for the page session. It survives Retry, Edit, Stop and route changes within the page, but not a reload. Phase 41 persists this store to localStorage. Design its shape so Phase 41 can serialize it without changes, for example `{levelId → {win, brim, thrifty, bestPeakHeldPct}}`. Never lower a stored star.
- **D-15:** Stopping a run after the Win latch still records every star that has already latched (Win, Thrifty if eligible, and Brim if latched). Latches are permanent, so this is deterministic and never lowers a best result. Stopping before a win records nothing.
- **D-16:** Criteria are visible before Play. During building and running, the result slot (the Phase 38 fixed-size slot) shows a compact checklist: "★ Win 60%", "★ Brim N%" and "★ Thrifty ≤ 1 tool", each marked as already earned when it is in the best-of store. The fill meter and the glass cup gain the Brim tick through the Phase 38 data-driven tick list. At the end, the result card shows this run's three rows (earned or not) plus the best-of stars. The win text becomes "Filled! {peak held %} of {target %}", using the peak held fill rounded down, with the same pure formatters as Phase 38.

#### Editor interactions
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

#### Keyboard parity
- **D-23:** Keys, active while the puzzle canvas region or the puzzle HUD has focus:
  - Arrow keys nudge the selected tool by one grid cell (0.05 m), and Shift+Arrow by 10 cells.
  - R rotates +15° and Shift+R rotates −15°.
  - Delete and Backspace remove the selected tool.
  - Space toggles Play/Stop.
  - Escape deselects and disarms.

  Tray items are focusable buttons. Pressing Enter on one places a tool at the build-area centre, snapped, and selects it. The canvas region is focusable with an accessible label. Placed tools must be selectable without a pointer; Tab and Shift+Tab cycle the selection while the canvas region has focus (exact binding at Claude's discretion). Editing keys do nothing outside building.

#### Fast-forward
- **D-24:** Speed changes only how many fixed steps run per presented frame: 1×, 2× or 4× the normal budget. It keeps the existing per-call cap (`MAX_ADVANCE_STEPS` stays 4) by calling advance several times per frame if needed. Debt is still dropped on slow devices. The outcome, the timer (step-based), the decision step and the settle tail are therefore identical at every speed. Speed persists across runs in the page session and starts at 1× on a new route. Scenes are unaffected.

#### Puzzle phase machine
- **D-25:** Extend `reducePuzzle` and the puzzle store so that placements, the selection and the armed kind live in building state, and running and ended carry the placements used for that run. New events are arm, place, select, move, rotate, nudge, remove, clear, stop and setSpeed. The validation answer from D-02 is stored next to the placements. Edit events in non-building phases are ignored (return the same state). Unit-test the reducer separately from the DOM.

#### HUD and camera rules
- **D-26:** New HUD leaves, the **tool tray** and the **selection bar**, carry `data-hud-obstacle` on the leaf. They are mounted when the puzzle route mounts and have fixed box sizes per layout that do not depend on phase, content or camera scale, so `safeFrame` never refits between building, running and ended (Phase 38 D-18). The tray goes at the bottom in portrait canvas-stage layout and at the side in landscape and desktop, at Claude's discretion within this rule. Extend the Phase 38 camera-stability e2e to cover placing, Play and Stop.

#### Tests and gates
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

### Deferred Ideas (OUT OF SCOPE)
- Drag-from-tray placement on desktop: optional polish, after tap-to-arm.
- Two-finger twist rotate: only if the selection-bar buttons test poorly on phones.
- Hint ladder (text nudge, then a ghost of one reference-solution tool): P2.
- Last-run water trail ghost: P3.
- Additional tool kinds (short plank, pin): Phase 40, as levels need them.
- localStorage persistence of stars and last placements: Phase 41 (PROG-01, PROG-04).
- Timed valve and its 4th pose value: Phase 40 (TOOL-05).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TOOL-01 | Per-level tool tray with fixed-size tools and remaining counts | `levelInfo().toolKinds/toolCounts` already exist in Rust (`info.rs`) but are **not yet parsed in TS** (`PuzzleLevelInfo` in `puzzle-session.ts` lacks them). Remaining = budget − placed of that kind (Pattern 4). Budget becomes 2 (D-07). |
| TOOL-02 | Place, move, rotate (15°), remove with mouse/touch, tap-to-place, ≥44 px targets | Editor pointer module attaches in `assignPuzzleCanvas` as a **separate handle** (Pitfall 6). Hit-testing uses inspector outlines plus a 22 CSS px distance slop (Pattern 5). The plank is 0.25 m thick, about 7.3 px at portrait 29.3 px/m, so the slop is required. |
| TOOL-03 | Invalid placements highlighted; Play disabled; Rust rejects the same | Rust `validate.rs` and `sat.rs` with an inspector export and `build_puzzle_core` parity (Patterns 1 and 2). Reference pose verified valid with 0.891 m clearance. |
| TOOL-04 | Play, Stop/Reset keeping placements, Clear | Preview-session/run-session split (Pattern 3). Stop maps to a fresh preview session plus `building` with the run's placements. Clear is a building-only reducer event. |
| TOOL-06 | Keyboard: arrows, R, Delete, Space | Pure key map (`keyboard.ts`) plus a panel-level `keydown` that skips native button activation (Pitfall 8). Enter on a tray item places a plank at build-area centre `(10, 64)`, and the reference is two ArrowLeft presses and Shift+R from there. |
| TOOL-07 | Fast-forward 2×/4× without changing outcome | The loop already steps once per capture. Speed multiplies `stepTime.stepCount` (Pattern 6). Outcome, decision step and held frame are per-step, so they are speed-invariant. |
| STAR-01 | Three criteria visible before Play | Brim and par come from `levelInfo()` (D-05). The result slot checklist and the Brim tick are added through `goalTicks` (format.ts). |
| STAR-02 | Stars on a winning run; best per star kept | `stars.ts` and an in-memory `star-store.ts` with an OR merge (Pattern 7). Recording happens at the decision step, or on Stop after a win (D-15). |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- `crates/liquidfun` must not change (D-30 and the milestone rule). All Rust work stays in `crates/liquidfun-wasm`.
- Rust: no `unwrap()`; `expect()` only where a panic is impossible; `maybe_` prefix for Options; `let ... else`; τ not π (no angle math is needed, since the τ/24 table exists); `thiserror` is not used in this bridge (it uses typed enums with static messages, so follow that). Pre-commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.
- TS: `camelCase`, `maybe` prefix for optional values and fields, early returns, no `innerHTML`; pnpm is not used (the repo uses bun with a lockfile, so respect it). Pre-commit: typecheck (`npx tsc --noEmit` / `bun run typecheck`), Vitest for touched suites, build.
- Tests: Arrange/Act/Assert with explicit comments, one concern per test.
- File-length limit: **628 lines** per tracked source file (`.rs`, `.ts`, `.tsx`, `.css`), enforced by `bun scripts/bright-builds-check.ts all`. [VERIFIED: `--help` output]
- Rust puzzle tests stay inside the `liquidfun-wasm` lib test binary. No new integration-test binaries, because macOS launch scanning costs about 2 minutes per new binary.
- Playground rules: `data-hud-obstacle` on leaves with camera-independent fixed sizes; `ScaleLegend` stays visible; levels never enter `SCENE_IDS`, README plans, `PORTRAIT_VIEW_BOUNDS` or the scene survey.
- Simulation stays renderer-independent; no wall-clock time in the puzzle path. `tests/puzzle-chrome.test.ts` greps puzzle components for `Date.now|performance.now|setTimeout|setInterval|innerHTML` and the word `tilt`.
- `.planning/**` must never be formatted with mdformat.
- GSD workflow: edits happen through `/gsd-execute-phase`.
- Bright Builds: functional core / imperative shell, parse boundary data into domain types, illegal states unrepresentable. Functions over about 161 lines are refactor triggers.

## Standard Stack

No new dependencies. Everything uses what is already pinned in the repo. [VERIFIED: `crates/liquidfun-wasm/Cargo.toml`, `web/package.json`]

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wasm-bindgen | =0.2.128 | Inspector export, new `PuzzleLevelInfo`/`ProofFrame` getters | Already the bridge's only binding crate; free `#[wasm_bindgen]` functions with `&[u8]`/`&[i32]` copied slices match the existing `newPuzzle` lane style |
| liquidfun (path) | workspace | `Vec2` only for validator math; no engine call in the validator | The validator is pure geometry on `[Vec2; 4]` |
| solid-js | 1.9.15 | Tray, selection bar, transport, overlay wiring | Existing player framework |
| @playwright/test | 1.63.0 | Editor e2e plus the new `hasTouch` project | Existing; `browser:install` installs **chromium only** |
| vitest | 5.0.1 | Pure core tests | Existing |
| typescript | 7.0.2 | | Existing (`skipLibCheck` override) |

### Supporting (existing in-repo modules to reuse)
| Module | Purpose | When to Use |
|--------|---------|-------------|
| `puzzle/angles.rs` `TAU_24` | Unit edge normals of a rotated plank, `(cos, sin)` and `(−sin, cos)`, with no sqrt | SAT axes for tools |
| `Placement::vertices()` | Exact builder vertices | Inspector outline lane and SAT polygon |
| `scene_spot/frame_hash.rs` `frame_hash(&ProofFrame)` | FNV-1a 64 over frame lanes | D-27 Rust permuted-order fingerprint |
| `web/scripts/puzzle-replay/hash.ts` `fnv1a64Lanes` | Same hash in TS | D-27 e2e held-frame hash (move to `src/` or import from there) |
| `goalTicks`, `floorPercent`, `formatSimulationTime` (`puzzle/format.ts`) | Brim tick, star percentages, timer | D-16 |
| `glassCup` / `drawGlassCup` | Brim tick on glass | Append a tick; no draw change |
| `unprojectPoint` / `projectPoint` (`render/camera.ts`) | Pointer to world, outline to screen | Editor and overlay |
| `cssPointFromClient` (`input/pointer.ts`) | CSS point from client coordinates | Editor pointer |
| `installControlledRefreshRate` (`e2e/player-helpers.ts`) | The "synthetic clock" used by the Phase 38 puzzle e2e | All new e2e |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Face-normal SAT with a gap threshold | Minkowski inflation of the plank half-extents (+0.08) then plain SAT | Equivalent for axis-aligned obstacles. The gap-threshold form is one routine for every pair and needs no inflated vertex set. Both fail closed. |
| Free function `inspectPuzzlePlacements` | Static method `ProofSession.inspectPlacements` | A static method avoids a new import in `puzzle-loader.ts`, but D-02 names a free function. Either works with wasm-bindgen 0.2.128. Recommend the free function for clarity, since it has no session semantics. |
| Rebuild the building session on every committed edit | Preview session with no placements, plus overlay-only tools | Rebuilding costs a world build (about 2,000 particles) per edit, and ARCHITECTURE Anti-Pattern 3 warns against it. The preview approach costs nothing per edit. |

**Installation:** none.

## Architecture Patterns

### Recommended Project Structure (new files; existing ones only lightly touched)

```
crates/liquidfun-wasm/src/puzzle/
├── sat.rs                # pure convex-quad separation with gap (D-04) + tests module
├── validate.rs           # PlacementStatus, inspect(spec, kinds, poses), validate_all (D-01..D-03)
├── validate/tests.rs     # every reason + reference pose + parity with build_puzzle_core
├── peak.rs               # PeakHeldTracker + brim latch + decided step (D-08..D-10)
├── peak/tests.rs
├── replay/fixture.rs     # (split out of replay.rs, 487 lines) grammar incl. peak_held, brim_step
crates/liquidfun-wasm/src/inspect.rs  (or puzzle/inspect.rs)  # #[wasm_bindgen] inspector + PlacementInspection

web/src/puzzle/
├── editor.ts             # pure building-state edits: arm/place/select/move/rotate/nudge/remove/clear
├── placement-status.ts   # status codes + player copy ("Overlaps the cup"), mirrors Rust (include_str! test)
├── stars.ts              # D-13 scoring
├── star-store.ts         # D-14 OR-merge store, serializable shape
├── keyboard.ts           # pure key -> editor/transport command map (D-23)
├── speed.ts              # 1/2/4 cycle + step-budget multiplier helper (D-24)
├── hit-test.ts           # point-to-quad distance + 44 px slop (D-18)
web/src/physics/
├── puzzle-inspector.ts   # wraps generated inspector, decodes lanes fail-closed (D-02)
├── frame-puzzle.ts       # (optional split) D-11 scalar parsing out of frame.ts
web/src/player/
├── puzzle-editor-input.ts  # pointer binding for building (attached from assignPuzzleCanvas)
├── puzzle-run.ts           # Play/Stop/Clear/run-session build, star recording (keeps puzzle-lifecycle.ts < 628)
web/src/render/puzzle-tools.ts      # tool overlay draw (normal / selected / invalid / locked)
web/src/components/
├── PuzzleToolTray.tsx, PuzzleSelectionBar.tsx, PuzzleTransport.tsx, PuzzleStarList.tsx
web/src/styles/puzzle-editor.css   # new stylesheet (puzzle.css is 420 lines)
web/e2e/
├── puzzle-editor.spec.ts           # mouse + keyboard + invalid + Clear + Stop + edit-history + fast-forward
├── puzzle-touch.spec.ts            # hasTouch project only
└── puzzle-editor-helpers.ts        # canvas point from world, placement attribute readers
```

### Pattern 1: Pure SAT with a gap (Rust, D-04)

**What:** Two convex quads overlap after inflation unless some face-normal axis separates their projections by **more than** `gap`. Unit normals: tool axes come straight from `TAU_24[angle]` as `(c, s)` and `(−s, c)`, which are unit to f32 rounding. Level quads in `first-pour` are all axis-aligned (`quad()`). For any future non-axis-aligned wall, compare `gap_n > gap · |n|` as `gap_n > 0 && gap_n² > gap² · |n|²`, which avoids `sqrt` (IEEE `sqrt` would also be deterministic). Since projection onto a unit axis is 1-Lipschitz, axis separation ≤ Euclidean distance, so this test fails closed: it never accepts a pair closer than `gap`.

**When to use:** tool vs wall, cup wall, cup interior, liquid block, no-place zone and other tools. Use `gap = 2 × spec.particle_radius` = 0.08 m.

```rust
// Source: derived from puzzle/tools.rs vertices() conventions; pure, no trig.
/// True when `a` and `b` are separated by more than `gap` along some edge normal.
pub(crate) fn separated_by_more_than(a: &[Vec2; 4], b: &[Vec2; 4], gap: f32) -> bool {
    edge_normals(a).into_iter().chain(edge_normals(b)).any(|normal| {
        let (a_min, a_max) = project(a, normal);
        let (b_min, b_max) = project(b, normal);
        // Strict: touching or closer than `gap` counts as overlap (fail closed).
        b_min - a_max > gap || a_min - b_max > gap
    })
}
```

Normalize the edge normal for each quad. For the axis-aligned and τ/24 quads used here the length is 1 to rounding, so the planner may normalize with `sqrt` once, or use the squared comparison. Verified clearances with this rule are in "Measured geometry" below. [VERIFIED: scratch SAT script against `first_pour.rs` literals]

### Pattern 2: One status pass for the inspector and `newPuzzle` (D-01..D-03)

```rust
// Source: proposed; mirrors Placements::parse lane checks in tools.rs.
#[repr(u8)]
pub(crate) enum PlacementStatus {
    Valid = 0, UnknownKind = 1, KindNotAllowed = 2, OverBudget = 3,
    AngleOutOfRange = 4, PositionOutOfRange = 5, OutsideBuildArea = 6,
    OverlapsWall = 7, OverlapsCupWall = 8, InsideCup = 9, OverlapsWater = 10,
    OverlapsTool = 11, OverlapsNoPlace = 12,
}

/// Lane errors (too many, length mismatch) fail the whole call; per-placement
/// problems become statuses in INPUT order (not canonical order).
pub(crate) fn inspect(spec: &LevelSpec, kinds: &[u8], poses: &[i32])
    -> Result<Inspection, PlacementError>;
```

- Check lane length and `MAX_PLACEMENTS` **before** allocating, as `Placements::parse` does.
- Report statuses in **input order**, so TS maps them by array index. Canonical sorting belongs only to the build.
- Use a fixed precedence per placement, with the first failing check winning: kind → allowed → angle → i16 range → build area → walls → cup walls → cup interior → water → no-place → tools → budget. OverBudget flags the placements beyond the count, in input order.
- Tool-tool overlap is symmetric, so flag **both** tools.
- For placements with no computable outline (unknown kind, angle ≥ 24, out of i16), emit 8 zeros in the outline lane. The status is authoritative, and TS never draws non-geometric statuses.
- `build_puzzle_core` calls `Placements::parse`, then runs the same overlap pass and maps the first invalid status to a new `PlacementError` variant (`OverlapsWall`, …). `SessionError::InvalidPlacement(PlacementError)` already carries it, with static messages that never echo input.
- Wasm wrapper: an inner `pub(crate) fn inspect_level(level_id: &str, …) -> Result<Inspection, SessionError>`, unit-tested natively. The `#[wasm_bindgen]` shell only converts to `JsError`. **Native tests cannot construct `JsError`**, as the existing comment in `lib.rs` tests notes. Return a `#[wasm_bindgen] pub struct PlacementInspection` with `statuses() -> Box<[u8]>` and `outlines() -> Box<[f32]>` (stride 8: x0, y0 … x3, y3).

### Pattern 3: Preview session in building, fresh run session on Play (TOOL-04, D-22)

```
building : session = newPuzzle(level, [])            // step-0 frame: walls + water only
           tools   = TS overlay from inspector outlines
Play     : runSession = newPuzzle(level, canonical(placements))   // sync; wasm already initialized
           dispose preview; state building -> running(placementsAtPlay)
Stop     : record latched stars (D-15); dispose run; newPuzzle(level, []) -> building(placements kept)
Retry    : ended -> running on newPuzzle(level, runPlacements)    (Phase 38 path, unchanged)
Edit     : ended -> building on newPuzzle(level, [])              (placements kept)
```

Why: `outline_segments()` in `builder.rs` includes plank edges, so a session built with placements draws its own plank outlines in `rigidSegments`. Those go stale after the first edit. Particles are created after the tools from the same group fill, so the step-0 particle positions are identical with or without planks. The building frame is therefore visually faithful. [VERIFIED: `builder.rs` creation order and `outline_segments`] `loadPuzzleSession` awaits `init()`. Add a sync builder for the run session once init has resolved, so Play does not pass through a `loading` flash. Keep a generation guard either way.

### Pattern 4: Editor state inside `building` (D-25)

```ts
type EditorTool = { readonly id: number; readonly placement: PuzzlePlacement };
type BuildingEditor = {
  readonly tools: readonly EditorTool[];        // creation order; ids never reach wasm
  readonly nextId: number;
  readonly maybeSelectedId: number | undefined;
  readonly maybeArmedKind: PuzzleToolKind | undefined;
  readonly revision: number;                    // bumps on every placement change
  readonly maybeInspection: { readonly revision: number; readonly statuses: readonly PlacementStatusCode[]; readonly outlines: readonly Quad[] } | undefined;
};
```

- The reducer stays pure. The shell runs the inspector after a placement-changing event and dispatches `{kind:"inspected", revision, …}`. The reducer drops it when `revision` is stale.
- `canEnablePlay = building && inspection.revision === revision && statuses.every(valid)`.
- `canonicalPlacements()` sorts by `(kindCode, x, y, angle)`, mirroring Rust `Placement: Ord`, for lanes and for the D-27 Vitest comparison.
- `running` and `ended` carry `placementsAtPlay`. Edit events in other phases return the same state object (the existing reducer convention).

### Pattern 5: Hit-testing with a screen-space slop (D-18)

`hit = distance(pointWorld, quad) <= HIT_SLOP_PX / camera.scale`, with `HIT_SLOP_PX = 22`. A 0.25 m plank at 29.3 px/m (portrait first-pour) is 7.3 px thick, so its hit band becomes 7.3 + 44 = 51.3 px ≥ 44. If several tools are hit, pick the smallest distance, and break ties by selected first, then highest id. Distance from a point to a convex quad: 0 if inside (all edge cross products ≥ 0 for counter-clockwise vertices), else the minimum point-segment distance. Pure and Vitest-friendly.

### Pattern 6: Fast-forward as a step-budget multiplier (D-24)

```ts
// in puzzle-loop.ts onFrame, after accumulateStepTime(...)
const requested = stepTime.stepCount * speed;           // 4 → up to 16 at 4×
presentSteps(session, camera, requested);                // still nextFrame(1) per step
// restore only the wall-time share of unrun steps:
// remainder += ((requested - ran) / speed) * STEP_SECONDS  (accumulateStepTime clamps to 4 steps anyway)
```

The 12 ms `FRAME_STEP_BUDGET_MS` still bounds real work per frame, and `accumulateStepTime` still clamps debt. Every step is captured, so the decision step, the peak at decision and the held frame (decision + 120) are identical at every speed. Put the multiplier and restore math in `speed.ts` with Vitest.

### Pattern 7: Stars and best-of store (D-13..D-15)

```ts
export type RunStars = { readonly win: boolean; readonly brim: boolean; readonly thrifty: boolean };
export function runStars(won: boolean, brimLatched: boolean, placementsAtPlay: number, parTools: number): RunStars {
  if (!won) return { win: false, brim: false, thrifty: false };
  return { win: true, brim: brimLatched, thrifty: placementsAtPlay <= parTools };
}
// Store: Map<PuzzleId, { win, brim, thrifty, bestPeakHeldPct }>, OR-merge, max for pct.
// snapshot(): plain JSON-safe object (Object.create(null) or entries array) for Phase 41.
```

Record at the decision step for won runs, or on Stop when the latest frame status is `won`. With D-15 the stars recorded on Stop are those latched at the Stop frame. Keep the store instance in `PuzzleRuntime`, created once per app root, not as a module global, so tests can create fresh stores.

### Pattern 8: Peak-held tracker (Rust, D-08..D-10)

```rust
/// Running max of the trailing `settle_steps` minimum; frozen after `step_limit`.
pub(crate) struct PeakHeldTracker { window: VecDeque<u32>, settle_steps: u32, step_limit: u32,
    brim_count: u32, peak: u32, maybe_brim_step: Option<u32> }
impl PeakHeldTracker {
    pub(crate) fn observe(&mut self, count: u32, step: u32) {
        if step > self.step_limit { return; }
        // push count, pop front beyond settle_steps; if window full, peak = max(peak, min(window))
        // if maybe_brim_step.is_none() && peak >= brim_count { maybe_brim_step = Some(step) }
    }
}
/// decided: Lost -> step_limit; Won -> brim step if latched, else step_limit once observed_step >= step_limit.
```

A 30-entry window scanned per step is trivial cost, with integers only. Run it in `PuzzleHooks::on_after_step` right after `tracker.observe`. Keep `GoalTracker` unchanged (D-08). Add a Rust equivalence test: over synthetic count sequences, `GoalTracker` wins at step `s` exactly when `s` is the first step with `peak >= target`. The measured reference run confirms this: peak 541 ≥ 540 first at step 201, the recorded win step. [VERIFIED: wasm run]

### Measured geometry and numbers (first-pour)

| Item | Value | Source |
|------|-------|--------|
| Reference plank `(8,64,23)` corners | (−1.0812, 3.4675) (1.8165, 2.6910) (1.8812, 2.9325) (−1.0165, 3.7090) | [VERIFIED: scratch SAT script with `TAU_24` f32 values] |
| Closest SAT separations of the reference | reservoir floor 0.891, cup interior 0.891, cup left wall 0.917, reservoir right wall 0.969 m | VALID (> 0.08) |
| Second valid test plank `(30,84,0)` | min separation 0.40 m (reservoir right wall); 0.617 m from the reference | Use for permuted-order and two-plank tests |
| Overlapping tool pair | `(8,64,0)` vs reference: separation −0.634 | Tool-tool invalid fixture |
| Build-area centre (Enter on tray) | grid `(10, 64)`; `(10,64,0)` is valid (like `(8,64,0)`) | Keyboard e2e: ArrowLeft ×2, then Shift+R gives `(8,64,23)` |
| "Lid" over the cup `(20,44,0)` | VALID (0.275 m from the cup) | Not a cheat: it blocks inflow. Level design note for Phase 40 |
| Peak held of the reference | 541 @201, 952 @300, 1331 @600, **1361 @720** (`full_count` 900) | [VERIFIED: generated wasm at HEAD, release-exact] |
| Brim candidates (decision step / peak@d / live count@d / held step / timer@d) | 720: 237/724/830/357/0:08.0 · 765: 248/767/873/368/0:07.8 · **810: 262/811/920/382/0:07.6** · 855: 272/855/955/392/0:07.4 · 900: 285/900/997/405/0:07.2 | [VERIFIED] |
| Empty run | count 0 throughout; lost at 720; peak 0 | [VERIFIED] |

**Recommendation: `brim_count = 810` (90%).** D-07 needs ≥ 630 (60% + 10 pp) and a reference margin of ≥ 45 over brim: 1331 at step 600 (the Phase 37 margin deadline `step_limit − 120`) ≥ 855. It is distinct from the Full tick at 100%, and the decision comes about 1 s of simulation after the win.

### Anti-Patterns to Avoid
- **A TS geometry mirror of the validator:** D-01 forbids it. TS uses only inspector outlines for drawing and hit-testing.
- **Calling `newPuzzle` per edit or per drag move:** use the preview session plus the inspector.
- **Adding the editor pointer as `session.maybeCanvasPointer`:** breaks the `assignPuzzleCanvas` wiring test (Pitfall 6).
- **Recording the win card values from a later frame:** the latch must be taken from the frame whose `stepIndex === decidedStep`. The loop already captures every step.
- **Global module state for stars or speed:** keep them in `PuzzleRuntime` so tests and route resets stay clean.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rotated tool vertices in TS | TS sin/cos rotation | Inspector `outlines` lane | Bit-identical to the bodies (D-02); avoids f64/f32 drift |
| Overlap validity in TS | TS SAT | Inspector `statuses` lane | Single source of truth (D-01) |
| Frame fingerprint | New hash | `frame_hash` (Rust) / `fnv1a64Lanes` (TS) | Already shared and cross-checked native vs wasm (Phase 37) |
| Percent formatting | New rounding | `floorPercent` | D-10/D-16 rounding-down rule and 100% cap |
| Fixture grammar | New format | Extend `replay.rs` / `fixture.ts` grammar with `peak_held`, `brim_step` | Byte-for-byte shared grammar contract |
| Camera mapping | Ad-hoc pixel math | `unprojectPoint`/`projectPoint`; e2e reads `data-world-frame` | Already matches the drawn world |
| Synthetic clock | New fake timers | `installControlledRefreshRate(page, hz)` | Used by all Phase 38 puzzle e2e |

**Key insight:** every hard-to-agree number (vertices, validity, outcome, peak, decision) is computed once in Rust and copied out. TS only presents, edits integers and merges booleans.

## Common Pitfalls

### Pitfall 1: Stale tool outlines from the building session
**What goes wrong:** Building shows the step-0 frame of a session built with old placements, so the plank edges in `rigidSegments` stay at the old pose under the overlay.
**Why:** `outline_segments()` includes planks.
**How to avoid:** Build the preview session with no placements (Pattern 3).
**Warning signs:** a ghost outline left behind after dragging a tool in a seam-injected e2e.

### Pitfall 2: The D-10 reducer still ends at the win latch
**What goes wrong:** `stepRunning` in `reduce-puzzle.ts` transitions to `ended` when `outcome !== "running"` and throws unless `maybeOutcomeStep === stepIndex`.
**How to avoid:** Stay `running` (with a "won, deciding" sub-state for the "Filled!" HUD) until `maybeDecidedStep === stepIndex`. Take the latch values from that frame: `peakHeldCount`, `brimStep`, and `goalCount` for lost runs. `stepEnded` must also check that `decidedStep` and `brimStep` never change. The tail end is `decidedStep + 120`.
**Warning signs:** the reference e2e ends at step 321 instead of 382.

### Pitfall 3: Phase 38 e2e literals that D-10 changes (list them in the SUMMARY)
In `web/e2e/puzzle.spec.ts`, `expectSolutionWin` asserts `"Filled! 76% of 60%"`, timer `"0:08.6"`, held step `"321"` and meter `aria-valuenow "76"`. With brim 810 and the decision at 262, these become `"Filled! 90% of 60%"`, held step `"382"`, and a timer of `"0:07.6"` if the timer freezes at the decision step. The meter value depends on the UI decision below. Lost-run literals (`"Out of time: 0% of 60%"`, `"0:00.0"`, held `"840"`) stay unchanged.
**Decision needed in UI-SPEC:** at the decision step the live count is 920 (100%) while peak held is 811 (90%). Recommend that the meter freezes at the **peak-held %** for won runs in `ended`, so it agrees with the card (UI-D5 spirit), and that the timer counts down until the decision step and freezes there.

### Pitfall 4: The frame-parse ripple and a stale generated wasm
The D-11 fields are required whenever the goal fields are present (fail closed), so **every** Vitest fake with `goalCount` needs the new getters: `tests/frame-puzzle.test.ts`, `puzzle-format`, `puzzle-lifecycle`, `puzzle-loop`, `puzzle-session`, `reduce-puzzle` and `scripts/puzzle-replay.ts`. `web/src/generated/liquidfun-wasm` is gitignored, so rebuild it with `just web-wasm` (or `bun scripts/web-build.ts wasm`) before any puzzle e2e or Bun replay. An old package makes every puzzle frame fail to parse.
**Do not** bound `peakHeldCount` by the current particle count. Eviction can drop particles after the peak. The existing `goalCount ≤ particleCount` check must not be copied to peak, in Rust `set_puzzle_status` or in TS.

### Pitfall 5: Brim and decision values seen at different steps per speed
**What goes wrong:** if the latch is read from "the presented frame" (up to 16 steps after the decision at 4×), the card percentage differs by speed.
**How to avoid:** keep `nextFrame(1)` per step and latch on `stepIndex === decidedStep`, as above. Do not batch `advance(4)` for fast-forward.

### Pitfall 6: Wiring tests that pin "no pointer on puzzle canvases"
`tests/puzzle-shell-wiring.test.ts` asserts `attachCanvasPointer` is **not** called, `session.maybeCanvasPointer` is `undefined` after `assignPuzzleCanvas`, and there is no `canvas-interactive` class. The e2e "turns scene interactions off" asserts `data-pointer-accepted` stays `"0"` after a drag in building. **How to avoid:** use a separate editor binding (`puzzle.maybeEditorInput`), never increment the scene pointer counter, and use a puzzle-specific cursor class. Add any new puzzle components to `PUZZLE_COMPONENTS` in `tests/puzzle-chrome.test.ts`. They must avoid `setTimeout`, `performance.now` and the substring `tilt`. Coalesce with `requestAnimationFrame` instead.

### Pitfall 7: The Playwright touch project runs every spec
**What goes wrong:** adding a second project makes `test:player`, `test:puzzle` and the full suite run **all** specs twice, including in mobile emulation.
**How to avoid:** give the new project `testMatch: /puzzle-touch\.spec\.ts$/` and add `testIgnore: /puzzle-touch\.spec\.ts$/` to the existing `chromium` project. Use Chromium, because `browser:install` installs chromium only: `{ ...devices["Pixel 7"], viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true }`, or an iPhone descriptor with `browserName: "chromium"`. [CITED: playwright.dev/docs/emulation — `hasTouch`, `isMobile`, device descriptors] Use `locator.tap()` and `page.touchscreen.tap(x, y)`, which need `hasTouch: true`.

### Pitfall 8: Keyboard double actions and traps
- Space or Enter on a focused button already clicks it. A panel-level Space handler must ignore events whose target is a `button`, `select`, `a` or `input`, or Play toggles twice.
- Call `preventDefault` for arrows and Space only when handled, so the page does not scroll.
- Tab cycling must not trap focus (WCAG 2.1.2). Recommendation: while the canvas region has focus, Tab moves the selection to the next tool and only `preventDefault`s while a next tool exists. From the last tool, Tab leaves naturally. Shift+Tab is symmetric.
- Ignore keys while the options drawer is open. Its sheet is portaled, so its keys do not bubble to the panel.
- Editing keys are no-ops outside building (D-23), matching the reducer returning the same state.

### Pitfall 9: The tap lands one grid cell off
In portrait (29.3 px/m) one 0.05 m cell is about 1.47 px. Snap with `Math.round(world * cellsPerMeter)`, where `cellsPerMeter = Math.round(1 / gridMeters) = 20`. Compute the e2e tap at the exact grid point; integer rounding of client coordinates (≤ 0.5 px, or 0.34 cell) still rounds to the right cell. Apply the touch **lift only to drags, never to tap-to-place**. Assert the resulting placement through a debug attribute (for example `data-puzzle-placements="plank:8,64,0"`) before rotating, so a miss is diagnosed. Take `data-world-frame` (canvas CSS px) together with `worldBoundsForPuzzle` imported from `src/catalog/puzzle-bounds.ts`, or publish `data-world-bounds`, to map world to page.

### Pitfall 10: Test and fixture churn from the budget change (D-07)
- `api_tests.rs` `invalid_placement_lanes_fail_closed` uses `[1,1]` with poses `8,64,23, 10,64,23` expecting `ToolBudgetExceeded`. With budget 2 it becomes a tool-tool overlap. Use 3 planks for the budget case.
- `api_tests.rs:95` expects `tool_counts() == [1]`.
- `builder/tests.rs` has `first_pour_allows_exactly_one_plank`.
The fixtures and `REFERENCE_PLANK_POSE` are unaffected.

### Pitfall 11: Phone-landscape rail space (UI-SPEC must solve)
At 844×390 with the Phase 38 rails, the free left-rail band runs from the timer bottom (228) to the transport top (338): **about 94 px**. The free right-rail band runs from the result card bottom (280) to the legend top (354): about 58 px after gaps. One 44 px tray row and a 44 px selection bar fit, but the transport cannot grow taller than 44 px without collision. The transport options are a 136×44 row of three 44 px icon buttons (Play/Stop with `aria-label` "Play"/"Stop", speed, Clear) with 2 px gaps, or moving speed and Clear elsewhere. Portrait 390×844 has room: the free world band is 552 px tall against 284 px needed. The landscape result card (136×220) must also hold the outcome plus 3 star rows plus 3 action buttons (about 272 px if naive), so it needs compact star rows or a resized fixed slot. Any fixed size is allowed as long as it is identical in every phase.

### Pitfall 12: Running past the decision without fast-forward on long Brim waits
A won run without Brim runs to `step_limit` (up to 8.6 s at 1× for first-pour). That is intended (D-10). Stop is the escape, and Stop after a win records the latched stars (D-15).

## Code Examples

### Inspector wasm export (shape)
```rust
// Source: pattern of lib.rs newPuzzle + info.rs getters (wasm-bindgen 0.2.128).
#[wasm_bindgen]
pub struct PlacementInspection { statuses: Vec<u8>, outlines: Vec<f32> }

#[wasm_bindgen]
impl PlacementInspection {
    #[must_use] pub fn statuses(&self) -> Box<[u8]> { self.statuses.clone().into_boxed_slice() }
    #[must_use] pub fn outlines(&self) -> Box<[f32]> { self.outlines.clone().into_boxed_slice() }
}

/// Stateless placement check for live editing (D-02); no session, no world.
#[wasm_bindgen(js_name = inspectPuzzlePlacements)]
pub fn inspect_puzzle_placements(level_id: &str, kinds: &[u8], poses: &[i32])
    -> Result<PlacementInspection, JsError> {
    puzzle::validate::inspect_level(level_id, kinds, poses)
        .map(PlacementInspection::from)
        .map_err(js_error)
}
```

### TS lane decoding (fail closed)
```ts
// Source: style of puzzle-session.ts parsePuzzleLevelInfo.
export function decodeInspection(count: number, statuses: Uint8Array, outlines: Float32Array): PlacementInspection {
  if (statuses.length !== count || outlines.length !== count * 8) throw new Error("Invalid placement inspection");
  const codes = Array.from(statuses, (code) => (isStatusCode(code) ? code : invalidInspection()));
  if (!Array.from(outlines).every(Number.isFinite)) return invalidInspection();
  return { statuses: codes, outlines: chunkQuads(outlines) };
}
```

### Frame scalar getters (follow `frame/puzzle.rs`)
```rust
#[wasm_bindgen(js_name = peakHeldCount)] pub fn peak_held_count(&self) -> Option<u32> { self.data.maybe_puzzle.map(|s| s.peak_held_count) }
#[wasm_bindgen(js_name = brimCount)]     pub fn brim_count(&self) -> Option<u32>      { self.data.maybe_puzzle.map(|s| s.brim_count) }
#[wasm_bindgen(js_name = brimStep)]      pub fn brim_step(&self) -> Option<u32>       { self.data.maybe_puzzle.and_then(|s| s.maybe_brim_step) }
#[wasm_bindgen(js_name = decidedStep)]   pub fn decided_step(&self) -> Option<u32>    { self.data.maybe_puzzle.and_then(|s| s.maybe_decided_step) }
```
Rust status consistency (move the check out of `frame.rs`, which is 542 lines, into `frame/puzzle.rs` or `goal.rs`):
- `target < brim ≤ full`
- `brim_step ≤ min(observed, step_limit)`
- brim present ⇒ outcome Won
- decided present ⇔ (Lost) or (Won and (brim present or observed ≥ step_limit)), and decided equals brim, else `step_limit`
- decided ≤ observed

TS parse mirrors these.

### Fixture extension (both grammars)
```
level first-pour
placement plank 8 64 23
outcome won
outcome_step 201
final_count 689
peak_held 811        # frame peakHeldCount at the decided step (record mode prints it)
brim_step 262        # or `none`
```
The empty fixture gets `peak_held 0` and `brim_step none`. Make the pair required for fixtures with expectations (strict grammar, same in Rust and TS). The replay loop must continue to `max(step_limit − 120, decided_step)` to read peak at the decided step. Check `peak_held`/`brim_step` exactly in release and wasm. In debug, check brim within `DEBUG_OUTCOME_STEP_TOLERANCE` and the Brim margin using `maybe_margin_win_step(counts, brim_count, full, settle, step_limit)`. The values in the comments are the measured wasm numbers. The executor must record them with `LIQUIDFUN_PUZZLE_RECORD=1 cargo test -p liquidfun-wasm --release --lib puzzle::replay -- --nocapture` and `bun scripts/puzzle-replay.ts replay --record`, not copy them from here.

### e2e world-to-page mapping
```ts
// data-world-frame = "left,top,right,bottom" in canvas CSS px (projectedWorldRect).
const [left, top, right] = frame.split(",").map(Number);
const scale = (right - left) / (bounds.maxX - bounds.minX);
const pageX = canvasBox.x + left + (worldX - bounds.minX) * scale;
const pageY = canvasBox.y + top + (bounds.maxY - worldY) * scale;
```

## Runtime State Inventory

Not a rename or refactor phase. Omitted.

## State of the Art

| Old Approach (Phase 38) | Current Approach (Phase 39) | Impact |
|--------------|------------------|--------|
| Play advances the same never-advanced building session | Building uses a no-placement preview session; Play builds a fresh run session | Allowed by Phase 38 D-07 ("or an equivalent fresh one") |
| Ended at the win latch; card shows count at the latch | Ended at the decision step; card shows peak-held % | D-10 / D-16; Phase 38 e2e literals change (Pitfall 3) |
| Budget 1 plank | Budget 2, `par_tools` 1, `brim_count` 810 recommended | Fixtures unchanged; three Rust tests change (Pitfall 10) |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Chromium `PointerEvent.clientX` precision (integer or fractional) is enough for grid-exact taps at 1.47 px per cell | Pitfall 9 | Off-by-one cell. Mitigated by asserting the placement attribute and computing the tap at the exact grid point |
| A2 | `newPuzzle` for first-pour (about 2,000 particles) builds in a few ms, fine synchronously on Play | Pattern 3 | A visible hitch on slow phones. Fallback: the async Retry-style path with a generation guard |
| A3 | The timer should count down to the decision step and the meter should freeze at peak-held % for won runs | Pitfall 3 | UX choice only. UI-SPEC decides; e2e literals follow |
| A4 | FNV hashing of the held frame (about 16 KB of f32 data with BigInt) once per run is cheap enough in the browser | D-27 e2e | Could cost a few ms. Hash only on `held`, never per frame |

## Open Questions

1. **Brim value: 810 or 765?**
   - Known: both satisfy D-07, and 810 decides at 262 and 765 at 248.
   - Unclear: how a weaker (two-plank) solution fares. Level tuning is Phase 40.
   - Recommendation: 810. Record it with the measured peak in the SUMMARY (D-07).
2. **`full_count` 900 is far below the real cup capacity (count reaches 1363)**
   - Fill clamps at 100% early, so Brim is reached by almost any win that keeps pouring. This is a Phase 40 tuning item (D-07 locks `full_count` here). Note it in the SUMMARY.
3. **Whether a `data-puzzle-held-hash` debug attribute or the window seam carries the D-27 e2e fingerprint**
   - Recommendation: a `main[data-puzzle-held-hash]` attribute set once when held, plus `data-puzzle-decided-step`, `data-puzzle-peak-held` and `data-puzzle-placements`. These are deterministic, non-secret and not URL-reachable.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | bridge | ✓ | 1.97.0 (pinned) | — |
| wasm32-unknown-unknown target | wasm build | ✓ | installed | — |
| wasm-pack (via `scripts/web-build.ts`) | regenerate wasm | ✓ (Phases 37/38 built with it) | — | — |
| bun | scripts, replay | ✓ | 1.4.2 | — |
| jq | fingerprint extraction | ✓ | 1.7.1 | — |
| Playwright chromium | e2e incl. touch project | ✓ | 1.63.0, multiple chromium builds cached | — |
| Playwright webkit | not needed | (cached, but `browser:install` only installs chromium) | — | Use chromium for `hasTouch` |

**Missing dependencies with no fallback:** none.

## Validation Architecture

(`workflow.nyquist_validation` is `false` in `.planning/config.json`; included because the orchestrator requested it.)

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (lib unit tests in `liquidfun-wasm`); Vitest 5.0.1; Playwright 1.63.0 |
| Config file | `web/vitest` defaults via `package.json`; `web/playwright.config.ts` |
| Quick run command | `cargo test -p liquidfun-wasm --lib puzzle::` · `cd web && bunx vitest run tests/<file>` |
| Full suite command | `cargo test --all-features` · `cd web && bun run test:unit && bunx playwright test e2e/puzzle*.spec.ts` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TOOL-03 | Each invalid reason, reference valid, inspector↔newPuzzle parity | Rust unit | `cargo test -p liquidfun-wasm --lib puzzle::validate` | ❌ Wave 0 |
| TOOL-03 | SAT gap edge cases (touching = overlap, rotated axes) | Rust unit | `cargo test -p liquidfun-wasm --lib puzzle::sat` | ❌ |
| TOOL-03 | Inspector lane decode fails closed | Vitest | `bunx vitest run tests/puzzle-inspector.test.ts` | ❌ |
| TOOL-03 | Rust/TS status code table parity | Rust `include_str!` | `cargo test -p liquidfun-wasm --lib puzzle::validate` | ❌ |
| TOOL-01/02/04 | Editor events (arm, place, select, move, rotate, nudge, remove, clear, stop), budget remaining, edit ignored outside building | Vitest | `bunx vitest run tests/puzzle-editor.test.ts tests/reduce-puzzle.test.ts` | ❌ / ✅ (extend) |
| TOOL-02 | Hit-test slop ≥ 44 px | Vitest | `bunx vitest run tests/puzzle-hit-test.test.ts` | ❌ |
| TOOL-01 | Tray with a fake 2-kind budget | Vitest component | `bunx vitest run tests/puzzle-tray.test.tsx` | ❌ |
| TOOL-06 | Key map | Vitest | `bunx vitest run tests/puzzle-keyboard.test.ts` | ❌ |
| TOOL-07 | Speed cycle and budget multiplier | Vitest | `bunx vitest run tests/puzzle-speed.test.ts` | ❌ |
| STAR-01/02 | `stars.ts`, store OR-merge never lowers | Vitest | `bunx vitest run tests/puzzle-stars.test.ts` | ❌ |
| D-08..D-11 | Peak tracker, brim latch, decided step, equivalence with GoalTracker; frame getters and status checks | Rust unit | `cargo test -p liquidfun-wasm --lib puzzle::peak frame::puzzle` | ❌ |
| D-11 | Frame parse of new scalars, fail closed | Vitest | `bunx vitest run tests/frame-puzzle.test.ts` | ✅ extend |
| D-12 | Fixtures with peak_held/brim_step | Rust + Bun | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` · `just puzzle-replay-wasm` | ✅ extend |
| D-27 | Permuted order gives the same fingerprint (Rust); edit history gives canonical equality (Vitest); UI edit history gives an equal held hash (e2e) | mixed | `cargo test … puzzle::session_tests` · Vitest · `bunx playwright test e2e/puzzle-editor.spec.ts` | ❌ |
| D-28 | Mouse place + rotate + Play gives a 3★ win; touch project same; 1×/2×/4× equal decided step, peak and held hash | e2e | `bunx playwright test e2e/puzzle-editor.spec.ts e2e/puzzle-touch.spec.ts` | ❌ |
| D-26 | Camera fixed across place, Play and Stop; HUD leaves clear | e2e | `bunx playwright test e2e/puzzle.spec.ts` (extend) | ✅ extend |
| D-30 | 25/25 fingerprints; engine untouched | gate | `just playground-scene-spot` + jq diff vs `target/phase39/base.txt`; `git diff --stat <base> -- crates/liquidfun` | ✅ |

### Sampling Rate
- **Per task commit:** the touched Rust module tests plus the touched Vitest files plus `bunx tsc --noEmit` plus `bun scripts/bright-builds-check.ts all`.
- **Per wave merge:** `cargo test -p liquidfun-wasm`, `bun run test:unit`, and after any wasm change `just web-wasm` then the puzzle e2e.
- **Phase gate:** all D-30 items.

### Wave 0 Gaps
- [ ] `target/phase39/base.txt`: 25 fingerprints from phase-base HEAD **before** any Rust change. HEAD `ee177a478` has Rust identical to `589cb0fb9` (empty `git diff --stat`), and `target/phase38/after-final.txt` equals `base.txt`, so it can cross-check.
- [ ] New test files listed above; e2e helpers for world-to-page mapping and debug attributes.

## Security Domain

Security enforcement is on (ASVS L1). There is no network, auth or storage in this phase; the attack surface is local input crossing into wasm and test-only globals.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Rust inspector/`newPuzzle` validate every lane (length before allocation, `MAX_PLACEMENTS` 16, i16/u8 ranges, kind allowlist, `angle < 24`). TS encoders reject non-integers and out-of-range values (`encodePlacementLanes`); inspector output is decoded fail-closed |
| V7 Error Handling | yes | Static bounded messages (`PlacementError::message`) that never echo input; TS failures route to `failScene` |
| V6 Cryptography | no | FNV is a fingerprint, not a security hash |
| V14 Configuration | yes | No new dependencies; the test seam is unreachable from the URL |

### Known Threat Patterns
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed or oversized lanes to `inspectPuzzlePlacements` / `newPuzzle` (huge `kinds`, mismatched `poses`, i32 extremes) | Tampering / DoS | Count and length checks before any allocation; `checked_mul`; `i16::try_from`; no panics (no indexing without proof, no `unwrap`); O(n²) with n ≤ 16 |
| Non-finite or garbage inspector output reaching the overlay or hit-test | Tampering | `decodeInspection` checks lengths, known codes and finite floats, and throws otherwise |
| Window test seam `__liquidfunPuzzleTestPlacements` | Spoofing / Elevation | Read once at route start via `Reflect.get`, validated by `parseSeamPlacement` (fresh objects, integer checks), level id must match; set only by Playwright `addInitScript`, never from URL or hash. Any new seam keeps these properties |
| Debug data attributes (placements, decided step, peak, held hash) | Information disclosure | Deterministic simulation state only; no user data or secrets |
| Star store shape later persisted (Phase 41) | Tampering (prototype pollution) | Use `Map` or `Object.create(null)` with allowlisted keys now, so Phase 41 can validate strictly |
| Keyboard handler | Tampering | Map key to command only; no DOM injection; ignore events from form controls |
| Level id from the hash | Injection | Unchanged Phase 38 allowlist (`PUZZLE_IDS`) and text-only rendering |

## Recommended Plan Breakdown (fine granularity)

| Wave | Plan | Content | Depends on | Files near limits to avoid |
|------|------|---------|------------|----------------------------|
| 1 | 39-01 Rust validator | **Task 1: fingerprint baseline `target/phase39/base.txt`.** `LevelSpec` additions (brim 810, par 1, `no_place: &[]`, budget 2), spec invariants in `validated_tracker` (target < brim ≤ full, par ≤ budget sum), `sat.rs`, `validate.rs`, `PlacementError` overlap variants, `build_puzzle_core` parity, inspector export, `levelInfo` `brimCount`/`parTools`, D-03/D-29 tests, Pitfall 10 test updates | — | `session.rs` 545, `frame.rs` 542, `lib.rs` 311 (put the export in a new module) |
| 1 | 39-02 TS pure core | `editor.ts`, `reduce-puzzle` D-25 events and D-10 decision states, `stars.ts`, `star-store.ts`, `keyboard.ts`, `speed.ts`, `hit-test.ts`, `placement-status.ts`, `canonicalPlacements`, D-27 Vitest #1, using agreed D-11 field names and fakes | field names in this doc | `tests/puzzle-lifecycle.test.ts` 588 (write new test files) |
| 2 | 39-03 Rust peak and frame | `peak.rs`, hooks integration, `PuzzleStatus` + `frame/puzzle.rs` getters and checks, D-27 Rust permuted-order test with `(8,64,23)` and `(30,84,0)` | 39-01 (`spec.rs` brim) | `goal/tests.rs` 451 → new `peak/tests.rs` |
| 3 | 39-04 Replay fixtures | Split `replay.rs` (move the grammar to `replay/fixture.rs`), add `peak_held`/`brim_step` to Rust + TS grammars, record values, Bun replay; existing outcome values unchanged | 39-03 | `replay.rs` 487, `fixture.ts` 379, `tests/puzzle-replay.test.ts` 400 |
| 3 | 39-05 TS wasm boundary | `just web-wasm`; frame D-11 parse (+ all fakes), `levelInfo` parse (`toolKinds`/`toolCounts`/`brimCount`/`parTools`), `puzzle-inspector.ts` decode, Brim tick in `goalTicks` | 39-03, 39-02 | `frame.ts` 346 (optionally split) |
| 4 | 39-06 UI-SPEC-driven components | Tray, selection bar, transport (Play/Stop, speed, Clear), star checklist and card rows, "Filled!" state, `puzzle-editor.css`, tool overlay `render/puzzle-tools.ts` | 39-05, UI-SPEC | `puzzle.css` 420, `PuzzlePanel.tsx` 316, `PuzzleHud.tsx` 139 |
| 4 | 39-07 Shell wiring | Preview/run sessions, Play/Stop/Clear, inspector per edit with rAF coalescing, editor pointer binding (separate handle), keyboard binding, speed in the loop, star recording, debug attributes | 39-05 | `puzzle-lifecycle.ts` 300, `scene-surface.ts` 440 (add only a hook call) |
| 5 | 39-08 e2e | Update `puzzle.spec.ts` D-10 literals (list them), camera stability with place/Play/Stop, `puzzle-editor.spec.ts`, touch project + `puzzle-touch.spec.ts`, fast-forward e2e | 39-06, 39-07 | `puzzle.spec.ts` 444 → new specs |
| 6 | 39-09 Gates | All D-30 checks; SUMMARY records brim, peak, fixture lines and e2e literal changes | all | — |

## Sources

### Primary (HIGH confidence)
- Repository source at HEAD `ee177a478`: `crates/liquidfun-wasm/src/puzzle/{tools,angles,spec,first_pour,goal,hooks,info,builder,replay}.rs`, `frame.rs`, `frame/puzzle.rs`, `lib.rs`, `session.rs`; `web/src/puzzle/*`, `web/src/player/puzzle-{lifecycle,loop,runtime}.ts`, `web/src/physics/{puzzle-session,puzzle-loader,frame,clock}.ts`, `web/src/components/Puzzle{Panel,Hud}.tsx`, `web/src/render/{camera,glass-cup}.ts`, `web/src/input/canvas-pointer.ts`, `web/e2e/puzzle{.spec,-helpers}.ts`, `web/playwright.config.ts`, `web/tests/puzzle-{chrome,shell-wiring}.test.ts`, `scripts/web-build.ts`, `justfile`
- Measured runs: a scratch Bun script against the generated wasm (`web/src/generated/liquidfun-wasm`, built after the last bridge commit `410e2111b`) and a scratch SAT script over `first_pour.rs` literals
- Phase 37/38 CONTEXT, UI-SPEC, and SUMMARY records (37-08, 38-01, 38-09, deferred-items)
- `bun scripts/bright-builds-check.ts --help` (628-line limit)

### Secondary (MEDIUM confidence)
- Playwright emulation docs (`hasTouch`, `isMobile`, device descriptors, `touchscreen.tap`) [CITED: https://playwright.dev/docs/emulation, https://playwright.dev/docs/api/class-touchscreen]

### Tertiary (LOW confidence)
- None relied on.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH. No new dependencies; versions read from manifests.
- Architecture: HIGH. Every integration point was read in source; the preview/run session split follows from `builder.rs`.
- Geometry and brim numbers: HIGH. Measured on the release-exact wasm.
- Phone-landscape layout fit: MEDIUM. Arithmetic from the UI-SPEC numbers, not rendered.
- Pitfalls: HIGH. Each is tied to a specific file or test.

**Research date:** 2026-10-12
**Valid until:** 2026-11-11 (stable in-repo code; re-measure the brim numbers if `first_pour.rs` or the engine changes)
