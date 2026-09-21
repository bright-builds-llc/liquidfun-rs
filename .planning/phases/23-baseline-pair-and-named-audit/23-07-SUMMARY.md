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
  - live samply rust.json.gz plus rust.json.syms.json at the same HEAD
  - copy-only audit-bundle stamp naming both source stamps
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

**Live unprofiled Dam Break Medium pair, live samply gzip plus `rust.json.syms.json`, and copy-only audit-bundle at `6d98531ac799987c209d3fd1e572e482fcab5da6`; profiled duration discarded.**

## Capture identity

- **MEASURED_HEAD:** `6d98531ac799987c209d3fd1e572e482fcab5da6`
- **Capture recipe:** `just playground-dam-break-bench` with default `--warmup 60 --steps 600`, no `--features`
- **samply --version at capture:** `samply 0.13.1` (confirmed before pair; required for Task 2)
- **git status --porcelain at capture:** not empty. Authorized clean-tree exception only:
  - `M .planning/config.json` (ephemeral `workflow._auto_chain_active=true` — not reverted, not committed)
  - `?? .vscode/` (pre-existing IDE settings — not committed)
  - No other dirty paths. Capture proceeded with `MEASURED_HEAD=$(git rev-parse HEAD)`.

## Task 1: Live unprofiled pair

Canonical same-HEAD pair (re-run after the Task 1 docs commit so pair/profile/bundle share one SHA):

- **PAIR_STAMP:** `2026-09-21T04-32-19Z`
- **path:** `target/dam-break-perf/2026-09-21T04-32-19Z/`
- **kind:** `unprofiled_pair`
- **timing_authority:** `unprofiled_wall_clock`
- **git_head:** `6d98531ac799987c209d3fd1e572e482fcab5da6` (equals MEASURED_HEAD)
- **particles:** 1920
- **warmup_steps:** 60
- **measured_steps:** 600
- **rust.wall_ms:** `71001.949958`
- **cpp.wall_ms:** `216.777375` (finite, > 0)
- **rust_over_cpp_ratio:** `327.53395024734476` (finite; this is the only 3× number)
- **stamp files:** `pair.json`, `pair.md` (no `rust.json.gz`)
- **profiled duration:** discarded / not present on this stamp

First pair at `2026-09-21T04-29-40Z` / `8deb39c762c9532bec2ee46db267f80112fc0585` remains on disk and is not the bundle source (docs-only HEAD drift after Task 1 commit). Physics bytes are unchanged; only the cited SHA moved.

Do not quote samply duration as the Dam Break ratio. Profile and dhat not run in Task 1.

## Task 2: Live samply profile and copy-only audit bundle

Invoked `just playground-dam-break-profile` with defaults (60/600). No `--release`. No `--features dhat-heap`. Did not run `just playground-dam-break-heap`.

- **PROFILE_STAMP:** `2026-09-21T04-32-56Z`
- **path:** `target/dam-break-perf/2026-09-21T04-32-56Z/`
- **ls -la filenames:** `profile-identity.json`, `rust.json.gz` (463913 bytes), `rust.json.syms.json` (191835 bytes)
- **A1 sidecar:** `rust.json.syms.json` (live `--unstable-presymbolicate`)
- **kind:** `samply_cpu`
- **not_timing_authority:** true
- **samply_version:** `0.13.1`
- **cargo_profile:** `profiling`
- **git_head:** `6d98531ac799987c209d3fd1e572e482fcab5da6` (equals pair stamp and MEASURED_HEAD)
- **warmup_steps:** 60
- **measured_steps:** 600
- **no pair.json** in the profile stamp
- **profiled duration:** discarded; not used as the Dam Break ratio

Invoked `just playground-dam-break-audit-bundle` with no extra flags (latest same-HEAD pair + profile).

- **BUNDLE_STAMP:** `2026-09-21T04-34-32Z`
- **path:** `target/dam-break-perf/2026-09-21T04-34-32Z/`
- **ls -la filenames:** `audit-bundle-identity.json`, `pair.json`, `pair.md`, `rust.json.gz`, `rust.json.syms.json`
- **kind:** `audit_bundle`
- **timing_authority:** `unprofiled_wall_clock` (copied pair.json remains the ratio)
- **not_timing_authority:** true (profile blob)
- **profile_blob:** `rust.json.gz`
- **git_head:** `6d98531ac799987c209d3fd1e572e482fcab5da6`
- **source_pair_stamp:** `2026-09-21T04-32-19Z`
- **source_profile_stamp:** `2026-09-21T04-32-56Z`
- **copied:** `pair.json`, `pair.md`, `rust.json.gz`, `rust.json.syms.json`
- **worktree_dirty:** true (authorized `config.json` + `.vscode/` plus this in-progress SUMMARY)
- **source stamps still present:** copy, not move; SHA-256 of pair.json/pair.md/rust.json.gz/rust.json.syms.json matched after copy
- **git ls-files 'target/dam-break-perf/**':** empty
- **dhat:** not run in this plan (D-13 waits on 23-08)

## Task 3: Durable primary-tree evidence

Resolved paths:

- **COMMON:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/.git`
- **WORKTREE_ROOT:** `/Users/peterryszkiewicz/Repos/liquidfun-rs`
- **WORKTREE_EVIDENCE:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/target/dam-break-perf`
- **DURABLE_EVIDENCE:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/target/dam-break-perf`
- **durable_copy:** same-tree (capture already ran on the primary tree; no `cp -R`, stamps not deleted)

The three exclusive stamps from Tasks 1–2 are already under DURABLE_EVIDENCE:

1. PAIR_STAMP `2026-09-21T04-32-19Z`
1. PROFILE_STAMP `2026-09-21T04-32-56Z`
1. BUNDLE_STAMP `2026-09-21T04-34-32Z`

Durable bundle contains nonempty `pair.json`, `pair.md`, `rust.json.gz`, `rust.json.syms.json`, and `audit-bundle-identity.json` with matching `git_head` `6d98531ac799987c209d3fd1e572e482fcab5da6`. Capture-checkout sources still exist. `git ls-files '*.json.gz' '*.trace' 'dhat-heap.json' 'target/dam-break-perf/**'` is empty. No profile blobs were written into `.planning/` or `docs/`.
