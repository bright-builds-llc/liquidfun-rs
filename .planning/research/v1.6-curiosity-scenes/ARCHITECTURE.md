# Architecture Research: v1.6 Physics Curiosity Scenes (scene integration)

**Domain:** Adding six playground scenes (Siphon, Archimedes screw, Sand hourglass, Cartesian diver, Rayleigh–Taylor instability, Fake ferrofluid) to the existing native-Rust → private wasm bridge → SolidJS playground.
**Researched:** 2026-10-14 (repo at `b151838be`)
**Confidence:** HIGH for the integration checklist, seams, and count sites (read directly from source). MEDIUM for the per-scene mechanism choices (engine APIs verified to exist; visual results need spikes).

## Bottom line

1. **No new bridge API, worker message, or engine change is needed.** All four new interactions fit the existing `SceneHooks` seams (`apply_pointer`, `apply_action`, `apply_control` → `ControlEffect::Live`, `on_advance`). Every engine call they need already exists in `crates/liquidfun` (verified below). Keep `crates/liquidfun` untouched, and the 25 fingerprints are safe by construction.
1. **Adding a scene touches about 9 Rust sites and 13 web sites.** About half are compile-forced (exhaustive `match` or `Record<SceneId, …>`). The other half are silent hard-coded counts and lists that fail only at test time. Fix the silent ones once, up front.
1. **Two files hit the 628-line Bright Builds limit before scene 31.** `web/src/catalog/scene-records.ts` is at 617 lines and `web/src/catalog/previews.tsx` at 562, and `scripts/bright-builds-check.ts:8` sets `FILE_LINE_LIMIT = 628` with no `.bright-builds-rules-checks.tsv` exceptions. Split both before adding the first record.
1. **The fingerprint gate must change shape.** Since v1.5 it has been "`diff base.txt after.txt` with exactly 25 lines." With new scenes, `just playground-scene-spot` emits 26, then 27, … 31 lines, so a plain `diff` fails on the first new scene. Gate on "every id in the baseline is present and identical; new ids may appear."
1. **Two physics facts change the risk order.** First, the particle pressure solver clamps pressure at zero (`crates/liquidfun/src/particle/solver/pressure.rs:86`, `max(0.0, weight - MIN_PARTICLE_WEIGHT)`), so water has no tension and a siphon will not pull on its own. Second, the v1.5 Phase 40-08 spike measured `POWDER` behaving like water to within about 25%, so an hourglass angle of repose will not come from the `POWDER` flag. Siphon and Hourglass need early time-boxed spikes and go last.

## Standard Architecture (as it exists today)

### System Overview

```
┌──────────────────────────────── web/src (SolidJS, main thread) ───────────────────────────────┐
│ catalog/scenes.ts SCENE_IDS ──► catalog/scene-records.ts SCENES ──► components/SceneControls  │
│      │                         (controls, hint, credits, viewBounds)   (preset│range│action)    │
│      ├──► catalog/portrait-bounds.ts PORTRAIT_VIEW_BOUNDS (Record<SceneId>)                    │
│      ├──► catalog/previews.tsx ScenePreview switch                                             │
│      └──► routing/hash, DemoNavigation sidebar (derived, no per-scene edits)                   │
│ input/canvas-pointer.ts ──(down/move/up/cancel, world coords)──┐                               │
│ player/worker-controls.ts ──(control name/value, action name)──┤                               │
├────────────────────────────────────────────────────────────────┼─ physics/worker-messages.ts ──┤
│ physics/simulation-worker.ts  op: control | action | pointer | gravity | advance | capture    │
├─────────────────────────── crates/liquidfun-wasm (private bridge) ────────────────────────────┤
│ lib.rs ProofSession: applyControl / applyAction / pointerAction / setGravity / advance        │
│ session.rs SessionCore ──► scene.rs parse_scene_id + build_scene ──► scene/<id>.rs build()    │
│   advance(n≤4): hooks.on_advance → evict_escaped → world.step → hooks.on_after_step           │
│   frame.rs capture: particles ≤16384, rigid segments ≤512, rigid circles ≤48                  │
│ scene_spot.rs SURVEY_SCENES (native survey + FNV-1a end-state fingerprint)                    │
├────────────────────────────────── crates/liquidfun (engine) ──────────────────────────────────┤
│ World, particle system/groups/forces, bodies, joints (unchanged in v1.5; keep unchanged)       │
└───────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Component responsibilities for a new scene

| Component | Owns | Status for v1.6 |
| --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/<id>.rs` (+ `<id>/tests.rs`, `<id>/geometry.rs`) | World construction, a `SceneHooks` impl, scene constants | **New** ×6 |
| `crates/liquidfun-wasm/src/scene.rs` | `mod`, `SceneId`, `parse_scene_id`, `build_scene`, `particle_iterations` | **Modified** |
| `web/src/catalog/<id>-controls.ts` | `SceneControl` constants for live sliders/spinners | **New** (screw only, maybe diver) |
| `web/src/catalog/scene-records*.ts` | Catalog record (title, description, hint, controls, credits, viewBounds) | **Modified + split** |
| `web/src/catalog/previews*.tsx` | Static SVG preview | **Modified + split** |
| `web/src/catalog/portrait-bounds.ts` | Phone frame per scene | **Modified** |
| `web/scripts/readme-svg/plans.ts`, `web/scripts/demo-media/model.ts` | README clip plan, capture plan | **Modified** |
| Tests and e2e lists | Contracts (listed below) | **Modified** |

## The per-scene integration checklist (template: v1.3 Phases 30–33, Washing Machine #51, Tesla Valve #65)

v1.3 split each scene into three or four plans: Rust scene plus proof test, then catalog/preview/portrait/README plan, then Chromium smoke, then sometimes a follow-up. See `.planning/phases/33-stacked-drip-fidget/33-0{1..4}-PLAN.md` `files_modified`. The Tesla Valve commit `d456577e0` is the most recent single-scene diff (24 files). It predates two later lists, `SURVEY_SCENES` (Phase 34) and the puzzle id cross-check (Phase 37), so add those to the list.

### A. Rust bridge (`crates/liquidfun-wasm`)

| # | File:line | Change | Forced? |
| --- | --- | --- | --- |
| R1 | `src/scene/<id>.rs` (+ `<id>/tests.rs`, optional `<id>/geometry.rs`) | **New.** `pub(super) fn build(presets) -> Result<BuiltScene, SessionError>` and a `SceneHooks` impl. Use the `foo.rs` + `foo/` layout. The catalog credit path must be a top-level `scene/<id>.rs`, because `web/src/catalog/links.ts:22-33` rejects subdirectories. | — |
| R2 | `src/scene.rs:9-36` | Add `mod <id>;` | compile |
| R3 | `src/scene.rs:45-71` | Add a `SceneId::<Variant>` | — |
| R4 | `src/scene.rs:73-101` | Add `"<id>" => Ok(SceneId::<Variant>)` | **silent** (unknown id is `Err`) |
| R5 | `src/scene.rs:214-250` `build_scene` | Add an arm | compile (exhaustive) |
| R6 | `src/scene.rs:106-115` `particle_iterations` | Add an arm only if the scene needs more than 2 particle substeps (fine grains, small vessels; see Liquid Tumbler, Stacked Drip, Tesla Valve) | optional |
| R7 | `src/scene/gravity_slider.rs:122,145` (`[false; 25]`, `[true; 25]`), `:161-189` `variant_index`, `:191-219` `all_scene_ids() -> [SceneId; 25]` | Bump to 31 and add the variant. `variant_index` is compile-forced, but the array sizes and `all_scene_ids` contents are **silent**: a missing id goes untested, and only the size mismatch fails to compile. | partly |
| R8 | `src/session/tests.rs:14-42` `parse_scene_id_maps_allowlisted_tokens` | Add the token | **silent** |
| R9 | `src/scene_spot.rs:46-104` `SURVEY_SCENES: [...; 25]` | Append `("<id>", SceneId::<Variant>, SurveyCue::…)` in catalog order and bump the length | compile (length) plus test `survey_scenes_match_catalog_scene_ids` (`scene_spot/tests.rs:27`) checks order against `SCENE_IDS` |
| R10 | `src/scene_spot/tests.rs:71` | Update the exact scripted list only if the new scene uses a non-Default cue | test |
| R11 | `src/puzzle/tests.rs:77` `assert_eq!(scene_ids.len(), 25)` | Bump | **silent** count |
| R12 | `src/session/escape.rs:33-41` `EscapeLimits::for_scene` | Only if the playfield exceeds 12 m fall / 48 m sideways (Wave Tank precedent) | optional |

### B. Web catalog and player (`web/src`)

| # | File:line | Change | Forced? |
| --- | --- | --- | --- |
| W1 | `catalog/scenes.ts:11-37` `SCENE_IDS` | Append the id. Order is sidebar, README, survey and demo-media order. | — (source of truth) |
| W2 | `catalog/scene-records.ts:124-617` `SCENES` | Add a record using `withGravitySlider([...])`, `WATCH_FIRST_HINT` or a written hint, `sceneSource("<id>.rs")`, `[SHOWCASE]`, and `viewBounds`. **Split first** (617 of 628 lines). | test `scenes.test.ts` |
| W3 | `catalog/<id>-controls.ts` | **New** when the scene has live sliders. Copy `washing-machine-speed.ts` (`surface: "hud"`, `widget: "spinner"`, `recreates: false`). | — |
| W4 | `components/scene-controls.ts:7-20` `DEFAULT_PRESET_VALUES` | Only for new `preset` controls. **Control ids share one namespace across scenes**, so choose unique ids. | — |
| W5 | `catalog/previews.tsx:510-562` `ScenePreview` switch plus a new `<Id>Preview()` | Add a case. **Split first** (562 lines, and six previews of 15–30 lines each exceed 628). | **silent** (no explicit return type and no `noImplicitReturns` in `web/tsconfig.json`; a missing case renders nothing and only the `shell.spec.ts` preview count catches it) |
| W6 | `catalog/portrait-bounds.ts:69-95` `PORTRAIT_VIEW_BOUNDS: Record<SceneId, WorldBounds>` | Add a phone frame (AGENTS.md "Playground canvas framing") | compile |
| W7 | `player/runtime.ts:10-11` `PAGE_SUMMARY` "All twenty-five demos" | Update the words | **silent** (e2e text) |
| W8 | `components/PlaygroundShell.tsx:73` comment "25-preview" | Update the comment | none |

### C. Scripts, tests, e2e

| # | File:line | Change | Forced? |
| --- | --- | --- | --- |
| S1 | `web/scripts/readme-svg/plans.ts:48-83` `README_SVG_PLANS` | Append `{ id, cues }` in catalog order. `assertReadmeSvgPlanCoverage` (`:141`) checks length, order and duplicates. Cue kinds today are only `action` and `pointer-up` (`:23-34`). | runtime assert |
| S2 | `docs/assets/readme/<id>-10s.svg` + `.webp` | Generated by `just readme-svg`. The README gallery is upserted. WebP must stay under GitHub's 50 MB warning (`readme-svg/webp.ts:94`). | — |
| S3 | `web/scripts/demo-media/model.ts:96-290` `SCENE_CAPTURE_PLANS` | Append `{ id, title, route, interactionStep: 180, action: click|drag }` | runtime assert + test |
| T1 | `web/tests/scenes.test.ts` `:25` `UI_SPEC_DESCRIPTIONS`, `:69` `UI_SPEC_HINTS` (both `Record<SceneId>`); `:140-176` id list and `toHaveLength(25)`; `:180-206` titles; `:239` `toBe(25)`; `:433-441` length 25 | Add entries and bump counts | Records compile; **counts silent** |
| T2 | `web/tests/scene-catalog-controls.test.ts:20-51` `RECREATING_CONTROL_IDS` | Append `"gravity"` (plus any recreating control) per scene; add per-scene control assertions near `:360-380` | test |
| T3 | `web/tests/scene-catalog-credits.test.ts:68` `expectedPaths: Record<SceneId>`, `:~213` inspiration list | Add | compile + test |
| T4 | `web/tests/portrait-bounds.test.ts:45` `MIN_HEIGHT_FRACTION: Record<SceneId>`, `:76` optional width fraction, `:~300-395` wall endpoints | Add a fraction and wall points | compile + test |
| T5 | `web/tests/demo-media-model.test.ts:28,32` `toHaveLength(25)`, `.at(-1) === "tesla-valve"` | Bump the count and change the last id | **silent** |
| T6 | `web/tests/controls.test.ts` | Add a HUD control test per new live control (Tesla Valve precedent at `:495+`); file is at 529 lines | — |
| E1 | `web/e2e/player-helpers.ts:30-56` `SCENE_HASH_PATHS: Record<SceneId>` | Add | compile (`bun run typecheck` covers e2e since Phase 44) |
| E2 | `web/e2e/player.spec.ts:40-56` `POINTER_CONTROL` (Partial) | Add `{ gesture: "click"|"drag", control: "<label>" }` for interactive scenes, otherwise the scene falls into the watch-first loop | **silent** |
| E3 | `web/e2e/shell.spec.ts:58,61` "All twenty-five demos"; `:235-236`, `:252-253` tab loops `< 28` (Dismiss + Puzzles + N links); `:247` `lastLink` `/Tesla Valve\b/`; `:323,332,335` "twenty-five" / `toHaveCount(25)` | Bump the counts and change the last title | **silent** |
| E4 | `web/e2e/hud-safe-frame.spec.ts:149-156` | Nothing to edit: `OVERLAP_CASES` iterates `SCENE_IDS` (2 viewports, or 4 when the scene has HUD controls). Each new scene adds 2–4 Chromium tests. | auto |
| E5 | `web/e2e/player-helpers.ts:23` `ALL_SCENE_TIMEOUT_MS = 380_000` | The sequential all-scene loops in `player.spec.ts:302,342` grow by 6 scenes, so raise this if they time out | maybe |

**Auto-derived, no per-scene edit:** hash routing, the sidebar, `ScaleLegend`, the HUD safe frame, `tools/xtask/src/playground/spot.rs:24-37` (parses `SCENE_IDS` from `scenes.ts`), and the gravity slider control (via `withGravitySlider`).

**Historical, do not edit:** `docs/benchmarks/scene-survey.md` ("all 25" is a dated record).

## Hard-coded 25 sites (25 → 31)

Silent sites (fail only at test time or not at all). Fix them once in the groundwork plan:

| Site | Recommended de-hardcode |
| --- | --- |
| `crates/liquidfun-wasm/src/scene/gravity_slider.rs:122,145,191` | Add `pub(crate) const ALL_SCENE_IDS: [SceneId; N]` beside the enum in `scene.rs` and size `seen` with `ALL_SCENE_IDS.len()`. Keep `variant_index` as the exhaustive `match`; it is the compile-time completeness check. |
| `crates/liquidfun-wasm/src/scene_spot.rs:46` | Keep the explicit array (the cue is per scene) and keep the order test. |
| `crates/liquidfun-wasm/src/puzzle/tests.rs:77` | `assert_eq!(scene_ids.len(), ALL_SCENE_IDS.len())` |
| `web/tests/scenes.test.ts:174,239,441` | `SCENE_IDS.length`. Keep the explicit id and title lists, which are intentional UI-SPEC contracts. |
| `web/tests/demo-media-model.test.ts:28,32` | `SCENE_IDS.length` and `SCENE_IDS.at(-1)` |
| `web/e2e/shell.spec.ts:58,61,235-253,323-335` | Import `PAGE_SUMMARY`, derive focusables as `SCENE_IDS.length + 2`, and take the last link from `SCENES.at(-1)!.title` |
| `web/src/player/runtime.ts:11` | Product copy stays a literal ("All thirty-one demos" at the end). It is the single edit per added scene. |

After this refactor, each new scene needs only the compile-forced entries plus R4, R8, R9, W1, W2, W7, S1, S3, T1 (text entries) and E2.

## New interaction seams and how they route

Everything already flows `canvas → attachCanvasPointer → worker {op:"pointer"} → ProofSession.pointerAction → SessionCore::apply_pointer → hooks.apply_pointer(kind, world_x, world_y)`. `web/src/input/pointer.ts:99-130` forwards `move` **only while a pointer is captured** (pressed). There is no hover. `syncCanvasInteractive` (`web/src/player/playground-session.ts:142-148`) enables the canvas for every scene route, so no per-scene enable flag is needed. Panel `action` buttons go `applyWorkerAction` → `{op:"action"}` → `hooks.apply_action`. While paused, applying an action also advances one step (`web/src/player/worker-controls.ts:104-140`). HUD surfaces render only `range` controls (`web/src/components/scene-controls.ts:37-43`); actions and presets live in the panel.

**Rule for all six scenes:** store user intent in the hooks struct and apply physics in `on_advance` once per fixed step. Never apply forces directly in `apply_pointer`, because the engine zeroes the particle force buffer after each `SolveForce` (commit `77fbd84`) and pointer messages arrive between advance batches of up to 4 steps (`MAX_ADVANCE_STEPS`, `session.rs:25`). Express every scene force along `world.gravity()` so the gravity slider and phone tilt (`SessionCore::set_gravity`, `session.rs:196`) keep working.

| Scene | Interaction | Seam (existing) | Engine call (verified to exist) | New infra? |
| --- | --- | --- | --- | --- |
| Fake ferrofluid | Spikes toward the pointer | `apply_pointer`: `Down`/`Move` store `maybe_magnet: Option<Vec2>`; `Up`/`Cancel` clear it (or return it to a resting magnet). `on_advance` applies per-particle forces. | `World::apply_particle_force(system, particle, force)` (`world/particle_object/system.rs:359`) or `apply_particle_force_range` (`:377`); the range force is **total, divided among members**, which is why `impulse.rs:189` multiplies by member count | None. Drag only (no hover). |
| Cartesian diver | Press and hold to squeeze | `apply_pointer`: `Down` → squeezed, `Up`/`Cancel` → released. Optional keyboard path: a panel `action` toggle ("Squeeze") or a live `preset` (`recreates: false`, Impulse "Push" precedent). | `World::set_body_gravity_scale` (`world/object/body_object.rs:359`; does **not** wake the body, so build the diver with sleep disabled); or `set_fixture_density` (`fixture_object.rs:271`, does **not** recompute mass) + `reset_body_mass_data` (`:238`). Optional visual bow: `collect_segments` returns a squeezed outline from hooks state. | None. A dedicated "hold" control kind would be new TS infra and is not recommended. |
| Sand hourglass | Flip to restart | `apply_action("flip")` (panel button) plus `apply_pointer(Up)` (tap, as in Impulse and Float or Sink) | Instant flip: a 180° rotation about the neck center, p′ = 2c − p, v′ = −v, through `set_particle_position` / `set_particle_velocity` (`system.rs:275,288`). This is exact when the glass is point-symmetric, and the walls never move. Animated flip: kinematic glass body via `set_body_angular_velocity` (`body_object.rs:139`) for a fixed step count, then snap with `set_body_transform` (`:446`). Tunneling risk (Washing Machine keeps its wall thick enough for one step at max speed). | None for the instant flip. Phone tilt already rotates gravity in every scene. |
| Archimedes screw | Motor speed | `apply_control("screw-speed", rpm)` → `ControlEffect::Live`. TS `SceneControl` `range`, `surface:"hud"`, `widget:"spinner"` (copy `web/src/catalog/washing-machine-speed.ts`). | Revolute motor `set_revolute_motor_speed` (`washing_machine.rs:277`) or kinematic flights via `set_body_linear_velocity` (`body_object.rs:123`) | One new `web/src/catalog/archimedes-screw-speed.ts` (pattern, not infra) |
| Rayleigh–Taylor | Watch-first | `on_advance` only | Per-group density does **not** exist: density is per particle system (`ParticleSystemDef::with_density`, `particle/definition/system_definition.rs:72`). `BuiltScene` holds one `particle_system`, and LiquidFun systems do not interact with each other. Emulate the heavy layer with an extra force on the heavy group along `world.gravity()`, `apply_particle_force_range(system, heavy_members, (ρ_ratio−1)·m·g·count)`. Group mass is available from `ParticleGroupView::mass` (`particle/group/view.rs:154`). | None, and no engine change |
| Siphon | Watch-first | `on_advance` only (or nothing) | Pressure ≥ 0 (`pressure.rs:86`), so there is no tension. Needs a primed tube plus `TENSILE`/`VISCOUS` flags, or an honest variant. **Spike before committing.** | None |

### Data flow: pointer-held force (ferrofluid; the diver is analogous)

```
pointerdown/move (captured) ─► canvas-pointer.ts send(kind, wx, wy)
   ─► worker {op:"pointer"} ─► ProofSession.pointerAction ─► SessionCore::apply_pointer
   ─► FerroHooks.apply_pointer: self.maybe_magnet = Some(world point)        (no physics here)
frame loop: worker {op:"advance", n≤4} ─► SessionCore::advance
   ─► for each step: FerroHooks.on_advance(world, system)
         read view positions; per particle in radius: apply_particle_force(...)
      ─► evict_escaped ─► world.step ─► on_after_step
pointerup/cancel/scene switch (CanvasPointerHandlers.cancel) ─► maybe_magnet = None
```

### Avoid new cue kinds (README, survey, demo-media)

README cues support only `action` and `pointer-up` (`web/scripts/readme-svg/plans.ts:23-34`, applied in `web/scripts/readme-svg.ts:142-154`). `SurveyCue` supports only `Default`, `Action` and `PointerUp` (`crates/liquidfun-wasm/src/scene_spot.rs:27-34,421-432`). Design each scene so it moves with zero cues or one action cue:

- Ferrofluid: a resting "magnet" point below the puddle spikes it with no input; the pointer drags the magnet. Uses `Default`.
- Diver: a toggle `action` ("squeeze"). The README can cue squeeze at one sample and release at a later one; the survey uses `Action("squeeze")`.
- Hourglass: sand pours from the start (`Default`). The README can add an `action` "flip" cue mid-clip.
- Screw, RT, Siphon: `Default`.

Only if the owner insists on pointer-only interaction should the plan add `pointer-down`/`pointer-move` README cues and `SurveyCue::PointerDown`. Both are additive and fingerprint-safe for the existing 25, but they touch `plans.ts`, `readme-svg.ts`, `scene_spot.rs` and the `only_idle_scenes_are_scripted` test.

### Frame-lane limits that constrain scene design

`crates/liquidfun-wasm/src/frame.rs:8-10` and `web/src/physics/frame.ts:7-8` hard-code `MAX_PARTICLE_COUNT = 16_384`, `MAX_RIGID_SEGMENTS = 512` and `MAX_RIGID_CIRCLES = 48` on both sides. **Hourglass sand made of rigid circle bodies (the usual way to get friction and an angle of repose) cannot exceed 48 circles** without raising the cap in both files and the worker frame validator. Raising it does not change fingerprints, since the fingerprint hashes world state rather than frames, but it is a shared contract change. Keep grains as particles (for example small `RIGID` particle clumps) unless a spike proves circles are needed.

## Fingerprint gate implications

The current procedure (Phase 40-01 Task 1, `.planning/phases/40-timed-valve-and-teaching-levels/40-01-PLAN.md:131-134`) is:

```
just playground-scene-spot   # cargo xtask playground scene-spot → target/dam-break-perf/<stamp>/scene-spot.json
jq -r '.scenes[] | "\(.scene) \(.fingerprint)"' "$S/scene-spot.json" | sort > target/phaseNN/base.txt
test "$(wc -l < base.txt)" -eq 25 ; … ; diff base.txt after.txt
```

The fingerprint is FNV-1a 64 over particle positions, velocities, colors and body state after 60 warmup and 120 measured steps (`scene_spot/fingerprint.rs`). Each scene runs 3 fresh times and fails on `NondeterministicRun`.

Required changes for v1.6:

1. **Capture `base25.txt` once at milestone base `b151838be`** (`target/v1.6/base25.txt`, gitignored). Cross-check it against `target/phase44/after-final.txt` if present. Run executors on the main tree, since `target/` is invisible to worktrees.
1. **Gate by join, not diff.** Every id in the protected set must be present with an identical fingerprint; new ids may appear:
   ```
   join base.txt after.txt | awk '$2 != $3 {print "CHANGED", $1; bad=1} END {exit bad}'
   test "$(join base.txt after.txt | wc -l)" -eq "$(wc -l < base.txt)"
   ```
1. **Grow the protected set per phase.** When scene K ships, the phase's `after-final.txt` (25 + K lines) becomes the next phase's base. Later phases then also protect earlier v1.6 scenes from shared-code drift.
1. **What can actually move the 25:** any edit in `crates/liquidfun`; changes to `session.rs` `advance`/`from_built` (step configuration, the escape pass), `session/escape.rs` shared limits, `scene.rs` `DEFAULT_PARTICLE_ITERATIONS`, `gravity_slider.rs` apply logic, `basin_family.rs`/`soup_family.rs` shared builders, and the three existing survey cues. **Safe:** new `match` arms, new scene modules, new `SurveyCue` variants, frame-cap changes, and all TS changes. Do not reuse or "improve" `basin_family`/`soup_family` for new scenes; copy what you need.
1. `--scene` filtering exists (`scene_spot.rs:293` `resolve_scene_filter`) for quick per-scene determinism checks, but the `just playground-scene-spot` recipe (`justfile:180`) passes no arguments. Call `cargo xtask playground scene-spot --scene <id>` directly.

## Architectural patterns to follow

### Pattern 1: Intent in hooks, physics in `on_advance`
**What:** `apply_pointer`, `apply_action` and `apply_control` only mutate hook state (or do a one-shot, step-independent edit such as the hourglass flip). Continuous forces are applied in `on_advance`.
**Why:** It is deterministic per fixed step regardless of how many steps the worker batches per frame, and it respects the per-step force-buffer reset.

### Pattern 2: Bridge-level emulation instead of engine features
**What:** Per-group "density" (RT), magnet fields (ferrofluid) and squeeze (diver) are body or particle forces and gravity scales applied by scene hooks.
**Why:** `crates/liquidfun` stays byte-identical, as in v1.5, and the existing 25 fingerprints cannot move. The PROJECT.md key context explicitly allows engine features only if existing scenes stay unchanged; the bridge route makes that trivially true.
**Trade-off:** Captions must describe what is really simulated (the Transparency constraint). For example, "the heavy layer is pulled down harder", not "density 1.5×".

### Pattern 3: One live HUD control file per scene
**What:** `web/src/catalog/<scene>-speed.ts` exports a `SceneControl` constant. The Rust hook returns `ControlEffect::Live`, and the worker skips re-applying actions after a recreate (`worker-controls.ts:51`).
**When:** The screw speed. Any slider that should not rebuild the world.

### Pattern 4: Point-symmetric flip as a state transform
**What:** For a vessel symmetric under 180° rotation about c, "flip" is (p, v) → (2c − p, −v) for every particle (and body).
**Trade-off:** It is instant (no rotation animation) but exact, cheap and tunneling-free. Add animated rotation later only if the owner wants it.

## Anti-patterns

- **Adding a `liquidfun` engine feature (per-group density) for RT.** This puts all 25 fingerprints at risk for a visual effect the bridge can produce. Do the bridge force instead.
- **Applying force inside `apply_pointer`.** The impulse fires once per pointer message instead of per step, varies with frame rate, and is wiped by the force-buffer reset.
- **Hard-coding "down" as `(0, -1)` in scene forces.** This breaks the gravity slider (2–80 m/s²) and phone tilt. Use `world.gravity()`.
- **Adding a new `SceneControl` kind ("hold") or HUD actions.** That touches `SceneControls.tsx`, `scene-controls.ts`, worker replay rules and e2e helpers for one scene. Canvas hold plus a panel action covers mouse, touch and keyboard.
- **Pointer hover for ferrofluid.** It would change the shared `reducePointerEvent` used by every scene and puzzle, and there is no hover on touch anyway.
- **Appending to `scene-records.ts`/`previews.tsx` without splitting.** `bun scripts/bright-builds-check.ts all` fails at 629 lines.
- **Plain `diff` fingerprint gate.** It fails the moment scene 26 exists, which tempts people to regenerate the base and lose protection.

## Scaling considerations (what grows with 31 scenes)

| Concern | Effect | Mitigation |
| --- | --- | --- |
| Chromium time | `hud-safe-frame.spec.ts` adds 2–4 tests per scene; `player.spec.ts` sequential loops grow by 6 | Raise `ALL_SCENE_TIMEOUT_MS` if needed; run the HUD spec in parallel (already `mode: "parallel"`) |
| `just playground-scene-spot` wall time | 6 more scenes × 3 runs × 180 steps | Fine (≤ 3 min per run timeout, `scene_spot.rs:23`); keep each new scene under about 3,000 particles like the v1.5 levels |
| `just readme-svg` | Regenerates every clip; writes only changed files | One new SVG/WebP pair per scene |
| Rust debug test binaries | macOS scans each new binary (~2 min per debug test binary, memory note) | Keep scene proofs in the existing lib test binary (`<id>/tests.rs`), not new integration-test binaries |

## Suggested build order (lowest risk first, shared groundwork before dependents)

| Phase | Content | Depends on | Risk | Why here |
| --- | --- | --- | --- | --- |
| **45** | **Groundwork + Cartesian Diver.** Plan 01: split `scene-records.ts` and `previews.tsx`; add `ALL_SCENE_IDS` and de-hardcode the silent counts; capture `target/v1.6/base25.txt` and adopt the join gate. Optional: time-boxed **siphon and hourglass feasibility spikes** (ignored tests, removed after, as in Phase 40-08) so those scenes can be reshaped or dropped early. Plans 02–04: diver scene (rigid buoyancy is proven by Float or Sink and the `float-up` level), catalog record, preview, portrait frame, README plan, Chromium smoke. | v1.5 archive | Low | Unblocks every later scene. The diver uses only existing seams (pointer hold plus an action toggle). |
| **46** | **Archimedes screw** with a live HUD speed spinner | 45 | Low–medium | Copies the Washing Machine motor/spinner pattern; segment count well under 512 |
| **47** | **Rayleigh–Taylor instability** (watch-first, two colors, heavy-group force along gravity, seeded interface perturbation) | 45 | Medium | Bridge-only force; risk is visual (fingers vs blobs) and particle budget |
| **48** | **Fake ferrofluid** (resting magnet plus pointer drag, per-particle force field) | 45 | Medium | Pure pointer seam. "Fake" gives latitude, but spikes need tuning. Check dark particles against the default theme. |
| **49** | **Siphon** (primed tube, `TENSILE`/`VISCOUS` water, levels equalize) | 45 spike result | Medium–high | No tension in the pressure solver, so it must be proven by the spike or reshaped honestly |
| **50** | **Sand hourglass** (granular look plus instant point-symmetric flip) | 45 spike result | High | `POWDER` ≈ water (40-08); rigid-circle grains hit the 48-circle frame cap. Last so it can slip. |

Ordering rationale: groundwork first because every scene edits the files that would exceed 628 lines and every phase needs the new gate. Then by physics risk: proven mechanisms (buoyancy, motors) before emulated ones (RT force, magnet field) before ones the engine fights (no tension, no granular friction). No scene depends on another scene's code. The only cross-scene dependencies are the Phase 45 groundwork and spike results.

Each scene phase keeps the v1.3 shape. Plan 01: Rust scene, a `SessionCore::advance(4)` proof test in `<id>/tests.rs`, gate R2–R12. Plan 02: catalog record, preview, portrait frame, README and demo-media plans, unit tests. Plan 03: Chromium smoke (`player.spec`, `shell.spec`, HUD safe frame) plus the join fingerprint gate. Optional Plan 04: a polish fix.

## Integration points (internal boundaries)

| Boundary | Communication | Notes |
| --- | --- | --- |
| `scenes.ts` `SCENE_IDS` ↔ Rust | Text include. `scene_spot/tests.rs:9` and `puzzle/tests.rs` `include_str!` parse `scenes.ts`; xtask parses it too | Rust tests fail if the TS order or ids diverge from `SURVEY_SCENES` |
| Catalog control id ↔ `SceneHooks::apply_control` | String name/value | Unknown names must return `SessionError::UnknownControl`. Gravity is stripped before scene presets (`gravity_slider.rs:54`). |
| Action name ↔ `apply_action` | String | Same names serve the panel button, README `action` cue and `SurveyCue::Action` |
| Pointer ↔ `apply_pointer` | `PointerKind` + world coords | `Cancel` arrives on scene switch or detach; always release held state on `Up` **and** `Cancel` |
| Hooks ↔ frame | `collect_segments` / `collect_circles` / `collect_circle_labels` | Caps of 512 segments and 48 circles |
| Credits ↔ `links.ts` | `sceneSource("<id>.rs")` | Top-level file only |

## Sources (all repository-local, read 2026-10-14)

- `crates/liquidfun-wasm/src/scene.rs`, `scene/gravity_slider.rs`, `session.rs`, `session/escape.rs`, `session/tests.rs`, `scene_spot.rs`, `scene_spot/tests.rs`, `scene_spot/fingerprint.rs`, `puzzle/tests.rs`, `frame.rs`, `scene/impulse.rs`, `scene/washing_machine.rs`
- `crates/liquidfun/src/world/particle_object/system.rs`, `world/object/body_object.rs`, `world/object/fixture_object.rs`, `particle/definition/system_definition.rs`, `particle/solver/pressure.rs`
- `web/src/catalog/{scenes,scene-records,scene-record-shared,portrait-bounds,previews,links,washing-machine-speed}.ts(x)`, `web/src/input/{pointer,canvas-pointer}.ts`, `web/src/components/{SceneControls.tsx,scene-controls.ts}`, `web/src/player/{worker-controls,playground-session,runtime}.ts`, `web/src/physics/{worker-messages,frame}.ts`
- `web/scripts/readme-svg/plans.ts`, `web/scripts/readme-svg.ts`, `web/scripts/demo-media/model.ts`, `web/tests/*` and `web/e2e/*` as cited, `scripts/bright-builds-check.ts:8`, `justfile:180`
- Templates: commits `d456577e0` (Tesla Valve), `c0fe8fdff` and `09ea81605` (Washing Machine); `.planning/phases/3{0..3}-*/` plans; `.planning/phases/40-timed-valve-and-teaching-levels/40-01-PLAN.md` (fingerprint baseline) and `40-08-SUMMARY.md` (powder spike)

---
*Architecture research for: v1.6 Physics Curiosity Scenes — scene integration*
*Researched: 2026-10-14*
