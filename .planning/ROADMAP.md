# Roadmap: liquidfun-rs

## Overview

v1.6 Physics Curiosity Scenes adds six small, self-contained playground scenes of physics people like to share and wonder about: a Cartesian diver, an Archimedes screw, a fake ferrofluid, a Rayleigh–Taylor instability, a siphon and a sand hourglass (SEED-004). Each scene lives in the private `liquidfun-wasm` bridge as a scene module built on existing public engine APIs, so `crates/liquidfun` and the 25 existing scene fingerprints stay unchanged. Phase 45 prepares the repository (a committed fingerprint baseline with a gate that joins on scene id, scene counts derived from the catalog, file splits) and runs time-boxed siphon and hourglass feasibility spikes. Then one scene ships per phase, lowest risk first: Diver, Screw, Ferrofluid, Rayleigh–Taylor, Siphon, Hourglass. The two spike-dependent scenes go last so they can slip; the hourglass returns to a seed if its spike fails. Both spikes failed their pre-recorded rules, and the owner confirmed both drops on 2026-10-14: the siphon returned to SEED-008 and the sand hourglass to SEED-007, so Phases 50 and 51 are marked dropped and v1.6 now targets four scenes. Phase numbering continues after 44. No crate publication.

## Milestones

- [ ] **v1.6 Physics Curiosity Scenes** — Phases 45–51 (planned 2026-10-14). Groundwork and siphon/hourglass feasibility spikes, then six curiosity scenes one per phase: Cartesian diver, Archimedes screw, fake ferrofluid, Rayleigh–Taylor instability, siphon and sand hourglass (conditional on its spike). The Phase 45 spikes dropped the siphon (SEED-008) and the sand hourglass (SEED-007), owner confirmed 2026-10-14, leaving four scenes. The 25 existing scenes stay unchanged. No package or tag authorization.
- [x] **v1.5 Puzzle Mode** — Phases 37–44 (archived 2026-10-14). Place-then-play cup puzzles with stars, six teaching levels proven by replayed fixtures, level select with per-browser progress, opt-in sound and a GPU Water render mode; Phase 44 closed audit tech debt. `crates/liquidfun` untouched and 25/25 scene fingerprints unchanged. Annotated tag `v1.5`; no crate. ([full roadmap](milestones/v1.5-ROADMAP.md))
- [x] **v1.4 Scenario Performance** — Phases 34–36 (archived 2026-10-11). Whole-catalog timing survey, seven profile-guided fixes with bit-identical behavior, and a before/after survey: 24 of 25 scenes faster and none slower, as local observations. Annotated tag `v1.4`; no crate. ([full roadmap](milestones/v1.4-ROADMAP.md))
- [x] **v1.3 Reference Testbed Scenes** — Phases 26–33 (archived 2026-09-28). Twelve missing JS testbed scenes, then Hydraulic Fountain, Wave Tank, Liquid Bubbler, and Stacked Drip. No crate or tag. ([full roadmap](milestones/v1.3-ROADMAP.md))
- [x] **v1.2 Native Performance Closing** — Phases 22–25 (archived 2026-09-21). Local Dam Break Medium recorded at ≤ 3× C++; no Phase 12 sealed matrix; no crate or tag. ([full roadmap](milestones/v1.2-ROADMAP.md))
- [x] **v1.1 Web Playground** — 6 phases / 39 plans complete; archived 2026-09-20 ([full roadmap](milestones/v1.1-ROADMAP.md)). Planning label only; no package or tag released.
- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Phases

- [x] **Phase 45: Curiosity Groundwork and Feasibility Spikes** — Pin the 25 fingerprints behind an id-join gate, derive scene counts from the catalog, split near-limit files, and decide go, reshape or drop for the siphon and hourglass. (completed 2026-10-14)
- [ ] **Phase 46: Cartesian Diver** — Press and hold the bottle to sink the diver, release to let it rise; first scene through the shared checklist.
- [ ] **Phase 47: Archimedes Screw** — Moving slanted vanes lift pockets of water up a tilted trough, with live speed and reverse.
- [ ] **Phase 48: Fake Ferrofluid** — A dark puddle rises into spikes toward a draggable magnet, with a safe strength range.
- [ ] **Phase 49: Rayleigh–Taylor Instability** — A heavier liquid layer fingers down through a lighter one, with a choosable weight ratio.
- [ ] **Phase 50: Siphon** — Dropped by the Phase 45 spike; SIPHON-01 and SIPHON-02 returned to SEED-008
- [ ] **Phase 51: Sand Hourglass** — Dropped by the Phase 45 spike; SAND-01 and SAND-02 returned to SEED-007

<details>
<summary>✅ v1.5 Puzzle Mode (Phases 37-44) — SHIPPED 2026-10-14</summary>

v1.5 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.5-ROADMAP.md](milestones/v1.5-ROADMAP.md).

- [x] Phase 37: Puzzle Core in the Bridge (8/8 plans) — completed 2026-10-12
- [x] Phase 38: Puzzle Route and Player Shell (9/9 plans) — completed 2026-10-12
- [x] Phase 39: Place-then-Play Tools and Stars (14/14 plans) — completed 2026-10-13
- [x] Phase 40: Timed Valve and Teaching Levels (11/11 plans) — completed 2026-10-13
- [x] Phase 41: Level Select and Progress (8/8 plans) — completed 2026-10-13
- [x] Phase 42: Opt-in Sound (5/5 plans) — completed 2026-10-13
- [x] Phase 43: GPU Water Render Mode (6/6 plans) — completed 2026-10-14
- [x] Phase 44: v1.5 Polish and Test Gaps (3/3 plans) — completed 2026-10-14

</details>

<details>
<summary>✅ v1.4 Scenario Performance (Phases 34-36) — SHIPPED 2026-10-11</summary>

v1.4 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.4-ROADMAP.md](milestones/v1.4-ROADMAP.md).

- [x] Phase 34: Scene Timing Survey (2/2 plans) — completed 2026-10-07
- [x] Phase 35: Speed Up the Slowest Scenes (8/8 plans) — completed 2026-10-10
- [x] Phase 36: Re-survey and Wrap Up (3/3 plans) — completed 2026-10-11

</details>

<details>
<summary>✅ v1.3 Reference Testbed Scenes (Phases 26-33) — SHIPPED 2026-09-28</summary>

v1.3 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.3-ROADMAP.md](milestones/v1.3-ROADMAP.md).

- [x] Phase 26: Catalog shell and basin scenes (5/5 plans) — completed 2026-09-22
- [x] Phase 27: Material flag groups (6/6 plans) — completed 2026-09-22
- [x] Phase 28: Interaction seams (8/8 plans) — completed 2026-09-22
- [x] Phase 29: Sparky, Drawing, and full catalog (5/5 plans) — completed 2026-09-27
- [x] Phase 30: Periodic hydraulic fountain (3/3 plans) — completed 2026-09-27
- [x] Phase 31: Sinusoidal wave tank (3/3 plans) — completed 2026-09-27
- [x] Phase 32: Liquid motion bubbler (4/4 plans) — completed 2026-09-27
- [x] Phase 33: Stacked drip fidget (4/4 plans) — completed 2026-09-28

</details>

<details>
<summary>✅ v1.2 Native Performance Closing (Phases 22-25) — SHIPPED 2026-09-21</summary>

v1.2 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.2-ROADMAP.md](milestones/v1.2-ROADMAP.md).

- [x] Phase 22: Observability shell (6/6 plans) — completed 2026-09-21
- [x] Phase 23: Baseline pair and named audit (9/9 plans) — completed 2026-09-21
- [x] Phase 24: Shared hot-path waves through 3× (6/6 plans) — completed 2026-09-21
- [x] Phase 25: WASM sanity and honest close (2/2 plans) — completed 2026-09-21

</details>

<details>
<summary>✅ v1.1 Web Playground (Phases 16-21) — SHIPPED 2026-09-20</summary>

v1.1 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.1-ROADMAP.md](milestones/v1.1-ROADMAP.md).

- [x] Phase 16: Rust WASM Browser Bridge (4/4 plans) — completed 2026-09-17
- [x] Phase 17: Shared Player and Early Pages Delivery (8/8 plans) — completed 2026-09-17
- [x] Phase 18: Six Native Physics Demos (10/10 plans) — completed 2026-09-18
- [x] Phase 19: Interaction Polish and Browser Verification (7/7 plans) — completed 2026-09-19
- [x] Phase 20: Playground catalog previews and Reset honesty (6/6 plans) — completed 2026-09-20
- [x] Phase 21: Playground leftover cleanup (4/4 plans) — completed 2026-09-20

</details>

## Phase Details

### Shared acceptance rules

These rules apply to every v1.6 phase. For traceability, PREP-01 is mapped to Phase 45 and SCENE-01, SCENE-02 and SCENE-03 are mapped to Phase 46, the first scene phase, but **every scene phase (46–51) must satisfy SCENE-01..03, and every phase must pass the PREP-01 fingerprint gate.**

- **The existing scenes stay unchanged (PREP-01).** After every phase, the committed, pinned list of scene fingerprints is compared by joining on scene id: every protected id must be present and identical, and new ids may appear. The protected set grows as each new scene ships. A baseline is never regenerated in the same commit as a scene change.
- **Every new scene meets the catalog rules (SCENE-01).** A catalog record with honest credits ("original experimental scene"), an in-app preview, a portrait frame in `web/src/catalog/portrait-bounds.ts` that fills a phone canvas (about 390 by 844 CSS pixels), a README SVG/WebP plan in `web/scripts/readme-svg/plans.ts` with `just readme-svg` output reviewed, the shared scale legend in meters, the gravity slider and the HUD safe frame. Its id joins `ALL_SCENE_IDS` and the survey list.
- **Every new scene moves on its own and stays light (SCENE-02).** It starts visibly doing its thing within about 1 s on default settings with no input, except where the scene invites a press (the diver). It stays at or below about 3,000 particles, at a toy meter scale, and the phase records its particle count and native ms/step.
- **Every caption is honest (SCENE-03).** The caption describes what the simulation actually does and names any faked mechanism plainly, for example "a scripted magnetic pull" or "extra weight on the heavy layer". Claims of suction, compressed air, simulated density or an unmeasured angle of repose are not allowed.
- **No engine change by default.** Scene mechanisms are bridge code on existing public `World` APIs through the existing `SceneHooks` seams. Continuous forces are applied once per fixed step in `on_advance`, follow `world.gravity()`, and treat pointer Up and Cancel alike. The only allowed engine addition is an off-by-default `World::set_particle_flags` matching upstream, and only if the hourglass spike needs it; it must leave every protected fingerprint unchanged.
- **No new dependencies or cue kinds.** No new Cargo or npm dependency, bridge API, worker message, or README/survey cue kind. Interactions are press, drag and toggle (no hover).
- **A spike decision binds its scene.** Phases 50 and 51 follow the go, reshape or drop decision recorded in Phase 45. A drop moves that scene's requirements back to a seed by owner decision rather than shipping a scene that claims something it does not do.
- Normal local checks gate every commit: `cargo fmt`, clippy, build and tests, plus the web unit and e2e tests a change touches.
- Research flags: Phase 45 (the spikes are the research), Phases 50 and 51 (run `/gsd-research-phase` after reading the spike summaries), Phase 49 (parameter sweep inside the phase), Phases 47 and 48 (short spikes inside the phase). Phase 46 uses standard patterns.

### Phase 45: Curiosity Groundwork and Feasibility Spikes

**Goal**: The playground can grow past 25 scenes without silently breaking anything, and the two riskiest scenes have measured go, reshape or drop decisions before any scene work starts.
**Depends on**: Phase 44 (v1.5 archive and the 25-scene fingerprints)
**Requirements**: PREP-01, PREP-02, PREP-03
**Success Criteria** (what must be TRUE):

1. A committed, pinned list of the 25 scene ids and their end-state fingerprints, with host and toolchain provenance, is recorded before any scene edit. A gate that joins on scene id passes when every protected id is present and identical, fails when one is missing or changed, and accepts new ids.
1. Scene counts in Rust, TypeScript, the scene survey and e2e tests derive from one catalog list (`ALL_SCENE_IDS` and the web catalog), no test or user-facing copy hard-codes 25 or "twenty-five", and the near-limit catalog files are split so a new scene record fits under the file-length limit. A written per-scene checklist lists every Rust and web site a new scene touches.
1. The siphon spike records measured results for each candidate mechanism, in order, against its written pass rule (water keeps transferring for at least 2 s after the source surface is 0.3 m below the crest, and the final level difference is under 0.15 m), and ends with a recorded go, reshape or drop decision.
1. The hourglass spike records the measured deposit slope against its written pass rule (at least 20°, stable within ±5° over 5 s, with no jam) and the measured flip cost, and ends with a recorded go or drop decision that the owner confirms.
1. The id-join gate reports 25/25 protected fingerprints unchanged on the final tree, and normal local checks pass.

**Plans**: 10 plans

Plans:
- [x] 45-01-PLAN.md — Capture and commit the pinned 25-scene fingerprint baseline before any code edit (PREP-01)
- [x] 45-02-PLAN.md — Tested xtask id-join gate and `just scene-fingerprint-gate` (PREP-01)
- [x] 45-03-PLAN.md — `ALL_SCENE_IDS` in catalog order; Rust counts derived from it (PREP-02)
- [x] 45-04-PLAN.md — Number-free copy; web unit and e2e counts derived from the catalog (PREP-02)
- [x] 45-05-PLAN.md — Split scene-records.ts and previews.tsx into family modules (PREP-02)
- [x] 45-06-PLAN.md — Siphon feasibility spike with a go, reshape or drop decision (PREP-03)
- [x] 45-07-PLAN.md — Hourglass feasibility spike, flip cost and decision by the pre-recorded rule (PREP-03)
- [x] 45-08-PLAN.md — Owner checkpoint for both spike decisions, then seed and roadmap bookkeeping (PREP-03)
- [x] 45-09-PLAN.md — Per-scene checklist docs/adding-a-scene.md and AGENTS.md pointer (PREP-01, PREP-02)
- [x] 45-10-PLAN.md — Final gate: 25/25 fingerprints and normal local checks (PREP-01..03)

### Phase 46: Cartesian Diver

**Goal**: Players can squeeze a bottle and watch a diver sink, then let go and watch it rise, in the first new scene to go through the shared per-scene checklist end to end.
**Depends on**: Phase 45
**Requirements**: DIVER-01, DIVER-02, DIVER-03, SCENE-01, SCENE-02, SCENE-03 (SCENE-01..03 apply to every scene phase 46–51; see Shared acceptance rules)
**Success Criteria** (what must be TRUE):

1. Opening the Cartesian diver scene shows a diver floating near the top of a closed bottle. Pressing and holding on the bottle, or toggling Squeeze from the keyboard, makes it sink to the bottom third within about 3 s, and releasing (including a cancelled pointer) makes it rise again.
1. While squeezed, the bottle visibly dents and the air bubble inside the diver shrinks, and a phone long press on the bottle does not open a system callout.
1. Native tests show the diver sinking when squeezed and rising when released at gravity 2, 10 and 80 m/s².
1. The scene meets every shared scene rule: catalog record with credits, preview, phone-filling portrait frame, README animated preview driven by the Squeeze toggle, scale legend, gravity slider and HUD safe frame, at most about 3,000 particles, and a caption saying that switching the diver's density stands in for the squeezed air bubble.
1. The id-join gate reports every protected fingerprint unchanged with the new diver id present, and the phase records the diver's particle count and ms/step.

**Plans**: 5 plans

Plans:
- [x] 46-01-PLAN.md — Rust bridge scene, hooks, draw-only dent and bubble, release sink/rise tuning at gravity 2/10/80
- [x] 46-02-PLAN.md — Catalog registration (Rust + web), record, preview, portrait frame, README/survey/demo plans, tests and e2e wiring
- [x] 46-03-PLAN.md — Shared long-press callout suppression on the scene canvas with phone-viewport e2e
- [ ] 46-04-PLAN.md — README preview generation and review, full gates, scene-spot numbers, 25/25 fingerprint gate
- [ ] 46-05-PLAN.md — Separate single-file baseline growth commit, gate rerun 26/26 unchanged
**UI hint**: yes

### Phase 47: Archimedes Screw

**Goal**: Players can watch a turning screw lift water uphill in pockets, and change its speed or run it backwards.
**Depends on**: Phase 46
**Requirements**: SCREW-01, SCREW-02
**Success Criteria** (what must be TRUE):

1. On load, moving slanted vanes in a tilted trough carry separate pockets of water from a low reservoir up to a higher basin, and the water returns down a chute so the scene keeps running.
1. Player can change the screw's speed live from a HUD control and run it in reverse, so it carries water back down.
1. A native 60 s run at default and maximum speed shows water gained in the upper basin, no particles leaking through walls or evicted, and a reservoir that never runs dry.
1. The scene meets every shared scene rule, with a caption saying flat vanes stand in for a turning helix in a 2D side view, and the id-join gate reports every protected fingerprint unchanged, including the diver's.

**Plans**: TBD
**UI hint**: yes

### Phase 48: Fake Ferrofluid

**Goal**: Players can push a magnet around under a dark liquid puddle and watch it rise into spikes that follow the magnet and collapse when it lets go.
**Depends on**: Phase 47
**Requirements**: FERRO-01, FERRO-02, FERRO-03
**Success Criteria** (what must be TRUE):

1. On load, a dark liquid puddle that is clearly visible on the dark canvas rises into several separate spikes (at least 4 surface peaks, not one blob) toward a magnet resting under the dish, and the magnet is drawn as a marker.
1. Pressing and dragging moves the magnet and the spikes follow it. Releasing, cancelling the pointer, or switching the magnet off makes the spikes collapse back into a puddle.
1. Player can adjust magnet strength only within a range that never flings the liquid apart, and a native 60 s test with the pointer held at maximum strength shows no runaway energy, no evictions and no failed step.
1. The magnet pull is applied once per fixed step and follows world gravity, so the gravity slider and phone tilt stay correct. The caption puts "fake" in the title or description and calls it a scripted magnetic pull. The scene meets every shared scene rule, and the id-join gate reports every protected fingerprint unchanged.

**Plans**: TBD
**UI hint**: yes

### Phase 49: Rayleigh–Taylor Instability

**Goal**: Players can watch a heavier liquid layer break into fingers that sink through a lighter one, and replay it with a different weight ratio.
**Depends on**: Phase 48 (reuses its gravity-aware, mass-calibrated per-step force helper)
**Requirements**: RT-01, RT-02
**Success Criteria** (what must be TRUE):

1. On load, a high-contrast heavy liquid layer above a light one develops fingers that sink through the light liquid (at least 3 heavy fingers past 20% of the tank height by about 4 s), and the layers overturn within about 10 s.
1. Player can choose the heavy-to-light weight ratio within a bounded range (about 1.2× to 3×) and restart the run, and the same settings always produce the same pattern (identical end-state fingerprints across repeated runs).
1. The scene uses one particle system with two groups and extra weight on the heavy group along world gravity, with no engine change. The caption says the heavy layer is given extra weight and never claims a simulated density.
1. The scene meets every shared scene rule, and the id-join gate reports every protected fingerprint unchanged.

**Plans**: TBD
**UI hint**: yes

### Phase 50: Siphon

**Status**: Dropped (Phase 45 siphon spike, 2026-10-14; owner confirmed); see SEED-008.
**Goal**: Players can watch water climb over a crest from a higher tank into a lower one until the levels match, and move the outlet tank to slow, stop or reverse the flow.
**Depends on**: Phase 45 (the siphon spike's go, reshape or drop decision); runs after Phase 49 so it can slip without blocking the other scenes
**Requirements**: SIPHON-01, SIPHON-02
**Success Criteria** (what must be TRUE):

1. The scene is built with the mechanism the Phase 45 siphon spike chose. If that decision was drop, the phase records it and the owner returns SIPHON-01 and SIPHON-02 to a seed instead of shipping overflow labelled as a siphon.
1. On load, water flows from a higher tank over a crest and down a primed tube into a lower tank, the levels converge visibly within about 10 s, and the flow stops when the levels are equal (final difference under 0.15 m).
1. Player can raise or lower the outlet tank, and the flow slows, stops or reverses to match.
1. The caption names the mechanism and credits any stand-in, never saying suction or atmospheric pressure unless that is what is modeled. The scene meets every shared scene rule, and the id-join gate reports every protected fingerprint unchanged.

**Plans**: TBD
**UI hint**: yes

### Phase 51: Sand Hourglass

**Status**: Dropped (Phase 45 hourglass spike, 2026-10-14; owner confirmed); see SEED-007.
**Goal**: Players can watch sand pour through an hourglass neck and pile up at a measured slope, then flip the glass and watch it run again. Conditional: ships only if the Phase 45 hourglass spike passed.
**Depends on**: Phase 45 (the hourglass spike's go or drop decision); runs last so it can slip or be dropped
**Requirements**: SAND-01, SAND-02
**Success Criteria** (what must be TRUE):

1. The scene ships only if the Phase 45 hourglass spike reached a pile slope of at least 20°. Otherwise the phase records the decision and SAND-01 and SAND-02 return to a seed (owner decision, 2026-10-14).
1. On load, sand pours through a narrow neck as a thin stream and builds a peaked pile in the lower bulb whose measured slope is at least 20° and stays within ±5° over 5 s.
1. Player can flip the hourglass. The glass rotates visibly, or flips instantly as a fallback, and all the sand runs again; the flip never works by inverting gravity.
1. The caption credits the sand rule and says "piles" only alongside the measured slope. The scene meets every shared scene rule, and the final id-join gate reports every protected fingerprint unchanged, including each new scene that shipped in v1.6.

**Plans**: TBD
**UI hint**: yes

## Progress

**Execution order:** 45 → 46 → 47 → 48 → 49 → 50 → 51. No scene depends on another scene's code except Phase 49 reusing Phase 48's force helper, so a spike result may reorder the scene phases. The shelved 27-phase v1.4 plan is preserved in git history (commit `011e80e`) and on branch `wip/v1.4-phase34-harness`.

| Phase | Milestone | Plans Complete | Status | Completed |
| --- | --- | --- | --- | --- |
| 16. Rust WASM Browser Bridge | v1.1 | 4/4 | Complete | 2026-09-17 |
| 17. Shared Player and Early Pages Delivery | v1.1 | 8/8 | Complete | 2026-09-17 |
| 18. Six Native Physics Demos | v1.1 | 10/10 | Complete | 2026-09-18 |
| 19. Interaction Polish and Browser Verification | v1.1 | 7/7 | Complete | 2026-09-19 |
| 20. Playground catalog previews and Reset honesty | v1.1 | 6/6 | Complete | 2026-09-20 |
| 21. Playground leftover cleanup | v1.1 | 4/4 | Complete | 2026-09-20 |
| 22. Observability shell | v1.2 | 6/6 | Complete | 2026-09-21 |
| 23. Baseline pair and named audit | v1.2 | 9/9 | Complete | 2026-09-21 |
| 24. Shared hot-path waves through 3× | v1.2 | 6/6 | Complete | 2026-09-21 |
| 25. WASM sanity and honest close | v1.2 | 2/2 | Complete | 2026-09-21 |
| 26. Catalog shell and basin scenes | v1.3 | 5/5 | Complete    | 2026-09-22 |
| 27. Material flag groups | v1.3 | 6/6 | Complete    | 2026-09-22 |
| 28. Interaction seams | v1.3 | 8/8 | Complete    | 2026-09-22 |
| 29. Sparky, Drawing, and full catalog | v1.3 | 5/5 | Complete    | 2026-09-27 |
| 30. Periodic hydraulic fountain | v1.3 | 3/3 | Complete    | 2026-09-27 |
| 31. Sinusoidal wave tank | v1.3 | 3/3 | Complete    | 2026-09-27 |
| 32. Liquid motion bubbler | v1.3 | 4/4 | Complete    | 2026-09-27 |
| 33. Stacked drip fidget | v1.3 | 4/4 | Complete   | 2026-09-28 |
| 34. Scene Timing Survey | v1.4 | 2/2 | Complete    | 2026-10-07 |
| 35. Speed Up the Slowest Scenes | v1.4 | 8/8 | Complete    | 2026-10-10 |
| 36. Re-survey and Wrap Up | v1.4 | 3/3 | Complete    | 2026-10-11 |
| 37. Puzzle Core in the Bridge | v1.5 | 8/8 | Complete    | 2026-10-12 |
| 38. Puzzle Route and Player Shell | v1.5 | 9/9 | Complete    | 2026-10-12 |
| 39. Place-then-Play Tools and Stars | v1.5 | 14/14 | Complete    | 2026-10-13 |
| 40. Timed Valve and Teaching Levels | v1.5 | 11/11 | Complete    | 2026-10-13 |
| 41. Level Select and Progress | v1.5 | 8/8 | Complete    | 2026-10-13 |
| 42. Opt-in Sound | v1.5 | 5/5 | Complete    | 2026-10-13 |
| 43. GPU Water Render Mode | v1.5 | 6/6 | Complete    | 2026-10-14 |
| 44. v1.5 Polish and Test Gaps | v1.5 | 3/3 | Complete    | 2026-10-14 |
| 45. Curiosity Groundwork and Feasibility Spikes | v1.6 | 10/10 | Complete    | 2026-10-14 |
| 46. Cartesian Diver | v1.6 | 3/5 | In Progress|  |
| 47. Archimedes Screw | v1.6 | 0/TBD | Not started | - |
| 48. Fake Ferrofluid | v1.6 | 0/TBD | Not started | - |
| 49. Rayleigh–Taylor Instability | v1.6 | 0/TBD | Not started | - |
| 50. Siphon | v1.6 | 0/TBD | Dropped (SEED-008) | - |
| 51. Sand Hourglass | v1.6 | 0/TBD | Dropped (SEED-007) | - |

v1.0 phases 1–15 remain in the [v1.0 roadmap archive](milestones/v1.0-ROADMAP.md).
