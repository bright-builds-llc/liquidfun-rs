---
phase: 09-particle-storage-lifecycle-and-coupling
plan: 31
subsystem: differential-evidence
tags: [rust, github-actions, exact-ref, schema-v4, compatibility, security]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    provides: Schema-v4 proof topology and byte-identical local canonical/sanitizer evidence
provides:
  - One immutable exact-SHA Oracle CI dispatch with no retry
  - Independently acquired and exact-ref validated canonical/sanitizer authority
  - Four narrowly promoted Phase 9 platform rows with Phase 10 untouched
  - Passed Phase 9 verifier with both final gaps closed
affects: [phase-10-particles, compatibility-matrix, oracle-evidence]
tech-stack:
  added: []
  patterns:
    - Seal, push, and prove exact branch equality before one-shot evidence dispatch
    - Bind platform promotion to closed run, job, artifact, digest, and semantic identities
key-files:
  created:
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-31-SUMMARY.md
  modified:
    - TESTING.md
    - reference/compatibility.json
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs
    - COMPATIBILITY.md
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-VERIFICATION.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
key-decisions:
  - "Accept only run 29661682074 at sealed SHA 9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce as Phase 9 platform authority."
  - "Promote platform evidence for exactly four Phase 9 rows while keeping all five Phase 10 rows wholly not_evidenced."
patterns-established:
  - "One-shot authority: prove zero prior exact-SHA dispatches, dispatch once, and treat ambiguity as read-only investigation rather than authorization to retry."
  - "Exact-ref promotion: independently query, download, pre-inspect, hash, extract, and validate each authority artifact."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T22:00:00Z
duration: 35min
completed: 2026-07-18
---

# Phase 09 Plan 31: Exact-Ref Authority and Platform Promotion Summary

**One sealed schema-v4 SHA, one successful Oracle CI dispatch, and one independently validated canonical/sanitizer pair now authorize exactly four Phase 9 platform claims.**

## Performance

- **Duration:** 35 min
- **Started:** 2026-07-18T21:25:54Z
- **Completed:** 2026-07-18T22:00:00Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- Sealed `9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce`, proved clean local/remote equality and zero prior exact-SHA dispatches, and issued exactly one Phase 9 Oracle CI dispatch without retry.
- Accepted successful run `29661682074`, canonical job/artifact `88125511292` / `8434547024`, and sanitizer job/artifact `88125511305` / `8434557009` only after independent API queries, safe archive inspection, recomputed SHA-256 values, and exact-ref validation.
- Verified schema v4 with 7 cases, 58 unique semantic bindings, complete 22-policy arrays, retained rigid `Match`, required proof topology, positive energy, nonempty stuck witnesses, and byte-identical canonical/sanitizer manifests.
- Promoted exactly four Phase 9 platform rows with a closed 15-reference authority set and preserved all four unevidenced dimensions for all five Phase 10 rows.
- Passed the full local/exact-ref/tooling/security matrix and a fresh goal-backward verifier at 7/7 truths, 5/5 roadmap criteria, and 14/14 requirements.

## Task Commits

Each task has an atomic execution record:

1. **Task 1: Seal one clean SHA and dispatch Oracle CI exactly once** - `9f2169a` (docs)
2. **Task 2: Acquire and exact-ref validate the sole authority pair** - `2a0ec73` (verification-only empty commit)
3. **Task 3: Promote four Phase 9 rows and verify the full matrix** - `7c4313b` (feat)

## Authority Record

| Identity | Value |
| --- | --- |
| Sealed authority SHA | `9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce` |
| Matching dispatches | 1 |
| Workflow run | `29661682074` |
| Canonical job | `88125511292` |
| Canonical artifact | `8434547024` |
| Canonical archive SHA-256 | `22a37f91965eaf494b3e1fea041e1c54da9be03c06da5e276a641ee6cf536084` |
| Sanitizer job | `88125511305` |
| Sanitizer artifact | `8434557009` |
| Sanitizer archive SHA-256 | `849b8dba5b4c5a0f5e6ea4cddf10bf8243a71bdeec3b75676677358aa34d4316` |
| Shared manifest SHA-256 | `74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c` |
| Semantic manifest SHA-256 | `a319f771c5d9e952b9389160bb3ad19ce487da43271e62568828ce2ae22a33aa` |

Runs `29439515367`, `29583793056`, `29625083184`, and `29652578231`, and artifacts `8423580554`, `8431920189`, and `8431922578`, remain denied.

## Files Created/Modified

- `TESTING.md` - Records the single-dispatch recovery procedure and immutable exact-ref authority.
- `reference/compatibility.json` - Promotes platform evidence for exactly four Phase 9 rows.
- `tools/xtask/src/inventory/validation.rs` - Pins the fresh closed authority set and retains stale/Phase 10 denial.
- `tools/xtask/tests/inventory_cli.rs` - Pins fresh authority acceptance and denylist, mixing, topology, semantic, digest, and Phase 10 regressions.
- `COMPATIBILITY.md` - Deterministically generated 177-row report with platform coverage increasing from 33 to 37 rows.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-VERIFICATION.md` - Records the fresh passed verifier and closure of both final gaps.
- `.planning/STATE.md` and `.planning/ROADMAP.md` - Advance formal plan and phase progress.

## Verification

The complete planned matrix passed:

- Inventory CLI: 21/21.
- Phase 9 evidence CLI: 20/20.
- Phase 9 corpus: 26 passed with one intentional regeneration test ignored.
- Particle oracle: 13/13.
- Particle protocol: 25/25.
- Local canonical and fail-fast sanitizer evidence: 7 cases and 58 bindings in both profiles.
- Exact-ref evidence: 7 cases and 58 bindings with all four denied run IDs supplied.
- Inventory generate/check twice: byte-identical, 177 rows.
- Provenance, dependency policy, schema drift, `actionlint`, Markdown, upstream read-only, diff checks, and the exact ordered Rust gate.
- Fresh `gsd-verifier`: passed, 7/7 truths, no regressions, no human verification.

## ASVS L1 Review

No high-severity finding was identified.

- Workflow identity is exact and uses `contents: read`; no secret enters evidence.
- Run/job/artifact cardinality, SHA, conclusion, expiry, API digest, and local archive digest are cross-checked.
- JSON/log sizes and protocol cardinalities are bounded; unknown fields fail closed.
- Archive and target paths reject absolute paths, traversal, symlinks, unsafe components, and unexpected files.
- Shell arguments are fixed or quoted and the workflow passes `actionlint`.
- Replay topology is validated before deduplication and adversarial alias/substitution regressions pass.
- Generated claims are deterministic and limited to exactly four reviewed rows.

## Decisions Made

- The fresh run is current authority only as the complete same-run canonical/sanitizer pair; neither artifact can substitute for the other.
- Payload and binding digest-set hashes are included in the ledger authority set alongside run, job, artifact, trace, manifest, semantic, commit, documentation, and summary references.
- Phase 10 remains entirely deferred even though the Phase 9 evidence pipeline now has current platform authority.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Staged intended generated claims for local read-only evidence generation**

- **Found during:** Task 3 local canonical evidence verification.
- **Issue:** The evidence script correctly rejected the uncommitted compatibility ledger/report diff even though those were the reviewed outputs under verification.
- **Fix:** Staged only `reference/compatibility.json` and `COMPATIBILITY.md`, reran canonical and sanitizer generation from the beginning, and required paired local validation.
- **Files modified:** None beyond planned files; only the Git index changed.
- **Verification:** Both scripts and paired local validation passed.
- **Committed in:** `7c4313b`.

**2. [AGENTS.md adjustment] Kept the TDD RED and GREEN phases in one committed change**

- **Found during:** Task 3 inventory authority TDD.
- **Issue:** The RED test failed as required, but repository policy forbids a commit unless the complete ordered Rust gate passes.
- **Fix:** Captured the expected failing test, implemented the fresh pins, reran the focused and complete suites, and committed only after GREEN and the full gate.
- **Files modified:** `tools/xtask/tests/inventory_cli.rs`, `tools/xtask/src/inventory/validation.rs`.
- **Verification:** The fresh authority acceptance test and all 21 inventory CLI tests passed.
- **Committed in:** `7c4313b`.

**3. [Rule 3 - Blocking] Completed state transition fields after tracker schema warnings**

- **Found during:** Final GSD tracking.
- **Issue:** `phase complete 09` warned that two legacy state fields were absent, and its repeated validation pass made the derived velocity counter non-idempotent.
- **Fix:** Kept the tool's successful Phase 10 transition, normalized the derived plan count to the authoritative 129/129 progress values, and used the existing STATE.md schema without adding fields.
- **Files modified:** `.planning/STATE.md`.
- **Verification:** Phase 09 has 31/31 summaries, the roadmap reports completion, and STATE.md points to Phase 10 as ready to plan.
- **Committed in:** Plan metadata commit.

**Total deviations:** 2 blocking auto-fixes and 1 repository-policy adjustment.

**Impact on plan:** Both changes preserved the planned trust boundary and added no scope.

## Issues Encountered

- The first local canonical script completed its corpus, provenance, and inventory checks but stopped at the expected dirty generated-claim assertion. After the scoped index adjustment, both profiles were rerun fully and passed.
- The archive inspection loop initially used a Bash array option under the repository's Zsh shell. It was rerun under explicit Bash before extraction; no archive was extracted before the safe checks passed.

## Authentication Gates

None. Existing GitHub CLI authentication was sufficient for read-only queries, the one authorized dispatch, and artifact downloads.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

Phase 9 is verifier-passed and ready to close. Phase 10 can proceed from current platform-authorized particle storage/lifecycle/contact foundations while retaining its five wholly unevidenced group, topology, source-area, and solver rows until their own implementation and evidence exist.

## Self-Check: PASSED

- All nine planned source, evidence, verification, summary, state, and roadmap paths exist.
- Task commits `9f2169a`, `2a0ec73`, and `7c4313b` exist.
- Phase completeness reports 31 plans and 31 summaries with no orphans.
- Lifecycle validation passes for lifecycle `09-2026-07-15T02-54-51`.
- The fresh verification report is newer than its plan and summary inputs and has status `passed`.
- No known stub or new unreviewed threat surface remains.
