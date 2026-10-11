---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: 2026-10-11T21:50:55.159Z
---

# Phase 37: Puzzle Core in the Bridge - Context

**Gathered:** 2026-10-11
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Add a `puzzle/` module to the private `crates/liquidfun-wasm` bridge. It builds a level from a level id and a placement list (empty allowed), steps it with the existing session loop, and decides cup fill, win and timeout per fixed step in Rust. The result must be the same natively and in wasm. Deliverables:

- One plain-water level.
- The `SessionSource` refactor.
- A `newPuzzle` constructor and `levelInfo()` on `ProofSession`.
- Frame puzzle scalars, with TS parsing.
- The pure goal counter and outcome latch.
- A reference-solution replay harness.
- The native-vs-wasm determinism spike, run first, with its result recorded.

Out of scope here: the puzzle route, HUD and player shell (Phase 38); the tool editor, full placement validator and stars (Phase 39); the valve and the other teaching levels (Phase 40). `crates/liquidfun` is not modified.

</domain>

<decisions>
## Implementation Decisions

### Determinism spike (first task)
- **D-01:** Plan 1 Task 1 is the spike. Build the wasm package (`bun scripts/web-build.ts wasm`). A Bun script loads `web/src/generated/liquidfun-wasm` the same way `web/scripts/readme-svg.ts` does, steps a small set of existing scenes for a fixed number of steps, and computes an FNV-1a 64 hash over the captured frame's f32 bits (particle positions, then rigid segments, then circles). A native Rust unit test computes the same hash for the same scenes and steps through `SessionCore::capture_frame`. The scene set must include at least one scene with rotating or motorized dynamic bodies (for example `theo-jansen` or `water-wheel`) plus `dam-break`, about 300 steps each.
- **D-02:** Record the spike result in the Plan 1 SUMMARY and the phase SUMMARY: bit-identical yes or no, per scene, with the first diverging step if any. Do not change engine trig or anything else in `crates/liquidfun` because of the result.
- **D-03:** Policy from the spike:
  - **If identical:** the replay contract is an exact outcome and outcome step, natively and in wasm.
  - **If not identical:** the contract becomes the same outcome with the solution winning by a fill margin of at least +5% of `full_count` over target, plus an outcome-step tolerance that the spike data sets and the SUMMARY records.
  - **Angle snapping either way:** static tool rotations come from a literal τ/24 `(cos, sin)` table baked into the polygon vertices, with body angle 0, so static tools never call `sin`/`cos`.

### Level registry and spec
- **D-04:** Levels use their own Rust allowlist: `LevelId` plus `parse_level_id` in `crates/liquidfun-wasm/src/puzzle.rs`, never `SceneId`/`parse_scene_id`. Add a unit test showing that no level id parses as a scene id and no scene id parses as a level id.
- **D-05:** Add a minimal TS mirror `PUZZLE_IDS` in a new `web/src/catalog/puzzles.ts`, separate from `SCENE_IDS` in `web/src/catalog/scenes.ts`. A Rust `include_str!` test checks that `PUZZLE_IDS` lists exactly the Rust allowlist. Do not add levels to the scene sidebar, README plans, portrait bounds or the scene survey. Phase 38 owns routing and bounds.
- **D-06:** The first level is `first-pour` ("First Pour"), plain water only. A declarative `const` `LevelSpec` holds:
  - walls (polygons)
  - liquid blocks (rectangle, spacing, color, counted flag)
  - cup walls and the cup interior rectangle
  - `target_count`, `full_count`, `settle_steps` (30) and `step_limit` (authored, about 600 to 900 steps)
  - the tool budget (kinds and counts)
  - a build-area rectangle

  One generic builder turns a `LevelSpec` plus placements into the existing `BuiltScene`. The particle budget is about 1.5k to 3k particles.
- **D-07:** Lay out `first-pour` so that zero placements loses (the water lands beside the cup) and one plank placement in the reference solution wins with margin. This gives the harness a meaningful win fixture and a meaningful loss fixture.

### Placements (minimal, construction input)
- **D-08:** Placements cross wasm as copied slices: `kinds: &[u8]`, plus `poses: &[i32]` with stride 3 `(x_grid, y_grid, angle_index)`. The grid is 0.05 m and `angle_index` is in `0..24` (τ/24 steps). Integers make the canonical snapping exact. A 4th stride value (valve time) is reserved for Phase 40 but not accepted now.
- **D-09:** This phase adds exactly one tool kind: a fixed-size static plank (a polygon at least 3 particle diameters thick, not an edge). Placements are created in canonical order: sort by `(kind, x, y, angle)` and create after the level geometry and before the liquid. Minimal fail-closed checks:
  - the kind is in the level's allowlist
  - the per-kind budget is respected
  - the slice lengths are consistent
  - `angle_index < 24`
  - the position is inside the build area

  Each failure returns a typed `SessionError`. Overlap checks, no-place zones and validator parity are Phase 39.
- **D-10:** Play and Reset always rebuild from `(level, placements)` at step 0. The bridge never edits a live puzzle world.

### Goal counting and outcome latch (pure core)
- **D-11:** Put the pure code in `puzzle/goal.rs`:
  - `count_in_goal(positions, counted_mask, interior) -> u32` counts a particle only if it is counted and its centre is strictly inside the interior rectangle. A particle on the boundary or rim line is outside.
  - Counted means the particle belongs to a group the level marks as counted, checked through the public `ParticleSystemView::group_ids()`. The planner may choose the cheapest correct read-only mapping. Non-counted groups (future jelly pad) are excluded.
- **D-12:** Make `GoalTracker::observe(count, step) -> Outcome` pure and step-latched:
  - `Running` while `count >= target` has not yet held for `settle_steps` consecutive steps.
  - `Won { step }` when the hold completes at a step `<= step_limit`.
  - `Lost { step: step_limit }` when `step_limit` passes without a win.
  - Once latched, the outcome never changes. A later slosh does not undo a win.
  - A splash that is above target for fewer than `settle_steps` steps does not win.
- **D-13:** Fill is `count / full_count`, clamped to 100%. `full_count` is authored and fixed, and eviction does not change it.
- **D-14:** The tracker runs in `PuzzleHooks::on_after_step`, after `world.step` and the eviction that `SessionCore::advance` already performs. It uses the session step index. Nothing in the puzzle path reads wall-clock time or rendered frames, and `MAX_ADVANCE_STEPS` stays 4. The session keeps stepping after the latch, so the HUD can let the water settle.

### Session and frame contract
- **D-15:** `SessionCore` replaces `scene_id: SceneId` with `source: SessionSource { Scene(SceneId), Puzzle(LevelId, Placements) }`. Scene paths (particle iterations, escape bounds, gravity-preset recreation) behave byte-for-byte as before. Puzzles use the default particle iterations and the shared escape bounds unless the level spec says otherwise. Scene controls, gravity presets and pointer input are rejected for puzzles with `UnknownControl`.
- **D-16:** `ProofSession` gains a static constructor `newPuzzle(levelId, kinds, poses)` and `levelInfo()`. `levelInfo()` returns:
  - level id, title, `target_count`, `full_count`, `settle_steps`, `step_limit`
  - the cup interior rectangle
  - the build area
  - the tool budget as flat typed arrays

  It is only valid on puzzle sessions; on a scene session it throws a typed error.
- **D-17:** `FrameData` gains optional puzzle scalars, `goalCount`, `goalTarget`, `goalFull`, `outcome` (0 running, 1 won, 2 lost), `outcomeStep` (`undefined` while running) and `stepLimit`. They are absent (undefined) for scene sessions, so existing scene frames are unchanged. `web/src/physics/frame.ts` parses them into an optional `maybePuzzle` field, with Vitest coverage, and existing fakes keep working.

### Reference-solution harness and replay
- **D-18:** Use a plain-text, line-based fixture with no serde, shared by Rust and TS: `level`, a `placement <kind> <x> <y> <angle>` line per placement, `outcome`, `outcome_step` and `final_count`. Store it under `crates/liquidfun-wasm/src/puzzle/fixtures/`. Record `first-pour-solution.txt` (won) and `first-pour-empty.txt` (lost).
- **D-19:** A native unit test replays each fixture through `SessionCore` and asserts the outcome and outcome step (and the margin under the D-03 fallback). A Bun script, exposed through a `just` recipe such as `just puzzle-replay-wasm`, replays the same fixtures through the generated wasm `newPuzzle` and asserts the same contract. Add a fixture re-record path, behind an explicit flag, that prints new values instead of overwriting silently.
- **D-20:** Keep puzzle tests as unit tests inside the existing `liquidfun-wasm` lib test binary (`src/puzzle/tests.rs` or similar). Do not add a new integration-test binary: each new macOS binary costs about 2 minutes of launch scan.

### Gates
- **D-21:** Before finishing, check:
  - `just playground-scene-spot` reports 25/25 end-state fingerprints identical to the pre-phase baseline. Record the baseline at the phase start from HEAD before the bridge changes.
  - `git diff --stat <phase-base> -- crates/liquidfun` is empty.
  - cargo fmt, clippy (`-D warnings`), build and tests all pass, plus the web Vitest tests the change touches and `bun scripts/bright-builds-check.ts all` (file-length limits).

### Claude's Discretion
- The exact `first-pour` geometry, particle count, step limit, target and full counts, chosen within D-06 and D-07 and tuned so that the solution wins with margin.
- Module split inside `puzzle/` (`spec.rs`, `level_first_pour.rs`, `tools.rs`, `goal.rs`, `hooks.rs`), within the file-length limits.
- How the counted mask is derived from group ids. It must be read-only and must not change any scene's behavior.
- The exact hash and serialization helper shared by the spike and replay scripts.
- Whether `levelInfo()` returns a small `#[wasm_bindgen]` struct or flat getters.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope and acceptance
- `.planning/ROADMAP.md` (Phase 37 and Shared acceptance rules): success criteria, the 25/25 fingerprint gate, the separate level registry and per-step decisions.
- `.planning/REQUIREMENTS.md`: GOAL-02 (settle window win) and GOAL-04 (deterministic replay natively and in the browser).
- `PROJECT-SCOPE.md`: hobby-project scope and the optional strict-qualification profile.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/testing.md`, `standards/languages/rust.md`: repo and Bright Builds rules (Arrange/Act/Assert, no `unwrap`, `maybe_` naming, τ).

### v1.5 research
- `.planning/research/v1.5-puzzle-mode/SUMMARY.md`: synthesis, open-decision table and Phase 37 deliverables.
- `.planning/research/v1.5-puzzle-mode/ARCHITECTURE.md`: `puzzle/` module, `SessionSource`, `newPuzzle`, frame scalars and `PuzzleHooks`.
- `.planning/research/v1.5-puzzle-mode/PITFALLS.md`: the native-vs-wasm divergence (Pitfall 3), counting edge cases (Pitfall 5) and creation order (Pitfall 2).
- `.planning/research/v1.5-puzzle-mode/STACK.md`: no new dependencies and the copied-slice transport.

### Bridge code
- `crates/liquidfun-wasm/src/scene.rs`: `BuiltScene`, `SceneHooks`, `build_scene` and `parse_scene_id`.
- `crates/liquidfun-wasm/src/session.rs`: `SessionCore`, `advance` (the eviction → step → `on_after_step` order), `capture_frame` and `MAX_ADVANCE_STEPS`.
- `crates/liquidfun-wasm/src/session/escape.rs`: per-scene escape bounds keyed by `SceneId`.
- `crates/liquidfun-wasm/src/frame.rs` and `crates/liquidfun-wasm/src/lib.rs`: `FrameData` and the `ProofSession`/`ProofFrame` wasm surface.
- `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs`: the pattern for a scene with `on_after_step` logic.
- `crates/liquidfun/src/particle/view.rs`: `group_ids()`, `positions()` and `flags()`, all read-only.
- `crates/liquidfun/src/math/transform.rs`: `Rot::from_angle` is the only constructor and uses trig, which is why D-03 bakes vertices instead.

### Web and tooling
- `web/src/physics/frame.ts`, `web/src/physics/session.ts`, `web/tests/session.test.ts`: TS frame parsing and fakes.
- `web/src/catalog/scenes.ts`: `SCENE_IDS`, which must not gain levels.
- `web/scripts/readme-svg.ts` (around lines 150 to 170): how a Bun script loads the generated wasm package.
- `scripts/web-build.ts`, `justfile` (`web-wasm`, `playground-scene-spot`): build and fingerprint commands.
- `tools/xtask/src/playground/spot.rs`: the scene-spot fingerprint tool used for the 25/25 gate.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `BuiltScene` plus `SceneHooks`: the puzzle builder returns a `BuiltScene` with `PuzzleHooks`, so the existing stepping, eviction and frame capture are reused unchanged.
- `attach_basin_fixture` and the basin helpers in `scene.rs` and `scene/basin_family.rs`: static polygon walls for the level and the cup.
- `SessionCore::capture_frame`: already copies positions, segments and circles. Puzzle scalars are added next to these.
- The FNV-1a 64 survey fingerprint from Phase 35 in `scene_spot`: a model for the spike and replay hash.
- The generated-wasm loading in `web/scripts/readme-svg.ts`: the template for the Bun replay script.

### Established Patterns
- Allowlisted string ids parsed into enums that fail closed with typed `SessionError` messages.
- Bounded copied lanes across wasm, with stride constants and checked multiplication.
- Unit tests in Arrange/Act/Assert form, kept inside the lib test binary.
- File-length limits enforced by `bun scripts/bright-builds-check.ts all`. Keep each puzzle file small.

### Integration Points
- `SessionCore` construction (`from_built`) and `advance`: replace `scene_id` with `SessionSource`.
- `escape::particle_has_escaped(scene: SceneId, …)`: needs a puzzle arm or a source-aware signature.
- `lib.rs` `ProofSession`: add `newPuzzle` and `levelInfo`. `ProofFrame` gains optional getters.
- `web/src/physics/frame.ts`: add optional puzzle fields to the parse.

</code_context>

<specifics>
## Specific Ideas

- Win rule wording: "fill stays at or above target for the authored settle window (about 30 steps, 0.5 s) before the step limit". A splash through the cup never wins, and a latched win survives a later slosh.
- Placement poses are stored as integers (0.05 m grid, τ/24 angle index), so canonical order and snapping are exact and serializable for the later SEED-002 share links.
- The spike doubles as the start of the dual-path replay tool. The same hash and fixture format serve both.

</specifics>

<deferred>
## Deferred Ideas

- The full placement validator (overlap with walls, cup, water and tools; no-place zones; TS parity): Phase 39.
- Timed valve (4th pose value): Phase 40.
- Playwright replay of a solution in a real browser: Phase 40 (Phase 37 replays wasm through Bun).
- Puzzle route, HUD, portrait bounds registry: Phase 38.
- Moving or buoyant cup region transforms and convex polygon cup interiors: Phase 40, if a level needs them.

</deferred>

---

*Phase: 37-puzzle-core-in-the-bridge*
*Context gathered: 2026-10-11*
