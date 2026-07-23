---
phase: 12-performance-portability-and-release-hardening
plan: "19"
subsystem: performance-benchmarking
tags: [cpp, jsonl, benchmarks, steady-clock, oracle, differential]
requires:
  - phase: 12-07
    provides: strict bounded benchmark request/result wire and paired identity contract
  - phase: 12-06
    provides: sealed catalog workloads and native measured-region semantics
provides:
  - long-lived C++ oracle dispatch for strict benchmark requests
  - request-local setup/action/checkpoint execution with narrow authoritative timing
  - monotonic reset epochs across rejected and completed benchmark attempts
  - optional non-authoritative common-parent diagnostic samples
affects: [paired-benchmark-runner, performance-reporting, release-evidence]
tech-stack:
  added: []
  patterns:
    - request-local catalog sessions with setup separated from logical actions
    - unprofiled authoritative samples followed by optional diagnostic samples
key-files:
  created:
    - tools/reference/src/benchmark_run.cpp
    - tools/reference/src/benchmark_run.hpp
  modified:
    - tools/reference/src/protocol.cpp
    - tools/reference/src/protocol.hpp
    - tools/reference/src/catalog_run_session.cpp
    - tools/reference/src/catalog_run_session.hpp
    - tools/reference/src/main.cpp
    - tools/reference/tests/protocol_tests.cpp
    - tools/reference/CMakeLists.txt
    - tools/reference/adapter-inputs.txt
key-decisions:
  - "Reuse one narrow catalog session seam for ordinary catalog execution and benchmark timing instead of duplicating native action semantics."
  - "Advance the benchmark reset epoch before strict decoding so every adapter-level rejection and completion consumes one request-local identity."
  - "Collect optional common-parent diagnostics in a separate post-authority sample so profiled durations never become regression authority."
patterns-established:
  - "C++ measured boundary: decode, semantic authority, warmup, and setup precede steady_clock; checkpoint capture, validation, teardown, diagnostics, and encoding follow it."
  - "Benchmark recovery: rejected records advance reset identity while all native world state remains stack- or request-owned."
requirements-completed: [PERF-02, PERF-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T23:53:16Z
duration: 15m
completed: 2026-07-23
---

# Phase 12 Plan 19: C++ Oracle Benchmark Endpoint Summary

**The existing long-lived C++ oracle now executes strict sealed benchmark requests with request-local worlds, narrow unprofiled timing, semantic checkpoint validation, and recoverable reset epochs.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-23T23:38:46Z
- **Completed:** 2026-07-23T23:53:16Z
- **Tasks:** 1
- **Files modified:** 10

## Accomplishments

- Added strict `benchmark_run_request` decoding and `benchmark_run_result` encoding to the existing JSONL oracle process without a second executable or ABI.
- Separated catalog setup, logical actions, checkpoint capture, and teardown so `steady_clock` covers only declared logical actions.
- Validated exact resolved bytes, policy identity, settings, bounds, schedule, semantic entity identity, and final checkpoint before accepting a duration.
- Advanced monotonic reset epochs across valid, malformed, unknown-field, oversized, horizon-mismatched, and hash-mismatched adapter requests without leaking world state.
- Added optional common-parent diagnostic samples after the authoritative unprofiled sample while keeping profiled totals out of the wire contract.

## TDD Evidence

- **RED:** Focused native tests initially failed because `benchmark_run.hpp` and the benchmark adapter did not exist.
- **GREEN:** Strict decoding, shared request-local catalog execution, process dispatch, result encoding, and reset recovery made the focused native suite pass.
- **REFACTOR:** Extracted the existing catalog execution lifecycle behind a PIMPL session seam so catalog runs and benchmarks share identical action semantics.
- The plan prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Execute bounded request-local C++ benchmark actions** - `a8951e9` (feat)

## Files Created/Modified

- `tools/reference/src/benchmark_run.cpp` - Validates resolved benchmark payloads, runs untimed authority/warmup, measures declared actions, validates checkpoints, and encodes results.
- `tools/reference/src/benchmark_run.hpp` - Declares the benchmark adapter, trace, and test observer events.
- `tools/reference/src/protocol.cpp` and `protocol.hpp` - Add strict benchmark kind, identity, settings, byte, policy, and bound decoding.
- `tools/reference/src/catalog_run_session.cpp` and `catalog_run_session.hpp` - Expose the shared request-local setup/action/checkpoint seam while preserving catalog output.
- `tools/reference/src/main.cpp` - Dispatches benchmark records through the existing exception-contained JSONL loop.
- `tools/reference/tests/protocol_tests.cpp` - Covers valid results, timing boundaries, diagnostics, rejection categories, reset advancement, and recovery.
- `tools/reference/CMakeLists.txt` and `adapter-inputs.txt` - Compile the endpoint and bind all behavior-affecting inputs into adapter provenance.

## Decisions Made

- Reused catalog action semantics through a narrow PIMPL session rather than copying the physics executor into the benchmark adapter.
- Captured an untimed semantic authority, completed the reviewed warmup, then constructed a fresh measured session before starting the authoritative clock.
- Emitted common-parent diagnostics only when the workload maps directly to a comparable parent phase; unsupported aggregate workloads retain `null` diagnostics.
- Kept malformed-request diagnostics on stderr and all benchmark protocol records on stdout through the existing process boundary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Wired the endpoint into the real process and provenance manifest**

- **Found during:** Task 1 (Execute bounded request-local C++ benchmark actions)
- **Issue:** The planned file list omitted `main.cpp`, the sole long-lived JSONL dispatch loop, and `adapter-inputs.txt`, the behavior-affecting provenance manifest.
- **Fix:** Added one closed benchmark dispatch branch and registered the new adapter sources in the provenance digest.
- **Files modified:** `tools/reference/src/main.cpp`, `tools/reference/adapter-inputs.txt`
- **Verification:** The real `liquidfun-reference` target compiles, focused protocol CTest passes, and xtask configuration accepts the regenerated adapter digest.
- **Committed in:** `a8951e9`

**2. [Rule 2 - Missing Critical] Exposed a shared setup/action/checkpoint lifecycle seam**

- **Found during:** Task 1 (Execute bounded request-local C++ benchmark actions)
- **Issue:** The only exported catalog function enclosed construction, all actions, checkpoint encoding, and teardown, making the required timing boundary impossible without duplicating the physics executor.
- **Fix:** Added a request-local PIMPL session that performs setup during construction, executes one logical action at a time, and captures checkpoints only on demand; the existing catalog adapter now uses the same seam.
- **Files modified:** `tools/reference/src/catalog_run_session.cpp`, `tools/reference/src/catalog_run_session.hpp`
- **Verification:** Existing catalog regression coverage and all new benchmark timing-boundary tests pass in the focused protocol CTest.
- **Committed in:** `a8951e9`

**Total deviations:** 2 auto-fixed (2 missing critical)

**Impact on plan:** Both expansions were required to satisfy the existing-process and narrow-timing truths without duplicating native execution logic. No published Rust API, dependency, or production physics behavior changed.

## Verification

- `cargo xtask upstream configure --preset oracle-debug` - passed through the documented configuration path.
- `cmake --build target/reference/oracle-debug --target liquidfun-reference-protocol-tests liquidfun-reference` - passed with repository-owned C++ warnings denied.
- `ctest --test-dir target/reference/oracle-debug --output-on-failure -R '^liquidfun-reference-protocol$'` - 1/1 passed.
- Ordered Rust pre-commit gate passed with `CARGO_TARGET_DIR=/tmp/liquidfun-phase12.OJRc0w` and `CARGO_BUILD_JOBS=1`: format, warning-denied Clippy, all-target build, and all-feature tests.
- `git diff --check` and staged-diff review found no whitespace errors or unintended task files.

## Known Stubs

None.

## Issues Encountered

- The existing oracle-debug build correctly rejected the changed adapter digest until reconfigured through `cargo xtask upstream configure --preset oracle-debug`.
- Local CMake 4.3.0 and Apple Clang 21 differ from canonical CMake 4.3.3 and Clang 22.1.8; xtask reported the expected non-blocking portability warnings.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The paired benchmark runner can now send identical strict resolved bytes to the native Rust and pinned C++ executors through one long-lived process.
- Reset identity, unprofiled timing authority, semantic checkpoint binding, and optional common-parent diagnostics are ready for fixed-sample statistical reporting.
- No blockers remain.

## Self-Check: PASSED

- All 10 implementation, integration, build, and provenance files exist.
- Task commit `a8951e9` exists in repository history.
- Summary file exists at the required Phase 12 path.

***

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
