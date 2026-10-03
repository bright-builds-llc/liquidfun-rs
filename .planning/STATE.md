---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Scenario Performance
status: planning
stopped_at: v1.4 roadmap initialized; Phase 34 ready to plan
last_updated: "2026-10-03T19:45:14Z"
last_activity: 2026-10-03
progress:
  total_phases: 27
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-10-03)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 34 — Measurement Harness and Original Campaign

## Current Position

Current Phase: 34
Current Phase Name: Measurement Harness and Original Campaign
Total Phases: 27
Current Plan: 0
Total Plans in Phase: 0

Phase: 34 of 60 — Measurement Harness and Original Campaign (1 of 27 in v1.4)
Plan: Not started; TBD during Phase 34 planning
**Status:** Ready to plan
Last activity: 2026-10-03 — v1.4 roadmap initialized; 36/36 requirements mapped

**Progress:** 0% [░░░░░░░░░░]

v1.3 remains archived. Historical phase directories and evidence are retained. This milestone definition does not execute scenario optimizations or authorize package publication or release tags.

## Accumulated Context

### Roadmap Evolution

- v1.4: Phases 34–60 — common original campaign, 25 catalog-order scene investigations, final whole-catalog closure.
- Phase 30 added: Periodic hydraulic fountain
- Phase 31 added: Sinusoidal wave tank
- Phase 32 added: Liquid motion bubbler
- Phase 33 added: Stacked drip fidget

### Decisions

v1.3 decisions are in `.planning/milestones/v1.3-STATE.md` and PROJECT.md Key Decisions.

- [v1.3]: Archive is a planning label. No crate publication. Annotated tag `v1.3` marks the archive.
- [v1.4]: Speed takes priority over compact memory; bounded measured increases are allowed. Preserve authored physics/settings, ordering, visuals and lifecycle; no promised percentage/FPS gain.
- [v1.4]: Finish the unchanged-source original all-25 campaign before hot-path edits; each scene then needs fresh current-before, simulation/render profiles, attributable changes/after and independent review. Evidence-backed no-safe-gain closure is allowed.
- [v1.4]: All-six-mode fidelity/default+full-count rendering, affected predecessor canaries and final all-25 coverage apply throughout; raw original/failed attempts remain immutable.
- The recorded ≤ 3× pair stays bound to git `89d34564`. A fresh unprofiled pair is required before claiming that ratio for later HEAD.

### Pending Todos

None.

### Blockers/Concerns

No roadmap blocker. Phase 34 must probe actual WASM/worker attribution, GPU/memory availability, live step-indexed input observation and original witnessed workload windows. No new performance/capability evidence exists yet. A fresh unprofiled pair is required before claiming the ≤ 3× ratio for any HEAD after `89d34564`.

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

## Session Continuity

Last session: 2026-10-03T19:45:14Z
Stopped at: v1.4 roadmap initialized; Phase 34 ready to plan
Resume file: None
