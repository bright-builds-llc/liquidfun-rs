---
phase: 06-minimal-rigid-world-vertical-slice
plan: "17"
subsystem: rigid-fixture-lifecycle
tags: [rust, differential, fixtures, provenance, d1, atomicity]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "15"
    provides: Complete non-dynamic contact admission witnesses
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "16"
    provides: Closed rigid request, action, step, and inertia contracts
provides:
  - Typed real-binary rigid fixture staging with declaration-first comparison
  - D1 authority checks before staging, review, and promotion mutations
  - Exact-hash rigid replay and real xtask/just lifecycle entrypoints
affects: [06-18-sanitizer-signoff, rigid-regression-evidence, fixture-promotion]
tech-stack:
  added: []
  patterns: [scenario-dispatched typed replay, validation-before-write transaction, repeated authority guard]
key-files:
  created:
    - crates/liquidfun-differential/src/rigid_fixtures.rs
    - crates/liquidfun-differential/tests/rigid_fixture_workflow.rs
  modified:
    - crates/liquidfun-differential/src/main.rs
    - crates/liquidfun-differential/src/fixtures/lifecycle.rs
    - crates/liquidfun-differential/src/fixtures/replay.rs
    - crates/liquidfun-differential/tests/fixtures/fake_oracle.rs
    - tools/xtask/tests/differential_cli.rs
    - justfile
key-decisions:
  - "Keep rigid fixture semantics in a cohesive typed transaction and reuse only the existing confined create-new/no-clobber storage primitives."
  - "Dispatch replay from the recorded phase-06-rigid-world identity and carry its validated BuildIdentity forward so review and promotion independently repeat the D1 guard."
  - "Use a decoded request plus deterministic native execution in the fake oracle, with truthful D1 and D2 handshake identities instead of bypassing the real binary." 
requirements-completed: [RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T17:13:13Z
duration: 14 min
completed: 2026-07-12
---

# Phase 6 Plan 17: Real Rigid Fixture Lifecycle Summary

**The advertised rigid fixture command now executes the real typed native/oracle transaction and permits candidate, review, or accepted-state mutation only after canonical D1 authority is proven.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-12T16:59:00Z
- **Completed:** 2026-07-12T17:13:13Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added `rigid_fixtures`, which binds the checked-in Phase 6 policy, decodes the fixed rigid request, executes native Rust, supervises the oracle, validates declaration order, compares semantic results, enforces bounds, and checks D1 before its first staging write.
- Extended exact-hash candidate replay so rigid review and promotion re-decode and recompare the stored transaction and independently re-check D1 before review receipts, accepted artifacts, or manifest mutation.
- Extended the separately compiled fake oracle to decode rigid requests and emit deterministic protocol-correct results and reset proof under truthful D1 or D2 identities without changing generic empty-world behavior.
- Proved the actual differential binary and xtask child accept D1 and reject D2 with no rejected candidate directory, accepted artifact, or manifest change.

## Task Commits

1. **Task 1: Implement typed rigid stage, replay, and promotion transactions** - `ff8a26c` (`feat`)
2. **Task 2: Prove the actual binary and xtask surface end to end** - `8a391cc` (`test`)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_fixtures.rs` - Cohesive typed stage/replay validation and post-authority storage seam.
- `crates/liquidfun-differential/src/fixtures/replay.rs` - Recorded-scenario dispatch and retained rigid identity.
- `crates/liquidfun-differential/src/fixtures/lifecycle.rs` - Repeated D1 checks immediately before review and promotion effects.
- `crates/liquidfun-differential/src/main.rs` - Real `fixture stage --scenario rigid-world` dispatch and usage contract.
- `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs` - Decoded deterministic rigid D1/D2 oracle behavior.
- `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs` - Real-binary acceptance, rejection, dirty replay, and child-status evidence.
- `tools/xtask/tests/differential_cli.rs` - Actual child execution through xtask, including D2 no-effect proof.
- `justfile` - Fixed rigid stage, review, and promotion delegates.

## Decisions Made

- The generic empty-world lifecycle remains unchanged; rigid replay is selected only after candidate hashes and the closed scenario identity validate.
- The first effectful staging operation remains below an explicit D1 guard. Replayed `BuildIdentity` is retained so review and promotion do not trust a cached boolean or flattened metadata claim.
- Real-binary tests create isolated committed repositories and place the fake oracle only under the reviewed preset output, preserving generator-dirty checks and executable confinement.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The plan named a nonexistent `verify plan-acceptance` helper. Equivalent repository-owned `verify plan-structure`, `verify artifacts`, `verify key-links`, lifecycle validation, explicit acceptance searches, and `git diff --check` all passed.
- Local real-oracle runs remain correctly classified D2 because the machine uses CMake 3.27.9 and Apple Clang 21 rather than canonical Linux pins. The isolated real-binary fixture tests provide a truthful canonical-D1 identity specifically for promotion-path evidence.

## Validation Evidence

- All four real-binary rigid lifecycle tests pass, including canonical promotion, dirty replay rejection, child failure propagation, and D2 rejection before staging-root creation.
- All 13 unchanged generic fixture workflow tests and all 24 xtask differential CLI tests pass.
- The xtask real-child test accepts `xtask-rigid-d1` and rejects `xtask-rigid-d2` without creating its candidate directory.
- Debug and release rigid comparisons plus debug rigid replay each match both required families under `phase6-v1`; local identities remain truthfully D2.
- Package isolation verifies 58 entries outside the repository.
- Before both task commits, the required sequence passed: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.
- Plan structure, artifacts, key links, lifecycle validation, acceptance searches, commit checks, and `git diff --check` pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-18 can run rigid protocol and compare surfaces under ASan/UBSan and complete final Phase 6 signoff.
- No accepted fixture or manifest was changed in the real repository; promotion remains an explicit reviewed D1-only action.

## Self-Check: PASSED

- Task commits `ff8a26c` and `8a391cc` exist in history.
- Both declared key files exist, and the real-binary test file exercises D1 and D2 through the production CLI.
- Phase 6 remains incomplete; only Plan 17 progress is recorded.

***

_Phase: 06-minimal-rigid-world-vertical-slice_
_Completed: 2026-07-12_
