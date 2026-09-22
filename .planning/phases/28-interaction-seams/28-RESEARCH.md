# Phase 28: Interaction seams - Research

**Researched:** 2026-09-22
**Domain:** Native WASM playground scenes — destroy-in-shape, particle-group force/impulse, prismatic lifecycle, live revolute motors, soft distance joints
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Catalog shell
- **D-01:** Append Soup (`soup`), Soup Stirrer (`soup-stirrer`), Impulse (`impulse`), Wave Machine (`wave-machine`), and Theo Jansen (`theo-jansen`) after the current eleven scene ids. Keep the existing order first. Hash routes stay `#/scene/{id}`.
- **D-02:** Each new entry is `ready: true` once its native scene runs, with a title, one-behavior description, and a compact static inline SVG preview captioned `Static preview`. Do not start a WASM world per card, capture live thumbnails, or animate fake physics.
- **D-03:** Keep `PlaygroundShell`: sticky header, desktop sidebar, semantic hash links, and the Kobalte Dialog drawer. Do not restore `.catalog-card`, a card grid, search, filters, or a separate catalog page. `web/e2e/shell.spec.ts` must keep asserting `.catalog-card` count is 0.

#### Per-scene controls
- **D-04:** Soup and Wave Machine expose play, pause, and Reset only. They are watch-first.
- **D-05:** Soup Stirrer adds one non-recreating action labeled `Toggle paddle rail`. The interaction hint says the action frees the paddle from its rail, and the next press puts it back. A click or tap on the canvas sends that same command. Reset disposes the session and restores the paddle on the rail.
- **D-06:** Impulse adds one non-recreating runtime preset `push-mode`, labeled `Push`, with values `force` (default) and `impulse`. A click or tap inside the box shoves the whole blob from its center toward the pointer using that mode. A click or tap outside the box does nothing. Reset restores `force`.
- **D-07:** Theo Jansen adds one non-recreating runtime preset `motor-direction`, labeled `Motor direction`, with values `forward` (default) and `reverse`. The walker starts moving forward under the particle load. Changing the preset flips the live revolute motor speed sign and does not recreate the world. Reset restores `forward`.
- **D-08:** Do not use freeglut key legends as the only controls. Keyboard access may mirror the labeled controls. Do not add pointer drag, material pickers, or particle-type presets. PRESET-01 stays future work.

#### Recognizable layouts
- **D-09:** Soup must show a basin of liquid and floating solid bits: a circle, two squares, and edge noodles, with particles carved out from under those fixtures. Match `testSoup.js` / `Soup.h` closely enough to be recognizable. Broth with bobbing solids is the must-see. Particle-parameter chrome is out.
- **D-10:** Soup Stirrer must reuse that soup, then add a dynamic circle paddle, particles carved out under the paddle, a prismatic rail, and a per-step stirring force while the rail is attached. Match `testSoupStirrer.js` / `SoupStirrer.h` closely enough to be recognizable. Freeing the rail lets the paddle leave the rail. Putting it back constrains the paddle again.
- **D-11:** Impulse must show one particle group in a box. A pointer shove moves that group as one blob. Match `testImpulse.js` / `Impulse.h` closely enough to be recognizable. Default shove is group force. The labeled switch uses group linear impulse. Particle-type presets are chrome.
- **D-12:** Wave Machine must show a motorized revolute tank of four thin walls with water inside. Each advance sets motor speed from simulated time, `0.05 * cos(t) * π`, using step count times `dt`. Pause freezes `t`. Reset returns `t` to zero. Match `testWaveMachine.js` / `WaveMachine.h` closely enough to be recognizable. No pointer control.
- **D-13:** Theo Jansen must show a ground, end walls, a chassis and legs built from revolute and soft distance joints, collision filtering so the walker does not self-collide, a motorized revolute, and a particle load on top. The machine walks. Reverse changes walk direction. Match `testTheoJansen.js` / `TheoJansen.h` closely enough to be recognizable. The limit-toggle key is chrome. Do not weld the legs into rigid polygons.

#### Native seams
- **D-14:** Author all five scenes in `liquidfun-wasm` on the public `liquidfun` API. Add engine behavior only when a listed scene cannot show the required behavior without it. Do not approximate carving, group shove, or motor speed in SolidJS.
- **D-15:** Soup and Soup Stirrer carving uses native destroy-in-shape before the first presented frame. Prefer a narrow public helper when the scene cannot express a particle-free pocket with existing queries. A focused test must show particles removed under a placed fixture. Do not enable destruction-by-age on these scenes.
- **D-16:** Impulse shove uses native particle-group force and linear impulse. A focused test must show the group's momentum change. Do not shove one particle, tween positions, or shake the camera.
- **D-17:** Wave Machine and Theo Jansen use the existing revolute motor mutators (`set_revolute_motor_enabled`, `set_revolute_motor_speed`) from scene `on_advance` and control handlers inside the session step. Do not set motor speed from a JavaScript animation frame outside that step. Water Wheel's motor-off revolute is the wrong pattern to copy.
- **D-18:** Soup Stirrer creates and destroys the prismatic joint through typed joint APIs. Stirring force stays in `on_advance` with the in-soup and max-speed guards while the rail is attached. Toggle twice and Reset must both be tested. Do not leak joints or leave an unconstrained paddle integrating unbounded force.
- **D-19:** Share one private soup builder inside `liquidfun-wasm` for Soup and Soup Stirrer. Stirrer composes that builder plus paddle, carve, prismatic joint, and stirring. Do not require the visitor to open Soup first, and do not fork a second soup layout.

#### Counts, credits, and proof
- **D-20:** Do not cut particle counts or raise the 4-step catch-up cap to look smoother. `MAX_ADVANCE_STEPS` stays 4. If a radius must change for playground cost, record that as an explicit playground adaptation in the scene notes. Do not silently bump Wave Machine or Impulse radii.
- **D-21:** Keep the Phase 18 credit chrome: implementation link to this repository's scene module, inspiration links, and third-party notices. Implementation links stay host-locked to this repo and must not point at `google/liquidfun` as the running code.
- **D-22:** Inspiration cites the pinned JS and C++ tests at commit `7f20402173fd143a3988c921bc384459c6a858f2`: Soup cites `testSoup.js` and `Soup.h`; Soup Stirrer cites `testSoupStirrer.js` and `SoupStirrer.h`; Impulse cites `testImpulse.js` and `Impulse.h`; Wave Machine cites `testWaveMachine.js` and `WaveMachine.h`; Theo Jansen cites `testTheoJansen.js` and `TheoJansen.h`. Preserve notices via `THIRD_PARTY_NOTICES.md` when upstream material is adapted.
- **D-23:** Copy identifies an experimental native Rust port with recognizable behavior. Do not claim sealed C++ parity, bit-exact layout, or that catalog previews are live simulations.
- **D-24:** Grow the existing checked scene-id factory. Keep one WASM world, copied typed-array frames, no raw pointers, no per-particle JS/Rust calls, generation-token loads, and dispose-on-switch. Pointer hits reuse the shared CSS-bound unproject path.
- **D-25:** Prove locally with the existing Chromium `just web-player-smoke` suite extended so the five new scenes open, play, pause, and reset, and so the current eleven scenes still open and run. Interactive smoke covers one Soup Stirrer rail toggle, one Impulse click inside the box, and one Theo Jansen direction change. Do not add Firefox, Safari, Linux qualification, or a live Pages redeploy as this phase's gate.
- **D-26:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion
- Exact basin coordinates, particle radius, group sizes, and particle counts, as long as the visitor can recognize the pinned tests and counts are not reduced to fake smoothness.
- Static SVG preview artwork, as long as each is captioned `Static preview` and does not imply a live simulation.
- Whether destroy-in-shape is a public `World` method or a scene-local helper over existing query and `mark_particle_for_destruction`, as long as the carve is native, runs before the first presented frame, and has a focused test.
- File split inside `crates/liquidfun-wasm/src/scene/` when a scene module approaches the file-length trigger.
- Exact button and preset wording, as long as free-or-restore, force-or-impulse, and forward-or-reverse stay obvious in the visible controls.
- Whether Theo Jansen also exposes a speed magnitude beside direction, as long as reverse works and the walker moves under the particle load.

### Deferred Ideas (OUT OF SCOPE)
- Impulse and Liquid Timer particle-type presets (PRESET-01) — later work.
- Sparky sparks, Drawing Particles paint, and the full twelve-scene catalog claim (PLAY-01, FX-01, FX-02) — Phase 29.
- Theo Jansen limit toggle and extra speed keys beyond forward and reverse.
- C++ particle-parameter panels on Impulse and Wave Machine.
- Cross-links from Soup to Soup Stirrer as a teaching path.
- Firefox, Safari, Linux qualification, and a live Pages redeploy — not this phase's gate.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ACT-01 | Visitor can watch Soup: a basin of liquid holds floating solid bits | Soup geometry from pinned `testSoup.js`; carve via destroy-in-shape pattern; shared soup builder; watch-first catalog entry |
| ACT-02 | Visitor can watch Soup Stirrer: paddle keeps stirring; free/restore rail | Compose soup builder + prismatic create/destroy + `on_advance` force guards; action + pointer toggle |
| ACT-03 | Visitor can click/tap Impulse and shove the whole particle blob | Group force/impulse via contiguous `member_ids` + range APIs; box hit-test; `push-mode` live preset |
| ACT-04 | Visitor can watch Wave Machine rock and slosh | Motorized revolute with `enable_motor: true`; `on_advance` sets `0.05 * cos(t) * π`; sim-time `t` |
| ACT-05 | Visitor can watch Theo Jansen walk under particle load and reverse motor | Soft `DistanceJointDef` (10 Hz / 0.5 damping), filter `groupIndex = -1`, live `set_revolute_motor_speed` sign flip |
</phase_requirements>

## Summary

Phase 28 adds five interaction-heavy playground scenes on top of the existing eleven-scene catalog and WASM session. The hard work is native seams, not UI chrome: Soup/Stirrer need particle carving under fixtures; Impulse needs whole-group shove; Wave Machine and Theo Jansen need live revolute motors (Water Wheel is the anti-pattern); Stirrer needs prismatic create/destroy; Theo Jansen needs soft distance joints and self-collision filtering.

Most joint and particle-force primitives already exist on the public `liquidfun` API. The one named gap is `DestroyParticlesInShape`: there is no public mirror, but `query_aabb_with_particles` + `Shape::test_point` + `mark_particle_for_destruction` are sufficient for a narrow helper (engine or scene-local). Group shove does not need a new World method if scenes copy `ParticleGroupView::member_ids()` and call the existing contiguous-range force/impulse APIs the way Jelly Drop already does.

**Primary recommendation:** Wave engine APIs only for a thin destroy-in-shape helper if scene-local duplication is ugly; otherwise ship five `liquidfun-wasm` scene modules (shared `soup` builder), catalog/previews/credits, and Chromium smoke extensions—keeping `MAX_ADVANCE_STEPS = 4` and pinned test radii.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory present in this repo. [VERIFIED: filesystem]

Applicable constraints instead come from Bright Builds standards referenced by CONTEXT.md and `standards-overrides.md`:

- Scene construction stays in `liquidfun-wasm`; physics stays in `liquidfun` (`standards/core/architecture.md`). [CITED: standards/core/architecture.md]
- Safe Rust, typed errors, focused unit tests for new seams (`standards/languages/rust.md`, `standards/core/testing.md`). [CITED: standards/core/testing.md]
- Playground shell uses shadcn-solid (Tailwind v4, Kobalte, Corvu drawer); keep `skipLibCheck` (`standards-overrides.md`). [VERIFIED: standards-overrides.md]
- Dark playground chrome already established; do not redesign the shell (`standards/core/frontend-ui.md`, D-03). [CITED: standards/core/frontend-ui.md]
- Local checks before commit; `just web-player-smoke` is the browser proof (`standards/core/verification.md`, D-25). [CITED: standards/core/verification.md]
- Hobby scope: no package publication from this phase (`PROJECT-SCOPE.md` / CONTEXT). [CITED: 28-CONTEXT.md]

## Standard Stack

### Core

| Library / surface | Version / pin | Purpose | Why Standard |
|-------------------|---------------|---------|--------------|
| `liquidfun` | workspace crate | Bodies, fixtures, joints, particles | Only allowed physics [VERIFIED: crates/liquidfun] |
| `liquidfun-wasm` | private crate | Scene factory, hooks, session | Existing playground bridge [VERIFIED: crates/liquidfun-wasm] |
| SolidJS player + catalog | `web/` | Controls, hash routes, previews | Locked shell (D-03) [VERIFIED: web/src/catalog/scenes.ts] |
| Upstream oracle tests | commit `7f204021…` | Geometry / interaction recognition | D-22 pin [VERIFIED: `git -C third_party/liquidfun rev-parse HEAD`] |

### Supporting (already present — do not replace)

| Surface | Purpose | When to Use |
|---------|---------|-------------|
| `ParticleSystemDef::with_damping` | Impulse 0.2 / Wave 0.2 / Stirrer 1.0 | Match pinned JS defs [VERIFIED: system_definition.rs] |
| `apply_particle_force_range` / `apply_particle_linear_impulse_range` | Impulse whole-blob shove | Contiguous group members [VERIFIED: particle_object/system.rs] |
| `mark_particle_for_destruction` | Carve pockets | After shape containment [VERIFIED: particle_object/particle.rs] |
| `query_aabb_with_particles` + `Shape::test_point` | Destroy-in-shape | Narrow helper input [VERIFIED: world/query.rs, collision/shape.rs] |
| `PrismaticJointDef` + `create_joint` / `destroy_joint` | Soup Stirrer rail | Toggle lifecycle [VERIFIED: joint defs + world/joint.rs] |
| `RevoluteJointDef::with_motor` + `set_revolute_motor_speed` | Wave / Theo | Live motors [VERIFIED: revolute.rs] |
| `DistanceJointDef::with_frequency(10)` + `with_damping_ratio(0.5)` | Theo legs | Soft suspension [VERIFIED: distance_pulley_mouse.rs] |
| `FilterData::new(_, _, -1)` | Theo self-collision off | Chassis/legs/wheel [VERIFIED: FilterData group_index] |
| `ChainShape` / `EdgeShape` / `PolygonShape` / `CircleShape` | Impulse box, soup noodles, walls | Already exported [VERIFIED: collision.rs] |
| `BodyMassData` / `set_body_mass_data` | Soup edge-noodle mass | Match JS `SetMassData` [VERIFIED: body_object.rs] |
| `apply_body_force_to_center` | Stirrer oscillatory force | `on_advance` only [VERIFIED: body_object.rs] |
| Playwright Chromium via `just web-player-smoke` | Gate | D-25 [VERIFIED: Justfile] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Scene-local carve helper | Public `World::destroy_particles_in_shape` | Public helper better if Drawing (Phase 29) will reuse; discretion allows either |
| Group force via member range | New `apply_particle_group_force(group, v)` | Nice API sugar; not required—Jelly Drop already proves the range path |
| Soft distance joints | Welded polygons | Forbidden by D-13 — walker will not recognize |
| Motor speed from JS rAF | Session `on_advance` | Forbidden by D-17 — breaks pause/4-step honesty |
| Raise `MAX_ADVANCE_STEPS` or shrink radii | Honest stutter / caps | Forbidden by D-20 |

**Installation:** No new npm/Cargo packages required for the locked design. [VERIFIED: existing workspace]

**Version verification:** Upstream submodule HEAD is `7f20402173fd143a3988c921bc384459c6a858f2`. Bun `1.4.2`, Cargo `1.97.0`, just `1.48.0` available locally. [VERIFIED: shell probes]

## Architecture Patterns

### Recommended Project Structure

```
crates/liquidfun/src/           # optional: destroy_particles_in_shape helper + unit test
crates/liquidfun-wasm/src/scene/
├── scene.rs                    # MODIFY: SceneId + parse + build_scene
├── soup_family.rs              # NEW: shared Soup basin/group/solids/carve
├── soup.rs                     # NEW: ACT-01 watch-first
├── soup_stirrer.rs             # NEW: ACT-02 compose + prismatic + stir
├── impulse.rs                  # NEW: ACT-03 box + group shove
├── wave_machine.rs             # NEW: ACT-04 motorized tank
└── theo_jansen.rs              # NEW: ACT-05 walker (+ optional tests/ submodule)
web/src/catalog/scenes.ts       # APPEND five SCENE_IDS + records
web/src/catalog/previews.tsx    # APPEND five Static preview SVGs
web/e2e/player-helpers.ts       # SCENE_PATHS + timeout if needed
web/e2e/player.spec.ts          # POINTER_CONTROL entries for 3 interactive scenes
web/e2e/shell.spec.ts           # Update “eleven demos” copy → sixteen when chrome changes
```

### Pattern 1: Checked scene-id factory growth

**What:** Extend `SceneId`, `parse_scene_id`, and `build_scene` together with `SCENE_IDS` so the catalog cannot advertise an unbuildable id.
**When to use:** Every new playground scene (established Phases 26–27).
**Example:** Mirror `SceneId::RigidParticles` wiring in `crates/liquidfun-wasm/src/scene.rs`. [VERIFIED: scene.rs]

### Pattern 2: Shared soup builder composition

**What:** Private `soup_family` builds basin ground (`m_ground`), water box group (radius `0.035`), carves under circle + two boxes, places three edge noodles with custom mass. Soup returns that world; Stirrer calls the same builder then adds paddle carve, prismatic to ground, damping `1.0`, stir hooks.
**When to use:** ACT-01 / ACT-02 (D-19).
**Upstream:** JS `new TestSoup()` then stirrer; C++ `SoupStirrer : public Soup` with `m_ground`. [VERIFIED: testSoup.js, SoupStirrer.h]

### Pattern 3: Destroy-in-shape carve

**What:** For each fixture shape at its body transform: compute AABB → `query_aabb_with_particles` → keep only `Particle` hits whose position passes `shape.test_point(transform, p)` → `mark_particle_for_destruction`. Run before first presented frame (during build or first advance before capture). Never enable `with_destruction_by_age` on these scenes.
**When to use:** Soup solids, Stirrer paddle pocket (D-15).
**Anti-copy:** Water Wheel emitter age destruction. [CITED: .planning/research/PITFALLS.md]

### Pattern 4: Group shove (Impulse)

**What:** On pointer up inside box `[-2,2]×[0,4]`, direction = normalize(pointer − boxCenter). Copy `particle_group_view(group).member_ids()`. If `force`: `apply_particle_force_range(system, members, direction * 1.0 * n)`. If `impulse`: `apply_particle_linear_impulse_range(..., direction * 0.005 * n)`. Outside box: no-op.
**When to use:** ACT-03 (D-06, D-16).
**Why multiply by `n`:** Rust `prepare_force` divides total force by member count; JS Impulse also scales by `GetParticleCount()` before `ApplyForce` / `ApplyLinearImpulse`. [VERIFIED: force.rs; testImpulse.js; jelly_drop.rs]

### Pattern 5: Live revolute motors in `on_advance`

**What:** Create revolute with `with_motor(true, speed, max_torque)`. Wave Machine: each `on_advance` does `t += 1/60` then `set_revolute_motor_speed(joint, 0.05 * cos(t) * π)`. Theo Jansen: build with motor on at `+2.0`; live preset flips sign via `set_revolute_motor_speed`.
**When to use:** ACT-04 / ACT-05 (D-12, D-17).
**Wrong pattern:** Water Wheel `enable_motor: false` and no motor writes. [VERIFIED: water_wheel.rs tests]

### Pattern 6: Prismatic rail toggle

**What:** Store `Option<JointId>`. Attach: `PrismaticJointDef` ground→paddle, `local_axis_a = (1,0)`, `collide_connected = true`, `local_anchor_a = paddle position`. Detach: `destroy_joint`. Action name and pointer up in soup AABB both call the same toggle. Stir force only when joint is `Some` and paddle in soup and speed &lt; 2.0.
**When to use:** ACT-02 (D-05, D-18).
**Upstream guards:** `InSoup` box `y∈(-1,2)`, `x∈(-3,3)`; force mag 10; oscillation 0.2 Hz; max speed 2. [VERIFIED: testSoupStirrer.js]

### Pattern 7: Watch-first vs interactive catalog contracts

**What:** `controls.length === 0` → watch-first (Soup, Wave Machine). Nonempty controls → interactive smoke expects a gesture map entry. Actions use `action(id, label)`; live presets use `runtimePreset` with `recreates: false` returning `ControlEffect::Live`.
**When to use:** Catalog + e2e (D-04–D-07). [VERIFIED: scenes.ts; player.spec.ts]

### Anti-Patterns to Avoid

- **Copy Water Wheel revolute for Wave/Theo:** motor-off hub never rocks/walks. [VERIFIED: water_wheel/tests.rs]
- **JS-side carving or shove:** violates D-14 / PITFALLS destroy/force fakes. [CITED: PITFALLS.md]
- **Enable destruction-by-age on Soup\*:** deletes static soup. [CITED: PITFALLS.md]
- **Forked Soup geometries:** Stirrer must share builder (D-19).
- **Weld Theo legs:** skip soft distance → collapse/jitter (D-13).
- **Silent radius inflation** on Impulse/Wave (`0.025`): only with documented playground adaptation (D-20).
- **Raise catch-up above 4:** `MAX_ADVANCE_STEPS` / `MAX_STEPS_PER_FRAME` stay 4. [VERIFIED: session.rs; clock.ts]
- **Restore `.catalog-card`:** shell e2e asserts count 0 (D-03).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Particle carve under shape | JS position deletion | AABB query + `test_point` + mark destroy | Native lifecycle; compaction |
| Whole-blob shove | Per-particle JS loops / camera shake | Contiguous range force/impulse | Already transactional; matches LF group APIs |
| Soft leg suspension | Custom spring forces | `DistanceJointDef` frequency/damping | Solver-owned softness |
| Prismatic rail | Manual velocity clamps as “rail” | Create/destroy `PrismaticJoint` | Correct DOF removal |
| Motor rocking | CSS/canvas animation of tank | `set_revolute_motor_speed` in `on_advance` | Sim-time + pause honesty |
| Self-collision filters | Skipping fixtures | `FilterData` groupIndex −1 | Upstream pattern |

**Key insight:** This phase is mostly WASM scene authorship on existing engine seams; only destroy-in-shape is a convenience gap, and even that is composable without a new solver.

## Common Pitfalls

### Pitfall 1: Soup solids overlap fluid (no carve)
**What goes wrong:** Broth looks like Particles-with-props; solids sink through packed water.
**Why:** Skipping `DestroyParticlesInShape` after placing fixtures.
**How to avoid:** Carve under each solid before first frame; unit-test particle-free pocket.
**Warning signs:** No destroy/mark calls in soup builder; density under fixtures unchanged after build.

### Pitfall 2: Impulse only pokes one particle
**What goes wrong:** Blob does not slosh as a unit.
**Why:** Copying Jelly Drop’s nearby-only pointer poke instead of full `member_ids` range.
**How to avoid:** Always apply to full contiguous group; assert group `linear_velocity` changes.
**Warning signs:** Call sites of `apply_particle_linear_impulse` (singular) in Impulse.

### Pitfall 3: Force magnitude off by factor of N
**What goes wrong:** Feeble shove or explosive velocities.
**Why:** Forgetting Rust distributes total force by dividing by count while JS pre-multiplies by N.
**How to avoid:** Pass `direction * magnitude * member_count` into range APIs (match testImpulse.js).
**Warning signs:** Using magnitude `1.0` / `0.005` without `* n`.

### Pitfall 4: Wave Machine static tank / wrong motor pattern
**What goes wrong:** Water sits still; or motor updated from wall clock outside session.
**Why:** Copying Water Wheel motor-off; or updating speed in SolidJS rAF.
**How to avoid:** `with_motor(true, …)` at create; mutate only in `on_advance` with `t += dt` per session step.
**Warning signs:** Empty `on_advance`; JS timers writing motor speed.

### Pitfall 5: Stirrer joint leak / unbounded force
**What goes wrong:** Toggle twice leaves orphan joints or free paddle + continuous 10 N force → runaway.
**Why:** Toggle flips a bool without `destroy_joint`, or applies force when joint is none without speed guard.
**How to avoid:** `Option<JointId>`; force only when attached + InSoup + speed &lt; max; test toggle×2 + Reset remount.
**Warning signs:** `create_joint` without matching destroy path.

### Pitfall 6: Theo Jansen collapse or self-collision
**What goes wrong:** Legs shred, lock, or explode; walker cannot step.
**Why:** Missing soft distance joints or `groupIndex = -1`.
**How to avoid:** Port CreateLeg distance set (freq 10, damp 0.5) and filters; motorized revolute between wheel and chassis.
**Warning signs:** Only polygons + one revolute; no `DistanceJointDef`.

### Pitfall 7: Frame-budget hitch under packed radii
**What goes wrong:** Tab hitch-loops; temptation to cut counts or raise step cap.
**Why:** Impulse/Wave radius `0.025` packs ~1.2k+ particles; Soup `0.035` similar.
**How to avoid:** Keep radii; plateau/honest notes; never raise `MAX_ADVANCE_STEPS`; optional timeout bump for 16-scene smoke only.
**Warning signs:** PR diffs changing Dam Break Medium or clock cap.

### Pitfall 8: Catalog / shell copy drift
**What goes wrong:** Footer still says “eleven demos”; smoke timeouts; missing POINTER_CONTROL.
**Why:** Phase 27 left eleven-scene chrome; interactive map is explicit.
**How to avoid:** Update `SCENE_IDS`, paths, footer copy, `ALL_SCENE_TIMEOUT_MS` if needed, and gesture map for stirrer/impulse/theo.
**Warning signs:** shell.spec still expects “All eleven demos”.

## Code Examples

Verified patterns from this repository and pinned upstream (abbreviated):

### Group contiguous force/impulse (Jelly Drop → Impulse)

```rust
// Source: crates/liquidfun-wasm/src/scene/jelly_drop.rs (member_ids + range)
// Impulse scale: third_party/.../testImpulse.js ApplyImpulseOrForce
let members = world.particle_group_view(group)?.member_ids().to_vec();
let n = members.len() as f32;
let vector = direction * magnitude * n;
world.apply_particle_force_range(system, &members, vector)?;
// or apply_particle_linear_impulse_range for push-mode=impulse
```

### Destroy-in-shape sketch (composable)

```rust
// Source APIs: World::query_aabb_with_particles, Shape::test_point,
// World::mark_particle_for_destruction [VERIFIED in-tree]
let aabb = shape.compute_aabb(transform, shape.child_index(0)?)?;
let mut doomed = Vec::new();
world.query_aabb_with_particles(aabb, |occurrence| {
    if let WorldQueryOccurrence::Particle(hit) = occurrence {
        // resolve position; if shape.test_point(transform, pos) { doomed.push(id) }
    }
    QueryDirective::Continue
})?;
for id in doomed {
    let _ = world.mark_particle_for_destruction(id);
}
```

### Wave Machine motor update

```rust
// Source: testWaveMachine.js Step — motorSpeed = 0.05 * cos(time) * Math.PI
// Wire through SceneHooks::on_advance + World::set_revolute_motor_speed
self.time += 1.0 / 60.0;
world.set_revolute_motor_speed(
    self.joint,
    0.05 * self.time.cos() * std::f32::consts::PI,
)?;
```

### Soft distance + filter (Theo)

```rust
// Source: testTheoJansen.js CreateLeg — frequencyHz=10, dampingRatio=0.5, groupIndex=-1
DistanceJointDef::new(a, b)?
    .with_frequency(10.0)?
    .with_damping_ratio(0.5)?;
FilterData::new(category, mask, -1);
```

### Live runtime preset (ControlEffect::Live)

```rust
// Source: water_wheel / fountain apply_control patterns
match (name, value) {
    ("push-mode", "force" | "impulse") => {
        self.push_mode = PushMode::parse(value)?;
        Ok(ControlEffect::Live)
    }
    ("motor-direction", "forward" | "reverse") => {
        let speed = if value == "reverse" { -MOTOR_SPEED } else { MOTOR_SPEED };
        world.set_revolute_motor_speed(self.motor, speed)?;
        Ok(ControlEffect::Live)
    }
    _ => Err(SessionError::UnknownControl),
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Water Wheel motor-off revolute | Wave/Theo motor-on + live speed mutation | Phase 28 | Do not copy Water Wheel joint setup |
| Jelly Drop partial poke | Impulse full-group shove | Phase 28 | Different range selection |
| Eleven-scene catalog | Sixteen scene ids (append five) | Phase 28 | Update e2e chrome strings/timeouts |
| Named DestroyParticlesInShape (lfjs) | Compose query + mark (optional thin helper) | Ongoing gap | Enough for Soup carve; Drawing (29) may want public helper |

**Deprecated/outdated:**
- Treating freeglut key legends (`l`/`f`, `a`/`s`/`d`/`m`) as primary UI — map to labeled presets/actions (D-08).
- Claiming sealed C++ parity for recognition ports (D-23).

## Capability Gap Matrix

| Capability | Public API today | Enough for Phase 28? | Recommendation |
|------------|------------------|----------------------|----------------|
| Destroy-in-shape | No named API; compose query+test+mark | Yes | Prefer thin helper (discretion); focused carve test |
| Group ApplyForce / ApplyLinearImpulse | Range APIs + `member_ids` | Yes | No new API required; optional sugar |
| Prismatic create/destroy | Yes | Yes | Use directly |
| Revolute motor mutate | Yes | Yes | Use from hooks only |
| Soft distance joints | Yes (`with_frequency` / `with_damping_ratio`) | Yes | Required for Theo |
| Filter groupIndex | Yes | Yes | −1 on walker parts |
| Edge noodle mass | `set_body_mass_data` | Yes | Match Soup mass 0.1 at midpoint |
| Particle damping | `with_damping` | Yes | Impulse/Wave 0.2; Stirrer 1.0 |
| Destruction-by-age | Exists | Do not use here | Soup\* stay age-off |

[VERIFIED: codebase grep 2026-09-22]

## Upstream Recognition Cheatsheet (pin `7f204021`)

| Scene | Radius | Must-see setup | Interaction |
|-------|--------|----------------|-------------|
| Soup | 0.035 | Basin polygons; water box 2×1 at (0,1); carve circle r=0.1 @ (0,0.5); boxes ±1; three edge noodles | Watch |
| Soup Stirrer | soup + damp 1.0 | Circle paddle r=0.4 @ (0,0.7); carve; prismatic on ground x-axis; Step force 10 rotating | Toggle joint (action+click in soup) |
| Impulse | 0.025, damp 0.2 | Chain loop box; group box 0.8×1.0 @ (0,1.01) | Click in box → force (def) / impulse |
| Wave Machine | 0.025, damp 0.2 | Dynamic hollow box; revolute motor maxTorque 1e7; fill 0.9×0.9 @ (0,1) | `motorSpeed = 0.05*cos(t)*π` |
| Theo Jansen | 0.2, damp 0.2 | Ground+walls; 40 balls; chassis/wheel; 6 CreateLeg; motor 2.0; particle slab 7×0.5 @ y=15 | Reverse motor speed sign |

Approximate particle loads at 2×radius spacing: Impulse ~1280, Wave ~1296, Soup ~1600, Theo slab ~70 — within existing 10240 frame cap. [ASSUMED: spacing ≈ 2×radius for order-of-magnitude only; exact sampler may differ]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Exact particle sampler spacing ≈ 2×radius for cost estimates | Upstream Cheatsheet | Counts differ; still must not cut counts for smoothness |
| A2 | JS `g_groundBody` in testSoupStirrer equals Soup basin ground semantically (C++ uses `m_ground`) | Pattern 6 | Wrong body choice for prismatic — prefer Soup’s basin static body explicitly |
| A3 | Optional public destroy-in-shape is better deferred until Drawing if scene-local helper stays short | Capability Gap | Phase 29 may rework helper location |

**If empty rows needed confirmation:** A1–A3 are low risk; locked decisions already allow discretion on helper placement.

## Open Questions

1. **Public vs scene-local destroy-in-shape**
   - What we know: Composable APIs exist; D-15 allows either; Drawing (29) will need erase.
   - What's unclear: Whether Phase 28 should invest in `World` API now.
   - Recommendation: Prefer a small public helper with one unit test if implementation is &lt; ~40 lines; else scene-private in `soup_family` and revisit in Phase 29.

2. **Theo speed magnitude control**
   - What we know: Discretion allows optional magnitude beside direction.
   - What's unclear: Whether it helps mobile visitors.
   - Recommendation: Ship direction-only first; add magnitude only if walking feels stuck after recognition port.

3. **Smoke timeout for 16 scenes**
   - What we know: Phase 27 raised `ALL_SCENE_TIMEOUT_MS` to 220_000 for eleven scenes.
   - What's unclear: Whether 16 scenes need another bump.
   - Recommendation: Measure in implementation; bump only if Chromium smoke times out—do not change physics caps.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Bun | web typecheck / smoke scripts | ✓ | 1.4.2 | — |
| Cargo / Rust | scene + engine tests | ✓ | 1.97.0 | — |
| just | `web-player-smoke` | ✓ | 1.48.0 | invoke `bun scripts/web-build.ts player-smoke` |
| Upstream submodule | inspiration geometry | ✓ | `7f204021…` | — |
| Playwright Chromium | D-25 gate | ✓ via smoke recipe | (repo-pinned) | — |

**Missing dependencies with no fallback:** None identified.

**Missing dependencies with fallback:** None.

Step 2.6 complete for toolchain deps; no blocking external services.

## Security Domain

`security_enforcement` not set to `false` in `.planning/config.json` (absent → enabled). [VERIFIED: config.json]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Playground is static/WASM, no login |
| V3 Session Management | no | No server sessions |
| V4 Access Control | no | No multi-tenant resources |
| V5 Input Validation | yes | Allowlisted scene ids, control names/values, finite pointer coords (`SessionError`) [VERIFIED: session.rs] |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for WASM playground scenes

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed control/pointer input | Tampering | Typed allowlists; reject unknown; finite checks |
| Unbounded particle growth | Denial of service | Existing max particle counts / frame cap; no Drawing paint this phase |
| Joint/force runaway (Stirrer) | Denial of service | Speed + InSoup guards; destroy joint on toggle |
| XSS via credits URLs | Spoofing | Keep host-locked implementation paths; pinned GitHub inspiration links only |

## Validation Architecture

Skipped — `workflow.nyquist_validation` is `false` in `.planning/config.json`. [VERIFIED: config.json]

Planner should still require (per D-15/16/18/25 and `standards/core/testing.md`):

- Focused Rust tests: Soup carve pocket; Impulse group momentum; Stirrer toggle×2 + Reset; Wave motor speed changes with sim `t`; Theo soft joints + motor reverse.
- Catalog Vitest Record tables extended for sixteen scene ids (Phase 27 lesson: incomplete tables break typecheck).
- `just web-player-smoke` Chromium: five new open/play/pause/reset; eleven regression; one gesture each for stirrer/impulse/theo.

## Sources

### Primary (HIGH confidence)
- In-tree public APIs: `crates/liquidfun/src/world/particle_object/system.rs`, `world/query.rs`, `world/joint/revolute.rs`, `joint/definition/*`, `collision/shape.rs`
- WASM patterns: `crates/liquidfun-wasm/src/scene.rs`, `session.rs`, `jelly_drop.rs`, `water_wheel.rs`
- Catalog/e2e: `web/src/catalog/scenes.ts`, `web/e2e/player.spec.ts`, `web/e2e/shell.spec.ts`
- Pinned upstream JS/C++ at `7f204021…`: `testSoup.js`, `testSoupStirrer.js`, `testImpulse.js`, `testWaveMachine.js`, `testTheoJansen.js`, `Soup.h`, `SoupStirrer.h`
- Phase research: `.planning/research/FEATURES.md`, `PITFALLS.md`, `ARCHITECTURE.md`
- Locked decisions: `.planning/phases/28-interaction-seams/28-CONTEXT.md`

### Secondary (MEDIUM confidence)
- Approximate particle counts from 2×radius grid heuristic (A1)
- Phase 27 timeout/chrome lessons from `.planning/STATE.md`

### Tertiary (LOW confidence)
- None material; Context7/Brave/Exa/Firecrawl unavailable in this session — all critical claims verified in-repo or at pinned submodule.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — APIs and pins verified in-tree
- Architecture: HIGH — matches Phases 26–27 factory + research ARCHITECTURE
- Pitfalls: HIGH — cross-checked with PITFALLS.md + Water Wheel anti-pattern tests

**Research date:** 2026-09-22
**Valid until:** 2026-10-22 (stable playground/engine surface; reopen if particle force semantics or joint APIs change)
