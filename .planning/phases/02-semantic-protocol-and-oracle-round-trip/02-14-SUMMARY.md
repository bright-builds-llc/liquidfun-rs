---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "14"
subsystem: harness-contract-and-ci
tags: [rust, xtask, documentation-contract, github-actions, differential, sanitizer, provenance]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Versioned protocol, typed comparator/reducer, bounded supervisor, reviewed replay, and safe contributor commands from Plans 02-01 through 02-13
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Cargo-only package isolation, pinned upstream/tool identities, private CMake boundary, and read-only CI permissions
provides:
  - Enforceable architecture and complete twelve-layer testing contract for the Phase-2 harness seam
  - Read-only xtask documentation checker with positive and semantic-negative contract coverage
  - Submodule-free Cargo CI coverage for every private Rust harness and failure fixture
  - Canonical one-shot, reuse, replay, provenance, and scheduled fail-fast sanitizer CI lanes
  - Two-request sanitizer reset/reuse proof with C++ and Rust reset epochs 1 then 2
affects: [phase-03-object-model, contributor-workflows, differential-evidence, scheduled-security, release-audit]

tech-stack:
  added: []
  patterns: [machine-audited documentation contracts, CI trust separation, read-only evidence assertions, two-request sanitizer reset proof]

key-files:
  created:
    - tools/xtask/src/docs.rs
    - tools/xtask/tests/docs_contract.rs
  modified:
    - ARCHITECTURE.md
    - TESTING.md
    - tools/xtask/src/main.rs
    - .github/workflows/ci.yml
    - .github/workflows/oracle.yml
    - crates/liquidfun-test-protocol/src/limits.rs
    - crates/liquidfun-differential/src/runner.rs
    - crates/liquidfun-differential/src/supervisor/profile.rs
    - crates/liquidfun-differential/tests/round_trip.rs

key-decisions:
  - "Make TESTING.md's exact twelve-row layer table executable policy through a strict read-only xtask checker rather than relying on prose review alone."
  - "Keep Cargo CI entirely submodule/CMake/oracle-free while canonical oracle CI alone owns real process round trips and read-only evidence assertions."
  - "Treat the sanitizer profile as a bounded two-request reused session so the scheduled command proves adapter reset epochs instead of repeating one-shot coverage."
  - "Upload only bounded target/differential/failures evidence, only on sanitizer-job failure, through a full-SHA-pinned reviewed action."

patterns-established:
  - "Documentation contract: every test layer has one validated status, command, prerequisite, artifact, retry, placement, and semantic interpretation."
  - "CI trust split: private Rust failure injection stays Cargo-only; C++ checkout, exact tools, real child execution, and sanitizers stay in oracle workflows."
  - "Read-only evidence: provenance, replay, aggregate checks, and CI round trips must leave protocol, scenarios, reference artifacts, and compatibility records byte-identical."

requirements-completed:
  - COMP-03
  - COMP-04
  - COMP-05
  - COMP-06
  - COMP-07
  - COMP-08
  - COMP-09
  - DOCS-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T11:48:52Z

duration: 22 min
completed: 2026-07-10
---

# Phase 2 Plan 14: Harness Contract and CI Evidence Summary

**The Phase-2 empty-world harness now has an executable architecture/testing contract, submodule-free Rust gates, real one-shot/reuse/replay CI, and a scheduled two-request fail-fast sanitizer proof.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-07-10T11:26:39Z
- **Completed:** 2026-07-10T11:48:52Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Replaced deferred harness prose with the implemented one-way dependency graph, strict protocol boundary, typed comparison policy, bounded process lifecycle, evidence boundary, and honest empty-world-only maturity statement.
- Published a complete twelve-layer testing contract covering current and deferred unit, integration, doctest, upstream, differential, property, regression, fuzz, Miri, sanitizer, benchmark, and coverage workflows with commands, prerequisites, artifacts, retry policy, placement, and interpretation.
- Added a strict `cargo xtask docs check` surface and eight focused command tests covering every missing layer, duplicate layers, every blank column, invalid enums, missing semantic markers, and all forbidden placeholders.
- Extended Cargo CI with explicit protocol, comparator, minimizer, supervisor-failure, provenance-schema, and docs-contract coverage without initializing C++.
- Extended oracle CI with real one-shot, two-request reuse, reviewed replay, repeated provenance, read-only evidence assertions, and a scheduled/manual ASan/UBSan lane that uploads only bounded failure evidence.
- Corrected the sanitizer lifecycle to execute two distinguishable requests through one child and prove both C++ and Rust reset epochs advance from 1 to 2.

## Task Commits

Each task was committed atomically:

1. **Task 1: Document the permanent protocol, diagnosis, and evidence boundaries** - `cacacc1` (`docs`)
1. **Task 2: Add CI round-trip/sanitizer gates and perform the final simplification pass** - `c56036b` (`ci`)

## Files Created/Modified

- `ARCHITECTURE.md` - Enforceable dependency direction, functional-core/process-shell split, C++ isolation, lifecycle, evidence, and scope boundaries.
- `TESTING.md` - Machine-audited layer table plus exact protocol versions, commands, diagnosis, review, sanitizer, coverage, and CI placement guidance.
- `tools/xtask/src/docs.rs` - Strict read-only parser and semantic validator for the DOCS-05 testing contract.
- `tools/xtask/src/main.rs` - Registered docs dispatch and aggregate-check integration.
- `tools/xtask/tests/docs_contract.rs` - Positive and exhaustive contract-rejection tests.
- `.github/workflows/ci.yml` - Explicit submodule-free private Rust harness verification.
- `.github/workflows/oracle.yml` - Canonical round-trip/replay gates and scheduled fail-fast sanitizer/reset job.
- `crates/liquidfun-test-protocol/src/limits.rs` - Two-request sanitizer lifecycle budget.
- `crates/liquidfun-differential/src/runner.rs` - Two distinguishable requests for both reuse and sanitizer profiles.
- `crates/liquidfun-differential/src/supervisor/profile.rs` - Reused sanitizer child lifecycle.
- `crates/liquidfun-differential/tests/round_trip.rs` - CLI regression proving sanitizer request count and reset epochs.

## Decisions Made

- Kept documentation enforcement narrow and deterministic: the checker parses one exact table, validates a closed vocabulary, and checks per-layer semantic markers without generating or rewriting Markdown.
- Kept workflows declarative. Xtask and typed Rust tests own validation; workflows select reviewed entrypoints and assert permissions, tool identity, failure retention, and evidence immutability.
- Used the existing synchronous supervisor for sanitizer reuse rather than adding a second implementation or async runtime.
- Preserved typed comparator and reduction policy without JSON-path rule engines, global sorting, tolerance widening, retry, or golden-data mutation.

## Verification Evidence

- `cargo test -p xtask --test docs_contract` passes all 8 documentation-contract tests, and `cargo xtask docs check` validates all 12 required layers.
- The required ordered default and workspace format, warning-denied Clippy, build, test, and warning-denied rustdoc sequence passes.
- Package isolation, upstream identity, provenance, debug configure/build, one-shot compare, two-request reuse, and reviewed-trace replay all pass.
- Both exact environment-prefixed `oracle-asan-ubsan` commands pass; one-shot reports one validated request and `sanitizer` reports two requests with C++/Rust reset epochs `1, 2`.
- `cargo xtask check`, `just check`, `actionlint`, and `mdformat --check ARCHITECTURE.md TESTING.md` pass.
- Repeated provenance, replay, and aggregate checks leave all tracked protocol, scenario, reference, compatibility, and published-crate bytes unchanged; `git diff --check` also passes.
- The explicit simplification scan confirms `crates/liquidfun` has no C++, serde, harness dependency, build script, Tokio, retry, or default-member leakage.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Made the sanitizer profile exercise its promised reset/reuse corpus**

- **Found during:** Task 2 exact sanitizer-command verification
- **Issue:** The command exited successfully but reported one request because the sanitizer profile had a one-request budget, did not keep the process, and the runner duplicated requests only for the reuse profile. This contradicted Plan 02-14 and TESTING.md.
- **Fix:** Set the sanitizer budget to two, reused the same bounded supervisor process, generated the second distinguishable request, and added CLI regression assertions for request count and C++/Rust reset epochs 1 then 2.
- **Files modified:** `crates/liquidfun-test-protocol/src/limits.rs`, `crates/liquidfun-differential/src/runner.rs`, `crates/liquidfun-differential/src/supervisor/profile.rs`, `crates/liquidfun-differential/tests/round_trip.rs`
- **Verification:** Focused limit, round-trip, and supervisor tests pass; the exact sanitizer command reports two validated requests and reset epochs `1, 2`.
- **Committed in:** `c56036b`

**2. [Rule 1 - Bug] Fixed a warning-denied workspace lint in the new docs test**

- **Found during:** Task 2 full-workspace simplification/verification pass
- **Issue:** The mandatory default-member Clippy gate did not compile private xtask tests, and workspace Clippy rejected an avoidable owned-string assignment in the new fixture mutation helper.
- **Fix:** Replaced the assignment with `clone_into` while preserving the test behavior.
- **Files modified:** `tools/xtask/tests/docs_contract.rs`
- **Verification:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` and the full workspace test suite pass.
- **Committed in:** `c56036b`

**3. [Rule 3 - Blocking] Used the registered replay command shape in final verification**

- **Found during:** Final ordered verification step 16
- **Issue:** The plan's replay command omitted the required `--session-profile` option, while the implemented and documented xtask boundary intentionally rejects incomplete option shapes.
- **Fix:** Ran replay with `--session-profile one-shot`, matching xtask help, TESTING.md, just, tests, and CI.
- **Files modified:** None.
- **Verification:** Reviewed-trace replay matched, and the repeated replay left tracked evidence unchanged.
- **Committed in:** Not applicable; verification-command correction only.

**4. [Rule 1 - Bug] Synchronized stale human-readable GSD completion fields**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` and `roadmap update-plan-progress 02` reported the correct 100% and 14/14 completion values but left the tracked body progress at 95% and the roadmap row at 13/14.
- **Fix:** Updated only the stale human-readable state position/progress and Phase-2 roadmap row to match the successful GSD tool results.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** State frontmatter and body now agree on verification/100%, fourteen Phase-2 summaries exist, and the roadmap reports 14/14 complete.
- **Committed in:** Plan metadata commit.

______________________________________________________________________

**Total deviations:** 4 auto-fixed (2 correctness bugs, 1 blocking verification-command correction, 1 metadata correctness bug)
**Impact on plan:** The fixes make the promised sanitizer evidence real, retain warning-denied workspace quality, and use the already-approved closed replay interface. No public API, physics, tolerance, schema, or evidence scope was expanded.

## Issues Encountered

- Local verification used CMake 3.27.9 and AppleClang 21, which the tool correctly reported as non-canonical warnings; the canonical GitHub job installs and asserts the pinned CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8 identities.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. Deferred fuzz, Miri, Rust sanitizer, benchmark, coverage, and broad physics suites are explicitly documented future layers rather than hidden stubs.

## Next Phase Readiness

- All fourteen Phase-2 plans now have implementation and verification evidence; the milestone is ready for phase-level verification before Phase 3 begins.
- Phase 3 can design handles, invalidation, callbacks, and public object-model semantics on top of a stable process-isolated comparison seam.
- Compatibility claims remain deliberately limited to the empty-world harness proof; rigid-body and particle parity remain unclaimed.

## Self-Check: PASSED

- Both task commits `cacacc1` and `c56036b` exist and exclude the pre-existing `.planning/config.json` change.
- All eleven created/modified task paths exist, and `crates/liquidfun` remains byte-identical.
- Summary lifecycle metadata and all eight requirement IDs match Plan 02-14 exactly.
- Full default/workspace, package, provenance, oracle, differential, sanitizer, docs, workflow, formatting, repeatability, and diff checks pass.

______________________________________________________________________

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
