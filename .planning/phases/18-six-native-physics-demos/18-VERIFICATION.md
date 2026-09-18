---
phase: 18-six-native-physics-demos
verified: 2026-09-18T05:47:53Z
status: passed
score: 6/6 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T05:47:53Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 18: Six Native Physics Demos Verification Report

**Phase Goal:** Visitors can choose six distinct, persistent physics scenes and experiment with each scene's real Rust behavior through a small, understandable control surface.
**Verified:** 2026-09-18T05:47:53Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

The phase goal holds in the live tree. The catalog is six ready cards with static previews and hash Open links. The shared player loads any allowlisted id through one WASM session, mounts two-to-four labeled controls with an explicit construction-reset sentence, and shows per-scene source/inspiration/notices beside the existing footer. Each scene module constructs a real `liquidfun::World` and is covered by native honesty tests. Pointer polish, narrow-width acceptance, and WEBTEST-01 remain Phase 19. A new Pages deploy is not a Phase 18 gate.

Objective UAT checkpoints that are statically or command-checkable (catalog ids/ready flags, control metadata, factory allowlist, credit host-lock, player session dispose, unit tests, existing smoke log, independent digest) were auto-passed with `verified_by: agent`. No checkpoint required human judgment.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The catalog presents Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel with names, descriptions and previews; each opens its working scene in the shared player with labeled, bounded controls and explicit reset-on-change behavior. | ✓ VERIFIED | `web/src/catalog/scenes.ts` marks all six `ready: true` with locked titles, one-sentence descriptions, and 2–4 labeled controls. `CatalogNav.tsx` renders static `ScenePreview` SVG plus caption `Static preview` and `href={\`#/scene/${scene.id}\`}`. `SceneControls.tsx` shows `Changing this setting recreates the scene from its documented initial state.` and `Apply setting` only when `recreates: true`. `App.tsx` loads any ready id via `loadSceneSession` and mounts those controls in `PlayerPanel` children. |
| 2 | Dam Break releases water into a basin with an interactive obstacle and repeatable reset; Fountain allows stream aiming or adjustment while lifetime/capacity limits make particle population plateau. | ✓ VERIFIED | `dam_break.rs` builds the documented Medium/Normal 192-particle basin, gravity `(0,-10)`, Small 64 / Large 280, Low/High gravity `(-6)/(-16)`, and live `drop-obstacle` / `reset-obstacle` transforms. `fountain.rs` emits with `with_lifetime(3.0)`, `with_maximum_count(320)`, `with_destruction_by_age(true)`; live `emission-rate` / `launch-speed` / `aim-angle`; test `default_medium_stream_plateaus_at_or_below_three_hundred_twenty`. |
| 3 | Float or Sink accepts body/density presets and displays native particle-body response; Color Mixer allows stirring and visibly uses the engine's contact-driven particle color changes rather than rendering-only blending. | ✓ VERIFIED | `float_or_sink.rs` pool is 15×12 water, densities Cork 0.3 / Wood 0.6 / Stone 2.0, live `body` + `drop-body`, and `cork_y > stone_y` after 120 steps. UI default `body: "wood"` matches `BodyPreset::Wood`. `color_mixer.rs` builds two `COLOR_MIXING` groups teal `(57,211,199,255)` and red `(248,113,113,255)`; Off `0.0` keeps lanes; Strong `0.5` changes captured color bytes; `stir-speed` applies live tangential forces. |
| 4 | Jelly Drop deforms an elastic particle shape through drops/pokes against obstacles with stable bounded presets; Water Wheel responds to a variable jet through native particle-body coupling and a joint, without scripted wheel rotation. | ✓ VERIFIED | `jelly_drop.rs` uses `ParticleGroupRecipe` with `ELASTIC \| SPRING`; shape/softness recreate; `poke-jelly` applies `apply_particle_linear_impulse_range`. `water_wheel.rs` creates `RevoluteJointDef` with motor disabled (speed/torque bits 0); paddles captured from `BodySnapshot::transform().apply`; jet/emission live; emission-off leaves angle nearly unchanged. |
| 5 | Visitors can reach the repository, every scene's implementation, inspiration and applicable notices from the site; explanations and stable source/provenance chrome truthfully identify the experimental Rust implementation. | ✓ VERIFIED | `SiteFooter` links `https://github.com/bright-builds-llc/liquidfun-rs`, MIT “Free and open source”, maintainer, version/commit/build. `SceneCredits` renders `Scene source`, host-locked `sceneBlobUrl` implementation, inspiration (showcase / pinned Faucet / particle guide), and `noticesBlobUrl` → `THIRD_PARTY_NOTICES.md`. Page copy and `PlayerPanel` say experimental Rust / `Rust engine · WebAssembly`. Implementation hrefs reject `google/liquidfun`. |
| 6 | Independent AI review acknowledges the six-scene diff and evidence at an exact digest; the implementing agent does not approve its own work. | ✓ VERIFIED | `18-REVIEW.md` binds approval to digest `1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`, reviewer `f5a42aa5-af9e-47d2-93c3-2e2c9312809d` (`gsd-code-reviewer`), disclosure “AI reviewer, not a human”, `implementing_or_fixing_executor: no`. Decision APPROVED; 0 critical / 0 unresolved warning. |

**Score:** 6/6 truths verified

Roadmap success criteria 1–5 are truths 1–5. Truth 6 is the non-duplicative 18-10 / D-17 review gate. Intermediate Plan 02/08 truths that required Dam Break-only `ready:true` were superseded by Plan 09 and are not scored as failures.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/liquidfun-wasm/src/scene.rs` | SceneId parse, factory match, SceneHooks | ✓ VERIFIED | 134 lines. `parse_scene_id` allowlists six hyphenated ids; `build_scene` matches all six modules. |
| `crates/liquidfun-wasm/src/lib.rs` | `ProofSession::new(String)` + apply_control/apply_action | ✓ VERIFIED | Parses id before `SessionCore::create`; wasm-bindgen `applyControl` / `applyAction`. |
| `crates/liquidfun-wasm/src/scene/dam_break.rs` | Evolved basin, water/gravity, obstacle actions | ✓ VERIFIED | 511 lines with construction presets and live obstacle actions. |
| `crates/liquidfun-wasm/src/scene/fountain.rs` | Bounded emitter and bowl | ✓ VERIFIED | 408 lines; lifetime + max 320 + destruction-by-age. |
| `crates/liquidfun-wasm/src/scene/float_or_sink.rs` | Density presets and drop-body | ✓ VERIFIED | 441 lines; native y-separation test. |
| `crates/liquidfun-wasm/src/scene/color_mixer.rs` | Two mixing groups, mix-strength, stir | ✓ VERIFIED | 464 lines; COLOR_MIXING + strength tests. |
| `crates/liquidfun-wasm/src/scene/jelly_drop.rs` | Elastic group, shape/softness, poke | ✓ VERIFIED | 469 lines; ELASTIC\|SPRING recipe. |
| `crates/liquidfun-wasm/src/scene/water_wheel.rs` | Motor-off wheel, live jet | ✓ VERIFIED | 665 lines; revolute hub + transform capture. |
| `web/src/catalog/scenes.ts` | Descriptions, controls, credits, ready flags | ✓ VERIFIED | 261 lines; six `ready: true` records. |
| `web/src/catalog/links.ts` | Host-locked blob URLs | ✓ VERIFIED | 50 lines; 40-hex SHA or `main`; scene-path allowlist. |
| `web/src/catalog/previews.tsx` | Static inline SVG per id | ✓ VERIFIED | 125 lines; no WASM import. |
| `web/src/components/CatalogNav.tsx` | Six-card hash Open | ✓ VERIFIED | 55 lines; wired from `SCENES`. |
| `web/src/components/SceneControls.tsx` | Labeled presets/actions + Apply setting | ✓ VERIFIED | 116 lines; construction hint + live apply. |
| `web/src/components/SceneCredits.tsx` | Implementation / inspiration / notices | ✓ VERIFIED | 83 lines; uses `sceneBlobUrl` / `noticesBlobUrl`. |
| `web/src/player/runtime.ts` | `isReadySceneRoute` / construction entries | ✓ VERIFIED | 75 lines; no leftover `isDamBreakRoute`. |
| `web/src/physics/loader.ts` | `loadSceneSession(id)` | ✓ VERIFIED | `new ProofSession(sceneId)`; Dam Break wrapper retained. |
| `web/src/physics/session.ts` | applyControl/applyAction owner | ✓ VERIFIED | Disposes generated session; 1–4 step cap. |
| `web/src/App.tsx` | One-session start/abandon + controls/credits | ✓ VERIFIED | 593 lines; `startScene` disposes prior world. |
| `web/src/app.css` | Three/two/one column card grid | ✓ VERIFIED | 3-col default; 2-col ≤720px; 1-col ≤480px. |
| `web/e2e/player.spec.ts` | Thin six-scene open/reset/switch/credits | ✓ VERIFIED | 283 lines; loops `SCENES`; no pointer/WEBTEST-01. |
| `.planning/phases/18-six-native-physics-demos/18-REVIEW.md` | Independent exact-digest acknowledgment | ✓ VERIFIED | Digest `1f6349eb…` bound to reviewer `f5a42aa5…`. |

`gsd-tools verify artifacts` reported `all_passed: true` on every Plan 01–10 artifact list.

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `lib.rs` | `scene.rs` | `parse_scene_id` then `SessionCore::create` | WIRED | Tool verified. |
| `loader.ts` | `ProofSession` | `new ProofSession(sceneId)` | WIRED | Tool regex escaped incorrectly; source line 9 is exact. |
| `links.ts` | 40-hex SHA gate | `FULL_SHA_PATTERN = /^[0-9a-f]{40}$/` | WIRED | Tool missed escaped regex; pattern is present. |
| `scenes.ts` | `crates/liquidfun-wasm/src/scene/*.rs` | `sceneSource(fileName)` | WIRED | Tool looked for a literal `dam_break.rs` path string; helper builds that path for all six modules. |
| `scene.rs` | each `scene/*.rs` | `foo::build` factory match | WIRED | All six modules matched. |
| `CatalogNav.tsx` | `#/scene/{id}` | Open href | WIRED | Tool quote-wrapping miss; line 43 is `href={\`#/scene/${scene.id}\`}`. |
| `previews.tsx` | `scenes.ts` SceneId | Static SVG, no ProofSession | WIRED | Tool verified. |
| `App.tsx` | `loader.ts` | `loadSceneSession` | WIRED | Tool verified. |
| `SceneControls.tsx` | Apply setting | Construction CTA | WIRED | Tool verified. |
| `SceneCredits.tsx` | `links.ts` | `sceneBlobUrl` / `noticesBlobUrl` | WIRED | Tool verified via `implementationHref`. |
| `player.spec.ts` | `/liquidfun-rs/#/scene/` | Six-id loop + Gravity Apply | WIRED | Tool verified. |
| `18-REVIEW.md` | digest | Separate reviewer + SHA-256 | WIRED | Tool verified. |
| `jelly_drop.rs` | `ParticleFlags::ELASTIC` | Group recipe | WIRED | Tool verified. |
| `water_wheel.rs` | `RevoluteJointDef::new` | Motor-off hub | WIRED | Tool verified. |
| `dam_break.rs` | `set_body_transform` | Obstacle drop/reset | WIRED | Tool verified. |
| `fountain.rs` | `with_lifetime` | Bounded emission | WIRED | Tool verified. |
| `color_mixer.rs` | `ParticleFlags::COLOR_MIXING` | Contact mixing | WIRED | Tool verified. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `CatalogNav` | `SCENES` titles/descriptions/previews | `web/src/catalog/scenes.ts` authored records | Yes — six locked catalog rows, not `[]` | ✓ FLOWING |
| `SceneControls` | `currentScene().controls` | Same catalog records via `App.tsx` | Yes — per-scene presets/actions | ✓ FLOWING |
| `SceneCredits` | `implementationPath` / inspiration | Catalog credits + `sceneBlobUrl` | Yes — host-locked blob URLs | ✓ FLOWING |
| `PlayerPanel` canvas | `ownedSession.nextFrame` | `ProofSession` capture of Rust world | Yes — copied typed frames from `SessionCore` | ✓ FLOWING |

Catalog previews are intentionally static SVG (D-02). That is authored illustration, not a hollow live-data path.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| All six catalog ids ready | Count `ready: true` in `scenes.ts` | 6 true, 0 false | ✓ PASS |
| Catalog Open hash links | `rg` `href={\`#/scene/${scene.id}\`}` | Line 43 | ✓ PASS |
| Factory + session construct | `new ProofSession(sceneId)` in loader | Present | ✓ PASS |
| Catalog/control/credit unit tests | `bun run test:unit -- tests/scenes.test.ts tests/controls.test.ts tests/credits.test.ts tests/links.test.ts tests/runtime.test.ts` | 5 files, 26 tests passed in 99ms | ✓ PASS |
| Local six-scene smoke log | `target/web-build/web-build.log` | `start player-smoke` … `complete player-smoke` | ✓ PASS |
| Independent digest present | `18-REVIEW.md` frontmatter | `1f6349eb…` + `f5a42aa5…` | ✓ PASS |

Skipped: live Chromium re-run and `cargo test -p liquidfun-wasm` (compile-heavy, >10s). Existing ignored smoke log and in-module Rust tests were inspected as evidence, not re-executed.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| WEB-01 | 18-08, 18-09, 18-10 | Six demo cards with names, descriptions, previews; open in shared player | ✓ SATISFIED | `CatalogNav` + `SCENES` + hash Open + `App` player |
| WEB-04 | 18-02, 18-09, 18-10 | Labeled bounded controls; reset-on-change indicated | ✓ SATISFIED | Catalog control metadata + `SceneControls` hint/Apply |
| WEB-08 | 18-02, 18-09, 18-10 | Repo, implementation, inspiration, notices; experimental Rust claims | ✓ SATISFIED | `SiteFooter` + `SceneCredits` + host-lock tests |
| DEMO-01 | 18-01, 18-06, 18-10 | Dam Break basin, obstacle, reset | ✓ SATISFIED | `dam_break.rs` + native tests |
| DEMO-02 | 18-06, 18-10 | Fountain aim/adjust with bounded emission | ✓ SATISFIED | `fountain.rs` plateau test |
| DEMO-03 | 18-03, 18-10 | Float or Sink density presets, native response | ✓ SATISFIED | cork-vs-stone y-separation test |
| DEMO-04 | 18-07, 18-10 | Color Mixer stir + contact-driven mixing | ✓ SATISFIED | COLOR_MIXING + Off/Strong color-lane tests |
| DEMO-05 | 18-04, 18-10 | Jelly Drop elastic drop/poke, bounded presets | ✓ SATISFIED | ELASTIC\|SPRING + poke impulse tests |
| DEMO-06 | 18-05, 18-10 | Water Wheel jet via joint, no scripted rotation | ✓ SATISFIED | motor-off revolute + angle tests |

No orphaned Phase 18 requirement IDs. `REQUIREMENTS.md` maps WEB-05, WEB-07, and WEBTEST-01 to Phase 19; they were not claimed by these plans and are not gaps here.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `crates/liquidfun-wasm/src/scene/jelly_drop.rs` | 74–80 | Unknown shape/softness construction tokens `unwrap_or` Circle/Medium | ℹ️ Info | Independent review IN-01. Browser only sends allowlisted select values. Live `apply_control` still fails closed. |
| `web/src/components/FallbackPanel.tsx` | 31 | Remaining `{title} is not ready yet` copy | ℹ️ Info | Dead visitor path while all six ids are `ready: true`. Empty/unknown fallbacks still match UI-SPEC. |
| `web/e2e/player.spec.ts` | — | Six-scene loop asserts Playing/title/credits/reset, not fountain plateau or color-lane bytes in the browser | ℹ️ Info | Physics honesty is in Rust unit tests. WEBTEST-01 is Phase 19. |

No TODO/FIXME/PLACEHOLDER stubs in catalog, controls, credits, or scene modules. No catalog WASM session. No `innerHTML`. No `isDamBreakRoute`.

### Human Verification Required

None. Visual/pointer feel and the complete six-scene browser/accessibility matrix are Phase 19 (`WEB-05`, `WEB-07`, `WEBTEST-01`). Phase 18 local proofs are objectively evidenced by source, unit tests, `player.spec.ts`, and the existing smoke log.

### Gaps Summary

No goal-blocking gaps. Later-phase work that is explicitly out of this contract:

- Pointer aiming/stirring/poking polish and stuck-pointer handling — Phase 19 WEB-05
- Narrow-width catalog/control accessibility acceptance — Phase 19 WEB-07
- Complete six-scene production-subpath / live Pages smoke — Phase 19 WEBTEST-01
- New GitHub Pages deploy — not a Phase 18 gate

Confirmation-bias notes (do not fail the phase): `18-10-SUMMARY.md` still names the pre-WR-01 digest `22635277…`; current `18-REVIEW.md` correctly binds `1f6349eb…` after commit `504d720`. Jelly Drop construction still fail-opens unknown tokens (IN-01).

---

_Verified: 2026-09-18T05:47:53Z_
_Verifier: Claude (gsd-verifier)_
