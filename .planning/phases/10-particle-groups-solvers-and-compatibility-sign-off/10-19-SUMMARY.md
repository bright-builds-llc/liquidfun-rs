---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "19"
subsystem: particle-pressure-solvers
tags: [rust, particles, pressure, damping, source-parity]

requires:
  - phase: 10-02
    provides: Checked particle-system coefficients and solver configuration
  - phase: 10-04
    provides: Aligned storage-owned particle solver scratch
  - phase: 10-05
    provides: Stable source-ordered particle contacts and weights
  - phase: 10-06
    provides: Stable source-ordered particle/body contact records
  - phase: 10-12
    provides: Exact particle topology and contact ordering evidence
provides:
  - Exact distinct S14 static-pressure and S15 pressure kernels
  - Exact distinct S16 damping and S17 extra-damping kernels
  - Checked bounded static-pressure iterations and atomic velocity-lane replacement
  - Focused control, gate, interaction, and traversal-order witnesses
affects: [10-20, 10-22, particle-solvers, particle-world-coupling, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Complete validated particle-velocity candidate committed through storage authority
    - Body-coupling trait preserving sequential body-contact velocity observation
    - Exact source-statement f32 grouping and stable contact traversal

key-files:
  created:
    - crates/liquidfun/src/particle/solver/pressure.rs
    - crates/liquidfun/src/particle/solver/pressure/tests.rs
  modified:
    - crates/liquidfun/src/particle/definition.rs
    - crates/liquidfun/src/particle/solver.rs
    - crates/liquidfun/src/particle/storage.rs

key-decisions:
  - "Keep S14-S17 crate-private and body-agnostic behind a sequential BodyCoupling contract so Plan 10-22 can adapt candidate world bodies without changing numerical order."
  - "Build a complete particle velocity candidate and commit it only through ParticleStorage, preserving lane ownership and invalidating statistics only for groups containing changed particles."
  - "Cap public static-pressure iterations at the reviewed project solver bound of 1024 before any repeated numerical work."

patterns-established:
  - "Pressure-family order: body contacts observe and update candidate bodies sequentially before particle contacts consume the evolving particle velocity candidate."
  - "Pressure suppression: POWDER and TENSILE clear only dynamic pressure, while STATIC_PRESSURE contributes only its aligned scratch lane."

requirements-completed: [PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T02:59:46Z

duration: 1h
completed: 2026-07-19
---

# Phase 10 Plan 19: Exact Pressure and Damping Passes Summary

**Four crate-private, source-ordered pressure and damping kernels now reproduce pinned LiquidFun coefficients, gates, limits, and sequential contact/body effects behind bounded checked inputs.**

## Performance

- **Duration:** 1h
- **Started:** 2026-07-20T01:59:57Z
- **Completed:** 2026-07-20T02:59:46Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Implemented S14 static pressure with configured strength, relaxation, iteration count, critical-pressure clamp, stable contact accumulation, and exact per-iteration scratch replacement.
- Implemented S15 pressure with configured `0.05` strength, minimum-weight and maximum-pressure limits, exact POWDER/TENSILE dynamic-pressure suppression, STATIC_PRESSURE addition, body-contact-first traversal, and paired particle impulses.
- Implemented S16 damping with source-grouped linear/quadratic terms and sequential current body/particle velocity observation across body contacts before particle contacts.
- Implemented S17 extra damping with independent per-particle STATIC_PRESSURE gates and the pinned half-impulse body response.
- Added a reviewed 1,024 static-pressure iteration bound so public configuration cannot drive unbounded repeated work.
- Added eleven focused witnesses covering configuration boundaries, coefficient sensitivity, convergence, suppression, traversal order, sequential body effects, independent extra-damping gates, missing bodies, and exact empty/no-contact controls.
- Preserved the Phase 9 `run_particle_contact_prefix` implementation and call graph unchanged for Plan 10-22's later one-shot orchestration replacement.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement S14-S17 as exact distinct passes** - `b6a1e8c` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver/pressure.rs` - Exact crate-private S14-S17 kernels, checked body coupling, and pinned numerical helpers.
- `crates/liquidfun/src/particle/solver/pressure/tests.rs` - Focused control, coefficient, gate, interaction, failure, and traversal-order evidence.
- `crates/liquidfun/src/particle/definition.rs` - Reviewed static-pressure iteration maximum and typed out-of-range rejection.
- `crates/liquidfun/src/particle/solver.rs` - Private pressure-family module registration only.
- `crates/liquidfun/src/particle/storage.rs` - Minimal validated whole-velocity candidate replacement under the existing lane authority.

## Decisions Made

- Kept world ownership outside the numerical module. The small private `BodyCoupling` contract exposes only body existence, velocity-at-point, and ordered impulse application, which is exactly the state S15-S17 require.
- Replaced particle velocities as one checked candidate rather than exposing mutable lane slices. This rejects length/non-finite failures before mutation and keeps `ParticleStorage` as the sole aligned-lane authority.
- Invalidated cached group statistics only for particles whose velocity bits changed, preserving exact no-op controls and unrelated group caches.
- Mirrored the project's reviewed 1,024 solver-work ceiling for static-pressure iterations instead of retaining the prior signed-32-bit capacity ceiling.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Critical] Bounded public static-pressure iterations**

- **Found during:** Task 1 threat-model review
- **Issue:** `with_static_pressure_iterations` rejected zero but allowed values up to `i32::MAX`, contradicting the plan's denial-of-service mitigation and fixed bounded-loop success criterion.
- **Fix:** Added a public reviewed maximum of 1,024 and a typed out-of-range error before the definition can reach the solver.
- **Files modified:** `crates/liquidfun/src/particle/definition.rs`, `crates/liquidfun/src/particle/solver/pressure/tests.rs`
- **Verification:** Zero and maximum-plus-one rejection witnesses, focused tests, and the complete ordered Rust gate passed.
- **Committed in:** `b6a1e8c`

**2. [Rule 3 - Blocking] Added one validated storage-owned velocity commit seam**

- **Found during:** Task 1 implementation
- **Issue:** Particle velocity lanes are intentionally private and had no complete-candidate replacement path, so exact multi-contact kernels could not remain atomic without widening mutable storage access.
- **Fix:** Added `replace_solver_velocities`, which validates alignment and finiteness before replacement and invalidates statistics only for groups containing changed particles.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`
- **Verification:** Missing-body and byte-identical no-contact controls passed alongside the complete ordered Rust gate.
- **Committed in:** `b6a1e8c`

**Total deviations:** 2 auto-fixed (1 critical, 1 blocking)
**Impact on plan:** Both changes directly enforce planned trust-boundary requirements; no dependency, unsafe/FFI code, external I/O, or orchestration behavior was added.

## Issues Encountered

- The first focused expected-bit calculations exposed one-ULP differences caused by source-grouped `f32` operations and a body-contact weight contribution. The expectations were re-derived from the pinned statements rather than blessing observed output.
- Full all-target compilation and integration-test launches were slow on macOS, but the exact required commands completed without narrowing or skipping any gate.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-22 can adapt its owned candidate body set to `BodyCoupling` and replace the Phase 9 prefix atomically with the four independently witnessed passes.
- Pressure-family scratch, coefficients, ordering, gates, and checked work bounds are complete.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all five created or modified task files and this summary exist.
- Confirmed task commit `b6a1e8c` exists.
- Confirmed no stub patterns, unsafe/FFI code, new network endpoints, auth paths, file-access surfaces, or schema changes were introduced.
- Confirmed the focused 11-test witness suite and exact ordered Rust gate all pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*
