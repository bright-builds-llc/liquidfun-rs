---
phase: 47-archimedes-screw
verified: 2026-10-14T22:53:46.287Z
status: passed
score: 4/4 roadmap success criteria verified (24/24 PLAN must-have truths)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: 2026-10-14T22:53:46.287Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 47: Archimedes Screw Verification Report

**Phase Goal:** Players can watch a turning screw lift water uphill in pockets, and change its speed or run it backwards.
**Verified:** 2026-10-14T22:53:46.287Z on HEAD 00296f088 (only `.planning/config.json` dirty)
**Status:** passed
**Re-verification:** No. This is the initial verification.

Provenance: 47-CONTEXT.md, 47-UI-SPEC.md, all five PLANs and all five SUMMARYs carry `lifecycle_mode: yolo` and `phase_lifecycle_id: 47-2026-10-14T20-01-42`. None is marked `direct-fallback`. `gsd-tools verify lifecycle 47 --require-plans --raw` prints `valid`.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | On load, moving slanted vanes in a tilted trough carry separate pockets of water from a low reservoir up to a higher basin, and the water returns down a chute so the scene keeps running. | VERIFIED | `archimedes_screw.rs`: one kinematic train body (`BodyType::Kinematic`, `with_sleeping_allowed(false)`) carries 7 slanted vanes (20 degrees, `VANE_TAN` from `TAU / 18`). The tube is tilted 30 degrees (`AXIS` from `TAU / 12`). `on_advance` sets the train velocity every fixed step, and `set_body_transform` snaps the train one pitch only when the phase wraps. The hooks start at `DEFAULT_SCREW_RPM = 30`, so the vanes move from the first step with no input. `geometry.rs` defines the reservoir, the closed tube, the weir basin, the chute, the hood and the return ramp. The code has no forces, impulses, emitters or teleports (grep is clean). I re-ran the release tests: `first_pocket_reaches_the_basin_within_five_seconds` (at least 30 particles in the basin by step 300), `readme_clip_shows_scooped_water_reaching_the_basin` (by particle id) and the spike test `sealed_pockets_hold_water_at_default_speed` (at least 15 particles per sealed pocket) all passed. I extracted frames 241 and 591 from the committed WebP myself. Frame 241 shows separate clumps between the slanted vanes and a thin layer in the basin. Frame 591 shows a full basin with drops falling down the chute and pockets still climbing. |
| 2 | Player can change the screw's speed live from a HUD control and run it in reverse, so it carries water back down. | VERIFIED | Rust: `screw-speed` takes `parse_screw_rpm` values (canonical tokens 0..=60) and `screw-direction` takes exactly `forward` or `reverse`. Both return `ControlEffect::Live` and store intent only, and `on_advance` reads `axial_speed(self.rpm, self.reverse)`. Web: `archimedes-screw-speed.ts` has `surface: "hud"`, `widget: "spinner"`, `recreates: false` and range 0 to 60 with default 30. The record uses `withGravitySlider([SCREW_SPEED_CONTROL, runtimePreset("screw-direction", "Direction", [forward, reverse])])`. Tests: `control_tests` (9 tests, which include session `Ok(false)` for both controls), `controls.test.ts` (the HUD ids are `["screw-speed"]` and the panel ids are `["screw-direction", "gravity"]`), and the release test `reverse_returns_water_to_the_reservoir` (47-02 measured a basin change of -41 and a reservoir change of +121). It passed on my rerun. The e2e `POINTER_CONTROL` entry activates the `Screw speed` control. 47-04 ran the e2e suite: 98 passed. |
| 3 | A native 60 s run at default and maximum speed shows water gained in the upper basin, no particles leaking through walls or evicted, and a reservoir that never runs dry. | VERIFIED | `flow_tests.rs` `default_speed_lifts_water_for_sixty_seconds` and `max_speed_lifts_water_for_sixty_seconds` each run 3,600 steps through `SessionCore`. Each asserts a basin gain of at least 100, zero particles more than r/2 inside any wall or vane (sampled every 4 steps), `live == built`, a minimum reservoir p95 surface of at least `INLET_COVER_Y`, and at most 2 % lost. I re-ran `cargo test -p liquidfun-wasm --release --lib archimedes_screw`: 62 passed, 0 failed, 0 ignored. That run covers all 8 flow and all 8 spike tests (`target/phase47-verify/release-tests.log`). 47-02 recorded a basin of 152 at default and 145 at max, minimum surfaces of 0.465 and 0.501 against -0.0137, and zero leaks and zero lost. The g2 and g80 smokes also pass. |
| 4 | The scene meets every shared scene rule, with a caption saying flat vanes stand in for a turning helix in a 2D side view, and the id-join gate reports every protected fingerprint unchanged, including the diver's. | VERIFIED | Caption: "...Flat vanes stand in for a turning helix in this 2D side view. This is an original experimental scene." A grep of the record for the UI-D66 forbidden claims finds none, and `scenes.test.ts` has the `SCREW_FORBIDDEN_CLAIMS` honesty test. SCENE-01: the credits use `sceneSource("archimedes_screw.rs")` and `[SHOWCASE]`. `ArchimedesScrewPreview` is wired through `previews.tsx:97`, and the portrait frame is `portrait-bounds.ts:100`. README plan `{ id: "archimedes-screw", cues: [] }`. The tracked SVG and WebP (600 frames, 1280x960) are linked at `README.md:200-204`. `withGravitySlider` is present, and the scale legend and HUD safe frame come from shared chrome (`hud-safe-frame.spec.ts` covers 4 viewports). The scene is last in `ALL_SCENE_IDS` (scene.rs:108), `SURVEY_SCENES` (Default cue) and `SCENE_IDS` (scenes.ts:40). SCENE-02: 1,729 particles and 0.688 ms/step median in native release (stamp 2026-10-14T22-42-14Z, start = end). Gate: I re-ran `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json`, which printed `PASS`, `protected: 27/27 unchanged` (including cartesian-diver and archimedes-screw) and `added (accepted): none`. `git diff --quiet 23541975c HEAD -- crates tools web Cargo.toml Cargo.lock xtask` holds, so the capture still represents the shipped code. |

**Score:** 4/4 roadmap truths verified. All 24 PLAN must-have truths are also verified, and they restate or refine these criteria.

### PLAN must-have notes

- **47-01 (geometry):** the plan's nominal tube was 5 m with 6 vanes and vane-parallel faces. The shipped tube is 6 m with 7 vanes. The bottom face leans 45 degrees, the inlet slab is undercut and, from 47-02, the top face is square to the axis. These are documented Rule 1 fixes within D-04 and D-12, and the truths about snap-inside-housing, embed, thickness and the speed assert still hold (geometry and spike tests pass). `visible_vane_outline` returns `Option<Vec<Vec2>>` instead of a parallelogram because of the leaning face. It still clips only against the two housing faces, and no shaft line is drawn.
- **47-02 truth 6 (README timing):** the plan's numeric form, "basin > tube count at load + 20", could not hold because the weir caps the basin near 150 and the tube holds 177 at load. The test instead counts reservoir-at-load particle ids that reach the basin by step 597 (at least 20; 41 measured). That checks the stated intent directly, and the roadmap criteria do not depend on the original numeric form.
- **47-04 truth 2 (26/26 plus added):** this was correct when 47-04 ran (`target/phase47/fingerprint-gate.log`). After 47-05, the final tree reports 27/27.
- **47-03 portrait frame:** the portrait frame reuses the landscape bounds. At 390x844 it is width-bound and fills 0.918 of the width and 0.3127 of the height, against a test floor of 0.3. AGENTS.md allows a wide scene to keep some vertical margin, and D-13 and UI-D72 chose this frame on purpose so no wall is clipped.

### Required Artifacts

| Artifact | Status | Details |
| --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/archimedes_screw.rs` | VERIFIED | 323 lines. Contains `impl SceneHooks for ArchimedesScrewHooks`, the live controls, `advance_phase`, the snap and the clamp. No `unwrap()`. |
| `.../archimedes_screw/geometry.rs` | VERIFIED | 546 lines. Wall quads, vane quads, clipped outlines, spawn grid and `FRAME_*`. Trig appears only in tests. |
| `.../archimedes_screw/spike_tests.rs` | VERIFIED | 237 lines. 8 release-only D-02 guards, each with `cfg_attr(debug_assertions, ignore`. |
| `.../archimedes_screw/flow_tests.rs` | VERIFIED | 256 lines. 8 release-only D-19, D-20 and D-17 tests. |
| `.../archimedes_screw/control_tests.rs` | VERIFIED | 283 lines. Allowlist and live-effect tests plus the TS-sync tests `web_speed_constants_match_rust` and `web_view_bounds_match_the_frame`. |
| `.../archimedes_screw/{tests,hooks_tests,test_support}.rs` | VERIFIED | 359, 278 and 235 lines. Every file is under the 628-line limit. |
| `web/src/catalog/archimedes-screw-speed.ts` | VERIFIED | `id: "screw-speed"`, HUD spinner, 0 to 60, default 30. |
| `web/src/catalog/scene-records-curiosity.ts` | VERIFIED | The record follows the diver, with the locked copy and `sceneSource("archimedes_screw.rs")`. |
| `web/src/catalog/previews-curiosity.tsx` | VERIFIED | `export function ArchimedesScrewPreview` matches the exact UI-D67 SVG. |
| `web/src/catalog/scene-view-bounds.ts` | VERIFIED | `ARCHIMEDES_SCREW_VIEW_BOUNDS` (-6.1, -2.0, 2.65, 4.45), with the docstring "Keep in sync with `FRAME_*`". |
| `docs/assets/readme/archimedes-screw-10s.svg` and `.webp` | VERIFIED | Tracked. The WebP has 600 frames. |
| `README.md` gallery entry | VERIFIED | Lines 200-204, between the markers. |
| `docs/benchmarks/scene-fingerprints.json` | VERIFIED | 27 entries. The last is `archimedes-screw` / `86a28d366447bbeb`. Provenance is 23541975c, stamp 2026-10-14T22-42-42Z. It was grown in single-file commit 4be8ffdab. |

### Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| `scene.rs` | `archimedes_screw.rs` | `SceneId::ArchimedesScrew => archimedes_screw::build` (289), `mod` (9), parse (139), `ALL_SCENE_IDS` (108) | WIRED |
| `on_advance` | `set_body_transform` + `set_body_linear_velocity` | A snap only on a phase wrap, then a clamped velocity toward the next lattice point | WIRED |
| `apply_control` | `on_advance` | `rpm` and `reverse` intent read by `axial_speed(self.rpm, self.reverse)` | WIRED |
| `scenes.ts` `SCENE_IDS` | Rust `ALL_SCENE_IDS` | `"archimedes-screw"` last in both, checked by the `include_str!` order test | WIRED |
| `scene-records-curiosity.ts` | `archimedes-screw-speed.ts` | `SCREW_SPEED_CONTROL` inside `withGravitySlider` | WIRED |
| `previews.tsx` | `previews-curiosity.tsx` | `case "archimedes-screw"` (97), before the never guard | WIRED |
| `README.md` | `archimedes-screw-10s.webp` | Gallery entry | WIRED |
| `scene-fingerprints.json` | full-catalog capture | `provenance.scene_spot_stamp` 2026-10-14T22-42-42Z | WIRED |
| `gravity_slider.rs` (`=> 26`), `session/tests.rs`, `scene_spot.rs`, `portrait-bounds.ts`, `plans.ts`, `demo-media/model.ts`, e2e `player-helpers.ts` / `player.spec.ts` | screw id | Registration entries present | WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Real data | Status |
| --- | --- | --- | --- | --- |
| `collect_segments` | vane outlines | `self.phase`, advanced each step from `axial_speed(rpm, reverse)` | Yes | FLOWING |
| Train motion | velocity | `train_position(next) - body_snapshot(train).position()`, clamped | Yes | FLOWING |
| Water lift | particle positions | Engine particle solver against the kinematic vanes and static walls (no scripted force) | Yes | FLOWING |
| HUD spinner | rpm | Web `SCREW_SPEED_CONTROL` to worker `apply_control("screw-speed")` to `self.rpm` | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Release scene, spike and flow tests | `cargo test -p liquidfun-wasm --release --lib archimedes_screw` | 62 passed, 0 failed, 0 ignored | PASS |
| Web typecheck and unit tests | `cd web && bun run typecheck && bun run test:unit` | 1,852 passed | PASS |
| Id-join gate on the final baseline | `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json` | PASS, 27/27 unchanged, added none | PASS |
| Capture still matches the code | `git diff --quiet 23541975c HEAD -- crates tools web Cargo.toml Cargo.lock xtask` | exit 0 | PASS |
| README clip shows pockets and spill | `webpmux -get frame 241/591` + `dwebp`, viewed | Pockets between vanes; full basin overflowing down the chute | PASS |
| Lifecycle provenance | `gsd-tools verify lifecycle 47 --require-plans --raw` | `valid` | PASS |

Committed evidence I did not rerun: `target/phase47/checks-04-run1.log` (all 15 local gates passed at 23541975c: fmt, both clippy runs, build, `cargo test --all-features` with 1,084 passed, liquidfun-wasm debug tests, xtask playground, web, bright-builds-check, web-build, 98 e2e passes, markdown-check, and the protected-files diff). Since then, only the baseline JSON and `.planning/**` have changed.

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| SCREW-01 | 47-01, 47-02, 47-03, 47-04, 47-05 | Moving slanted vanes in a tilted trough carry separate pockets from a low reservoir to a higher basin, and the water returns so the scene keeps running. | SATISFIED | Truths 1, 3 and 4: the kinematic vane train, the spike pocket guard, the 60 s flow tests, the chute return (D-07, no teleport), the README frames and the baseline growth. |
| SCREW-02 | 47-02, 47-03, 47-04 | Player can change the screw's speed live and run it in reverse so it carries water back down. | SATISFIED | Truth 2: the live `screw-speed` HUD spinner, the `screw-direction` preset and the release reverse test. |

No orphaned requirements. REQUIREMENTS.md maps exactly SCREW-01 and SCREW-02 to Phase 47, and both appear in plan frontmatter. SCREW-03 is a future requirement. Their Pending status in REQUIREMENTS.md is expected; the orchestrator ticks them.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| (screw module) | none | A grep for forces, impulses, `BodyType::Dynamic`, revolute joints, `unwrap()`, `println!`/`eprintln!`, TODO/FIXME and `consts::PI` matched nothing | none | none |
| `archimedes_screw/control_tests.rs` | 246-258 | The TS-sync test does not check `SCREW_SPEED_MIN`, `SCREW_SPEED_STEP` or the direction tokens (review IN-04) | Info | The web unit tests pin the web side, and the Rust tests pin the `forward`/`reverse` tokens. It is a drift guard only. |
| `archimedes_screw/tests.rs` | 71-107 | The snap-housing tests check one pose per end (review IN-01) | Info | The release spike tests (zero leaks at 30 and 60 rpm in both directions) cover the actual behavior. |
| `archimedes_screw.rs` | 88-89 | `advance_phase` can return exactly `PITCH` because of f32 rounding (review IN-07) | Info | Harmless: it gives the same lattice. |
| `web/scripts/readme-svg/` (pre-existing) | n/a | The standalone animated SVG sweeps lines when the segment order changes (`deferred-items.md`) | Info | Predates this phase (washing-machine, hydraulic-fountain and stacked-drip are also affected). The README gallery embeds the WebP, which is clean. |

The code review (47-REVIEW.md) reports 0 critical, 0 warning and 7 info findings. None of them blocks the goal.

### Human Verification Required

None. Every criterion was checked objectively: release tests, web unit tests, the gate, and README frames extracted from the committed WebP. A real-iPhone check is optional under the hobby-project scope.

### Gaps Summary

There are no gaps. The scene ships one kinematic train of slanted vanes that lifts separate pockets up a closed 30-degree tube into a weir basin. The basin overflows down a chute back to the reservoir. Speed and direction are live controls. Release tests prove 60 s of lifting at default and maximum speed with zero leaks and zero evictions while the inlet stays covered. They also prove that reverse returns water, speed 0 holds still, and gravity 2 and 80 stay sealed. The caption names the flat-vane stand-in honestly. The protected baseline grew to 27 ids in its own commit, and the gate reports 27/27 unchanged.

---

_Verified: 2026-10-14T22:53:46.287Z_
_Verifier: Claude (gsd-verifier)_
