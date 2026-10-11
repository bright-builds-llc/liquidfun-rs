# Phase 37: Puzzle Core in the Bridge - Research

**Researched:** 2026-10-11 (HEAD `207114dfa`)
**Domain:** Private Rust/WASM bridge (`crates/liquidfun-wasm`) refactor + pure goal core + Bun/wasm replay tooling
**Confidence:** HIGH (code-grounded; wasm-bindgen surface and native-vs-wasm behavior verified empirically in this session)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Determinism spike (first task)
- **D-01:** Plan 1 Task 1 is the spike. Build the wasm package (`bun scripts/web-build.ts wasm`). A Bun script loads `web/src/generated/liquidfun-wasm` the same way `web/scripts/readme-svg.ts` does, steps a small set of existing scenes for a fixed number of steps, and computes an FNV-1a 64 hash over the captured frame's f32 bits (particle positions, then rigid segments, then circles). A native Rust unit test computes the same hash for the same scenes and steps through `SessionCore::capture_frame`. The scene set must include at least one scene with rotating or motorized dynamic bodies (for example `theo-jansen` or `water-wheel`) plus `dam-break`, about 300 steps each.
- **D-02:** Record the spike result in the Plan 1 SUMMARY and the phase SUMMARY: bit-identical yes or no, per scene, with the first diverging step if any. Do not change engine trig or anything else in `crates/liquidfun` because of the result.
- **D-03:** Policy from the spike:
  - **If identical:** the replay contract is an exact outcome and outcome step, natively and in wasm.
  - **If not identical:** the contract becomes the same outcome with the solution winning by a fill margin of at least +5% of `full_count` over target, plus an outcome-step tolerance that the spike data sets and the SUMMARY records.
  - **Angle snapping either way:** static tool rotations come from a literal τ/24 `(cos, sin)` table baked into the polygon vertices, with body angle 0, so static tools never call `sin`/`cos`.

#### Level registry and spec
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

#### Placements (minimal, construction input)
- **D-08:** Placements cross wasm as copied slices: `kinds: &[u8]`, plus `poses: &[i32]` with stride 3 `(x_grid, y_grid, angle_index)`. The grid is 0.05 m and `angle_index` is in `0..24` (τ/24 steps). Integers make the canonical snapping exact. A 4th stride value (valve time) is reserved for Phase 40 but not accepted now.
- **D-09:** This phase adds exactly one tool kind: a fixed-size static plank (a polygon at least 3 particle diameters thick, not an edge). Placements are created in canonical order: sort by `(kind, x, y, angle)` and create after the level geometry and before the liquid. Minimal fail-closed checks:
  - the kind is in the level's allowlist
  - the per-kind budget is respected
  - the slice lengths are consistent
  - `angle_index < 24`
  - the position is inside the build area

  Each failure returns a typed `SessionError`. Overlap checks, no-place zones and validator parity are Phase 39.
- **D-10:** Play and Reset always rebuild from `(level, placements)` at step 0. The bridge never edits a live puzzle world.

#### Goal counting and outcome latch (pure core)
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

#### Session and frame contract
- **D-15:** `SessionCore` replaces `scene_id: SceneId` with `source: SessionSource { Scene(SceneId), Puzzle(LevelId, Placements) }`. Scene paths (particle iterations, escape bounds, gravity-preset recreation) behave byte-for-byte as before. Puzzles use the default particle iterations and the shared escape bounds unless the level spec says otherwise. Scene controls, gravity presets and pointer input are rejected for puzzles with `UnknownControl`.
- **D-16:** `ProofSession` gains a static constructor `newPuzzle(levelId, kinds, poses)` and `levelInfo()`. `levelInfo()` returns:
  - level id, title, `target_count`, `full_count`, `settle_steps`, `step_limit`
  - the cup interior rectangle
  - the build area
  - the tool budget as flat typed arrays

  It is only valid on puzzle sessions; on a scene session it throws a typed error.
- **D-17:** `FrameData` gains optional puzzle scalars, `goalCount`, `goalTarget`, `goalFull`, `outcome` (0 running, 1 won, 2 lost), `outcomeStep` (`undefined` while running) and `stepLimit`. They are absent (undefined) for scene sessions, so existing scene frames are unchanged. `web/src/physics/frame.ts` parses them into an optional `maybePuzzle` field, with Vitest coverage, and existing fakes keep working.

#### Reference-solution harness and replay
- **D-18:** Use a plain-text, line-based fixture with no serde, shared by Rust and TS: `level`, a `placement <kind> <x> <y> <angle>` line per placement, `outcome`, `outcome_step` and `final_count`. Store it under `crates/liquidfun-wasm/src/puzzle/fixtures/`. Record `first-pour-solution.txt` (won) and `first-pour-empty.txt` (lost).
- **D-19:** A native unit test replays each fixture through `SessionCore` and asserts the outcome and outcome step (and the margin under the D-03 fallback). A Bun script, exposed through a `just` recipe such as `just puzzle-replay-wasm`, replays the same fixtures through the generated wasm `newPuzzle` and asserts the same contract. Add a fixture re-record path, behind an explicit flag, that prints new values instead of overwriting silently.
- **D-20:** Keep puzzle tests as unit tests inside the existing `liquidfun-wasm` lib test binary (`src/puzzle/tests.rs` or similar). Do not add a new integration-test binary: each new macOS binary costs about 2 minutes of launch scan.

#### Gates
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

### Deferred Ideas (OUT OF SCOPE)
- The full placement validator (overlap with walls, cup, water and tools; no-place zones; TS parity): Phase 39.
- Timed valve (4th pose value): Phase 40.
- Playwright replay of a solution in a real browser: Phase 40 (Phase 37 replays wasm through Bun).
- Puzzle route, HUD, portrait bounds registry: Phase 38.
- Moving or buoyant cup region transforms and convex polygon cup interiors: Phase 40, if a level needs them.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GOAL-02 | Player wins when the fill stays at or above the target for a short settle time (about 0.5 s of simulation) before the time limit; a splash passing through does not win. | Pure `GoalTracker::observe` state machine (Pattern 4), observed once per fixed step in `PuzzleHooks::on_after_step` with a hooks-owned step counter that equals `SessionCore::step_index` (Pattern 3); `count_in_goal` generic over the group key so counting edge cases are pure unit tests (Pattern 5). |
| GOAL-04 | The same level and placements always produce the same outcome regardless of device speed; a recorded solution replays to the same result natively and in the browser. | Integer placements + τ/24 baked vertices (Pattern 6); fixture replay natively (`SessionCore`) and in wasm via Bun (Pattern 8); pre-spike evidence: native **release** equals wasm bit-for-bit in 23/25 scenes, but native **debug** (the `cargo test` profile) diverges in 12/25 scenes (Pitfall 1) — the replay contract must account for this. |
</phase_requirements>

## Summary

All integration points named in CONTEXT.md exist and are small. `SessionCore` (`crates/liquidfun-wasm/src/session.rs`, 449 lines) holds `scene_id: SceneId` and uses it in exactly five places: `particle_iterations(id)` at construction (line 112), `evict_escaped_particles(..., self.scene_id)` in `advance` (167) and `advance_profiled` (209), and `from_built(self.scene_id, presets)` in the two recreate branches of `apply_control` (244, 255). `escape::particle_has_escaped` only distinguishes `SceneId::WaveTank` (80 m / 80 m) from everything else (12 m fall / 48 m sideways). A `SessionSource` refactor that maps every `Scene(id)` arm to exactly these values is fingerprint-neutral by construction, and `SessionCore::create(SceneId)` must keep its signature because ~60 test call sites and `scene_spot.rs` use it.

I ran a **pre-spike** in this session (scratch crates outside the repo, nothing committed): the same FNV-1a 64 hash D-01 specifies, over particle positions → rigid segments → rigid circles of each captured frame, for all 25 scenes × 300 steps. **Native release vs wasm (wasm-pack release): 23/25 bit-identical for every step; `theo-jansen` first diverges at step 70 and `tesla-valve` at step 5** — both trig-heavy (motorized linkage; trig-built geometry). `dam-break` and `water-wheel` are identical. **But native debug (opt-level 0, what `cargo test` builds) diverges from native release in 12/25 scenes — `dam-break` at step 91, `water-wheel` at 39, `theo-jansen` at 2 — and debug matches wasm nowhere that release doesn't.** Release with `debug_assertions` on still equals release, so the difference is optimization level, not the `cfg(debug_assertions)` validation code. This is the single most important planning fact: a native unit test run under plain `cargo test` cannot be the "native" side of an exact native-vs-wasm comparison. The official spike (D-01) must run its native test both under `cargo test` (debug) and `cargo test --release`, and attribute divergence correctly.

The wasm-bindgen 0.2.128 surface was verified empirically: a static `#[wasm_bindgen(js_name = newPuzzle)] fn new_puzzle(id: String, kinds: &[u8], poses: &[i32]) -> Result<Self, JsError>` generates `static newPuzzle(id: string, kinds: Uint8Array, poses: Int32Array): ProofSession`; `Option<u32>`/`Option<u8>` getters generate `number | undefined`; a second `#[wasm_bindgen] impl ProofFrame` block in a child module works; a JsError thrown from the static constructor arrives as an `Error` in Bun. `bun scripts/web-build.ts wasm` took 9.9 s wall (warm cache); the Bun loader pattern from `readme-svg.ts` hashed 300 steps of dam-break in ~0.3 s.

**Primary recommendation:** Keep the scene path byte-identical by deriving `EscapeLimits` and particle iterations from `SessionSource` with the exact current mapping; put all puzzle logic (spec, builder, tools, pure goal core, hooks, replay/fixture test helpers) under `src/puzzle/`; carry puzzle status through an additive `SceneHooks::maybe_puzzle_status()` default method and a `FrameData` setter (not a new `FrameData::new` argument); and define "natively" in the exact replay contract as the **release** profile (`cargo test --release`), with the debug-profile unit test asserting outcome + margin + step tolerance.

## Project Constraints (from CLAUDE.md / AGENTS.md / standards)

- `crates/liquidfun` is not modified (also D-21). Production physics is native Rust; no runtime C++.
- Safe Rust only: `#![forbid(unsafe_code)]` in `liquidfun-wasm/src/lib.rs`; workspace lints `unsafe_code = "forbid"`, `missing_docs = "warn"`, clippy `all` + `pedantic` at warn, gated with `-D warnings`.
- No `unwrap()` in production code; propagate errors; `expect()` only for genuinely impossible states (tests use `expect` with messages).
- `maybe_` prefix for `Option` internals (Rust) and `maybe` prefix for optional TS fields/params (`maybePuzzle`).
- Full rotations use τ (`std::f32::consts::TAU`); angle steps are τ/24.
- Determinism: stable ordering and reproducible scenarios take precedence; no `HashMap` iteration in solver-visible order.
- Functional core / imperative shell: pure `goal.rs` and placement parsing; hooks/session are the shell.
- Prefer `foo.rs` + `foo/` over `foo/mod.rs` for new multi-file modules (`puzzle.rs` + `puzzle/`, `frame.rs` + `frame/`).
- Unit tests: one concern each, explicit `// Arrange`, `// Act`, `// Assert` comments.
- File length: `FILE_LINE_LIMIT = 628` physical lines per tracked source file (`scripts/bright-builds-check.ts:8`); no `.bright-builds-rules-checks.tsv` allowlist exists, so there are no exceptions. Functions over ~161 lines are refactor triggers.
- Don't hide foreign-language logic in strings; checked-in scripts should be rerunnable with breadcrumb logs/summaries in a gitignored location (`target/...`).
- Pre-commit: `cargo fmt`, clippy `-D warnings`, build, tests, plus touched web unit tests and `bun scripts/bright-builds-check.ts all`. `.planning/**` must never be mdformatted; non-GSD Markdown changes need `just markdown-check`.
- GSD workflow: edits through `/gsd-execute-phase`.
- Levels never enter `SCENE_IDS`, the scene sidebar, README SVG plans (`web/scripts/readme-svg/plans.ts`), portrait bounds or the scene survey (the README-gallery and portrait-frame rules in CLAUDE.md apply only to catalog scenes).
- Memory: macOS launch scan (~2 min per fresh ~40 MB debug test binary); run per-crate, avoid launching binaries in parallel, keep iterations small and commit promptly.

## Standard Stack

No new dependencies (milestone STACK.md; re-confirmed). Everything below is already pinned in the repo.

### Core
| Library / Tool | Version | Purpose | Evidence |
|---------|---------|---------|--------------|
| Rust toolchain | 1.97.0 (`rust-toolchain.toml`) | bridge + tests | [VERIFIED: rustc --version] |
| `wasm-bindgen` | `=0.2.128` (`crates/liquidfun-wasm/Cargo.toml`) | JS bindings: slices in, `Option<u32>`/`Option<u8>` out, static ctor | [VERIFIED: scratch wasm-pack build + Bun run this session] |
| `wasm-pack` | 0.15.0 (required exactly by `scripts/web-build.ts:17`) | builds `web/src/generated/liquidfun-wasm` | [VERIFIED: wasm-pack --version; cache has wasm-bindgen-cargo-install-0.2.128 and wasm-opt] |
| Bun | 1.4.2 (`web/package.json` packageManager) | spike/replay scripts | [VERIFIED: bun --version] |
| Vitest | 5.0.1 | `web/tests/**/*.test.ts` | [VERIFIED: web/package.json, vitest.config.ts] |
| TypeScript | 7.0.2, `exactOptionalPropertyTypes: true`, `noUncheckedIndexedAccess: true`, includes `scripts` | typecheck of new scripts | [VERIFIED: web/tsconfig.json] |

### Supporting (existing, reuse)
| Asset | Location | Use |
|---------|---------|-------------|
| `Fnv1a` (FNV-1a 64, `mix_byte/mix_u32/mix_f32`) | `crates/liquidfun-wasm/src/scene_spot/fingerprint.rs` (`pub(super)`, native-only module) | Widen to `pub(crate)` for the native spike/replay frame hash; no value change. |
| `attach_basin_fixture(world, body, &[Vec2; 4])` | `scene.rs:259` | Static quad fixtures for walls/cup/planks (density 0, friction 0.2). |
| `ParticleGroupSource::filled_shapes` + `ParticleGroupRecipe::new(.., ParticleGroupDestination::New).with_particle_flags(WATER).with_color(..)` | e.g. `scene/soup_family.rs:128-150` | Liquid blocks as groups (returns `ParticleGroupId`). |
| `parseRenderFrame`, `RawProofFrame` | `web/src/physics/frame.ts` | Add optional puzzle methods / `maybePuzzle`. |
| Generated-wasm loader | `web/scripts/readme-svg.ts:156-174` | Template for the Bun spike/replay script. |

**Installation:** none.

## Architecture Patterns

### Recommended Project Structure
```
crates/liquidfun-wasm/src/
├── lib.rs                  # MODIFIED: mod puzzle; pub use puzzle::PuzzleLevelInfo; ProofSession::new_puzzle, level_info
├── session.rs              # MODIFIED: SessionSource, from_source, create_puzzle, level_info, puzzle rejections
├── session/escape.rs       # MODIFIED: EscapeLimits { fall, escape }; for_scene(SceneId); SHARED
├── scene.rs                # MODIFIED (additive): SceneHooks::maybe_puzzle_status default None
├── frame.rs                # MODIFIED (small): FrameData.maybe_puzzle + setter; mod puzzle;
├── frame/puzzle.rs         # NEW: PuzzleFrameStatus + second #[wasm_bindgen] impl ProofFrame (6 getters)
├── puzzle.rs               # NEW: LevelId, parse_level_id, LEVEL_IDS, level_spec(id), re-exports
└── puzzle/
    ├── spec.rs             # NEW: LevelSpec, LiquidBlock, GridRect, ToolBudget (const-constructible)
    ├── first_pour.rs       # NEW: const FIRST_POUR: LevelSpec
    ├── tools.rs            # NEW: ToolKind, AngleIndex, Placement, Placements::parse (fail closed, canonical sort), plank vertices
    ├── angles.rs           # NEW (or inside tools.rs): const TAU_24: [(f32, f32); 24]
    ├── goal.rs             # NEW: pure count_in_goal, fill_ratio, GoalTracker, Outcome
    ├── builder.rs          # NEW: build_level(spec, &Placements) -> BuiltScene
    ├── hooks.rs            # NEW: PuzzleHooks: SceneHooks
    ├── info.rs             # NEW: #[wasm_bindgen] PuzzleLevelInfo
    ├── fixtures/first-pour-solution.txt, first-pour-empty.txt
    ├── tests.rs            # NEW (cfg(test)): allowlist, include_str PUZZLE_IDS, build/frame/levelInfo
    ├── goal/tests.rs       # NEW: counting + tracker AAA tests (or #[cfg(test)] mod tests in goal.rs)
    └── replay.rs (+ replay/tests.rs)  # NEW (cfg(test)): fixture parser, frame hash, spike test, fixture replay
web/
├── src/catalog/puzzles.ts            # NEW: export const PUZZLE_IDS = ["first-pour"] as const
├── src/physics/frame.ts              # MODIFIED: optional puzzle methods + maybePuzzle parse
├── scripts/puzzle-replay.ts          # NEW: Bun entry (spike + replay modes, --record prints)
├── scripts/puzzle-replay/{fixture,hash,wasm}.ts  # NEW: pure parser, FNV-1a 64 (BigInt), loader
├── tests/frame-puzzle.test.ts        # NEW: maybePuzzle parse + existing-fake compatibility
└── tests/puzzle-replay.test.ts       # NEW: fixture parser + FNV vectors
justfile                               # MODIFIED: puzzle-replay-wasm (+ optional puzzle-spike-wasm)
```
Budget notes (lines now / limit 628): `session.rs` 449, `frame.rs` 513, `lib.rs` 267, `scene.rs` 321, `session/tests.rs` 580 (do not grow it), `frame.ts` 221, `frame.test.ts` 482. Put new puzzle frame getters in `frame/puzzle.rs`, and new session tests in `puzzle/tests.rs`, not `session/tests.rs`. [VERIFIED: wc -l]

### Pattern 1: `SessionSource` with a byte-identical scene arm
**What:** Replace the `scene_id` field; precompute what the step loop needs at construction.
```rust
// session.rs (sketch)
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SessionSource {
    Scene(SceneId),
    Puzzle(LevelId, Placements),
}

pub(crate) fn create(id: SceneId) -> Result<Self, SessionError> {      // signature unchanged
    Self::from_source(SessionSource::Scene(id), Vec::new())
}
pub(crate) fn create_puzzle(level: LevelId, placements: Placements) -> Result<Self, SessionError> {
    Self::from_source(SessionSource::Puzzle(level, placements), Vec::new())
}
fn from_source(source: SessionSource, presets: Vec<(String, String)>) -> Result<Self, SessionError> {
    let (built, particle_iterations, escape_limits) = match &source {
        SessionSource::Scene(id) => (build_scene(*id, &presets)?,
                                     crate::scene::particle_iterations(*id),
                                     EscapeLimits::for_scene(*id)),
        SessionSource::Puzzle(level, placements) => (puzzle::build_level(*level, placements)?,
                                     puzzle::PARTICLE_ITERATIONS,   // = 2, the scene default
                                     EscapeLimits::SHARED),
    };
    // ... identical StepConfiguration::new(1.0 / 60.0, 8, 3).with_particle_iterations(..)
}
```
`advance` / `advance_profiled` call `evict_escaped_particles(&mut self.world, self.particle_system, self.particle_radius, self.escape_limits)` (disjoint field borrows are fine). `apply_control`'s two `from_built(self.scene_id, ..)` calls become `from_source(self.source.clone(), ..)`, but for puzzles `apply_control` returns `Err(UnknownControl)` **before** the gravity branch (line 239), because that branch runs ahead of the hooks.

### Pattern 2: `EscapeLimits` keeps the existing test API
```rust
// session/escape.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EscapeLimits { fall: f32, escape: f32 }
impl EscapeLimits {
    pub(crate) const SHARED: Self = Self { fall: BELOW_GROUND_METERS, escape: ESCAPE_METERS };
    pub(crate) const fn for_scene(scene: SceneId) -> Self {
        match scene {
            SceneId::WaveTank => Self { fall: WAVE_TANK_ESCAPE_METERS, escape: WAVE_TANK_ESCAPE_METERS },
            _ => Self::SHARED,
        }
    }
}
// Keep `particle_has_escaped(scene: SceneId, ..)` as a thin wrapper over a limits-based fn,
// so the 13 call sites in session/tests.rs (580 lines) do not change.
```

### Pattern 3: Puzzle status through an additive hook + frame setter
- `SceneHooks` gains `fn maybe_puzzle_status(&self) -> Option<PuzzleFrameStatus> { None }` (all 25 scene hooks untouched).
- `PuzzleHooks::on_after_step` increments its own `steps_taken` and observes. Inside `SessionCore::advance`, `self.step_index` is only updated **after** the loop (session.rs:193), so the hooks must not read it mid-batch; a hooks-owned counter starting at 0 at build equals the session step index after every step (both `advance` and `advance_profiled` call `on_after_step` exactly once per step). Add a test asserting `frame.step_index() == outcome_step` bookkeeping after `advance(4)` batches.
- Compute the initial `goal_count` at build (read-only scan) so the step-0 frame is truthful.
- `FrameData` gets `maybe_puzzle: Option<PuzzleFrameStatus>` (initialized `None` in `new`) and `set_puzzle_status(..)`, mirroring `set_circle_labels`; `capture_frame` calls it when `self.hooks.maybe_puzzle_status()` is `Some`. Do **not** add a `FrameData::new` argument: ~10 test call sites use the 7-arg form.

### Pattern 4: Pure step-latched `GoalTracker`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome { Running, Won { step: u32 }, Lost { step: u32 } }

pub(crate) struct GoalTracker { target: u32, settle_steps: u32, step_limit: u32, held: u32, outcome: Outcome }

impl GoalTracker {
    /// Fallible constructor: 1 <= target <= full, 1 <= settle_steps <= step_limit.
    pub(crate) fn observe(&mut self, count: u32, step: u32) -> Outcome {
        if self.outcome != Outcome::Running { return self.outcome; }       // latch
        self.held = if count >= self.target { self.held.saturating_add(1) } else { 0 };
        if self.held >= self.settle_steps && step <= self.step_limit {
            self.outcome = Outcome::Won { step };                           // win checked before loss
        } else if step >= self.step_limit {
            self.outcome = Outcome::Lost { step: self.step_limit };
        }
        self.outcome
    }
}
```
Unit tests (AAA, one concern each): hold from step s → `Won{s + settle - 1}`; splash of `settle - 1` steps → Running; dip resets the hold; win completing exactly at `step_limit` → Won; never holding → `Lost{step_limit}` at step_limit; observe after a win with count 0 → still Won; observe after a loss with a held count → still Lost.

### Pattern 5: Counting generic over the group key (testable without engine handles)
`ParticleGroupId` is an opaque handle with no public constructor (`liquidfun/src/identity.rs:169-224`), so a pure test cannot fabricate one. Make the counter generic:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GoalRect { min: Vec2, max: Vec2 }   // fallible ctor: finite, min < max

pub(crate) fn count_in_goal<G: PartialEq>(
    positions: &[Vec2], groups: &[Option<G>], counted: &[G], interior: GoalRect,
) -> u32 {
    positions.iter().zip(groups)
        .filter(|(_p, maybe_group)| maybe_group.as_ref().is_some_and(|g| counted.contains(g)))
        .filter(|(p, _)| interior.min.x < p.x && p.x < interior.max.x
                      && interior.min.y < p.y && p.y < interior.max.y)   // strict; NaN -> false
        .fold(0_u32, |n, _| n.saturating_add(1))
}
pub(crate) fn fill_ratio(count: u32, full_count: u32) -> f32 { /* min(count/full, 1.0); full>0 by spec */ }
```
Production call: `count_in_goal(view.positions(), view.group_ids(), &self.counted_groups, spec.cup_interior)` — `counted_groups: Vec<ParticleGroupId>` holds the ids returned by `create_particle_group` for blocks flagged counted (1–3 entries, so `contains` is cheap; no per-step allocation). Tests use `u8` keys and cover: inside, on-boundary (each edge), just above the rim, outside wall, empty slices, a non-counted group, `None` group, and "evicted" (fewer positions, full_count unchanged). `positions()` and `group_ids()` are aligned per particle [VERIFIED: particle/view.rs:95-115].

### Pattern 6: Exact placements (integers in, baked vertices out)
- Parse `kinds: &[u8]`, `poses: &[i32]`: check `poses.len() == kinds.len() * 3` with `checked_mul`, a `MAX_PLACEMENTS` bound (e.g. 16) **before** allocating, `ToolKind::try_from(u8)`, level allowlist, per-kind budget, `AngleIndex::new(u8::try_from(a)?)` with `< 24`, and the build-area check **in grid integers** (exact). Convert grid to `i16` via `i16::try_from` (fail closed) so world meters are `f32::from(x) / 20.0` — lint-clean (clippy pedantic `cast_precision_loss` rejects `i32 as f32`) and exact enough (correctly rounded division).
- Canonical order: `#[derive(PartialOrd, Ord)] struct Placement { kind, x, y, angle }` (field order = sort key) and `sort_unstable()`.
- Plank vertices: rotate the four local corners by `TAU_24[angle]` and add the centre, then `PolygonShape::new(&[Vec2; 4])` on a static body with `BodyDef::default()` (static, position 0, angle 0). Never `PolygonShape::oriented_box(.., angle)` for tools: it calls `Rotation::from_angle` → `sin`/`cos` [VERIFIED: collision/shape/polygon.rs:72-81]. `PolygonShape::new` gift-wraps ≤ 8 finite, non-welded, non-collinear points and derives normals with `sqrt`/division only.
- Use `std::f32::consts::FRAC_1_SQRT_2` for the ±45° entries and digit separators (`0.965_925_8`) to satisfy clippy `unreadable_literal`/`approx_constant`; unit-test each entry against f64 `cos`/`sin` within 1e-7 and `c² + s² ≈ 1`.

```rust
// f32-rounded cos/sin of k·τ/24 (k = 0..24); computed and checked this session.
pub(crate) const TAU_24: [(f32, f32); 24] = [
    (1.0, 0.0), (0.965_925_8, 0.258_819_04), (0.866_025_4, 0.5),
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2), (0.5, 0.866_025_4), (0.258_819_04, 0.965_925_8),
    (0.0, 1.0), (-0.258_819_04, 0.965_925_8), (-0.5, 0.866_025_4),
    (-FRAC_1_SQRT_2, FRAC_1_SQRT_2), (-0.866_025_4, 0.5), (-0.965_925_8, 0.258_819_04),
    (-1.0, 0.0), (-0.965_925_8, -0.258_819_04), (-0.866_025_4, -0.5),
    (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2), (-0.5, -0.866_025_4), (-0.258_819_04, -0.965_925_8),
    (0.0, -1.0), (0.258_819_04, -0.965_925_8), (0.5, -0.866_025_4),
    (FRAC_1_SQRT_2, -FRAC_1_SQRT_2), (0.866_025_4, -0.5), (0.965_925_8, -0.258_819_04),
];
```

### Pattern 7: Builder order (D-09) and static geometry
1. `World::new()`, gravity `(0, -10)` set once (no gravity preset path for puzzles).
2. Static walls and cup walls (`attach_basin_fixture` quads on one static body), in spec order.
3. Planks in canonical order (one static body per plank, or all on one static body; either is deterministic — one body per plank keeps future per-tool identity simple).
4. Particle system (`ParticleSystemDef::default().with_radius(r).with_maximum_count(cap)`), then liquid blocks as groups from `PolygonShape::new` rectangles (angle-free), recording counted group ids.
5. Read `particle_count` and assert `full_count <= particle_count` and `target_count <= full_count` in a test (group fill count is engine-determined by stride `PARTICLE_STRIDE (0.75) × diameter`, so the authored numbers must be checked against the built count).
Spec sanity: no liquid block overlaps the cup interior or the build area (assert initial `goal_count == 0`).

### Pattern 8: Fixture replay, natively and in wasm
Fixture (D-18), e.g. `first-pour-solution.txt`:
```
level first-pour
placement plank 12 -4 3
outcome won
outcome_step 412
final_count 1530
```
Define `final_count` precisely as **the goal count in the frame at `outcome_step`**. Both sides step with `advance(1)` and capture every step (needed to read the count at the outcome step and to evaluate the margin), stopping at the latch.
- Margin under the D-03 fallback without porting the tracker to TS: from the per-step count trace, check `min(count over the settle window ending at outcome_step) >= target + ceil(0.05 × full_count)` — equivalent to "the solution would still win with a 5%-higher target at that step" (a stricter alternative: re-run the pure `GoalTracker` over the trace with `target + margin` natively).
- Native: `#[test]` per fixture in `puzzle/replay/tests.rs` using `include_str!("../fixtures/…")`; `LIQUIDFUN_PUZZLE_RECORD=1` prints recorded values instead of asserting (never writes files).
- Wasm: `web/scripts/puzzle-replay.ts` reads the same files from `crates/liquidfun-wasm/src/puzzle/fixtures/`, builds `Uint8Array`/`Int32Array`, calls `ProofSession.newPuzzle`, steps, asserts; `--record` prints. Logs to `target/puzzle-replay/`.

### Pattern 9: Spike flow (Plan 1 Task 1)
1. `bun scripts/web-build.ts wasm` (≈10 s warm).
2. `cd web && bun scripts/puzzle-replay.ts spike > ../target/phase37/spike-wasm.txt` — lines `scene step hash16` for `dam-break`, `water-wheel`, `theo-jansen`, steps 1..=300 (D-01 set; one cheap extra rotating scene costs nothing).
3. Native `#[ignore = "phase 37 spike; run with --ignored --nocapture"]` test printing the same lines via `SessionCore::capture_frame` → `ProofFrame::from(frame)` lane getters → `Fnv1a`:
   - `cargo test -p liquidfun-wasm --lib spike -- --ignored --nocapture | grep '^SPIKE ' > target/phase37/spike-native-debug.txt`
   - `cargo test -p liquidfun-wasm --release --lib spike -- --ignored --nocapture | grep '^SPIKE ' > target/phase37/spike-native-release.txt`
4. First diverging step per scene: `diff` or a short `python3`/`awk` compare; record debug-vs-wasm, release-vs-wasm and debug-vs-release in the SUMMARY.

### Anti-Patterns to Avoid
- Adding a `SceneId::Puzzle…` variant or touching `SURVEY_SCENES`/`parse_scene_id` (breaks the 25-scene coverage tests and D-04).
- Calling `oriented_box(.., angle)`, `Rotation::from_angle`, or setting a non-zero `BodyDef` angle for tools.
- Reading `self.step_index` from hooks mid-batch, or deciding outcomes from captured frames in TS.
- Allocating a counted mask `Vec<bool>` per step (pass group slices instead).
- Comparing a **debug** native run against wasm and calling the difference "native vs wasm".

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FNV-1a 64 in Rust | a new hasher | `scene_spot::fingerprint::Fnv1a` (widen visibility) | already the Phase 35 survey contract; LE byte order, `to_bits` |
| Static wall quads | custom fixture code | `attach_basin_fixture` (`scene.rs:259`) | same density/friction as scenes |
| Liquid blocks | per-particle `create_particle_with_def` loops | `ParticleGroupSource::filled_shapes` + `ParticleGroupRecipe` | yields a `ParticleGroupId` for counting; engine stride |
| Polygon hull/normals | manual normals | `PolygonShape::new` | validates convexity/welds, sqrt-only normals |
| wasm loading in Bun | a custom loader | `readme-svg.ts` `loadWasm()` pattern | proven with `module_or_path` bytes |
| JSON/serde fixtures | serde in the bridge | plain-text line format (D-18) | no new deps; trivially shared |
| Fingerprint comparison | a new xtask mode | `jq` over `scene-spot.json` / JSONL (commands below) | `just playground-scene-spot` prints no fingerprints in its table |

**Key insight:** every risky piece here is determinism. Reusing the engine's own constructors with integer inputs and literal tables, and reading state only after `world.step`, keeps the puzzle path free of platform libm calls.

## Runtime State Inventory

This is a code refactor (`scene_id` → `source`) with no rename of persisted identifiers.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — no datastore holds `SceneId`/`scene_id`; puzzle progress storage is Phase 41. | none |
| Live service config | None — no external service config involved. | none |
| OS-registered state | None. | none |
| Secrets/env vars | None. New opt-in env var `LIQUIDFUN_PUZZLE_RECORD` (test-only, prints). | none |
| Build artifacts | `web/src/generated/liquidfun-wasm/` (gitignored, `.gitignore:5`) is stale after every bridge change; it was rebuilt at HEAD during this research (2026-10-11 16:58). `target/release/playground-scene-spot` and `target/phase35/before-full.jsonl` exist locally. | Rebuild wasm (`bun scripts/web-build.ts wasm`) before any Bun spike/replay run; rebuild the spot binary before the gate. |

## Common Pitfalls

### Pitfall 1: Debug-profile native tests are not the "native" side of native-vs-wasm
**What goes wrong:** `cargo test` builds opt-level 0. In this session, native debug frame hashes diverged from native release in 12/25 scenes (dam-break @91, water-wheel @39, theo-jansen @2, tesla-valve @1, wave-machine @34, soup @32, soup-stirrer @13, liquid-bubbler @28, surface-tension @86, stacked-drip @111, washing-machine @121, elastic-particles @237), while native release matched wasm in 23/25. [VERIFIED: scratch probes `ProofSession` via the public rlib API, 300 steps, per-step FNV-1a 64]
**Why:** Optimization level, not `cfg(debug_assertions)` (release + `CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true` == release). Root cause not identified; a plausible but unverified mechanism is signed-zero/NaN selection in `f32::min`/`max` differing between unoptimized libcalls and optimized instructions [ASSUMED].
**How to avoid:** Run the spike's native side in both profiles and attribute results; make any exact native-vs-wasm assertion use `cargo test --release`; let the default debug unit test assert outcome equality + margin + an outcome-step tolerance (or record per-profile values). Never "fix" this in `crates/liquidfun` (D-02).
**Warning signs:** Fixture recorded under `--release` failing only under plain `cargo test`.

### Pitfall 2: Trig-dependent dynamics diverge in wasm
**What:** `theo-jansen` (motorized linkage) diverged from step 70, `tesla-valve` from step 5 (release native vs wasm). Rotating dynamic bodies call `Rotation::from_angle` → `sin`/`cos` each step (platform libm natively, Rust `libm` on wasm32).
**Avoid:** Puzzle levels in this phase: static, angle-0 bodies plus particles only; tool rotation via `TAU_24`. Particle-only scenes with static/dynamic circles (dam-break, water-wheel) stayed bit-identical over 300 steps.

### Pitfall 3: Puzzle rejection must precede the gravity-control branch
`SessionCore::apply_control` handles `"gravity"` before calling hooks (session.rs:239-246) and would call `from_built(self.scene_id, ..)`. Reject puzzles at the top of `apply_control`. `apply_action`/`apply_pointer` reach the hooks, so `PuzzleHooks` returns `UnknownControl` (note `apply_pointer` returns `InvalidPointer` for non-finite coordinates before the hooks — acceptable, still fail-closed).

### Pitfall 4: `restoreAuthoredGravity` failure disposes the TS session
`web/src/physics/session.ts:150-161` disposes the session if `restoreAuthoredGravity` throws (while `setGravity` errors are swallowed). If Rust rejects `set_gravity` for puzzles, make `restore_authored_gravity` an `Ok(())` no-op for puzzle sources (gravity never changed). See Open Question 2.

### Pitfall 5: Private type in a public wasm signature
`lib.rs` declares `mod frame;` and re-exports `pub use frame::ProofFrame;`. A `ProofSession::level_info(&self) -> Result<PuzzleLevelInfo, JsError>` returning a type from a private `mod puzzle` trips `private_interfaces` (warn → error under `-D warnings`). Add `pub use puzzle::PuzzleLevelInfo;` and document every `pub` item (`missing_docs`).

### Pitfall 6: `JsError` cannot be constructed natively
Native tests call `build_core(..)`/core methods for error paths (see `lib.rs` tests). Expose `SessionCore::level_info() -> Result<LevelInfo, SessionError>` and a `build_puzzle_core(id, kinds, poses) -> Result<SessionCore, SessionError>` helper; test those, not the `JsError` wrappers.

### Pitfall 7: TS optional fields under `exactOptionalPropertyTypes`
`RenderFrame.maybePuzzle?: PuzzleFrameStatus` cannot be assigned `undefined`; use `...(maybePuzzle === undefined ? {} : { maybePuzzle })`. Declare the new `RawProofFrame` methods optional (`goalCount?(): number | undefined;`) so `FakeRawProofFrame implements RawProofFrame` in `web/tests/frame.test.ts:39` and `web/tests/session.test.ts:9`, plus the literal in `worker-messages.ts:158-172`, still compile; call them with `rawFrame.goalCount?.()`. Fail closed if the scalars are partially present, `outcome ∉ {0,1,2}`, `outcomeStep` presence ≠ (`outcome !== 0`), or `goalTarget > goalFull`. Note: the worker path (`validateTransferredFrame`) drops puzzle fields — fine, puzzles use the direct backend (Phase 38 territory).

### Pitfall 8: Clippy pedantic on casts and literals
`i32 as f32` (`cast_precision_loss`), `usize as u32` (`cast_possible_truncation`), unseparated long float literals (`unreadable_literal`), and approximations of known constants (`approx_constant`) all fail `-D warnings`. Use `i16::try_from` + `f32::from`, `u32::try_from`, `FRAC_1_SQRT_2`, digit separators.

### Pitfall 9: `cargo` at the repo root only targets `crates/liquidfun`
`default-members = ["crates/liquidfun"]`; `just clippy`/`just test` therefore skip the bridge. Always pass `-p liquidfun-wasm`. `cargo fmt --all` covers the workspace.

### Pitfall 10: Debug test runtime and launch scan
A debug run of all 25 scenes × 300 steps took 5 min 26 s in this session (release+assertions: 27 s). Keep `first-pour` near 1.5–2k particles and `step_limit` 600–720; mark the spike test `#[ignore]`. Each new lib test binary build waits on the macOS scan once (~2 min for a ~40 MB debug binary; `--release` is a separate binary). Avoid parallel binary launches.

### Pitfall 11: `include_str!` paths and file limits
From `crates/liquidfun-wasm/src/puzzle/tests.rs`, the TS file is `include_str!("../../../../web/src/catalog/puzzles.ts")` (four `..`, as in `scene_spot/tests.rs:9`). `web/src/catalog/scene-records.ts` is at 617/628 — do not touch it.

## Code Examples

### Bun: load generated wasm and hash a frame (verified this session)
```typescript
// Source: web/scripts/readme-svg.ts:156-174 loader + D-01 hash order; ran in Bun 1.4.2
const OFFSET = 0xcbf29ce484222325n, PRIME = 0x100000001b3n, MASK = (1n << 64n) - 1n;
function mixLane(hash: bigint, lane: Float32Array): bigint {
  const view = new DataView(lane.buffer, lane.byteOffset, lane.byteLength); // explicit LE bytes
  for (let index = 0; index < lane.byteLength; index += 1) {
    hash ^= BigInt(view.getUint8(index));
    hash = (hash * PRIME) & MASK;
  }
  return hash;
}
// per step: session.advance(1); const frame = session.captureFrame();
// hash = mixLane(mixLane(mixLane(OFFSET, frame.particlePositions()), frame.rigidSegments()), frame.rigidCircles());
// frame.free();
```
Pin both implementations with shared vectors: FNV-1a 64 of empty input = `cbf29ce484222325`, of `"a"` = `af63dc4c8601ec8c` [ASSUMED: standard published FNV test vectors], plus one lane vector (e.g. `[1.0, -0.0]`) computed in Rust and asserted in Vitest.

### wasm-bindgen surface (verified with 0.2.128 + wasm-pack 0.15.0)
```rust
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
impl ProofSession {
    /// Builds an allowlisted puzzle level from copied placement lanes.
    ///
    /// # Errors
    /// Returns a bounded JavaScript error for an unknown level or an invalid placement.
    #[wasm_bindgen(js_name = newPuzzle)]
    pub fn new_puzzle(level_id: String, kinds: &[u8], poses: &[i32]) -> Result<ProofSession, JsError> {
        build_puzzle_core(&level_id, kinds, poses).map(|core| Self { core }).map_err(js_error)
    }
}
// Generated: static newPuzzle(level_id: string, kinds: Uint8Array, poses: Int32Array): ProofSession;
// frame/puzzle.rs, second impl block:  #[wasm_bindgen(js_name = goalCount)] pub fn goal_count(&self) -> Option<u32>
// Generated: goalCount(): number | undefined;
```

### Native frame hash (reuse `Fnv1a`)
```rust
// cfg(test); Fnv1a made pub(crate) in scene_spot/fingerprint.rs (native-only module, tests are native)
fn frame_hash(frame: &ProofFrame) -> u64 {
    let mut hash = Fnv1a::new();
    for lane in [frame.particle_positions(), frame.rigid_segments(), frame.rigid_circles()] {
        for value in lane.iter() { hash.mix_f32(*value); }
    }
    hash.finish()
}
```

## State of the Art

| Old | Current | Impact |
|-----|---------|--------|
| `SessionCore { scene_id: SceneId }` | `SessionCore { source: SessionSource, escape_limits }` | scenes unchanged; puzzles get their own builder |
| Native-only fingerprints (Phase 35, release) | Per-step frame hash in both native and wasm | first measured cross-target evidence (pre-spike: 23/25 identical in release) |

**Deprecated/outdated:** milestone ARCHITECTURE.md's `poses: &[f32]` with radians is superseded by D-08 (`&[i32]`, grid + τ/24 index).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Debug/release divergence comes from optimization (e.g. min/max zero/NaN selection), mechanism unverified | Pitfall 1 | None for planning — the measured divergence is the actionable fact |
| A2 | FNV-1a 64 test vectors `""` → `cbf29ce484222325`, `"a"` → `af63dc4c8601ec8c` | Code Examples | A wrong vector would fail its own test immediately; low |
| A3 | `first-pour` with static angle-0 geometry will replay bit-identically in release native vs wasm (inferred from 23/25 particle/static scenes) | Summary, Pitfall 2 | The official spike + replay decide; fallback contract (D-03) covers it |
| A4 | Debug-profile replay of a ~2k-particle, ~700-step level takes ~10–20 s per fixture | Pitfall 10 | Slower CI/local loop; tune particle count/step limit |

## Open Questions

1. **What "natively" means in the D-03 contract, given Pitfall 1**
   - Known: release native == wasm in 23/25 scenes; debug native diverges in 12/25.
   - Unclear: whether the owner intended the debug unit test to be the exact native side.
   - Recommendation: record all three comparisons in the spike SUMMARY; apply D-03 "identical" to release-native vs wasm (asserted via `cargo test --release` + `just puzzle-replay-wasm`), and have the default debug unit test assert the fallback form (same outcome, +5% margin, outcome-step tolerance from the debug-vs-release spread measured on `first-pour`). Alternative (not recommended now): `[profile.dev.package.liquidfun] opt-level = 3`, which would change every existing debug test's trajectories.
2. **Reject `set_gravity` (tilt) for puzzles in Rust now?** D-15 lists controls, gravity presets and pointer input. Tilt is an in-run input (Pitfall 8 of the milestone research). Recommendation: reject `set_gravity` with `UnknownControl` for puzzle sources and make `restore_authored_gravity` an `Ok` no-op for them (Pitfall 4); Phase 38 removes the TS tilt binding.
3. **`levelInfo()` shape:** recommend a `#[wasm_bindgen] pub struct PuzzleLevelInfo` with getters (`levelId`, `title`, `targetCount`, `fullCount`, `settleSteps`, `stepLimit`, `cupInterior(): Float32Array [minX,minY,maxX,maxY]`, `buildArea(): Int32Array` in grid units plus `gridMeters()`, `toolKinds(): Uint8Array`, `toolCounts(): Uint32Array`) — one call, keeps `lib.rs` small.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc/cargo | all Rust work | ✓ | 1.97.0 | — |
| target `wasm32-unknown-unknown` | wasm build | ✓ | installed | — |
| wasm-pack | `bun scripts/web-build.ts wasm` | ✓ | 0.15.0 (exact required) | — |
| wasm-bindgen CLI 0.2.128 + wasm-opt | wasm-pack | ✓ | cached in `~/Library/Caches/.wasm-pack` (global `wasm-bindgen` is 0.2.106 but unused) | — |
| bun | scripts, vitest | ✓ | 1.4.2 | — |
| node | — | ✓ | v24.13.0 | — |
| jq | fingerprint compare | ✓ | 1.7.1 | python3 3.14.6 |
| just | recipes | ✓ | 1.48.0 | run underlying commands |
| `target/phase35/before-full.jsonl` | optional second baseline | ✓ | 25 records; HEAD matches it 25/25 (`--runs 1`, verified) | fresh baseline from HEAD (D-21) |

**Missing dependencies with no fallback:** none.

## Validation Architecture

(Included at the orchestrator's request; `workflow.nyquist_validation` is false in `.planning/config.json`.)

### Test Framework
| Property | Value |
|----------|-------|
| Rust | built-in harness, lib test binary of `liquidfun-wasm` (no `tests/` dir; keep it that way) |
| Web | Vitest 5.0.1, `web/vitest.config.ts` (`tests/**/*.test.ts`, node env) |
| Quick run | `cargo test -p liquidfun-wasm --lib puzzle` |
| Full suite | `cargo test -p liquidfun-wasm --all-features` (316 tests at v1.4) + `cd web && bun run test:unit` |

### Gate commands
| Gate | Command |
|------|---------|
| Baseline (phase start, before bridge edits) | `cargo build --release -p liquidfun-wasm --bin playground-scene-spot && mkdir -p target/phase37/bin && cp target/release/playground-scene-spot target/phase37/bin/spot-base && target/phase37/bin/spot-base --runs 1 > target/phase37/spot-base.jsonl && git rev-parse HEAD > target/phase37/phase-base.txt` |
| Format | `cargo fmt --all --check` |
| Lint | `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` |
| Build | `cargo build -p liquidfun-wasm --all-targets --all-features` |
| Rust tests (debug) | `cargo test -p liquidfun-wasm --all-features` |
| Rust exact replay (release) | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` |
| Spike (Plan 1 Task 1) | `bun scripts/web-build.ts wasm && (cd web && bun scripts/puzzle-replay.ts spike) > target/phase37/spike-wasm.txt`; `cargo test -p liquidfun-wasm --lib spike -- --ignored --nocapture \| grep '^SPIKE ' > target/phase37/spike-native-debug.txt`; same with `--release` → `spike-native-release.txt`; `diff` each pair |
| Wasm replay | `just puzzle-replay-wasm` (recipe: `bun scripts/web-build.ts wasm` then `cd web && bun scripts/puzzle-replay.ts replay`) |
| Web typecheck | `cd web && bun run typecheck` |
| Web unit (touched) | `cd web && bunx vitest run tests/frame.test.ts tests/frame-puzzle.test.ts tests/session.test.ts tests/worker-messages.test.ts tests/puzzle-replay.test.ts` |
| File lengths etc. | `bun scripts/bright-builds-check.ts all` |
| Engine untouched | `git diff --stat $(cat target/phase37/phase-base.txt) -- crates/liquidfun` (must print nothing) |
| 25/25 fingerprints | `just playground-scene-spot 2>&1 \| tee target/phase37/spot-after.log`, then `S=$(ls -td target/dam-break-perf/*/ \| head -1); jq -r '.scenes[] \| "\(.scene) \(.fingerprint)"' "$S/scene-spot.json" \| sort > target/phase37/after.txt; jq -r '"\(.scene) \(.fingerprint)"' target/phase37/spot-base.jsonl \| sort > target/phase37/base.txt; diff target/phase37/base.txt target/phase37/after.txt && test "$(wc -l < target/phase37/after.txt)" -eq 25 && echo "25/25 identical"` |

Facts behind the fingerprint gate [VERIFIED: code + run]: the spot binary prints one JSON object per scene with a `fingerprint` (16 hex) taken from the **first** run after 60 warmup + 120 measured steps (`scene_spot.rs:330`), so `--runs 1` and `--runs 3` give the same fingerprint (HEAD `--runs 1` equals `before-full.jsonl` `--runs 3`, 25/25). `just playground-scene-spot` (xtask, `--release`, runs 3, ~30 s) writes `target/dam-break-perf/<UTC stamp>/scene-spot.json` and prints a ranked table **without** fingerprints, so the comparison must read the JSON. A direct `--runs 1` run takes ~8 s.

### Phase Requirements → Test Map
| Req ID | Behavior | Type | Command | File Exists? |
|--------|----------|------|---------|-------------|
| GOAL-02 | settle-window win, splash no-win, loss at limit, latch survives slosh | unit (pure) | `cargo test -p liquidfun-wasm --lib puzzle::goal` | ❌ Wave 0 |
| GOAL-02 | counting edges (rim, boundary, outside wall, empty, evicted, non-counted group), fill clamp | unit (pure) | `cargo test -p liquidfun-wasm --lib puzzle::goal` | ❌ Wave 0 |
| GOAL-02 | build `first-pour` (empty and with plank), frame scalars + `level_info` | unit (session) | `cargo test -p liquidfun-wasm --lib puzzle::tests` | ❌ Wave 0 |
| GOAL-04 | fixture replay natively (debug contract) | unit | `cargo test -p liquidfun-wasm --lib puzzle::replay` | ❌ Wave 0 |
| GOAL-04 | fixture replay release-native exact | unit (release) | `cargo test -p liquidfun-wasm --release --lib puzzle::replay` | ❌ Wave 0 |
| GOAL-04 | fixture replay in wasm | script | `just puzzle-replay-wasm` | ❌ Wave 0 |
| GOAL-04 | `maybePuzzle` parse, fakes unchanged | vitest | `cd web && bunx vitest run tests/frame-puzzle.test.ts tests/frame.test.ts` | ❌ Wave 0 |
| D-04/D-05 | level/scene allowlists disjoint; `PUZZLE_IDS` == Rust allowlist | unit | `cargo test -p liquidfun-wasm --lib puzzle::tests` | ❌ Wave 0 |

### Wave 0 Gaps
- [ ] `crates/liquidfun-wasm/src/puzzle/goal.rs` tests — GOAL-02 pure core
- [ ] `crates/liquidfun-wasm/src/puzzle/tests.rs` — allowlists, include_str, build/frame/levelInfo, placement rejections (one test per D-09 check)
- [ ] `crates/liquidfun-wasm/src/puzzle/replay.rs` (+ tests) — fixture parser, frame hash, `#[ignore]` spike, fixture replay
- [ ] `web/tests/frame-puzzle.test.ts`, `web/tests/puzzle-replay.test.ts`
- [ ] `justfile` recipe `puzzle-replay-wasm`

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | allowlisted `LevelId` (fail closed), typed parse of `kinds`/`poses` with length/stride/count bounds before allocation, integer grid range via `i16::try_from`, angle `< 24`, build-area check in integers |
| V6 Cryptography | no | FNV-1a is a non-cryptographic fingerprint only |

| Pattern | STRIDE | Mitigation |
|---------|--------|------------|
| Oversized placement arrays from JS | DoS | `MAX_PLACEMENTS` checked on `kinds.len()` before any allocation; the copied slice itself is bounded by wasm memory |
| Malformed/partial frame scalars | Tampering | TS parse fails closed (all-or-nothing, value ranges) |
| Level geometry from JS | Tampering | not accepted — levels are Rust `const` data (SEED-002 out of scope) |

## Sources

### Primary (HIGH confidence)
- Repository source at `207114dfa`: `crates/liquidfun-wasm/src/{session.rs, session/escape.rs, session/tests.rs, scene.rs, frame.rs, lib.rs, scene_spot.rs, scene_spot/fingerprint.rs, scene_spot/tests.rs, bin/playground_scene_spot.rs, scene/{dam_break,basin_family,soup_family,liquid_timer,hydraulic_fountain}.rs}`; `crates/liquidfun/src/{particle/view.rs, identity.rs, collision/shape/polygon.rs, math/transform.rs, world/body.rs, world/particle_object/group.rs}`; `tools/xtask/src/playground/{spot.rs, spot/args.rs, spot/survey_table.rs, stamp.rs}`; `scripts/{web-build.ts, bright-builds-check.ts}`; `web/{package.json, tsconfig.json, vitest.config.ts, scripts/readme-svg.ts, src/physics/{frame,session,worker-messages}.ts, tests/{frame,session}.test.ts}`; `justfile`; `Cargo.toml`.
- wasm-bindgen 0.2.128 source (`~/.cargo/registry/.../wasm-bindgen-0.2.128/src/convert/{impls,slices}.rs`): `Option<u32>` f64-sentinel ABI, `Option<u8>` u32-sentinel ABI, slice `vectors!` for `u8 … i32 … f32`.
- Empirical runs this session: scratch wasm-pack build + Bun run of a `newPuzzle`-shaped API; `bun scripts/web-build.ts wasm` (9.87 s); native (debug, release, release+debug-assertions) vs wasm per-step frame hashes for all 25 scenes × 300 steps; `playground-scene-spot --runs 1` vs `target/phase35/before-full.jsonl` (25/25 equal).

### Secondary (MEDIUM confidence)
- `.planning/research/v1.5-puzzle-mode/{SUMMARY,ARCHITECTURE,PITFALLS}.md` (milestone research, built on).

### Tertiary (LOW confidence)
- Mechanism of the debug/release float divergence (A1).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — pinned versions verified locally; no new deps.
- Architecture: HIGH — every touch point read; wasm-bindgen shapes built and run.
- Pitfalls: HIGH for measured divergence and lint/TS rules; LOW only for the divergence root cause.

**Research date:** 2026-10-11
**Valid until:** 2026-11-10 (bridge code moves; re-check line counts and the spike if `crates/liquidfun` changes)
