---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "01"
subsystem: particles
tags: [rust, particle-groups, domain-types, borrow-scoped-views, bitflags]

requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    provides: "Stable ParticleId and ParticleGroupId identities, checked particle definitions, owned storage, and borrow-scoped particle views"
provides:
  - "Owned invariant-bearing filled-shape, stroke-shape, and explicit-position group sources"
  - "Independent new-group and append-target destinations with exact public group flags"
  - "Checked generic particle-group recipes with typed validation failures"
  - "Borrow-scoped stable-ID group inspection with aligned optional depth"
affects: [10-09-group-world-api, particle-storage, group-topology, public-api]

tech-stack:
  added: []
  patterns:
    - "Owned source wrappers make empty and contradictory group inputs unrepresentable"
    - "Public flag newtype retains unknown public bits while stripping upstream-private bits"
    - "Crate-private checked view construction protects aligned borrowed semantic lanes"

key-files:
  created:
    - crates/liquidfun/src/particle/group.rs
    - crates/liquidfun/src/particle/group/tests.rs
  modified:
    - crates/liquidfun/src/particle.rs

key-decisions:
  - "Retain unknown public particle-group bits for ParticleFlags policy symmetry, but strip the pinned upstream-private 0x0018 mask at every public construction and bit operation."
  - "Keep ParticleGroupView construction crate-private, reject depth/member misalignment, and normalize retained-empty aggregate statistics to exact positive zero."

patterns-established:
  - "Particle-group geometry and destination are independent sum types rather than nullable C++-shaped fields."
  - "Particle-group views expose stable semantic identities and borrowed aligned values without dense rows or mutable storage."

requirements-completed: [PART-09, PART-10, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T10:54:31Z

duration: 2h 18m
completed: 2026-07-19
---

# Phase 10 Plan 01: Particle Group Contract Summary

**Owned checked group recipes and borrow-scoped stable-ID inspection establish the safe public contract without exposing private lifecycle bits or dense storage**

## Performance

- **Duration:** 2h 18m
- **Started:** 2026-07-19T08:36:19Z
- **Completed:** 2026-07-19T10:54:31Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Encoded exactly one filled union, stroke shape, or explicit-position source with non-empty owned inputs and source-specific shape validation.
- Separated new-group versus append-to-group destination from geometry while preserving every checked LiquidFun creation property and typed association.
- Exposed exactly the three named public group flags while retaining other unknown public bits and making the two upstream-private bits unrepresentable.
- Added a crate-private constructed `ParticleGroupView` with stable member identities, aligned optional depth, complete aggregate observations, and compile-fail lifetime/API guards.

## Task Commits

Each task was committed atomically:

1. **Task 1: Encode invariant-bearing recipes and group flags** - `dc65c89`
1. **Task 2: Define the borrow-scoped group view contract** - `beb3f70`

## Files Created/Modified

- `crates/liquidfun/src/particle/group.rs` - Owned source, destination, recipe, public flags, typed errors, and borrow-scoped view contracts.
- `crates/liquidfun/src/particle/group/tests.rs` - Focused Arrange/Act/Assert coverage and exact empty/alignment behavior.
- `crates/liquidfun/src/particle.rs` - Curated particle-group public exports.

## Decisions Made

- Unknown group bits follow the existing particle-flag round-trip policy except for the pinned upstream-private `0x0018` mask, which is always stripped.
- The group view derives position and angle from its transform, borrows source-ordered member/depth slices, and normalizes only empty aggregate statistics while preserving the group transform.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Prevented private group bits from entering the public flag value**

- **Found during:** Task 2 threat-surface review
- **Issue:** The initial `bitflags` value inherited `from_bits_retain`, which could construct and expose the two pinned upstream-private lifecycle/cache bits despite omitting named constants.
- **Fix:** Replaced the public `bitflags` declaration with a narrow value type whose retained-bit constructor and bit operations always remove `0x0018`; added a regression assertion.
- **Files modified:** `crates/liquidfun/src/particle/group.rs`, `crates/liquidfun/src/particle/group/tests.rs`
- **Verification:** Exact named-bit assertions, private-mask rejection, strict Clippy, full all-feature tests, and rustdoc all pass.
- **Committed in:** `beb3f70`

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The correction enforces the plan's core privilege-boundary invariant without expanding scope.

## Issues Encountered

- The first Task 1 strict-Clippy pass rejected exact float assertions; comparing exact `to_bits()` values preserved the intended contract and passed the ordered gate.
- Shared-workspace integration binary startup was slow. The exact commands remained unchanged and completed successfully with redirected logs.

## Known Stubs

- `crates/liquidfun/src/particle/group.rs:674` - The crate-private `ParticleGroupView` constructor/state seam is intentionally not yet called by production world code; Plan 10-09 wires it after same-world and same-system validation.
- `crates/liquidfun/src/particle/group.rs:336` - The public recipe contract is intentionally defined before its transactional world consumer; Plan 10-09 owns group creation integration.

These seams are intentional outputs of this contract-first plan and do not prevent its goal from being achieved.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 10-02 through 10-08 can build storage, topology, and group mutation internals against closed source and flag contracts.
- Plan 10-09 can consume the recipes and construct views only after live same-system handle validation.
- No blockers remain.

## Self-Check: PASSED

- Created and modified files exist.
- Task commits `dc65c89` and `beb3f70` exist on the current branch.
- Plan artifacts and key links pass the GSD validators.
- The final ordered Rust gate, focused group coverage, compile-fail doctests, and rustdoc passed.

***

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*
