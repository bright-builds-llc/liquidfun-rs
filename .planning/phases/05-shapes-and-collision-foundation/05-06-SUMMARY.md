---
phase: 05-shapes-and-collision-foundation
plan: "06"
subsystem: collision-time-of-impact
tags: [rust, collision, continuous-collision, toi, gjk, diagnostics]
requires:
  - phase: 05-shapes-and-collision-foundation
    plan: "03"
    provides: checked distance proxies, simplex cache, witnesses, and source-ordered GJK
  - phase: 04-math-settings-and-numerical-policy
    provides: checked sweeps, pinned scalar order, slop constants, and tau normalization
provides:
  - checked immutable shape-child TOI inputs and closed state/time outputs
  - source-ordered points, face-A, and face-B separation functions
  - exact bounded alternating-root TOI kernel with private diagnostic evidence
  - deterministic motion, topology, tie, cap, and pinned large-angle witnesses
affects: [05-07-differential-evidence, phase-6-rigid-world, continuous-collision]
tech-stack:
  added: []
  patterns: [checked borrowed input, closed output state, copied sweep normalization, bounded private trace]
key-files:
  created:
    - crates/liquidfun/src/collision/toi/separation.rs
    - crates/liquidfun/tests/collision_toi.rs
  modified:
    - crates/liquidfun/src/collision/toi.rs
key-decisions:
  - "Keep support indices, separation kind, root method, branch history, and iteration evidence private while exposing only checked input and closed state/time output."
  - "Preserve the pinned target/tolerance expressions, strict inequalities, first-support ties, root alternation, and exact iteration caps without adaptive policy."
  - "Normalize copied sweeps inside the kernel so large-angle compatibility does not mutate caller-owned values."
patterns-established:
  - "TOI boundary pattern: validate child topology and t_max before entering the numerical kernel."
  - "Differential evidence pattern: retain bounded semantic diagnostics privately without wall-clock state, pointers, or public iteration APIs."
requirements-completed:
  - COLL-06
  - COLL-07
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T17:44:05Z
duration: 16 min
completed: 2026-07-11
---

# Phase 5 Plan 06: Time of Impact Summary

**A native checked TOI kernel now preserves the pinned separation-axis search, fixed tolerances, alternating roots, and exact caps while exposing only a small closed public result.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-11T17:27:54Z
- **Completed:** 2026-07-11T17:44:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added checked borrowed TOI inputs over two validated shape children, copied sweeps, and finite `t_max` in `0.0..=1.0`, plus closed `Overlapped`, `Touching`, `Separated`, and `Failed` output states.
- Implemented private points, face-A, and face-B separation initialization, minimum separation, and evaluation with source-ordered support directions and strict first-vertex ties.
- Translated the pinned outer/push-back/root loops with target `max(LINEAR_SLOP, total_radius - 3 * LINEAR_SLOP)`, tolerance `0.25 * LINEAR_SLOP`, caps 20/50/`MAX_POLYGON_VERTICES`, and bisection-first alternating secant roots.
- Added bounded private diagnostics for separation kinds, root witnesses and methods, branch paths, iteration counts, and exact cap termination causes without exposing raw indices or profiling clocks.
- Proved initial overlap, touching at zero, separation through `t_max`, translation, combined translation/rotation, rotation without translation, near tangency, edge and chain children, symmetric ties, and the pinned large-angle testbed case.

## Task Commits

Each implementation task and acceptance correction was committed atomically:

1. **Task 1: Add checked TOI boundary and private separation functions** - `9e190ff` (`feat`)
2. **Task 2: Lock capped root behavior and witness matrix** - `4205d57` (`test`)
3. **Acceptance audit: Explicitly bound private branch diagnostics** - `330e7c7` (`fix`)

## Files Created/Modified

- `crates/liquidfun/src/collision/toi.rs` - Defines checked public input/output, the source-ordered capped TOI loop, alternating root search, and bounded private diagnostics.
- `crates/liquidfun/src/collision/toi/separation.rs` - Defines private shape-child support views and points/face-A/face-B separation behavior.
- `crates/liquidfun/tests/collision_toi.rs` - Exercises the public validation, motion, topology, tie, tolerance, cap, and pinned testbed contracts.

## Decisions Made

- The public input borrows immutable shapes but copies `Sweep` values. The kernel normalizes only those copies, preserving caller state while retaining upstream large-angle behavior.
- Child coordinates are revalidated against each selected shape before proxy construction, so a valid coordinate from a larger foreign topology cannot enter support lookup.
- Separation indices use a private closed enum rather than upstream's `-1` sentinels, preserving legal state combinations without exposing storage details.
- The distance cache remains the source of the separation axis, and support ties update only on strict improvement to retain the first source-ordered vertex.
- Diagnostics remain semantic and bounded: no public iteration fields, raw support storage, pointer identities, timers, clocks, or global call counters were introduced.
- The `toi.rs` entrypoint exceeds the general size guideline because its production half intentionally preserves one readable source-ordered kernel and its private test half verifies otherwise-unobservable diagnostics. Separation support logic is already isolated in the planned child module; another production seam would obscure source comparison.

## TDD Evidence

- **RED Task 1:** The focused public input test failed on unresolved `TimeOfImpactInput`, `TimeOfImpactState`, and `time_of_impact` imports before the checked boundary existed.
- **GREEN Task 1:** Five public input/copy tests and four private separation-kind/tie tests passed, followed by the complete mandated Rust gate.
- **RED Task 2:** Private exact-formula and cap tests failed on missing `target_and_tolerance` and `iteration_reached_cap` helpers before the decision points were isolated.
- **GREEN Task 2:** Formula, cap, root-order, termination, motion, topology, tie, tolerance, and testbed filters passed, followed by the complete mandated Rust gate.
- **REFACTOR:** Centralized target/tolerance construction and exact cap recording so production branches and private diagnostics share the same tested decisions.

## Deviations from Plan

### Implementation Sequencing

- **Found during:** Task 1
- **Issue:** The public `time_of_impact` contract required an executable kernel, while Task 2 separately called for completing and exhaustively witnessing that loop.
- **Resolution:** Implemented the source-ordered loop with the checked public boundary in Task 1, then used Task 2 for exact decision isolation, cap diagnostics, and the complete witness matrix.
- **Impact:** No file, dependency, or behavior scope expansion; the planned three files and two conceptual tasks remained unchanged.

### Acceptance Audit Commit

- **Found during:** Plan-level simplification and evidence review
- **Issue:** Every diagnostic collection was algorithmically capped, but `is_bounded` did not explicitly check the branch-history vector.
- **Resolution:** Added the conservative source-derived branch-history bound and a focused at/above-bound test in `330e7c7`.
- **Impact:** Private evidence is now mechanically bounded across every collection; no public API changed.

## Issues Encountered

- Another local process held the shared Cargo target lock, so all development and verification commands used the isolated `target/plan05-06` directory. No external process or user artifact was modified.
- A deterministic rotating-box probe did not find a natural public `Failed` witness in its bounded grid. Exact outer/root/push-back cap boundaries, cap labels, and closed `Failed` output are therefore tested privately at the decision boundary, while public adversarial witnesses prove bounded closed results.
- `cargo package` reports the repository's existing warnings that integration tests are excluded from the published include list. The verified 48-file package contains only Cargo metadata, documentation/license files, and native Rust sources.

## Verification

- All focused `input`, `separation`, `states`, `caps`, and `testbed_witness` commands passed.
- Every implementation and acceptance-audit commit was preceded by `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests in the mandated order.
- The final full gate passed 136 library tests, 17 TOI integration tests, all existing integration suites, and 10 rustdoc compile-fail tests.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps` passed on the final implementation.
- `cargo package -p liquidfun --allow-dirty` packaged and verified successfully; the package list contains no C++ source or private oracle tooling.
- Formula, strict-inequality, support-direction, root-parity, and exact-cap source audits passed against pinned `b2TimeOfImpact.cpp` and the testbed witness.
- Forbidden public iteration/support, timer/clock, and unsafe scans passed.
- `git diff --check` passed, and `.planning/STATE.md` plus `.planning/config.json` remained outside every commit.

## Requirement Scope

- `COLL-06` now supports checked shape-child sweep queries and the pinned closed TOI states across the planned motion and topology witnesses.
- `COLL-07` now has exact private evidence for separation kind, root method, termination branch, cap boundary, and bounded diagnostic history.
- World contact ownership, substepping, wake behavior, and solver integration remain Phase 6 work.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 05-07 can consume bounded private TOI diagnostics for first-divergence comparison without widening the consumer API.
- Phase 6 can call the checked TOI kernel from world-owned continuous-collision orchestration while retaining native Rust isolation.

## Self-Check: PASSED

- Implementation commits `9e190ff`, `4205d57`, and `330e7c7` exist in history.
- All three planned production/test artifacts exist on disk.
- Full Rust, rustdoc, package, source-order, forbidden-pattern, and diff checks passed.
- Only pre-existing orchestrator changes to `.planning/STATE.md` and `.planning/config.json` remain outside the plan commits.
- No push was performed.

***

_Phase: 05-shapes-and-collision-foundation_
_Completed: 2026-07-11_
