---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: milestone
current_phase: 34
current_phase_name: Scene Timing Survey
current_plan: 2
status: executing
stopped_at: Completed 34-01-PLAN.md
last_updated: "2026-10-07T06:19:25.679Z"
last_activity: 2026-10-07
progress:
  total_phases: 3
  completed_phases: 0
  total_plans: 2
  completed_plans: 1
  percent: 50
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-10-06)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 34 — Scene Timing Survey

## Current Position

Current Phase: 34
Current Phase Name: Scene Timing Survey
Total Phases: 3
Current Plan: 2
Total Plans in Phase: 2

Phase: 34 (Scene Timing Survey) — EXECUTING
Plan: 2 of 2
**Status:** Ready to execute
Last activity: 2026-10-07

**Progress:** [█████░░░░░] 50%

v1.3 remains archived. This milestone does not authorize package publication or release tags.

## Accumulated Context

### Roadmap Evolution

- v1.4 (2026-10-06): Simplified to Phases 34–36 — timing survey, targeted fixes for the slowest scenes, re-survey. The original Phases 34–60 plan is in commit `011e80e`.
- Phase 30 added: Periodic hydraulic fountain
- Phase 31 added: Sinusoidal wave tank
- Phase 32 added: Liquid motion bubbler
- Phase 33 added: Stacked drip fidget

### Decisions

v1.3 decisions are in `.planning/milestones/v1.3-STATE.md` and PROJECT.md Key Decisions.

- [v1.3]: Archive is a planning label. No crate publication. Annotated tag `v1.3` marks the archive.
- [v1.4]: Speed takes priority over compact memory; bounded measured increases are allowed. Preserve authored physics/settings, ordering, visuals and lifecycle; no promised percentage/FPS gain.
- [v1.4]: Simplified 2026-10-06. Survey all scenes, fix only the slowest few with profile-backed before/after gains; keep measurement out of production scene code. The Phase 34 harness (plans 1–9, ~150k lines, ~3.8 GB evidence) is shelved on `wip/v1.4-phase34-harness`; evidence moved to `~/Archives/liquidfun-rs-v1.4-phase34-evidence/`.
- The recorded ≤ 3× pair stays bound to git `89d34564`. A fresh unprofiled pair is required before claiming that ratio for later HEAD.
- [Phase 34]: Scene survey SCENE_WALL_TIMEOUT applies per run; start_particles read before the survey cue; even-run median averages middle values

### Pending Todos

None.

### Blockers/Concerns

None. A fresh unprofiled pair is still required before claiming the ≤ 3× Dam Break ratio for any HEAD after `89d34564`.

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements.
- Historical decisions: `.planning/milestones/v1.3-STATE.md`, `.planning/milestones/v1.2-STATE.md`, `.planning/milestones/v1.1-STATE.md`, `.planning/milestones/v1.0-STATE.md`, `.planning/MILESTONES.md`.

### Quick Tasks Completed

| ID | Description | Date | Status | Directory |
| --- | --- | --- | --- | --- |
| 260921-tbx | Global wireframe stroke width slider from 0.1 to 1.5, default 0.3 | 2026-09-22 | complete | [260921-tbx](./quick/260921-tbx-global-wireframe-stroke-width-slider-fro/) |
| 261001-lhi | Resolve main pull conflicts and publish verified scene changes | 2026-10-01 | complete | [261001-lhi](./quick/261001-lhi-resolve-main-pull-conflicts-and-commit-s/) |
| 261001-tvf | Match Tesla Valve reference with alternating curved lobes and closed splitters | 2026-10-01 | complete | [261001-tvf](./quick/261001-tvf-reshape-tesla-valve-with-alternating-cur/) |
| 261001-uk9 | Widen Tesla Valve necks and measure higher flow | 2026-10-01 | complete | [261001-uk9](./quick/261001-uk9-widen-tesla-valve-stage-necks-and-verify/) |
| 261001-w0a | Smooth Tesla Valve curvature and refine particle flow | 2026-10-01 | complete | [261001-w0a](./quick/261001-w0a-smooth-tesla-valve-curvature-with-dense-/) |
| 261002-1yc | Circular Tesla Valve bends and uniform channel width | 2026-10-02 | complete | [261002-1yc](./quick/261002-1yc-make-tesla-valve-bends-circular-and-flow/) |
| 261002-ejw | Set Tesla Valve clear pipe width to six centimeters | 2026-10-02 | complete | [261002-ejw](./quick/261002-ejw-set-tesla-valve-clear-pipe-width-to-six-/) |
| 261002-f8p | Continue Tesla Valve loop stems into the main pipe | 2026-10-02 | complete | [261002-f8p](./quick/261002-f8p-continue-tesla-valve-loop-stems-into-the/) |
| 261002-irz | Five Tesla performance changes with immutable serial benchmark history | 2026-10-02 | complete | [261002-irz](./quick/261002-irz-optimize-tesla-valve-physics-and-worker-/) |
| fast-261007 | Fix wasm32 particle join bound overflow that capped Drawing near ~1,600 particles (commit 04216676b) | 2026-10-07 | complete | - |

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |
| Phase 30 P01 | 41 min | 2 tasks | 5 files |
| Phase 30 P02 | 7 min | 2 tasks | 13 files |
| Phase 30 P03 | 5 min | 2 tasks | 1 files |
| Phase 31 P01 | 13 min | 2 tasks | 5 files |
| Phase 31 P02 | 5 min | 2 tasks | 13 files |
| Phase 31 P03 | 3 min | 2 tasks | 1 files |
| Phase 32 P01 | 20 min | 2 tasks | 5 files |
| Phase 32 P02 | 7 min | 2 tasks | 13 files |
| Phase 32 P03 | 4 min | 2 tasks | 1 files |
| Phase 32 P04 | 24 min | 2 tasks | 3 files |
| Phase 33 P01 | 21 min | 2 tasks | 6 files |
| Phase 33 P02 | 7 min | 2 tasks | 13 files |
| Phase 33 P03 | 3 min | 2 tasks | 1 files |
| Phase 33 P04 | 25 min | 2 tasks | 4 files |
| Phase 34 P01 | 26min | 2 tasks | 8 files |

## Session Continuity

Last session: 2026-10-07T06:19:25.677Z
Stopped at: Completed 34-01-PLAN.md
Resume file: None
