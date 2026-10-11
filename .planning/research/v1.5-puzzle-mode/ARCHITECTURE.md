# Architecture Research

**Domain:** Puzzle mode ("get the water into the cup") added to the existing liquidfun-rs WASM playground
**Researched:** 2026-10-11
**Confidence:** HIGH for integration points (read from the current source), MEDIUM for performance and feel assumptions (not measured yet)

All file paths are relative to the repository root `/Users/peterryszkiewicz/Repos/liquidfun-rs`. This milestone needs **no change to the engine crate `crates/liquidfun`**. Everything new goes in the private bridge `crates/liquidfun-wasm` and in `web/`.

## Existing Architecture (what the code does today)

Facts confirmed by reading the code:

| Fact | Where | Why it matters for puzzles |
| --- | --- | --- |
| Scenes are an allowlisted `SceneId` enum, each built by a Rust `build(presets)` that returns `BuiltScene { world, particle_system, particle_radius, hooks: Box<dyn SceneHooks> }`. | `crates/liquidfun-wasm/src/scene.rs` | Puzzle levels can reuse `BuiltScene` and `SceneHooks` and get stepping, eviction, capture and pointer plumbing for free. |
| `SessionCore` owns the world, steps 1–4 fixed 1/60 s steps per call (`MAX_ADVANCE_STEPS = 4`), calls `hooks.on_advance`, then `evict_escaped_particles`, then `world.step`, then `hooks.on_after_step(transitions)`. It rebuilds on a construction control (`ControlEffect::Recreated`). | `crates/liquidfun-wasm/src/session.rs` | Goal counting fits in `on_after_step` (once per fixed step). Construction-from-input already exists as a pattern (presets cause a rebuild). |
| `SessionCore` is keyed by `SceneId`, which `escape.rs` (eviction limits) and `particle_iterations(id)` also read. | `session.rs`, `session/escape.rs`, `scene.rs` | A puzzle session needs a small "session source" generalization, not a `SceneId` variant that pretends to be a catalog scene. |
| `ProofSession` (wasm-bindgen) exposes `new(sceneId)`, `applyControl`, `applyAction`, `pointerAction`, `setGravity`, `advance`, `captureFrame`. | `crates/liquidfun-wasm/src/lib.rs` | Add one constructor (`newPuzzle`) and a few frame getters. The TS session adapter and frame loop stay the same. |
| Frames are copied, validated typed lanes plus scalar diagnostics (`maxSpeed`, `stuckCandidateCount`, `bodyContactCount`), with circle labels. Caps: 16,384 particles, 512 segments, 48 circles. | `crates/liquidfun-wasm/src/frame.rs`, `web/src/physics/frame.ts` | Puzzle status (count, outcome, step) and audio signals are new scalar fields. Foam for GPU water needs a new per-particle lane. |
| **Only Tesla Valve runs in the Web Worker.** `simulation-worker.ts` hard-codes `loadSceneSession("tesla-valve")`, and `scene-lifecycle.ts` picks the worker only for `id === "tesla-valve"`. Every other scene uses the synchronous direct `SceneSession` on the main thread. | `web/src/physics/simulation-worker.ts`, `web/src/player/scene-lifecycle.ts` | Puzzles should use the **direct** backend. Tool placements cross the JS-to-WASM boundary, not the worker boundary. |
| `SCENE_IDS` in `web/src/catalog/scenes.ts` is parsed by Rust tests and xtask (`crates/liquidfun-wasm/src/scene_spot/tests.rs`, `tools/xtask/src/playground/spot.rs`). It also keys `PORTRAIT_VIEW_BOUNDS`, the README SVG plans (`assertReadmeSvgPlanCoverage`), and `web/e2e/hud-safe-frame.spec.ts`. | listed files | **Puzzle levels must not be added to `SCENE_IDS`.** Doing so would break the 25-scene survey, README gallery coverage and portrait tests, and would make puzzles look like catalog scenes. |
| Routing: `SceneRoute = scene | empty | unknown`. Anything other than `#/scene/<id>` is `unknown`. `bindNavigation`'s effect and `handleHashChange` call `abandonScene()` whenever the route is not a ready scene. | `web/src/routing/hash.ts`, `web/src/player/playground-session.ts`, `web/src/player/scene-lifecycle.ts` | A `#/puzzle/<id>` route would be torn down today. The lifecycle needs a "playable target" abstraction. |
| Render modes: `RenderMode` union with `particleSurface()` mapping. `shaded-blob` already uses a second WebGL2 canvas (`webgl-particles.ts`). `presentSceneFrame` falls back to the canvas path when `drawShadedBlob` returns `false`. | `web/src/render/mode.ts`, `web/src/render/present-frame.ts`, `web/src/render/webgl-particles.ts` | GPU water is one more `RenderMode` and surface, using the same fallback rule. |
| Preferences use a `loadX` / `persistX` pair with try/catch around `localStorage` and versioned keys (`liquidfun.render-mode.v1`). | `web/src/render/mode.ts`, `web/src/player/appearance.ts` | Puzzle progress copies this pattern, so the game stays playable when storage is blocked. |
| Kinematic bodies driven by `set_body_linear_velocity` in `on_advance` (Hydraulic Fountain piston). Revolute motors are live-adjustable (`set_revolute_motor_speed`). `destroy_body` and `set_body_transform` exist. | `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`, `crates/liquidfun/src/world/...` | A valve needs no new engine API. |
| Engine particle region query exists: `World::query_particle_system_aabb`. A plain scan of `particle_system_view(system).positions()` is also available. | `crates/liquidfun/src/particle/query.rs` | Goal counting needs no new engine API. |

## Standard Architecture (recommended)

### System Overview

```
┌──────────────────────────────── web/ (main thread) ────────────────────────────────┐
│ Routing            #/scene/<id> (unchanged)  #/puzzles  #/puzzle/<id>   (NEW kinds)  │
│     │                                                                                │
│ Lifecycle    PlayableTarget = {scene,id} | {puzzle,id,placements}  (MODIFIED)        │
│     │                                                                                │
│ ┌─────────────┐  ┌──────────────────┐  ┌──────────────┐  ┌───────────────────────┐   │
│ │ Puzzle store│  │ Placement editor │  │ Puzzle HUD   │  │ Progress (localStorage│   │
│ │ phase,      │◄─┤ drag/rotate/     │  │ fill, timer, │  │ v1, in-memory fallback│   │
│ │ placements  │  │ remove (paused)  │  │ tray, result │  │ (NEW)                 │   │
│ │ (NEW)       │  │ (NEW)            │  │ (NEW)        │  └───────────────────────┘   │
│ └──────┬──────┘  └──────────────────┘  └──────▲───────┘                              │
│        │ newPuzzle(id, kinds, poses)          │ frame.maybePuzzle                    │
│ ┌──────▼──────────────────────────────────────┴───────┐   ┌───────────────────────┐  │
│ │ SceneSession (direct backend, unchanged adapter)    │──►│ Audio (WebAudio, NEW) │  │
│ │ frame loop / camera / safe frame / scale legend     │   │ reads frame signals   │  │
│ └──────┬──────────────────────────────────────────────┘   └───────────────────────┘  │
│        │ RenderFrame (+ puzzle status, + audio signals, + speed lane for GPU water)  │
│ ┌──────▼───────────── render ─────────────────────────────────────────────────────┐  │
│ │ presentSceneFrame → 2D canvas | shaded-blob WebGL | water WebGL (NEW) + overlay │  │
│ └─────────────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────┬─────────────────────────────────────────────┘
                                        │ wasm-bindgen (copied lanes, allowlisted ids)
┌──────────────────────── crates/liquidfun-wasm (private bridge) ─────────────────────┐
│ ProofSession::new(scene) (unchanged)    ProofSession::new_puzzle(level, kinds, poses)│
│        │                                         │                                   │
│ SessionCore  source = Scene(SceneId) | Puzzle(LevelId)        (MODIFIED, small)      │
│        │                                         │                                   │
│ scene/* builders (unchanged)          puzzle/ (NEW)                                  │
│                                        ├─ levels: const LevelSpec data              │
│                                        ├─ builder: LevelSpec → BuiltScene           │
│                                        ├─ tools: validate placements → bodies       │
│                                        ├─ goal: per-step count + outcome latch      │
│                                        └─ valve: kinematic gate driven per step     │
└───────────────────────────────────────┬─────────────────────────────────────────────┘
                                        │ public safe API only (no changes)
┌───────────────────────────── crates/liquidfun (engine) ─────────────────────────────┐
│ World, bodies, fixtures, particle systems, queries — UNCHANGED, fingerprints intact │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Status | Responsibility | Location |
| --- | --- | --- | --- |
| `LevelSpec` and level table | NEW | Declarative level data: walls, liquid blocks (kind and flags), cup interior rectangle, target count, step limit, settle window, tool budget, allowed tool kinds, build zones, optional level valve | `crates/liquidfun-wasm/src/puzzle/levels.rs` (+ `levels/<id>.rs` if a spec grows) |
| Level builder | NEW | `LevelSpec` + validated placements → `BuiltScene` with `PuzzleHooks`. Reuses particle and wall helpers from the existing scene families. | `crates/liquidfun-wasm/src/puzzle/builder.rs` |
| Tool placement model | NEW | Parses the flat `kinds: &[u8]` and `poses: &[f32]` (stride 3: x, y, angle in radians). Rejects non-finite values, unknown kinds, over-budget counts, and poses outside build zones. Creates static or kinematic bodies in a stable order. | `crates/liquidfun-wasm/src/puzzle/tools.rs` |
| Goal counter and outcome | NEW | Pure functional core: `count_in_cup(positions, rect) -> u32`. Outcome latch: `Running → Won { step } | Lost`, requiring the count to stay at or above target for a settle window before the step limit. | `crates/liquidfun-wasm/src/puzzle/goal.rs` |
| Valve | NEW | A kinematic gate body moved by `set_body_linear_velocity` over a fixed number of steps when opened (the Hydraulic Fountain pattern). Opened by a named action. | `crates/liquidfun-wasm/src/puzzle/valve.rs` |
| `SceneHooks` trait | MODIFIED (additive) | Add `fn maybe_puzzle_status(&self) -> Option<PuzzleStatus> { None }`. Optional: `fn audio_signals(&self)`. Existing scenes stay untouched through the default method. | `crates/liquidfun-wasm/src/scene.rs` |
| `SessionCore` | MODIFIED | Replace the `scene_id: SceneId` field with `source: SessionSource { Scene(SceneId), Puzzle(LevelId) }` for escape limits and particle iterations. Add a `create_puzzle` constructor. Accumulate per-batch impact counts for audio. Copy puzzle status into the frame. | `crates/liquidfun-wasm/src/session.rs`, `session/escape.rs` |
| `FrameData` / `ProofFrame` | MODIFIED (additive) | Optional puzzle scalars (`goalCount`, `goalTarget`, `outcome`, `outcomeStep`, `stepLimit`), audio scalars (`impactCount`, `impactStrength`), and later an optional `particleSpeeds` lane | `crates/liquidfun-wasm/src/frame.rs` |
| `ProofSession` | MODIFIED (additive) | `#[wasm_bindgen(js_name = newPuzzle)] fn new_puzzle(level_id: String, kinds: Box<[u8]>, poses: Box<[f32]>)` and `levelInfo()` (cup rectangle, build zones, budget, target, limit) | `crates/liquidfun-wasm/src/lib.rs` |
| Puzzle solution tests | NEW | Native cargo tests: each level's **reference solution wins** at a fixed step with a recorded fingerprint, and **no tools loses**. The empty-tools run proves the level is not trivial. | `crates/liquidfun-wasm/src/puzzle/tests/solutions.rs` |
| TS level metadata | NEW | `PUZZLE_IDS` (separate from `SCENE_IDS`), title, teaching line, hint, tool palette labels, star thresholds, landscape and portrait bounds | `web/src/puzzle/levels.ts`, `web/src/puzzle/portrait-bounds.ts` |
| Route parser | MODIFIED | Add `{ kind: "puzzles" }` (level select) and `{ kind: "puzzle"; id: PuzzleId }` | `web/src/routing/hash.ts` |
| Lifecycle | MODIFIED | Generalize `beginScene(id)` to `beginTarget(target)`. Treat puzzle routes as playable so `abandonScene` is not triggered. Choose bounds by target. | `web/src/player/scene-lifecycle.ts`, `playground-session.ts`, `runtime.ts`, `frame-loop.ts` (`startScene`) |
| Puzzle store | NEW | Phase machine (`editing → running ⇄ paused → won | lost`), placements, tools used, last outcome. Reset returns to `editing` with placements kept. | `web/src/puzzle/store.ts` (functional reducer) + `web/src/puzzle/session.ts` (shell) |
| Placement editor | NEW | Pointer gestures while `editing`: tap a tray item to add, drag to move, rotate handle or two-finger twist, drag to tray or long-press to remove. Hit testing in world space with the shared camera `unprojectPoint`. Ghost drawing. | `web/src/puzzle/editor.ts`, `web/src/puzzle/overlay.ts` |
| Pointer router | MODIFIED | `forwardScenePointer` sends pointers to the editor in `editing`, to WASM in `running` (valve tap), and to nothing in `won` or `lost` | `web/src/input/canvas-pointer.ts`, `web/src/player/scene-surface.ts` |
| Puzzle HUD | NEW | Fill meter, step-based timer, tool tray with budget, result card with stars. Every leaf carries `data-hud-obstacle`. | `web/src/components/puzzle/*.tsx` |
| Level select and navigation | NEW and MODIFIED | `#/puzzles` grid with stars and lock state; a "Puzzles" section in `DemoNavigation` | `web/src/components/puzzle/LevelSelect.tsx`, `web/src/components/DemoNavigation.tsx`, `web/src/player/navigation.ts` |
| Progress | NEW | `liquidfun.puzzle-progress.v1`: best stars, best fill, fewest tools per level. Parsed and validated, with an in-memory copy when storage throws. | `web/src/puzzle/progress.ts` |
| Audio | NEW | Opt-in WebAudio synth. `AudioContext` created only on an unmute gesture. Maps frame signals to splash, impact and chime voices. Never writes back to the simulation. | `web/src/audio/*.ts`, toggle in `SceneControls.tsx` or the HUD |
| GPU water mode | NEW and MODIFIED | `RenderMode` `"water"`, `particleSurface` `"webgl-water"`, a new `webgl-water.ts` that reuses the field pass and adds refraction, highlights and foam. Falls back to the canvas path when unavailable. | `web/src/render/mode.ts`, `present-frame.ts`, new `web/src/render/webgl-water.ts` |

## Recommended Project Structure

```
crates/liquidfun-wasm/src/
├── puzzle.rs                 # NEW: LevelId allowlist, parse_level_id, PuzzleStatus, re-exports
├── puzzle/
│   ├── levels.rs             # NEW: const LevelSpec table (5–8 levels)
│   ├── builder.rs            # NEW: LevelSpec + placements → BuiltScene
│   ├── tools.rs              # NEW: placement parsing/validation, tool body creation
│   ├── goal.rs               # NEW: pure counting + outcome latch (unit tested)
│   ├── valve.rs              # NEW: kinematic gate state machine
│   ├── hooks.rs              # NEW: PuzzleHooks: SceneHooks
│   └── tests/solutions.rs    # NEW: reference-solution wins, empty-tools loses
├── scene.rs                  # MODIFIED: additive SceneHooks default method
├── session.rs                # MODIFIED: SessionSource, create_puzzle, audio tallies
├── session/escape.rs         # MODIFIED: limits keyed by SessionSource
├── frame.rs                  # MODIFIED: optional puzzle + audio scalars, later speed lane
└── lib.rs                    # MODIFIED: ProofSession::new_puzzle, level_info

web/src/
├── puzzle/                   # NEW: game logic kept out of player/ and catalog/
│   ├── levels.ts             # PUZZLE_IDS, metadata, star rules
│   ├── portrait-bounds.ts    # PUZZLE_PORTRAIT_BOUNDS keyed by PuzzleId
│   ├── store.ts              # pure phase reducer + placement ops
│   ├── stars.ts              # pure star scoring
│   ├── editor.ts             # pointer gestures → store ops
│   ├── overlay.ts            # cup, build zones, ghosts, fill tint (2D canvas)
│   ├── progress.ts           # localStorage v1 with fallback
│   └── session.ts            # loadPuzzleSession(id, placements)
├── audio/                    # NEW: synth.ts, signals.ts (pure frame→event mapping)
├── components/puzzle/        # NEW: PuzzleHud, ToolTray, ResultCard, LevelSelect
├── routing/hash.ts           # MODIFIED
├── player/*                  # MODIFIED: target abstraction, pointer routing
├── physics/frame.ts          # MODIFIED: maybePuzzle, audio scalars, maybeSpeeds
├── physics/worker-messages.ts# MODIFIED: validateTransferredFrame passes new fields through
└── render/{mode,present-frame}.ts, webgl-water.ts   # MODIFIED / NEW (closing phase)
```

### Structure Rationale

- **`puzzle/` beside `scene/`, not inside it.** Puzzles are not catalog scenes. They have no README plan, no scene survey entry, and no `SceneId`. A separate module keeps the 25-scene coverage tests honest and avoids conditionals in the shared scene registry.
- **Pure cores (`goal.rs`, `store.ts`, `stars.ts`, `audio/signals.ts`)** follow the functional-core/imperative-shell rule in `standards/core/architecture.md` and give cheap unit tests that do not need WASM.

## Key Decisions

### D1. Where level definitions live: Rust `LevelSpec` data, with TS presentation metadata

**Recommendation:** Levels are allowlisted Rust data (`const LevelSpec`) built by **one generic builder**. They are not eight bespoke builders and not TS JSON sent to WASM.

- Matches the existing security and validation model: allowlisted ids plus bounded named inputs. Sending arbitrary level geometry from JS would make `liquidfun-wasm` parse and validate a general scene language. That is SEED-002 (sandbox editor) scope, which is explicitly out of scope.
- Game rules that are evaluated **per fixed step** (target count, settle window, step limit, cup rectangle, build zones, tool budget) must be in Rust anyway. See D3.
- Native reference-solution tests can build levels without a browser.
- A declarative spec keeps SEED-002 possible later: a future editor could serialize a `LevelSpec`.
- TS owns presentation only: title, teaching text, hint, star thresholds and display, view and portrait bounds. Level select needs titles without constructing a WASM world.
- **Avoid two sources of truth:** TS reads cup, zones, budget, target and limit from `ProofSession.levelInfo()` after construction. Add a Rust test that `include_str!`s `web/src/puzzle/levels.ts` and checks `PUZZLE_IDS` against the Rust allowlist, the same way `scene_spot/tests.rs` checks `SCENE_IDS`.

### D2. How tool placements cross the boundary and become bodies

There are two boundaries: main thread to worker, and JS to WASM. Only Tesla Valve uses the worker, and that worker hard-codes its scene. **Puzzles use the direct backend**, so placements only cross JS to WASM.

**Recommendation: placements are construction input**, like presets.

1. **Editing (paused at step 0).** Construct `ProofSession.newPuzzle(id, [], [])` and show its step-0 frame. Placements live in the TS puzzle store and are drawn as **ghosts** in a 2D overlay. No WASM call happens per drag, which keeps editing smooth on phones. TS validates for UX (budget, build zone, overlap feedback).
2. **Play.** Dispose the preview session and construct `ProofSession.newPuzzle(id, kinds: Uint8Array, poses: Float32Array)`. Rust validates again (fail closed with a typed `SessionError::InvalidPlacement`), creates the level, then creates tool bodies **in placement order** before step 0. The generation counter in `scene-lifecycle.ts` already guards this async swap.
3. **Reset.** Construct the same call again with the stored placements, which returns to `editing`. Determinism follows directly: same level, same placements and same inputs give a bit-identical run.

Encoding: flat typed arrays (`kinds` u8, `poses` f32 with stride 3), with at most 8 tools. This matches the bounded copied-lane style of `frame.rs`. JSON is not needed.

Tools in v1.5 should be **static** (ramp = box or edge polygon, pin = small circle) or **kinematic** (valve gate). Static tools never move, so TS can draw tool highlights from its own placements instead of needing per-body identity in the frame. Body outlines still appear in the existing `rigidSegments` and `rigidCircles` lanes, within the 512-segment and 48-circle caps.

Build zones are level rectangles where tools may sit. The initial liquid never starts inside a build zone, so a new static body never spawns inside particles.

If a level later proves too heavy for the main thread, generalize the worker's `init` op to `{ target: { kind: "puzzle", id, kinds, poses } }`, validated in `parseWorkerRequest`, and pass the new frame fields through `validateTransferredFrame`. This is deferred: no worker change is needed for v1.5.

### D3. Where goal counting belongs: the bridge (`PuzzleHooks::on_after_step`), not the engine and not the web layer

| Option | Verdict | Reason |
| --- | --- | --- |
| Engine API (`crates/liquidfun`) | No | Not needed. `particle_system_view().positions()` and `query_particle_system_aabb` already exist. Adding engine code risks the 25 scene fingerprints for no gain. |
| Web layer reading frame positions | **No, this breaks determinism** | Frames are captured once per 1–4 steps, and the batch size depends on frame rate (`clock.ts`, `MAX_ADVANCE_STEPS = 4`). A win checked per frame would change with device speed. A splash passing through the cup could count on a 60 Hz display and be missed on a 30 Hz one. Win or lose must not depend on the device. |
| **Bridge, once per fixed step** | **Yes** | `PuzzleHooks::on_after_step` scans `positions()` against the cup's interior rectangle in O(n) with no allocation (16,384 particles at most, a few microseconds). It updates the latch, and `capture_frame` copies the status. This is a read-only observation of the world. |

Determinism and fingerprint implications:

- Counting is read-only, so it cannot change world state. Existing scenes keep the default `maybe_puzzle_status() -> None` and do no extra work.
- The `SessionCore` change from `SceneId` to `SessionSource` and the new frame fields are refactors. **Gate them on bit-identical `just playground-scene-spot` fingerprints for all 25 scenes**, the same keep rule as v1.4.
- Puzzle levels get their **own** regression guard (the reference-solution test). Do not add them to the scene survey, which is parsed from `SCENE_IDS`.
- The timer is **step-based**: limit = `step_limit` fixed steps, and the display shows `stepIndex / 60`. Hidden tabs and slow devices pause or slow game time instead of stealing it.
- Use a **settle rule**: Won requires count ≥ target for `settle_steps` consecutive steps (for example 30). Otherwise splashes passing through the cup trigger false wins. Flagged in pitfalls.
- If a level has non-liquid particles (an elastic sponge, powder that should not count), count by particle color or flag class, not raw position. Decide per level in `LevelSpec`.

### D4. Sharing the player, canvas and HUD without disturbing scenes

- Add `PlayableTarget = { kind: "scene"; id: SceneId } | { kind: "puzzle"; id: PuzzleId }`. Replace `maybeReadySceneId(route)` call sites in the lifecycle, the `bindNavigation` effect and `frameDeps.startScene` with `maybePlayableTarget(route)`. Scene behavior stays the same because the `scene` arm runs exactly the current code.
- Bounds: add `worldBoundsForTarget(target, w, h)`. Scenes delegate to `worldBoundsForViewport`. Puzzles use `PUZZLE_VIEW_BOUNDS` and `PUZZLE_PORTRAIT_BOUNDS`, with a test copied from `web/tests/portrait-bounds.test.ts` that fills an iPhone canvas of about 390×844.
- The camera, safe frame (`data-hud-obstacle`), scale legend, render modes, zoom and debug readout are reused as they are. The scale legend appears automatically because it is shell chrome.
- Puzzle HUD components mount only when `route.kind === "puzzle"`. Each leaf (fill meter, timer, tray, result card) carries `data-hud-obstacle` and keeps a camera-independent size, per the repo rule. Extend `web/e2e/hud-safe-frame.spec.ts` with a puzzle list.
- Hide scene-only chrome on puzzle routes: scene controls, SVG export pane, and tilt gravity (tilt would make solutions depend on how the device is held).
- Transport: Play in `editing` constructs the placed session. Pause and Resume keep working. Reset returns to `editing`. While `editing`, the canvas pointer goes to the editor, not to `session.pointerAction`.

### D5. Valve design

One mechanism serves two uses:

- **Level valve** (the "valve" teaching level): a gate that is part of the level. The player opens it during play by tapping it or pressing a HUD button. That calls `applyAction("valve-open")`, and `PuzzleHooks` starts a kinematic slide: `set_body_linear_velocity` for N steps, then zero, then a fixed open pose. This is the same deterministic per-step pattern as the Hydraulic Fountain piston.
- **Valve tool** (optional): a placeable gate with the same action keyed by tool index (`valve-open:<n>`).

Determinism note: actions are applied between `advance` calls, so they always land on a step boundary. Record `(stepIndex, action)` in the puzzle store. The rule "same placements plus same actions at the same steps gives the same result" holds, and a replay log could support an "instant replay" later. A cheaper variant that is fully input-free is a **timed valve**: the player sets its open time while editing, stored as a fourth pose value. Pick this if tap timing feels unfair.

Avoid destroying the gate body. It works, but a sliding gate reads better visually, and it keeps body count stable for the frame lanes.

### D6. Where audio signals come from

| Signal | Source | New code |
| --- | --- | --- |
| Splash into the cup | Change in `goalCount` between frames | Web only (`audio/signals.ts`) |
| General splash intensity | `maxSpeed`, `bodyContactCount` change (already in the frame) | Web only |
| Rigid impacts | Count of `ContactTransitionKind::Begin` events, plus a strength proxy such as the bodies' relative speed, **accumulated across the 1–4 step batch** in `SessionCore::advance` and reset at the start of each batch | Bridge: two new `FrameDiagnostics` scalars (read-only, fingerprint-neutral) |
| Win chime and lose tone | Outcome edge in `maybePuzzle` | Web only |

The audio layer is purely downstream. It creates the `AudioContext` only on the unmute gesture, is muted by default, persists the mute state with the same storage pattern, rate-limits voices, and never calls back into the session. It also works for the 25 scenes (SEED-005), but ship it on puzzle routes first.

### D7. Where GPU water plugs in

- `web/src/render/mode.ts`: add `"water"` to `RenderMode`, `maybeParseRenderMode`, and `RENDER_MODE_GROUPS` (new "GPU" group or "Surface"). Map `particleSurface("water")` to `"webgl-water"`, and make `needsWebglSurface` true for both WebGL surfaces. `svgRenderMode("water")` maps to `"solid"` like the other surface modes, so SVG and WebP exports and the README pipeline are unchanged.
- `web/src/render/webgl-water.ts` (new): reuse the additive point-splat field FBO from `webgl-particles.ts` (extract a shared `field-pass.ts`). Composite with thickness tint, a specular highlight from the field gradient, and refraction that samples a texture of the static rigid layer (`static-layer.ts` already caches walls on a canvas, so upload it with `texImage2D`). Foam is driven by particle speed.
- Foam needs **per-particle speed**: add an optional `particleSpeeds` lane (Float32, stride 1) to `FrameData`, `RawProofFrame`, `RenderFrame` and `frameTransferables`. Do not derive speed in TS from frame-to-frame position deltas: particle order changes when particles are destroyed (eviction, drains), so deltas would pair the wrong particles. The lane is copied data, so fingerprints are unchanged, but run the scene-spot gate anyway.
- `present-frame.ts`: `webglCovered = (water && drawWater(...)) || (blob && drawShadedBlob(...))`. If water is unavailable (no WebGL2, a shader compile fails, context loss), fall back to `drawShadedBlob`, then to the canvas path. Never leave a blank frame.
- Puzzle overlays (cup outline, ghosts) draw on the 2D canvas **after** particles, so they work in every mode.

## Architectural Patterns

### Pattern 1: Construction input instead of live mutation (placements)

**What:** Treat player placements like presets. Rebuild the world from `(level, placements)` instead of mutating a running world.
**When to use:** Anything the player sets before step 0.
**Trade-offs:** One world construction per Play and Reset. Measure this on the largest level. If it is slow, the fallback is to add tool bodies to a fresh step-0 world via `placeTools`, which is equivalent if creation order is the same. Pro: Reset and determinism come for free.

```rust
// crates/liquidfun-wasm/src/lib.rs (sketch)
#[wasm_bindgen(js_name = newPuzzle)]
pub fn new_puzzle(level_id: String, kinds: Box<[u8]>, poses: Box<[f32]>) -> Result<ProofSession, JsError> {
    let level = puzzle::parse_level_id(&level_id).map_err(js_error)?;
    let placements = puzzle::tools::parse_placements(level, &kinds, &poses).map_err(js_error)?;
    SessionCore::create_puzzle(level, &placements).map(|core| Self { core }).map_err(js_error)
}
```

### Pattern 2: Per-step observation in hooks, per-frame publication in capture

**What:** Anything whose result must not depend on frame rate (goal latch, impact tallies) is computed in `on_after_step` or in the `advance` loop. `capture_frame` only copies it.
**When to use:** Win, lose, scoring, and audio event counts.
**Trade-offs:** A few scalars added to the frame contract (Rust validation, TS parser, worker pass-through).

```rust
// puzzle/goal.rs (pure core)
pub(crate) fn count_in_cup(positions: &[Vec2], cup: CupRect) -> u32 { /* strict interior test */ }
pub(crate) fn advance_outcome(state: Outcome, count: u32, step: u32, spec: &GoalSpec) -> Outcome { /* settle window, limit */ }
```

### Pattern 3: Phase reducer for the puzzle UI

**What:** A pure `reducePuzzle(state, event)` with phases `editing | running | paused | won | lost`. Events are `addTool`, `moveTool`, `rotateTool`, `removeTool`, `play`, `pause`, `frame(status)` and `reset`. The Solid shell maps them to session calls.
**When to use:** All puzzle UI state. It keeps the existing `PlayerView` untouched. Puzzle phase is an overlay on `playing` and `paused`, not a replacement.

## Data Flow

### Edit → Play → Win

```
pointer (editing) → editor.ts → reducePuzzle(addTool/moveTool) → overlay ghosts redraw (no WASM)
Play → kinds/poses typed arrays → ProofSession.newPuzzle(id, kinds, poses)
     → Rust validate → build level → create tool bodies → step 0 frame
frame loop: advance(1..4) → per step: hooks.on_advance (valve) → evict → world.step
     → PuzzleHooks.on_after_step: count_in_cup → advance_outcome
captureFrame → frame.maybePuzzle {count, target, outcome, outcomeStep, stepLimit}
     → reducePuzzle(frame) → HUD fill/timer → on Won|Lost: pause loop, stars.ts, progress.ts
     → audio/signals.ts (if unmuted) → WebAudio voices
Reset → newPuzzle(id, same placements) → phase editing
```

### Progress

```
progress.ts load (try/catch) → in-memory map → LevelSelect stars/locks
Won → merge best {stars, fill, tools} → persist (try/catch; failure keeps memory copy)
```

## Scaling Considerations

| Concern | Approach |
| --- | --- |
| Per-step goal scan | O(n), n ≤ 16,384, contiguous slice. Negligible next to the particle solver. Only puzzle sessions run it. |
| World construction on Play and Reset | Measure on the largest level (v1.4 already noted grouped creation load time as a leftover, SEED-006). Keep level particle counts modest, about 2–4k. If construction is too slow, use the `placeTools` fallback. |
| Main-thread stepping | Keep levels well below Tesla Valve or Liquid Tumbler cost so the direct backend stays smooth on phones. Use the worker only if a level proves heavy. |
| Frame contract growth | New fields are scalars plus one optional lane. Keep every cap explicit and validated on both sides. |

## Anti-Patterns

### Anti-Pattern 1: Adding puzzle levels to `SCENE_IDS` or `SceneId`

**What people do:** Treat a level as "just another scene" to reuse the catalog.
**Why it's wrong:** It breaks the scene survey (xtask parses `SCENE_IDS`), README gallery coverage, portrait-bounds and HUD e2e coverage. It also puts placement-dependent construction into a registry built for preset-only scenes.
**Do this instead:** Use a separate `PUZZLE_IDS` and Rust `LevelId`, and a `SessionSource` enum in `SessionCore`.

### Anti-Pattern 2: Judging win or lose from rendered frames or wall-clock time

**What people do:** Count particles in TS from `particlePositions`, or run the timer on `performance.now()`.
**Why it's wrong:** Outcomes would change with frame rate, step batching and tab visibility. "A working solution always reproduces" would no longer hold.
**Do this instead:** Per-step latch in Rust and a step-based timer.

### Anti-Pattern 3: Calling WASM on every drag move while editing

**What people do:** Rebuild the world on each pointer move to show real bodies.
**Why it's wrong:** Construction cost on every move makes phone editing janky.
**Do this instead:** TS ghosts while editing. Build once on Play. Optionally rebuild a preview on pointer-up only.

### Anti-Pattern 4: Engine changes for game rules

**What people do:** Add a "goal region" or "sensor counter" concept to `crates/liquidfun`.
**Why it's wrong:** It is game logic, it risks the scene fingerprints, and it widens the published crate's API for a private demo.
**Do this instead:** Keep it in the private bridge using existing public queries.

### Anti-Pattern 5: Audio or render code that feeds back into the simulation

**What people do:** Trigger physics effects from sound events or renderer state.
**Why it's wrong:** It breaks determinism and the renderer-independence constraint.
**Do this instead:** Keep audio and GPU water strictly downstream of `RenderFrame`.

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
| --- | --- | --- |
| Puzzle store ↔ WASM | `newPuzzle(id, Uint8Array, Float32Array)`, `applyAction("valve-open")`, `levelInfo()` | Direct backend. Rust validates every input again. |
| WASM → renderer, HUD, audio | `RenderFrame` + `maybePuzzle` + audio scalars (+ `particleSpeeds` later) | Update `frame.ts` parse, `worker-messages.ts validateTransferredFrame`, and `frameTransferables` together, even though puzzles do not use the worker. |
| Routing ↔ lifecycle | `PlayableTarget` | The scene arm runs today's code unchanged. |
| Progress ↔ storage | try/catch `localStorage`, versioned key | Same pattern as `render/mode.ts`. |
| TS ids ↔ Rust allowlist | Rust test `include_str!("…/web/src/puzzle/levels.ts")` | Same pattern as `scene_spot/tests.rs`. |

## Suggested Build Order (phases continue after 36)

1. **Puzzle core in the bridge plus one level (no tools yet).**
   - `puzzle/` module: `LevelSpec`, builder, `goal.rs` latch, `PuzzleHooks`, `SessionSource` refactor, `newPuzzle` with an empty placement list, frame puzzle scalars, TS frame parse.
   - Level 1 is "plain water". Its reference solution can be zero tools at first, or use a hard-coded test placement through the native API.
   - Gates: 25 scene fingerprints bit-identical; native goal and latch unit tests.
   - *Dependency root: everything else reads `maybePuzzle`.*
2. **Puzzle route and shared player shell.**
   - `#/puzzle/<id>` parsing, `PlayableTarget` lifecycle, puzzle bounds and portrait frame, HUD (fill meter, step timer, result card), and hiding scene-only chrome.
   - Gates: e2e HUD safe frame for puzzles; scene routes unchanged (existing `just web-player-smoke`).
3. **Place-then-play tools.**
   - Rust `tools.rs` (validation, bodies, budget, build zones), TS store and phase reducer, editor gestures (mouse and touch: drag, rotate, remove), ghost overlay, pointer router, Reset returning to placements, star scoring.
   - Gates: reference-solution test, empty-tools-loses test, and a Chromium e2e that places tools and wins with the deterministic clock (`__liquidfunSyntheticAnimationClock`).
4. **Valve plus the teaching level set (5–8).**
   - Valve mechanism (level valve first, optional tool valve, action log). Levels for plain water, viscous, powder, elastic sponge, buoyancy, valve and the finale, each with a reference solution, portrait frame and metadata. Decide the counting class (liquid vs sponge particles) per level.
   - Needs tools from phase 3 and goal counting from phase 1.
5. **Level select and progress.**
   - `#/puzzles` grid, navigation section, `progress.ts` with blocked-storage fallback, and an unlock rule.
   - This could merge into phase 4 if small, but keeping it separate gives a clean, testable storage boundary.
6. **Opt-in sound.**
   - Bridge impact tallies (fingerprint gate), `audio/signals.ts` pure mapping, synth and mute toggle.
   - Depends on outcome edges (phase 1) and level content to tune against (phase 4).
7. **GPU water render mode (closing).**
   - `particleSpeeds` lane (fingerprint gate), shared field pass, `webgl-water.ts`, mode wiring and fallback chain, export mapping to `solid`.
   - Independent of puzzle logic, so it can slip without blocking the game.

**Ordering rationale:** the goal latch and frame contract come first because the HUD, stars, progress, sound and solution tests all consume them. The route and shell come before tools so tools are built inside the real player. Tools come before the level set because most levels are unsolvable without them. Sound and GPU water only read frames and carry no game risk, so they go last. Phases 1, 6 and 7 touch the bridge and frame contract, so each re-runs `just playground-scene-spot` and requires identical fingerprints.

**Research flags:**

- Phase 3: touch gesture design for rotation on phones (two-finger twist vs a rotate handle) needs a short spike.
- Phase 4: per-level tuning is empirical, so solvability and difficulty need playtesting. Decide the sponge-particle counting rule.
- Phase 7: refraction needs a background texture and WebGL2 context-loss handling. Spike on a phone GPU.
- Phases 1–2: standard patterns from the existing code. Low research need.

## Sources

Read in the repository (HIGH confidence, current HEAD `153669d86`):

- `crates/liquidfun-wasm/src/{lib.rs, scene.rs, session.rs, session/escape.rs, frame.rs, scene_spot.rs, scene_spot/fingerprint.rs}`
- `crates/liquidfun-wasm/src/scene/{tesla_valve.rs, hydraulic_fountain.rs, sparky.rs}`
- `crates/liquidfun/src/particle/query.rs`, `crates/liquidfun/src/world/particle_object/particle.rs`, `crates/liquidfun/src/world/contact.rs`, `crates/liquidfun/src/world/object/body_object.rs`
- `web/src/physics/{simulation-worker.ts, worker-messages.ts, worker-session.ts, live-session.ts, session.ts, loader.ts, frame.ts}`
- `web/src/player/{scene-lifecycle.ts, playground-session.ts, scene-runtime.ts, runtime.ts, frame-loop.ts, view.ts, appearance.ts, scene-surface.ts}`
- `web/src/render/{mode.ts, present-frame.ts, webgl-particles.ts, static-layer.ts}`, `web/src/routing/hash.ts`, `web/src/catalog/{scenes.ts, portrait-bounds.ts}`, `web/src/input/canvas-pointer.ts`
- `tools/xtask/src/playground/spot.rs` (parses `SCENE_IDS`)
- `.planning/PROJECT.md`, `.planning/seeds/SEED-001`, `SEED-003`, `SEED-005`, `CLAUDE.md` repo-local rules

---
*Architecture research for: v1.5 Puzzle Mode*
*Researched: 2026-10-11*
