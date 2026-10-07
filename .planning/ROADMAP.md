# Roadmap: liquidfun-rs

## Overview

v1.4 Scenario Performance (simplified 2026-10-06) makes the playground scenes that actually feel slow faster. A lightweight native timing survey ranks all 25 catalog scenes; the slowest few get profile-guided fixes with simple before/after numbers; a final survey shows the whole-catalog picture. Existing physics, settings, controls and visuals stay unchanged. The original 27-phase per-scene evidence campaign was shelved after its harness phase grew without producing any speedup: its work is preserved on branch `wip/v1.4-phase34-harness` and its ~3.8 GB of evidence outside the repository.

## Milestones

- [ ] **v1.4 Scenario Performance** — Phases 34–36 (planned 2026-10-03, simplified 2026-10-06). Timing survey, targeted fixes for the slowest scenes, final re-survey. No package or tag authorization.
- [x] **v1.3 Reference Testbed Scenes** — Phases 26–33 (archived 2026-09-28). Twelve missing JS testbed scenes, then Hydraulic Fountain, Wave Tank, Liquid Bubbler, and Stacked Drip. No crate or tag. ([full roadmap](milestones/v1.3-ROADMAP.md))
- [x] **v1.2 Native Performance Closing** — Phases 22–25 (archived 2026-09-21). Local Dam Break Medium recorded at ≤ 3× C++; no Phase 12 sealed matrix; no crate or tag. ([full roadmap](milestones/v1.2-ROADMAP.md))
- [x] **v1.1 Web Playground** — 6 phases / 39 plans complete; archived 2026-09-20 ([full roadmap](milestones/v1.1-ROADMAP.md)). Planning label only; no package or tag released.
- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Phases

- [ ] **Phase 34: Scene Timing Survey** — Extend `just playground-scene-spot` to every catalog scene and commit a ranked timing table.
- [ ] **Phase 35: Speed Up the Slowest Scenes** — Profile and fix the slowest scenes from the survey, one plan per scene.
- [ ] **Phase 36: Re-survey and Wrap Up** — Rerun the survey on final source, publish before/after, and close v1.4.

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

- Keep authored physics and presentation: timestep, iterations, counts, geometry, materials, emission, controls and rendering stay as they are. No lower-quality fast modes.
- Measure before changing: a kept change needs a profile naming the hot path and a before/after survey result beyond run-to-run noise (median of at least three runs on the same machine). Revert changes without a gain.
- Keep measurement out of production scene code; the survey drives scenes through the existing `SessionCore` API.
- Normal local checks (`cargo fmt`, clippy, build, tests, and the web tests a change touches) gate every commit. Numbers are local observations, not public speed claims.

### Phase 34: Scene Timing Survey

**Goal**: Maintainers can rank every playground scene by native simulation cost with one command.
**Depends on**: Phase 33 (current 25-scene catalog)
**Requirements**: PERF-07
**Success Criteria** (what must be TRUE):

1. `just playground-scene-spot` steps every catalog scene, and a test fails when a catalog scene is missing from its list.
1. The output reports, per scene, median ms/step over repeated runs and the live particle count.
1. Scenes that are idle by default (for example Drawing or Sparky) get a small scripted interaction so their timing reflects real use; the table says which.
1. `docs/benchmarks/scene-survey.md` records the ranked table with commit, machine and command.

**Plans**: 2 plans

Plans:
- [x] 34-01-PLAN.md — Extend the native stepper and xtask driver to every catalog scene with runs, cues, catalog coverage test and ranked table
- [x] 34-02-PLAN.md — Run the survey on committed source and commit docs/benchmarks/scene-survey.md

### Phase 35: Speed Up the Slowest Scenes

**Goal**: The slowest scenes from the survey run measurably faster with unchanged behavior.
**Depends on**: Phase 34
**Requirements**: PERF-08, PERF-09
**Success Criteria** (what must be TRUE):

1. The top three to five scenes by survey cost (plus any that visibly drop frames in the browser) each have a profile naming their hot path.
1. Each kept change shows a before/after survey gain beyond noise; unhelpful attempts are reverted and noted.
1. Existing Rust and web tests pass, and scene behavior, settings and visuals are unchanged.

**Plans**: TBD (one per targeted scene)

### Phase 36: Re-survey and Wrap Up

**Goal**: Maintainers can see the before/after picture for the whole catalog.
**Depends on**: Phase 35
**Requirements**: PERF-10
**Success Criteria** (what must be TRUE):

1. The survey reruns on final source, and `docs/benchmarks/scene-survey.md` shows before/after for all 25 scenes.
1. Performance notes in user-facing docs cite only these measured local results.
1. v1.4 is archived with any leftover ideas recorded as future work.

**Plans**: TBD

## Progress

**Execution order:** 34 → 35 → 36. The shelved 27-phase plan is preserved in git history (commit `011e80e`) and branch `wip/v1.4-phase34-harness`.

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
| 34. Scene Timing Survey | v1.4 | 2/2 | Complete   | 2026-10-07 |
| 35. Speed Up the Slowest Scenes | v1.4 | 0/TBD | Not started | - |
| 36. Re-survey and Wrap Up | v1.4 | 0/TBD | Not started | - |

v1.0 phases 1–15 remain in the [v1.0 roadmap archive](milestones/v1.0-ROADMAP.md).
