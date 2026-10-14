# Roadmap: liquidfun-rs

## Overview

No milestone is active. v1.5 Puzzle Mode was archived on 2026-10-14. Start the next milestone with `/gsd-new-milestone`, and continue phase numbering after 44.

## Milestones

- [x] **v1.5 Puzzle Mode** — Phases 37–44 (archived 2026-10-14). Place-then-play cup puzzles with stars, six teaching levels proven by replayed fixtures, level select with per-browser progress, opt-in sound and a GPU Water render mode; Phase 44 closed audit tech debt. `crates/liquidfun` untouched and 25/25 scene fingerprints unchanged. Annotated tag `v1.5`; no crate. ([full roadmap](milestones/v1.5-ROADMAP.md))
- [x] **v1.4 Scenario Performance** — Phases 34–36 (archived 2026-10-11). Whole-catalog timing survey, seven profile-guided fixes with bit-identical behavior, and a before/after survey: 24 of 25 scenes faster and none slower, as local observations. Annotated tag `v1.4`; no crate. ([full roadmap](milestones/v1.4-ROADMAP.md))
- [x] **v1.3 Reference Testbed Scenes** — Phases 26–33 (archived 2026-09-28). Twelve missing JS testbed scenes, then Hydraulic Fountain, Wave Tank, Liquid Bubbler, and Stacked Drip. No crate or tag. ([full roadmap](milestones/v1.3-ROADMAP.md))
- [x] **v1.2 Native Performance Closing** — Phases 22–25 (archived 2026-09-21). Local Dam Break Medium recorded at ≤ 3× C++; no Phase 12 sealed matrix; no crate or tag. ([full roadmap](milestones/v1.2-ROADMAP.md))
- [x] **v1.1 Web Playground** — 6 phases / 39 plans complete; archived 2026-09-20 ([full roadmap](milestones/v1.1-ROADMAP.md)). Planning label only; no package or tag released.
- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Phases

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

## Progress

The shelved 27-phase v1.4 plan is preserved in git history (commit `011e80e`) and on branch `wip/v1.4-phase34-harness`.

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

v1.0 phases 1–15 remain in the [v1.0 roadmap archive](milestones/v1.0-ROADMAP.md).
