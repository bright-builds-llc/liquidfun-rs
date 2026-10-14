---
phase: 46-cartesian-diver
verified: 2026-10-14T19:46:29Z
status: passed
score: 5/5 roadmap success criteria verified (23/23 merged must-haves)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: 2026-10-14T19:46:29Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 46: Cartesian Diver Verification Report

**Phase Goal:** Players can squeeze a bottle and watch a diver sink, then let go and watch it rise, in the first new scene to go through the shared per-scene checklist end to end.
**Verified:** 2026-10-14T19:46:29Z on HEAD 7230d5d47 (only `.planning/config.json` dirty)
**Status:** passed
**Re-verification:** No. This is the initial verification.

Provenance: 46-CONTEXT.md, all five PLANs and all five SUMMARYs carry `lifecycle_mode: yolo` and `phase_lifecycle_id: 46-2026-10-14T17-38-05`. None is marked `direct-fallback`.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Opening the scene shows a diver floating near the top of a closed bottle. Press-and-hold on the bottle, or the Squeeze toggle, sinks it to the bottom third within about 3 s. Releasing it, including a cancelled pointer, makes it rise again. | VERIFIED | `cartesian_diver.rs`: four wall quads include a closed cap, `DIVER_START` is (0, 4.6), and sleep is disabled on the diver. The hooks follow D-01/D-02: Down inside `in_squeeze_region` squeezes, and Up or Cancel release the squeeze only after a press inside the bottle (the WR-01 fix). `apply_action("squeeze")` toggles. Release tests that I re-ran passed: `diver_floats_in_the_top_third_at_rest`, `squeezed_diver_sinks_at_gravity_ten` (window 180 steps = 3 s; 46-01 measured 68 steps), `released_diver_rises_at_gravity_ten`, and `pointer_hold_sinks_and_cancel_releases`. The web path forwards `pointercancel` and `lostpointercapture` as `cancel` (`web/src/input/canvas-pointer.ts:21-22,180`). README frame 11 shows the diver at the top at rest. |
| 2 | While squeezed, the bottle visibly dents and the air bubble inside the diver shrinks. A phone long press on the bottle opens no system callout. | VERIFIED | The 14-segment outline bows inward by up to 0.25 m (`side_point`, a TAU half-sine), and the bubble shrinks from 0.6R to 0.3R (`bubble_circle`). Both are covered by unit tests. I viewed `target/phase46/readme-frame-201-small.png`, which shows the bowed walls and the small bubble, and frame 11, which shows straight walls and the large bubble. The callout suppression lives in `player-viewport.css:47-53` (`-webkit-touch-callout: none`, `-webkit-user-select: none`, `user-select: none`, plus `touch-action: none` from the shared `canvas` rule). `PlayerPanel.tsx:112,206` calls `preventDefault` on `contextmenu`. `canvas-callout.spec.ts` (390x844, touch) checks the computed styles, that a dispatched contextmenu is prevented, and the built CSS text, and `canvas-callout.test.ts` checks the CSS source. A real-iPhone long press is an optional manual check (D-14). |
| 3 | Native tests show the diver sinking when squeezed and rising when released at gravity 2, 10 and 80 m/s². | VERIFIED | I re-ran `cargo test -p liquidfun-wasm --release cartesian_diver`: 45 passed, 0 failed, 0 ignored. This covers sink and rise at g2 and g10, sink at g80, `released_diver_rises_in_the_water_column_at_gravity_eighty` (D-07a target; it also asserts that the diver starts below the target, so the check cannot pass without movement), the long-hold floor-stick guard, the README schedule and g80 containment. D-07a is a recorded amendment that changes only the g80 rise target. The roadmap criterion holds as written. |
| 4 | The scene meets every shared rule: catalog record with credits, preview, phone-filling portrait frame, README animated preview driven by the Squeeze toggle, scale legend, gravity slider, HUD safe frame, at most about 3,000 particles, and a caption saying that switching the diver's density stands in for the squeezed air bubble. | VERIFIED | `scene-records-curiosity.ts`: the credits use `sceneSource("cartesian_diver.rs")` and SHOWCASE, the description includes "original experimental scene", the controls are `withGravitySlider([action("squeeze","Squeeze")])`, and the caption says "Switching the diver to a heavier density stands in for its squeezed air bubble. The bottle dent and the bubble are drawn, not simulated." The preview is `CartesianDiverPreview`, reached through the `previews.tsx` switch. The portrait frame is `portrait-bounds.ts:97`, and `portrait-bounds.test.ts` checks a fill of at least 0.75 and the six wall points. The README plan uses `squeeze` at samples 20 and 110 (`plans.ts:83-89`), and the SVG and WebP are tracked and linked at `README.md:194-198`. `hud-safe-frame.spec.ts` iterates `SCENE_IDS`, and the scale legend is shared player chrome. The scene has 1,936 particles. |
| 5 | The id-join gate reports every protected fingerprint unchanged with the new diver id present, and the phase records the diver's particle count and ms/step. | VERIFIED | `target/phase46/fingerprint-gate-final-after-review-fix.log` (observed at git 7230d5d47, full_catalog true) reports PASS with 26/26 protected unchanged, including cartesian-diver. Before the baseline grew, 46-04 reported 25/25 with `cartesian-diver fa849f40b8c2a791` added. The baseline grew in commit 18c8a5e92, which touches only `docs/benchmarks/scene-fingerprints.json` (26 entries, diver last). 46-04-SUMMARY records start 1936, end 1936 and a median of 0.582 ms/step in native release. |

**Score:** 5/5 roadmap truths verified. All 18 PLAN must-have truths are also verified, and they restate or refine these criteria.

### PLAN must-have notes

- 46-01 truth 4 still says "top third at gravity 80". D-07a superseded that wording in CONTEXT before 46-01 was implemented. The roadmap criterion 3 is satisfied.
- The 46-04 gate truth (25/25 plus an added id) was correct when 46-04 ran. After 46-05 the gate on the final tree reports 26/26.
- 46-02 deviation: the demo-media capture plan uses a canvas click instead of the named action, because `SceneCaptureAction` has no action kind. The README and survey use the named `squeeze` action as D-17 requires, so this does not affect any success criterion.

### Required Artifacts

| Artifact | Status | Details |
| --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/cartesian_diver.rs` | VERIFIED | 264 lines. Contains `impl SceneHooks for CartesianDiverHooks`, with no `unwrap` in production code. |
| `.../cartesian_diver/geometry.rs` | VERIFIED | 228 lines. Contains `outline_segments`, `in_squeeze_region`, `bubble_circle` and the wall quads. |
| `.../cartesian_diver/tests.rs` | VERIFIED | 569 lines. 34 unit tests, including the WR-01 regression `tap_outside_the_bottle_keeps_a_latched_squeeze`. |
| `.../cartesian_diver/motion_tests.rs` | VERIFIED | 332 lines. 11 tests, each with `cfg_attr(debug_assertions, ignore = ...)`. |
| `web/src/catalog/scene-records-curiosity.ts` | VERIFIED | Defines `CURIOSITY_SCENE_RECORDS`. |
| `web/src/catalog/previews-curiosity.tsx` | VERIFIED | Defines `export function CartesianDiverPreview`, which draws the bowed sides and the small bubble. |
| `web/src/catalog/scene-view-bounds.ts` | VERIFIED | `CARTESIAN_DIVER_VIEW_BOUNDS` is (-1.6, -0.4, 1.6, 6.0). |
| `web/src/styles/player-viewport.css` | VERIFIED | Contains `-webkit-touch-callout: none` on `canvas.scene-canvas`. |
| `web/e2e/canvas-callout.spec.ts`, `web/tests/canvas-callout.test.ts` | VERIFIED | Both present and substantive. |
| `docs/assets/readme/cartesian-diver-10s.svg` and `.webp` | VERIFIED | Tracked. The SVG is 5.1 MB and the WebP 24 MB, in line with the other scenes. |
| `README.md` gallery entry | VERIFIED | Contains `cartesian-diver-10s.webp`. |
| `docs/benchmarks/scene-fingerprints.json` | VERIFIED | Contains `"id": "cartesian-diver"`. Provenance points at 89983a79b, stamp 2026-10-14T19-29-46Z. |

### Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| `scene.rs` | `cartesian_diver.rs` | `SceneId::CartesianDiver => cartesian_diver::build` (line 284), plus `mod`, parse (135) and `ALL_SCENE_IDS` (105) | WIRED |
| `on_advance` | `set_fixture_density` + `reset_body_mass_data` | Runs once per fixed step, only when the ramp changes | WIRED |
| `scenes.ts` `SCENE_IDS` | Rust `ALL_SCENE_IDS` | `"cartesian-diver"` last in both, with an order test | WIRED |
| `scene-records.ts` | `scene-records-curiosity.ts` | `...CURIOSITY_SCENE_RECORDS` (line 22) | WIRED |
| `previews.tsx` | `previews-curiosity.tsx` | `case "cartesian-diver"` (line 92) | WIRED |
| `PlayerPanel.tsx` | canvas contextmenu | `addEventListener("contextmenu", preventCanvasContextMenu)` in the ref callback | WIRED |
| `README.md` | `cartesian-diver-10s.webp` | Gallery entry between the markers | WIRED |
| `scene_spot.rs` `SURVEY_SCENES` | squeeze action | `SurveyCue::Action("squeeze")` | WIRED |
| `gravity_slider.rs`, `session/tests.rs`, `portrait-bounds.ts`, `demo-media/model.ts`, e2e `player-helpers.ts` / `player.spec.ts` `POINTER_CONTROL` | diver id | Registration entries present | WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Real data | Status |
| --- | --- | --- | --- | --- |
| `collect_segments` | outline bow | `self.squeeze()`, the ramp driven by the input state | Yes | FLOWING |
| `collect_circles` | diver and bubble | `world.body_snapshot(diver).position()` and `world.gravity()` | Yes | FLOWING |
| Diver motion | mass | Fixture density written from the ramp, then `reset_body_mass_data`. Buoyancy comes from the engine's particle water, with no scripted force (D-08). | Yes | FLOWING |

### Behavioral Spot-Checks (re-run on the final tree)

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Sink and rise at g 2, 10 and 80, README schedule, containment, input model | `cargo test -p liquidfun-wasm --release cartesian_diver` | 45 passed, 0 failed, 0 ignored | PASS |
| Web catalog, portrait, README plan, controls, credits, callout CSS | `cd web && bun run typecheck && bun run test:unit` | 120 files, 1849 tests passed, exit 0 | PASS |
| File lengths and managed checks | `bun scripts/bright-builds-check.ts all` | findings=0 | PASS |
| Formatting and lint | `cargo fmt --all --check` and `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | Both clean | PASS |
| Fingerprint gate on HEAD 7230d5d47 | `target/phase46/fingerprint-gate-final-after-review-fix.log` (evidence reviewed, not re-run) | PASS, 26/26 unchanged | PASS |
| E2E (player, shell, hud-safe-frame, canvas-callout) | 46-04 gate step 13, 97 passed (not re-run) | The only later code change is the Rust pointer guard, which is covered by unit tests and the gate | PASS (recorded) |

### Shared Acceptance Rules

| Rule | Status | Evidence |
| --- | --- | --- |
| Existing scenes unchanged (PREP-01); the baseline is never regenerated in the same commit as a scene change | PASS | The final gate reports 26/26. 18c8a5e92 changes a single file. |
| SCENE-01 catalog rules | PASS | See truth 4. |
| SCENE-02 light and visibly moving | PASS | The press-invited exception applies to the diver. The diver floats at rest, with 1,936 of the 3,000-particle budget and 0.58 ms/step. |
| SCENE-03 honest caption | PASS | The caption names the density switch and says the dent and bubble are drawn. A grep found no "compressed air", "suction", "simulated density" or "real pressure" claims. |
| No engine change | PASS | `git diff --stat 7b489b4ef HEAD -- crates/liquidfun` is empty. Up and Cancel are treated alike, and density changes happen only in `on_advance`. |
| No new dependencies or cue kinds | PASS | No Cargo or npm manifest or lockfile changed. The README and survey cues reuse the existing `action` and `SurveyCue::Action` kinds. Input is press and toggle only. |
| Normal local checks | PASS | See the spot-checks above and the 46-04 15-step gate log. |

### Requirements Coverage

| Requirement | Source plans | Status | Evidence |
| --- | --- | --- | --- |
| DIVER-01 | 46-01, 46-02 | SATISFIED | Truth 1 |
| DIVER-02 | 46-01 | SATISFIED | Truth 2 (dent and bubble) |
| DIVER-03 | 46-01, 46-03 | SATISFIED | Truth 3 (gravity range) and truth 2 (long press) |
| SCENE-01 | 46-02, 46-04, 46-05 | SATISFIED | Truths 4 and 5 |
| SCENE-02 | 46-01, 46-04 | SATISFIED | 1,936 particles and 0.582 ms/step recorded |
| SCENE-03 | 46-02 | SATISFIED | Caption check |

Every plan's requirement IDs account for all six requirements, and REQUIREMENTS.md maps no other ID to Phase 46, so none are orphaned. DIVER-04 is a future requirement and out of scope.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| (phase files) | - | No TODO, FIXME, placeholder or `unwrap()` in production code | - | - |
| `motion_tests.rs` | - | The release-only motion tests have no `just` recipe or CI lane (REVIEW IN-03) | Info | The core timing behavior is checked by hand or at phase gates. The fingerprint gate catches drift but not the sink and rise semantics. |
| `geometry.rs` | 54-56 | The hit region extends about 0.45 m past the drawn outline (REVIEW IN-01) | Info | This matches D-02 ("outer rectangle plus ~0.3 m"). It only affects how generous the touch target is. |
| `scene-records-curiosity.ts` | 13 | The Squeeze button is a toggle with no `aria-pressed` (REVIEW IN-06) | Info | The hint says "use Squeeze again". The same pattern exists elsewhere. |

### Optional Manual Checks (not gating)

1. **Real-iPhone long press on the bottle.** Press and hold the bottle in Safari on an iPhone. Expected: no callout and no text selection, and the diver sinks while held. This is listed as optional, not a gate, in CONTEXT D-14. Chromium cannot compute `-webkit-touch-callout`, but the declaration is verified in both the source and the built CSS, and the computed `user-select` and `touch-action` values plus the prevented contextmenu are verified in e2e.
1. **WR-01 feel.** Latch with Squeeze, then press and release inside the bottle. The release clears the latch ("latest input wins"), and the hint says the same. Unit tests cover this. Whether it feels right is a subjective call.

### Gaps Summary

None. Every roadmap success criterion, every PLAN must-have, every shared acceptance rule and all six requirement IDs are verified on the final tree. I independently re-ran the release motion suite (45/45), the web typecheck and unit tests (1849/1849), the Bright Builds checks (0 findings), and rustfmt and clippy. Only Info-level review items remain.

---

_Verified: 2026-10-14T19:46:29Z_
_Verifier: Claude (gsd-verifier)_
