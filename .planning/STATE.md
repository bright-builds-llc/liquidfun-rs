---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Native Performance Closing
status: executing
stopped_at: Completed 22-01-PLAN.md
last_updated: "2026-09-20T23:49:13.478Z"
last_activity: 2026-09-20
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 6
  completed_plans: 1
  percent: 17
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-20)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 22 — Observability shell

## Current Position

Phase: 22 (Observability shell) — EXECUTING
Plan: 2 of 6
Status: Ready to execute
Last activity: 2026-09-20

Progress: [██░░░░░░░░] 17%

v1.2 Native Performance Closing. Gate is unprofiled `just playground-dam-break-bench` (Rust `--release` vs `oracle-release` Dam Break Medium ≤ 3×). Evidence dir `target/dam-break-perf/<utc-stamp>/`. v1.1 phase directories remain on disk. No package publication or release tag.

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |
| Phase 22 P01 | 31 min | 2 tasks | 6 files |

v1.1 plan durations remain in `.planning/milestones/v1.1-STATE.md`.

## Accumulated Context

### Decisions

- [v1.2]: Dam Break Medium native pair is the numeric gate — Rust wall time ≤ 3× pinned C++ on the same host, scalar `--release` vs `oracle-release`.
- [v1.2]: Hunt shared particle/rigid hot paths; other scenes are spot-checked. Do not revive the Phase 12 sealed 32-case public matrix.
- [v1.2]: Hold the scalar deterministic compatibility baseline. SIMD/parallel stay explicit opt-in.
- [v1.2]: Native is the C++ comparison. After the native gate, record a lightweight WASM/playground sanity check; WASM is not compared to C++.
- [v1.2]: Scripted local pair + CPU profiles write dated gitignored evidence under `target/dam-break-perf/<utc-stamp>/`; committed notes name hot functions without becoming a sealed public claim.
- [v1.2]: Unprofiled `just playground-dam-break-bench` is the 3× authority; samply / `[profile.profiling]` / `step_profiled` / dhat timings are never that number.
- [v1.2]: Phase numbering continues after 21 (Phases 22–25). Do not reset to Phase 1.

v1.1 playground decisions remain in `.planning/milestones/v1.1-STATE.md` and PROJECT.md Key Decisions.

- [Phase 22]: Keep dam-break-bench as the only playground subcommand; pair stays stdout-only until 22-02. — D-05: do not add profile or timer commands in 22-01.
- [Phase 22]: Mint stamps with exclusive create_dir under target/dam-break-perf and bump one Unix second on AlreadyExists, capped at 8 attempts. — D-01 fail-closed mint; never remove_dir_all or merge into a pre-existing stamp.
- [Phase 22]: Honor LIQUIDFUN_XTASK_GIT as a Command program path for git rev-parse HEAD. — Matches upstream tool_program pattern; unset env still runs git.
- [Phase 22]: Do not mark PERF-PAIR complete in this plan; pair.json and pair.md persistence is 22-02. — 22-01 delivered the split and stamp core only.

### Pending Todos

Execute 22-02 (`/gsd-execute-phase 22`). Phase 24 planning should wait for Phase 23 named shares.

### Blockers/Concerns

- Dominant kernel unknown until Phase 23 audit; do not pre-select SIMD, PGO, or `unsafe` indexing.
- samply setup / signing on this Mac is host-specific; Phase 22 must fail closed with install text.
- Exploratory Dam Break Medium pair remains ~300× (`docs/playground-dam-break-timing.md`); that gap is the canary, not a public claim.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260919-eut | Split oversized WASM scene tests to satisfy Bright Builds CI | 2026-09-19 | 888b5e8 | [260919-eut-split-oversized-wasm-scene-tests-to-sati](./quick/260919-eut-split-oversized-wasm-scene-tests-to-sati/) |
| 260919-jco | Final responsive shell review fixes | 2026-09-19 | 680b191 | [260919-jco-final-whole-change-responsive-playground](./quick/260919-jco-final-whole-change-responsive-playground/) |
| 260919-wbn | Raise webapp particle frame cap 20x to 10240 and increase scene particle counts about 10x, packing finer so world volumes stay similar | 2026-09-20 | 3a047bf | [260919-wbn-raise-webapp-particle-frame-cap-20x-to-1](./quick/260919-wbn-raise-webapp-particle-frame-cap-20x-to-1/) |

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements.
- Historical decisions: `.planning/milestones/v1.1-STATE.md`, `.planning/milestones/v1.0-STATE.md`, `.planning/MILESTONES.md`.

## Session Continuity

Last session: 2026-09-20T23:48:12.473Z
Stopped at: Completed 22-01-PLAN.md
Resume file: None
