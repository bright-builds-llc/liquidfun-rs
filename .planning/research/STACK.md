# Stack Research — v1.3 Reference Testbed Scenes

**Domain:** Porting twelve JavaScript LiquidFun testbed scenes into the existing Rust/WASM SolidJS playground
**Researched:** 2026-09-21
**Confidence:** HIGH for “no new physics/frontend framework”; HIGH for upstream scene API needs (pinned JS + C++ sources); MEDIUM for exact WASM frame-cap numbers until each scene’s draw list is counted in implementation

This file covers **v1.3 stack additions for the twelve new playground scenes only**. Do not re-derive the v1.0 Cargo foundation or the v1.1 SolidJS/WASM gallery baseline. Keep Rust 1.97.0, Edition 2024, publishable `liquidfun` free of C++/wasm-pack/renderer deps, private `liquidfun-wasm`, SolidJS/Vite in `web/`, pinned upstream `7f20402173fd143a3988c921bc384459c6a858f2`, hobby scope (no crate publish, no public benchmark, no default SIMD/Rayon, `unsafe_code = "forbid"`). Dam Break already exists and stays.

## Executive Recommendation

**Do not add a new production crate dependency, a second physics engine, glam, Rayon/SIMD defaults, or a new frontend framework.** Prefer capabilities already inside `liquidfun` and the existing `liquidfun-wasm` + SolidJS player.

The twelve scenes need **engine scene modules and a few adapter/API seams**, not stack replacement:

1. Keep `liquidfun` on `bitflags` 2.13.0 only (workspace also pins `thiserror` 2.0.18 for tooling; do not pull either into the published crate solely for these demos).
1. Keep private `liquidfun-wasm` on `wasm-bindgen = "=0.2.128"` and the existing `SceneHooks` / `ProofSession` / copied typed-array frame path.
1. Keep `web/` on SolidJS 1.9.15, Vite 8.3.0, `@kobalte/core` 0.13.x, Bun 1.4.2, Playwright 1.63.0 — catalog and control registration only.
1. Implement missing **behavior** in `liquidfun` (shape-scoped particle destroy helper if needed) and missing **playground plumbing** in `liquidfun-wasm` / `web/` (scene ids, raised rigid draw caps, optional per-scene step hook for Sparky).

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `liquidfun` (workspace) | path crate, current HEAD | Sole physics runtime for every new scene | Already exposes the particle flags, group flags, joints, forces/impulses, split, filters, and shapes these demos need; adding another engine would violate package isolation and parity goals |
| `liquidfun-wasm` | private `publish = false` | Scene factories, stepping, pointer/control bridge | Existing allowlisted `SceneId` + `SceneHooks` pattern is how Dam Break / Jelly Drop / Water Wheel already ship |
| `wasm-bindgen` | `=0.2.128` | Rust↔JS session API | Current crates.io 0.2 line (published 2026-09-04); already pinned; no reason to churn for scene ports |
| SolidJS | 1.9.15 | Catalog, controls, player shell | Current registry release; matches v1.1 playground; no UI rewrite |
| Vite | 8.3.0 | Static Pages build | Current registry release; already wired to wasm-pack output |
| Canvas 2D | browser built-in | Draw copied particle/rigid lanes | Existing five-lane frame export; escalate only if a scene proves Canvas 2D insufficient |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `bitflags` | 2.13.0 | `ParticleFlags` / group flag bit patterns | Already in `liquidfun`; reuse for every flag combo below — do not invent parallel enums |
| `@kobalte/core` | 0.13.12 (workspace); 0.13.14 current on npm | Accessible dialog/shell | Keep unless a catalog change forces a lockfile update; bump is optional polish, not a scene requirement |
| Playwright | 1.63.0 | Chromium `just web-player-smoke` | Extend smoke coverage per new scene; do not add a second E2E runner |
| Upstream JS tests | pinned submodule paths under `lfjs/testbed/tests/` | Behavioral/layout oracle for ports | Read these first when implementing each scene |
| Upstream C++ Testbed headers | `Testbed/Tests/*.h` | Flag/joint/callback clarity when JS is thin | Prefer C++ when naming solver flags, joints, or listeners |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| Existing `wasm-pack` + `just` web recipes | Rebuild private WASM package | No new packaging tool |
| `just web-player-smoke` | Chromium regression | Grow allowlists with new scene ids |
| Pinned C++ oracle (optional manual) | Differential spot-check of new scene recipes | Not a playground dependency; never link into `liquidfun` |

## Scene → Capability Map (prefer in-crate APIs)

Upstream JS sources live under `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/`. Matching C++ headers are under `.../Testbed/Tests/` (Surface Tension C++ file is `ParticlesSurfaceTension.h`).

| Scene | JS / C++ sources | Particle / group needs | Rigid / joint / callback needs | Stack implication |
|-------|------------------|------------------------|--------------------------------|-------------------|
| Drawing Particles | `testDrawingParticles.js` / `DrawingParticles.h` | Many flag presets: elastic, powder, spring, tensile, viscous, wall, barrier(+wall/elastic/spring), wall+repulsive, color-mixing, zombie; often `| reactive` when painting wall/spring/elastic; group `solid` / `rigid|solid`; `CreateParticleGroup` while dragging; `DestroyParticlesInShape`; `SplitParticleGroup` on rigid+zombie; `ParticleGroupDestroyed` | Pointer paint; no new joint | **Engine:** reuse flags + `split_particle_group`; add or compose shape-scoped destroy. **WASM:** paint via `pointer_action`; track last group via lifecycle/report. **No new crate** |
| Elastic Particles | `testElasticParticles.js` / `ElasticParticles.h` | `spring` + `solid` circle; two `elastic` + `solid` groups; colors | One dynamic body | Already proven by Jelly Drop (`ELASTIC` / strength). Scene module only |
| Impulse | `testImpulse.js` / `Impulse.h` | Default water group; optional color-mixing coloring | Pointer direction → `ApplyLinearImpulse` or `ApplyForce` on group | Use `apply_particle_linear_impulse_range` / `apply_particle_force_range` over group members (Jelly Drop poke pattern). Scene controls for impulse vs force |
| Liquid Timer | `testLiquidTimer.js` / `LiquidTimer.h` | Default `tensile \| viscous` (C++ particle-parameter table) | Static basin geometry | Flags + strengths already on `ParticleSystemDef`. Scene module only |
| Particles | `testParticles.js` / `Particles.h` | Parameterized flags + optional `ColorParticleGroup` | One dynamic body | Scene module; color stripes are scene-local buffer writes, not a new library |
| Rigid Particles | `testRigidParticles.js` / `RigidParticles.h` | `groupFlags = rigid \| solid` (three colored groups) | One dynamic body | `ParticleGroupFlags::RIGID \| SOLID` already solved. Scene module only |
| Soup | `testSoup.js` / `Soup.h` | Water (params exclude wall/barrier); `DestroyParticlesInShape` under dropped bodies | Several dynamic boxes/circles | Shape-scoped destroy helper + body creation. No new joint crate |
| Soup Stirrer | `testSoupStirrer.js` / `SoupStirrer.h` | Extends Soup; destroy under stirrer | `b2PrismaticJoint` ground↔stirrer; toggle destroy/create joint; `ApplyForceToCenter` while in soup | `PrismaticJointDef` already public; `apply_body_force_to_center` exists. Scene module |
| Sparky | `testSparky.js` / `Sparky.h` | Powder VFX groups; fade colors; `DestroyParticles` | Six dynamic circles; **`BeginContact`** spawns VFX; timed fade | **Do not** add a contact-listener crate. Use step report `ContactTransitionKind::Begin` (or `CollisionDecisionHook::observe` + post-step queue) inside `SceneHooks::on_advance`. `WorldCommand` today is destroy-only — spawn VFX **after** unlock, not via expanding FFI. Raise circle/segment frame caps |
| Surface Tension | `testSurfaceTension.js` / `ParticlesSurfaceTension.h` | `tensile \| colorMixing` × three colors | One dynamic body | Color Mixer + tensile flags already exist. Scene module |
| Theo Jansen | `testTheoJansen.js` / `TheoJansen.h` | JS adds a water particle slab above the walker (C++ header is rigid-only — **follow JS** for playground parity) | Chassis/wheel `groupIndex = -1`; motorized `b2RevoluteJoint`; many `b2DistanceJoint` + revolute legs; 40 balls; keyboard motor controls | `FilterData::group_index`, `RevoluteJointDef` + `set_revolute_motor_*`, `DistanceJointDef` already public. **Raise** `MAX_RIGID_CIRCLES` / `MAX_RIGID_SEGMENTS` so balls + legs fit the copied frame |
| Wave Machine | `testWaveMachine.js` / `WaveMachine.h` | Water box group; optional color-mixing | Motorized revolute joint; `SetMotorSpeed(0.05 * cos(t) * π)` each step | Same motor APIs as Water Wheel (motor off there; enable + update speed here). Scene `on_advance` only |

### Already-proven playground precedents

| Capability | Existing scene | Reuse for |
|------------|----------------|-----------|
| Water group + basin | Dam Break | Particles, Soup, Wave Machine, Liquid Timer |
| `COLOR_MIXING` | Color Mixer | Drawing (color-mixing mode), Surface Tension, optional Impulse/Particles coloring |
| `ELASTIC` / soft strengths | Jelly Drop | Elastic Particles, Drawing elastic modes |
| Revolute hub | Water Wheel | Wave Machine, Theo Jansen motor |
| Contiguous group impulse | Jelly Drop poke | Impulse |
| Pointer world coords | Shared player | Drawing paint, Impulse flick, Soup Stirrer toggle |

## Installation

No new packages are required for the milestone’s physics or UI stack. Scene work stays in-repo:

```bash
# unchanged private WASM package (existing recipe)
# wasm-pack build crates/liquidfun-wasm --target web --release

# unchanged web app (existing lockfile)
cd web && bun install --frozen-lockfile

# optional polish only if touching package.json anyway
# bun add @kobalte/core@0.13.14
```

Engine/WASM code changes (not installs):

- Add twelve `crates/liquidfun-wasm/src/scene/*.rs` modules + `SceneId` / catalog entries in `web/src/catalog/scenes.ts`.
- Prefer a thin `liquidfun` helper approximating `b2ParticleSystem::DestroyParticlesInShape` (AABB query + `Shape::test_point` + mark/destroy) over a third-party geometry crate.
- Raise `MAX_RIGID_SEGMENTS` / `MAX_RIGID_CIRCLES` in `crates/liquidfun-wasm/src/frame.rs` for Theo Jansen and Sparky (current caps 16 / 8 are below JS ball/VFX body counts).
- For Sparky, plumb contact-begin observation through the existing step report / hook model; keep `NoDecisionHook` for scenes that do not need it.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Native `liquidfun` scenes in `liquidfun-wasm` | Embed or transpile upstream `lfjs` | Never for this milestone — would ship a second physics runtime and break “native Rust” playground claims |
| Existing SolidJS + Canvas 2D player | New frontend framework or WebGL/WebGPU | Only if a measured render bottleneck appears after ports; none of the twelve scenes require it at research time |
| `query_particle_system_aabb` + `test_point` + destroy | Add `glam` / another math crate for shape tests | Never — shapes already expose `test_point` |
| Post-step VFX spawn in `SceneHooks` | Expand `WorldCommand` to create particle groups mid-step | Prefer post-step first; widen deferred commands only if Sparky cannot meet interaction goals without it |
| `PrismaticJointDef` / `DistanceJointDef` / revolute motors in-crate | Reimplement joints in JS or add Rapier/nphysics | Never — joints already live in `liquidfun` |
| Existing Pointer Events → `pointer_action` | `MouseJoint` for every interactive scene | Prefer pointer hooks (Drawing/Impulse/Stirrer). Reserve `MouseJointDef` only if a scene truly needs spring-drag toward the cursor |
| Keep Catalog + Kobalte shell | MysticUI / Tailwind redesign | Out of scope for v1.3 scene ports |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `glam`, `nalgebra`, or another math crate | Operation grouping/API drift vs LiquidFun `f32` oracle; project already forbids casual adoption | `liquidfun::math` + collision shapes |
| Rayon / default SIMD / relaxing `unsafe_code = "forbid"` | Explicitly out of v1.3; hobby determinism baseline | Scalar kernels already in crate |
| Second physics engine (Rapier, Box2D-rs, Matter.js, lfjs in browser) | Breaks native-Rust playground story and package isolation | `liquidfun` only |
| New frontend framework (React, Svelte, SolidStart SSR) | Rewrites a working Pages player for no scene benefit | Extend SolidJS catalog |
| Adding C++, wasm-pack, or renderer deps to publishable `liquidfun` | Violates packaging constraint | Keep bridge private in `liquidfun-wasm` |
| Mid-step mutable world listeners modeled on raw C++ `b2ContactListener` with unrestricted mutation | Rust hooks intentionally forbid mid-step world mutation | Observe + deferred/post-step scene logic |
| Publishing these scenes as a new crates.io feature surface | Milestone is playground ports, not a release | Private WASM scenes + Pages |
| Public benchmark or Dam Break pair gate for each new scene | Out of scope | Optional manual oracle spot-check only |

## Stack Patterns by Variant

**If the scene is particle-flag driven (Elastic, Rigid, Surface Tension, Liquid Timer, Particles):**

- Build with `ParticleGroupRecipe` + existing flags/group flags.
- Register catalog presets that recreate the world (same pattern as Jelly Drop softness/shape).

**If the scene paints or clears particles under shapes (Drawing Particles, Soup, Soup Stirrer):**

- Implement destroy-in-shape once in `liquidfun` or a private wasm helper shared by those scenes.
- Do not pull JS geometry libraries.

**If the scene is joint-motor driven (Wave Machine, Theo Jansen, Soup Stirrer):**

- Construct joints with existing `*JointDef` types; mutate motors with `set_revolute_motor_speed` / enable APIs each `on_advance`.
- Draw with raised rigid lane caps; Theo Jansen should emit many circles (balls) plus leg segments.

**If the scene reacts to rigid contacts (Sparky):**

- Detect `ContactTransitionKind::Begin` after step (or hook observe without world mutation), then `create_particle_group` with `POWDER` and fade colors in `on_advance`.
- Keep VFX particle counts inside `MAX_PARTICLE_COUNT` (10240).

**If a C++ header omits particles but JS adds them (Theo Jansen):**

- Follow the **JavaScript** testbed file for playground recognition; cite both sources in scene credits.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `liquidfun` path + `bitflags` 2.13.0 | `liquidfun-wasm` | No new liquidfun production deps required for these scenes |
| `wasm-bindgen` 0.2.128 | Rust 1.97.0 / wasm32-unknown-unknown | Keep exact pin; regenerate package after scene API exports |
| SolidJS 1.9.15 | Vite 8.3.0 + `vite-plugin-solid` 2.11.14 | Already integrated |
| `@kobalte/core` 0.13.12 | Solid 1.9.x | 0.13.14 available; optional bump |
| Frame caps today (16 segments / 8 circles / 10240 particles) | Theo Jansen (40 balls) / Sparky (6 circles + VFX) | **Incompatible until caps raised** — adapter change, not a new library |
| `WorldCommand::{DestroyBody, DestroyFixture}` | Sparky VFX spawn | Insufficient for create-group; use post-step scene logic |

## Integration Points (existing workspace)

| Layer | Change for v1.3 | Must not change |
|-------|-----------------|-----------------|
| `crates/liquidfun` | Optional destroy-in-shape helper; only if scenes cannot compose queries cleanly | Public dep graph; no glam/rayon/renderer |
| `crates/liquidfun-wasm` | Twelve scene modules; `SceneId`; frame caps; Sparky contact observation; motor updates | Remains `publish = false`; no C++ |
| `web/src/catalog/scenes.ts` | Twelve ready entries, controls, credits linking JS+C++ inspiration | Solid/Vite/Kobalte stack |
| CI / Pages | Same OIDC Pages pipeline; smoke tests grow | No new Linux qualification gate |

## Sources

- Pinned upstream JS tests: `third_party/liquidfun/liquidfun/Box2D/lfjs/testbed/tests/test{DrawingParticles,ElasticParticles,Impulse,LiquidTimer,Particles,RigidParticles,Soup,SoupStirrer,Sparky,SurfaceTension,TheoJansen,WaveMachine}.js` — HIGH
- Pinned upstream C++ headers: `.../Testbed/Tests/{DrawingParticles,ElasticParticles,Impulse,LiquidTimer,Particles,RigidParticles,Soup,SoupStirrer,Sparky,ParticlesSurfaceTension,TheoJansen,WaveMachine}.h` — HIGH
- Official particle flag reference: [b2Particle.h](https://google.github.io/liquidfun/API-Ref/html/b2_particle_8h.html) and [Particle Module guide](https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html) — HIGH
- Official JS testbed catalog: [google.github.io/liquidfun/testbed](http://google.github.io/liquidfun/testbed/index.html) — HIGH
- Local crate evidence: `crates/liquidfun` particle flags/solver gates, joint defs (`Revolute`/`Prismatic`/`Distance`), force/impulse range APIs, `split_particle_group`, `CollisionDecisionHook`, `query_particle_system_aabb`, shape `test_point`; `crates/liquidfun-wasm` frame caps and `NoDecisionHook` session stepping; `web/package.json` pins — HIGH
- crates.io `wasm-bindgen` 0.2.128 (2026-09-04); npm `solid-js` 1.9.15, `vite` 8.3.0, `@kobalte/core` 0.13.14 — HIGH version observation
- Destroy-in-shape composition vs first-class API exact shape — MEDIUM until implementation chooses public helper vs private wasm helper

---
*Stack research for: v1.3 Reference Testbed Scenes*
*Researched: 2026-09-21*
