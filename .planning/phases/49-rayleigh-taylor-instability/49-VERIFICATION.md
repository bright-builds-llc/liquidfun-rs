---
phase: 49-rayleigh-taylor-instability
verified: 2026-10-19T03:32:18Z
status: passed
score: 28/28 must-haves verified (4 roadmap success criteria + 24 plan truths)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: 2026-10-19T03:32:18Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 49: Rayleigh–Taylor Instability Verification Report

**Phase Goal:** Players can watch a heavier liquid layer break into fingers that sink through a lighter one, and replay it with a different weight ratio.
**Verified:** 2026-10-19T03:32:18Z (first written 2026-10-19T03:27:28Z as human_needed; updated after the orchestrator supplied browser evidence)
**Status:** passed
**Re-verification:** No. This is the initial verification.
**Verifier:** Claude (gsd-verifier), an AI verifier. This is not human approval.

## Goal Achievement

### Roadmap Success Criteria (the contract)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC1 | On load, a high-contrast heavy layer over a light one develops fingers (at least 3 past 20% of the tank height by about 4 s) and overturns within about 10 s | VERIFIED | I reran `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor`: 80 passed, 0 ignored, including `flow_tests::default_ratio_grows_three_fingers_by_four_seconds` (summary: step 114, limit 240), `default_ratio_overturns_by_ten_seconds` (step 156, limit 600), `spike_tests::default_ratio_fingers_are_not_rain`, `settle_frame_shows_fewer_than_three_fingers` and `ratio_one_point_two_overturns_by_step_1200`. The depth line is the settled interface minus 0.2 x the 1.6 m inner height (D-06a), so a slumping column cannot satisfy it. I viewed README frames 31, 115 and 571: ripple at 0.5 s, about 5 separate orange fingers at 1.9 s, orange packed on the floor at 9.5 s. |
| SC2 | The player can choose a weight ratio in about 1.2x to 3x and restart, and the same settings always give the same pattern | VERIFIED | `WeightRatio::parse` accepts exactly `1.2/1.5/2/3` (`rayleigh_taylor.rs:57-65`). `apply_control` returns `Recreated` for a listed token (`:306`), and `SessionCore::apply_control` stores the preset and rebuilds (`session.rs:336-339`). The web side has `WEIGHT_RATIO_CONTROL` (`kind: "preset"`, `recreates: true`, 4 options) and `DEFAULT_PRESET_VALUES["weight-ratio"] = "2"`. `each_ratio_replays_to_the_same_end_state` (bit-identical after 600 steps at every ratio) and `ratio_three_differs_from_the_default_ratio` both pass. A fresh scene-spot process I ran gave fingerprint `ac59254e2231967c`, identical to the 49-04 capture and the baseline. |
| SC3 | One particle system with two groups; extra weight on the heavy group along world gravity; no engine change; the caption says "extra weight" and claims no simulated density | VERIFIED | `build_parts` creates one system and two `create_layer` groups. `on_advance` reads `GravityFrame::from_gravity(world.gravity())` every step and calls `particle_force::apply_group_acceleration(.., frame.up * -extra)` on the heavy group only. `git diff fa11f9b00^ HEAD -- crates/liquidfun Cargo.toml Cargo.lock crates/*/Cargo.toml web/package.json` is empty. No COLOR_MIXING, TENSILE or SOLID flag appears. The caption reads "Both layers are the same particle liquid, and the orange one is given extra weight". A grep for density/denser/atwood/simulated/growth rate in the record finds nothing. |
| SC4 | The scene meets every shared rule, and the id-join gate reports every protected fingerprint unchanged | VERIFIED | SCENE-01..03 are mapped below. I reran `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json`: PASS, 29/29 unchanged, added none. That capture was taken at e495c30f5, and `git diff --quiet e495c30f5 HEAD -- crates tools web` holds, so it still represents HEAD. The first 28 baseline entries are byte-identical to those before 9e7e61cfc, and the 28/28 gate log (`target/phase49/fingerprint-gate.log`) is consistent with that. |

### Plan Must-Have Truths

| Plan | Truth (abridged) | Status | Evidence |
|------|------------------|--------|----------|
| 01 | Closed tank and lid; one system with two groups (light `#3B82F6` below, heavy `#FB923C` above a polynomial-seeded interface); at most 3,000 particles; no COLOR_MIXING, TENSILE or SOLID | VERIFIED | `rayleigh_taylor.rs:32-36, 186-261`. Tests `build_creates_exactly_two_groups_in_one_system`, `each_layer_carries_its_own_color`, `every_particle_carries_the_tuned_flags_only`, `heavy_group_starts_on_top` and `lattice_is_bit_identical_across_builds` pass. 1,484 particles. |
| 01 | Ramped extra weight along -up from live gravity; none during the settle; no frame means no force; no hard-coded -y | VERIFIED | `on_advance` (`:273-296`); `ramp_factor` (`:265-270`). Tests `ramp_*` and `zero_gravity_applies_no_extra_weight`. The only fixed -10 is the initial `set_gravity` at build. |
| 01 | `weight-ratio` exact tokens with default 2; `Recreated` for valid tokens; everything else fails closed; pointer returns Ok | VERIFIED | `ratio_from_presets`, `apply_control`, `apply_action`, `apply_pointer`. Tests `build_rejects_unknown_presets_and_tokens`, `hooks_reject_*`, `hooks_ignore_every_pointer_kind`, `session_rejects_an_unlisted_ratio`. |
| 01 | `expect(dead_code)` removed in the same commit as the first caller, with no other particle_force line changed | VERIFIED | fa11f9b00 numstat for particle_force.rs is `0 7`, and the same commit adds rayleigh_taylor.rs. The phase diff of particle_force.rs is only that attribute. |
| 01 | Time-boxed spike against rules fixed before measuring; first passing candidate locked as TUNING; sweep table recorded | VERIFIED | 49-01-SUMMARY has sweep 1 (invalidated) and sweep 2 (23 configs). The D-06a amendment commit 5756a5c29 precedes c22ee16dd. The 10 spike guards pass in release, including S0, S6 (ripple deepens at least 0.06 m by step 60) and S5. |
| 02 | Debug twin: no extra weight in the settle; force lane adds the expected force to heavy rows only under tilted gravity | VERIFIED | `force_lane_tests.rs`: 6 tests pass (debug-safe, not ignored). |
| 02 | Release: at least 3 fingers by step 240 and overturn by step 600 at ratio 2 and g 10 | VERIFIED | flow_tests pass, as in SC1. |
| 02 | Bit-identical replay at every ratio; ratio 3 differs from ratio 2 | VERIFIED | As in SC2. |
| 02 | Sideways gravity moves the heavy centroid at least 0.2 m in 120 steps | VERIFIED | `sideways_gravity_moves_the_heavy_layer_toward_it` passes (summary: 0.738 m). |
| 02 | 60 s at ratio 3 and smokes at g2, g80 and g80 x ratio 3: live == built, no failed step, every particle in the tank | VERIFIED | The 4 flow smokes pass. A fresh scene-spot run gives end 1,484 = start 1,484. |
| 03 | Sidebar lists the scene last after Fake Ferrofluid, with the locked copy, credits and static preview | VERIFIED | `scene-records-curiosity.ts:76-91`; `RayleighTaylorPreview` (`previews-curiosity.tsx:61`); `previews.tsx:103`. |
| 03 | Weight ratio select and gravity slider; the pick rebuilds through Recreated; Reset returns to the defaults; no action, pointer or HUD control | VERIFIED | `withGravitySlider([WEIGHT_RATIO_CONTROL])`; `web/tests/rayleigh-taylor.test.ts` checks panel `["weight-ratio","gravity"]` and HUD `[]`; no POINTER_CONTROL entry. Reset uses the shared `recreateScene`. Agent browser check (see Agent-Performed UAT): selecting 3 rebuilt the scene, and Reset returned to 2. |
| 03 | Portrait frame fits the whole tank, walls and lid included | VERIFIED | `PORTRAIT_VIEW_BOUNDS["rayleigh-taylor"]`. The portrait test checks 8 wall points, with MIN_HEIGHT_FRACTION 0.29. The 390x844 screenshot shows the full tank across the width. |
| 03 | Rust ALL_SCENE_IDS, SURVEY_SCENES and web SCENE_IDS list rayleigh-taylor last; survey cue Default; README cues []; demo-media idle click; e2e hash path | VERIFIED | `scene.rs:115`, `scene_spot.rs:120-121`, `scenes.ts:44`, `plans.ts:98`, `model.ts:314`, `player-helpers.ts:59`. |
| 03 | Honest caption with no forbidden claims | VERIFIED | As in SC3, plus the web honesty guard test. |
| 03 | Web bounds, tokens and default match Rust | VERIFIED | `sync_tests.rs`: 3 tests pass in release. |
| 04 | Layers distinct from each other and from the canvas in four render modes at both viewports | VERIFIED | 31 screenshots in `target/phase49/color/`. I viewed the Shaded blob portrait at 2 s and GPU Water desktop at 2 s: orange and blue are clearly distinct against #071018. |
| 04 | README 10 s clip: ripple, deepening, at least 3 fingers, orange mostly below by 9.5 s | VERIFIED | Assets committed in e495c30f5, along with the README gallery entry (`README.md:212-216`). I viewed frames 31, 115 and 571. |
| 04 | Gate 28/28 with rayleigh-taylor added | VERIFIED | `target/phase49/fingerprint-gate.log`, consistent with the byte-identical first 28 baseline entries. |
| 04 | scene-spot numbers recorded: start at most 3,000, end = start, at most about 1.5 ms/step | VERIFIED | Recorded 1,484/1,484 at 0.363 ms. My independent run (stamp 2026-10-19T03-26-25Z) gives 1,484/1,484 at 0.350 ms. |
| 04 | Every normal local check passes | VERIFIED | `target/phase49/checks-04.log`: ALL CHECKS PASSED, exit 0, at e495c30f5. Code at HEAD is unchanged since then. The e2e log includes both rayleigh-taylor HUD safe-frame cases. |
| 05 | Baseline protects 29 ids, rayleigh-taylor last, provenance updated | VERIFIED | `.scenes \| length` is 29, and the last entry is `rayleigh-taylor ac59254e2231967c`. |
| 05 | Baseline growth is its own single-file commit | VERIFIED | `git show --name-only 9e7e61cfc` lists only `docs/benchmarks/scene-fingerprints.json`. |
| 05 | Gate rerun 29/29, no additions | VERIFIED | Reran by this verifier, as in SC4. |

**Score:** 28/28 truths verified.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/liquidfun-wasm/src/scene/rayleigh_taylor.rs` | WeightRatio, Tuning, build, hooks | VERIFIED | 355 lines; wired from `scene.rs:300` |
| `.../rayleigh_taylor/geometry.rs` | tank, seed, lattice, frame | VERIFIED | 151 lines |
| `.../rayleigh_taylor/test_support.rs` | finger count, overturn and other metrics | VERIFIED | 453 lines |
| `.../rayleigh_taylor/spike_tests.rs` | release-only D-06 guards | VERIFIED | 10 tests pass in release |
| `.../rayleigh_taylor/force_lane_tests.rs` (plan: `hooks_tests.rs`) | settle twin and force lane | VERIFIED | Renamed, as documented in 49-02-SUMMARY |
| `.../rayleigh_taylor/flow_tests.rs` | release session outcomes | VERIFIED | 9 tests pass |
| `.../rayleigh_taylor/sync_tests.rs` | TS/Rust sync | VERIFIED | 3 tests pass |
| `web/src/catalog/rayleigh-taylor-ratio.ts` | WEIGHT_RATIO_CONTROL | VERIFIED | Imported by the curiosity record |
| `web/src/catalog/scene-records-curiosity.ts` | record | VERIFIED | |
| `web/src/catalog/previews-curiosity.tsx` | RayleighTaylorPreview | VERIFIED | Wired in the `previews.tsx` switch |
| `web/src/catalog/scene-view-bounds.ts` | RAYLEIGH_TAYLOR_VIEW_BOUNDS | VERIFIED | Used by the record and the portrait map |
| `web/tests/rayleigh-taylor.test.ts` | honesty and control shape | VERIFIED | Unit suite passed in checks-04 |
| `docs/assets/readme/rayleigh-taylor-10s.{svg,webp}` | README preview | VERIFIED | Tracked; the WebP (33.8 MB) is within the repo range (the largest is 47.4 MB) |
| `README.md` gallery entry | link to the WebP | VERIFIED | Lines 212-216 |
| `docs/benchmarks/scene-fingerprints.json` | 29 ids | VERIFIED | |

### Key Link Verification

| From | To | Via | Status |
|------|----|-----|--------|
| `scene.rs` build_scene | `rayleigh_taylor::build` | `SceneId::RayleighTaylor => rayleigh_taylor::build(&scene_presets)` | WIRED |
| `on_advance` | `particle_force::apply_group_acceleration` | live `GravityFrame` each step, stored heavy group id | WIRED |
| `apply_control` | `SessionCore::apply_control` | `ControlEffect::Recreated` stores the preset and rebuilds | WIRED |
| flow_tests | `apply_control("weight-ratio", ..)` | `session_with` | WIRED |
| force_lane_tests | `on_advance` | `forces()` snapshot diff | WIRED |
| `scenes.ts` SCENE_IDS | `ALL_SCENE_IDS` | include_str! order test | WIRED |
| curiosity record | `WEIGHT_RATIO_CONTROL` | `withGravitySlider([...])` | WIRED |
| `scene-controls.ts` default | `WeightRatio::DEFAULT` | `"weight-ratio": "2"` and the sync test | WIRED |
| `README.md` | `rayleigh-taylor-10s.webp` | gallery entry | WIRED |
| baseline | capture 2026-10-19T03-11-46Z | `scene_spot_stamp` provenance | WIRED |
| `gravity_slider.rs` | variant index | `SceneId::RayleighTaylor => 28` | WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Real data | Status |
|----------|------|--------|-----------|--------|
| Scene canvas | particle positions and colors | live `World` stepped by the session; heavy group pushed by `apply_group_acceleration` | Yes: the fingerprint is stable and the screenshots show live motion | FLOWING |
| Weight ratio select | preset value | web preset → Rust `apply_control` → `store_preset` → `build(presets)` → `ratio_from_presets` | Yes: session tests show ratio 3 diverges from 2 | FLOWING |

### Behavioral Spot-Checks (run by this verifier)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All scene tests in release | `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor` | 80 passed, 0 failed, 0 ignored (2.0 s) | PASS |
| Protected fingerprints | `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json` | PASS 29/29, changed none, missing none | PASS |
| Capture still represents HEAD | `git diff --quiet e495c30f5 HEAD -- crates tools web` | exit 0 | PASS |
| Budget, eviction, speed, determinism across processes | `cargo xtask playground scene-spot --scene rayleigh-taylor` | 1,484 → 1,484, 0.3497 ms/step, fingerprint `ac59254e2231967c` (= baseline) | PASS |
| Baseline growth touches nothing else | `diff` of the first 28 entries before and after 9e7e61cfc; `git show --name-only 9e7e61cfc` | identical; one file | PASS |
| No engine or dependency change | `git diff --stat fa11f9b00^ HEAD -- crates/liquidfun Cargo.toml Cargo.lock crates/*/Cargo.toml web/package.json` | empty | PASS |
| Lifecycle provenance | `gsd-tools verify lifecycle 49 --require-plans --raw` | `valid` | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| RT-01 | 01, 02, 04 | Heavy layer develops sinking fingers and overturns within about 10 s | SATISFIED | SC1 |
| RT-02 | 01, 02, 03, 05 | Choose the weight ratio and restart; same settings give the same pattern | SATISFIED | SC2, plus the agent browser check of the select and Reset |
| SCENE-01 | 03, 04 | Catalog with honest credits, preview, portrait frame, README preview, scale legend, gravity slider, HUD safe frame | SATISFIED | Record and credits (`sceneSource`, SHOWCASE); preview; portrait map and test; README assets and plan; `withGravitySlider`; the scale legend is visible in the screenshots ("1 m", "20 cm"); hud-safe-frame e2e at 390x844 and 1280x800 passed; ALL_SCENE_IDS and SURVEY_SCENES |
| SCENE-02 | 01, 02, 04 | Moves on its own on defaults; at most about 3,000 particles; light | SATISFIED | S6 ripple guard and frame-61 trough evidence; 1,484 particles; 0.35 ms/step native release |
| SCENE-03 | 03, 04 | Caption names the fake plainly | SATISFIED | "the orange one is given extra weight"; honesty guard test; README alt-text grep |
| PREP-01 | 04, 05 (gate) | Protected fingerprints unchanged by id join | SATISFIED | 28/28 before growth, then 29/29 after growth (rerun by this verifier) |

No orphaned requirements. REQUIREMENTS.md maps only RT-01 and RT-02 to Phase 49, and both appear in plan frontmatter. RT-03 is a future requirement and correctly out of scope. RT-01 and RT-02 are still marked Pending in REQUIREMENTS.md by design; the orchestrator ticks them after verification.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (scene and web files) | - | TODO/FIXME/placeholder/println/unimplemented scan | none found | - |
| `rayleigh_taylor/test_support.rs` | 254-267 | `ripple_depth` returns a `NEG_INFINITY` sentinel that could make S6 pass vacuously if the lattice changes (49-REVIEW IN-01) | Info | Test-only robustness; not live today |
| `rayleigh_taylor/test_support.rs` | 27-28, 47-50, 104 | Stale bin-width comment and literal bin bounds (IN-02) | Info | Test-only |
| `rayleigh_taylor/spike_tests.rs` | 21-26 | S4 doc says sampling starts at step 30, but the spike samples only after the settle (IN-03); flow_tests samples step 30 | Info | Test-only |
| `rayleigh_taylor/tests.rs` | 165-177 | Wall-thickness test is tautological; the real guard is the const assert at `rayleigh_taylor.rs:126` (IN-04) | Info | Test-only |
| spike_tests / flow_tests | - | Duplicate 60 s ratio-3 runs (IN-05) | Info | About 2 s of release test time |
| `rayleigh_taylor/hook_tests.rs` | 16 | Duplicated 3,000 cap (IN-06) | Info | Test-only |

No blockers or warnings. Production code has no `unwrap()`. Every file is at most 455 lines, under the 628-line limit.

### Notable Deviations (documented, accepted)

- **D-06a amendment.** The tank changed from about square (2.8 x 3.2 m) to wide (2.8 x 1.6 m) after the first sweep passed only because the column collapsed during the settle. The amendment was recorded in CONTEXT.md before the rerun, and no deadline was loosened. Because the tank is now wide, D-13's "a tall tank suits the phone frame" premise no longer holds. The portrait frame fills the phone width with vertical margin (MIN_HEIGHT_FRACTION 0.29), which the AGENTS.md framing rule allows ("a wide scene may still have some vertical margin").
- **Timeline.** Fingers appear at about 1.9 s and the overturn at about 2.6 s, well ahead of the 4 s and 10 s targets. After about 4 s the clip shows a long settled tail. That satisfies the success criteria; whether the pacing feels right is an observation for the owner's discretion (see Non-blocking Observations).
- **Renamed file.** The plan's `hooks_tests.rs` was renamed `force_lane_tests.rs` to avoid confusion with `hook_tests.rs`. The tests are the same, and no new test binary was added.

### Agent-Performed UAT (AGENTS.md "Agent-Performed Simple UAT")

These checkpoints were objective, so agents closed them (`verified_by: agent`). None of this is human approval.

| Checkpoint | Result | Verified by | Evidence |
|------------|--------|-------------|----------|
| Weight ratio select and Reset scene in a browser | pass | agent (orchestrator, Claude; reported to this verifier, who did not repeat the run) | On 2026-10-19 the orchestrator opened `vite preview` of `web/dist` at route `/liquidfun-rs/#/scene/rayleigh-taylor`, 1280x800. The controls sheet shows a `Weight ratio` select with options `1.2:1.2x, 1.5:1.5x, 2:2x, 3:3x` and value `2`. Selecting `3` (input and change events) took `data-step-index` from 64 to 1 within 250 ms, and the select stayed at `3`: the scene rebuilt with the new ratio. Clicking `Reset scene` took `data-step-index` from 46 to 1 and set the select back to `2`, with no `[role=alert]` element. The console showed no errors or logs. This verifier confirmed that `git diff --quiet e495c30f5 HEAD -- web` holds, and `web/dist/index.html` is dated 2026-10-19 03:09Z (the checks-04 web build), so the build under test matches HEAD. |
| Fingers versus rain or blob (SC1) | pass | agent (this verifier, plus the orchestrator) | The criterion is decided by the release finger metric with the rain veto: first 3 or more fingers at step 114, rain share 0.000 at step 240, overturn at step 156. This verifier and the orchestrator both viewed README frames 115 (several separate orange fingers, with blue rising between them) and 571 (orange settled under blue). Plan 04 screenshots cover four render modes at 0.5, 2, 4 and 10 s, on desktop and portrait. |
| Tilt keeps "heavy" pointing down | pass | agent (this verifier) | `force_lane_tests::extra_weight_follows_tilted_gravity` (heavy rows only, along -up of a 45-degree gravity), `flow_tests::sideways_gravity_moves_the_heavy_layer_toward_it` (0.738 m in 120 steps), and `on_advance` reads `world.gravity()` every step. |

### Non-blocking Observations (owner's discretion)

1. **Pacing.** The fingers arrive at about 1.9 s and the overturn at about 2.6 s, well ahead of the 4 s and 10 s targets. From about 4 s on, the 10 s README clip is a mostly settled, mixed tail. This meets the criterion. If the owner wants the overturn to fill more of the clip, slowing it is a tuning follow-up, such as a longer settle or a lower default ratio, and it would need to rerun the release guards. It is not a gap.
1. **How the fingers look.** SPH smooths the fingers into blobby shapes, and the blue plume at about 2 s breaks through the top surface with a few loose droplets. The caption stays qualitative, as D-15 requires. Whether this looks good enough is an aesthetic call.
1. **Phone frame.** The amended wide tank (D-06a) fills the phone width and leaves vertical margin, at a fitted height share of 0.297. AGENTS.md allows this for wide scenes, and the portrait test passes.
1. **Physical-phone playback.** No v1.6 phase requires a run on a physical phone. SCENE-02's evidence rule is the recorded particle count and native ms/step (1,484 particles, 0.35 to 0.36 ms/step, against a budget of about 3,000 and about 1.5 ms). The 390x844 viewport is covered by the hud-safe-frame e2e and the screenshots. A real-device check is optional for the owner and does not block any criterion.
1. **Review Info items IN-01 to IN-06** are robustness and duplication notes in test-only code.

### Human Verification Required

None. Every success criterion can be decided from code, deterministic release tests, the fingerprint gate, the committed README frames, and the agent-performed browser check above.

### Gaps Summary

No gaps. All four roadmap success criteria and all 24 plan must-have truths are verified against the code, with release tests, the fingerprint gate and an independent scene-spot run executed by this verifier. The orchestrator checked the browser path for the Weight ratio select and Reset. Every requirement ID (RT-01, RT-02, SCENE-01..03, PREP-01) is accounted for. The observations above are about look and feel, for the owner's discretion.

---

_Verified: 2026-10-19T03:32:18Z_
_Verifier: Claude (gsd-verifier), AI verifier_
