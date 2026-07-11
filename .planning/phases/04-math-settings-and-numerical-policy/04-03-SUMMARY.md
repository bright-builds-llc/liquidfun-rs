---
phase: 04-math-settings-and-numerical-policy
plan: "03"
subsystem: differential-policy
tags: [rust, ieee-754, tolerances, determinism, diagnostics]
requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Exact FloatBits transport, typed comparison, and first-divergence reports
provides:
  - Strict sorted and hashed Phase 4 field-policy profile
  - Explicit zero, non-finite, collection, horizon, and evidence-tier semantics
  - Policy-aware float matching and bounded numeric diagnostics
affects: [04-04-native-math-probes, 04-05-cpp-probes, phase-5-collision]
tech-stack:
  added: []
  patterns: [closed policy registry, exact transport versus semantic comparison, bounded first-divergence evidence]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/tolerance/policy.rs
    - protocol/tolerances/phase4-v1.toml
  modified:
    - crates/liquidfun-test-protocol/src/tolerance.rs
    - crates/liquidfun-differential/src/comparator.rs
    - crates/liquidfun-differential/src/report.rs
    - crates/liquidfun-differential/tests/comparison.rs
key-decisions:
  - "Sort explicit semantic paths before hashing so policy identity is independent of TOML field order while duplicate paths still fail closed."
  - "Keep Phase 2 comparison APIs stable and add a complete FieldPolicy-aware matcher rather than introducing a global epsilon or changing exact transport."
  - "Attach operation horizon and canonical evidence tier to existing first-divergence reports while bounding neighboring context and sibling counts."
requirements-completed: [COLL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
generated_at: 2026-07-11T05:27:00Z
duration: 15 min
completed: 2026-07-11
---

# Phase 4 Plan 03: Versioned Numerical Policy and Diagnostics Summary

**A strict explicit field-policy registry now separates exact float transport from reviewed equality and produces bounded IEEE-aware first-divergence diagnostics.**

## Performance

- **Duration:** 15 min
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added strict TOML decoding with unknown-field rejection, bounded paths/justifications/horizons, duplicate detection, deterministic sorting, and a complete SHA-256 identity.
- Added explicit signed-zero, non-finite, collection, horizon, and D0-D3 evidence-tier types with no implicit field fallback.
- Checked in `phase4-v1` policies for discrete validity, exact constants, ULP local kernels, and bounded absolute-relative transform composition.
- Added policy-aware matching for transported NaNs, arithmetic NaNs, infinities, signed zeros, and existing exact/absolute/relative/ULP rules.
- Extended first-divergence evidence with float class/sign, absolute/relative/ULP diagnostics, fixed operation horizon, canonical tier, and bounded sibling count while preserving stable failure signatures.

## Task Commits

1. **Task 1: Model and validate the complete policy registry** - `e701a50`
2. **Task 2: Apply policy and enrich first-divergence evidence** - `e641578`

## Deviations from Plan

- The registered executor stalled without writing after two bounded attempts, so the orchestrator used the execute-phase inline fallback and completed the same planned files, tests, gates, and atomic commits.
- Phase 2 `ToleranceProfile` remains unchanged for current empty-world traces; Phase 4 adds `Phase4PolicyProfile` as the explicit math-probe policy consumed by subsequent plans.

## Verification

- Phase 4 tolerance filter: 10 tests passed.
- Differential comparison integration suite: 13 tests passed, including `deliberate_mismatch_reports_phase4_diagnostics`.
- The no-global-policy fallback check passed.
- Before each task commit, the exact ordered gate passed: format, strict all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests.
- Final existing engine suite passed: 72 unit/property tests, 15 integration tests, and 6 doctests.

## Next Phase Readiness

- Plan 04-04 can attach every pure math probe result to one explicit semantic field and fixed horizon.
- Plan 04-05 can reject incompatible compiler/runtime identity before applying these numerical policies.

## Self-Check: PASSED

- Commits `e701a50` and `e641578` exist.
- All six planned artifacts exist and focused/full gates pass.
- No global epsilon, wildcard field policy, automatic tolerance growth, or unbounded diagnostic context was introduced.

***

_Phase: 04-math-settings-and-numerical-policy_
_Completed: 2026-07-11_
