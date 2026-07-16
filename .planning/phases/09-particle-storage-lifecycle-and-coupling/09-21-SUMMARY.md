---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "21"
subsystem: phase9-differential-comparison
tags: [rust, cpp, particles, comparator, policy-registry, request-digest]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "20"
    provides: request/result lifecycle validation and action-specific particle observations
provides:
  - exhaustive typed Phase 9 particle observation comparison with stable first-divergence evidence
  - complete fail-closed consumption of the reviewed phase9-v1 policy registry
  - one-request native Rust versus pinned C++ runner with role-bound request digests
affects: [09-22, 09-23, 09-24, phase-10]
tech-stack:
  added: []
  patterns: [closed typed observation walker, canonical request role binding, physics-versus-harness classification]
key-files:
  created:
    - crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs
  modified:
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_world/phase9.rs
    - crates/liquidfun-differential/tests/particle_protocol.rs
    - crates/liquidfun-differential/tests/particle_oracle.rs
key-decisions:
  - "Treat observation-variant disagreement, invalid parallel structure, unknown policy, and non-finite numeric state as harness failure rather than physics evidence."
  - "Serialize and hash one bounded JSONL request before decoding the exact value supplied to both native and process roles."
patterns-established:
  - "Every Phase 9 comparator run validates the exact ordered policy list before walking observations and returns the complete consumed-path ledger."
  - "Physics disagreement is a normal first-divergence outcome; process, request, policy, and structural failures remain typed errors."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-16T06:58:25Z
duration: 34m
completed: 2026-07-16
---

# Phase 9 Plan 21: Native-versus-C++ Semantic Comparison Summary

**One canonical request now runs through native Rust and the pinned process oracle before every typed Phase 9 particle observation is compared under the complete closed policy registry.**

## Performance

- **Duration:** 34m
- **Started:** 2026-07-16T06:24:47Z
- **Completed:** 2026-07-16T06:58:25Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added an exhaustive walker for all nine `Phase9ParticleObservation` variants, preserving timeline, checkpoint, and observation source order in stable mismatch signatures.
- Applied exact discrete, exact-bit, four-ULP, absolute-relative, and dimensioned-absolute policies with finite-value guards and explicit reviewed constants.
- Made missing, duplicate, reordered, unknown, and wildcard policy registries fail closed while returning every required consumed path for a successful comparison.
- Added a runner that serializes and hashes one bounded request, executes its decoded value through native Rust and the selected process oracle, validates both results, and keeps physics mismatch distinct from harness failure.
- Proved a mixed live-rigid/live-particle request matches the rebuilt debug oracle, a deterministic field mutation retains its first-divergence identity, and malformed child output remains a process harness error.

## Task Commits

Each task was committed atomically:

1. **Task 1: Compare every Phase 9 semantic field under one named policy** - `a5d047d` (feat)
1. **Task 2: Run the exact same request through native Rust and pinned C++** - `e372ef7` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs` - Owns policy validation, the exhaustive observation walker, reviewed numeric helpers, and stable first-divergence evidence.
- `crates/liquidfun-differential/src/rigid_world/phase9.rs` - Exports the comparator beside the existing Phase 9 adapter and policy declarations.
- `crates/liquidfun-differential/src/rigid_world.rs` - Exposes the one-request/two-engine runner and typed physics-versus-harness result boundary.
- `crates/liquidfun-differential/tests/particle_protocol.rs` - Covers complete policy consumption, fail-closed registry defects, policy-class mutations, and replay-stable paths.
- `crates/liquidfun-differential/tests/particle_oracle.rs` - Covers real debug-oracle comparison, identical role digests, semantic mutation, and malformed-child classification.

## Decisions Made

- Kept the comparator inside the existing rigid-world Phase 9 deep module rather than introducing a parallel harness or a generic wildcard tolerance engine.
- Counted the complete reviewed policy registry as consumed only after exact ordered validation; the typed walker then selects only those closed paths.
- Classified mismatched observation variants and malformed parallel arrays as harness defects because they do not represent aligned physics fields.
- Used the existing bounded supervisor unchanged so handshake, provenance, output bounds, stderr separation, timeout, kill, and reap behavior continue to protect the C++ boundary.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first exact-path mutation attempted to clear an already-empty body identity list and therefore produced no disagreement. The regression was corrected to mutate a request-valid statistics count before GREEN acceptance.
- Running the new comparator against the broader existing all-action request exposed a real `particle.query.order` mismatch: native returned two particle IDs while the C++ result returned none. The plan-required mixed rigid/particle request compares cleanly; Plan 09-22 must include and close the broader executable query witness rather than hiding this first divergence.
- Shared-worktree Cargo and dynamic-loader contention delayed several focused runs, but every retained run completed normally and no check was skipped.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CR-01 and G09-DIFFERENTIAL-COMPARISON now have an executed same-request Rust-versus-C++ comparison and a closed, fully consumed policy registry.
- Plan 09-22 can bind executable branch witnesses to this runner and must resolve the newly exposed all-action query-order mismatch before evidence generation.
- Debug, release, and ASan/UBSan oracle presets configure and build locally; local AppleClang remains non-promotable D2 evidence.

## Self-Check: PASSED

- The created comparator file exists and both task commits contain plan ID `09-21`.
- Focused comparator tests pass 4/4; focused differential runner tests pass 3/3.
- Oracle debug, release, and ASan/UBSan presets configure and build with the pinned submodule unchanged.
- Both mandatory pre-commit sequences passed format, warning-denied Clippy, all-target/all-feature build, all-feature tests, and 16 doctests.
- No Phase 10 topology, pair/triad generation, particle solver, unsafe Rust, wildcard policy, or compatibility promotion was introduced.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-16*
