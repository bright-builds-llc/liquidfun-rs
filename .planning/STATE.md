---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Web Playground
status: executing
stopped_at: Completed 21-01-PLAN.md
last_updated: "2026-09-20T20:16:30.008Z"
last_activity: 2026-09-20
progress:
  total_phases: 6
  completed_phases: 5
  total_plans: 39
  completed_plans: 36
  percent: 92
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-17)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 21 — playground-leftover-cleanup

## Current Position

Phase: 21 (playground-leftover-cleanup) — EXECUTING
Plan: 2 of 4
Status: Ready to execute
Last activity: 2026-09-20

Progress: [█████████░] 92%

The prior 16 phases and 252 plans remain archived history. New work starts at Phase 16. The owner confirmed six interactive demos using our Rust engine via WASM, playful catalog/player controls and automatic GitHub Pages deployment from main.

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |

## Accumulated Context

| Phase 16 P01 | 13 min | 3 tasks | 7 files |
| Phase 16 P02 | 7 min | 2 tasks | 14 files |
| Phase 16 P03 | 6 min | 2 tasks | 7 files |
| Phase 16 P04 | 50min | 3 tasks | 9 files |
| Phase 17-shared-player-and-early-pages-delivery P01 | 2 min | 2 tasks | 4 files |
| Phase 17 P02 | 2 min | 2 tasks | 4 files |
| Phase 17 P03 | 2 min | 2 tasks | 4 files |
| Phase 17 P04 | 3min | 2 tasks | 7 files |
| Phase 17 P05 | 7min | 2 tasks | 9 files |
| Phase 17-shared-player-and-early-pages-delivery P06 | 6min | 2 tasks | 7 files |
| Phase 17 P07 | 2min | 2 tasks | 1 files |
| Phase 18 P01 | 10 min | 2 tasks | 10 files |
| Phase 18 P02 | 2 min | 2 tasks | 4 files |
| Phase 18-six-native-physics-demos P03 | 10 min | 2 tasks | 2 files |
| Phase 18-six-native-physics-demos P04 | 13 min | 2 tasks | 2 files |
| Phase 18-six-native-physics-demos P05 | 8 min | 2 tasks | 2 files |
| Phase 18 P08 | 2 min | 2 tasks | 5 files |
| Phase 18-six-native-physics-demos P06 | 12 min | 2 tasks | 3 files |
| Phase 18-six-native-physics-demos P07 | 13 min | 2 tasks | 2 files |
| Phase 18-six-native-physics-demos P09 | 6 min | 2 tasks | 15 files |
| Phase 18 P10 | 6min | 2 tasks | 5 files |
| Phase 19 P01 | 2min | 2 tasks | 4 files |
| Phase 19 P02 | 5min | 2 tasks | 9 files |
| Phase 19 P03 | 4min | 2 tasks | 3 files |
| Phase 19-interaction-polish-and-browser-verification P04 | 4 min | 2 tasks | 5 files |
| Phase 19 P05 | 2min | 2 tasks | 5 files |
| Phase 19-interaction-polish-and-browser-verification P06 | 4min | 2 tasks | 4 files |
| Phase 19-interaction-polish-and-browser-verification P07 | 10min | 2 tasks | 6 files |
| Phase 20 P01 | 2 min | 2 tasks | 3 files |
| Phase 20 P02 | 2min | 2 tasks | 3 files |
| Phase 20 P03 | 4min | 2 tasks | 3 files |
| Phase 20 P04 | 20min | 2 tasks | 2 files |
| Phase 20 P05 | 22min | 2 tasks | 6 files |
| Phase 20 P06 | 16min | 2 tasks | 1 files |
| Phase 21 P01 | 3 min | 2 tasks | 3 files |

### Decisions

- Prove a real Rust WASM browser step first, then deploy a shared SolidJS player early before expanding the six scenes.
- Approved scenes: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel; all physics comes from this Rust engine.
- Prefer a private typed package, owned bulk frames, one active session and bounded simulation time/resources. Native Cargo consumers remain isolated.
- Every main push triggers a same-checkout site/WASM build and Pages delivery with latest-main protection. This is web delivery, not Linux native qualification.
- [Phase 16]: Keep liquidfun as the sole default workspace member and depend outward from the private wrapper.
- [Phase 16]: Copy five validated bounded numeric lanes into JavaScript-owned typed arrays instead of exposing WASM memory.
- [Phase 16]: Reject invalid advance counts and step-index overflow before invoking the engine.
- [Phase 16]: Record compilation and ES-module import only; real Chromium execution remains Plan 16-04.
- [Phase 16]: Regenerate ignored wasm-pack output from the current checkout before frontend verification or build.
- [Phase 16]: Defer the complete Vite build until Plan 16-03 supplies renderer and application entrypoints.
- [Phase 16]: Poison the TypeScript session owner after any advance, capture, parse, or frame-cleanup failure.
- [Phase 16]: Keep world fitting and y-axis inversion in one pure camera module while Canvas effects consume validated bulk arrays.
- [Phase 16]: Represent loading, running, failure, and disposed as a tagged union carrying only valid frame observations.
- [Phase 16]: Advance exactly one Rust frame per animation callback and derive browser proof attributes from consecutive Rust frame lanes.
- [Phase 16]: Allocate every smoke attempt before execution so failures remain immutable forensic records.
- [Phase 16]: Use Canvas pixel SHA-256 changes and attached PNG bytes alongside Rust movement counters.
- [Phase 16]: Redraw only the last Rust frame on resize and disconnect resize effects before terminal cleanup.
- [Phase 16]: Bind implementation, declarations, logs, metadata, and PNG bytes into one independent review digest.
- [Phase 17]: Keep #/scene and #/scene/ as empty so later UI can show the empty-hash fallback instead of unknown copy. — D-06 requires empty and unknown hashes to stay distinct useful states.
- [Phase 17]: Do not lowercase hash tokens; Dam-Break stays unknown with maybeRaw preserved. — Canonical ids are lowercase hyphenated tokens. Coercing case would hide invalid shared URLs.
- [Phase 17]: Known not-ready ids parse as scene; only dam-break is ready in catalog data. — Readiness is catalog metadata, not parser output, so later player code cannot construct a WASM world from a parsed scene kind alone.
- [Phase 17]: Keep acceptedStepCount pure: no performance.now, document.hidden, leftover accumulator, or rAF. — D-12 caps accepted wall-clock delta at four 1/60-second steps. Hidden-tab pause and leftover catch-up belong to the later player shell.
- [Phase 17]: Guard 1..=4 in TypeScript before generated advance; poison invalid counts without calling advance. — Rust already rejects 0 and 5+. TypeScript must not cross the WASM boundary with an illegal count or leak generated exception text.
- [Phase 17]: Forward one advance(n) plus one capture instead of calling nextFrame four times. — Four nextFrame calls would capture four times. WASM-04 needs one animation callback to step 1-4 ticks and still capture once.
- [Phase 17]: Production and preview Vite base is /liquidfun-rs/; local serve stays /. — Local vite serve must keep root hrefs; only production and preview-of-production need the GitHub Pages project prefix.
- [Phase 17]: Short commit labels are the first 12 lowercase hex characters of a full SHA. — Operability and UI-SPEC allow 7-12 characters; 12 stays unique while remaining readable in footer chrome.
- [Phase 17]: Commit and build URLs are accepted only on https://github.com/bright-builds-llc/liquidfun-rs/. — Footer links must not be built from hash-route input or arbitrary env strings; javascript: and off-host URLs stay undefined.
- [Phase 17]: just web-build fails unless dist/index.html contains /liquidfun-rs/assets/ and a .wasm file exists. — A root-base Vite build would 404 WASM on GitHub Pages; fail closed before any deploy job.
- [Phase 17]: Record D-09 as a Phase 17 semantic-HTML plus one scoped CSS exception; revisit MysticUI/Tailwind on 2026-12-17. — Pages delivery must not be blocked by adopting MysticUI or Tailwind in this thin slice.
- [Phase 17]: Keep the Scenes catalog label as a styled paragraph so the player or fallback heading remains the only h2. — 17-UI-SPEC allows only one h2 on a given view; the catalog label is not that heading.
- [Phase 17]: Leave App.tsx on the Phase 16 proof shell; Plan 17-05 mounts these presentational panels. — This plan owns chrome only and must not wire the WASM session.
- [Phase 17]: Reuse loadProofSession/ProofSession as named Dam Break instead of renaming the generated class. — wasm-pack output stays ignored; the existing basin constructor already matches the documented Dam Break reset constants.
- [Phase 17]: Extract observeFrame helpers so App.tsx stays under the 628-line file trigger. — The plan allowed a split; frame observation is not session ownership and can stay a pure helper.
- [Phase 17]: Allow explicit undefined on optional chrome props for exactOptionalPropertyTypes. — Solid call sites pass absent current-scene and failure-details values; widening the optional type unblocked typecheck without extra JSX branches.
- [Phase 17]: Player smoke is a web-build player-smoke mode that reuses the existing build and does not allocate a Phase 16 closure attempt. — D-17 local proofs must not run the Phase 16 forensic closure matrix. just web-smoke stays the opt-in forensic path that still sets PHASE16_CLOSURE_ATTEMPT_DIR.
- [Phase 17]: Hidden-tab proof uses Object.defineProperty plus a MutationObserver so the first resume frame is measured, not a later poll. — document.hidden is read-only. Playwright poll can sample a later animation callback and see a step delta of 5, which is not a catch-up storm. The first data-step-index mutation is the next observed frame.
- [Phase 17]: Every main and pull_request run builds WASM plus web/dist from the same checkout with no path filters. — HOST-01 requires a same-checkout site+WASM build on every main push so a Rust-only change still ships updated physics.
- [Phase 17]: Deploy uses job-scoped pages: write plus id-token: write and the github-pages environment; no PAT. — HOST-02 forbids a personal token; write permissions stay on deploy-pages so pull requests keep contents: read only.
- [Phase 17]: cancel-in-progress is false on main so an in-flight deploy finishes while queued revisions may coalesce. — Cancelling a mid-flight Pages deploy can strand a newer revision; official starter keeps the in-progress run and coalesces queued main SHAs.
- [Phase 18]: parse_scene_id accepts only the six lowercase hyphenated tokens and does not coerce case. — Canonical ids are locked. Coercing case would hide invalid shared URLs and construct the wrong world.
- [Phase 18]: SessionCore stores a generic preset bag and rebuilds on ControlEffect::Recreated so later scene files do not edit session.rs. — Wave-2 scene plans must own only their scene file. The generic bag has to exist in Plan 01.
- [Phase 18]: Native ProofSession error-path tests use build_core because wasm-bindgen JsError cannot be constructed on non-wasm targets. — JsError::new panics in cargo test --lib on macOS. The constructor still maps SessionError through js_error for WASM.
- [Phase 18]: Keep build(presets) on every scene module, including stubs that ignore the bag. — Plan-checker required the presets argument so later files do not change the factory signature.
- [Phase 18]: Include the UI-SPEC Poke jelly action on jelly-drop even though the machine-id table omitted it. — UI-SPEC and D-05 list the poke action; catalog chrome later needs that control id.
- [Phase 18]: Throw a fixed allowlist error for traversal or non-scene implementation paths. — D-13 and T-18-02-02 require fail-closed scene paths so google/liquidfun never becomes implementation.
- [Phase 18]: Copy the 40-hex SHA pattern into links.ts instead of importing Vite env from build-info. — Catalog URLs must stay hash-route independent and must not accept caller-supplied origins.
- [Phase 18-six-native-physics-demos]: Keep build(presets) on Float or Sink and ignore the bag; density is live on drop-body. — Plan 01 locked the factory signature. Density is a live drop preset, not a construction reset.
- [Phase 18-six-native-physics-demos]: Drop a dynamic circle at (0, 6) with fixture densities 0.3/0.6/2.0 against particle density 1.0. — Matches the locked research enumerations and the cork-versus-stone honesty test spawn.
- [Phase 18-six-native-physics-demos]: Cork-versus-stone y-separation is proven by native coupling; no engine API expansion. — D-09 and D-11 forbid fake buoyancy and public liquidfun changes to save the title.
- [Phase 18-six-native-physics-demos]: Keep build(presets) and parse shape/softness from the bag; defaults are circle and medium 1.0. — Plan 01 locked the factory signature. Softness is recipe strength on reset, not a live setter.
- [Phase 18-six-native-physics-demos]: Poke impulse is (0, -8) on a contiguous first-third of group member ids. — A whole-group poke launched particles past the camera box. A contiguous slice deforms the blob without shredding.
- [Phase 18-six-native-physics-demos]: Overlapping bars near y=1 plus system elastic/spring 0.75 keep the blob inside the camera box after poke. — Assumption A2: retune count/strength/shelf rather than expand the public engine API.
- [Phase 18-six-native-physics-demos]: Keep build(presets) and ignore the bag; jet strength and emission are live. — Plan 01 locked the factory signature. Jet and emission are runtime policy, not construction resets.
- [Phase 18-six-native-physics-demos]: Hub at (0, 3) uses RevoluteJointDef::new without a motor; paddles are local fixtures plus transformed segments. — D-09 and D-11 forbid motor spin. Capture must follow BodySnapshot::transform().apply.
- [Phase 18-six-native-physics-demos]: Medium jet at 8 m/s from the left plus two WATER particles per step rotates the wheel natively; no D-11 pinwheel fallback. — Assumption A3 held on the first tuned emit. Do not enable the motor or substitute the scene.
- [Phase 18]: Mark current only when maybeCurrentSceneId matches a ready id so stub cards never get aria-current. — Plan 08 ships cards before Plan 09 flips ready. Current chrome must not imply a stub scene is playing.
- [Phase 18]: Use an inset 4px accent bar via box-shadow so the 1px card border stays visible. — 18-UI-SPEC requires both a 1px #2A3441 card border and a 4px current-scene accent on the inline start.
- [Phase 18]: Keep the not-ready fallback branch as unused defensive copy; do not edit PAGE_SUMMARY. — Plan 09 owns ready flags and the six-demo page summary. Plan 08 only updates empty/unknown fallback strings.
- [Phase 18-six-native-physics-demos]: Keep Dam Break Medium 16x12=192 and Normal gravity (0,-10) as the documented default; Small is 8x8=64 and Large is 20x14=280. — D-10 says evolve the existing basin. Plan 01 already locked Medium/Normal as the documented 192-particle world.
- [Phase 18-six-native-physics-demos]: Drop-obstacle raises the existing circle to (2.5, 7.2) and wakes it; reset-obstacle restores (2.5, 5.5); neither recreates the world. — D-05 and D-06 keep obstacle actions live. Recreating would discard the same body identity.
- [Phase 18-six-native-physics-demos]: Fountain defaults to Medium/Up/Medium, emits in on_advance with lifetime 3s and maximum_count 320, and treats a full system as a no-op. — DEMO-02 and Pitfall 2 require a plateau below the 512 frame cap. A full-system create must not poison the session.
- [Phase 18-six-native-physics-demos]: Two filled circles of radius 1.15 at (-1.0, 2.2) teal and (1.0, 2.2) red sit in contact so COLOR_MIXING can run. — Groups must overlap or sit in contact for the engine color-mixing pass; separate isolated blobs would never change captured color lanes.
- [Phase 18-six-native-physics-demos]: Mix-strength Off 0.0 / Gentle 0.25 / Strong 0.5 recreates; default Strong. Stir-speed Off/Slow/Fast is live; default Slow. — There is no live system-def setter for color_mixing_strength. Stir is a per-advance force and must not recreate the world.
- [Phase 18-six-native-physics-demos]: Replace the leftover Color Mixer stub assertion with a live-world create check; keep SceneUnimplemented for fail-closed vocabulary. — Full liquidfun-wasm lib tests still asserted Color Mixer was unimplemented after the scene constructed.
- [Phase 18]: Construction CTA is Apply setting, not the single word Apply. — UI-SPEC FLAG overrides the one-word Apply so the button names the thing being applied.
- [Phase 18]: Extract control/credit helpers to .ts so vitest node can test them without Solid JSX. — Vitest include is tests/**/*.test.ts and fails Solid JSX import analysis.
- [Phase 18]: Reset keeps last applied construction presets and startScene/abandonScene dispose the prior owner. — D-08 one-session teardown plus UI-SPEC Reset that restores documented initial state plus last construction presets.
- [Phase 18]: Local Chromium proofs are the Phase 18 gate; a new Pages URL is not required. — D-16 requires catalog and hash open, reset, credits, and dispose-on-switch locally. Phase 19 owns WEBTEST-01 and any later Pages polish.
- [Phase 18]: Independent AI review acknowledges digest 22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c; the implementing executor does not approve its own work. — D-17 and the 2026-09-16 owner policy require a separate identified AI reviewer bound to an exact digest. Passing just web-player-smoke is not the acknowledgment.
- [Phase 18]: Reset proofs wait for data-step-index greater than 4 so restart is distinguishable from the first presented frame. — presentOwnedFrame advances one step before Playing, so a reset from step 1 cannot prove a restarted series.
- [Phase 19]: Invert projectPoint with WORLD_BOUNDS on the shared Camera; never use canvas.width or devicePixelRatio.
- [Phase 19]: cssPointFromClient returns CSS pixels only and rejects non-finite samples and empty rects.
- [Phase 19]: reducePointerEvent stores one maybePointerId, ignores uncaptured moves, and clears on up, cancel, and lostpointercapture without reading isTrusted.
- [Phase 19]: Parse down|move|up|cancel and finite f32 values in SessionCore before any World mutation.
- [Phase 19]: Color Mixer, Jelly Drop, and Water Wheel compile with no-op apply_pointer until 19-03.
- [Phase 19]: Dam Break captured drag clamps to the basin; Fountain aims from the nozzle; Float or Sink drops at clamped x and y=6.0.
- [Phase 19]: Localized stir and poke loop per-particle engine methods; never pass scattered nearby ids to range APIs.
- [Phase 19]: Color Mixer Up/Cancel clear maybe_pointer so leftover tangent force cannot persist.
- [Phase 19]: Water Wheel pointer steers jet direction from JET_POSITION; Cancel restores default velocity; motor stays off.
- [Phase 19]: TypeScript forwards only allowlisted finite pointer samples and poisons only on generated failure. — Rejected kinds and NaN must not dispose a live session; only generated throws poison.
- [Phase 19]: One canvas Pointer Events adapter captures, unprojects through the live camera, and cancels on every teardown path. — Device pixels never enter physics; leftover capture cannot survive pause, reset, switch, fail, hidden, or cleanup.
- [Phase 19]: touch-action: none stays on canvas only so page chrome keeps native scroll. — D-06 forbids making the document a touch trap; preventDefault is canvas-only after capture.
- [Phase 19]: Render locked two-sentence interactionHint as figcaption text, never innerHTML or a third Space-to-pause sentence.
- [Phase 19]: Add select:focus-visible to the existing 2px / 4px #39D3C7 rule instead of a new focus system.
- [Phase 19]: Keep App.tsx under the 628-line cap by inlining the ready-scene helper when passing the hint.
- [Phase 19]: Keep a single chromium Playwright project and assert DOM attributes, not PNG hashes.
- [Phase 19]: Extract player-helpers.ts so player.spec.ts stays under the 400-line split trigger.
- [Phase 19]: Scroll the canvas into view before pointer gestures so the catalog cannot intercept the hit.
- [Phase 19]: Ordinary non-force push of d3d8688 triggered Pages run 35415816988; live origin is https://bright-builds-llc.github.io/liquidfun-rs/.
- [Phase 19]: Independent AI review acknowledges digest a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef; the implementing executor does not approve its own work.
- [Phase 19]: No crate/npm publish and no release tag.
- [Phase 20]: sceneControlsIdentity interpolates sceneId:generation so Solid Show stays truthy at generation 0. — A raw generation number is falsy at 0 and would unmount SceneControls.
- [Phase 20]: An empty construction bag yields no constructionEntriesForScene rows so Reset can rebuild WASM build([]) defaults. — D-08 requires construction selects to return to documented initials after Reset.
- [Phase 20]: initialPresetValue tests load real catalog presets from maybeSceneById instead of duplicating option tables. — Catalog option ids stay the single source of truth for Reset-honest labels.
- [Phase 20]: Restore f054135 ScenePreview geometries with a classless SVG so .demo-nav-preview owns the compact frame. — Compact demo-nav CSS must style the SVG; catalog-preview was the deleted card class.
- [Phase 20]: Keep Static preview caption in DemoNavigation, not inside ScenePreview, so the helper stays illustration-only. — D-03 caption is list-item copy; dual mounts must not introduce SVG ids.
- [Phase 20]: Add only .demo-nav-preview* rules; do not restore .catalog-card or the three-column card grid. — D-04 keeps the Kobalte shell; card grid broke narrow widths.
- [Phase 20]: Start resetGeneration at 1 and pass sceneControlsIdentity into Show keyed; never pass a raw generation number. — A raw generation number is falsy at 0 and would unmount SceneControls.
- [Phase 20]: recreateScene assigns constructionValues = {} then bumps resetGeneration then startScene. — D-08 requires construction selects to return to documented initials after Reset.
- [Phase 20]: Keep SceneCredits outside the keyed Show so credits do not remount with selects. — Only PresetControl pendingValue needs a remount; credits have no snapshot state.
- [Phase 20]: Scope desktop Static preview counts to .demo-sidebar so dual DemoNavigation cannot count 12.
- [Phase 20]: Keep mobile preview asserts on getByRole dialog named Demos, not .demo-sidebar.
- [Phase 20]: Pause playing scenes before Tab-heavy drawer and 375px checks so compact previews cannot starve keyboard smoke.
- [Phase 20]: resetNearZero watches the first restarted data-step-index instead of sampling after catch-up frames. — After 20-03 keyed remount, Playwright sampled step 29 after an actually-reset world. Observe the first restarted index so 4-step catch-up cannot hide Reset.
- [Phase 20]: Construction Reset tests use SIX_SCENE_TIMEOUT_MS because Apply large then Reset rebuilds two WASM worlds. — Dam Break Large then documented-initial rebuilds exceeded the 30s Playwright default while chrome already showed Medium.
- [Phase 20]: The 240-frame demo-media clock test uses a 120s timeout so just web-player-smoke can finish. — The numbered-frame capture is on the test:player allowlist and cannot complete in 30s even serially.
- [Phase 20]: Independent AI review acknowledges digest 7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f; the implementing executor does not approve its own work. — D-11 and the 2026-09-16 owner policy require a separate identified AI reviewer bound to an exact digest. Passing just web-player-smoke is not the acknowledgment.
- [Phase 20]: Digest is SHA-256 of concatenated listed file bytes in 20-06-PLAN.md order; passing just web-player-smoke is not the acknowledgment. — Record the exact cat | shasum -a 256 command so a later reviewer can recompute the same 64-hex digest.
- [Phase 20]: No crate/npm publish, no release tag, and no new GitHub Pages URL. — D-11 local Chromium smoke is sufficient; publication remains separately unauthorized.
- [Phase 21]: Delete unused loadProofSession with no compatibility alias; keep generated ProofSession. — D-01. App already constructs worlds through loadSceneSession; generated ProofSession is not renamed.
- [Phase 21]: Leave rust-wasm-proof.spec.ts out of test:player; do not fold Dispose-session or PNG-hash into just web-player-smoke. — D-02. Product Chromium gate stays the four-file test:player allowlist.
- [Phase 21]: Document just web-smoke as historical Phase 16 forensic chrome, not the v1.1 product gate. — D-03. Forensic smoke is not expected to pass against current Play/Pause/Reset chrome.

### Roadmap Evolution

- Phase 20 added: Playground catalog previews and Reset honesty (v1.1 audit gap closure for WEB-01 and WEB-03)
- Phase 21 added: Playground leftover cleanup (unused proof helper, dead FallbackPanel branches, scene file-lengths)

### Pending Todos

No new milestone todos captured. Plan Phase 20 next.

### Blockers/Concerns

- Chromium closure attempt 10 is current for source `80d4d7b`: all 16 source-bound checks pass, strengthened validators and bounded failure records are exercised, and separate AI review approves digest `7b63ca2e7f1580a8a509e8265aca968667bc8443b9055651e285991a0cd3ff38`. Attempt 8 remains historical.
- Pages setup/access and final URL remain unverified; Phase 17 verifies deployment and project-subpath assets early.
- Floating, elastic and wheel scene stability require modest visual experiments in Phase 18; no fake physics or silently replaced approved scenes.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260919-eut | Split oversized WASM scene tests to satisfy Bright Builds CI | 2026-09-19 | 888b5e8 | [260919-eut-split-oversized-wasm-scene-tests-to-sati](./quick/260919-eut-split-oversized-wasm-scene-tests-to-sati/) |
| 260919-jco | Final responsive shell review fixes | 2026-09-19 | 680b191 | [260919-jco-final-whole-change-responsive-playground](./quick/260919-jco-final-whole-change-responsive-playground/) |
| 260919-wbn | Raise webapp particle frame cap 20x to 10240 and increase scene particle counts about 10x, packing finer so world volumes stay similar | 2026-09-20 | 3a047bf | [260919-wbn-raise-webapp-particle-frame-cap-20x-to-1](./quick/260919-wbn-raise-webapp-particle-frame-cap-20x-to-1/) |

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag. The v1.0 label identifies planning history only.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements. Strict certification remains not release-ready.
- Before eventual publication, verify the declared Rust 1.92 minimum and obtain separate publication authority. Current GUI behavior was not verified by the hobby wrap-up.
- Historical decisions and completion metrics: `.planning/milestones/v1.0-STATE.md` and `.planning/MILESTONES.md`.
- Three historical debug records remain preserved; Phase 15 completion classifies their later evidence without claiming new fixes.

## Session Continuity

Last session: 2026-09-20T20:16:12.682Z
Stopped at: Completed 21-01-PLAN.md
Resume file: None
