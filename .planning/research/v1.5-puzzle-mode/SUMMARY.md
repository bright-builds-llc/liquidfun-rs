# Project Research Summary

**Project:** liquidfun-rs, milestone v1.5 Puzzle Mode
**Domain:** Deterministic place-then-play physics puzzle ("get the water into the cup") layered onto the existing Rust/WASM + SolidJS playground
**Researched:** 2026-10-11
**Confidence:** HIGH for integration points and the no-new-dependency stack (read from repo source at `153669d86`); MEDIUM for genre UX, level feel and iOS audio/GPU behavior; MEDIUM–LOW for native-vs-wasm bit equality (mechanism confirmed, not measured)

Detailed files: [STACK.md](STACK.md), [FEATURES.md](FEATURES.md), [ARCHITECTURE.md](ARCHITECTURE.md), [PITFALLS.md](PITFALLS.md).

## Executive Summary

v1.5 turns the playground into a small Incredible Machine-style game. While the level is paused at step 0, the player places a limited bin of fixed-size tools. They press Play, and the level is judged on whether a target number of particles is held in the cup before a step-based deadline. Stars use Cut the Rope's model of independent, pre-visible criteria. The fill goal follows Sugar Sugar ("the cup is full at N particles"). All four researchers reached the same conclusion: **no new npm package, no new Cargo crate and no change to the `crates/liquidfun` engine are needed.** Goal counting, placement validation, tool bodies, the outcome latch and the valve all go in the private `liquidfun-wasm` bridge, using existing public queries (`particle_system_view().positions()`, `PolygonShape::test_point`, kinematic velocity driving, revolute joints). Everything else is TypeScript on platform APIs the repo already uses: Pointer Events, guarded `localStorage`, hash routing, WebGL2 and Web Audio. All 25 scene fingerprints therefore stay unchanged by construction. The bridge refactors (`SessionSource`, new frame scalars, the later speed lane) are each gated on `just playground-scene-spot` reporting 25/25 identical fingerprints.

The approach depends on two rules that every researcher raised on their own. **(1) Placements are construction input.** The world is rebuilt from `(level, canonical placements)` on Play and Reset, never edited live. Tools are created in canonical order at step 0, and placements snap to a grid (0.05 m, angles in τ/24 steps, stored as integers). **(2) Win, timer and stars are decided per fixed step in Rust**, not from rendered frames or wall-clock time. The playground caps catch-up at 4 steps per advance and drops debt on slow devices and hidden tabs, so any frame- or clock-based rule would make a solution pass on a laptop and fail on a phone. With both rules in place, "a working solution always reproduces" can be enforced by a native reference-solution test per level. Levels stay out of the 25-scene catalog. They get a separate `LevelId`/`PUZZLE_IDS` registry, their own portrait frames and their own coverage tests, so README gallery coverage, the scene survey and the portrait/HUD tests do not fire on levels.

The main risks are determinism leaks, level quality and phone UX. Determinism can leak through creation order, live edits, tilt/pointer/gravity inputs, wall-clock timers, or a native-vs-wasm divergence in `sin`/`cos`. Level quality can fail through unsolvable, trivial or knife-edge levels with no automated proof. Phone UX can fail through tiny hit areas, camera refits when the HUD changes, and storage-blocked lockouts. The mitigations are concrete: a step-latched `GoalTracker` with a settle window, an authored fill denominator, a per-level reference-solution plus zero-tools-loses plus nudge-sweep test, a single authoritative Rust validator with no-place zones, puzzle routes that disable tilt/pointer/gravity/zoom, a fixed set of HUD obstacle elements, and soft unlocks. Sound and GPU water only read frames, so they go last and can slip without blocking the game. One existing issue should be fixed regardless: `webgl-particles.ts` requires `EXT_color_buffer_float` but renders to RGBA16F, which probably forces today's shaded-blob mode to fall back to canvas on iPhones.

## Key Findings

### Recommended Stack

No version changes and no installs. Every v1.5 capability maps to code the repo already has or to a platform API (see the STACK.md capability table). The researchers rejected Tone.js/Howler (bundle size and asset licensing for three sounds), interactjs/use-gesture (they conflict with existing pointer forwarding and pinch zoom), idb-keyval (progress is a few hundred bytes), three.js/Pixi/twgl (they would duplicate the camera, safe-frame and fallback logic), `@solidjs/router`, and `serde_json` in the bridge (typed arrays are enough). If hand-tuning procedural sound stalls, vendoring the ~1 KB zzfx function with its MIT license is the only acceptable fallback. A `wasm-bindgen` bump to 0.2.129 belongs in a separate quick task.

**Core technologies (all existing):**

- **Rust `liquidfun-wasm` bridge**: per-step goal count, outcome latch, placement validation, tool bodies and valve. Read-only use of public engine APIs keeps the fingerprints safe.
- **wasm-bindgen `=0.2.128` copied slices**: placements cross as `kinds: &[u8]` + `poses: &[f32]` (stride 3 or 4). This fits the bridge's existing bounded copied-lane rule.
- **SolidJS 1.9.15 + Kobalte 0.13.12 + Tailwind/CVA**: level select, tool tray, HUD chips and result card.
- **Pointer Events + `unprojectPoint`**: paused-state tool hit-testing and dragging in world meters, written as pure TS geometry.
- **Guarded `localStorage`** (the `hud-sliders-preference.ts` provider pattern): one versioned key, `liquidfun.puzzle-progress.v1`, with an in-memory fallback. Unlocks are derived, not stored.
- **Web Audio (hand-written)**: noise buffer → biquad → gain for splashes, an oscillator pitch drop for impacts, sine partials for the chime. The `AudioContext` is created only inside the unmute click.
- **WebGL2 (existing `webgl-particles.ts`)**: GPU water adds MRT, refraction from a static background texture, Blinn-Phong specular, Beer-Lambert tint and speed-driven foam. Split into `webgl-water.ts` with a shared field pass.
- **Vitest / Playwright / native Rust tests**: pure-core unit tests, synthetic-clock solution e2e, and reference-solution replay tests in the existing `liquidfun-wasm` test binary. Adding a binary costs about 2 minutes of macOS launch scan.

### Expected Features

**Must have (table stakes):**

- Cup goal that counts qualifying particles (filtered by group or flag) by centre inside the interior region, with a **fixed authored per-level denominator** and a live fill %.
- Held-target debounce (the settle window) so a splash through the cup cannot trigger a false win.
- **Step-based timer** shown as `step / 60` that runs only during Play. Build mode is untimed.
- Win and fail overlays ("Out of time: 54% of 80%") with Next/Retry/Edit tools. The run ends early once the outcome is settled.
- Fill meter at the cup with a target line and star ticks, plus HUD fill and timer chips (`data-hud-obstacle` on each leaf, `tabular-nums`).
- Per-level tool bin with counts. Fixed sizes, with each size as its own tool type. Place, move (keeping the grab offset), rotate (τ/24 snap), remove. Red invalid ghosts, and Play refused while anything is invalid.
- Explicit Build and Run modes. **Stop/Reset returns to Build with placements kept.** A separate, secondary "Clear tools" action.
- Three stars with criteria visible before Play: ★ Win (target held), ★ Brim (bonus fill held), ★ Thrifty (won with at most par tools). The best result is never lowered.
- `#/puzzles` level select (5–8 cards: title, "teaches X" line, stars, lock state) and `#/puzzle/<id>`, with linear soft unlocks.
- One idea per level with a one-line caption. Level 1 is nearly trivial and shows a one-time dismissible ghost-hand hint.
- Phone support: a portrait frame per level (about 390×844), a bottom tray in portrait and side tray in landscape, 44 px hit areas, tap-to-place, `touch-action: none`, and no tilt, pointer gestures or zoom in puzzles.
- Opt-in sound: muted by default, a persisted toggle, a win chime, cup "plinks", impacts, a voice cap and rate limit, and suspend when the tab is hidden.

**Should have (differentiators; pick a few):**

- Real LiquidFun materials as the lessons (viscous, powder, elastic, buoyancy). No casual puzzle game ships this.
- A determinism promise shown to players: "Your solution always works the same way."
- Rising-pitch plinks tied to fill %. This is the cheapest high-value sound cue.
- Fast-forward 2×/4×, which is safe because the timer is step-based. It fixes The Incredible Machine's most-cited complaint.
- A hint ladder (text nudge, then a ghost of one tool from the reference solution).
- Keyboard placement parity (arrows nudge, R/Shift+R rotate, Delete removes, Space plays). This also makes e2e tests easier.
- Glass-cup art with an etched target line. GPU water is the owner-planned closing phase.

**Defer (later milestones):**

- Last-run water trail ghost, two-finger rotate, and Sugar Sugar-style color-filter or multi-cup goals.
- Level editor, share links and a level JSON schema (SEED-002). Keep the placement data serializable so they can be built later.
- Anti-features that must not creep in: live poking or tapping during Run, a wall-clock timer, star-gated unlocks, lives/energy/hint currency, accounts or leaderboards, freeform resize, unlimited tools, pixel-perfect levels, and win-on-first-touch.

### Architecture Approach

Add a `puzzle/` module next to `scene/` in `crates/liquidfun-wasm`. It contains a declarative `const LevelSpec` table, one generic builder that returns the existing `BuiltScene`, `tools.rs` (parse, validate, canonical-order body creation), a pure `goal.rs` (count plus outcome latch), `valve.rs` (a kinematic gate using the Hydraulic Fountain pattern), and `PuzzleHooks: SceneHooks`. The goal runs in `on_after_step`. `SessionCore` replaces its `SceneId` field with `SessionSource { Scene, Puzzle }`. `ProofSession` gains `newPuzzle(levelId, kinds, poses)` and `levelInfo()`, and frames gain optional puzzle and audio scalars. Puzzles use the **direct (main-thread) backend**. The worker currently hard-codes Tesla Valve, so placements only cross JS to WASM. While editing, tools are TS ghosts drawn over the step-0 frame, with no WASM call per drag. Play and Reset construct a fresh session. On the web side, a `PlayableTarget` union replaces `maybeReadySceneId`, and its scene arm runs today's code unchanged. A pure `reducePuzzle` phase machine overlays the existing player view, and the puzzle HUD mounts only on puzzle routes.

**Major components:**

1. **`LevelSpec` + builder (Rust)**: walls, liquid blocks, cup interior, target, full count, step limit, settle steps, tool budget, build zones and no-place zones. TS reads the geometry back through `levelInfo()`, so there is one source of truth. A Rust `include_str!` test checks `PUZZLE_IDS` against the Rust allowlist.
1. **Goal counter + outcome latch (Rust, pure)**: `count_in_goal(positions, filter, region)` and `GoalTracker::observe(count, step) -> Running | Won{step} | Lost`, copied into the frame by `capture_frame`.
1. **Placement validator + tool creation (Rust)**: the authoritative check for kind allowlist, budget, build zones, no-place zones inflated by one particle diameter, tool-tool overlap and finite values. It fails closed with a typed error.
1. **Puzzle store + editor (TS)**: phases `editing → running ⇄ paused → won | lost`, pointer routing (editor while editing, nothing to WASM during Run unless a valve action is chosen), and star scoring in `stars.ts`.
1. **Lifecycle, routing and bounds (TS)**: `#/puzzles` and `#/puzzle/<id>` route kinds, `worldBoundsForTarget`, `PUZZLE_PORTRAIT_BOUNDS` with a fill test, and scene-only chrome (controls, export pane, tilt, gravity) hidden.
1. **Progress, Audio and GPU water (TS)**: downstream consumers only. None of them write back to the simulation.

### Critical Pitfalls

1. **Wall-clock or frame-based timer and win**: define limits in steps and latch the outcome in Rust after each `world.step`. Never raise `MAX_ADVANCE_STEPS` (it is asserted at 4). Grep the puzzle path for `performance.now`, `Date.now` and `setTimeout`.
1. **Creation order and live-world edits**: keep placements as a canonical list sorted by tool-slot index, do a full rebuild at step 0 on Play and Reset, and never call `destroy_body` from editing code. Create the elastic "sponge" group last. Regression: an identical fingerprint after "place, remove, re-place, reset ×3".
1. **Native-vs-wasm divergence**: `Rot::new` uses std `sin`/`cos`, which is platform libm natively and musl-derived `libm` on wasm32. Run a spike at the start of the core phase. If results differ, do not touch engine trig, which would change all 25 fingerprints. Instead snap angles to a τ/24 table, prefer static or kinematic tools, require solutions to win with a margin of about +5% fill, and replay solutions in Playwright as well as natively.
1. **Levels added to the scene catalog**: this breaks README plan coverage, `SURVEY_SCENES: [_; 25]`, portrait and HUD tests, and the scene sidebar. `scene-records.ts` is also already at 617 of the 628-line limit. Use a separate registry on both sides.
1. **Goal counting edge cases and unproven levels**: count by centre in the interior only, after the step, filtered by group. Use an authored `full_count` that eviction does not change, clamp the display to 100%, and transform the region if the cup moves. For every level, test that the 1★ and 3★ solutions win with margin, that zero tools loses, and that a ±1 grid nudge sweep mostly still wins.
1. **Also high-impact**: overlap at Play start (spawn explosions, cup lids, slivers; minimum tool thickness of 3 particle diameters, polygons not edges). Tilt is re-applied to every new session by `reapplyStoredTiltGravity`. The HUD refits the camera on Play (keep the same obstacle set and swap content inside fixed slots). Storage-blocked lockouts: the `window.localStorage` getter itself throws, and Safari's 7-day storage wipe applies.

## Implications for Roadmap

ARCHITECTURE.md and PITFALLS.md arrived at essentially the same dependency order on their own, and FEATURES.md's dependency graph agrees: **core loop → shell/route → tools → levels → progress/select → sound → GPU water.** Phases continue after 36.

### Phase 37: Puzzle Core in the Bridge

**Rationale:** Everything else reads the outcome and frame contract: the HUD, stars, progress, sound and solution tests. This is also the riskiest determinism work, so it should be retired first.
**Delivers:** the `puzzle/` module, `LevelSpec` and builder, the `SessionSource` refactor, `newPuzzle` (empty placements allowed), `levelInfo()`, the pure `count_in_goal` and `GoalTracker` with settle window and step limit, the frame puzzle scalars (`goalCount`, `goalTarget`, `goalFull`, `outcome`, `outcomeStep`, `stepLimit`) with TS parsing, one plain-water level, a reference-solution test harness (using a hard-coded native placement), and the **native-vs-wasm determinism spike** with its result recorded.
**Addresses:** GOAL, TIME, WIN (logic side).
**Avoids:** Pitfalls 1, 3 (spike), 4 (Rust side) and 5.
**Gate:** 25/25 scene fingerprints identical, plus Arrange/Act/Assert unit tests for the boundary, rim, outside-wall, empty and eviction cases.

### Phase 38: Puzzle Route and Shared Player Shell

**Rationale:** Tools should be built inside the real player, not in a test harness. The route and lifecycle change is the main regression risk to the 25 scenes, so it gets its own phase.
**Delivers:** the `#/puzzle/<id>` route kind (`#/puzzles` can be stubbed), the `PlayableTarget` lifecycle, `worldBoundsForTarget`, `PUZZLE_PORTRAIT_BOUNDS` with a fill test, the puzzle HUD (fill meter at the cup with target line, step timer, result card) using fixed obstacle slots, scene-only chrome hidden, tilt/gravity/pointer gestures/zoom disabled on puzzle routes, and a useful fallback for `#/puzzle/<unknown>`.
**Uses:** the existing camera, safe frame, `ScaleLegend` and `hash.ts`.
**Avoids:** Pitfalls 4 (TS side), 8 (input routing) and 10.
**Gate:** `just web-player-smoke` unchanged, existing hash tests untouched, and an e2e check that the camera scale does not change across Play.

### Phase 39: Place-then-Play Tools

**Rationale:** Most levels cannot be solved without tools, and the canonical placement model must be settled before any level is tuned.
**Delivers:** Rust `tools.rs` (validation, canonical order, static ramp and pin tools), the TS store and `reducePuzzle`, the editor (tap-to-place, drag with grab offset, rotate in τ/24 steps through buttons or a handle, remove), the ghost overlay, the pointer router, Reset keeping placements and Clear tools, `stars.ts`, keyboard parity, and a Playwright `hasTouch` project.
**Addresses:** TOOL, PLACE, MODE, STAR (scoring).
**Avoids:** Pitfalls 2, 7 and 9.
**Gate:** the permuted-edit-history fingerprint test, validator unit tests, a synthetic-clock e2e that places a tool and wins, and new reset-honesty assertions.

### Phase 40: Valve and the Teaching Level Set

**Rationale:** Needs tools (Phase 39) and the goal counter (Phase 37). Tuning is empirical and is the largest schedule risk.
**Delivers:** the valve mechanism (see the open decisions), then 5–8 levels: plain water, viscous, powder, elastic "sponge", buoyancy, valve, and a finale combining at most three earlier ideas. Each level gets a caption, portrait frame, `full_count` calibration, counting filter, a 1★ and 3★ fixture, a zero-tools-loses test, a nudge sweep, and a particle budget of about 1.5–3k for main-thread phones. At least one solution is replayed in Playwright.
**Avoids:** Pitfalls 3 (dual-path replay), 6 and 14.

### Phase 41: Level Select and Progress

**Rationale:** Needs stable level ids and the star schema. It is a clean, testable storage boundary. It could merge into Phase 40 if small, but the research recommends keeping it separate.
**Delivers:** the `#/puzzles` grid, a Puzzles section in `DemoNavigation`, `progress.ts` (versioned key, strict validation, null-prototype or `Map` objects, best-of merge per criterion, in-memory fallback), soft unlocks, Next level, and an optional saved last-placement set per level keyed by level revision.
**Avoids:** Pitfall 11.
**Gate:** unit tests for a throwing getter, a throwing `setItem`, corrupt JSON and a future version, plus a blocked-storage e2e.

### Phase 42: Opt-in Sound

**Rationale:** Purely downstream. It needs outcome edges (Phase 37) and real level content to tune against (Phase 40).
**Delivers:** bridge impact tallies (Begin-contact count and strength, accumulated per batch; fingerprint gate), a pure `audio/signals.ts` mapping, an injectable synth, the mute toggle, the chime, cup plinks with rising pitch, impacts, a voice cap of about 6, rate limits, suspend on hidden tabs and route changes, and a "No sound? Check the silent switch." hint.
**Avoids:** Pitfall 12.

### Phase 43: GPU Water Render Mode (closing)

**Rationale:** Independent of puzzle logic and the riskiest on device. It can slip without blocking the game.
**Delivers:** an optional `particleSpeeds` lane filled only when the mode is active (fingerprint gate), an extracted shared field pass, `webgl-water.ts` (MRT, refraction from a background texture uploaded once, specular, tint, foam, `highp`), the `"water"` render mode with the fallback chain water → shaded-blob → canvas, a repaint of the held frame on context restore, exports mapped to `solid`, and puzzle overlays drawn on top.
**Avoids:** Pitfall 13.

### Phase Ordering Rationale

- The goal latch and frame contract come first because every downstream feature consumes them, and determinism is cheapest to fix before levels are tuned against it.
- The shell comes before tools so the editor is built inside the real player with real HUD obstacle and safe-frame behavior.
- Tools come before levels because the canonical placement model and validator define what a solution is. Re-recording fixtures after a model change is the main recovery cost.
- Phases 37, 42 and 43 touch the bridge or frame contract. Each re-runs `just playground-scene-spot` and requires 25/25 identical fingerprints.
- Sound and GPU water only read frames, so they carry no game-correctness risk and go last.
- **Recommended quick task before or alongside Phase 38:** accept either `EXT_color_buffer_float` or `EXT_color_buffer_half_float` in `webgl-particles.ts` and rely on the existing `checkFramebufferStatus`. This is an existing bug that probably affects today's shaded-blob mode on iOS, and the fix is small. The alternative is to fold it into Phase 43. Either way, it needs verification on a device.

### Research Flags

Phases likely needing `/gsd-research-phase` or a spike during planning:

- **Phase 37:** the native-vs-wasm determinism spike, whose result shapes the angle-snapping and margin policy for every level.
- **Phase 39:** touch rotation (selection-bar buttons vs a visible handle) and the validator parity approach (a TS mirror with a parity test vs calling the Rust validator on pointer-up).
- **Phase 40:** empirical level tuning and playtesting, valve mechanics, sponge counting and filtering, buoyant moving-cup region transforms, and per-level particle budgets.
- **Phase 43:** iOS float render targets, `mediump`/`highp` behavior, refraction background cost and context loss. Spike on a phone GPU.

Phases with standard patterns (skip research):

- **Phase 38:** routing, lifecycle and portrait frames follow established repo patterns.
- **Phase 41:** follows the guarded-storage pattern from `hud-sliders-preference.ts` and `render/mode.ts`.
- **Phase 42:** the Web Audio recipes are well documented. Only an iOS manual check is needed.

## Open Decisions for the Owner

These are not reopened owner decisions. They are new questions the research raised. Recommendations are given per researcher, and the synthesis recommendation is in **bold**.

| Decision | STACK | FEATURES | ARCHITECTURE | PITFALLS | Synthesis recommendation |
| --- | --- | --- | --- | --- | --- |
| **Valve model** | Authored timed or threshold behavior; place-then-play only. Live control only through a recorded `applyAction`. | Configured in Build: a timed gate (delay chosen while building) or a passive one-way flap. Live tapping not recommended. | Level valve opened by a tap or HUD button through `applyAction("valve-open")` with a `(step, action)` log. A timed valve (open time as a 4th pose value) is the input-free alternative. | Scheduled tool (open step set while paused) or passive. Live toggling only if recorded and stored in the fixture, which is extra scope. | **Timed valve set in Build (4th pose value, open step snapped). No live tapping in v1.5.** 3 of 4 favor it, and it keeps solutions input-free. |
| **Sponge semantics** | (not addressed) | LiquidFun elastic groups deform but do not absorb. Teach it as a squishy deflector or "jelly pad", or fake absorption with destroy-in-shape. | Decide the counting class per level (liquid vs sponge particles). | Elastic particle group, created last in canonical order, excluded from the count. | **An honest elastic deflector (keep "sponge" as a name or rename it "jelly pad"), excluded from counting. No absorption mechanic.** |
| **Star merge across runs** | Stores `bestStars`, `bestFillPct`. | Best-of per criterion across runs (recommended), or all three in one run. | Stores best stars, fill and fewest tools. | (not addressed) | **Best-of merge per criterion across runs.** |
| **Soft vs star-gated locks** | Unlocks derived from solved levels plus order. | Linear soft locks. Deep links always open with a "skipped ahead" note. Star-gating is an anti-feature. Alternative: fully open with recommended-order badges. | "An unlock rule" (unspecified). | Soft unlock. Deep links always work. | **Linear soft locks derived from solved levels. Deep links always open.** |
| **Hints cost** | (not addressed) | Free and star-neutral (recommended), or hint use blocks Thrifty. | Hint text in TS metadata. | (not addressed) | **Free and star-neutral. Hints are a P2 nice-to-have.** |
| **Storage-blocked unlock behavior** | In-memory progress for the session, all levels playable, optional "progress won't be saved" note. | Fully playable; deep links always open. | In-memory copy when storage throws. | **Unlock every level** when storage is unavailable. | **Unlock all levels and show a one-line "progress won't be saved" note.** |
| **Win hold window** | Track contiguous steps at or above target. | About 0.5–1 s of sim time. Latch Won once held. The run ends when Brim is held or at the timer. | `settle_steps` of about 30. | 30 consecutive steps (0.5 s). UX: a non-blocking win banner; let the run finish. | **30 steps (0.5 s), authored per level in `LevelSpec`. Won latches (later slosh does not cost the win). End the run at Brim-held or timeout, showing a non-blocking banner first.** |
| **Native-vs-wasm determinism spike** | (not addressed) | (not addressed) | (not addressed) | Required at the start of P-Core. If outputs differ, use the angle table, margins and browser replays, and leave engine trig alone. | **Run it as the first task of Phase 37 and record the result in the phase summary.** |

Smaller design choices where the researchers diverged, with the synthesis pick:

- **Placement transport:** use ARCHITECTURE's `newPuzzle(id, kinds, poses)` constructor on the direct backend rather than STACK's `set_placements` worker op. The worker currently serves only Tesla Valve.
- **Validator location:** Rust is authoritative (all four agree). Avoid duplicating geometry by reading zones from `levelInfo()`. A TS mirror for live red/green feedback is acceptable only with a parity test.
- **Zoom/pan in puzzles:** FEATURES and PITFALLS want a fixed per-level frame and no zoom. ARCHITECTURE says zoom is reused as-is. **Disable zoom/pan on puzzle routes.**
- **Rotation on touch:** no two-finger twist (STACK, FEATURES MVP and PITFALLS agree). Use τ/24 snap with buttons or a handle outside the tool body.
- **Goal region shape:** a convex interior polygon (a rectangle is fine for most cups), counted by centre. Use the cup body's transform if the cup moves.
- **Naming:** use the `liquidfun.puzzle-progress.v1` key, which matches existing `liquidfun.*.v1` keys, and the routes `#/puzzles` and `#/puzzle/<id>`.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH | No new dependencies, verified against repo source and npm/crates.io on 2026-10-11. iOS audio-session and float-target details are MEDIUM and device-dependent. |
| Features | MEDIUM | Genre conventions agree across 6 reference games, but formal design write-ups are sparse. Repo-coupled claims are HIGH. |
| Architecture | HIGH | Integration points were read from current HEAD `153669d86`: worker hard-coding, `SessionCore` structure, `SCENE_IDS` consumers. Performance assumptions (world construction time, particle budgets) are not measured yet. |
| Pitfalls | HIGH / MEDIUM | Code-grounded pitfalls are HIGH (clock caps, tilt re-apply, file-length limit, extension gate). Browser-platform items are MEDIUM. Native-vs-wasm divergence is a confirmed mechanism with an unmeasured effect. |

**Overall confidence:** HIGH for the approach and phase order. MEDIUM for level feel, phone UX details and on-device GPU/audio behavior.

### Gaps to Address

- **Native-vs-wasm bit equality:** unmeasured. The Phase 37 spike decides how strict the angle-snapping and margin policy must be.
- **World construction time on Play/Reset:** unmeasured. Measure it on the largest level. Fallback: add tools to a fresh step-0 world in canonical order. SEED-006 already lists grouped-creation load time as a v1.4 leftover.
- **Main-thread step cost per level on phones:** budget about 1.5–3k particles per level. Move to the worker backend only if needed, and then add one non-synthetic-clock worker e2e.
- **iOS `EXT_color_buffer_float` vs half-float:** WebKit reports conflict. Verify the shaded-blob fallback on a real iPhone before and after the fix.
- **iOS silent switch:** Web Audio follows it by default. Keep that default and show a hint. Do not force `navigator.audioSession.type = "playback"`. Needs a manual device check.
- **Safari 7-day storage wipe:** current iOS behavior was not re-verified. The UI should say progress is per browser and may be cleared.
- **Timer vs real seconds on slow devices:** step-time runs slower than wall time when steps are budget-limited. Label it as simulation time, or use a progress bar.
- **Win-time UX:** the run ends at Brim but the player may want to keep watching. Settle this in Phase 38 or 39 UX review.

## Sources

### Primary (HIGH confidence)

- Repository source at `153669d86`: `crates/liquidfun-wasm/src/{lib,session,frame,scene,scene_spot}.rs`, `session/escape.rs`, `scene/hydraulic_fountain.rs`. `crates/liquidfun/src/{particle/query.rs, collision/shape/polygon.rs, world/particle_coupling.rs, math/transform.rs, joint/definition.rs}`. `web/src/physics/{clock,simulation-worker,worker-messages,frame}.ts`. `web/src/player/{scene-lifecycle,playground-session,hud-obstacles,hud-sliders-preference}.ts`. `web/src/render/{webgl-particles,present-frame,mode,safe-frame,static-layer}.ts`. `web/src/routing/hash.ts`, `web/src/catalog/{scenes,portrait-bounds,scene-records}.ts`. `web/src/input/{canvas-pointer,pointer,tilt-binding}.ts`, `web/playwright.config.ts`, `scripts/bright-builds-check.ts`, `tools/xtask/src/playground/spot.rs`.
- npm registry and crates.io API, 2026-10-11: current versions of solid-js, Kobalte, the rejected libraries, and wasm-bindgen 0.2.129.
- [Chrome: Web Audio, Autoplay Policy and Games](https://developer.chrome.com/blog/web-audio-autoplay): the context must be resumed from a user gesture.

### Secondary (MEDIUM confidence)

- Genre references: Hardcore Gaming 101 and Andy Matuschak on The Incredible Machine, Fantastic Contraption, Poly Bridge, Cut the Rope wiki and Arkadium, Where's My Water coverage, and MobyGames' Sugar Sugar page. Covers the parts bin, Stop keeping parts, 3-star models, the 100-grain cup and the pixel-perfect and no-fast-forward criticisms.
- [MDN Autoplay guide](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Autoplay), [MDN AudioSession](https://developer.mozilla.org/docs/Web/API/AudioSession), WebKit bugs 237322 and 264473: autoplay rules and the iOS silent switch.
- WebKit bugs 217107 and 247240: iOS 16-bit float render targets.
- rust-lang/compiler-builtins PR #248 and the `libm` crate: wasm32 math comes from a separate musl-derived libm.
- Safari 7-day script-writable storage cap (Search Engine Land, M. Tsai, WebKit bug 237350).

### Tertiary (LOW confidence)

- web3dsurvey extension statistics for `EXT_color_buffer_float` and `EXT_float_blend` (methodology unclear).
- Training-data knowledge on mobile `mediump` being fp16 and per-page WebGL context limits. Validate on a device.
- A single developer account on touch drag offset (Game Developer).

---
*Research completed: 2026-10-11*
*Ready for roadmap: yes*
