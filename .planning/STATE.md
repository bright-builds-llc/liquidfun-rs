---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: milestone
current_phase: 36
current_phase_name: re survey and wrap up
current_plan: Not started
status: planning
stopped_at: Completed 35-08-PLAN.md
last_updated: "2026-10-10T09:43:42.440Z"
last_activity: 2026-10-10
progress:
  total_phases: 3
  completed_phases: 2
  total_plans: 10
  completed_plans: 10
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-10-06)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 35 — Speed Up the Slowest Scenes

## Current Position

Current Phase: 36
Current Phase Name: re survey and wrap up
Total Phases: 3
Current Plan: Not started
Total Plans in Phase: 8

Phase: 35 (Speed Up the Slowest Scenes) — EXECUTING
Plan: 8 of 8
**Status:** Ready to plan
Last activity: 2026-10-10

**Progress:** [██████████] 100%

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
- [Phase 34]: Recorded scene survey: stamp 2026-10-07T06-20-54Z on 96a3ac6a6; top 5 liquid-tumbler, tesla-valve, stacked-drip, washing-machine, particles
- [Phase 35]: Survey fingerprint is FNV-1a 64 over f32 bits (particles, color lane marker, bodies) after the first run; runs in one invocation must agree
- [Phase 35]: Survey scene filter (repeatable --scene) in bin and xtask; unknown or duplicate ids fail closed before any stamp
- [Phase 35]: Phase before binary target/phase35/bin/spot-before at 809582431; every keep decision uses ABBA runs of saved binaries plus keep_rule.py against before-full.jsonl fingerprints
- [Phase 35]: Target hot paths: damping (liquid-tumbler 22.9%, stacked-drip 17.1%, particles 22.8% self), visit_sorted_tag_indices_in_aabb 13.7% (tesla-valve), full-scan CCD push_fixture_particle_hit 15.2% self / 29.1% incl (washing-machine); D-03 browser check not run
- [Phase 35]: 35-03 A1 kept (91b27a6d6): retained sorted contact-proxy order + sort_unstable_by_key((tag,row)); liquid-tumbler ~24.2-24.4 -> 23.3 ms/step in ABBA, gains also for stacked-drip and particles, 25/25 fingerprints equal
- [Phase 35]: 35-03 A2 reverted: bounded insertion sort beat A1 on liquid-tumbler/stacked-drip but particles and washing-machine regressed in both pairs; diff in target/phase35/attempts/A2.patch
- [Phase 35]: 35-04 A3 reverted (0a2fe8cb8, revert 7f0b36b18): chain child edge hoist cut liquid-tumbler 9.4%/10.4% with 25/25 fingerprints, but soup-stirrer (no chain) exceeded its base max in both isolated pairs
- [Phase 35]: 35-04 A3b reverted (never committed, attempts/A3b.patch): no-branch Shape::Edge variant cut liquid-tumbler 7.6%/7.3% with 25/25 fingerprints, but fountain (no chain) exceeded its base max in both isolated pairs; neither regression reproduced in a later diagnostic pair
- [Phase 35]: 35-05 A4 reverted: per-row AABB tag query gained on no target and regressed liquid-tumbler plus four scenes in both pairs (never committed; attempts/A4.patch)
- [Phase 35]: 35-05 A5 kept (216de3929): bitset walk for body-contact candidate rows, liquid-tumbler -3.0%/-3.0%, tesla-valve -0.9%/-2.2%, 25/25 fingerprints
- [Phase 35]: 35-06 A6 kept (2256dd8cd): verified conservative query pad for moving fixtures at iteration 0 replaces the 0..n CCD scan; washing-machine -29.0%/-28.9% in ABBA, soup-stirrer/water-wheel/theo-jansen faster, 25/25 fingerprints; A7 not tried
- [Phase 35]: 35-07 A8 kept (5695f4b39): ungrouped particle creation reuses group_records with the rebuild's empty-record normalization instead of the O(n) group-lane clone and rebuild; tesla-valve -7.8%/-12.2%, fountain and water-wheel about -20%, 25/25 fingerprints
- [Phase 35]: 35-07 A9 kept (f7041fc75): eviction index uses an in-tree deterministic ParticleIdHasher, in-place resequence and bulk BTreeMap builds; tesla-valve -9.1%/-8.1%, fountain and water-wheel about -27%, 25/25 fingerprints
- [Phase 35]: 35-08 close-out: final HEAD 202075791 has 25/25 fingerprints equal to the phase before; cumulative spot-before vs spot-final ABBA puts all five targets below the before min in both pairs (liquid-tumbler -5.8%/-8.5%, tesla-valve -19.7%/-14.6%, stacked-drip -9.9%/-6.4%, washing-machine -31.8%/-31.7%, particles -5.1%/-7.4%)
- [Phase 35]: 35-08: A3/A3b stay reverted under D-11; possible host-noise evidence and a retry idea recorded in 35-PROFILES.md Notes for Phase 36

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
| Phase 34 P02 | 5min | 2 tasks | 2 files |
| Phase 35 P01 | 445min | 2 tasks | 11 files |
| Phase 35 P02 | 40min | 2 tasks | 1 files |
| Phase 35 P03 | 732min | 2 tasks | 6 files |
| Phase 35 P04 | 659min | 2 tasks | 7 files |
| Phase 35 P05 | 422min | 3 tasks | 3 files |
| Phase 35 P06 | 478min | 2 tasks | 4 files |
| Phase 35 P07 | 411min | 2 tasks | 5 files |
| Phase 35 P08 | 84min | 2 tasks | 3 files |

## Session Continuity

Last session: 2026-10-10T01:38:14.360Z
Stopped at: Completed 35-08-PLAN.md
Resume file: None
