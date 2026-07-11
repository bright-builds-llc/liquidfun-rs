---
phase: 04-math-settings-and-numerical-policy
plan: "06"
subsystem: verification-entrypoints-and-ci
tags: [xtask, ci, json-schema, determinism, differential-testing]
requires:
  - phase: 04-math-settings-and-numerical-policy
    plan: "05"
    provides: Exact C++ math probes and strict floating build identity
provides:
  - Closed local compare, replay, and two-run D0 commands for the Phase 4 math-probe corpus
  - Deterministic scenario and trace schema presentations for math-probe inputs and outputs
  - Canonical debug/release oracle coverage plus supported-platform Cargo-only math and policy coverage
affects: [04-07-documentation-signoff, phase-5-collision, canonical-oracle-ci]
tech-stack:
  added: []
  patterns: [allowlisted orchestration, typed policy comparison, byte-identical replay, read-only canonical evidence]
key-files:
  modified:
    - tools/xtask/src/differential.rs
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json
    - .github/workflows/oracle.yml
    - .github/workflows/ci.yml
    - justfile
key-decisions:
  - "Run Phase 4 comparison inside xtask against the typed native executor so contributor commands cannot substitute executable, path, compiler, or policy inputs."
  - "Treat D0 as complete byte equality across two independent one-shot oracle processes, including the validated handshake and reset proof."
  - "Keep canonical evidence read-only while supported OS jobs exercise only native public math and numerical-policy contracts."
patterns-established:
  - "Verification routing: only math-probes, reviewed debug/release presets, one-shot comparison, and exactly two D0 runs are accepted."
  - "Schema authority: generated presentations remain byte-stable while typed Rust and C++ validation owns cross-field semantics."
requirements-completed: [COLL-01, COLL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
generated_at: 2026-07-11T06:48:02Z
duration: 31 min
completed: 2026-07-11
---

# Phase 4 Plan 06: Verification Entry Points and Canonical CI Summary

**Phase 4 now has one closed local/CI command surface that compares all 39 math probes, replays fixed evidence, and proves same-build byte stability without exposing arbitrary execution inputs.**

## Performance

- **Duration:** 31 min
- **Completed:** 2026-07-11T06:48:02Z
- **Tasks:** 1
- **Files modified:** 12

## Accomplishments

- Added allowlisted `math-probes` compare/replay commands and a fixed two-run D0 command that reject arbitrary paths, executables, compiler flags, profiles, presets, and run counts before effects.
- Validated the oracle handshake and complete Phase 4 identity, then compared ordered case metadata, IEEE class/sign metadata, exact discrete branches, and every float value under the registered `phase4-v1` field policy.
- Added deterministic closed JSON Schema presentations for every bounded math-probe operand, result value, result branch, horizon, and one-shot reset proof.
- Added transparent just recipes, supported Linux/macOS/Windows native math-policy tests, and canonical Linux debug/release probes, identity rejection, D0 repetition, and final read-only evidence checks.
- Preserved full action SHA pins, exact tool checksums, read-only workflow permissions, and the existing separation between compare/replay and reviewed fixture promotion.

## Task Commits

1. **Task 1: Wire allowlisted commands, schemas, and CI lanes** - `e99f971` (feat)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Add private typed comparison dependencies to xtask**

- **Found during:** Task 1
- **Issue:** The planned xtask command could not validate typed probe results or apply the reviewed field-policy registry using its existing orchestration-only dependency set.
- **Fix:** Added private path dependencies on `liquidfun-differential` and `liquidfun-test-protocol`; the published `liquidfun` crate and default consumer path remain unchanged.
- **Files modified:** `tools/xtask/Cargo.toml`, `Cargo.lock`
- **Verification:** Strict all-target Clippy/build and Cargo package isolation tests pass.
- **Committed in:** `e99f971`

**2. [Rule 3 - Blocking] Update renderer authority and schema consumers with regenerated presentations**

- **Found during:** Task 1
- **Issue:** Editing tracked JSON alone would violate the byte-stable renderer contract, and one fixture assertion assumed the previous single-variant schema layout.
- **Fix:** Extended the test-only renderer, regenerated scenario/trace schemas deterministically, added math-probe schema assertions, and pointed the existing fixture check at the ordinary-scenario `oneOf` branch.
- **Files modified:** `crates/liquidfun-test-protocol/src/schema.rs`, `crates/liquidfun-test-protocol/src/schema/tests.rs`, `crates/liquidfun-test-protocol/tests/fixtures.rs`
- **Verification:** Schema byte-stability and all fixture tests pass; `$defs` keep repeated bit-vector structures centralized.
- **Committed in:** `e99f971`

**3. [Rule 3 - Blocking] Add command-level rejection coverage**

- **Found during:** Task 1
- **Issue:** The planned file list did not include the existing xtask CLI integration suite, but the new allowlist and fixed run-count contract required regression protection.
- **Fix:** Added accepted compare/replay/D0 cases plus rejected reuse and non-two-run cases.
- **Files modified:** `tools/xtask/tests/differential_cli.rs`
- **Verification:** All 13 differential CLI tests pass.
- **Committed in:** `e99f971`

**Total deviations:** 3 auto-fixed (3 blocking). **Impact:** Private tooling, deterministic presentations, and boundary tests only; no consumer API, C++ runtime dependency, or fixture-promotion authority was added.

## Issues Encountered

- The first schema fixture run exposed the expected old top-level `properties` assumption after the schema became a two-variant `oneOf`; the assertion now selects the ordinary physics branch explicitly.
- Local Apple Clang remains D2 as recorded by Plan 04-05. Both debug and release comparisons pass under the reviewed policy, while canonical D1 remains confined to pinned Linux Clang 22 CI.

## Verification

- `cargo test -p xtask differential --all-features`, the full 13-test `differential_cli` suite, schema tests, and fixture tests pass.
- Debug compare, release compare, debug replay, and two independent byte-identical debug D0 runs pass for all 39 ordered probe cases.
- Workflow lint/static review, just recipe listing, full-SHA action inspection, checksum inspection, permission review, promotion-path review, and `git diff --check` pass.
- Ordered Rust gate passed before the task commit: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.

## User Setup Required

None - local noncanonical toolchains remain classified automatically, and canonical CI installs exact reviewed tools by checksum.

## Next Phase Readiness

- Plan 04-07 can document and sign off the executable numerical policy without overstating local D2 evidence as canonical D1 parity.
- Collision planning can reuse the named commands, typed policy lookup, schema presentations, and read-only CI evidence boundary.

## Self-Check: PASSED

- Task commit `e99f971` exists in history.
- Every modified schema, workflow, just, Cargo, xtask, and boundary-test artifact exists.
- All named focused checks, four real math-probe commands, workflow review, and the exact ordered full Rust gate pass.
- Compare/replay cannot promote fixtures, and unknown scenario/preset/profile/path/run-count input is rejected before execution.

***

_Phase: 04-math-settings-and-numerical-policy_
_Completed: 2026-07-11_
