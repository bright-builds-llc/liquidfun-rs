# Phase 18: Six Native Physics Demos - Research

**Researched:** 2026-09-18
**Domain:** Native Rust scene authorship, SolidJS catalog/player controls, WASM session factory
**Confidence:** HIGH for existing APIs, player contracts, and locked decisions; MEDIUM for Float or Sink / Jelly Drop / Water Wheel visual satisfaction until native spikes run

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Catalog cards and previews

- **D-01:** Replace the Phase 17 honest name list with a six-card catalog. Every card shows the approved title, a short one-behavior description, a static preview, and one Open action that loads `#/scene/{id}` in the shared player. All six ids remain `dam-break`, `fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, and `water-wheel`.
- **D-02:** Previews are static inline SVG or CSS illustrations authored in the frontend. Do not start a WASM world per card, capture live Canvas thumbnails, or animate fake physics in the catalog. Catalog metadata lives in `web/src/catalog/` and must not own live sessions.
- **D-03:** Mark every approved scene ready once it has a working native implementation. Keep unknown/empty hash fallbacks from Phase 17. Do not keep "not ready yet" chips on shipped scenes, and do not invent extra catalog search, filters, or a scene editor.

### Per-scene control surface

- **D-04:** Each scene exposes two or three labeled, bounded controls in the shared player, plus the existing play/pause/reset chrome. Prefer discrete presets and constrained numeric ranges over unbounded sliders. Changing a construction preset must recreate the world; the UI must say so before the visitor applies it.
- **D-05:** Locked first-pass controls, matching the approved feature table:
  - Dam Break: water-amount preset, gravity preset, obstacle drop/reset.
  - Fountain: emission-rate preset, launch-speed preset, aim-angle preset.
  - Float or Sink: body/density preset, drop-body action.
  - Color Mixer: mix-strength preset, stir-speed preset.
  - Jelly Drop: shape preset, softness preset applied on reset, drop/poke action.
  - Water Wheel: jet-strength preset, emission on/off.
- **D-06:** Runtime-only controls may update the live session without reset when the engine can apply them safely (for example jet strength or emission toggle). If a control cannot be applied live, it resets and the copy says so. Pointer aiming, stirring, poking, and stuck-pointer polish remain Phase 19 unless a control above already covers the documented interaction as a labeled button or preset.

### Scene authorship and physics honesty

- **D-07:** Author all six persistent scenes in `liquidfun-wasm` using the public `liquidfun` API. Diagnostic protocol recipes and the desktop testbed are capability references only; do not import them into the browser package or replay two-particle inspect-and-destroy schedules as gallery demos.
- **D-08:** Keep one opaque session that exclusively owns one world. Expand construction from Dam Break-only `ProofSession::new()` to a checked scene-id factory. Preserve copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch.
- **D-09:** All particle motion, collisions, buoyancy response, color mixing, elastic deformation, and wheel rotation come from this Rust engine. Ban fake buoyancy animation, rendering-only color blending advertised as mixing, and scripted wheel rotation. Emitters, drops, and stir inputs are scene controllers that call engine creation/forces, not JavaScript position animation.
- **D-10:** Prove Float or Sink, Jelly Drop, and Water Wheel early in the phase because they are the least certain compositions. Evolve the existing Dam Break basin into the named demo rather than replacing it with canned motion. Fountain and Color Mixer follow the same shared session/control contract.
- **D-11:** A failed visual experiment requires investigation or an explicit recorded scope decision. Do not silently substitute a different scene, keep a broken name with fake physics, or expand the public engine API solely to preserve a catchy title. Water Wheel may fall back to a simpler pinwheel-and-dropped-balls toy only through that explicit revision.

### Source, credits, and notices

- **D-12:** Each player view includes the current scene's implementation link (this repository's Rust scene module), inspiration link(s), and any applicable notice. Keep the Phase 17 footer for repository, MIT "free and open source" copy, Peter Ryszkiewicz/OpenLinks, and version/commit/build provenance.
- **D-13:** Implementation links point at this project's scene source, not upstream C++ as if it were the running code. Inspiration may cite the LiquidFun showcase, Faucet, particle guide, or other FEATURES.md references. An inspiration link does not replace a source notice. If a scene adapts upstream material, record the exact pinned revision/file and preserve notices via `THIRD_PARTY_NOTICES.md`.
- **D-14:** Copy must identify an experimental native Rust implementation. Do not claim complete LiquidFun parity, physically accurate pigment chemistry, or that catalog previews are live simulations.

### Design system and verification

- **D-15:** Keep semantic HTML plus the existing scoped dark CSS tokens. Do not adopt MysticUI, Tailwind, shadcn, or an icon package in this phase. Six static cards do not justify a design-system migration; the Phase 17 D-09 exception remains in force until the 2026-12-17 review date unless a later phase records a new override.
- **D-16:** Prove each scene locally: it opens from the catalog and hash URL, steps visible native state, reset restores the documented initial state, construction-control changes reset with labeled copy, credits/notices are reachable, and switching scenes disposes the prior world. Do not require the Phase 19 six-scene pointer/accessibility matrix or a new Linux qualification campaign.
- **D-17:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion

- Exact particle counts, basin/bowl/wheel dimensions, colors, camera bounds, and control enumerations within the locked names, bounded-resource, and honesty rules.
- Exact card layout, SVG preview artwork, credit-panel markup, and control labels within the locked dark semantic-HTML contract.
- Exact WASM session factory naming and module split, provided one world remains private and frames stay copied owned arrays.
- Whether Dam Break's existing dynamic circle remains the obstacle or is replaced by a documented equivalent rigid body, as long as reset restores it and the visitor can drop/reset it through a labeled control.

### Deferred Ideas (OUT OF SCOPE)

- Pointer/touch aiming, stirring, poking, stuck-pointer handling, and ordinary scrolling-outside-player acceptance — Phase 19 (`WEB-05`).
- Dark-default playful catalog accessibility, keyboard/focus/contrast acceptance at narrow widths, and concise text interaction instructions as a milestone gate — Phase 19 (`WEB-07`).
- Focused real-browser six-scene smoke plus production-subpath/live Pages gallery checks — Phase 19 (`WEBTEST-01`).
- MysticUI/Tailwind/shadcn adoption — not justified by six static cards; revisit on 2026-12-17 or in a later UI phase with a new override.
- Seed/control values in share links, scene editor, diagnostic-catalog gallery, WASM workers/threads/zero-copy, WebGPU, npm/crates.io publication — outside v1.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WEB-01 | Six-card catalog with names, descriptions, and previews; Open loads the shared player | Extend `web/src/catalog/scenes.ts` with descriptions/preview ids; replace `CatalogNav` with static SVG/CSS cards; keep `#/scene/{id}` |
| WEB-04 | Two or three labeled bounded controls per scene; say when a change resets | Catalog control metadata + WASM `apply_control`/`apply_action`; construction presets recreate; runtime jet/emission stay live |
| WEB-08 | Repository, per-scene implementation, inspiration, and notices; truthful experimental Rust claims | Keep `SiteFooter`; add player credit panel; blob URLs to this repo's scene modules; FEATURES inspiration URLs; `THIRD_PARTY_NOTICES.md` |
| DEMO-01 | Dam Break releases water into a basin with an interactive obstacle and repeatable reset | Evolve `crates/liquidfun-wasm/src/scene.rs` constants; add water-amount, gravity, obstacle drop/reset |
| DEMO-02 | Fountain aims/adjusts a bounded stream; lifetime/capacity keep population plateaued | `ParticleDef::with_lifetime` + `ParticleSystemDef::with_maximum_count` + `with_destruction_by_age`; emit in `advance` |
| DEMO-03 | Float or Sink drops body/density presets; native particle-body response, no fake buoyancy | Public fixture density + existing particle-body impulse coupling; prove light vs heavy y-separation natively first |
| DEMO-04 | Color Mixer stirs groups and shows contact-driven channel mixing, not canvas blending | `ParticleFlags::COLOR_MIXING` + `color_mixing_strength`; assert frame color lanes change; label as particle-color mixing |
| DEMO-05 | Jelly Drop drops/pokes an elastic shape against obstacles with bounded presets and reset | `ParticleGroupRecipe` + `ParticleFlags::ELASTIC` (+ optional `SPRING`); softness = recipe strength on reset; poke = labeled impulse |
| DEMO-06 | Water Wheel jet turns a pinned paddle wheel through native coupling and a joint; no scripted rotation | `RevoluteJointDef` with motor off; emit jet; prove angle change from particles; capture transformed paddle segments |
</phase_requirements>

## Summary

Phase 18 is scene content on a finished player bridge. Dam Break already constructs, steps, and draws from copied Rust frames. The work is to (1) turn `ProofSession::new()` into a checked six-id factory, (2) author five more persistent worlds plus Dam Break controls in unpublished `liquidfun-wasm`, (3) replace the honest name list with six static cards, and (4) add labeled presets/actions plus per-scene credits. Pointer polish and the complete six-scene browser smoke stay in Phase 19.

The public engine already has the needed constructors: water particles, lifetime/capacity eviction, contact-driven `COLOR_MIXING`, checked elastic group recipes, particle-body impulse coupling, and revolute joints. Diagnostic catalog recipes are two-particle inspect-and-destroy schedules and must not be imported. Frame limits already cap 512 particles, 16 segments, and 8 circles — keep every scene inside those bounds.

**Primary recommendation:** Spike Float or Sink, Jelly Drop, and Water Wheel as native `liquidfun-wasm` tests first; only then wire Fountain, Color Mixer, Dam Break controls, the six-card catalog, and the shared control/credit chrome.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory is present in this repository. Treat the following repo-local and managed standards as planner constraints with the same authority as locked CONTEXT decisions:

- Hobby scope in `AGENTS.md` / `PROJECT-SCOPE.md`: no Linux qualification campaign; local proofs plus truthful evidence. [VERIFIED: PROJECT-SCOPE.md]
- Independent AI review (2026-09-16): implementing agent must not approve its own work. [VERIFIED: AGENTS.md]
- Standing authorization (2026-09-13) covers iteration, tests, and ordinary main pushes; package release remains separately authorized. [VERIFIED: AGENTS.md]
- Semantic HTML plus one scoped CSS file through 2026-12-17 (`standards-overrides.md`). Do not adopt MysticUI/Tailwind/shadcn. [VERIFIED: standards-overrides.md]
- Functional core / imperative shell; parse at boundaries; `maybe_` naming; files over ~628 lines and functions over ~161 lines are refactor triggers. [VERIFIED: standards/core/architecture.md, standards/core/code-shape.md]
- Dark default, public source identity, truthful FOSS copy, Peter/OpenLinks, version/commit/build provenance. [VERIFIED: standards/core/frontend-ui.md]
- New Rust modules use `foo.rs` + `foo/`, not `foo/mod.rs`. [VERIFIED: standards/languages/rust.md]
- Unit-test pure/business logic in Arrange/Act/Assert. [VERIFIED: standards/core/testing.md]
- `.planning/**` is parser-owned GSD content; do not format it with mdformat. [VERIFIED: AGENTS.md]

## Standard Stack

No new production libraries. Reuse the Phase 16/17 toolchain and the public `liquidfun` API.

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `liquidfun` | workspace path | Physics constructors, stepping, views | Sole runtime engine; browser scenes depend outward only [VERIFIED: crates/liquidfun/src/lib.rs] |
| `liquidfun-wasm` | unpublished `cdylib`+`rlib` | Scene factory, controls, copied frames | Existing private wrapper; keep `liquidfun` as sole default member [VERIFIED: crates/liquidfun-wasm/Cargo.toml] |
| `wasm-bindgen` | `=0.2.128` | JS constructor/methods | Already pinned; constructor may take a scene-id `String` [CITED: wasm-bindgen.github.io constructor docs] |
| SolidJS | 1.9.15 | Catalog, player, controls | Existing `web/` app [VERIFIED: web/package.json] |
| Vite | 8.3.0 | Static `/liquidfun-rs/` build | Pages path already proven in Phase 17 [VERIFIED: web/package.json] |
| Bun | 1.4.2 | Install/scripts | Existing `packageManager` [VERIFIED: web/package.json] |
| Canvas 2D | browser API | Draw copied frames | Already draws particles, segments, circles [VERIFIED: web/src/render/canvas.ts] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| TypeScript | 7.0.2 | Catalog/control types | Existing typecheck [VERIFIED: web/package.json] |
| Vitest | 5.0.1 | Catalog, control, credit unit tests | Extend `web/tests/scenes.test.ts` [VERIFIED: web/package.json] |
| Playwright | 1.63.0 | Thin local open/reset/switch proofs | Extend `web/e2e/player.spec.ts`; do not become WEBTEST-01 [VERIFIED: web/package.json] |
| `wasm-pack` | 0.15.0 | Regenerated ignored package | `just web-wasm` before frontend verification [VERIFIED: local `wasm-pack --version`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static inline SVG previews | Live WASM thumbnail per card | Forbidden by D-02; multiplies sessions |
| Semantic HTML/CSS cards | MysticUI + Tailwind | Forbidden by D-15 until 2026-12-17 |
| Scene modules in `liquidfun` | Public engine scene crate | Violates isolation; scenes stay in unpublished wasm wrapper |
| Diagnostic protocol playback | Import `liquidfun-test-protocol` | Two-particle destroy schedules; D-07 forbids |
| Pointer stirring/poking | Canvas pointer events | Phase 19 WEB-05; use labeled buttons now |

**Installation:** none. Existing `just web-wasm`, `just web-build`, and `cargo test -p liquidfun-wasm` are the commands.

**Version verification:** `web/package.json` and `crates/liquidfun-wasm/Cargo.toml` inspected 2026-09-18. Local rustc 1.97.0, wasm-pack 0.15.0, Bun 1.4.2, `wasm32-unknown-unknown` installed.

## Architecture Patterns

### Recommended Project Structure

```
crates/liquidfun-wasm/src/
├── lib.rs                 # ProofSession factory + apply_control/apply_action
├── session.rs             # one World, advance, capture, scene controllers
├── frame.rs               # keep 512 / 16 / 8 copied-lane caps
├── scene.rs               # SceneId parse + shared basin helpers
└── scene/
    ├── dam_break.rs
    ├── fountain.rs
    ├── float_or_sink.rs
    ├── color_mixer.rs
    ├── jelly_drop.rs
    └── water_wheel.rs

web/src/
├── catalog/
│   ├── scenes.ts          # ids, titles, ready, descriptions, controls, credits
│   └── previews.tsx       # static inline SVG per id
├── components/
│   ├── CatalogNav.tsx     # six-card grid, hash Open only
│   ├── PlayerPanel.tsx    # generalized title/status/canvas + control slot
│   ├── SceneControls.tsx  # labeled presets/actions + reset copy
│   ├── SceneCredits.tsx   # implementation / inspiration / notices
│   └── SiteFooter.tsx     # unchanged site chrome
├── physics/
│   ├── loader.ts          # ProofSession.create(sceneId)
│   └── session.ts         # nextFrame + applyControl/applyAction + dispose
└── App.tsx                # any ready id; extract lifecycle if file exceeds ~628
```

Use `scene.rs` + `scene/` (not `scene/mod.rs`) per `standards/languages/rust.md`. [VERIFIED: standards/languages/rust.md]

### Pattern 1: Checked scene-id factory

**What:** One exported session. Construction parses a lowercase hyphenated id and builds that world. Unknown ids fail before `World::new()`.
**When to use:** Every load, reset, retry, and construction-preset change.
**Example:**

```rust
// Source: wasm-bindgen constructor docs + current ProofSession
#[wasm_bindgen(constructor)]
pub fn new(scene_id: String) -> Result<ProofSession, JsError> {
    SessionCore::create(parse_scene_id(&scene_id)?).map(|core| Self { core }).map_err(js_error)
}
```

Replace Dam Break-only `new()`. Update `loadProofSession` to `loadSceneSession(id)` and all native tests. Keeping a zero-arg constructor that still builds Dam Break would hide factory bugs. [CITED: https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/constructor.html]

### Pattern 2: Construction vs runtime controls

**What:** Catalog metadata declares whether a control recreates the world. Rust enforces the same split.
**When to use:** Every D-05 control.

| Control | Apply how | Recreates world? |
|---------|-----------|------------------|
| Dam Break water-amount, gravity | Rebuild scene with new constants | Yes |
| Dam Break obstacle drop/reset | `World::set_body_transform` / restore documented pose | No |
| Fountain emission-rate, launch-speed, aim-angle | Store on session; next emit uses them | No |
| Float or Sink body/density | Remember preset for the next drop | No |
| Float or Sink drop-body | `create_body` + density fixture | No |
| Color Mixer mix-strength | `ParticleSystemDef::with_color_mixing_strength` at construct | Yes — no live setter [VERIFIED: crates/liquidfun/src/world/particle_object/system.rs] |
| Color Mixer stir-speed | `World::apply_particle_force_range` each `advance` | No |
| Jelly shape / softness | Group recipe at construct/reset | Yes |
| Jelly drop/poke | New group or `apply_particle_linear_impulse_range` | No |
| Water Wheel jet-strength, emission on/off | Session emit policy | No |

UI copy before a recreating change: `Changing this setting recreates the scene from its documented initial state.`

### Pattern 3: Scene controllers inside `advance`

**What:** Emission, stir forces, and scheduled drops run in Rust during `SessionCore::advance`, then `World::step`. JavaScript never writes particle positions.
**When to use:** Fountain, Color Mixer stir, Water Wheel jet.
**Why:** D-09. The existing `advance(1..=4)` loop is the insertion point. [VERIFIED: crates/liquidfun-wasm/src/session.rs]

### Pattern 4: Generalized frame capture

**What:** `capture_frame` already supports variable particle counts and multiple circles/segments. Stop assuming one dynamic circle and three basin segments.
**When to use:** Every scene. Water Wheel emits transformed paddle segments via `BodySnapshot::transform().apply(local_point)`. [VERIFIED: crates/liquidfun/src/world/body.rs, crates/liquidfun/src/math/transform.rs]

Keep shared camera bounds `(-6,-1)..(6,8)` unless a spike proves a scene cannot fit. `web/src/render/camera.ts` is hardcoded to those bounds; changing per scene is optional discretion and costs extra tests. [VERIFIED: web/src/render/camera.ts]

### Pattern 5: Catalog metadata without sessions

**What:** `SCENES` gains `description`, `previewId`, `controls`, `credits`, and `ready: true` for every shipped id. Previews are Solid components returning inline SVG. Open is `<a href="#/scene/{id}">`.
**When to use:** WEB-01. Do not construct WASM from catalog code.

After all six ship, delete Ready / Not ready yet chips. Keep empty and unknown fallbacks; the not-ready fallback becomes unused for approved ids and may remain as defensive copy only. Update page summary and `web/tests/scenes.test.ts`, which currently assert only Dam Break is ready. [VERIFIED: web/src/catalog/scenes.ts, web/tests/scenes.test.ts]

### Anti-Patterns to Avoid

- **Live catalog thumbnails:** starts extra worlds; D-02 forbids.
- **Diagnostic recipe playback:** two particles then destroy; D-07 forbids.
- **JS position animation for float/mix/spin:** fake physics; D-09 forbids.
- **Revolute motor to spin the wheel:** scripted rotation; keep `enable_motor` false. [VERIFIED: crates/liquidfun/src/joint/definition/revolute_prismatic.rs]
- **Expanding `liquidfun` public API to save a title:** D-11 forbids.
- **Putting credits only in the footer:** D-12 requires per-scene links beside the player.
- **Claiming pigment chemistry or live previews:** D-14 forbids.
- **Growing `App.tsx` past the 628-line trigger:** extract lifecycle and control handlers. Current file is 486 lines. [VERIFIED: web/src/App.tsx]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Color mixing | Canvas blend / lerp in JS | `ParticleFlags::COLOR_MIXING` + system `color_mixing_strength` | Solver mixes channels only when both particles carry the flag [VERIFIED: crates/liquidfun/src/particle/solver/material.rs] |
| Elastic recovery | Custom springs in JS | `ParticleGroupRecipe` + `ELASTIC` / `SPRING` | Public group sampling and triad solver already exist [VERIFIED: crates/liquidfun/src/world/particle_object/group.rs] |
| Wheel hinge | Scripted `angle += dt` | `RevoluteJointDef` motor-off + particle-body impulses | Coupling already applies body impulses [VERIFIED: crates/liquidfun/src/world/particle_coupling/body_coupling.rs] |
| Fountain cap | JS particle list | `with_lifetime` + `with_maximum_count` + `with_destruction_by_age` | Default `destroy_by_age` is already true [VERIFIED: crates/liquidfun/src/particle/definition/system_definition.rs] |
| Buoyancy | Fake upward JS motion | Fixture density vs particle density + coupling | Engine applies contact impulses to dynamic bodies [VERIFIED: body_coupling.rs] |
| Hash routing | New router | `maybeParseSceneRoute` | Empty vs unknown already distinct [VERIFIED: web/src/routing/hash.ts] |
| Camera / y-invert | New projection | `createCamera` / `projectPoint` | Keep shared world box [VERIFIED: web/src/render/camera.ts] |
| Pages / WASM build | New workflow | Existing `just web-*` and Pages workflow | HOST-* complete in Phase 17 |
| Design system | Tailwind/MysticUI | Existing tokens in `web/src/app.css` | D-15 |

**Key insight:** The engine can already do these six toys. The risk is composition and honesty, not missing solvers. Do not add engine APIs or UI libraries to paper over a weak scene.

## Common Pitfalls

### Pitfall 1: Treating diagnostic recipes as demos

**What goes wrong:** Gallery plays two particles, inspects, destroys — empty page.
**Why it happens:** Protocol catalog is a capability checklist, not a visual scene. [VERIFIED: crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs]
**How to avoid:** Author persistent groups/emitters in `liquidfun-wasm`. Use protocol files as API reminders only.
**Warning signs:** Import of `liquidfun-test-protocol` or a two-particle create/destroy loop.

### Pitfall 2: Unbounded Fountain or jet emission

**What goes wrong:** Particle count climbs to the 512 frame cap, then `capture_frame` fails and poisons the session.
**Why it happens:** `FrameData` rejects more than 512 particles. [VERIFIED: crates/liquidfun-wasm/src/frame.rs]
**How to avoid:** Set `maximum_count` well below 512 (recommend 256–384), finite lifetimes, and `destroy_by_age(true)`. Native-test plateau: after N steps, count is ≤ max and not still strictly increasing.
**Warning signs:** `FrameCaptureFailed` after a long play; `particle_count` equals 512.

### Pitfall 3: Fake or over-promised physics

**What goes wrong:** Float does not bob, jelly shreds, wheel is motor-driven, mixer only overlaps translucent dots.
**Why it happens:** Names outrun composition; JS is the easy fake.
**How to avoid:** Native spike tests before UI. Color test must assert `particle_colors()` bytes change after contact. Wheel test must disable motor and show angle change only while the jet emits. Float test compares light vs heavy body `y` after the same steps.
**Warning signs:** `enable_motor(true)`, canvas color interpolation, `set_body_transform` every frame to spin the wheel.

### Pitfall 4: Leaking worlds when switching six scenes

**What goes wrong:** Old Dam Break keeps stepping under Fountain.
**Why it happens:** `App.tsx` still special-cases Dam Break (`isDamBreakRoute`, `startDamBreak`, `abandonDamBreak`). [VERIFIED: web/src/App.tsx]
**How to avoid:** Generalize to `isReadySceneRoute` / `startScene(id)` / `abandonScene`. Keep generation tokens, `dispose()`, and stale-load discard from Phase 17.
**Warning signs:** `data-scene` changes but previous particle colors remain; two WASM sessions constructed without `free()`.

### Pitfall 5: Construction controls applied live, or silent reset

**What goes wrong:** Gravity or mix-strength appears to change but does not, or the world resets with no copy.
**Why it happens:** `color_mixing_strength` and particle counts live on construction definitions; there is no public live particle-system-def setter. [VERIFIED: system.rs only exposes `set_particle_system_paused` among post-create system mutations]
**How to avoid:** Recreate on those presets; show the reset sentence before apply. Runtime emit/stir/jet/drop stay live.
**Warning signs:** Mix-strength `<select>` that only stores a JS variable.

### Pitfall 6: Implementation link points at C++ LiquidFun

**What goes wrong:** WEB-08 implies the running code is upstream C++.
**Why it happens:** Inspiration sources are Google URLs.
**How to avoid:** Implementation href is `https://github.com/bright-builds-llc/liquidfun-rs/blob/{sha}/{scene_path}` using the same SHA gate as `readBuildInfo` (`40` hex, host-locked). If SHA is missing, fall back to `.../blob/main/{scene_path}` or the repo root — never `google/liquidfun` as implementation. Inspiration is a separate list. [VERIFIED: web/src/build-info.ts]

### Pitfall 7: Catalog CSS reintroduces a design system

**What goes wrong:** Tailwind or icon package lands “just for cards.”
**Why it happens:** Six cards look like a component-library job.
**How to avoid:** CSS grid on existing tokens; inline SVG; no new dependency. Reuse 44px min block size for Open / Apply.
**Warning signs:** `package.json` gains `tailwindcss` or `@mystic-ui`.

## Code Examples

Verified patterns from this repository and official wasm-bindgen docs.

### Scene factory and control result

```rust
// Source: crates/liquidfun-wasm/src/lib.rs (evolve) + wasm-bindgen constructor docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SceneId {
    DamBreak,
    Fountain,
    FloatOrSink,
    ColorMixer,
    JellyDrop,
    WaterWheel,
}

pub(crate) fn parse_scene_id(raw: &str) -> Result<SceneId, SessionError> {
    match raw {
        "dam-break" => Ok(SceneId::DamBreak),
        "fountain" => Ok(SceneId::Fountain),
        "float-or-sink" => Ok(SceneId::FloatOrSink),
        "color-mixer" => Ok(SceneId::ColorMixer),
        "jelly-drop" => Ok(SceneId::JellyDrop),
        "water-wheel" => Ok(SceneId::WaterWheel),
        _ => Err(SessionError::UnknownScene),
    }
}

/// `true` means the caller already holds a new world (construction reset).
pub fn apply_control(&mut self, name: &str, value: &str) -> Result<bool, JsError> { /* ... */ }
pub fn apply_action(&mut self, name: &str) -> Result<(), JsError> { /* ... */ }
```

### Elastic jelly group (public API)

```rust
// Source: crates/liquidfun/src/particle/group/recipe.rs
//         crates/liquidfun/src/world/particle_object/group.rs
use liquidfun::collision::{CircleShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{ParticleColor, ParticleFlags};

let source = ParticleGroupSource::filled_shapes(vec![Shape::from(
    CircleShape::new(Vec2::ZERO, 1.2)?,
)])?;
let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
    .with_particle_flags(ParticleFlags::ELASTIC | ParticleFlags::SPRING)
    .with_strength(softness)?
    .with_color(ParticleColor::new(244, 114, 182, 255));
let group = world.create_particle_group(system, &recipe)?;
```

`ParticleGroupRecipe` is **not** re-exported from the crate root; import `liquidfun::particle::*`. [VERIFIED: crates/liquidfun/src/lib.rs, crates/liquidfun/src/particle.rs]

### Bounded fountain emit

```rust
// Source: crates/liquidfun/src/particle/definition/particle.rs
//         crates/liquidfun/src/particle/definition/system_definition.rs
let system_def = ParticleSystemDef::default()
    .with_radius(0.18)?
    .with_maximum_count(320)?
    .with_destruction_by_age(true);
let particle = ParticleDef::default()
    .with_flags(ParticleFlags::WATER)
    .with_position(nozzle)?
    .with_velocity(aim * launch_speed)?
    .with_lifetime(3.0)?
    .with_color(FOUNTAIN_COLOR);
let receipt = world.create_particle_with_def(system, None, &particle)?;
```

Treat a full system as a no-op emit, not a session poison, unless `destruction_occurrences` or construction errors say otherwise.

### Motor-off water wheel

```rust
// Source: crates/liquidfun/src/joint/definition/revolute_prismatic.rs
let joint = RevoluteJointDef::new(ground, wheel)?
    .with_frame(hub_local_ground, hub_local_wheel, 0.0)?;
world.create_joint(JointDef::from(joint))?;
// Do not call with_motor. Rotation must come from particle-body coupling.
```

Capture paddles:

```rust
let pose = world.body_snapshot(wheel)?.transform();
for [a, b] in PADDLE_LOCAL_SEGMENTS {
    let start = pose.apply(a);
    let end = pose.apply(b);
    // push into rigid_segments lane
}
```

[VERIFIED: crates/liquidfun/src/math/transform.rs]

### Color-mixing honesty test

```rust
// Source: crates/liquidfun/src/particle/solver/material.rs
// Mix only when both particles have COLOR_MIXING; strength 0 skips the pass.
let before = capture(&session).particle_colors();
session.advance(4)?;
let after = capture(&session).particle_colors();
assert_ne!(before, after); // contact-driven channel change
```

Default `color_mixing_strength` is `0.5`. Preset `Off` uses `0.0` and must keep colors stable. [VERIFIED: system_definition.rs Default, material.rs]

### Catalog control metadata

```ts
// Source: evolve web/src/catalog/scenes.ts
export type SceneControl =
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "preset";
      readonly recreates: boolean;
      readonly values: readonly { readonly id: string; readonly label: string }[];
    }
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "action";
      readonly recreates: false;
    };
```

Render presets as `<label>` + `<select>` (or a small radio group), actions as `<button type="button">`. Show reset copy only when `recreates` is true.

## Recommended First-Pass Enumerations

Discretion values. Stay under frame caps and reuse the current basin box.

| Scene | Preset values | Notes |
|-------|---------------|-------|
| Dam Break water-amount | Small 8×8=64; Medium 16×12=192 (current); Large 20×14=280 | Evolve, do not discard, documented constants in `scene.rs` |
| Dam Break gravity | Low `(0,-6)`; Normal `(0,-10)`; High `(0,-16)` | Recreates world |
| Dam Break obstacle | Keep current circle `(2.5, 5.5)` r=`0.75` | Drop = raise to `(2.5, 7.2)` and wake; Reset = documented pose |
| Fountain | Rate Off/Low/Med/High; speed 4/8/12; aim `-τ/8`, `0`, `+τ/8` from up | Lifetime 3s; max 320 |
| Float or Sink | Cork 0.3; Wood 0.6; Stone 2.0 vs particle density 1.0 | One drop action; pool of ~180 water particles |
| Color Mixer | Mix Off/Gentle 0.25/Strong 0.5; stir Off/Slow/Fast | Two colored `COLOR_MIXING` groups; stir = tangential force |
| Jelly Drop | Circle / square filled shapes; Soft 0.4 / Medium 1.0 / Firm 2.0 strength | `ELASTIC\|SPRING`; poke = labeled impulse on group members |
| Water Wheel | Jet Weak/Med/Strong; emission On/Off | 4 paddles as local segments; hub circle; motor off |

Colors: keep Dam Break teal `(57, 211, 199, 255)` as the water identity; give mixer two distinct hues (for example accent teal vs `#F87171` destructive red) so channel change is visible.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Honest six-name list, only Dam Break ready | Six ready cards with static previews | Phase 18 | WEB-01 |
| `ProofSession::new()` Dam Break only | Checked scene-id constructor | Phase 18 | Factory for all demos |
| Diagnostic two-particle recipes | Persistent authored scenes | 2026-09-17 research | Do not replay protocol catalog |
| MysticUI default for new Solid apps | Semantic HTML + scoped CSS through 2026-12-17 | Phase 17 D-09 / override | Do not migrate in this phase |
| Human-only independent review | Identified independent AI reviewer | 2026-09-16 | D-17 |

**Deprecated/outdated:**

- Phase 17 “Not ready yet” chips and Dam Break-only player copy — remove once all six construct.
- Phase 16 “Dispose session” / proof-page wording — already removed; do not revive.
- Importing desktop testbed or differential runner into the browser package — still forbidden.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Modest cork/wood/stone densities will show visible native float vs sink without a new buoyancy API | DEMO-03 | Need recorded scope decision or retune; do not fake motion |
| A2 | A small `ELASTIC\|SPRING` filled group stays coherent after a poke impulse at ≤512 particles | DEMO-05 | Reduce count/strength or record limitation; do not expand engine API |
| A3 | A motor-off revolute wheel with 4 paddles will rotate from a bounded jet | DEMO-06 | Only then consider the documented pinwheel-and-balls fallback |
| A4 | Shared camera bounds `(-6,-1)..(6,8)` fit all six toys | Architecture | Per-scene bounds become necessary; update camera tests |
| A5 | Fountain/jet emission and stir forces are safe to apply live without reset | Controls | If engine rejects mid-step create/force, mark those controls as recreating |

No other `[ASSUMED]` product or compliance claims. Engine API presence is verified from source.

## Open Questions

1. **Will Float or Sink, Jelly Drop, and Water Wheel look convincing at ≤384 particles?**
   - What we know: coupling, elastic groups, and revolute joints exist in source. [VERIFIED]
   - What's unclear: visual satisfaction in WASM at playground scale.
   - Recommendation: native spike plan first (D-10). If a scene fails, investigate or record an explicit revision — never silent substitution (D-11).

2. **Should implementation links pin the deployed SHA or `main`?**
   - What we know: footer already gates 40-char SHAs on `bright-builds-llc/liquidfun-rs`. [VERIFIED: web/src/build-info.ts]
   - What's unclear: local `vite` serve has no SHA.
   - Recommendation: SHA blob URL when `VITE_GIT_SHA` is valid; otherwise `blob/main/{path}`. Same host lock.

3. **Does Phase 18 Playwright need all six scenes in Chromium?**
   - What we know: D-16 requires local proof each scene opens, steps, resets, and disposes. WEBTEST-01 is Phase 19.
   - What's unclear: how much browser vs native evidence the verifier will accept.
   - Recommendation: native tests for physics honesty; one Playwright loop that opens each card/hash, asserts Playing + title + credits, resets, and switches scenes. No pointer matrix.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Scene native tests + wasm-pack | ✓ | 1.97.0 | — |
| `wasm32-unknown-unknown` | `just web-wasm` | ✓ | installed | `rustup target add` |
| wasm-pack | Generated bindings | ✓ | 0.15.0 | — |
| Bun | web unit tests / build | ✓ | 1.4.2 | — |
| just | Recipe facade | ✓ | 1.48.0 (stack pin 1.55.1) | Call `bun scripts/web-build.ts` directly |
| Playwright Chromium | Thin D-16 browser proofs | ✓ (existing `web` scripts) | 1.63.0 | Native proofs still required |
| C++ / Linux oracle | Not this phase | n/a | — | Do not invoke |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** just 1.48.0 vs documented 1.55.1 — recipes still run.

**Step 2.5 Runtime State Inventory:** SKIPPED (feature phase, not a rename/migration).

**Step 2.6:** External tools probed 2026-09-18 on the development Mac.

## Security Domain

`workflow.security_enforcement` is not set to `false` in `.planning/config.json` (absent = enabled).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Static playground; no accounts |
| V3 Session Management | no | In-memory WASM world only; dispose on leave |
| V4 Access Control | no | No privileged operations |
| V5 Input Validation | yes | Parse scene ids and control names/values at TS and Rust boundaries; reject unknown; no `innerHTML` for errors [VERIFIED: web/src/App.tsx already prefixes `Details:` in DEV only] |
| V6 Cryptography | no | No secrets or tokens in this phase |

### Known Threat Patterns for Rust/WASM playground

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unknown scene id / control name | Tampering | `parse_scene_id` / checked control enums; empty vs unknown fallback; no eval of hash |
| Oversized emission | Denial of service | `maximum_count` + lifetime + frame cap; poison+recreate on capture failure |
| WASM trap / panic | Denial of service | Recreate session; do not claim catch-unwind [VERIFIED: 16-CONTEXT D-07] |
| Open-redirect credits | Spoofing | Host-lock implementation/commit URLs like `readBuildInfo`; `rel="noopener noreferrer"` |
| XSS via error text | Tampering | Fixed copy + escaped DEV details; text nodes only |
| Fake-physics social claim | Spoofing | D-14 copy rules; implementation link is this repo |

Do not add workers, SharedArrayBuffer, or raw WASM memory views.

## Sources

### Primary (HIGH confidence)

- `crates/liquidfun-wasm/src/{lib,scene,session,frame}.rs` — Dam Break-only factory, reset constants, 512/16/8 frame caps
- `crates/liquidfun/src/lib.rs`, `particle.rs`, `particle/group/recipe.rs`, `particle/definition/{particle,system_definition}.rs`, `particle/solver/material.rs`, `world/particle_object/{group,system}.rs`, `world/particle_coupling/body_coupling.rs`, `joint/definition/revolute_prismatic.rs`, `world/body.rs`, `math/transform.rs` — public APIs used above
- `web/src/{catalog/scenes.ts,components/*,App.tsx,physics/*,routing/hash.ts,render/camera.ts,build-info.ts}` — Phase 17 player contract
- `web/tests/scenes.test.ts`, `web/e2e/player.spec.ts` — tests that must be generalized
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — locked decisions
- `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md` § Phase 18
- [wasm-bindgen constructor](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/constructor.html) — factory may take a `String`
- [LiquidFun showcase](https://google.github.io/liquidfun/) — inspiration only
- [Particle guide](https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html) — inspiration vocabulary
- Pinned Faucet inspiration: `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Faucet.h` [VERIFIED: THIRD_PARTY_NOTICES.md pin]
- `THIRD_PARTY_NOTICES.md`, `LICENSE` — notice and MIT FOSS copy
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/{architecture,code-shape,frontend-ui,testing,verification}.md`, `standards/languages/{rust,typescript-javascript}.md`

### Secondary (MEDIUM confidence)

- `.planning/research/v1.1/{FEATURES,ARCHITECTURE,STACK,PITFALLS,SUMMARY}.md` — 2026-09-17 recommendations; WASM/browser path now exists, so integration confidence is higher than that research recorded
- FEATURES suggested controls — now locked by D-05; enumerations remain discretion

### Tertiary (LOW confidence)

- Visual satisfaction of float / jelly / wheel at playground particle counts — requires the D-10 spike

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — versions and crates inspected in-repo; no new libraries
- Architecture: HIGH — factory, control split, module layout, and catalog/player seams are concrete
- Pitfalls: HIGH mechanisms / MEDIUM scene tuning — leak/cap/honesty traps are known; spike outcomes are not

**Research date:** 2026-09-18
**Valid until:** 2026-10-18 (stable engine APIs; re-check only if `liquidfun` public particle/joint surface changes)
