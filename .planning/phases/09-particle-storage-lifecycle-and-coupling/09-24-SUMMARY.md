---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "24"
subsystem: phase9-evidence-promotion
tags: [particles, compatibility, differential, sanitizer, supply-chain]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "23"
    provides: approved exact-ref canonical and sanitizer evidence pair
provides:
  - fail-closed Phase 9 evidence authority bound to one approved run and SHA
  - promoted evidence references for four supported compatibility rows
  - regression rejection for superseded, partial, substituted, and Phase 10 authority
affects: [phase-10, compatibility-inventory, oracle-ci]
tech-stack:
  added: [cargo-deny 0.20.2 policy]
  patterns: [exact authority allowlist, rejected-authority denylist, byte-idempotent inventory generation]
key-files:
  created:
    - deny.toml
  modified:
    - TESTING.md
    - reference/compatibility.json
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs
key-decisions:
  - "Promote only the four scoped Phase 9 rows using run 29583793056, full SHA b27fc14f6b29fb82ca815fa1effba71bae09d424, and its exact canonical/sanitizer artifact pair."
  - "Reject superseded run 29439515367 and preserve every Phase 10 group, topology, pair, triad, source-area, and solver row as not_evidenced."
  - "Make cargo-deny meaningful with an explicit permissive-license allowlist and deny unknown registry or git sources."
patterns-established:
  - "A Phase 9 promotion is valid only when both passing traces, the byte-identical complete manifest, native/C++ comparison, approved head, run, jobs, and artifacts match the pinned authority set."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-17T14:04:19Z
duration: 22 min
completed: 2026-07-17
---

# Phase 9 Plan 24: Evidence Promotion and Closure Summary

**Phase 9 compatibility evidence now fails closed around one approved exact-ref canonical/sanitizer pair, promotes only four supported rows, and keeps all Phase 10 behavior explicitly unpromoted.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-07-17T13:42:46Z
- **Completed:** 2026-07-17T14:04:19Z
- **Tasks:** 3
- **Files modified:** 6 implementation, evidence, policy, and planning files

## Accomplishments

- Independently re-queried run `29583793056`, both successful authority jobs, and both unexpired artifact records before changing the compatibility ledger.
- Recomputed both artifact archive identities, trace hashes, and manifest hashes; proved passing `7 passed; 0 failed; 1 ignored` logs, seven cases, 58 unique branches, 22 policy paths per case, and valid request/native/oracle/comparison digests.
- Documented immutable approved authority in `TESTING.md`, including the explicit rejection of superseded run `29439515367`.
- Rebound only the two supported public-API rows and two supported particle subsystem rows to the approved authority.
- Added fail-closed inventory validation and tests for superseded runs, failed traces, incomplete manifests, substituted artifacts, partial authority, stale differential references, and every deferred Phase 10 row family.
- Regenerated and checked the inventory twice with byte-identical output while leaving parser-owned `COMPATIBILITY.md` untouched by hand.
- Passed native Rust, differential, debug/release oracle, ASan/UBSan, provenance, inventory, supply-chain, schema, workflow, Markdown, and upstream read-only gates.

## Task Commits

1. **Task 1: Validate and record approved Phase 9 authority** - `2655f95`
1. **Task 2: Promote only supported rows with fail-closed inventory validation** - `1b11ebb`
1. **Task 3: Run closure/security gates and restore the missing dependency policy** - `c9cf0d5`

**Plan metadata:** committed after summary and state tracking verification.

## Authority Promoted

- **Approved SHA:** `b27fc14f6b29fb82ca815fa1effba71bae09d424`
- **Workflow run:** `29583793056`
- **Canonical artifact:** ID `8408156562`, API digest `sha256:faaf24c870826251f0dd1d507ba9c335269b78433ba1ce2ee0e1995336f0139a`
- **Sanitizer artifact:** ID `8408174081`, API digest `sha256:f4b30cebed7b81a41282a33d45b81231485a2fa0c3a958c7b68a3ecbad086e7c`
- **Canonical trace:** `5c6805e0e998394947439bf6eb295526130cb4db81a67e0f560c6d6bc3f33545`
- **Sanitizer trace:** `4261dbe8993155dd7ab9e7992f90bef60e57762c36a66ea97b3bc9131804508a`
- **Shared manifest:** `b7bb43a6ce083fe543bf6eb3f92b1b4f663d4bd6520dbb83c3a00072a3010a8b`
- **Pinned upstream:** `7f20402173fd143a3988c921bc384459c6a858f2`

The validator accepts this authority only as a complete set. It rejects old run and artifact identities, old failed trace digests, the incomplete former manifest, the Plan 09-16 differential reference, partial sets, substituted artifacts, and any Phase 10 evidence promotion.

## Closure Map

| Gap | Executable regression or evidence |
| --- | --- |
| `G09-STEP-GUARD` / `WR-01` | `particle_step_guards` integration tests and the executable Phase 9 corpus |
| `G09-ZOMBIE-AUTHORITY` / `WR-02` | `particle_zombie_authority` integration tests and lifecycle corpus case |
| `G09-EVICTION-OCCURRENCE` / `WR-03` | capacity-eviction regressions, independent property model, and lifecycle corpus case |
| `G09-PERMUTATION-WEIGHTS` / `WR-04` | `particle_permutation_coherence`, permutation properties, and storage corpus case |
| `G09-PROTOCOL-VALIDATION` / `WR-05..08` | request/result mutation tests across Rust codec, native runner, and C++ oracle |
| `G09-DIFFERENTIAL-COMPARISON` / `CR-01` | same-request native-versus-C++ semantic comparator with fully consumed policies |
| `G09-EXECUTABLE-COVERAGE` / `CR-02` | seven digest-bound cases covering exactly 58 retained branches |
| `G09-EVIDENCE-PIPELINE` | exact-ref canonical and ASan/UBSan artifacts with passing traces and identical manifests |

Cross-engine stable-ID rotation remains deferred to Phase 10, as do public particle groups, topology, pairs, triads, source-area behavior, and particle solver families.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features` including 16 doctests
- `cargo test -p xtask --test inventory_cli` - 17 passed
- `cargo xtask inventory generate && cargo xtask inventory check` twice with byte-identical generated report
- Fresh `oracle-debug` and `oracle-release` configure/build
- Canonical local evidence - 7 passed, 0 failed, 1 ignored
- Fresh `oracle-asan-ubsan` configure/build, protocol CTest, and evidence - 7 passed, 0 failed, 1 ignored
- `cargo test -p liquidfun-differential --test phase9_corpus` - 7 passed, 0 failed, 1 ignored
- `cargo xtask provenance check`
- `cargo xtask inventory check`
- `cargo deny check` - advisories, bans, licenses, and sources passed
- Schema drift verification - `drift_detected:false`, `blocking:false`
- `actionlint .github/workflows/oracle.yml`
- `just markdown-check`
- `git diff --exit-code -- third_party/liquidfun`
- `git diff --check`

Local macOS oracle runs correctly reported noncanonical CMake 3.27.9 and Apple Clang 21 identities. Those local D2 runs were used only for verification; promoted D1 authority remains the approved Linux artifacts with CMake 4.3.3 and Clang 22.1.8.

## ASVS L1 Review

No high-severity findings were identified.

- JSONL requests and results are schema-validated, bounded, fail-fast, and compared from the same request bytes.
- Canonical and sanitizer subprocesses have explicit modes, result validation, and successful fail-fast sanitizer coverage.
- Workflow authority is read-only, exact-SHA/run/job/artifact bound, and digest checked before promotion.
- Evidence output accepts only fixed relative paths, rejects absolute and parent traversal, and emits protocol data separately from diagnostics.
- The workflow uses `permissions: contents: read`; no secret values enter evidence logs or artifact identity.
- Shell inputs are constrained to the `canonical`/`sanitizer` enum and fixed repository commands; actionlint passed and no dynamic evaluation is used.
- The newly configured dependency policy denies unknown registries and git sources while allowing only reviewed permissive licenses.

## Files Created/Modified

- `TESTING.md` - Records immutable approved run, job, artifact, digest, and validation procedure.
- `reference/compatibility.json` - Rebinds the four supported Phase 9 rows to approved authority.
- `tools/xtask/src/inventory/validation.rs` - Enforces the complete authority set and rejected-authority denylist.
- `tools/xtask/tests/inventory_cli.rs` - Covers accepted authority and every fail-closed rejection path.
- `deny.toml` - Defines all-feature advisory, license, bans, and dependency-source policy.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-24-SUMMARY.md` - Records Phase 9 closure.

## Decisions Made

- Only run `29583793056` at the approved full SHA and its exact canonical/sanitizer artifact pair can support Phase 9 promotion.
- Passing logs and complete identical manifests are content requirements, not merely links attached to compatibility rows.
- Only four rows were rebound because they are the rows fully supported by the Phase 9 evidence contract; row status was not inflated.
- All Phase 10 rows remain `not_evidenced`, and the inventory validator treats any attempted Phase 9 promotion of them as an error.
- Private workspace path dependencies may remain unversioned; unknown external registry and git sources are denied.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added the missing cargo-deny policy**

- **Found during:** Task 3 full verification
- **Issue:** `cargo deny check` was required by the plan, but `cargo-deny` was not installed and the repository had no configuration. After installing the pinned `0.20.2`, default license policy rejected every dependency, including MIT and Apache-2.0.
- **Fix:** Installed the pinned tool and added a minimal reviewed policy covering all workspace features, permissive licenses, advisories, and trusted dependency sources.
- **Files modified:** `deny.toml`
- **Commit:** `c9cf0d5`

## Issues Encountered

- `cargo deny check` reports one non-blocking duplicate-version warning for `winnow` through the pinned TOML dependency graph. All four deny checks pass; changing transitive dependency resolution is outside this evidence-promotion plan.

## User Setup Required

None.

## Next Phase Readiness

- All Phase 9 G09/CR/WR closure items have executable regression or exact-ref evidence.
- The compatibility ledger is bound to passing complete authority and rejects known superseded evidence.
- Phase 10 can add groups, topology, pairs, triads, stable-ID rotation evidence, source-area behavior, and solver families without inheriting a false Phase 9 claim.

## Self-Check: PASSED

- `TESTING.md`, `reference/compatibility.json`, inventory validation/tests, `deny.toml`, and this summary exist.
- Task commits `2655f95`, `1b11ebb`, and `c9cf0d5` exist in repository history.
- Canonical and sanitizer manifests are byte-identical and contain seven cases, 58 unique branches, and 22 distinct policy paths per case.
- Inventory generation is byte-idempotent, all Phase 10 rows remain `not_evidenced`, and the superseded authority is rejected.
- All required Rust, oracle, sanitizer, provenance, inventory, dependency, schema, workflow, Markdown, upstream read-only, and diff gates passed.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-17*
