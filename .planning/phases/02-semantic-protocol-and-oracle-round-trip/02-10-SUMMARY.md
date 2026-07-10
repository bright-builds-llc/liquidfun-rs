---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "10"
subsystem: differential-process-runner
tags: [rust, process-supervision, jsonl, differential-testing, asan, ubsan]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Validated protocol/trace types, comparator/minimizer, permanent empty-world corpus, C++ oracle, and immutable build identity from Plans 02-05, 02-06, and 02-09
provides:
  - Private native Rust empty-world adapter with exact-bit semantic traces and reset epochs
  - Confined one-shot, bounded-reuse, and sanitizer child supervision with concurrent bounded drains
  - Exhaustive injected failure classification with poison, kill, reap, join, and retained evidence
  - Thin compare/replay/minimize CLI vocabulary with deterministic machine results and distinct exit codes
  - Real debug and sanitizer Rust/C++ one-shot and two-request reuse Matches
affects: [02-11, 02-12, differential-evidence, replay, regression-promotion, testing-documentation]

tech-stack:
  added: []
  patterns: [private native adapter seam, explicit child-session enum, allowlisted canonical executable resolution, concurrent first-last stderr retention, deterministic machine outcome reports]

key-files:
  created:
    - crates/liquidfun-differential/src/rust_adapter.rs
    - crates/liquidfun-differential/src/supervisor.rs
    - crates/liquidfun-differential/src/supervisor/executable.rs
    - crates/liquidfun-differential/src/supervisor/failure.rs
    - crates/liquidfun-differential/src/supervisor/stdio.rs
    - crates/liquidfun-differential/src/runner.rs
    - crates/liquidfun-differential/src/main.rs
    - crates/liquidfun-differential/tests/rust_adapter.rs
    - crates/liquidfun-differential/tests/supervisor_failures.rs
    - crates/liquidfun-differential/tests/round_trip.rs
    - crates/liquidfun-differential/tests/fixtures/fake_oracle.rs
  modified:
    - crates/liquidfun-differential/Cargo.toml
    - crates/liquidfun-differential/src/lib.rs

key-decisions:
  - "Keep Phase-2 native execution private and model only gravity, exact f32 timestep accumulation, zero semantic counts, ordered checkpoints, and reset epochs."
  - "Represent child lifecycle as one synchronous enum state machine and make one-shot, finite reuse, and sanitizer runs immutable named profiles of the same supervisor."
  - "Retain bounded first/last stderr windows while draining all child output concurrently, and classify every poison path before kill/wait/join teardown returns evidence."
  - "Resolve only regular non-symlink executables beneath canonical target/reference/<preset> paths and launch them with structured Command arguments and no shell."
  - "Keep machine stdout distinct from child protocol output and assign Match, physics mismatch, and harness failure separate result kinds and exit codes."

patterns-established:
  - "Supervisor ownership: stdout/stderr workers start before handshake; the controlling thread alone owns request sequencing and deadlines."
  - "Failure evidence: request/scenario/session identities, elapsed time, last record, exit, bounded stderr, kill, reap, and limit profile travel together."
  - "Integration gating: ordinary Cargo tests never configure or build C++; real tests run only when the reviewed executable already exists and otherwise emit a labeled prerequisite."

requirements-completed:
  - COMP-04
  - COMP-05
  - COMP-08
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T10:04:08Z

duration: 33 min
completed: 2026-07-10
---

# Phase 2 Plan 10: Native Adapter and Hardened Oracle Round Trip Summary

**The same validated empty-world request now produces provenance-checked native Rust and isolated C++ semantic Matches through one hardened one-shot/reuse/sanitizer supervisor.**

## Performance

- **Duration:** 33 min
- **Started:** 2026-07-10T09:31:09Z
- **Completed:** 2026-07-10T10:04:08Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments

- Added a private native adapter that emits exact ordered checkpoint IDs, zero world counts, exact `0.5`/`1.0` simulation-time bits, scenario/tolerance/build hashes, and independently advancing reset epochs without changing `liquidfun`.
- Added a confined sequential child supervisor with reviewed one-shot/reuse/sanitizer profiles, startup/request deadlines, finite process cycling, concurrent stdout/stderr drains, incremental framing/size checks, and poison/reap behavior.
- Covered startup, request, exit, signal, sanitizer, EOF, partial, malformed, unknown, oversized, trace/total overflow, request/identity/sequence/reset, scenario-rejection, and adapter failure classes through a separately compiled fake child.
- Proved that 1 MiB concurrent stderr cannot deadlock valid stdout and that retained first/last diagnostics stay at 256 KiB with exact total/truncation evidence.
- Added deterministic CLI machine results and concise stderr summaries for named compare, named/exact replay, and allowlisted minimize command dispatch, with distinct match/mismatch/harness exit codes.
- Produced real `oracle-debug` and `oracle-asan-ubsan` Matches for one-shot and two distinguishable reuse requests with C++ and Rust epochs `1/2`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Execute the bounded scenario through a private native Rust adapter** - `e858736` (`feat`)
2. **Task 2: Supervise one-shot and bounded-reuse child sessions safely** - `0daf0c5` (`feat`)
3. **Task 3: Prove compare, replay, and real two-request round trips** - `9171f0b` (`feat`)
4. **Cross-platform regression guard: keep signal injection Unix-scoped** - `5ad9ecb` (`fix`)

## Files Created/Modified

- `crates/liquidfun-differential/src/rust_adapter.rs` - Private native exact-bit empty-world executor and reset proof.
- `crates/liquidfun-differential/src/supervisor.rs` - One controlling-thread session state machine and request lifecycle.
- `crates/liquidfun-differential/src/supervisor/executable.rs` - Preset allowlist, canonical confinement, symlink/file/executable checks.
- `crates/liquidfun-differential/src/supervisor/failure.rs` - Exit/sanitizer classification and bounded evidence construction.
- `crates/liquidfun-differential/src/supervisor/stdio.rs` - Concurrent record-bounded stdout and first/last bounded stderr drains.
- `crates/liquidfun-differential/src/runner.rs` - Named/exact request orchestration over both adapters and the comparator.
- `crates/liquidfun-differential/src/main.rs` - Allowlisted compare/replay/minimize parsing and deterministic result rendering.
- `crates/liquidfun-differential/tests/rust_adapter.rs` - Exact trace, identity/hash, invalid-input, and reset-isolation coverage.
- `crates/liquidfun-differential/tests/supervisor_failures.rs` - Complete D-10 lifecycle/failure/resource/deadlock coverage.
- `crates/liquidfun-differential/tests/round_trip.rs` - Fake CLI classification plus real debug one-shot/reuse and exact replay coverage.
- `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs` - Separately compiled deterministic child behavior injector.
- `crates/liquidfun-differential/Cargo.toml` - Registers the fake oracle binary as private all-target test infrastructure.
- `crates/liquidfun-differential/src/lib.rs` - Exports private adapter, runner, and supervisor APIs.

## Decisions Made

- The Rust adapter records the selected upstream revision only as the comparison-oracle identity; `engine_kind = native_rust`, `cmake_preset = native-rust`, Cargo package revision, and Rust build fields keep its own provenance explicit.
- Reuse remains synchronous with one `&mut self` request at a time; after 100 successful requests the supervisor closes/reaps the child and starts a new epoch-1 generation.
- Any protocol/resource/sanitizer/reset failure poisons the child. The harness never retries the deterministic request and returns only after stdin closure, conditional kill, wait/reap, and drain-thread joins.
- The CLI owns only parsing/rendering. Validated request execution and comparison remain library functions, and child stdout never leaks into machine result stdout.

## Verification Evidence

- TDD RED was observed for each task on the absent adapter, absent supervisor, and absent runner/CLI before implementation.
- `cargo test -p liquidfun-differential --test rust_adapter -- --nocapture` passed 4 focused adapter tests.
- `cargo test -p liquidfun-differential --test supervisor_failures -- --nocapture` passed 7 injected lifecycle/failure tests, including real 5-second/10-second deadlines, 65 MiB output pressure, 100-request cycling, and pipe-deadlock pressure.
- `cargo test -p liquidfun-differential --test round_trip -- --nocapture` passed 4 fake/real CLI and round-trip tests.
- The required ordered default-member sequence passed before every commit and at completion: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Full workspace warning-denied Clippy, all-target/all-feature build, all-feature tests, and warning-denied rustdoc passed; this included 41 protocol unit tests, 8 fixture tests, 32 differential tests, and all xtask suites.
- `cargo xtask upstream verify`, debug configure/build, sanitizer configure/build, and `cargo xtask package verify` passed; the upstream submodule remained clean.
- Real debug commands produced one-shot Match epoch `1`, reuse Matches epochs `1/2`, and replay Match.
- Real fail-fast ASan/UBSan commands produced sanitizer-profile Match epoch `1` and reuse Matches epochs `1/2`.
- Static acceptance scans found structured `Command`, piped stdio, threads, kill/wait/join lifecycle primitives, no shell launch, no public engine model/unsafe/unwrap in the native adapter, and no diff beneath `crates/liquidfun`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Register the fake child as a separately compiled private binary**

- **Found during:** Task 2 RED
- **Issue:** Integration tests cannot receive a `CARGO_BIN_EXE_*` path for an unregistered source fixture.
- **Fix:** Added a private `[[bin]]` target in `crates/liquidfun-differential/Cargo.toml`; ordinary default-member Cargo behavior remains unchanged.
- **Files modified:** `crates/liquidfun-differential/Cargo.toml`
- **Verification:** All-target workspace build/test compiles the fixture, while package verification still publishes only `liquidfun`.
- **Committed in:** `0daf0c5`

**2. [Rule 1 - Bug] Remove a transient placeholder-child leak during session simplification**

- **Found during:** Task 2 simplification pass
- **Issue:** An initial ownership workaround replaced the handshaking child with a short-lived placeholder process, which could leave unjoined workers.
- **Fix:** Made handshake completion consume and return the real typed child state, preserving ownership across both success and poison paths without any placeholder.
- **Files modified:** `crates/liquidfun-differential/src/supervisor.rs`
- **Verification:** Full failure suite passed twice after the change; every poison path reports reaped evidence.
- **Committed in:** `0daf0c5`

**3. [Rule 2 - Missing Critical] Split the supervisor into cohesive modules below the file-size trigger**

- **Found during:** Task 2 simplification pass
- **Issue:** The first complete implementation exceeded the repository's 628-line source-file refactor trigger.
- **Fix:** Kept the 623-line lifecycle entrypoint and extracted canonical executable resolution, failure evidence, and stdio draining into focused child modules.
- **Files modified:** `crates/liquidfun-differential/src/supervisor.rs`, `crates/liquidfun-differential/src/supervisor/executable.rs`, `crates/liquidfun-differential/src/supervisor/failure.rs`, `crates/liquidfun-differential/src/supervisor/stdio.rs`
- **Verification:** Warning-denied Clippy, full workspace tests/docs, and acceptance lifecycle scans passed.
- **Committed in:** `0daf0c5`

**4. [Rule 1 - Bug] Scope signal injection to platforms with signal exit evidence**

- **Found during:** Final cross-platform review
- **Issue:** `ChildSignaled` is observable through Unix `ExitStatusExt`; Windows aborts present as nonzero exits instead.
- **Fix:** Kept nonzero-exit coverage cross-platform and gated the signal-specific injected assertion to Unix.
- **Files modified:** `crates/liquidfun-differential/tests/supervisor_failures.rs`
- **Verification:** Focused supervisor suite and the required Rust gate passed after the correction.
- **Committed in:** `5ad9ecb`

**5. [Rule 1 - Bug] Synchronize stale human-readable GSD progress**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` and `roadmap update-plan-progress 02` returned the correct 79% and 10/14 disk-derived values but left the tracked body progress at 74% and 9/14.
- **Fix:** Updated only the stale human-readable state progress bar and Phase-2 roadmap row to the successful tool results.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** Ten Phase-2 summaries exist; state reports 15/19 and 79%, and the roadmap reports 10/14.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 5 auto-fixed (3 bugs, 1 blocking test-infrastructure gap, 1 missing structural safeguard)
**Impact on plan:** Every deviation tightened lifecycle correctness, portability, or maintainability without widening the physics model, public consumer surface, process allowlist, or artifact-promotion scope.

## Issues Encountered

- TDD RED commits were not created because repository policy requires the complete Rust gate to pass before every commit. Each RED failure was observed first, then the green task outcome was committed atomically.
- Local CMake 3.27.9 and AppleClang 21.0.0 differ from the canonical 4.3.3 and 22.1.8 identities. Xtask reported the expected warnings, recorded the actual build identity, and all debug/sanitizer builds and runs passed.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-11 to stage/replay/promote reviewed trace evidence through this validated execution seam.
- Ready for Plan 02-12 documentation and verification wiring to describe the established command, failure, and local-versus-oracle prerequisites.
- The native adapter deliberately remains empty-world-only; public worlds, handles, storage, callbacks, and nonempty physics remain Phase 3+ work.

## Self-Check: PASSED

- All thirteen implementation/test/config paths listed in this summary exist.
- Commits `e858736`, `0daf0c5`, `9171f0b`, and `5ad9ecb` exist in repository history.
- Summary lifecycle metadata and all four requirement IDs match Plan 02-10.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
