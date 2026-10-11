# Pitfalls Research

**Domain:** Adding a deterministic "get the water into the cup" puzzle mode (goal zone, place-then-play tools, hand-written levels, progress, sound, GPU water) to the existing liquidfun-rs WASM/SolidJS playground
**Researched:** 2026-10-11
**Confidence:** HIGH for code-grounded integration pitfalls (read from the repo); MEDIUM for browser-platform behavior (MDN plus community sources); MEDIUM for the native-vs-wasm trig divergence (mechanism confirmed, not yet measured in this repo)

Phase labels below use the topic names the roadmap is likely to adopt. Numbering continues after Phase 36:

- **P-Core**: level core, goal counting, win rule and step-count timer (engine/wasm side)
- **P-Tools**: tool model, placement validation and canonical build order
- **P-Levels**: the 5–8 levels plus solvability tests
- **P-UI**: `#/puzzle/<id>` route, level select, HUD, touch, portrait frames and progress storage
- **P-Sound**: opt-in WebAudio
- **P-GPU**: GPU water render mode (closing phase)

---

## Critical Pitfalls

### Pitfall 1: Timer and win check driven by wall-clock time or render frames instead of the step index

**What goes wrong:**
A level is solvable on a fast desktop but fails on a phone, or a recorded solution wins in one run and loses in another. The star or time-of-win depends on the device frame rate.

**Why it happens:**
The playground turns wall time into steps with a hard cap and drops what it cannot run:

- `web/src/physics/clock.ts` clamps debt to `MAX_DELTA_SECONDS = 4 * (1/60)`. `accumulateStepTime` discards anything above that.
- `FRAME_STEP_BUDGET_MS = 12` makes `advance` stop early when the budget is used (`simulation-worker.ts`, `budgeted`).
- `restoreUnrunSteps` puts unrun steps back only up to the same cap.
- Rust enforces `MAX_ADVANCE_STEPS = 4` (`crates/liquidfun-wasm/src/session.rs`).

On a slow phone, ten wall-clock seconds might cover 400 steps instead of 600. Frames are captured after 1–4 steps, so a check run on render frames sees the state at batch boundaries that differ between devices. Hidden tabs also cancel debt (`cancelWorkerFrames`).

**How to avoid:**

- Define every level duration as a step count, for example `time_limit_steps = 600`. Show the timer as `step_index / 60`. On slow devices the timer then runs slower than real time. That is honest and keeps solutions reproducible.
- Evaluate the goal **inside the Rust `advance` loop after each `world.step`**, next to `on_after_step`. Latch a result such as `Won { step }` or `Lost`, and expose it as frame data. Never decide win or loss in TypeScript from captured frames.
- Stars must depend only on that latched state (tools used, fill at a defined step). Never use wall-clock time.
- Never add a "catch-up" mode that raises `MAX_ADVANCE_STEPS` for puzzles. The constant is asserted in tests (`dam_break_timers.rs` checks `MAX_ADVANCE_STEPS: u32 = 4`) and documented as fixed since v1.3.

**Warning signs:**
`performance.now()`, `Date.now()` or `setTimeout` in puzzle timer code. Win logic in a `.ts` file that reads `frame.particlePositions`. A test that needs a real-time wait.

**Phase to address:** P-Core

---

### Pitfall 2: Tool creation order, handle reuse or edits on a live world change the solver order

**What goes wrong:**
The same placements give different results depending on the order the player placed, moved or removed tools, or on how many times they pressed Reset. A recorded solution stops reproducing.

**Why it happens:**
Body, fixture, joint and particle-group creation order sets arena slots, broad-phase proxy order and contact order. LiquidFun results depend on that order, and the project treats stable ordering as a hard constraint. Common mistakes:

- creating tool bodies in UI interaction order
- destroying and re-creating a tool's body on a live world after each drag, so generational handles and proxies are recycled differently
- adding tools to a world that has already stepped

**How to avoid:**

- Treat placements as plain data: an ordered list sorted by a canonical key (tool slot index in the level's toolbox, then a stable id). Keep it out of the engine while editing.
- On Play and on Reset, **rebuild the whole level from scratch**. `SessionCore::from_built` already does this for recreated controls. Then add the base geometry and the tools in canonical order at step 0. Never mutate a live world to apply edits.
- Allow editing only at step 0. "Edit" during or after a run means "Reset to step 0 with the current placements".
- A sponge tool is an elastic particle group. Create it last, in canonical order, because particle-group creation order changes particle indices.

**Warning signs:**
`destroy_body` called from tool editing code. Placement state kept as a `Map` iterated in insertion order. A solution test that passes alone but fails after a "place, remove, place" sequence.

**Phase to address:** P-Tools (the data model and rebuild path), with a regression test in P-Levels

---

### Pitfall 3: Native solution tests pass, but the browser (wasm32) diverges

**What goes wrong:**
The headless Rust test proves a level is solvable, but the same placements fail in the browser, or the reverse.

**Why it happens:**
`Rot::new` in `crates/liquidfun/src/math/transform.rs` calls `angle.sin()` and `angle.cos()` from the standard library. Native builds call the platform libm (Apple libm on the macOS CI host). `wasm32-unknown-unknown` uses the musl-derived Rust `libm` through compiler-builtins. These are separate implementations and are not required to match bit for bit. Every rotating body re-derives its rotation each step. A one-ULP difference grows chaotically in particle simulations over hundreds of steps. No repo evidence shows native and wasm fingerprints match: the v1.4 fingerprints are native only (`scene_spot/fingerprint.rs`, `docs/benchmarks/scene-survey.md`).

**How to avoid:**

- At the start of P-Core, run a spike: one rotating-body scene for N steps, natively and as wasm (Node or Playwright), and compare `end_state_fingerprint`. Record the result in the phase summary.
- If they differ, do not change engine trig for puzzles alone, because that would change the 25 fingerprints. Instead:
  1. Snap tool angles to a coarse grid (for example multiples of τ/24). Initial transforms then come from a fixed table rather than runtime `sin`/`cos` of player input.
  2. Prefer static or kinematic tools for solutions that depend on exact geometry.
  3. Require solutions to win with margin (pitfall 6).
  4. Replay every known solution in the browser path too (Playwright with the synthetic clock), not only natively.
- Treat "wins natively" as necessary, not sufficient.

**Warning signs:**
A level whose known solution wins by fewer than about 5% fill. A solution that depends on a spinning or tumbling dynamic tool. Browser e2e solution replays that fail intermittently across Chromium versions.

**Phase to address:** P-Core (spike), P-Levels (dual-path replay)

---

### Pitfall 4: Puzzle levels added to the scene catalog break the catalog rules, tests and fingerprints

**What goes wrong:**
Adding level ids to `SCENE_IDS` (`web/src/catalog/scenes.ts`) or the Rust `SceneId` enum sets off a chain of failures and unwanted features:

- `assertReadmeSvgPlanCoverage` fails, and the README gallery gains puzzle previews.
- `PORTRAIT_VIEW_BOUNDS` is keyed by every `SceneId` (`portrait-bounds.test.ts`).
- `hud-safe-frame.spec.ts`, `scenes.test.ts` (`UI_SPEC_DESCRIPTIONS: Record<SceneId, string>`), `navigation.test.ts`, demo media and DemoNavigation all iterate the catalog.
- `scene_spot.rs` hard-codes `SURVEY_SCENES: [...; 25]`, and the fingerprint table assumes 25.
- Levels appear in the scene sidebar.
- Shared scene code gains a gravity slider (`withGravitySlider`), which recreates the world.

Also, `web/src/catalog/scene-records.ts` is already 617 lines against the repo's `FILE_LINE_LIMIT = 628` (`scripts/bright-builds-check.ts`), so appending level records there fails `bun scripts/bright-builds-check.ts all`.

**Why it happens:**
"Reuse the canvas, camera, HUD safe frame and scale legend" gets read as "make levels scenes".

**How to avoid:**

- Create a **separate puzzle catalog** (`web/src/puzzle/levels.ts` with its own `LEVEL_IDS` / `LevelId`) and a separate route kind. Reuse the player surface, camera fit, `ScaleLegend` and safe frame by passing `WorldBounds` and portrait bounds through, not by joining `SceneId`.
- On the Rust side, add a parallel builder (for example a `puzzle` module and `LevelId`) that returns the same `BuiltScene`. Leave the 25-arm `SceneId`, `SURVEY_SCENES` and `parse_scene_id` untouched.
- Copy the repo's catalog rules to levels deliberately:
  - each level needs a portrait frame, with a `Record<LevelId, WorldBounds>` and a fill test
  - the scale legend is always visible and `maybePixelsPerMeter` is published
  - new chrome uses `data-hud-obstacle` on leaf elements
- Do not add levels to the README plans unless the owner asks for that.
- After any change under `crates/liquidfun` or shared wasm scene helpers (`attach_basin_fixture`, `escape.rs`), run `just playground-scene-spot` and confirm 25/25 fingerprints match the Phase 35 baseline.

**Warning signs:**
Diffs touching `scenes.ts` `SCENE_IDS`, `scene_spot.rs`, or `web/scripts/readme-svg/plans.ts` in a puzzle phase. A file-length check failure on `scene-records.ts`.

**Phase to address:** P-Core (Rust boundary), P-UI (TS catalog and route)

---

### Pitfall 5: Goal-zone counting with an undefined boundary, splash counting and a drifting denominator

**What goes wrong:**
Fill % flickers, jumps above 100%, or counts water flying over the cup. Water that was destroyed silently lowers the total so fill looks higher. The sponge's own particles count as water. Viscous films on the outside of the cup wall count. Wins happen on a splash.

**Why it happens:**

- Counting with a particle radius against the cup's outer rectangle includes particles straddling the walls and those above the rim.
- `evict_escaped_particles` (`session/escape.rs`) destroys particles every step once they fall about 12 m along gravity or leave the play area. Particle lifetimes and destroy-in-shape (Soup family) also remove particles. "Percent of live particles" then drifts.
- `query_particle_system_aabb` goes through proxies that are refreshed in the step. Querying after a mid-step mutation reads stale proxies.

**How to avoid:**

- Define the goal region once in Rust as the **cup interior polygon**: inside the inner wall faces and below the rim. Count by **particle centre only**. Write a pure function `count_in_goal(positions, flags/groups, region) -> u32` with Arrange/Act/Assert tests for:
  - a particle on the boundary
  - a particle just above the rim
  - a particle outside the wall
  - an empty system
- Iterate positions linearly; a few thousand particles cost almost nothing. Do this after the step, so the count does not depend on proxy freshness.
- Count only goal material. Filter by particle group or flags so sponge and powder-as-obstacle particles are excluded unless a level says otherwise.
- Fill % = `in_goal / level.full_count`, where `full_count` is an **authored constant per level**. Calibrate it once from a reference run with the cup filled to the brim, and assert `full_count <= source particle count`. It never changes with eviction. Clamp the display to 100%.
- Splash rule: win only when `fill >= target` holds for M consecutive steps (for example 30 steps, half a second) before the time limit. Unit-test this as a pure state machine (`GoalTracker::observe(count, step) -> Outcome`).
- If a level's cup can move (buoyancy), transform the region by the cup body's transform each step.

**Warning signs:**
Fill above 100%. Fill that rises when particles fall off screen. A win flash on the first splash. Different counts in TS (drawing) and Rust (logic).

**Phase to address:** P-Core

---

### Pitfall 6: Levels that are unsolvable, trivially solvable or knife-edge, with no automated proof

**What goes wrong:**
A level ships that no one can solve, that wins with zero tools, or whose intended solution works only at one exact position. Later engine fixes (even fingerprint-preserving refactors elsewhere, or real behavior fixes) silently break solvability.

**Why it happens:**
Levels are tuned by hand in the browser. Nobody records the solution as data, and the level is never re-run after an engine change.

**How to avoid:** for every level, check in a `LevelSolution` fixture (canonical placements in snapped world units) and add native Rust tests in the existing `liquidfun-wasm` test binary that:

1. replay the 1-star known solution for `time_limit_steps` and assert `Won` with a margin, for example peak sustained fill at least target + 5%
2. replay a 3-star solution and assert three stars (proves the top rating is reachable)
3. run the **zero-tool baseline** and assert `Lost` (proves it is not trivial). Level 1 may be the documented exception if it teaches pressing Play.
4. run a **robustness sweep**: nudge each solution placement by ±1 grid step and require most variants to still win. This rejects knife-edge designs.
5. assert every solution placement passes the same validator the UI uses (pitfall 7)

Also:

- Add one Playwright spec that loads `#/puzzle/<id>` under `__liquidfunSyntheticAnimationClock`, applies the fixture through the real UI or a test hook, and asserts the win banner. This covers the JS-to-WASM placement path and pitfall 3.
- Keep these tests in existing test binaries. On this Mac, each new test binary waits about 2 minutes on a security scan (memory: macOS launch stalls).

**Warning signs:**
A level without a `*_SOLUTION` constant. A solution test with no margin assertion. A level whose zero-tool run reaches more than 50% of target.

**Phase to address:** P-Levels (with the test harness built at the end of P-Core so P-Tools can use it)

---

### Pitfall 7: Placement overlap with walls, tools, the water body or the goal at Play start causes explosions, tunneling or cheats

**What goes wrong:**

- A ramp placed inside the starting water block shoves particles out at huge speed. They leak through walls or get evicted.
- Two dynamic tools overlapping at step 0 fly apart.
- A static tool overlapping a static wall has no collision response, but it seals an exit or forms a funnel through geometry.
- A tool laid across the cup mouth acts as a lid or a "pre-filled" pocket.
- Gaps narrower than one particle diameter between a tool and a wall jam or leak unpredictably.
- Thin tools (edges, slivers) let fast particles squeeze through at corners. The port has a swept moving-fixture query, but corners and chain gaps are still weak spots.

**Why it happens:**
Free placement is validated in screen space, with f64 values in TS, while the engine uses f32. Or it is not validated at all.

**How to avoid:**

- Snap positions to a world grid (for example 0.05 m) and angles to τ/24. Store them as integers. One snapped value then means the same thing in TS, Rust, storage and fixtures. This also covers the f32-versus-f64 placement precision trap.
- Run one validator, implemented in Rust as a pure function and exposed to TS. It rejects:
  - overlaps with level geometry inflated by one particle diameter
  - overlaps with other tools
  - overlaps with authored **no-place zones**: the water source region, the cup interior and mouth, and level-specific exclusions
  - placements outside the playable bounds
- Draw invalid ghosts in red and refuse Play while any placement is invalid. Do not try to resolve overlaps.
- Set a minimum tool thickness of at least 3 particle diameters. Use polygons, not edges, for tools.
- Pins: a pin must land on a pinnable level body or tool. Otherwise reject it, rather than creating a joint to nothing.

**Warning signs:**
Max particle speed in `FrameDiagnostics` spikes at step 1. Many particles evicted in the first 30 steps. Validation code in TypeScript that duplicates Rust geometry.

**Phase to address:** P-Tools

---

### Pitfall 8: In-play inputs (tilt, gravity slider, scene pointer, valve taps) break reproducibility or allow cheats

**What goes wrong:**
A known solution fails because the phone was tilted. Or the player wins by shaking the phone, poking water with the scene pointer, or changing gravity.

**Why it happens:**

- `reapplyStoredTiltGravity` (`web/src/input/tilt-binding.ts`) re-applies live accelerometer gravity to **every** new session.
- The gravity slider control recreates the world.
- `forwardScenePointer` sends pointer actions while `playing`, at wall-clock-dependent step boundaries.
- A player-toggled valve during play has the same timing problem.

**How to avoid:**

- In puzzle sessions, disable the tilt binding (or keep it but call `restoreAuthoredGravity`), and offer no gravity or preset controls.
- Route canvas pointer input in puzzle mode only to the placement editor, and only while paused at step 0.
- Make valves deterministic. Either it is a **scheduled tool** whose open step is set while paused ("opens at 2.0 s"), or it is passive (a float, a pressure gate, or a timed kinematic gate). If live toggling is a must-have, record it as `(step_index, action)` and apply it at the next step boundary in Rust. Store it in the solution fixture. This is extra scope.

**Warning signs:**
The tilt indicator is visible on a puzzle route. A `pointer` worker op is sent while a level is playing. A valve that works by tapping during play with no step stamp.

**Phase to address:** P-Tools (valve design), P-UI (input routing)

---

### Pitfall 9: Touch placement that is unusable on phones

**What goes wrong:**

- A tool is 0.4 m long, which is about 25 CSS px in a portrait frame, so the rotate handle is untappable.
- The finger hides the thing being placed.
- A drag starting in the DOM toolbox never reaches the canvas.
- A long-press opens the iOS callout or text selection.
- Edge drags trigger Safari back-swipe.
- Pan mode and placement fight over the same gesture.
- Two-finger rotate does nothing.

**Why it happens:**

- `web/src/input/pointer.ts` tracks **one** `maybePointerId`. Multi-touch gestures are not modelled.
- `attachCanvasPointer` captures the pointer on the canvas only. Drags that start on the toolbox hold capture there.
- The canvas viewport sets `touch-action: none` (`player-viewport.css`), so pinch-zoom on the canvas is already suppressed.
- Zoom and pan are toggled tools (`panEnabled`).

**How to avoid:**

- Hit-test in CSS pixels, with at least a 44 px target around each tool, independent of world size.
- Use tap-to-select, then drag anywhere to move. Put Rotate (±τ/24 steps) and Remove on large buttons in a selection bar instead of tiny handles.
- Draw the dragged ghost offset above the finger.
- Add tools by tapping a toolbox item, which places it at a free default spot. Avoid cross-element drag-and-drop.
- Keep the single-pointer reducer. Do not add pinch-rotate.
- Disable pan mode while a tool is selected. Add `-webkit-touch-callout: none` and `user-select: none` to the stage.
- Give keyboard parity: arrows nudge one grid step, R and Shift+R rotate, Delete removes. The playground already keeps labeled keyboard controls, and e2e tests are easier to write with them.
- Add a Playwright project with `hasTouch`/`isMobile` (only Desktop Chrome exists in `playwright.config.ts`).

**Warning signs:**
Hit areas computed from world size. No e2e test with touch emulation. QA only on desktop.

**Phase to address:** P-UI

---

### Pitfall 10: Puzzle HUD chrome that refits the camera between editing and playing

**What goes wrong:**
Pressing Play hides the toolbox, the safe frame refits, and the whole world jumps or rescales. The timer text width changes from "9.9" to "10.0" and the camera jitters. A full-width toolbox marked as an obstacle shrinks the world to a sliver, or the fallback (`SAFE_FRAME_FALLBACK_RATIO = 0.5`) ignores it and the tray covers the cup.

**Why it happens:**
`createHudObstacles` (`web/src/player/hud-obstacles.ts`) refits the camera whenever a marked element appears, disappears or resizes. The repo rule requires `data-hud-obstacle` on leaf elements with camera-independent sizes.

**How to avoid:**

- Keep the same set of obstacle elements in the edit and play states. Swap contents (toolbox to fill meter) inside a fixed-size slot. Fix widths with `tabular-nums` and minimum widths for the timer and fill readouts.
- Mark each toolbox button, not the tray.
- Size portrait frames per level to the cup, the source and the full tool area. Include the walls, per the existing portrait rule.
- Extend a `hud-safe-frame`-style e2e to every level.

**Warning signs:**
`camera.scale` changes on Play in an e2e assertion. The safe frame `usedFallback` is true on a 390 by 844 viewport.

**Phase to address:** P-UI

---

### Pitfall 11: Progress storage that crashes or locks players out when storage is blocked or wiped

**What goes wrong:**
The app throws on load in Safari with storage disabled, in sandboxed iframes, or with cookies blocked. Reading the `window.localStorage` getter itself can throw `SecurityError`, and `setItem` can throw `QuotaExceededError`. With progress unavailable, levels 2–8 stay locked forever. Safari's 7-day inactivity cap on script-writable storage wipes progress. Tampered or old-schema JSON crashes parsing.

**How to avoid:**

- Reuse the existing pattern (`hud-sliders-preference.ts`, `appearance`). Pass a provider `() => window.localStorage` and call it **inside** `try`. Unavailable storage falls back to in-memory state.
- Use one versioned key (`liquidfun.puzzle-progress.v1`). Validate strictly: allowlisted level ids, stars as an integer from 0 to 3, unknown fields ignored. Build objects with `Object.create(null)` or a `Map`, never by spreading parsed JSON into config.
- Use soft unlocks. **Deep links to any level always work.** If storage is unavailable, unlock every level, so "fully playable when storage is blocked" holds literally.
- Saved placements (optional) are keyed by level **revision**. Drop them if the level geometry changes.
- Say in the UI that progress is per browser and may be cleared.
- Unit-test with throwing getters, a throwing `setItem`, corrupt JSON and a future version.

**Warning signs:**
`window.localStorage.getItem` called outside `try`. Locked-level gating on direct routes. No test for a throwing provider.

**Phase to address:** P-UI

---

### Pitfall 12: WebAudio that never plays, plays too much or leaks

**What goes wrong:**

- Sound never starts because the `AudioContext` was created before a user gesture and stays `suspended`.
- On iOS the ring/silent switch mutes Web Audio, and testers file "sound broken" bugs.
- Hundreds of particle contacts per frame each start a voice, causing clipping, CPU spikes and GC churn.
- A new `AudioContext` per level hits browser limits and leaks.
- Sound keeps playing in hidden tabs.
- Audio timing is tied to how events are batched from frames.

**Why it happens:**
Browsers apply autoplay rules to Web Audio: `start()` outside a user input event is subject to autoplay blocking, and Firefox requires sticky activation by default. Frame data only carries aggregates (`body_contact_count`, `max_speed`), so naive per-particle sounds need new per-particle event data.

**How to avoid:**

- Use one lazily created `AudioContext` per page. Create and `resume()` it inside the click handler of the "Sound on" toggle; muted is the default, per SEED-005.
- Derive splash and impact cues from per-frame aggregates added to `FrameDiagnostics`, for example the change in goal count or the count of body-contact events. Keep these additions read-only so fingerprints are unaffected.
- Rate-limit each cue class (for example at most one splash per 80 ms), cap concurrent voices (about 6) with voice stealing, and route through one master gain plus a compressor.
- Suspend on `visibilitychange` hidden and on navigation away. Audio must never feed back into the simulation.
- Test with a fake `AudioContext` in unit tests (cue mapping and rate limiting). Cover only the toggle in e2e.

**Warning signs:**
`new AudioContext()` at module load. A `createOscillator` call per particle. No voice cap. A puzzle phase trying to mix sound and GPU work.

**Phase to address:** P-Sound

---

### Pitfall 13: GPU water mode that fails on mobile GPUs, loses context or breaks fallback and exports

**What goes wrong:**

- The water is black or blank on iPhones.
- After the tab is backgrounded, the context is lost and nothing redraws while paused, which is the editing state where puzzle players spend most of their time.
- Refraction needs the 2D layer (walls, tools) as a texture, and uploading it each frame stalls mobile GPUs.
- A stored render-mode preference for an unsupported mode leaves users with nothing.
- README and demo exports change.
- Creating a fresh WebGL context per level eventually exhausts the browser's context limit, and old contexts get dropped.

**Why it happens:**

- The existing shaded-blob path (`web/src/render/webgl-particles.ts`) requires `EXT_color_buffer_float`. When it is missing, it sets `shadersAvailable = false` and falls back to canvas.
- Its shaders use `precision mediump float`, which is fp16 on many mobile GPUs. That is fine for blobs but too coarse for refraction offsets and world-space highlights.
- It handles `webglcontextlost`/`webglcontextrestored` by resetting resources. Nothing repaints a held paused frame on restore.

**How to avoid:**

- Build GPU water as a **new optional `RenderMode`** on the existing separate particle canvas, the `maybeParticleCanvas` layer in `present-frame.ts`. The main canvas already holds a 2D context, and one canvas cannot hold both.
- Keep `shaded-blob` as the default and the canvas modes as the fallback. Add the new token to `maybeParseRenderMode` so a stored value parses, but fall back at draw time when it is unsupported.
- Use `highp` in fragment shaders. WebGL2 (GLSL ES 3.00) guarantees it.
- Probe float render targets and fall back to half-float or RGBA8 encoding. Treat iOS support for `EXT_color_buffer_float` as unverified until tested on a device.
- Refract a static background texture uploaded **once** per level build or on resize, not every frame.
- On `webglcontextrestored`, call the existing `paintHeldFrame`.
- Release contexts on dispose (`WEBGL_lose_context`) and reuse one context across levels.
- Exclude the GPU mode from README SVG/WebP export paths.
- Cap work on phones (field resolution scaled with `cappedDevicePixelRatio`).

**Warning signs:**
Tested only on a desktop Chromium. No test for a forced `loseContext()`. The GPU mode set as the default. Export scripts importing GPU modules.

**Phase to address:** P-GPU (closing phase; it must not block earlier phases)

---

### Pitfall 14: Scope creep toward a level editor or sandbox

**What goes wrong:**
"Tools you can place" grows into "walls you can draw", then a level JSON format, then share links, then an in-app editor. That is SEED-002, which PROJECT.md marks out of scope. Milestones overrun. v1.4 shows this risk is real: an overgrown harness was shelved (memory: prefer lightweight process).

**How to avoid:**

- Write levels as Rust code with typed constants, in the style of the existing `scene/*.rs` builders, plus a small TS record for title, hint, portrait bounds and toolbox labels.
- Each level gets a fixed toolbox (kinds and counts). No free-form geometry. No URL-encoded placements. No in-app authoring UI.
- A debug-only "print placements as a Rust fixture" button is acceptable. It speeds up recording solutions and does not count as an editor.
- Any request for shareable solutions or custom levels goes to SEED-002.

**Warning signs:**
A JSON schema or loader for levels. Tool kinds beyond those the levels need. A "custom level" route.

**Phase to address:** all phases; enforce in roadmap success criteria

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Count goal particles in TS from render frames | No wasm API change | Frame-rate-dependent wins, duplicated geometry, no headless proof | Never; display-only meters may read Rust's count |
| Duplicate goal and no-place geometry in TS and Rust | Fast drawing | Drift: visible cup outline differs from the counted region | Only if a test asserts TS and Rust geometry are equal; better to export geometry from Rust in a level-info call |
| Add levels to `SCENE_IDS` | Reuses the whole scene pipeline at once | README, survey, portrait and nav tests and fingerprint lists all churn; levels appear in the scene list | Never |
| Wall-clock timer | Matches the "seconds" mental model | Solutions not reproducible; harder on slow phones | Never |
| Hand-tune levels with no recorded solution | Faster level authoring | Unsolvable levels after any engine change | Never past a draft; every merged level needs a fixture |
| Free-float (unsnapped) placements | Smoother feel | f32/f64 validation disagreement; fragile fixtures | Visual drag may be continuous, but the committed placement snaps on drop |
| Run every level on the main thread (direct backend) | Simpler; matches the e2e synthetic-clock path | Jank while dragging on phones for heavy levels | Acceptable if each level stays under about 3k particles; otherwise see the next row |
| Put heavy levels on the worker backend | Smooth UI | `scene-lifecycle.ts` bypasses the worker under `__liquidfunSyntheticAnimationClock`, so e2e tests exercise a different backend than production | Acceptable only with one non-synthetic worker e2e for that level |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Hash routing (`web/src/routing/hash.ts`) | Extending `maybeParseSceneRoute` so `#/puzzle/x` stops returning `unknown`, which changes the scene fallback (`FallbackPanel` "Scene not found") or empty-hash normalization | Add a separate `puzzle` route kind with its own allowlist parser; keep the existing scene tests unchanged and add new ones |
| `SessionCore` (`session.rs`) | Adding puzzle fields that change `advance()` for all scenes (extra hooks, a different eviction order) | Puzzle evaluation goes in a hook or wrapper called after `on_after_step`; existing scenes get a no-op; verify 25/25 fingerprints |
| `evict_escaped_particles` | Assuming the particle count is constant | The goal denominator is authored; eviction only lowers what can be delivered |
| Frame protocol (`frame.rs`, `worker-messages.ts`, `frame.ts`) | Changing lane layouts or validation limits for puzzle data | Add new optional scalars (goal count, outcome, step) with strict parsing; keep existing lanes byte-identical |
| Reset semantics (`reset-honesty.spec.ts`) | Reusing "Reset scene", which remounts `DEFAULT_PRESET_VALUES`, so it clears placements | Separate honest labels: "Reset" (water back, tools kept) and "Clear tools"; add new reset-honesty assertions |
| Tilt and gravity controls | Shared player chrome shows the accelerometer and gravity slider on levels | Puzzle chrome omits them; the session restores authored gravity |
| Generated wasm bindings | Editing `web/src/generated/liquidfun-wasm` by hand | Regenerate through the existing build; add exports via `lib.rs` wrappers |
| `just` / CI | New puzzle test binaries and Playwright projects that slow the single macOS CI job | Keep Rust tests in the `liquidfun-wasm` lib tests; keep puzzle e2e small; mark heavy sweeps as manual |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Rebuilding the world on every drag move | Drag lag, GC churn | Ghosts drawn in TS; Rust validates on drop; rebuild only on Play or Reset | Any level with more than about 1k particles on phones |
| Too many particles per level | Steps dropped, so the timer runs slowly in wall time; the level feels sluggish | Budget each level (for example 1.5–3k particles) and check it with the existing `scene-spot` timing approach | Phones below 60 steps/s |
| Robustness sweeps in default tests | Slow local and CI runs | Sweep a small neighborhood by default; run the full sweep through a manual `just` recipe | More than about 8 levels × 3 runs × 600 steps |
| Per-particle audio events across the boundary | Large frames, main-thread stalls | Aggregate cues in Rust per step | Thousands of contacts |
| Per-frame `texImage2D` of the 2D canvas for refraction | Mobile frame drops | Static background texture uploaded once | Any iOS device |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Trusting parsed localStorage JSON (merging into objects) | Prototype pollution or crashes from tampered or stale data | Strict validation, allowlisted keys, `Map` or null-prototype objects |
| Unvalidated level id from the hash, used in lookups or `innerHTML` | Injection or a crash | Allowlist `LEVEL_IDS`, like `SCENE_IDS`; render with Solid text bindings only |
| Placements from storage passed to wasm unchecked | Non-finite values reach the engine (`InvalidPointer`-style failures) | Integer grid units, range-checked in TS and again in Rust |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| The timer disagrees with real seconds on slow devices without explanation | "The timer is broken" | Label it as simulation time, or show a progress bar instead of seconds |
| A loss with no reason given | Frustration | Show the final fill against the target and where the water went (for example "38% fell off screen") |
| Stars defined vaguely ("fewer tools or higher fill") | Players cannot tell how to improve | Show explicit criteria on the level card: 1★ target reached, 2★ at most N tools, 3★ fill of at least X% |
| Levels locked behind progress when storage fails | Blocked players | Soft unlock; deep links always work |
| A teaching level that introduces two ideas at once | Players do not learn the mechanic | One new idea per level, as in the target list; the finale combines them |
| A win that interrupts with a modal immediately | Players miss watching the water | Show a non-blocking banner; let the run finish and then show Retry / Next |
| Sound or GPU modes on by default | Surprise noise; battery drain | Both opt-in; GPU mode offered, not forced |

## "Looks Done But Isn't" Checklist

- [ ] **Goal zone:** often counts by bounding box with radius. Verify centre-in-interior unit tests (boundary, rim, outside wall) and that the sponge or other groups are excluded.
- [ ] **Timer:** often wall-clock. Verify no `performance.now`/`Date.now` in the timer or win path, and that the outcome is latched in Rust by step index.
- [ ] **Determinism:** often only replay-tested once. Verify canonical-order rebuild after a "place, remove, re-place, reset ×3" sequence yields an identical fingerprint.
- [ ] **Native vs wasm:** often assumed equal. Verify the P-Core spike result is recorded and a Playwright replay of at least one solution wins.
- [ ] **Each level:** often lacks a proof. Verify a 1★ and a 3★ fixture win with margin, the zero-tool run loses, and the nudge sweep passes.
- [ ] **Placement:** often validated only visually. Verify the shared Rust validator rejects overlap with water, cup, walls and tools, and that Play is disabled while anything is invalid.
- [ ] **Existing scenes:** often silently changed. Verify `just playground-scene-spot` shows 25/25 fingerprints equal, existing web unit and e2e tests pass, and the README plans are unchanged.
- [ ] **Routing:** verify `#/scene/<id>`, empty hash and unknown-hash behavior are unchanged, and `#/puzzle/<unknown>` shows a useful fallback.
- [ ] **Phone:** verify a portrait frame for every level fills a 390 by 844 viewport, the camera does not move on Play, a touch e2e passes, and targets are at least 44 px.
- [ ] **Storage:** verify a throwing getter, a throwing `setItem`, corrupt JSON and a future version are each tested, and all levels are playable with storage blocked.
- [ ] **Sound:** verify it is muted by default, the context starts only from the toggle gesture, voices are capped, it suspends when hidden, and it works on iOS with the ring switch on.
- [ ] **GPU water:** verify a forced `loseContext()` then restore repaints the paused frame, the canvas fallback works when float targets are missing, and exports are unaffected.
- [ ] **File sizes:** verify `bun scripts/bright-builds-check.ts all` passes (628-line limit; `scene-records.ts` already has 617 lines).

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Wall-clock timer shipped | MEDIUM | Move evaluation into Rust `advance`; convert limits to steps; re-record solutions |
| Order-dependent tool creation | MEDIUM | Introduce the canonical placement list and full rebuild; re-run all solution fixtures |
| Native/wasm divergence found late | MEDIUM–HIGH | Snap angles to a table; re-tune knife-edge levels for margin; add browser replays. Changing engine trig would change all 25 fingerprints and needs a deliberate, owner-approved re-baseline |
| Levels added to `SCENE_IDS` | MEDIUM | Split into `LEVEL_IDS`; revert README, survey and portrait additions |
| Engine change broke a fingerprint | LOW–MEDIUM | Bisect with `just playground-scene-spot`; make puzzle logic read-only or move it behind the puzzle hook |
| Level found unsolvable after release | LOW | The fixture test fails first; re-tune the level constants and bump the level revision so stale saved placements are dropped |
| Storage crash in the field | LOW | Wrap access in a provider with try; ship the in-memory fallback |
| GPU mode broken on a device class | LOW | Capability probe sends it to the canvas fallback; hide the mode option where the probe fails |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Wall-clock / frame-based timer and win | P-Core | Rust unit tests for `GoalTracker`; grep shows no wall-clock time in the puzzle path; a synthetic-clock e2e wins at the same step on every run |
| 2. Creation order and live-world edits | P-Tools | Fingerprint equal after a permuted edit history and repeated resets |
| 3. Native vs wasm divergence | P-Core spike, P-Levels | Recorded spike result; Playwright solution replay passes |
| 4. Catalog and fingerprint breakage | P-Core, P-UI | 25/25 fingerprints; existing tests untouched and green; README plans unchanged; file-length check passes |
| 5. Goal counting edge cases | P-Core | Boundary, rim, eviction and group-filter unit tests; fill clamped |
| 6. Unsolvable or trivial levels | P-Levels (harness end of P-Core) | Per-level 1★ and 3★ win, zero-tool loss, nudge sweep |
| 7. Placement overlap | P-Tools | Validator unit tests; no step-1 speed spike in solution runs |
| 8. In-play inputs and valve timing | P-Tools, P-UI | Tilt and gravity absent on puzzle routes; valve state is step-scheduled or recorded |
| 9. Touch UX | P-UI | Touch-emulated Playwright placement; 44 px target audit; keyboard parity tests |
| 10. HUD refits camera | P-UI | e2e asserts camera scale is unchanged across Play; per-level portrait-fill test |
| 11. Storage failures | P-UI | Throwing-provider and corrupt-data unit tests; blocked-storage e2e |
| 12. WebAudio | P-Sound | Fake-context unit tests for rate limiting and voice cap; toggle e2e; manual iOS check |
| 13. GPU context, precision, fallback | P-GPU | Forced context-loss test; fallback test with the extension missing; exports diffed unchanged |
| 14. Editor scope creep | All (roadmap criteria) | No level schema or loader; toolbox fixed per level |

## Sources

- Repository code read for this research (HIGH):
  - `web/src/physics/clock.ts`, `web/src/physics/simulation-worker.ts`, `web/src/player/worker-frame-loop.ts`, `web/src/player/frame-loop.ts`, `web/src/player/scene-lifecycle.ts`
  - `crates/liquidfun-wasm/src/session.rs`, `session/escape.rs`, `scene.rs`, `scene_spot.rs`, `scene_spot/fingerprint.rs`
  - `crates/liquidfun/src/math/transform.rs` (std `sin`/`cos` in `Rot::new`), `crates/liquidfun/src/particle/query.rs`, `crates/liquidfun/src/world/object/lifecycle.rs`
  - `web/src/routing/hash.ts`, `web/src/catalog/scenes.ts`, `web/src/catalog/portrait-bounds.ts`, `web/tests/scenes.test.ts`, `web/tests/portrait-bounds.test.ts`
  - `web/src/input/canvas-pointer.ts`, `web/src/input/pointer.ts`, `web/src/input/tilt-binding.ts`, `web/src/styles/player-viewport.css`
  - `web/src/render/safe-frame.ts`, `web/src/player/hud-obstacles.ts`, `web/src/player/hud-sliders-preference.ts`
  - `web/src/render/webgl-particles.ts`, `web/src/render/present-frame.ts`, `web/src/render/mode.ts`
  - `web/playwright.config.ts`, `scripts/bright-builds-check.ts`
  - `.planning/PROJECT.md`, `.planning/seeds/SEED-001/003/005`, `CLAUDE.md` repo rules
- [MDN: Autoplay guide for media and Web Audio APIs](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Autoplay) — Web Audio `start()` outside user input is subject to autoplay rules; Firefox sticky activation; `navigator.getAutoplayPolicy("audiocontext")` (MEDIUM–HIGH)
- [rust-lang/compiler-builtins PR #248](https://github.com/rust-lang/compiler-builtins/pull/248) and [libm crate](https://github.com/eldruin/libm) — wasm32-unknown-unknown math comes from a musl-derived Rust libm, separate from the native platform libm (MEDIUM: mechanism confirmed; bit divergence in this repo not yet measured)
- [Search Engine Land: Safari's 7-day cap on script-writable storage](https://searchengineland.com/what-safaris-7-day-cap-on-script-writeable-storage-means-for-pwa-developers-332519), [M. Tsai: Safari 13.1 7-day storage](https://mjtsai.com/blog/2020/03/26/safari-13-1-third-party-cookie-blocking-and-7-day-script-writeable-storage/), [WebKit bug 237350](https://bugs.webkit.org/show_bug.cgi?id=237350) — localStorage may be wiped after 7 days of Safari use without interaction (MEDIUM; current iOS behavior not re-verified)
- Training-data knowledge, flagged for on-device validation (LOW–MEDIUM): iOS ring/silent switch muting Web Audio; mediump being fp16 on mobile GPUs; iOS support for `EXT_color_buffer_float`; per-page WebGL context limits

---
*Pitfalls research for: v1.5 Puzzle Mode on the liquidfun-rs web playground*
*Researched: 2026-10-11*
