# liquidfun-rs

## What This Is

`liquidfun-rs` is a fun, experimental, open-source Rust implementation of Google's LiquidFun physics engine for learning, games, simulations, and visualization. It develops useful native Rust behavior incrementally against a pinned upstream reference; complete parity and production certification are optional ambitions.

Visitors can also explore twenty-five native scenes and play six place-then-play cup puzzles in a SolidJS GitHub Pages playground compiled from this engine to WebAssembly. The repository retains upstream C++ LiquidFun as a read-only development oracle for research, differential testing, reference data, and benchmark comparison. Ordinary users of the published Rust library must not need the upstream source, a C++ compiler, Bazel, or any cross-language runtime component.

## Core Value

Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

## Current scope decision — 2026-09-16

The owner prioritizes a fun hobby project over production-quality certification. Apply `PROJECT-SCOPE.md`: mandatory Linux x64 qualification and the dedicated performance-host completion gate are removed. Historical evidence remains truthful and source-bound; strict qualification is opt-in. The owner selected local checks plus one macOS CI job and optional manual expensive suites. The accepted experimental preparation checklist is in RELEASE.md. APIs may evolve with incompatible changes documented before release; durable API/MSRV guarantees are deferred. Preserve Rust 1.97.0 and the declared minimum 1.92, verifying that minimum before publication or deliberately revising manifest and docs together.

## Current Milestone: v1.6 Physics Curiosity Scenes

**Goal:** Add six small, self-contained playground scenes of physics people like to share and wonder about, each recognizable at a glance on desktop and phone (SEED-004).

**Target features:**

- Siphon: water climbs over a crest and drains to the lower tank until the levels equalize
- Archimedes screw: a rotating helix lifts material up a slope
- Sand hourglass: powder pours through a neck and piles at an angle of repose; flip to restart
- Cartesian diver: pressing the bottle sinks the diver, releasing lets it rise
- Rayleigh–Taylor instability: heavy liquid fingers down through light liquid
- Fake ferrofluid: dark liquid spikes toward the pointer

The Phase 45 feasibility spikes dropped the siphon and the sand hourglass (owner confirmed 2026-10-14). They returned to SEED-008 and SEED-007, so v1.6 now targets the other four scenes.

**Key context:** Every new scene follows the repository's catalog rules (catalog entry, portrait frame, README SVG/WebP plan, shared scale legend, HUD safe frame). The 25 existing scene fingerprints must stay identical; any new engine or bridge feature (for example per-group density for Rayleigh–Taylor or a pointer force for the ferrofluid) must leave existing scenes unchanged. The riskiest scenes go last so they can slip. Phase numbering continues at 45. No crate publication.

## Current State

v1.5 Puzzle Mode is archived (2026-10-14): 8 phases (37–44), 64 plans and 112 tasks. Annotated tag `v1.5` marks the archive; no crate was published. The playground gained a place-then-play puzzle game: six teaching levels at `#/puzzle/<id>` with a tool tray, a timed valve, Win/Brim/Thrifty stars, a `#/puzzles` level grid with per-browser progress, opt-in synthesized sound and a GPU Water render mode. Fill, win, timeout and stars are decided per fixed step in the private `liquidfun-wasm` bridge, and every level's recorded solutions replay identically in native release, wasm and the browser. `crates/liquidfun` was never modified and all 25 scene fingerprints are unchanged. The milestone audit passed (33/33 requirements, 23/23 cross-phase links, 6/6 flows); owner-deferred tech debt is listed in `milestones/v1.5-MILESTONE-AUDIT.md`.

v1.6 Physics Curiosity Scenes has finished its scene phases. Phase 45 (groundwork and feasibility spikes), Phase 46 (Cartesian Diver), Phase 47 (Archimedes Screw), Phase 48 (Fake Ferrofluid) and Phase 49 (Rayleigh–Taylor Instability) are complete. Phases 50 and 51 were dropped by the Phase 45 spikes. The protected fingerprint set now holds 29 scenes, and the milestone is ready for its audit.

v1.4 Scenario Performance is archived (2026-10-11): 3 phases, 13 plans and 27 tasks, plus 4 follow-up quick tasks. Annotated tag `v1.4` marks the archive; no crate was published. Seven profile-guided engine fixes were kept, each with bit-identical end-state fingerprints on all 25 scenes. A back-to-back before/after survey on one machine found 24 of 25 catalog scenes faster, 1 within noise and none slower (`docs/benchmarks/scene-survey.md`; local observations, not public speed claims). Leftover ideas are recorded in `milestones/v1.4-REQUIREMENTS.md`.

<details>
<summary>v1.5 phase-by-phase completion notes</summary>

Phase 37 is complete (2026-10-12). The private `liquidfun-wasm` bridge now builds the `first-pour` level from a level id and integer placements, and decides cup fill, win (a 30-step settle window) and timeout per fixed step in Rust. Recorded fixtures replay to the same outcome step in native release and wasm. Native debug builds drift from release, so their checks allow a small outcome-step tolerance and a +5% fill margin. All 25 scene fingerprints are unchanged and `crates/liquidfun` was not modified. Browser (Playwright) replay is left for Phase 40.

Phase 38 is complete (2026-10-12). `#/puzzle/first-pour` opens in the existing player on the main-thread backend. It has a glass cup with an etched target line and star ticks, plus fixed-slot HUD leaves for the fill meter, the simulation-time countdown, Play, and a result card with Next, Retry and Edit. Unknown level ids show a fallback. Scene interactions and chrome are off on puzzle routes, and the camera frame and scale stay fixed from building through running to the result. Levels use their own route parser and bounds registry, so scene tests, the sidebar and README plans are untouched. All 25 scene fingerprints are unchanged.

Phase 39 is complete (2026-10-13). Players solve `first-pour` by placing tools from a per-level tray (2 planks, par 1). They tap to place, drag to move, rotate in 15° steps and remove tools with mouse, touch or keyboard, then press Play. A single Rust validator, run through a stateless wasm inspector and again by `newPuzzle`, marks overlapping or out-of-area tools red and disables Play. Stop returns to building with the tools kept, and Clear removes them. Rust tracks the peak held fill per step. A winning run is decided at the Brim latch or the step limit, and earns ★ Win, ★ Brim (90%) and ★ Thrifty (≤ par tools). The best result per star is kept for the page session. Fast-forward at 2× and 4× reaches the same outcome as 1×. All 25 scene fingerprints are unchanged and `crates/liquidfun` was not modified. Known for Phase 40: `full_count` 900 is below the real cup capacity, so almost any win also earns Brim.

Phase 40 is complete (2026-10-13). A timed valve tool opens at a player-set simulation time (0.5 s steps, set with a stepper or `[`/`]`), so the same time always gives the same outcome. Six teaching levels ship with one-line captions: `first-pour` (plain water), `thick-syrup` (viscous), `jelly-pad` (an elastic pad that never counts), `float-up` (buoyancy), `timed-gate` (valve timing) and `grand-finale` (jelly, valve and syrup). The powder level was dropped after a spike found no powder behavior distinct enough to teach. Every level has recorded fixtures: a three-star reference that wins with margin, an empty run that loses, a release-lane nudge sweep, and asserted controls proving its idea matters. All 14 fixtures replay identically in native release and wasm, and the timed-gate reference replays in the browser. `first-pour` shows a one-time dismissible placement hint. New levels fill at least half a phone's height; `first-pour` keeps its locked 34% frame. Each level stays under 3,000 particles. All 25 scene fingerprints are unchanged and `crates/liquidfun` was not modified.

Phase 41 is complete (2026-10-13). A text-only Puzzles link in the sidebar opens `#/puzzles`, a level grid built from TS metadata and saved progress, with no wasm load. Each card shows the level's number, title, idea, three labelled stars (Win, Brim, Thrifty), a best-fill line once solved, and a lock state. Progress lives under one versioned localStorage key (`liquidfun.puzzle-progress.v1`). Stars only ever merge upward, including against other tabs, because every write re-reads the stored value first. Levels unlock in order, and a solved level always stays unlocked. Deep links open any level, and on the last level the result card's Next slot becomes All levels. Degraded mode turns on for a blocked or throwing storage, a failed probe or write, corrupt or wrong-shape JSON, or a future version. It unlocks every level, keeps play working in memory, shows a note in the grid and the Puzzle options sheet, and never writes, so stored data is never destroyed. Each level reopens with its last placements. Those are discarded when the level's Rust FNV-1a 64 fingerprint (`levelHash`, pinned in `level-hashes.txt` and checked in both native and wasm builds) no longer matches. All 25 scene fingerprints are unchanged and `crates/liquidfun` was not modified.

Phase 42 is complete (2026-10-13). Puzzle routes have opt-in synthesized Web Audio sound: splashes, impacts, plinks that rise along a pentatonic scale as the cup fills, and a three-note win chime. It uses at most 6 voices at once, with rate limits per sound and a compressor. Sound is off by default. A HUD speaker button and a mirrored "Sound effects" checkbox in the Puzzle options drawer turn it on, and the choice is stored under `liquidfun.puzzle-sound.v1`. No audio context exists until the player opts in and makes a trusted gesture. Sound suspends when the tab is hidden or the player leaves the puzzle, and an audio failure turns it off for the session without failing the run. The drawer hint reads "No sound on iPhone? Check the silent switch." The bridge reports three read-only per-step values on puzzle frames: new splash contacts, rigid impacts that began, and the strongest of their impulses. They come from data the step already produces. Removing the tally leaves particle positions bit-identical, and sound on or off gives the same puzzle outcome. All 25 scene fingerprints are unchanged and `crates/liquidfun` was not modified.

Phase 43 is complete (2026-10-14), closing the v1.5 phases. A "Water" particle render mode in both scene and puzzle pickers draws particles with WebGL2 at high precision: refraction of a walls-only backdrop, highlights, a tint that keeps each particle's color, and white foam where particles move fast. The cup, meter and tools draw above it, and Shaded blob stays the default. Every frame tries Water, then Shaded blob, then the canvas, and a lost WebGL context falls back and repaints on restore; devices that cannot run it show "Water (unavailable)". Shaded blob now also accepts half-float render targets. The bridge produces a per-particle speed lane only when a capture asks for it, which happens only while Water is selected and available; capturing it leaves the simulation bit-identical, puzzle outcomes match in every mode, and exports keep their styles. All 25 scene fingerprints are unchanged and `crates/liquidfun` was not modified.

Phase 44 is complete (2026-10-14), closing tech debt the owner chose from the v1.5 milestone audit. On portrait phones, two-kind tool trays stack each count under the full tool name, so "Plank" and "Valve" are no longer cut off at 390 or 360 px. The valve opening time is now the stepper group's accessible description while a valve is selected. The TypeScript valve tick is capped at 255 like the Rust lane, and the test seam defaults a valve's open tick. New e2e tests click Next from a won level and run timed-gate at 1×, 2× and 4× with identical outcomes. `bun run typecheck` now also typechecks `playwright.config.ts` and all e2e code, with 16 errors fixed at the root. `crates/` was not modified, so the 25 scene fingerprints still hold.

</details>

Milestone v1.4 Scenario Performance was initialized on 2026-10-03 and simplified on 2026-10-06 to three phases: a timing survey, targeted fixes for the slowest scenes, and a final re-survey. Existing behavior, settings and visuals stay unchanged, and measured memory increases are allowed. Phase 34 is complete (2026-10-07): `just playground-scene-spot` times all 25 catalog scenes natively (median of 3 fresh runs) and ranks them. `docs/benchmarks/scene-survey.md` records the first table from commit `96a3ac6a6`. Liquid Tumbler leads at about 24.4 ms/step, ahead of Tesla Valve, Stacked Drip, Washing Machine and Particles. These are local observations, not public speed claims. Phase 35 is complete (2026-10-10): profile-guided engine fixes, each kept only with a before/after gain beyond noise and bit-identical end-state fingerprints on all 25 scenes, made the five slowest scenes faster on this machine (Washing Machine about −32%, Tesla Valve −15 to −20%, Stacked Drip −6 to −10%, Liquid Tumbler −6 to −8%, Particles −5 to −7%). Reverted attempts and profiles are recorded in the phase's 35-PROFILES.md. Later quick tasks amended the keep rule with a calibrated 4% noise floor and kept two more fixes (chain-edge hoist and a bounded insertion sort). Phase 36 is complete (2026-10-11): a back-to-back before/after survey of all 25 catalog scenes on one machine shows 24 faster and 1 within noise, with none slower and identical fingerprints, published in `docs/benchmarks/scene-survey.md`. Leftover ideas are recorded under REQUIREMENTS.md future work, and v1.4 is ready to archive.

v1.3 Reference Testbed Scenes is archived (2026-09-28): 8 phases, 38 plans, and 76 tasks. At archive time the playground listed twenty-three ready scenes. That was the twelve missing JavaScript testbed ports, the six earlier scenes, Liquid Tumbler, and four original scenes: Hydraulic Fountain, Wave Tank, Liquid Bubbler, and Stacked Drip. Credits for the ports cite the pinned LiquidFun tests. `MAX_ADVANCE_STEPS` stays 4. The audit passed 15/15 requirements. This is not sealed C++ parity or crate publication. Annotated tag `v1.3` marks the planning archive. See `.planning/MILESTONES.md` and `.planning/milestones/v1.3-REQUIREMENTS.md`.

v1.2 Native Performance Closing is archived (2026-09-21): 4 phases, 23 plans, and 47 tasks. Unprofiled Dam Break Medium on one macOS host recorded `rust_over_cpp_ratio` `2.956857456935513` at stamp `2026-09-21T20-38-50Z` (git `89d3456406c3c79ed500192bca9491c707033647`). `reviewed_reports` stays empty. README and crates.io have no universal “Rust is N×” claim. `just web-player-smoke` recorded 37 Chromium tests, compared with native Rust only. This is not crate publication and not a git release tag. Commit `77fbd84` (zero the force buffer after SolveForce) is after that stamp, so the ratio is not a measurement of later HEAD. See `.planning/MILESTONES.md` and `.planning/milestones/v1.2-REQUIREMENTS.md`.

The v1.1 Web Playground planning milestone is archived (2026-09-20): 6 phases and 39 plans completed. Visitors can run Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel at `https://bright-builds-llc.github.io/liquidfun-rs/` with in-app SVG previews, play/pause/reset, honest Reset labels, pointer interaction, and automatic Pages delivery. This is not a crate or npm release. See `.planning/MILESTONES.md` and `.planning/milestones/v1.1-REQUIREMENTS.md`.

The v1.0 Experimental Foundation planning milestone remains archived (2026-09-17): 16 phases and 252 active plans under hobby scope. Native checks, isolated packaging and macOS CI were verified at `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`.

<details>
<summary>v1.3 phase-by-phase completion notes</summary>

Phase 26: Particles and Liquid Timer as watch-first basin ports. The original six scenes stay.

Phase 27: Surface Tension, Elastic Particles, and Rigid Particles as flag-group showcases.

Phase 28: Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen with native destroy-in-shape, group shove, and live revolute motors.

Phase 29: Sparky sparks and Drawing Particles paint. All twelve missing testbed scenes are in the catalog.

Phase 30: Hydraulic Fountain. A timed piston moves one water group through a throat. Fountain stays the aimed emitter.

Phase 31: Wave Tank. A sine-driven end platform sends a wave to the far wall. Wave Machine stays the rocking tank.

Phase 32: Liquid Bubbler. Amber liquid turns a motor-off wheel, and a side shaft returns an original particle. Water Wheel stays.

Phase 33: Stacked Drip. Teal liquid tips three motor-off trays, and a side shaft returns an original particle. Liquid Timer and Liquid Bubbler stay.

</details>

<details>
<summary>v1.2 phase-by-phase completion notes</summary>

Phase 22: exclusive unprofiled Dam Break stamps, fail-closed samply 0.13.1 profiles, and separate parent-phase timers. Independent review digest `52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06`.

Phase 23: `docs/native-performance-audit.md` names `particle_rows` from live pair and samply at `6d98531ac799987c209d3fd1e572e482fcab5da6` (unprofiled ratio `327.53395024734476`) and records a private dhat dump. Independent review digest `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`.

Phase 24: shared scalar kernels until gate stamp `2026-09-21T20-38-50Z` / ratio `2.956857456935513`; five-scene spot-check; same-cluster note; empty `reviewed_reports`. Independent review digest `c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d`.

Phase 25: remaining-delta notes and `just web-player-smoke` (37 tests). Optional step-time note cites native `ms_per_step` `1.053103` and is never versus `oracle-release`.

</details>

<details>
<summary>v1.1 phase-by-phase completion notes</summary>

Phase 16: private WASM wrapper, five copied frame lanes, Chromium proof, independent review, 20/20 closed threats.

Phase 17: shared SolidJS player, Dam Break on Pages, OIDC delivery. Recorded deploy SHA `50a15562b356ed941266eedddc636df3f76e7e7e`.

Phase 18: six native scenes with labeled controls and host-locked credits.

Phase 19: Pointer Events gestures, keyboard/narrow-width polish, Chromium plus live Pages at source `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`.

Phase 20: Kobalte catalog SVG previews and Reset remount to `DEFAULT_PRESET_VALUES`. Chromium `just web-player-smoke` 37 tests.

Phase 21: removed unused `loadProofSession`, unknown-only `FallbackPanel`, named scenes under 628 lines, `upstream_cli` split. Independent review digest `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`.

</details>

## Next Milestone

v1.6 Physics Curiosity Scenes is active (see Current Milestone). It promotes SEED-004. The remaining dormant seeds in `.planning/seeds/` are SEED-002 sandbox editor with shareable links, SEED-006 the v1.4 leftovers, SEED-007 sand hourglass and SEED-008 siphon (both returned by the Phase 45 feasibility spikes, owner confirmed 2026-10-14). v1.5 future ideas (HINT-01, BADGE-01, TRAIL-01, ROTATE-01, GOAL-05, VALVE-02, SPONGE-01, LEVEL-05, EDITOR-01) and owner-deferred v1.5 tech debt are recorded in `milestones/v1.5-REQUIREMENTS.md` and `milestones/v1.5-MILESTONE-AUDIT.md`.

Still deferred: DRAW-02's unrelated material-matrix expansion, PRESET-01 product preset additions, PARITY-01 sealed per-scene differential certification, BOX2D-01 upstream-only suites, public universal speed claims and package publication. Default unsafe, nondeterministic parallel or approximate fast modes are not authorized.

## Requirements

### Active

v1.6 Physics Curiosity Scenes requirements are defined in `.planning/REQUIREMENTS.md`.

### Validated

- [x] Phase 34 completed PERF-07: `just playground-scene-spot` ranks all 25 catalog scenes by native median ms/step, and a test fails when a catalog scene is missing. The first table is in `docs/benchmarks/scene-survey.md`. Local observations only. Validated in Phase 34: Scene Timing Survey.
- [x] Phase 35 completed PERF-08 and PERF-09: profile-guided fixes for the five slowest scenes, each kept only with an ABBA before/after gain beyond noise; all 25 scene fingerprints unchanged; Rust workspace and web unit tests pass. Local observations only. Validated in Phase 35: Speed Up the Slowest Scenes.
- [x] Phase 38 completed GOAL-01, GOAL-03 and PUZ-02..PUZ-06: puzzle route `#/puzzle/<id>` with a glass cup, fill meter, step timer and result card in fixed HUD slots, scene interactions off, a fixed camera across puzzle phases, and a separate puzzle bounds registry; 25/25 scene fingerprints unchanged; `just web-player-smoke` and 16 puzzle Chromium tests pass. The glass look and the meter placement were approved by the owner. Validated in Phase 38: Puzzle Route and Player Shell.
- [x] Phase 39 completed TOOL-01..TOOL-04, TOOL-06, TOOL-07, STAR-01 and STAR-02: place-then-play tool tray and editor (mouse, touch, keyboard), one Rust placement validator shared by the live inspector and `newPuzzle`, Stop/Clear, Win/Brim/Thrifty stars with a session best-of store, and 1×/2×/4× fast-forward with identical outcomes. 25/25 scene fingerprints unchanged; 29 puzzle Playwright tests (desktop and touch) pass. Verified by an independent AI verifier. Validated in Phase 39: Place-then-Play Tools and Stars.
- [x] Phase 40 completed TOOL-05 and LEVEL-01..LEVEL-04: timed valve tool with a building-time open-time stepper, six teaching levels with captions (powder dropped after a spike), per-level solvability proofs (fixtures, nudge sweeps, idea controls), exact native/wasm replay and one browser replay, a one-time first-level hint, and portrait frames and particle budgets per level. 25/25 scene fingerprints unchanged; 57 puzzle Playwright tests pass. Verified by an independent AI verifier. Validated in Phase 40: Timed Valve and Teaching Levels.
- [x] Phase 41 completed PUZ-01 and PROG-01..PROG-04: `#/puzzles` level grid with a sidebar link, persisted stars that only merge upward (including across tabs), soft linear unlocks with open deep links, degraded mode for unavailable storage that never writes, and restored placements guarded by a pinned Rust level hash. 25/25 scene fingerprints unchanged; 81 puzzle Playwright tests and 1543 unit tests pass. Verified by an independent AI verifier. Validated in Phase 41: Level Select and Progress.
- [x] Phase 45 completed PREP-01..PREP-03: a committed, pinned baseline of the 25 scene fingerprints (`docs/benchmarks/scene-fingerprints.json`) and a tested id-join gate (`just scene-fingerprint-gate`) that accepts new ids; scene counts derived from `ALL_SCENE_IDS` and `SCENE_IDS` with number-free copy; the near-limit catalog files split into family modules; a per-scene checklist in `docs/adding-a-scene.md`; and measured siphon and hourglass spikes, both dropped to SEED-008 and SEED-007 with owner confirmation. 25/25 scene fingerprints unchanged and `crates/liquidfun` untouched. Verified by an independent AI verifier. Validated in Phase 45: Curiosity Groundwork and Feasibility Spikes.
- [x] Phase 46 completed DIVER-01..DIVER-03 and SCENE-01..SCENE-03: the `cartesian-diver` playground scene. Pressing and holding the bottle, or the Squeeze action, switches the diver to a heavier density so it sinks, and releasing switches it back so it rises. The bottle dent and the shrinking air bubble are drawn only, and the caption says the density switch stands in for the squeezed bubble. Release tests show sink and rise at gravity 2, 10 and 80; the gravity-80 rise is judged against the settled water column (D-07a). Long-press callouts and the context menu are suppressed on every scene canvas. The scene has a catalog record, preview, portrait frame and README preview, and uses 1,936 particles at 0.582 ms/step. 25/25 prior fingerprints unchanged, then the baseline grew to 26 in its own commit; `crates/liquidfun` untouched. Verified by an independent AI verifier. Validated in Phase 46: Cartesian Diver.
- [x] Phase 47 completed SCREW-01 and SCREW-02: the `archimedes-screw` playground scene. One kinematic vane train slides up a 30° tube and snaps back one pitch inside solid end housings, so slanted vanes carry separate pockets of water from the reservoir into a weir basin that overflows down a chute back to the reservoir. A live `Screw speed` HUD spinner (0–60 rpm, default 30) and a `Direction` preset run it forward or in reverse, and reverse scoops basin water back down. The caption says flat vanes stand in for a turning helix in a 2D side view. Release tests show 60 s of lifting at default and maximum speed with no leaks, evictions or dry reservoir, reverse draining the basin, speed 0 holding still, and stable runs at gravity 2 and 80. The scene has a catalog record, preview, portrait frame and README preview, and uses 1,729 particles at 0.688 ms/step. 26/26 prior fingerprints unchanged, then the baseline grew to 27 in its own commit; `crates/liquidfun` untouched. Verified by an independent AI verifier. Validated in Phase 47: Archimedes Screw.
- [x] Phase 48 completed FERRO-01..FERRO-03: the `fake-ferrofluid` playground scene. A graphite-violet `TENSILE | VISCOUS` puddle (`#8A84AD`) sits in a dish over a resting bar magnet and rises into a row of separate spikes. A scripted, gravity-aware pull applied once per fixed step lifts it: a polynomial spike-column field with a capped acceleration, drag, and a sideways pull that fades beyond the reach. Press and drag moves the magnet (capped 1.4 m above the dish), and the spikes follow it under, inside or above the puddle. Pointer Up or Cancel parks and switches it off, so the spikes collapse. A `Magnet` action toggles it, and a live `Magnet strength` HUD spinner runs 0–100 % (default 65). The caption calls it a scripted magnetic pull. A new shared `scene/particle_force.rs` helper converts per-particle accelerations to calibrated forces for Phase 49. Release tests show 4+ peaks by step 12, collapse within 30 steps, re-rise within 18 steps, and 60 s full-strength holds and drag paths with no escapes, evictions or energy growth at gravity 2, 10 and 80. 990 particles at 0.316 ms/step. 27/27 prior fingerprints unchanged, then the baseline grew to 28 in its own commit; `crates/liquidfun` untouched. Verified by an independent AI verifier. Validated in Phase 48: Fake Ferrofluid.
- [x] Phase 49 completed RT-01 and RT-02: the `rayleigh-taylor` playground scene. A heavy orange layer (`#FB923C`) rests on a light blue one (`#3B82F6`) in a closed 2.8 m by 1.6 m tank. Both are groups in one plain-water particle system (r 0.035, pressure strength 2.0, damping 0.2, 1,484 particles). After a 36-step settle and a 24-step ramp, extra weight of (ratio − 1)·|g| along the live gravity vector goes to the heavy group only, once per fixed step, through the Phase 48 `particle_force` helper. A seeded polynomial ripple grows into fingers. A recreating `Weight ratio` preset (1.2×, 1.5×, 2× default, 3×) reruns the scene, and Reset scene restores the defaults. The caption says the orange layer is given extra weight and claims no density. The first spike sweep was invalidated, because the tall column collapsed under the engine's pressure cap during settle. Amendment D-06a then measured from the settled interface and added a no-slump rule, with no deadline relaxed. Release tests show 3+ fingers by step 114 (deadline 240) and overturn by step 156 at 2× and by step 378 at 1.2×. They also show bit-identical replays at every ratio, extra weight that follows a tilted gravity vector, and stable 60 s and gravity 2 and 80 runs. 0.36 ms/step native. 28/28 prior fingerprints unchanged, then the baseline grew to 29 in its own commit; `crates/liquidfun` untouched. Verified by an independent AI verifier. Validated in Phase 49: Rayleigh–Taylor Instability.
- [x] Phase 44 closed owner-selected v1.5 audit tech debt (no new requirements): full tool names in phone trays, valve time exposed to assistive technology, TS valve tick capped at 255, Next-navigation and timed-gate 1×/2×/4× e2e, and a strict e2e typecheck in `bun run typecheck`. 10/10 phase gates pass on the final tree (104 puzzle, 13 Water and 64 HUD Playwright tests; 1844 unit tests). Verified by an independent AI verifier. Validated in Phase 44: v1.5 Polish and Test Gaps.
- [x] Phase 43 completed WATER-01..WATER-03: an opt-in WebGL2 "Water" render mode (refraction, highlights, color-keeping tint, speed-driven foam) in scenes and puzzles with overlays on top, a per-frame Water → Shaded blob → canvas fallback, context-loss recovery, half-float targets, and a speed lane produced only while Water is active. 25/25 scene fingerprints unchanged; 27/27 phase gates, 13 Water Playwright tests and 1840 unit tests pass. Verified by an independent AI verifier; the owner approved the iPhone and real-GPU look checks. Validated in Phase 43: GPU Water Render Mode.
- [x] Phase 42 completed SOUND-01..SOUND-03: opt-in Web Audio puzzle sound (splash, impact, rising plink, win chime) with a 6-voice cap. It is off by default with a remembered toggle, creates no audio context before the player opts in with a gesture, suspends when the tab is hidden or the player leaves, and shows a silent-switch hint. The bridge's read-only per-step sound values leave the simulation unchanged. 25/25 scene fingerprints unchanged; 32/32 phase gates, 99 puzzle Playwright tests and 1725 unit tests pass. Verified by an independent AI verifier; the owner approved the listening and iPhone checks. Validated in Phase 42: Opt-in Sound.
- [x] Phase 36 completed PERF-10: whole-catalog before/after table for the final v1.4 source in `docs/benchmarks/scene-survey.md` (24 faster, 1 within noise, 0 slower; local observations only). Validated in Phase 36: Re-survey and Wrap Up.

- ✓ v1.5 Puzzle Mode — 33/33 requirements (GOAL, TOOL, STAR, LEVEL, PUZ, PROG, SOUND, WATER), 8 phases, 64 plans, audit passed 2026-10-14. Six deterministic cup puzzles with stars, level select and progress, opt-in sound and a GPU Water render mode; the engine and 25 scenes unchanged. Not a crate release.

- ✓ v1.4 Scenario Performance — 4/4 requirements (PERF-07 to PERF-10), 3 phases, 13 plans. A whole-catalog survey, seven bit-identical speed fixes and a before/after table with 24 of 25 scenes faster. Local observations; not a crate release.

- ✓ v1.3 Reference Testbed Scenes — 15/15 requirements, 8 phases, 38 plans, audit passed 2026-09-28. Twenty-three playground scenes, including the twelve missing testbed ports and four original scenes. Not a crate release.

- [x] Phase 29 completed PLAY-01, FX-01, and FX-02: Drawing Particles and Sparky follow Liquid Tumbler in the existing catalog; colliding circles throw fading powder sparks after the step; paint starts in an empty vessel and elastic clumps instead of flowing like water; the original six scenes stay. `just web-player-smoke` passed 44 Chromium tests. Recognizable ports, not sealed C++ parity. Validated in Phase 29: Sparky, Drawing, and full catalog.

- [x] Phase 26 completed PLAY-02, PLAY-03, BASIN-01, and BASIN-02: Particles and Liquid Timer play, pause, and reset in the existing catalog; credits cite the pinned LiquidFun tests; water falls in an open basin with a dropped ball; tensile, viscous liquid drains through shelves into bottom columns; the original six scenes stay. `just web-player-smoke` passed 38 Chromium tests. Recognizable ports, not sealed C++ parity. Validated in Phase 26: Catalog shell and basin scenes.

- [x] Phase 25 completed PERF-NOTES and PERF-WASM: honesty docs lead with gate stamp `2026-09-21T20-38-50Z` / `rust_over_cpp_ratio` `2.956857456935513` and label kernel-HEAD `2026-09-21T20-36-30Z` as a sibling; `just web-player-smoke` passed 37 Chromium tests; optional step-time note cites native `ms_per_step` `1.053103` with no prior WASM sample and never versus `oracle-release`. `reviewed_reports` stays empty. Not a public “Rust is N×” claim or crate publication. Validated in Phase 25: WASM sanity and honest close.

- [x] Phase 24 completed PERF-ADMIT, PERF-SHARED, PERF-GATE, PERF-BASELINE, PERF-SPOT, and PERF-CANARY2: shared scalar `particle_rows` / leftover kernels until unprofiled Dam Break Medium `pair.json` `rust_over_cpp_ratio` `2.956857456935513` (stamp `2026-09-21T20-38-50Z`); five-scene `just playground-scene-spot`; empty `reviewed_reports`; `unsafe_code = "forbid"`. Independent AI review digest `c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d` (`issues_found`, WR-01). Not crate publication or a Phase 12 public claim.

- [x] Phase 23 completed PERF-AUDIT and PERF-HEAP: committed `docs/native-performance-audit.md` names `particle_rows` (~93.6% self) from live pair+samply at MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`, unprofiled ratio `327.53395024734476`; copy-only audit-bundle; private dhat when allocator/`Vec` needles matched; timing doc refreshed as an unreviewed sample. Independent AI review digest `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`. This is not the 3× gate or crate publication.

- [x] Phase 22 completed PERF-PAIR, PERF-PROFILE, and PERF-TIMERS: `just playground-dam-break-bench` persists exclusive `target/dam-break-perf/<utc-stamp>/` `pair.json`/`pair.md`; `just playground-dam-break-profile` wraps samply 0.13.1 onto `[profile.profiling]` and writes `rust.json.gz` plus identity with `not_timing_authority`; `just playground-dam-break-timers` emits parent-phase timers on a sibling process. Missing samply fails closed. `liquidfun` stays bitflags-only. Independent AI review digest `52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06`. This is not the 3× gate, a named-function audit, or crate publication.

- ✓ v1.1 Web Playground — 22/22 requirements, 6 phases, 39 plans, audit passed 2026-09-20. Hosted playground at `https://bright-builds-llc.github.io/liquidfun-rs/`. Not a crate release.

- [x] Phase 21 leftover cleanup: deleted unused `loadProofSession`; kept `rust-wasm-proof.spec.ts` as opt-in historical `just web-smoke`; collapsed `FallbackPanel` to unknown-only Scene not found plus Open Dam Break; confirmed `color_mixer.rs` / `dam_break.rs` / `water_wheel.rs` under 628 lines; split `tools/xtask/tests/upstream_cli.rs` so Bright Builds `all` exits 0. Chromium `just web-player-smoke` passed 37 tests. Independent AI review approved digest `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`. This is not crate publication.
- [x] Phase 20 completed WEB-01 and WEB-03 gap closure: six in-app DemoNavigation entries show names, short descriptions, and compact static SVG `Static preview` illustrations without restoring the card grid; Reset remounts preset selects to `DEFAULT_PRESET_VALUES`; Chromium `just web-player-smoke` passed 37 tests including sidebar/drawer previews and Reset honesty. Independent AI review approved digest `7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`. This is not crate publication.
- [x] Phase 19 completed WEB-05, WEB-07, and WEBTEST-01: each of the six scenes has one documented canvas Pointer Events gesture through a shared CSS-bound camera unproject and WASM `pointer_action`, labeled keyboard controls remain, per-scene figcaption instructions and `select:focus-visible` are in place, Chromium `just web-player-smoke` covers pointer/cancel/resize/375px/hidden-tab, and live Pages at `https://bright-builds-llc.github.io/liquidfun-rs/` source `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e` serves WASM as `application/wasm`. Independent AI review approved digest `a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`. This is not crate publication.
- [x] Phase 18 completed WEB-01, WEB-04, WEB-08, and DEMO-01 through DEMO-06: six ready native scenes run in the shared player with static catalog cards, labeled bounded controls, explicit reset-on-change, and per-scene implementation/inspiration/notice links. Local Chromium `just web-player-smoke` proves open/reset/switch. Independent AI review approved digest `1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`. This is not WEB-05/WEB-07/WEBTEST-01 coverage or package publication.
- [x] Phase 17 completed WASM-04, WEB-02, WEB-03, WEB-06, and HOST-01 through HOST-03: Dam Break runs in the shared player on the live Pages project path, unknown scene hashes fall back usefully, playback/reset and loading/retry work, session teardown and hidden-tab catch-up stay bounded, and every main push builds WASM plus the site from the same checkout then deploys with OIDC. Recorded URL `https://bright-builds-llc.github.io/liquidfun-rs/`; source `50a15562b356ed941266eedddc636df3f76e7e7e`. Independent AI review approved digest `a962555da21e1da79b860c37b003d29fa7f3d6683ef16db1b2beac2a151bf893`. This is not six-scene WEB-01 coverage or package publication.
- [x] Phase 16 completed WASM-01 through WASM-03: generated Rust WebAssembly visibly advances a real particle/rigid scene in Chromium, the reproducible pinned SolidJS build preserves native Cargo/package isolation, and five bounded copied typed arrays cross the browser boundary without raw pointers or per-particle calls. Explicit disposal, exact-digest independent AI review, and 20/20 closed threats were verified against source-bound closure attempt 10.
- [x] Phase 15 hobby wrap-up completed HOBBY-01 through HOBBY-03: experimental guidance, a short preparation checklist, native/package verification and exact-source macOS CI. Checked implementation: `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`; later planning-record changes do not claim new runtime verification. Strict certification remains deferred and no package was published.

<!-- Shipped and confirmed valuable. -->

- [x] Phase 1 selected and pinned official LiquidFun commit `7f20402173fd143a3988c921bc384459c6a858f2`, with release ancestry, licensing, notices, source-mapping, and intentional update rules recorded.
- [x] Phase 1 established a Cargo-first resolver-3 workspace with one publishable `liquidfun` crate, a private `xtask`, a pinned development toolchain, a provisional Rust 1.92 MSRV, and package isolation from C++/reference inputs.
- [x] Phase 1 created an authoritative 177-row compatibility ledger, deterministic 161-entry discovery snapshot, generated human report, and fail-closed inventory/provenance/package checks.
- [x] Phase 2 established a bounded, versioned semantic protocol and private Rust/C++ differential harness with strict provenance, typed comparison and failure classification, replay/minimization, reviewed fixture promotion, and verified one-shot, reuse, and sanitizer empty-world round trips.
- [x] Phase 3 established world-scoped typed handles, checked invalidation and exhaustion, upstream-ordered owned destruction evidence, restricted hooks and deferred mutation, typed user associations, and a stable-particle dense-storage spike with clean review and 17/17 verified must-haves.
- [x] Phase 4 established the public `f32` math/settings foundation, checked matrix/transform/sweep contracts, a closed per-observable numerical policy, bit-faithful Rust/C++ probes, fail-closed build identity, typed mismatch evidence, and 25/25 verified must-haves.
- [x] Phase 5 established immutable validated shapes, source-ordered distance/manifold/tree/broad-phase/TOI kernels, pure pair/filter/refilter seams, and a fail-closed 78-family Rust/C++ collision probe with 35/35 verified must-haves; world-owned contact lifecycle remains Phase 6 work.
- [x] Phase 6 established the native body, fixture, sensor, and contact vertical slice with stable typed identity; atomic mass, proxy, mutation, and destruction behavior; source-ordered contact solving and deferred hooks; checkout-bound oracle provenance; and debug, release, replay, determinism, and sanitizer differential evidence with 77/77 verified must-haves.
- [x] Phase 11 accounted for all 388 upstream corpus rows, established one 43-scenario renderer-neutral catalog across headless, oracle, regression, benchmark, and private testbed consumers, preserved package isolation, and passed 94/94 must-haves including agent-controlled desktop comparison UAT.

- [x] Phase 13.1 restored structural compliance and closed its verification gaps with a complete 72-command local/canonical matrix, independently validated durable evidence, and formal verification at 36/36 with the accepted integration-history override.

- [x] Phase 14 repaired Windows particle-group scratch allocation, preserved typed rejection and complete rollback contracts, and passed 13/13 must-haves. Both fixed regressions and 957 package tests pass on Windows, Linux and macOS at one source SHA, with complete Cargo CI success; canonical candidate acceptance belongs to the deferred optional strict qualification campaign.

### Historical full-parity ambitions

These retained goals describe the original broad implementation ambition, not active Phase 15 acceptance. Current hobby completion follows PROJECT-SCOPE.md and the bounded experimental preparation checklist; broader parity and platform qualification remain optional.

- [ ] Extend the implemented minimal rigid-world slice with complete force and impulse application, configurable solvers, sleeping, continuous collision detection, world queries, and world ray casts from the selected LiquidFun revision.
- [ ] Implement all supported joints, standalone rope, remaining callbacks and listeners, diagnostic dump, debug drawing abstractions, and the broad rigid-body compatibility gate.
- [ ] Implement the full LiquidFun particle system, including storage, creation, destruction, lifetimes, buffers, spatial proxies, contacts, body contacts, groups, flags, pair/triad logic, queries, ray casts, callbacks, and every upstream solver behavior.
- [ ] Extend the Phase 2 C++/Rust reference harness beyond verified empty-world, math, collision, and minimal rigid-world probes to broad rigid-solver, joint, and particle scenarios with their semantic state, tolerance, regression, and diagnostic requirements.
- [ ] Create layered unit, integration, upstream-compatibility, differential, property, fuzz, Miri, sanitizer, and regression testing appropriate to each subsystem.
- [ ] Extend the Phase 4-6 numerical-stability, ordering, determinism, and platform-tier policy with broad rigid-solver, joint, and particle observables and tolerances before treating those later differential results as compatibility evidence.
- [ ] Measure performance against equivalent upstream C++ workloads and optimize only from profiling evidence without silently sacrificing API clarity, safety, determinism, or parity.
- [ ] Support Linux x86_64, Linux ARM64, macOS ARM64, macOS x86_64 where practical, and Windows x86_64, while investigating WASM, mobile, and realistic smaller `no_std` subsets.
- [ ] Provide discoverable repository automation through a root `justfile`, CI, pinned toolchains, dependency/license policy, documentation checks, coverage, benchmarks, and optional manual extended verification.
- [ ] Maintain accurate project, architecture, upstream, compatibility, testing, benchmarking, safety, contribution, release, and roadmap documentation without claiming unverified maturity or parity.

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Using the upstream C++ library as the runtime implementation — the production deliverable must be an independent Rust engine.
- Publishing a thin Rust binding layer as the port — FFI exists only for development-time reference, testing, and benchmarking workflows.
- Requiring C++, Bazel, the upstream submodule, or reference data for ordinary Cargo consumers — cross-language tooling is a repository-development concern.
- Mechanically translating the whole upstream repository in one pass — each subsystem must move through inventory, API design, minimal implementation, unit tests, upstream comparison, differential validation, optimization, documentation, and compatibility sign-off.
- Claiming full parity or production readiness before the traceability matrix has no unexplained gaps and the documented acceptance evidence exists — status reporting must remain precise.
- Treating an unrelated modern Box2D implementation as automatically compatible — reuse requires license review and behavioral validation against the selected LiquidFun ancestry.
- Coupling core simulation crates to a rendering or game-engine framework — visualization is optional and simulation remains headless.
- Introducing default parallel simulation that changes deterministic or upstream-compatible behavior — any such mode must be explicit and documented.
- Promising complete-engine `no_std`, embedded, iOS, Android, or WASM support before feasibility is established — smaller portable subsets may be pursued when evidence supports them.
- Selecting a final project license before upstream and derivative-work compatibility is reviewed — required notices and provenance come first.

## Hobby success standard

Useful simulations and examples work on the development machine, relevant local checks pass, and limitations are described honestly. Work can proceed without Linux x64 qualification or a dedicated performance runner.

## Optional full-parity success standard

The previous strict target remains available as an optional ambition. A future explicit claim of full feature parity requires all of the following:

- Every relevant public upstream feature is implemented or explicitly documented as irrelevant to the Rust library.
- Every upstream particle behavior, supported shape, supported joint, world operation, callback, and query has a Rust equivalent.
- Upstream examples have Rust equivalents, and upstream tests are ported, replaced, or explicitly accounted for.
- Differential tests cover representative and edge-case behavior using semantic state, reproducible inputs, and documented tolerances.
- Known behavioral, numerical, ordering, platform, API, and performance differences are documented.
- Performance is measured against upstream with comparable workloads and optimization levels.
- The production Rust library has no runtime dependency on C++ or the upstream source.
- The compatibility matrix contains no unexplained gaps.

## Historical delivery strategy

Development proceeds incrementally and in dependency order:

1. Establish repository foundations, toolchains, documentation, licensing analysis, CI, upstream pinning, and the build-orchestration decision.
1. Inventory upstream features, APIs, tests, examples, build systems, Box2D ancestry, and LiquidFun-specific changes.
1. Define the compatibility matrix, numerical policy, reference harness, and Rust ownership/object model.
1. Port foundational math, shapes, collision primitives, dynamic tree, and broad phase.
1. Port rigid-body dynamics, contacts, solvers, sleeping, continuous collision detection, queries, and ray casts.
1. Port joints, callbacks, filters, listeners, and debug draw abstractions.
1. Establish broad rigid-body differential validation.
1. Port particle storage, lifecycle, spatial structures, contacts, and body contacts.
1. Port particle solvers, flags, behaviors, groups, and pair/triad logic incrementally.
1. Establish comprehensive particle differential validation.
1. Port examples and testbed scenarios with headless support.
1. Optimize from benchmarks and profiling, then harden safety, platforms, documentation, and release policy.
1. Complete a parity audit before publication claims.

The ordering is a starting hypothesis. Research may refine phase boundaries, but it must preserve dependency-aware, testable progress and early risk retirement.

## Historical expected early deliverables

Before substantial physics porting begins, the project should have:

- A refined project definition, scoped requirements, executable roadmap, risk register, and milestone acceptance criteria.
- `ARCHITECTURE.md`, `UPSTREAM.md`, `COMPATIBILITY.md`, `TESTING.md`, and dependency/licensing analysis.
- Decision records for build orchestration, the Rust object model, and differential testing.
- A pinned upstream Git submodule and documented reference build/test commands.
- Initial Cargo workspace scaffolding, a root `justfile`, pinned Rust toolchain, and minimal CI for both the Rust skeleton and upstream reference implementation.
- A verified subsystem/API inventory and initial compatibility traceability matrix.

## Historical phase context

The snapshots below describe their original phases, not current work or next steps. Current State and Next Milestone Goals above supersede their unfinished-target wording. The completed hobby milestone does not claim full parity.

- Phase 1 is complete: the repository now contains a Cargo-first Rust scaffold, private orchestration, an immutable upstream oracle, reproducible CMake/Ninja build commands, compatibility/provenance records, package isolation, and separated CI workflows. Broad physics behavior is not implemented yet.
- Phase 2 is complete: the repository now has a strict semantic JSONL contract, native Rust and process-isolated C++ adapters, typed comparison and failure taxonomy, replay/minimization and evidence lifecycles, and verified empty-world one-shot, reuse, and sanitizer round trips. This proves the harness seam, not broad physics parity.
- Phase 3 is complete: the native crate now has typed world-scoped identities, explicit stale/cross-world/cross-system failures, checked arena retirement, upstream-ordered destruction cascades with pre-mutation snapshots, restricted step hooks, bounded deferred commands, panic poisoning, typed association side tables, and stable particle identity over transactional dense permutations. Broad solver behavior and the complete Phase 9 particle-buffer API remain deferred.
- Phase 4 is complete: consumers have documented source-ordered scalar, vector, matrix, rotation, transform, sweep, and fixed-setting APIs; a closed 25-path policy defines special values, comparison modes, horizons, ordering, and authority tiers; and supervised Rust/C++ probes verify 39 ordered cases in debug and release. Local AppleClang evidence remains non-promotable D2, while canonical D1 execution remains pinned CI evidence.
- Phase 5 is complete: immutable checked shapes, source-ordered collision kernels, dynamic-tree and broad-phase behavior, and TOI are implemented and covered by the fail-closed collision differential probe.
- Phase 6 is complete: native bodies, fixtures, sensors, contact lifecycle, the minimal contact solver, deferred hook mutation, and semantic rigid-world differential execution now work end to end. The next target is Phase 7's complete rigid solver, sleeping, world operations, and CCD; broad rigid parity remains unclaimed.
- Phase 11 is complete: every upstream corpus row has a terminal reviewed disposition; the shared renderer-neutral catalog drives headless execution, exact oracle comparison, regressions, benchmarks, and the private interactive testbed; the published crate remains renderer-free. Phase 12 owns performance, portability, coverage, dependency hardening, and release readiness.
- Google LiquidFun extends the Box2D 2.3.0 / revision-280 lineage. Official commit `7f20402173fd143a3988c921bc384459c6a858f2` is the immutable behavioral oracle; `UPSTREAM.md` and ADR 0001 record the release-to-candidate delta and maintenance state.
- The upstream C++ implementation is the behavioral oracle during development, not a production dependency or the desired public architecture.
- Particle simulation is a central deliverable, not an optional extension after rigid-body work.
- The project is necessarily long-running and may span multiple milestones; compatibility status must therefore be visible and evidence-based throughout development.
- Safe abstractions may alter layout, identity, ordering, and performance relative to C++. These differences need deliberate designs and differential evidence rather than incidental behavior.
- Callback reentrancy, mutation during stepping, stable object identity, destruction invalidation, intrusive-list replacement, external particle buffers, and user data are early architectural risks.
- Numerical divergence may arise from floating-point precision, compiler optimization, SIMD, collection ordering, solver order, and platform behavior. Exact bit parity is not assumed where unjustified.
- Existing permissively licensed Rust implementations may be considered only after provenance, license, ancestry, API, and behavioral compatibility review.
- Repository tooling should make advanced workflows discoverable without hiding their underlying commands or making ordinary Cargo use opaque.

## Constraints

- **v1.4 performance priority**: Prioritize measured simulation/rendering speed over memory compactness. Larger bounded caches, reusable buffers and precomputed structures are allowed. Existing behavior and fidelity are preserved; source settings are not reduced to manufacture performance gains. Track memory increases with the scope/availability of the measurement.
- **Implementation**: Production physics behavior must be native Rust — runtime delegation to upstream C++ is prohibited.
- **Reference isolation**: FFI and C++ builds are limited to differential testing, comparison, reference generation, benchmark comparison, and upstream test/example execution — published crates remain independent.
- **Build system**: Cargo is primary and sufficient for normal users — Bazel, CMake, or hybrid orchestration requires a documented evidence-based decision.
- **Upstream provenance**: The canonical source and exact revision must be pinned before implementation assumptions harden — moving branches are not acceptable references.
- **Licensing**: Upstream LiquidFun, Box2D, copied or translated code, tests, data, and all dependencies require explicit license review and attribution — final project licensing follows compatibility analysis.
- **Safety**: Safe Rust is the default — every `unsafe` block must be narrow, justified by a measurable need, document its invariant with a `SAFETY:` comment, and receive focused tests where practical.
- **API design**: APIs are experimental and may evolve; document incompatible changes before release. Preserve safe handles, checked mutation, explicit lifetimes/invalidation, and private storage details. Durable API guarantees are deferred.
- **Behavior**: Improve parity incrementally against the pinned upstream behavior; document known differences and retain regression protection. Exhaustive closure is an optional strict qualification goal.
- **Determinism**: Stable ordering and reproducible seeded scenarios take precedence over unproven parallel or SIMD gains — nondeterministic acceleration must be explicit.
- **Testing**: Meaningful semantic state must be compared — serialized raw memory alone is not an acceptable compatibility oracle.
- **Quality**: Production code avoids `unwrap()`, propagates errors, uses useful invariant messages for genuinely impossible states, documents public APIs, and follows the repository's Rust and Bright Builds guidance.
- **Angles and naming**: Full rotations use tau-based expressions, and optional internal values use `maybe_` naming where it improves clarity — project conventions remain consistent with repository standards.
- **Architecture**: Prefer cohesive deep modules and functional-core/imperative-shell separation — do not over-fragment crates or hide substantial foreign-language logic inside strings.
- **Rendering**: Simulation stays renderer-independent, optional, and headless — testbed framework choices cannot dictate core architecture.
- **Platforms**: One macOS Cargo CI job records tested coverage. Non-macOS platforms are best effort with optional manual testing; retain existing platform fixes and regressions.
- **CI cost**: Keep local checks and one macOS Cargo CI job. Cross-platform, differential, sanitizer, fuzz, coverage and benchmark suites are optional manual checks.
- **Transparency**: Documentation and README maturity claims must match verified implementation and compatibility evidence — incomplete parity is never marketed as complete.

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Build a genuine Rust implementation rather than production C++ bindings | Independence, safety, idiomatic APIs, Cargo usability, and long-term maintainability are core goals | Accepted in Phase 1 architecture and package boundaries |
| Use a pinned upstream C++ LiquidFun revision as the behavioral oracle | Compatibility needs a stable, inspectable target and reproducible evidence | Accepted in Phase 1: `7f20402173fd143a3988c921bc384459c6a858f2` |
| Restrict FFI and C++ tooling to development-time comparison workflows | Published Rust users must not inherit a C++ runtime or toolchain dependency | Accepted in Phase 1; package isolation verified |
| Keep Cargo primary and sufficient for ordinary use | Rust consumers and contributors need a conventional, transparent workflow | Accepted in Phase 1; `liquidfun` is the sole default member |
| Evaluate Bazel or hybrid orchestration before adoption | Cross-language and CI orchestration may help, but maintenance cost must be justified | Bazel deferred in Phase 1 absent measured need |
| Prefer safe Rust and encapsulate any necessary unsafe code | Safety must not be traded away without measurable benefit and explicit invariants | — Pending |
| Treat compatibility inventory and differential testing as first-class product work | Parity claims require traceable evidence, not implementation intuition | Inventory accepted in Phase 1; differential protocol begins in Phase 2 |
| Design the Rust ownership, handle, callback, and user-data model before broad porting | C++ pointer and mutation semantics are foundational and expensive to revise late | Accepted and verified in Phase 3; future subsystem APIs must preserve the documented identity, callback, event, mutation, and storage boundaries |
| Keep particle systems in core scope | Particle behavior is the defining LiquidFun extension and cannot be deferred as optional polish | — Pending |
| Keep rendering optional and simulation headless | Core portability, testing, server use, and framework independence depend on this boundary | Accepted and verified in Phase 11: the catalog/controller/checkpoint model is renderer-neutral, `liquidfun-testbed` is private and non-default, and package isolation rejects renderer dependencies from `liquidfun` |
| Prioritize correctness and parity before optimization | Premature layout, SIMD, or parallel decisions could hide incompatibilities and destabilize the API | Accepted in Phase 4 through source-ordered scalar kernels and fail-closed rejection of nonbaseline CPU, SIMD/FMA, contraction, and unsafe floating options |
| Require explicit compatibility sign-off per subsystem | Incremental, reviewable evidence prevents premature global parity claims | Accepted for math/settings in Phase 4, collision in Phase 5, and the minimal rigid-world slice in Phase 6; later subsystems must repeat scoped inventory, unit, differential, documentation, review, and security evidence |
| Keep a bounded WASM playground independent of native Cargo consumers | Visitors can explore scenes in a browser without making C++ or wasm-pack a published-crate requirement | Accepted in v1.1: private `liquidfun-wasm`, SolidJS Pages site, `liquidfun` remains the sole default member |
| Ship six native playground scenes instead of a scene editor or JS physics clone | Owner-approved catalog using existing engine capabilities | Accepted in v1.1: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel |
| Use semantic HTML plus Kobalte Dialog for the responsive playground shell | Accessible modal behavior without a broad design-system migration | Accepted in v1.1; recorded in `standards-overrides.md` |
| Deploy the playground from main with OIDC Pages, not a PAT | Automatic website delivery without reviving Linux native qualification | Accepted in v1.1 HOST-01..03 |
| Treat v1.1 as a planning label, not a package or git release tag | Archive completion is not publication | Accepted 2026-09-20; same policy as v1.0 |
| Use Dam Break Medium native pair as the v1.2 numeric gate (Rust ≤ 3× C++) | The existing exploratory pair already shows ~300×; a same-order close is the honest first bar | ✓ Achieved at stamp `2026-09-21T20-38-50Z`, ratio `2.956857456935513`; not a public claim |
| Profile and fix shared particle/rigid hot paths; do not revive the Phase 12 sealed 32-case public matrix | Phase 12 defined method, but the reviewed-report manifest is empty; this milestone is observability plus closing the canary | ✓ Shared scalar kernels plus a same-cluster spot-check; `reviewed_reports` stays empty |
| Hold the scalar deterministic compatibility baseline while chasing the ~300× gap | Correctness and determinism still beat unproven SIMD/parallel defaults; SIMD/parallel stay explicit opt-in | ✓ Held: no default SIMD or Rayon, `unsafe_code = "forbid"` |
| Native is the C++ comparison; WASM/playground is a post-gate sanity check only | WASM vs C++ is not a fair pair; still record whether the playground improved | ✓ `just web-player-smoke` recorded; the note never compares WASM to `oracle-release` |
| Treat v1.2 as a planning label, not a package or git release tag | Archive completion is not publication | Accepted 2026-09-21; same policy as v1.0 and v1.1 |
| Port the twelve missing JavaScript testbed scenes in the existing player, then four original scenes | Recognizable behavior with honest credits; Fountain, Wave Machine, Water Wheel, and Liquid Timer stay | ✓ Shipped in v1.3; the catalog has 23 ready scenes |
| Treat v1.3 as a planning label, not a package release | Archive completion is not crate publication | Accepted 2026-09-28. The owner later authorized annotated tag `v1.3` as a milestone marker, not a crates.io release |
| Simplify v1.4 to a survey plus targeted fixes | The per-scene evidence harness grew without producing speedups | ✓ Good. Accepted 2026-10-06; three phases delivered measurable gains. |
| Keep only fixes with bit-identical fingerprints and a gain beyond noise (D-11) | Speed without behavior drift | ✓ Good. Seven fixes kept, and several attempts were honestly reverted. |
| Amend D-11 with a 4% noise floor | A/A calibration showed the original rule rejected 1 of 3 runs where nothing changed | ⚠️ Revisit. Accepted 2026-10-10 by the owner. It let A2r be kept with a particles cost of +7.4% / +2.6%, which the owner accepted. |
| Treat v1.4 as a planning label with annotated tag `v1.4` | Archive marker, not crate publication | Accepted 2026-10-11 by the owner. |
| Keep puzzle logic in the private `liquidfun-wasm` bridge, not the engine | The engine and the 25 scene fingerprints must stay unchanged | ✓ Good. `crates/liquidfun` untouched for all of v1.5; 25/25 fingerprints identical from Phase 37 to Phase 44. |
| Decide fill, win, timeout and stars per fixed step in Rust, with recorded replay fixtures | Same placements must give the same outcome on any device | ✓ Good. Fixtures replay identically in native release, wasm and the browser, at 1×, 2× and 4×. |
| Timed valve set while building; jelly pad never counts; soft linear unlocks with open deep links | Owner decisions at milestone start | ✓ Good. Shipped in Phases 40–41. |
| Drop powder as a teaching idea (D-25) | A spike found no powder behavior distinct enough to teach | ✓ Accepted. Buoyancy (float-up) shipped instead. |
| GPU Water as an opt-in mode with a per-frame fallback chain and an opt-in speed lane | Never leave a blank canvas; do not change physics or exports | ✓ Good. Shaded blob stays the default; owner approved iPhone and desktop GPU checks. |
| Add Phase 44 from the milestone audit for owner-selected tech debt only | Keep the closing pass small | ✓ Good. Remaining debt reviewed and deferred by the owner. |
| Treat v1.5 as a planning label with annotated tag `v1.5` | Archive marker, not crate publication | Accepted 2026-10-14 by the owner. |
| Prioritize scenario simulation/rendering speed over memory compactness in v1.4 | User explicitly accepts increased memory usage to obtain speed | Selected 2026-10-03; gains must be demonstrated on matched workloads |
| Preserve existing behavior and fidelity while optimizing v1.4 | User selected this policy in milestone questioning | Selected 2026-10-03; no default lower-quality fast modes |
| Keep all-scene original baselines plus fresh scene-local before/after records | Shared engine/renderer fixes can change later scenes before their own turn | Superseded 2026-10-06 by the simplified survey-and-targeted-fix plan |
| Simplify v1.4 to a timing survey plus targeted fixes | The per-scene evidence campaign spent its harness phase without a speedup; hobby scope favors fast useful iteration | Selected 2026-10-06; harness shelved on `wip/v1.4-phase34-harness` |

## Open Questions

- Which handle, arena, lifetime, callback, user-data, and mutation model best balances safety, API ergonomics, identity stability, and upstream behavior?
- Which observables and tolerances define acceptable parity for each subsystem and platform?
- How should deterministic iteration and contact ordering be preserved without exposing internal storage?
- Which existing permissively licensed Rust components, if any, are compatible enough to reuse after audit?
- Which lightweight rendering stack best supports an optional upstream-equivalent testbed without coupling the core library?
- Which subsets can realistically support full-engine WASM, mobile, or `no_std` without distorting the native library? A bounded playground WASM wrapper exists; complete-engine WASM support is still unclaimed.

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
1. Requirements validated? → Move to Validated with phase reference
1. New requirements emerged? → Add to Active
1. Decisions to log? → Add to Key Decisions
1. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
1. Core Value check — still the right priority?
1. Audit Out of Scope — reasons still valid?
1. Update Context with current state

______________________________________________________________________

*Last updated: 2026-10-19 after Phase 49 (Rayleigh–Taylor Instability). Strict native certification remains optional. Package publication remains separately authorized.*
