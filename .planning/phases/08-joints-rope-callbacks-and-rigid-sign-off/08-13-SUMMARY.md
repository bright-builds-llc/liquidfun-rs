---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "13"
subsystem: differential-evidence
tags: [cpp, rust, joints, rope, comparator, sanitizer]
requires:
  - phase: 08-12
    provides: native execution and semantic observations for the complete phase8-v1 corpus
provides:
  - strict bounded C++ decoding and pinned-upstream execution for all nineteen rigid witness families
  - closed phase8-v1 comparison across joint, rope, lifecycle, reconstruction, and diagnostic evidence
  - exact debug, release, replay, determinism, and fail-fast sanitizer command evidence
affects: [rigid-sign-off, compatibility-matrix, phase-8-audit]
tech-stack:
  added: []
  patterns: [defined-state semantic adapter, closed field registry, process-isolated sanitizer evidence]
key-files:
  created:
    - tools/reference/src/rigid_world_phase8_decode.hpp
    - tools/reference/src/rigid_world_phase8_execute.hpp
    - crates/liquidfun-differential/tests/phase8_comparator.rs
  modified:
    - tools/reference/src/rigid_world.cpp
    - tools/reference/tests/protocol_tests.cpp
    - crates/liquidfun-differential/src/rigid_evidence.rs
    - crates/liquidfun-differential/src/rigid_evidence/phase8.rs
    - tools/xtask/src/differential.rs
key-decisions:
  - "The C++ oracle returns defined positive zero for reaction fields before solver initialization because several pinned joint getters otherwise read uninitialized solver scratch; initialized reactions remain exact upstream getter bits."
  - "Phase 8 comparison inherits the complete Phase 7 comparator before applying its closed local observation registry, so retained families cannot disappear silently."
  - "Rigid-world contributor commands now bind the accepted request to phase8-v1 and preserve the closed compare, replay, two-run D0, and sanitizer shapes."
patterns-established:
  - "Defined-state oracle projection: semantic evidence never reads uninitialized upstream scratch, while initialized-state bits remain unmodified."
  - "Phase-local comparator layering: inherited evidence is checked first, followed by exhaustive closed local structural and numeric paths."
requirements-completed: [RIGD-11, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T03:00:05Z
duration: 29min
completed: 2026-07-14
---

# Phase 8 Plan 13: C++ Oracle and Differential Evidence Summary

**Rust and the isolated pinned C++ oracle now execute and compare the complete nineteen-family phase8-v1 corpus under a closed semantic policy, including fail-fast sanitizer evidence.**

## Performance

- **Duration:** 29 min
- **Started:** 2026-07-14T02:31:00Z
- **Completed:** 2026-07-14T03:00:05Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- Added duplicate-aware bounded decoding for the ten Phase 8 timelines, including all joint and rope declarations, closed action kinds, exact finite bit fields, identity uniqueness, and N/N+1 limits.
- Executed all eleven pinned upstream joint definitions, all closed mutations, gear dependencies, standalone rope evolution, deterministic teardown, reconstruction, and diagnostic snapshots in fresh request-scoped worlds.
- Prevented undefined reaction evidence before solver initialization while preserving exact pinned getter bits after a real solver step, with a focused C++ regression.
- Added exhaustive Phase 8 comparison for ordered observations, joint configuration and reactions, rope vertices, lifecycle identity, reconstruction dependencies, and diagnostic counts and quality.
- Proved signed zero remains distinct and that structural, numeric, missing-policy, and wildcard-policy mutations fail at stable first-divergence paths.
- Opened the former Plan 08-12 C++ supervisor gate and switched all rigid-world evidence commands to phase8-v1.
- Matched all nineteen required families in debug, optimized release, replay, two-run D0, and ASan/UBSan executions.

## Task Commits

Each task was committed atomically:

1. **Task 08-13-01: Implement strict C++ Phase 8 execution and semantic traces** - `2460859` (feat)
1. **Task 08-13-02: Close comparator coverage and evidence command shapes** - `de9204a` (feat)

## Files Created/Modified

- `tools/reference/src/rigid_world_phase8_decode.hpp` - Validates the closed Phase 8 family registry, declarations, action kinds, identities, bounds, and finite bit fields.
- `tools/reference/src/rigid_world_phase8_execute.hpp` - Executes pinned joints and rope, emits semantic observations, and enforces complete teardown.
- `tools/reference/src/rigid_world.cpp` - Dispatches Phase 8 timelines and exposes the defined-state reaction regression seam.
- `tools/reference/tests/protocol_tests.cpp` - Covers all families, joint and rope observations, malformed inputs, N+1 bounds, reset, and reaction initialization.
- `crates/liquidfun-differential/src/rigid_evidence/phase8.rs` - Compares every closed Phase 8 structural and numeric observation path.
- `crates/liquidfun-differential/tests/phase8_comparator.rs` - Pins mutation detection, wildcard rejection, and distinct signed-zero semantics.
- `crates/liquidfun-differential/src/supervisor/rigid_world.rs` - Removes the completed C++ backend gate.
- `tools/xtask/src/differential.rs` - Binds rigid compare, replay, minimization, and D0 evidence to phase8-v1.
- `tools/xtask/tests/differential_cli.rs` - Verifies closed command shapes and resolves real test binaries from the configured Cargo target directory.

## Decisions Made

- Treated uninitialized upstream joint solver scratch as an adapter safety boundary, not a parity value. Pre-solver reactions are defined positive zero; after stepping, the regression proves bit-exact passthrough from the pinned getter.
- Kept distinct signed-zero comparison in the Phase 8 policy and added a regression that reports a positive-zero versus negative-zero reaction mismatch.
- Reused Phase 7 comparison before Phase 8 local comparison so all retained rigid-body evidence remains mandatory.
- Kept C++ implementation in private reference headers and out of Rust strings and published crates.

## Deviations from Plan

### Automatically Fixed Issues

**1. The Plan 08-12 supervisor gate and its integration assertions remained active**

- **Found during:** First real Phase 8 supervisor execution
- **Issue:** The prior plan intentionally rejected Phase 8 before spawning C++ and its tests pinned that temporary behavior.
- **Fix:** Removed the completed gate and converted the assertions into real nineteen-family execution checks.
- **Files modified:** Supervisor and retained rigid-world integration tests.
- **Verification:** Phase 8 native tests and all 45 rigid-world integration tests pass.

**2. Real-binary xtask tests ignored the required isolated Cargo target directory**

- **Found during:** Focused xtask CLI verification
- **Issue:** Two tests launched `target/debug` directly and could exercise an older protocol codec when `CARGO_TARGET_DIR` was set.
- **Fix:** Resolved real differential and fake-oracle binaries from `CARGO_TARGET_DIR`, falling back to the repository target only when unset.
- **Files modified:** `tools/xtask/tests/differential_cli.rs`.
- **Verification:** All 27 xtask CLI tests pass under `/tmp/liquidfun-rs-target`.

**3. Pinned joint reaction getters can read uninitialized solver scratch before the first step**

- **Found during:** Exact signed-zero comparison and sanitizer audit
- **Issue:** Several upstream joint constructors initialize impulses but not every scratch vector used by `GetReactionForce`; observing immediately after creation is undefined and produced unstable zero signs.
- **Fix:** Added a semantic defined-state guard that returns positive zero before solver initialization and exact upstream getter bits afterward.
- **Files modified:** Phase 8 C++ executor, adapter seam, and protocol tests.
- **Verification:** Focused pre/post-step regression, debug CTest, sanitizer CTest, and exact comparison all pass; distinct signed-zero policy remains enforced.

## Issues Encountered

- Local CMake 3.27.9 and Apple Clang 21 differ from the canonical CMake 4.3.3 and Clang 22.1.8 pins, so successful local cross-engine results remain D2-supported evidence rather than D1 canonical promotion.
- A deliberately invalid D0 invocation containing `--session-profile` failed closed; the registered `verify-determinism --runs 2` shape then passed.

## Verification

- Debug C++ protocol CTest - 1 passed.
- ASan/UBSan fail-fast C++ protocol CTest - 1 passed with no sanitizer finding.
- `cargo test -p liquidfun-differential --test phase8_comparator --all-features` - 4 passed.
- `cargo test -p liquidfun-differential --test rigid_world_phase8 --all-features` - 6 passed.
- `cargo test -p liquidfun-differential --test rigid_world --all-features` - 45 passed.
- `cargo test -p xtask --test differential_cli --all-features` - 27 passed.
- Debug compare and replay - all 19 required families matched under phase8-v1.
- Debug D0 - 2 byte-identical native and oracle runs.
- Release compare - all 19 required families matched under phase8-v1.
- ASan/UBSan compare - all 19 required families matched under phase8-v1.
- Ordered Rust gate: format, Clippy with denied warnings, all-target build, and all-feature tests - passed before each task commit.
- `git diff --check` - passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 08-14 can audit rigid-world API, compatibility, safety, and evidence claims against a complete cross-engine Phase 8 execution surface.
- Canonical CI still owns D1 promotion on the pinned CMake and Clang toolchain.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*
