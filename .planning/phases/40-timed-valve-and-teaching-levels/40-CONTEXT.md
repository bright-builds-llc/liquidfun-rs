---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 40-2026-10-13T03-11-05
generated_at: 2026-10-13T03:12:06.379Z
---

# Phase 40: Timed Valve and Teaching Levels - Context

**Gathered:** 2026-10-12
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

This phase adds a timed valve tool and grows the level registry from one level (`first-pour`) to a teaching set of 5–8 levels. Each level introduces one idea with a one-line caption, and automated tests prove that every level can be solved. The first level shows a one-time, dismissible placement hint. Each level gets a portrait frame and a particle budget. The 25 scene fingerprints stay unchanged.

Requirements: TOOL-05, LEVEL-01, LEVEL-02, LEVEL-03, LEVEL-04.

Out of this phase:

- The `#/puzzles` grid, the Puzzles nav entry, persisted progress, unlocks and last-placement restore (Phase 41).
- Sound (Phase 42).
- GPU water (Phase 43).
- Live valve tapping (VALVE-02).
- Hint ladders (HINT-01).
- More levels beyond the teaching set (LEVEL-05).

</domain>

<decisions>
## Implementation Decisions

### Timed valve tool (TOOL-05)

- **D-01:** The valve is a new tool kind, `ToolKind::Valve`, with wire code 2. It is a fixed-size static gate, shorter than the plank. A size of about 1.2 m × 0.25 m is a starting point, at Claude's discretion. It exists from step 0 and blocks flow until its open step. At the open step, the bridge destroys the gate body with the public `World::destroy_body`. The gate does not slide or animate. Its outline uses the same baked τ/24 vertices and SAT validation as the plank. It rotates in 15° steps like the plank.
- **D-02:** The pose lane becomes stride 4 for every kind: `(x_grid, y_grid, angle_index, open_tick)`. This uses the 4th value that Phase 37 D-08 reserved. `open_tick` counts half-seconds, so the valve opens at step `open_tick × 30`. For a valve, `open_tick` must be in `1..=step_limit / 30`. Non-valve kinds must send `0`, and any other value fails closed. Integer ticks keep snapping exact and outcomes reproducible. Canonical order sorts by `(kind, x, y, angle, open_tick)`.
- **D-03:** The valve opens before `world.step` runs for its open step index. Use the session's own step index, never wall-clock time or rendered frames. `MAX_ADVANCE_STEPS` stays 4. The gate lifecycle lives in puzzle code, either a new `puzzle/valve.rs` or `PuzzleHooks`. If the existing hooks only run after a step, add a pre-step puzzle hook, or open the valve after step `open_step - 1`, which is equivalent. Pin the chosen order in a unit test. Scene hook behavior must stay byte-for-byte unchanged.
- **D-04:** The Rust validator owns valve checks and adds a reason such as "Valve time out of range", reported through `inspectPuzzlePlacements`. `newPuzzle` fails closed on the same check, as in Phase 39 D-03. Valve overlap rules are the same as the plank's.
- **D-05:** A newly placed valve opens at 2.0 s (`open_tick = 4`) by default. While building, the player sets the time with a −/+ 0.5 s stepper in the existing selection bar, which is a fixed HUD slot. The stepper is always mounted. It is disabled unless a valve is selected while building, so the obstacle set never changes. The default keys are `[` and `]`, at Claude's discretion; they must not clash with the Phase 39 keys. The time edit is a new `reducePuzzle` event (for example `setValveTime`), and it is ignored outside building.
- **D-06:** The overlay draws each valve with its open time as a label, such as "2.0 s". While running, the valve counts down. Once the frame step reaches its open step, the valve is drawn as a faint open outline, so the drawing matches the destroyed body. This is derived in TS from the frame step and the placement, and no new frame scalar is needed.
- **D-07:** Fixtures gain an optional 5th field, `placement valve <x> <y> <angle> <open_tick>`. Plank lines stay 4 fields and mean `open_tick` 0. The existing `first-pour` fixtures must replay to the same `outcome`, `outcome_step`, `final_count`, `peak_held` and `brim_step`. If any value changes, stop and investigate. Both the Rust and Bun fixture parsers learn the new field.
- **D-08:** The same opening time must always give the same outcome. Prove this with a Rust test that builds the valve level twice with identical placements, and with the release-native vs wasm replay of the valve level's reference fixture.

### Level set (LEVEL-01)

- **D-09:** Ship **7 levels** in this order, each teaching one idea. Ids and titles are at Claude's discretion; working defaults are shown here.

  1. `first-pour`: plain water. The geometry, fixtures and stars stay unchanged.
  1. Viscous (`thick-syrup`): a slow, clinging liquid that needs a steeper or longer path.
  1. Powder (`sand-chute`): grains pile up instead of levelling out, so the tools must funnel them.
  1. Jelly pad (`jelly-pad`): a soft elastic deflector that redirects water and never counts.
  1. Buoyancy (`float-up`): a floating body that matters to the solution.
  1. Timed valve (`timed-gate`): the open time decides the outcome.
  1. Finale (`grand-finale`): combines at most three earlier ideas, including the valve.

  If buoyancy cannot pass the D-15 solvability tests after reasonable tuning, drop it and ship 6 levels, which is still within 5–8. Record the drop in the phase summary. Do not drop any other level.
- **D-10:** `LiquidBlock` gains a material field mapped to `ParticleFlags`: water, viscous, powder or elastic. Make it a small enum or a flags constant, at Claude's discretion. `first-pour` keeps exactly its current flags, so its fixtures are unchanged. Counting stays group-based through the existing `counted` flag. Material blocks are still created after the tools, in authored order.
- **D-11:** The jelly pad is level-authored, not a tool. It is an elastic particle group with `counted: false`, resting on or anchored by static level geometry so that it stays in the stream's path. The player's planks route water onto it so that it deflects the water toward the cup. A unit test shows that pad particles never count, even when they sit inside the cup interior. If tuning shows a fixed pad cannot teach the idea, Claude may make it a placeable tool instead, and must record the change in the summary.
- **D-12:** The buoyancy level uses a level-authored dynamic body with a float-like density, following the `scene/float_or_sink.rs` precedent. The cup stays static, and the counted region (`cup_interior`) never moves, so no moving-cup region transforms are needed. The mechanism is at Claude's discretion during tuning, for example a float that rises with the water and tips or blocks flow. Buoyancy must be necessary to the reference solution.
- **D-13:** The valve level must make timing matter. A run with the valve opened too early loses, a run with it opened too late loses (out of time), and a time window wins. A suggested design that uses only static geometry: two reservoirs share a narrow chute. Releasing both at once overflows the chute and spills, and staggering the releases with a valve gets both through. Prove the window with a Rust test that sweeps `open_tick` across its range.
- **D-14:** Each level is a const `LevelSpec` in its own file under `puzzle/`, following `first_pour.rs`. Each is added to `LevelId`, `LEVEL_IDS`, `level_spec`, the TS `PUZZLE_IDS` (the `include_str!` allowlist test must pass) and `PUZZLE_TITLES`. Add a new tool kind (for example a short plank) only when a level needs one, per Phase 39 D-06. Tool budgets, `par_tools` and `brim_count` are authored per level by the Phase 39 D-07 rule: Brim is at least target + 10 percentage points, and the reference peak held clears Brim by at least 5% of `full_count`. Next on the result card now goes to the following level, and on the last level it stays disabled with "No next level yet".

### Solvability proof (LEVEL-02)

- **D-15:** Each level has recorded fixtures under `puzzle/fixtures/`: `<id>-solution.txt`, a reference solution that earns all three stars (Win, Brim, and placement count ≤ par), and `<id>-empty.txt`, a run with no tools that loses. "Wins with margin" means two things. The Win latches at least 60 steps (1 s) before `step_limit`. The fixture also meets the Phase 37 D-03/D-22 fill-margin fallback that the debug test lane asserts.
- **D-16:** The nudge sweep moves each reference placement ±1 and ±2 grid cells in x and in y, one placement and one axis at a time. Valves additionally get ±1 `open_tick`. Invalid nudged sets are skipped and counted separately. "Mostly still win" means at least 70% of the valid nudged variants still reach the Win outcome. The sweep runs only in the release lane (`cargo test -p liquidfun-wasm --release`, via `#[cfg_attr(debug_assertions, ignore)]` or an equivalent), because debug runs are slow. The default debug lane still runs each level's solution and empty fixtures. Keep all puzzle tests in the existing lib test binary (Phase 37 D-20).
- **D-17:** `just puzzle-replay-wasm` replays every level's fixtures through the generated wasm. Add one Playwright e2e that replays a reference solution in the browser and sees the win card. Use the valve level, because it exercises the stride-4 lane and the valve time UI, by setting the time through the stepper. This satisfies "at least one solution replays in the browser".
- **D-18:** Tuning is empirical and is the largest schedule risk. Use test-only probe helpers like those in Phase 37-06 to print outcomes, peak held and outcome steps while authoring. Do not ship probe output as evidence. Only the recorded fixtures and the assertions count.

### First-level hint (LEVEL-03)

- **D-19:** Add a fixed-size **caption** HUD leaf that shows each level's one-line caption. It carries `data-hud-obstacle`, mounts with the puzzle route, and keeps a box size that is fixed per layout. On `first-pour`, until the hint is dismissed, the same leaf shows the placement hint instead, for example "Tap the plank, then tap the canvas to place it", with a Dismiss button of at least 44 px. Showing or hiding the hint never changes the obstacle set, so the camera never refits. This is required by Phase 38 D-18.
- **D-20:** Dismissal is permanent. It is stored under a versioned localStorage key, and every access is wrapped in try/catch. If storage is unavailable, the dismissal holds in memory for the page session. Placing the first tool on `first-pour` also counts as dismissal. Keep the storage helper small and separate so Phase 41's `progress.ts` can absorb or reuse it. Add Vitest coverage for a throwing getter, a throwing `setItem` and an already-dismissed state.
- **D-21:** Captions live in TS level metadata next to `PUZZLE_TITLES`, keyed by `PuzzleId`, so Phase 41's grid can show them before wasm loads. They are not cross-checked against Rust. Titles keep the D-26 cross-check.

### Frames and budgets (LEVEL-04)

- **D-22:** Author new levels to suit portrait: roughly 7–9 m wide and 9–12 m tall, unlike `first-pour`'s 12 m width. Then each portrait rectangle in `puzzle-bounds.ts` fills a 390×844 phone well. Every level gets landscape and portrait rectangles that contain the cup, the walls, the build area and the liquid. `web/tests/puzzle-bounds.test.ts` covers every `PuzzleId` under the Phase 38 D-21 rule, using level geometry hardcoded from each Rust spec.
- **D-23:** The particle budget is at most **3,000 particles per level at step 0**, including jelly pad particles. `maximum_particle_count` is authored per level with headroom. A Rust unit test asserts the step-0 particle count against the budget for every level. Record a one-off release-native ms/step per level in the phase summary for information only; it does not gate the phase.

### Gates

- **D-24:** The phase is done only if all of these hold:
  - `just playground-scene-spot` reports 25/25 fingerprints unchanged.
  - `crates/liquidfun` is untouched.
  - Every level's fixtures pass in the debug lane, the release lane and `just puzzle-replay-wasm`.
  - The nudge sweep passes in release.
  - The puzzle e2e specs pass, including the new valve browser replay and a hint dismissal check.
  - The normal local checks pass: fmt, clippy, build, tests, and the web unit tests and type check.

### Claude's Discretion

- Exact level ids, titles, captions, geometry, materials' tuning values, and the buoyancy and jelly mechanisms within D-11 and D-12.
- The valve's exact size, default time and keyboard keys within D-05.
- Whether valve logic sits in `valve.rs` or in `PuzzleHooks`.
- Field names for the material enum.

### Folded Todos

None. No pending todos matched the phase.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope and rules

- `.planning/ROADMAP.md`: Phase 40 success criteria and the shared acceptance rules (fingerprints, registry separation, per-step outcome, construction-time placements).
- `.planning/REQUIREMENTS.md`: TOOL-05, LEVEL-01 through LEVEL-04, and the Out of Scope table.
- `.planning/research/v1.5-puzzle-mode/SUMMARY.md`: Phase 40 delivery notes, and the valve-model and sponge-semantics decisions.
- `.planning/research/v1.5-puzzle-mode/PITFALLS.md`: Pitfalls 3, 6 and 14, which Phase 40 must avoid.
- `PROJECT-SCOPE.md`: hobby-project scope and the experimental policies.

### Prior phase decisions

- `.planning/phases/37-puzzle-core-in-the-bridge/37-CONTEXT.md`: D-03/D-22 (determinism and margin policy), D-08 (pose lanes with the reserved 4th value), D-18/D-19 (fixtures and replay) and D-20 (a single test binary).
- `.planning/phases/38-puzzle-route-and-player-shell/38-CONTEXT.md`: D-15 (Next), D-18 (fixed HUD leaves), D-20/D-21 (puzzle bounds and their test) and D-26 (the title table).
- `.planning/phases/39-place-then-play-tools-and-stars/39-CONTEXT.md`: D-01 through D-07 (validator, inspector, budgets, brim/par rule), D-19 (selection bar), D-23 (keys), D-25 (reducer events) and D-27/D-28 (edit-history and e2e gates).

### Code anchors

- `crates/liquidfun-wasm/src/puzzle.rs`, `puzzle/spec.rs`, `puzzle/first_pour.rs`, `puzzle/tools.rs`, `puzzle/validate.rs`, `puzzle/builder.rs`, `puzzle/hooks.rs` and `puzzle/replay.rs`: the level registry, specs, tools, validation, the build order, step hooks and fixture replay.
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs`: the buoyancy precedent.
- `web/src/catalog/puzzles.ts`, `web/src/catalog/puzzle-levels.ts` and `web/src/catalog/puzzle-bounds.ts`: the TS registry, titles and frames.
- `web/src/puzzle/reduce-puzzle.ts`, `web/src/puzzle/placements.ts`, `web/src/puzzle/keyboard.ts` and `web/src/puzzle/editor.ts`: the editor state, lanes and keys.
- `web/e2e/puzzle*.spec.ts`: the existing puzzle e2e coverage and helpers.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `LevelSpec`, `LiquidBlock` (it already has `counted`), `GridRect`, `ToolAllowance` and the `quad` helper make adding a level mostly a matter of data.
- The SAT validator and `inspectPuzzlePlacements` extend to a new kind through the kind table.
- The replay fixtures, the Rust and Bun replays and the re-record flag extend to new levels with no new harness.
- The scene code already uses the `VISCOUS`, `POWDER` and `ELASTIC` flags, so the materials are proven in the engine.

### Established Patterns

- Builds are created in a fixed order: level geometry, then tools in canonical order, then the liquid. The world is rebuilt at step 0 on every Play and Reset.
- The HUD uses fixed slots with `data-hud-obstacle` on leaves, and the obstacle set never changes between phases.
- Puzzle tests stay unit tests in the `liquidfun-wasm` lib binary, because each new macOS binary costs about 2 minutes of launch scan.

### Integration Points

- `POSE_STRIDE` changes in `tools.rs`, the TS lane encoder, the fixtures and `inspectPuzzlePlacements`.
- `PUZZLE_IDS` and `LEVEL_IDS`, `PUZZLE_TITLES`, the new captions, `PUZZLE_VIEW_BOUNDS` and `PUZZLE_PORTRAIT_BOUNDS`.
- The selection bar (valve stepper), the overlay (valve label and open state), and the caption/hint HUD leaf.

</code_context>

<specifics>
## Specific Ideas

- Owner decision (v1.5): the valve is timed and set while building, with no live tapping. The elastic level is a "jelly pad" whose particles never count.
- Captions are one line and introduce exactly one idea. The finale says what it combines.

</specifics>

<deferred>
## Deferred Ideas

- Live valve tapping during a run (VALVE-02).
- A hint ladder per level (HINT-01).
- A sponge absorption mechanic (SPONGE-01).
- More levels (LEVEL-05).
- Moving or buoyant cups with a moving count region, which this phase avoids by D-12.

</deferred>

---

*Phase: 40-timed-valve-and-teaching-levels*
*Context gathered: 2026-10-12*
