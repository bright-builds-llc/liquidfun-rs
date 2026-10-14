# Roadmap: liquidfun-rs

## Overview

v1.5 Puzzle Mode adds a small game to the playground. While a level is paused, the player places a few fixed-size tools, presses Play, and tries to get enough water into the cup before the timer runs out. The puzzle core lives in the private `liquidfun-wasm` bridge and reads existing public engine queries, so the `crates/liquidfun` engine and the 25 scene fingerprints stay unchanged. The order follows the research: puzzle core and determinism first, then the route and player shell, tools and stars, the timed valve and teaching levels, level select and progress, opt-in sound, and GPU water as the closing phase. Phase numbering continues after 36.

## Milestones

- [ ] **v1.5 Puzzle Mode** — Phases 37–44 (planned 2026-10-11; Phase 44 added from the audit 2026-10-14). Place-then-play cup puzzles with stars, 5–8 teaching levels, per-browser progress, opt-in sound and a GPU water render mode. The 25 scenes stay unchanged. No package or tag authorization.
- [x] **v1.4 Scenario Performance** — Phases 34–36 (archived 2026-10-11). Whole-catalog timing survey, seven profile-guided fixes with bit-identical behavior, and a before/after survey: 24 of 25 scenes faster and none slower, as local observations. Annotated tag `v1.4`; no crate. ([full roadmap](milestones/v1.4-ROADMAP.md))
- [x] **v1.3 Reference Testbed Scenes** — Phases 26–33 (archived 2026-09-28). Twelve missing JS testbed scenes, then Hydraulic Fountain, Wave Tank, Liquid Bubbler, and Stacked Drip. No crate or tag. ([full roadmap](milestones/v1.3-ROADMAP.md))
- [x] **v1.2 Native Performance Closing** — Phases 22–25 (archived 2026-09-21). Local Dam Break Medium recorded at ≤ 3× C++; no Phase 12 sealed matrix; no crate or tag. ([full roadmap](milestones/v1.2-ROADMAP.md))
- [x] **v1.1 Web Playground** — 6 phases / 39 plans complete; archived 2026-09-20 ([full roadmap](milestones/v1.1-ROADMAP.md)). Planning label only; no package or tag released.
- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Phases

- [x] **Phase 37: Puzzle Core in the Bridge** — Build levels from (level, placements) and decide fill, win and timeout per fixed step in Rust, identically natively and in wasm.
- [x] **Phase 38: Puzzle Route and Player Shell** — Open a level at `#/puzzle/<id>` in the shared player with a glass cup, fill meter, step timer and result card.
- [x] **Phase 39: Place-then-Play Tools and Stars** — Place, move, rotate and remove tools while building, then Play, Stop/Reset and earn stars.
- [x] **Phase 40: Timed Valve and Teaching Levels** — A timed valve plus 5–8 teaching levels, each proven solvable by automated tests.
- [x] **Phase 41: Level Select and Progress** — `#/puzzles` grid, soft unlocks and per-browser progress that survives blocked storage.
- [x] **Phase 42: Opt-in Sound** — Muted-by-default splashes, impacts, filling plinks and a win chime.
- [x] **Phase 43: GPU Water Render Mode** — Closing phase: refraction, highlights and foam with safe fallbacks.
- [ ] **Phase 44: v1.5 Polish and Test Gaps** — Audit cleanup: phone tray names, valve accessibility, Next and valve fast-forward e2e, typechecked e2e code.

<details>
<summary>✅ v1.4 Scenario Performance (Phases 34-36) — SHIPPED 2026-10-11</summary>

v1.4 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.4-ROADMAP.md](milestones/v1.4-ROADMAP.md).

- [x] Phase 34: Scene Timing Survey (2/2 plans) — completed 2026-10-07
- [x] Phase 35: Speed Up the Slowest Scenes (8/8 plans) — completed 2026-10-10
- [x] Phase 36: Re-survey and Wrap Up (3/3 plans) — completed 2026-10-11

</details>

<details>
<summary>✅ v1.3 Reference Testbed Scenes (Phases 26-33) — SHIPPED 2026-09-28</summary>

v1.3 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.3-ROADMAP.md](milestones/v1.3-ROADMAP.md).

- [x] Phase 26: Catalog shell and basin scenes (5/5 plans) — completed 2026-09-22
- [x] Phase 27: Material flag groups (6/6 plans) — completed 2026-09-22
- [x] Phase 28: Interaction seams (8/8 plans) — completed 2026-09-22
- [x] Phase 29: Sparky, Drawing, and full catalog (5/5 plans) — completed 2026-09-27
- [x] Phase 30: Periodic hydraulic fountain (3/3 plans) — completed 2026-09-27
- [x] Phase 31: Sinusoidal wave tank (3/3 plans) — completed 2026-09-27
- [x] Phase 32: Liquid motion bubbler (4/4 plans) — completed 2026-09-27
- [x] Phase 33: Stacked drip fidget (4/4 plans) — completed 2026-09-28

</details>

<details>
<summary>✅ v1.2 Native Performance Closing (Phases 22-25) — SHIPPED 2026-09-21</summary>

v1.2 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.2-ROADMAP.md](milestones/v1.2-ROADMAP.md).

- [x] Phase 22: Observability shell (6/6 plans) — completed 2026-09-21
- [x] Phase 23: Baseline pair and named audit (9/9 plans) — completed 2026-09-21
- [x] Phase 24: Shared hot-path waves through 3× (6/6 plans) — completed 2026-09-21
- [x] Phase 25: WASM sanity and honest close (2/2 plans) — completed 2026-09-21

</details>

<details>
<summary>✅ v1.1 Web Playground (Phases 16-21) — SHIPPED 2026-09-20</summary>

v1.1 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.1-ROADMAP.md](milestones/v1.1-ROADMAP.md).

- [x] Phase 16: Rust WASM Browser Bridge (4/4 plans) — completed 2026-09-17
- [x] Phase 17: Shared Player and Early Pages Delivery (8/8 plans) — completed 2026-09-17
- [x] Phase 18: Six Native Physics Demos (10/10 plans) — completed 2026-09-18
- [x] Phase 19: Interaction Polish and Browser Verification (7/7 plans) — completed 2026-09-19
- [x] Phase 20: Playground catalog previews and Reset honesty (6/6 plans) — completed 2026-09-20
- [x] Phase 21: Playground leftover cleanup (4/4 plans) — completed 2026-09-20

</details>

## Phase Details

### Shared acceptance rules

- **The 25 scenes stay unchanged.** No engine change may alter a scene fingerprint. Every phase that touches the wasm bridge or the frame contract reruns `just playground-scene-spot` and requires 25/25 identical end-state fingerprints.
- **Levels live in their own registry** (`LevelId` / `PUZZLE_IDS` in Rust and TS, with their own portrait frames and coverage tests). Levels never enter `SCENE_IDS`, the scene sidebar, the README gallery plans or the scene survey.
- **The outcome is decided per fixed step in Rust.** Fill, the settle window, the timer, win and timeout latch after each `world.step`. Nothing in the puzzle path uses wall-clock time or rendered frames. `MAX_ADVANCE_STEPS` stays 4.
- **Placements are construction input.** The world is rebuilt at step 0 from `(level, canonical placements)` on Play and Reset. Positions snap to a grid and angles to 15° (τ/24) steps, and tools are created in canonical order.
- **Owner decisions:** the valve is timed and set while building, with no live tapping during a run. The elastic level uses a "jelly pad" deflector whose particles never count. Stars keep the best result per criterion across runs. Locks are soft and linear, and deep links always open. When storage is blocked, every level is unlocked and a note says progress won't be saved.
- Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, plus the web unit and e2e tests a change touches.
- Research flags: Phase 37 (native-vs-wasm spike), Phase 39 (touch rotation and validator parity), Phase 40 (level tuning, valve, jelly pad counting, buoyant cup region, particle budgets) and Phase 43 (iOS float render targets, context loss).

### Phase 37: Puzzle Core in the Bridge

**Goal**: A level can be built in the bridge from a level id and a placement list, and its cup fill, win and timeout are decided per fixed step in Rust, with the same result natively and in wasm.
**Depends on**: Phase 36 (v1.4 engine and 25-scene fingerprints)
**Requirements**: GOAL-02, GOAL-04
**Success Criteria** (what must be TRUE):

1. A native test builds the first plain-water level from its level id and a placement list (empty allowed), steps it, and reads the cup count, target, full count, outcome, outcome step and step limit from each frame and from `levelInfo()`.
1. A level wins only after its fill stays at or above the target for the authored settle window (about 30 steps, 0.5 s) before the step limit. A splash that passes through the cup does not win, a run that never holds the target latches a loss at the limit, and a later slosh does not undo a win.
1. Counting unit tests in Arrange/Act/Assert form show that only counted particles with centres inside the cup interior count. They cover the rim, outside-wall, empty, evicted and non-counted-group cases, and fill is measured against the authored full count and clamped to 100%.
1. The native-vs-wasm determinism spike is run as the first task and its result is recorded in the phase summary. A reference-solution harness replays a recorded `(level, placements)` fixture to the same outcome and outcome step natively, and the angle-snapping and fill-margin policy for levels is set from the spike result.
1. `just playground-scene-spot` reports 25/25 scene fingerprints unchanged, `crates/liquidfun` is untouched, and level ids are checked against a separate Rust allowlist outside `SCENE_IDS`.

**Plans**: 8 plans

Plans:
- [x] 37-01-PLAN.md — Baseline fingerprints, then the native-vs-wasm determinism spike and shared FNV frame hash (wave 1)
- [x] 37-02-PLAN.md — TS frame parsing of optional puzzle scalars into maybePuzzle (wave 1)
- [x] 37-03-PLAN.md — Pure goal counter, step-latched GoalTracker, LevelId allowlist and PUZZLE_IDS mirror (wave 2)
- [x] 37-04-PLAN.md — SessionSource / EscapeLimits refactor with a 25/25 fingerprint proof (wave 2)
- [x] 37-05-PLAN.md — Plank tool, τ/24 table and integer placements; first-pour LevelSpec, builder and PuzzleHooks (wave 3)
- [x] 37-06-PLAN.md — Puzzle sessions, input rejections and first-pour tuning probes (wave 4)
- [x] 37-07-PLAN.md — newPuzzle, levelInfo and frame puzzle scalars on the wasm surface (wave 5)
- [x] 37-08-PLAN.md — Recorded fixtures, native and Bun wasm replay, and final phase gates (wave 6)

### Phase 38: Puzzle Route and Player Shell

**Goal**: Players can open a level at `#/puzzle/<id>` inside the existing player and watch a run with a glass cup, a live fill meter, a step timer and a result card, while the 25 scenes behave exactly as before.
**Depends on**: Phase 37
**Requirements**: GOAL-01, GOAL-03, PUZ-02, PUZ-03, PUZ-04, PUZ-05, PUZ-06
**Success Criteria** (what must be TRUE):

1. Opening `#/puzzle/<first-level>` shows the level with the cup drawn as glass, an etched target line and star ticks, and a fill meter at the cup whose percentage rises as only that level's water settles inside it. Opening `#/puzzle/<unknown>` shows a helpful fallback with a way back.
1. The timer runs on simulation time and only during a run. When the limit passes without a win, the result card says, for example, "Out of time: 54% of 80%". A win shows the result card with Next, Retry and Edit.
1. On puzzle routes, pointer poke and drag, tilt, the gravity slider, zoom and pan are inactive, scene-only chrome is hidden, and an e2e check shows the camera scale does not change between building and running.
1. The fill meter, timer and result card are `data-hud-obstacle` leaves in fixed slots, the scale legend is shown, and the level's portrait frame comes from a separate puzzle bounds registry with its own fill test.
1. The 25 scenes behave exactly as before: 25/25 fingerprints unchanged, `just web-player-smoke` and the existing hash, portrait-bounds, HUD safe-frame and README plan coverage tests pass without edits, and no level appears in `SCENE_IDS` or the scene sidebar.

**Plans**: 9 plans

Plans:

- [x] 38-01-PLAN.md — Playable route parser, PlayableTarget, level title table, and the 25-scene fingerprint baseline (wave 1)
- [x] 38-02-PLAN.md — Pure puzzle core: phase reducer, timer/fill/result view models, placement encoder and test seam, controller contract (wave 1)
- [x] 38-03-PLAN.md — Puzzle bounds registry with phone fill test, and the glass-cup overlay (wave 1)
- [x] 38-04-PLAN.md — Direct puzzle session adapter with step-0 capture and parsed level info (wave 1)
- [x] 38-05-PLAN.md — Puzzle runtime, frame loop with settle tail and hold, and lifecycle/controller (wave 2)
- [x] 38-06-PLAN.md — Puzzle UI: panel, fixed-slot HUD, result card, options drawer, unknown-level fallback (wave 2)
- [x] 38-07-PLAN.md — Shell wiring: PlayableRoute, target dispatch, glass hook, PlaygroundStage switch (wave 3)
- [x] 38-08-PLAN.md — Puzzle e2e: outcomes, seam win, fixed camera, chrome off, fallback, scene return (wave 4)
- [x] 38-09-PLAN.md — D-23 gates: 25/25 fingerprints, protected files, Rust/web checks, puzzle and scene suites (wave 5)

**UI hint**: yes

### Phase 39: Place-then-Play Tools and Stars

**Goal**: Players can solve a level by placing a limited set of fixed-size tools while building, pressing Play and earning up to three stars.
**Depends on**: Phase 38
**Requirements**: TOOL-01, TOOL-02, TOOL-03, TOOL-04, TOOL-06, TOOL-07, STAR-01, STAR-02
**Success Criteria** (what must be TRUE):

1. Player sees a per-level tool tray with fixed-size tools and remaining counts. With mouse or touch, they can tap to place, drag, rotate in 15° steps and remove tools using targets of at least 44 px, and they can do the same from the keyboard (arrows, R, Delete, Space).
1. Placements that overlap walls, the cup, water or other tools, or that leave the build area, are highlighted as invalid and Play stays disabled. The Rust validator rejects the same placements.
1. Play runs the level. Stop/Reset returns to building with placements kept and replays to the same outcome, and Clear removes all tools. Placing, removing, re-placing and resetting three times gives the same level fingerprint as a single clean placement.
1. Player sees the ★ Win, ★ Brim and ★ Thrifty criteria before playing. A winning run awards stars, and the best result per star is kept across runs.
1. Fast-forward at 2× and 4× gives the same outcome step and fill as 1×, a synthetic-clock e2e places a tool and wins (including a touch project), and 25/25 scene fingerprints are unchanged.

**Plans**: 14 plans

Plans:

- [x] 39-01-PLAN.md — Fingerprint baseline, then the Rust SAT placement validator, stateless inspector, newPuzzle parity and brim/par/no-place spec data (wave 1)
- [x] 39-02-PLAN.md — TS editor core: status codes and copy, building editor with live and committed inspections, hit-testing and editor view models (wave 1)
- [x] 39-03-PLAN.md — TS stars, best-of store, star rows, keyboard map and fast-forward helpers (wave 1)
- [x] 39-04-PLAN.md — TS frame parsing of peak held, Brim and decided step with fail-closed rules (wave 1)
- [x] 39-05-PLAN.md — Rust peak-held tracker, Brim latch, decided step, frame getters and permuted-order fingerprint (wave 2)
- [x] 39-06-PLAN.md — Rebuilt wasm, level-info budget/brim/par parsing, inspector decoding and status-code parity (wave 3)
- [x] 39-07-PLAN.md — Phase machine with editor, Stop, speed and the D-10 decision, plus HUD view models (wave 4)
- [x] 39-08-PLAN.md — Replay fixtures and harnesses with peak_held and brim_step (wave 4)
- [x] 39-09-PLAN.md — Shell wiring: preview/run sessions, editing with committed/live inspection, stars and held hash (wave 5)
- [x] 39-10-PLAN.md — Fast-forward step multiplier in the puzzle loop (wave 5)
- [x] 39-11-PLAN.md — Tool overlay, building-only pointer editor and debug attributes (wave 6)
- [x] 39-12-PLAN.md — Tool tray, selection bar, transport, star list, result card and keyboard-operable panel (wave 6)
- [x] 39-13-PLAN.md — Editor, keyboard, touch, edit-history and fast-forward e2e; Phase 38 D-10 literal updates (wave 7)
- [x] 39-14-PLAN.md — D-30 phase gates: 25/25 fingerprints, engine untouched, replays, suites and grep gates (wave 8)

**UI hint**: yes

### Phase 40: Timed Valve and Teaching Levels

**Goal**: Players can work through 5–8 teaching levels, each introducing one idea, and every level is proven solvable by automated tests.
**Depends on**: Phase 39
**Requirements**: TOOL-05, LEVEL-01, LEVEL-02, LEVEL-03, LEVEL-04
**Success Criteria** (what must be TRUE):

1. Player can place a timed valve and set its opening time while building. It opens at that simulation time with no input during the run, and the same opening time always gives the same outcome.
1. Player can play 5–8 levels, each with a one-line caption introducing one idea from plain water, viscous liquid, powder, jelly pad, buoyancy, timed valve and a finale combining earlier ideas. In the jelly pad level, the pad deflects water and its particles never count toward the fill.
1. For every level, tests show that a reference solution earning all three stars wins with margin, a run with no tools loses, and small grid nudges of the solution mostly still win. At least one solution replays to the same result in the browser as natively.
1. The first level shows a placement hint that the player can dismiss and that does not return after dismissal.
1. Each level has a portrait frame that fills a phone canvas (about 390 by 844 CSS pixels) and stays within its particle budget, and 25/25 scene fingerprints are unchanged.

**Plans**: 11 plans

Plans:
- [x] 40-01-PLAN.md — Fingerprint baseline, then Rust stride-4 lanes, ToolKind::Valve, status 13, 5/6-token fixture grammar and the pre-step valve opening (wave 1)
- [x] 40-02-PLAN.md — Bun fixture grammar and registry-derived fixture files; TS stride-4 lanes, valve code, status 13, editor and debug attribute (wave 2)
- [x] 40-03-PLAN.md — Liquid materials, system tuning and circle floats; tuning probe and the table-driven level proof harness (wave 2)
- [x] 40-04-PLAN.md — Valve time UI: setValveTime, [ and ] keys, the always-mounted stepper, labels, countdown and open outline (wave 3)
- [x] 40-05-PLAN.md — Caption leaf, one-time first-level hint with guarded storage, and hint e2e (wave 4)
- [x] 40-06-PLAN.md — thick-syrup and jelly-pad levels with fixtures, captions, frames and proofs (wave 5)
- [x] 40-07-PLAN.md — timed-gate valve level, open-tick window sweep, reproducibility and the browser valve replay (wave 6)
- [x] 40-08-PLAN.md — Powder spike, then ship sand-chute with a measured caption or drop it under D-25 (wave 7)
- [x] 40-09-PLAN.md — float-up buoyancy level, or drop under D-09 (wave 8)
- [x] 40-10-PLAN.md — grand-finale level and the every-level route e2e (wave 9)
- [x] 40-11-PLAN.md — Phase gates (D-24) and the decision record (wave 10)
**UI hint**: yes

### Phase 41: Level Select and Progress

**Goal**: Players can browse the levels from a Puzzles section and keep their stars, unlocks and last placements across visits in this browser.
**Depends on**: Phase 40
**Requirements**: PUZ-01, PROG-01, PROG-02, PROG-03, PROG-04
**Success Criteria** (what must be TRUE):

1. A Puzzles link in site navigation opens `#/puzzles`, a level grid showing each level's title, idea, stars and lock state.
1. Stars and solved levels persist across reloads, and a later worse run never lowers a saved star.
1. Levels unlock in order as earlier ones are solved, a direct link opens any level even when locked, and Next on the result card goes to the next level.
1. When storage is unavailable (a throwing getter or `setItem`, corrupt JSON, or a future version), every level is unlocked, the game stays fully playable, and a note says progress won't be saved.
1. Reopening a level restores the player's last placements, and saved placements are discarded when the level has changed.

**Plans**: 8 plans

Plans:
- [x] 41-01-PLAN.md — Fingerprint baseline, then the pinned Rust `levelHash` getter, TS parse and the wasm replay pin check (wave 1)
- [x] 41-02-PLAN.md — Pure progress core: v1 document parse/serialize/merge, unlock rule, grid view model and restore decision (wave 2)
- [x] 41-03-PLAN.md — Progress shell with probe and degraded mode; star hydration and write-through; controller degraded flag (wave 3)
- [x] 41-04-PLAN.md — Restore saved placements on level entry and save after committed edits and at Play (wave 4)
- [x] 41-05-PLAN.md — `#/puzzles` route, level grid page and the sidebar Puzzles link (wave 5)
- [x] 41-06-PLAN.md — All levels links: options sheet with note, unknown-level fallback, last-level result action (wave 6)
- [x] 41-07-PLAN.md — Playwright puzzle-progress spec on desktop and phone, added to test:puzzle (wave 7)
- [x] 41-08-PLAN.md — Phase gates (D-21) and the decision record (wave 8)

**UI hint**: yes

### Phase 42: Opt-in Sound

**Goal**: Players who turn sound on hear the puzzle: splashes, impacts, plinks that rise as the cup fills, and a win chime.
**Depends on**: Phase 40 (real levels to tune against)
**Requirements**: SOUND-01, SOUND-02, SOUND-03
**Success Criteria** (what must be TRUE):

1. Sound is off by default, and no audio starts until the player turns it on. The toggle is remembered across visits.
1. With sound on, the player hears splash and impact sounds, plinks that rise in pitch as the cup fills, and a win chime, with a cap on simultaneous sounds.
1. Sound pauses when the tab is hidden or the player leaves the puzzle, and a hint mentions the phone silent switch.
1. The impact data the bridge reports for sound does not change the simulation: 25/25 scene fingerprints are unchanged, and puzzle outcomes are the same with sound on and off.

**Plans**: 5 plans

Plans:
- [x] 42-01-PLAN.md — Fingerprint baseline, read-only Rust sound aggregates on puzzle frames with determinism tests and tuning probe, fail-closed TS parse (wave 1)
- [x] 42-02-PLAN.md — Pure sound core: preference, per-frame fold, pentatonic plinks and voice params, 6-voice cap with rate limits, trigger rules (wave 2)
- [x] 42-03-PLAN.md — Web Audio engine shell, puzzle sound glue (gesture unlock, visibility, failure) and loop/lifecycle wiring (wave 3)
- [x] 42-04-PLAN.md — HUD sound toggle, drawer and form-factor Sound section with silent-switch hint, slot widening (wave 4)
- [x] 42-05-PLAN.md — Fake-AudioContext Playwright spec and the full phase gate with 25/25 fingerprints (wave 5)

**UI hint**: yes

### Phase 43: GPU Water Render Mode

**Goal**: Players can view water with refraction, highlights and foam in scenes and puzzles, with fallbacks that never leave a blank canvas.
**Depends on**: Phase 38 (puzzle player); runs last so it can slip without blocking the game
**Requirements**: WATER-01, WATER-02, WATER-03
**Success Criteria** (what must be TRUE):

1. Player can choose a "Water" render mode that shows refraction, highlights and foam on fast-moving water, in scenes and in puzzles, with the cup, meter and tool overlays drawn on top.
1. When the device cannot run the mode, the view falls back to shaded blob and then canvas. When the WebGL context is lost, it falls back and then recovers when the context is restored.
1. Exports keep their existing styles while Water mode is selected.
1. The particle speed data used for foam is produced only while Water mode is active: 25/25 scene fingerprints are unchanged, and puzzle outcomes are the same in every render mode.

**Plans**: 6 plans

Plans:
- [x] 43-01-PLAN.md — Fingerprint baseline, Rust per-capture speed lane with on/off determinism tests, fail-closed TS lane parse (wave 1)
- [x] 43-02-PLAN.md — Shared per-canvas field pass, half-float render targets, ordered loss/restore repaint hook (wave 1)
- [x] 43-03-PLAN.md — Stable-walls refraction backdrop, highp water shaders, capability probe and drawWater (wave 2)
- [x] 43-04-PLAN.md — Water mode, fallback chain with data-drawn-surface, transparent overlays, shared pickers with "Water (unavailable)" (wave 3)
- [x] 43-05-PLAN.md — Speed lane only in Water for scene, puzzle and worker owners (wave 4)
- [x] 43-06-PLAN.md — Playwright Water spec, by-eye tuning and the phase gates with 25/25 fingerprints (wave 5)
**UI hint**: yes

### Phase 44: v1.5 Polish and Test Gaps

**Goal**: Close the owner-selected tech debt from the v1.5 milestone audit so the puzzle game reads well on phones, is accessible, and its e2e code and remaining flows are covered by checks.
**Depends on**: Phase 43
**Requirements**: None new (tech-debt cleanup; touches TOOL-01, TOOL-05, TOOL-07, PUZ-01, PROG-02 behavior without changing them)
**Gap Closure**: Tech debt from `.planning/v1.5-MILESTONE-AUDIT.md` (no blocking gaps)
**Success Criteria** (what must be TRUE):

1. On a 390×844 phone viewport, every tool name in every level's tray (including grand-finale's planks and valve) is shown in full, not truncated, and the tray keeps its fixed HUD obstacle box and 44 px touch targets.
1. A selected valve's opening time is announced to assistive technology (not `aria-hidden`), and the TypeScript valve tick is capped at 255 to match the Rust lane.
1. An e2e test wins a level, clicks Next and lands on the next level ready to build; another runs the timed-gate reference at 2× and 4× and reaches the same outcome step as 1×.
1. `web/playwright.config.ts` and the `web/e2e` helpers are inside the TypeScript typecheck and `npx tsc --noEmit` passes; puzzle outcomes and the 25 scene fingerprints are unchanged.

**Plans**: 3 plans

Plans:
- [ ] 44-01-PLAN.md — Valve tick cap and seam open-tick default (unit), stacked portrait two-kind tray chip, valve readout as the stepper description, tray-fit e2e at 390/360 wide (wave 1)
- [ ] 44-02-PLAN.md — Strict tsconfig.e2e.json in `bun run typecheck` and root fixes for the 16 e2e type errors (wave 1)
- [ ] 44-03-PLAN.md — Next-from-won and timed-gate 1×/2×/4× e2e with valve ARIA hooks, then the 10 phase gates and grand-finale screenshot (wave 2)
**UI hint**: yes

## Progress

**Execution order:** 37 → 38 → 39 → 40 → 41 → 42 → 43 → 44. The shelved 27-phase v1.4 plan is preserved in git history (commit `011e80e`) and on branch `wip/v1.4-phase34-harness`.

| Phase | Milestone | Plans Complete | Status | Completed |
| --- | --- | --- | --- | --- |
| 16. Rust WASM Browser Bridge | v1.1 | 4/4 | Complete | 2026-09-17 |
| 17. Shared Player and Early Pages Delivery | v1.1 | 8/8 | Complete | 2026-09-17 |
| 18. Six Native Physics Demos | v1.1 | 10/10 | Complete | 2026-09-18 |
| 19. Interaction Polish and Browser Verification | v1.1 | 7/7 | Complete | 2026-09-19 |
| 20. Playground catalog previews and Reset honesty | v1.1 | 6/6 | Complete | 2026-09-20 |
| 21. Playground leftover cleanup | v1.1 | 4/4 | Complete | 2026-09-20 |
| 22. Observability shell | v1.2 | 6/6 | Complete | 2026-09-21 |
| 23. Baseline pair and named audit | v1.2 | 9/9 | Complete | 2026-09-21 |
| 24. Shared hot-path waves through 3× | v1.2 | 6/6 | Complete | 2026-09-21 |
| 25. WASM sanity and honest close | v1.2 | 2/2 | Complete | 2026-09-21 |
| 26. Catalog shell and basin scenes | v1.3 | 5/5 | Complete    | 2026-09-22 |
| 27. Material flag groups | v1.3 | 6/6 | Complete    | 2026-09-22 |
| 28. Interaction seams | v1.3 | 8/8 | Complete    | 2026-09-22 |
| 29. Sparky, Drawing, and full catalog | v1.3 | 5/5 | Complete    | 2026-09-27 |
| 30. Periodic hydraulic fountain | v1.3 | 3/3 | Complete    | 2026-09-27 |
| 31. Sinusoidal wave tank | v1.3 | 3/3 | Complete    | 2026-09-27 |
| 32. Liquid motion bubbler | v1.3 | 4/4 | Complete    | 2026-09-27 |
| 33. Stacked drip fidget | v1.3 | 4/4 | Complete   | 2026-09-28 |
| 34. Scene Timing Survey | v1.4 | 2/2 | Complete    | 2026-10-07 |
| 35. Speed Up the Slowest Scenes | v1.4 | 8/8 | Complete    | 2026-10-10 |
| 36. Re-survey and Wrap Up | v1.4 | 3/3 | Complete    | 2026-10-11 |
| 37. Puzzle Core in the Bridge | v1.5 | 8/8 | Complete    | 2026-10-12 |
| 38. Puzzle Route and Player Shell | v1.5 | 9/9 | Complete    | 2026-10-12 |
| 39. Place-then-Play Tools and Stars | v1.5 | 14/14 | Complete    | 2026-10-13 |
| 40. Timed Valve and Teaching Levels | v1.5 | 11/11 | Complete    | 2026-10-13 |
| 41. Level Select and Progress | v1.5 | 8/8 | Complete    | 2026-10-13 |
| 42. Opt-in Sound | v1.5 | 5/5 | Complete    | 2026-10-13 |
| 43. GPU Water Render Mode | v1.5 | 6/6 | Complete    | 2026-10-14 |
| 44. v1.5 Polish and Test Gaps | v1.5 | 0/TBD | Not started | - |

v1.0 phases 1–15 remain in the [v1.0 roadmap archive](milestones/v1.0-ROADMAP.md).
