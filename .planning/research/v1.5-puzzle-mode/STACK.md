# Stack Research: v1.5 Puzzle Mode

**Domain:** Deterministic physics puzzle game ("get the water into the cup") layered onto the existing liquidfun-rs WASM + SolidJS playground
**Researched:** 2026-10-11
**Confidence:** HIGH for "no new dependencies" and the integration points (verified against repo source); MEDIUM for iOS-specific GPU/audio behavior (verified with WebKit bugs and MDN, but device-dependent)

## Bottom Line

**v1.5 does not need any new npm package or Cargo crate.** The engine already has the queries the goal counter needs. The WASM bridge already moves typed arrays. The web app already handles Pointer Events with capture, guarded `localStorage`, hash routing, and a WebGL2 float-framebuffer particle pipeline with a canvas fallback. Every new capability maps to a platform API or code the repo already has:

| New capability | Decision | Where it lives |
|---|---|---|
| Particle-in-cup counting | **Count in Rust inside `liquidfun-wasm`, once per fixed step.** No engine change. Do not count in JS from frames. | New puzzle session code in `crates/liquidfun-wasm/src/` |
| Hit-test, drag, rotate and remove tools while paused | **Pure TypeScript geometry in world meters** on top of the existing Pointer Events and `unprojectPoint`. No gesture library and no engine query. | `web/src/puzzle/` (new), reusing `web/src/input/canvas-pointer.ts` and `web/src/render/camera.ts` |
| Apply placements to physics | **Rebuild the level world from (level, placements) on Play and Reset.** Placements cross as copied typed arrays (`&[u8]` / `&[f32]` wasm-bindgen params). | New `ProofSession` method plus worker op |
| Sound | **Web Audio API, hand-written** (oscillator, gain, biquad and a generated noise buffer). No Tone.js, Howler or zzfx. | `web/src/audio/` (new) |
| GPU water | **Extend the existing WebGL2 pipeline** in `webgl-particles.ts` (`#version 300 es`, RGBA16F field FBO). Add MRT and a refraction/specular/foam composite shader. No three.js, PixiJS or twgl. | `web/src/render/` |
| Progress persistence | **`localStorage` behind the existing storage-provider + try/catch pattern**, with an in-memory fallback. No idb-keyval. | `web/src/puzzle/progress.ts` (new) |
| `#/puzzle/<id>` routing | **Extend `web/src/routing/hash.ts`.** No router library. | `web/src/routing/` |
| Level select UI | **Existing Kobalte + Tailwind + CVA.** | `web/src/components/` |

## Recommended Stack

### Core Technologies (all existing, no version change required)

| Technology | Pinned in repo | Latest observed (2026-10-11) | Purpose in v1.5 | Why keep as-is |
|---|---|---|---|---|
| Rust `liquidfun` engine | workspace | n/a | Physics for levels and tools: static and kinematic bodies, revolute joints for valves, elastic and powder particle groups | Already exposes `World::query_particle_system_aabb`, `PolygonShape::test_point`, `particle_system_view().positions()`, body create/destroy and revolute/prismatic/mouse joints. The goal counter and tools need **zero engine changes**, so all 25 scene fingerprints stay untouched by construction. |
| `wasm-bindgen` | `=0.2.128` | 0.2.129 (2026-09-25) | Export the puzzle session API | Slice parameters (`&[f32]`, `&[u8]`) and returned `Vec<f32>`/`Box<[f32]>` are already supported. A bump is unnecessary and would need a matching `wasm-bindgen-cli` change. Leave the bump to a separate quick task. |
| `solid-js` | 1.9.15 | 1.9.17 | Puzzle shell, level select, toolbox, win overlay | Patch-level drift only. No v1.5 feature needs the newer patch. |
| `@kobalte/core` | 0.13.12 | 0.13.14 | Accessible level-select dialog/list, toolbox buttons, mute toggle | Already licensed and verified (`verify:kobalte-licenses`). Patch drift only. |
| WebGL2 (platform) | n/a | Baseline in all evergreen browsers, iOS Safari 15+ | GPU water render mode | The repo already runs a WebGL2 `#version 300 es` field-accumulate plus composite pass with context-loss handling and a canvas fallback. |
| Web Audio API (platform) | n/a | Baseline | Splash, impact and win-chime synthesis | Covers short procedural SFX in about 150 lines with no assets or licenses. |
| Pointer Events (platform) | n/a | Baseline | Mouse and touch tool dragging | Already used with `setPointerCapture` and `touch-action: none` on the viewport (`player-viewport.css`). |
| Web Storage (platform) | n/a | Baseline | Stars and unlocks | Already used for render mode and HUD slider preferences through guarded providers. |

### Supporting Libraries

**None.** The table below covers the libraries a typical "puzzle game in the browser" stack would add, and why each one is redundant here.

| Library | Latest version | What it would do | Why not needed |
|---|---|---|---|
| `tone` | 15.1.22 | Audio synthesis and scheduling framework | Hundreds of KB for three sounds. Plain `OscillatorNode` + `GainNode` envelopes + a noise `AudioBufferSourceNode` through a `BiquadFilterNode` cover splash, impact and chime. |
| `howler` | 2.2.4 (last published 2023-09) | Sample playback with unlock helpers | Needs audio assets (and their licenses). Stale. Unlocking is a single `AudioContext.resume()` inside the unmute click. |
| `zzfx` | 1.4.0 | Tiny procedural SFX | Closest fit and very small, but the owner prefers platform APIs and the needed sounds are simple. It is the only acceptable fallback if hand tuning stalls; vendor the ~1 KB function rather than adding a dependency. |
| `interactjs` | 1.10.28 | Drag, rotate and gesture handling | The tools are a few rectangles and circles in world space. Pointer Events + `unprojectPoint` + rotation handles do the job. A gesture library would fight the existing canvas pointer forwarding and HUD obstacle logic. |
| `@use-gesture/vanilla` | 10.3.1 (2024-03) | Gesture recognition | Same reason as interactjs, and it is unmaintained. |
| `idb-keyval` | 6.3.0 | IndexedDB key-value store | Progress is a few hundred bytes of JSON. `localStorage` is synchronous, simpler and already wrapped. |
| `twgl.js` / `pixi.js` / three.js | 7.0.0 / 8.22.0 / n/a | WebGL helpers or renderers | The existing hand-rolled WebGL2 module already handles program linking, VAOs, FBOs and context loss. A renderer would duplicate the camera and HUD safe-frame math and inflate the Pages bundle. |
| Any router (`@solidjs/router`) | n/a | Routing | `hash.ts` is 64 lines of tested parsing. Add a `puzzle` route kind. |
| `serde` / `serde_json` in `liquidfun-wasm` | n/a | Parse JSON placements | Typed-array params avoid a parser in the WASM bundle and keep the bridge's "bounded copied lanes" design. |

### Development Tools (all existing)

| Tool | Version | v1.5 use |
|---|---|---|
| Vitest | 5.0.1 | Pure functions: tool hit-testing, placement quantization and validation, star scoring, progress parse/migrate, hash routing, audio parameter mapping |
| Playwright | 1.63.0 (Chromium) | Puzzle e2e: drag a tool, press Play, assert win state; HUD safe frame for puzzle chrome; storage-blocked context. Headless Chromium runs WebGL2 through SwiftShader, which the existing `webgl-render-*` tests already rely on. |
| Rust tests in `liquidfun-wasm` | toolchain 1.97.0 | Deterministic solution replay per level (placements in, win at step N out), goal count unit tests, placement validation rejection |
| `just playground-scene-spot` | existing | Confirms the 25 scene fingerprints and timings are unchanged after the puzzle session code lands |

## Integration Details by Capability

### 1. Goal zone counting: count in Rust, per step (HIGH confidence)

**Recommendation:** Add a puzzle session in `liquidfun-wasm` that, after **every fixed step** inside `advance()`, counts live particles whose centers lie inside the level's goal polygon. It records fill state (current count, peak, first step at or above target, contiguous steps at or above target). It exposes these as scalars on the captured frame, next to `max_speed` and `body_contact_count` in `FrameDiagnostics`.

**Why Rust rather than JS counting from frames:**
- Frames are captured **per render**, not per step. `advance` runs up to `MAX_ADVANCE_STEPS = 4` steps between captures, and the count depends on frame pacing and hidden-tab catch-up. A win evaluated on captured frames could differ between a 60 Hz laptop and a 120 Hz phone, or after a backgrounded tab. That breaks the core promise that "a working solution always reproduces."
- The win rule and the timer must be judged on the deterministic step clock. Per-step counting in Rust is the only place that sees every step without copying 16k positions per step across the worker boundary.
- Cost is trivial. One pass over at most 16,384 `Vec2`s per step with a convex point-in-polygon test is far below current step costs (the slowest scene is about 24 ms/step natively).

**How, with existing APIs only:**
- Read positions through the existing `SessionCore::read_particles` / `particle_system_view(..).positions()`. Test with a convex polygon (the cup interior) using either a local half-plane test or the engine's public `PolygonShape::test_point(transform, point)` with an identity transform. Reusing the engine's test keeps the geometry semantics consistent.
- `World::query_particle_system_aabb` (centers strictly inside, source proxy order) also works for rectangular cups. A full scan with a polygon test is simpler, handles slanted cup walls, and avoids proxy construction each step.
- For "only the right liquid counts" levels, filter by particle group or flags from the same view (`flags()`, group membership). Decide per level and record it in the level definition.
- **Do not** use a sensor fixture for the goal. Particle coupling skips sensor fixtures (`world/particle_coupling.rs`), so a sensor gives no particle counts anyway, and it would add contact-manager work.
- **Engine crate is not modified.** This is read-only observation in the private bridge, so scene fingerprints cannot change. The puzzle code path runs only for puzzle sessions.

**Frame contract additions (small and bounded):** `goalCount: u32`, `goalTarget: u32`, `goalTotal: u32` (eligible particles), and `outcome` (`running | won | timed-out`) plus `outcomeStep`. Add them to `frame.rs` / `ProofFrame` getters and to `web/src/physics/frame.ts` parsing with the same validation style (non-negative integers, finite values).

### 2. Tool placement: TS geometry while paused, Rust rebuild on Play (HIGH confidence)

**Recommendation:** While paused, tools are **UI objects in world meters owned by TypeScript**. Dragging, rotating, hit-testing, overlap checks and drawing all happen in JS over the held frame, with no WASM round trip per pointer move. On **Play**, the worker sends the quantized placement list to Rust, which **recreates the level world** from its authored definition plus the placements and then steps. **Reset** recreates from the same list, which gives "Reset returns to the player's placements" for free.

**Why:**
- Determinism: building the world fresh from `(level, placements)` means a given solution always produces the same run. Mutating a live world mid-sim would make the outcome depend on when edits happened.
- The existing `applyControl` already returns "world was recreated", so rebuild-on-change is an established pattern in this bridge.
- Paused-state hit-testing is pure geometry on a handful of tools: oriented-rectangle and circle point tests and rotation-handle hit circles. It is easier to unit test in Vitest than an engine query, and it needs no world.

**Transport (no new deps):**
- New wasm export, for example `ProofSession::set_placements(kinds: &[u8], params: &[f32]) -> Result<bool, JsError>`, with a fixed stride per tool (x, y, angle, plus a variant parameter). wasm-bindgen copies slices in, which matches the bridge's "copied typed arrays, no raw pointers" rule from Phase 16.
- New worker op `{ op: "placements", kinds: Uint8Array, params: Float32Array }` in `worker-messages.ts`, validated like the existing ops (finite, bounded count).
- Rust validates each placement against the level's allowlist (tool kind allowed, count within inventory, inside the build area, not in no-build zones) and rejects with a bounded error. TS duplicates the checks only for immediate UI feedback; Rust stays authoritative.

**Quantize placements** (for example 0.05 m position steps and 7.5° or 15° angle steps, with angles stored as fractions of τ). This keeps drags easy on touch, makes the JS f64 to Rust f32 conversion produce identical values every time, and makes stored or replayed solutions bit-reproducible.

**Rotation on phones:** use a visible rotate handle and snap steps, plus keyboard shortcuts on desktop. Two-finger rotate gestures conflict with the existing pinch/zoom camera and need a gesture recognizer. Leave them out.

**HUD rules still apply:** the toolbox tray and per-tool handles are on-canvas chrome. Put `data-hud-obstacle` on leaf elements so `safe-frame.ts` fits the world around them, and add a portrait frame per level, mirroring `PORTRAIT_VIEW_BOUNDS`.

### 3. Sound: plain Web Audio, lazily created, muted by default (HIGH, iOS details MEDIUM)

- Create the `AudioContext` **only inside the first unmute click handler** and call `resume()` there. That satisfies autoplay policies on all browsers and keeps "muted by default" honest: no context exists until opt-in.
- Synthesis recipes (no assets):
  - **Splash:** short white-noise `AudioBuffer` (generated once) → `BiquadFilterNode` (bandpass, frequency scaled by intensity) → `GainNode` with a fast attack/decay envelope.
  - **Impact:** short sine/triangle `OscillatorNode` with a pitch drop and exponential gain decay.
  - **Win chime:** two or three sine partials on a major arpeggio, with `setValueAtTime` / `exponentialRampToValueAtTime` envelopes.
- **Signals come from frames, not the sim.** Drive sounds from per-frame deltas the worker already has or gains cheaply: change in goal count (particles entering the cup), `body_contact_count` delta, and `max_speed`. Rate-limit triggers (for example at most one splash per 80 ms) and cap concurrent voices so a dam break doesn't produce a wall of noise. Audio never feeds back into physics.
- Call `suspend()` on `visibilitychange` hidden and when leaving the puzzle route. Persist the mute choice through the same guarded storage provider.
- **iOS silent switch (MEDIUM):** Safari's default audio session mutes Web Audio when the ringer switch is on silent. Setting `navigator.audioSession.type = "playback"` (Safari 16.4+/iOS 17, not Baseline) overrides that, but it also **pauses other audio on the device**, such as the user's music. For an opt-in toy, keep the default behavior and add a UI hint ("No sound? Check the silent switch."). Do not force `playback`.
- Testability: wrap context creation behind an injectable factory so Vitest tests the pure mapping (signal → frequency/gain/duration) without a real `AudioContext`. No `standardized-audio-context` polyfill is needed.

### 4. GPU water render mode: extend the existing WebGL2 module (HIGH, mobile formats MEDIUM)

Build on `web/src/render/webgl-particles.ts`, which already does:
- additive point splats into an **RGBA16F** framebuffer (`gl.HALF_FLOAT`) with `blendFunc(ONE, ONE)`,
- a full-screen composite pass with smoothstep edge and density shading,
- context loss/restore handling, and a `false` return that makes `present-frame.ts` fall back to canvas modes.

**Additions (all WebGL2 core features):**
- **Multiple render targets** (`drawBuffers`, core in WebGL2): attachment 0 keeps premultiplied color plus coverage; attachment 1 accumulates **per-particle speed** for foam and spray, plus a thickness channel.
- **Composite shader:** surface normal from central differences of the coverage field; **refraction** by offsetting the UV into a background texture; **Blinn-Phong specular** from a fixed light; **Beer-Lambert thickness tint**; **foam** where accumulated speed or a sharp coverage gradient (the edge) is high.
- **Background for refraction:** the rigid scene is drawn on the 2D canvas, not in GL. Either upload the cached static layer once per change with `texImage2D(canvas)` (cheap, since it is already cached in `static-layer.ts`), or refract a procedural shader backdrop. Do not re-upload the full 2D canvas every frame on phones.
- **Frame contract:** add one optional `particleSpeeds: Float32Array` lane (1 float per particle, computed from the existing `view.velocities()` that `capture_frame` already reads for `max_speed`). Use one scalar rather than a full velocity vector to keep the copy small (at most 64 KB per frame at the 16,384 cap). Only fill it when the water mode is active, through a capture option, so other modes pay nothing.

**Mobile float-target pitfall to fix while here:** the current code **requires `EXT_color_buffer_float`** but renders to RGBA16F. WebKit engineers have stated that iOS historically supports only 16-bit float render targets, and reports conflict on whether `EXT_color_buffer_float` is exposed on iOS. Accept **either `EXT_color_buffer_float` or `EXT_color_buffer_half_float`**, then trust `checkFramebufferStatus` (already present) for the final verdict. Otherwise the new water mode (and today's shaded blob) may silently fall back to canvas on iPhones. Additive blending into 32-bit float targets would need `EXT_float_blend`, so stay on RGBA16F.

**Exports:** the animated SVG/WebP README pipeline should keep the existing modes. GPU water is a live-view mode with the canvas fallback, not an export format.

### 5. Progress persistence: guarded localStorage (HIGH)

- Follow `hud-sliders-preference.ts`: a `() => Storage` provider, every `getItem`/`setItem` in `try/catch` (accessing `window.localStorage` itself throws `SecurityError` when cookies or storage are blocked), and an **in-memory store** that keeps the session fully playable.
- One versioned key, for example `liquidfun.puzzle-progress.v1`, holding `{ version: 1, levels: { [levelId]: { bestStars, bestFillPct, solved } } }`. Parse defensively: unknown level ids are ignored, values are clamped, and bad JSON starts fresh without throwing. Unlocks are **derived** from solved levels plus the level order, not stored, so changing level order never strands a player.
- Optional: store the best solution's quantized placements per level. They are tiny and replay deterministically, which enables a "show my solution" Reset.

### 6. Routing and catalog (HIGH)

- Extend `SceneRoute` in `hash.ts` with `{ kind: "puzzle-index" }` and `{ kind: "puzzle"; id: PuzzleId }` for `#/puzzles` and `#/puzzle/<id>`. Keep `#/scene/<id>` parsing untouched and extend `tests/hash.test.ts`.
- Puzzle levels are **not catalog scenes**. Keep them out of `web/src/catalog/scenes.ts` so the 25-scene invariants (README SVG plan coverage, portrait-bounds coverage, scene-spot survey and fingerprints) don't fire on levels. Give levels a parallel `PuzzleId` registry with its own portrait-frame map and its own coverage test, mirroring `portrait-bounds.test.ts`. The scale legend and HUD safe frame still apply because levels reuse the player canvas.
- On the Rust side, parse a separate allowlist (for example `parse_puzzle_id`) rather than adding `SceneId` variants. Then `scene_spot` and the fingerprint tooling stay exactly the 25 scenes.

## Installation

```bash
# Nothing to install. v1.5 adds no npm packages and no Cargo crates.
# Existing workflow is unchanged:
cd web && bun install --frozen-lockfile
```

## Alternatives Considered

| Recommended | Alternative | When the alternative would win |
|---|---|---|
| Per-step goal count in Rust bridge | JS counting from captured frames | Only for a purely cosmetic live fill meter. It must never decide win, stars or timer outcome. |
| Per-step goal count in Rust bridge | New engine API (for example `count_particles_in_shape`) in `crates/liquidfun` | If a second consumer (native testbed, examples) needs it. Even then it is read-only and fingerprint-safe, but it widens the published API for no v1.5 benefit. |
| Rebuild world from (level, placements) | Live-mutate bodies (`set_transform`) on Play | If world construction for a level becomes slow (over about 100 ms). Unlikely at puzzle scale; measure first. |
| TS hit-testing | Engine `query_aabb` + `test_point` via WASM | If tools become complex compound shapes authored only in Rust. Even then, export the outlines once and hit-test in TS. |
| Typed-array placement params | JSON string through existing `applyControl` | If placement data grows nested. It would pull in `serde_json` or hand parsing on the Rust side, so stay with typed arrays. |
| Hand-written Web Audio | Vendored zzfx function (MIT, ~1 KB) | If tuning procedural sounds by hand takes longer than about one plan. Vendor the function with its license; don't add the npm dependency. |
| Extend `webgl-particles.ts` | Separate new GL module | If the composite shader plus MRT pushes the file well past the 500-line guideline. Split into `webgl-water.ts` that shares the context and field helpers. |
| `localStorage` | IndexedDB | Never at this data size. |

## What NOT to Use

| Avoid | Why | Use instead |
|---|---|---|
| Wall-clock timers (`setTimeout`, `performance.now`) for the level timer or win | Breaks determinism. Results would vary with frame rate and tab visibility. | Step-index deadline: `deadlineStep = seconds × 60` at the 1/60 s timestep, judged in Rust |
| Win or stars computed from captured frames | Frame sampling is pacing-dependent (up to 4 steps per advance). | Rust-side per-step outcome exposed on the frame |
| Sensor fixtures as the goal detector | Particles skip sensor fixtures in particle coupling, so they never produce counts. | Polygon/AABB test on particle positions |
| Adding puzzle levels to `SceneId` / `SCENE_IDS` | Trips README SVG plan coverage, scene-spot survey and fingerprint tooling, and changes the "25 scenes" contract. | Separate `PuzzleId` allowlists in Rust and TS |
| Unquantized float placements | Sub-pixel drag differences make "the same" solution produce different runs; replay storage gets fragile. | Snap to a grid and angle step before sending |
| Gesture libraries (interactjs, use-gesture, Hammer) | Conflict with existing canvas pointer forwarding, pinch zoom and HUD obstacle handling. Unneeded for a few tools. | Pointer Events + `setPointerCapture` + rotate handle |
| Audio libraries (Tone.js, Howler) and sample assets | Bundle weight, asset licensing and unlock complexity for three sounds | Web Audio nodes |
| Forcing `navigator.audioSession.type = "playback"` | Pauses the user's other audio on iOS and is not Baseline | Default session plus a silent-switch hint |
| three.js / PixiJS / twgl for the water mode | Duplicates camera, safe-frame and fallback logic; large bundle | Existing WebGL2 module + MRT |
| Requiring only `EXT_color_buffer_float` for RGBA16F targets | Likely forces the iOS fallback | Accept half-float or float extension, then check FBO completeness |
| Bumping `wasm-bindgen` mid-milestone | Requires a lockstep CLI bump and re-verifying the bridge, unrelated to the features | Separate quick task if wanted |

## Stack Patterns by Variant

**If a level uses multiple liquids and only one counts:**
- Filter in the Rust counter by particle group or flag, and give the counted fluid a distinct color so the rule is visible.

**If a valve tool needs player control during play:**
- Prefer an authored timed or threshold behavior inside the level, such as a revolute joint with a motor schedule or a gate opened at step N. If it must be interactive mid-run, route it through the existing `applyAction` allowlist, which is a deterministic input at a recorded step. Replay determinism then holds only if the action step is recorded, so prefer place-then-play-only tools in v1.5.

**If WebGL2 or float targets are unavailable:**
- The water mode reports `false` and `present-frame.ts` already falls back to canvas surface modes. Show the mode as unavailable in the controls sheet rather than silently switching.

**If storage is blocked:**
- In-memory progress for the session, with all levels playable. Optionally show a one-line "progress won't be saved" note.

## Version Compatibility

| Component | Compatible with | Notes |
|---|---|---|
| `wasm-bindgen =0.2.128` | `&[u8]`, `&[f32]` params; `Vec<f32>` returns | Already used for frame lanes. No version change. |
| WebGL2 MRT (`drawBuffers`) | All WebGL2 contexts | Core feature. RGBA16F color attachments need `EXT_color_buffer_half_float` or `EXT_color_buffer_float`. |
| `EXT_color_buffer_half_float` on iOS | Safari 15+ WebGL2 (WebKit bug 217107 moved it to WebGL2 contexts) | Verify on device. Keep the `checkFramebufferStatus` check. |
| Web Audio `AudioContext` | All target browsers | Needs a user gesture to start. Create it inside the unmute handler. |
| `navigator.audioSession` | Safari 16.4+ only (experimental) | Not used by default. Feature-detect if ever adopted. |
| Pointer Events + `setPointerCapture` | All target browsers | Already in use. Keep `touch-action: none` on the canvas viewport. |
| Playwright 1.63 Chromium headless | WebGL2 via SwiftShader | Existing WebGL tests prove it. Don't screenshot-diff shader output; assert the mode engaged and fell back correctly. |

## Sources

- Repository source, inspected 2026-10-11 (HIGH):
  - `crates/liquidfun/src/particle/query.rs` (`query_particle_system_aabb`, centers strictly inside)
  - `crates/liquidfun/src/collision/shape/polygon.rs` (`test_point`)
  - `crates/liquidfun/src/world/particle_coupling.rs` (sensor fixtures skipped)
  - `crates/liquidfun/src/joint/definition.rs` (Revolute, Prismatic, Mouse)
  - `crates/liquidfun-wasm/src/{lib.rs,session.rs,frame.rs,scene.rs}` (bridge API, `FrameDiagnostics`, 16,384 particle cap, recreate-on-control)
  - `web/src/render/webgl-particles.ts` (`#version 300 es`, RGBA16F `HALF_FLOAT` FBO, `EXT_color_buffer_float` gate, fallback)
  - `web/src/render/present-frame.ts`, `web/src/render/camera.ts` (`unprojectPoint`), `web/src/input/canvas-pointer.ts` (`setPointerCapture`)
  - `web/src/player/hud-sliders-preference.ts` (guarded storage pattern), `web/src/routing/hash.ts`, `web/package.json`
- npm registry `npm view`, 2026-10-11 (HIGH for version numbers): solid-js 1.9.17, @kobalte/core 0.13.14, tone 15.1.22, howler 2.2.4, zzfx 1.4.0, interactjs 1.10.28, @use-gesture/vanilla 10.3.1, idb-keyval 6.3.0, twgl.js 7.0.0, pixi.js 8.22.0
- crates.io API, 2026-10-11: wasm-bindgen max stable 0.2.129 (HIGH)
- [MDN: AudioSession.type](https://developer.mozilla.org/docs/Web/API/AudioSession/type) and [MDN: AudioSession](https://developer.mozilla.org/docs/Web/API/AudioSession) — "playback" semantics; not Baseline (MEDIUM)
- [WebKit bug 237322](https://auto-bugs.webkit.org/show_bug.cgi?id=237322) and [adactio.com webaudio notes](https://adactio.com/tags/webaudio) — silent switch mutes Web Audio by default; `playback` workaround since iOS 17 (MEDIUM)
- [WebKit bug 217107: Fix EXT_color_buffer_half_float on iOS](https://bugs.webkit.org/show_bug.cgi?id=217107) and [WebKit bug 247240](https://bugs.webkit.org/show_bug.cgi?id=247240) — iOS 16-bit float targets; conflicting reports on 32-bit `EXT_color_buffer_float` (MEDIUM)
- [web3dsurvey: EXT_color_buffer_float](https://web3dsurvey.com/webgl2/extensions/EXT_color_buffer_float) and [EXT_float_blend](https://web3dsurvey.com/webgl2/extensions/EXT_float_blend) — crowd-sourced support data (LOW, methodology unclear)
