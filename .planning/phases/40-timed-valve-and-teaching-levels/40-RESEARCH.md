# Phase 40: Timed Valve and Teaching Levels - Research

**Researched:** 2026-10-12
**Domain:** Deterministic puzzle levels in the private `liquidfun-wasm` bridge (Rust), plus the SolidJS puzzle shell (TS)
**Confidence:** HIGH for the integration points and the valve mechanism (read from HEAD `c53d7a0bf` and measured). MEDIUM for level tuning (measured with an out-of-repo probe, but every level still needs empirical authoring). LOW for the powder lesson as currently worded. Measurements show it does not behave as described (see Open Question 1).

## Summary

Every integration point is a small, local change, and no engine change is needed. The session already runs a **pre-step hook**: `SessionCore::advance` calls `hooks.on_advance` before eviction and before `world.step` (`crates/liquidfun-wasm/src/session.rs:215-245`), and `PuzzleHooks::on_advance` is a no-op today (`puzzle/hooks.rs:47-53`). The valve can therefore open inside `PuzzleHooks::on_advance` when `observed_step + 1 == open_tick * 30`, with no change to `session.rs`, the `SceneHooks` trait or any scene. Destroying a static body mid-run through the public `World::destroy_body` has two shipped precedents: Tesla Valve (`scene/tesla_valve.rs:226-244`) and Hydraulic Fountain (`scene/hydraulic_fountain.rs:478-497`), which destroys a plate while particles touch it. Particle-body contacts are regenerated from live fixtures every step (`crates/liquidfun/src/world/particle_coupling.rs:205-245`). Destroying a contact wakes any rigid body that was touching (`contact_manager.rs:429-432`).

The stride change 3→4 is mechanical but wide. There are about 70 stride-3 pose literals in Rust tests, several tests that currently assert "valve is rejected" or "kind code 2 is unknown", the TS lane encoder, the Bun fixture grammar, the debug attribute and the e2e helpers (all listed below). If the builder creates planks and water exactly as today, the `first-pour` fixtures stay bit-identical.

The out-of-repo probe (release, radius 0.04 m, same API calls as `puzzle/builder.rs`) found **four facts that should shape the plan**:

1. A player-placed gate leaks heavily. The validator requires more than one particle diameter (0.08 m) of clearance from every wall (`validate.rs:263`). With that clearance, a water reservoir loses about 47% of its contents through the slit in 6 s. Viscous liquid at `viscous_strength` 1.0 loses about 3% in the same time.
1. `POWDER` behaves almost exactly like water in this engine. It spreads the same, drains the same and moves along ramps the same. It does not pile up.
1. `VISCOUS` at the default strength (0.25) is barely distinguishable from water. At 1.0–2.0 it crawls down a 15° ramp. The step fails with `ParticleLifecycleInvariant` at 2.5 and above.
1. A `SOLID` elastic pad in a static cradle stays put and keeps water out. Without `SOLID`, water soaks into it. A density-0.3 circle float rises within 2 s.

Release-native step cost is about 0.5 ms/step at 2,000 particles and about 0.79 ms/step at 3,000.

**Primary recommendation:** Open the valve in `PuzzleHooks::on_advance` and drop its segments in the same call. Land the stride-4 change with the `first-pour` fixtures unchanged before any tuning. Add a per-level `material` on `LiquidBlock` plus an optional system-strength override on `LevelSpec`. Run material and valve spikes before authoring geometry, and resolve the powder and valve-leak design questions early.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Timed valve tool (TOOL-05)

- **D-01:** The valve is a new tool kind, `ToolKind::Valve`, with wire code 2. It is a fixed-size static gate, shorter than the plank. A size of about 1.2 m × 0.25 m is a starting point, at Claude's discretion. It exists from step 0 and blocks flow until its open step. At the open step, the bridge destroys the gate body with the public `World::destroy_body`. The gate does not slide or animate. Its outline uses the same baked τ/24 vertices and SAT validation as the plank. It rotates in 15° steps like the plank.
- **D-02:** The pose lane becomes stride 4 for every kind: `(x_grid, y_grid, angle_index, open_tick)`. This uses the 4th value that Phase 37 D-08 reserved. `open_tick` counts half-seconds, so the valve opens at step `open_tick × 30`. For a valve, `open_tick` must be in `1..=step_limit / 30`. Non-valve kinds must send `0`, and any other value fails closed. Integer ticks keep snapping exact and outcomes reproducible. Canonical order sorts by `(kind, x, y, angle, open_tick)`.
- **D-03:** The valve opens before `world.step` runs for its open step index. Use the session's own step index, never wall-clock time or rendered frames. `MAX_ADVANCE_STEPS` stays 4. The gate lifecycle lives in puzzle code, either a new `puzzle/valve.rs` or `PuzzleHooks`. If the existing hooks only run after a step, add a pre-step puzzle hook, or open the valve after step `open_step - 1`, which is equivalent. Pin the chosen order in a unit test. Scene hook behavior must stay byte-for-byte unchanged.
- **D-04:** The Rust validator owns valve checks and adds a reason such as "Valve time out of range", reported through `inspectPuzzlePlacements`. `newPuzzle` fails closed on the same check, as in Phase 39 D-03. Valve overlap rules are the same as the plank's.
- **D-05:** A newly placed valve opens at 2.0 s (`open_tick = 4`) by default. While building, the player sets the time with a −/+ 0.5 s stepper in the existing selection bar, which is a fixed HUD slot. The stepper is always mounted. It is disabled unless a valve is selected while building, so the obstacle set never changes. The default keys are `[` and `]`, at Claude's discretion; they must not clash with the Phase 39 keys. The time edit is a new `reducePuzzle` event (for example `setValveTime`), and it is ignored outside building.
- **D-06:** The overlay draws each valve with its open time as a label, such as "2.0 s". While running, the valve counts down. Once the frame step reaches its open step, the valve is drawn as a faint open outline, so the drawing matches the destroyed body. This is derived in TS from the frame step and the placement, and no new frame scalar is needed.
- **D-07:** Fixtures gain an optional 5th field, `placement valve <x> <y> <angle> <open_tick>`. Plank lines stay 4 fields and mean `open_tick` 0. The existing `first-pour` fixtures must replay to the same `outcome`, `outcome_step`, `final_count`, `peak_held` and `brim_step`. If any value changes, stop and investigate. Both the Rust and Bun fixture parsers learn the new field.
- **D-08:** The same opening time must always give the same outcome. Prove this with a Rust test that builds the valve level twice with identical placements, and with the release-native vs wasm replay of the valve level's reference fixture.

#### Level set (LEVEL-01)

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

#### Solvability proof (LEVEL-02)

- **D-15:** Each level has recorded fixtures under `puzzle/fixtures/`: `<id>-solution.txt`, a reference solution that earns all three stars (Win, Brim, and placement count ≤ par), and `<id>-empty.txt`, a run with no tools that loses. "Wins with margin" means two things. The Win latches at least 60 steps (1 s) before `step_limit`. The fixture also meets the Phase 37 D-03/D-22 fill-margin fallback that the debug test lane asserts.
- **D-16:** The nudge sweep moves each reference placement ±1 and ±2 grid cells in x and in y, one placement and one axis at a time. Valves additionally get ±1 `open_tick`. Invalid nudged sets are skipped and counted separately. "Mostly still win" means at least 70% of the valid nudged variants still reach the Win outcome. The sweep runs only in the release lane (`cargo test -p liquidfun-wasm --release`, via `#[cfg_attr(debug_assertions, ignore)]` or an equivalent), because debug runs are slow. The default debug lane still runs each level's solution and empty fixtures. Keep all puzzle tests in the existing lib test binary (Phase 37 D-20).
- **D-17:** `just puzzle-replay-wasm` replays every level's fixtures through the generated wasm. Add one Playwright e2e that replays a reference solution in the browser and sees the win card. Use the valve level, because it exercises the stride-4 lane and the valve time UI, by setting the time through the stepper. This satisfies "at least one solution replays in the browser".
- **D-18:** Tuning is empirical and is the largest schedule risk. Use test-only probe helpers like those in Phase 37-06 to print outcomes, peak held and outcome steps while authoring. Do not ship probe output as evidence. Only the recorded fixtures and the assertions count.

#### First-level hint (LEVEL-03)

- **D-19:** Add a fixed-size **caption** HUD leaf that shows each level's one-line caption. It carries `data-hud-obstacle`, mounts with the puzzle route, and keeps a box size that is fixed per layout. On `first-pour`, until the hint is dismissed, the same leaf shows the placement hint instead, for example "Tap the plank, then tap the canvas to place it", with a Dismiss button of at least 44 px. Showing or hiding the hint never changes the obstacle set, so the camera never refits. This is required by Phase 38 D-18.
- **D-20:** Dismissal is permanent. It is stored under a versioned localStorage key, and every access is wrapped in try/catch. If storage is unavailable, the dismissal holds in memory for the page session. Placing the first tool on `first-pour` also counts as dismissal. Keep the storage helper small and separate so Phase 41's `progress.ts` can absorb or reuse it. Add Vitest coverage for a throwing getter, a throwing `setItem` and an already-dismissed state.
- **D-21:** Captions live in TS level metadata next to `PUZZLE_TITLES`, keyed by `PuzzleId`, so Phase 41's grid can show them before wasm loads. They are not cross-checked against Rust. Titles keep the D-26 cross-check.

#### Frames and budgets (LEVEL-04)

- **D-22:** Author new levels to suit portrait: roughly 7–9 m wide and 9–12 m tall, unlike `first-pour`'s 12 m width. Then each portrait rectangle in `puzzle-bounds.ts` fills a 390×844 phone well. Every level gets landscape and portrait rectangles that contain the cup, the walls, the build area and the liquid. `web/tests/puzzle-bounds.test.ts` covers every `PuzzleId` under the Phase 38 D-21 rule, using level geometry hardcoded from each Rust spec.
- **D-23:** The particle budget is at most **3,000 particles per level at step 0**, including jelly pad particles. `maximum_particle_count` is authored per level with headroom. A Rust unit test asserts the step-0 particle count against the budget for every level. Record a one-off release-native ms/step per level in the phase summary for information only; it does not gate the phase.

#### Gates

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

### Deferred Ideas (OUT OF SCOPE)

- Live valve tapping during a run (VALVE-02).
- A hint ladder per level (HINT-01).
- A sponge absorption mechanic (SPONGE-01).
- More levels (LEVEL-05).
- Moving or buoyant cups with a moving count region, which this phase avoids by D-12.

Also out of this phase (CONTEXT.md domain): the `#/puzzles` grid, the Puzzles nav entry, persisted progress, unlocks and last-placement restore (Phase 41), sound (Phase 42), and GPU water (Phase 43).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| TOOL-05 | Player can place a timed valve and set its opening time while building. | The pre-step hook already exists (`session.rs:216-217`), and `destroy_body` precedents exist (Tesla Valve, Hydraulic Fountain). This document gives the stride-4 file inventory, validator status 13, reducer, key, stepper and overlay plan, and the leak finding (Pitfall 1). |
| LEVEL-01 | 5–8 teaching levels with one-line captions; the jelly pad deflects and never counts. | Material mapping and measured tuning bands (viscous 1.0–2.0; elastic requires `SOLID`; float density 0.3–0.6). Group-based counting excludes pad groups (`goal.rs:51-67`, `builder.rs:126-133`). The powder finding (Open Question 1) affects one level. |
| LEVEL-02 | Every level proven solvable: a 3★ reference wins with margin, the empty run loses, nudges mostly win, and at least one browser replay. | Replay harness and contract (`replay.rs:150-261`), the record flag, a table-driven fixture test design, a release-only nudge sweep with a non-vacuity floor, measured run times, Bun replay, and an e2e valve replay plan. |
| LEVEL-03 | One-time dismissible placement hint on the first level. | A fixed caption leaf inside `PuzzleHud` and the desktop status row, and the guarded-storage provider pattern (`player/hud-sliders-preference.ts:22-46`). The Phase 39 "no web storage in puzzle modules" grep gate must be narrowed. |
| LEVEL-04 | Per-level portrait frame filling 390×844; particle budget; 25/25 fingerprints. | `puzzle-bounds.ts` and its test rule (`puzzle-bounds.test.ts:19-22, 104-134`), fill arithmetic for 7–9 m × 9–12 m levels, measured release step costs, a step-0 count test, and the scene-spot gate. |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- `crates/liquidfun` must not be modified. Production physics stays native Rust. Puzzle code lives in the private `liquidfun-wasm` bridge.
- Levels never enter `SCENE_IDS`, the scene sidebar, the README gallery plans (`web/scripts/readme-svg/plans.ts`), `PORTRAIT_VIEW_BOUNDS` or the scene survey. The README-gallery and portrait-bounds rules in AGENTS.md apply to scenes only, so they do not fire for levels.
- HUD rules: every new on-canvas chrome element carries `data-hud-obstacle` on its leaf, not on a full-width container, and keeps a box size that does not depend on camera scale. The `ScaleLegend` must stay visible.
- Rust style: no `unwrap()` in production code, errors propagate, `maybe_` prefixes for optional values, tau-based angles (the τ/24 table), `let...else`, and `foo.rs` + `foo/` module layout. Tests use Arrange/Act/Assert comments.
- TS style: `camelCase`, the `maybe` prefix for nullable values, early returns. Prefer pnpm only when there is no lock file; this repo uses **bun**.
- Pre-commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, plus `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` (the wasm crate is not a default member), build, and tests. Web: `bunx tsc --noEmit` and `bun run test:unit`. Also run `bun scripts/bright-builds-check.ts all`, which flags files over about 628 lines and functions over about 161 lines.
- `.planning/**` must never be formatted with mdformat. If non-GSD Markdown changes, run `just markdown-check`.
- Determinism: stable order, no `HashMap` iteration in solver-visible order, no wall-clock time in the puzzle path.
- Memory notes: run executors sequentially on the main tree (no worktrees). Each new macOS test binary costs about 2 minutes of launch scan, so keep puzzle tests in the lib binary. SUMMARY lifecycle frontmatter must be complete. Keep the process lightweight.

## Standard Stack

No new dependencies. Everything uses code the repo already has. [VERIFIED: codebase grep]

### Core

| Component | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `liquidfun` engine (public API, read-only) | workspace 0.0.0 | `World::destroy_body`, particle flags, `ParticleSystemDef` strengths, dynamic bodies | The engine is already used by all 25 scenes. The phase forbids engine edits. |
| `liquidfun-wasm` puzzle module | workspace | `LevelSpec`, builder, `PuzzleHooks`, validator, replay harness | Phase 37–39 infrastructure. Adding a level is mostly data. |
| wasm-bindgen | =0.2.128 (pinned) | `newPuzzle(levelId, kinds: &[u8], poses: &[i32])` and `inspectPuzzlePlacements` | The signature does not change. Only the stride does. |
| SolidJS + Kobalte + Tailwind (existing) | 1.9.15 / 0.13.12 | Caption leaf, stepper buttons | Existing shell. |
| Vitest / Playwright | 5.0.1 / 1.63.0 | Unit tests and e2e | Existing. |

### Supporting

| Tool | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| Rust | 1.97.0 | Development pin | All Rust work. [VERIFIED: `rustc --version`] |
| bun | 1.4.2 | Web scripts, Bun wasm replay | `just puzzle-replay-wasm`. [VERIFIED] |
| wasm-pack | 0.15.0 | `just web-wasm` | Required before the Bun replay and e2e. [VERIFIED] |
| just | 1.48.0 | Recipes | [VERIFIED] |

**Installation:** none.

## Architecture Patterns

### Recommended file layout (Rust)

```
crates/liquidfun-wasm/src/puzzle/
├── tools.rs              # + ToolKind::Valve = 2, POSE_STRIDE = 4, OpenTick, per-kind half extents
├── valve.rs              # (optional) pure valve schedule: open_step(), due(observed_step)
├── spec.rs               # + LiquidMaterial enum on LiquidBlock, + optional system tuning, + floats
├── builder.rs            # create_tools returns valve BodyIds; material → (ParticleFlags, GroupFlags)
├── hooks.rs              # on_advance opens due valves and drops their segments; collect_circles for floats
├── validate.rs           # + PlacementStatus::ValveTimeOutOfRange = 13 (never renumber)
├── first_pour.rs         # untouched except a `material: Water` field
├── thick_syrup.rs, sand_chute.rs, jelly_pad.rs, float_up.rs, timed_gate.rs, grand_finale.rs
├── levels_tests.rs       # #[cfg(test)] table-driven fixtures, 3-star, budget, nudge sweep, window sweep
└── fixtures/<id>-solution.txt, <id>-empty.txt (+ timed-gate-early.txt, timed-gate-late.txt)
```

### Pattern 1: Valve opens in the existing pre-step hook

**What:** The step order is fixed in `SessionCore::advance`: `on_advance` (`session.rs:216-217`), then `evict_escaped_particles` (218-226), then `world.step` (227-237), then `on_after_step` (238-245). `advance_profiled` uses the same order (252-291). `PuzzleHooks::on_after_step` increments `observed_step` (`hooks.rs:67-70`), so inside `on_advance` the step about to run is `observed_step + 1`. The valve opens when `observed_step + 1 == open_tick * 30`. The frame for step `S = open_tick*30` then has no gate, which matches D-06's TS rule "drawn open once frame step ≥ S". No new trait method or `session.rs` edit is needed, and scene hooks are untouched. [VERIFIED: session.rs, hooks.rs]

**Example:**

```rust
// Source: pattern derived from hooks.rs:47-83 and tesla_valve.rs:226-244 (HEAD c53d7a0bf)
fn on_advance(&mut self, world: &mut World, _system: ParticleSystemId) -> Result<(), SessionError> {
    let next_step = self.observed_step.checked_add(1).ok_or(SessionError::StepIndexExhausted)?;
    let mut opened_any = false;
    for valve in self.valves.iter_mut().filter(|valve| valve.is_due(next_step)) {
        let Some(body) = valve.maybe_body.take() else { continue };
        world.destroy_body(body).map_err(|_error| SessionError::StepFailed)?;
        opened_any = true;
    }
    if opened_any {
        self.segments = outline_segments(self.spec, &self.closed_placements());
    }
    Ok(())
}
```

Pin the order in a unit test: the frame at step `S-1` has the gate's 4 segments and the frame at step `S` does not, and a particle resting on the gate falls after step `S`.

### Pattern 2: Integer `open_tick` in `Placement`, derived `Ord`

Add `open_tick: u8` as the **last** field of `Placement` (`tools.rs:129-135`) so `#[derive(Ord)]` sorts by `(kind, x, y, angle, open_tick)`, as D-02 requires. `decode_pose` (`tools.rs:246-258`) destructures `[x, y, angle, open_tick]`. The range check needs `spec.step_limit`, so it belongs in `parse_one` (`tools.rs:262-280`), after the build-area check. `parse_one` currently receives only budget and build area, so pass `step_limit` or `&LevelSpec`. Valve: `1..=step_limit/30`. Every other kind: exactly `0`. Out-of-range values map to a new `PlacementError::ValveTimeOutOfRange`, and then to `PlacementStatus::ValveTimeOutOfRange = 13`. This is a **geometric** status, because the outline exists, so TS `isGeometricStatus` (`web/src/puzzle/placement-status.ts:56-62`) needs no change except `MAX_STATUS_CODE` and the `REASONS` entry. `Placement::vertices()` (`tools.rs:169-186`) must pick half extents by kind (valve 0.6 × 0.125 for 1.2 × 0.25 m).

### Pattern 3: Material on `LiquidBlock`, strengths on the system

`create_block` hard-codes `ParticleFlags::WATER` and uses the default group flags (`builder.rs:136-154`). Map a small enum:

| `LiquidMaterial` | Particle flags | Group flags | Probe evidence |
| --- | --- | --- | --- |
| `Water` | `WATER` (0) | none (keep exactly today's call chain) | first-pour unchanged |
| `Viscous` | `VISCOUS` | none | needs system `viscous_strength` 1.0–2.0 to read as "thick" |
| `Powder` | `POWDER` | none | behaves like water (Open Question 1) |
| `Elastic` | `ELASTIC` | `SOLID` (required) | without `SOLID`, water soaks into the pad |

`viscous_strength` and `elastic_strength` belong to the **system** (`ParticleSystemDef`, defaults at `crates/liquidfun/src/particle/definition/system_definition.rs:555-575`: viscous 0.25, elastic 0.25, powder 0.5). A level has exactly one system (`builder.rs:118-125`). Add an optional `LevelSpec` field such as `system_tuning: SystemTuning { maybe_viscous_strength: Option<f32>, maybe_elastic_strength: Option<f32> }`. `None` means "do not call the builder method", so `first-pour`'s `ParticleSystemDef` stays identical. Flags are per particle, so water and viscous groups can share a level.

### Pattern 4: Level-authored float as a circle body

Follow `float_or_sink.rs:184-216`: a `BodyType::Dynamic` body with one `CircleShape` at local `(0,0)` and a density of about 0.3–0.6 (the particle density is 1.0). Create it with the level geometry, before the tools: geometry → floats → tools → liquid. Then `first-pour` (no floats) is unchanged. `PuzzleHooks::collect_circles` (`hooks.rs:144-146`, empty today) returns `body_snapshot(body).position()` and the radius. **Use a circle:** its collision geometry and mass centre do not depend on body rotation. The engine's runtime `sin`/`cos`, which diverged native-vs-wasm in the Theo Jansen spike, therefore never feeds into contact geometry. The Phase 37 spike found dam-break (a dynamic circle) bit-identical between wasm and release-native for 300 steps (37-01-SUMMARY). The float's start area must be an obstacle for tools. The simplest way is an authored `no_place` `GridRect` covering its start position and path. That needs no new status code.

### Pattern 5: Table-driven per-level tests in the lib binary

Today `replay/tests.rs:369-377` has one test per `first-pour` fixture. Generalize it to a `const FIXTURES: [(&str, &str); N] = [("first-pour-solution.txt", include_str!(...)), ...]` with one `#[test]` per fixture file. Separate tests per file keep failures readable and let them run in parallel. Then add per-level tests:

- **3★:** the solution fixture has `brim_step` other than `none` and `placements.len() <= spec.par_tools`.
- **Margin:** `check_contract` (`replay.rs:223-261`) already requires the raised-target win by `step_limit - 120` (`MARGIN_WIN_SLACK_STEPS`, `goal.rs:87-88`). This implies D-15's "≥ 60 steps before the limit". Add an explicit `outcome_step + 60 <= step_limit` assertion for readability.
- **Budget:** `SessionCore::create_puzzle(level, Placements::empty())?.particle_count() <= 3000` and `<= spec.maximum_particle_count`, for every `LEVEL_IDS` entry.
- **Jelly never counts:** a goal-count unit test that places pad-group positions inside `cup_interior` with `counted_groups` lacking the pad group. Also run a session-level check where the pad's step-0 rect overlaps the cup region in a test-only spec, or check that the pad group is absent from the hooks' counted groups.
- **Nudge sweep (release only):** `#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]`.
- **Valve window sweep:** `open_tick` from 1 to 24. Release only, plus early and late loss fixtures that run in every lane.

### Anti-Patterns to Avoid

- **Adding a new `SceneHooks` method or touching `session.rs`.** `on_advance` already runs before the step. Changing the trait risks the 25/25 fingerprints for no gain.
- **Calling `destroy_body` from editing code or between runs.** Placements stay construction input, and the world is rebuilt at step 0 (Phase 37 D-10).
- **Making the float a rotating polygon.** Runtime trig enters contact geometry and risks native/wasm divergence (Pitfall 3 in the milestone research).
- **Putting captions in `web/src/catalog/puzzles.ts`.** Its array literal is parsed by the Rust `include_str!` test (`puzzle/tests.rs:84-94`). Put captions in `puzzle-levels.ts`.
- **Per-frame wasm calls for the valve countdown.** D-06 derives it in TS from `stepIndex` and the placement.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Removing the gate | Moving the body far away, disabling fixtures, a kinematic slide | `World::destroy_body` (engine `world/object/lifecycle.rs:81`) | Wakes touching bodies, ends contacts, and is already used mid-run by two scenes. |
| Pre-step scheduling | A new trait hook or a session field | `SceneHooks::on_advance` in `PuzzleHooks` | It already runs before eviction and step in both advance paths. |
| Buoyancy | A custom force or a fake rise | Dynamic circle with density < 1 among density-1 particles (`float_or_sink.rs:143-150`) | The engine couples particles to bodies. The probe shows a rise within 2 s. |
| Counting exclusion | Position or flag filters for the pad | The existing `counted_groups` (`builder.rs:126-133`, `goal.rs:51-67`) | Group ids are stable per particle. |
| Storage guard | Ad-hoc `localStorage` calls | The provider pattern from `player/hud-sliders-preference.ts:22-46` (`() => window.localStorage` inside the `try`) | A throwing getter is caught because the getter is called inside the try. |
| Fixture recording | Writing fixtures programmatically | `LIQUIDFUN_PUZZLE_RECORD=1 cargo test -p liquidfun-wasm --release --lib puzzle::replay -- --nocapture` (`replay.rs:230`, `replay/tests.rs:85-122`) | It prints `RECORD` lines from input lines only, and a human pastes them in. |
| Wasm parity | A new browser harness | `just puzzle-replay-wasm`, after adding the files to `FIXTURE_FILES` (`web/scripts/puzzle-replay/fixture.ts:73-76`) | The exact contract is already in place. |
| Per-level timing | Adding levels to `scene_spot` | A one-off `#[ignore]` release probe test using `std::time::Instant` around 720 `advance(1)` calls | The shared rules keep levels out of the scene survey. |

## Common Pitfalls

### Pitfall 1: A player-placed valve cannot seal: clearance slits leak (CRITICAL)

**What goes wrong:** The validator inflates every tool by one particle diameter. `first_overlap` uses `gap = 2.0 * spec.particle_radius` and requires `separated_by_more_than(quad, obstacle, gap)` (`validate.rs:258-293`). So a valve always leaves a slit wider than 0.08 m to the walls it is meant to seal. Particle centres are not kept a radius away from fixtures, so particles stream through such slits.
**Measured** (probe; reservoir of 1,404 particles at radius 0.04, 0.4 m floor hole, 1.2 × 0.25 m gate with its top 0.085 m below the hole) [VERIFIED: probe]:

| Liquid / variant | Leaked by 1 s / 2 s / 4 s / 6 s (gate closed) | 2 s after opening |
| --- | --- | --- |
| water | 94 / 221 / 454 / 659 (47%) | 1,146 |
| water + side flanges | 71 / 163 / 337 / 494 (35%) | 1,058 |
| water, 0.2 m hole + flanges | 50 / 124 / 271 / 411 | 714 |
| water, 0.1 m hole + flanges | 16 / 46 / 132 / 217 | 348 (the hole throttles the flow) |
| viscous, strength 1.0 | 0 / 1 / 23 / 47 (3%) | 525 |
| viscous, strength 1.5 | 0 / 0 / 12 / 29 | 412 |
| viscous, strength 2.0 | 0 / 0 / 5 / 18 | 340 |

**How to avoid:** Choose one of these before any valve-level geometry is authored (see Open Question 2): (a) hold a **viscous** reservoir (strength about 1.0) behind the valve, reusing the idea taught in level 2; (b) design the timing puzzle so the valve diverts or deflects a stream instead of sealing a reservoir; (c) accept a trickle and tune the window around it. D-04 locks "overlap rules same as the plank", so a valve-seat exemption would need owner sign-off.
**Warning signs:** The "too early" run and the reference run give similar fill. The `open_tick` sweep shows no losing band at high ticks.

### Pitfall 2: `POWDER` does not pile up in this engine

**What goes wrong:** D-09's working description ("grains pile up instead of levelling out") does not match measured behavior. LiquidFun particles have no inter-particle friction. The powder flag only changes repulsion.
**Measured** [VERIFIED: probe]:

| Test | Water | Powder |
| --- | --- | --- |
| 0.4 m outlet, drained at 2 s / 4 s / 8 s / 12 s (1,989 particles) | 971 / 1604 / 1929 / 1970 | 957 / 1582 / 1911 / 1963 |
| 1.2 × 2.4 m block on a flat floor, spread at 6 s / 12 s | 10.71 / 10.97 m | 10.70 / 11.02 m |
| 15° ramp, mean x at 2 s / 4 s | 0.40 / 4.83 | 0.28 / 4.88 |
| 0.12 m slit, drained at 12 s | 1,165 | 1,143 |

Raising `powder_strength` to 1.0 changed nothing. `POWDER | VISCOUS` behaved like powder. `damping` 4 produced runaway particles (the top particle reached 11.4 m).
**How to avoid:** Run a powder spike as the first powder-level task and resolve Open Question 1 before authoring geometry.

### Pitfall 3: Viscous strength above about 2.0 fails the step

**What goes wrong:** With radius 0.04, `with_viscous_strength(2.5)`, `(3.0)` and `(4.0)` made `world.step` return `StepError::ParticleLifecycleInvariant` within 720 steps. Values 1.0, 1.5 and 2.0 ran cleanly [VERIFIED: probe]. In the bridge a failed step becomes `SessionError::StepFailed`, and the shell shows its failure overlay.
**How to avoid:** Author viscous levels at 1.0–1.5, which leaves headroom. The default 0.25 is too weak to teach anything: it drained 1,881 vs 1,970 water through a 0.4 m gap.
**Warning signs:** Any probe or replay panicking with `advance(1) succeeds` failing.

### Pitfall 4: The jelly pad must be a `SOLID` group

**What goes wrong:** An `ELASTIC` group without `ParticleGroupFlags::SOLID` lets water soak in. 161 water particles sat inside the pad box after 12 s, against 3–16 with `SOLID` [VERIFIED: probe]. With `SOLID`, 192 pad particles stayed in a 0.3 m-deep static cradle. The centroid drifted by less than 0.11 m over 720 steps, both at elastic strength 0.25 and at 0.75, with and without `SPRING`.
**How to avoid:** `Elastic → (ELASTIC, SOLID)`. Seat the pad in authored static geometry (a cradle or notch). Static level walls have no clearance rule, so they can hold it snugly. The pad's spawn rectangle must not intersect walls.

### Pitfall 5: The stride change silently breaks dozens of literals and some "valve is rejected" tests

**Files that must change** [VERIFIED: grep]:

- Rust: `tools.rs:12-13` (doc and constant), `tools.rs:200, 229, 243-248, 260-268` (doc comments and `[x, y, angle]` destructuring), `validate.rs:14, 192`, `inspect.rs:45` (doc), `lib.rs:54-55` (doc), `first_pour.rs:82` (`REFERENCE_PLANK_POSE: [i32; 3]` becomes `[i32; 4]`), `replay/fixture.rs:44-51, 199-208` (arity), `replay/fixture.rs:252-264` (lanes), `session_tests.rs:254-275` (pose arrays), `api_tests.rs:182-186`, `builder/tests.rs`, `goal/tests.rs`, `tools/tests.rs` (12 literal arrays), and `validate/tests.rs` (30 literal arrays).
- Rust tests whose meaning flips: `tools/tests.rs:92-95` (stride-4 "valve time" currently rejected), `tools/tests.rs:108` (`parse(&[2], …)` expects an unknown kind), `tools/tests.rs:309` (`from_token("valve") == None`), `validate/tests.rs:370` (`&[2]` as unknown), and `replay/fixture/tests.rs:210` (`placement valve 1 2 3` rejected).
- TS: `web/src/puzzle/placements.ts:4` (codes), `:10-15` (type), `:17` (comment), `:23` (stride), `:38-47` (checks), `:56` (set), `:79-95` (seam parse). Also `web/src/puzzle/editor.ts:74` (the unchanged check must compare `openTick`), `:142` (`placeTool` default `openTick` by kind: valve 4, others 0), and `:283-286` (canonical sort tie-breaker). Also `web/src/puzzle/debug-attributes.ts:5-9`, `web/src/physics/puzzle-session.ts:128` (code map, which is automatic via `PUZZLE_TOOL_CODES`), `web/src/components/PuzzleToolTray.tsx:16` (an exhaustive switch needs a `valve` icon), and `web/src/puzzle/placement-status.ts:4-18, 32, 35-48`.
- TS tests: `web/tests/placement-status-parity.test.ts:15` (`EXPECTED_STATUS_COUNT` becomes 14), `web/tests/puzzle-level-info.test.ts:95` (uses code 2 as "unknown", so switch it to 3), and `web/tests/puzzle-replay.test.ts:255` (the valve line).
- Bun: `web/scripts/puzzle-replay/fixture.ts:16-21` (type), `:73-79` (`FIXTURE_FILES`, `TOOL_CODES`), `:285-…` (`fixtureLanes`).
- e2e: `web/e2e/puzzle-editor-helpers.ts:66` (`worldToPage` hard-codes `"first-pour"`), `:19-25` (first-pour constants), `web/e2e/puzzle-helpers.ts:8, 14-16, 144-…` (path and fixture).

**How to avoid:** Land the stride change as one wave with no level changes. Then prove `first-pour-solution.txt` replays **exactly** in release (`won 201 / 689 / peak 811 / brim 262`) and through `just puzzle-replay-wasm`.

### Pitfall 6: Fixture grammar ambiguity

**What goes wrong:** D-07 says plank lines stay 4 fields and valve lines have 5, and both parsers must accept exactly the same text.
**How to avoid:** Make the arity depend on the kind: `plank` needs exactly 5 tokens and `valve` needs exactly 6. A 6-token plank line is a grammar error, which is stricter than "`open_tick` must be 0" and keeps one place to fail. Mirror the same `fixture line N: expected K space-separated tokens` message in Rust (`fixture.rs:183-191`) and Bun. Keep the debug attribute consistent: `plank:x,y,angle` (unchanged, so the Phase 39 e2e literals stay valid) and `valve:x,y,angle,openTick`.

### Pitfall 7: "Next" on first-pour stops being disabled

**What goes wrong:** `web/e2e/puzzle.spec.ts:42, 90` asserts `"Next: No next level yet"` is disabled on first-pour. Once `PUZZLE_IDS` has a second level, `maybeNextPuzzleId` (`catalog/puzzle-levels.ts:31-33`) returns it and the result card renders a link.
**How to avoid:** Update that e2e to expect a Next link to `#/puzzle/<second-id>`. Keep the Vitest at `web/tests/puzzle-format.test.ts:287` (it tests the formatter with `undefined`). Add an e2e or unit test that the last level shows the disabled form.

### Pitfall 8: An empty run must stay far below target, not just lose

**What goes wrong:** `check_lost_margin` (`replay.rs:306-325`) requires the loss to latch exactly at `step_limit` **and** every per-step count to stay at or below `target − 5% of full`. A jelly-pad or powder geometry where the untouched stream partly lands in the cup fails the empty fixture even though the run loses.
**How to avoid:** Author every level so that, with no tools, the natural flow path misses the cup entirely. Check this with the probe before tuning the reference solution.

### Pitfall 9: A nudge sweep that passes vacuously

**What goes wrong:** If most nudged sets hit a wall or the water, they are skipped, and 70% of a tiny valid set proves nothing.
**How to avoid:** Assert a floor on valid variants, for example `valid >= 4` and `valid >= 50%` of generated variants. Report `wins/valid/skipped` in the assertion message. Generate variants per D-16: placement i, axis x or y, delta in {−2, −1, +1, +2}. Valves also get `open_tick` ±1, skipping out-of-range values.

### Pitfall 10: The caption leaf changes every first-pour camera frame

**What goes wrong:** A new `data-hud-obstacle` leaf shrinks the safe frame, so `data-pixels-per-meter` on first-pour changes from its Phase 39 value. The e2e tests compute page coordinates from the published camera (`puzzle-editor-helpers.ts:50-78`), so they keep working. Any hard-coded pixel expectation would not. The hint/caption swap must never mount or unmount the leaf.
**How to avoid:** Use one leaf with a fixed CSS box per layout, sized for the longer of hint and caption, for example two lines plus a 44 px button. Swap only its content. Add an e2e that reads `data-pixels-per-meter` before and after Dismiss and requires them to be equal.

### Pitfall 11: Phase 39 grep gates and protected tests

**What goes wrong:** The Phase 39 gate 21 asserted "puzzle modules use no web storage". The new hint helper uses `localStorage`.
**How to avoid:** Narrow the gate. The only storage access in the puzzle path is the new helper file, through the provider. Protected files that must stay unedited: `web/tests/{hash,portrait-bounds,readme-svg}.test.ts` and `web/e2e/hud-safe-frame.spec.ts`.

### Pitfall 12: Native vs wasm for new materials is not spiked

**What goes wrong:** The Phase 37 spike covered water, a dynamic circle and a revolute wheel. It did not cover elastic, viscous or powder solvers. If any of them diverges, `WASM_REPLAY_CONTRACT = "exact"` (`web/scripts/puzzle-replay/fixture.ts:92`) fails for that level.
**How to avoid:** Run `just puzzle-replay-wasm` as soon as the first non-water level has fixtures. These solvers use only sqrt and arithmetic, with no trig, so exactness is likely but unproven [ASSUMED]. If one diverges, a per-fixture fallback contract with the +5% margin is the documented Phase 37 D-03 fallback. Record it.

## Code Examples

### Material mapping (keeps first-pour identical)

```rust
// Source: builder.rs:136-154 pattern; flags from crates/liquidfun/src/particle/definition.rs:20-34
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiquidMaterial { Water, Viscous, Powder, Elastic }

impl LiquidMaterial {
    const fn flags(self) -> (ParticleFlags, ParticleGroupFlags) {
        match self {
            Self::Water => (ParticleFlags::WATER, ParticleGroupFlags::empty()),
            Self::Viscous => (ParticleFlags::VISCOUS, ParticleGroupFlags::empty()),
            Self::Powder => (ParticleFlags::POWDER, ParticleGroupFlags::empty()),
            Self::Elastic => (ParticleFlags::ELASTIC, ParticleGroupFlags::SOLID),
        }
    }
}
```

### Guarded dismissal storage (TS)

```ts
// Source: pattern from web/src/player/hud-sliders-preference.ts:22-46
export const FIRST_HINT_STORAGE_KEY = "liquidfun.puzzle-first-hint.v1";
type HintStorage = Pick<Storage, "getItem" | "setItem">;
export type HintStorageProvider = () => HintStorage;

export function loadHintDismissed(provider: HintStorageProvider): boolean {
  try {
    return provider().getItem(FIRST_HINT_STORAGE_KEY) === "dismissed";
  } catch {
    return false;
  }
}

export function persistHintDismissed(provider: HintStorageProvider): void {
  try {
    provider().setItem(FIRST_HINT_STORAGE_KEY, "dismissed");
  } catch {
    // The in-memory dismissal (a signal owned by the caller) holds for this page session.
  }
}
```

### Release-only sweep marker

```rust
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16); run with --release")]
fn timed_gate_nudges_mostly_still_win() { /* Arrange / Act / Assert */ }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Valve as a kinematic gate (v1.5 research ARCHITECTURE) | Static gate destroyed at the open step (CONTEXT D-01) | 2026-10-12 discuss-phase | No body motion. Destruction is a one-time event in `on_advance`. |
| Stride 3 with the 4th value rejected (`tools.rs:12`) | Stride 4 for every kind (D-02) | Phase 40 | All lanes, tests and fixtures change at once. |
| "Sponge" | "Jelly pad" elastic deflector, never counted | v1.5 owner decision | No absorption mechanic (SPONGE-01 deferred). |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | Phone wasm steps cost roughly 3–6× release-native Mac (so 3,000 particles ≈ 2.5–5 ms/step). | Summary, budgets | A level could miss 60 steps/s at 4× on phones. Outcomes stay correct (step-based), but feel suffers. Informational only per D-23. |
| A2 | Reusing viscous liquid in the valve level still counts as "introducing one idea" (timing). | Pitfall 1, Open Q2 | If the owner disagrees, the valve level needs a deflector design or a seat exemption. |
| A3 | Elastic, viscous and powder solvers are bit-identical between release-native and wasm. | Pitfall 12 | The Bun exact contract fails for those levels, and a fallback contract is needed. |
| A4 | A two-line fixed caption box fits both the hint and the captions at 390 px width without truncation. | Pitfall 10 | It may need ellipsis with a `title`, as `PuzzleTitle` does (`PuzzleHud.tsx:8-14`). |
| A5 | A circle float's contacts do not depend on rotation, so the engine's trig cannot cause native/wasm divergence. | Pattern 4 | This is reasoned from the circle's local centre at (0,0) and is backed by the dam-break spike. Friction-driven angular velocity still integrates, but does not move geometry. |

## Open Questions (RESOLVED)

1. **What does the powder level teach, given that `POWDER` behaves like water? (blocks `sand-chute` authoring)**
   - RESOLVED: see 40-CONTEXT.md D-25.
   - What we know: four probes show no piling, no different drainage and no different ramp flow (Pitfall 2). D-09 forbids dropping any level except buoyancy, and the lesson details are Claude's discretion.
   - What's unclear: whether any authored, honest powder behavior is visible enough to be "one idea".
   - Recommendation: make the first powder-level task a time-boxed spike for a distinguishable behavior. Candidates: a larger per-level particle radius (coarse grains, a system-wide setting, allowed per level), how powder interacts with a jelly or viscous neighbour, or splash and scatter off a plank. If nothing reads as distinct, escalate to the owner: either reword the caption honestly ("Sand pours like water here") or replace the idea within LEVEL-01's "drawn from" list. Do not ship a caption that claims piling.
2. **How does the valve level hold liquid despite the clearance leak? (blocks `timed-gate` and the finale)**
   - RESOLVED: see 40-CONTEXT.md D-26.
   - What we know: water leaks 35–47% in 6 s through the mandatory slit. Viscous liquid at 1.0 leaks 3% (Pitfall 1). D-04 locks plank overlap rules.
   - Recommendation: (a) a viscous reservoir (strength 1.0–1.5) held by the valve feeding the shared chute, with a water reservoir released freely, which fits D-13's two-reservoir idea. Prove the early/late/window bands with the sweep. Fall back to (b), a valve that diverts a stream for its closed period. Record the choice in the summary.
3. **Exact fixture arity rule for a 6-token plank line.**
   - RESOLVED: see 40-CONTEXT.md D-27.
   - Recommendation: reject it as a grammar error in both parsers (Pitfall 6).
4. **The buoyancy mechanism (D-12).**
   - RESOLVED: see 40-CONTEXT.md D-28.
   - What we know: a density-0.3 circle (r 0.3 m) rises from the floor to the surface within 120 steps, density 0.6 within 240, and density 2 sinks [VERIFIED: probe]. Particle impulses wake bodies (`particle_coupling/executor.rs:158` uses `WakePolicy::Wake`).
   - Recommendation: a float in a static well that rises as the player's planks route water into the well, until it closes a spill gap or lifts into a deflecting position. Level-authored static seats may fit the ball snugly, because only tools have the clearance rule. Use the D-09 drop rule if the tests cannot pass.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Rust toolchain | All Rust work | ✓ | 1.97.0 | — |
| bun | Web scripts, Bun replay, Vitest | ✓ | 1.4.2 | — |
| wasm-pack | `just web-wasm`, `puzzle-replay-wasm`, e2e | ✓ | 0.15.0 | — |
| just | Recipes | ✓ | 1.48.0 | — |
| Playwright chromium | Puzzle e2e | ✓ | @playwright/test 1.63.0, browsers cached | — |
| Python 3.13 | `just markdown-check` (only if non-GSD Markdown changes) | ✓ | 3.13.12 | — |

**Missing dependencies:** none.

## Validation Architecture

(`workflow.nyquist_validation` is `false` in `.planning/config.json`. This section is included because the orchestrator asked for it.)

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Rust built-in harness (lib binary of `liquidfun-wasm`), Vitest 5.0.1, Playwright 1.63.0, Bun replay script |
| Config | `web/vitest` defaults, `web/playwright.config.ts` (projects `chromium` and `touch`, lines 39-51) |
| Quick run | `cargo test -p liquidfun-wasm --lib puzzle::` (debug, about 10 s for two 720-step runs measured; budget about 7 s per level fixture in parallel) |
| Full suite | See the per-requirement commands below. |

### Measured run costs

| Run | Debug | Release |
| --- | --- | --- |
| One 720-step first-pour replay (about 2,000 particles) | about 7–8 s (`puzzle::replay`: 27 tests, 7.82 s) [VERIFIED] | 0.38 s [VERIFIED] |
| Step cost at about 2,000 / 3,000 particles | — | 0.50 / 0.79 ms/step (probe) [VERIFIED] |
| Nudge sweep: 7 levels × ≤ 26 variants | about 20 min (not acceptable) | about 1–1.5 min serial, about 15 s with one test per level |
| Valve window sweep (24 ticks) | about 3 min | about 10 s |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command |
| --- | --- | --- | --- |
| TOOL-05 | Stride-4 parse, valve range 1..=limit/30, non-valve tick 0, canonical order with `open_tick` | unit (Rust) | `cargo test -p liquidfun-wasm --lib puzzle::tools` |
| TOOL-05 | Status 13 from the inspector and `newPuzzle` parity | unit (Rust + TS) | `cargo test -p liquidfun-wasm --lib puzzle::validate` and `cd web && bunx vitest run tests/placement-status-parity.test.ts tests/puzzle-inspector.test.ts` |
| TOOL-05 | Gate present at frame S−1, absent at frame S; order pinned | unit (Rust) | `cargo test -p liquidfun-wasm --lib puzzle::hooks` (new tests) |
| TOOL-05 | Same time gives same outcome (two builds, same fingerprint) | unit (Rust) | `cargo test -p liquidfun-wasm --lib puzzle::levels_tests::timed_gate_is_reproducible` |
| TOOL-05 | `setValveTime` reducer, `[`/`]` keys, stepper enabled only with a selected valve, overlay label and open style | unit (TS) | `cd web && bunx vitest run tests/reduce-puzzle-editor.test.ts tests/puzzle-keyboard.test.ts tests/puzzle-tools-render.test.ts` |
| LEVEL-01 | 7 (or 6) levels registered; ids mirrored; titles; captions for every id | unit | `cargo test -p liquidfun-wasm --lib puzzle::tests` and `cd web && bun run test:unit` |
| LEVEL-01 | Jelly pad particles never count, even inside the cup | unit (Rust) | `cargo test -p liquidfun-wasm --lib puzzle::goal` plus the new jelly test |
| LEVEL-02 | Every solution and empty fixture (debug fallback contract) | integration (Rust) | `cargo test -p liquidfun-wasm --lib puzzle::replay` |
| LEVEL-02 | Every fixture exact in release; nudge sweep ≥ 70% of valid variants; window sweep | integration (Rust) | `cargo test -p liquidfun-wasm --release --lib puzzle::` |
| LEVEL-02 | Every fixture through generated wasm | integration (Bun) | `just puzzle-replay-wasm` |
| LEVEL-02 | Valve solution replays in the browser via the stepper | e2e | `just web-wasm && cd web && bun run build:app && CI=true bun run test:puzzle` (add the new spec to the `test:puzzle` list, `web/package.json:18`) |
| LEVEL-03 | Hint storage: throwing getter, throwing setItem, already dismissed | unit (TS) | `cd web && bunx vitest run tests/puzzle-first-hint.test.ts` |
| LEVEL-03 | Dismiss persists across reload; camera scale unchanged; placing the first tool dismisses | e2e | in `CI=true bun run test:puzzle` |
| LEVEL-04 | Portrait/landscape frames contain the geometry and fill the phone for every `PuzzleId` | unit (TS) | `cd web && bunx vitest run tests/puzzle-bounds.test.ts` |
| LEVEL-04 | Step-0 count ≤ 3000 and ≤ `maximum_particle_count` for every level | unit (Rust) | `cargo test -p liquidfun-wasm --lib puzzle::levels_tests::step_zero_particles_fit_the_budget` |
| LEVEL-04 / gates | 25/25 fingerprints unchanged | integration | `just playground-scene-spot`, then diff with `target/phase40/base.txt` (record at phase base) |
| Gates | Engine untouched | static | `git diff --stat <phase-base> -- crates/liquidfun` is empty |
| Gates | fmt, clippy, build, tests, typecheck, Bright Builds | static/unit | `cargo fmt --all --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`; `cargo test -p liquidfun-wasm --all-features`; `cd web && bunx tsc --noEmit && bun run test:unit`; `bun scripts/bright-builds-check.ts all`; `just web-player-smoke` |

### Sampling Rate

- **Per task commit:** `cargo test -p liquidfun-wasm --lib puzzle::<module>` and the touched Vitest files.
- **Per wave merge:** the debug and release `puzzle::` lanes, plus `just puzzle-replay-wasm` once fixtures exist.
- **Phase gate:** the full list above, including e2e and scene-spot.

### Wave 0 Gaps

- [ ] Record the `target/phase40/base.txt` scene-spot baseline at the phase base, before any bridge edit.
- [ ] Rust test-only probe helper (`#[ignore]`) that prints outcome, outcome step, peak held and the count curve for `(level, placements)`, plus an `Instant`-based ms/step line (D-18, D-23).
- [ ] Table-driven fixture test scaffold (Pattern 5) and `levels_tests.rs`.
- [ ] `web/tests/puzzle-first-hint.test.ts` (new).

## Security Domain

`security_enforcement` is absent from the config, so it is treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Rust fail-closed lane parsing (`check_lanes` before allocation, `tools.rs:225-233`), the new `open_tick` range check, the strict fixture grammar, TS `checkPlacement` integer and i32 checks (`placements.ts:38-47`), and the strict hint storage token (`=== "dismissed"`) |
| V6 Cryptography | no | — |
| V14 Configuration | partial | The test seam (`__liquidfunPuzzleTestPlacements`) stays readable only through Playwright `addInitScript`, never from the URL |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Crafted lanes (huge `open_tick`, non-zero tick on a plank, wrong stride) | Tampering / DoS | `MAX_PLACEMENTS` 16 and the stride check before allocation. Range-check `open_tick` and fail closed. Messages never echo inputs (`tools.rs:295-347`). |
| Storage blocked or poisoned | DoS | try/catch around the provider. Accept only the exact token. Fall back to memory. |
| HTML injection through captions or labels | Tampering | Captions are static TS constants rendered as text. Keep the Phase 39 "no innerHTML" grep gate. |
| Wall-clock dependence (reproducibility "cheat") | Tampering | The valve uses `observed_step` only. Keep the grep gate for `Date.now`, `performance.now` and `setTimeout` in the puzzle path. |

## Recommended Plan Breakdown (waves)

This order isolates tuning risk. Executors run sequentially on the main tree, so "parallel" means the plans are independent, not that they run concurrently.

1. **Wave 1, Plan 01: Baseline and spikes.** Record the scene-spot baseline. Add the in-repo `#[ignore]` probe helper. Run the **material and valve spikes** in-repo to settle Open Questions 1, 2 and 4 with numbers, and record them in the plan summary. No production change.
1. **Wave 2, Plan 02: Rust valve and stride 4.** Add `ToolKind::Valve`, `open_tick`, status 13, the validator, `PuzzleHooks::on_advance` opening, segment removal, the Rust fixture grammar and every Rust test literal. **Gate:** first-pour fixtures exact in release, debug contract passes, 25/25 fingerprints.
1. **Wave 2, Plan 03: TS lanes and Bun grammar.** Stride-4 encoder, the seam, the `valve` code, status 13 and its reason, the debug attribute, editor sort and equality, the Bun fixture parser and lanes, and TS tests (kind code 2). **Gate:** `just puzzle-replay-wasm` is still exact for first-pour, and typecheck plus Vitest pass.
1. **Wave 3, Plan 04: Level infrastructure.** `LiquidMaterial`, `SystemTuning`, float bodies (circle) with `collect_circles`, the table-driven fixture tests, the 3★/margin/budget tests, the nudge-sweep and window-sweep harnesses, TS `PUZZLE_CAPTIONS` next to `PUZZLE_TITLES`, and generalized e2e helpers (`worldToPage(levelId, …)`). first-pour stays unchanged.
1. **Wave 4, Plans 05–07: Levels in batches.** Each plan lands complete levels: spec file, fixtures, Rust and TS registry, title, caption, both bounds, and `LEVEL_GEOMETRY` in `puzzle-bounds.test.ts`. Registry entries must land together because the `include_str!` test and the TS `Record<PuzzleId, …>` types fail otherwise. Suggested batches: (05) `thick-syrup` + `jelly-pad`, the lowest risk; (06) `timed-gate`, with its window sweep and early/late fixtures, plus `sand-chute` per the spike outcome; (07) `float-up` (may be dropped under D-09) + `grand-finale`. Run `just puzzle-replay-wasm` after each batch.
1. **Wave 3–4 (independent of levels), Plan 08: Valve UI.** `setValveTime` event, `[`/`]` keys, the always-mounted stepper in `PuzzleSelectionBar` with fixed CSS widths per layout, the tray valve icon, overlay label, countdown and faint-open style (a pure `valveOverlay(state)` function with Vitest), and announcements. Test with a fake two-kind budget.
1. **Plan 09: Caption and hint leaf plus storage helper.** Desktop status-row variant, `first-pour` hint swap, dismissal on first place, and Next behavior.
1. **Plan 10: e2e.** Valve browser replay via stepper (exact fixture values), hint dismissal with reload and an unchanged camera scale, the updated first-pour Next assertion, and a last-level disabled Next.
1. **Plan 11: Phase gates (D-24) and the summary.** Record per-level ms/step, any buoyancy drop, the powder and valve design decisions, and the brim/par values.

A UI phase (`ui_phase: true`) may produce `40-UI-SPEC.md` for the stepper, caption box and overlay label. The planner should use it for exact sizes.

## Sources

### Primary (HIGH confidence)

- Codebase at HEAD `c53d7a0bf`: `crates/liquidfun-wasm/src/{session.rs, scene.rs, puzzle.rs, puzzle/*.rs, scene/{tesla_valve,hydraulic_fountain,float_or_sink,jelly_drop,elastic_particles,liquid_timer,sparky}.rs, session/escape.rs}`; `crates/liquidfun/src/world/{object/lifecycle.rs, contact_manager.rs, particle_coupling.rs}`, `particle/definition*.rs`; `web/src/{puzzle,catalog,components,render,player}/…`, `web/scripts/puzzle-replay/fixture.ts`, `web/e2e/puzzle*.ts`, `web/tests/puzzle-bounds.test.ts`, `web/tests/placement-status-parity.test.ts`.
- Measured: `cargo test -p liquidfun-wasm --lib puzzle::replay` (debug 7.82 s) and `--release` (0.38 s for one fixture).
- Out-of-repo probe (`scratchpad/probe`, path dependency on `crates/liquidfun`, release, rustc 1.97.0): pour, pile, ramp, sieve, jelly, float and leak scenarios. The numbers are quoted in the Pitfalls. The probe is not repo evidence and is not shipped.
- `.planning/phases/37-puzzle-core-in-the-bridge/37-01-SUMMARY.md` (native-vs-wasm spike); `.planning/phases/39-place-then-play-tools-and-stars/39-14-SUMMARY.md` and `39-VERIFICATION.md` (gate commands).

### Secondary (MEDIUM confidence)

- `.planning/research/v1.5-puzzle-mode/SUMMARY.md` and `PITFALLS.md` (Pitfalls 3, 6, 14).

### Tertiary (LOW confidence)

- Phone wasm step-cost multiplier (A1). Not measured in this session.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH. No new dependencies; versions verified locally.
- Architecture (valve hook, stride, materials, floats): HIGH. Read from source with line citations and backed by precedents and probes.
- Level tuning: MEDIUM. Material bands are measured, but every level's geometry is still empirical. Powder is LOW until Open Question 1 is resolved.
- Pitfalls: HIGH. Each is code-grounded or measured.

**Research date:** 2026-10-12
**Valid until:** 2026-11-11 (stable codebase; re-check line numbers if Phase 40 plans are created after other bridge edits)
