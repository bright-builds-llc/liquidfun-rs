---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "31"
subsystem: particle-differential-evidence
tags: [github-actions, exact-ref, provenance, artifact-validation, d1-authority]
requires:
  - phase: 10-30
    provides: Identity-last Phase 10 canonical and sanitizer evidence production
provides:
  - One successful same-SHA Phase 10 canonical and sanitizer D1 authority pair
  - Independently downloaded, archive-safe, exact-ref validated evidence for five cases and 80 semantic leaves
  - Immutable run, job, artifact, digest, toolchain, and failed-attempt denial records
affects: [10-32, phase10-compatibility-promotion, phase10-verification]
tech-stack:
  added: []
  patterns:
    - Re-query live GitHub identity independently before trusting downloaded evidence
    - Inspect exact artifact archives before extraction and validate each profile separately
key-files:
  created:
    - target/phase10-evidence/run.json
    - target/phase10-evidence/phase10-canonical
    - target/phase10-evidence/phase10-sanitizer
  modified:
    - TESTING.md
key-decisions:
  - "Accept only run 29832646127 at repaired immutable SHA b20328aec9697353e322e022cd289e65d5a31340 as the Phase 10 D1 candidate authority."
  - "Keep failed run 29831597090 and artifacts 8495653581 and 8495705068 permanently denylisted alongside the historical embedded denysets."
  - "Preserve no-force-push history during the remote race, then require one successful dispatch for the repaired SHA before exact-ref validation."
patterns-established:
  - "Authority snapshot: run.json must reproduce fresh run, job, artifact, SHA, API digest, size, expiry, and URL data exactly."
  - "Archive admission: validate integrity, normalized paths, depth, modes, duplicates, case-fold collisions, topology, size, and digest before exact-ref content validation."
requirements-completed: [PART-18]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T13:18:27Z
duration: 39m
completed: 2026-07-21
---

# Phase 10 Plan 31: Fresh D1 Authority Acquisition Summary

**One immutable repaired SHA now has a sole successful Oracle CI dispatch whose independent canonical and sanitizer archives validate as the complete five-case, 80-leaf Phase 10 D1 authority candidate.**

## Performance

- **Duration:** 39m
- **Started:** 2026-07-21T12:39:49Z
- **Completed:** 2026-07-21T13:18:27Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Proved repaired implementation SHA `b20328aec9697353e322e022cd289e65d5a31340` equals `origin/main` and has exactly one `Oracle CI` `workflow_dispatch`, successful run `29832646127`.
- Re-queried all live run, job, and artifact APIs independently. Canonical job `88641473476`, sanitizer job `88641473497`, macOS job `88641473484`, and Windows job `88641473543` all succeeded.
- Independently downloaded artifacts `8496062831` and `8496084932`; recomputed their 353,179-byte and 353,175-byte archive digests and matched the GitHub API values exactly.
- Validated both 56-file archives before extraction, kept their extracted contents separate, and passed exact-ref validation for five cases, 80 semantic leaves, every proof role, passing log, D0/replay/debug/release witness, shared manifest, identity, and digest.
- Added immutable authority URLs and the reusable denylisted exact-ref procedure to `TESTING.md` without changing the compatibility ledger or generated report.

## Task Commits

1. **Task 1: Push the existing clean implementation HEAD, dispatch once, and capture immutable run identity** - no task commit by policy; authority acquisition and `run.json` remain gitignored.
1. **Task 2: Redownload and independently exact-ref validate both artifacts** - `3179e7b` (docs)

## Files Created/Modified

- `target/phase10-evidence/run.json` - Fresh run, job, artifact, SHA, API digest, toolchain, URL, and timestamp authority snapshot.
- `target/phase10-evidence/phase10-canonical` - Independently extracted canonical proof set after archive safety validation.
- `target/phase10-evidence/phase10-sanitizer` - Independently extracted sanitizer proof set after archive safety validation.
- `TESTING.md` - Immutable repaired run/job/artifact references, failed-attempt denial, and reusable exact-ref procedure.

## Decisions Made

- The failed initial attempt is historical forensic evidence only. Its run, canonical artifact, and sanitizer artifact are explicit validator denylist inputs and cannot be combined with the repaired run.
- The repaired SHA is a new immutable authority boundary. The live API shows exactly one workflow dispatch for that SHA, so the recovery did not reuse or relabel the failed run.
- The canonical and sanitizer archives remain independent. No file was copied between directories, and exact-ref validation required both archives to agree on the complete semantic manifest.

## Authority Record

| Evidence | Immutable identity | Result |
| --- | --- | --- |
| Repaired implementation | `b20328aec9697353e322e022cd289e65d5a31340` | Local and remote SHA equal before dispatch |
| Successful run | `29832646127` | Sole repaired-SHA dispatch; success |
| Canonical job/artifact | `88641473476` / `8496062831` | Success; 353,179 bytes; `sha256:7b04bdc6715eef0803b5e4ed84ecc8d755559622134715e2ababab491b7cc493` |
| Sanitizer job/artifact | `88641473497` / `8496084932` | Success; 353,175 bytes; `sha256:a416aa078d02e743f4a0882947718f5352df9092a3e206a0b2959a6999a966d9` |
| Portability jobs | macOS `88641473484`; Windows `88641473543` | Both success; not members of the D1 artifact pair |
| Failed prior authority | run `29831597090`; artifacts `8495653581`, `8495705068` | Permanently denylisted |

The canonical and sanitizer identities both bind Linux x86_64, Rust 1.97.0, Clang 22.1.8, upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`, protocol `rigid-world-phase10-v1`, generator `phase10-corpus-v1`, and semantic manifest digest `9f9fd558a6897a43c3fc9faecdce4879efebc7c7d706dc6a1d6577655fa9887b`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Preserved history through a Bright Builds remote race**

- **Found during:** Task 1 clean implementation push
- **Issue:** Two normal pushes were rejected with zero remote mutation. HEAD `0cb96a7` was rejected after `origin/main` raced to `1bb917f`; HEAD `353051f` was then rejected because cherry-picking copied that remote tree without making the original remote commit a DAG ancestor.
- **Fix:** Created history-only reconciliation merge `341fa70`, preserving tree identity while making the remote history an ancestor, then pushed normally. No force push occurred.
- **Files modified:** Git history only; the reconciliation merge introduced no implementation-tree change beyond the Bright Builds update.
- **Verification:** Fetch plus local/remote SHA equality passed before the initial dispatch.
- **Committed in:** `341fa70`

**2. [Rule 1 - Bug] Repaired Windows capture portability after the failed authority run**

- **Found during:** Task 1 initial Oracle CI run
- **Issue:** Initial run `29831597090` at SHA `341fa70` failed because the Phase 10 capture code was not portable on Windows. The run still produced canonical artifact `8495653581` and sanitizer artifact `8495705068`, so neither partial output could remain eligible.
- **Fix:** Commit `b20328a` made the capture portable on Windows and refreshed the affected provenance digests. The complete ordered Rust gate passed, the repair was pushed normally, and the repaired SHA received one later dispatch only.
- **Files modified:** `tools/reference/src/rigid_world_phase10_capture.hpp` and the Phase 9/10 provenance records.
- **Verification:** Repaired run `29832646127` completed all four jobs successfully; exact-SHA API cardinality is one; failed identities are validator denylist inputs.
- **Committed in:** `b20328a`

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** The original failed run was never accepted, relabeled, or retried. Recovery moved to a new immutable repair SHA, retained complete normal-push history, denylisted every failed Phase 10 authority identity, and preserved the compatibility boundary unchanged.

## Issues Encountered

- A first download command used Zsh's reserved `path` array name as a loop variable, which made `gh` unavailable and created only a zero-byte redirection target. No archive was downloaded or extracted. The empty file was removed, the download was repeated with a non-reserved variable, and all safety, size, and digest checks passed before extraction.
- Runtime tracking and `_auto_chain_active` state made the worktree intentionally non-clean during the resumed task. Those exact files were isolated in a uniquely named stash for the clean Task 1 gate, then restored safely before Task 2; none entered either commit.

## Verification

- Clean repaired-HEAD gate passed in exact order: `cargo fmt --all`, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests.
- Fresh Task 2 commit gate passed in the same exact order, including 409 library tests, every integration suite, and 19 doctests.
- GitHub live run/job/artifact data matched every corresponding `run.json` field and proved exactly one repaired-SHA dispatch.
- Both artifacts passed ZIP integrity, normalized relative path, depth, duplicate/case-fold, regular-mode/no-link, size, archive digest, closed file topology, and exact-ref validation.
- Exact-ref validation passed with failed run `29831597090`, failed artifacts `8495653581` and `8495705068`, and every historical embedded run/artifact denyset supplied.
- `cargo xtask provenance check`, pinned mdformat 1.0.0 under Python 3.13, aggregate `just markdown-check`, Plan 31 exact verification, and the 1/1 key-link check passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-32 can promote only outcomes proven by repaired run `29832646127` and its exact canonical/sanitizer artifact pair.
- The failed run and artifacts remain explicit denial inputs. No compatibility ledger or generated report changed in this plan.

## Self-Check: PASSED

- Confirmed Task 2 commit `3179e7b` exists and contains only `TESTING.md`.
- Confirmed the gitignored evidence snapshot and two extracted directories exist and exact-ref validation reports five cases and 80 semantic leaves.
- Confirmed no dispatch, rerun, push, compatibility edit, or generated-report edit occurred during the resumed authority-validation segment.
- Confirmed `.planning/config.json`, `.planning/agent-history.json`, and `.planning/current-agent-id.txt` remain uncommitted runtime state.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-21*
