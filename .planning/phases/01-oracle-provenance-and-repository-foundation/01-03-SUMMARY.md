---
phase: 01-oracle-provenance-and-repository-foundation
plan: "03"
subsystem: build-tooling
tags: [cmake, ninja, clang, git-submodule, xtask]

requires:
  - phase: 01-oracle-provenance-and-repository-foundation/01
    provides: Immutable upstream gitlink, identity lock, and read-only oracle policy
  - phase: 01-oracle-provenance-and-repository-foundation/02
    provides: Cargo-first workspace and dependency-free private xtask dispatcher
provides:
  - External CMake 3.25+ wrapper for the pinned legacy LiquidFun build
  - Allowlisted Ninja presets for debug, release, and Clang sanitizer configurations
  - Structured upstream verify, configure, and build commands with typed failures
  - Command-level provenance and process-failure integration coverage
affects: [01-04, 01-05, oracle, provenance, ci, reference-builds]

tech-stack:
  added: [CMake wrapper, Ninja configure and build presets]
  patterns: [external legacy-policy adaptation, structured subprocess arguments, injected executable test seams]

key-files:
  created:
    - tools/reference/CMakeLists.txt
    - tools/reference/CMakePresets.json
    - tools/xtask/tests/upstream_cli.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  modified:
    - tools/xtask/src/main.rs
    - tools/xtask/src/upstream.rs

key-decisions:
  - "Validate lock, gitlink, checkout, origin URL, and clean state before every configure or build operation."
  - "Keep all legacy CMake policy adaptation and build outputs outside the read-only upstream tree."
  - "Inject executable paths through explicit test environment variables while retaining structured Command arguments in production."

patterns-established:
  - "Oracle gate: configure and build cannot run until all pinned provenance identities agree."
  - "Preset gate: contributor input selects only oracle-debug, oracle-release, or oracle-asan-ubsan."

requirements-completed:
  - FND-02
  - FND-03
  - FND-08
  - TEST-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T03:15:04Z

duration: 13 min
completed: 2026-07-10
---

# Phase 1 Plan 03: Reproducible Oracle Build Path Summary

**External modern CMake wrapper with provenance-gated xtask commands, three fixed Ninja presets, and isolated command-failure coverage**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-10T03:01:42Z
- **Completed:** 2026-07-10T03:15:04Z
- **Tasks:** 3
- **Implementation files:** 6

## Accomplishments

- Added a CMake 3.25+ wrapper that supplies the legacy 3.5 policy floor before
  loading the untouched upstream project, disables examples and unit tests for
  ordinary oracle builds, and writes only below `target/reference/`.
- Added exact `oracle-debug`, `oracle-release`, and `oracle-asan-ubsan` Ninja
  configure/build presets with assertions, ordinary optimization, or Clang
  ASan/UBSan respectively, without fast-math or native-only CPU tuning.
- Implemented provenance-gated `cargo xtask upstream verify`, `configure`, and
  `build` commands with full-SHA, gitlink, checkout, cleanliness, URL, tool,
  preset, and subprocess validation.
- Added seven independent integration cases using temporary repository fixtures
  and injected compiled fake tools, so failure paths require no C++ build.

## Task Commits

1. **Task 1: Add the external CMake wrapper and presets** - `ac3bffb`
   (`chore`)
2. **Task 2: Implement upstream verify/configure/build commands** - `4647ecb`
   (`feat`)
3. **Task 3: Add upstream command integration coverage** - `627e517`
   (`test`)

## Files Created/Modified

- `tools/reference/CMakeLists.txt` - Owns modern policy adaptation and disables
  unneeded upstream targets without modifying the submodule.
- `tools/reference/CMakePresets.json` - Defines the three allowed Ninja
  configure/build identities and out-of-tree binary directories.
- `tools/xtask/src/upstream.rs` - Parses and validates the lock, submodule, tool
  identities, preset input, and structured Git/CMake subprocess results.
- `tools/xtask/src/main.rs` - Carries typed upstream failures through the
  existing dispatcher.
- `tools/xtask/tests/upstream_cli.rs` - Covers success and six independent
  negative command conditions with explicit Arrange/Act/Assert sections.
- `tools/xtask/tests/fixtures/fake_upstream_tool.rs` - Supplies deterministic
  Git, CMake, Ninja, and compiler process behavior to integration tests.

## Decisions Made

- Re-verify immutable upstream identity before configure and build instead of
  trusting an earlier standalone verification result.
- Represent command input as an exact preset allowlist and every process
  argument separately; no shell interpreter or command string is used.
- Treat CMake 3.25 and Ninja 1.11 as local floors, record canonical version
  differences as warnings, and fail versions below supported floors.
- Compile the standard-library fake process fixture during the integration test
  run, preserving the dependency-free xtask and avoiding platform-specific
  shell test shims.

## Verification Evidence

- The required Rust sequence passed in order: `cargo fmt --all`, `cargo clippy
  --all-targets --all-features -- -D warnings`, `cargo build --all-targets
  --all-features`, and `cargo test --all-features`.
- `cargo clippy -p xtask --all-targets --all-features -- -D warnings` and
  `cargo test -p xtask` passed. The latter ran five dispatcher unit tests and
  seven upstream command integration tests.
- `cmake --list-presets -S tools/reference` listed exactly `oracle-debug`,
  `oracle-release`, and `oracle-asan-ubsan`.
- Real `cargo xtask upstream verify`, `configure --preset oracle-debug`, and
  `build --preset oracle-debug` commands passed. The first build compiled 54
  C++ objects and linked
  `target/reference/oracle-debug/upstream/Box2D/libliquidfun.a`.
- The local smoke recorded CMake 3.27.9, Ninja 1.13.2, and Apple Clang 21.0.0 on
  macOS ARM64. The command correctly warned that CMake and compiler versions
  differ from canonical CMake 4.3.3 and Clang 22.1.8.
- Preset, cache, and compile-command scans found no `-ffast-math` or
  `-march=native`; the upstream worktree remained clean.
- Cargo metadata still selects only `liquidfun` by default, and `cargo package
  -p liquidfun --list --allow-dirty` contains no tooling, reference, upstream,
  CMake, C, or C++ content.
- `git diff --check` passed, and final status contained only the three protected
  orchestrator-owned planning-file modifications.

## Platform Limitations

- The canonical Linux CMake 4.3.3 and Clang 22.1.8 pins were not installed on
  this macOS ARM64 host. The supported-floor configuration and real Apple Clang
  build passed; canonical Linux identity remains a CI/oracle-lane responsibility.
- The upstream CMake 2.8 declaration emits a deprecation warning under local
  CMake 3.27.9. The external `CMAKE_POLICY_VERSION_MINIMUM=3.5` adaptation is
  committed for the CMake 4 canonical path without altering upstream source.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- CMake build-preset discovery does not accept configure-mode `-S` syntax with
  `cmake --build`. Running the build from `tools/reference`, as the plan
  required for xtask, resolved the initial invocation error and established the
  final cross-platform `current_dir` behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for `01-04-PLAN.md` to consume the verified immutable oracle through
  the existing inventory seam.
- Ready for `01-05-PLAN.md` to expose these commands through thin contributor
  aliases and split Cargo-only from oracle CI lanes.
- No oracle provenance, preset-input, submodule-mutation, or consumer-package
  blocker remains for later Phase 1 work.

## Self-Check: PASSED

- All six implementation/test files and the summary exist.
- Three atomic `01-03` task commits are present in git history.
- Summary lifecycle metadata matches Plan 01-03, and all four requirement IDs
  are copied verbatim.
- Real oracle-debug configure/build evidence exists under `target/reference/`,
  the upstream worktree is clean, and protected planning files remain
  unstaged and uncommitted.

***

_Phase: 01-oracle-provenance-and-repository-foundation_
_Completed: 2026-07-10_
