---
phase: 11-examples-headless-tooling-and-testbed
plan: "29"
subsystem: testing-tooling
tags: [package-isolation, headless-ci, corpus-closure, evidence-audit, cargo-deny]
requires:
  - phase: 11-18
    provides: bounded failure evidence and renderer-neutral comparison authority
  - phase: 11-23
    provides: reviewed Phase 11 exact-reference compatibility authority
  - phase: 11-27
    provides: regression, benchmark, and passive testbed scenario convergence
  - phase: 11-28
    provides: closed 388-item semantic corpus with zero unresolved rows
provides:
  - fail-closed published-crate metadata, dependency, archive, and extracted-build isolation
  - submodule-free corpus/report and headless private-tooling CI contracts
  - executable mapping of all 26 Phase 11 decisions and eight requirements to evidence
  - bounded private-testbed advisory waiver with mandatory Phase 12 renderer replacement
affects: [phase-12-release-hardening, packaging, ci, compatibility-audit]
tech-stack:
  added: []
  patterns: [extracted-package verification, passive renderer isolation, closed advisory waiver, exact evidence audit]
key-files:
  created:
    - .planning/phases/11-examples-headless-tooling-and-testbed/11-29-SUMMARY.md
  modified:
    - tools/xtask/src/package.rs
    - tools/xtask/src/package/metadata.rs
    - tools/xtask/tests/package_cli.rs
    - .github/workflows/ci.yml
    - ARCHITECTURE.md
    - TESTING.md
    - deny.toml
key-decisions:
  - "Keep liquidfun as the sole default publishable package and reject renderer, protocol, differential, benchmark, testbed, graphics, and native-source leakage at metadata and archive boundaries."
  - "Permit exactly RUSTSEC-2025-0035 and RUSTSEC-2026-0192 only in the private unpublished Macroquad testbed graph, with renderer replacement mandatory in Phase 12 before release claims."
  - "Keep Cargo CI submodule-free by validating the checked corpus snapshot, closure, and generated report while reserving live discovery and C++ execution for oracle jobs."
patterns-established:
  - "Package isolation is proven from resolved metadata, inspected archive contents, and an extracted external build/test with private paths and display state absent."
  - "Security exceptions are executable contracts with an exact allowlist, package-graph isolation proof, and an explicit removal milestone."
requirements-completed: [RIGD-10, TEST-03, EXMP-01, EXMP-02, EXMP-03, EXMP-04, EXMP-05, EXMP-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T16:08:01Z
duration: 44min
completed: 2026-07-22
---

# Phase 11 Plan 29: Final Isolation and Evidence Audit Summary

**Extracted-package isolation, zero-gap corpus closure, headless CI, and an executable audit bind every Phase 11 decision and requirement without broadening Phase 12 claims.**

## Performance

- **Duration:** 44 min
- **Started:** 2026-07-22T15:24:27Z
- **Completed:** 2026-07-22T16:08:01Z
- **Tasks:** 1
- **Files modified:** 20

## Accomplishments

- Hardened Cargo metadata and crate-archive validation so `liquidfun` remains the sole default publishable package and rejects private tooling, renderer, graphics, protocol, benchmark, testbed, native-source, and C++ leakage.
- Added submodule-free CI coverage for the 388-item snapshot, zero-unresolved closure, byte-stable generated report, extracted consumer package, every private Rust tool, and the testbed with display variables unset.
- Documented the one-way catalog/controller/observation/checkpoint/comparison/oracle/passive-renderer architecture and exact corpus, replay, D0-D3, failure-bundle, acquisition, and testbed commands.
- Added an executable audit for D-01 through D-26 and RIGD-10, TEST-03, and EXMP-01 through EXMP-06.
- Bound the two approved Macroquad advisories to the private unpublished testbed only, proved the published crate is renderer-free, and recorded mandatory Phase 12 renderer replacement.

## TDD Evidence

- **RED:** `cargo test -p xtask --test package_cli` passed 7 existing tests and failed five new isolation/CI/audit cases before implementation.
- **GREEN:** The final target passes 13/13, including graphics/testbed archive rejection, second-publishable-package rejection, headless corpus/testbed CI, the complete decision audit, and the exact advisory waiver contract.
- The intentionally failing RED state was not committed.

## Task Commits

Each task was committed atomically:

1. **Task 1: Seal package isolation, docs, CI, and full decision coverage** - `5040398` (feat)

## Files Created/Modified

- `tools/xtask/src/package.rs` - Rejects private-capability paths, graphics assets, native sources, and unsafe archive content before extracted package execution.
- `tools/xtask/src/package/metadata.rs` - Enforces one published package, private workspace packages, and renderer/tooling-free production dependencies.
- `tools/xtask/tests/package_cli.rs` - Proves package, CI, decision-matrix, and bounded advisory-waiver contracts.
- `.github/workflows/ci.yml` - Runs submodule-free corpus/report closure plus headless private tooling and testbed gates.
- `ARCHITECTURE.md` - Records the integrated Phase 11 dependency direction and semantic authority boundaries.
- `TESTING.md` - Records exact commands, evidence tiers, diagnostic limits, acquisition rules, and all decision/requirement evidence.
- `crates/liquidfun-differential/src/fixtures/domain.rs` - Accepts the registered Phase 11 evidence schema while rejecting unregistered manifest tables.
- `deny.toml` - Contains only the two approved private-testbed advisory exceptions.
- `.codex/tasks/todo.md` - Tracks mandatory Phase 12 renderer replacement and waiver removal.

## Decisions Made

- Selected a bounded advisory waiver instead of an out-of-scope renderer migration because neither advisory has a safe Macroquad upgrade and the dependency is private, non-default, unpublished, and absent from the consumer crate.
- Kept `cargo xtask inventory check` out of submodule-free CI because live discovery requires the pinned checkout; CI instead regenerates `UPSTREAM-CORPUS.md` and fails on byte drift.
- Preserved the historical 33-row Phase 8 evidence contract by selecting rows bound to the exact Phase 8 references while allowing 13 separately evidenced Phase 11 promotions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Refreshed stale packaged topology provenance**

- **Found during:** Extracted `cargo xtask package verify`
- **Issue:** The packaged particle topology provenance retained an obsolete adapter digest after the canonical adapter manifest changed.
- **Fix:** Updated only the copied digest while preserving byte-identity verification.
- **Files modified:** `crates/liquidfun/src/particle/testdata/group-topology-witnesses.provenance.json`
- **Verification:** Extracted package build/test passes with 171 entries.
- **Committed in:** `5040398`

**2. [Rule 3 - Blocking] Repaired stale documentation contracts**

- **Found during:** `cargo xtask check` and `docs_contract`
- **Issue:** Aggregate compatibility markers, the retired Phase 8 ownership marker, platform-evidence selection, and one workflow-step scope predated the final Phase 11 promotions.
- **Fix:** Refreshed exact totals, removed only the obsolete Phase 8 RIGD-10 marker, selected Phase 8 rows by exact evidence identity, and bounded the artifact assertion to its upload step.
- **Files modified:** `tools/xtask/src/docs.rs`, `tools/xtask/tests/docs_contract.rs`
- **Verification:** `docs_contract` passes 35/35 and `cargo xtask check` passes.
- **Committed in:** `5040398`

**3. [Rule 3 - Blocking] Accepted the registered Phase 11 artifact schema in differential fixtures**

- **Found during:** Broad private-tooling test execution
- **Issue:** The differential fixture manifest reader rejected the already-promoted `[artifact_schemas.phase11_evidence]` table before intended workflow assertions.
- **Fix:** Added the exact closed schema, retained unknown-table rejection, updated the legacy v2 fixture, and added focused accept/reject tests.
- **Files modified:** `crates/liquidfun-differential/src/fixtures/domain.rs`, `crates/liquidfun-differential/src/fixtures/storage.rs`, `crates/liquidfun-differential/tests/fixture_workflow.rs`
- **Verification:** `fixture_workflow` passes 13/13 and `rigid_fixture_workflow` passes 15/15.
- **Committed in:** `5040398`

**4. [Rule 3 - Blocking] Repaired the authorized Markdown baseline**

- **Found during:** `just markdown-check`
- **Issue:** Six authorized non-GSD documents predated the repository's mdformat 1.0.0 baseline.
- **Fix:** Applied mdformat only to `UPSTREAM.md`, `standards-overrides.md`, `THIRD_PARTY_NOTICES.md`, `docs/decisions/0001-oracle-selection.md`, `crates/liquidfun-testbed/CAPABILITY.md`, and the Plan-owned `ARCHITECTURE.md`.
- **Verification:** `just markdown-check` passes without formatting `.planning/**`.
- **Committed in:** `5040398`

### Approved Architectural Adjustment

**5. [Rule 4 - Security policy] Bound two no-fix renderer advisories to the private testbed**

- **Decision:** The user explicitly selected the bounded waiver over blocking Plan 29 or migrating renderers inside Phase 11.
- **Boundary:** Exactly `RUSTSEC-2025-0035` and `RUSTSEC-2026-0192`; no other advisory is ignored, and `liquidfun` has no renderer dependency.
- **Follow-up:** Phase 12 must replace Macroquad and remove both ignores before release-readiness claims.
- **Verification:** `cargo deny --locked check` passes; `package_cli::advisory_waiver_is_bounded_to_the_private_testbed` fails closed on waiver or graph expansion.
- **Committed in:** `5040398`

**Total deviations:** 5 (4 blocking repairs, 1 explicitly approved security-policy adjustment)

**Impact on plan:** All changes close correctness, verification, or security-policy gaps required by the final Phase 11 audit. The published crate surface did not expand.

## Issues Encountered

- The plan's written `cargo deny check --locked` order is unsupported by the installed cargo-deny; the supported equivalent `cargo deny --locked check` passed with advisories, bans, licenses, and sources all green. Duplicate-version warnings remain policy-allowed.
- Broad private-crate execution in this isolated branch reaches six pre-existing `round_trip` failures in the explicitly fenced `prefix.rs` area. No fenced file was touched or copied. All Plan 29-owned suites and the mandatory default-member gate pass; parent integration must rerun the broad command with the fenced baseline repair present.
- Live `cargo xtask inventory check` correctly requires the uninitialized pinned submodule and was not treated as a Cargo-only gate. Snapshot, closure, report generation, package, docs, and Cargo-only provenance all pass without native source.

## Known Stubs

None. Stub-pattern matches are test inputs or intentional empty policy arrays, not incomplete runtime behavior.

## Verification

- Mandatory ordered Rust gate passed: `cargo fmt --all`, deny-warnings Clippy, all-target/all-feature build, and all-feature default-member tests.
- Package isolation passed with 171 extracted entries built and tested outside the repository.
- Corpus snapshot and closure passed with 388 items and zero unresolved; report regeneration produced no tracked diff.
- `cargo xtask check`, `cargo deny --locked check`, actionlint, `just markdown-check`, docs contracts, package CLI, and pinned-submodule read-only diff passed.
- Headless testbed build/test passed all 38 integration tests with display variables unset.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 11 package, corpus, evidence, documentation, and renderer-isolation contracts are complete for parent integration.
- Phase 12 must replace Macroquad, remove both advisory ignores, and prove performance, portability, safety, packaging, coverage, and release readiness.
- Parent integration must rerun the broad private-crate CI command after applying the separately fenced `prefix.rs` repair; this isolated branch deliberately leaves that file untouched.

## Self-Check: PASSED

- Confirmed the summary and all declared key files exist.
- Confirmed task commit `5040398` exists and contains no fenced-file changes.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
