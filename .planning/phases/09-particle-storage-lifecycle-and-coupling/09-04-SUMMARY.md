---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "04"
subsystem: particle-buffers
tags: [rust, particles, owned-buffers, fixed-capacity, growable-capacity]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "02"
    provides: authoritative particle lane storage and atomic total-lane permutations
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "03"
    provides: system-owned particle storage, public creation, and complete teardown evidence
provides:
  - safe transfer of complete owned particle lane bundles into and out of a live system
  - explicit fixed and growable capacity contracts independent of allocator capacity
  - allocation-preserving teardown after transactional particle permutations
affects: [09-05, 09-06, 09-07, 09-08, 09-09, phase-10]
tech-stack:
  added: []
  patterns: [owned lane transfer, explicit declared capacity, teardown receipt]
key-files:
  created:
    - crates/liquidfun/src/particle/buffer.rs
    - crates/liquidfun/tests/particle_buffers.rs
  modified:
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/definition.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/permutation.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/world/particle_object.rs
key-decisions:
  - "Expose positions, velocities, flags, and optional colors as the supported consumer-supplied lanes; application associations and engine-derived lanes retain their existing owners."
  - "Treat ParticleBufferMode's declared count as the behavioral authority; inspect Vec capacity only to validate that transferred backing can satisfy the declared initial or fixed allocation."
  - "Prepare permutations transactionally, then copy validated rows back into transferred vectors so successful compaction preserves caller-supplied allocations."
patterns-established:
  - "Adoption consumes one complete lane bundle and every rejection returns that bundle unchanged to the caller."
  - "System teardown returns destruction records and the final semantic lane contents in one owned receipt."
requirements-completed: [API-10, PART-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T06:08:00Z
duration: 28 min
completed: 2026-07-15
---

# Phase 9 Plan 04: Owned Particle Buffer Contracts Summary

**Added a safe owned-lane transfer API whose explicit fixed/growable contracts remain transactional through creation, compaction, failure, teardown, and reuse.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-07-15T05:40:00Z
- **Completed:** 2026-07-15T06:08:00Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added public owned lane bundles for positions, velocities, flags, and optional colors, with all-or-nothing validation and ownership-returning errors.
- Added world adoption and teardown APIs that install the bundle in the existing authoritative `ParticleStorage` and return its final semantic contents with destruction records.
- Made fixed fullness and growable maximum behavior depend on explicit declarations rather than allocator capacity, with failures preserving particle rows, diagnostics, and transferred allocations.
- Preserved the original required and optional vector allocations across transactional compaction instead of replacing them with permutation scratch vectors.
- Covered unequal backing capacities, optional-lane presence, exact fullness, maximum conflicts, compaction, repeated reuse, and compile-time alias exclusion.

## TDD Evidence

### Task 1: Validate and adopt complete owned lane bundles

- **RED:** `cargo test -p liquidfun --test particle_buffers adoption` failed because the public buffer types and world adoption/teardown methods did not exist.
- **GREEN:** The focused adoption suite passed 2 tests after adding validated ownership transfer, world integration, teardown receipts, and crate-root reachability.

### Task 2: Enforce fixed/growable/full capacity transactionality

- **RED:** `cargo test -p liquidfun --test particle_buffers` failed because returned lanes lacked the required clear-and-reuse operation.
- **GREEN:** The complete buffer suite passed 9 tests after adding allocation-retaining `clear` and the fixed, growable, optional-lane, failure-atomicity, compaction, and repeated-cycle cases.

### Simplification and correctness pass

- **RED:** The new allocation-identity assertion failed after compaction because permutation commit replaced the caller's vector with a scratch vector.
- **GREEN:** Required and optional caller-supplied lane pointers remain identical through compaction and teardown after commit began clearing and extending existing allocations.

## Task Commits

Each implementation or verification unit was committed atomically:

1. **Adopt owned particle buffer bundles** - `2cad273` (feat)
1. **Prove buffer capacity transactionality** - `78ced0d` (test)
1. **Preserve supplied lanes through permutations** - `456ecb1` (fix)

## Files Created/Modified

- `crates/liquidfun/src/particle/buffer.rs` - Owned lane, bundle, mode, validation error, adoption error, and teardown receipt contracts.
- `crates/liquidfun/tests/particle_buffers.rs` - Black-box adoption, capacity, failure, compaction, reuse, and allocation-identity regressions.
- `crates/liquidfun/src/lib.rs` - Crate-root re-exports for the supported buffer API.
- `crates/liquidfun/src/particle.rs` - Particle-namespace declaration and re-exports.
- `crates/liquidfun/src/particle/definition.rs` - Private conversion from the public buffer mode to the existing capacity policy.
- `crates/liquidfun/src/particle/storage.rs` - Transfers the public lanes into and out of the sole authoritative storage owner.
- `crates/liquidfun/src/particle/storage/permutation.rs` - Commits prepared row permutations into existing transferred allocations.
- `crates/liquidfun/src/world/particle_object.rs` - Public adoption entrypoint over the established system-creation transaction.
- `crates/liquidfun/src/world/object.rs` - Teardown receipt plumbing through the established destruction cascade.

## Decisions Made

- The safe external-buffer equivalent transfers owned vectors rather than accepting borrowed slices, raw pointers, allocator traits, or foreign-memory callbacks. Rust move and borrow rules therefore keep one unambiguous owner throughout adoption.
- Positions, velocities, flags, and optional colors are consumer-supplied. Group membership and derived solver lanes remain engine-owned, while application associations remain in `AssociationMap`; this avoids introducing duplicate authorities.
- Fixed bundles validate backing capacity up front and never grow. Growable bundles validate their declared initial backing, then follow the independently declared system maximum.
- Teardown returns both the existing ordered destruction records and the final supplied lanes. Callers can clear those lanes without releasing allocations and adopt them into another system.

## Deviations from Plan

### Auto-fixed Blocking Integration Files

**1. Connected owned lanes to the existing authoritative storage**

- **Rule:** Rule 3 - Blocking
- **Found during:** Task 1
- **Issue:** The new public contract could not transfer ownership or return final semantic contents without a narrow adapter on the system's sole `ParticleStorage` owner.
- **Fix:** Added bundle conversion at storage construction and teardown; no parallel buffer authority was introduced.
- **File:** `crates/liquidfun/src/particle/storage.rs`
- **Verification:** Adoption, teardown, compaction, and repeated reuse tests pass, along with the all-feature suite.

**2. Reused the established transactional system-creation boundary**

- **Rule:** Rule 3 - Blocking
- **Found during:** Task 1
- **Issue:** Adoption had to reconcile buffer capacity with the checked particle-system definition and return the bundle on either definition or world insertion failure.
- **Fix:** Added one world adoption method that derives the definition capacity from the buffer mode and delegates to the existing system insertion path.
- **File:** `crates/liquidfun/src/world/particle_object.rs`
- **Verification:** Fixed maximum conflicts and fixed-full failures return ownership without mutating world particle state or diagnostic identity.

**3. Returned lanes through the established destruction transaction**

- **Rule:** Rule 3 - Blocking
- **Found during:** Task 1
- **Issue:** Complete teardown evidence was already produced inside the private world object cascade, so returning lanes elsewhere would duplicate removal authority or lose records.
- **Fix:** Extended that transaction to carry final lanes into the public teardown receipt.
- **File:** `crates/liquidfun/src/world/object.rs`
- **Verification:** Teardown returns ordered destruction records and the compacted survivor lanes; ordinary system destruction continues to pass existing lifecycle tests.

### Auto-fixed Correctness Issue

**4. Preserved caller-supplied allocations during permutation commit**

- **Rule:** Rule 1 - Bug
- **Found during:** Post-Task-2 simplification and diff review
- **Issue:** The transactional permutation prepared scratch vectors correctly but then replaced required and optional transferred vectors during commit, violating the promise to return the original owned allocations.
- **Fix:** Commit now clears and extends existing row-lane vectors from the validated candidate, preserving their allocation identities while retaining atomic preparation.
- **File:** `crates/liquidfun/src/particle/storage/permutation.rs`
- **Verification:** A focused pointer-identity regression failed before the fix and passes afterward for positions and optional colors; all permutation tests and the all-feature suite pass.

**Total deviations:** 4 auto-fixed (3 blocking integration files, 1 correctness bug). No Phase 10 solver behavior, Phase 09-06 eviction policy, unsafe code, raw-pointer API, allocator abstraction, or GPU interoperability was added.

## Issues Encountered

- The first warning-denied gate found a public panic-doc obligation and a large inline error payload. Capacity conversion moved behind a private helper and the returned bundle was boxed before the ordered gate was restarted and passed.
- The final simplification pass exposed allocation replacement during compaction. The regression and focused fix were committed separately before this summary.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later Phase 9 plans can use the established declared-capacity and teardown contracts without adding a second particle-storage authority.
- Phase 09-06 remains responsible for destroy-by-age eviction. The growable maximum regression explicitly disables that policy and proves typed no-effect fullness instead.
- Phase 10 remains outside this plan; no particle contacts, pair/triad generation, or solver behavior was added.

## Self-Check: PASSED

- All three implementation/test commits are present and all 9 changed source/test files exist.
- `cargo check -p liquidfun --all-features` passes.
- The focused adoption suite passes 2 tests and the complete particle-buffer suite passes all 9 tests.
- `cargo test -p liquidfun --doc` passes the lane-alias compile-time rejection example and all other doctests.
- The ordered format, warning-denied Clippy, all-target/all-feature build, and all-feature test gates pass.
- The base-to-head scan finds no unsafe code, raw-pointer types, allocator traits, or GPU interoperability in the changed buffer path.
- `.planning/STATE.md` and `.planning/ROADMAP.md` are unchanged.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
