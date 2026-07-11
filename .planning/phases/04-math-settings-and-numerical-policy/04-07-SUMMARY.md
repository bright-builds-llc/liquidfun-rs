---
phase: 04-math-settings-and-numerical-policy
plan: "07"
subsystem: numerical-policy-documentation
tags: [documentation, compatibility-inventory, evidence, math, ieee-754]
requires:
  - phase: 04-math-settings-and-numerical-policy
    plan: "06"
    provides: Closed math-probe commands, schemas, and canonical/read-only CI routing
provides:
  - Discoverable public math/settings architecture and numerical-policy contract
  - Exact contributor commands, prerequisites, evidence locations, and promotion authority
  - Evidence-scoped compatibility status for the three Phase 4 math/settings rows
affects: [phase-5-collision, compatibility-reporting, contributor-verification]
tech-stack:
  added: []
  patterns: [machine-audited documentation contracts, evidence-derived compatibility claims, conservative platform status]
key-files:
  modified:
    - tools/xtask/src/docs.rs
    - tools/xtask/tests/docs_contract.rs
    - ARCHITECTURE.md
    - TESTING.md
    - COMPATIBILITY.md
    - README.md
    - reference/compatibility.json
key-decisions:
  - "Report local passing Rust/C++ math probes as scoped D2 differential evidence while leaving canonical D1 and platform validation unclaimed."
  - "Keep b2Settings differential validation absent because the cross-language corpus does not directly execute the complete settings constant surface."
  - "Make Phase 4 headings, policy names, special-value rules, horizons, tier authority, exact commands, evidence counts, and maturity limits executable documentation contracts."
patterns-established:
  - "Documentation sign-off: human claims must contain machine-checked policy and command markers and agree with the generated compatibility ledger."
  - "Compatibility promotion: change the machine-readable ledger first, regenerate the report, and cite only executable evidence for each independent dimension."
requirements-completed: [COLL-01, COLL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
generated_at: 2026-07-11T06:58:24Z
duration: 17 min
completed: 2026-07-11
---

# Phase 4 Plan 07: Numerical Policy Documentation and Compatibility Sign-Off Summary

**The public math contract, numerical policy, exact probe commands, and compatibility status are now discoverable, machine-audited, and limited to Phase 4 evidence.**

## Performance

- **Duration:** 17 min
- **Completed:** 2026-07-11T06:58:24Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Documented all public Phase 4 math types, initialized safe-Rust differences, immutable settings, MKS/radian units, column-major matrices, transform direction, and sweep conventions.
- Documented exact-bit transport separately from `ExactBits`, `Ulps`, `Absolute`, and `AbsoluteRelative` comparison, including NaN, infinity, signed-zero, collection, horizon, and D0-D3 authority rules.
- Published the exact debug, release, replay, and fixed two-run D0 commands with prerequisites, evidence locations, read-only behavior, and the rule that only D1 may promote canonical fixtures.
- Extended `cargo xtask docs check` and command-level tests to reject missing Phase 4 contracts, compatibility-count drift, and accidental absolute user paths.
- Updated only the b2Math, b2Settings, and common math/settings inventory rows; shapes, collision, solvers, particles, platform parity, performance parity, and production maturity remain pending.

## Task Commit

1. **Task 1: Document contracts and evidence-backed status** - `08745ae` (docs)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Update the machine-readable compatibility ledger before regenerating the report**

- **Found during:** Task 1
- **Issue:** `COMPATIBILITY.md` is generated and explicitly forbids direct editing, while the plan's file list named only the generated report.
- **Fix:** Updated exactly three entries in `reference/compatibility.json`, then regenerated `COMPATIBILITY.md` through `cargo xtask inventory generate`.
- **Files modified:** `reference/compatibility.json`
- **Verification:** Inventory validation and exact generated-report checks pass; no other compatibility row changed.
- **Committed in:** `08745ae`

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** One required source-of-truth file was added to scope; the compatibility change itself stayed limited to the three planned rows.

## Issues Encountered

- Local Apple Clang 21 remains noncanonical D2 evidence, as established by Plan 04-05. Debug and release comparisons pass, but no D1 or complete supported-platform claim was added.
- The math probe corpus does not directly execute the complete settings constant surface. The b2Settings row is therefore implemented, unit-tested, and documented as different, but not marked differentially or platform validated.

## Verification

- `cargo test -p xtask --test docs_contract` passed all 13 documentation contract tests.
- `cargo xtask docs check`, `cargo xtask inventory check`, and `cargo xtask provenance check` passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` passed.
- `cargo xtask package verify` built and tested the 26-entry package outside the repository with no C++ or private harness dependency.
- Debug and release math-probe compare, debug replay, and two independent byte-identical debug D0 runs passed for all 39 ordered cases.
- The forbidden-overclaim scan, `git diff --check`, and review for local paths, frontmatter safety, unrelated churn, and unsupported Phase 5+ claims passed.
- Ordered Rust gate passed before the task commit: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.

## User Setup Required

None - the documented local oracle commands use existing reviewed repository prerequisites.

## Next Phase Readiness

- Phase 4 implementation plans are complete and ready for phase-level review, security audit, and verification.
- Phase 5 collision work can consume the public math/settings conventions, closed numerical policy, and exact probe command surface without inferring broader parity.

## Self-Check: PASSED

- Task commit `08745ae` exists in history.
- All seven modified artifacts exist, and the compatibility report is byte-identical to its validated ledger rendering.
- Every focused documentation/evidence/package/oracle check and the exact ordered full Rust gate pass.
- Compatibility counts are 3 implemented, 3 unit-tested, 2 differentially validated, and 0 platform validated; no later-phase capability is promoted.

***

_Phase: 04-math-settings-and-numerical-policy_
_Completed: 2026-07-11_
