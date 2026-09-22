# Phase 26: Catalog shell and basin scenes - Research

**Researched:** 2026-09-21
**Domain:** SolidJS playground catalog + native Rust WASM scene ports (Particles, Liquid Timer)
**Confidence:** HIGH

## Summary

Phase 26 appends two watch-first testbed ports—Particles and Liquid Timer—to the existing shared catalog/player without replacing Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, or Water Wheel. Both layouts are fully specified by pinned lfjs + C++ sources at commit `7f20402173fd143a3988c921bc384459c6a858f2`, and the public `liquidfun` API already exposes the geometry and particle flags those scenes need. `[VERIFIED: third_party/liquidfun HEAD + crates/liquidfun particle/collision APIs]`

The critical planning seam is not a missing tensile/viscous solver: `ParticleFlags::TENSILE` / `VISCOUS`, material solver passes, `ParticleSystemDef` strength fields, `EdgeShape` / `ChainShape`, circle/box particle groups, and dynamic circle bodies are all present and already used by playground scenes or engine tests. The work is synchronized catalog metadata, WASM `SceneId` factory growth, static SVG previews, honest pinned-test credits, headless scene construction tests, and Chromium smoke that covers open/play/pause/reset for eight scenes while keeping watch-first scenes free of pointer/preset chrome. `[VERIFIED: crates/liquidfun-wasm/src/scene.rs + web/src/catalog/scenes.ts + web/e2e/player.spec.ts]`

**Primary recommendation:** Implement scene-only ports in `liquidfun-wasm` (no engine feature work unless recognizability fails after wiring `TENSILE | VISCOUS` and edge shelves); extend `SCENE_IDS` / `SceneId` / factory / previews / credits together; adapt player e2e so watch-first scenes prove play/pause/reset without requiring pointer gestures or labeled controls.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
#### Catalog shell
- **D-01:** Append Particles (`particles`) and Liquid Timer (`liquid-timer`) to the existing `DemoNavigation` sidebar and mobile drawer. Keep the current six scene ids and their current order first. Hash routes stay `#/scene/{id}`.
- **D-02:** Each new entry is `ready: true` once its native scene runs, with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Do not start a WASM world per card, capture live thumbnails, or animate fake physics.
- **D-03:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, filters, or a separate catalog page. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.

#### Watch-first controls
- **D-04:** Particles and Liquid Timer expose play, pause, and Reset only. No water-amount, gravity, or particle-type presets in this phase. PRESET-01 stays future work.
- **D-05:** Interaction copy says these scenes are watch-first. Do not add pointer drag, click impulse, or keyboard material modes.
- **D-06:** Reset disposes the session and recreates the documented initial layout. Follow the Phase 20 remount rule if any construction selects exist later: visible preset labels return to `DEFAULT_PRESET_VALUES`.

#### Pinned-test credits
- **D-07:** Keep the Phase 18 credit chrome: implementation link to this repository's scene module, inspiration links, and third-party notices. Implementation links stay host-locked to this repo and must not point at `google/liquidfun` as the running code.
- **D-08:** Inspiration for Particles cites pinned `testParticles.js` and `Particles.h` at commit `7f20402173fd143a3988c921bc384459c6a858f2`. Inspiration for Liquid Timer cites pinned `testLiquidTimer.js` and `LiquidTimer.h` at that same commit. Preserve notices via `THIRD_PARTY_NOTICES.md` when upstream material is adapted.
- **D-09:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations.

#### Recognizable basin layouts
- **D-10:** Particles must show an open basin (floor plus slanted side walls), a falling water group, and one dynamic ball that drops into the water. Match `testParticles.js` / `Particles.h` closely enough to be recognizable: water circle and a rigid ball above it. Damping tweaks and the C++ particle-type picker are chrome, not this phase.
- **D-11:** Liquid Timer must show tensile and viscous liquid draining through a vertical gap and zig-zag shelves into four bottom columns. Match `testLiquidTimer.js` / `LiquidTimer.h` closely enough to be recognizable. Alternate particle-parameter sets are chrome (PRESET-01).
- **D-12:** Author both scenes in `liquidfun-wasm` on the public `liquidfun` API. Add engine behavior only when a listed scene cannot run without it. Do not cut particle counts or raise the 4-step catch-up cap to look smoother. Do not import the desktop testbed or replay diagnostic protocol recipes as gallery demos.

#### Shared session and proof
- **D-13:** Grow the existing checked scene-id factory. Keep one WASM world, copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch. `MAX_ADVANCE_STEPS` stays 4.
- **D-14:** Prove locally with the existing Chromium `just web-player-smoke` suite extended so Particles and Liquid Timer open, play, pause, and reset, and so the original six scenes still open and run. Do not add Firefox/Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-15:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact basin coordinates, particle radius, and particle counts, as long as the visitor can recognize the pinned tests and counts are not reduced to fake smoothness.
- Static SVG preview artwork, as long as each is captioned `Static preview` and does not imply a live simulation.
- Whether tensile and viscous strengths use existing `ParticleSystemDef` fields or a narrowly justified engine addition when the scene cannot drain recognizably without it.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.

### Deferred Ideas (OUT OF SCOPE)
- Surface Tension, Elastic Particles, and Rigid Particles — Phase 27.
- Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen — Phase 28.
- Sparky, Drawing Particles, and the full twelve-scene catalog claim (PLAY-01) — Phase 29.
- Liquid Timer extra particle-type presets (PRESET-01) and the Drawing Particles keyboard matrix (DRAW-02).
- Sealed per-scene differential evidence (PARITY-01) and commented-out Box2D-only tests (BOX2D-01).
- Optional pointer nudge on watch-first scenes. Not required for recognizability.
- Cross-links between related scenes (Particles as the baseline for later material scenes).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PLAY-02 | Visitor can play, pause, and reset each new scene, and reset restores that scene's initial layout. | Existing `SessionCore::create` + dispose-on-reset; player chrome already provides Play/Pause/Reset; e2e must cover the two new ids without requiring scene-local controls. |
| PLAY-03 | Each new scene credits the pinned LiquidFun test it ports. | Extend `SceneCredits` inspiration links to pinned blob URLs for `testParticles.js`/`Particles.h` and `testLiquidTimer.js`/`LiquidTimer.h` at `7f204021…`; keep host-locked implementation paths under `crates/liquidfun-wasm/src/scene/`. |
| BASIN-01 | Visitor can watch Particles: water falls in an open basin and a ball drops into it. | Port floor + slanted walls + water circle + dynamic ball from pinned JS/C++; reuse Dam Break / Float or Sink body+group patterns. |
| BASIN-02 | Visitor can watch Liquid Timer: tensile, viscous liquid drains through shelves into bottom columns. | Port chain-loop container + `TENSILE \| VISCOUS` slab + edge shelves/columns from pinned JS/C++; no engine addition expected if default strengths suffice. |
</phase_requirements>

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. `[VERIFIED: Glob .cursor/rules]`

Apply repo-local and Bright Builds standards instead:

- Hobby scope / experimental API / no package publication from this phase (`PROJECT-SCOPE.md`, `AGENTS.md`).
- Scene construction in WASM shell; physics in `liquidfun`; renderer-free engine (`standards/core/architecture.md`).
- Safe Rust, no `unwrap` in production paths, `foo.rs` + `foo/` module shape (`standards/languages/rust.md`).
- SolidJS + Bun web tooling; Kobalte Dialog override for shell (`standards/languages/typescript-javascript.md`, `standards-overrides.md`).
- Dark playground chrome already established (`standards/core/frontend-ui.md`).
- Focused unit tests for new scene construction and catalog records (`standards/core/testing.md`).
- Local checks before commit; Chromium gate is `just web-player-smoke` (`standards/core/verification.md`, D-14).
- Do not format `.planning/**` with mdformat (`AGENTS.md` Repo-Local Guidance).

## Standard Stack

### Core

| Library / surface | Version / pin | Purpose | Why Standard |
|-------------------|---------------|---------|--------------|
| `liquidfun` (workspace crate) | repo pin, Rust 1.97.0 toolchain | Public physics API for scene construction | Only allowed production physics path `[VERIFIED: rust-toolchain.toml + crates/liquidfun]` |
| `liquidfun-wasm` | unpublished workspace crate | Scene factory, session, frame copy | Existing player boundary `[VERIFIED: crates/liquidfun-wasm]` |
| SolidJS playground (`web/`) | current `web/package.json` | Catalog, player, hash routes | Locked by Phases 17–20 `[VERIFIED: web/src]` |
| `@kobalte/core` | 0.13.12 (override) | Mobile demos drawer | `standards-overrides.md` `[CITED: standards-overrides.md]` |
| Playwright Chromium | via `just web-player-smoke` | Browser proof | D-14 gate `[VERIFIED: justfile + web/package.json]` |
| Pinned upstream LiquidFun | `7f20402173fd143a3988c921bc384459c6a858f2` | Layout/flag oracle | Submodule HEAD matches CONTEXT `[VERIFIED: git -C third_party/liquidfun rev-parse HEAD]` |

### Supporting

| Library / surface | Version | Purpose | When to Use |
|-------------------|---------|---------|-------------|
| `ParticleFlags::TENSILE \| VISCOUS` | in-tree | Liquid Timer material recipe | Group flags on `ParticleGroupRecipe` |
| `ParticleSystemDef::{viscous_strength, surface_tension_*}` | defaults `0.25` / `0.2` / `0.2` | Strength knobs without new API | Only if drain is unrecognizable at defaults `[VERIFIED: system_definition.rs Default]` |
| `ChainShape::closed` / `EdgeShape::new` | in-tree | Timer walls and shelves | Fixture construction `[VERIFIED: collision/shape/{chain,edge}.rs]` |
| `PolygonShape::oriented_box` / `CircleShape` | in-tree | Timer slab + Particles water/ball | Match JS `SetAsBox` / circle groups |
| Vitest (`web/tests`) | current web suite | Catalog/credit unit tests | Update six→eight scene assertions |
| Cargo unit tests in scene modules | existing pattern | Construction / flag / layout asserts | Mirror `dam_break/tests.rs`, `jelly_drop` tests |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Scene-only tensile/viscous wiring | New engine material APIs | Forbidden unless scene cannot drain; STACK already says scene-only `[CITED: .planning/research/STACK.md]` |
| Live WASM thumbnails | Static SVG | Violates D-02 / Phase 20 |
| Second player or card grid | Existing shell | Violates D-03 |
| Cutting particle counts / raising `MAX_ADVANCE_STEPS` | Keep radius ~0.025–0.035 and cap 4 | Violates D-12 / D-13 |
| Claiming C++ parity | Recognizable port copy | Violates D-09 |

**Installation:** No new packages. Use existing Cargo workspace + `bun` web toolchain.

**Version verification:** Toolchain and pins verified locally on 2026-09-21: Rust `1.97.0`, Bun `1.4.2`, just `1.48.0`, Node `v24.13.0`, upstream submodule `7f204021…`. `[VERIFIED: shell probes]`

## Architecture Patterns

### Recommended Project Structure

```
crates/liquidfun-wasm/src/
├── scene.rs                 # SceneId, parse_scene_id, build_scene match
├── scene/
│   ├── particles.rs         # NEW: open basin + water + ball
│   ├── liquid_timer.rs      # NEW: chain + edges + tensile|viscous slab
│   ├── particles/tests.rs   # optional if file-length trigger
│   └── …existing six…
web/src/
├── catalog/scenes.ts        # append SCENE_IDS + SCENES records
├── catalog/previews.tsx     # Particles + Liquid Timer SVG cases
├── components/DemoNavigation.tsx  # unchanged pattern (maps SCENES)
└── components/scene-credits.ts    # reuse host-locked helpers
web/e2e/
├── player.spec.ts           # open/play/pause/reset for eight; watch-first path
├── shell.spec.ts            # keep .catalog-card count 0; preview counts grow
└── player-helpers.ts        # SCENE_HASH_PATHS + timeouts
```

### Pattern 1: Synchronized catalog ↔ WASM allowlist

**What:** `web/src/catalog/scenes.ts` `SCENE_IDS` and `crates/liquidfun-wasm` `SceneId` / `parse_scene_id` / `build_scene` must gain `particles` and `liquid-timer` in the same change set.
**When to use:** Every new gallery scene (CONTEXT integration points).
**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene.rs (existing pattern)
"dam-break" => Ok(SceneId::DamBreak),
// Add:
"particles" => Ok(SceneId::Particles),
"liquid-timer" => Ok(SceneId::LiquidTimer),
```

### Pattern 2: Watch-first SceneHooks with static segment/circle export

**What:** Build world once; `on_advance` no-op (or empty); reject unknown controls/actions; `apply_pointer` no-op or ignore; `collect_segments` / `collect_circles` expose basin edges and the ball for canvas draw.
**When to use:** Particles and Liquid Timer (D-04, D-05).
**Example:** Follow Dam Break’s static `basin_segments` + circle collect, without drag hooks. Liquid Timer should list every shelf/column edge as `RigidSegment` so zig-zag geometry is visible.

### Pattern 3: Particle group recipe with flags

**What:** Create system with pinned radius; build `ParticleGroupRecipe` from filled circle/box source; set particle flags; transform to world pose.
**When to use:** Particles water (`WATER` / default flags); Liquid Timer slab (`TENSILE | VISCOUS`).
**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/jelly_drop.rs + color_mixer.rs (existing)
let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
    .with_particle_flags(ParticleFlags::TENSILE | ParticleFlags::VISCOUS)
    .with_transform(Transform::from_position_angle(center, 0.0))?;
world.create_particle_group(system, &recipe)?;
```

### Pattern 4: Static catalog preview switch

**What:** Extend `ScenePreview` exhaustive switch with token-only SVG; caption remains `Static preview` in `DemoNavigation`.
**When to use:** Every new `SceneId` (TypeScript exhaustiveness).

### Anti-Patterns to Avoid

- **Advertising before factory support:** Catalog `ready: true` without WASM `parse_scene_id` / `build_scene` → load failure.
- **Treating JS chrome as required:** Particle-type picker, damping-only tweaks, alternate Liquid Timer params (PRESET-01).
- **Pointer/control e2e for watch-first scenes:** Current `POINTER_CONTROL` map requires a gesture + labeled control per `SceneId`; must be split for empty-control scenes. `[VERIFIED: web/e2e/player.spec.ts]`
- **Raising catch-up or shrinking counts for FPS:** Violates D-12 / D-13.
- **Engine feature for “maybe nicer viscosity” without a failed recognizability attempt:** Discretion allows strengths; not a new subsystem.
- **Restoring `.catalog-card`:** Forbidden by D-03 and shell smoke.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tensile/viscous material | Custom JS forces / fake shaders | `ParticleFlags` + existing solver passes | Passes already gated in `manifest.rs` `[VERIFIED]` |
| Edge shelves | Polygon approximations in TS | `EdgeShape` fixtures + `collect_segments` | Collision + draw already exist |
| Closed timer bowl | Manual wall polygons only | `ChainShape::closed` (or four edges) | Matches lfjs loop; `Shape::from(ChainShape)` works `[VERIFIED: collision_shapes tests]` |
| Catalog routing | New router | `#/scene/{id}` + `SCENE_IDS` allowlist | Locked Phases 17–20 |
| Credits host lock | Raw google/liquidfun implementation URL | `implementationHref` / `sceneBlobUrl` | D-07 |
| Browser proof | New suite / Pages deploy | Extend `just web-player-smoke` | D-14 |
| Live thumbnails | Per-card WASM | Static SVG in `previews.tsx` | D-02 |

**Key insight:** This phase is catalog + scene composition. The engine already has the material and geometry primitives; planning risk is seam synchronization and e2e assumptions that every scene has pointer controls.

## Common Pitfalls

### Pitfall 1: Exhaustive TypeScript / Playwright maps break on new SceneId

**What goes wrong:** Adding ids without updating `POINTER_CONTROL`, `SCENE_HASH_PATHS`, `UI_SPEC_DESCRIPTIONS`, and `ScenePreview` switch fails compile or smoke.
**Why it happens:** `Record<SceneId, …>` and exhaustive switches.
**How to avoid:** Treat catalog, helpers, vitest fixtures, and WASM enum as one checklist; for watch-first scenes, add a separate e2e path that only asserts play/pause/reset and chrome (no gesture/control).
**Warning signs:** `tsc` errors on missing Record keys; Playwright timeout waiting for a control that does not exist.

### Pitfall 2: Wrong particle flag recipe on Liquid Timer

**What goes wrong:** Scene looks like ordinary water; drain does not bead/cling.
**Why it happens:** Defaulting to `WATER` or confusing group flags with particle flags.
**How to avoid:** Assert created particles carry `TENSILE | VISCOUS` bits in a headless WASM test (same pattern as Jelly Drop elastic asserts). JS and C++ default param are `b2_tensileParticle | b2_viscousParticle`. `[VERIFIED: testLiquidTimer.js + LiquidTimer.h k_paramValues]`
**Warning signs:** Aggregate flags in debugger lack tensile/viscous; liquid floods like Dam Break water.

### Pitfall 3: Age destruction or emitter patterns copied onto static groups

**What goes wrong:** Particles vanish mid-demo.
**Why it happens:** Fountain/Water Wheel enable destruction-by-age for emitters; default `ParticleSystemDef` also has `destroy_by_age: true`, but recipes with non-positive lifetime are infinite. Copying emitter lifetime settings is the real risk. `[VERIFIED: system_definition.rs Default + fountain.rs]`
**How to avoid:** Use filled-group recipes with default/infinite lifetime; do not enable emitter-style lifetimes for these two scenes.
**Warning signs:** Particle count drops to zero while paused or after short play.

### Pitfall 4: Invisible or incomplete rigid draw for Liquid Timer

**What goes wrong:** Drain geometry exists in physics but canvas only shows particles.
**Why it happens:** Hooks forget to export many edge segments.
**How to avoid:** Encode every shelf/column edge in `collect_segments` (static list is fine).
**Warning signs:** Hourglass “empty box” visually; visitor cannot see zig-zag.

### Pitfall 5: Six-scene hardcoding in unit tests and timeouts

**What goes wrong:** Vitest still expects exactly six ids; smoke timeout too short for eight open/reset loops.
**Why it happens:** `web/tests/scenes.test.ts` locks six; `SIX_SCENE_TIMEOUT_MS = 120_000` names the old count. `[VERIFIED]`
**How to avoid:** Update catalog unit tests to eight-in-order; raise or rename timeout budget for all-scene loops.
**Warning signs:** Vitest fails on “lists six locked scenes”; Playwright timeout on open-each-scene test.

### Pitfall 6: Camera framing confusion

**What goes wrong:** Timer looks tiny or Particles ball clips.
**Why it happens:** Shared `WORLD_BOUNDS` is fixed `(-6,-1)–(6,8)` and already covers both layouts, but Timer’s smaller world appears zoomed-out. `[VERIFIED: web/src/render/camera.ts]`
**How to avoid:** Keep fixed camera (do not add per-scene cameras this phase); place geometry at pinned coordinates so both remain inside bounds.
**Warning signs:** Geometry outside ±6 x or y>8 / y<-1.

## Code Examples

### Particles layout constants (pinned JS)

```javascript
// Source: third_party/liquidfun/.../testParticles.js
// Floor: (-4,-1)-(4,0); left wall slant; right wall slant
// Water: circle center (0,3) radius 2; system radius 0.035
// Ball: dynamic circle (0,8) radius 0.5 density 0.5
```

### Liquid Timer flag + shelves (pinned JS)

```javascript
// Source: third_party/liquidfun/.../testLiquidTimer.js
psd.radius = 0.025;
pd.flags = b2_tensileParticle | b2_viscousParticle;
// Top gap between x=-1.2 and x=-1.1 at y=3.2; diagonals; columns at x=±1.2,±0.4
```

### Existing WASM circle group + flags

```rust
// Source: crates/liquidfun-wasm/src/scene/color_mixer.rs
let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
    .with_particle_flags(MIXING_FLAGS)
    .with_color(color)
    .with_transform(Transform::from_position_angle(center, 0.0))?;
world.create_particle_group(system, &recipe)?;
```

### Catalog append shape

```typescript
// Source: web/src/catalog/scenes.ts (extend; keep first six order)
export const SCENE_IDS = [
  "dam-break",
  "fountain",
  "float-or-sink",
  "color-mixer",
  "jelly-drop",
  "water-wheel",
  "particles",
  "liquid-timer",
] as const;
```

### Credit URL shape (mirror Fountain faucet pin)

```typescript
// Pattern from scenes.ts PINNED_FAUCET
href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testParticles.js"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Six original playground scenes only | Append official lfjs menu ports into same player | v1.3 Phase 26 | Catalog grows without second engine |
| Card-grid `.catalog-card` | Sidebar/drawer + static SVG | Phase 20 | Keep list UX |
| Sealed parity ambition | Recognizable native ports | PROJECT-SCOPE / v1.3 research | Honesty in copy and credits |

**Deprecated/outdated:**

- Requiring particle-type chrome for Particles / Liquid Timer (C++ TestMain parameter UI).
- Treating live catalog thumbnails as required.

## Engine Gap Verdict (planner-critical)

| Capability | Status | Evidence | Phase 26 action |
|------------|--------|----------|-----------------|
| `TENSILE` / `VISCOUS` flags | Present | `ParticleFlags` bits 1<<7 / 1<<5 | Use on Liquid Timer group |
| Tensile + viscous solver passes | Present | `particle/solver/material.rs` + `manifest.rs` gates | No new pass |
| Strength fields | Present | `with_viscous_strength`, `with_surface_tension_*` | Discretion only if defaults fail recognition |
| Circle/box particle groups | Present | Color Mixer / Jelly Drop | Particles water + Timer slab |
| Dynamic circle body | Present | Dam Break / Float or Sink | Particles ball |
| Polygon basin walls | Present | `attach_basin_fixture` | Particles slanted walls |
| `EdgeShape` fixtures | Present | Public API + differential recipes | Liquid Timer shelves/columns |
| `ChainShape::closed` fixtures | Present | `Shape::from(ChainShape)` + rigid tests | Timer outer loop |
| Destroy-in-shape / joints / contact hooks | Not needed | Deferred phases | Out of scope |

**Verdict:** Plan as **scene-module-only**. Treat a narrow `ParticleSystemDef` strength tweak as discretion, not Wave 0 engine work. Escalate to engine only if a construction attempt proves the public API cannot produce recognizable drain after correct flags and geometry. `[VERIFIED: API surfaces + STACK.md row for Liquid Timer]`

## Catalog & Test Seams (planner-critical)

### Catalog seams

1. `SCENE_IDS` order: six existing first, then `particles`, `liquid-timer`.
2. `SCENES` records: `ready: true`, watch-first `interactionHint`, `controls: []`, credits with pinned JS+C++ inspiration.
3. `previews.tsx` exhaustive switch + DemoNavigation caption unchanged.
4. Vitest `scenes.test.ts` / navigation tests: stop hardcoding six.
5. Optional: `THIRD_PARTY_NOTICES.md` if adapting upstream snippets beyond link citation (D-08).

### WASM seams

1. `SceneId` enum + `parse_scene_id` + `build_scene` + session allowlist tests in `session.rs`.
2. New modules `particles.rs` / `liquid_timer.rs` (split tests submodule if file-length pressure; jelly_drop is already 605 lines). `[VERIFIED: wc -l]`
3. Keep `MAX_ADVANCE_STEPS = 4`.
4. Headless tests: construct; Particles has circle+ball; Liquid Timer particles include `TENSILE|VISCOUS`; advance N steps without panic; reset path covered by session recreate.

### Test / smoke seams

1. Extend `SCENE_HASH_PATHS`.
2. Split player e2e: interactive six keep gesture+control loop; watch-first two assert play/pause/reset + credits only.
3. `opens each native scene…` already iterates `SCENES` — will pick up new entries once catalog grows; ensure timeout budget.
4. `shell.spec.ts`: keep `.catalog-card` count 0; expect eight `Static preview` captions in sidebar.
5. Gate command remains `just web-player-smoke` (`bun scripts/web-build.ts player-smoke` → `test:player`).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Default viscous/surface-tension strengths yield recognizable Liquid Timer drain once flags+geometry match | Engine Gap Verdict | May need discretionary strength tweaks (still not new APIs) |
| A2 | Fixed `WORLD_BOUNDS` framing is acceptable for Timer’s smaller world without per-scene cameras | Pitfall 6 | Visitor may find Timer hard to see; still in-bounds |

**If wrong:** Stay within Claude’s Discretion (coordinates/strengths); do not invent per-scene camera systems in this phase unless discuss-phase revisits.

## Open Questions (RESOLVED)

1. **Should inspirations list both `.js` and `.h` as separate links, or one combined credit row?**
   - What we know: D-08 requires citing both files; Fountain today cites one pinned `.h` plus showcase.
   - What's unclear: Exact UI density preference.
   - Recommendation: Two inspiration entries per scene (JS test + C++ header) at the pinned commit, plus optional showcase link if copy stays consistent with existing demos.
   - RESOLVED: Two inspiration entries per scene (pinned JS test and C++ header). Plans 03 and 26-UI-SPEC implement this.

2. **Exact Playwright structure for watch-first scenes**
   - What we know: Current all-scene pointer test cannot apply as-is.
   - What's unclear: Whether to filter by `controls.length === 0` or maintain an explicit watch-first id list.
   - Recommendation: Prefer `controls.length === 0` (or missing POINTER_CONTROL entry) so Phase 27 watch-first material scenes reuse the same rule.
   - RESOLVED: Filter watch-first scenes by `controls.length === 0`. Plan 05 implements this.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | WASM scene modules | ✓ | 1.97.0 | — |
| bun | web build + smoke | ✓ | 1.4.2 | — |
| just | `web-player-smoke` | ✓ | 1.48.0 | call `bun scripts/web-build.ts player-smoke` |
| node | Playwright stack | ✓ | v24.13.0 | — |
| third_party/liquidfun pin | Layout/credits | ✓ | `7f204021…` | — |
| Chromium (Playwright) | D-14 gate | ✓ (install via smoke) | managed by `browser:install` | — |

**Missing dependencies with no fallback:** None identified.

**Missing dependencies with fallback:** None required.

Step 2.6: External tools probed; phase is code/config within existing toolchain.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Static Pages playground; no auth |
| V3 Session Management | no | No user accounts |
| V4 Access Control | no | Public read-only demo |
| V5 Input Validation | yes | Allowlisted `SceneId` / control tokens; reject unknown hash with fallback chrome |
| V6 Cryptography | no | No crypto features in phase |

### Known Threat Patterns for WASM playground scenes

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Arbitrary scene id / control injection | Tampering | `parse_scene_id` / unknown-control errors; catalog allowlist |
| Host confusion on “implementation” links | Spoofing | Host-locked `implementationHref` (D-07) |
| Overclaiming parity / live previews | Elevation of privilege (trust) | D-09 honesty copy; `Static preview` caption |
| Resource exhaustion via catch-up | Denial of service | `MAX_ADVANCE_STEPS = 4` unchanged |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/26-catalog-shell-and-basin-scenes/26-CONTEXT.md` — locked decisions
- `third_party/liquidfun` @ `7f204021…` — `testParticles.js`, `Particles.h`, `testLiquidTimer.js`, `LiquidTimer.h`
- `crates/liquidfun/src/particle/definition.rs` — flag bits
- `crates/liquidfun/src/particle/definition/system_definition.rs` — strengths / defaults
- `crates/liquidfun/src/particle/solver/{material,manifest}.rs` — tensile/viscous passes
- `crates/liquidfun-wasm/src/scene.rs` + existing scene modules — factory patterns
- `web/src/catalog/{scenes.ts,previews.tsx}` + `web/e2e/player.spec.ts` — catalog/smoke seams
- `.planning/research/{FEATURES,ARCHITECTURE,STACK}.md` — recognition cards and capability map

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` — flag-recipe and performance pitfalls for gallery ports
- `standards-overrides.md` — Kobalte / skipLibCheck exceptions

### Tertiary (LOW confidence)

- A1/A2 assumptions on default strengths and framing acceptability (see Assumptions Log)

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — in-repo pins and APIs verified
- Architecture: HIGH — existing scene/session/catalog patterns are concrete
- Pitfalls: HIGH — e2e/catalog exhaustive-map risks verified in live tests
- Engine-gap “no new API needed”: HIGH for flags/geometry; MEDIUM for default-strength recognizability (A1)

**Research date:** 2026-09-21
**Valid until:** 2026-10-21 (stable in-repo surfaces; re-check if particle solver or player e2e contracts change)
