---
phase: 11-examples-headless-tooling-and-testbed
plan: "18"
subsystem: headless-package-isolation
tags: [headless, package, cargo-metadata, archive, ci, observations]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "08"
    provides: Public bounded semantic world observations
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "09"
    provides: Stable renderer-neutral debug primitives
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "14"
    provides: Native capture, exact replay, and semantic comparison
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "15"
    provides: Bounded headless catalog command surface
provides:
  - Default-feature public observability gate over counts, contacts, particles, broad phase, statistics, and debug primitives
  - Headless named and seeded resolution, controller, family execution, capture, replay, comparison, and oracle-prerequisite gate
  - Metadata, archive, evidence, and extracted-crate package verifier with submodule-free CI enforcement
affects: [phase11-testbed, phase11-evidence, release-packaging, cargo-ci]
tech-stack:
  added: []
  patterns:
    - Verify the publishable dependency graph from Cargo metadata and the normalized packaged manifest
    - Preserve repository-backed unit-test evidence through byte-identical package-local test data
key-files:
  created:
    - crates/liquidfun/tests/phase11_public_observability.rs
    - crates/liquidfun-differential/tests/headless_catalog.rs
    - tools/xtask/src/package/metadata.rs
    - crates/liquidfun/src/particle/testdata/group-topology-witnesses.json
  modified:
    - tools/xtask/src/package.rs
    - tools/xtask/tests/package_cli.rs
    - .github/workflows/ci.yml
    - crates/liquidfun/Cargo.toml
key-decisions:
  - "Keep the established empty non-default differential-internals feature while requiring default features to remain empty and rejecting private, renderer, native, benchmark, protocol, or testbed dependency activation."
  - "Package exact copies of the two Phase 10 witness artifacts and byte-check them against canonical repository artifacts before every real package verification."
  - "Treat unavailable C++ oracle tooling as exit-69 prerequisite diagnostics; native Rust execution remains valid but is never labeled a comparison match."
patterns-established:
  - "Package verification: metadata policy, bounded regular-file archive inspection, normalized manifest policy, evidence equality, then isolated build and test."
  - "Pre-renderer gate: public API, native controller/capture/comparison, package CLI, and isolated package verification run together in submodule-free CI."
requirements-completed: [RIGD-10, EXMP-02, EXMP-03, EXMP-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T05:08:00Z
duration: 48 min
completed: 2026-07-22
---

# Phase 11 Plan 18: Headless Capability and Package Isolation Summary

**The complete renderer-free catalog workflow and the standalone published crate now pass one submodule-free CI gate, including an extracted `.crate` build and test.**

## Performance

- **Duration:** 48 min
- **Started:** 2026-07-22T04:20:00Z
- **Completed:** 2026-07-22T05:08:00Z
- **Tasks:** 1
- **Files modified:** 14

## Accomplishments

- Added public default-feature coverage for exact world counts, broad-phase tree metrics, current rigid and particle contacts, broad-phase records, particle statistics, and stable debug primitives.
- Added five end-to-end headless tests covering named and seeded resolution, pause/step/restart/settings/action controller semantics, eight representative rigid/joint/rope/particle/group/query/callback/mutation families, exact capture replay, semantic comparison, and explicit missing-oracle diagnostics.
- Extended package verification to inspect Cargo metadata and the normalized archive manifest, require `liquidfun` as the sole publishable default member, reject forbidden or path dependencies and forbidden feature activation, and bound archive entry count and expanded size.
- Made the Phase 10 witness-backed unit tests self-contained by packaging byte-identical reviewed JSON artifacts, with verifier-enforced equality to the canonical repository evidence.
- Moved the focused public/headless/package gate into the existing `submodules: false` Cargo CI job before later presentation work.

## TDD Evidence

- **RED:** The new package CLI tests failed because a private `liquidfun-differential` dependency and a second default workspace member both unexpectedly passed the prior archive-only verifier.
- **GREEN:** The package CLI suite passes 8/8, the public suite passes 1/1, the headless suite passes 5/5, and `cargo xtask package verify` reports 171 entries built and tested outside the repository.
- **REFACTOR:** Cargo metadata policy moved into a focused module; package orchestration remains cohesive at 510 lines, while metadata validation is isolated at 248 lines.

The intentionally failing RED state was not committed because repository policy requires every commit to follow a completely passing ordered Rust gate.

## Task Commits

1. **Task 1: Close public workflow, headless E2E, and package isolation before renderer selection** - `5dec76b` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun/tests/phase11_public_observability.rs` - Default-feature observation and renderer-neutral primitive contract.
- `crates/liquidfun-differential/tests/headless_catalog.rs` - Complete controller, family, capture, replay, compare, seed, and missing-oracle headless gate.
- `tools/xtask/src/package.rs` - Bounded archive inspection, evidence equality, environment isolation, correct target-directory archive lookup, and extracted build/test.
- `tools/xtask/src/package/metadata.rs` - Workspace metadata and normalized packaged-manifest dependency/feature policy.
- `tools/xtask/tests/package_cli.rs` - Metadata, archive, environment, and CI contract tests.
- `crates/liquidfun/src/particle/testdata/group-topology-witnesses.json` - Exact package-local copy of reviewed Phase 10 topology evidence.
- `crates/liquidfun/src/particle/testdata/group-topology-witnesses.provenance.json` - Exact package-local copy of the evidence provenance envelope.
- `crates/liquidfun/Cargo.toml` - Includes the package-local JSON evidence in the published archive.
- `.github/workflows/ci.yml` - Runs the focused gate in the submodule-free quality job.

## Decisions Made

- Kept graphical, testbed, benchmark, protocol, differential harness, and native dependencies entirely outside the publishable crate while preserving the established empty diagnostic feature required by private repository tests.
- Validated the source workspace and the normalized archive separately so workspace inheritance or package normalization cannot conceal a forbidden dependency or feature.
- Rejected non-regular archive records before extraction, capped records and expanded bytes, and removed display and repository-control environment variables from extracted build/test subprocesses.
- Kept the missing-oracle case distinct from physics comparison: native execution remains available, but exit 69 states the exact prerequisite and no Match outcome is constructed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Packaged Phase 10 tests depended on repository-only witness paths**

- **Found during:** Task 1 package verification
- **Issue:** Eight `include_str!` calls reached `../../reference/artifacts/phase10`, so the extracted published crate could not compile its tests.
- **Fix:** Added byte-identical package-local witness and provenance files, redirected the assertions to those files, and made real package verification reject any drift from canonical repository artifacts.
- **Files modified:** `crates/liquidfun/Cargo.toml`, four particle test modules, two package-local evidence files, and `tools/xtask/src/package.rs`
- **Verification:** Both file pairs have identical SHA-256 values; extracted package build and test pass.
- **Committed in:** `5dec76b`

**2. [Rule 1 - Bug] Package verifier could select a stale archive under an isolated Cargo target**

- **Found during:** Task 1 package verification
- **Issue:** `cargo package` honored `CARGO_TARGET_DIR`, but the verifier always reopened `repository/target/package`, allowing a stale archive to mask current source changes.
- **Fix:** Resolve the generated `.crate` from the same absolute or repository-relative `CARGO_TARGET_DIR` used by Cargo.
- **Files modified:** `tools/xtask/src/package.rs`
- **Verification:** The isolated target produces and verifies the current 171-entry archive.
- **Committed in:** `5dec76b`

**Total deviations:** 2 auto-fixed (1 blocking issue, 1 correctness bug). **Impact:** Both fixes were necessary to make the planned package evidence current, self-contained, and trustworthy; no renderer or consumer dependency was added.

## Issues Encountered

- The initial public test used two obsolete diagnostic accessor names; it was corrected to the public `tree_balance` and `tree_quality` API before the behavioral RED run.
- The initial native scenario-action selection pointed at an already-applied setup action; the test now selects the declared first logical action and proves the intended controller boundary.

## Security Verification

- Cargo metadata and packaged manifests reject path dependencies plus private, renderer, window, game-engine, C++ bridge, benchmark, protocol, and testbed dependencies.
- Archive inspection permits only normalized regular files/directories, rejects traversal, links, native sources, and forbidden prefixes, and enforces reviewed entry and unpacked-byte limits before extraction.
- Extracted builds and tests run under a temporary target without repository, third-party, reference, display, or package-test control variables.
- Missing oracle execution is an explicit bounded prerequisite diagnostic and cannot be spoofed as a semantic match.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-18's `RIGD-10`, `EXMP-02`, `EXMP-03`, and `EXMP-06` mappings are implemented and recorded in summary frontmatter. Global requirement checkboxes remain unchanged until the remaining Phase 11 visual and evidence plans close their complete scopes.

## User Setup Required

None - no graphical environment, initialized upstream checkout, or external service is required.

## Next Phase Readiness

- The later testbed renderer can consume the already-proven semantic primitive and controller surfaces without altering simulation, capture, comparison, or package authority.
- The submodule-free CI gate now blocks regressions in ordinary consumer isolation before later visual work.
- No blocker remains for Plan 11-19.

## Self-Check: PASSED

- Confirmed all four primary created artifacts exist and implementation commit `5dec76b` is present.
- Confirmed public tests pass 1/1, headless tests pass 5/5, package CLI tests pass 8/8, focused deny-warnings Clippy passes, and workflow YAML parses.
- Confirmed `cargo xtask package verify` byte-checks both evidence files and builds/tests the current 171-entry extracted archive outside the repository.
- Confirmed exact ordered `cargo fmt --all`, full all-targets/all-features deny-warnings Clippy, all-targets/all-features build, and all-features test gates pass with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-18`.
- Confirmed the four fenced pre-existing edits remain unstaged and uncommitted.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
