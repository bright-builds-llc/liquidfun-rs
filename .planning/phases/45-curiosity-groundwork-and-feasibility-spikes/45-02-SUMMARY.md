---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "02"
subsystem: scene-fingerprints
tags: [fingerprints, gate, xtask, scene-spot, prep-01]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:00:51.212Z"
requires:
  - "45-01: docs/benchmarks/scene-fingerprints.json baseline and the 2026-10-14T15-53-11Z scene-spot capture"
provides:
  - "cargo xtask playground scene-fingerprint-gate [--baseline <path>] [--spot <path> | --stamp <utc>]"
  - "just scene-fingerprint-gate *args (thin recipe)"
  - "spot::capture(args) -> PathBuf of the new scene-spot.json"
affects: [45-10, 46, 47, 48, 49, 50, 51]
tech-stack:
  added: []
  patterns:
    - "Functional core (parse_baseline, parse_observed, compare_fingerprints, render_report, provenance_warnings) with a thin I/O shell in fingerprint_gate::run"
    - "foo.rs + foo/ layout: fingerprint_gate.rs with fingerprint_gate/args.rs and fingerprint_gate/tests.rs"
key-files:
  created:
    - tools/xtask/src/playground/fingerprint_gate.rs
    - tools/xtask/src/playground/fingerprint_gate/args.rs
    - tools/xtask/src/playground/fingerprint_gate/tests.rs
  modified:
    - tools/xtask/src/playground/spot.rs
    - tools/xtask/src/playground.rs
    - tools/xtask/src/main.rs
    - justfile
key-decisions:
  - "45-02: scene-fingerprint-gate joins on scene id (BTreeMap/BTreeSet); changed/missing protected ids fail, unknown observed ids pass as additions; report shape fixed (PASS/FAIL, protected n/m unchanged, changed, missing, added (accepted), baseline, observed)"
  - "45-02: fresh mode by default (spot::capture(&[]) full catalog); --spot or --stamp read a saved capture; --stamp goes through stamp::parse_stamp_name; os/arch/compiler mismatch is a warning, not a failure"
  - "45-02: report paths under the repo root print repo-relative so pasted reports carry no home path"
requirements-completed: [PREP-01]
duration: 7min
completed: 2026-10-14
---

# Phase 45 Plan 02: Scene Fingerprint Gate Summary

**`just scene-fingerprint-gate` now compares a fresh or saved scene-spot capture with the pinned baseline by scene id. It reports 25/25 unchanged on the Plan 01 capture and fails, naming `dam-break`, when the baseline is tampered with.**

## Performance

- **Duration:** about 7 min
- **Started:** 2026-10-14T15:54:42Z
- **Completed:** 2026-10-14T16:00:51Z
- **Tasks:** 2
- **Files modified:** 7 (3 created, 4 modified)

## Accomplishments

- Pure gate core: `parse_baseline` and `parse_observed` fail closed on wrong `kind`, a schema other than 1, an empty baseline, malformed fingerprints and duplicate ids. `compare_fingerprints` walks the baseline in order (unchanged, changed, missing), then the observed capture in order (added). `render_report` prints the fixed report shape.
- CLI shell: `--baseline` (default `docs/benchmarks/scene-fingerprints.json`), `--spot <path>`, or `--stamp <utc>` resolved under `target/dam-break-perf/`. With neither flag it runs a fresh full-catalog capture, and scene-spot's own 3-run determinism check still applies. Passing both `--spot` and `--stamp`, an unknown flag, a missing value or a repeated flag gives a usage error. A missing file gives a `gate` error that names the path.
- `spot::run` is now `capture(args).map(|_path| ())`. scene-spot output is unchanged. `spot.rs` is 582 lines, under the 628 limit.
- 26 unit tests: 18 for the core (one per behavior, plus parse order and the provenance warning) and 8 for argument parsing. A new `usage_lists_scene_fingerprint_gate` test was added, and `missing_subcommand_is_a_usage_error` was updated.

## Gate Report (Plan 01 capture, `target/phase45/gate-02.log`)

```
Scene fingerprint gate: PASS
- protected: 25/25 unchanged
- changed: none
- missing: none
- added (accepted): none
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 04650ff5ade210a0cbaf9770f38ef6fbfebad54a, 2026-10-14T15-53-11Z)
- observed: target/dam-break-perf/2026-10-14T15-53-11Z/scene-spot.json (git 04650ff5ade210a0cbaf9770f38ef6fbfebad54a, full_catalog true)
```

## Negative Check (`target/phase45/gate-02-negative.log`)

The second baseline fingerprint (`dam-break`) was set to `0000000000000000` in a copy at `target/phase45/gate-02-tampered.json`. The tracked baseline was not edited, and `git status --porcelain docs/benchmarks/` is empty. The command exited with code 1:

```
Scene fingerprint gate: FAIL
- protected: 24/25 unchanged
- changed: dam-break 0000000000000000 -> 4fa65d209b7e782f
- missing: none
- added (accepted): none
- baseline: target/phase45/gate-02-tampered.json (captured at 04650ff5ade210a0cbaf9770f38ef6fbfebad54a, 2026-10-14T15-53-11Z)
- observed: target/dam-break-perf/2026-10-14T15-53-11Z/scene-spot.json (git 04650ff5ade210a0cbaf9770f38ef6fbfebad54a, full_catalog true)
error: playground/gate: scene fingerprint gate failed: 1 changed, 0 missing
```

## Extra Checks

- `--stamp 2026-10-14T15-53-11Z`: PASS, 25/25 unchanged.
- Fresh mode (`just scene-fingerprint-gate`, no flags): new capture `target/dam-break-perf/2026-10-14T16-00-20Z/scene-spot.json`, PASS 25/25, exit 0, no provenance warning (`target/phase45/gate-02-fresh.log`).
- `--spot` with `--stamp` gives `playground/usage: --spot and --stamp cannot be used together`. `--spot /nonexistent.json` gives ``playground/gate: `/nonexistent.json` is not an existing file``.
- `cargo fmt --all`, `cargo clippy -p xtask --all-targets --all-features -- -D warnings` clean; `cargo test -p xtask --bin xtask playground::` 99 passed; `cargo test -p xtask --test playground_cli` 24 passed; `bun scripts/bright-builds-check.ts all` findings=0.

## Task Commits

1. **Task 1: Pure gate core with unit tests**: `c090d675e` (test, RED with 17 failing tests) and `6658c8fb1` (feat, GREEN with 17 passing)
1. **Task 2: CLI shell, spot::capture refactor, usage text and just recipe**: `ec0dd960e` (feat)

## Files Created/Modified

- `tools/xtask/src/playground/fingerprint_gate.rs` - core types and functions, `run` shell, provenance warnings, JSON reading (366 lines)
- `tools/xtask/src/playground/fingerprint_gate/args.rs` - `GateArgs`, `ObservedSource`, `parse_gate_args` with inline AAA tests
- `tools/xtask/src/playground/fingerprint_gate/tests.rs` - AAA tests for the pure core
- `tools/xtask/src/playground/spot.rs` - `capture` returns the spot path; `is_fingerprint` and `SPOT_FILE_NAME` are `pub(super)`
- `tools/xtask/src/playground.rs` - subcommand arm, USAGE, missing-subcommand sentence, new usage test
- `tools/xtask/src/main.rs` - help line mentions the scene fingerprint gate
- `justfile` - `scene-fingerprint-gate *args` recipe

## Decisions Made

- Paths under the repository root print relative to it in report labels, so reports pasted into summaries carry no home-directory path. Paths outside the root print unchanged.
- The baseline is read and validated before a fresh capture starts, so a bad baseline fails fast instead of after a run of several minutes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Argument parsing moved to `fingerprint_gate/args.rs` from the start**
- **Found during:** Task 2
- **Issue:** Core plus shell plus argument parsing would have been well over the plan's limit of about 350 lines for `fingerprint_gate.rs`.
- **Fix:** Used the plan's fallback, `fingerprint_gate/args.rs`, with 8 inline AAA tests (traversal stamp, both sources together, unknown flag, missing value, repeated flag). The core file is 366 lines.
- **Commit:** ec0dd960e

**2. [Rule 2 - Missing critical] `SPOT_FILE_NAME` is now `pub(super)`**
- **Found during:** Task 2
- **Issue:** `--stamp` resolution needs the same file name that scene-spot writes, and copying the string would let the two drift apart.
- **Fix:** The gate reuses `spot::SPOT_FILE_NAME`.
- **Commit:** ec0dd960e

The plan's output section asks for `generated_by: gsd-executor`. I followed the orchestrator's `gsd-execute-phase`, as Plan 01 did.

## Issues Encountered

None.

## Next Phase Readiness

- Every later phase can run `just scene-fingerprint-gate` (fresh) or `just scene-fingerprint-gate --stamp <utc>` and paste the report into its summary.
- Plan 10's final-tree run can use fresh mode directly.

## Self-Check: PASSED

- FOUND: tools/xtask/src/playground/fingerprint_gate.rs
- FOUND: tools/xtask/src/playground/fingerprint_gate/args.rs
- FOUND: tools/xtask/src/playground/fingerprint_gate/tests.rs
- FOUND: target/phase45/gate-02.log, target/phase45/gate-02-negative.log
- FOUND: commits c090d675e, 6658c8fb1, ec0dd960e
