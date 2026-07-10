---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "03"
subsystem: differential-testing-protocol
tags: [rust, jsonl, serde, scenario, trace-state-machine, tolerance]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Invariant-bearing protocol versions, IDs, exact float bits, limits, provenance, and harness failure taxonomy from Plan 02-02
provides:
  - Strict limit-aware UTF-8 JSONL framing and direct typed record decoding
  - Validated named and seeded phase-2 empty-world scenarios with ordered commands and checkpoints
  - Provenance-bound handshake and trace begin/checkpoint/end state-machine validation
  - Typed exact, numeric, and collection tolerance policy vocabulary with deterministic profile identity
affects: [02-04, 02-05, 02-06, 02-10, protocol-fixtures, comparator, process-supervisor]

tech-stack:
  added: []
  patterns: [bounded serde visitors, raw-to-validated domain parsing, consuming trace state machine, deterministic length-prefixed payload hashing]

key-files:
  created:
    - crates/liquidfun-test-protocol/src/codec.rs
    - crates/liquidfun-test-protocol/src/scenario.rs
    - crates/liquidfun-test-protocol/src/scenario/tests.rs
    - crates/liquidfun-test-protocol/src/tolerance.rs
    - crates/liquidfun-test-protocol/src/trace.rs
    - crates/liquidfun-test-protocol/src/trace/tests.rs
  modified:
    - crates/liquidfun-test-protocol/src/lib.rs

key-decisions:
  - "Decode one newline-complete record directly into strict raw structs with deny_unknown_fields and bounded visitors, then construct validated domain values only after cross-field checks pass."
  - "Keep scenario schema 1 deliberately empty-world-only while preserving a permanent ordered command/checkpoint seam for later engine adapters."
  - "Require trace identity on begin, every checkpoint, and end, and accept a trace only after exact count, payload-hash, and reset-epoch proof validation."
  - "Represent numeric thresholds as exact FloatBits inside closed field policies; phase2-v1 compares simulation time by ExactBits and checkpoints as Ordered."

patterns-established:
  - "Strict wire boundary: bytes/newline/depth are checked before direct Serde decoding, while strings and collections reject N + 1 during typed visitation."
  - "Trace authority: provenance and request identity validate before semantic fields, and only a complete begin -> checkpoint* -> end stream can produce ValidatedTrace."

requirements-completed:
  - COMP-03
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T07:24:15Z

duration: 32 min
completed: 2026-07-10
---

# Phase 2 Plan 03: Strict Scenario and Provenance-Bound Trace Protocol Summary

**Bounded JSONL scenario decoding now feeds invariant-bearing empty-world domain values, while only provenance-matched, exactly ordered, hash-complete, reset-proven trace streams can become validated comparison inputs.**

## Performance

- **Duration:** 32 min
- **Started:** 2026-07-10T06:51:40Z
- **Completed:** 2026-07-10T07:24:15Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added strict one-record UTF-8 JSONL framing with distinct blank, invalid UTF-8, partial, trailing, oversized, depth, duplicate, unknown, version, and typed-limit classifications.
- Added named/seeded phase-2 scenario validation for exact float bits, empty entities, ordered unique commands/checkpoints, solver bounds, references, source identity, and aggregate resource limits.
- Added startup handshake validation against the pinned oracle revision and independently recomputed build identity.
- Added a consuming trace validator for exact begin/checkpoint/end sequencing, request and build identity, scenario/tolerance identity, checkpoint order, zero world counts, payload hashes, and adapter reset proof.
- Added exhaustive typed float/discrete/collection policy enums and a deterministic `phase2-v1` profile with exact simulation-time bits.
- Added 21 focused codec, scenario, trace, and tolerance tests, bringing the protocol crate to 37 passing unit tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Parse strict bounded JSONL scenarios into domain types** - `7040662` (`feat`)
2. **Task 2: Validate handshake and streamed trace state** - `976499f` (`feat`)
3. **Task 2 invariant follow-up: Reject invalid trace source identities** - `5110b4c` (`fix`)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/lib.rs` - Registers and exports codec, scenario, tolerance, and trace modules and exposes validated current-version constants.
- `crates/liquidfun-test-protocol/src/codec.rs` - Implements strict JSONL framing, depth checks, typed error classification, bounded string/collection visitors, and newline-complete encoding.
- `crates/liquidfun-test-protocol/src/scenario.rs` - Defines raw strict scenario records and validated source, command, checkpoint, request, and scenario domain types.
- `crates/liquidfun-test-protocol/src/scenario/tests.rs` - Covers accepted scenarios, exact float transport, fail-closed shapes, references, solver counts, and every researched N/N+1 boundary.
- `crates/liquidfun-test-protocol/src/tolerance.rs` - Defines exact/absolute/relative/ULP float policies, exact discrete policy, ordered/set/multiset collection policies, and the hashed phase-2 profile.
- `crates/liquidfun-test-protocol/src/trace.rs` - Defines strict handshake/trace decoding, provenance validation, world-count records, canonical payload hashing, and the trace state machine.
- `crates/liquidfun-test-protocol/src/trace/tests.rs` - Covers handshake ordering/provenance, zero/one/multiple checkpoint traces, every transition class, identity/hash/count/reset failures, and trace-source rejection.

## Decisions Made

- Used the validated version, identifier, float-bit, limit, hash, build-identity, and failure authorities from Plan 02-02 instead of adding parallel primitive representations.
- Kept permanent semantic breadth to ordered steps, checkpoints, `world_counts`, and `simulation_time`; schema-1 entity definitions fail closed instead of anticipating Phase 3 object semantics.
- Hashed scenario JSON deterministically and trace checkpoint payloads as length-prefixed deterministic JSON bytes, keeping transport fidelity separate from comparison policy.
- Kept comparison behavior out of this crate: it declares exhaustive typed policies and validated traces, while Plan 02-04 owns comparison outcomes and diagnostics.

## Verification Evidence

- TDD RED for Task 1 produced the expected unresolved strict-codec/scenario API errors before implementation; Task 2 RED likewise failed on absent trace-state and tolerance APIs.
- `cargo test -p liquidfun-test-protocol scenario -- --nocapture` passed all 8 scenario tests.
- `cargo test -p liquidfun-test-protocol codec -- --nocapture` passed all 4 codec tests.
- `cargo test -p liquidfun-test-protocol trace -- --nocapture` passed all 7 trace tests after the invariant follow-up.
- `cargo test -p liquidfun-test-protocol tolerance -- --nocapture` passed both tolerance tests.
- `cargo test -p liquidfun-test-protocol --all-features` passed all 37 unit tests and doctests.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun-test-protocol --all-features --no-deps` passed.
- `cargo xtask package verify` passed and proved serialization dependencies remain outside the published `liquidfun` archive.
- Static acceptance checks found strict `deny_unknown_fields` coverage, all required trace/reset/identity fields, and every closed tolerance policy; no `serde_json::Value`, `unwrap(`, generic `epsilon`, or `DEFAULT_EPSILON` was found.
- Before each implementation commit, the required ordered repository gate passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `git diff --check` passed, and `.planning/config.json` remained unstaged and uncommitted.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Revalidated trace source identity during raw-record conversion**

- **Found during:** Post-Task 2 invariant and threat-boundary scan
- **Issue:** Strict trace decoding bounded named/seeded source strings but initially converted them without rejecting an empty name/generator ID or zero generator version, allowing an invalid domain value before state validation.
- **Fix:** Made raw trace-source conversion fallible with the same nonempty/version invariant enforced by scenario decoding and added a boundary regression test.
- **Files modified:** `crates/liquidfun-test-protocol/src/trace.rs`, `crates/liquidfun-test-protocol/src/trace/tests.rs`
- **Verification:** The new focused test passes, all 7 trace tests pass, and the full ordered Rust gate passes.
- **Committed in:** `5110b4c`

***

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix closes a trust-boundary gap required by the plan; it adds no protocol breadth or scope beyond strict typed parsing.

## Issues Encountered

- The TDD RED states were not committed because repository policy requires all four Rust gates to pass before every commit. Each RED failure was observed first, then each completed task was committed atomically after GREEN verification.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-04 to consume `ValidatedTrace`, `FloatPolicy`, `DiscretePolicy`, and `CollectionPolicy` in the pure comparator and first-divergence report model.
- Ready for Plans 02-05 and 02-10 to serialize deterministic fixtures and feed strict records through native/C++ adapters and process supervision.
- No protocol-shape, provenance, resource-limit, reset-proof, consumer-packaging, or verification blocker remains.

## Self-Check: PASSED

- All seven implementation/test files listed in this summary exist.
- Task commits `7040662`, `976499f`, and `5110b4c` exist in repository history.
- Summary lifecycle metadata matches Plan 02-03, and all three requirement IDs are copied verbatim.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
