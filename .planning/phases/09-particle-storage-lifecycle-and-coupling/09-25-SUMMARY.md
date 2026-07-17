---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "25"
subsystem: phase9-differential-comparison
tags: [particles, differential, oracle, retained-rigid, evidence-integrity]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "24"
    provides: reviewed Phase 9 particle comparator and promoted evidence corpus
provides:
  - canonical retained Phase 6 through Phase 8 comparison before Phase 9 particle policies
  - typed retained-rigid harness failures and first-divergence mismatch outcomes
  - persistent particle-coupled rigid snapshots across downstream oracle checkpoints
affects: [phase-09-verification, phase-10, compatibility-evidence]
tech-stack:
  added: []
  patterns: [embedded digest-checked policy authority, retained-first comparator composition, sticky oracle state overlay]
key-files:
  created: []
  modified:
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs
    - crates/liquidfun-differential/src/rigid_evidence/phase7.rs
    - crates/liquidfun-differential/tests/phase9_corpus.rs
    - crates/liquidfun-differential/tests/particle_oracle.rs
    - tools/reference/src/rigid_world_phase9_execute.hpp
key-decisions:
  - "Expose one public Phase 9 complete-comparison seam with no injectable policy arguments; load and digest-check the exact embedded Phase 6, 7, and 8 profiles internally."
  - "Return the first retained rigid mismatch immediately and evaluate the 22 particle policies only after the complete retained comparator returns Match."
  - "Treat particle effects on rigid bodies as persistent oracle state and fail closed when the combined Phase 9 executor encounters an unsupported retained action."
patterns-established:
  - "Later-phase observations are projected out of retained walkers while their original source indices are preserved for deterministic failure signatures."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-07, PART-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-17T21:25:59Z
duration: 71 min
completed: 2026-07-17
---

# Phase 9 Plan 25: Retained Rigid Comparator Closure Summary

**Phase 9 now proves every retained Phase 6 through Phase 8 rigid result before particle comparison, with canonical digest-checked policies and deterministic retained-first mismatch evidence.**

## Performance

- **Duration:** 71 min
- **Started:** 2026-07-17T20:14:40Z
- **Completed:** 2026-07-17T21:25:59Z
- **Tasks:** 3
- **Files modified:** 6 implementation, oracle, and regression files

## Accomplishments

- Added `compare_complete_phase9_rigid_world_results`, which accepts one request/native/oracle result triple and exposes no caller-supplied retained policy authority.
- Embedded the exact checked-in Phase 6, 7, and 8 TOML profiles, verified each content SHA-256 before parsing, and translated parser or digest failures into typed harness errors.
- Composed `compare_phase8_rigid_world_results` before the particle-only comparator and added named retained harness-failure and mismatch variants.
- Added independent body, fixture, retained numeric, retained-before-particle, and subprocess regressions that assert the same deterministic first-divergence signatures as the Phase 8 comparator.
- Preserved the established 22-path consumed-policy evidence contract while keeping the complete comparator call singular in `run_phase9_differential`.
- Corrected the C++ Phase 9 overlay so particle-induced rigid state survives particle-system teardown and remains authoritative at every downstream retained checkpoint.
- Made the combined oracle executor handle the retained noncolliding body/fixture lifecycle actions and reject unsupported retained actions instead of silently emitting baseline state.
- Preserved required retained witness checkpoints in synthetic particle-oracle requests while keeping separate Phase 9 observation checkpoints.

## Task Commits

1. **Tasks 1-3: Prove and close retained Phase 9 comparator coverage** - `8f50662`

**Plan metadata:** committed after summary and state tracking verification.

## Verification

- `cargo test -p liquidfun-differential --test phase9_corpus` - 12 passed, 0 failed, 1 ignored
- `cargo test -p liquidfun-differential --test particle_oracle` - 13 passed, 0 failed
- `cargo test -p liquidfun-differential --test particle_protocol` - 25 passed, 0 failed
- Fresh `oracle-debug` configure and build after the adapter change
- Strict executable Phase 9 corpus - all cases returned Match after the oracle overlay fix
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features` including 16 doctests
- `git diff --check`

Local oracle verification reported the existing noncanonical macOS tool identities: CMake 3.27.9 and Apple Clang 21 instead of canonical CMake 4.3.3 and Clang 22.1.8. The local run was verification only and did not promote or rewrite compatibility evidence.

## Files Modified

- `crates/liquidfun-differential/src/rigid_world.rs` - Owns canonical retained profile loading, complete comparator composition, and the singular runner call.
- `crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs` - Adds retained harness-error and first-divergence mismatch variants.
- `crates/liquidfun-differential/src/rigid_evidence/phase7.rs` - Projects later particle observations out of the retained walker while preserving source indices.
- `crates/liquidfun-differential/tests/phase9_corpus.rs` - Covers body, fixture, numeric, ordering, and subprocess retained mismatches.
- `crates/liquidfun-differential/tests/particle_oracle.rs` - Preserves retained witness authority in synthetic Phase 9 requests.
- `tools/reference/src/rigid_world_phase9_execute.hpp` - Keeps particle-coupled rigid state sticky and executes downstream retained lifecycle mutations.

## Decisions Made

- Canonical retained policies are compile-time embedded inputs with reviewed content digests; the public API cannot substitute profiles.
- Retained rigid comparison is the sole first stage. Particle comparison is unreachable after a retained harness error or mismatch.
- Phase 9 runner evidence continues to report all 22 reviewed particle policy paths independently of the match/mismatch outcome.
- Combined oracle state, not the separately executed baseline result, owns all body and fixture snapshots after any Phase 9 action can affect rigid state.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Made the retained Phase 7 walker ignore later particle observations**

- **Found during:** Task 2 focused retained-mismatch verification
- **Issue:** The inherited observation walker panicked when a structurally valid Phase 9 result added `Particle` observations after the retained variants.
- **Fix:** Projected particle observations out before retained comparison while retaining original observation indices for action/stage attribution.
- **Files modified:** `crates/liquidfun-differential/src/rigid_evidence/phase7.rs`
- **Commit:** `8f50662`

**2. [Rule 1 - Bug] Preserved particle-coupled rigid state in the C++ oracle overlay**

- **Found during:** Task 2 executable corpus verification
- **Issue:** Native and oracle body state matched at the Phase 9 checkpoint, but the oracle reset downstream retained checkpoints to the separately executed Phase 8 baseline after clearing a per-checkpoint flag. The strict comparator correctly exposed the false result.
- **Fix:** Made Phase 9 rigid-state authority sticky, executed every retained action used by the affected timeline on the combined world, and rejected unknown retained actions.
- **Files modified:** `tools/reference/src/rigid_world_phase9_execute.hpp`
- **Commit:** `8f50662`

**3. [Rule 3 - Blocking] Preserved retained witness registries in synthetic oracle requests**

- **Found during:** Task 2 `particle_oracle` verification after rebuilding the C++ adapter
- **Issue:** Test helpers converted required retained checkpoints into Phase 9 checkpoints. The fresh fail-closed oracle correctly rejected the now-incomplete retained witness registry.
- **Fix:** Kept a distinct retained checkpoint and a phase-local Phase 9 observation checkpoint with correct destruction counts.
- **Files modified:** `crates/liquidfun-differential/tests/particle_oracle.rs`
- **Commit:** `8f50662`

## Issues Encountered

- The first fresh oracle build correctly rejected a stale configured adapter digest after the C++ source changed. Re-running configure recomputed the source manifest digest before the successful build.
- The full `cargo test --all-features` gate traversed the repository's many integration binaries serially but remained active and completed without failure.

## User Setup Required

None.

## Next Phase Readiness

- `G09-DIFFERENTIAL-COMPARISON` now has executable body, fixture, numeric, ordering, and subprocess regression coverage.
- Phase 9 comparison is a strict semantic superset of retained Phase 8 comparison without weakening the particle policy registry.
- No compatibility authority, generated report, workflow, fixture, manifest, or Phase 10 file was promoted or rewritten.

## Self-Check: PASSED

- All six implementation, oracle, and regression files plus this summary exist.
- Implementation commit `8f50662` exists in repository history.
- Focused Phase 9 suites, fresh oracle corpus execution, all four mandatory Rust commands, and `git diff --check` passed.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-17*
