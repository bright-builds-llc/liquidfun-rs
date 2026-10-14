# Phase 46: Cartesian Diver - Research

**Researched:** 2026-10-14
**Domain:** New playground scene (Rust `liquidfun-wasm` bridge scene + SolidJS catalog/player wiring), rigid-body buoyancy in LiquidFun particle water
**Confidence:** HIGH for code sites, signatures and wiring; MEDIUM for physics tuning numbers (must be measured in release native tests)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Squeeze input model
- **D-01:** One `squeezed: bool` intent lives in the hooks struct, and the last input wins. Pointer `Down` inside the bottle hit region sets it true. Pointer `Up` and `Cancel` set it false wherever they happen. The panel action `squeeze` (label "Squeeze") flips it. No physics runs in `apply_pointer` or `apply_action`; `on_advance` applies the state once per fixed step (ARCHITECTURE rule).
- **D-02:** The hit region is the bottle's outer rectangle plus a small margin, about 0.3 m. A `Down` outside it does nothing, so the hint can say "press and hold the bottle". Pointer `Move` is ignored.
- **D-03:** The keyboard path is the existing panel `action("squeeze", "Squeeze")` control. It adds no HUD control, new control kind or "hold" infrastructure. The README and survey cue this same action.

#### Sink and rise mechanism
- **D-04:** The diver is a single dynamic **circle** body (the v1.5 D-12 lesson that circles avoid native/wasm `sin`/`cos` divergence). Its radius is at least about 6 particle diameters, so single particle hits do not knock it around. Sleep is disabled on the body, so a density change is never ignored.
- **D-05:** Squeeze switches the diver fixture's density through `set_fixture_density` and then `reset_body_mass_data`. The released density is clearly below water and the squeezed density is clearly above it. The research starting point is about 0.5 released and 3 or more squeezed (PITFALLS 6), because water compresses with depth and densities near 1.5 to 2.0 hover mid-bottle. Tune the final values in native tests. Gravity scale is not used, because the caption names density switching.
- **D-06:** Ramp the effective density and the visual squeeze value over a short time, about 0.2 to 0.3 s, so the sink starts smoothly. Add moderate linear damping for a calm sink and rise. Exact values are Claude's discretion, within the timing rules in D-07.
- **D-07:** Timing is judged at gravity 10. While squeezed, the diver's center reaches the bottom third of the bottle within about 3 s. After release it returns to the top third within about 4 s. Native release-mode tests repeat the sink and rise at gravity 2, 10 and 80. Gravity 10 uses the 3 s and 4 s windows. Gravity 2 and 80 may use windows scaled by about √(10/g) with a sane cap, but must still show a full sink and a full rise. Tests assert outcomes with tolerance on the body y position, not exact trajectories.
- **D-08:** Every force comes from the engine's gravity and buoyancy, so the gravity slider and phone tilt keep working. The scene adds no scripted force on the diver and never pushes the water.

#### Squeeze visuals
- **D-09:** The bottle dent is **draw-only**. While squeezed, `collect_segments` returns side walls bowed inward by a few particle diameters at mid-height, following the same ramped squeeze value. The physical walls never move, so the water is never pushed and the scene stays stable at every gravity. The caption does not claim that the dent pushes the water.
- **D-10:** The air bubble is a drawn circle inside the diver, returned by `collect_circles` together with the diver circle. Its radius shrinks from about 0.6 to about 0.3 of the diver radius with the ramped squeeze value. The bubble has no fixture and no physics.

#### Bottle geometry and scale
- **D-11:** Use toy meter scale: particle radius about 0.04 to 0.06 m, thick overlapping static polygon walls, and a closed cap on top so water cannot leave. A tall bottle of roughly 2 to 2.5 m interior width and 5 to 6 m interior height is filled with particle water to just below the cap, leaving a small headspace. The diver floats at the surface near the top on load. Keep the scene at or below about 3,000 particles. The exact geometry and particle radius are Claude's discretion within this budget.
- **D-12:** The portrait frame in `web/src/catalog/portrait-bounds.ts` fits the bottle, walls included, with a small margin. A tall bottle is naturally portrait. The landscape `viewBounds` frames the whole bottle.
- **D-13:** Add `particle_iterations` (R6) or `EscapeLimits` (R12) arms only if tests show they are needed.

#### Phone long-press callout
- **D-14:** Suppress iOS and Android long-press callouts on the scene canvas for every scene. Add `-webkit-touch-callout: none` and `user-select: none` (with the `-webkit-` prefix) to `canvas.scene-canvas`, and call `preventDefault` on `contextmenu` for the scene canvas. This is a shared web change with no effect on simulation fingerprints. An e2e check under the mobile viewport asserts the computed styles and that a dispatched `contextmenu` event on the canvas is default-prevented. A real-iPhone long press is an optional manual check, not a gate.

#### Catalog, copy and cues
- **D-15:** Use id `cartesian-diver`, module `cartesian_diver`, variant `CartesianDiver`, title "Cartesian Diver" and catalog family `curiosity`. This is the family's first scene, so create `web/src/catalog/scene-records-curiosity.ts` and `web/src/catalog/previews-curiosity.tsx` and append them to the `SCENES` spread and the `ScenePreview` switch in catalog order. Credits say "original experimental scene", and the source is `sceneSource("cartesian_diver.rs")`.
- **D-16:** Use a written hint instead of `WATCH_FIRST_HINT`, for example "Press and hold the bottle (or use Squeeze) to sink the diver; let go and it rises." The caption must say plainly that switching the diver's density stands in for the squeezed air bubble, for example: "A diver floats in a closed bottle of particle water. Squeezing switches the diver to a heavier density, standing in for its squeezed air bubble, so it sinks; letting go switches it back and it rises. The dent and the bubble are drawn." Claims of compressed air, real pressure or simulated gas are not allowed (SCENE-03).
- **D-17:** The README plan cues the `squeeze` action at about sample 20 (about 1 s) and again at about sample 110 (about 5.5 s), so the 10 s clip shows a full sink and a full rise. Check the timing against the actual clip and adjust the sample indices if needed. The survey entry is `SurveyCue::Action("squeeze")`, and `only_idle_scenes_are_scripted` is updated (R10). Demo-media `SCENE_CAPTURE_PLANS` uses the same action. E2E `POINTER_CONTROL` gets an entry so the interactive scene is not dropped into the watch-first loop.
- **D-18:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed. The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

#### Gates and evidence
- **D-19:** Before the phase closes, `just scene-fingerprint-gate` must report all 25 protected ids unchanged with `cartesian-diver` under "added (accepted)". Record the diver's `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene cartesian-diver` in the phase summary.
- **D-20:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 26 protected ids unchanged. Phase 47 then protects the diver.
- **D-21:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player, hud-safe-frame and the new long-press check), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact densities, ramp time, damping, diver radius, bottle dimensions, particle radius and count, within D-05 to D-11.
- Bottle-dent shape (how many segments, bow profile), and the preview drawing style.
- Plan breakdown and wave ordering, provided the baseline growth commit (D-20) is last and separate.
- Whether to split `cartesian_diver.rs` into `cartesian_diver/geometry.rs` and `cartesian_diver/tests.rs` (the `foo.rs` + `foo/` layout) to stay under the file-length limit.

### Deferred Ideas (OUT OF SCOPE)
- Depth-dependent "no return" behavior and multiple divers (DIVER-04, future requirement).
- A squeeze-strength slider for hovering the diver mid-bottle (research differentiator, not in DIVER-01..03).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DIVER-01 | Diver floats near the top of a closed bottle; press-and-hold on the bottle or the Squeeze toggle sinks it; release rises | Hooks pattern (§Pattern 1), pointer flow (§Pattern 3), density switch API (§Pattern 2), timing tests (§Validation) |
| DIVER-02 | Bottle visibly dents and the drawn bubble shrinks while squeezed | Draw-only `collect_segments`/`collect_circles` (§Pattern 4); renderer facts (§Rendering facts) |
| DIVER-03 | Reliable sink/rise across gravity 2..80; phone long press opens no callout | Compression scaling analysis (§Pitfall 1, 2); callout CSS + contextmenu, Chromium limitation verified (§Pattern 6, Pitfall 5) |
| SCENE-01 | Catalog record + credits, preview, portrait frame, README preview, scale legend, gravity slider, HUD safe frame | Full site map (§Site Map), test constraints (§Pitfall 6) |
| SCENE-02 | Moves on its own except where it invites a press; ≤ ~3,000 particles; smooth on phones | Particle budget table (§Geometry budget); scene-spot recording (§Gates) |
| SCENE-03 | Honest caption naming the faked mechanism | Caption rules + scenes.test.ts format constraints (§Pitfall 6) |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Follow `docs/adding-a-scene.md` for every Rust and web site; the scene fingerprint baseline grows only in its own commit. [VERIFIED: AGENTS.md, docs/adding-a-scene.md]
- No change to `crates/liquidfun` (engine); bridge code only in `crates/liquidfun-wasm` on existing public `World` APIs. No new dependencies, no new cue kinds, bridge APIs or worker messages. [VERIFIED: ROADMAP shared rules]
- Rust: no `unwrap()` in production; `?`/`map_err` propagation; `expect()` only in tests; `maybe_` prefix for `Option`s; `foo.rs` + `foo/` module layout; tau, not pi, for angles (`std::f32::consts::TAU`); `let...else` for early returns. [VERIFIED: global CLAUDE.md]
- Tests: one concern per test, Arrange/Act/Assert comments. [VERIFIED: global CLAUDE.md]
- Files ≤ 628 physical lines (`bun scripts/bright-builds-check.ts all`). [VERIFIED: scripts/bright-builds-check.ts `FILE_LINE_LIMIT = 628`]
- TS: `camelCase`, `maybe` prefix for nullable/optional values; bun is the web runner (`bun.lock`). [VERIFIED: repo]
- Pre-commit Rust: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. [VERIFIED: global CLAUDE.md, justfile]
- `.planning/**` is never mdformatted; run `just markdown-check` after changing non-GSD Markdown (README gallery update from `just readme-svg` counts). [VERIFIED: CLAUDE.md Repo-Local Guidance]
- New playground scene must: add `README_SVG_PLANS` entry, add `PORTRAIT_VIEW_BOUNDS` entry, show the scale legend (shared, automatic), keep new HUD chrome off (none here). [VERIFIED: AGENTS.md]
- Simulation stays renderer-independent; caption honesty (Transparency constraint). [VERIFIED: CLAUDE.md]
- New tests go into the existing `liquidfun-wasm` lib test binary (a new test binary costs ~2 min of macOS scanning). [VERIFIED: CONTEXT.md + user memory "macOS launch stalls"]
- Execute plans sequentially on the main tree (no worktrees). [VERIFIED: user memory]

## Summary

Every seam this phase needs already exists. `SceneHooks` (`crates/liquidfun-wasm/src/scene.rs:174-228`) gives `on_advance` (called once per fixed step before `World::step`), `apply_action`, `apply_pointer(kind, world_x, world_y)`, and draw-only `collect_segments`/`collect_circles`. The frame packs segments as 4 floats and circles as 3 floats and does not read world fixtures for drawing, so a bowed draw-only wall and a drawn bubble circle need no physics and never affect the fingerprint. The fingerprint hashes only the world's particles and bodies (`scene_spot/fingerprint.rs`). The engine provides `World::set_fixture_density` (it does **not** recompute mass), `World::reset_body_mass_data`, `BodyDef::with_sleeping_allowed(false)`, `BodyDef::with_linear_damping`, `BodyDef::with_fixed_rotation`, and `ParticleSystemDef::with_pressure_strength`. `create_fixture` returns the `FixtureId` that the hooks need to keep.

The real risk is physics tuning across the gravity range, not wiring. LiquidFun pressure per unit weight is `pressure_strength * density * (2r * inv_dt_substep)²` (`particle/solver/pressure.rs:233-240`, verified), while hydrostatic load scales with `g * depth`. Water compression, and with it the depth at which a given body density hovers, therefore scales roughly with `g / (pressure_strength * r² * particle_iterations²)`. The 40-09 table (r = 0.04 m, default strength 0.05) shows density 2.0 hovering at 3 m depth at g = 10. At g = 80 the same water compresses about 8 times more, so a squeezed density of "3" may hover mid-bottle at g = 80 unless the scene stiffens its water. Possible scene-local levers are a larger particle radius (0.05 to 0.06), a higher `pressure_strength` (Hydraulic Fountain already ships 0.25), a higher squeezed density, and, last, an R6 `particle_iterations` arm. At g = 2 the opposite problem appears. Linear body damping caps the speed at `a/c`, so sink time scales like 1/g instead of 1/√g. Keep damping low (about 0.5 to 1.5) so that g = 2 fits a √5-scaled window.

Two web facts change the plan. (1) Chromium does **not** support `-webkit-touch-callout`: the property is dropped from both computed style and CSSOM (verified with the repo's Playwright Chromium). So the e2e cannot assert it through `getComputedStyle`. Assert `user-select`, `-webkit-user-select`, `touch-action` and `contextmenu.defaultPrevented` in e2e, and check the callout declaration in the built CSS text or a source-text unit test. (2) `web/tests/scenes.test.ts` requires every non-watch-first hint to contain "Labeled controls also work from the keyboard.", so the D-16 example hint needs that sentence appended. Demo-media `SceneAction` supports only canvas `click`/`drag`, not named actions, so D-17's "same action" for demo-media must be a click inside the bottle.

**Primary recommendation:** Build `scene/cartesian_diver.rs` (+ `cartesian_diver/geometry.rs`, `cartesian_diver/tests.rs`) on the Float or Sink pattern. Start with a release-mode tuning test at **g = 80 first**, using particle radius 0.05 to 0.06, `pressure_strength` about 0.2, released density 0.5, squeezed density 4 to 6, and linear damping ≤ 1.5. Lock the numbers, then do the catalog and web wiring, then the long-press change, then the README assets and gates, and finally the separate baseline-growth commit.

## Standard Stack

No new dependencies (locked by ROADMAP). Everything is in-repo:

| Component | Version | Purpose | Source |
|-----------|---------|---------|--------|
| `liquidfun` crate (in-repo) | workspace | `World`, `BodyDef`, `FixtureDef`, `CircleShape`, `PolygonShape`, `ParticleSystemDef` | [VERIFIED: codebase] |
| `liquidfun-wasm` (in-repo, private) | workspace | Scene bridge, `SessionCore`, `SceneHooks`, `scene_spot` | [VERIFIED: codebase] |
| Rust toolchain | 1.97.0 | build/test | [VERIFIED: `rustc --version`] |
| bun | 1.4.2 | web typecheck/unit, scripts | [VERIFIED: `bun --version`] |
| Vitest (node env) | repo pin | web unit tests (`environment: "node"`, no DOM) | [VERIFIED: web/vitest.config.ts] |
| Playwright Chromium | repo pin (chromium-1243 cached) | e2e; `touch` project uses Pixel 7 descriptor at 390x844 | [VERIFIED: web/playwright.config.ts, ~/Library/Caches/ms-playwright] |
| ffmpeg | 8.1.1 | `just readme-svg` WebP encode | [VERIFIED: `ffmpeg -version`] |

**Installation:** none.

## Site Map (exact code sites)

### Rust (`crates/liquidfun-wasm/src`)

| # | File:location | Edit | Current shape |
|---|---------------|------|---------------|
| R1 | `scene/cartesian_diver.rs` (new) + `scene/cartesian_diver/geometry.rs`, `scene/cartesian_diver/tests.rs` | Builder `pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError>` + `CartesianDiverHooks` | Template: `scene/float_or_sink.rs` (561 lines, inline tests) and `scene/washing_machine.rs` (`mod geometry;` at top, `#[cfg(test)] mod tests;` at bottom) |
| R2 | `scene.rs` lines 9-33 | `mod cartesian_diver;` (list is alphabetical; insert after `basin_family`) | |
| R3/R5 | `scene.rs` `enum SceneId` (l.42-68) and `build_scene` match (l.230-256) | `CartesianDiver` + `SceneId::CartesianDiver => cartesian_diver::build(&scene_presets),` | `build_scene` applies the gravity preset after build, so the builder must set its authored gravity `(0, -10)` and must not read the `gravity` preset |
| R4 | `scene.rs` `parse_scene_id` (l.103-132) | `"cartesian-diver" => Ok(SceneId::CartesianDiver),` | |
| R6 | `scene.rs` `particle_iterations` (l.138-145) | Only if tuning needs it (D-13) | `SceneId::TeslaValve => tesla_valve::PARTICLE_ITERATIONS` precedent |
| R7 | `scene.rs` `ALL_SCENE_IDS` (l.73-99) + `scene/gravity_slider.rs` `variant_index` (l.161-189) | Append `SceneId::CartesianDiver` last; add `SceneId::CartesianDiver => 25,` | |
| R8 | `session/tests.rs` `parse_scene_id_maps_allowlisted_tokens` (l.16-42) | Append `("cartesian-diver", SceneId::CartesianDiver),` | file is 580 lines; +1 is fine |
| R9 | `scene_spot.rs` `SURVEY_SCENES` (l.46-104) | Append `("cartesian-diver", SceneId::CartesianDiver, SurveyCue::Action("squeeze")),` | the cue is applied once before the 60 warmup + 120 measured steps |
| R10 | `scene_spot/tests.rs` `only_idle_scenes_are_scripted` (l.71) | `["float-or-sink", "impulse", "drawing-particles", "cartesian-diver"]` | `survey_cues_apply_to_fresh_sessions` then also proves `apply_action("squeeze")` returns `Ok` |
| R12 | `session/escape.rs` | Not needed: a bottle inside y ∈ [-0.5, 6.5] and x ∈ ±2 is far inside the 12 m fall and 48 m sideways limits | [VERIFIED: escape.rs constants] |

### Web

| # | File | Edit |
|---|------|------|
| W1 | `web/src/catalog/scenes.ts` `SCENE_IDS` | Append `"cartesian-diver"` last |
| W2 | `web/src/catalog/scene-records-curiosity.ts` (new) exporting `CURIOSITY_SCENE_RECORDS`; `web/src/catalog/scene-records.ts` | Append `...CURIOSITY_SCENE_RECORDS` after `...MACHINE_SCENE_RECORDS` |
| W5 | `web/src/catalog/previews-curiosity.tsx` (new) `CartesianDiverPreview()`; `web/src/catalog/previews.tsx` | Import + `case "cartesian-diver": return <CartesianDiverPreview />;` (the `never` guard fails compile until added) |
| W6 | `web/src/catalog/portrait-bounds.ts` `PORTRAIT_VIEW_BOUNDS` | `"cartesian-diver": CARTESIAN_DIVER_PORTRAIT` (named const with a comment deriving it from geometry, Tesla Valve style) |
| S1 | `web/scripts/readme-svg/plans.ts` `README_SVG_PLANS` | `{ id: "cartesian-diver", cues: [{ atSample: 20, kind: "action", name: "squeeze" }, { atSample: 110, kind: "action", name: "squeeze" }] }` |
| S2 | `docs/assets/readme/cartesian-diver-10s.svg` + `.webp`, README gallery | `just readme-svg`, review, `just markdown-check` |
| S3 | `web/scripts/demo-media/model.ts` `SCENE_CAPTURE_PLANS` | `{ id: "cartesian-diver", title: "Cartesian Diver", route: "/liquidfun-rs/#/scene/cartesian-diver", interactionStep: 180, action: { kind: "click", point: { x: 0.5, y: 0.5 } } }` (see Pitfall 7) |
| T1 | `web/tests/scenes.test.ts` | `UI_SPEC_DESCRIPTIONS`, `UI_SPEC_HINTS` (`Record<SceneId,string>`), `expectedIds`, `expectedTitles`; optionally add `"Squeeze"` to `LOCKED_ACTION_LABELS` |
| T2 | `web/tests/scene-catalog-controls.test.ts` `RECREATING_CONTROL_IDS` | Append `"gravity"` (the action has `recreates: false`) |
| T3 | `web/tests/scene-catalog-credits.test.ts` `expectedPaths` | `"cartesian-diver": "crates/liquidfun-wasm/src/scene/cartesian_diver.rs"`; optionally `expect(inspirationById["cartesian-diver"]).toEqual([SHOWCASE_HREF])` |
| T4 | `web/tests/portrait-bounds.test.ts` | `MIN_HEIGHT_FRACTION["cartesian-diver"]` (about 0.85 for a frame whose aspect is near the phone); wall points in the `wallEndpoints` map (inner wall corners and cap) |
| E1 | `web/e2e/player-helpers.ts` `SCENE_HASH_PATHS` | `"cartesian-diver": "/liquidfun-rs/#/scene/cartesian-diver"` (file 580 lines; +1 OK) |
| E2 | `web/e2e/player.spec.ts` `POINTER_CONTROL` | `"cartesian-diver": { gesture: "click", control: "Squeeze" }` |
| D-14 | `web/src/styles/player-viewport.css` `canvas.scene-canvas` (l.47-50); `web/src/input/canvas-pointer.ts` `attachCanvasPointer` | CSS declarations; `contextmenu` listener added and removed alongside the pointer listeners |
| D-14 e2e | new `web/e2e/scene-touch.spec.ts`; `web/playwright.config.ts` | Touch project `testMatch: /(puzzle|scene)-touch\.spec\.ts$/` and the chromium `testIgnore` set to the same regex |

**Already derived (no edit):** hash routing, sidebar, `ScaleLegend`, gravity slider via `withGravitySlider`, `hud-safe-frame.spec.ts` (iterates `SCENE_IDS`), shell focus counts, xtask scene list, `PAGE_SUMMARY`. [VERIFIED: docs/adding-a-scene.md "Already derived"]

## Architecture Patterns

### Recommended module layout
```
crates/liquidfun-wasm/src/scene/
├── cartesian_diver.rs          # consts, build(), CartesianDiverHooks + SceneHooks impl, squeeze ramp
└── cartesian_diver/
    ├── geometry.rs             # bottle walls (polygons), inner-face segments, dent polyline, hit test
    └── tests.rs                # construction, input model, mass reset, sink/rise at g 2/10/80 (release)
web/src/catalog/
├── scene-records-curiosity.ts  # CURIOSITY_SCENE_RECORDS (Phases 47-49 append here)
└── previews-curiosity.tsx      # CartesianDiverPreview
```

### Pattern 1: Intent in hooks, physics in `on_advance` (D-01)
**What:** `apply_pointer`/`apply_action` only mutate `squeezed`. `on_advance` ramps `squeeze` (0..1) toward the target by `STEP_DT / RAMP_SECONDS`. When the ramp value changed, it writes the lerped density and resets mass.
**Facts:** `SessionCore::advance` calls `hooks.on_advance(&mut world, system)` once per fixed step before `world.step` (`session.rs:226-229`). Step dt is a constant 1/60 (`session.rs:166`). Scenes define `const STEP_DT: f32 = 1.0 / 60.0;` (`soup_stirrer.rs:26`).
```rust
// Sketch based on float_or_sink.rs / soup_stirrer.rs conventions
struct CartesianDiverHooks {
    diver: BodyId,
    diver_fixture: FixtureId,      // returned by world.create_fixture
    squeezed: bool,                // intent; last input wins
    squeeze: f32,                  // ramped 0..1, drives density, dent and bubble
}

fn on_advance(&mut self, world: &mut World, _system: ParticleSystemId) -> Result<(), SessionError> {
    let target = if self.squeezed { 1.0 } else { 0.0 };
    let next = step_toward(self.squeeze, target, STEP_DT / RAMP_SECONDS);
    if next.to_bits() == self.squeeze.to_bits() {
        return Ok(());
    }
    self.squeeze = next;
    let density = RELEASED_DENSITY + (SQUEEZED_DENSITY - RELEASED_DENSITY) * next;
    world.set_fixture_density(self.diver_fixture, density).map_err(|_error| SessionError::StepFailed)?;
    world.reset_body_mass_data(self.diver).map_err(|_error| SessionError::StepFailed)
}
```
Write density only on change, so the idle scene does no per-step work and stays deterministic.

### Pattern 2: Diver body (D-04, D-05)
```rust
// Source: liquidfun BodyDef API (crates/liquidfun/src/world/body.rs:266-370)
let definition = BodyDef::new(BodyType::Dynamic, DIVER_START, 0.0, true)   // 4th arg is `active`, NOT allow-sleep
    .map_err(|_error| SceneError::Body)?
    .with_linear_damping(DIVER_LINEAR_DAMPING)
    .map_err(|_error| SceneError::Body)?
    .with_sleeping_allowed(false)
    .with_fixed_rotation(true);   // optional; circle at local origin, rotation is invisible and adds nothing
let body = world.create_body(&definition).map_err(|_error| SceneError::Body)?;
let circle = CircleShape::new(Vec2::ZERO, DIVER_RADIUS).map_err(|_error| SceneError::Geometry)?;
let fixture_def = FixtureDef::new(Shape::from(circle), RELEASED_DENSITY, 0.2, 0.0, false, FilterData::default())
    .map_err(|_error| SceneError::Fixture)?;
let diver_fixture = world.create_fixture(body, &fixture_def).map_err(|_error| SceneError::Fixture)?; // -> FixtureId
```
- `set_fixture_density(fixture, density)` "Changes fixture density without implicitly recomputing its body's mass" (`fixture_object.rs:265-284`). `reset_body_mass_data(body)` recomputes from fixtures (`fixture_object.rs:233-244`). [VERIFIED: codebase]
- The circle is centered at the body origin, so the local center does not move and `reset_body_mass_data` does not shift velocity. [ASSUMED: Box2D ResetMassData semantics; low risk for a centered circle]
- `body_snapshot(body)?.mass()` (`BodySnapshot::mass`, `world/body.rs:543`) lets a unit test prove the mass changed. [VERIFIED]
- Sleep alternatives exist: `world.set_body_sleeping_allowed(body, false)` wakes immediately (`body_object.rs:318`). The session's `set_gravity` (phone tilt) already calls `wake_dynamic_bodies` (`session.rs:196-205`). [VERIFIED]

### Pattern 3: Pointer and action input (D-01..D-03)
- Rust receives `PointerKind::{Down, Move, Up, Cancel}` with finite world coordinates, validated by `SessionCore::apply_pointer` (`session.rs:349-367`). [VERIFIED]
- **Never return `Err` from `apply_pointer` for an ignored pointer**: an error goes to `onFailure` → `failScene` (`canvas-pointer.ts:58-89`). Return `Ok(())` for Move and for a Down outside the bottle, as Float or Sink does. [VERIFIED]
- `apply_action`: `if name != SQUEEZE_ACTION { return Err(SessionError::UnknownControl); }` then flip `squeezed` (soup_stirrer pattern, `soup_stirrer.rs:224-234`). `apply_control` returns `Err(UnknownControl)` for everything. The session intercepts `gravity` before the hooks (`session.rs:318-327`). [VERIFIED]
- Web pipeline: a captured pointer `up`/`cancel` is forwarded wherever it lands. `cssPointFromClient` does not clamp to the canvas rect, and `lostpointercapture` maps to `cancel` (`pointer.ts:96-130`). `cancel()` on detach sends `cancel` with the last world point. D-01 "Up/Cancel release wherever they happen" therefore needs no web change. [VERIFIED]
- Changing gravity rebuilds the session (`session.rs:322-327`, `Self::from_source`), so `squeezed` resets to false. That is expected; tests must set gravity **before** squeezing. [VERIFIED]

### Pattern 4: Draw-only dent and bubble (D-09, D-10)
- `collect_segments(&self, world) -> Result<Vec<RigidSegment>, _>` and `collect_circles(&self, world) -> Result<Vec<(Vec2, f32)>, _>` are pure drawing lanes. Float or Sink returns constant basin segments that are separate from its polygon fixtures (`float_or_sink.rs:118-133, 275-290`). [VERIFIED]
- Draw the inner faces (floor, two sides, cap; an optional drawn-only neck above the cap). For the dent, replace each side with N ≈ 8 to 12 chords: `x(y) = x_wall ∓ squeeze * DENT_DEPTH * sin(TAU * 0.5 * (y - y_floor) / interior_height)`. Use `TAU * 0.5`, not `PI` (repo rule). Keep `DENT_DEPTH` at about 2 to 3 particle diameters (about 0.2 to 0.3 m), because particles next to the wall stay where they are and will show outside a deep drawn dent.
- Circles: `[(diver_pos, R), (bubble_pos, R * lerp(0.6, 0.3, squeeze))]`, diver first. The renderer fills each circle with `RIGID_FILL_COLOR` and then strokes it (`web/src/render/canvas.ts:238-275`), so the bubble shows as an outline inside the diver. A bubble offset upward by `R - r_bubble - margin` reads as trapped air; keep it fully inside the diver. [VERIFIED renderer; offset is discretion]
- `collect_circle_labels` default is empty → labels resized to blanks (`session.rs:522-528`). [VERIFIED]
- Neither lane enters the fingerprint, which hashes particles plus body position/angle/velocity only (`scene_spot/fingerprint.rs:50-89`). [VERIFIED]

### Pattern 5: Release-only native tests
```rust
// Source: crates/liquidfun-wasm/src/puzzle/levels_tests/level_proofs.rs:44
#[cfg_attr(debug_assertions, ignore = "release-only sink/rise timing (D-07)")]
#[test]
fn squeezed_diver_reaches_bottom_third_at_gravity_80() { /* ... */ }
```
- Run them with `cargo test -p liquidfun-wasm --release --lib cartesian_diver`. Debug builds diverge from release (STATE 37-01), so timing assertions must not run in debug. [VERIFIED]
- Session-level driving: `SessionCore::create(SceneId::CartesianDiver)`, `session.apply_control("gravity", "80")` (rebuilds and returns `Ok(true)`), `session.apply_action("squeeze")`, `session.apply_pointer("down", x, y)`, and `session.advance(n)` with `n ≤ MAX_ADVANCE_STEPS = 4` (loop in chunks of 4, as in `float_or_sink.rs` `advance_steps`). Read the diver center from `ProofFrame::from(session.capture_frame()?).rigid_circles()[0..3]` (x, y, r of circle 0), or keep a test-only hook accessor. [VERIFIED]
- Hooks-level tests (no session) can destructure `super::build(&[])` into `BuiltScene { mut world, particle_system, mut hooks, .. }` (`float_or_sink.rs:494-500`). [VERIFIED]

### Pattern 6: Long-press callout suppression (D-14)
```css
/* web/src/styles/player-viewport.css — extend the existing rule at l.47 */
canvas.scene-canvas {
  position: relative;
  z-index: 1;
  -webkit-touch-callout: none;
  -webkit-user-select: none;
  user-select: none;
}
```
```ts
// web/src/input/canvas-pointer.ts — inside attachCanvasPointer
function onContextMenu(event: Event): void {
  event.preventDefault();
}
canvas.addEventListener("contextmenu", onContextMenu);
// ...and in detach(): canvas.removeEventListener("contextmenu", onContextMenu);
```
`attachCanvasPointer` is the only pointer pipeline for the scene canvas (`web/src/player/scene-surface.ts:180-188`), and it already owns listener attach and detach. A one-line `onContextMenu` on the `<canvas class="scene-canvas">` in `web/src/components/PlayerPanel.tsx:196` is an equally valid alternative. The listener approach is easier to unit test in the node Vitest env with a plain `EventTarget` fake. The `touch-action: none` declaration is already on all canvases (`player-viewport.css:45`). The CSS rule also matches the puzzle canvas, which shares the `scene-canvas` class (`PuzzlePanel.tsx:320`), and that is harmless.

### Anti-Patterns to Avoid
- **Physics in `apply_pointer`/`apply_action`:** violates D-01. Changing density there also misses the ramp.
- **Moving walls for the dent:** pressurizes a closed particle bottle and pushes water. D-09 locks it as draw-only.
- **`set_body_gravity_scale`:** rejected by D-05 because the caption names density. Also, it does not wake the body.
- **`HashMap` or float-equality branching that changes step order:** keep deterministic state (`bool` + `f32`).
- **Returning `Err` for an ignored pointer:** fails the scene in the browser.
- **Reading the `gravity` preset in the builder:** `build_scene` strips it and applies it after the build (`scene.rs:220-258`).

## Geometry budget (D-11) [derived arithmetic; tune in tests]

Particle stride in LiquidFun is 0.75 × diameter (Hydraulic Fountain's `POOL_STRIDE = r * 2 * 0.75`). Water area = interior width × water height.

| Particle r | Grid spacing (0.75·2r) | 2.2 m × 5.0 m water | 2.4 m × 5.2 m water | 6 diameters (D-04) |
|------------|------------------------|---------------------|---------------------|--------------------|
| 0.04 | 0.06 | ~36 × 83 ≈ 3,000 (at budget) | ~40 × 87 ≈ 3,480 (over) | 0.48 m |
| 0.05 | 0.075 | ~29 × 66 ≈ 1,900 | ~32 × 69 ≈ 2,200 | 0.60 m |
| 0.06 | 0.09 | ~24 × 55 ≈ 1,300 | ~26 × 57 ≈ 1,500 | 0.72 m |

Subtract the particles under the diver: skip grid points within `R + r` of the diver start. Recommended start: **r = 0.05 m, interior about 2.4 m × 5.6 m, water to about 5.0 m, diver R about 0.45 to 0.6 m**. That gives about 1,900 to 2,100 particles, enough headspace for a floating diver, and a 2.4 m wide bottle that still leaves at least 0.6 m (12 particle diameters) of water around the diver.

D-04 says "radius at least about 6 particle diameters". PITFALLS 6 phrases the risk as a diver "under about 6 particle diameters" (likely meaning across). Either reading works at r = 0.05, where R = 0.45 to 0.6 m. Note in the plan which one was applied. [ASSUMED interpretation]

Use thick overlapping static polygon walls, about 0.3 to 0.5 m thick, with a floor and cap that overlap the side walls (`attach_basin_fixture` in `scene.rs:271-291` takes 4 vertices with friction 0.2). The portrait frame is then the outer wall rectangle plus about 0.1 m. For example, with a 2.4 m × 5.6 m interior and 0.3 m walls, the outer box is 3.0 × 6.2 m, an aspect of 0.48 against the phone's 0.46. That fills width and height (`DEFAULT_MIN_WIDTH_FRACTION = 0.85`, so `MIN_HEIGHT_FRACTION` about 0.85 is plausible). Measure it in the test before setting the threshold.

Fill the water with a per-particle grid, as Float or Sink does (`float_or_sink.rs:139-171`). This gives an exact, assertable count. Use `ParticleFlags::WATER` and the shared water color `(77, 163, 255, 255)`. Set `with_maximum_count` just above the count.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Buoyancy | Scripted upward force | Engine particle–body coupling (Float or Sink proves it) | D-08: gravity slider and tilt must keep working |
| Mass update | Manual `set_body_mass_data` math | `set_fixture_density` + `reset_body_mass_data` | Recomputes mass and inertia from the fixture |
| Keeping the diver awake | Periodic wake impulses | `BodyDef::with_sleeping_allowed(false)` | Precedent in 7 scenes (`hydraulic_fountain.rs:246` etc.) |
| Pointer gesture tracking | New hold/press state in TS | Existing `reducePointerEvent` capture pipeline | Already sends down/up/cancel, including outside the canvas |
| Keyboard squeeze | New "hold" control kind | `action("squeeze", "Squeeze")` from `scene-record-shared.ts` | D-03; no new infrastructure |
| Release-only test gating | Custom env flags | `#[cfg_attr(debug_assertions, ignore = "...")]` | Established in `level_proofs.rs` |
| Fingerprint comparison | Ad-hoc jq | `just scene-fingerprint-gate` | The PREP-01 gate is the xtask |

## Runtime State Inventory

Not a rename or migration phase. One stateful artifact applies: `docs/benchmarks/scene-fingerprints.json` must **not** change until the separate final D-20 commit. Generated wasm in `web/src/generated/liquidfun-wasm` must be rebuilt (`bun scripts/web-build.ts wasm`, which `just readme-svg`/`just web-build` run) before web e2e or README capture sees the new scene. [VERIFIED: justfile]

## Common Pitfalls

### Pitfall 1: The squeezed diver hovers mid-bottle at gravity 80
**What goes wrong:** At g = 80 the squeezed diver stops between top and bottom.
**Why:** Pressure per weight is `pressure_strength · density · (2r · inv_dt_sub)²` (`pressure.rs:82, 233-240`), and `inv_dt_sub = 60 · particle_iterations` (`particle_coupling/executor.rs:58-71`). Hydrostatic load scales with `g · depth`, so compression scales about with `g / (strength · r² · iterations²)`. In the 40-09 table (r 0.04, default strength 0.05, g 10), density 2.0 hovers at 3 m depth. At g = 80, compression is about 8 times larger. [VERIFIED formula; scaling estimate ASSUMED]
**How to avoid:** Tune at **g = 80 first**. In order, use these levers: r 0.05 to 0.06 (1.6 to 2.25 times stiffer than 0.04); `ParticleSystemDef::with_pressure_strength(0.2..0.25)` (4 to 5 times; Hydraulic Fountain ships 0.25); squeezed density 4 to 6 or higher; and only then an R6 `particle_iterations` arm (3 or 4 iterations is 2.25 to 4 times stiffer, at about 1.5 to 2 times particle cost; D-13). Keep the water column at or below about 5 m.
**Warning signs:** The diver's y at g = 80 stops changing while it is still above the bottom-third line.

### Pitfall 2: Linear damping makes gravity 2 too slow
**What goes wrong:** At g = 2 the sink takes far longer than the √(10/2) ≈ 2.24× window (about 6.7 s sink, 9 s rise).
**Why:** With linear damping `c`, terminal speed is about `a/c`, and `a ∝ g`, so time ∝ 1/g (5 times slower at g = 2). Particle drag is closer to quadratic (time ∝ 1/√g). [ASSUMED analysis]
**How to avoid:** Keep `DIVER_LINEAR_DAMPING` at about 0.5 to 1.5 and let water drag do the calming. Example at g = 2 with c = 2: net sink acceleration of about 1.35 m/s² gives a terminal speed of about 0.7 m/s and 3.5 m in about 5.5 s, which is borderline. With c = 4 it fails.
**Test windows:** g = 10 uses 3 s and 4 s. g = 2 uses ⌈3·√5⌉ = 7 s and ⌈4·√5⌉ = 9 s. g = 80 should **not** shrink below the g = 10 windows ("sane cap"); use 3 s and 4 s as upper bounds. Let the initial lattice settle for about 1 s after a gravity rebuild before squeezing, because the column compresses on load (strongly at g = 80).

### Pitfall 3: `set_fixture_density` without `reset_body_mass_data`
**What goes wrong:** Nothing happens on squeeze. [VERIFIED: doc comment `fixture_object.rs:265`]
**Guard:** A unit test asserts `body_snapshot(diver).mass()` rises after the ramp completes.

### Pitfall 4: Diver overlaps particles, the cap or walls on load
**What goes wrong:** Particles spawned inside the diver explode outward, or a diver touching the cap jams.
**How to avoid:** Skip grid particles within `R + r` of the diver start. Start the diver centered near the water surface, with headspace ≥ R + a few particle diameters below the cap. A released diver (density 0.5) floats about half out of the water, so it may rest against the cap if headspace < R. That is acceptable, but keep it clear at load.

### Pitfall 5: Asserting `-webkit-touch-callout` through computed style in Chromium
**What goes wrong:** The e2e fails because Chromium drops the property.
**Evidence:** A Chromium probe with the repo's Playwright returned `getPropertyValue("-webkit-touch-callout") === ""`. The CSSOM rule text was `canvas.scene-canvas { user-select: none; touch-action: none; }`, and `user-select`/`-webkit-user-select` computed `none`. A dispatched cancelable `contextmenu` with a `preventDefault` listener gave `defaultPrevented === true`. WebKit is not installed for this Playwright pin (`webkit-2359` missing). [VERIFIED: local Chromium probe, 2026-10-14]
**How to avoid:** In the e2e, assert `user-select`, `-webkit-user-select` and `touch-action` computed `none`, plus `contextmenu.defaultPrevented`. Prove the callout declaration either by fetching the built stylesheet text (`fetch(link.href).then(r => r.text())` contains `-webkit-touch-callout:none`, which proves it survived the Tailwind/Vite build) or by a Vitest source-text test reading `src/styles/player-viewport.css`. Also record this deviation from D-14's literal "asserts the computed styles" wording in the plan. The real-iPhone check stays optional.

### Pitfall 6: Catalog copy rules enforced by `web/tests/scenes.test.ts`
- Every hint for a scene **not** in `WATCH_FIRST_SCENE_IDS` must contain `"Labeled controls also work from the keyboard."` (l.268-285). The D-16 example hint must have it appended, e.g. "Press and hold the bottle, or use Squeeze, to sink the diver. Let go, or use Squeeze again, and it rises. Labeled controls also work from the keyboard." [VERIFIED]
- Hints must not contain "Press Space to pause" or "Live frame from this repository's Rust engine" (`FORBIDDEN_HINT_PHRASES`). [VERIFIED]
- Descriptions are UI-SPEC locked strings. Every machine scene description ends with "This is an original experimental scene." Follow that convention, e.g.: "Watch a diver float in a closed bottle of particle water. Squeezing switches the diver to a heavier density, which stands in for its squeezed air bubble, so it sinks. Letting go switches it back and it rises. The dent and the bubble are drawn. This is an original experimental scene." Avoid "compressed air", "pressure" and "gas" (SCENE-03). [VERIFIED convention; wording is discretion]
- Credits: `implementationPath: sceneSource("cartesian_diver.rs")`, `inspiration: [SHOWCASE]` (W2). `links.ts` rejects scene paths containing `/`. [VERIFIED]

### Pitfall 7: Demo-media cannot fire a named action
`SceneAction` in `web/scripts/demo-media/model.ts:8-14` is only `{kind:"click", point}` or `{kind:"drag", start, end}`, and `interactionStep` is the literal type `180`. "Uses the same action" (D-17) is not expressible without a new kind, which is forbidden. Use a canvas click inside the bottle (`{ x: 0.5, y: 0.5 }`): it produces Down then Up (a brief squeeze), and `pointerAccepted` increments, which is what the capture asserts (`capture.ts:442-455`). Demo-media is an optional local workflow (`just demo-media`). [VERIFIED]

### Pitfall 8: README timing window
At 20 Hz samples (`SVG_SAMPLE_HZ = 20`, 3 steps per sample, 201 samples per 10 s; `web/src/export/duration.ts`), sample 20 = 1.0 s and sample 110 = 5.5 s. A 3 s sink plus a 4 s rise would end at 9.5 s, which is tight. Tune the g = 10 rise to about 2 to 3 s, or move the release cue to about sample 100. A cue at sample k runs before the 3 steps that produce sample k (`schedule.ts:18-36`). [VERIFIED]

### Pitfall 9: e2e interactive loop expectations
`POINTER_CONTROL` entry `{ gesture: "click", control: "Squeeze" }`: `clickCanvas` clicks at ratio (0.5, 0.45) of the canvas (`player-helpers.ts:456-462`). In the landscape 16:9 desktop canvas with a centered bottle, that point is inside the hit region. `expectAcceptedPointerGesture` requires `data-pointer-accepted > 0` and the last kind `up` or `down`. `activateLabeledControl("Squeeze")` falls through to `getByRole("button", { name: "Squeeze", exact: true })` and opens the scene controls if needed. [VERIFIED]

## Code Examples

### Hit test and pointer handling
```rust
fn apply_pointer(&mut self, _world: &mut World, _system: ParticleSystemId,
                 kind: PointerKind, world_x: f32, world_y: f32) -> Result<(), SessionError> {
    match kind {
        PointerKind::Down if geometry::in_squeeze_region(Vec2::new(world_x, world_y)) => self.squeezed = true,
        PointerKind::Up | PointerKind::Cancel => self.squeezed = false,
        PointerKind::Down | PointerKind::Move => {}
    }
    Ok(())
}
```

### Session-level release test (shape)
```rust
#[cfg_attr(debug_assertions, ignore = "release-only sink/rise timing (D-07)")]
#[test]
fn squeezed_diver_sinks_to_the_bottom_third_at_gravity_ten() {
    // Arrange
    let mut session = SessionCore::create(SceneId::CartesianDiver).expect("diver scene should construct");
    advance_steps(&mut session, SETTLE_STEPS);
    // Act
    session.apply_action("squeeze").expect("squeeze should toggle");
    let maybe_step = first_step_where(&mut session, SINK_LIMIT_STEPS_G10, |y| y < BOTTOM_THIRD_Y);
    // Assert
    assert!(maybe_step.is_some(), "diver should reach the bottom third within 3 s at g 10");
}
```

### Readme plan entry
```ts
{
  id: "cartesian-diver",
  cues: [
    { atSample: 20, kind: "action", name: "squeeze" },
    { atSample: 110, kind: "action", name: "squeeze" },
  ],
},
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hard-coded "25" scene counts | Derived from `ALL_SCENE_IDS` / `SCENE_IDS` | Phase 45 | Adding a scene touches only the listed sites |
| Monolithic `scene-records.ts`/`previews.tsx` | Family modules spread into `SCENES` | Phase 45 (45-05) | Diver creates the `curiosity` family modules |
| Full fingerprint regeneration | id-join gate, baseline grows in a separate commit | Phase 45 | D-19/D-20 sequencing |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Compression and hover depth scale about with g/(strength·r²·iterations²) | Pitfall 1 | Tuning lever order may differ; the g = 80 sweep finds out quickly |
| A2 | Particle drag on the diver is close to quadratic, so with low linear damping sink time ∝ 1/√g | Pitfall 2 | g = 2 window could still fail; then raise squeezed density or relax the window with a documented cap |
| A3 | Starting numbers (r 0.05, strength about 0.2, densities 0.5/4 to 6, damping ≤ 1.5) | Summary, Geometry | Only starting points; native tests decide |
| A4 | `reset_body_mass_data` keeps velocity unchanged for a body-centered circle | Pattern 2 | Low; a sanity test asserts motion continuity anyway |
| A5 | D-04 "6 particle diameters" interpretation (radius vs diameter) | Geometry budget | Visual size only; both readings fit the budget at r = 0.05 |
| A6 | The Tailwind/Vite build keeps `-webkit-touch-callout` in the bundled CSS | Pitfall 5 | If stripped, the e2e stylesheet-text check catches it |

## Open Questions (RESOLVED)

1. **Does g = 80 need an R6 `particle_iterations` arm?**
   - Known: the stiffness formula and three scene-local levers (radius, pressure strength, squeezed density).
   - Unclear: whether those three are enough without the substep cost.
   - Recommendation: the first plan runs a release-mode sweep test at g = 80 (it may be an `#[ignore]` spike removed after, Phase 40-08/45 style). Add R6 only if needed, and record ms/step.
   - RESOLVED: conditional R6 lever in 46-01 Task 2. The release-mode g = 80 tuning sweep tries particle radius, pressure strength and squeezed density first; the `SceneId::CartesianDiver => cartesian_diver::PARTICLE_ITERATIONS` arm (value 3) is added only if those three fail (D-13), and 46-01-SUMMARY records ms/step.
2. **Exact e2e assertion for the callout (D-14 wording vs Chromium reality)**
   - Recommendation: computed `user-select`/`touch-action` + `contextmenu.defaultPrevented` in e2e, plus a built-CSS text check for `-webkit-touch-callout:none`. Note the deviation in the plan.
   - RESOLVED: computed-style e2e + CSS source-text test per UI-D64 (46-03). `web/e2e/canvas-callout.spec.ts` asserts computed `user-select: none`, `touch-action: none` and a default-prevented cancelable `contextmenu` at 390x844; `web/tests/canvas-callout.test.ts` checks the CSS source for `-webkit-touch-callout: none`, which Chromium drops from computed style.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Native tests, scene-spot, gate | ✓ | 1.97.0 | — |
| bun | Web typecheck/unit, scripts | ✓ | 1.4.2 | — |
| just | Recipes | ✓ | 1.48.0 | Run underlying commands |
| Playwright Chromium | e2e (player, hud-safe-frame, scene-touch) | ✓ | cached chromium-1243 | — |
| Playwright WebKit | Not required | ✗ (webkit-2359 not installed) | — | Chromium + CSS text check; optional real iPhone |
| ffmpeg | `just readme-svg` WebP | ✓ | 8.1.1 | — |
| Python 3.13 + mdformat 1.0.0 | `just markdown-check` (README gallery) | not probed | — | Required by repo guidance |

## Validation Architecture

(`workflow.nyquist_validation` is false. This section is included because the orchestrator asked for exact commands per requirement.)

| Req | Behavior | Type | Command |
|-----|----------|------|---------|
| DIVER-01 | Builds; floats near top at rest; Down in region / action squeezes; Up/Cancel/outside behave per D-01/D-02; mass changes after ramp | Rust unit (debug OK) | `cargo test -p liquidfun-wasm --lib cartesian_diver` |
| DIVER-01/03 | Sink to bottom third and rise to top third at g 2, 10, 80 within the D-07 windows | Rust release | `cargo test -p liquidfun-wasm --release --lib cartesian_diver` |
| DIVER-02 | Segments bow inward and the bubble radius shrinks with `squeeze`; unsqueezed segments are straight | Rust unit | `cargo test -p liquidfun-wasm --lib cartesian_diver` |
| DIVER-03 (callout) | Canvas `user-select:none`, `touch-action:none`, `contextmenu` prevented, built CSS has `-webkit-touch-callout:none` | e2e (touch project) | `just web-build && cd web && CI=true bunx playwright test e2e/scene-touch.spec.ts --project touch` |
| SCENE-01 | Ordering/registry: Rust id order, survey order, gravity slider builds | Rust unit | `cargo test -p liquidfun-wasm --lib -- scene::tests scene_spot gravity_slider session::tests` |
| SCENE-01 | Catalog, credits, controls, portrait, README plan coverage, demo-media plans, scale legend | Web unit | `cd web && bun run typecheck && bun run test:unit` |
| SCENE-01 | Interactive loop + HUD safe frame | e2e | `just web-build && cd web && CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts --project chromium` |
| SCENE-01 | README assets | Script + review | `just readme-svg && just markdown-check` |
| SCENE-02 | ≤ ~3,000 particles; ms/step recorded | Rust unit + survey | `cargo test -p liquidfun-wasm --lib cartesian_diver` (count assert); `cargo xtask playground scene-spot --scene cartesian-diver` |
| SCENE-03 | Locked description/hint strings | Web unit | `cd web && bun run test:unit` (scenes.test.ts) |
| PREP-01 | 25 protected unchanged, diver added | Gate | `just scene-fingerprint-gate`; after the D-20 commit: `just scene-fingerprint-gate --spot target/dam-break-perf/<stamp>/scene-spot.json` (26 unchanged) |
| Repo | Lint/format/length | Checks | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && bun scripts/bright-builds-check.ts all` |

**Wave 0 gaps:** `crates/liquidfun-wasm/src/scene/cartesian_diver/tests.rs` (new; part of the lib test binary, so no new binary scan) and `web/e2e/scene-touch.spec.ts` plus the `playwright.config.ts` regex change. Optionally add a Vitest test for the contextmenu listener with an `EventTarget` fake (the node env has no DOM).

**Sampling:** per task, run the targeted `cargo test ... --lib cartesian_diver` and `bun run test:unit`. Before the phase gate, run the full `cargo test --all-features`, the touched e2e specs, `just scene-fingerprint-gate`, and the bright-builds check.

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Existing: `parse_pointer_kind` allowlist, finite-coordinate check (`session.rs:349-367`); the action name is allowlisted and fails closed with `UnknownControl` |
| V2/V3/V4/V6 | no | Static client-side playground; no auth, sessions or crypto |

| Threat | STRIDE | Mitigation |
|--------|--------|-----------|
| Unbounded or NaN pointer coordinates | Tampering / DoS | Already rejected before hooks (`InvalidPointer`) |
| Unknown action string | Tampering | `Err(UnknownControl)`, never a panic |

## Sources

### Primary (HIGH, verified in this session)
- `crates/liquidfun-wasm/src/scene.rs` (SceneHooks, registry), `scene/float_or_sink.rs`, `scene/soup_stirrer.rs`, `scene/hydraulic_fountain.rs`, `scene/washing_machine.rs` + `washing_machine/tests.rs`, `scene/gravity_slider.rs`
- `crates/liquidfun-wasm/src/session.rs` (advance order, gravity rebuild, capture), `session/escape.rs`, `scene_spot.rs`, `scene_spot/fingerprint.rs`, `scene_spot/tests.rs`
- `crates/liquidfun/src/world/object/fixture_object.rs:17-30, 233-284`, `world/object/body_object.rs:280-375`, `world/body.rs:255-370, 543`, `particle/solver/pressure.rs:34-240`, `world/particle_coupling/executor.rs:58-71`, `particle/definition/system_definition.rs:134, 564`
- `web/src/catalog/*`, `web/src/input/canvas-pointer.ts`, `web/src/input/pointer.ts`, `web/src/styles/player-viewport.css`, `web/src/render/canvas.ts`, `web/scripts/readme-svg/{plans,schedule}.ts`, `web/src/export/duration.ts`, `web/scripts/demo-media/{model,capture}.ts`, `web/tests/{scenes,portrait-bounds,scene-catalog-controls,scene-catalog-credits}.test.ts`, `web/e2e/{player.spec,player-helpers,hud-safe-frame.spec,puzzle-touch.spec}.ts`, `web/playwright.config.ts`
- Local Chromium probe (Playwright, 2026-10-14): `-webkit-touch-callout` dropped; `user-select` computed `none`; `contextmenu` preventDefault observable
- `.planning/phases/40-*/40-09-SUMMARY.md` static pool buoyancy table; `.planning/research/v1.6-curiosity-scenes/{FEATURES,PITFALLS,ARCHITECTURE}.md`

### Secondary (MEDIUM)
- MDN `-webkit-touch-callout` page: non-standard, WebKit/iOS Safari feature (the compatibility table was not readable; the Chromium behavior was verified locally instead)

## Metadata

**Confidence breakdown:**
- Code sites and wiring: HIGH. Every site was read at its current line numbers.
- Engine APIs: HIGH. Signatures and doc comments were read.
- Physics tuning numbers: MEDIUM-LOW. The formula is verified, but the scaling and starting values are estimates that the first plan's release sweep must settle.
- Web long-press approach: HIGH. Chromium behavior was probed directly.

**Research date:** 2026-10-14
**Valid until:** 2026-11-13 (stable in-repo code; re-check line numbers if Phase 46 plans are delayed past other edits)
