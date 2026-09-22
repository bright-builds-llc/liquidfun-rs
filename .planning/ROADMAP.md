# Roadmap: liquidfun-rs

## Overview

v1.3 Reference Testbed Scenes is active. Visitors get the twelve missing JavaScript LiquidFun testbed scenes in the existing SolidJS playground as recognizable native Rust/WASM ports, then two original hydraulics scenes and two liquid fidget toys. The six current scenes stay. Phase numbering continues after 25 (Phases 26–33). This is not sealed C++ parity, a public benchmark, crate publication, or a git release tag.

## Milestones

- [ ] **v1.3 Reference Testbed Scenes** — Phases 26–33 (in progress). Twelve missing JS testbed scenes, then a periodic hydraulic fountain, a sinusoidal wave tank, and two liquid fidget toys.
- [x] **v1.2 Native Performance Closing** — Phases 22–25 (archived 2026-09-21). Local Dam Break Medium recorded at ≤ 3× C++; no Phase 12 sealed matrix; no crate or tag. ([full roadmap](milestones/v1.2-ROADMAP.md))
- [x] **v1.1 Web Playground** — 6 phases / 39 plans complete; archived 2026-09-20 ([full roadmap](milestones/v1.1-ROADMAP.md)). Planning label only; no package or tag released.
- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Phases

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

### 🚧 v1.3 Reference Testbed Scenes (In Progress)

**Milestone Goal:** A visitor can open every JavaScript LiquidFun testbed scene the playground does not already have, running on native `liquidfun` in the existing SolidJS player. Recognizable ports with honest credits; not sealed C++ parity.

- [x] **Phase 26: Catalog shell and basin scenes** - Shared player growth plus Particles and Liquid Timer (completed 2026-09-22)
- [x] **Phase 27: Material flag groups** - Surface Tension, Elastic Particles, and Rigid Particles (completed 2026-09-22)
- [ ] **Phase 28: Interaction seams** - Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen
- [ ] **Phase 29: Sparky, Drawing, and full catalog** - Contact sparks, paint presets, and all twelve scenes openable
- [ ] **Phase 30: Periodic hydraulic fountain** - A timed piston squeezes liquid so it travels elsewhere
- [ ] **Phase 31: Sinusoidal wave tank** - A still pool whose end platform rises and falls and sends waves
- [ ] **Phase 32: Liquid motion bubbler** - Colored liquid drips through a narrow waist and turns a small wheel
- [ ] **Phase 33: Stacked drip fidget** - Liquid drains through a stack of moving parts, and each part reacts when the drip reaches it

## Phase Details

### Phase 26: Catalog shell and basin scenes
**Goal**: Visitors can use the shared catalog and player with the first watch-first testbed ports—Particles and Liquid Timer—without losing the six existing scenes.
**Depends on**: Phase 25 (v1.2 complete)
**Requirements**: PLAY-02, PLAY-03, BASIN-01, BASIN-02
**Success Criteria** (what must be TRUE):
  1. Visitor can play, pause, and reset Particles and Liquid Timer, and reset restores each scene's initial layout.
  2. Particles and Liquid Timer each credit the pinned LiquidFun test they port.
  3. Visitor can watch Particles: water falls in an open basin and a ball drops into it.
  4. Visitor can watch Liquid Timer: tensile, viscous liquid drains through shelves into bottom columns.
  5. Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel remain available and runnable in the same player.
**Plans**: 5 plans
Plans:
- [x] 26-01-PLAN.md — Particles WASM scene (open basin + water + ball)
- [x] 26-02-PLAN.md — Liquid Timer WASM scene (tensile/viscous drain)
- [x] 26-03-PLAN.md — Catalog append, credits, previews, hide empty controls
- [x] 26-04-PLAN.md — Eight-scene Vitest contract + capture-plan sync
- [x] 26-05-PLAN.md — Chromium smoke: watch-first play/pause/reset + eight previews
**UI hint**: yes

### Phase 27: Material flag groups
**Goal**: Visitors can watch the three material showcase scenes—Surface Tension, Elastic Particles, and Rigid Particles—as recognizable flag-group ports beside the existing demos.
**Depends on**: Phase 26
**Requirements**: MAT-01, MAT-02, MAT-03
**Success Criteria** (what must be TRUE):
  1. Visitor can open, play, pause, and reset Surface Tension, Elastic Particles, and Rigid Particles from the catalog, with credits to the pinned LiquidFun tests.
  2. Visitor can watch Surface Tension: three colored tensile groups bead and bleed color when a ball hits them.
  3. Visitor can watch Elastic Particles: three soft clumps deform when a ball falls on them.
  4. Visitor can watch Rigid Particles: three colored clumps stay solid and do not stretch like jelly when a ball hits them.
**Plans**: 6 plans
Plans:
- [x] 27-01-PLAN.md — Surface Tension WASM scene (TENSILE|COLOR_MIXING + basin family)
- [x] 27-02-PLAN.md — Elastic Particles WASM scene (SPRING/ELASTIC + SOLID)
- [x] 27-03-PLAN.md — Rigid Particles WASM scene (RIGID|SOLID)
- [x] 27-04-PLAN.md — Catalog append, credits, previews, eleven-demo PAGE_SUMMARY
- [x] 27-05-PLAN.md — Eleven-scene Vitest contract + capture-plan sync
- [x] 27-06-PLAN.md — Chromium smoke: watch-first material play/pause/reset + eleven previews
**UI hint**: yes

### Phase 28: Interaction seams
**Goal**: Visitors can run the interaction-heavy ports—Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen—using native destroy-in-shape, group impulse, and live revolute motors (no JS physics fakes).
**Depends on**: Phase 27
**Requirements**: ACT-01, ACT-02, ACT-03, ACT-04, ACT-05
**Success Criteria** (what must be TRUE):
  1. Visitor can watch Soup: a basin of liquid holds floating solid bits.
  2. Visitor can watch Soup Stirrer: a paddle keeps stirring that soup, and the visitor can free the paddle from its rail or put it back.
  3. Visitor can click or tap Impulse and shove the whole particle blob.
  4. Visitor can watch Wave Machine rock on its own and slosh the water inside.
  5. Visitor can watch Theo Jansen walk under a particle load and can reverse its motor.
**Plans**: 8 plans
Plans:
- [x] 28-01-PLAN.md — Destroy-in-shape + shared soup_family + Soup WASM scene
- [x] 28-02-PLAN.md — Soup Stirrer prismatic rail toggle + stir guards
- [ ] 28-03-PLAN.md — Impulse whole-blob shove + live push-mode
- [ ] 28-04-PLAN.md — Wave Machine sim-time revolute motor
- [ ] 28-05-PLAN.md — Theo Jansen soft legs + live motor reverse
- [ ] 28-06-PLAN.md — Catalog append, controls, credits, sixteen-demo PAGE_SUMMARY
- [ ] 28-07-PLAN.md — Sixteen-scene Vitest contract + capture-plan sync
- [ ] 28-08-PLAN.md — Chromium smoke: five new scenes + gestures + eleven regression
**UI hint**: yes

### Phase 29: Sparky, Drawing, and full catalog
**Goal**: Visitors can watch Sparky sparks and paint Drawing Particles, and can open all twelve missing testbed scenes from the catalog while the original six remain.
**Depends on**: Phase 28
**Requirements**: PLAY-01, FX-01, FX-02
**Success Criteria** (what must be TRUE):
  1. Visitor can open Drawing Particles, Elastic Particles, Impulse, Liquid Timer, Particles, Rigid Particles, Soup, Soup Stirrer, Sparky, Surface Tension, Theo Jansen, and Wave Machine from the catalog, and the existing six scenes remain available.
  2. Visitor can watch Sparky: colliding circles throw fading particle sparks (post-step contact observation; no mid-step world mutation; no FFI expansion).
  3. Visitor can paint Drawing Particles into an empty vessel, and at least one non-water material looks different from plain water.
  4. Developer can confirm scene docs and catalog wording describe recognizable ports, not sealed C++ parity, and do not cut particle counts or raise the 4-step catch-up cap to fake smoothness.
**Plans**: TBD
**UI hint**: yes

### Phase 30: Periodic hydraulic fountain

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 29
**Plans:** 2/8 plans executed

Plans:
- [ ] TBD (run /gsd-plan-phase 30 to break down)

### Phase 31: Sinusoidal wave tank

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 30
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 31 to break down)

### Phase 32: Liquid motion bubbler

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 31
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 32 to break down)

### Phase 33: Stacked drip fidget

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 32
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 33 to break down)

## Progress

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
| 28. Interaction seams | v1.3 | 2/8 | In Progress|  |
| 29. Sparky, Drawing, and full catalog | v1.3 | 0/TBD | Not started | - |
| 30. Periodic hydraulic fountain | v1.3 | 0/TBD | Not started | - |
| 31. Sinusoidal wave tank | v1.3 | 0/TBD | Not started | - |
| 32. Liquid motion bubbler | v1.3 | 0/TBD | Not started | - |
| 33. Stacked drip fidget | v1.3 | 0/TBD | Not started | - |

v1.0 phases 1–15 remain in the [v1.0 roadmap archive](milestones/v1.0-ROADMAP.md).
