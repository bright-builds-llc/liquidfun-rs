---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: 2026-10-13T18:17:53.102Z
---

# Phase 42: Opt-in Sound - Context

**Gathered:** 2026-10-13
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Puzzle routes (`#/puzzle/<id>`) get opt-in synthesized sound: splashes, impacts, plinks that rise in pitch as the cup fills, and a win chime. Sound is off by default. The player turns it on with a toggle, and the choice is remembered. A cap limits how many sounds play at once. Sound pauses when the tab is hidden or the player leaves the puzzle, and a hint mentions the phone silent switch. Sound only consumes frame data. It never feeds back into the simulation: 25/25 scene fingerprints stay identical, and puzzle outcomes are the same with sound on and off.

Out of scope:

- sound in the 25 catalog scenes
- UI ticks for placing, rotating or removing tools
- a volume slider
- forcing `navigator.audioSession`
- sample assets or audio libraries
- GPU water (Phase 43)
- new levels

</domain>

<decisions>
## Implementation Decisions

### Toggle, default and persistence (SOUND-01)

- **D-01:** One speaker toggle button in the puzzle HUD. It sits inside the existing `puzzle-options-slot` (which carries `data-hud-obstacle`) next to the Puzzle options trigger. It is 44 px, matches the `hud-button` styling and has a fixed box size, so the obstacle set never changes when it toggles. It is a real `<button>` with `aria-pressed` and an accessible name of "Sound" (the icon shows a speaker with or without sound waves). Puzzle routes only. Scene routes and the `#/puzzles` grid get no sound control.
- **D-02:** The Puzzle options drawer gets a "Sound" section. It has a checkbox or switch that mirrors the HUD toggle (one shared state) and the silent-switch hint (D-11).
- **D-03:** The preference is stored under a new versioned key, `liquidfun.puzzle-sound.v1`. Only the exact token `"on"` means on. A missing, unreadable or any other value means off. Follow the provider and try/catch pattern in `web/src/player/first-hint-preference.ts`. Keep this separate from `liquidfun.puzzle-progress.v1` and the first-hint key. Blocked storage or a throwing `setItem` keeps the choice in memory for the page session, and the toggle keeps working.
- **D-04:** No `AudioContext` exists until the player has opted in **and** a user gesture has happened:
  - Turning sound on through the toggle creates the single page-session context (or resumes it) inside that click handler.
  - If the stored preference is "on" when a page loads, the toggle shows as on, but no context is created and nothing plays until the first trusted `pointerdown` or `keydown` on a puzzle route. That gesture creates and resumes the context.
  - Turning sound off suspends the context and stops every voice. The context is reused if the player turns sound on again. It is never closed during the page session.
- **D-05:** If `AudioContext` (or `webkitAudioContext`) is unavailable or creation throws, the toggle renders disabled with an accessible name like "Sound unavailable in this browser", in the same box. The game keeps working. Errors surface in the console and are not swallowed silently.

### Sounds and signals (SOUND-02)

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

### Pausing and the silent-switch hint (SOUND-03)

- **D-10:** Hiding and leaving:
  - On `visibilitychange` to hidden, suspend the context and stop active voices. On visible, resume only if sound is on and the route is still a puzzle route.
  - Leaving the puzzle route through the existing `leavePuzzle` / teardown path suspends the context and stops all voices. The context is kept for reuse.
  - Stop, Edit and Retry stop any active voices immediately.
  - No timers or scheduled audio may survive a route change or a newer build. Respect the loop's `isCurrent()` generation guard.
- **D-11:** The hint reads "No sound on iPhone? Check the silent switch." (exact wording at Claude's discretion). It shows in the drawer's Sound section next to the mirror switch, and it is the HUD toggle's `title` or description. Show it on every platform; do not sniff the user agent. Do not set `navigator.audioSession.type` (it pauses the user's other audio, and it is not Baseline).

### Determinism and verification (criterion 4)

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap

- `.planning/ROADMAP.md` § Phase 42: goal and the four success criteria
- `.planning/REQUIREMENTS.md`: SOUND-01, SOUND-02, SOUND-03
- `.planning/seeds/SEED-005-sonification.md`: opt-in, muted by default, never feeds the simulation; breadcrumbs
- `.planning/STATE.md` § Decisions [v1.5]: no engine change may alter the 25 scene fingerprints, and bridge or frame-contract phases need 25/25 identical fingerprints

### Milestone research

- `.planning/research/v1.5-puzzle-mode/STACK.md` § 3 "Sound": plain Web Audio, lazy context in the unmute handler, synthesis recipes, rate limits, suspend on hidden, iOS silent switch, injectable factory
- `.planning/research/v1.5-puzzle-mode/FEATURES.md` § "Sound (opt-in)" and the rising-pitch plink differentiator
- `.planning/research/v1.5-puzzle-mode/ARCHITECTURE.md`: audio scalars on `FrameData` / `ProofFrame`, `web/src/audio/*`, and the puzzle direct backend (puzzles run on the main thread, not in the worker)

### Prior phase decisions

- `.planning/phases/38-puzzle-route-and-player-shell/38-CONTEXT.md`: HUD obstacle rules (fixed leaf boxes) and the Puzzle options drawer
- `.planning/phases/40-timed-valve-and-teaching-levels/40-CONTEXT.md`: versioned storage key and try/catch preference pattern
- `.planning/phases/41-level-select-and-progress/41-CONTEXT.md`: separate keys per concern, degraded storage still playable, `leavePuzzle` teardown

### Repository rules

- `AGENTS.md` / `CLAUDE.md`: repo-local guidance; new on-canvas chrome carries `data-hud-obstacle` on its leaf with a camera-independent box size
- `standards/core/frontend-ui.md`, `standards-overrides.md` (shadcn-solid shell)
- `standards/core/testing.md`, `standards/languages/typescript-javascript.md`, `standards/languages/rust.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `web/src/player/first-hint-preference.ts`: the storage provider, try/catch load and persist, and lazy page-session memory. Copy this shape for the sound preference.
- `web/src/components/PuzzleOptionsDrawer.tsx`: `PuzzleOptionsTrigger` (44 px `hud-button`) and `PuzzleOptionsSheet`, which takes `children`, so a Sound section slots in.
- `web/src/components/PuzzleHud.tsx`: `puzzle-options-slot` with `data-hud-obstacle`, the home for the toggle.
- `web/src/physics/frame-puzzle.ts` and `crates/liquidfun-wasm/src/frame/puzzle.rs`: the optional, fail-closed puzzle scalar pattern to extend with impact and splash scalars.
- `crates/liquidfun-wasm/src/frame.rs`: `body_contact_count` and `max_speed` diagnostics already on frames. `crates/liquidfun-wasm/src/session.rs:428` reads `statistics.body_contact_count()`.
- `crates/liquidfun/src/world/step/report.rs`: `StepReport::contact_solves()` and `events()` (possible impulse source; check the collection cost and whether collection is already on).

### Established Patterns

- `web/src/player/puzzle-loop.ts`: steps one capture at a time, then `drawSceneFrame` and then `publish(frame, observation, state)`. Already skips work when `document.hidden`. The sound hook belongs after publish, or inside the lifecycle's publish.
- `web/src/player/puzzle-lifecycle.ts`: owns the session, generation guard and teardown. Hook the sound engine's stop and suspend here.
- Pure core plus thin shell with injected providers (progress, placements, hint preference).

### Integration Points

- New `web/src/audio/` module. Toggle in the puzzle HUD slot, and a mirror plus hint in the options drawer.
- Bridge: new read-only puzzle frame getters for splash and impact aggregates, and their TS parse in `frame-puzzle.ts` / `RenderFrame.maybePuzzle`.
- E2E: a new `web/e2e/puzzle-sound.spec.ts` using `puzzle-helpers.ts`.

</code_context>

<specifics>
## Specific Ideas

- The plink pitch acts as an audio fill meter. The player should hear the cup getting close to the target without looking.
- Keep it gentle. This is an opt-in toy, not an arcade, so cap aggressively rather than risk a wall of noise on big pours.

</specifics>

<deferred>
## Deferred Ideas

- Sound for the 25 catalog scenes (SEED-005 mentions Liquid Bubbler, Stacked Drip, Dam Break and Sparky). That would be its own phase.
- Very quiet UI ticks for place, rotate snap, remove and Play (FEATURES.md table stakes, not in SOUND-01..03).
- A volume slider, and a loss sound.
- Opting in to `navigator.audioSession.type = "playback"` behind a setting.

### Reviewed Todos (not folded)

None. No pending todos matched this phase.

</deferred>

---

*Phase: 42-opt-in-sound*
*Context gathered: 2026-10-13*
