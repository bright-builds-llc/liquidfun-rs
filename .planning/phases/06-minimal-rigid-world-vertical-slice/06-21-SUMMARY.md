---
phase: 06-minimal-rigid-world-vertical-slice
plan: "21"
subsystem: rigid-fixture-provenance
tags: [rust, differential, fixtures, provenance, sha256, d1]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "17"
    provides: Real D1-gated rigid fixture staging, replay, review, and promotion
provides:
  - Shared current-checkout adapter and effective compile-command identity validator
  - Pre-mutation checkout binding for rigid stage, review, and promotion
  - Real-child stale-adapter and stale-compile exact no-effect regressions
affects: [06-22-completion-signoff, rigid-regression-evidence, oracle-provenance]
tech-stack:
  added: []
  patterns: [pure identity core with thin adapters, recompute-before-mutation provenance]
key-files:
  created:
    - crates/liquidfun-differential/src/oracle_identity.rs
    - crates/liquidfun-differential/tests/oracle_identity.rs
  modified:
    - crates/liquidfun-differential/src/rigid_fixtures.rs
    - crates/liquidfun-differential/tests/fixtures/fake_oracle.rs
    - crates/liquidfun-differential/tests/rigid_fixture_workflow.rs
    - tools/xtask/src/differential.rs
    - tools/xtask/src/upstream.rs
    - tools/xtask/tests/differential_cli.rs
key-decisions:
  - "Keep adapter-source and four-unit effective compile hashing in one private differential identity module shared by ordinary execution and fixture lifecycle paths."
  - "Recompute checkout identity during every rigid candidate replay so review and promotion cannot rely on a cached stage-time result."
  - "Make fake D1 children derive normal digests from their isolated repository and corrupt exactly one digest for each stale-binary regression."
patterns-established:
  - "Oracle checkout identity: validate confined adapter inputs and normalized compile commands before any evidence mutation."
  - "Fixture provenance drift: stage, review, and promotion each derive authority from current bytes and the recorded preset."
requirements-completed: [RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T21:25:12Z
duration: 16 min
completed: 2026-07-12
---

# Phase 6 Plan 21: Rigid Fixture Checkout Provenance Summary

**Rigid D1 fixture mutations now require adapter and effective compile digests independently recomputed from the current checkout before stage, review, or promotion writes.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-12T21:09:00Z
- **Completed:** 2026-07-12T21:25:12Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Extracted one typed `oracle_identity` core for confined adapter-manifest hashing, normalized four-result-unit compile database hashing, and current-checkout identity validation.
- Routed ordinary rigid execution, rigid staging, candidate replay, review, promotion, and upstream configure through the shared digest implementation without duplicating the trust decision.
- Added real separately compiled child modes whose normal D1 identity derives from isolated repository bytes and whose stale modes alter exactly one adapter or compile digest.
- Proved stale stage attempts create no staging root, candidate, review receipt, accepted trace/regression, or manifest mutation, and proved post-stage drift fails again before review and promotion writes.

## Task Commits

1. **Task 1: Centralize current-checkout oracle identity validation** - `1dc1f4d` (`fix`)
2. **Task 2: Prove stale canonical-looking binaries cannot mutate fixture state** - `21aaee7` (`test`)
3. **Task 2 verification repair: Point the moved coverage contract at the shared module** - `41389a8` (`test`)

## Files Created/Modified

- `crates/liquidfun-differential/src/oracle_identity.rs` - Typed, bounded adapter and effective compile identity core.
- `crates/liquidfun-differential/tests/oracle_identity.rs` - Confinement, malformed database, relocation, and mismatch tests.
- `crates/liquidfun-differential/src/rigid_fixtures.rs` - Checkout validation before stage authority and during every replay.
- `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs` - Truthful current-checkout D1/D2 identities and isolated stale digest modes.
- `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs` - Exact no-effect stage, review, and promotion provenance regressions.
- `tools/xtask/src/differential.rs` - Ordinary rigid execution delegates identity validation to the shared core.
- `tools/xtask/src/upstream.rs` - Configure-time adapter hashing delegates to the same core.
- `tools/xtask/tests/differential_cli.rs` - Real xtask stale-child failure propagation and moved-module contract.

## Decisions Made

- Diagnostics name only a bounded mismatch class, manifest line/input index, or one of four closed translation-unit names; they never print source bytes, compile commands, or absolute repository paths.
- Candidate metadata continues to record the child identity, but authority comes from fresh comparison against current source and the effective database selected by `metadata.preset`.
- The generic empty-world fixture lifecycle stays unchanged and does not acquire the rigid current-checkout contract.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated a moved implementation coverage contract**

- **Found during:** Final all-xtask fixture verification.
- **Issue:** `collision_compile_database_identity_is_covered_by_unit_digest_tests` still searched `tools/xtask/src/differential.rs` after compile-database identity moved to the shared differential module.
- **Fix:** Pointed the compile-time contract at `crates/liquidfun-differential/src/oracle_identity.rs`.
- **Files modified:** `tools/xtask/tests/differential_cli.rs`.
- **Verification:** All 28 differential CLI tests and the exact ordered Rust gate pass.
- **Committed in:** `41389a8`.

**Total deviations:** 1 auto-fixed blocking issue. **Impact:** The repair preserves the intended coverage assertion and introduces no behavior or scope change.

## Issues Encountered

- The tasks were marked TDD, but deliberately failing RED commits were not created because repository instructions require formatting, strict Clippy, all-target build, and all-feature tests to pass before every commit. Regression tests and implementation were committed as green task outcomes.
- Local CMake 3.27.9 and Apple Clang 21.0.0 remain truthful D2 evidence; canonical Linux Clang 22.1.8 remains the only D1 authority.

## Validation Evidence

- Identity core: 5/5 tests pass, covering current acceptance, stale adapter/compile rejection, unsafe and duplicate manifest paths, missing inputs/units, duplicate units, divergent flags, malformed JSON, and relocation-stable normalization.
- Rigid fixture lifecycle: 7/7 real-binary tests pass, including exact stale-adapter/stale-compile no-effect snapshots and fresh review/promotion rechecks.
- Generic fixture lifecycle: all 13 unchanged tests pass; xtask differential CLI: all 28 tests pass, including real canonical D1, D2 rejection, and stale identity rejection.
- Real oracle evidence: debug compare D2, release compare D2, debug replay D2, and two debug runs byte-identical at D0.
- Package isolation verifies 58 entries outside the repository; provenance verifies the pinned oracle and one artifact record.
- The exact ordered Rust gate passed before every code/test commit: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.
- Plan structure, lifecycle, acceptance searches, single-implementation review, and `git diff --check` pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-20 remains the next executable plan because Wave 14 ran 06-21 before its sibling in this sequential shared-main execution.
- After 06-20, Plan 06-22 can rerun the completion matrix and formal signoff. Phase 6 and RIGD-04 remain unclosed until that verification succeeds.

## Self-Check: PASSED

- Task commits `1dc1f4d`, `21aaee7`, and `41389a8` exist in history.
- Both declared created files exist, and the shared validator has exactly one implementation with stage, replay, and ordinary rigid callers.
- No accepted fixture, regression, or artifact manifest changed in the real repository.
- Phase 6 remains incomplete and the next current position remains Plan 06-20.

***

*Phase: 06-minimal-rigid-world-vertical-slice*
*Completed: 2026-07-12*
