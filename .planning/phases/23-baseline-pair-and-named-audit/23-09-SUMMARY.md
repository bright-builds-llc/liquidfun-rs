---
phase: 23-baseline-pair-and-named-audit
plan: "09"
subsystem: isolation-honesty
tags: [package-verify, cargo-tree, bright-builds, dhat-isolation, independent-review]

requires:
  - phase: 23-baseline-pair-and-named-audit
    provides: committed docs/native-performance-audit.md with live named functions
  - phase: 23-baseline-pair-and-named-audit
    provides: private dhat dump under gitignored heap stamp after allocator/Vec needles
provides:
  - BENCHMARKING.md pointer to unreviewed named-function notes
  - isolation evidence that liquidfun stays bitflags-only
  - explicit record that D-16 independent AI review is not self-approved
affects:
  - /gsd-verify-work 23
  - independent AI review
  - Phase 24 planning after named shares

tech-stack:
  added: []
  patterns:
    - package verify plus cargo tree bitflags-only isolation
    - just remains a one-line cargo xtask printer
    - implementer does not write a review acknowledgment

key-files:
  created: []
  modified:
    - BENCHMARKING.md

key-decisions:
  - "D-16 independent AI review is not performed by this implementing agent; /gsd-verify-work 23 and a separate AI reviewer remain required."
  - "BENCHMARKING.md points at docs/native-performance-audit.md as unreviewed SHA-bound notes; raw json.gz, dhat-heap.json, and trace stay gitignored and out of the empty manifest."
  - "xtask playground units run as cargo test -p xtask playground because the package has no lib target."

patterns-established:
  - "Isolation gates: cargo xtask package verify plus cargo tree -p liquidfun --edges normal showing only bitflags."
  - "The implementing agent must not approve Phase 23; D-16 is a later separate-AI step."

requirements-completed: [PERF-AUDIT, PERF-HEAP]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:58:40Z

duration: 6min
completed: 2026-09-21
---

# Phase 23 Plan 09: Isolation and Honesty Summary

**Package isolation, Bright Builds file-length, and an honest BENCHMARKING.md pointer to unreviewed named-function notes; D-16 independent AI review remains a later separate-AI step.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-09-21T04:52:14Z
- **Completed:** 2026-09-21T04:58:40Z
- **Tasks:** 2
- **Files modified:** 1

## Independent review (D-16) — not satisfied here

This plan does **not** write a REVIEW.md acknowledgment bound to the implementing agent's identity. Independent AI review (D-16, owner 2026-09-16) is **not** performed by this implementer and is **not** satisfied by passing automated checks.

`/gsd-verify-work 23` and a **separate** AI reviewer remain required. The implementing agent must not approve the phase.

## Accomplishments

- Pointed `BENCHMARKING.md` Exploratory local diagnosis at `[docs/native-performance-audit.md](docs/native-performance-audit.md)` as unreviewed SHA-bound named-function notes, without pasting wall-ms, host CPU, or a public “Rust is N× slower” claim.
- Proved published `liquidfun` stays bitflags-only after optional `dhat` entered the workspace lockfile (`package verify` + `cargo tree --edges normal`).
- Confirmed Bright Builds file-length, just/xtask one-line recipes, empty `reviewed_reports`, no committed gzip/trace/dhat dumps, and no physics-kernel diff in this phase.

## Task Commits

Each task was committed atomically:

1. **Task 1: Point BENCHMARKING.md at the unreviewed audit notes** - `83086f8` (docs)
1. **Task 2: Isolation, file-length, and no self-approval** - verification-only; no additional tree change (no empty commit)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Isolation evidence (Task 2)

| Check | Result |
| --- | --- |
| `cargo xtask package verify` | exit 0; `package verified: 238 entries built and tested outside the repository` |
| `cargo tree -p liquidfun --edges normal` | `liquidfun` → `bitflags v2.13.0` only; no `dhat`, `samply`, `serde`, `serde_json`, `flate2`, or `cmake` |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -p xtask -p liquidfun-wasm -p liquidfun --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test -p xtask playground -- --test-threads=1` | 48 playground unit tests passed (see deviation: `--lib` is invalid) |
| `cargo test -p xtask --test playground_cli -- --test-threads=1` | 20 passed |
| `cargo deny --locked check` | advisories ok, bans ok, licenses ok, sources ok |
| `bun scripts/bright-builds-check.ts all` | `file-lengths` findings=0, all findings=0 |
| `just markdown-check` | exit 0 |

Assertions:

- justfile playground recipes are exactly five one-line cargo xtask aliases for dam-break-bench, profile, timers, audit-bundle, and heap; no samply, cmake, or dhat flags in justfile.
- workspace Cargo.toml has profiling debug true and no release-profile override.
- published liquidfun Cargo.toml has bitflags and no dhat, serde, or samply.
- playground pair.rs argv has no dhat-heap feature flag.
- dam-break-bench default path has no step_profiled.
- reference/performance/manifest.toml reviewed_reports is empty; git diff of that file is empty.
- git diff of crates/liquidfun/src is empty.
- git ls-files finds no committed json.gz, trace, or dhat-heap.json blobs.
- no playground or playground_cli mod.rs crate roots.
- docs/native-performance-audit.md contains Unreviewed local sample and Not found.

## Files Created/Modified

- `BENCHMARKING.md` — unreviewed pointer to `docs/native-performance-audit.md`; forbids committing `.json.gz` / `dhat-heap.json` / `.trace` and copying playground numbers into the empty manifest

## Decisions Made

- Do not self-approve. D-16 independent AI review remains after `/gsd-verify-work 23` and must be a separate AI reviewer.
- Keep `BENCHMARKING.md` as a path pointer, not a public speed claim or manifest row.
- Run xtask playground units without `--lib` because `xtask` has no library target.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Ran playground units without `--lib`**
- **Found during:** Task 2 (isolation commands)
- **Issue:** `cargo test -p xtask --lib playground` failed with `error: no library targets found in package xtask` (exit 101). Same constraint recorded in 23-02 / 23-05.
- **Fix:** Ran `cargo test -p xtask playground -- --test-threads=1` instead (48 passed). CLI tests still used `--test playground_cli` as written.
- **Files modified:** none
- **Verification:** 48 unit tests passed; 20 `playground_cli` tests passed
- **Committed in:** n/a (verification-only)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Command spelling only. Isolation, honesty, and Bright Builds gates still passed as specified.

## Issues Encountered

None beyond the expected `--lib` mismatch on `xtask`.

## Authentication Gates

None.

## Known Stubs

None.

## Threat Flags

None. Task 1 only added a docs pointer; Task 2 ran isolation gates. No new network, auth, file-access, or schema surface.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 23 plans are complete on disk after this SUMMARY. Ready for `/gsd-verify-work 23`, then independent AI review by a **separate** agent. This implementer does not approve the phase.
- Phase 24 should start from named shares in `docs/native-performance-audit.md` (`particle_rows` first), not SIMD/Rayon/`unsafe` indexing.
- Do not quote samply or dhat duration as the 3× number.
- Do not `git add` `*.json.gz`, `*.trace`, or `dhat-heap.json`.
- Do not copy playground numbers into `reference/performance/manifest.toml`.

## Self-Check: PASSED

- FOUND: `BENCHMARKING.md`
- FOUND: `.planning/phases/23-baseline-pair-and-named-audit/23-09-SUMMARY.md`
- FOUND: `docs/native-performance-audit.md`
- FOUND: `83086f8` Task 1
- FOUND: `git ls-files '*.json.gz'` empty
- FOUND: SUMMARY states D-16 independent AI review is not self-approved
