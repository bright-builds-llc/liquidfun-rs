# Phase 47: Archimedes Screw - Research

**Researched:** 2026-10-14
**Domain:** Kinematic vane-train mechanism in the private `liquidfun-wasm` bridge, plus the per-scene catalog checklist (Rust + SolidJS web)
**Confidence:** HIGH for the engine facts, the wrap safety and the integration sites; MEDIUM for the geometry numbers (starting points that the spike and tuning tests confirm)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Vane mechanism and wrap
- **D-01:** The vanes are **kinematic** bodies driven by velocity every fixed step in `on_advance`. They are never dynamic bodies on a revolute motor. One revolution advances the vanes by exactly one pitch, so axial speed = pitch × rpm / 60.
- **D-02:** The research-recommended first candidate is one kinematic vane-train body that moves along the tube axis. After one pitch it snaps back exactly one pitch with `set_body_transform`, so the vane lattice is geometrically unchanged at the snap. The fallback is closed-loop paddles that run up through the tube and return outside it, below the tube floor. Each paddle's velocity is set every step toward its path point. Plan 1 starts with a short, time-boxed wrap and leak spike that measures both candidates against written pass rules: no particles behind walls, no evictions, no burst of particle speed at the wrap, and pockets that visibly hold water. Keep the first candidate that passes and record the measurements in the plan summary.
- **D-03:** A vane never appears, disappears or teleports inside water. Any wrap or snap point lies in a sealed housing or dry region that particles cannot reach. Vane ends are embedded in the thick static tube walls (kinematic and static bodies do not collide), so the vane closes the tube gap.
- **D-04:** Clearance and tunneling guards follow PITFALLS 7. Vanes are at least about 3 particle diameters thick. Any vane-to-wall clearance stays between about 1 and 1.5 particle diameters, or the vane overlaps the wall. A `const _: () = assert!(MAX_AXIAL_SPEED < PARTICLE_DIAMETER / SIM_DT)` with margin guards the maximum speed, as in `hydraulic_fountain.rs`.
- **D-05:** Vanes are slanted, not perpendicular to the axis, so in side view they read as the flight lines of a helix. Exact slant is Claude's discretion. Some leak-back past the vanes is real physics and is acceptable as long as the basin gains water.

#### Water loop and reverse
- **D-06:** The tube is closed (floor and roof). Its lower end is submerged in the reservoir, and its upper end opens into the upper basin below the basin's overflow lip, so the screw is symmetric: forward lifts water and reverse scoops it back down.
- **D-07:** The basin has an overflow weir. Water above the lip spills into a return chute made of static walls that drops it back into the reservoir. No particles are teleported, emitted or destroyed. The chute follows the Liquid Bubbler and Stacked Drip side-return precedent.
- **D-08:** Size the total water so the reservoir still covers the tube inlet when the basin is at its lip and every pocket in the tube is full. The reservoir never runs dry at default or maximum speed.

#### Speed and direction controls
- **D-09:** Control `screw-speed`, labeled "Screw speed", is a live HUD `range` spinner in rpm (`kind: "range"`, `recreates: false`, `surface: "hud"`, `widget: "spinner"`). The Rust hook returns `ControlEffect::Live`. It lives in a new `web/src/catalog/archimedes-screw-speed.ts`, copied from `washing-machine-speed.ts`. The minimum is 0 (stopped), and the default and maximum are tuned in native tests. The default must deliver the first pocket into the basin within about 5 s at gravity 10. The maximum stays under the D-04 tunneling assert with margin and is about twice the default. The research starting point is a default of about 20 to 30 rpm, with no default above 100 rpm.
- **D-10:** Control `screw-direction`, labeled "Direction", is a live runtime preset with options `forward` ("Forward", the default) and `reverse` ("Reverse"), following the Theo Jansen `motor-direction` precedent. Reverse negates the axial velocity at the current speed. Speed and direction are independent, and the hooks state applies both in `on_advance`.
- **D-11:** No scene-specific pointer behavior. `apply_pointer` keeps the default, and dragging the screw is an anti-feature.

#### Geometry and scale
- **D-12:** Use toy meter scale like the diver: particle radius about 0.04 to 0.06 m, thick overlapping static polygon walls, a tube tilted about 30° (allowed 25 to 35°), and an interior tube gap of about 6 to 8 particle diameters. The pitch must be long enough that each pocket holds a visible clump of water, about half full at default speed. Keep the scene at or below about 3,000 particles. Exact dimensions, pitch, particle radius and count are Claude's discretion within this budget.
- **D-13:** The scene is wide. The landscape `viewBounds` frames the reservoir, tube, basin and chute. The portrait frame in `web/src/catalog/portrait-bounds.ts` is sized to the action: the tube plus both water bodies, walls included. Some vertical margin is acceptable for a wide scene.
- **D-14:** Optional decoration (Claude's discretion): a drawn shaft line along the tube axis in `collect_segments`. The total must stay within the frame caps (512 segments, 48 circles). No helix overlay.

#### Catalog, copy and cues
- **D-15:** Use id `archimedes-screw`, module `archimedes_screw`, variant `ArchimedesScrew`, title "Archimedes Screw" and catalog family `curiosity`. Append the record to `web/src/catalog/scene-records-curiosity.ts` and the preview to `web/src/catalog/previews-curiosity.tsx`, after the diver in catalog order. Credits say "original experimental scene", and the source is `sceneSource("archimedes_screw.rs")`.
- **D-16:** The caption says plainly that flat vanes stand in for a turning helix in a 2D side view. For example: "Slanted vanes slide up a tilted tube and carry pockets of water from the low reservoir into the upper basin, which overflows back down a chute. Flat vanes stand in for a turning helix in this 2D side view." Write a hint instead of `WATCH_FIRST_HINT`, for example "Use Screw speed to speed up or stop the screw, and Direction to run it in reverse. Labeled controls also work from the keyboard." Do not claim rotation, torque or a 3D helix.
- **D-17:** The README preview and survey are watch-first at default settings, with no cues. The 10 s clip must show at least one pocket climbing the full tube and spilling into the basin. Check this against the generated clip and adjust the default speed, not a cue, if it fails. Add the README plan in `web/scripts/readme-svg/plans.ts`, the survey entry, and the demo-media `SCENE_CAPTURE_PLANS` entry as idle.
- **D-18:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed (commit only the new WebP and SVG for this scene). The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

#### Tests and evidence
- **D-19:** Native release-mode tests in the existing `liquidfun-wasm` lib test binary cover each of these:
  - A 60 s run at gravity 10 and default speed gains water in the basin, has no particle behind a wall, has no evictions, and keeps the reservoir covering the inlet.
  - The same 60 s run at maximum speed meets the same checks.
  - A reverse run that starts after the basin has gained water shows the basin count falling and the reservoir count rising.
  - Speed 0 holds the vanes still.
  - The control allowlist rejects bad values.
  Assertions use counts and regions with tolerance, not exact trajectories or step indices.
- **D-20:** A shorter stability smoke at gravity 2 and 80 asserts no leaks and no evictions only. Lifting is not required there (the g80 pressure cap squashes columns; see the Phase 46 D-07a lesson).
- **D-21:** Before the phase closes, `just scene-fingerprint-gate` must report all 26 protected ids unchanged with `archimedes-screw` under "added (accepted)". Record `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene archimedes-screw` in the phase summary.
- **D-22:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 27 protected ids unchanged.
- **D-23:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player and hud-safe-frame), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact pitch, vane slant and thickness, tube dimensions, tilt within 25 to 35°, particle radius and count, default and maximum rpm, and basin and chute shapes, within D-04, D-09 and D-12.
- Which wrap candidate ships, decided by the D-02 spike evidence.
- Preview drawing style and whether to draw a shaft line.
- Plan breakdown and wave ordering, provided the spike comes first and the baseline growth commit (D-22) is last and separate.
- Whether to split `archimedes_screw.rs` into `archimedes_screw/geometry.rs` and `archimedes_screw/tests.rs` (the `foo.rs` + `foo/` layout) to stay under the file-length limit.

### Deferred Ideas (OUT OF SCOPE)
- A sand or powder material and a tilt-angle control with spill-back above about 40° (SCREW-03, future requirement).
- A rotating helix overlay drawn along the shaft (research P3 differentiator).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SCREW-01 | Player sees moving slanted vanes in a tilted trough carry separate pockets of water from a low reservoir up to a higher basin, with the water returning so the scene keeps running. | Engine facts on the wrap (§ Engine Facts), the probe results (§ Wrap Probe), the geometry sketch with housings, openings, basin, weir and chute (§ Geometry Sketch), the water-budget rule (Pitfall 6), and the 60 s release test shapes (§ Code Examples) |
| SCREW-02 | Player can change the screw's speed live and run it in reverse so it carries water back down. | The live `screw-speed` spinner and `screw-direction` preset sites (§ Site Map), the Rust allowlist and parser (§ Code Examples), web replay of live values after a gravity rebuild (`web/src/player/worker-controls.ts:44-53`), and the reverse probe (no crush when vanes leave a housing under water) |
| SCENE-01..03 (shared) | Catalog record, preview, portrait frame, README preview, legend, gravity slider, HUD safe frame, ≤ ~3,000 particles, honest caption | § Site Map (every R/W/S/T/E row), Pitfalls 8 to 10 (copy rules), particle budget in § Geometry Sketch |
| PREP-01 (shared gate) | Every protected fingerprint unchanged, new id added | § Fingerprint Gate |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Follow `docs/adding-a-scene.md` for every Rust and web site. The fingerprint baseline grows only in its own commit. [VERIFIED: AGENTS.md, docs/adding-a-scene.md]
- No change to `crates/liquidfun` (engine). Scene mechanisms are bridge code in `crates/liquidfun-wasm` on existing public `World` APIs. No new dependencies, cue kinds, bridge APIs or worker messages. [VERIFIED: ROADMAP shared rules]
- Rust: no `unwrap()` in production code; propagate with `?`/`map_err`; `expect()` only in tests; `maybe_` prefix for `Option`s; `foo.rs` + `foo/` module layout; tau, not pi, for angles (`std::f32::consts::TAU`); `let...else` for early returns. [VERIFIED: global CLAUDE.md]
- Tests: one concern per test, with Arrange/Act/Assert comments. [VERIFIED: global CLAUDE.md]
- Files ≤ 628 physical lines (`FILE_LINE_LIMIT = 628` in `scripts/bright-builds-check.ts:8`). [VERIFIED]
- TypeScript: `camelCase`, `maybe` prefix for nullable values; bun runs the web tooling. [VERIFIED]
- Pre-commit Rust: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. [VERIFIED: global CLAUDE.md]
- `.planning/**` is never mdformatted. Run `just markdown-check` after changing non-GSD Markdown; the README gallery update from `just readme-svg` counts. [VERIFIED: CLAUDE.md Repo-Local Guidance]
- A new scene needs a `README_SVG_PLANS` entry, a `PORTRAIT_VIEW_BOUNDS` entry and the shared scale legend (automatic). New HUD chrome must carry `data-hud-obstacle`; the spinner gets it from shared plumbing. [VERIFIED: AGENTS.md]
- Captions must be honest (Transparency, SCENE-03). [VERIFIED]
- New tests go into the existing `liquidfun-wasm` lib test binary; a new test binary costs about 2 minutes of macOS scanning. [VERIFIED: CONTEXT.md, user memory "macOS launch stalls"]
- Run plan executors one at a time on the main tree, without worktrees. [VERIFIED: user memory]
- Executor SUMMARY files need lifecycle frontmatter (`generated_by`, `lifecycle_mode`, `phase_lifecycle_id`, `generated_at`). [VERIFIED: user memory]

## Summary

**The snap-one-pitch wrap is safe in this engine. Ship candidate A (one kinematic vane train that snaps back one pitch) unless the spike's in-scene measurements say otherwise.** Particle-to-body collision rebuilds each particle's ray start from the body's *previous* transform, which is the body sweep at alpha 0 (`world/particle_coupling.rs:498-503`, used on particle iteration 0 only, `particle/solver/boundary/collision.rs:24-34`). `World::set_body_transform` rebuilds the sweep with `c0 == c` (`world/body/state.rs:222-252`). The next particle pass therefore sees previous == current and treats the snapped body as momentarily static (`particle_coupling.rs:268`). A snap is never read as a one-pitch velocity. Upstream LiquidFun matches: `b2Body::SetTransform` sets `m_xf0 = m_xf` (`third_party/liquidfun/.../b2Body.cpp:433`). The only cost is that one step loses the moving-wall remap (a `v·dt` displacement of about 0.01 m at 0.5 m/s).

A throwaway native release probe confirmed this. It ran 270 to 591 particles at r = 0.05 m, 2 particle iterations, gravity 10, in a 0.7 m closed tube with 0.3 m vanes embedded 0.1 m in the walls. Steps with a snap never showed a higher particle speed than steps without one, and a long continuous lattice with no snap gave the same leak counts in every case. The real hazard is **geometric**: a vane that slides into a closed end housing crushes the water in front of it. Without an opening, about 40 particles ended up inside vanes and about 15 inside walls, both with and without snapping. With a floor opening at least one pitch long that runs up to the housing face, whether it drains into air or into a submerged reservoir, and in reverse when vanes scoop under water, **zero** particles entered a vane or wall and no particle was lost.

So the scene design rule is this. Both ends of the vane track end inside solid static housings (bottom cap, top cap). Each housing is preceded by a floor opening at least `pitch + vane overhang` long that reaches the housing face: the inlet over the reservoir and the outlet over the basin. The housing faces are cut parallel to the slanted vanes. All other registration work copies the Phase 46 diver checklist. The live controls copy Washing Machine's `drum-speed` (HUD spinner) and Theo Jansen's `motor-direction` (runtime preset in the panel). Web already replays live values after a gravity rebuild.

**Primary recommendation:** Plan 1 builds the scene core with candidate A on the geometry sketch below and runs the D-02 spike tests against the written pass rules. Use pure translation along a constant tube axis, so there is no per-step trig. Plan 2 adds the controls and the tuned D-19/D-20 release tests. Plan 3 is catalog and web wiring. Plan 4 covers README assets, the gates and the scene-spot numbers. Plan 5 is the separate baseline-growth commit.

## Standard Stack

No new dependencies (locked). Everything is in the repository.

| Component | Version | Purpose | Source |
|-----------|---------|---------|--------|
| `liquidfun` (in-repo) | workspace 0.0.0 | `World`, `BodyDef` (`BodyType::Kinematic`, `with_sleeping_allowed(false)`, `with_fixed_rotation(true)`), `PolygonShape` (`new`, `oriented_box`, `test_point`), `FixtureDef`, `ParticleSystemDef` (`with_radius`, `with_damping`, `with_pressure_strength`, `with_strict_contact_check`) | [VERIFIED: codebase] |
| `liquidfun-wasm` (private) | workspace | `SceneHooks`, `SessionCore`, `scene_spot`, gravity slider | [VERIFIED: codebase] |
| Rust | 1.97.0 (pinned in `rust-toolchain.toml`; the system default is 1.91.1, which the 1.92 MSRV rejects) | build/test | [VERIFIED: probe build] |
| bun / just / ffmpeg / webpmux / dwebp | installed | web, recipes, README assets, clip review | [VERIFIED: `command -v`] |
| mdformat | 1.0.0 + gfm 1.0.0 + frontmatter 2.1.2 at `~/.local/bin/mdformat` | `just markdown-check` | [VERIFIED] |

**Installation:** none.

## Engine Facts (Question 1)

| Fact | Evidence | Consequence |
|------|----------|-------------|
| Particle CCD against a body casts a ray from `start` to `end = p + dt·v`. On particle iteration 0, `start` maps the particle from the body's previous frame into its current frame; later iterations use the particle position. | `crates/liquidfun/src/particle/solver/boundary/collision.rs:9-35`, `:68-80` | A moving body pushes particles by its displacement since the previous step [VERIFIED] |
| The previous transform is `body.state.sweep().transform_at(0.0)`, and the current transform is `body.state.transform()`. | `crates/liquidfun/src/world/particle_coupling.rs:498-503` | A sweep reset erases the displacement [VERIFIED] |
| `set_body_transform` installs `Sweep::new(local, c, c, angle, angle, 0)`, so `c0 == c`. | `crates/liquidfun/src/world/object/body_object.rs:446-471` → `world/body/state.rs:222-252` (sweep at `:235-242`) | **A snap is not seen as velocity** [VERIFIED] |
| The rigid solve writes `c0 = old center, c = new center`. | `world/body/state.rs:138-175` (`:158-165`), called from `world/island.rs:252` | Normal steps couple at `v·dt` [VERIFIED] |
| Particles solve before the rigid contact solve in `World::step`. | `crates/liquidfun/src/world/step/execution.rs:177-210` | The step after a snap sees previous == current. The next step is normal [VERIFIED] |
| When previous == current, the fixture is queried as static. | `particle_coupling.rs:268, 279` | No widened query and no fling [VERIFIED] |
| Upstream behaves the same: `SetTransform` sets `m_xf0 = m_xf` and `m_sweep.c0 = m_sweep.c`. | `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.cpp:423-446`; `b2ParticleSystem.cpp:2801-2808` uses `m_xf0` | Parity behavior, not a port quirk [VERIFIED] |
| Contacts need at least one dynamic body, so kinematic vanes pass through static walls. | `crates/liquidfun/src/world/contact_manager/update.rs:26-27`; Hydraulic Fountain plates cover the walls (`hydraulic_fountain.rs:62-64`) | D-03 embedding works [VERIFIED] |
| `set_body_linear_velocity` wakes a non-static body when the velocity is nonzero. | `body_object.rs:115-129` | Still use `with_sleeping_allowed(false)` (Hydraulic Fountain precedent, `hydraulic_fountain.rs:246`) [VERIFIED] |
| Session order per fixed step: `on_advance` → eviction → `world.step` → `on_after_step`. | `crates/liquidfun-wasm/src/session.rs:227-257` | Snap and set velocity inside `on_advance` [VERIFIED] |
| The particle speed clamp is about the diameter times the substep inverse dt (0.1 m × 120 = 12 m/s at r 0.05 with 2 iterations). | The probe observed an exact 12.000 m/s cap on free-falling and crushed particles | A "burst" pass rule should compare wrap steps with other steps, not use a fixed absolute limit [VERIFIED: probe; formula ASSUMED] |

### Wrap Probe (throwaway, not committed)

A scratch crate depending on `crates/liquidfun` by path ran `cargo build --release` and finished in 0.2 s. Setup: r 0.05, damping 0.2, pressure strength 0.25, strict contact check, 2 particle iterations, `StepConfiguration::new(1/60, 8, 3)`. Tube interior 0.7 m tall and 6 m long, walls 0.4 m thick, end housings 1.6 m. Vanes 0.3 m thick, embedded 0.1 m into the floor and roof, pitch 0.8 m, 9 vanes. "Snap" snaps one pitch. "Long" is a continuous 15-vane lattice with no snap. Counts start after a 60-step settle. "Inside" means a particle center more than r/2 inside the polygon.

| Variant | Speeds (m/s) | Snap-step max particle speed vs other steps | Inside vane / inside wall (snap) | Inside vane / inside wall (long) | Lost |
|---|---|---|---|---|---|
| 30° tilt, full pockets, closed top housing (no outlet), 4 s | 0.33 | 0.50 vs 1.26 | 0 / 0 (water not yet at housing) | 0 / 0 | 0 |
| same | 0.67 | 0.97 vs 1.59 | **38 / 16** (crushed at housing) | **40 / 15** | 0 |
| 30° tilt, floor outlet 1.0 m (1.25 pitch) before the housing, draining into air, 10 s | 0.33, 0.67 | no burst | 0 / 0 | 0 / 0 | 0 (all drained) |
| Flat tube, gravity down, outlet over a reservoir whose surface is above the tube floor (submerged), 20 s | 0.33, 0.67 | 3.2–4.4 vs 4.5–4.7 | 0 / 0 | 0 / 0 | 0 of 591 |
| Same submerged geometry in reverse: vanes leave the housing under water and their embedded tips re-enter the floor edge, 6 s | −0.33, −0.67 | 1.3–1.7 vs 2.8–5.7 | 0 / 0 | 0 / 0 | 0 of 471 |

Conclusions: (1) snapping adds no speed burst; (2) pinching against a closed housing is the failure mode, independent of the wrap method; (3) an outlet or inlet opening at least one pitch long that runs up to the housing face removes it, even when submerged. [VERIFIED: probe run 2026-10-14]

A probe bug is itself a pitfall: particles spawned overlapping a kinematic vane launched at the 12 m/s clamp on the first step. See Pitfall 3.

## Geometry Sketch (Candidate A, starting numbers)

Work in a tube frame: axis `u = (cos 30°, sin 30°) = (0.8660254, 0.5)` and normal `n = (−0.5, 0.8660254)`. A world point is `O + s·u + t·n`. Here `s` is the axial distance from the bottom housing face and `t` is the height above the floor line. Hard-code `u` and `n` as `f32` literals documented as `TAU / 12`. Add a debug unit test that they match `(TAU / 12.0).cos()/.sin()` within 1e-6. The scene then calls no trig per step and builds the same on native and wasm.

```
                                         top housing (solid, s ∈ [L, L+C])
                         roof (t ∈ [h, h+W]) ─────────────────────/####/
                 /  /  /  /  /  /   ← slanted vanes, pitch p    /####/
  bottom        /  /  /  /  /  /                               /####/
  housing  ####/──────────────────── floor (t ∈ [−W, 0]) ──┐  /####/
  (solid)  ####   inlet opening        sealed run           │  outlet opening
  s∈[−C,0] ####   s ∈ [0, a]                                │  s ∈ [L−b, L]
     ~~~~~~~~~~~~~~ reservoir ~~~~~~~~~~                    basin ── weir lip ─┐
                                                                               │ chute
     ←──────── sloped return floor (water runs back left) ──────────────────────┘
```

| Parameter | Start value | Rule |
|---|---|---|
| Particle radius r | 0.05 m (d 0.1) | Same as the diver; D-12 |
| Tilt θ | 30° (`TAU / 12`) | D-12 |
| Interior gap h | 0.7 m (7 d) | D-12: 6 to 8 d |
| Floor and roof thickness W | 0.3–0.4 m | ≥ 3 d, and thicker than the vane embed so no tip pokes out |
| Vane thickness w | 0.3 m (3 d), measured normal to the vane | D-04 |
| Vane embed e | 0.1 m (1 d) into the floor and roof | D-03 overlap. Keep it short: the tip pinch at an opening edge scales with e |
| Vane slant α | 20° from `n`, top leaning uphill | D-05; a pure visual, since a sealed closed tube holds water at any slant |
| Pitch p | 1.0 m | The pocket clear length is about 0.68 m, so the area is about 0.48 m² full (about 85 particles at stride 0.075) and about 42 half full |
| Tube length L (housing face to housing face) | 5.0 m (lift 2.5 m) or 6.0 m (lift 3.0 m) | `L = (N + 1)·p − C` for N + 1 vanes |
| Housing length C | 1.0 m | Must contain the full vane s-extent `w/cos α + (h + 2e)·tan α ≈ 0.65 m` with margin |
| Inlet length a, outlet length b | 1.4 m each | ≥ p + vane overhang `(h + e)·tan α ≈ 0.29` + margin. Each opening must run up to the housing face |
| Lattice | vanes at `s_k = s₀ + k·p + φ`, k = 0..N, `s₀ = −C/2`, φ ∈ [0, p) | At φ = 0, vane 0 is fully inside the bottom housing. At φ → p, vane N is at `s₀ + (N + 1)p = L + C/2`, fully inside the top housing |
| Housing faces | Cut parallel to the vanes, not perpendicular to the axis | A vane then slides in flush with no closing wedge |
| Default rpm | 30 (v = 0.5 m/s, 2 s per pocket) | At L = 5: inlet → spill ≈ 7.2 s and seal → spill ≈ 4.4 s, which meets D-17's 10 s clip. At L = 6 the inlet → spill time is 9.2 s, too tight; use 36 rpm there |
| Max rpm | 60 (v = 1.0 m/s) | D-04: `d / dt = 6 m/s`, so `assert!(MAX_AXIAL_SPEED < 0.5 · PARTICLE_DIAMETER / SIM_DT)` leaves a 3× margin |

Example world anchors for `O = (−4.0, −0.3)` and L = 6 (computed): floor at s = 0 → (−4.00, −0.30); inlet seal point (floor, s = 1.4) → (−2.79, 0.40); outlet start (floor, s = 4.6) → (−0.02, 2.00); floor at s = L → (1.20, 2.70); roof at s = L → (0.85, 3.31); bottom housing far corners (−4.72, −1.06) and (−5.37, 0.07); top housing far corners (2.21, 2.94) and (1.56, 4.07). For L = 5, shift the top end back by `u` (−0.87, −0.5).

**Water bodies (starting shape):**
- **Reservoir:** a trough whose left part is flat (floor about y = −1.0 under the bottom housing; the housing may overlap the floor, since static polygons may overlap). Its floor then **slopes up to the right** under the tube (about 20°), so chute water runs back left by gravity. Target surface: at or above the inlet seal height (y ≈ 0.4 with the anchors above). Pockets then seal about half full, and the level falls as the basin fills (self-regulating).
- **Basin:** under the outlet opening. Floor about 0.7 m below the lip. The left wall top overlaps the tube floor slab. The weir lip on the right wall sits above the lowest outlet floor point (y ≈ 2.0 at L = 6), so the outlet is submerged when the basin is at its lip (D-06), and below the top housing's underside.
- **Chute:** a vertical channel outside the basin's right wall (outer wall about 0.5 m to the right). It drops onto the high end of the sloped reservoir floor, which carries the water back to the inlet. This is a gravity return, with no lift plate as in Bubbler or Drip; it shares only their "no teleport" rule.
- **Budget (D-08):** `W_total ≥ basin_to_lip + N_sealed·pocket_full + chute_in_transit + reservoir_at_seal_height`. With these numbers it is about 300–400 (basin) + about 250 (pockets) + about 900–1,200 (reservoir) ≈ 1,500–2,000 particles, inside the 3,000 budget. Pre-fill the sealed pockets about half at load, so the first pocket reaches the basin within about one cycle (≈ 2 s, D-09 and SCENE-02). The basin starts empty.

**Candidate B (fallback, sketch only):** closed-loop paddles, each its own kinematic body, using **translation only**. A paddle runs up the tube and **slides lengthwise along its own slant into the thick floor slab** just before the top mouth. It returns down *inside* the solid floor slab, which is dry because particles cannot be inside a static polygon, and slides lengthwise back out near the bottom mouth. Its faces move parallel to themselves at the transitions, so nothing pinches. The slab must be thicker than the paddle length, about 1 m. Expect about 12 bodies for L = 5. Velocity is set toward the path point each step, as D-02 says. Water resting on a retracting paddle falls back, so the retraction must happen right at the open mouth. Use B only if A fails the spike.

## Architecture Patterns

### Recommended module layout
```
crates/liquidfun-wasm/src/scene/
├── archimedes_screw.rs            # build(), ArchimedesScrewHooks, controls, collect_segments (credit path)
└── archimedes_screw/
    ├── geometry.rs                # tube frame constants, wall/housing/vane polygons, regions, spawn points, clipping
    ├── tests.rs                   # debug-safe: parse/allowlist, phase math, geometry invariants, speed-0, build at g2/g80
    └── flow_tests.rs              # release-only (cfg_attr(debug_assertions, ignore)): D-02 spike rules, D-19, D-20
```
Hydraulic Fountain is 529 lines without its tests, so keep the geometry out of the hooks file. [VERIFIED: wc]

### Pattern 1: Intent in hooks, motion in `on_advance` (D-01, D-10)
- Hooks state: `train: BodyId`, `phase: f32` in `[0, p)`, `rpm: u16`, `reverse: bool`.
- `apply_control` only stores intent and returns `Live`. `on_advance` computes the signed step `±p·rpm/60·SIM_DT`. A pure `advance_phase(phase, step) -> (next_phase, snap)` decides whether to snap, where `snap` is −p, 0 or +p. On a snap, call `set_body_transform(origin + (phase + snap)·u, 0.0)`. Then set the velocity toward the path point: `(origin + next·u − current) / SIM_DT`, clamped to `MAX_AXIAL_SPEED`, which is the closed-loop form of `hydraulic_fountain.rs:200-204`. [VERIFIED: pattern; ASSUMED: exact shape]
- Pure translation along a constant `u`: angle 0, `with_fixed_rotation(true)`, `with_sleeping_allowed(false)`.
- Speed 0 gives velocity 0 and no snap, so the vanes hold still (D-19).

### Pattern 2: Snap only inside housings (D-03)
- The lattice geometry guarantees that the vanes added or removed at a snap (vane 0 at φ = 0 and vane N at φ = p) lie fully inside solid housings. Add a debug unit test: for both snap positions, every vertex of that vane is inside a housing polygon (`PolygonShape::test_point`).
- Add a geometry test for each opening: it is at least `p + overhang` long and ends at the housing face (no floor between the opening and the face).

### Pattern 3: Drawing (D-14)
- Draw the static wall outlines (inner faces, as in Hydraulic Fountain's `WALL_SEGMENTS`, or polygon loops), plus vanes **clipped to the open tube interior** (`0 ≤ s ≤ L`, `0 ≤ t ≤ h`, using four half-plane clips on a parallelogram). Vanes then appear to slide into the housings, and the snap is invisible. An optional shaft line runs at `t = h/2` from s = 0 to L. The total is well under 512 segments (about 60 wall + 6 × 6 vane + 1). Frame drawing reads only `collect_segments`, never fixtures. [VERIFIED: `scene.rs:241`, Phase 46 research]

### Pattern 4: Live controls (copy two precedents)
- Rust `screw-speed`: a whole-number rpm parser copied from `washing_machine.rs:257-271` (rejects empty, non-digit and leading-zero values, and values above max). Rust `screw-direction`: match `"forward"` or `"reverse"` exactly (`theo_jansen.rs:409-423`). Anything else returns `Err(SessionError::UnknownControl)`.
- `build(presets)` must return `Err(UnknownControl)` for any non-empty `presets`. `build_scene` already strips `gravity` (`scene.rs:254-257`). Live values are never passed at build.
- Web: after a recreating control (gravity), `applyWorkerControl` replays every stored non-recreating control value (`web/src/player/worker-controls.ts:44-53`), so speed and direction survive a gravity change with no new code. [VERIFIED]
- The `screw-direction` default is the first option, `forward`, through `initialPresetValue` (`web/src/components/scene-controls.ts:69-90`). No `DEFAULT_PRESET_VALUES` entry is needed (W4 is optional). [VERIFIED]
- Surfaces: the range spinner goes on the HUD, and the preset goes in the panel (`sceneControlsForSurface`, `scene-controls.ts:27-41`). [VERIFIED]

### Pattern 5: Release-only flow tests with session counts
- Session-level runs: `SessionCore::create(SceneId::ArchimedesScrew)`, `apply_control("gravity", "80")` (rebuild), then re-apply speed and direction, `advance` in chunks of `MAX_ADVANCE_STEPS = 4`, and read with `session.read_particles(|world, system| …)`. **Evictions:** `session.live_particle_count()? == session.particle_count()` at the end (eviction is the only way particles disappear here). [VERIFIED: `cartesian_diver/motion_tests.rs:40-77, 299-320`]
- Spike and wrap-burst tests: expose a concrete constructor such as `fn build_parts() -> Result<(World, ParticleSystemId, ArchimedesScrewHooks), SceneError>`, which `build` wraps. Tests then read `hooks.phase` or a snap counter and step `world.step(...)` themselves. `BuiltScene.hooks` is a trait object (`scene.rs:183-188`).
- Gate each with `#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]`. Run with `cargo test -p liquidfun-wasm --release --lib archimedes_screw`.

### Anti-Patterns to Avoid
- **A closed housing without an opening of at least one pitch in front of it.** The probe shows about 40 particles crushed into vanes and walls.
- **Per-step `sin`/`cos`** or rotating bodies (native/wasm divergence; Theo Jansen diverged at step 70).
- **Spawning particles that overlap vanes or walls.** They launch at the 12 m/s clamp on the first step.
- **A "lift force" on particles**, an emitter or teleporting water (SCREW-01, D-07 anti-features).
- **Exact step indices in assertions** (D-19). Use counts, regions and tolerance windows.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Point-in-wall checks | Your own polygon math | `PolygonShape::test_point(Transform::IDENTITY, p)` plus `point_to_segment_distance` for depth (pattern in `scene/tesla_valve/tests.rs:442-480`) | Already used for leak tests |
| Eviction detection | Hooking into eviction | `live_particle_count()` against `particle_count()` | Eviction happens only in `SessionCore::advance` |
| Water-surface height | Ad-hoc max y | The p95-percentile idea from `cartesian_diver/geometry.rs` (`water_surface_y`). Copy it locally rather than editing diver code | Robust to splashes |
| rpm parsing | New parser rules | Copy `parse_drum_rpm` (`washing_machine.rs:258-271`) | Same canonical tokens the web range control emits |
| Live replay after gravity | Rust-side persistence | The existing web replay (`worker-controls.ts:44-53`) | Already shipped |
| Speed guard | Runtime checks | `const _: () = assert!(...)` (`hydraulic_fountain.rs:37`) | Compile-time |

## Common Pitfalls

### Pitfall 1: Water crushed into a closed housing
**What goes wrong:** Particles end up inside vanes or walls, then leak or are ejected. **Why:** A kinematic vane is infinitely strong, and the shrinking volume in front of it has no outlet. **Avoid:** Put an inlet or outlet floor opening of at least `p + overhang` before each housing, running up to the housing face, and cut the housing faces parallel to the vanes. **Warning signs:** Nonzero "inside vane" or "inside wall" counts near a housing. [VERIFIED: probe]

### Pitfall 2: Treating the snap as the risk
The snap is invisible to particle CCD (Engine Facts). Spend the spike time on the geometry (housings, openings, basin, chute), not on alternative wrap mechanics. Keep a "wrap-step speed ≤ max non-wrap speed" assertion as a regression guard. [VERIFIED]

### Pitfall 3: Spawn overlaps
Initial particle points inside a vane, wall or housing launch at the velocity clamp. Filter spawn points with `test_point` on every static and vane polygon, expanded by r (or a distance check of at least r). [VERIFIED: probe bug]

### Pitfall 4: High gravity
At g = 80 the particle pressure cap (0.25 × critical, `particle/solver/pressure.rs`) squashes columns (Phase 46 D-07a). D-20 already limits g 2 and g 80 to "no leaks, no evictions". Any height target, such as the inlet-covered surface, must be measured from the settled surface, never from fixed bottle-style heights. [VERIFIED: user memory, 46-01-SUMMARY]

### Pitfall 5: Vane tip entering a floor edge
Where a vane passes from an opening onto solid floor (inlet in forward, outlet in reverse), its embedded tip closes a small wedge against the floor's edge face. The probe saw no crush with a 0.1 m embed. Keep `e` about 1 d, and do not thicken the tip. [VERIFIED: reverse probe]

### Pitfall 6: Reservoir runs dry or dead pools collect water
**What goes wrong:** The lift stops, or water parks somewhere it never returns from (for example on the tube roof or in a V between the tube underside and a wall). **Avoid:** Use the D-08 budget formula. Every surface water can reach must drain by gravity to the basin or the reservoir. Assert in the 60 s tests that the reservoir surface (p95 of reservoir-region particles) is at or above the inlet seal height, and that a "lost-region" count (particles outside the reservoir, tube, basin and chute regions) stays near 0.

### Pitfall 7: README timing (D-17)
At 20 Hz samples × 3 steps (`SVG_SAMPLE_HZ = 20`), the clip is 600 steps. With pre-filled pockets, the first spill happens in under 2 s. A pocket scooped at about 1 s must spill before 10 s: `(L − b)/v ≤ ~8.5 s`. L = 5 at 30 rpm gives 7.2 s; L = 6 needs about 36 rpm. Check the generated clip and adjust the default rpm, never add a cue. [VERIFIED: Phase 46 research on sample timing; arithmetic above]

### Pitfall 8: Catalog copy rules (`web/tests/scenes.test.ts`)
- Descriptions are locked UI-SPEC strings, and every original scene ends with "This is an original experimental scene." Append it to the D-16 caption. [VERIFIED: `scenes.test.ts:55-68`]
- The hint must contain "Labeled controls also work from the keyboard." (the screw is not in `WATCH_FIRST_SCENE_IDS`), and the D-16 example already does. [VERIFIED: `scenes.test.ts:115-128, 280-300`]
- Add a screw honesty guard like `DIVER_FORBIDDEN_CLAIMS`. Forbid "torque", "motor", "rotating", "3d", "suction" and "lift force". Require "Flat vanes stand in for a turning helix". [ASSUMED: wording; the pattern is VERIFIED at `scenes.test.ts:138-146, 323-338`]

### Pitfall 9: e2e loop membership
`POINTER_CONTROL` is a `Partial` record. Washing Machine has no scene pointer behavior and is still listed as `{ gesture: "click", control: "Drum speed" }`. The session accepts the pointer either way, and `activateLabeledControl` finds the `spinbutton` and fills its max. Use `{ gesture: "click", control: "Screw speed" }`. If "Direction" were used instead, `SELECT_NEXT_VALUE` would need `Direction: "reverse"`. [VERIFIED: `player.spec.ts:40-56`, `player-helpers.ts:58-65, 477-506`]

### Pitfall 10: Tooling on this Mac
`just readme-svg` needs `LIQUIDFUN_README_FONT_FILE`. Review frames with `webpmux` and `dwebp`, because ffmpeg cannot decode the animated WebP. Put `~/.local/bin` first on PATH for mdformat. Commit only this scene's new SVG and WebP. [VERIFIED: 46-04-SUMMARY, user memory]

## Code Examples

### Phase advance and snap (pure, unit-testable)
```rust
/// Returns the next lattice phase in [0, PITCH) and the snap to apply first (−PITCH, 0 or +PITCH).
fn advance_phase(phase: f32, step: f32) -> (f32, f32) {
    let next = phase + step;
    if next >= PITCH {
        return (next - PITCH, -PITCH);
    }
    if next < 0.0 {
        return (next + PITCH, PITCH);
    }
    (next, 0.0)
}

fn on_advance(&mut self, world: &mut World, _system: ParticleSystemId) -> Result<(), SessionError> {
    let (next, snap) = advance_phase(self.phase, axial_speed(self.rpm, self.reverse) * SIM_DT);
    if snap != 0.0 {
        // The vane added or removed by this snap lies inside a solid housing (geometry test).
        world
            .set_body_transform(self.train, train_position(self.phase + snap), 0.0)
            .map_err(|_error| SessionError::StepFailed)?;
    }
    let current = world
        .body_snapshot(self.train)
        .map_err(|_error| SessionError::StepFailed)?
        .position();
    let velocity = clamp_speed((train_position(next) - current) * (1.0 / SIM_DT));
    world
        .set_body_linear_velocity(self.train, velocity)
        .map_err(|_error| SessionError::StepFailed)?;
    self.phase = next;
    Ok(())
}
```
`train_position(phase) = TRAIN_ORIGIN + AXIS * phase`, and `axial_speed(rpm, reverse) = ±PITCH * f32::from(rpm) / 60.0`. [ASSUMED: shape; the APIs are VERIFIED]

### Speed guard
```rust
const PARTICLE_DIAMETER: f32 = PARTICLE_RADIUS * 2.0;
const MAX_SCREW_RPM: u16 = 60;
// `f32::from` is not const; a widening u16 → f32 `as` cast is exact here.
const MAX_AXIAL_SPEED: f32 = PITCH * (MAX_SCREW_RPM as f32) / 60.0;
// One step of vane travel stays under half a particle diameter (D-04).
const _: () = assert!(MAX_AXIAL_SPEED < 0.5 * PARTICLE_DIAMETER / SIM_DT);
```

### Controls
```rust
fn apply_control(&mut self, _world: &mut World, _system: ParticleSystemId, name: &str, value: &str)
    -> Result<ControlEffect, SessionError> {
    match name {
        SCREW_SPEED_CONTROL => {
            let Some(rpm) = parse_screw_rpm(value) else { return Err(SessionError::UnknownControl) };
            self.rpm = rpm;
        }
        SCREW_DIRECTION_CONTROL => match value {
            "forward" => self.reverse = false,
            "reverse" => self.reverse = true,
            _ => return Err(SessionError::UnknownControl),
        },
        _ => return Err(SessionError::UnknownControl),
    }
    Ok(ControlEffect::Live)
}
```

### Leak check helper
```rust
/// True when `point` is more than `depth` inside `polygon` (a wall, housing or vane).
fn deeper_than(point: Vec2, polygon: &PolygonShape, transform: Transform, depth: f32) -> bool {
    if !polygon.test_point(transform, point).expect("finite point") {
        return false;
    }
    let local = transform.inverse_apply(point);
    let vertices = polygon.vertices();
    (0..vertices.len())
        .map(|i| point_to_segment_distance(local, vertices[i], vertices[(i + 1) % vertices.len()]))
        .fold(f32::INFINITY, f32::min)
        > depth
}
```

### D-19 test shape (release)
```rust
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn default_speed_lifts_water_for_sixty_seconds_without_leaks() {
    // Arrange
    let mut session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");
    let built = session.particle_count();
    let basin_before = basin_count(&session);

    // Act
    let report = run_checked(&mut session, SIXTY_SECONDS_STEPS); // per-chunk leak, lost-region and inlet-covered checks

    // Assert
    assert_eq!(session.live_particle_count().expect("count"), built, "no evictions");
    assert_eq!(report.max_inside_walls, 0);
    assert!(report.min_reservoir_surface >= INLET_SEAL_Y, "{report:?}");
    assert!(basin_count(&session) > basin_before + BASIN_GAIN_MIN, "{report:?}");
}
```

### Web control file and record
```ts
// web/src/catalog/archimedes-screw-speed.ts (copy of washing-machine-speed.ts)
export const SCREW_SPEED_MIN = 0;
export const SCREW_SPEED_MAX = 60;
export const SCREW_SPEED_STEP = 1;
export const SCREW_SPEED_DEFAULT = 30;
export const SCREW_SPEED_TICKS = [SCREW_SPEED_MIN, SCREW_SPEED_DEFAULT, SCREW_SPEED_MAX] as const;
export const SCREW_SPEED_CONTROL: SceneControl = {
  id: "screw-speed", label: "Screw speed", kind: "range", recreates: false,
  surface: "hud", widget: "spinner", min: SCREW_SPEED_MIN, max: SCREW_SPEED_MAX,
  step: SCREW_SPEED_STEP, defaultValue: SCREW_SPEED_DEFAULT, unit: "rpm",
  scale: "linear", ticks: SCREW_SPEED_TICKS,
};

// scene-records-curiosity.ts, after the diver
controls: withGravitySlider([
  SCREW_SPEED_CONTROL,
  runtimePreset("screw-direction", "Direction", [option("forward", "Forward"), option("reverse", "Reverse")]),
]),
```
Optionally, add a Rust test that `include_str!`s `archimedes-screw-speed.ts` and checks `SCREW_SPEED_DEFAULT` and `SCREW_SPEED_MAX` against the Rust constants (the pattern of `scene.rs:327`). No such cross-check exists for Washing Machine today. [VERIFIED: grep]

## Site Map (exact sites)

### Rust (`crates/liquidfun-wasm/src`)
| # | Site | Edit |
|---|------|------|
| R1 | `scene/archimedes_screw.rs` + `archimedes_screw/{geometry,tests,flow_tests}.rs` | New scene |
| R2 | `scene.rs:9-37` | `mod archimedes_screw;` (alphabetical: first) |
| R3/R5 | `scene.rs:46-73` `SceneId`, `scene.rs:258-285` `build_scene` | `ArchimedesScrew` + arm |
| R4 | `scene.rs:108-138` `parse_scene_id` | `"archimedes-screw" => Ok(SceneId::ArchimedesScrew)` |
| R6 | `scene.rs:144-151` `particle_iterations` | None expected (the probe was clean with 2) |
| R7 | `scene.rs:79-106` `ALL_SCENE_IDS` (append last); `scene/gravity_slider.rs:161-189` `variant_index` | `SceneId::ArchimedesScrew => 26`. Add to `ALL_SCENE_IDS` only together with web `SCENE_IDS` (order test `scene.rs:344-361`), as in 46-01/46-02 |
| R8 | `session/tests.rs:14-42` | Append the token (file is 581 lines) |
| R9 | `scene_spot.rs:46-108` `SURVEY_SCENES` | `("archimedes-screw", SceneId::ArchimedesScrew, SurveyCue::Default)` |
| R10 | `scene_spot/tests.rs:71-78` | No change (the scene is idle) |
| R12 | `session/escape.rs` | Not needed (geometry within about ±6 m; limits are 12 m fall and 48 m sideways) |
| R13 | `archimedes_screw/flow_tests.rs` | Release-only tests |

### Web
| # | Site | Edit |
|---|------|------|
| W1 | `web/src/catalog/scenes.ts:38` `SCENE_IDS` | Append `"archimedes-screw"` after `"cartesian-diver"` |
| W2 | `web/src/catalog/scene-records-curiosity.ts` | Record (copy, D-16 caption + "This is an original experimental scene.", hint, controls, `viewBounds: ARCHIMEDES_SCREW_VIEW_BOUNDS`, `sceneSource("archimedes_screw.rs")`, `[SHOWCASE]`) |
| W3 | `web/src/catalog/archimedes-screw-speed.ts` (new) | Above |
| W4 | `DEFAULT_PRESET_VALUES` | Not needed (`forward` is the first option) |
| W5 | `previews-curiosity.tsx` `ArchimedesScrewPreview()`; `previews.tsx` `case "archimedes-screw"` (the `never` guard fails until added) | Tilted tube, slanted vanes, two water bodies |
| W6 | `scene-view-bounds.ts` `ARCHIMEDES_SCREW_VIEW_BOUNDS` (re-export through `scene-records.ts` and `scenes.ts`, as the diver does); `portrait-bounds.ts:97` | Landscape covers the reservoir, tube, basin and chute. Portrait covers the tube plus both water bodies, walls included |
| S1 | `web/scripts/readme-svg/plans.ts:84-91` | `{ id: "archimedes-screw", cues: [] }` |
| S2 | `docs/assets/readme/archimedes-screw-10s.{svg,webp}`, README gallery | `just readme-svg`, review, `just markdown-check` |
| S3 | `web/scripts/demo-media/model.ts:288-294` | `{ id: "archimedes-screw", title: "Archimedes Screw", route: "/liquidfun-rs/#/scene/archimedes-screw", interactionStep: 180, action: { kind: "click", point: { x: 0.5, y: 0.5 } } }` (the idle-scene convention) |
| T1 | `web/tests/scenes.test.ts` (516 lines) | `UI_SPEC_DESCRIPTIONS`, `UI_SPEC_HINTS`, id and title lists, plus an honesty guard |
| T2 | `web/tests/scene-catalog-controls.test.ts:22-55` | Append `"gravity"` to `RECREATING_CONTROL_IDS`, and add a block asserting `[SCREW_SPEED_CONTROL, screw-direction preset, GRAVITY_CONTROL]` |
| T3 | `web/tests/scene-catalog-credits.test.ts:97, 215` | Path and `[SHOWCASE_HREF]` |
| T4 | `web/tests/portrait-bounds.test.ts:45-73, ~395` | `MIN_HEIGHT_FRACTION["archimedes-screw"]` (a wide frame: about 0.3–0.4), optionally `MIN_WIDTH_FRACTION`, plus wall points (housing far corners, basin lip, reservoir walls) |
| T6 | `web/tests/controls.test.ts:472-499` (529 lines) | A "keeps Screw speed on the HUD" test: HUD `["screw-speed"]`, panel `["screw-direction", "gravity"]`, step ±1 |
| E1 | `web/e2e/player-helpers.ts:56` | `"archimedes-screw": "/liquidfun-rs/#/scene/archimedes-screw"` |
| E2 | `web/e2e/player.spec.ts:56` | `"archimedes-screw": { gesture: "click", control: "Screw speed" }` |

**Already derived (no edit):** routing, sidebar, `ScaleLegend`, gravity slider, `hud-safe-frame.spec.ts` (iterates `SCENE_IDS`), shell counts, xtask scene list, `PAGE_SUMMARY`. [VERIFIED: docs/adding-a-scene.md]

## Fingerprint Gate

- Baseline: `docs/benchmarks/scene-fingerprints.json` holds 26 ids, `cartesian-diver` included (provenance git `89983a79b`, stamp `2026-10-14T19-29-46Z`, 60 warmup + 120 measured steps, 3 runs). [VERIFIED: jq]
- Gate: `just scene-fingerprint-gate` (`cargo xtask playground scene-fingerprint-gate`), or `--spot target/dam-break-perf/<stamp>/scene-spot.json` to reuse a capture. Expect "protected: 26/26 unchanged" and "added (accepted): archimedes-screw <hex>". [VERIFIED: justfile:185-186, 46-04-SUMMARY]
- Per-scene numbers: `cargo xtask playground scene-spot --scene archimedes-screw` → `start_particles`, `end_particles`, `median_ms_per_step` (the diver measured 0.58 ms at 1,936 particles). [VERIFIED]
- D-22: a separate commit appends `{ "id": "archimedes-screw", "fingerprint": "<16 hex>" }` in catalog order from a fresh full capture (`just playground-scene-spot`), updates `provenance`, and reruns the gate, which must report 27/27. [VERIFIED: docs/adding-a-scene.md]

## Recommended Plan Breakdown

| Plan | Content | Risk |
|---|---|---|
| 47-01 | **Spike + scene core (time-boxed).** Geometry module (tube frame, housings, openings, basin, weir, chute, sloped reservoir), vane train (candidate A), hooks with fixed default speed, spawn filter, `build_parts`. Register the parse/build/variant_index/session token only. Release spike tests against the D-02 pass rules: no particle deeper than r/2 in any wall, housing or vane after settle; no evictions; wrap-step max speed ≤ max non-wrap speed; a sealed mid-tube pocket holds at least about 15 particles; the basin gains water. Run them forward and reverse at default and 2× speed. If A fails, try B within the time box. Record the measurements and the decision in the SUMMARY | MEDIUM: geometry tuning (dead pools, budget) |
| 47-02 | Controls (`screw-speed`, `screw-direction`), drawing (clipped vanes, optional shaft), tuned constants, D-19/D-20 release tests, the README-timing check (inlet → spill within the clip) | LOW to MEDIUM |
| 47-03 | Catalog: `ALL_SCENE_IDS`/`SURVEY_SCENES` + web `SCENE_IDS`, record, speed file, preview, view and portrait bounds, README/demo plans, T1–T6, E1/E2 | LOW |
| 47-04 | `just readme-svg` + clip review, full gates (cargo, web unit, player + hud-safe-frame e2e, bright-builds, markdown), scene-spot numbers, `just scene-fingerprint-gate` (26/26 + screw added) | LOW |
| 47-05 | Single-file baseline growth commit, gate rerun 27/27 | LOW |

Execute sequentially on the main tree.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| PITFALLS 7 warned that `set_body_transform` "drops it inside the pool and blasts particles away" | Verified: a snap resets `c0`, so there is no blast. Only a *visible appear or disappear inside water* or a *closing housing* is harmful | This research | Candidate A is the default |
| Closed-loop paddles with turning ends (angles) | If needed, paddles that retract lengthwise into a solid floor slab (translation only) | This research | No native/wasm trig divergence |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The geometry sketch numbers (L, p, α, a, b, C, reservoir/basin/chute shapes) will pass in the real tilted scene, as the probe passed in simpler geometry | Geometry Sketch | Tuning iterations in 47-01. The probe already covered both submerged and free outlets |
| A2 | Pockets look "about half full" when the reservoir surface sits near the seal height | Geometry Sketch | Visual only. Adjust the total water or the inlet height |
| A3 | The velocity clamp is diameter × substep inverse dt (12 m/s here) | Engine Facts | The burst rule is relative anyway |
| A4 | 30 rpm default and 60 rpm max with L = 5 meet D-09 and D-17 | Geometry Sketch, Pitfall 7 | Adjust the rpm defaults (TS and Rust together) |
| A5 | Wording of the screw honesty guard list | Pitfall 8 | Copy review only |
| A6 | The Candidate B sketch is feasible | Geometry Sketch | Used only if A fails |

## Open Questions (RESOLVED)

1. **Reservoir surface metric.** Known: the diver uses a p95 surface. Unclear: which particles count as "reservoir" while the inlet section of the tube is flooded. Recommendation: define the reservoir region as the trough polygon minus the tube band, and measure p95 there.
   RESOLVED: adopted in 47-01-PLAN.md `<facts>` "Regions" (reservoir = trough interior minus the tube interior) and Task 3 `test_support.rs` `reservoir_surface_y` (p95 of reservoir-region y); 47-02 asserts it against `INLET_COVER_Y`.
2. **Exact basin and weir placement relative to the top housing underside.** The lip must be above the outlet floor's low point and below the housing underside. Recommendation: assert both in a geometry unit test.
   RESOLVED: adopted in 47-01-PLAN.md Task 1 `<behavior>`: the weir lip y is above the floor-line y at `q = L - b` and below the top housing's lowest vertex y (geometry unit test in `archimedes_screw/tests.rs`).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Tests, scene-spot, gate | ✓ | 1.97.0 via `rust-toolchain.toml` (system default 1.91.1) | — |
| bun | Web typecheck/unit, scripts | ✓ | installed | — |
| just | Recipes | ✓ | `/opt/homebrew/bin/just` | Run the underlying commands |
| ffmpeg, webpmux, dwebp | `just readme-svg`, clip review | ✓ | installed | — |
| mdformat 1.0.0 + gfm | `just markdown-check` | ✓ | `~/.local/bin/mdformat` (put it first on PATH) | — |
| DejaVu Sans font file | README WebP | via `LIQUIDFUN_README_FONT_FILE` | — | Documented override |
| Playwright Chromium | player + hud-safe-frame e2e | ✓ (Phase 46 used it) | cached | — |

## Verification Commands

| Req | Behavior | Command |
|-----|----------|---------|
| SCREW-01/02 (debug) | Parse/allowlist, phase math, housing/opening geometry, speed 0 holds, builds at g 2/80 | `cargo test -p liquidfun-wasm --lib archimedes_screw` |
| SCREW-01/02 (release) | Spike rules, 60 s default/max, reverse, g 2/80 smoke | `cargo test -p liquidfun-wasm --release --lib archimedes_screw` |
| Registration | Order, survey, gravity slider, session tokens | `cargo test -p liquidfun-wasm --lib -- scene::tests scene_spot gravity_slider session::tests` |
| SCENE-01..03 | Catalog, controls, credits, portrait, README plan, demo media, legend, copy | `cd web && bun run typecheck && bun run test:unit` |
| e2e | Player loops, HUD safe frame | `just web-build && cd web && CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts --project chromium` |
| README | Assets | `LIQUIDFUN_README_FONT_FILE=<ttf> just readme-svg && just markdown-check` |
| PREP-01 | Gate | `just scene-fingerprint-gate` |
| Repo | Format, lint, length | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && bun scripts/bright-builds-check.ts all` |

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Allowlisted control names; canonical whole-number rpm (0..=max, no leading zeros); exact `forward`/`reverse`; everything else `Err(UnknownControl)`, never a panic. `build` rejects unknown presets |
| V2/V3/V4/V6 | no | Static client-side playground |

| Threat | STRIDE | Mitigation |
|--------|--------|-----------|
| Out-of-range or malformed rpm (for example `"999"`, `"01"`, `"-5"`, `"1.0"`) | Tampering / DoS (tunneling) | Parser rejects. The compile-time speed assert bounds the maximum |
| Unknown control or preset | Tampering | `UnknownControl` |

## Sources

### Primary (HIGH, verified in this session)
- `crates/liquidfun/src/particle/solver/boundary/collision.rs`, `world/particle_coupling.rs`, `world/body/state.rs`, `world/object/body_object.rs`, `world/step/execution.rs`, `world/contact_manager/update.rs`: the wrap and collision mechanics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.cpp:423-446`, `b2World.cpp:463`, `Particle/b2ParticleSystem.cpp:2801-2808`: upstream parity.
- `crates/liquidfun-wasm/src/{scene.rs, session.rs, session/escape.rs, scene_spot.rs, scene/hydraulic_fountain.rs, scene/washing_machine.rs, scene/theo_jansen.rs, scene/cartesian_diver/*, scene/tesla_valve/*}`.
- `web/src/catalog/*`, `web/src/components/scene-controls.ts`, `web/src/player/worker-controls.ts`, `web/tests/*`, `web/e2e/player*.ts`.
- `docs/adding-a-scene.md`, `.planning/phases/46-cartesian-diver/*`, `.planning/research/v1.6-curiosity-scenes/*`.
- Native release probe (scratch crate, 2026-10-14): the snap, closed-housing, free-outlet, submerged-outlet and reverse variants.

### Secondary (MEDIUM)
- `.planning/research/v1.6-curiosity-scenes/FEATURES.md`: real-screw reference (about 30° tilt, below 100 rpm, about half-full pockets; vendor sources, LOW).

## Metadata

**Confidence breakdown:**
- Engine wrap facts: HIGH (code + upstream + probe)
- Integration sites: HIGH (grep of every diver site)
- Geometry numbers and defaults: MEDIUM (arithmetic plus a simplified-geometry probe; the spike confirms them in the scene)
- Pitfalls: HIGH for crush and spawn (observed); MEDIUM for budget and dead pools

**Research date:** 2026-10-14
**Valid until:** 2026-11-13 (stable in-repo code)
