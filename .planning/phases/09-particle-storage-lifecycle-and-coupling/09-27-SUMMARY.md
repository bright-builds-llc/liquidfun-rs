---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "27"
subsystem: phase9-portable-evidence
tags: [particles, differential, evidence, exact-ref, archives, integrity]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "26"
    provides: typed executable coverage for all 58 reviewed Phase 9 semantic branches
provides:
  - digest-bound retained Phase 8 and complete Phase 9 proof for every evidence case
  - one bounded typed validator shared by local, pre-identity, and exact-ref evidence paths
  - fresh local canonical and fail-fast sanitizer evidence for seven cases and 58 semantic bindings
affects: [phase-09-verification, exact-ref-publication, evidence-promotion]
tech-stack:
  added: []
  patterns: [identity-last evidence closure, canonical semantic digests, typed exact-ref metadata, archive preflight]
key-files:
  created:
    - tools/xtask/src/phase9_evidence.rs
    - tools/xtask/tests/phase9_evidence_cli.rs
  modified:
    - crates/liquidfun-differential/tests/phase9_corpus.rs
    - scripts/phase9-evidence.sh
    - tools/xtask/src/main.rs
key-decisions:
  - "The typed xtask validator owns reusable manifest, payload, trace, identity, exact-ref metadata, and archive validation; the shell runner only orchestrates generation and writes identity last."
  - "Every case binds retained Phase 6/7/8 policy identities, a successful phase8-v1 rigid result, exact semantic witnesses, persisted request/result/comparison payloads, and the complete ordered case manifest."
  - "Exact-ref authority requires closed run, job, artifact, archive, live-snapshot, and approved-head agreement, with historical runs 29439515367 and 29583793056 explicitly denied."
patterns-established:
  - "Evidence identity is valid only after typed content validation succeeds over the exact bounded regular-file set."
  - "Semantic witness assertions are validated both structurally and against decoded native/oracle result observations."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T00:58:21Z
duration: 37m
completed: 2026-07-18
---

# Phase 9 Plan 27: Portable Evidence Validator Summary

**Every Phase 9 case now carries portable retained-rigid and semantic proof, closed by one typed local/exact-ref validator before identity is emitted.**

## Performance

- **Duration:** 37m
- **Started:** 2026-07-18T00:20:57Z
- **Completed:** 2026-07-18T00:58:21Z
- **Tasks:** 3
- **Files modified:** 5 implementation and test files

## Accomplishments

- Extended all seven evidence records with exact Phase 6, 7, and 8 policy digests, a successful `phase8-v1` retained-rigid comparison digest, 58 typed semantic witness bindings, and persisted request/native/oracle/complete-comparison payload hashes.
- Bound the ordered complete case records into one canonical semantic manifest digest shared by canonical and sanitizer evidence.
- Added `cargo xtask phase9-evidence validate` for local and exact-ref evidence with typed bounded deserialization, normalized target-relative paths, exact regular-file inventories, symlink rejection, trace and identity closure, payload decoding, semantic result checks, and digest recomputation.
- Closed exact-ref authority over the approved full SHA, workflow dispatch, named successful jobs, unique unexpired artifacts, API digests, live metadata snapshots, archive contents, and the two explicitly denied historical run IDs.
- Replaced duplicated shell predicates with `phase9-evidence validate-content`, so the identity-last runner uses the same typed semantic and payload checks before creating `identity.json`.
- Produced fresh local canonical and ASan/UBSan evidence and validated seven cases, exactly 58 unique semantic bindings, and identical canonical/sanitizer manifests.

## Task Commits

1. **Task 1: Emit retained-rigid and semantic witness records into every case** - `c521ec9`
2. **Task 2: Implement one bounded local and exact-ref evidence validator** - `b812ab9`
3. **Task 3: Produce and validate fresh local canonical and sanitizer evidence** - `6f9f96f`

**Plan metadata:** committed after summary and state tracking verification.

## Verification

- `cargo test -p liquidfun-differential --test phase9_corpus retained_rigid -- --nocapture` - 2 passed
- `cargo test -p liquidfun-differential --test phase9_corpus witness_binding -- --nocapture` - 11 passed
- `cargo test -p xtask --test phase9_evidence_cli` - 9 passed
- Exact ordered Rust gate: `cargo fmt --all`, Clippy with all targets/features and warnings denied, all-target build, and all-feature tests including doctests
- Fresh `oracle-debug` and `oracle-release` configure/build
- Canonical complete corpus - 24 passed, 0 failed, 1 ignored regeneration tool
- Fresh `oracle-asan-ubsan` configure/build and protocol-test target
- Fail-fast ASan/UBSan protocol CTest - 1 passed, 0 failed
- Complete fail-fast sanitizer corpus - 24 passed, 0 failed, 1 ignored regeneration tool
- Local canonical/sanitizer evidence validation - 7 cases and 58 semantic bindings verified
- `cargo xtask provenance check`
- `git diff --exit-code -- third_party/liquidfun`
- `git diff --check`

Local oracle builds reported the existing noncanonical development identities: CMake 3.27.9 and Apple Clang 21 rather than canonical CMake 4.3.3 and Clang 22.1.8. The resulting evidence remains local and no compatibility authority was promoted.

## Files Modified

- `crates/liquidfun-differential/tests/phase9_corpus.rs` - Emits versioned retained-rigid, semantic-witness, payload, and manifest digests; tests omission and corruption; asserts typed validation precedes identity emission.
- `scripts/phase9-evidence.sh` - Persists case payloads, delegates content closure to xtask, and writes the complete identity inventory last.
- `tools/xtask/src/phase9_evidence.rs` - Implements bounded local, pre-identity, and exact-ref validation of semantic content, metadata, identities, and archives.
- `tools/xtask/src/main.rs` - Routes the closed `phase9-evidence` command.
- `tools/xtask/tests/phase9_evidence_cli.rs` - Covers valid local/exact-ref evidence and run, metadata, file, symlink, identity, semantic, policy, payload, and digest corruption.

## Decisions Made

- Canonical serialization of typed records, rather than generic JSON map order, defines every semantic and retained-rigid digest.
- Native and oracle payloads are independently decoded and checked against their request; collision-energy and stuck-candidate assertions must also hold in the bound decoded checkpoint observations.
- Archive entries are inspected before any extraction and must exactly match the extracted evidence contract; unsafe absolute, traversal, symlink, extra, and missing entries fail closed.
- Local evidence uses run ID `0` and head `local`; exact-ref identities must instead equal the reviewed run ID and approved 40-character head.
- The evidence runner retains shell only for orchestration, hashing the final inventory, and identity emission; reusable evidence policy lives in the typed Rust module.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Bound semantic assertions to decoded result values**

- **Found during:** Task 2 validator review
- **Issue:** Structural witness validation proved positive/nonempty assertion parameters but did not independently prove the decoded native and oracle statistics satisfied collision-energy and stuck-candidate assertions.
- **Fix:** Located the bound checkpoint statistics in each decoded result and required positive finite energy plus all stable stuck-candidate identities.
- **Files modified:** `tools/xtask/src/phase9_evidence.rs`
- **Verification:** Valid local evidence passes; zero-energy and empty-stuck mutation cases fail closed.
- **Committed in:** `b812ab9`

**2. [Rule 3 - Blocking] Updated runner contract after consolidating validation**

- **Found during:** Task 3 first canonical evidence run
- **Issue:** The runner contract test still required removed grep predicates after the shared typed validator replaced them.
- **Fix:** Asserted that `phase9-evidence validate-content` occurs before the identity write and retained command-failure/failed-log no-identity behavior tests.
- **Files modified:** `crates/liquidfun-differential/tests/phase9_corpus.rs`
- **Verification:** The focused contract test and complete canonical/sanitizer corpora pass.
- **Committed in:** `6f9f96f`

***

**Total deviations:** 2 auto-fixed (1 Rule 2 critical evidence check, 1 Rule 3 blocking contract update)
**Impact on plan:** Both changes strengthened the requested shared fail-closed contract without adding publication or compatibility-authority scope.

## Issues Encountered

- The first canonical run intentionally stopped before identity because the old static contract test detected the validator consolidation. After the contract was updated to the new typed boundary, the complete canonical rerun passed and emitted identity.
- Typed semantic mutation tests initially recomputed digests through generic JSON values, whose key ordering differed from the contract serialization. The tests now recompute through the same typed records as production.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

- Phase 9 has portable local evidence ready for independent exact-ref publication and promotion workflows.
- The validator can consume downloaded exact-ref metadata and archives unchanged while explicitly rejecting the two superseded historical runs.
- No publication, remote dispatch, ledger mutation, compatibility promotion, or upstream source modification occurred.

## Self-Check: PASSED

- All five implementation/test paths and this summary exist.
- Task commits `c521ec9`, `b812ab9`, and `6f9f96f` exist in repository history.
- Every focused, canonical, sanitizer, full Rust, provenance, upstream read-only, and diff gate passed.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-18*
