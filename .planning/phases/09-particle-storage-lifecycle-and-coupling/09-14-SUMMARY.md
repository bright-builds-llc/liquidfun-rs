---
phase: 09-particle-storage-lifecycle-and-coupling
plan: 14
subsystem: testing
tags: [phase9-v1, differential, oracle, github-actions, sanitizer, determinism]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: 13
    provides: strict bounded Phase 9 execution in the pinned C++ oracle
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: 15
    provides: pinned equal-lifetime and strict-contact witness bytes
provides:
  - closed exact-once Phase 9 branch declaration corpus
  - retained Phase 6-8 and pinned-oracle byte bindings
  - explicit canonical and sanitizer Phase 9 evidence dispatch paths
  - recomputable trace, manifest, and exact-run artifact identities
affects: [09-16, phase9-evidence, compatibility-promotion]
tech-stack:
  added: []
  patterns:
    - strict manifest coverage checked against an independent required-branch registry
    - workflow artifacts bind included trace and manifest bytes by SHA-256
key-files:
  created:
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json
    - crates/liquidfun-differential/tests/phase9_corpus.rs
  modified:
    - .github/workflows/oracle.yml
    - TESTING.md
key-decisions:
  - "Keep Phase 9 evidence opt-in through one closed workflow_dispatch choice and leave ordinary Phase 8 CI behavior unchanged."
  - "Require CI-only oracle modes to fail when a requested debug, release, or sanitizer binary is absent while ordinary Cargo-only tests remain oracle-independent."
  - "Bind artifact identity to included trace and manifest files with recomputable lowercase SHA-256 values."
patterns-established:
  - "Closed corpus: every reviewed branch token occurs exactly once and missing or unknown coverage fails."
  - "Exact evidence identity: run, commit, toolchain, target, policy, trace, and manifest travel together."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T09:49:13Z
duration: 25min
completed: 2026-07-15
---

# Phase 9 Plan 14: Closed Corpus and Evidence Workflow Summary

**A strict 59-branch Phase 9 corpus now preserves retained rigid identity, rejects Phase 10 observations, and drives opt-in canonical and sanitizer jobs whose artifacts bind exact trace and manifest bytes.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-07-15T09:24:00Z
- **Completed:** 2026-07-15T09:49:13Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Declared every Phase 9 storage, lifecycle, contact, coupling, force, statistic, query, ray, and evidence branch exactly once across seven bounded cases.
- Bound the corpus to the unchanged Phase 6-8 request, pinned upstream revision, and canonical equal-lifetime/strict-contact witness bytes.
- Proved native D0 byte identity and added required CI modes for debug replay, debug/release agreement, and fail-fast sanitizer execution.
- Added exact `phase9` workflow dispatch, artifact names, toolchain identities, included trace/manifest files, and independently recomputable digests without dispatching or promoting evidence.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the closed Phase 9 witness corpus** - `bde7d94` (test)
1. **Task 2: Add commit-ready Phase 9 canonical and sanitizer workflow jobs** - `6ef1f82` (ci)

## Files Created/Modified

- `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json` - Exact corpus cases, authority classes, byte identities, exclusions, and 59 reviewed branch declarations.
- `crates/liquidfun-differential/tests/phase9_corpus.rs` - Strict manifest closure, bounded execution, D0, retained-family, Phase 10 rejection, and required-oracle gates.
- `.github/workflows/oracle.yml` - Explicit Phase 9 canonical/sanitizer execution and identity-bound artifact publication.
- `TESTING.md` - Exact dispatch, job, artifact, identity, digest, and later promotion contract.

## Decisions Made

- Kept Phase 9 evidence opt-in so pull requests, pushes, schedules, and default manual Phase 8 runs retain their prior behavior.
- Used one environment-selected test entrypoint to make missing required oracle binaries fail hard in remote jobs without making ordinary Cargo-only development depend on C++.
- Treated the trace and manifest as ordinary artifact-relative files and recorded their SHA-256 values in `identity.json`, allowing consumers to recompute both digests after download.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added required-oracle execution modes to the corpus test**

- **Found during:** Task 2 (Add commit-ready Phase 9 canonical and sanitizer workflow jobs)
- **Issue:** The existing xtask `rigid-world` scenario remains the retained Phase 8 request, so invoking it could not prove the newly closed Phase 9 corpus against debug, release, and sanitizer binaries.
- **Fix:** Added explicit `canonical` and `sanitizer` modes to `phase9_corpus`; canonical runs byte-identical debug replay plus debug/release result agreement, while sanitizer runs the same bounded corpus through the ASan/UBSan executable.
- **Files modified:** `crates/liquidfun-differential/tests/phase9_corpus.rs`, `.github/workflows/oracle.yml`
- **Verification:** Focused corpus tests pass locally, actionlint validates the workflow, and absent binaries become hard failures only when CI explicitly selects a required mode.
- **Committed in:** `6ef1f82`

***

**Total deviations:** 1 auto-fixed (1 blocking).
**Impact on plan:** The fix was necessary to make the planned workflow execute Phase 9 evidence rather than relabel retained Phase 8 commands. No public API or evidence-promotion scope was added.

## Issues Encountered

- TDD RED first exposed the missing test target, then a type mismatch, and finally an `ExpectedCountMismatch` caused by reusing a retained rigid checkpoint. The final fixture inserts a separate zero-rigid-state Phase 9 checkpoint and leaves every retained checkpoint unchanged.
- The isolated worktree initially lacked its submodule checkout. Initial inventory validation therefore reported a missing discovery root; initializing the exact pinned submodule made the same check pass at 177 rows without modifying the submodule.
- The repository requires all four Rust gates to pass before every commit, so the failing RED state was recorded but not committed. Both task commits were created only after the prescribed green gate sequence.

## Verification

- `cargo test -p liquidfun-differential --test phase9_corpus` passed all five tests.
- Before each commit, `cargo fmt --all`, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests passed in order.
- `cargo xtask inventory check` passed with 177 compatibility rows.
- `just markdown-check` passed with mdformat 1.0.0.
- `actionlint .github/workflows/oracle.yml`, `git diff --check`, and the pinned-submodule read-only diff passed.
- No remote workflow dispatch, artifact promotion, or compatibility claim occurred.

## User Setup Required

None.

## Next Phase Readiness

- Ready for 09-16 to push one exact commit, dispatch `evidence_phase=phase9`, and verify the canonical and sanitizer artifact identities.
- Remote D1 evidence remains intentionally absent until that checkpoint runs and is explicitly approved.

## Self-Check: PASSED

- Task commits `bde7d94` and `6ef1f82` exist.
- Both key created files exist and the workflow references the corpus test directly.
- `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
