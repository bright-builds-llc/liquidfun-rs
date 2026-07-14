---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "22"
subsystem: rigid-comparator-review-and-oracle-readiness
tags: [differential, comparator, policy, review, github-actions, sanitizers]
requires:
  - phase: 08-21
    provides: pinned C++ execution for the strengthened Phase 8 corpus
provides:
  - closed inherited Phase 8 comparison and replay transaction
  - complete local debug, release, replay, D0, and sanitizer evidence
  - resolved Phase 8 review blockers and exact-identity Oracle CI readiness
affects: [08-23, canonical-evidence, rigid-sign-off]
tech-stack:
  added: []
  patterns: [stable first-divergence paths, field-specific locked residual policy, exact-run evidence identity]
key-files:
  created:
    - .planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-22-SUMMARY.md
  modified:
    - crates/liquidfun-differential/src/rigid_evidence/phase8.rs
    - crates/liquidfun-differential/src/rigid_fixtures.rs
    - crates/liquidfun-differential/tests/phase8_comparator.rs
    - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
    - protocol/fixtures/accepted/rigid-world-request.jsonl
    - protocol/tolerances/phase8-v1.toml
    - .planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-REVIEW.md
    - .github/workflows/oracle.yml
key-decisions:
  - "Classify near-zero iterative joint residuals with field-specific absolute/relative bounds locked by the Phase 8 parser; retain signed-zero distinction and exact structural comparison."
  - "Count lifecycle occurrences before ordered comparison so missing or duplicate events fail at a stable multiplicity path."
  - "Write one exact identity per successful Phase 8 Oracle job and defer every D1 claim to the Plan 08-23 exact-ref checkpoint."
requirements-completed: [RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T06:28:08Z
duration: 30min
completed: 2026-07-14
---

# Phase 8 Plan 22: Comparator, Local Evidence, Review Closure, and Oracle Readiness Summary

**Phase 8 comparison now covers the strengthened nineteen-family corpus, both review blockers are resolved by complete local evidence, and Oracle CI is ready for the human-controlled exact-ref checkpoint.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-07-14T05:58:00Z
- **Completed:** 2026-07-14T06:28:08Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Migrated rigid fixture staging, replay, minimization, and workflow tests from the Phase 7 boundary to the inherited Phase 8 comparator and policy identity.
- Added stable mismatch coverage for lifecycle multiplicity, topology, branch/reaction fields, signed zero, policy closure, and every A/B/C/D gear result lane.
- Diagnosed the first live revolute mismatch as a near-zero iterative residual in the deliberately overconstrained joint-definition witness, then locked reviewed field-specific bounds in both policy text and parser authority.
- Configured and built debug, release, and ASan/UBSan presets before use; passed protocol CTest, all three complete-corpus comparisons, replay, exactly two D0 runs, and the ordered Rust gate.
- Preserved the original review evidence while resolving CR-08-14-01 and CR-08-14-02 with implementation commits and exact local results.
- Prepared canonical and sanitizer jobs with Clang 22, Phase 8 labels, policy checks, read-only assertions, logs, Plan 08-23-compatible identities, and exact SHA-scoped success artifacts.

## Task Commits

1. **Task 08-22-01: Close comparator coverage and command contracts** - `dc809b9` (fix)
2. **Task 08-22-02: Run complete local evidence and resolve both review findings** - `427a4f6` (docs)
3. **Task 08-22-03: Prepare exact-identity Oracle CI after review closure** - `f6e16a6` (ci)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_evidence/phase8.rs` - Fails lifecycle count drift at a stable semantic path before ordered comparison.
- `crates/liquidfun-differential/src/rigid_fixtures.rs` - Runs staging, replay, and minimization through Phase 8 policy and inherited comparison.
- `crates/liquidfun-differential/tests/phase8_comparator.rs` - Exercises stable Phase 8 structural, numeric, lifecycle, gear, signed-zero, and fail-closed policy paths.
- `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs` and `tools/xtask/tests/differential_cli.rs` - Bind fixture and CLI behavior to Phase 8 artifacts.
- `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs` and `protocol/tolerances/phase8-v1.toml` - Lock the reviewed field-specific residual thresholds and canonical policy hash.
- `protocol/fixtures/accepted/rigid-world-request.jsonl` - Binds the accepted complete corpus to the reviewed Phase 8 policy hash.
- `.planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-REVIEW.md` - Records both findings resolved with zero blockers and local D2 evidence only.
- `.github/workflows/oracle.yml` - Prepares exact-identity Phase 8 canonical and sanitizer evidence jobs without dispatching them.

## Decisions Made

- Treated source-faithful near-zero solver residuals as field-specific policy questions only after auditing the Rust and pinned C++ equations and observing the complete overconstrained witness output.
- Kept every threshold explicit and parser-locked; wildcard paths, missing paths, threshold drift, structural drift, and signed-zero collapse remain rejected.
- Kept canonical publication separate from local completion. Local Apple Clang 21/CMake 3.27 results resolve implementation review but cannot satisfy the canonical Clang 22/CMake 4.3 D1 checkpoint.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Migrated the production fixture transaction to Phase 8**

- **Found during:** Focused xtask fixture verification
- **Issue:** The accepted request had already moved to the Phase 8 policy, but fixture staging and replay still loaded and compared only through Phase 7.
- **Fix:** Added Phase 8 policy loading and inherited comparison to staging, replay, minimization, and fixture tests.
- **Files modified:** `crates/liquidfun-differential/src/rigid_fixtures.rs`, `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs`, `tools/xtask/tests/differential_cli.rs`
- **Verification:** xtask differential CLI passed 27/27; rigid fixture workflow passed 15/15; the full ordered Rust gate passed.
- **Committed in:** `dc809b9`

**2. [Rule 1 - Bug] Reclassified source-faithful iterative residuals without hidden policy widening**

- **Found during:** Fresh oracle-debug complete-corpus comparison
- **Issue:** Four-ULP comparison rejected near-zero joint coordinate, speed, reaction force, and reaction torque residuals from the deliberately overconstrained joint-definition witness despite source-matched equations.
- **Fix:** Replaced those paths with reviewed field-specific absolute/relative bounds, documented their dimensional rationale, locked their exact bits in parser authority, and added threshold-widening regressions.
- **Files modified:** `protocol/tolerances/phase8-v1.toml`, `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs`, `protocol/fixtures/accepted/rigid-world-request.jsonl`
- **Verification:** Debug, release, replay, D0, and sanitizer evidence passed across all 19 required families; wildcard, missing, and widened policies still fail closed.
- **Committed in:** `dc809b9`

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug). **Impact:** Both changes were required to make the planned Phase 8 transaction internally consistent and evidence-backed; neither weakens structural, identity, ordering, or signed-zero guarantees.

## Issues Encountered

- The initial debug comparison exposed `joint-def-revolute` coordinate `0x00000000` versus pinned C++ `0x33a2c494`. Audit showed equivalent staged equations and a broader family of near-zero/cache residuals in the intentionally overconstrained witness, leading to the explicit policy correction above.
- Local tools remain noncanonical: CMake 3.27.9 and Apple Clang 21.0.0 differ from the required CMake 4.3.3 and Clang 22.1.8. All local results remain D2-supported.

## Validation Evidence

- Phase 8 comparator mutations: 6/6 passed.
- xtask differential CLI: 27/27 passed.
- Rigid fixture workflow: 15/15 passed.
- Fresh configure/build: `oracle-debug`, `oracle-release`, and `oracle-asan-ubsan` passed before evidence execution.
- Protocol CTest: debug 1/1 passed; fail-fast ASan/UBSan 1/1 passed.
- Complete corpus: debug compare, release compare, replay, and sanitizer compare each matched 19/19 required families under `phase8-v1`.
- D0: exactly one `verify-determinism --runs 2` command produced two byte-identical native and oracle-debug runs.
- Workflow: provenance check passed; YAML parsed; identity shape matched Plan 08-23 `jq`; exact identity/artifact counts and command order checks passed.
- Ordered Rust gate with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-target`: `cargo fmt --all`, warning-denied clippy, all-target/all-feature build, and all-feature tests passed (185 library tests, every integration target, and 13 doctests).
- `git diff --check`: passed.

## User Setup Required

None.

## Next Phase Readiness

- Plan 08-23 may push the exact clean commit, dispatch the existing human-controlled Oracle workflow, and validate only the two exact SHA-scoped artifacts.
- The requirements array above is copied from the plan contract; Phase 8 requirements remain pending canonical D1 evidence and final sign-off in Plans 08-23 and 08-24.
- This plan made no push, workflow dispatch, or compatibility claim.

## Self-Check: PASSED

- All three task commits exist and the task allowlists are clean.
- Both review findings are resolved with zero blocking/open findings.
- Lifecycle ID `8-2026-07-13T21-26-30` matches the plan.
- No canonical artifact was created or claimed locally.
- The only unrelated worktree change remains `.codex/tasks/todo.md`.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*
