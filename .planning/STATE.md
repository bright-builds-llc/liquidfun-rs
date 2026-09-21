---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Native Performance Closing
status: verifying
stopped_at: Phase 22 complete; next is Phase 23
last_updated: "2026-09-21T01:12:27.238Z"
last_activity: 2026-09-21
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 6
  completed_plans: 6
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-20)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 22 — Observability shell

## Current Position

Phase: 23
Plan: Not started
Status: Phase complete — ready for verification
Last activity: 2026-09-21

Progress: [██████████] 100%

v1.2 Native Performance Closing. Gate is unprofiled `just playground-dam-break-bench` (Rust `--release` vs `oracle-release` Dam Break Medium ≤ 3×). Evidence dir `target/dam-break-perf/<utc-stamp>/`. v1.1 phase directories remain on disk. No package publication or release tag.

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |
| Phase 22 P01 | 31 min | 2 tasks | 6 files |
| Phase 22 P02 | 9 min | 2 tasks | 4 files |
| Phase 22 P03 | 8 min | 2 tasks | 3 files |
| Phase 22 P04 | 9 min | 2 tasks | 6 files |
| Phase 22 P05 | 13 min | 2 tasks | 11 files |
| Phase 22 P06 | 8 min | 2 tasks | 10 files |

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
- [Phase 22]: Persist pair.json and pair.md only after exclusive stamp mint and a finite C++ wall_ms; zero C++ wall is a bench error with no files written. — D-01/D-02: never Inf, never overwrite an existing stamp.
- [Phase 22]: Keep just playground-dam-break-bench as the one-line cargo xtask alias; do not hide cmake or samply in just. — D-05: unprofiled 3x-authority recipe name stays locked.
- [Phase 22]: Honor LIQUIDFUN_XTASK_CARGO else CARGO else cargo, and LIQUIDFUN_XTASK_STAMP_UNIX as test-only unix seconds. — Command path injection plus integer-only stamp clock for fake-tool CLI collision coverage.
- [Phase 22]: pair.json kind is unprofiled_pair with timing_authority unprofiled_wall_clock; omit samply/profile keys. — D-03/PERF-PAIR: unprofiled wall-clock remains the 3x number.
- [Phase 22]: Declare workspace [profile.profiling] with inherits = release, debug = true, strip = false; do not create [profile.release] or set CARGO_PROFILE_RELEASE_DEBUG. — D-08/D-10: named profile keeps the unprofiled --release gate binary unchanged.
- [Phase 22]: samply_record_argv is flags only: record --save-only --unstable-presymbolicate -o gz -- profiling binary --warmup N --steps M; never cargo. — Program path stays on Command::new so 22-04 can spawn without shell joining; Pitfall 3 forbids samply record cargo.
- [Phase 22]: Missing or non-0.13.1 samply is PlaygroundError kind samply with cargo install --locked, brew install samply, and samply setup; no skip wording. — D-09 fail-closed: no Ok skip path and no placeholder rust.json.gz.
- [Phase 22]: Do not mark PERF-PROFILE complete in this plan; dam-break-profile spawn and just alias are 22-04. — This plan delivered the Cargo profile and pure argv/version core only; PERF-PROFILE still needs the CLI wrap.
- [Phase 22]: just playground-dam-break-profile is a one-line cargo xtask alias with no cmake or samply flags. — D-06/D-07: just remains a one-line printer; xtask owns samply and cargo orchestration.
- [Phase 22]: Profile cargo uses --profile profiling and never cargo run --release; samply argv never starts with cargo. — D-08/D-10/Pitfall 3: named profiling profile keeps the unprofiled --release gate binary unchanged.
- [Phase 22]: Missing samply (LIQUIDFUN_XTASK_SAMPLY at a nonexistent path) fails closed with 0.13.1 install text and writes no rust.json.gz. — D-09/T-22-04-06: success tests inject fake samply; fail-closed tests use a missing production lookup path.
- [Phase 22]: profile-identity.json is kind samply_cpu with not_timing_authority true; the stamp has no pair.json. — D-03: profiled wall times must never become the 3x number or enter pair.json.
- [Phase 22]: dam-break-timers is a separate native binary; dam-break-bench still calls only ordinary advance/World::step. — D-10/D-11: step_profiled must not contaminate the unprofiled 3x gate process.
- [Phase 22]: just playground-dam-break-timers is a one-line cargo xtask alias with no cmake or samply flags. — D-06/D-07: just remains a one-line printer; xtask owns cargo and stamp persist.
- [Phase 22]: A fake timers run writes timers.json into a new stamp with not_timing_authority true and no pair.json. — D-04/D-10: timer walls are diagnostic only and must never become the 3x number.
- [Phase 22]: MAX_ADVANCE_STEPS remains 4; measured steps loop advance_profiled(1). — Pitfall 6: do not pass 600 as a single advance count.
- [Phase 22]: BENCHMARKING.md names gitignored target/dam-break-perf stamps and denies profiled/timer sibling stamps as the 3x number; it does not paste wall-ms. — D-14: one-sentence stamp note is allowed; public Dam Break numbers and the empty manifest stay unmodified.
- [Phase 22]: liquidfun production deps stay bitflags-only; cargo tree --edges normal is liquidfun -> bitflags only. — D-12 isolation: package verify plus cargo tree; do not add profiler, samply, dhat, CMake, or serde to liquidfun.
- [Phase 22]: Independent AI review remains a later policy step; this plan records automated evidence and does not self-approve the phase. — Owner policy 2026-09-16: the implementing agent must not approve its own work. T-22-06-05 accept.

### Pending Todos

Verify Phase 22 (`/gsd-verify-work 22`); independent AI review. Phase 24 planning should wait for Phase 23 named shares.

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

Last session: 2026-09-21T01:12:27.234Z
Stopped at: Phase 22 complete; next is Phase 23
Resume file: .planning/phases/22-observability-shell/22-VERIFICATION.md
