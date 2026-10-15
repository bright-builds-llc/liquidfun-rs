---
phase: 48-fake-ferrofluid
verified: 2026-10-15T08:01:54.699Z
status: passed
score: 4/4 roadmap success criteria verified (20/20 merged must-haves)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: 2026-10-15T08:01:54.699Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 48: Fake Ferrofluid Verification Report

**Phase Goal:** Players can push a magnet around under a dark liquid puddle and watch it rise into spikes that follow the magnet and collapse when it lets go.
**Verified:** 2026-10-15T08:01:54.699Z
**Status:** passed
**Re-verification:** No (initial verification)
**Lifecycle:** `gsd-tools verify lifecycle 48 --require-plans --raw` returned `valid`. CONTEXT, all 5 PLANs and all 5 SUMMARYs share `lifecycle_mode: yolo` and `phase_lifecycle_id: 48-2026-10-15T04-20-00`.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | On load, a dark liquid puddle that is clearly visible on the dark canvas rises into several separate spikes (at least 4 surface peaks, not one blob) toward a magnet resting under the dish, and the magnet is drawn as a marker. | VERIFIED | `fake_ferrofluid.rs` builds one TENSILE \| VISCOUS group of 990 particles in `#8A84AD`, with `magnet_on: true` at `MAGNET_REST` (0, -0.35) under the floor. Release tests `load_forms_four_peaks_within_ninety_steps` (first 4 or more peaks at step 12, 0.2 s) and `default_strength_shows_five_to_nine_spikes` (5 peaks at step 180, with a 0.15 m prominence metric over 71 bins) pass. `collect_segments` appends `geometry::magnet_marker` (8 segments on, 4 off). Screenshots `target/phase48/color/shaded-blob-desktop.png` and `-portrait.png` show 7 separated crests over the bar marker, clearly visible on `#071018`. See the observations on "dark" and crest height below. |
| 2 | Pressing and dragging moves the magnet and the spikes follow it. Releasing, cancelling the pointer, or switching the magnet off makes the spikes collapse back into a puddle. | VERIFIED | In `apply_pointer`, Down turns the magnet on at `clamp_magnet(p)`, Move while held follows, and Up and Cancel share one arm that switches the magnet off and parks it at rest. Release tests pass: three `dragging_the_magnet_*_moves_the_crown_toward_it` tests (centroid shift 1.168 m with at least 4 peaks), `pointer_up_flattens_the_puddle`, `pointer_cancel_flattens_the_puddle`, `magnet_action_off_flattens_the_puddle` and `magnet_action_on_reforms_four_peaks_within_ninety_steps`. README frames 351 and 471 show the crown, then a flat puddle with the parked outline-only marker. |
| 3 | Player can adjust magnet strength only within a range that never flings the liquid apart, and a native 60 s test with the pointer held at maximum strength shows no runaway energy, no evictions and no failed step. | VERIFIED | `apply_control` accepts only `parse_whole_number(value, 100)` and returns `ControlEffect::Live`; the web spinner is 0..100. Five session holds `held_*_at_full_strength_stays_stable` run 3,600 steps with the pointer held. Each asserts no particle out of the dish or escaped, max speed at most 4 m/s, no late speed growth, second-half KE at most 1.25 x first half + 0.5, and live == built particles, and every `advance` is `expect`ed Ok. Two drag-path tests at full strength to the clamp extremes (`dragging_up_then_past_the_wall…`, `dragging_past_the_wall_then_up…`) keep every particle in the dish after the WR-02 `MAGNET_MAX_Y` 1.4 m clamp. |
| 4 | The magnet pull is applied once per fixed step and follows world gravity, so the gravity slider and phone tilt stay correct. The caption puts "fake" in the title or description and calls it a scripted magnetic pull. The scene meets every shared scene rule, and the id-join gate reports every protected fingerprint unchanged. | VERIFIED | The force is applied only in `on_advance`, which calls `GravityFrame::from_gravity(world.gravity())` every step; the other hooks only store intent (`seam_hooks_store_intent_without_touching_the_world`). `field.rs` uses only `frame.up`, `frame.across` and `frame.magnitude`, with no hard-coded -y. `tilted_gravity_lifts_along_the_tilted_up`, `quarter_turn_of_gravity_turns_the_pull` and the gravity 2 and 80 smokes pass. The title is "Fake Ferrofluid", and the description says "A scripted magnetic pull lifts the liquid into the spikes. The physics engine has no magnetism". Shared rules: see Requirements Coverage. The gate rerun at HEAD `af12ad96a` (fresh full capture `2026-10-15T08-01-10Z`) reports PASS, protected 28/28 unchanged, none changed, missing or added. |

**Score:** 4/4 roadmap truths verified. All 20 PLAN must-have truths, merged across plans 01-05, are covered by the evidence above and below.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/liquidfun-wasm/src/scene/particle_force.rs` | GravityFrame, ForceScale, apply_accelerations, apply_group_acceleration | VERIFIED | 182 lines. Calls `apply_particle_force_range`. Has 9 tests, including a calibration within 1e-3 at 2 and 3 iterations. `apply_group_acceleration` is `expect(dead_code)` outside tests by design (reserved for Phase 49). |
| `crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs` | build, hooks, SceneHooks impl | VERIFIED | 282 lines. Wired through `SceneId::FakeFerrofluid => fake_ferrofluid::build`. |
| `.../fake_ferrofluid/field.rs` | bounded polynomial pull, cap, drag | VERIFIED | `magnet_acceleration` is capped at `1.5·G·S`. `far_fade` (WR-01) is included. No transcendental functions. |
| `.../fake_ferrofluid/geometry.rs` | dish, MAGNET_REST, clamp, marker, frame | VERIFIED | `WALL_TOP_Y` 3.0, `MAGNET_MAX_Y = WALL_TOP_Y - reach` (1.4). Compile-time asserts on the frame and clamp. |
| `.../fake_ferrofluid/{spike,flow,control,sync}_tests.rs`, `test_support.rs`, `tests.rs` | Release D-05/D-23 guards, debug seam tests, R14 sync | VERIFIED | 83 release tests passed in 3.83 s. Every file is 579 lines or fewer (limit 628). |
| `web/src/catalog/fake-ferrofluid-strength.ts` | `magnet-strength` live HUD spinner | VERIFIED | range 0..100, default 65, `recreates: false`, `surface: "hud"`, `widget: "spinner"` |
| `web/src/catalog/scene-records-curiosity.ts` | record after the screw | VERIFIED | `withGravitySlider([MAGNET_STRENGTH_CONTROL, action("magnet","Magnet")])`, `sceneSource("fake_ferrofluid.rs")`, SHOWCASE inspiration |
| `web/src/catalog/previews-curiosity.tsx` + `previews.tsx` | static preview | VERIFIED | `FakeFerrofluidPreview` (7-spike crown, `FERRO`); `case "fake-ferrofluid"` |
| `web/src/catalog/scene-view-bounds.ts`, `portrait-bounds.ts` | landscape and portrait frames | VERIFIED | `FAKE_FERROFLUID_VIEW_BOUNDS` (-3.6, -0.8, 3.6, 3.6) is used for portrait. Its R14 sync test passes. |
| `docs/assets/readme/fake-ferrofluid-10s.{svg,webp}` + README gallery | README preview | VERIFIED | 2.6 MB SVG and 12.4 MB WebP. README lines 206-210 link the scene route. |
| `docs/benchmarks/scene-fingerprints.json` | 28 protected ids, fake-ferrofluid last | VERIFIED | 28 entries, last `fake-ferrofluid e73ef71f0615b4eb`. Commit `2db76fda3` changes only this file. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `scene.rs` build_scene | `fake_ferrofluid::build` | match arm | WIRED | `scene.rs:295` |
| `FakeFerrofluidHooks::on_advance` | `particle_force::apply_accelerations` | GravityFrame from `world.gravity()` each step, `magnet_acceleration - v·drag` | WIRED | Called each fixed step. Returns early when off or when gravity is zero. |
| `particle_force.rs` | `World::apply_particle_force_range` | `scale.force(a)` = a·m·iterations | WIRED | per particle and per group paths |
| pointer, action and control hooks | `on_advance` | `magnet_on`, `magnet_position`, `pointer_held`, `strength_percent` stored as intent | WIRED | No world access in the seam hooks |
| `collect_segments` | `geometry::magnet_marker` | `GravityFrame::from_gravity(world.gravity())` | WIRED | dish first, then marker |
| web `SCENE_IDS` | Rust `ALL_SCENE_IDS` / `SURVEY_SCENES` | include_str order test | WIRED | Last in all three. `scene_spot.rs:115` uses `SurveyCue::Default`. `gravity_slider` index 27. |
| `scene-records-curiosity.ts` | `fake-ferrofluid-strength.ts` | `MAGNET_STRENGTH_CONTROL` import | WIRED | |
| `README.md` | `fake-ferrofluid-10s.webp` | gallery upsert | WIRED | |
| `readme-svg/plans.ts` | `magnet` action | cues at samples 120 and 160 | WIRED | `readme_clip_shows_spikes_collapse_and_rerise` passes |
| e2e `player.spec.ts` | scene | `POINTER_CONTROL { gesture: "drag", control: "Magnet" }` | WIRED | `player-helpers.ts:58` hash path |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| Scene particles (render) | particle positions | engine particle system driven by `on_advance` forces | Yes. 990 live particles; peaks measured from live positions in session tests | FLOWING |
| Magnet marker | `magnet_position`, `magnet_on` | pointer and action seams | Yes. Segment count changes 8 to 4 on off, as seen in README frames 351 and 471 | FLOWING |
| Strength spinner | `strength_percent` | `apply_control` Live | Yes. `the_session_applies_strength_live_and_rebuilds_for_gravity`; strength 0 gives a flat puddle | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Scene outcomes (peaks, drag, collapse, holds, gravity smokes, README timing) | `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid` | 83 passed, 0 failed, 0 ignored (3.83 s) | PASS |
| Force helper calibration and tilt | `cargo test -p liquidfun-wasm --release --lib particle_force` | 9 passed | PASS |
| PREP-01 id-join gate on HEAD | `just scene-fingerprint-gate` (fresh full capture `target/dam-break-perf/2026-10-15T08-01-10Z`, git `af12ad96a`) | PASS, protected 28/28 unchanged, changed/missing/added none | PASS |
| Particle budget and cost (SCENE-02) | same capture, fake-ferrofluid row | start 990, end 990, median 0.316 ms/step | PASS |
| File-length and lesson standards | `bun scripts/bright-builds-check.ts all` | findings=0 | PASS |
| Full workspace regression (prior run at HEAD) | `target/phase48/regression-gate.log` | exit=0 | PASS (log reviewed, not re-run) |
| HUD safe frame and player e2e | `target/phase48/checks-04/14-e2e.log` | 102 passed, including 4 fake-ferrofluid HUD cases (390x844, 1280x800, 844x390, 768x1024) | PASS (run at f798bff91; afterwards only a comment changed in web, plus the clamp and fade changes in Rust) |
| No engine or dependency change | `git diff c5462cb40~1 HEAD -- crates/liquidfun Cargo.lock web/package.json` | empty | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| FERRO-01 | 48-01..05 | Dark puddle visible on the dark canvas rises into several separate spikes toward a magnet resting under the dish | SATISFIED | SC1 evidence. 5 to 7 crests by metric and screenshot; magnet at rest under the floor |
| FERRO-02 | 48-02, 48-03, 48-04 | Press and drag moves the magnet; spikes follow and collapse on release or off | SATISFIED | SC2 evidence. Drag centroid tests; Up, Cancel and action collapse tests; e2e drag plus Magnet button |
| FERRO-03 | 48-01..04 | Strength range never flings liquid apart; stable while the pointer is held | SATISFIED | SC3 evidence. Clamped 0..100; five 60 s held holds; full-strength drag paths to the clamp extremes |
| SCENE-01 (shared) | 48-03, 48-04 | Catalog, credits, preview, portrait frame, README preview, scale legend, gravity slider, HUD safe frame | SATISFIED | Record with "original experimental scene" copy and SHOWCASE credit; preview; `PORTRAIT_VIEW_BOUNDS` entry and portrait test (0.25 min height, width-bound 0.918); README SVG/WebP reviewed (frames 61/351/471/591); `withGravitySlider`; legend visible in screenshots ("50 cm", "2 m"); hud-safe-frame e2e passes for the scene; id in `ALL_SCENE_IDS` and `SURVEY_SCENES` |
| SCENE-02 (shared) | 48-04 | Moves on its own within about 1 s; at most about 3,000 particles; count and ms/step recorded | SATISFIED | First spikes at 0.2 s with no input; 990 particles; 0.314 ms/step (48-04) and 0.316 ms/step (this run) |
| SCENE-03 (shared) | 48-03 | Honest caption naming the faked mechanism | SATISFIED | Description says "A scripted magnetic pull…" and "The physics engine has no magnetism, and the magnet is only a drawn marker"; `FERRO_FORBIDDEN_CLAIMS` guard in `web/tests/scenes.test.ts` |
| PREP-01 (shared gate) | 48-04, 48-05 | Protected fingerprints unchanged; baseline grows only in its own commit | SATISFIED | 27/27 unchanged plus fake-ferrofluid added (48-04 log); single-file commit `2db76fda3`; 28/28 unchanged at HEAD (this run) |

No orphaned requirements: REQUIREMENTS.md maps only FERRO-01..03 to Phase 48, and every plan claims them. FERRO-04 (hysteresis) is a future requirement and out of scope.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | No TODO, FIXME, `unwrap()`, `todo!`, `println!` or `eprintln!` in the phase's scene, helper or web catalog files | — | — |
| `fake_ferrofluid.rs` | 102 | `set_gravity(Vec2::new(0.0, -10.0))` | Info | Initial world gravity only, the same pattern as every scene. The field reads `world.gravity()` each step, so no field term hard-codes -y. |
| `particle_force.rs` | `apply_group_acceleration` | `expect(dead_code)` outside tests | Info | Reserved for Phase 49 by design (D-12) and tested |

### Non-blocking Observations (judgement notes)

1. **"Dark" liquid reads mid-tone.** `#8A84AD` reads as a muted lavender-gray on `#071018`, not as a near-black ferrofluid. SC1 couples "dark" with "clearly visible on the dark canvas", and D-16 settles that tension: "dark blue-gray or graphite violet, not black… must stay readable… after the 0.73 density-shade floor". UI-SPEC chose `#8A84AD` as the darkest candidate that keeps 3:1 contrast at that floor, and circle wireframe is already the faintest mode. Visibility, the measurable half of the criterion, clearly passes. The color is clearly distinct from the shared water blue `#4DA3FF`. I judge this to meet the criterion as specified. If the owner wants a darker look, it is a tuning follow-up, not a gap.
1. **Spikes are low, rounded crests in a tall frame.** The criterion asks for at least 4 separate surface peaks, not one blob. The metric (0.15 m prominence) counts 5, and the screenshots show 7 clearly separated crests. Because the 48-02 stability work raised the walls to 3.0 m, the crests (about 0.28 m above a 0.43 m puddle) fill only the bottom quarter of the dish. On a 390x844 phone each crest is about 15 px tall: countable, but small. This is an aesthetic follow-up, such as lower walls now that WR-02 caps the magnet at 1.4 m, or a higher default strength within 5 to 9 peaks. It is not a criterion failure.
1. **WR-01 and WR-02 were tagged "requires human verification"** in 48-REVIEW-FIX because they change field logic and drag reach. Both are covered by deterministic tests: `pull_vanishes_beyond_twice_the_reach`, the two full-strength drag-path containment tests, `pointer_down_above_the_frame_clamps_one_reach_below_the_wall_tops`, and the unchanged default fingerprint. The remaining question is how the 1.4 m drag ceiling feels in play. No success criterion depends on that, so it is not a human-verification blocker.
1. **GPU Water shows small white specks** inside the puddle (48-04). Readability is unaffected.

### Human Verification Required

None. Every success criterion can be decided from code, deterministic release tests, the fingerprint gate and the committed screenshots or README frames.

### Gaps Summary

There are no gaps. The scene exists and is substantive. The pull runs once per fixed step in the gravity frame, through the shared `particle_force` helper. The pointer, action and live strength seams store intent only. The marker draws on and off states, and the catalog, preview, portrait frame, README clip, survey, demo-media and e2e sites are all wired. Release tests prove peaks on load, drag-follow, collapse on Up, Cancel and the action, a flat puddle at strength 0, stable 60 s held runs at full strength, and stability at gravity 2 and 80. The baseline grew in its own single-file commit, and a fresh full capture at HEAD reports 28/28 protected fingerprints unchanged. The observations above are about look and feel, for the owner's discretion.

---

_Verified: 2026-10-15T08:01:54.699Z_
_Verifier: Claude (gsd-verifier)_
