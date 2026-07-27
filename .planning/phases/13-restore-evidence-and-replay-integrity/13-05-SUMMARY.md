---
phase: 13-restore-evidence-and-replay-integrity
plan: '05'
subsystem: reviewed-evidence-acceptance
tags: [differential-replay, transactional-promotion, exact-head-acceptance, provenance]
requires:
  - phase: 13-restore-evidence-and-replay-integrity
    plan: '04'
    provides: Independently reviewed seven-path evidence promotion and P/B/R/Q history
provides:
  - Projection-aware reviewed live replay with bounded typed failure evidence
  - Schema-v2 incremental review and promotion with path and content digests
  - Canonical exact-head P/B/R/Q/A terminal identity accepted on Linux
affects: [phase-14-windows-particle-groups, phase-15-candidate-evidence]
tech-stack:
  added: []
  patterns:
    - Share one acquisition path between canonical production and reviewed live replay
    - Normalize only self-referential receipt digest fields while binding every other byte
    - Publish terminal identity only after one ordered exact-head acceptance state machine
key-files:
  created: []
  modified:
    - crates/liquidfun-differential/src/failure_bundle/catalog.rs
    - tools/xtask/src/phase13_evidence.rs
    - tools/xtask/src/phase13_evidence/promotion.rs
    - tools/xtask/src/phase13_acceptance.rs
    - tools/xtask/src/provenance/evidence_schema.rs
    - reference/artifacts/phase13/promotion-receipt.json
    - reference/artifacts/manifest.toml
    - .github/workflows/phase13-acceptance.yml
key-decisions:
  - "Compare the reviewed legacy physics projection during live acceptance while requiring the expanded capture difference to remain exactly the reviewed capture_schema_drift."
  - "Hash the receipt through a domain-separated semantic leaf that normalizes only its two self-referential content-digest fields."
  - "Treat provenance-parser support for explicit digest modes as acceptance-only because its files are outside both declared producer-affecting closures."
patterns-established:
  - "Review subject completeness: bind P/B/R, seven exact replacements, changed and unchanged classifications, content digests, and deterministic diff."
  - "Exact-head publication: P/B/R/Q/A ancestry, ordered steps, and immutable reviewed bytes precede terminal identity."
requirements-completed: [FND-04, COMP-04, COMP-05, COMP-08, TEST-07, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-07-25T18-25-02
generated_at: 2026-07-27T15:03:58Z
duration: 13h 25m
completed: 2026-07-27
---

# Phase 13 Plan 05: Reviewed Replay Recovery and Exact-Head Acceptance Summary

**Projection-aware live replay, schema-v2 incremental promotion, and a canonical Linux terminal identity now prove the exact P/B/R/Q/A recovery chain**

## Performance

- **Duration:** 13h 25m
- **Started:** 2026-07-27T01:39:07Z
- **Completed:** 2026-07-27T15:03:58Z
- **Tasks:** 5
- **Files modified:** 17

## Accomplishments

- Replaced raw expanded-capture comparison with one strict reviewed live replay that shares production acquisition, matches `legacy_physics_v1`, and requires the expanded difference to remain the reviewed `capture_schema_drift`.
- Added bounded typed failure evidence with exact request authority, RFC 6901 first-divergence pointers, comparison surfaces, captures where available, and separate catalog/acceptance failure roots.
- Promoted one freshly acknowledged schema-v2 review subject across seven reviewed paths while changing only three mechanically different files and preserving four members byte-for-byte.
- Produced and independently acquired canonical bundle `fd7fa1a857c0b8cab3ee02fc1d61a45290b632173a4a1f80a790d4334c7453b2` from exact producer `6e8261a66a67a05bf3fadb4ad9d818121c395324`.
- Passed canonical exact-head acceptance run `30277799121` at `dbaa64819debc5da268d32fcd342da7632ac6370` and independently validated its schema-v2 terminal identity artifact.

## Task Commits

1. **Task 1: Add shared projection-aware live replay and typed failure evidence** - `2f51f7f`, corrected by authoritative producer `6e8261a`
2. **Task 2: Upgrade promotion and exact-head acceptance to schema v2** - `2f51f7f`, corrected by `6e8261a`
3. **Task 3: Produce P/B, establish R, and prepare the review subject** - `6e8261a` (P), `88aba11` (R)
4. **Task 4: Obtain fresh schema-v2 review acknowledgment** - external acknowledgment of `58e41c6d754341f9dba8a9fbfb1a0c2d4dbc485fdf46129a680a62e2af5a5735`
5. **Task 5: Promote Q, establish A, and pass canonical acceptance** - `9f3c7c3` (Q), `241e75e` (first A), `dbaa648` (accepted A)

## Exact Recovery Identities

| Identity | Value |
| --- | --- |
| Producer P | `6e8261a66a67a05bf3fadb4ad9d818121c395324` |
| Bundle B | `fd7fa1a857c0b8cab3ee02fc1d61a45290b632173a4a1f80a790d4334c7453b2` |
| Review base R | `88aba114356cd84c9464d4e6ff62f1d6d3872af7` |
| Promotion Q | `9f3c7c3480a7e371b4d7c39f7050da3ed4a660e5` |
| Accepted head A | `dbaa64819debc5da268d32fcd342da7632ac6370` |
| Review SHA-256 | `58e41c6d754341f9dba8a9fbfb1a0c2d4dbc485fdf46129a680a62e2af5a5735` |
| Promoted content SHA-256 | `ca1dd6abeab2977949507aa9ad88e7abf3e9b29f8f4b21570ee725685806a4bb` |
| Changed content SHA-256 | `6ad850785b64ce948323bc0f8e67d681f6451decae68b94c1c2d5b783b080795` |

Q has R as its sole parent and the exact P/B/R trailers. Its Git diff equals the three recorded changed paths: artifact manifest, schema-v2 receipt, and witness provenance. Catalog source, replay evidence, witness data, and source map are unchanged from R. All seven reviewed paths are byte-identical from Q through accepted A.

## Canonical Evidence

- Producer run `30232297731` passed at exact P and uploaded artifact `8640500578`, named `phase13-staged-30232297731-6e8261a66a67a05bf3fadb4ad9d818121c395324`, with provider digest `sha256:040d7f02c32c40ef6b208f3daf63fb1d458c0cb8cc78cc3d8ccd13e21488e0a7`.
- Acceptance run `30277799121` passed at exact A. It skipped failure upload and published terminal artifact `8657594142`, named `phase13-terminal-identity-30277799121-dbaa64819debc5da268d32fcd342da7632ac6370`, with provider digest `sha256:6e51b5f49937e283761ec9c805552af1de4da2a6cc28fe8c5f1b2e63fc02a304`.
- The downloaded identity is schema v2 and records the exact P/B/R/Q/A chain, full and changed content digests, pinned upstream and oracle identities, four reviewed evidence digests, and seven ordered successful steps ending in `xtask phase13 evidence live-check --tracked --require-reviewed`.

## Files Created/Modified

- `crates/liquidfun-differential/src/failure_bundle/catalog.rs` and `crates/liquidfun-differential/src/failure_bundle/catalog/replay.rs` - Projection-aware replayable catalog failure bundles.
- `crates/liquidfun-differential/tests/catalog_failures.rs` - Exact authority, harness taxonomy, and projection failure regressions.
- `tools/xtask/src/phase13_evidence.rs` - Shared acquisition, strict tracked/live equality, RFC 6901 failure evidence, and reviewed live-check command.
- `tools/xtask/src/phase13_evidence/promotion.rs` - Schema-v2 review packet, content digests, explicit ledger modes, incremental classification, and exact acquisition tuple.
- `tools/xtask/src/phase13_acceptance.rs` - P/B/R/Q/A identity, changed/unchanged byte proofs, ordered live replay, and terminal publication.
- `tools/xtask/src/provenance/evidence_schema.rs` - Fail-closed exact and normalized digest-mode validation.
- `reference/artifacts/phase13/promotion-receipt.json` - Non-circular schema-v2 promotion and content identity.
- `reference/artifacts/manifest.toml` - Explicit exact-byte and normalized-receipt digest modes.
- `.github/workflows/phase13-acceptance.yml` - Success-only terminal identity and both bounded failure roots.
- `.planning/phases/13-restore-evidence-and-replay-integrity/13-04-SUMMARY.md` - Append-only recovery history preserving the original promotion.

## Decisions Made

- Used the existing `compare_catalog_physics_projection` for acceptance rather than weakening generic catalog comparison.
- Kept path-set hashes separate from content-set hashes so membership and bytes are independently explicit.
- Broke receipt/manifest self-reference with one domain-separated normalized receipt leaf; every non-digest receipt field and every other member byte remains bound.
- Required the manifest to declare one known digest mode per Phase 13 row and reject missing or unknown modes.
- Preserved every failed or superseded run and used a new acknowledgment instead of reusing the original promotion authority.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected incomplete exact evidence semantics before authoritative production**

- **Found during:** Tasks 1-2 compliance review
- **Issue:** Initial review hashed path names instead of path/content pairs, emitted JSONPath instead of RFC 6901, and represented request authority with an abbreviated identity.
- **Fix:** Added full and changed content digests, RFC 6901 escaping, and explicit upstream revision, resolved scenario path, and full sealed-input hash.
- **Files modified:** Phase 13 evidence, promotion, acceptance, and contract tests
- **Verification:** Focused suites passed 70/70; the authoritative producer and bundle were regenerated.
- **Committed in:** `6e8261a`

**2. [Rule 1 - Bug] Stabilized normalized content digests after changed-path classification**

- **Found during:** Task 2 simplification and diff review
- **Issue:** The first implementation calculated content digests from the provisional all-changed receipt before final changed/unchanged classification.
- **Fix:** Classified first with normalized digest fields empty, calculated content digests from the classified replacement set, then rendered and revalidated the final stable receipt.
- **Files modified:** `tools/xtask/src/phase13_evidence/promotion.rs`
- **Verification:** Stable recomputation and tampered-claim regressions passed.
- **Committed in:** `6e8261a`

**3. [Rule 1 - Bug] Extended strict provenance parsing for explicit digest modes**

- **Found during:** Task 5 canonical acceptance run `30277369306`
- **Issue:** The promoted manifest correctly declared `digest_mode`, but the independent provenance parser rejected the new field before live replay.
- **Fix:** Required `phase13_receipt_semantic_v2` only for the receipt and `exact_bytes_sha256` for every other promoted artifact; added missing, unknown, and wrong-mode regressions.
- **Files modified:** `tools/xtask/src/provenance/evidence_schema.rs`, `tools/xtask/tests/phase9_witness_provenance.rs`
- **Verification:** Focused regression, full provenance check, mandatory Rust sequence, and canonical rerun passed.
- **Committed in:** `dbaa648`

**Total deviations:** 3 auto-fixed bugs. **Impact:** Each fix strengthened the exact reviewed contract. The acceptance-only parser repair is outside both producer-affecting closures, so P/B/R/Q and the fresh acknowledgment remained valid.

## Failed and Superseded Audit History

| Run / candidate | Outcome | Preserved reason |
| --- | --- | --- |
| `30211150612` at `ced2134` | Failed | Temporary-repository negative tests omitted reviewed replay evidence. |
| `30211470256` at `32e2c93` | Failed | Test-only repair changed the producer-affecting replay closure after promotion. |
| `30211674242` at `d15191e` | Failed | Raw expanded catalog comparison treated reviewed debug capture drift as physics divergence. |
| Producer `2f51f7f`, run `30231510280` | Successful but superseded | Pre-acquisition review found incomplete content-digest, pointer, and exact-authority semantics. |
| `30277369306` at `241e75e` | Failed | Strict provenance parser had not declared the explicit Phase 13 digest mode. |

GitHub rejected raw full-SHA workflow dispatch refs with HTTP 422. Each dispatch therefore used `main` only after verifying the remote branch equaled the intended full SHA, and each resulting run's `headSha` was independently checked.

Native macOS Cargo compilation repeatedly stalled, so the already-authorized exact Rust 1.97 Docker fallback ran every required precommit sequence. Repository-wide `just markdown-check` also reports 11 unrelated pre-existing Markdown formatting failures; the changed non-GSD lesson file passed targeted mdformat and `.planning/**` remained parser-owned.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 13's exact-head provenance, reviewed replay, and canonical Linux acceptance gaps are closed with retained terminal evidence.
- Phase 14 can begin the Windows particle-group invariant repair from accepted head `dbaa64819debc5da268d32fcd342da7632ac6370`.
- No human verification or unresolved Phase 13 blocker remains.

## Self-Check: PASSED

- All key implementation, evidence, workflow, UAT, and verification files exist.
- Producer P, review base R, promotion Q, first acceptance head, and accepted A commits resolve in Git history.
- Downloaded terminal `identity.json` matches SHA-256 `46b42effa1def2a61095a25d88955c8fe7fcba677158039a09ef32208446d25f` and validates schema v2, exact P/B/R/Q/A identities, ordered step names, and seven successful results.

***

*Phase: 13-restore-evidence-and-replay-integrity*
*Completed: 2026-07-27*
