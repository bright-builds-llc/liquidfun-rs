---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "09"
subsystem: cpp-oracle-build-provenance
tags: [rust, cmake, cpp, sha256, build-identity, asan, ubsan]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Strict C++ empty-world oracle executable and registered native targets from Plan 02-08
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Immutable upstream lock, clean-submodule verification, and allowlisted xtask orchestration
provides:
  - Configured immutable C++ oracle identity bound to independently checked revision and adapter-source digest
  - Allowlisted xtask configure arguments and liquidfun-reference-only build targeting
  - Out-of-tree debug, release, and fail-fast ASan/UBSan oracle builds
affects: [02-10, process-supervisor, oracle-handshake, sanitizer-lanes, differential-evidence]

tech-stack:
  added: []
  patterns: [path-bound source digests, configured immutable provenance headers, structured CMake arguments, fail-fast sanitizer presets]

key-files:
  created:
    - tools/reference/src/build_identity.hpp.in
  modified:
    - tools/reference/CMakeLists.txt
    - tools/reference/CMakePresets.json
    - tools/reference/src/main.cpp
    - tools/xtask/src/upstream.rs
    - tools/xtask/tests/upstream_cli.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs

key-decisions:
  - "Hash the fixed repository-relative adapter source list as path=content-sha256 records so xtask and CMake independently derive the same identity without accepting caller paths."
  - "Require the verified lock revision and adapter digest as separate structured CMake cache arguments, then configure all startup identity fields into an out-of-tree generated header."
  - "Keep sanitizer findings fail-fast while narrowly demoting two modern Clang warnings emitted by the untouched 2014 upstream sources under their own warning-denial policy."

patterns-established:
  - "Identity binding: xtask verifies the lock/submodule, computes the adapter digest, and passes both expected identities to CMake, which recomputes the digest before generating the child handshake constants."
  - "Target confinement: configure accepts only three named presets and build always names liquidfun-reference, leaving Box2D transitive."

requirements-completed:
  - COMP-04
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T09:26:37Z

duration: 12 min
completed: 2026-07-10
---

# Phase 2 Plan 09: Immutable Oracle Build Identity and Sanitizer Controls Summary

**Every C++ oracle child now starts with configured revision, adapter, compiler, target, flag, preset, and sanitizer provenance, while xtask admits only reviewed presets and the registered executable target.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-07-10T09:14:45Z
- **Completed:** 2026-07-10T09:26:37Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Added an immutable configured build-identity header containing the pinned revision, adapter revision/content SHA-256, CMake preset, complete compiler identity/version, compiler target plus system/architecture, build type, effective compile/link flags, and sanitizer mode.
- Made xtask compute the same path-bound adapter source digest, pass revision/digest as separate `-D` arguments, reject unreviewed presets or extra path input, and build only `liquidfun-reference`.
- Preserved all generated headers and binaries below `target/reference/<preset>` and kept the upstream submodule clean.
- Made ASan/UBSan builds non-recovering and configured `ASAN_OPTIONS=abort_on_error=1:halt_on_error=1` plus `UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1`.
- Added focused command tests for exact configure/build arguments and fail-closed input validation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Bind immutable identity and allowlisted reference builds** - `bc420a4` (`feat`)

## Files Created/Modified

- `tools/reference/src/build_identity.hpp.in` - Configured immutable startup identity constants.
- `tools/reference/CMakeLists.txt` - Independently recomputes the adapter digest, validates expected identities, configures the generated header, and records effective build fields.
- `tools/reference/CMakePresets.json` - Defines debug/release outputs and fail-fast ASan/UBSan flags and runtime options.
- `tools/reference/src/main.cpp` - Constructs the handshake identity only from the generated immutable header.
- `tools/xtask/src/upstream.rs` - Computes adapter identity, passes structured configure values, validates preset-only input, and builds the registered executable target.
- `tools/xtask/tests/upstream_cli.rs` - Proves exact arguments, target confinement, and preset/path rejection.
- `tools/xtask/tests/fixtures/fake_upstream_tool.rs` - Captures fake CMake arguments for command-level assertions.

## Decisions Made

- Included repository-relative source paths in the adapter digest input, preventing same-content path substitution from producing an indistinguishable identity.
- Kept the fixed adapter-source allowlist in both xtask and CMake so neither boundary accepts contributor-provided filesystem paths.
- Used cache variables rather than `*_FLAGS_INIT` for sanitizer flags so rerunning a reviewed preset converges existing build trees as well as clean CI trees.
- Kept modern-Clang compatibility flags scoped to sanitized upstream `Box2D` compilation; repository-authored targets retain `-Werror` or `/WX`.

## Verification Evidence

- TDD RED: `cargo test -p xtask --test upstream_cli` passed 8 existing tests and failed only the new configure-identity and executable-target assertions before implementation.
- TDD GREEN: `cargo test -p xtask --test upstream_cli` passes all 10 command tests.
- `cargo xtask upstream configure/build` passes for `oracle-debug`, `oracle-release`, and `oracle-asan-ubsan`; every binary is under its matching `target/reference/<preset>` directory.
- Debug native CTest passes the registered `liquidfun-reference-protocol` test.
- Sanitizer native CTest passes with the configured fail-fast ASan and UBSan environment.
- Debug and sanitizer child handshakes report the configured preset, full AppleClang version, `native|Darwin|arm64` target identity, effective flags, adapter SHA-256, and correct sanitizer mode.
- `cargo xtask package verify` and `cargo xtask upstream verify` pass.
- The required ordered repository gate passed before the task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Acceptance scans find `liquidfun-reference`, C++17, generated identity, SHA-256, `-fno-sanitize-recover=undefined`, `halt_on_error=1`, and `abort_on_error=1` controls.
- `git -C third_party/liquidfun status --short` is empty; all `build`-named paths reported by the broad acceptance `find` are tracked upstream source/documentation paths, and no `build_identity.hpp` exists inside the submodule.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wire the generated identity into the existing child entrypoint**

- **Found during:** Task 1 (Bind immutable identity and allowlisted reference builds)
- **Issue:** The plan's declared file list omitted `main.cpp`, but the existing child constructed identity from a hard-coded revision and compile definitions, so a configured header could not become the startup authority without changing the entrypoint.
- **Fix:** Included the generated header and replaced every hard-coded/definition-based identity field with its configured immutable constant.
- **Files modified:** `tools/reference/src/main.cpp`
- **Verification:** Debug and sanitizer handshakes exactly reflect their generated headers; native builds and CTest pass.
- **Committed in:** `bc420a4`

**2. [Rule 3 - Blocking] Preserve sanitizer coverage with modern AppleClang**

- **Found during:** Task 1 sanitizer build verification
- **Issue:** The untouched 2014 upstream particle source triggers `nodiscard` and non-trivial memory-call warnings under modern AppleClang, and upstream's own `-Werror` converted them into build failures before sanitizer tests could run.
- **Fix:** Scoped `-Wno-error=unused-result` and `-Wno-error=nontrivial-memcall` to Clang sanitizer builds of the upstream `Box2D` target only; the warnings remain visible and repository-owned warning denial is unchanged.
- **Files modified:** `tools/reference/CMakeLists.txt`
- **Verification:** The sanitizer oracle and native test executable build, warnings remain visible, fail-fast CTest passes, and the submodule remains clean.
- **Committed in:** `bc420a4`

**3. [Rule 1 - Bug] Synchronize stale human-readable GSD progress**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` and `roadmap update-plan-progress 02` returned the correct 74% and 9/14 disk-derived results but left the tracked body progress values at 68% and 8/14.
- **Fix:** Updated only the stale human-readable state progress bar and Phase-2 roadmap row to match the successful GSD tool results.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** Nine Phase-2 summaries exist, the roadmap reports 9/14, and both state progress representations report 74%.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 3 auto-fixed (2 blocking integration/build issues, 1 workflow-tool bug)
**Impact on plan:** The implementation fixes are necessary to make the configured identity authoritative and the sanitizer lane executable; the metadata correction keeps tracked progress internally consistent. None widens protocol or consumer scope.

## Issues Encountered

- The first generated-header dependency list used two spellings of the same template path, which Ninja rejected as a duplicate CMake rerun input. Canonicalizing each adapter path with `file(REAL_PATH ...)` fixed the generated graph.
- Existing sanitizer build trees retained old `*_FLAGS_INIT` values. Moving reviewed flags to preset cache variables made repeated configure runs converge without deleting build output.
- TDD RED was observed but not committed because repository policy requires the complete ordered Rust gate to pass before every commit.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-10 to supervise the child, validate the configured startup identity before requests, and classify sanitizer/process failures.
- Debug, release, and sanitizer presets all configure and build through the allowlisted xtask path.
- No identity, target-confinement, sanitizer, package-isolation, or upstream-cleanliness blocker remains.

## Self-Check: PASSED

- The configured header template and all six modified implementation/test files exist.
- Task commit `bc420a4` exists and contains exactly the seven task-owned files.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-09.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
