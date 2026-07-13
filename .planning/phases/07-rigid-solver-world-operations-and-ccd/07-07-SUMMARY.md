---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "07"
subsystem: rigid-origin-shifting
tags: [rust, origin-shift, broad-phase, transactional-commit, covariance]
requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "06"
    provides: Semantic world AABB query and ray-cast surfaces
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "04"
    provides: Transactional body, sweep, and proxy staging patterns
provides:
  - Checked atomic World origin shifting across body transforms, sweeps, fixture proxies, and dynamic-tree bounds
  - Topology-preserving broad-phase prepare/commit translation with stable proxy and move-buffer identity
  - Public AABB-query and ray-cast translation-covariance evidence
  - Typed no-effect rejection for locked, non-finite, overflowing, and internally inconsistent shifts
affects: [07-11, phase-7-rigid-evidence, joints]
tech-stack:
  added: []
  patterns: [checked prepare-then-commit, opaque broad-phase candidate, semantic translation covariance]
key-files:
  created:
    - crates/liquidfun/src/world/origin.rs
    - crates/liquidfun/tests/rigid_origin_shift.rs
  modified:
    - crates/liquidfun/src/collision/broad_phase.rs
    - crates/liquidfun/src/collision/tree.rs
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/world.rs
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun/src/world/object.rs
    - crates/liquidfun/src/world/proxy.rs
    - crates/liquidfun/src/world/step.rs
key-decisions:
  - "Prepare every translated body, proxy, and tree value before replacing any live world state."
  - "Shift the existing dynamic tree in place through an opaque checked candidate so topology, proxy generations, and move-buffer contents remain unchanged."
  - "Compare public query and ray results by semantic fixture-child identity and declared numeric meaning without adding a callback-order contract."
patterns-established:
  - "Origin-shift transaction: validate lock and arithmetic, stage all coordinate-owning lanes, then perform one infallible world commit."
  - "Translation covariance: translate query inputs with the world and compare semantic multisets, exact fractions and normals, and shifted hit points."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T05:45:22Z
duration: 20 min
completed: 2026-07-13
---

# Phase 7 Plan 07: Transactional Origin Shifting Summary

**Rigid worlds now support checked atomic origin translation that preserves world identity, broad-phase topology, contacts, and translation-covariant public observations.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-13T05:25:12Z
- **Completed:** 2026-07-13T05:45:22Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added `World::shift_origin` with typed rejection for poisoned or locked worlds, non-finite shifts, body/proxy arithmetic overflow, inconsistent proxy ownership, and invalid internal broad-phase state.
- Staged complete body transform and sweep translations together with fixture proxy bookkeeping and dynamic-tree bounds before one no-fail commit.
- Preserved handles, contacts, filters, velocities, forces, torque, sleep state, tree topology, free-list links, proxy generations, and move-buffer contents across successful shifts.
- Proved AABB occurrence-multiset and termination covariance for duplicate fixture children, static bodies, sleeping bodies, and retained contacts.
- Proved ray continue, ignore, and clip covariance through stable semantic identity, exact fraction bits and normals, and translated hit points, plus empty-world, large finite, and repeated inverse shifts.

## Task Commits

Each task was committed atomically after the exact ordered Rust gate passed:

1. **Task 1: Prepare a complete checked origin-shift candidate** - `649af8b` (`feat`)
2. **Task 2: Prove query and ray-cast covariance across origin shifts** - `fce8f3f` (`test`)

## Files Created/Modified

- `crates/liquidfun/src/world/origin.rs` - Complete world-level prepare/commit transaction, typed errors, and private rollback/preservation evidence.
- `crates/liquidfun/tests/rigid_origin_shift.rs` - Public rejection, query, ray, contact, sleeping/static, empty, large-shift, and round-trip covariance evidence.
- `crates/liquidfun/src/collision/tree.rs` - Checked tree-origin candidate and in-place commit with topology/proxy-identity tests.
- `crates/liquidfun/src/collision/broad_phase.rs` - Opaque broad-phase shift candidate and commit surface that preserves the move buffer.
- `crates/liquidfun/src/world/body.rs` - Complete checked translation of body transform and sweep world-space lanes.
- `crates/liquidfun/src/world/proxy.rs` - Checked fixture-proxy candidate staging with live payload and child-order validation.
- `crates/liquidfun/src/world/object.rs` - Narrow sibling-module access needed to stage body and fixture lanes as one transaction.
- `crates/liquidfun/src/world/step.rs` - Narrow lock-state query and test-only lock seam for typed no-effect rejection evidence.
- `crates/liquidfun/src/world.rs` and `crates/liquidfun/src/lib.rs` - Origin module integration and curated public error export.

## Decisions Made

- Kept the candidate boundary deep: `World` owns the cross-lane transaction, while body, fixture-proxy, broad-phase, and tree modules prepare only their cohesive state.
- Used finite subtraction checks for every translated coordinate instead of assuming that a finite shift preserves finite state.
- Kept the tree allocation and topology intact. The commit updates only active-node AABBs after validation and does not rebuild, reinsert, reorder, or regenerate proxies.
- Preserved proxy-local bookkeeping alongside tree fat AABBs so later synchronization and CCD observe one coordinate frame.
- Reserved joint-anchor translation for Phase 8 as planned; Plan 07-07 changes only rigid body, sweep, fixture proxy, and broad-phase coordinates currently owned by the world.

## Test Evidence

- Task 1 RED failed with missing `OriginShiftError` and `World::shift_origin` symbols, proving the public behavior was absent before implementation.
- Task 1 GREEN passed invalid-input atomicity, tree topology/proxy identity, locked-world rollback, inconsistent-proxy rollback, and successful full-state preservation tests.
- Task 2 began after Task 1 had already implemented the complete behavior, so its first public black-box covariance run passed 4/4. This task added coverage rather than requiring another production change; no artificial RED was recorded.
- Focused final checks passed:
  - `cargo test -p liquidfun --test rigid_origin_shift` - 4/4
  - `cargo test -p liquidfun --test rigid_world_queries` - 15/15
  - `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings`
- The exact ordered Rust gate passed before both retained task commits:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`
- The final Task 2 gate exited 0 after all 147 library tests, every integration target including the 4 origin-shift and 15 world-query cases, and all 12 doctests.
- Complete-diff checks reported no whitespace errors, placeholder implementation, `unsafe` code, network endpoint, or authentication path in the added lines.

## Simplification Review

- One world transaction and one opaque broad-phase candidate cover every fallible coordinate update; no rollback log, tree reconstruction, public proxy API, or new dependency is needed.
- Existing body snapshots, fixture semantic identities, query visitors, and ray-hit types provide all public evidence, avoiding test-only production collectors or ordering layers.
- The implementation reuses the existing tree and proxy storage in place, which is both smaller and more faithful than re-creating broad-phase state.
- The only widened module visibility is sibling-private and supports the single deep transaction; storage representation remains hidden from consumers.

## Deviations from Plan

- `crates/liquidfun/src/world/body.rs`, `crates/liquidfun/src/world/step.rs`, and `crates/liquidfun/src/lib.rs` were additionally modified. Complete sweep translation belongs with `BodyState`, lock rejection needed a narrow internal/test seam, and the typed public method error needed a curated crate-root export. These are direct requirements of the plan rather than expanded feature scope.
- A locked world cannot be constructed through the safe public borrowing API while simultaneously calling `&mut World::shift_origin`; typed locked rejection is therefore proved in the module-private world test. All externally representable rejection and covariance behavior remains covered through black-box integration tests.

### Process adjustment: RED evidence was not committed

- Repository policy requires the complete ordered Rust gate before every commit. The intentionally failing Task 1 RED state was run and captured but not committed; only verified GREEN task states were retained.

## Issues Encountered

- A read-only diff scan initially used `status` as a shell variable, which is reserved by zsh. The scan was rerun with a non-reserved variable and completed successfully; no repository state changed.
- Cargo commands yielded through retained sessions during the full gates. The authoritative session exit code was observed before either commit; no commit was made while a gate process was running.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-11 can record origin-shift query multisets, termination counts, and closest-ray semantics through the established public evidence surface.
- Phase 8 can extend the same prepare/commit translation contract to joint-owned world-space anchors without weakening current rigid-world atomicity.
- No production blocker or residual origin-shift stub remains.

## Self-Check: PASSED

- Task commits `649af8b` and `fce8f3f` exist, and both declared created files exist.
- All ten implementation/test files in the two-task diff are represented above; focused and full verification passes.
- Diff, stub, and threat scans found no unintended side effect, unsafe block, network/authentication surface, or filesystem boundary.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-13*
