---
phase: 24-shared-hot-path-waves-through-3
plan: "05"
subsystem: performance-spot
tags: [PERF-SPOT, PERF-CANARY2, native_scene_spot, playground]

requires:
  - phase: 24-shared-hot-path-waves-through-3
    provides: unprofiled Dam Break Medium pair.json rust_over_cpp_ratio <= 3.0 on this host
provides:
  - just playground-scene-spot one-line native --release five-scene recipe
  - exclusive native_scene_spot stamp 2026-09-21T21-10-50Z with not_timing_authority
  - same-cluster PERF-CANARY2 note (no second canary profile)
affects:
  - 24-06 isolation and empty-manifest close
  - PERF-NOTES / PERF-WASM in Phase 25

tech-stack:
  added: []
  patterns:
    - exclusive native_scene_spot stamp; not_timing_authority; no C++ pair
    - SessionCore::create plus loop advance(1); live_particle_count snapshot
    - just remains a one-line cargo xtask printer

key-files:
  created:
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/bin/playground_scene_spot.rs
    - tools/xtask/src/playground/spot.rs
    - tools/xtask/tests/playground_cli/spot.rs
    - docs/playground-scene-spot-check.md
  modified:
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/Cargo.toml
    - tools/xtask/src/playground.rs
    - tools/xtask/src/main.rs
    - tools/xtask/tests/playground_cli.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs
    - justfile

key-decisions:
  - "PERF-CANARY2 is same-cluster; Fountain and Water Wheel extra wall is emission 1→3200 particle-contact work, not a rigid-solve hang. No second native profile stamp."
  - "Spot JSON is exclusive kind native_scene_spot with not_timing_authority true; no rust_over_cpp_ratio and no pair.json. Gate stamps 2026-09-21T20-36-30Z and 2026-09-21T20-38-50Z were not overwritten."
  - "live_particle_count snapshots the live particle system; construction particle_count is stale after Fountain emit."
  - "just playground-scene-spot is a one-line cargo xtask printer; warmup 60 / measured 120 / advance(1); dam-break-bench is not the spot timer."

patterns-established:
  - "Native scene spot-checks mint a new exclusive stamp and never merge into the Dam Break pair stamp."
  - "Same-cluster canary is a committed notes sentence unless a scene's time or failure mode clearly differs."

requirements-completed: [PERF-SPOT, PERF-CANARY2]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T21:16:49Z

duration: 36min
completed: 2026-09-21
---

# Phase 24 Plan 05: Five-Scene Native Spot-Check Summary

**Native `--release` headless `just playground-scene-spot` writes exclusive `native_scene_spot` walls for Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel, with same-cluster PERF-CANARY2 (no second canary).**

## Performance

- **Duration:** 36 min
- **Started:** 2026-09-21T20:40:08Z
- **Completed:** 2026-09-21T21:16:49Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Shipped `playground-scene-spot` (native binary + xtask + one-line `just` alias) that steps five playground scenes with `SessionCore::create` and loop `advance(1)` (warmup 60, measured 120, 180s per-scene timeout).
- Live exclusive stamp `target/dam-break-perf/2026-09-21T21-10-50Z/scene-spot.json` is `kind native_scene_spot`, `not_timing_authority` true, no `rust_over_cpp_ratio`, no `pair.json`. All five scenes finished with `timed_out` false. Fountain and Water Wheel grew 1→3200 particles, so emission hooks ran.
- Committed unreviewed notes in `docs/playground-scene-spot-check.md`. PERF-CANARY2 is **same-cluster**: extra Fountain/Water Wheel wall is particle-contact emission, not a rigid-solve hang. No second native profile.
- Dam Break gate stamps `2026-09-21T20-36-30Z` and `2026-09-21T20-38-50Z` were not overwritten. `reference/performance/manifest.toml` stayed empty.

## Live spot stamp (not the 3× number)

- Stamp: `target/dam-break-perf/2026-09-21T21-10-50Z/`
- `kind`: `native_scene_spot`
- `not_timing_authority`: true
- `git_head`: `cfdbaabd4b56f9b1d35e50ee283b30d87c540715`
- OS/arch/CPU: `macos` / `aarch64` / `Apple M4 Max`
- rustc: `rustc 1.97.0 (2d8144b78 2026-07-07)`
- warmup 60; measured 120; `advance(1)`

| Scene | start_particles | end_particles | wall_ms | ms/step |
| --- | --- | --- | --- | --- |
| Fountain | 1 | 3200 | 2421.112916 | 20.175941 |
| Float or Sink | 1800 | 1800 | 58.265083 | 0.485542 |
| Color Mixer | 1154 | 1154 | 43.558125 | 0.362984 |
| Jelly Drop | 793 | 793 | 31.168333 | 0.259736 |
| Water Wheel | 1 | 3200 | 2584.517583 | 21.537647 |

## Task Commits

1. **Task 1 RED:** `82ad84f` test(24-05): add failing tests for playground scene-spot
1. **Task 1 GREEN:** `cfdbaab` feat(24-05): implement native playground scene-spot
1. **Task 2 docs:** `6414c27` docs(24-05): record unreviewed five-scene spot-check

**Plan metadata:** (this commit)

No REFACTOR commit: GREEN code already met file-length, `foo.rs` plus `foo/`, and clippy on the new files.

## Files Created/Modified

- `crates/liquidfun-wasm/src/scene_spot.rs` — native-only five-scene `SessionCore` stepper
- `crates/liquidfun-wasm/src/bin/playground_scene_spot.rs` — JSONL one object per scene
- `crates/liquidfun-wasm/src/session.rs` — `live_particle_count` snapshot helper
- `tools/xtask/src/playground/spot.rs` — exclusive stamp persist for `scene-spot.json`
- `tools/xtask/tests/playground_cli/spot.rs` — fake-cargo CLI coverage
- `justfile` — `playground-scene-spot:` → `cargo xtask playground scene-spot`
- `docs/playground-scene-spot-check.md` — unreviewed table plus same-cluster note

## Decisions Made

- PERF-CANARY2 is same-cluster. Fountain and Water Wheel take more wall because they emit up to the 3200-particle cap during 180 `advance(1)` steps; that is still particle-contact work, not a different failure mode. No second native profile stamp.
- Spot JSON is exclusive `native_scene_spot` with `not_timing_authority` true. It must never become the Dam Break 3× number and must not clobber gate stamps.
- `live_particle_count` reads a live snapshot so Fountain end counts can grow. Construction `particle_count` is stale after emit.
- `just playground-scene-spot` stays a one-line printer. Spot defaults are warmup 60 / measured 120, not Dam Break 600. `dam-break-bench` is not the spot timer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Import `USAGE` in RED playground unit tests**
- **Found during:** Task 1 RED
- **Issue:** Tests referenced `USAGE` outside the module that defined it.
- **Fix:** Import `USAGE` in the new tests so RED failed on missing `scene-spot` rather than on compile.
- **Files modified:** `tools/xtask/src/playground.rs`
- **Verification:** RED tests failed on the intended missing command/alias.
- **Committed in:** `82ad84f`

**2. [Rule 1 - Bug] Clippy on new spot persist helpers**
- **Found during:** Task 1 GREEN
- **Issue:** `needless_pass_by_value` on `assemble_report(Vec)`, redundant closure, `duration_suboptimal_units` on `from_secs(180)`.
- **Fix:** `assemble_report(&[Value])`, method map for `ParticleSystemSnapshot::particle_count`, `Duration::from_mins(3)`.
- **Files modified:** `tools/xtask/src/playground/spot.rs`, `crates/liquidfun-wasm/src/scene_spot.rs`
- **Verification:** Clippy clean on the new files; fake-cargo CLI tests passed.
- **Committed in:** `cfdbaab`

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Required for compile/clippy. No scope creep. No second canary. `dam-break-bench` was not reused.

## Issues Encountered

- Workspace `cargo clippy` still reports a pre-existing `unnecessary_wraps` finding in `liquidfun` (out of scope for this plan; not caused by scene-spot files).
- `git commit` via shell heredoc is wrapped with a Co-authored-by trailer; task commits used `gsd-tools.cjs commit`.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. Five live scenes wrote finite `wall_ms` with `timed_out` false.

## Next Phase Readiness

Ready for 24-06 isolation, empty manifest, and no self-approval. Independent review remains a later policy step; this executor did not self-approve.

*Phase: 24-shared-hot-path-waves-through-3*
*Completed: 2026-09-21*

## Self-Check: PASSED
