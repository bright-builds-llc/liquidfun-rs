# Requirements: liquidfun-rs v1.5 Puzzle Mode

**Defined:** 2026-10-11
**Core Value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

**Milestone goal:** Add a game to the playground: place a few tools while a level is paused, press play, and get enough water into the cup before the timer runs out.

Research: `.planning/research/v1.5-puzzle-mode/SUMMARY.md`.

## v1.5 Requirements

### Cup goal and outcome

- [x] **GOAL-01**: Player sees a live fill % for the cup, counting only the level's water particles whose centres are inside the cup, against a fixed capacity authored per level.
- [x] **GOAL-02**: Player wins when the fill stays at or above the target for a short settle time (about 0.5 s of simulation) before the time limit; a splash passing through does not win.
- [x] **GOAL-03**: Player sees a timer that runs on simulation time and only during a run; the run fails with a message such as "Out of time: 54% of 80%" when the limit passes without a win.
- [ ] **GOAL-04**: The same level and placements always produce the same outcome regardless of device speed; a recorded solution replays to the same result natively and in the browser.

### Tools and play

- [x] **TOOL-01**: Player sees a per-level tool tray with fixed-size tools and remaining counts.
- [x] **TOOL-02**: While building, player can place, move, rotate (15° steps) and remove tools with mouse or touch, including tap-to-place and touch targets of at least 44 px.
- [x] **TOOL-03**: Player sees invalid placements highlighted (overlapping walls, the cup, water or other tools, or outside the build area), and Play is disabled while any placement is invalid.
- [x] **TOOL-04**: Player can press Play to run, and Stop/Reset to return to building with placements kept; a separate Clear action removes all tools.
- [x] **TOOL-05**: Player can place a timed valve and set its opening time while building.
- [x] **TOOL-06**: Player can place, nudge, rotate, remove and play from the keyboard (arrows, R, Delete, Space).
- [x] **TOOL-07**: Player can fast-forward a run at 2× or 4× without changing its outcome.

### Stars

- [x] **STAR-01**: Player sees three star criteria before playing: ★ Win, ★ Brim (a higher fill held) and ★ Thrifty (at most the par tool count).
- [x] **STAR-02**: Player earns stars on a winning run, and the best result per star is kept across runs.

### Levels

- [x] **LEVEL-01**: Player can play 5–8 teaching levels, each introducing one idea with a one-line caption, drawn from: plain water, viscous liquid, powder, jelly pad (an elastic deflector whose particles never count), buoyancy, timed valve, and a finale combining earlier ideas.
- [x] **LEVEL-02**: Every level is proven solvable by automated tests: a reference solution earning all three stars wins with margin, a run with no tools loses, and small nudges of the solution mostly still win.
- [x] **LEVEL-03**: The first level shows a one-time, dismissible placement hint.
- [x] **LEVEL-04**: Each level has its own portrait frame that fills a phone canvas and stays within a particle budget that runs smoothly on phones.

### Puzzle section

- [ ] **PUZ-01**: Player can open a Puzzles section (`#/puzzles`) from site navigation and see a level grid with title, idea, stars and lock state.
- [x] **PUZ-02**: Player can open any level directly at `#/puzzle/<id>`; an unknown id shows a helpful fallback.
- [x] **PUZ-03**: In puzzles, scene interactions (pointer poke and drag, tilt, gravity slider, zoom and pan) are disabled, and the camera does not jump when switching between building and running.
- [x] **PUZ-04**: Player sees a puzzle HUD that follows existing chrome rules (`data-hud-obstacle` leaves, scale legend): a fill meter at the cup with target line and star ticks, the timer, and a result card with Next, Retry and Edit.
- [x] **PUZ-05**: Player sees the cup drawn as glass with an etched target line and star ticks.
- [x] **PUZ-06**: The 25 existing scenes behave exactly as before: 25/25 end-state fingerprints unchanged and existing tests passing.

### Progress

- [ ] **PROG-01**: Player's stars and solved levels persist in this browser across visits.
- [ ] **PROG-02**: Levels unlock in order as earlier ones are solved, and direct links always open any level.
- [ ] **PROG-03**: When storage is unavailable, every level is unlocked, the game stays fully playable, and a note says progress won't be saved.
- [ ] **PROG-04**: Each level reopens with the player's last placements, which are discarded if the level has changed.

### Sound

- [ ] **SOUND-01**: Sound is off by default; player can turn it on with a toggle that is remembered.
- [ ] **SOUND-02**: With sound on, player hears splash and impact sounds, rising-pitch plinks as the cup fills, and a win chime, with a cap on simultaneous sounds.
- [ ] **SOUND-03**: Sound pauses when the tab is hidden or the player leaves the puzzle, and a hint mentions the phone silent switch.

### GPU water

- [ ] **WATER-01**: Player can choose a "Water" render mode with refraction, highlights and foam on fast-moving water.
- [ ] **WATER-02**: When the device cannot run it or the WebGL context is lost, the view falls back to shaded blob and then canvas, and recovers when the context is restored.
- [ ] **WATER-03**: Water mode works in scenes and puzzles, and exports keep their existing styles.

## Future Requirements

Deferred. Tracked but not in the current roadmap.

- **HINT-01**: Hint ladder per level (text nudge, then a ghost of one reference-solution tool), free and star-neutral.
- **BADGE-01**: Determinism note in the level UI ("Your solution always plays out the same").
- **TRAIL-01**: Ghost of the last run's water path.
- **ROTATE-01**: Two-finger rotate on touch.
- **GOAL-05**: Color-filter or multi-cup goals.
- **VALVE-02**: Live valve tapping during a run, recorded by step.
- **SPONGE-01**: Sponge absorption mechanic.
- **LEVEL-05**: More levels beyond the teaching set.
- **EDITOR-01**: Level editor and share links (SEED-002).

## Out of Scope

| Feature | Reason |
| --- | --- |
| Accounts, servers, leaderboards | Hobby static site; progress is per browser |
| Wall-clock timers | Would make outcomes device-dependent; timing is in simulation steps |
| Star-gated unlocks | Blocks players with only 5–8 levels; soft locks chosen |
| Lives, energy, hint currency | Anti-features for a free hobby game |
| Freeform tool resize, unlimited tools | Undermines the puzzle; fixed sizes and per-level budgets |
| Poking or tilting during a run | Breaks place-then-play and reproducible solutions |
| Engine changes that alter scene fingerprints | The 25 scenes must stay unchanged |
| Crate or package publication | Separately authorized |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
| --- | --- | --- |
| GOAL-01 | Phase 38 | Complete |
| GOAL-02 | Phase 37 | Complete |
| GOAL-03 | Phase 38 | Complete |
| GOAL-04 | Phase 37 | Pending |
| TOOL-01 | Phase 39 | Complete |
| TOOL-02 | Phase 39 | Complete |
| TOOL-03 | Phase 39 | Complete |
| TOOL-04 | Phase 39 | Complete |
| TOOL-05 | Phase 40 | Complete |
| TOOL-06 | Phase 39 | Complete |
| TOOL-07 | Phase 39 | Complete |
| STAR-01 | Phase 39 | Complete |
| STAR-02 | Phase 39 | Complete |
| LEVEL-01 | Phase 40 | Complete |
| LEVEL-02 | Phase 40 | Complete |
| LEVEL-03 | Phase 40 | Complete |
| LEVEL-04 | Phase 40 | Complete |
| PUZ-01 | Phase 41 | Pending |
| PUZ-02 | Phase 38 | Complete |
| PUZ-03 | Phase 38 | Complete |
| PUZ-04 | Phase 38 | Complete |
| PUZ-05 | Phase 38 | Complete |
| PUZ-06 | Phase 38 | Complete |
| PROG-01 | Phase 41 | Pending |
| PROG-02 | Phase 41 | Pending |
| PROG-03 | Phase 41 | Pending |
| PROG-04 | Phase 41 | Pending |
| SOUND-01 | Phase 42 | Pending |
| SOUND-02 | Phase 42 | Pending |
| SOUND-03 | Phase 42 | Pending |
| WATER-01 | Phase 43 | Pending |
| WATER-02 | Phase 43 | Pending |
| WATER-03 | Phase 43 | Pending |

**Coverage:**

- v1.5 requirements: 33 total
- Mapped to phases: 33
- Unmapped: 0 ✓

---
*Requirements defined: 2026-10-11*
*Last updated: 2026-10-11 after v1.5 roadmap creation (Phases 37–43)*
