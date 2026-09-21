---
phase: 23-baseline-pair-and-named-audit
plan: "07"
subsystem: observability-evidence
tags: [dam-break, unprofiled-pair, samply, audit-bundle]
requires:
  - phase: 23-baseline-pair-and-named-audit
    provides: copy-only dam-break-audit-bundle command
  - phase: 23-baseline-pair-and-named-audit
    provides: fake-cargo heap CLI coverage (no live dhat in this plan)
provides:
  - live unprofiled Dam Break pair stamp at MEASURED_HEAD
affects:
  - 23-08 named-function ranking from live stamps
tech-stack:
  added: []
  patterns:
    - live unprofiled pair.json is the only Dam Break ratio
key-files:
  created:
    - .planning/phases/23-baseline-pair-and-named-audit/23-07-SUMMARY.md
  modified: []
key-decisions: []
patterns-established: []
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:29:40Z
duration: in-progress
completed: in-progress
---

# Phase 23 Plan 07: Live Capture Summary (in progress)

**Live unprofiled Dam Break Medium pair at `8deb39c762c9532bec2ee46db267f80112fc0585` with finite Rust/C++ walls; profile and bundle still pending.**

## Capture identity

- **MEASURED_HEAD:** `8deb39c762c9532bec2ee46db267f80112fc0585`
- **Capture recipe:** `just playground-dam-break-bench` with default `--warmup 60 --steps 600`, no `--features`
- **samply --version at capture:** `samply 0.13.1` (confirmed before pair; required for Task 2)
- **git status --porcelain at capture:** not empty. Authorized clean-tree exception only:
  - `M .planning/config.json` (ephemeral `workflow._auto_chain_active=true` — not reverted, not committed)
  - `?? .vscode/` (pre-existing IDE settings — not committed)
  - No other dirty paths. Capture proceeded with `MEASURED_HEAD=$(git rev-parse HEAD)`.

## Task 1: Live unprofiled pair

- **PAIR_STAMP:** `2026-09-21T04-29-40Z`
- **path:** `target/dam-break-perf/2026-09-21T04-29-40Z/`
- **kind:** `unprofiled_pair`
- **timing_authority:** `unprofiled_wall_clock`
- **git_head:** `8deb39c762c9532bec2ee46db267f80112fc0585` (equals MEASURED_HEAD)
- **particles:** 1920
- **warmup_steps:** 60
- **measured_steps:** 600
- **rust.wall_ms:** `68697.839708`
- **cpp.wall_ms:** `215.04925` (finite, > 0)
- **rust_over_cpp_ratio:** `319.45165913389604` (finite; this is the only 3× number)
- **stamp files:** `pair.json`, `pair.md` (no `rust.json.gz`)
- **profiled duration:** discarded / not present on this stamp

Do not quote samply duration as the Dam Break ratio. Profile and dhat not run in Task 1.
