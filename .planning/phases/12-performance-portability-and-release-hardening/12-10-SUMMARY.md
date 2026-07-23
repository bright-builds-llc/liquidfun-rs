---
phase: 12-performance-portability-and-release-hardening
plan: "10"
subsystem: safety-evidence
tags: [typed-validation, regression-registry, coverage, path-confinement, release-audit]
requires:
  - phase: 12-17
    provides: exact nightly-2026-07-15 identity shared by unstable safety and coverage evidence
  - phase: 12-09
    provides: closed finding classifications and minimized regression handoff
provides:
  - reusable typed regression manifest, result-set, coverage contract, and coverage-record validators
  - closed tracked-registry and coverage xtask commands
  - exact candidate-bound result-directory validation with identity-last publication
  - deterministic complete regression execution-list projection
affects: [phase-12-release-audit, phase-12-evidence-lanes, phase-12-regression-runner]
tech-stack:
  added: []
  patterns: [functional validation core with thin CLI shell, identity-last evidence publication, distinct non-parity evidence kinds]
key-files:
  created:
    - tools/xtask/src/safety_evidence.rs
    - tools/xtask/src/safety_evidence/contract.rs
    - tools/xtask/tests/safety_evidence_contract.rs
    - reference/regressions/manifest.toml
    - reference/coverage/contract.json
  modified:
    - tools/xtask/src/main.rs
key-decisions:
  - "Keep the tracked regression registry truthfully empty until a real minimized reviewed finding exists; do not invent parity evidence."
  - "Require result paths to equal target/phase12-regressions/<candidate> exactly and publish identity.json only after complete typed validation."
  - "Keep Rust sanitizer, C++ ASan/UBSan, Rust coverage, C++ coverage, and differential coverage as five distinct non-parity evidence kinds."
patterns-established:
  - "Shared authority: CLI adapters and the later release validator call the same typed contract functions."
  - "Evidence confinement: normalized ordinary paths, no symlink components, exact byte hashes, and bounded regular files precede authority."
requirements-completed: [API-12, TEST-06, TEST-07, TEST-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T21:23:29Z
duration: 15m
completed: 2026-07-23
---

# Phase 12 Plan 10: Typed Safety, Regression, and Coverage Evidence Summary

**One reusable typed authority now validates reviewed regressions, confined candidate results, and five distinct non-parity safety and coverage evidence kinds before release aggregation.**

## Performance

- **Duration:** 15m
- **Started:** 2026-07-23T21:08:53Z
- **Completed:** 2026-07-23T21:23:29Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added a strict regression manifest v1 with exact field registry, minimized-byte hash verification, closed finding classes, candidate/fix provenance, review state, first-divergence signature, and named test identity.
- Added typed coverage contract and produced-record validation that keeps Rust sanitizer, C++ ASan/UBSan, Rust coverage, C++ coverage, and differential leaves separate with `parity_authority: false`.
- Added closed `validate-regressions`, deterministic `--emit-execution-list`, confined `validate-regression-results`, and `validate-coverage` commands.
- Rejects absolute, parent, alternate-root, candidate-mismatched, symlinked, incomplete, duplicated, unregistered, mixed-identity, pre-identified, and unexpected result sets.
- Verifies confined coverage artifact bytes and toolchain identities through the same typed core reserved for the later release validator.

## TDD Evidence

- Thirteen focused tests cover valid tracked authorities and result sets plus unknown fields, duplicate records, wrong hashes, invalid classifications, merged evidence kinds, parity claims, incomplete leaves, unstable projections, missing command registration, and every results-path substitution.
- Tests were developed with the implementation and no failing RED state was committed, as required by the plan.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define safety, regression, and coverage evidence contracts** - `96006ce` (feat)

## Files Created/Modified

- `tools/xtask/src/safety_evidence/contract.rs` - Shared typed validation core for manifests, result sets, coverage authorities, produced records, hashes, and confinement.
- `tools/xtask/src/safety_evidence.rs` - Thin fixed-path CLI shell and identity-last result publication.
- `tools/xtask/src/main.rs` - Registers the closed `safety-evidence` command family.
- `tools/xtask/tests/safety_evidence_contract.rs` - Exercises schema negatives, deterministic projection, CLI registration, and exact path-confinement cases.
- `reference/regressions/manifest.toml` - Versioned reviewed regression authority with a complete exact record-field registry.
- `reference/coverage/contract.json` - Versioned five-kind coverage authority with explicit no-parity semantics.

## Decisions Made

- The regression registry begins empty because the repository has no accepted real minimized physics mismatch; empty deterministic output is more truthful than inventing a regression.
- Result-set validation accepts one canonical lowercase full SHA and exactly one normalized candidate directory beneath `target/phase12-regressions`.
- `identity.json` is forbidden before validation and created with `create_new` only after the complete result set passes.
- Produced coverage records must bind exact artifact bytes, candidate commit, section-appropriate toolchain identity, complete exercised/missed subsystem leaves, and one of five non-overlapping evidence kinds.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Bound produced coverage records to confined artifact bytes**

- **Found during:** Task 1 simplification and security review.
- **Issue:** The initial produced-record validator checked artifact hash syntax but did not read and hash the referenced confined artifact.
- **Fix:** Added ordinary-path component checks, canonical repository confinement, bounded regular-file reads, exact SHA-256 verification, duplicate artifact rejection, and evidence-kind-specific toolchain binding.
- **Files modified:** `tools/xtask/src/safety_evidence/contract.rs`, `tools/xtask/tests/safety_evidence_contract.rs`.
- **Verification:** Focused tests reject wrong artifact hashes and mixed toolchains; all 13 contract/CLI tests pass.
- **Committed in:** `96006ce`.

**Total deviations:** 1 auto-fixed (1 missing critical)

**Impact on plan:** The correction strengthens the planned tamper boundary without widening CLI paths, adding dependencies, or changing production behavior.

## Verification

- `cargo test -p xtask --test safety_evidence_contract` - 13 passed.
- `cargo clippy -p xtask --all-targets --all-features -- -D warnings` - passed.
- `cargo xtask safety-evidence validate-regressions` - passed with the truthful zero-entry reviewed registry.
- Two `validate-regressions --emit-execution-list` outputs were byte-identical and parsed as the complete empty JSON list.
- `cargo xtask safety-evidence validate-regression-results --help` - closed candidate/path interface registered.
- `cargo xtask safety-evidence validate-coverage` - passed with `parity_authority=false`.
- Exact ordered repository gate passed: format, warning-denied Clippy, all-target build, and all-feature tests.
- Prohibited network, unsafe, and `unwrap()` pattern scan passed.
- Staged diff contained exactly the six plan-owned files and passed `git diff --check`.

## Known Stubs

None. `regressions = []` is the validated truthful authority state until a real minimized finding completes review, not an unwired placeholder.

## Issues Encountered

- Directly compiling the shared contract module inside the integration test made future Plan 12-15 APIs appear used in tests but unused in the current binary. Reasoned narrow `dead_code` allowances document that planned reuse seam while strict Clippy remains clean.

## User Setup Required

None - all validators use tracked repository authorities and ordinary local filesystem inputs.

## Next Phase Readiness

- Plan 12-15 can call the same typed functions directly when aggregating release evidence.
- Plan 12-25 can consume the deterministic reviewed execution list without shell-owned parsing.
- Plan 12-24 can produce the five distinct evidence record kinds without granting coverage parity authority.
- No Plan 12-10 blockers remain.

## Self-Check: PASSED

- All six implementation/authority files and this summary exist.
- Task commit `96006ce` exists in repository history.
- Stub scan found no incomplete implementation markers; the intentionally empty reviewed registry is documented above.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
