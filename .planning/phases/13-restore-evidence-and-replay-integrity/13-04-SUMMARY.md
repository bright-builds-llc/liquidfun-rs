---
phase: 13-restore-evidence-and-replay-integrity
plan: '04'
subsystem: reviewed-evidence-promotion
tags: [transactional-promotion, independent-review, provenance, git-ancestry]
requires:
  - phase: 13-restore-evidence-and-replay-integrity
    plan: '03'
    provides: Canonical Linux evidence producer and immutable acquisition contract
provides:
  - Independently acknowledged deterministic seven-path evidence promotion
  - Transactional replacement with rollback through post-write validation
  - Non-circular P/B/R receipt and exact Q ancestry/trailer contract
affects: [13-05-final-acceptance, phase-15-candidate-evidence]
tech-stack:
  added: []
  patterns:
    - Prepare and acknowledge an exact deterministic diff before tracked replacement
    - Roll back the complete bounded transaction when writes or post-write checks fail
key-files:
  created:
    - reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json
    - reference/artifacts/phase13/promotion-receipt.json
    - tools/xtask/src/phase13_evidence/promotion.rs
    - tools/xtask/src/phase13_evidence/promotion/transaction.rs
    - tools/xtask/tests/phase13_promotion_contract.rs
  modified:
    - crates/liquidfun-differential/src/fixtures/replay/catalog.rs
    - reference/artifacts/manifest.toml
    - reference/artifacts/phase9/lifecycle-contact-witnesses.json
    - reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json
    - reference/source-map.toml
key-decisions:
  - "Keep review acknowledgement external to the reviewed seven paths so acknowledgement cannot change the diff it authorizes."
  - "Treat post-write validation as part of the transaction and roll back every promoted path on failure."
  - "Reject the first canonical bundle after scoped provenance exposed a 123-file closure/full-176-material identity conflation; correct and rerun the independent producer instead of editing reviewed bytes."
requirements-completed: [FND-04, COMP-04, COMP-05, COMP-08, TEST-07, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-07-25T18-25-02
generated_at: 2026-07-26T16:01:01Z
duration: 9h 15m
completed: 2026-07-26
---

# Phase 13 Plan 04: Independently Reviewed Evidence Promotion Summary

**Canonical Linux witness and replay evidence now move through one acknowledged seven-path transaction with complete provenance, rollback, and P → R → Q history**

## Performance

- **Duration:** 9h 15m
- **Started:** 2026-07-26T06:46:00Z
- **Completed:** 2026-07-26T16:01:01Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Added deterministic preparation, exact-digest human acknowledgement, bounded transactional replacement, post-write validation, and promotion-readiness checks.
- Corrected the canonical producer after the required provenance UAT exposed that the first bundle confused the 123-file witness closure with the full 176-entry scoped-material identity.
- Produced and acquired corrected canonical Linux bundle `1eba915ed7cb634b54e0f8d89b0d2be4112bae6f3d3adac6e83ffc355217d775` from run `30192804429`.
- Promoted exactly seven independently acknowledged paths in Q `60325e3118fdf89a8290d63de3ac3374e6f135d1`, whose first parent is R `741e9c5000445a385008f81bb701a5bb91d0e5b4` and whose P/B/R trailers all parse.

## Task Commits

1. **Task 1: Implement the transactional promoter and review contract** - `18e2509`
2. **Task 1 correction: Roll back failed post-write validation** - `a0afa49`
3. **Task 2: Correct and reacquire canonical producer evidence** - `dbf5044`, `741e9c5`
4. **Task 3: Promote exactly seven reviewed evidence paths as Q** - `60325e3`

## Files Created/Modified

- `tools/xtask/src/phase13_evidence/promotion.rs` - Deterministic prepare, acknowledgement, promotion, ledger, and readiness contract.
- `tools/xtask/src/phase13_evidence/promotion/transaction.rs` - Bounded replacement with complete rollback through post-write validation.
- `tools/xtask/tests/phase13_promotion_contract.rs` - Negative acknowledgement/path/ledger tests and transaction rollback regressions.
- `tools/xtask/src/phase13_evidence.rs` - Correct full scoped-material identity in canonical witness provenance.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.json` - Corrected canonical Linux lifecycle/contact witness.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json` - Exact producer, compiler, full materials, and witness identity.
- `reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json` - Reviewed rigid-stack replay evidence.
- `reference/artifacts/phase13/promotion-receipt.json` - Non-circular P/B/R acquisition and Q relationship contract.
- `reference/artifacts/manifest.toml` and `reference/source-map.toml` - Complete tracked evidence ledger and FND-04 traceability.

## Decisions Made

- Kept the acknowledgement outside tracked promoted bytes and bound it to the exact deterministic diff SHA-256.
- Extended transaction rollback across the final worktree/path validation boundary.
- Rejected and replaced invalid canonical evidence through a new exact-SHA Linux run rather than blessing or hand-editing its values.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Included newly created artifacts in exact-path validation**

- **Found during:** Task 3 promotion UAT
- **Issue:** `git diff --name-only` omitted two untracked reviewed artifacts and reported five paths instead of seven.
- **Fix:** Unioned tracked differences with repository-confined untracked paths.
- **Files modified:** `tools/xtask/src/phase13_evidence/promotion.rs`
- **Verification:** Transactional promotion and exact seven-path validation passed.
- **Committed in:** `a0afa49`

**2. [Rule 2 - Missing Critical] Rolled back failures after all writes**

- **Found during:** Task 3 promotion UAT
- **Issue:** Post-write validation occurred outside the rollback boundary.
- **Fix:** Made final validation part of the bounded transaction and added a regression proving every original path is restored.
- **Files modified:** `tools/xtask/src/phase13_evidence/promotion/transaction.rs`, `tools/xtask/tests/phase13_promotion_contract.rs`
- **Verification:** Promotion contract suite passed 11/11.
- **Committed in:** `a0afa49`

**3. [Rule 1 - Bug] Separated full scoped materials from the witness file closure**

- **Found during:** Required `cargo xtask provenance check`
- **Issue:** The first canonical bundle recorded the 123-file closure digest in provenance fields defined for all 176 scoped materials.
- **Fix:** Corrected producer material identity generation, added a regression, reran canonical Linux, and reacquired a new immutable bundle.
- **Files modified:** `tools/xtask/src/phase13_evidence.rs`, `tools/xtask/tests/phase13_evidence_contract.rs`, promoter acquisition constants
- **Verification:** Corrected canonical run succeeded; acquisition, scoped provenance, reviewed evidence, and full Rust checks passed.
- **Committed in:** `dbf5044`, `741e9c5`

**Total deviations:** 3 auto-fixed (2 bugs, 1 missing critical validation boundary). **Impact:** All fixes were required to preserve exact review, rollback, and provenance integrity; no authority value came from local or self-blessing output.

## Issues Encountered

- GitHub workflow dispatch does not accept a raw commit SHA as `--ref`; dispatch used `main` only after verifying the remote branch pointed to the exact producer SHA, and the resulting run's `headSha` was checked mechanically.
- The first Q message placed trailers in separate paragraphs; before push, the message alone was amended so all three trailers form one parsable block.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 13-05 can derive P, B, R, and Q from tracked evidence and Git history.
- Scoped provenance, reviewed replay, ledger hashes, and the full Rust verification surface are green.
- The dedicated exact-A Phase 13 acceptance state machine and workflow remain to be implemented.

***

*Phase: 13-restore-evidence-and-replay-integrity*
*Completed: 2026-07-26*

## Plan 13-05 Schema-v2 Recovery

The original Plan 13-04 promotion history above remains the audit record for the
first reviewed promotion. Plan 13-05 recovered the acceptance chain without
rewriting or erasing that history:

- Canonical acceptance run `30211470256` exposed replay-closure drift after the
  original promotion, so its test-only repair could not be accepted as a final
  descendant.
- Producer candidate `2f51f7ff878049ed364d8930e9c8524c0625a13c`
  and successful run `30231510280` were preserved but superseded before
  acquisition because review found incomplete content-digest, JSON-pointer, and
  exact request-authority semantics.
- The authoritative producer is
  `6e8261a66a67a05bf3fadb4ad9d818121c395324`. Canonical run
  `30232297731` produced bundle
  `fd7fa1a857c0b8cab3ee02fc1d61a45290b632173a4a1f80a790d4334c7453b2`
  as provider artifact `8640500578`.
- Constants-only review base
  `88aba114356cd84c9464d4e6ff62f1d6d3872af7` retained byte-identical
  witness and replay closures from the producer.
- Reviewer `pRizz` freshly acknowledged schema-v2 review subject
  `58e41c6d754341f9dba8a9fbfb1a0c2d4dbc485fdf46129a680a62e2af5a5735`.
  That subject bound every reviewed replacement, separate changed and unchanged
  classifications, path-set digests, and normalized content-set digests.
- Recovery promotion `9f3c7c3480a7e371b4d7c39f7050da3ed4a660e5`
  has the review base as its sole parent and contains the exact P/B/R trailers.
  It changes only the manifest, schema-v2 receipt, and witness provenance;
  catalog source, replay evidence, witness data, and source map remain
  byte-identical to the review base.
- The receipt artifact row explicitly uses
  `phase13_receipt_semantic_v2`; every other artifact row uses exact-byte
  SHA-256. Acceptance separately requires all seven promoted files to remain
  byte-identical from the promotion through the final acceptance head.

The required Rust 1.97 format, Clippy, all-target build, and all-feature test
sequence passed before the recovery promotion. Final canonical acceptance is
recorded by Plan 13-05 rather than retroactively replacing the original Plan
13-04 result.

### Acceptance-only retry

- Exact-head acceptance run `30277369306` at
  `241e75e56567ee6f6f1b7a12d2db889591235754` failed closed during
  provenance validation because the strict Phase 13 evidence record parser had
  not yet declared the new `digest_mode` field.
- The repair adds fail-closed mode validation to provenance: the receipt row
  requires `phase13_receipt_semantic_v2`, while every other promoted artifact
  row requires `exact_bytes_sha256`. The parser and regression test are outside
  both declared producer-affecting closures, so the retry remains an acceptance
  descendant of the same reviewed Q without repromotion.
