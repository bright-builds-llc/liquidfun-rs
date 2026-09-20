---
phase: 260919-wbn-raise-webapp-particle-frame-cap-20x-to-1
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun-wasm/src/frame.rs
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/dam_break.rs
  - crates/liquidfun-wasm/src/scene/dam_break/tests.rs
  - crates/liquidfun-wasm/src/scene/float_or_sink.rs
  - crates/liquidfun-wasm/src/scene/color_mixer.rs
  - crates/liquidfun-wasm/src/scene/color_mixer/tests.rs
  - crates/liquidfun-wasm/src/scene/jelly_drop.rs
  - crates/liquidfun-wasm/src/scene/fountain.rs
  - crates/liquidfun-wasm/src/scene/water_wheel.rs
  - crates/liquidfun-wasm/src/scene/water_wheel/tests.rs
  - web/src/physics/frame.ts
  - web/tests/frame.test.ts
  - web/e2e/rust-wasm-proof.spec.ts
autonomous: true
requirements:
  - WASM-02
  - DEMO-01
  - DEMO-02
  - DEMO-03
  - DEMO-04
  - DEMO-05
  - DEMO-06
user_setup: []
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 260919-wbn
generated_at: "2026-09-20T04:16:00Z"
must_haves:
  truths:
    - "Copied WASM and JS frame parsers accept up to 10240 particles and reject 10241."
    - "Dam Break Medium is 48×40 = 1920 particles at radius 0.06324555 and spacing 0.101193; Small is 25×26 = 650; Large is 63×44 = 2772; system cap is 10240."
    - "The other five playground scenes run ~10× particle counts with ~1/sqrt(10) radii, unchanged rigid-body sizes, and emitter plateaus at or below 3200."
    - "Occupied water/jelly volumes stay similar; canvas has no extra draw-scale; demo-media is not regenerated unless a test fails without it."
  artifacts:
    - path: crates/liquidfun-wasm/src/frame.rs
      provides: "Rust copied-frame particle cap 10240"
      contains: "MAX_PARTICLE_COUNT: usize = 10240"
    - path: crates/liquidfun-wasm/src/session.rs
      provides: "Capture rejects above 10240 before FrameData::new"
      contains: "MAX_FRAME_PARTICLES: usize = 10240"
    - path: web/src/physics/frame.ts
      provides: "JS parser particle cap 10240"
      contains: "MAX_PARTICLE_COUNT = 10240"
    - path: crates/liquidfun-wasm/src/scene/dam_break.rs
      provides: "Finer 10× Dam Break grids and radius/spacing"
      contains: "PARTICLE_COLUMNS: u8 = 48"
    - path: crates/liquidfun-wasm/src/scene/fountain.rs
      provides: "10× fountain emission and 3200 cap"
      contains: "MAXIMUM_PARTICLE_COUNT: usize = 3200"
  key_links:
    - from: crates/liquidfun-wasm/src/frame.rs
      to: web/src/physics/frame.ts
      via: "identical MAX_PARTICLE_COUNT = 10240"
      pattern: "MAX_PARTICLE_COUNT.*= 10240"
    - from: crates/liquidfun-wasm/src/session.rs
      to: crates/liquidfun-wasm/src/frame.rs
      via: "capture checks MAX_FRAME_PARTICLES then FrameData::new"
      pattern: "MAX_FRAME_PARTICLES"
    - from: crates/liquidfun-wasm/src/scene/dam_break.rs
      to: crates/liquidfun-wasm/src/scene.rs
      via: "documented Medium 48×40 = 1920, radius 0.06324555, spacing 0.101193, cap 10240"
      pattern: "48×40 = 1920"
---

<objective>
Raise the playground copied-frame particle cap from 512 to 10240 and pack all six WASM scenes about 10× denser so world volumes stay similar.

Purpose: Visitors see finer, smaller particles without growing basins or rigid bodies, and without a separate canvas draw-scale.
Output: Matching Rust/JS caps, updated scene recipes, and native plus web frame tests. No engine-wide `liquidfun` default changes. No `.planning` commit from the executor.
</objective>

<execution_context>
@$HOME/.cursor/get-shit-done/workflows/execute-plan.md
@$HOME/.cursor/get-shit-done/templates/summary.md
</execution_context>

<context>
@PROJECT-SCOPE.md
@AGENTS.md
@.planning/STATE.md
@crates/liquidfun-wasm/src/frame.rs
@crates/liquidfun-wasm/src/session.rs
@web/src/physics/frame.ts

Locked packing factor `1/sqrt(10) ≈ 0.316227766`. Use the exact literals below; do not invent a canvas scale and do not edit `crates/liquidfun`.

Do not change rigid geometry: Dam Break obstacle `0.75` at `(2.5, 5.5)`, Float or Sink drop radius `0.5`, Water Wheel hub `0.35` / paddles / `PADDLE_HALF_WIDTH`, basin walls, or FixtureDef friction `0.2`.

`web/tests/canvas.test.ts` fixture radius `0.2` stays (D-14). `frame.rs` dummy lane radii `0.2` in non-scene unit tests may stay.

Do not regenerate demo-media unless a test fails without it. Catalog stills will look stale (particles ~0.32× diameter). Note that in the SUMMARY.

Standing authorization (AGENTS.md, 2026-09-13) covers iteration and one atomic code commit. Do not commit `.planning/**` or `.vscode/`.

<interfaces>
Rust/JS frame caps must remain identical:

```rust
const MAX_PARTICLE_COUNT: usize = 10240; // crates/liquidfun-wasm/src/frame.rs
const MAX_FRAME_PARTICLES: usize = 10240; // crates/liquidfun-wasm/src/session.rs capture_frame
```

```ts
const MAX_PARTICLE_COUNT = 10240; // web/src/physics/frame.ts
```

Scene constants (per D-03–D-07):

| Scene | radius | spacing / emit stagger | grid or fill | maximum_count |
| --- | --- | --- | --- | --- |
| Dam Break | `0.06324555` | spacing `0.101193` (ratio 1.6) | Medium `48×40=1920`; Small `25×26=650`; Large `63×44=2772` | `10240` |
| Float or Sink | `0.06324555` | spacing `0.101193` | `45×40=1800` | `3840` |
| Color Mixer | `0.05692` | none (group fill) | `GROUP_RADIUS = 1.15` unchanged | `2200` |
| Jelly Drop | `0.050596` | none (group fill) | `JELLY_HALF_EXTENT = 1.2` unchanged | `2200` |
| Fountain | `0.05692` | emit `0.025298` (was `0.08`) | Low/Med/High emit `10/20/30` per step | `3200` |
| Water Wheel | `0.050596` | emit `0.044272` (was `index * 0.14`) | `EMIT_PER_STEP = 20` | `3200` |

Introduce `EMIT_SPACING` on Fountain and Water Wheel. Do not reuse Water Wheel `PADDLE_HALF_WIDTH` (also `0.14`) for emit stagger.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Raise copied frame particle cap to 10240</name>
  <files>crates/liquidfun-wasm/src/frame.rs, crates/liquidfun-wasm/src/session.rs, web/src/physics/frame.ts, web/tests/frame.test.ts</files>
  <behavior>
    - Test: `parseRenderFrame` rejects `particleCount: 10241` and accepts a well-formed 10240-particle fake frame.
    - Test: `FrameData::new` still returns `ParticleCountExceeded` for `MAX_PARTICLE_COUNT + 1` (existing test tracks the constant).
    - Test: `session.rs` capture still uses the same numeric cap as `frame.rs` (`MAX_FRAME_PARTICLES == 10240`).
  </behavior>
  <action>
Raise the copied WASM/JS frame particle cap from 512 to 10240 in both languages (D-01). Do this before any scene exceeds 512 or `capture_frame` will fail closed.

1. In `web/tests/frame.test.ts`, rename `rejects particle counts above 512` to `rejects particle counts above 10240`, use `particleCount: 10241`, and add one Arrange/Act/Assert test that a 10240-count fake frame with matching lane lengths parses. Keep existing stride/type/NaN tests.
2. Set `MAX_PARTICLE_COUNT = 10240` in `web/src/physics/frame.ts`.
3. Set `MAX_PARTICLE_COUNT` to `10240` in `crates/liquidfun-wasm/src/frame.rs`. Set `MAX_FRAME_PARTICLES` to `10240` in `crates/liquidfun-wasm/src/session.rs`. Do not retune Dam Break `192` / radius `0.2` assertions yet (Task 2).
4. Do not change `MAX_RIGID_SEGMENTS` / `MAX_RIGID_CIRCLES`. Do not edit `crates/liquidfun`.

Rust 2024: no `unwrap` in production; `maybe_` naming stays as-is.
  </action>
  <verify>
    <automated>cargo test -p liquidfun-wasm --lib frame::tests -- --nocapture &amp;&amp; bun --cwd web run test:unit tests/frame.test.ts</automated>
  </verify>
  <done>Rust `MAX_PARTICLE_COUNT`, session `MAX_FRAME_PARTICLES`, and JS `MAX_PARTICLE_COUNT` are all 10240. JS rejects 10241 and accepts 10240. Rigid caps unchanged.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Pack Dam Break, Float or Sink, Color Mixer, and Jelly Drop ~10×</name>
  <files>crates/liquidfun-wasm/src/scene.rs, crates/liquidfun-wasm/src/scene/dam_break.rs, crates/liquidfun-wasm/src/scene/dam_break/tests.rs, crates/liquidfun-wasm/src/scene/float_or_sink.rs, crates/liquidfun-wasm/src/scene/color_mixer.rs, crates/liquidfun-wasm/src/scene/color_mixer/tests.rs, crates/liquidfun-wasm/src/scene/jelly_drop.rs, crates/liquidfun-wasm/src/session.rs, crates/liquidfun-wasm/src/lib.rs</files>
  <behavior>
    - Dam Break default create: particle count 1920, radii all `0.06324555`, `maximum_count() == Some(10240)`, gravity still `(0, -10)`.
    - Dam Break water-amount: small 650, large 2772, medium 1920.
    - Float or Sink default pool: exactly 1800 particles, `maximum_count` 3840, drop-body radii still `0.5`.
    - Color Mixer and Jelly Drop: `GROUP_RADIUS` / `JELLY_HALF_EXTENT` unchanged; constructed counts land in ~10× the old ranges and `≤ 2200`.
  </behavior>
  <action>
Increase scene counts ~10× and pack finer so occupied volume stays similar (D-02, D-03, D-04, D-05, D-07). Update tests that pin old numbers (D-10). One concept per test; keep Arrange/Act/Assert.

**Dam Break** (`dam_break.rs` + `dam_break/tests.rs` + `scene.rs` module docs):
- `PARTICLE_RADIUS = 0.06324555`, `PARTICLE_SPACING = 0.101193`.
- Medium `48×40` (`PARTICLE_COUNT = 48 * 12` is wrong — use `48 * 40 = 1920`). Small `25×26 = 650`. Large `63×44 = 2772`. Keep `u8` columns/rows.
- `MAXIMUM_PARTICLE_COUNT = 10240` (Large 2772 plus headroom).
- Origins, basin, obstacle radius/positions, drop-obstacle, pointer clamps: unchanged (D-08).
- Update `scene.rs` docs from `16×12 = 192`, radius `0.2`, spacing `0.32`, cap 512 to `48×40 = 1920`, radius `0.06324555`, spacing `0.101193`, cap 10240.
- Tests: `default_create_is_still_medium_normal_basin` and `water_amount_presets_recreate_with_locked_counts` → 1920 / 650 / 2772.

**Float or Sink**:
- Same radius/spacing as Dam Break. Grid `45×40 = 1800`. `MAXIMUM_PARTICLE_COUNT = 3840` (10× of 384, still ≤ 10240).
- `PARTICLE_ORIGIN`, drop radius `0.5`, densities, body cap 4: unchanged.
- Tests: replace `(120..=220)` / `<= 384` with exact `1800` and `<= 3840`.

**Color Mixer / Jelly Drop** (D-05):
- Color Mixer radius `0.05692`, cap `2200`. Jelly radius `0.050596`, cap `2200`.
- Do not change `GROUP_RADIUS`, `JELLY_HALF_EXTENT`, mix strengths, poke impulse, or rigid bars.
- If a filled group exceeds 2200 and construction fails, raise that scene's `maximum_count` only as far as needed, still `≤ 10240`, and pin the measured count. Do not change public engine APIs.
- Tests: Color Mixer `(40..=220)` → `(400..=2200)` in `color_mixer/tests.rs` and `session.rs` `create_color_mixer_constructs_a_live_world`. Jelly `(8..=220)` → `(80..=2200)`; `<= 512` → `<= 10240`.

**Session / lib Dam Break pins** in `session.rs` and `lib.rs`: every `192` live count → `1920`; `repeat(192)` color lanes → `1920`; `vec![0.2; 192]` radii → `vec![0.06324555; 1920]`; `maximum_count() == Some(512)` → `Some(10240)`; position/color/radius lane lengths `192 * stride` → `1920 * stride`. Do not rewrite unrelated parse/pointer tests beyond those counts.

Do not edit Fountain or Water Wheel here.
  </action>
  <verify>
    <automated>cargo test -p liquidfun-wasm --lib -- --nocapture</automated>
  </verify>
  <done>Dam Break Medium is 1920 at the locked radius/spacing with cap 10240. Float or Sink is 1800/3840. Color Mixer and Jelly keep group extents, sit near 10× old counts, and stay ≤ 2200 unless a measured fill required a documented cap bump ≤ 10240. Rigid bodies unchanged. Fountain/Water Wheel still at old emit rates but under the new frame cap.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Scale Fountain and Water Wheel emission ~10× and update remaining pins</name>
  <files>crates/liquidfun-wasm/src/scene/fountain.rs, crates/liquidfun-wasm/src/scene/water_wheel.rs, crates/liquidfun-wasm/src/scene/water_wheel/tests.rs, web/e2e/rust-wasm-proof.spec.ts</files>
  <behavior>
    - Fountain default Medium stream plateaus at or below 3200 after 240 steps; last 30 steps climb by at most 20 (10× the old +2 slack).
    - Fountain `emission-rate=off` still does not increase count. Lifetime stays 3.0 unless a plateau test cannot pass, in which case tweak lifetime only enough to stay below 3200 (D-06).
    - Water Wheel 240 on-steps plateau at or below 3200 and do not climb over the last 30 steps. Hub/paddle sizes unchanged; motor stays off.
    - E2E Dam Break copy and proof JSON use 1920 particles, not 192.
  </behavior>
  <action>
Scale emitter scenes the same packing way (D-03, D-06). Keep lifetimes unless tests require a small tweak to plateau below the new cap.

**Fountain**:
- `PARTICLE_RADIUS = 0.05692`, `MAXIMUM_PARTICLE_COUNT = 3200`.
- `particles_per_step`: Off 0, Low 10, Medium 20, High 30 (`u8` is enough).
- Replace `index * 0.08` with `EMIT_SPACING = 0.025298` so 10–30 nozzle particles occupy similar world width (D-03 packing). Do not change `NOZZLE_POSITION`, speeds, or tau aim angles.
- Rename/update `default_medium_stream_plateaus_at_or_below_three_hundred_twenty`: `end_count <= 3200` and `end_count <= mid_count + 20`. Change the diagnostic string that says "climbing toward 512" to 10240.

**Water Wheel**:
- `PARTICLE_RADIUS = 0.050596`, `MAXIMUM_PARTICLE_COUNT = 3200`, `EMIT_PER_STEP = 20`.
- Add `EMIT_SPACING = 0.044272` for `JET_POSITION.y + f32::from(index) * EMIT_SPACING`. Leave `PADDLE_HALF_WIDTH = 0.14`, hub, paddles, `JET_POSITION`, and fixture friction untouched (D-08).
- `water_wheel/tests.rs`: `end_count <= 320` → `<= 3200`. Keep motor-off and rotation proofs.

**E2E** (`web/e2e/rust-wasm-proof.spec.ts`): `"Particles: 192"` → `"Particles: 1920"`; `counts.particles: 192` → `1920`. Do not run `just demo-media` unless a test fails without new bytes (D-11).

After tests pass: `cargo fmt --all`. One atomic git commit of production/test code only (D-12). Do not stage `.planning/**` or `.vscode/`. Message should explain why: finer ~10× playground particles under a 10240 copied-frame cap.
  </action>
  <verify>
    <automated>cargo test -p liquidfun-wasm --lib -- --nocapture &amp;&amp; bun --cwd web run test:unit tests/frame.test.ts</automated>
  </verify>
  <done>Fountain and Water Wheel caps are 3200, emit ~10× per step with finer nozzle stagger, and plateau tests pass. E2E Dam Break count is 1920. No canvas draw-scale. No demo-media regeneration unless a failing test required it. Code commit does not include `.planning` docs.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
| --- | --- |
| WASM capture → JS `parseRenderFrame` | Untrusted or malformed copied lanes; count must stay bounded. |
| Scene constructors / emitters → particle system | Emission and group fill can grow until `maximum_count` and the frame cap. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
| --- | --- | --- | --- | --- |
| T-wbn-01 | D | `capture_frame` / `parseRenderFrame` | mitigate | Keep a hard 10240 particle cap in Rust `FrameData`, session capture, and JS `parseBoundedCount`; reject 10241. |
| T-wbn-02 | D | Fountain / Water Wheel emitters | mitigate | Keep destruction-by-age plus `maximum_count` 3200; full-system create remains a no-op; plateau tests must stay below 3200. |
| T-wbn-03 | T | Scene `maximum_count` | mitigate | Every scene cap stays ≤ 10240; Dam Break Large 2772 uses 10240 only as headroom. |
| T-wbn-04 | I | Public `liquidfun` defaults | mitigate | Do not edit `crates/liquidfun` radius/spacing defaults; playground-only constants. |
| T-wbn-05 | E | WASM/JS cap skew | mitigate | Same literal 10240 in `frame.rs`, `session.rs`, and `frame.ts`. |
</threat_model>

<verification>
1. `cargo test -p liquidfun-wasm --lib`
2. `bun --cwd web run test:unit tests/frame.test.ts`
3. Confirm no `crates/liquidfun` diff and no canvas camera/draw-scale change.
4. Confirm rigid radii (0.75, 0.5, 0.35, paddles) unchanged.
5. Note stale demo-media stills in SUMMARY; regenerate only if a test failed without it.
</verification>

<success_criteria>
- [ ] Frame caps are 10240 in Rust frame, Rust capture, and JS parser, with tests for 10240 accept / 10241 reject
- [ ] Dam Break Medium/Small/Large are 1920/650/2772 at radius 0.06324555 and spacing 0.101193, system cap 10240
- [ ] Float or Sink is 1800 particles, cap 3840; Color Mixer and Jelly Drop ~10× via finer radius and unchanged group extents, cap 2200
- [ ] Fountain and Water Wheel emit ~10× per step, cap 3200, finer emit stagger, lifetimes unchanged unless plateau tests required a small tweak
- [ ] Rigid bodies and canvas draw-scale unchanged; no engine-wide particle default edits
- [ ] Native wasm lib tests and web frame tests pass; executor committed code only
</success_criteria>

<output>
After completion, create `.planning/quick/260919-wbn-raise-webapp-particle-frame-cap-20x-to-1/260919-wbn-SUMMARY.md` (executor may write it; do not git-add `.planning/**`).
</output>
