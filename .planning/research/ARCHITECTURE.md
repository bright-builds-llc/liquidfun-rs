# Architecture Research

**Domain:** LiquidFun playground scene integration (v1.3 Reference Testbed Scenes)
**Researched:** 2026-09-21
**Confidence:** HIGH for existing player/WASM/scene seams and public engine surfaces inspected in-tree; MEDIUM for whether Sparky needs a session-level step-hook change versus post-step contact observation, and for how recognizable Drawing Particles must be without a named `DestroyParticlesInShape` helper

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                 SolidJS playground (web/)                    │
│  catalog/scenes.ts  →  player  →  Canvas 2D draw of frames  │
├─────────────────────────────────────────────────────────────┤
│              liquidfun-wasm (private, publish=false)         │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐ │
│  │ ProofSession │→ │ SessionCore  │→ │ SceneHooks + World│ │
│  │ (wasm bind)  │  │ advance/ctl  │  │ scene/<name>.rs   │ │
│  └──────┬───────┘  └──────┬───────┘  └─────────┬─────────┘ │
│         │                 │                     │           │
│         └──── ProofFrame (copied typed arrays) ─┘           │
├─────────────────────────────────────────────────────────────┤
│              liquidfun (published, renderer-free)            │
│  World · bodies/fixtures · joints · particle system/groups  │
└─────────────────────────────────────────────────────────────┘
```

v1.3 does **not** introduce a second world type, a JS physics engine, or a new frame protocol. Each of the twelve upstream testbed scenes becomes one more allowlisted `SceneId` module that builds a native `World`, implements `SceneHooks`, and ships under the existing player.

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `crates/liquidfun` | Native physics only | Public world/body/joint/particle APIs already used by Dam Break, Jelly Drop, Water Wheel, Color Mixer, Fountain |
| `crates/liquidfun-wasm/src/scene.rs` | Allowlist + shared basin helpers | Extend `SceneId`, `parse_scene_id`, `build_scene`; keep `BuiltScene` / `SceneHooks` |
| `crates/liquidfun-wasm/src/scene/*.rs` | One scene builder + hooks | New modules mirroring `dam_break.rs` / `jelly_drop.rs` / `water_wheel.rs` |
| `crates/liquidfun-wasm/src/session.rs` | Own world, step 1–4, capture frame | Reuse as-is for most scenes; may need a pluggable `CollisionDecisionHook` for Sparky |
| `ProofSession` / `ProofFrame` | JS boundary | Keep constructor id, `applyControl`, `applyAction`, `pointerAction`, `advance`, `captureFrame` |
| `web/src/catalog/scenes.ts` | Titles, controls, credits, hints | Add twelve records; `SCENE_IDS` union grows |
| `web/src/player` + `render` | Load scene id, draw lanes | No physics; only consumes copied arrays |

## Recommended Project Structure

```
crates/liquidfun/                 # engine gains only scene-blocking APIs
crates/liquidfun-wasm/src/
├── scene.rs                      # MODIFY: SceneId + factory
├── session.rs                    # MODIFY only if Sparky needs step hooks
├── frame.rs                      # KEEP: five copied lanes
├── lib.rs                        # KEEP: ProofSession surface
└── scene/
    ├── dam_break.rs              # KEEP (existing six)
    ├── …                         # KEEP
    ├── particles.rs              # NEW
    ├── liquid_timer.rs           # NEW
    ├── surface_tension.rs        # NEW
    ├── elastic_particles.rs      # NEW
    ├── rigid_particles.rs        # NEW
    ├── impulse.rs                # NEW
    ├── wave_machine.rs           # NEW
    ├── soup.rs                   # NEW
    ├── soup_stirrer.rs           # NEW
    ├── theo_jansen.rs            # NEW (empty or unused particle system)
    ├── sparky.rs                 # NEW
    └── drawing_particles.rs      # NEW (last)
web/src/catalog/
├── scenes.ts                     # MODIFY: catalog + controls
└── previews.tsx                  # MODIFY: static SVG previews
```

### Structure Rationale

- **One module per scene:** Matches the v1.1 pattern (`SceneId` → `build(presets)` → `Box<dyn SceneHooks>`), keeps files under the existing size budget, and isolates controls/pointer behavior.
- **Engine stays renderer-free:** Scene layout, pointer mapping, and catalog copy stay in `liquidfun-wasm` / `web/`. Missing physics goes into `liquidfun` only when a listed scene cannot run without it.
- **Theo Jansen exception:** Upstream is particle-free. Keep `BuiltScene.particle_system` required: create a live system with zero particles so frame capture and the player remain uniform.

## Architectural Patterns

### Pattern 1: Allowlisted Scene Factory

**What:** Hyphenated JS id → `SceneId` → `build_scene` → `BuiltScene { world, particle_system, particle_radius, hooks }`.
**When to use:** Every new playground scene.
**Trade-offs:** Explicit allowlist prevents arbitrary scene loading; adding a scene touches Rust enum + TS catalog together.

**Example:**
```rust
// scene.rs — additive only
"wave-machine" => Ok(SceneId::WaveMachine),
SceneId::WaveMachine => wave_machine::build(presets),
```

### Pattern 2: SceneHooks for Per-Frame and Input Behavior

**What:** `on_advance`, `apply_control` (`Live` | `Recreated`), `apply_action`, `apply_pointer`, `collect_segments` / `collect_circles`.
**When to use:** Emission (Fountain/Water Wheel), motor updates (Wave Machine), stirrer force (Soup Stirrer), paint (Drawing Particles), VFX fade (Sparky).
**Trade-offs:** Scene-owned state stays in the hook struct; session never learns scene specifics. Recreate-on-preset already works via `ControlEffect::Recreated`.

### Pattern 3: Capability Clusters Share One Engine Seam

**What:** Group scenes that need the same particle flags, group flags, joints, or world operations; land the seam once, then port multiple scenes.
**When to use:** Roadmap phasing and avoiding thrashing public APIs.
**Trade-offs:** Slightly slower first scene in a cluster; much faster siblings.

## Data Flow

### Request Flow

```
Catalog click / hash route (scene id)
    ↓
ProofSession::new(id) → SessionCore::from_built → scene::build
    ↓
rAF loop: pointerAction / applyControl / applyAction
    ↓
SessionCore::advance → hooks.on_advance → World::step(NoDecisionHook*)
    ↓
capture_frame → ParticleSystemView + hooks.collect_* → ProofFrame
    ↓
Canvas draws positions/colors/segments/circles
```

\*Sparky may need a scene-owned `CollisionDecisionHook` instead of hardcoded `NoDecisionHook` (see Integration Points).

### State Management

```
presets: Vec<(name, value)>     # session-owned; recreate rebuilds world
hooks: Box<dyn SceneHooks>      # scene-owned live state (drag, motors, VFX)
world: World                    # single native authority
```

### Key Data Flows

1. **Preset recreate:** Control with `recreates: true` → `ControlEffect::Recreated` → `SessionCore` rebuilds from stored presets (Dam Break water/gravity pattern).
2. **Live control:** Stir speed, jet strength, motor toggle → mutate bodies/joints/particles without rebuild.
3. **Pointer:** CSS camera unproject → `pointer_action(kind, x, y)` → scene hook (drag, poke, paint, impulse aim).
4. **Frame:** Positions + optional colors + rigid segments/circles copied once per capture; no per-particle JS calls.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 6 → 18 scenes | Same session/player; catalog length and allowlist grow; keep one active session |
| Heavier scenes (Drawing Particles, Sparky VFX) | Stay under existing `MAX_FRAME_PARTICLES` (10240) and 1–4 steps/frame; tighten scene caps before changing the frame contract |
| Many concurrent visitors (Pages) | Still one world per browser tab; no server simulation |

### Scaling Priorities

1. **First bottleneck:** Particle count / step cost on main thread — reuse Fountain’s lifetime plateau and Dam Break’s cap patterns; do not raise the frame particle ceiling casually.
2. **Second bottleneck:** Catalog/UX noise — keep one gesture per scene and labeled presets; do not grow `ProofSession` method surface.

## Anti-Patterns

### Anti-Pattern 1: Second World or JS Physics Clone

**What people do:** Port LiquidFun JS testbed math into TypeScript “for the gallery.”
**Why it's wrong:** Violates the milestone (“running on this engine”), duplicates behavior, and drifts from `liquidfun`.
**Do this instead:** Native `World` in Rust scene modules; JS only loads WASM and draws frames.

### Anti-Pattern 2: Expanding ProofFrame for One Scene

**What people do:** Add new typed-array lanes for sparks, joints, or debug strings.
**Why it's wrong:** Forces player/renderer churn for twelve scenes that already fit positions/colors/segments/circles.
**Do this instead:** Encode sparks as ordinary particles with colors; encode machines as segments/circles from `collect_*`.

### Anti-Pattern 3: Broad Engine Rewrite Before First Scene

**What people do:** Re-architect particle groups because Drawing Particles is hard.
**Why it's wrong:** Most scenes already map to existing APIs (groups, elastic/spring, color mixing, revolute, lifetimes).
**Do this instead:** Ship low-dependency basins first; add the smallest missing helpers (`destroy particles in shape`, step-hook plug-in) only when blocked.

### Anti-Pattern 4: Claiming Exact C++ Parity

**What people do:** Treat visual sameness as sealed differential parity.
**Why it's wrong:** Milestone explicitly is not a sealed C++ parity claim.
**Do this instead:** “Recognizable port” in the existing player; document known simplifications (parameter UI as presets, not TestMain particle chrome).

## Integration Points

### Capability Clusters (scenes sharing one seam)

| Cluster | Scenes | Engine / WASM needs |
|---------|--------|---------------------|
| Basin + filled group + optional body | Particles, Liquid Timer | `create_particle_group` + polygon/circle sources; Liquid Timer adds `EdgeShape` fixtures (collision shapes already exist). Parameterized flags via presets (`WATER`, `COLOR_MIXING`, …) |
| Tensile / color mixing | Surface Tension | `ParticleFlags::TENSILE \| COLOR_MIXING`; solver passes already gated in `phase10-pass-graph-v1`. Color Mixer already proves mixing |
| Elastic / spring solid groups | Elastic Particles | `ELASTIC` / `SPRING` + `ParticleGroupFlags::SOLID`; Jelly Drop already constructs this |
| Rigid solid groups | Rigid Particles | `ParticleGroupFlags::RIGID \| SOLID`; rigid damping/projection passes already exist |
| Group impulse / force | Impulse | Public `apply_particle_force_range` / `apply_particle_linear_impulse_range` on contiguous `member_ids()` from `particle_group_view` — no new joint; pointer-up aims direction |
| Revolute motor drive | Wave Machine | `RevoluteJointDef` with motor + `set_revolute_motor_speed` / `set_revolute_motor_enabled` in `on_advance` (Water Wheel has revolute without motor) |
| Soup + destroy-under-shape | Soup | Dynamic bodies; **named `DestroyParticlesInShape` is absent** — approximate with `query_aabb_with_particles` + point-in-shape filter + `mark_particle_for_destruction`, or add a thin public helper. Edge debris uses `EdgeShape` + `set_body_mass_data` |
| Prismatic stirrer | Soup Stirrer | Extends Soup: `PrismaticJointDef`, `destroy_joint` toggle, `apply_body_force_to_center` in `on_advance` |
| Multi-joint walker | Theo Jansen | `DistanceJointDef` (frequency/damping) + motorized `RevoluteJointDef` + `FilterData::group_index(-1)`; empty particle system; keyboard/actions for motor speed |
| Contact-triggered VFX | Sparky | Rigid circle bodies + fixture tagging (`AssociationMap`) + explosion `create_particle_group` with lifetimes/colors. **Gap:** `SessionCore` always steps with `NoDecisionHook`. Need either (a) pluggable `CollisionDecisionHook` that records contact points for `on_advance` VFX, or (b) post-step contact observation via public diagnostics — prefer (a) to match upstream BeginContact timing |
| Pointer drawing | Drawing Particles | Paint on `pointer` move: destroy-in-shape, `ParticleGroupDestination::AppendTo` join, many flag/group combos, `REACTIVE` for wall/spring/elastic, `split_particle_group` when rigid groups gain zombies. Largest control surface |

### Engine: already exposed (reuse)

| Surface | Evidence in existing playground / crate |
|---------|----------------------------------------|
| `World`, bodies, polygon/circle fixtures, gravity | Dam Break, Float or Sink |
| `ParticleFlags` bit set including `WATER`, `WALL`, `SPRING`, `ELASTIC`, `VISCOUS`, `POWDER`, `TENSILE`, `COLOR_MIXING`, `BARRIER`, `REPULSIVE`, `ZOMBIE`, `REACTIVE`, … | `particle/definition.rs`; Color Mixer / Jelly Drop / Fountain |
| Solver passes for those flags + `SOLID` / `RIGID` groups | `particle/solver/manifest.rs` |
| `ParticleGroupRecipe` / `Source` / `Destination::New` / `AppendTo` | Color Mixer, Jelly Drop; unit tests for append |
| `create_particle_group`, `join_particle_groups`, `split_particle_group`, `set_particle_group_flags` | Public world particle-object API |
| Lifetimes + `destruction_by_age` | Fountain, Water Wheel |
| Per-particle / range force & impulse | `apply_particle_force_range`, `apply_particle_linear_impulse_range` |
| Revolute / prismatic / distance / mouse joints + destroy | Water Wheel revolute; joint module complete enough for Wave Machine / Soup Stirrer / Theo Jansen |
| `set_revolute_motor_*`, `FilterData` group index | Public joint/body APIs |
| `EdgeShape`, `ChainShape`, `set_body_mass_data`, `apply_body_force_to_center` | Collision + body control |
| `query_aabb_with_particles`, `mark_particle_for_destruction` | Query + lifecycle (enough to emulate destroy-in-shape) |
| `CollisionDecisionHook::observe` / `command` | Exists; not yet wired through WASM session |

### Engine / WASM: likely must gain or wire

| Gap | Needed by | Recommendation |
|-----|-----------|----------------|
| Convenience `destroy_particles_in_shape` (or documented scene helper) | Soup, Drawing Particles | Prefer a small public world helper if two scenes need identical semantics; else one shared private helper in `liquidfun-wasm` |
| Session step-hook plug-in | Sparky | Extend `SessionCore::advance` to use a scene-provided hook or a thin adapter owned by Sparky hooks — do not invent a second stepping path |
| Catalog + `SceneId` + smoke coverage | All twelve | Mechanical; follow Dam Break registration pattern |
| Optional empty particle system policy | Theo Jansen | Document “zero particles OK”; no `Option` in `BuiltScene` unless forced |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `liquidfun` ↔ `liquidfun-wasm` | Public Rust API only | No `differential-internals` in playground builds |
| WASM ↔ SolidJS | `ProofSession` + copied arrays | Do not expose `BodyId` / `ParticleId` as JS numbers |
| Scene ↔ Session | `SceneHooks` trait | Add methods only if Sparky/Drawing cannot fit `on_advance` + pointer |
| Catalog ↔ Player | `SceneId` string union | Keep hyphenated ids aligned with Rust parse table |

### Suggested Build Order

Dependency-aware order for roadmap phases (group by shared capability):

1. **Particles** then **Liquid Timer** — basin + group + edges; validates catalog/player scaling with almost no new engine surface.
2. **Surface Tension**, **Elastic Particles**, **Rigid Particles** — flag/group variants of Color Mixer / Jelly Drop patterns; prove tensile and rigid groups in WASM.
3. **Impulse** — group-wide force/impulse via existing range APIs + pointer.
4. **Wave Machine** — first motorized revolute + `on_advance` speed update (unlocks later motor scenes).
5. **Soup** then **Soup Stirrer** — land destroy-under-shape helper, then prismatic toggle + stirring force.
6. **Theo Jansen** — distance + revolute motors + collision filtering; empty particle system; pure rigid stress test for draw segments.
7. **Sparky** — contact-triggered particle bursts; requires session hook wiring; uses lifetimes/colors.
8. **Drawing Particles** last — depends on destroy-in-shape, append/join, split, and most particle/group flags; largest UX surface (map keyboard parameter matrix to presets/actions).

Keep the existing six scenes untouched except shared allowlist/catalog growth.

### New vs Modified (explicit)

| Kind | Items |
|------|-------|
| **New** | Twelve `scene/<name>.rs` modules (+ optional `*/tests.rs`); catalog entries; SVG previews; scene unit tests following Dam Break patterns |
| **Modified** | `scene.rs` (`SceneId`/parse/factory), `web/src/catalog/scenes.ts` (+ previews/links), smoke specs for open/reset/switch; possibly `session.rs` for Sparky hooks; possibly a small `liquidfun` destroy-in-shape helper |
| **Unchanged** | `ProofFrame` lane contract; published crate renderer-free rule; single `World` type; no JS solver |

## Sources

- In-repo WASM scene seam: `crates/liquidfun-wasm/src/scene.rs`, `session.rs`, `lib.rs`; existing scenes `dam_break.rs`, `jelly_drop.rs`, `color_mixer.rs`, `fountain.rs`, `water_wheel.rs`
- In-repo engine: `crates/liquidfun/src/particle/definition.rs` (flags), `particle/group/flags.rs`, `particle/solver/manifest.rs` (pass graph), `world/particle_object/system.rs` (force/impulse ranges), `world/joint.rs` + revolute/prismatic/distance modules, `world/step/hook.rs` (`CollisionDecisionHook`)
- Pinned upstream testbed (oracle commit `7f20402173fd143a3988c921bc384459c6a858f2`): `third_party/liquidfun/.../Testbed/Tests/{DrawingParticles,ElasticParticles,Impulse,LiquidTimer,Particles,RigidParticles,Soup,SoupStirrer,Sparky,ParticlesSurfaceTension,TheoJansen,WaveMachine}.h`
- Prior gallery architecture: `.planning/research/v1.1/ARCHITECTURE.md`
- Milestone scope: `.planning/PROJECT.md` (v1.3 Reference Testbed Scenes)

---
*Architecture research for: liquidfun-rs v1.3 playground scene expansion*
*Researched: 2026-09-21*
