---
phase: 04-math-settings-and-numerical-policy
plan: "04"
subsystem: differential-math-probes
tags: [rust, ieee-754, jsonl, exact-bits, differential-testing]
requires:
  - phase: 04-math-settings-and-numerical-policy
    plan: "01"
    provides: Source-ordered scalar and vector kernels
  - phase: 04-math-settings-and-numerical-policy
    plan: "02"
    provides: Matrix, transform, rotation, and checked sweep kernels
  - phase: 04-math-settings-and-numerical-policy
    plan: "03"
    provides: Closed phase4-v1 numerical policy paths and horizons
provides:
  - Strict bounded JSONL math-probe request contract with closed operation and operand enums
  - Exact-bit result records with IEEE class, sign, discrete branch, and policy-path metadata
  - Stateless native executor and complete deterministic Phase 4 probe corpus
affects: [04-05-cpp-math-probes, phase-5-collision]
tech-stack:
  added: []
  patterns: [closed native dispatch, exact-bit wire transport, bounded scenario horizons, stateless pure executor]
key-files:
  created:
    - crates/liquidfun-test-protocol/src/scenario/math_probe.rs
    - crates/liquidfun-differential/src/math_probe.rs
    - protocol/fixtures/accepted/math-probe-request.jsonl
    - scenarios/phase-04/math-probes.json
  modified:
    - crates/liquidfun-test-protocol/src/scenario.rs
    - crates/liquidfun-test-protocol/src/trace.rs
    - crates/liquidfun-differential/src/lib.rs
    - crates/liquidfun-differential/src/rust_adapter.rs
key-decisions:
  - "Use a separate strict math-probe request boundary so pure probes cannot create or depend on world and solver state."
  - "Pair every operation with a closed structured input variant and validate the pair before exhaustive native dispatch."
  - "Carry exact bits, IEEE class, sign, discrete branch results, policy path, and fixed horizon together in ordered probe results."
patterns-established:
  - "Probe safety: bounded decode, duplicate rejection, closed enum dispatch, and no executable/path/function-name input surface."
  - "Numeric witnesses: preserve source-order operators and fixed exact-bit operands without unsafe code or fused multiply-add."
requirements-completed: [COLL-01, COLL-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
generated_at: 2026-07-11T05:59:58Z
duration: 28 min
completed: 2026-07-11
---

# Phase 4 Plan 04: Bounded Native Math Probe Contract Summary

**A strict exact-bit wire contract now drives every Phase 4 pure math operation and numerical witness through a bounded stateless native executor.**

## Performance

- **Duration:** 28 min
- **Completed:** 2026-07-11T05:59:58Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Added the complete closed operation enum for scalar, vector, matrix, rotation, transform, sweep, and all five numeric witness families.
- Added bounded structured operands, stable case identities, duplicate detection, strict unknown-field/operation rejection, maximum case counts, and maximum 32-step horizons before execution.
- Added ordered exact-bit outputs with zero/subnormal/normal/infinite/NaN classification, sign metadata, discrete predicates/branches, explicit `phase4-v1` policy paths, and fixed operation or scenario-step horizons.
- Added a stateless native executor that dispatches directly to `liquidfun::math` without constructing world or solver state and never uses `unsafe`, `unwrap()`, or fused multiply-add.
- Checked in a byte-stable 39-case corpus covering signed zeros, subnormals, normal boundaries, finite maxima, infinities, NaN payloads, epsilon neighbors, singular and near-singular matrices, tau and large angles, transforms, sweeps, repeated composition, and every numeric witness.

## Task Commits

1. **Task 1: Implement exact-bit protocol, native dispatch, and complete witness corpus** - `430e5bd` (feat)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Register the new native executor module in the differential crate**

- **Found during:** Task 1
- **Issue:** The planned new `math_probe.rs` executor requires crate-root registration to compile and be reachable by `rust_adapter.rs`; `crates/liquidfun-differential/src/lib.rs` was not listed in the plan files.
- **Fix:** Added the private module declaration and curated re-export.
- **Files modified:** `crates/liquidfun-differential/src/lib.rs`
- **Verification:** Strict Clippy, all-target build, focused executor tests, and the full test gate pass.
- **Committed in:** `430e5bd`

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Two module-wiring lines only; no protocol or public engine scope expansion.

## Issues Encountered

- The oversize-case regression initially asserted a narrower codec subcategory than the synthetic oversized record guaranteed. The test now verifies the required fail-closed codec rejection while the bounded decoder remains the enforcing mechanism.

## Verification

- `cargo test -p liquidfun-test-protocol math_probe --all-features` passed.
- `cargo test -p liquidfun-test-protocol math_probe_scenario_is_byte_stable --all-features` passed.
- `cargo test -p liquidfun-differential native_math_probe_executes_complete_witness_corpus --all-features` passed.
- `cargo test -p liquidfun-differential native_math_probe_witness_bits_are_exact --all-features` passed with exact cancellation, halfway, overflow, underflow, and non-fused FMA witness bits.
- Ordered Rust gate passed: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.
- `git diff --check` passed, and forbidden-surface review found no executable, compiler-flag, raw-pointer, arbitrary function-name, `unsafe`, `unwrap()`, or `mul_add` input/implementation path.

## User Setup Required

None - no external service or tool configuration required.

## Next Phase Readiness

- Plan 04-05 can implement the C++ adapter against one complete validated request/result contract and deterministic corpus.
- Canonical C++ evidence was not generated or promoted in this plan.

## Self-Check: PASSED

- Task commit `430e5bd` exists in history.
- All four created artifacts and four modified integration files exist.
- Every focused named check and the exact ordered full Rust gate pass.
- The checked-in corpus contains all required operation families, exceptional classes, numeric witnesses, and bounded repeated-composition cases.

***

_Phase: 04-math-settings-and-numerical-policy_
_Completed: 2026-07-11_
