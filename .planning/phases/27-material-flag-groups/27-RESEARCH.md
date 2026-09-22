# Phase 27: Material flag groups - Research

**Researched:** 2026-09-22
**Domain:** Native Rust WASM scene ports (Surface Tension, Elastic Particles, Rigid Particles) + SolidJS catalog append
**Confidence:** HIGH

## Summary

Phase 27 appends three watch-first material showcase scenes after the current eight catalog ids. All three layouts are fully specified by pinned lfjs + C++ sources at commit `7f20402173fd143a3988c921bc384459c6a858f2`, and the public `liquidfun` API already exposes every flag, group flag, recipe field, and body primitive those scenes need. `[VERIFIED: third_party/liquidfun HEAD + crates/liquidfun particle/group APIs + solver manifest]`

There is **no evidenced public-API gap** that blocks MAT-01, MAT-02, or MAT-03. Tensile, color-mixing, elastic, spring, solid, and rigid solver passes already exist; Color Mixer and Jelly Drop / Liquid Timer already exercise color mixing, elastic/spring, and tensile paths in the WASM shell. Rigid `RIGID | SOLID` groups are first-time gallery consumers of an already-tested engine path. `[VERIFIED: crates/liquidfun/src/particle/solver/manifest.rs + jelly_drop.rs + color_mixer.rs + liquid_timer.rs + rigid solver unit tests]`

**Primary recommendation:** Implement three scene-only modules in `liquidfun-wasm` (optional private shared basin helper), wire `SceneId` / `SCENE_IDS` / previews / credits / capture plans / smoke together, and assert flag recipes in headless tests. Do not open engine work unless a wired scene fails the required visual contrast after correct flags are set.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
#### Catalog shell
- **D-01:** Append Surface Tension (`surface-tension`), Elastic Particles (`elastic-particles`), and Rigid Particles (`rigid-particles`) after the current eight scene ids. Keep the existing order first. Hash routes stay `#/scene/{id}`.
- **D-02:** Each new entry is `ready: true` once its native scene runs, with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Do not start a WASM world per card, capture live thumbnails, or animate fake physics.
- **D-03:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, filters, or a separate catalog page. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.

#### Watch-first controls
- **D-04:** Surface Tension, Elastic Particles, and Rigid Particles expose play, pause, and Reset only. No particle-type, stiffness, or color pickers in this phase. PRESET-01 stays future work.
- **D-05:** Interaction copy says these scenes are watch-first. Do not add pointer drag, click impulse, or keyboard material modes.
- **D-06:** Reset disposes the session and recreates the documented initial layout.

#### Recognizable flag-group layouts
- **D-07:** Surface Tension must show a basin, three tensile and color-mixing groups (red circle, green circle, blue box), and one falling dynamic ball. The groups bead, and color bleeds when the ball hits them. Match `testSurfaceTension.js` / `ParticlesSurfaceTension.h` closely enough to be recognizable. Extra particle types are chrome.
- **D-08:** Elastic Particles must show a basin, a red spring-and-solid circle, a green elastic-and-solid circle, a blue elastic-and-solid box with angle and spin, and one falling dynamic circle. The three soft clumps deform when the ball hits them. Match `testElasticParticles.js` / `ElasticParticles.h` closely enough to be recognizable. Exact recovery stiffness and upstream JS bugs are chrome; recognizable soft motion beats bit-exact recovery.
- **D-09:** Rigid Particles must use the same basin family as Elastic Particles, with three colored rigid-and-solid groups (circles plus a spinning box) and one falling ball. The clumps stay solid and do not stretch like jelly. Match `testRigidParticles.js` / `RigidParticles.h` closely enough to be recognizable. Bit-perfect rigid-solver parity is chrome.

#### Pinned-test credits
- **D-10:** Keep the Phase 18 credit chrome: implementation link to this repository's scene module, inspiration links, and third-party notices. Implementation links stay host-locked to this repo and must not point at `google/liquidfun` as the running code.
- **D-11:** Inspiration for Surface Tension cites pinned `testSurfaceTension.js` and `ParticlesSurfaceTension.h` at commit `7f20402173fd143a3988c921bc384459c6a858f2`. Elastic Particles cites `testElasticParticles.js` and `ElasticParticles.h`. Rigid Particles cites `testRigidParticles.js` and `RigidParticles.h`. Same pin. Preserve notices via `THIRD_PARTY_NOTICES.md` when upstream material is adapted.
- **D-12:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations.

#### Engine additions
- **D-13:** Author all three scenes in `liquidfun-wasm` on the public `liquidfun` API. Prefer existing `TENSILE`, `COLOR_MIXING`, `ELASTIC`, `SPRING`, `SOLID`, and `RIGID` paths. Color Mixer already proves color mixing; Jelly Drop already proves elastic groups. Add engine behavior only when a listed scene cannot show the required contrast without it.
- **D-14:** Do not cut particle counts or raise the 4-step catch-up cap to look smoother. Do not import the desktop testbed or replay diagnostic protocol recipes as gallery demos.

#### Shared session and proof
- **D-15:** Grow the existing checked scene-id factory. Keep one WASM world, copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch. `MAX_ADVANCE_STEPS` stays 4.
- **D-16:** Prove locally with the existing Chromium `just web-player-smoke` suite extended so the three new scenes open, play, pause, and reset, and so the current eight scenes still open and run. Do not add Firefox/Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-17:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact basin coordinates, particle radius, group sizes, box angle, and particle counts, as long as the visitor can recognize the pinned tests and counts are not reduced to fake smoothness.
- Static SVG preview artwork, as long as each is captioned `Static preview` and does not imply a live simulation.
- A private shared basin helper inside `liquidfun-wasm` when it prevents the three layouts from drifting, without a new public engine API.
- Whether a missing tensile, color-bleed, elastic, or rigid behavior needs a narrowly justified engine addition when the scene cannot show the required contrast without it.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.

### Deferred Ideas (OUT OF SCOPE)
- Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen — Phase 28.
- Sparky, Drawing Particles, and the full twelve-scene catalog claim (PLAY-01) — Phase 29.
- Material and particle-type pickers (PRESET-01) and the Drawing Particles keyboard matrix (DRAW-02).
- Sealed per-scene differential evidence (PARITY-01) and commented-out Box2D-only tests (BOX2D-01).
- Optional pointer nudge on watch-first scenes. Not required for recognizability.
- Cross-links between related scenes (Surface Tension and Color Mixer, Elastic Particles and Jelly Drop).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MAT-01 | Visitor can watch Surface Tension: three colored tensile groups bead and bleed color when a ball hits them. | Port vertical-wall basin + three `TENSILE \| COLOR_MIXING` groups + dynamic ball from pinned JS/C++; Color Mixer proves mixing; Liquid Timer proves tensile; set `with_damping(0.2)` per JS. |
| MAT-02 | Visitor can watch Elastic Particles: three soft clumps deform when a ball falls on them. | Port shared basin + red `SPRING`+`SOLID`, green `ELASTIC`+`SOLID`, blue `ELASTIC`+`SOLID` spinning box + ball; Jelly Drop proves elastic/spring particle flags; use `with_group_flags(SOLID)`, `with_transform` angle, `with_angular_velocity(2.0)`. |
| MAT-03 | Visitor can watch Rigid Particles: three colored clumps stay solid and do not stretch like jelly when a ball hits them. | Same basin family as Elastic; three groups with `ParticleGroupFlags::RIGID \| SOLID` and default `WATER` particle flags; rigid damping/projection passes already gated in the solver manifest. |
</phase_requirements>

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. `[VERIFIED: Glob .cursor/rules]`

Apply repo-local and Bright Builds standards instead:

- Hobby scope / experimental API / no package publication from this phase (`PROJECT-SCOPE.md`, `AGENTS.md`).
- Scene construction in WASM shell; physics in `liquidfun`; renderer-free engine (`standards/core/architecture.md`).
- Safe Rust, no `unwrap` in production paths, `foo.rs` + `foo/` module shape (`standards/languages/rust.md`).
- SolidJS + Bun web tooling; Kobalte Dialog override for shell (`standards/languages/typescript-javascript.md`, `standards-overrides.md`).
- Local checks before commit; Chromium `just web-player-smoke` is the browser gate (`standards/core/verification.md`).
- Do not format `.planning/**` with mdformat (`AGENTS.md` Repo-Local Guidance).

## Capability Verdict (API vs engine gap)

| Scene | Buildable on public API today? | Exact flags / recipe | Engine gap? |
|-------|--------------------------------|----------------------|-------------|
| Surface Tension | **Yes** | Particle `TENSILE \| COLOR_MIXING` on all three groups; no group flags; system `radius=0.035`, `damping=0.2` (JS); colors RGB; dynamic circle density `0.5` at `(0,8)` | **None evidenced.** Tensile + color-mixing passes exist; Liquid Timer / Color Mixer already use those flags in WASM. |
| Elastic Particles | **Yes** | Red: particle `SPRING` + group `SOLID`; green: particle `ELASTIC` + group `SOLID`; blue: particle `ELASTIC` + group `SOLID`, transform angle `-0.5`, angular velocity `2.0`; radius `0.035` | **None evidenced.** Jelly Drop already creates `ELASTIC \| SPRING` groups; recipe exposes `with_group_flags`, `with_angular_velocity`, `with_transform`. |
| Rigid Particles | **Yes** | Group `RIGID \| SOLID` only (particle flags default `WATER`); same basin/spinning box/ball as Elastic | **None evidenced as API gap.** First gallery use of `RIGID \| SOLID`, but public recipe + `PassId::Rigid` / `RigidDamping` already exist and have unit tests. |

If a wired scene fails recognizability after correct flags, treat that as discretionary narrow engine work (D-13 / Claude's Discretion)—not as a planning blocker for missing methods.

## Standard Stack

### Core
| Library / surface | Version / pin | Purpose | Why Standard |
|-------------------|---------------|---------|--------------|
| `liquidfun` | workspace crate | Physics + particle flags/groups | Only production engine; scenes must stay on public API. `[VERIFIED: crates/liquidfun]` |
| `liquidfun-wasm` | workspace crate | Scene factory, session, copied frames | Existing allowlisted `SceneId` / `build_scene` pattern from Phases 18–26. `[VERIFIED: crates/liquidfun-wasm/src/scene.rs]` |
| SolidJS player + catalog | repo `web/` | Catalog, hash routes, play/pause/reset | Locked Phase 26 shell; append-only. `[VERIFIED: web/src/catalog/scenes.ts]` |
| Upstream LiquidFun | `7f20402173fd143a3988c921bc384459c6a858f2` | Scene geometry + flag recipes | Pin confirmed on submodule HEAD. `[VERIFIED: git -C third_party/liquidfun rev-parse HEAD]` |

### Supporting
| Library / surface | Version | Purpose | When to Use |
|-------------------|---------|---------|-------------|
| `ParticleSystemDef::{with_radius,with_damping,with_color_mixing_strength,with_elastic_strength,with_spring_strength,…}` | current crate | System strengths | Surface Tension damping + optional mixing/elastic strengths if defaults look flat. `[VERIFIED: system_definition.rs]` |
| `ParticleGroupRecipe::{with_particle_flags,with_group_flags,with_transform,with_angular_velocity,with_color,with_strength}` | current crate | Group recipes | All three scenes. `[VERIFIED: recipe.rs + particle_groups.rs]` |
| `PolygonShape::oriented_box` / vertex `PolygonShape::new` | current crate | Blue boxes | Elastic/Rigid use half-extents `(1, 0.5)` + group transform; Surface Tension uses absolute vertices. `[VERIFIED: polygon.rs + pinned tests]` |
| Playwright / `just web-player-smoke` | repo scripts | Chromium gate | Extend watch-first branch via `controls.length === 0`. `[VERIFIED: web/e2e/player.spec.ts]` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| New scene modules | Re-label Color Mixer / Jelly Drop | **Forbidden** — wrong flag story and catalog honesty (PITFALLS). |
| Engine feature work first | Scene-only ports | Wrong order — no API gap found; engine only if contrast fails after wiring. |
| Raising `MAX_ADVANCE_STEPS` | Keep 4 | Locked (D-14 / D-15); smoothness theater is out of scope. |
| Live WASM thumbnails | Static SVG `Static preview` | Locked (D-02). |

**Installation:** None — use existing workspace crates and `web/` toolchain.

**Version verification:** Upstream pin `7f204021…` confirmed; Rust toolchain `1.97.0`; Bun `1.4.2`; just `1.48.0` on the research host. `[VERIFIED: shell probes 2026-09-22]`

## Architecture Patterns

### Recommended Project Structure
```
crates/liquidfun-wasm/src/scene.rs          # SceneId + parse + build_scene match arms
crates/liquidfun-wasm/src/scene/
  surface_tension.rs                       # MAT-01
  elastic_particles.rs                     # MAT-02
  rigid_particles.rs                       # MAT-03
  basin_family.rs (optional, private)      # shared vertical-wall basin + ball helper
  particles.rs / liquid_timer.rs / …       # existing eight (unchanged recipes)
web/src/catalog/scenes.ts                  # append three SCENE_IDS + records
web/src/catalog/previews.tsx               # three Static preview SVGs
web/scripts/demo-media/model.ts            # SCENE_CAPTURE_PLANS length/order
web/e2e/player-helpers.ts                  # SCENE_HASH_PATHS
web/e2e/player.spec.ts / shell.spec.ts     # eleven-scene copy + focus wrap
```

### Pattern 1: Synchronized allowlist growth (Phase 26)
**What:** Extend Rust `SceneId` / `parse_scene_id` / `build_scene` and TS `SCENE_IDS` / hash routes / capture plans in the same wave so the catalog cannot advertise a scene the session cannot build.
**When to use:** Every new ready scene.
**Example:**
```rust
// Source: crates/liquidfun-wasm/src/scene.rs (Phase 26 pattern)
"particles" => Ok(SceneId::Particles),
"liquid-timer" => Ok(SceneId::LiquidTimer),
// Phase 27 adds:
// "surface-tension" | "elastic-particles" | "rigid-particles"
```

### Pattern 2: Watch-first hooks
**What:** Empty `controls: []`, `WATCH_FIRST_HINT`, reject non-empty presets, no-op / Ok pointer handlers, static `collect_segments` + dynamic ball via `collect_circles`.
**When to use:** All three Phase 27 scenes (D-04–D-06).
**Example:** Follow `particles.rs` / `liquid_timer.rs` — e2e already selects watch-first by `controls.length === 0`. `[VERIFIED: web/e2e/player.spec.ts]`

### Pattern 3: Flag-recipe headless asserts
**What:** After `SessionCore::create`, read particle/group flags and assert bits match the pinned recipe (Liquid Timer pattern for `TENSILE | VISCOUS`).
**When to use:** Each new scene module's `#[cfg(test)]`.
**Example:**
```rust
// Source: crates/liquidfun-wasm/src/scene/liquid_timer.rs tests
assert!(flags.contains(ParticleFlags::TENSILE) && flags.contains(ParticleFlags::VISCOUS));
```

### Anti-Patterns to Avoid
- **Reuse Color Mixer as Surface Tension:** mixing-only, wrong radius/damping (`0.05692` vs `0.035` / `0.2`).
- **Copy Jelly `ELASTIC|SPRING` into all three Elastic groups:** collapses spring vs elastic contrast; omit `SOLID`.
- **Put `ELASTIC` particle flags on Rigid Particles:** Rigid uses **group** `RIGID | SOLID` only.
- **Use Particles angled walls for this basin family:** Surface Tension / Elastic / Rigid use vertical side walls ending at `y=2`, not Particles' slanted tops at `y=3`.
- **Cut particle counts or raise `MAX_ADVANCE_STEPS`:** locked.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Scene factory / session | New WASM world model | Existing `SceneId` + `SessionCore` | Generation tokens, dispose, frame lanes already proven. |
| Catalog / hash routing | New router | `SCENE_IDS` + `maybeParseSceneRoute` | Allowlist already security-hardened. |
| Soft / mixing / tensile solvers | Custom forces in scene hooks | Public flags + existing solver passes | Passes are gated in `manifest.rs`. |
| Rigid solid clumps | Fake kinematic bodies | `ParticleGroupFlags::RIGID \| SOLID` | Upstream recipe; rigid projection/damping already implemented. |
| Spinning box particle group | Manual per-particle velocities | `with_transform(angle)` + `with_angular_velocity(2.0)` | Recipe applies sampling velocities. |
| Browser proof | New Firefox/Safari matrix | Extend `just web-player-smoke` | Locked D-16. |

**Key insight:** This phase is integration and catalog honesty, not physics invention—unless a correctly flagged scene still cannot show the required contrast.

## Common Pitfalls

### Pitfall 1: Flag-recipe collapse
**What goes wrong:** Elastic and Rigid look identical; Surface Tension looks like Color Mixer.
**Why it happens:** Copying Jelly Drop or Color Mixer recipes, or omitting `SOLID` / `TENSILE`.
**How to avoid:** Assert exact bits after construction; keep three Elastic groups distinct; keep Rigid free of elastic particle flags.
**Warning signs:** One softness feel for all clumps; no color bleed without beading (or beading without bleed).

### Pitfall 2: Wrong basin family
**What goes wrong:** Walls look like Particles (angled) or Color Mixer bowl.
**Why it happens:** Reusing `particles.rs` coordinates by habit.
**How to avoid:** Copy the Elastic/Rigid/Surface Tension vertical-wall polygons from the pinned tests (shared private helper encouraged).
**Warning signs:** Side walls rise past `y=2` with slanted tops.

### Pitfall 3: Catalog / factory drift
**What goes wrong:** Sidebar advertises eleven scenes but WASM rejects the new ids.
**Why it happens:** Updating only `scenes.ts` or only `scene.rs`.
**How to avoid:** Same-wave allowlist: Rust enum + parse + build, TS `SCENE_IDS`, previews, `SCENE_HASH_PATHS`, `SCENE_CAPTURE_PLANS`, `PAGE_SUMMARY` count, shell focus wrap math.
**Warning signs:** `UnknownScene` in console; capture-plan length assert fails.

### Pitfall 4: Shell chrome count rot
**What goes wrong:** Drawer reverse-tab / “eight demos” copy still says eight after eleven scenes.
**Why it happens:** Phase 26 raised wrap count for nine focusables (Dismiss + 8 links).
**How to avoid:** Update `PAGE_SUMMARY`, shell e2e “eleven static previews”, and focus wrap: Dismiss + 11 links = 12 focusables; adjust reverse-tab modulus accordingly. `[VERIFIED: web/e2e/shell.spec.ts comments]`
**Warning signs:** Focus lands on wrong last link (`Rigid Particles` should become last).

### Pitfall 5: Parity overclaim / upstream “buggy” demos
**What goes wrong:** Docs claim sealed C++ parity; Elastic/Rigid “fix” marketed as correctness.
**Why it happens:** Upstream JS files literally say the tests are buggy.
**How to avoid:** Honest experimental copy (D-12); treat recovery stiffness as chrome.
**Warning signs:** “identical to LiquidFun” in UI strings.

## Code Examples

### Surface Tension group recipe
```rust
// Source: pinned testSurfaceTension.js + liquidfun public API
// Groups 1–2: circles at (0,2) and (-1,2), radius 0.5
// Group 3: box vertices (0,3),(2,3),(2,3.5),(0,3.5)
let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
    .with_particle_flags(ParticleFlags::TENSILE | ParticleFlags::COLOR_MIXING)
    .with_color(color)
    .with_transform(Transform::from_position_angle(center, 0.0))?;
// ParticleSystemDef: radius 0.035, damping 0.2 (JS); optional color_mixing_strength
```

### Elastic Particles group recipes
```rust
// Red circle (0,3) r=0.5
.with_particle_flags(ParticleFlags::SPRING)
.with_group_flags(ParticleGroupFlags::SOLID)
// Green circle (-1,3) r=0.5
.with_particle_flags(ParticleFlags::ELASTIC)
.with_group_flags(ParticleGroupFlags::SOLID)
// Blue box half-extents 1×0.5 at (1,4), angle -0.5, ω = 2
.with_particle_flags(ParticleFlags::ELASTIC)
.with_group_flags(ParticleGroupFlags::SOLID)
.with_transform(Transform::from_position_angle(Vec2::new(1.0, 4.0), -0.5))?
.with_angular_velocity(2.0)?
```

### Rigid Particles group recipe
```rust
// Same shapes/poses as Elastic; no particle flag override (defaults to WATER)
.with_group_flags(ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID)
.with_color(color)
```

### Shared integration points (must touch together)
| Seam | Path | Change |
|------|------|--------|
| Scene enum / factory | `crates/liquidfun-wasm/src/scene.rs` | `SurfaceTension`, `ElasticParticles`, `RigidParticles` + parse + `build_scene` arms |
| Session allowlist tests | `crates/liquidfun-wasm/src/session/tests.rs` | Extend `parse_scene_id` token table |
| Catalog ids | `web/src/catalog/scenes.ts` | Append three ids after `liquid-timer`; empty controls; pinned credits |
| Previews | `web/src/catalog/previews.tsx` | Exhaustive `ScenePreview` cases |
| Capture plans | `web/scripts/demo-media/model.ts` | Append three watch-first plans (center click stub OK) |
| Hash helpers | `web/e2e/player-helpers.ts` | `SCENE_HASH_PATHS` |
| Smoke | `web/e2e/player.spec.ts`, `shell.spec.ts` | Auto via `controls.length === 0`; update eight→eleven copy/focus wrap |
| Page summary | `web/src/player/runtime.ts` | “All eleven demos …” |
| Cap gate | `crates/liquidfun-wasm/src/session.rs` | Confirm `MAX_ADVANCE_STEPS = 4` unchanged |

### Pinned layout constants (do not invent alternate geometry)

**Shared basin (Surface Tension / Elastic / Rigid):**
```text
Floor: (-4,-1), (4,-1), (4,0), (-4,0)
Left:  (-4,-0.1), (-2,-0.1), (-2,2), (-4,2)
Right: (2,-0.1), (4,-0.1), (4,2), (2,2)
Particle radius: 0.035
Dynamic ball: Dynamic, (0,8), r=0.5, density 0.5
Gravity: (0, -10)  [match Particles / existing scenes]
```

**Surface Tension extras:** system damping `0.2` (JS only; C++ omits — prefer JS for recognizability); circles at `(0,2)` / `(-1,2)` r=`0.5`; blue box vertices as above; colors `(255,0,0,255)`, `(0,255,0,255)`, `(0,0,255,255)`.

**Elastic / Rigid extras:** circles at `(0,3)` / `(-1,3)` r=`0.5`; blue `SetAsBox(1, 0.5)` at `(1,4)`, angle `-0.5`, angularVelocity `2`.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Six interactive demos | Eight scenes (six interactive + two watch-first) | Phase 26 | Watch-first e2e branch exists for Phase 27 reuse |
| Color Mixer / Jelly Drop as material teaching | Dedicated Surface Tension / Elastic / Rigid ports | Phase 27 | Correct flag contrast without replacing originals |
| Rigid groups only in engine tests | First playground rigid-group demo | Phase 27 | Proves `RIGID \| SOLID` in WASM gallery |

**Deprecated/outdated:**
- Treating Color Mixer as “already covering” Surface Tension.
- Treating Jelly Drop as the Elastic Particles port (different layout, combined `ELASTIC|SPRING`, no `SOLID`, interactive poke).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Default `ParticleSystemDef` surface-tension / color-mixing / elastic / spring strengths will be recognizable enough without tuning | Capability Verdict | Planner may need a follow-up task to set `with_surface_tension_*` / `with_color_mixing_strength` / elastic-spring strengths if first wiring looks flat — still not an API gap |
| A2 | Approximate ~700 particles across three r=0.5 groups at radius 0.035 fits frame/perf budget without cutting counts | Pitfalls / Environment | If WASM is too slow, still cannot raise catch-up cap or cut counts; may need honesty note only |

No other `[ASSUMED]` claims; flag/API/solver presence was verified in-repo.

## Open Questions

1. **Shared basin helper vs three copies**
   - What we know: Elastic and Rigid basins are identical; Surface Tension walls match that family (circles sit lower at y=2).
   - What's unclear: Whether the planner prefers one private helper module now or copies then extracts.
   - Recommendation: Private `basin_family` helper in wave 1 to prevent wall drift (Claude's Discretion).

2. **Strength tuning before claiming contrast failure**
   - What we know: API exposes damping and material strengths; defaults may suffice.
   - What's unclear: Whether first visual pass needs strength knobs.
   - Recommendation: Wire exact flags first; only then consider strength setters; only then consider engine work.

3. **UI-SPEC for Phase 27**
   - What we know: `workflow.ui_phase` is true; Phase 26 produced `26-UI-SPEC.md`; Phase 27 CONTEXT is mostly append-same-chrome.
   - What's unclear: Whether `/gsd-ui-phase` will regenerate a thin 27-UI-SPEC or reuse 26 tokens.
   - Recommendation: Planner should expect a thin UI contract (eleven-scene copy, three previews, last drawer link = Rigid Particles) rather than a redesign.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust / cargo | Scene modules + tests | ✓ | 1.97.0 | — |
| Bun | Web catalog / smoke | ✓ | 1.4.2 | — |
| just | `web-player-smoke` | ✓ | 1.48.0 | Invoke `bun scripts/web-build.ts player-smoke` |
| Upstream submodule at pin | Credits + geometry | ✓ | `7f204021…` | — |
| Chromium (Playwright) | Smoke gate | ✓ (assumed via existing smoke) | — | Install via existing web scripts if missing |

**Missing dependencies with no fallback:** None identified for planning.

**Missing dependencies with fallback:** None blocking.

Step 2.6: External tools required — audited above.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | N/A — static playground |
| V3 Session Management | no | Opaque WASM session tokens already; no new auth |
| V4 Access Control | no | N/A |
| V5 Input Validation | yes | Allowlist `parse_scene_id` / `SCENE_IDS` only; reject unknown hash tokens |
| V6 Cryptography | no | N/A |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unknown scene id coercion | Tampering | Exact lowercase allowlist; no case folding |
| Catalog advertising unfinished WASM | Spoofing / Integrity | Ready only when factory builds; synchronized ids |
| Host-unlocked “implementation” link to google/liquidfun | Spoofing | Keep host-locked scene module links (D-10) |

## Sources

### Primary (HIGH confidence)
- Pinned lfjs: `testSurfaceTension.js`, `testElasticParticles.js`, `testRigidParticles.js` at `7f204021…`
- Pinned C++: `ParticlesSurfaceTension.h`, `ElasticParticles.h`, `RigidParticles.h`
- `crates/liquidfun/src/particle/definition.rs` — `TENSILE`, `COLOR_MIXING`, `ELASTIC`, `SPRING`
- `crates/liquidfun/src/particle/group/flags.rs` — `SOLID`, `RIGID`
- `crates/liquidfun/src/particle/group/recipe.rs` — recipe builders
- `crates/liquidfun/src/particle/solver/manifest.rs` — Tensile, ColorMixing, Elastic, Spring, Solid, RigidDamping, Rigid passes
- `crates/liquidfun-wasm/src/scene/{particles,liquid_timer,jelly_drop,color_mixer,scene}.rs`
- `web/src/catalog/scenes.ts`, `previews.tsx`, `web/e2e/player.spec.ts`
- `.planning/phases/27-material-flag-groups/27-CONTEXT.md`
- `.planning/phases/26-catalog-shell-and-basin-scenes/26-{CONTEXT,RESEARCH,01-PLAN,03-PLAN,05-PLAN}.md`

### Secondary (MEDIUM confidence)
- `.planning/research/{FEATURES,ARCHITECTURE,STACK,PITFALLS}.md` — recognition cards and pitfall catalog (cross-checked against live sources)

### Tertiary (LOW confidence)
- Approximate particle-count estimate for radius `0.035` groups (planning budget intuition only)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified in-repo APIs and Phase 26 integration seams
- Architecture: HIGH — clear factory/catalog pattern; optional helper is discretionary
- Pitfalls: HIGH — confirmed against pinned tests and Phase 26 smoke/copy traps
- Engine-gap claim: HIGH that **no API is missing**; MEDIUM that default strengths will look “beady / soft / rigid” without tuning (A1)

**Research date:** 2026-09-22
**Valid until:** 2026-10-22 (stable scene-port domain; re-check if particle public API changes)
