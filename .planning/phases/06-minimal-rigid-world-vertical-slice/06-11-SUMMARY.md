---
phase: 06-minimal-rigid-world-vertical-slice
plan: "11"
subsystem: cpp-oracle-build-identity
tags: [cpp, cmake, provenance, compile-database, rigid-world, sha256]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "10"
    provides: Pinned rigid-world translation unit and its execution, decode, and trace headers
  - phase: 05-shapes-and-collision-foundation
    plan: "07"
    provides: Three-unit compile-database identity and checked-in adapter source manifest
provides:
  - Adapter content identity covering rigid_world.cpp, rigid_world.hpp, rigid_world_decode.hpp, and rigid_world_trace.hpp
  - Exact four-unit compile-database identity including rigid_world.cpp
  - Fail-closed rejection of missing, duplicate, or flag-divergent result translation units
  - Source and included-header mutation evidence for rigid adapter content identity
affects: [06-12-rigid-evidence, oracle-provenance, differential-supervision]
tech-stack:
  added: []
  patterns: [exact result-unit registry, per-unit compile-signature parity, independently reproduced source digest]
key-files:
  created: []
  modified:
    - tools/reference/adapter-inputs.txt
    - tools/reference/CMakeLists.txt
    - tools/reference/src/generate_build_identity.cmake.in
    - tools/xtask/src/differential.rs
    - tools/xtask/src/upstream.rs
key-decisions:
  - "Treat collision_probe.cpp, math_probe.cpp, protocol_bits.cpp, and rigid_world.cpp as one exact compile-identity set and reject per-unit flag drift before hashing."
  - "Include every rigid implementation and included header, especially rigid_world_decode.hpp, in the independently reproduced adapter content digest."
patterns-established:
  - "Result-unit identity: require each reviewed translation unit exactly once, normalize only repository/build locations and the unit filename, then require one shared effective compile signature."
requirements-completed: [RIGD-01, RIGD-02, RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T06:13:08Z
duration: 25 min
completed: 2026-07-12
---

# Phase 6 Plan 11: Rigid Oracle Build Identity Summary

**The rigid C++ oracle now has content and compile-command provenance covering its implementation plus all three included headers, with exact four-unit scalar flag parity enforced independently by CMake and xtask.**

## Performance

- **Duration:** 25 min
- **Completed:** 2026-07-12T06:13:08Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Added `rigid_world.cpp`, `rigid_world.hpp`, `rigid_world_decode.hpp`, and `rigid_world_trace.hpp` to the adapter content manifest used independently by xtask and CMake.
- Extended generated build identity from three to exactly four reviewed result translation units, including `rigid_world.cpp` under the existing strict warning and floating-point options.
- Made both CMake and Rust identity calculation reject missing units, duplicate units, and per-unit effective compile-command divergence before accepting a digest.
- Added tests proving actual adapter content identity changes when either the rigid implementation or included decoder header changes.
- Exercised the generated CMake script against missing, duplicate, and divergent real compile-database mutations, then restored and accepted the baseline.

## Task Commits

1. **Task 1: Bind rigid adapter sources into reproducible build identity** - `c9f5008` (`feat`)

## Files Created/Modified

- `tools/reference/adapter-inputs.txt` - Registers all result-affecting rigid sources and included headers.
- `tools/reference/CMakeLists.txt` - Reports the complete four-unit feature set and rebuilds identity when the rigid translation unit changes.
- `tools/reference/src/generate_build_identity.cmake.in` - Enforces the exact result-unit set and one shared effective compile signature.
- `tools/xtask/src/differential.rs` - Independently mirrors four-unit compile identity and tests missing, duplicate, and divergent commands.
- `tools/xtask/src/upstream.rs` - Proves implementation and included-header mutations change the actual adapter digest.

## Decisions Made

- Compile identity is accepted only when all four result translation units occur exactly once and normalize to the same effective compile command apart from their unit filenames.
- Adapter identity covers included implementation headers directly rather than relying on the translation unit alone to imply header provenance.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added tests at the authoritative Rust identity seams omitted from the plan file list**

- **Found during:** Task 1 (Bind rigid adapter sources into reproducible build identity)
- **Issue:** The plan required compile-database negative tests and source/header mutation tests, but its declared modified-file list contained only CMake and C++ protocol-test paths. The independently used identity algorithms live in `tools/xtask/src/differential.rs` and `tools/xtask/src/upstream.rs`.
- **Fix:** Updated those two existing identity seams directly and added focused Arrange/Act/Assert tests without creating a duplicate digest implementation.
- **Files modified:** `tools/xtask/src/differential.rs`, `tools/xtask/src/upstream.rs`
- **Verification:** Full xtask tests, named rigid identity tests, actual CMake compile-database mutations, and all repository Rust gates passed.
- **Committed in:** `c9f5008`

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The change closes the requested trust boundary at both independent identity implementations without widening runtime or public APIs.

## Issues Encountered

- An initial strict CTest invocation used the repository root with a preset owned by `tools/reference`; rerunning with the configured build directory passed. No source change was needed.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `cargo test -p xtask rigid_compile_database_identity --all-features`
- `cargo test -p xtask rigid_adapter_identity --all-features`
- `cargo xtask upstream configure --preset oracle-debug`
- `cargo xtask upstream build --preset oracle-debug`
- `cmake --build target/reference/oracle-debug --target liquidfun-reference-protocol-tests`
- `ctest --test-dir target/reference/oracle-debug --output-on-failure`
- Generated CMake identity rejected missing, duplicate, and flag-divergent compile-database mutations and accepted the restored baseline.
- Adapter-input completeness and `git diff --check` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-12 can trust that any change to rigid execution, declarations, decoding, trace encoding, or effective scalar compilation changes the child build identity.
- Local oracle evidence used Apple Clang 21.0.0 and CMake 3.27.9, so canonical Clang 22.1.8 / CMake 4.3.3 authority remains a CI responsibility.

## Self-Check: PASSED

- Task commit `c9f5008` exists.
- All five modified implementation/test files exist and are contained in the task commit.
- `rigid_world_decode.hpp` and every other rigid result input occur in `tools/reference/adapter-inputs.txt`.
- `.planning/config.json`, `.planning/STATE.md`, and `.planning/ROADMAP.md` remained outside the task commit and summary changes.
- No stubs or new security-relevant runtime surfaces were introduced.

***

_Phase: 06-minimal-rigid-world-vertical-slice_
_Completed: 2026-07-12_
