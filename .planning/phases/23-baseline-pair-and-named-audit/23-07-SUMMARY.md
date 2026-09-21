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
  - same-tree durable evidence under target/dam-break-perf
affects:
  - 23-08 named-function ranking from live stamps

tech-stack:
  added: []
  patterns:
    - live unprofiled pair.json is the only Dam Break ratio
    - exclusive sibling stamps plus copy-only audit bundle
    - durable_copy same-tree when capture is not worktree-isolated

key-files:
  created:
    - .planning/phases/23-baseline-pair-and-named-audit/23-07-SUMMARY.md
  modified: []

key-decisions:
  - "Canonical live stamps cite MEASURED_HEAD 6d98531ac799987c209d3fd1e572e482fcab5da6 after a docs-only Task 1 commit; pair was re-run so pair/profile/bundle share that SHA."
  - "Only pair.json unprofiled walls and rust_over_cpp_ratio are the Dam Break number; samply duration is discarded."
  - "Live --unstable-presymbolicate sidecar on this host is rust.json.syms.json; the bundle copied it byte-identical."
  - "durable_copy is same-tree: capture already wrote stamps under the primary-tree target/dam-break-perf."
  - "Do not mark PERF-AUDIT complete in this plan; named-function notes remain 23-08."

patterns-established:
  - "Do not commit between live pair and live profile if that would split git_head; bundle requires matching current HEAD."
  - "A1: copy whatever *syms* file actually appears; here rust.json.syms.json."
  - "Never git add *.json.gz, *.syms.json, *.trace, dhat-heap.json, or target/."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:36:28Z

duration: 8min
completed: 2026-09-21
---

# Phase 23 Plan 07: Live Capture Summary

**Live unprofiled Dam Break Medium pair, live samply `rust.json.gz` plus `rust.json.syms.json`, and copy-only audit-bundle at `6d98531ac799987c209d3fd1e572e482fcab5da6`; profiled duration discarded.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-09-21T04:27:42Z
- **Completed:** 2026-09-21T04:36:28Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Ran live `just playground-dam-break-bench` (defaults 60 warmup + 600 steps, 1920 particles, no `--features`) and persisted `pair.json` / `pair.md` as `unprofiled_pair`.
- Ran live `just playground-dam-break-profile` (same 60/600, `--profile profiling`, no `--release`) and persisted nonempty `rust.json.gz` plus `rust.json.syms.json`.
- Minted copy-only `just playground-dam-break-audit-bundle` stamp containing both sources and identity sidecar; sources were not moved.
- Left the three stamps on the primary-tree evidence root (`durable_copy: same-tree`) without `git add` of profile blobs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Live unprofiled pair at current HEAD** - `6d98531` (docs)
1. **Task 2: Live samply profile, list sidecars, mint the audit bundle** - `8498ac6` (docs)
1. **Task 3: Copy the three live stamps to the primary-tree evidence root** - `c43da57` (docs)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Capture identity

- **MEASURED_HEAD:** `6d98531ac799987c209d3fd1e572e482fcab5da6`
- **Capture recipe:** `just playground-dam-break-bench` with default `--warmup 60 --steps 600`, no `--features`
- **samply --version at capture:** `samply 0.13.1` (confirmed before pair)
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
- **worktree_dirty:** true (authorized `config.json` + `.vscode/` plus the in-progress SUMMARY at bundle time)
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

## Files Created/Modified

- `.planning/phases/23-baseline-pair-and-named-audit/23-07-SUMMARY.md` — SHA-bound stamp names, walls/ratio, sidecar `ls`, durable path
- Gitignored live stamps under `target/dam-break-perf/` (not committed)

## Decisions Made

- Canonical evidence SHA is `6d98531ac799987c209d3fd1e572e482fcab5da6` (Task 1 SUMMARY commit). Physics matches `8deb39c7…`; pair was re-run so bundle same-HEAD checks pass.
- Record only `pair.json` `rust.wall_ms` `71001.949958`, `cpp.wall_ms` `216.777375`, and `rust_over_cpp_ratio` `327.53395024734476`. Discard samply / `[profile.profiling]` duration.
- Confirm A1: this host wrote `rust.json.syms.json` beside `rust.json.gz`; bundle copied it.
- `durable_copy: same-tree` — no worktree isolation in this session.
- Leave REQUIREMENTS `PERF-AUDIT` pending for 23-08 named-function notes (`docs/native-performance-audit.md`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Re-ran unprofiled pair after Task 1 docs commit**
- **Found during:** Task 2 (same-HEAD bundle requires pair `git_head` == current `git rev-parse HEAD`)
- **Issue:** Task 1 committed SUMMARY at `6d98531`, while the first pair stamp recorded `8deb39c7…`. A profile at the new HEAD would fail `dam-break-audit-bundle`.
- **Fix:** Re-ran `just playground-dam-break-bench` at `6d98531` (still 60+600 / 1920, no `--features`). Kept the first stamp on disk; bundle sources the second pair.
- **Files modified:** gitignored `target/dam-break-perf/2026-09-21T04-32-19Z/` only
- **Verification:** new `pair.json` `git_head` equals MEASURED_HEAD; finite ratio; no `rust.json.gz` on the pair stamp
- **Committed in:** n/a (evidence not committed); SUMMARY update in `8498ac6`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for D-01 same-SHA pair+profile+bundle. No physics edits, no recipe shrink, no dhat, no `git add` of blobs.

## Issues Encountered

Authorized dirty tree at capture (`config.json` auto-chain flag and `.vscode/`). Bundle identity recorded `worktree_dirty: true`. Not treated as a failing dirty physics tree.

## Authentication Gates

None.

## Known Stubs

None. Live `rust.json.gz` and `rust.json.syms.json` are gitignored host evidence, not fake-tool fixtures.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-08 to rank names from `DURABLE_EVIDENCE` bundle `2026-09-21T04-34-32Z` at MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`.
- 23-08 must use `pair.json` walls/ratio only; do not quote samply duration as 3×.
- Heap run-or-skip waits on the 23-08 allocator/`Vec` heuristic (D-13). Do not run dhat from this plan.
- Do not `git add` `*.json.gz`.
- Do not mark PERF-AUDIT complete until named notes exist.

## Self-Check: PASSED

- FOUND: `.planning/phases/23-baseline-pair-and-named-audit/23-07-SUMMARY.md`
- FOUND: `6d98531` Task 1, `8498ac6` Task 2, `c43da57` Task 3
- FOUND: pair `2026-09-21T04-32-19Z`, profile `2026-09-21T04-32-56Z`, bundle `2026-09-21T04-34-32Z`
- FOUND: `git ls-files '*.json.gz'` empty

---
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*
