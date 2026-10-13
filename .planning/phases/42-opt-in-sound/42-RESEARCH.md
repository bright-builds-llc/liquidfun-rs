# Phase 42: Opt-in Sound - Research

**Researched:** 2026-10-13
**Domain:** Read-only bridge aggregates (Rust/WASM) + Web Audio synthesis shell + SolidJS HUD toggle
**Confidence:** HIGH for the code-grounded integration (bridge, frame contract, loop, lifecycle, HUD, tests); MEDIUM for tuning thresholds (splash churn, impact audibility) and iOS behaviour

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Toggle, default and persistence (SOUND-01)

- **D-01:** One speaker toggle button in the puzzle HUD. It sits inside the existing `puzzle-options-slot` (which carries `data-hud-obstacle`) next to the Puzzle options trigger. It is 44 px, matches the `hud-button` styling and has a fixed box size, so the obstacle set never changes when it toggles. It is a real `<button>` with `aria-pressed` and an accessible name of "Sound" (the icon shows a speaker with or without sound waves). Puzzle routes only. Scene routes and the `#/puzzles` grid get no sound control.
- **D-02:** The Puzzle options drawer gets a "Sound" section. It has a checkbox or switch that mirrors the HUD toggle (one shared state) and the silent-switch hint (D-11).
- **D-03:** The preference is stored under a new versioned key, `liquidfun.puzzle-sound.v1`. Only the exact token `"on"` means on. A missing, unreadable or any other value means off. Follow the provider and try/catch pattern in `web/src/player/first-hint-preference.ts`. Keep this separate from `liquidfun.puzzle-progress.v1` and the first-hint key. Blocked storage or a throwing `setItem` keeps the choice in memory for the page session, and the toggle keeps working.
- **D-04:** No `AudioContext` exists until the player has opted in **and** a user gesture has happened:
  - Turning sound on through the toggle creates the single page-session context (or resumes it) inside that click handler.
  - If the stored preference is "on" when a page loads, the toggle shows as on, but no context is created and nothing plays until the first trusted `pointerdown` or `keydown` on a puzzle route. That gesture creates and resumes the context.
  - Turning sound off suspends the context and stops every voice. The context is reused if the player turns sound on again. It is never closed during the page session.
- **D-05:** If `AudioContext` (or `webkitAudioContext`) is unavailable or creation throws, the toggle renders disabled with an accessible name like "Sound unavailable in this browser", in the same box. The game keeps working. Errors surface in the console and are not swallowed silently.

#### Sounds and signals (SOUND-02)

- **D-06:** Four voices, all synthesized with plain Web Audio nodes. Use no assets, no Tone.js, Howler or zzfx dependency, and no other audio library. Vendoring zzfx is the fallback only if hand tuning stalls (STACK.md).
  - **Splash:** a short noise buffer, generated once per context, through a bandpass `BiquadFilterNode` and a fast attack/decay `GainNode`. Intensity scales gain and filter frequency.
  - **Impact:** a short sine or triangle `OscillatorNode` with a pitch drop and exponential gain decay. Strength scales gain.
  - **Plink:** a short sine blip when particles enter the cup. Pitch rises with fill, `goalCount / goalTarget`, clamped to 0..1. It steps along a major-pentatonic scale over about two octaves, so the player hears progress toward the target. Several entries in one presented frame make one plink.
  - **Win chime:** a three-note major arpeggio of sine partials. It plays once when the outcome changes to `won`.
  - No loss sound.
- **D-07:** Signals:
  - **Plinks** come from the change in the existing `goalCount` puzzle scalar between presented frames. This needs no bridge change.
  - **Chime** comes from the outcome transition already in `PuzzleState` / the puzzle status.
  - **Splash and impact** need new **read-only** per-step aggregate scalars on puzzle frames from the bridge, for example particle-on-body contact activity for splashes and rigid-body contact impulse or new-contact strength for impacts. They follow the optional, fail-closed puzzle scalar pattern in `crates/liquidfun-wasm/src/frame/puzzle.rs` and `web/src/physics/frame-puzzle.ts`. Research picks the exact source: existing `body_contact_count` and particle statistics, `StepReport` contact solves, or observation data.
  - Constraints on these signals:
    - Reading them must not change stepping, contact order or any float result.
    - Prefer using data the step already produces, and do not modify `crates/liquidfun`. If a change there proves necessary, it must be additive and read-only, and the planner must record why.
    - Scene frames keep their current shape. Use puzzle-only getters, or getters that are `undefined` on scene sessions.
    - Signals cover the steps since the previous capture. The puzzle loop captures one step at a time, so per-step aggregates are enough.
- **D-08:** The sound engine is a pure consumer of presented frames and puzzle state. It is called from the puzzle loop's publish path **after** dispatch and draw. It never calls into the session, never changes `PuzzleState`, and never affects timing or the step budget. A failure in the audio layer must not fail the puzzle run. Log it and turn sound off for the session; do not throw into the frame loop.
- **D-09:** Voice cap and rate limits:
  - At most **6** simultaneous voices overall.
  - Per-voice minimum intervals: about 80 ms for splash, about 60 ms for impact, about 50 ms for plink.
  - The win chime always plays. It may steal the oldest voices.
  - A `DynamicsCompressorNode` and a master gain of about 0.5 sit before the destination, so bursts don't clip.
  - These are tunable constants at Claude's discretion. The cap and the limits must exist and be unit-tested.
  - Sounds play only during a run (from Play until the outcome is held). Building is silent. After the win chime, no new splash, impact or plink voices start during the settle tail. Under fast-forward, triggers are evaluated per presented frame, so the rate limits and coalescing keep 2× and 4× from becoming noise.

#### Pausing and the silent-switch hint (SOUND-03)

- **D-10:** Hiding and leaving:
  - On `visibilitychange` to hidden, suspend the context and stop active voices. On visible, resume only if sound is on and the route is still a puzzle route.
  - Leaving the puzzle route through the existing `leavePuzzle` / teardown path suspends the context and stops all voices. The context is kept for reuse.
  - Stop, Edit and Retry stop any active voices immediately.
  - No timers or scheduled audio may survive a route change or a newer build. Respect the loop's `isCurrent()` generation guard.
- **D-11:** The hint reads "No sound on iPhone? Check the silent switch." (exact wording at Claude's discretion). It shows in the drawer's Sound section next to the mirror switch, and it is the HUD toggle's `title` or description. Show it on every platform; do not sniff the user agent. Do not set `navigator.audioSession.type` (it pauses the user's other audio, and it is not Baseline).

#### Determinism and verification (criterion 4)

- **D-12:** The new bridge scalars are read-only getters. Native Rust tests must show:
  - each level's reference-solution replay has the same outcome, outcome step and frame fingerprint as before the change (existing pins stay green unchanged)
  - the new scalars are deterministic across runs
  - the 25 scene fingerprints are unchanged (`just playground-scene-spot` or the existing fingerprint test path, 25/25 identical)
- **D-13:** A Playwright test injects a fake `AudioContext` with `addInitScript` and checks that:
  - no context is constructed on page load or before the toggle, including with a stored "on" until a gesture
  - the toggle persists across a reload
  - the context is suspended on hidden and on leaving the puzzle
  - a level reaches the same outcome and outcome step with sound on and off
- **D-14:** Vitest covers the pure core:
  - preference parse and persist
  - fill-to-pitch mapping and the scale step
  - signal-to-voice-parameter mapping
  - rate limiter
  - voice cap with chime stealing
  - trigger rules (run phase only, no voices after the win, one plink per frame)

  The Web Audio shell takes an injectable context factory, so tests need no real `AudioContext`.

### Claude's Discretion

- Exact synthesis parameters, scale root, envelope times, gains, filter frequencies, and the numeric rate limits and cap (within D-09).
- Module layout under `web/src/audio/` (for example `sound-preference.ts`, `voices.ts`, `sound-engine.ts`, `triggers.ts`) and component names (for example `PuzzleSoundToggle`).
- The exact bridge scalar names and the aggregate definition for splash and impact (within D-07).
- The icon artwork, and whether the drawer control is a checkbox or a switch.

### Deferred Ideas (OUT OF SCOPE)

- Sound for the 25 catalog scenes (SEED-005 mentions Liquid Bubbler, Stacked Drip, Dam Break and Sparky). That would be its own phase.
- Very quiet UI ticks for place, rotate snap, remove and Play (FEATURES.md table stakes, not in SOUND-01..03).
- A volume slider, and a loss sound.
- Opting in to `navigator.audioSession.type = "playback"` behind a setting.
- (Domain out of scope also lists: sample assets or audio libraries, GPU water (Phase 43), new levels.)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SOUND-01 | Sound is off by default; player can turn it on with a toggle that is remembered. | `sound-preference.ts` mirrors `first-hint-preference.ts` (provider + try/catch + lazy memory); toggle in `puzzle-options-slot` (CSS width change required, see Pitfall 4); context created lazily via injectable factory only in a trusted gesture (Pattern 4); Playwright fake `AudioContext` (Code Examples) proves zero constructions before opt-in. |
| SOUND-02 | With sound on: splash and impact sounds, rising-pitch plinks as the cup fills, a win chime, a cap on simultaneous sounds. | Bridge `SoundTally` (new `puzzle/sound.rs`) produces `splashContacts`, `impactCount`, `impactImpulse` per step from data the step already produces (`ParticleSystemView::body_contacts`, `StepReport::contact_transitions/contact_solves`); TS folds per-step signals across one presented frame in `puzzle-loop.ts`; pure `triggers.ts` / `voices.ts` / `voice-pool.ts` decide voices; chime on the won edge (`isDeciding` or ended-won). |
| SOUND-03 | Sound pauses when the tab is hidden or the player leaves the puzzle; a hint mentions the phone silent switch. | Visibility + gesture listeners scoped to the puzzle panel lifetime; `stopAll()` hooked into `incrementGeneration` (covers Play/Stop/Edit/Retry/level change) plus `leavePuzzle` / `releaseFailedPuzzle` suspend; hint in drawer + `aria-describedby`/`title` on the HUD toggle. |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- New on-canvas chrome carries `data-hud-obstacle` on its **leaf** element with a camera-independent fixed box; `puzzle-options-slot` is the leaf (the toggle itself must NOT carry `data-hud-obstacle`; `tests/puzzle-chrome.test.ts` forbids it in `PuzzleOptionsDrawer.tsx`).
- Simulation stays renderer-independent; audio is a pure downstream consumer; no engine change may alter the 25 scene fingerprints (STATE.md v1.5 decision). Phases touching the bridge/frame contract need 25/25 identical fingerprints.
- `crates/liquidfun` stays untouched (prior phases gate `git diff --stat $BASE -- crates/liquidfun` empty). This research found **no reason to touch it**.
- Prior phase gates also kept `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/scene/`, `web/src/styles/canvas-stage.css`, `web/src/components/CanvasStage.tsx` and protected tests (`web/tests/{hash,portrait-bounds,readme-svg}.test.ts`, `web/e2e/{hud-safe-frame,shell}.spec.ts`) unedited. The recommended design touches none of them.
- Rust: no `unwrap()` outside tests, `#[must_use]` on getters, docs on every item, `clippy::pedantic -D warnings`, `maybe_` names for `Option`, `foo.rs` + `foo/tests.rs` (never `mod.rs`), keep new tests inside the existing `liquidfun-wasm` lib test binary (each new macOS test binary stalls ~2 min on first launch; run cargo sequentially).
- TS: `maybe` prefix for nullable names, functional core / imperative shell, Arrange/Act/Assert comments, one concern per test, no class inheritance (a factory function returning an object is the house style), early returns.
- File-length trigger ~628 lines (`bun scripts/bright-builds-check.ts all`). Watch `session.rs` (572), `frame.rs` (534), `PuzzlePanel.tsx` (445), `puzzle.css` (451).
- Error handling: propagate; D-08 explicitly requires the audio shell to catch, `console.error`, and disable sound for the session rather than throw into the loop (this is a logged, visible degradation, not a silent swallow).
- Puzzle source files may not use `Date.now`, `performance.now`, `setTimeout`, `setInterval`, or `innerHTML` (`tests/puzzle-chrome.test.ts` WALL_CLOCK gate over `PUZZLE_COMPONENTS`; Phase 41 grep gate over `web/src/puzzle`, `web/src/components/Puzzle*.tsx`, several `web/src/player/puzzle-*.ts`). The audio clock is `AudioContext.currentTime`.
- localStorage is allowlisted to specific files (Phase 41 `grep-storage-allowlist` gate). The new sound-preference file adds one entry; the Phase 42 gate script must list it.
- Package manager: bun (`bun.lock`). No new npm packages or Cargo crates.
- Dark-mode default, SolidJS + shadcn-solid shell (standards-overrides.md).
- Never stage `.planning/config.json`; do not tick requirements in REQUIREMENTS.md from executors; commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (gsd-tools commit drops the trailer — use plain `git commit`).

## Summary

The step already produces everything sound needs, and reading it is provably side-effect free. `SessionCore::advance` (`crates/liquidfun-wasm/src/session.rs:254`) receives a full `StepReport` every step and currently only forwards `report.contact_transitions()` to the hooks. `StepReport::contact_solves()` is **always populated** (it is filtered out of the lifecycle list unconditionally in `world/step/execution.rs:289`), so there is no opt-in cost or behaviour switch. `ParticleSystemView::body_contacts()` exposes every particle-fixture contact with stable `ParticleId`/`FixtureId`. `World` read paths have no interior mutability (no `Cell`/`RefCell` in `crates/liquidfun/src`; the only atomics are creation-time key counters), so reading cannot alter stepping, contact order or floats. `crates/liquidfun` needs **no change**.

Recommended bridge scalars, computed per fixed step by a new `SoundTally` owned by `SessionCore` only for puzzle sessions: `splashContacts` = number of particle-fixture contacts present this step that were not present the previous step; `impactCount` = number of non-sensor rigid `ContactTransitionKind::Begin` transitions; `impactImpulse` = the largest summed normal impulse of those new contacts, matched to `contact_solves()`/`continuous_contact_solves()` by `(fixtures, child_indices)`. They ride on `FrameData` as a separate `maybe_sound` field (not inside `PuzzleStatus`, which is constructed literally in five Rust test files and ~18 TS fixtures) and are exposed as `Option` getters in `frame/puzzle.rs`, so scene frames are unchanged (`undefined` in JS). One important finding: **only Float Up has a dynamic rigid body** (`puzzle/builder.rs:126`; every tool and wall is static and valves are destroyed static bodies). Rigid impacts will be audible in one of six levels unless the web layer also maps a splash burst onset to the impact voice (see Open Question 1).

On the web side, the puzzle loop presents the **last** of up to N single-step captures per animation frame (`puzzle-loop.ts` `presentSteps`), so per-step scalars on the presented frame alone would drop intermediate steps under 2×/4×. Fold them in the loop (pure, total, no throw) and pass the fold to `publish`. The sound engine is a factory-built shell (injectable context factory, master gain → compressor → destination, one noise buffer per context) driven by pure modules (`triggers.ts`, `voices.ts`, `voice-pool.ts`, `sound-preference.ts`) that take `nowSeconds` from `AudioContext.currentTime`. Stop-all belongs in `incrementGeneration` (every Play/Stop/Edit/Retry/level change bumps it) plus `leavePuzzle`/`releaseFailedPuzzle`; visibility and gesture listeners live for the puzzle panel's lifetime. The HTML spec makes `pointerdown` an activation-triggering event **only for mouse**; on touch it is `pointerup`/`touchend`, so the stored-"on" unlock must also listen to `pointerup` (Pitfall 1).

**Primary recommendation:** Add `puzzle/sound.rs` (pure aggregates + `SoundTally`) fed from `SessionCore::advance`'s existing `StepReport`, expose three optional `ProofFrame` getters, fold them per presented frame in `puzzle-loop.ts`, and drive a factory-injected Web Audio shell from pure trigger/voice/pool modules; prove determinism with existing pins, a tally-on vs tally-off per-step position comparison, 25/25 scene fingerprints, and a Playwright sound-on/off `RunFingerprint` equality test with a fake `AudioContext`.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Web Audio API (browser built-in) | Baseline | Synthesis: `OscillatorNode`, `AudioBufferSourceNode`, `BiquadFilterNode`, `GainNode`, `DynamicsCompressorNode` | Locked by D-06; zero bundle weight, no asset licensing [CITED: developer.mozilla.org Web Audio API] |
| `liquidfun` (in-repo) | workspace | `StepReport::contact_transitions/contact_solves/continuous_contact_solves`, `ParticleSystemView::body_contacts`, `ContactTransitionKind`, `ManagedContactSnapshot`, `ContactPointSnapshot::normal_impulse` — all `pub` and re-exported from `lib.rs:248-291` | [VERIFIED: codebase grep] |
| `wasm-bindgen` | =0.2.128 (crate pin; do not bump) | `Option<u32>`/`Option<f32>` getters return `undefined` in JS | Existing puzzle getters use the same pattern [VERIFIED: frame/puzzle.rs] |
| SolidJS | 1.9.15 (package.json) | Toggle component, shared signal | Existing shell [VERIFIED: web/package.json] |
| TypeScript | 7.0.2 | DOM lib already has `AudioContextState = "closed" \| "interrupted" \| "running" \| "suspended"`; `webkitAudioContext` is NOT typed (declare a local type) | [VERIFIED: node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts:44248] |
| Rust std `HashSet` + `BuildHasherDefault<std::hash::DefaultHasher>` | std (stable since 1.76; MSRV 1.92) | Previous-step contact set for "new contact" counting with a fixed, reproducible hasher | No new crate; ids are `Eq + Hash` but not `Ord` [VERIFIED: liquidfun/src/identity.rs:171] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | 5.0.1 (`environment: "node"`) | Pure core tests; no DOM, so the shell is tested with a fake context factory object | All D-14 tests [VERIFIED: web/vitest.config.ts] |
| Playwright | 1.63.0 (Chromium) | `addInitScript` fake `AudioContext`, trusted input via `page.mouse`/`page.keyboard`, `setDocumentHidden` helper | D-13 e2e [VERIFIED: web/package.json, e2e/player-helpers.ts:346] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| New-contact set diff | `body_contact_count` delta between steps | Already on frames, but a steady pour onto a ramp has constant count (arrivals ≈ departures), so it is silent exactly when water is flowing. Rejected. |
| New-contact set diff | Flag-gated `StepLifecycleEvent::ParticleBodyContact` events | Needs per-particle listener flags set at build time and consumes the `StepLimits` event budget (risk of `LimitExceeded` → `StepFailed`). Changes particle state. Rejected. |
| `SoundTally` in `SessionCore` | New default method on `SceneHooks` (`scene.rs`) | Cleaner dispatch, but touches `scene.rs`, which prior phases kept untouched by gate. Rejected unless the planner wants it. |
| `maybe_sound` field on `FrameData` | Add fields to `PuzzleStatus` | Breaks literal constructions in `puzzle/hooks.rs`, `goal.rs`, `builder/tests.rs`, `goal/tests.rs`, `frame/puzzle/tests.rs`, and the TS `PuzzleFrameStatus` fixtures in ~18 test files; also entangles audio with `status_is_consistent`. Rejected. |
| Hand-written synthesis | Vendored zzfx (MIT, ~1 KB) | Only if tuning stalls (D-06). |

**Installation:** Nothing to install. No npm packages, no Cargo crates.

**Version verification:** No new packages; versions above read from `web/package.json`, `rust-toolchain.toml` (1.97.0) and the generated TS lib in `node_modules` on 2026-10-13.

## Architecture Patterns

### Recommended Project Structure

```
crates/liquidfun-wasm/src/
├── puzzle/sound.rs            # NEW: SoundSignals, SoundTally, pure count/impulse helpers
├── puzzle/sound/tests.rs      # NEW: determinism, read-only proof, level sanity
├── puzzle.rs                  # MODIFIED: `pub(crate) mod sound;` (alphabetical)
├── session.rs                 # MODIFIED: `maybe_sound: Option<SoundTally>`; observe after on_after_step; attach in annotate_frame
├── frame.rs                   # MODIFIED: `maybe_sound` field + `set_puzzle_sound` (requires maybe_puzzle)
└── frame/puzzle.rs            # MODIFIED: splashContacts / impactCount / impactImpulse getters

web/src/
├── audio/
│   ├── sound-signals.ts       # NEW pure: PresentedSoundSignals, NO_SOUND_SIGNALS, foldSoundSignals
│   ├── triggers.ts            # NEW pure: decideVoices(memory, input) -> { memory, requests }
│   ├── voices.ts              # NEW pure: plink scale, splash/impact/chime parameter mapping
│   ├── voice-pool.ts          # NEW pure: cap 6, prune by endsAt, chime steals oldest
│   └── sound-engine.ts        # NEW shell: factory, master chain, noise buffer, play/stopAll/suspend/resume
├── player/
│   ├── sound-preference.ts    # NEW: liquidfun.puzzle-sound.v1, mirror of first-hint-preference.ts
│   ├── puzzle-sound.ts        # NEW: Solid state (enabled/available signals) + gesture/visibility glue
│   ├── puzzle-runtime.ts      # MODIFIED: `sound` member (degraded no-op default for node tests)
│   ├── puzzle-host.ts         # MODIFIED: incrementGeneration stops voices
│   ├── puzzle-loop.ts         # MODIFIED: fold per-step signals, pass to publish
│   ├── puzzle-run.ts          # MODIFIED: startLoop publish → sound.present(...)
│   ├── puzzle-lifecycle.ts    # MODIFIED: leavePuzzle / releaseFailedPuzzle suspend; controller sound accessors
│   └── playground-session.ts  # MODIFIED: pass the browser sound to createPuzzleRuntime
├── physics/frame.ts           # MODIFIED: optional RawProofFrame getters; RenderFrame.maybeSound
├── physics/frame-sound.ts     # NEW (or inside frame-puzzle.ts): parsePuzzleSound, fail closed
├── puzzle/controller.ts       # MODIFIED: soundEnabled / soundAvailable / setSoundEnabled
├── components/PuzzleSoundToggle.tsx  # NEW: HUD button + drawer section (no data-hud-obstacle)
├── components/PuzzleHud.tsx   # MODIFIED: second child in puzzle-options-slot
├── components/PuzzlePanel.tsx # MODIFIED: pass toggle + drawer section; form-factor control-row too
└── styles/puzzle.css          # MODIFIED: slot 96×44, caption/title offsets, pressed style
```

### Pattern 1: Read-only per-step aggregate in the bridge

**What:** `SessionCore` owns `maybe_sound: Option<SoundTally>`, `Some` only for `SessionSource::Puzzle`. In `advance` (and `advance_profiled`, for consistency), after `on_after_step` succeeds, call `tally.observe(&self.world, self.particle_system, &report)`. `capture_frame` → `annotate_frame` attaches `tally.signals()` after the puzzle status.
**When to use:** Any frame data that must not depend on frame rate.
**Why it is safe:** `observe` takes `&World` and `&StepReport`; no engine API it calls mutates anything; the report would otherwise be dropped. Scene sessions keep `None`, so scene stepping executes byte-identical code paths except one `if let None`.
**Session file length:** `session.rs` is 572 lines; keep the added code to ~15 lines (field, init in `from_built`, two observe calls, one attach). Put all logic in `puzzle/sound.rs`.

### Pattern 2: Separate optional frame group, fail closed

**What:** Rust `FrameData::set_puzzle_sound(signals)` returns `Err(FrameError::InvalidDiagnostic)` if `maybe_puzzle` is `None` or `impact_impulse` is not finite / negative, or `impact_count == 0 && impact_impulse != 0.0`. TS `parsePuzzleSound(rawFrame, maybePuzzle)`: all three getters `undefined` → `undefined`; any partial set → `invalidFrame()`; present without `maybePuzzle` → `invalidFrame()`; counts via `parseNonNegativeInteger`; impulse finite and ≥ 0.
**Fixture impact:** if the planner requires sound scalars on every puzzle frame (stricter), update the two raw-frame fakes that define puzzle getters without sound: `tests/puzzle-session.test.ts:78`, `tests/puzzle-host-fixture.ts:113`. Recommended: require them on puzzle frames (fail closed), and update those two fakes plus `tests/frame-puzzle.test.ts`.

### Pattern 3: Fold per-step signals across one presented frame

**What:** In `presentSteps`, `let signals = NO_SOUND_SIGNALS;` and inside `stepOnce` after dispatch: `signals = foldSoundSignals(signals, stepped.maybeSound);`. Pass `signals` as a 4th argument to `deps.publish`. The fold sums `splashContacts` and `impactCount`, keeps `max(impactImpulse)`, and counts `steps`, so triggers can normalize splash intensity per step (2×/4× do not get louder).
**Why:** The loop captures each step (`session.nextFrame(1)`) but presents only the last; D-07's "per-step aggregates are enough" holds only if every step's aggregate is folded. Existing `tests/puzzle-loop.test.ts:113` passes a 3-parameter `publish`; a 4-parameter type stays compatible.

### Pattern 4: Lazy, gesture-gated single context with an injectable factory

**What:** `createSoundEngine({ maybeCreateContext })` where the browser factory is `() => new (window.AudioContext ?? window.webkitAudioContext)()`. Availability for rendering is `typeof ctor === "function"` (no construction). Construction happens only in `enable()` (called from the toggle's click handler) or `unlockFromGesture(event)` (called from a capture-phase `pointerdown`/`pointerup`/`keydown` listener while on a puzzle route, `event.isTrusted` only, and only when enabled and no context exists or `state !== "running"`).
**Resume handling:** call `resume()` synchronously in the handler; attach `.catch(logAndDisable)`; never `await` before `resume()`. Treat `"interrupted"` (iOS) like `"suspended"`.

### Pattern 5: Stateless-where-possible trigger rules

```
active  = state.kind === "running" || (state.kind === "ended" && !state.held)
won     = (state.kind === "running" && state.status.outcome === "won")
       || (state.kind === "ended"   && state.latch.outcome === "won")
newRun  = memory.maybeLastStepIndex === undefined || frame.stepIndex <= memory.maybeLastStepIndex
chime   = won && !memory.wasWon && !newRun-reset-suppressed
others  = active && !won → splash / impact / plink candidates, then rate limits
plink   = goalCount > memory.lastGoalCount (one per presented frame), pitch from clamp(goalCount / goalTarget)
```
Win latches during `running` (the "Filled!" deciding phase, `isDeciding`), not only at `ended`; a won run keeps running until Brim or the step limit (`reduce-puzzle.ts:stepRunning`). The chime must fire at the first presented frame whose status says `won`.

### Pattern 6: Voice pool with time-based pruning

Prune voices by their scheduled `endsAt <= now` before counting (`onended` does not fire while the context is suspended). Non-chime requests are **dropped** when the pool is full (gentle, D-specifics); the chime evicts the oldest voice(s) to fit. Treat the three chime notes as one pool entry (or three; tunable, but test whichever is chosen).

### Anti-Patterns to Avoid

- **`new AudioContext()` at module load or in a Solid `createSignal` initializer:** violates D-04 and triggers Chrome's "AudioContext was not allowed to start" warning.
- **Per-particle voices or per-step (not per-presented-frame) triggering:** produces walls of noise at 4×.
- **Using `performance.now()` for rate limits:** banned in puzzle files by grep gates; use `context.currentTime`.
- **Suspending before stopping:** suspended contexts freeze `currentTime`; scheduled sources (chime notes) resume later. Always `stopAll()` first.
- **Reading `document.visibilityState` only:** the existing e2e helper overrides `document.hidden`; read `document.hidden`.
- **Iterating a `HashSet` to produce anything:** count by iterating the deterministic `view.body_contacts()` order and testing membership in the previous set.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rigid impact strength | Velocity differencing across frames or body snapshots | `StepReport::contact_solves()` normal impulses of `Begin` contacts | Post-solve velocities already lost the approach speed; the solver impulse is the physical impact measure and is already computed. |
| Clip protection | Manual gain normalization | `DynamicsCompressorNode` + master `GainNode` (~0.5) | Locked by D-09; browser-native limiter. |
| Envelopes | `setTimeout`-driven gain changes | `AudioParam.setValueAtTime` / `exponentialRampToValueAtTime` / `setTargetAtTime` on the audio clock | Sample-accurate, survives main-thread jank, no wall-clock timers. |
| Voice end tracking | Timers | Known `endsAt` from the schedule + `stop(endsAt)`; `onended` only for `disconnect()` cleanup | Timers are banned and do not pause with the context. |
| Visibility simulation in e2e | New helper | `setDocumentHidden` in `e2e/player-helpers.ts:346` | Existing, tested pattern. |
| Run equality in e2e | New attribute reads | `readRunFingerprint` in `e2e/puzzle-editor-helpers.ts:171` (outcome step, decided step, peak held, held hash) and `installSolutionPlacements` | Already proves 1×/2×/4× equality. |
| Storage preference | New pattern | Copy `web/src/player/first-hint-preference.ts` shape | D-03. |

**Key insight:** Every number sound needs is already computed by the step or the frame; the work is reading, folding and rate-limiting, not measuring.

## Common Pitfalls

### Pitfall 1: Touch `pointerdown` is not a user activation

**What goes wrong:** With a stored "on", the first tap creates the context in `pointerdown` but it stays `suspended` on phones; nothing plays.
**Why:** The HTML spec's activation-triggering input events are `keydown` (not Esc or UA shortcuts), `mousedown`, `pointerdown` **only if `pointerType` is "mouse"**, `pointerup` **only if `pointerType` is not "mouse"**, and `touchend` [CITED: html.spec.whatwg.org/multipage/interaction.html#activation-triggering-input-event].
**How to avoid:** Listen to `pointerdown`, `pointerup` and `keydown` (capture phase, passive). Create on the first, and call `resume()` on each trusted event while enabled and `state !== "running"`. This is a small, reasoned extension of D-04's "pointerdown or keydown" wording; record it in the plan.
**Warning signs:** e2e with mouse passes but phone has no sound until the toggle is tapped twice.

### Pitfall 2: Splash churn from resting liquid

**What goes wrong:** Settled water jitters in and out of wall contact, so `splashContacts` never reaches zero and the splash voice hisses through the whole run and tail.
**Why:** The new-contact count measures arrivals, not speed; resting contact flicker is plausible but unmeasured in this session [ASSUMED].
**How to avoid:** Wave 0 probe (test-only, like `puzzle/probe.rs`): print per-step `splashContacts`, `impactCount`, `impactImpulse` and frame `maxSpeed` for each level's solution and empty fixture. Set a minimum per-step threshold and gate splash by `maxSpeed` (already on every frame) from that data. Normalize by folded `steps`.
**Warning signs:** Splash voices continue after the stream visibly stops.

### Pitfall 3: Impacts only exist in Float Up

**What goes wrong:** SOUND-02 "impact sounds" are never heard in five of six levels.
**Why:** `create_floats` is the only `BodyType::Dynamic` creation in the puzzle builder; walls, cup walls and tools are static, and valves are static bodies destroyed at their open tick [VERIFIED: crates/liquidfun-wasm/src/puzzle/builder.rs].
**How to avoid:** See Open Question 1 (recommended: also trigger the impact voice on a splash burst onset in the web layer). Verify impacts on `float-up` in the e2e/manual check.

### Pitfall 4: Widening `puzzle-options-slot` collides with other HUD leaves

**What goes wrong:** A second 44 px button forces the slot to 96×44 (44 + 8 gap + 44). The landscape caption ends at `right: inset + 52px` and the portrait title ends 68 px from the right edge (`left: inset + 52px; width: calc(100% - 120px)`), so both overlap the wider slot [VERIFIED: web/src/styles/puzzle.css:271-352].
**How to avoid:** Landscape caption `right: calc(var(--puzzle-inset-right) + 104px)`; portrait title narrowed so its right edge clears 104 px (the top-left 52 px is the "Demos" sidebar trigger). Landscape slot 96 px fits inside the 136 px right rail, so the world frame does not change; portrait band height is unchanged. Keep `width` and `height` in both layout blocks (`tests/puzzle-chrome.test.ts` sized-leaf checks) and update its leaf/size expectations if they list the slot. `overlappingLeaves` / `overlappingObstacles` in puzzle e2e catch regressions.
**Warning signs:** `overlappingLeaves(geometry)` non-empty in `puzzle-editor.spec.ts` layout tests.

### Pitfall 5: `exponentialRampToValueAtTime(0, …)` throws

**What goes wrong:** Envelope code throws inside the engine; D-08 handling disables sound for the session.
**Why:** The target value must be positive [CITED: developer.mozilla.org/en-US/docs/Web/API/AudioParam/exponentialRampToValueAtTime].
**How to avoid:** Ramp to `0.0001` (or use `setTargetAtTime(0, t, τ)`), then `stop(endsAt)`.

### Pitfall 6: `stop()` on an unstarted node throws

**What goes wrong:** `stopAll()` throws `InvalidStateError` for a node created but not yet `start()`ed.
**Why:** [CITED: developer.mozilla.org/en-US/docs/Web/API/AudioScheduledSourceNode/stop]. A future `start(when)` followed by `stop(now)` is fine (it never plays), and repeated `stop()` calls are allowed.
**How to avoid:** Register a voice in the pool only after `start()`; in `stopAll()`, cancel gain automation, set gain to 0, `stop()` each source, `disconnect()`.

### Pitfall 7: iOS "interrupted" and silent output

**What goes wrong:** After backgrounding, iOS Safari reports `"interrupted"`, or reports `"running"` but is silent.
**Why:** Non-standard WebKit behaviour; community reports suggest suspend + resume on visibility helps [CITED: bugs.webkit.org/show_bug.cgi?id=263627; MEDIUM].
**How to avoid:** Treat `"interrupted"` as not running; on visible (puzzle route, enabled) call `resume()`; retry `resume()` on the next trusted gesture. Manual iPhone check with the ring switch on and off; the hint covers the silent switch (D-11).

### Pitfall 8: Audio work leaking past a generation

**What goes wrong:** A chime scheduled 200 ms ahead plays after Retry or a route change.
**How to avoid:** `stopAll()` in `incrementGeneration` (every Play, Stop, Edit/Retry via `buildPuzzle`, and level change calls it); `leavePuzzle` and `releaseFailedPuzzle` call `suspend()` (stop first). The trigger memory resets when `stepIndex` goes backwards.

### Pitfall 9: Node-environment unit tests constructing the browser engine

**What goes wrong:** `createPuzzleRuntime()` is called in four Vitest files (`node` environment, no `window`); a default that touches `window` throws.
**How to avoid:** `createPuzzleRuntime(progress, sound = createDisabledPuzzleSound())`; only `playground-session.ts:125` passes the browser instance. Keep the module-level preference memory lazy (like `firstHintMemory`).

### Pitfall 10: Debug vs release determinism profiles

**What goes wrong:** A new Rust test compares native debug sound scalars to recorded values and fails in release, or vice versa.
**Why:** Native debug diverges from release (Phase 37 spike: dam-break @91, theo-jansen @2); replay exactness is asserted in release only.
**How to avoid:** New determinism tests compare two runs (or tally-on vs tally-off) within the same profile, never against recorded literals.

## Code Examples

### Rust: pure aggregates (new `puzzle/sound.rs`)

```rust
// Source: designed against crates/liquidfun/src/world/step/report.rs,
// world/contact.rs and particle/view.rs public APIs (verified this session).
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, DefaultHasher};

use liquidfun::{
    ContactTransitionKind, FixtureId, ParticleId, ParticleSystemId, StepReport, World,
};

use crate::session::SessionError;

type ContactKey = (ParticleId, FixtureId);
/// Fixed-key hasher: membership only, never iterated.
type ContactSet = HashSet<ContactKey, BuildHasherDefault<DefaultHasher>>;

/// Read-only audio aggregates for one fixed step (Phase 42 D-07).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct SoundSignals {
    /// Particle-fixture contacts present now that were absent last step.
    pub(crate) splash_contacts: u32,
    /// Non-sensor rigid contacts that began touching this step.
    pub(crate) impact_count: u32,
    /// Largest summed normal impulse of those contacts, in N·s.
    pub(crate) impact_impulse: f32,
}

/// Per-session tally; observes the world after each step and never writes.
#[derive(Default)]
pub(crate) struct SoundTally {
    previous: ContactSet,
    scratch: ContactSet,
    signals: SoundSignals,
}

impl SoundTally {
    pub(crate) fn observe(
        &mut self,
        world: &World,
        system: ParticleSystemId,
        report: &StepReport,
    ) -> Result<(), SessionError> {
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::StepFailed)?;
        self.scratch.clear();
        let mut fresh = 0_usize;
        for contact in view.body_contacts() {
            let key = (contact.particle(), contact.fixture());
            if !self.previous.contains(&key) {
                fresh += 1;
            }
            self.scratch.insert(key);
        }
        std::mem::swap(&mut self.previous, &mut self.scratch);
        let (impact_count, impact_impulse) = rigid_impacts(report);
        self.signals = SoundSignals {
            splash_contacts: u32::try_from(fresh).unwrap_or(u32::MAX),
            impact_count,
            impact_impulse,
        };
        Ok(())
    }

    pub(crate) const fn signals(&self) -> SoundSignals {
        self.signals
    }
}

/// Count and strongest impulse of rigid contacts that began this step.
fn rigid_impacts(report: &StepReport) -> (u32, f32) {
    let mut count = 0_u32;
    let mut strongest = 0.0_f32;
    for transition in report.contact_transitions() {
        let contact = transition.contact();
        if transition.kind() != ContactTransitionKind::Begin || contact.is_sensor() {
            continue;
        }
        count = count.saturating_add(1);
        let key = (contact.fixtures(), contact.child_indices());
        let impulse = report
            .contact_solves()
            .iter()
            .chain(report.continuous_contact_solves())
            .filter(|solve| (solve.contact().fixtures(), solve.contact().child_indices()) == key)
            .map(|solve| solve.contact().points().iter().map(|p| p.normal_impulse().max(0.0)).sum::<f32>())
            .fold(0.0_f32, f32::max);
        strongest = strongest.max(impulse);
    }
    (count, strongest)
}
```
Note: `u32::try_from(..).unwrap_or(u32::MAX)` is a saturating conversion, not an `unwrap()`. Verify `clippy::pedantic` accepts the closure and long chains; split into helpers if it flags them.

### Rust: wire into `SessionCore::advance` (sketch)

```rust
// after the existing on_after_step block, per step:
if let Some(tally) = self.maybe_sound.as_mut() {
    tally.observe(&self.world, self.particle_system, &report)?;
}
// annotate_frame, after set_puzzle_status:
if let Some(tally) = &self.maybe_sound {
    frame.set_puzzle_sound(tally.signals()).map_err(|_error| SessionError::FrameCaptureFailed)?;
}
```

### TS: sound preference (mirror of first-hint-preference.ts)

```ts
// Source: web/src/player/first-hint-preference.ts (repo pattern)
export const SOUND_STORAGE_KEY = "liquidfun.puzzle-sound.v1";
const ON_TOKEN = "on";
const OFF_TOKEN = "off";
type SoundStorage = Pick<Storage, "getItem" | "setItem">;
export type SoundStorageProvider = () => SoundStorage;
export const browserSoundStorage: SoundStorageProvider = () => window.localStorage;

export function loadSoundOn(provider: SoundStorageProvider): boolean {
  try {
    return provider().getItem(SOUND_STORAGE_KEY) === ON_TOKEN;
  } catch {
    // Unreadable storage means off (D-03).
    return false;
  }
}

export function persistSoundOn(provider: SoundStorageProvider, on: boolean): void {
  try {
    provider().setItem(SOUND_STORAGE_KEY, on ? ON_TOKEN : OFF_TOKEN);
  } catch {
    // The in-memory choice still holds for this page session (D-03).
  }
}
```

### TS: engine shell essentials

```ts
type WebkitWindow = Window & { readonly webkitAudioContext?: typeof AudioContext };

export function browserContextFactory(): (() => AudioContext) | undefined {
  const maybeCtor = window.AudioContext ?? (window as WebkitWindow).webkitAudioContext;
  return typeof maybeCtor === "function" ? () => new maybeCtor() : undefined;
}

function createMasterChain(context: AudioContext): GainNode {
  const master = context.createGain();
  master.gain.value = 0.5;
  const compressor = context.createDynamicsCompressor();
  master.connect(compressor);
  compressor.connect(context.destination);
  return master;
}

function createNoiseBuffer(context: AudioContext, seconds = 0.25): AudioBuffer {
  const buffer = context.createBuffer(1, Math.ceil(context.sampleRate * seconds), context.sampleRate);
  const data = buffer.getChannelData(0);
  // Audio-only randomness: never touches the simulation.
  for (let index = 0; index < data.length; index += 1) data[index] = Math.random() * 2 - 1;
  return buffer;
}

function playPlink(context: AudioContext, master: GainNode, frequency: number, now: number) {
  const oscillator = context.createOscillator();
  const gain = context.createGain();
  oscillator.type = "sine";
  oscillator.frequency.setValueAtTime(frequency, now);
  gain.gain.setValueAtTime(0.0001, now);
  gain.gain.exponentialRampToValueAtTime(0.3, now + 0.005);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.12);
  oscillator.connect(gain).connect(master);
  oscillator.start(now);
  oscillator.stop(now + 0.13);
  oscillator.onended = () => gain.disconnect();
  return { sources: [oscillator], gain, endsAt: now + 0.13 };
}
```

### TS: plink pitch (pure)

```ts
// Two octaves of major pentatonic: 11 steps from the root (discretion: root C5).
const PENTATONIC_SEMITONES = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24] as const;
export function plinkFrequency(goalCount: number, goalTarget: number, rootHz = 523.25): number {
  const fill = goalTarget > 0 ? Math.min(1, Math.max(0, goalCount / goalTarget)) : 0;
  const step = Math.round(fill * (PENTATONIC_SEMITONES.length - 1));
  return rootHz * 2 ** ((PENTATONIC_SEMITONES[step] ?? 0) / 12);
}
```

### Playwright: fake AudioContext via addInitScript

```ts
// Source: pattern from e2e/puzzle-helpers.ts installSolutionPlacements + player-helpers.ts
export async function installFakeAudio(page: Page): Promise<void> {
  await page.addInitScript(() => {
    type Log = { constructed: number; states: string[]; started: number };
    const log: Log = { constructed: 0, states: [], started: 0 };
    Object.defineProperty(window, "__liquidfunAudioLog", { value: log, configurable: true });
    const param = () => ({
      value: 0,
      setValueAtTime() { return this; },
      linearRampToValueAtTime() { return this; },
      exponentialRampToValueAtTime() { return this; },
      setTargetAtTime() { return this; },
      cancelScheduledValues() { return this; },
    });
    const node = () => ({
      gain: param(), frequency: param(), Q: param(), detune: param(),
      threshold: param(), knee: param(), ratio: param(), attack: param(), release: param(),
      type: "", buffer: null, onended: null,
      connect(target: unknown) { return target; },
      disconnect() {},
      start() { log.started += 1; },
      stop() {},
    });
    class FakeAudioContext {
      state = "suspended";
      sampleRate = 48_000;
      destination = node();
      get currentTime() { return performance.now() / 1000; }
      constructor() { log.constructed += 1; log.states.push("constructed"); }
      resume() { this.state = "running"; log.states.push("running"); return Promise.resolve(); }
      suspend() { this.state = "suspended"; log.states.push("suspended"); return Promise.resolve(); }
      close() { this.state = "closed"; log.states.push("closed"); return Promise.resolve(); }
      createGain() { return node(); }
      createOscillator() { return node(); }
      createBiquadFilter() { return node(); }
      createBufferSource() { return node(); }
      createDynamicsCompressor() { return node(); }
      createBuffer(_channels: number, length: number) {
        const data = new Float32Array(length);
        return { length, getChannelData: () => data };
      }
    }
    Object.defineProperty(window, "AudioContext", { value: FakeAudioContext, configurable: true });
    Object.defineProperty(window, "webkitAudioContext", { value: undefined, configurable: true });
  });
}
// Unavailable variant: define both AudioContext and webkitAudioContext as undefined.
```
`performance.now()` here is test code in `web/e2e/`, outside the puzzle grep gates. Trusted input for the stored-"on" unlock: `page.mouse.click(x, y)` on a neutral element (for example the title) or `page.keyboard.press("a")`; avoid modifier-only keys and Escape (not activation-triggering).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `webkitAudioContext` + silent-buffer unlock on iOS | Unprefixed `AudioContext`, `resume()` inside a user activation | Safari 14.1 unprefixed (ASSUMED date) | Keep the prefix fallback (D-05) but expect it unused. |
| Autoplay "user gesture" folklore | HTML user activation model (transient vs sticky, defined activation-triggering events) | HTML Standard, current | Touch needs `pointerup`/`touchend` (Pitfall 1). |
| `AudioContextState` 3 values | Adds `"interrupted"` (WebKit) — present in TS 7 DOM lib | TS lib verified 2026-10-13 | Handle it explicitly. |
| Force `navigator.audioSession.type = "playback"` | Leave default session + silent-switch hint | Locked D-11 | Respects the user's other audio. |

**Deprecated/outdated:** `ScriptProcessorNode` (do not use); creating one `AudioContext` per level (browser limits, leaks).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Resting liquid produces continuous new-contact churn that must be gated | Pitfall 2 | Either the gate is unnecessary (harmless) or thresholds are too low and splash hisses; Wave 0 probe resolves it. |
| A2 | Rigid `Begin` contacts in Float Up are solved in the same step, so impulse matching finds a non-zero value | Code Examples (`rigid_impacts`) | Impact strength 0 → weak or no impact voice; a Rust test asserting Float Up reports an impulse > 0 resolves it. |
| A3 | Chrome's `resume()` without activation leaves a pending promise rather than rejecting | Pattern 4 | Only affects logging noise; guard against stacking resumes with an in-flight flag. |
| A4 | Unprefixed `AudioContext` in Safari since 14.1 | State of the Art | None in practice; the fallback stays. |
| A5 | Adding ~15 lines keeps `session.rs` under the 628-line trigger | Pattern 1 | Bright Builds checker flags it; extract the per-step body into a helper. |

## Open Questions

1. **Impacts in levels without dynamic rigid bodies**
   - What we know: only Float Up has a dynamic body; the rigid `impactCount` / `impactImpulse` scalars will be zero in five levels.
   - What's unclear: whether SOUND-02's "impact sounds" should be audible in every level.
   - Recommendation: keep the bridge scalars rigid-only (honest naming), and in `triggers.ts` also fire the impact voice on a **splash burst onset** (per-step splash contacts crossing a high threshold after a quiet window, for example the liquid's first hit on a plank or the cup floor). This stays within D-07 ("new-contact strength for impacts") and the discretion over aggregate definitions. Record it as a plan decision.

2. **"Turn sound off for the session" on audio failure (D-08)**
   - What's unclear: whether to persist "off".
   - Recommendation: disable in memory only (toggle shows off and disabled with the unavailable name if creation failed; otherwise off), do not overwrite the stored preference, so a later visit tries again.

3. **Form-factor (non canvas-stage) puzzle shell**
   - What we know: `PuzzlePanel` renders a `control-row` fallback without the HUD when `canvasStage` is false (`withFormFactorShell` in e2e).
   - Recommendation: render the same toggle in that control row (no `data-hud-obstacle` there) so SOUND-01 is reachable in both shells; the drawer section exists only on the canvas stage.

4. **Duplicate accessible name "Sound"**
   - The HUD button and the drawer control would both be named "Sound". Recommendation: keep the button "Sound" (D-01) and label the drawer checkbox "Sound effects" (or keep "Sound" and rely on roles in tests). UI-SPEC decides.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | bridge, tests | yes | 1.97.0 | — |
| wasm-pack | `just web-wasm`, `puzzle-replay-wasm` | yes | 0.15.0 (script-checked) | — |
| bun | web scripts, typecheck, vitest | yes | 1.4.2 | — |
| just | recipes | yes | 1.48.0 | call the underlying commands |
| jq | fingerprint extraction | yes | 1.7.1 | — |
| Playwright Chromium | e2e | yes (several cached builds) | @playwright/test 1.63.0 | `bun run browser:install` |
| Real iPhone | silent-switch/interrupted check | not verifiable here | — | manual UAT, flagged |

**Missing dependencies with no fallback:** none for automated work. The iOS check is manual.

## Validation Architecture

(Included at the orchestrator's request; `workflow.nyquist_validation` is `false` in `.planning/config.json`.)

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (liquidfun-wasm lib binary); Vitest 5.0.1 (node env); Playwright 1.63.0 |
| Config file | `web/vitest.config.ts`, `web/playwright.config.ts` |
| Quick run command | `cargo test -p liquidfun-wasm --lib puzzle::sound` ; `cd web && bun run test:unit -- tests/sound-*.test.ts` |
| Full suite command | Phase gate script (below) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SOUND-01 | Preference parse/persist, exact "on" token, throwing storage | unit | `cd web && bun run test:unit -- tests/sound-preference.test.ts` | Wave 0 |
| SOUND-01 | No context on load, none before toggle, none with stored "on" before a gesture; toggle persists across reload; unavailable renders disabled | e2e | `cd web && bun run build:app && CI=true bunx playwright test e2e/puzzle-sound.spec.ts` | Wave 0 |
| SOUND-02 | Fill→pitch, pentatonic step, voice parameter mapping | unit | `bun run test:unit -- tests/sound-voices.test.ts` | Wave 0 |
| SOUND-02 | Rate limits (80/60/50 ms), cap 6, chime steals oldest | unit | `bun run test:unit -- tests/sound-voice-pool.test.ts` | Wave 0 |
| SOUND-02 | Run-phase only, no voices after won, one plink per frame, chime once on the won edge, reset on new run | unit | `bun run test:unit -- tests/sound-triggers.test.ts` | Wave 0 |
| SOUND-02 | Fold sums/maxes per step; total on undefined | unit | `bun run test:unit -- tests/sound-signals.test.ts` | Wave 0 |
| SOUND-02 | Frame parse fail-closed (partial, scene with sound, NaN impulse) | unit | `bun run test:unit -- tests/frame-sound.test.ts` (or extend `frame-puzzle.test.ts`) | Wave 0 |
| SOUND-02 | Engine shell with fake factory: master chain built once, noise buffer once, stopAll stops started sources, failure disables and logs | unit | `bun run test:unit -- tests/sound-engine.test.ts` | Wave 0 |
| SOUND-03 | Hidden → suspended + voices stopped; visible → resumed only if on and puzzle route; leave puzzle → suspended | e2e + unit | `CI=true bunx playwright test e2e/puzzle-sound.spec.ts` | Wave 0 |
| SOUND-03 | Hint text in drawer and on toggle (`title`/`aria-describedby`) | e2e / source grep | same spec; `tests/puzzle-chrome.test.ts` additions | Wave 0 |
| Crit. 4 | Sound scalars deterministic across runs per level | Rust unit | `cargo test -p liquidfun-wasm --lib puzzle::sound` | Wave 0 |
| Crit. 4 | Tally on vs off: identical per-step particle position bits and goal status for N steps (test-only constructor with `maybe_sound = None`) | Rust unit | same | Wave 0 |
| Crit. 4 | Scene frames: sound getters `None` | Rust unit | same | Wave 0 |
| Crit. 4 | Existing replay pins unchanged | Rust + wasm | `cargo test -p liquidfun-wasm --release --lib puzzle::replay`; `just puzzle-replay-wasm`; `git diff --quiet $BASE -- crates/liquidfun-wasm/src/puzzle/fixtures` | yes |
| Crit. 4 | 25/25 scene fingerprints | gate | `just playground-scene-spot`, then `S=$(ls -td target/dam-break-perf/*/ \| head -1); jq -r '.scenes[] \| "\(.scene) \(.fingerprint)"' "$S/scene-spot.json" \| sort > target/phase42/after.txt; diff target/phase42/base.txt target/phase42/after.txt` | baseline in Wave 0 |
| Crit. 4 | Same `RunFingerprint` (outcome step, decided step, peak held, held hash) with sound on and off | e2e | `puzzle-sound.spec.ts` using `installSolutionPlacements` + `readRunFingerprint` | Wave 0 |

### Sampling Rate

- **Per task commit (Rust):** `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build -p liquidfun-wasm --all-targets --all-features`; `cargo test -p liquidfun-wasm`; `bun scripts/bright-builds-check.ts all`.
- **Per task commit (TS):** `cd web && bun run typecheck && bun run test:unit`; `bun scripts/bright-builds-check.ts all`.
- **Phase gate:** a `target/phase42/checks-NN.sh` script in the Phase 41 format with: fingerprints (25/25), engine-untouched, puzzle-fixtures-unchanged, scenes-untouched (same path list as Phase 41, which includes `crates/liquidfun-wasm/src/scene.rs` and `scene/`), protected-tests-unedited, cargo fmt/clippy/build/test, release replay, `just puzzle-replay-wasm`, typecheck, vitest, bright-builds, `just web-player-smoke`, `CI=true bunx playwright test e2e/hud-safe-frame.spec.ts`, `CI=true bun run test:puzzle` (add `e2e/puzzle-sound.spec.ts` to the `test:puzzle` script), markdown (if non-.planning Markdown changed), grep gates: no wall clock in puzzle/audio sources (`web/src/audio` included), storage allowlist updated with the sound-preference file, no `innerHTML`, no `data-hud-obstacle` in `PuzzleOptionsDrawer.tsx`/`PuzzleSoundToggle.tsx`, `! grep -rn "audioSession" web/src`, `! grep -rnE "tone|howler|zzfx" web/package.json`, `! grep -rn "new AudioContext" web/src | grep -v sound-engine.ts`.

### Wave 0 Gaps

- [ ] `target/phase42/phase-base.txt` and `target/phase42/base.txt` recorded **before any Rust edit** (cross-check against `target/phase41/after-final.txt`)
- [ ] Test-only probe printing per-step sound scalars + `maxSpeed` for every level's solution and empty fixture (tuning data for thresholds; Pitfall 2, Open Question 1)
- [ ] `crates/liquidfun-wasm/src/puzzle/sound/tests.rs`
- [ ] `web/tests/sound-{preference,signals,triggers,voices,voice-pool,engine}.test.ts`, frame sound parse tests
- [ ] `web/e2e/puzzle-sound.spec.ts` + fake audio helper (in a new `web/e2e/sound-helpers.ts`, not in protected specs)

## Security Domain

ASVS Level 1 view; the phase adds no network, auth, assets or new dependencies.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Fail-closed parse of new frame scalars (`parseNonNegativeInteger`, finite ≥ 0 impulse, all-or-nothing, puzzle-only) in TS and `set_puzzle_sound` validation in Rust; storage token compared by exact equality (`=== "on"`), never parsed or evaluated |
| V6 Cryptography | no | `Math.random()` only for audio noise, never for anything security- or simulation-relevant |
| V8 Data Protection | yes (minimal) | One non-personal preference token in localStorage; no PII |
| V10 Malicious Code / V14 Configuration | yes | No new npm or Cargo dependencies; no vendored code (zzfx only if tuning stalls, with its MIT license) |
| V12 Files and Resources | no | No audio assets fetched |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tampered `liquidfun.puzzle-sound.v1` value | Tampering | Exact-token check; anything else is off; throwing storage handled |
| Malformed or hostile frame values (NaN, huge counts) | Tampering / DoS | Fail-closed parse; voice cap and rate limits bound audio work regardless of counts |
| Audio node explosion on big pours | DoS | Cap 6 voices, per-voice minimum intervals, one plink per presented frame, pruning by `endsAt` |
| Surprise audio / autoplay abuse | Elevation (UX) | Off by default; context only after explicit opt-in plus trusted activation; no `audioSession` override |
| Audio failure taking down the run | DoS | Engine methods catch, `console.error`, disable sound for the session (D-08); never throw into the loop |
| Simulation influenced by audio | Tampering (integrity) | One-way data flow (frame → audio); tally-on/off Rust test; sound-on/off e2e `RunFingerprint` equality; 25/25 fingerprints |

## Sources

### Primary (HIGH confidence)

- Repository source read 2026-10-13: `crates/liquidfun-wasm/src/{session.rs,frame.rs,frame/puzzle.rs,puzzle.rs,puzzle/hooks.rs,puzzle/builder.rs,puzzle/float_up.rs,puzzle/replay.rs}`, `crates/liquidfun/src/world/step/{report.rs,execution.rs}`, `world/object/solver.rs`, `world/contact.rs`, `world/contact_solver.rs`, `particle/{statistics.rs,view.rs}`, `identity.rs`, `collision/types.rs`; `web/src/player/{puzzle-loop,puzzle-lifecycle,puzzle-runtime,puzzle-host,puzzle-run,playground-session,scene-lifecycle,first-hint-preference}.ts`, `web/src/physics/{frame,frame-puzzle,frame-scalars}.ts`, `web/src/puzzle/{reduce-puzzle,debug-attributes,controller}.ts`, `web/src/components/{PuzzleHud,PuzzleOptionsDrawer,PuzzlePanel,CanvasStage,PlayerPanel}.tsx`, `web/src/styles/{puzzle,canvas-stage}.css`, `web/tests/puzzle-chrome.test.ts`, `web/e2e/{puzzle-helpers,puzzle-editor-helpers,player-helpers,puzzle.spec,puzzle-editor.spec}.ts`, `justfile`, `web/package.json`, `web/vitest.config.ts`, `web/tsconfig.json`
- Phase 40/41 plans and summaries (gate scripts, fingerprint recipe, protected files)
- [HTML Standard: activation-triggering input event](https://html.spec.whatwg.org/multipage/interaction.html#activation-triggering-input-event)
- [MDN: AudioScheduledSourceNode.stop()](https://developer.mozilla.org/en-US/docs/Web/API/AudioScheduledSourceNode/stop)
- [MDN: AudioParam.exponentialRampToValueAtTime()](https://developer.mozilla.org/en-US/docs/Web/API/AudioParam/exponentialRampToValueAtTime)
- TypeScript 7.0.2 bundled `lib.dom.d.ts` (`AudioContextState` includes `"interrupted"`)

### Secondary (MEDIUM confidence)

- [Chrome: Autoplay policy](https://developer.chrome.com/blog/autoplay) — AudioContext created before a gesture starts suspended; call `resume()` after interaction
- [WebKit bug 263627: AudioContext not consistently resumed when brought to foreground](https://bugs.webkit.org/show_bug.cgi?id=263627), [WebKit bug 276016](https://bugs.webkit.org/show_bug.cgi?id=276016) — iOS `"interrupted"` behaviour
- Milestone research `.planning/research/v1.5-puzzle-mode/{STACK,FEATURES,ARCHITECTURE,PITFALLS}.md`

### Tertiary (LOW confidence)

- Community reports of silent-but-running contexts on iOS 18/26 (via WebKit bug threads); manual device check required

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; APIs verified in source and docs
- Architecture: HIGH — every hook point read in code; read-only safety verified (no interior mutability, solves always collected)
- Pitfalls: MEDIUM — layout collisions and activation rules verified; splash churn and impact audibility need the Wave 0 probe; iOS behaviour needs a device

**Research date:** 2026-10-13
**Valid until:** 2026-11-12 (stable in-repo surface; re-check iOS behaviour if Safari updates)
