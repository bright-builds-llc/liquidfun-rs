# Phase 48: Fake Ferrofluid - Research

**Researched:** 2026-10-14
**Domain:** A scripted, gravity-aware per-particle force field in the private `liquidfun-wasm` bridge, a shared mass-calibrated force helper (reused by Phase 49), and the per-scene catalog checklist (Rust + SolidJS web)
**Confidence:** HIGH for the engine facts, the force calibration (measured), the integration sites and the field design (measured in a throwaway probe); MEDIUM for the exact geometry numbers (probe starting points that the in-scene spike confirms)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Field model and spikes
- **D-01:** The pull is a scripted fake field, never a pure point attraction toward the magnet (research anti-feature: it makes a ball or flings droplets). Its three parts are: (a) a **lift along the field axis**, pointing out of the puddle opposite to `world.gravity()`, whose strength falls off smoothly with distance from the magnet; (b) a **spatial modulation** that concentrates the lift into separate spike columns, either `1 + a·cos(k·(s − s_mag))` along the surface or N fixed "spike tip" targets spaced across the dish and centered on the magnet; and (c) a weak lateral pull toward the magnet along the surface, so the puddle creeps toward an off-center magnet and spike tips lean toward it. Here `s` is the coordinate across the dish, perpendicular to gravity.
- **D-02:** The profile is bounded and smooth. It has a softened core (no singularity), a smooth falloff to an outer radius, and an acceleration cap of at most about 2 to 3 times the gravity magnitude at maximum strength. Affected particles get explicit velocity-proportional drag, so the field settles and holds instead of oscillating (PITFALLS 5).
- **D-03:** The liquid is one particle group with `TENSILE | VISCOUS` flags (the Liquid Timer precedent) so spikes stay cohesive. Viscous strength stays at or below 1.5, because the step fails above about 2.0.
- **D-04:** Target 5 to 9 spikes across the dish at default strength. The roadmap floor is at least 4 separate surface peaks. Spikes appear within about 1 s of the magnet energizing at gravity 10 and collapse within about 1 to 1.5 s after it switches off.
- **D-05:** Plan 1 starts with a short, time-boxed stability and appearance spike. It measures the cos-modulation candidate and the fixed spike-tip candidate against written pass rules: at least 4 separate peaks within about 1.5 s at default strength and gravity 10; a collapse to a flat surface within about 1.5 s after off; and, during a 60 s hold at maximum strength, no eviction, no failed step and a max particle speed that does not grow over time. Keep the first candidate that passes and record the measurements in the plan summary. If neither passes, stop and record a decision checkpoint (an amendment like Phase 46 D-07a) instead of loosening the floor silently.

#### Magnet state and pointer
- **D-06:** The hooks hold `magnet_on: bool` and `magnet_position: Vec2`. On load the magnet is on and rests under the center of the dish floor.
- **D-07:** Pointer `Down` anywhere on the canvas turns the magnet on and moves it to the pointer. `Move` while pressed updates the position. The position is clamped to the scene bounds, and non-finite input is rejected. `Up` and `Cancel` are handled identically: the magnet switches off and parks at its rest position under the dish, so the spikes collapse. The next press, or the Magnet action, turns it back on. There is no hover.
- **D-08:** No physics runs in `apply_pointer`, `apply_action` or `apply_control`. They store intent only. `on_advance` applies the field once per fixed step (Color Mixer pattern and ARCHITECTURE rule), so behavior does not depend on frame rate.

#### Strength and on/off controls
- **D-09:** Control `magnet-strength`, labeled "Magnet strength", is a live HUD `range` spinner in percent (`kind: "range"`, `recreates: false`, `surface: "hud"`, `widget: "spinner"`). The Rust hook returns `ControlEffect::Live`. It lives in a new `web/src/catalog/fake-ferrofluid-strength.ts`, copied from `archimedes-screw-speed.ts`. The range is 0 to 100. 0 means no pull, and 100 is the tuned safe maximum, which never detaches liquid. The default is tuned so the load shows 5 to 9 spikes, at roughly 60 to 70.
- **D-10:** Action `magnet`, labeled "Magnet", is a panel action that toggles `magnet_on` at the current position. It is the keyboard path for on and off and the hook for README cues (Diver `squeeze` precedent). Turning it on after a release energizes the magnet at its rest position.
- **D-11:** Strength and on/off are independent. A strength of 0 with the magnet on shows a flat puddle.

#### Shared gravity-aware force helper
- **D-12:** Add a small shared bridge module, for example `crates/liquidfun-wasm/src/scene/particle_force.rs`, that converts a per-particle acceleration into the force for one fixed step. It uses the computed particle mass (`density·(stride·2r)²`) and the particle-iteration factor, applies with `apply_particle_force` or `apply_particle_force_range` (range totals are multiplied by member count, as in `impulse.rs`), and exposes gravity-direction helpers built from `world.gravity()`. Phase 49 reuses it for the Rayleigh–Taylor extra weight. Do not reuse or "improve" `basin_family` or `soup_family`.
- **D-13:** Measure the calibration in the Plan 1 spike, because the research derived it from reading code, not from a run. A unit test checks that a known acceleration request produces that acceleration within tolerance, and a second checks that the lift direction follows a tilted gravity vector.
- **D-14:** Every field term scales with the gravity magnitude and points relative to the gravity direction. No term hard-codes −y, so the gravity slider (2 to 80) and phone tilt stay correct.

#### Geometry, color and marker
- **D-15:** Use toy meter scale. The particle radius is about 0.04 to 0.06 m, the dish is a shallow, wide static dish with thick overlapping walls, and the puddle depth is a few particle layers, enough to form tall spikes. Keep all geometry within about 10 m of the origin (eviction rule), and keep the scene at or below about 3,000 particles and about 1.5 ms/step native release. Exact sizes are Claude's discretion.
- **D-16:** The liquid color is a dark blue-gray or graphite violet, not black. It must stay readable on the `#071018` canvas after the 0.73 density-shade floor in Shaded blob (default), Water, circle wireframe (README) and the static preview. Check all four before the phase closes.
- **D-17:** The magnet marker is world content drawn through `collect_segments` and/or `collect_circles`, for example a small bar or horseshoe at `magnet_position`. When off, the marker shows a parked, inactive look, such as an outline only. It stays within the frame caps (512 segments, 48 circles). It is not HUD chrome and does not carry `data-hud-obstacle`.
- **D-18:** The scene is wide. The landscape `viewBounds` frames the dish plus enough space above it for spikes and a dragged magnet. The portrait frame in `web/src/catalog/portrait-bounds.ts` is sized to the dish, the spikes and the rest magnet, walls included.

#### Catalog, caption and cues
- **D-19:** Use id `fake-ferrofluid`, module `fake_ferrofluid`, variant `FakeFerrofluid`, title "Fake Ferrofluid" and catalog family `curiosity`. Append the record to `web/src/catalog/scene-records-curiosity.ts` and the preview to `web/src/catalog/previews-curiosity.tsx`, after the screw in catalog order. Credits say "original experimental scene", and the source is `sceneSource("fake_ferrofluid.rs")`.
- **D-20:** The caption calls the pull scripted. For example: "A dark liquid puddle sits over a magnet and rises into a row of spikes. The pull is a scripted magnetic pull that lifts the liquid into spike columns, not a simulated magnetic field." Write a hint instead of `WATCH_FIRST_HINT`, for example "Press and drag to move the magnet; let go to switch it off. Use Magnet to toggle it and Magnet strength to change the pull." Do not claim real magnetism, a field solve or measured physics.
- **D-21:** The README plan is watch-first at default settings. It cues the `magnet` action at about 6 s (off) and about 8 s (on), so the 10 s clip shows spikes, a collapse and a re-rise. Check the sample indices against the generated clip and adjust them if needed. The survey entry is `SurveyCue::Default`, because the resting magnet is the active, expensive state. The demo-media `SCENE_CAPTURE_PLANS` entry is idle. E2E `POINTER_CONTROL` gets an entry so the pointer-driven scene is not dropped into the watch-first loop.
- **D-22:** Every site in `docs/adding-a-scene.md` is filled. `just readme-svg` output is generated, committed and reviewed (commit only this scene's new WebP and SVG). The scale legend, gravity slider (`withGravitySlider`) and HUD safe frame come from the shared catalog plumbing.

#### Tests and evidence
- **D-23:** Native release-mode tests live in the existing `liquidfun-wasm` lib test binary, one concern per test:
  - On load at gravity 10 and default strength, at least 4 separate surface peaks form within about 1.5 s.
  - Dragging the magnet off-center moves the peak centroid toward it.
  - Pointer `Up` collapses the spikes to a flat surface within about 1.5 s.
  - Pointer `Cancel` collapses them the same way.
  - The `magnet` action off collapses them the same way.
  - Strength 0 with the magnet on keeps the surface flat.
  - A 60 s hold at strength 100, with the pointer held at three positions (under the dish, above the puddle and off-center), has no eviction, no failed step, a bounded max speed and a kinetic-energy proxy that does not grow from the first half to the second.
  - Stability smokes at gravity 2 and 80 have no eviction and no failed step. Spikes are not required at gravity 80, because the pressure cap squashes columns (Phase 46 D-07a lesson).
  - The control allowlist rejects bad values.
  - The helper unit tests from D-13 pass.
  Peaks are measured from a binned surface-height profile across the dish, with tolerances, not exact trajectories.
- **D-24:** Before the phase closes, `just scene-fingerprint-gate` must report all 27 protected ids unchanged with `fake-ferrofluid` under "added (accepted)". Record `start_particles`, `end_particles` and native release `median_ms_per_step` from `cargo xtask playground scene-spot --scene fake-ferrofluid` in the phase summary.
- **D-25:** After the scene ships, grow the protected baseline in its own final commit that changes nothing else (Phase 45 D-05). Then rerun the gate so it reports 28 protected ids unchanged.
- **D-26:** Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, the web typecheck and unit tests, the e2e specs the change touches (player and hud-safe-frame), `bun scripts/bright-builds-check.ts all` (628-line limit) and `just markdown-check` for touched Markdown.

### Claude's Discretion
- Exact field formula, falloff radii, modulation wavelength, drag, acceleration cap and default strength, within D-01 to D-04 and D-09. The Plan 1 spike evidence picks the spike candidate (D-05).
- Dish shape and size, particle radius and count, and puddle depth, within D-15.
- Exact color hex values, within D-16.
- Marker drawing style, within D-17.
- Helper module name and API shape, within D-12 to D-14.
- Plan breakdown and wave ordering, provided the spike comes first and the baseline growth commit (D-25) is last and separate.
- Whether to split `fake_ferrofluid.rs` into `fake_ferrofluid/field.rs` and `fake_ferrofluid/tests.rs` (the `foo.rs` + `foo/` layout) to stay under the file-length limit.

### Deferred Ideas (OUT OF SCOPE)
- Hysteresis: spikes that persist slightly below onset as strength ramps down (FERRO-04, future requirement).
- A magnet position preset (below the dish versus follow the pointer) and a GPU Water tint tuned for ferrofluid gloss (research differentiators).
- Held-pointer README cue kinds (not allowed by the shared rules this milestone).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FERRO-01 | Player sees a dark liquid puddle (visible on the dark canvas) rise into several separate spikes toward a magnet. The magnet rests under the dish by default. | § Field Design (measured: 7 peaks at default strength within 0.2–0.3 s), § Color (contrast table), § Marker drawing, § Surface-Peak Metric |
| FERRO-02 | Player can press and drag to move the magnet. The spikes follow, and they collapse when the magnet is released or switched off. | § Pointer and Action Seam, § Probe Results (moving the magnet 1.5 m moved the peak centroid about 0.9 m within 0.5 s; collapse to flat within 0.3–0.8 s) |
| FERRO-03 | Player can adjust magnet strength within a range that never flings the liquid apart, and the scene stays stable while the pointer is held. | § Engine Facts (calibration measured exact), § Field Design (conservative potential + drag), § Probe Results (60 s holds at max strength in 5 magnet positions, g 2/80: 0 escapes, bounded speed, KE falling), Pitfall 2 (wall height) |
| SCENE-01..03 (shared) | Catalog record, preview, portrait frame, README preview, legend, gravity slider, HUD safe frame, ≤ ~3,000 particles, honest caption | § Site Map (every R/W/S/T/E row), Pitfalls 6–9 (copy rules, hint reminder, honesty guard), § Geometry (990 particles, ~0.4 ms/step) |
| PREP-01 (shared gate) | Every protected fingerprint unchanged, new id added | § Fingerprint Gate; Pitfall 4 (deterministic field math) |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Follow `docs/adding-a-scene.md` for every Rust and web site. The fingerprint baseline grows only in its own commit. [VERIFIED: AGENTS.md, docs/adding-a-scene.md]
- No change to `crates/liquidfun` (engine). Scene mechanisms are bridge code in `crates/liquidfun-wasm` on existing public `World` APIs. No new dependencies, cue kinds, bridge APIs or worker messages. [VERIFIED: ROADMAP shared rules]
- Continuous forces are applied once per fixed step in `on_advance`, follow `world.gravity()`, and treat pointer Up and Cancel alike. [VERIFIED: ROADMAP shared rules]
- Rust: no `unwrap()` in production code; propagate with `?`/`map_err`; `expect()` only in tests; `maybe_` prefix for `Option`s; `foo.rs` + `foo/` module layout; tau, not pi, for angles; `let...else` for early returns. [VERIFIED: global CLAUDE.md]
- Tests: one concern per test, with Arrange/Act/Assert comments. [VERIFIED: global CLAUDE.md]
- Files ≤ 628 physical lines (`bun scripts/bright-builds-check.ts all`). [VERIFIED: CONTEXT D-26]
- TypeScript: `camelCase`, `maybe` prefix for nullable values; bun runs the web tooling. [VERIFIED]
- Pre-commit Rust: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. [VERIFIED: global CLAUDE.md]
- `.planning/**` is never mdformatted. Run `just markdown-check` after changing non-GSD Markdown (README gallery update and any `docs/adding-a-scene.md` edit count). [VERIFIED: CLAUDE.md Repo-Local Guidance]
- A new scene needs a `README_SVG_PLANS` entry, a `PORTRAIT_VIEW_BOUNDS` entry and the shared scale legend (automatic). [VERIFIED: AGENTS.md]
- Captions must be honest (Transparency, SCENE-03). [VERIFIED]
- New tests go into the existing `liquidfun-wasm` lib test binary; a new test binary costs about 2 minutes of macOS scanning. [VERIFIED: CONTEXT, user memory "macOS launch stalls"]
- Run plan executors one at a time on the main tree, without worktrees. [VERIFIED: user memory]
- Executor SUMMARY files need lifecycle frontmatter (`generated_by`, `lifecycle_mode`, `phase_lifecycle_id`, `generated_at`). [VERIFIED: user memory]
- `just readme-svg` needs `LIQUIDFUN_README_FONT_FILE`; commit only this scene's new SVG and WebP; put `~/.local/bin` (pipx mdformat) first on PATH. [VERIFIED: user memory, 47-04-SUMMARY]

## Summary

**The force calibration is now measured, not just derived: `F = m·a·particle_iterations` with `m = density·(PARTICLE_STRIDE·2r)²` produces exactly `Δv = a·dt` per fixed step (measured ratio 1.0000 at 1, 2, 3 and 4 particle iterations).** The engine accumulates applied forces on a per-particle force lane and consumes them once, in the first particle substep, with the substep `dt = dt_step / iterations` (`particle/solver/preparation.rs:53-86`, dispatched with the substep in `world/particle_coupling/executor.rs:58-71, 276-280`). Upstream LiquidFun does the same (`b2ParticleSystem.cpp:3019-3022, 3762-3770`). `apply_particle_force_range` divides the total force by the member count (`particle/force.rs:72-91`), so a group-wide acceleration needs `a·m·iterations·count` (the `impulse.rs:186-192` pattern). Forces are rejected only for non-finite vectors, stale handles, empty or non-contiguous ranges and `WALL` particles.

**A throwaway native release probe (scratch crate on `crates/liquidfun` by path, 2026-10-14) tested both D-05 candidates.** Candidate (a), written as a **conservative potential** `Ψ(s,h) = G·S·L·E(h)·R(s)·M(s)` whose gradient gives the lift along `up` and the lateral pull along `across`, formed 7 separate peaks at default strength within 0.2–0.3 s at g 10. It held 60 s at maximum strength with the magnet under the dish, above the puddle, off-center, inside the puddle and in the puddle at a wall, with zero particles leaving the dish, no step failures and falling kinetic energy. It collapsed to a flat surface 0.3–0.8 s after switch-off. The envelope gradient automatically supplies D-01(c)'s lateral pull toward the magnet: moving the magnet from x = 0 to x = 1.5 shifted the crown about 0.9 m within 0.5 s. Candidate (b), N fixed spike-tip targets, produced 0–3 round blobs (point attractors), which is the anti-feature. **Ship candidate (a).** Implement it with polynomial bumps only (no `cos`, `sin`, `exp` or `powi` per step), so native, wasm and the fingerprint capture stay bit-reproducible (v1.5 D-12 lesson). The probe's polynomial variant matched the cosine and exponential variant in every metric.

One measured hazard is new. When the magnet sits **inside or just under the liquid surface** (the most likely tap), the lift near it exceeds gravity and raises a column 1.0 m (cap 1.5 g) to 1.5 m (cap 2.5 g) above the floor. In the probe, 1.2 m dish walls let one particle over; 1.6 m walls held for 60 s. Size the walls from the measured worst-case top plus margin, and add an "inside the puddle" hold position to the D-23 hold test (Pitfall 2).

**Primary recommendation:** Plan 1 builds the shared `particle_force.rs` helper with its calibration and tilt tests, then the scene core with the polynomial candidate (a) and the starting numbers below. Its release spike tests check the D-05 rules in the real scene, including the in-puddle hold. Plan 2 adds the pointer and action seam, the strength control, the marker and the D-23 release tests. Plan 3 is catalog and web wiring. Plan 4 covers README assets, the gates and the scene-spot numbers. Plan 5 is the separate baseline-growth commit.

## Standard Stack

No new dependencies (locked). Everything is in the repository.

| Component | Version | Purpose | Source |
|-----------|---------|---------|--------|
| `liquidfun` (in-repo) | workspace | `World::apply_particle_force`, `apply_particle_force_range`, `particle_system_view` (`positions`, `velocities`, `particle_ids`), `particle_system_snapshot(..).definition()` (`radius`, `density`), `World::gravity`, `ParticleGroupRecipe` + `ParticleFlags::TENSILE | VISCOUS`, `math::settings::PARTICLE_STRIDE` (0.75) | [VERIFIED: codebase] |
| `liquidfun-wasm` (private) | workspace | `SceneHooks`, `SessionCore`, `parse_whole_rpm` (shared whole-number token parser, `scene.rs:327-342`), `scene_spot`, gravity slider | [VERIFIED: codebase] |
| Rust | 1.97.0 (pinned in `rust-toolchain.toml`) | build/test | [VERIFIED] |
| bun / just / ffmpeg / webpmux / dwebp / mdformat | installed | web, recipes, README assets, clip review, Markdown | [VERIFIED: `command -v`] |

**Installation:** none.

## Engine Facts

| Fact | Evidence | Consequence |
|------|----------|-------------|
| `apply_particle_force(id, F)` resolves the owning system, then calls `apply_particle_force_range(system, &[id], F)`. | `world/particle_object/system.rs:359-366` | Per-particle forces are allowed; each call is O(1) plus two tiny `Vec` allocations [VERIFIED] |
| The range call adds `F / count` to each member's force lane. It rejects non-finite x/y, empty ranges, non-contiguous ranges, stale handles and any `WALL` particle, with no mutation. | `particle/force.rs:72-91, 136-150`; `storage/runtime.rs:478-500` | TENSILE/VISCOUS liquid is accepted. Group totals must be multiplied by member count [VERIFIED] |
| A nonzero force marks the system "pending force". The `Force` pass consumes it once per world step, in the first substep: `v += (dt/iterations)·F/m`, then the lane is zeroed and the marker cleared. | `storage/runtime.rs:510-518`; `particle/solver/preparation.rs:53-86`; `world/particle_coupling/executor.rs:58-71, 276-280` | `F = m·a·iterations` gives `Δv = a·dt` per step [VERIFIED: code + probe] |
| `m = density·(PARTICLE_STRIDE·2r)²` (the inverse used by the Force pass). | `particle/solver/preparation.rs:121-125`; `PARTICLE_STRIDE = 0.75` at `math/settings.rs:84` | r 0.04 → m = 0.0036; r 0.05 → m = 0.005625 (density 1) [VERIFIED] |
| Upstream parity: `SolveForce(subStep)` inside the iteration loop, then `m_hasForce = false`. | `third_party/liquidfun/.../b2ParticleSystem.cpp:3019-3022, 3762-3770` | Not a port quirk [VERIFIED] |
| **Measured calibration:** 125 particles in zero gravity, `a = (3, −2)`, `F = m·a·iterations`; mean Δv minus a no-force twin equals `a·dt` with ratio **1.0000** for iterations 1, 2, 3 and 4. | Probe run 2026-10-14 | D-13 unit test can assert within 1e-3 relative [VERIFIED: probe] |
| `TENSILE` (and `POWDER`) particles get **zero** normal particle pressure; their pressure comes from the tensile pass (`surface_tension_pressure_strength`, default 0.2, × critical velocity). | `particle/solver/pressure.rs:89-100`; `material.rs:122-176`; upstream `b2ParticleSystem.cpp:3208-3215` | `with_pressure_strength` does nothing for this liquid. Tune `with_surface_tension_pressure_strength` and `..._normal_strength` if needed [VERIFIED] |
| Viscous pass: `impulse = viscous_strength·weight·Δv` between neighbors (default 0.25). | `material.rs:28-68`; default at `system_definition.rs:567` | D-03 ≤ 1.5. The probe used the default 0.25 [VERIFIED] |
| Particle damping (`with_damping`, default 1.0) acts only on particle–body contacts; nothing damps free-surface motion. | pressure.rs damping kernel (S14–S17) | D-02 drag must be explicit [VERIFIED: code read; ASSUMED: no other free-particle damping pass applies] |
| Velocity clamp ≈ critical velocity = `2r × 60 × iterations` (9.6 m/s at r 0.04, 2 iterations). | `critical_velocity` at `pressure.rs:233`, `material.rs:264`; probe g 80 runs saturated at 9.60 m/s | Max-speed rule: assert well below the clamp at g 10 [VERIFIED: probe] |
| `World::gravity()` returns live gravity; tilt changes it through `SessionCore::set_gravity` without a rebuild; the slider rebuilds the world. | `world/config.rs:270-275`; `session.rs:196-216, 318-341` | Read gravity inside `on_advance` every step [VERIFIED] |
| Session order per step: `on_advance` → eviction → `world.step` → `on_after_step`; 1–4 steps per `advance`. | `session.rs:218-268` | Apply the field in `on_advance`. One step = one field application regardless of frame rate [VERIFIED] |
| Pointer coordinates are checked for finiteness at the session boundary; the kind token is parsed first. | `session.rs:349-366` | Hooks receive only finite `world_x`, `world_y` [VERIFIED] |
| The web sends `move` only for the captured pointer between `down` and `up`/`cancel`. | `web/src/input/pointer.ts:101-131` | Still track `pointer_held` in Rust (defensive, and the diver precedent) [VERIFIED] |
| The particle system definition is readable at runtime: `world.particle_system_snapshot(system)?.definition()`, with `.radius()` and `.density()`. | `world/particle_object/system.rs:164-176`; `particle_object.rs:54-60` | The helper computes mass from the live definition, not copied constants [VERIFIED] |

## Field Design (Candidate (a), measured)

### Frame
For gravity `g` with `G = |g| > ε`: `up = −g/G`, `across = (up.y, −up.x)` (for g = (0, −10): up = (0, 1), across = (1, 0)). For a particle at `p` and magnet at `m`: `r = p − m`, `h = r·up` (height above the magnet along the field axis), `s = r·across` (offset across the dish). If `G ≤ ε`, apply nothing. [VERIFIED: probe]

### Potential (conservative, bounded, polynomial)
`Ψ(s, h) = G · S · L · E(h) · Q(s)`, `Q(s) = R(s) · M(s)`, and the acceleration is its gradient:

- `a_up = G·S·L · E′(h) · Q(s)`. This is the lift along the field axis, opposite gravity (D-01a).
- `a_across = G·S·L · E(h) · Q′(s)`. This pulls toward each column (spike) center and, through `R′`, toward the magnet (D-01c).
- Clamp `|a| ≤ CAP·G·S`, so strength 0 gives no pull (D-11) and the bound holds at maximum strength (D-02).
- Drag: `a_drag = −β · R(s) · v` for every particle while the magnet is on (D-02).

Polynomial pieces. Each is C¹ and bounded, has compact support, and calls no transcendental function:

| Piece | Formula | Derivative |
|---|---|---|
| Vertical reach `E′(h)` (softened core: value 1 at the magnet, no singularity) | `x = h/H`; `(1 − x²)²` for `|x| < 1`, else 0 | — |
| Its integral `E(h)` (odd, bounded by `8H/15`) | `H·(x − 2x³/3 + x⁵/5)` for `|x| < 1`, else `sign(h)·8H/15` | `E′` |
| Envelope `R(s)` (smooth falloff to outer radius `W`) | `y = s/W`; `(1 − y²)²` for `|y| < 1`, else 0 | `2(1 − y²)·(−2s/W²)` |
| Column modulation `M(s)` (periodic, wavelength `λ_s`, spikes centered on the magnet) | `u = s/λ_s`; `t = u − u.round()`; `b = 1 − 4t²`; `M = b⁴` (write `b2 = b*b; b2*b2`) | `4b³·(−8t)/λ_s` |

Why a potential: a conservative field plus drag cannot pump energy while the magnet is still. The total mechanical energy can only fall, so the hold settles (FERRO-03, PITFALLS 5). Liquid fills the total potential `G·h − Ψ` from the bottom, so the free surface bulges upward into columns above each `M` peak instead of forming one ball (the point-attractor anti-feature). D-01(b) names a `1 + a·cos` modulation. The periodic polynomial bump `b⁴` plays that role, with peaks at `s − s_mag = k·λ_s`. Its exact form is within "Claude's discretion: exact field formula". [VERIFIED: probe; energy argument is standard mechanics, ASSUMED to carry over exactly to the particle solver]

### Starting parameters (probe-validated, `MODE=2` polynomial variant)

| Parameter | Start value | Note |
|---|---|---|
| Particle radius r | **0.04 m** (d 0.08, stride 0.06) | D-15. r 0.05 gave only 390–520 particles at the same dish size, with stubbier spikes |
| Flags / viscous / surface tension | `TENSILE | VISCOUS`; viscous 0.25 (default); surface tension pressure 0.2 and normal 0.2 (defaults) | D-03 |
| Particle iterations | 2 (shared default; no R6 arm) | |
| Dish inner width | 6.0 m (inner faces x = ±3.0), floor top y = 0 | Walls at least 3 d thick, overlapping the floor slab |
| Puddle fill | 0.6 m box (settles to about 0.5 m) → **990 particles** | Room in the budget to go to about 1,500 if the look needs it |
| Wall inner height | **≥ worst-case measured top + 0.3 m** (about 1.4 m at CAP 1.5, about 1.8 m at CAP 2.5) | Pitfall 2 |
| Magnet rest | (0, −0.35), under the floor center (the field origin is the magnet position) | Floor slab top at 0, e.g. y ∈ [−0.25, 0] |
| `λ_s` (column wavelength) | 0.65 m | 7 peaks in the 6 m dish |
| `W` (envelope half-width) | 2.6 m | |
| `H` (vertical reach) | 1.6 m | |
| `L` (strength scale) | 2.5 | |
| `CAP` | **1.5–2.5** (× G × S) | Trade crown height for containment (Pitfall 2); start at 2.0 |
| Drag β | 3 s⁻¹ × R(s) | Stable at g 2, 10 and 80 without G scaling |
| Strength | S = percent / 100; default **65**, max 100 | Default 65 → 5–7 peaks; S = 0.3 → flat (0 peaks) |

### Probe Results (native release, Apple M4 Max, 990 particles)

Metrics: peaks = local maxima of a 3-bin-median-smoothed max-y profile (bins 1 d wide, 2 d wall margin), prominence ≥ 0.15 m. "First 4" = time after energizing to ≥ 4 peaks (sampled every 0.1 s). "Collapse" = time after off until 0 peaks and profile range ≤ 0.12 m. "Out" = particles outside the dish interior. "Iso" = particles with no neighbor within 3r.

| Run (polynomial field, CAP 2.5 unless noted) | First 4 | Peaks | Top (m) | Collapse | v max hold (m/s) | Out | Iso |
|---|---|---|---|---|---|---|---|
| g 10, S 0.65 (default), magnet at rest | 0.2 s | 7 | 0.79 | 0.7 s | 0.60 | 0 | 0 |
| g 10, S 1.0, rest, 60 s hold | 0.2 s | 7 | 0.98 | 0.8 s | 0.61 → 0.63 | 0 | 0 |
| g 10, S 1.0, magnet above puddle (y 1.2), 60 s | 0.2 s | 6 | 0.66 | 0.8 s | 0.62 → 0.63 | 0 | 0 |
| g 10, S 1.0, off-center (x 2.0), 60 s | 0.2 s | 5 | 1.01 | — (liquid piled to one side) | 0.61 → 0.61 | 0 | 0 |
| g 10, S 1.0, **inside the puddle** (y 0.4), 60 s, 1.6 m walls | 0.3 s | 5 | **1.47** | — | 0.92 → 0.60 | 0 | 0 |
| g 10, S 1.0, in the puddle at the wall (x 2.9, y 0.4), 60 s | — | 2 | 1.47 | — | 1.71 → 1.30 | 0 | 1 |
| g 10, S 1.0, CAP 1.5, magnet at rest / in the puddle (y 0.3) | 0.2 s | 7 / 5 | 0.79 / **1.00** | 0.7 / 0.8 s | ≤ 0.61 | 0 | 0 |
| g 10, S 0.65, CAP 1.5 | 0.2 s | 5 | 0.69 | 0.6 s | 0.60 | 0 | 0 |
| g 80, S 1.0, rest, 60 s | 0.1 s | 7 | 0.97 | 0.4 s | 3.20 → 3.28 | 0 | 1 |
| g 80, S 1.0, in the puddle, 60 s | 0.2 s | 5 | 1.35 | 0.4 s | 3.25 → 5.43 (KE mean falls) | 0 | 0 |
| g 2, S 1.0, rest, 60 s | 0.4 s | 5 | 0.95 | — | 0.17 → 0.03 | 0 | 0 |
| g 2, S 0.65 | none | 1 | 0.70 | — | ≤ 0.17 | 0 | 0 |
| Magnet stepped from x 0 to x 1.5 at S 0.65 | — | 7 → 5 | — | — | — | 0 | 0 |
| **Candidate (b)** 7 tip targets, reach 0.9 m, S 1.0 / 0.65 | 0.4 s / none | 5 / **3** (round blobs at the center) | 1.00 / 0.93 | 0.8 / 0.7 s | 0.60 | 0 | 0 |

In the stepped-magnet run, the peak x positions went from `[−2.00 −1.36 −0.72 −0.08 0.64 1.28 1.92]` to `[−0.48 0.16 0.80 1.44 2.08]` within 0.5 s. The cosine and exponential variant (`λ = 0.7`, `L = 4`, `W` Gaussian 1.8, `M = ((1+cos)/2)⁴`) gave the same picture: 7 peaks at default, 9 at max, collapse ≤ 0.8 s, 0 out at 1.6 m walls, a column top of 1.57 m with the magnet in the liquid, and 0.31–0.33 ms/step. Median ms/step with the field: 0.37–0.40 (polynomial) and 0.31–0.33 (exp/cos) native release at 990 particles. These include the field and the per-particle force calls; the same scene without forces measured 0.26–0.29. [VERIFIED: probe 2026-10-14]

At 45° tilt the 6 m puddle overflows even 2.5 m walls (the probe has no eviction, so it panicked with `PositionOutOfTagRange`); 17° tilt was fine. In the session, eviction removes the spilled particles, like any open basin. Tilt is not in the D-23 no-eviction set. [VERIFIED: probe]

## Shared Force Helper (D-12..D-14; reused by Phase 49)

Location: `crates/liquidfun-wasm/src/scene/particle_force.rs` (sibling of `gravity_slider.rs`; declare `pub(crate) mod particle_force;` in `scene.rs` or `mod` with `pub(super)` items). Keep it scene-agnostic: no magnet or field terms (those live in `fake_ferrofluid/field.rs`).

```rust
//! Converts per-particle accelerations into one fixed step of particle force.
//!
//! The engine consumes accumulated particle force once per world step, in the
//! first particle substep (`dt / iterations`), so `F = m·a·iterations` adds
//! exactly `a·dt` of velocity per step (measured, research 48).

use liquidfun::math::Vec2;
use liquidfun::math::settings::PARTICLE_STRIDE;
use liquidfun::{ParticleGroupId, ParticleId, ParticleSystemId, World};

use crate::session::SessionError;

/// Gravity magnitude and the field axes built from `world.gravity()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GravityFrame {
    pub(crate) magnitude: f32,
    /// Opposite to gravity.
    pub(crate) up: Vec2,
    /// `up` turned a quarter turn clockwise: (up.y, -up.x).
    pub(crate) across: Vec2,
}

impl GravityFrame {
    /// `None` when gravity is too small to define a direction.
    pub(crate) fn from_gravity(gravity: Vec2) -> Option<Self> { /* length > 1e-4, finite */ }
}

/// Force per unit acceleration for one particle: mass × particle iterations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ForceScale(f32);

impl ForceScale {
    /// Reads radius and density from the live system definition.
    pub(crate) fn for_system(world: &World, system: ParticleSystemId, particle_iterations: u32)
        -> Result<Self, SessionError> { /* density * (PARTICLE_STRIDE * 2r)^2 * iterations; finite > 0 */ }
    pub(crate) fn force(self, acceleration: Vec2) -> Vec2 { acceleration * self.0 }
}

/// Reusable copies of the particle lanes, so `on_advance` allocates nothing after the first step.
#[derive(Default)]
pub(crate) struct ParticleScratch { positions: Vec<Vec2>, velocities: Vec<Vec2>, ids: Vec<ParticleId> }

/// Applies `acceleration(position, velocity)` to every particle; zero results are skipped.
pub(crate) fn apply_accelerations(
    world: &mut World, system: ParticleSystemId, scale: ForceScale, scratch: &mut ParticleScratch,
    mut acceleration: impl FnMut(Vec2, Vec2) -> Vec2,
) -> Result<(), SessionError> { /* copy view lanes into scratch, then apply_particle_force per id */ }

/// Adds one acceleration to every member of a group with a single range call (Phase 49).
pub(crate) fn apply_group_acceleration(
    world: &mut World, system: ParticleSystemId, group: ParticleGroupId, scale: ForceScale, acceleration: Vec2,
) -> Result<(), SessionError> { /* members = particle_group_view(group).member_ids(); total = scale.force(a) * count */ }
```
[ASSUMED: exact names and shape; the APIs used are VERIFIED]

- Pass the scene's iteration count from `crate::scene::particle_iterations(SceneId::FakeFerrofluid)` (a `const fn`, `scene.rs:146-155`), so the helper never assumes 2.
- Compute `ForceScale` once at build. The definition is fixed for the session; the gravity slider rebuilds everything.
- Map `ParticleForceError` and `HandleError` to `SessionError::StepFailed`, as `color_mixer.rs` maps view errors.
- Phase 49 usage: `let frame = GravityFrame::from_gravity(world.gravity())`, then `apply_group_acceleration(world, system, heavy_group, scale, frame.up * (-(ratio - 1.0) * frame.magnitude))`. That is one range call per step.
- D-13 tests (debug-safe, in `particle_force` tests): (1) build a small zero-gravity box, settle it, apply `a = (3, −2)` through the helper, step once, and assert the mean Δv minus a no-force twin is within 1e-3 relative of `a/60` (the probe shape gives 1.0000). (2) `GravityFrame::from_gravity((7.07, −7.07))` gives `up ≈ (−0.707, 0.707)`, and the ferro lift computed with it points along that `up`. Add (3): `from_gravity(ZERO)` is `None`.

## Architecture Patterns

### Recommended module layout
```
crates/liquidfun-wasm/src/scene/
├── particle_force.rs            # shared helper (D-12), with #[cfg(test)] mod tests
├── particle_force/tests.rs      # D-13 calibration + tilt tests (if the file grows)
├── fake_ferrofluid.rs           # build(), build_parts(), hooks, controls, action, pointer, collect_* (credit path)
└── fake_ferrofluid/
    ├── geometry.rs              # dish quads, wall segments, spawn box, MAGNET_REST, FRAME_* bounds, marker outline
    ├── field.rs                 # FieldParams, polynomial pieces, magnet_acceleration (pure, unit-tested)
    ├── test_support.rs          # surface profile, peaks, centroid, KE, out-of-dish, run helpers
    ├── tests.rs                 # debug-safe: field math, allowlist, pointer/action intent, geometry, TS sync
    ├── spike_tests.rs           # release-only D-05 rules (Plan 1)
    └── flow_tests.rs            # release-only D-23 outcomes (Plan 2)
```

### Pattern 1: Intent in hooks, field in `on_advance` (D-06..D-08, D-11)
- Hooks: `magnet_on: bool`, `magnet_position: Vec2`, `pointer_held: bool`, `strength_percent: u8` (default 65), `scale: ForceScale`, `scratch: ParticleScratch`, `wall_segments: Vec<RigidSegment>`.
- `on_advance`: return early if `!magnet_on`. Otherwise get `frame = GravityFrame::from_gravity(world.gravity())` (no frame means no field), `S = f32::from(strength_percent) / 100.0`, then `apply_accelerations(.., |p, v| field::magnet_acceleration(&PARAMS, frame, S, magnet_position, p) - v * (DRAG * field::envelope(frame, magnet_position, p)))`. With S = 0 the field is zero, but drag still applies, so the puddle stays flat (D-11).
- No force while off, so the collapse is plain gravity plus TENSILE/VISCOUS (probe: 0.3–0.8 s).

### Pattern 2: Pointer and action seam (D-07, D-10)
```rust
fn apply_pointer(&mut self, _world: &mut World, _system: ParticleSystemId, kind: PointerKind, x: f32, y: f32)
    -> Result<(), SessionError> {
    match kind {
        PointerKind::Down => {
            self.pointer_held = true;
            self.magnet_on = true;
            self.magnet_position = geometry::clamp_to_frame(Vec2::new(x, y));
        }
        PointerKind::Move if self.pointer_held => {
            self.magnet_position = geometry::clamp_to_frame(Vec2::new(x, y));
        }
        PointerKind::Up | PointerKind::Cancel => {
            self.pointer_held = false;
            self.magnet_on = false;
            self.magnet_position = geometry::MAGNET_REST;
        }
        // An ignored pointer must stay Ok: an error would fail the scene in the browser.
        PointerKind::Move => {}
    }
    Ok(())
}
```
- `apply_action("magnet")` flips `magnet_on` at the current position. Anything else returns `Err(UnknownControl)`. After a release the position is already `MAGNET_REST`, so "on" energizes at rest (D-10).
- Clamp to the landscape `FRAME_*` rectangle that the web `FAKE_FERROFLUID_VIEW_BOUNDS` mirrors. Add an R14 `include_str!` test comparing the f32 bits (`archimedes_screw/control_tests.rs:375-398` pattern). Finiteness is already guaranteed by `session.rs:356-358`; add a hooks-level test that the session rejects `NaN` with `InvalidPointer` and leaves the magnet unchanged.
- Decide Up/Cancel whether or not `pointer_held` is set (D-07 says they always switch off). Up without a held press is harmless.

### Pattern 3: Strength control (D-09)
- `apply_control("magnet-strength", value)`: `parse_whole_rpm(value, 100)` (`scene.rs:327-342`: digits only, no sign, decimal point or leading zero, ≤ max). Then store the value and return `ControlEffect::Live`. Anything else returns `Err(UnknownControl)`. The parser is named for rpm but is a generic whole-number token parser; reuse it rather than duplicating it.
- `build(presets)` returns `Err(UnknownControl)` for any non-empty presets (the screw precedent, `archimedes_screw.rs:118-121`). The gravity preset is stripped by `build_scene` (`scene.rs:258-263`).
- The web replays live values after a gravity rebuild (`web/src/player/worker-controls.ts:44-53`, verified in Phase 47 research). No Rust persistence is needed.
- Range readout: `formatRangeReadout` prints `"65 %"` for unit `"%"`. `formatRangeValueText` has no `%` branch, so screen readers hear `"65 %"` (`web/src/components/range-control.ts:71-108`). Optionally add `if (unit === "%") return \`${value} percent\`` with a unit test (UI-SPEC decides). [VERIFIED: code]

### Pattern 4: Drawing (D-17)
- Static dish outline: inner faces as `RigidSegment`s (about 3–6 segments).
- Magnet marker: a bar about 0.6 × 0.2 m centered on `magnet_position`, oriented along `across` (4 segments) plus a pole divider (1 segment). When on, add 3–5 short "field" ticks above the top pole along `up` (≈ 0.15 m each). When off, draw only the outline. Total ≤ 20 segments, far under 512. Optional: `collect_circles` for two pole caps (≤ 48). [ASSUMED: style]
- Colors are fixed by the renderer: segments use `BASIN_STROKE #94A3B8`, circles `RIGID_STROKE #CBD5E1` (`web/src/export/animated-svg.ts:10-12`). So "on" and "off" must differ by shape (ticks present or absent), not by color.
- Use `world.gravity()` in `collect_segments` for the marker orientation (as the diver's bubble does, `cartesian_diver.rs:255-258`), so the marker tilts with gravity.

### Pattern 5: Release-only tests on a session
- `SessionCore::create(SceneId::FakeFerrofluid)`; `session.apply_pointer("down", x, y)` / `"move"` / `"up"` / `"cancel"`; `session.apply_action("magnet")`; `session.apply_control("magnet-strength", "100")`; `session.apply_control("gravity", "80")` (rebuild, then re-apply strength); `advance(4)` in chunks (`MAX_ADVANCE_STEPS = 4`); read with `session.read_particles(|world, system| …)` (`session.rs:564`). Evictions: `live_particle_count()? == particle_count()` (`session.rs:550-562`). [VERIFIED: `cartesian_diver/motion_tests.rs` pattern]
- Gate each with `#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]`. Run with `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid`.
- Cost: at about 0.4 ms/step, a 60 s hold is about 1.5 s of release time. Four hold positions plus g 2/80 smokes and the collapse tests total about 10–15 s. [VERIFIED: probe timing]

### Anti-Patterns to Avoid
- **Point attraction toward the magnet or tip targets.** The probe's tip candidate made 0–3 round blobs.
- **A lift that is not the gradient of the same potential as the lateral term.** A curl-free field plus drag is what guarantees settling.
- **`sin`, `cos`, `exp`, `powf` or `powi` per particle per step.** Native and wasm libm can differ in the last bit (v1.5 D-12 lesson, Phase 46 D-04, Theo Jansen divergence). Use `+ − × ÷`, `round`, `abs`, `sqrt` (via `length`) only. All are IEEE-exact or correctly rounded.
- **Forces in `apply_pointer`, `apply_action` or `apply_control`** (D-08).
- **Hard-coded `−y`** anywhere in the field or marker (D-14).
- **Exact step indices or trajectories in assertions.** Use the binned profile with tolerances (D-23).

## Surface-Peak Metric (for test_support.rs)

Validated in the probe at r 0.04 (gravity straight down in all D-23 tests):

1. Bins of width `d = 2r` across the dish interior from `−3.0 + 2d` to `3.0 − 2d` (≈ 71 bins). For each bin, the top is the max `y` of particles in the bin; an empty bin is 0 (the floor).
2. Smooth with a 3-bin median (robust to one splashing particle).
3. Peak: a smoothed bin `i` with `top[i-1] < top[i] ≥ top[i+1]` (edges allowed) and **topographic prominence ≥ 0.15 m** (about 2 d). Prominence is `top[i] − max(left_min, right_min)`, where each side's min runs until a higher bin or the edge.
4. Flat: zero peaks at prominence 0.15 **and** the smoothed-profile range (max − min) ≤ 0.12 m (1.5 d). The settled puddle reads 0.01–0.03 m.
5. Peak centroid: the mean x of the peak bins (or prominence-weighted). The probe magnet step 0 → 1.5 moved it from about −0.05 to about +0.8.
6. KE proxy: `Σ ½|v|²` per sample. Compare **means** over the two halves of the hold, excluding the first 2 s transient: `mean(h2) ≤ 1.25·mean(h1) + 0.5`. Do not compare the max of max speed per half; at g 80 with the magnet in the liquid, max speed fluctuated 3.2 → 5.4 while KE fell.
7. Max speed bound at g 10: ≤ 4.0 m/s throughout the hold (probe ≤ 2.6, clamp 9.6).
8. Out-of-dish count: particles outside the dish interior rectangle (x beyond the inner faces + r, or y < −r). Must be 0 at g 10. Eviction: `live == built`.
9. Optional "no flinging" check: isolated particles (no neighbor within 3r) ≤ 3. This is O(n²), so sample every 30 steps; release only.

Starting thresholds for the D-23 tests: ≥ 4 peaks by step 90 (1.5 s) at default; collapse by 90 steps after Up, Cancel or action-off, from the default rest state (the probe measured ≤ 0.8 s); strength 0: flat at 1.5 s and 3 s; drag: centroid moves ≥ 0.4 × the magnet displacement toward it within 1.5 s.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Whole-number percent parsing | A new parser | `parse_whole_rpm(value, 100)` (`scene.rs:327-342`) | The same canonical tokens the web range control emits |
| Force-to-velocity conversion | Ad-hoc constants copied into the scene | `particle_force::ForceScale::for_system` reading the live definition | Radius or density changes stay correct; Phase 49 reuses it |
| Group-wide force | A per-particle loop | `apply_particle_force_range` × member count (`impulse.rs:180-198`) | One validation and one copy |
| Eviction detection | Hooking into `escape.rs` | `live_particle_count()` vs `particle_count()` | Eviction happens only in `SessionCore::advance` |
| Live replay after gravity | Rust-side persistence | Web replay (`worker-controls.ts:44-53`) | Already shipped |
| Web/Rust constant sync | Manual review | An R14 `include_str!` f32-bits test (`archimedes_screw/control_tests.rs:375-398`) | Catches `-2.0` vs `-2.5` drift |
| Magnet orientation under tilt | Manual trig | `GravityFrame` `up` and `across` | No trig, and tilt-correct |

## Common Pitfalls

### Pitfall 1: Candidate (b) or a radial pull makes blobs
**What goes wrong:** Liquid gathers into round balls at the attractor points; with 7 targets the probe got 3 blobs at default strength. **Avoid:** Use candidate (a) as a potential with column modulation. Only build (b) in Plan 1 if (a) fails the in-scene rules (D-05 "keep the first that passes"; the Phase 47 precedent never built candidate B). [VERIFIED: probe]

### Pitfall 2: The magnet inside the liquid lifts a tall column over the walls
**What goes wrong:** Near the magnet `a_up` reaches `CAP·G·S`, above `G`, so liquid levitates into a column. Tops measured 1.0 m (CAP 1.5) to 1.47–1.57 m (CAP 2.5) with the magnet at y 0.1–0.55. A 1.2 m wall let a particle out. The bar magnet "under the dish" never hits this, but a tap on the puddle does. **Avoid:** (1) add "inside the puddle" (for example (0, 0.3) and near a wall) to the D-23 hold positions; (2) size the wall height as the measured worst-case top + 0.3 m; (3) if the walls would look too tall for a "shallow dish", lower CAP toward 1.5 (default still 5 peaks, max 7). **Warning signs:** a nonzero out-of-dish count in the hold tests. [VERIFIED: probe]

### Pitfall 3: High and low gravity
At g 80 the spikes still form (7 peaks) and stay contained with the magnet under the dish. With the magnet in the liquid, particles hit the 9.6 m/s clamp, so assert only "no eviction, no failed step" (D-23). At g 2, surface tension does not scale with G while the field does, so spikes are weak at default strength (1 peak) and appear only near maximum strength. Do not assert spikes at g 2. Height targets must come from the settled surface (user memory "High-gravity water collapse"). [VERIFIED: probe]

### Pitfall 4: Non-deterministic field math breaks the fingerprint and native/wasm parity
Per-step `cos`, `sin` and `exp` can differ in the last bit between Apple libm and Rust's wasm libm (the v1.5 D-12 lesson recorded in 46-CONTEXT D-04 and 47-CONTEXT). Native runs remain repeatable, so the gate itself would pass. The browser and README clip would drift from the native tests, though, and the field is evaluated about 1,000 times per step. Use the polynomial pieces. Write `b⁴` as `(b*b)*(b*b)`, not `powi` (its lowering is not guaranteed identical). Rust never contracts `a*b + c` into FMA implicitly. [VERIFIED: project lesson; ASSUMED: libm last-bit differences specifically for `cosf`/`expf`]

### Pitfall 5: Off-center collapse is slower
With the magnet far off-center, the field drags the liquid to one side. After Up, the puddle needs more than 1 s to level, because liquid must flow back across 3 m. The D-23 collapse tests start from the default rest state, where collapse takes ≤ 0.8 s. Do not chain "drag off-center, then Up, then flat within 1.5 s" without measuring it. [VERIFIED: probe "collapse —" rows]

### Pitfall 6: The caption honesty guard trips on D-20's own wording
The example caption says "not a simulated magnetic field". A forbidden-claims list containing "magnetic field" or "simulated magnetic field" would fail on that negation. **Avoid:** Require `toContain("scripted magnetic pull")` and forbid claims that never appear negated, for example `"magnetostatic"`, `"field solver"`, `"real magnet"`, `"maxwell"`, `"rosensweig"`, `"measured"`, `"simulates magnetism"`. [VERIFIED: `web/tests/scenes.test.ts:147-171, 341-372` pattern; ASSUMED: word list]

### Pitfall 7: The hint must carry the keyboard reminder
`fake-ferrofluid` is interactive (not in `WATCH_FIRST_SCENE_IDS`, `web/tests/scenes.test.ts:120-131`). Every interactive hint must contain "Labeled controls also work from the keyboard." (`scenes.test.ts:303-322`), and D-20's example does not. Append it. Descriptions are locked UI-SPEC strings and every original scene's description ends with "This is an original experimental scene." [VERIFIED]

### Pitfall 8: File-length pressure on shared test files
`web/tests/controls.test.ts` is 559 lines; a HUD spinner test adds about 30. `web/tests/scenes.test.ts` is 550 lines; description, hint, two list entries and a guard add about 25. `web/e2e/player-helpers.ts` is 582 lines and `crates/liquidfun-wasm/src/session/tests.rs` 582; each gains 1 line. All stay under 628, but check `bun scripts/bright-builds-check.ts all` after editing. Split `controls.test.ts` if it crosses about 600. [VERIFIED: `wc -l`]

### Pitfall 9: Color disappears in some render mode
In the Water mode shade pass, body color = particle color × the density-shade curve, mixed with refracted backdrop by `exp(−0.9·sum)`, plus specular. Thin regions trend toward the canvas color, and foam whitens above 3.5 m/s (`water-shade.frag.glsl`; `water-uniforms.ts:19-24`). Contrast vs `#071018`, computed with WCAG relative luminance (unshaded / at the 0.73 floor):

| Candidate | Unshaded | At 0.73 floor |
|---|---|---|
| `#5B5F7A` (91, 95, 122) | 3.07 | 2.03 |
| `#7C7799` (124, 119, 153) | 4.52 | 2.73 |
| `#80809F` (128, 128, 159) | 5.02 | 2.99 |
| **`#8A84AD` (138, 132, 173)** graphite violet | **5.46** | **3.21** |
| `#4DA3FF` (shared water, reference) | 7.30 | 4.08 |

Start at `#8A84AD` (`ParticleColor::new(138, 132, 173, 255)`), which keeps ≥ 3:1 at the floor and reads clearly darker and grayer than the shared water. Add a `FERRO` constant to `preview-kit.tsx` or `previews-curiosity.tsx` for the static preview. Check all four modes by eye (D-16). [VERIFIED: computed; ASSUMED: 3:1 as the readability bar]

### Pitfall 10: README clip timing
Samples are 20 Hz × 3 steps (`web/src/export/duration.ts:4-7`); cues fire in `beforeSample(k)` (`web/scripts/readme-svg.ts:127-153`). D-21: `{ atSample: 120, kind: "action", name: "magnet" }` (6 s, off) and `{ atSample: 160, kind: "action", name: "magnet" }` (8 s, on). The probe shows collapse ≤ 0.8 s (done by about 6.8 s) and re-rise ≤ 0.3 s (spikes by about 8.3 s), both inside the clip. Verify on the generated WebP with `webpmux -get frame N` + `dwebp`. [VERIFIED: code; timing from probe]

### Pitfall 11: Tooling on this Mac
`just readme-svg` needs `LIQUIDFUN_README_FONT_FILE=/usr/local/texlive/2025/texmf-dist/fonts/truetype/public/dejavu/DejaVuSans.ttf`. Restore other scenes' re-encoded WebPs with `git checkout --` (Phase 47 restored 25). Put `~/.local/bin` first on PATH for mdformat. [VERIFIED: 47-04-SUMMARY, user memory]

## Code Examples

### Field (pure, `fake_ferrofluid/field.rs`)
```rust
/// Bounded, C¹ bump: (1 - x²)² inside |x| < 1, else 0.
fn bump(x: f32) -> f32 {
    if x.abs() >= 1.0 { return 0.0; }
    let q = 1.0 - x * x;
    q * q
}

/// Integral of `bump(h / reach)` over h: odd and bounded by 8·reach/15.
fn bump_integral(h: f32, reach: f32) -> f32 {
    let x = h / reach;
    if x.abs() >= 1.0 { return x.signum() * reach * 8.0 / 15.0; }
    let x3 = x * x * x;
    reach * (x - 2.0 * x3 / 3.0 + x3 * x * x / 5.0)
}

/// Column modulation b⁴ with b = 1 - 4t², t = s/λ - round(s/λ); returns (M, dM/ds).
fn columns(s: f32, wavelength: f32) -> (f32, f32) {
    let u = s / wavelength;
    let t = u - u.round();
    let b = 1.0 - 4.0 * t * t;
    let b2 = b * b;
    (b2 * b2, 4.0 * b2 * b * (-8.0 * t) / wavelength)
}

/// Scripted acceleration (no drag) at `position` for a magnet at `magnet`.
pub(super) fn magnet_acceleration(p: &FieldParams, frame: GravityFrame, strength: f32, magnet: Vec2, position: Vec2) -> Vec2 {
    let offset = position - magnet;
    let h = offset.x * frame.up.x + offset.y * frame.up.y;
    let s = offset.x * frame.across.x + offset.y * frame.across.y;
    let y = s / p.envelope_half_width;
    let (envelope, envelope_slope) = if y.abs() < 1.0 {
        let q = 1.0 - y * y;
        (q * q, 2.0 * q * (-2.0 * s / (p.envelope_half_width * p.envelope_half_width)))
    } else {
        (0.0, 0.0)
    };
    let (m, m_slope) = columns(s, p.wavelength);
    let q = envelope * m;
    let q_slope = envelope_slope * m + envelope * m_slope;
    let scale = frame.magnitude * strength * p.strength_scale;
    let lift = scale * bump(h / p.reach) * q;
    let lateral = scale * bump_integral(h, p.reach) * q_slope;
    let acceleration = frame.up * lift + frame.across * lateral;
    let cap = p.cap * frame.magnitude * strength;
    let length = acceleration.length();
    if length <= cap { return acceleration; }
    acceleration * (cap / length)
}
```
Unit tests (debug-safe): zero at strength 0; `|a| ≤ cap·G·S` on a grid; lift positive directly above the magnet; symmetric in ±s; rotating gravity by 90° rotates the output by 90° (D-14); the field is zero outside the envelope; a finite-difference curl check `∂a_up/∂s ≈ ∂a_across/∂h` away from the cap (documents the conservative design). [ASSUMED: names; math VERIFIED by probe]

### Hooks `on_advance`
```rust
fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> Result<(), SessionError> {
    if !self.magnet_on {
        return Ok(());
    }
    let Some(frame) = GravityFrame::from_gravity(world.gravity()) else {
        return Ok(());
    };
    let strength = f32::from(self.strength_percent) / 100.0;
    let magnet = self.magnet_position;
    particle_force::apply_accelerations(world, system, self.scale, &mut self.scratch, |position, velocity| {
        field::magnet_acceleration(&FIELD, frame, strength, magnet, position)
            - velocity * (DRAG_PER_SECOND * field::envelope(&FIELD, frame, magnet, position))
    })
}
```

### Calibration test (D-13)
```rust
#[test]
fn requested_acceleration_changes_velocity_by_a_dt_per_step() {
    // Arrange: zero-gravity box of particles, settled 30 steps; a twin without force.
    let acceleration = Vec2::new(3.0, -2.0);
    // Act: apply_accelerations(.., |_, _| acceleration) then one step; subtract the twin's mean Δv.
    // Assert
    assert!((mean_delta - acceleration * (1.0 / 60.0)).length() <= 1e-3 * (acceleration * (1.0 / 60.0)).length());
}
```

### Web control file
```ts
// web/src/catalog/fake-ferrofluid-strength.ts (copy of archimedes-screw-speed.ts)
export const MAGNET_STRENGTH_MIN = 0;
export const MAGNET_STRENGTH_MAX = 100;
export const MAGNET_STRENGTH_STEP = 1;
export const MAGNET_STRENGTH_DEFAULT = 65;
export const MAGNET_STRENGTH_TICKS = [MAGNET_STRENGTH_MIN, MAGNET_STRENGTH_DEFAULT, MAGNET_STRENGTH_MAX] as const;
export const MAGNET_STRENGTH_CONTROL: SceneControl = {
  id: "magnet-strength", label: "Magnet strength", kind: "range", recreates: false,
  surface: "hud", widget: "spinner", min: MAGNET_STRENGTH_MIN, max: MAGNET_STRENGTH_MAX,
  step: MAGNET_STRENGTH_STEP, defaultValue: MAGNET_STRENGTH_DEFAULT, unit: "%",
  scale: "linear", ticks: MAGNET_STRENGTH_TICKS,
};
// record: controls: withGravitySlider([MAGNET_STRENGTH_CONTROL, action("magnet", "Magnet")])
```
Add an R14 Rust test reading this file to compare `DEFAULT` and `MAX` with the Rust constants (the screw `web_speed_constants_match_rust` precedent).

## Site Map (exact sites)

### Rust (`crates/liquidfun-wasm/src`)
| # | Site | Edit |
|---|------|------|
| R1 | `scene/fake_ferrofluid.rs` + `fake_ferrofluid/{geometry,field,test_support,tests,spike_tests,flow_tests}.rs` | New scene |
| new | `scene/particle_force.rs` (+ tests) | Shared helper (D-12) |
| R2 | `scene.rs:9-37` | `mod fake_ferrofluid;` and `mod particle_force;` (alphabetical) |
| R3/R5 | `scene.rs:46-74` `SceneId`, `scene.rs:254-290` `build_scene` | `FakeFerrofluid` + arm |
| R4 | `scene.rs:108-139` `parse_scene_id` | `"fake-ferrofluid" => Ok(SceneId::FakeFerrofluid)` |
| R6 | `scene.rs:146-155` | None (2 iterations) |
| R7 | `scene.rs:78-109` `ALL_SCENE_IDS` (append last) together with web `SCENE_IDS`; `gravity_slider.rs:161-190` `variant_index` → `SceneId::FakeFerrofluid => 27` | Order test `scene.rs:365-382` |
| R8 | `session/tests.rs:14-44` | Append `("fake-ferrofluid", SceneId::FakeFerrofluid)` |
| R9 | `scene_spot.rs:46-114` `SURVEY_SCENES` | `("fake-ferrofluid", SceneId::FakeFerrofluid, SurveyCue::Default)` |
| R10 | `scene_spot/tests.rs:55-80` | No change (Default cue) |
| R12 | `session/escape.rs` | Not needed (geometry within ±4 m) |
| R13 | `spike_tests.rs`, `flow_tests.rs` | Release-only |
| R14 | `fake_ferrofluid/tests.rs` | `include_str!` sync of strength constants and view bounds |

### Web
| # | Site | Edit |
|---|------|------|
| W1 | `web/src/catalog/scenes.ts:38-41` `SCENE_IDS` | Append `"fake-ferrofluid"` |
| W2 | `scene-records-curiosity.ts` (54 lines) | Record after the screw: D-20 caption + "This is an original experimental scene.", hint + keyboard reminder, `withGravitySlider([MAGNET_STRENGTH_CONTROL, action("magnet", "Magnet")])`, `viewBounds: FAKE_FERROFLUID_VIEW_BOUNDS`, `sceneSource("fake_ferrofluid.rs")`, `[SHOWCASE]` |
| W3 | `web/src/catalog/fake-ferrofluid-strength.ts` (new) | Above |
| W4 | `DEFAULT_PRESET_VALUES` | Not needed (no preset) |
| W5 | `previews-curiosity.tsx` `FakeFerrofluidPreview()`; `previews.tsx` `case "fake-ferrofluid"` (`never` guard) | Dish, dark puddle with a 5–7 spike crown, bar magnet below |
| W6 | `scene-view-bounds.ts` `FAKE_FERROFLUID_VIEW_BOUNDS` (re-export via `scene-records.ts` → `scenes.ts`); `portrait-bounds.ts:97-101` | Landscape: dish walls ±3.3 plus margin, from y ≈ −0.9 (rest magnet) to about wall top + 0.6. Portrait: the same or tighter, walls included |
| S1 | `web/scripts/readme-svg/plans.ts:84-92` | `{ id: "fake-ferrofluid", cues: [{ atSample: 120, kind: "action", name: "magnet" }, { atSample: 160, kind: "action", name: "magnet" }] }` |
| S2 | `docs/assets/readme/fake-ferrofluid-10s.{svg,webp}`, README gallery | `just readme-svg`, review, `just markdown-check` |
| S3 | `web/scripts/demo-media/model.ts:296-301` | `{ id: "fake-ferrofluid", title: "Fake Ferrofluid", route: "/liquidfun-rs/#/scene/fake-ferrofluid", interactionStep: 180, action: { kind: "click", point: { x: 0.5, y: 0.5 } } }` (idle convention; the click toggles the magnet on at the center briefly, which is fine) |
| T1 | `web/tests/scenes.test.ts` | Descriptions, hints, id and title lists, `FERRO_FORBIDDEN_CLAIMS` guard requiring "scripted magnetic pull" (Pitfall 6) |
| T2 | `web/tests/scene-catalog-controls.test.ts` | Append `"gravity"` to `RECREATING_CONTROL_IDS`; a block asserting `[MAGNET_STRENGTH_CONTROL, action magnet, GRAVITY_CONTROL]` |
| T3 | `web/tests/scene-catalog-credits.test.ts:98, 217` | Path and `[SHOWCASE_HREF]` |
| T4 | `web/tests/portrait-bounds.test.ts:45-76, ~406` | `MIN_HEIGHT_FRACTION["fake-ferrofluid"]` (wide frame is width-bound: about (height/width)·390/844 ≈ 0.2–0.3), wall points (dish corners, wall tops, magnet rest) |
| T6 | `web/tests/controls.test.ts:~503` | "keeps Magnet strength on the HUD": HUD `["magnet-strength"]`, panel `["magnet", "gravity"]`, step ±1 |
| E1 | `web/e2e/player-helpers.ts:57` | `"fake-ferrofluid": "/liquidfun-rs/#/scene/fake-ferrofluid"` |
| E2 | `web/e2e/player.spec.ts:40-56` | `"fake-ferrofluid": { gesture: "drag", control: "Magnet strength" }`. Drag exercises Down/Move/Up; `activateLabeledControl` fills the spinbutton max (`player-helpers.ts:477-506`) |

**Already derived (no edit):** routing, sidebar, `ScaleLegend`, gravity slider, `hud-safe-frame.spec.ts`, shell counts, xtask scene list, `PAGE_SUMMARY`. [VERIFIED: docs/adding-a-scene.md]

## Fingerprint Gate

- Baseline: `docs/benchmarks/scene-fingerprints.json` protects 27 ids, `archimedes-screw` last (Phase 47-05 commit `4be8ffdab`). [VERIFIED: STATE]
- Gate: `just scene-fingerprint-gate`, or `--spot target/dam-break-perf/<stamp>/scene-spot.json` to reuse a capture. Expect "protected: 27/27 unchanged" and "added (accepted): fake-ferrofluid <hex>". The new shared `particle_force.rs` must not be wired into any existing scene, or protected fingerprints may move.
- Per-scene numbers: `cargo xtask playground scene-spot --scene fake-ferrofluid` → `start_particles`, `end_particles`, `median_ms_per_step`. Expect about 990 particles (with 0 evicted, start = end) and about 0.4 ms/step; the screw measured 0.69 ms at 1,729.
- D-25: a separate single-file commit appends `{ "id": "fake-ferrofluid", "fingerprint": "<16 hex>" }` from the full capture, updates `provenance`, and reruns the gate, which must report 28/28.

## Recommended Plan Breakdown

| Plan | Content | Risk |
|---|---|---|
| 48-01 | **Helper + scene core + spike (time-boxed).** `particle_force.rs` with D-13 tests (calibration, tilt, zero gravity). `fake_ferrofluid` geometry (dish, walls sized per Pitfall 2), the polynomial `field.rs` with unit tests, hooks with the magnet on at rest and a fixed default strength, `build_parts`, `test_support` (profile, peaks, centroid, KE, out-of-dish). Register only parse/build/`variant_index`/session token (not `ALL_SCENE_IDS` or `SURVEY_SCENES`, which ship with web `SCENE_IDS` in Plan 3, as in 46-01 and 47-01). Release spike tests against the D-05 rules: ≥ 4 peaks ≤ 1.5 s at default, flat ≤ 1.5 s after off, and a 60 s hold at max with 0 evicted, 0 out, bounded max speed and no KE growth, at rest, above, off-center and **in the puddle**. Candidate (b) only if (a) fails. Record the measurements and the decision in the SUMMARY | LOW–MEDIUM (wall height vs CAP) |
| 48-02 | Pointer seam (Down/Move/Up/Cancel, clamp, `pointer_held`), `magnet` action, `magnet-strength` control with allowlist, marker drawing, the D-23 release tests (load peaks, drag centroid, Up/Cancel/action collapse, strength 0 flat, 60 s holds, g 2/80 smokes), README-timing check (collapse before sample 160 and re-rise after it) | LOW |
| 48-03 | Catalog: `ALL_SCENE_IDS`/`SURVEY_SCENES` + web `SCENE_IDS`, record, strength file, preview, view and portrait bounds, README/demo plans, T1–T6, E1/E2, R14 sync tests | LOW |
| 48-04 | `just readme-svg` + clip review (spikes, collapse after 6 s, re-rise after 8 s), color check in four modes, full gates (cargo, web unit, player + hud-safe-frame e2e, bright-builds, markdown), scene-spot numbers, `just scene-fingerprint-gate` (27/27 + ferrofluid added) | LOW |
| 48-05 | Single-file baseline growth commit, gate rerun 28/28 | LOW |

Execute sequentially on the main tree.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Calibration `F = m·a·iterations` "from reading code; measure it" (v1.6 SUMMARY) | Measured exact (ratio 1.0000, iterations 1–4) | This research | D-13 test can use a tight tolerance |
| Lift and lateral terms tuned independently (D-01 a/c) | One potential's gradient supplies both; the lateral pull toward the magnet comes from the envelope slope | This research | Guaranteed settle with drag; fewer knobs |
| `1 + a·cos(k·s)` modulation | Periodic polynomial bump `(1 − 4t²)⁴` | This research | Native/wasm bit-identical field |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The probe's starting numbers (r 0.04, 6 m dish, 0.6 m fill, λ_s 0.65, W 2.6, H 1.6, L 2.5, CAP 1.5–2.5, β 3) reproduce in the session scene with the real dish and walls | Field Design | Plan 1 spike tunes them; the probe already used the same engine, flags and step configuration |
| A2 | Apple libm and Rust wasm libm `cosf`/`expf` can differ in the last bit | Pitfall 4 | If they never differ, polynomials still cost nothing; no downside |
| A3 | No other pass damps free-surface particle motion | Engine Facts | Drag might be redundant; harmless |
| A4 | 3:1 contrast at the 0.73 floor is the right readability bar | Pitfall 9 | Visual check in four modes is the real gate (D-16) |
| A5 | Helper and field function names and shapes | Shared Force Helper, Code Examples | Naming only |
| A6 | The magnet marker style (bar plus field ticks when on) | Pattern 4 | Visual only; UI-SPEC may refine |
| A7 | The honesty-guard word list | Pitfall 6 | Copy review only |
| A8 | The "inside the puddle" hold position is an acceptable addition to D-23's three positions | Pitfall 2 | It adds coverage without removing any locked case; if the user objects, keep it as a spike-only check |

## Open Questions (RESOLVED)

1. **CAP vs wall height (Pitfall 2). (RESOLVED)** Known: CAP 2.5 gives taller crowns at max (top ≈ 1.0 m at rest) but 1.5 m columns with the magnet in the liquid; CAP 1.5 keeps the worst case near 1.0 m with 5–7 peaks. Unclear: which looks better in the real scene. Recommendation: Plan 1 measures both in the scene and picks the lowest CAP that still gives ≥ 5 peaks at default, then sets walls to the measured worst-case top + 0.3 m.
   **RESOLVED:** Plan 01 Task 3 applies the CAP choice rule: measure P1 at cap 1.5, 2.0 and 2.5, keep the lowest cap whose step-180 peak count at default is ≥ 5 (compile-time `CAP <= 3.0`), then set `WALL_TOP_Y` = round_up_to_0.1(worst measured column top over the five P3 holds + 0.3) and `FRAME_MAX_Y >= WALL_TOP_Y + 0.6`; the P4 release test `column_tops_stay_three_tenths_below_the_wall_tops` guards it and the SUMMARY records the measurements and decision.
2. **Particle count. (RESOLVED)** The probe used 990 (r 0.04). There is budget for about 1,500 (wider dish or deeper fill) at well under 1.5 ms/step. Recommendation: keep about 1,000 unless the README clip looks sparse.
   **RESOLVED:** Plan 01 builds one filled puddle box of about 990 particles (radius 0.04, 0.6 m deep) with `MAXIMUM_PARTICLE_COUNT = 3000` as the D-15 budget cap; a build test asserts 800..=3000 particles. Puddle depth 0.5..0.8 m stays a Plan 01 tuning lever; the final count is recorded in the 48-01 SUMMARY.
3. **Percent readout wording. (RESOLVED)** `"65 %"` vs a spoken "percent" branch in `range-control.ts`. Recommendation: leave to the UI-SPEC; adding the branch is a 3-line change plus a unit test.
   **RESOLVED:** UI-SPEC UI-D78: the visible readout stays the shared `formatRangeReadout` (`65 %`, no readout branch), and `formatRangeValueText` gains one `unit === "%"` branch so screen readers hear `65 percent`. Plan 03 Task 2 implements it with the "speaks magnet strength in percent" test in `web/tests/fake-ferrofluid-controls.test.ts`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Tests, scene-spot, gate | ✓ | 1.97.0 via `rust-toolchain.toml` | — |
| bun | Web typecheck/unit, scripts | ✓ | installed | — |
| just | Recipes | ✓ | installed | Run the underlying commands |
| ffmpeg, webpmux, dwebp | `just readme-svg`, clip review | ✓ | installed | — |
| mdformat 1.0.0 | `just markdown-check` | ✓ | `~/.local/bin/mdformat` | — |
| DejaVu Sans font file | README WebP | ✓ | `/usr/local/texlive/2025/.../DejaVuSans.ttf` | — |
| Playwright Chromium | player + hud-safe-frame e2e | ✓ (Phases 46–47 used it) | cached | — |

## Verification Commands

| Req | Behavior | Command |
|-----|----------|---------|
| D-13 | Helper calibration, tilt and zero-gravity tests | `cargo test -p liquidfun-wasm --lib particle_force` |
| FERRO-01..03 (debug) | Field math, allowlist, pointer/action intent, geometry, TS sync | `cargo test -p liquidfun-wasm --lib fake_ferrofluid` |
| FERRO-01..03 (release) | Spike rules, D-23 outcomes, 60 s holds, g 2/80 | `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid` |
| Registration | Order, survey, gravity slider, session tokens | `cargo test -p liquidfun-wasm --lib -- scene::tests scene_spot gravity_slider session::tests` |
| SCENE-01..03 | Catalog, controls, credits, portrait, README plan, demo media, legend, copy | `cd web && bun run typecheck && bun run test:unit` |
| e2e | Player loops, HUD safe frame | `just web-build && cd web && CI=true bunx playwright test e2e/player.spec.ts e2e/hud-safe-frame.spec.ts --project chromium` |
| README | Assets | `LIQUIDFUN_README_FONT_FILE=<ttf> just readme-svg && just markdown-check` |
| PREP-01 | Gate | `just scene-fingerprint-gate` |
| Repo | Format, lint, length | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && bun scripts/bright-builds-check.ts all` |

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Allowlisted control and action names; canonical whole-number percent `0..=100` via `parse_whole_rpm`; pointer kind parsed and coordinates finite-checked at the session boundary; the magnet clamped to the frame; everything else `Err(UnknownControl)` or `Err(InvalidPointer)`, never a panic. `build` rejects unknown presets |
| V2/V3/V4/V6 | no | Static client-side playground |

| Threat | STRIDE | Mitigation |
|--------|--------|-----------|
| Out-of-range or malformed strength (`"101"`, `"065"`, `"-5"`, `"6.5"`) | Tampering / DoS (flinging) | Parser rejects; `CAP·G·S` bounds the acceleration |
| Non-finite or far-away pointer | Tampering / DoS | Session rejects non-finite input; hooks clamp to `FRAME_*` |
| Unknown control, action or preset | Tampering | `UnknownControl` |

## Sources

### Primary (HIGH, verified in this session)
- `crates/liquidfun/src/particle/force.rs`, `particle/solver/preparation.rs`, `particle/solver/pressure.rs`, `particle/solver/material.rs`, `particle/storage/runtime.rs`, `world/particle_coupling/executor.rs`, `world/particle_object/system.rs`, `world/particle_object.rs`, `world/config.rs`, `math/settings.rs`, `particle/definition/system_definition.rs`: the force path, mass, substep, tensile pressure and defaults.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp:3019-3022, 3208-3215, 3762-3770`: upstream parity.
- `crates/liquidfun-wasm/src/{scene.rs, session.rs, session/escape.rs, scene_spot.rs, scene/color_mixer.rs, scene/impulse.rs, scene/liquid_timer.rs, scene/cartesian_diver.rs, scene/archimedes_screw.rs, scene/archimedes_screw/control_tests.rs, scene/gravity_slider.rs}`.
- `web/src/catalog/*`, `web/src/components/range-control.ts`, `web/src/input/pointer.ts`, `web/src/render/{density-shade.ts, water-uniforms.ts, water-shaders/water-shade.frag.glsl}`, `web/src/export/{animated-svg.ts, duration.ts}`, `web/scripts/{readme-svg.ts, readme-svg/plans.ts, demo-media/model.ts}`, `web/tests/*`, `web/e2e/player*.ts`.
- `docs/adding-a-scene.md`, `.planning/phases/47-archimedes-screw/*`, `.planning/research/v1.6-curiosity-scenes/{PITFALLS,SUMMARY}.md`.
- Native release probe (scratch crate depending on `crates/liquidfun` by path, not committed, 2026-10-14): calibration at iterations 1–4; candidates (a) exp/cos and polynomial, and (b) tip targets; 60 s holds in five magnet positions; g 2, 10 and 80; 17° and 45° tilt; magnet step response; ms/step.

### Secondary (MEDIUM)
- `.planning/research/v1.6-curiosity-scenes/FEATURES.md` § 6: real-world reference and anti-features (not re-read in depth this session).

## Metadata

**Confidence breakdown:**
- Engine force facts and calibration: HIGH (code + upstream + measured)
- Field design and stability: HIGH for the mechanism (measured in five positions and three gravities); MEDIUM for the exact numbers in the real scene
- Integration sites: HIGH (grep of every Phase 47 site)
- Pitfalls: HIGH for the wall-height and blob findings (observed); MEDIUM for the libm divergence specifics

**Research date:** 2026-10-14
**Valid until:** 2026-11-13 (stable in-repo code)
