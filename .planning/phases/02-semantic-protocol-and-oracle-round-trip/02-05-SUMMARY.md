---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "05"
subsystem: differential-testing-protocol
tags: [rust, jsonl, fixtures, exact-float-bits, provenance, reset-proof]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Strict typed protocol authority and deterministic schema/tolerance presentations from Plans 02-03 and 02-04
provides:
  - Permanent bounded empty-world named scenario with two ordered checkpoint boundaries
  - Canonical accepted request and complete provenance-bound reset-proven trace fixtures
  - Five typed rejection fixtures for duplicate, unknown-kind, partial, version, and ID-limit failures
  - Read-only integration tests that decode, classify, validate, and canonicalize every fixture in memory
affects: [02-06, 02-10, cpp-oracle-adapter, rust-adapter, protocol-fixtures, differential-replay]

tech-stack:
  added: []
  patterns: [checked-in canonical JSONL corpus, public typed fixture validation, in-memory byte canonicalization]

key-files:
  created:
    - scenarios/phase-02/empty-world.json
    - protocol/fixtures/accepted/empty-world-request.jsonl
    - protocol/fixtures/accepted/empty-world-trace.jsonl
    - protocol/fixtures/rejected/duplicate-member.jsonl
    - protocol/fixtures/rejected/unknown-record-kind.jsonl
    - protocol/fixtures/rejected/partial-record.jsonl
    - protocol/fixtures/rejected/unsupported-version.jsonl
    - protocol/fixtures/rejected/oversized-id.jsonl
    - crates/liquidfun-test-protocol/tests/fixtures.rs
  modified: []

key-decisions:
  - "Use exact 0.5-second timestep bit patterns so two ordered empty-world checkpoints have distinguishable, exactly representable simulation times."
  - "Canonicalize checked-in request, handshake, and trace records only in memory through validated public protocol values; verification never regenerates or rewrites the corpus."
  - "Keep malformed corpus cases minimal so each rejected file reaches one intended stable codec category before unrelated validation can obscure the failure."

patterns-established:
  - "Fixture ownership: one integration target names every permanent corpus path and asserts its typed accepted or rejected contract."
  - "Read-only evidence: run canonical checks repeatedly against staged bytes and require a clean tracked-path diff."

requirements-completed:
  - COMP-03
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T08:00:09Z

duration: 9 min
completed: 2026-07-10
---

# Phase 2 Plan 05: Permanent Scenario and Protocol Fixture Corpus Summary

**A canonical exact-bit empty-world scenario now drives byte-stable accepted request/trace records and five distinctly classified strict-boundary rejection fixtures.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-07-10T07:50:25Z
- **Completed:** 2026-07-10T08:00:09Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added a named empty-world scenario with exact gravity/timestep bits, bounded solver settings, two ordered step/checkpoint boundaries, and exact world-count/simulation-time observables.
- Added a canonical request plus a complete handshake/begin/checkpoint/end trace whose request, scenario, tolerance, build identity, payload hash, checkpoint count, and reset proof all validate together.
- Added five minimal rejection records that deterministically classify duplicate members, unknown record kinds, missing final newline, unsupported protocol versions, and oversized typed IDs.
- Added eight focused Arrange/Act/Assert integration tests that consume every corpus path through public protocol APIs and compare canonical encodings without writing files.

## Task Commits

Each task was committed atomically:

1. **Task 1: Check the named scenario and protocol fixture corpus** - `b5a3bd7` (`feat`)

## Files Created/Modified

- `scenarios/phase-02/empty-world.json` - Permanent bounded named scenario with two exact-bit steps and ordered checkpoints.
- `protocol/fixtures/accepted/empty-world-request.jsonl` - Canonical validated scenario request bound to the reviewed Phase-2 tolerance identity.
- `protocol/fixtures/accepted/empty-world-trace.jsonl` - Complete provenance-checked, two-checkpoint, reset-proven C++-oracle trace stream.
- `protocol/fixtures/rejected/duplicate-member.jsonl` - Duplicate-member codec rejection.
- `protocol/fixtures/rejected/unknown-record-kind.jsonl` - Unknown tagged record-kind rejection.
- `protocol/fixtures/rejected/partial-record.jsonl` - Intentional missing-final-newline framing rejection.
- `protocol/fixtures/rejected/unsupported-version.jsonl` - Unsupported protocol-version rejection.
- `protocol/fixtures/rejected/oversized-id.jsonl` - Typed 129-byte request-ID boundary rejection.
- `crates/liquidfun-test-protocol/tests/fixtures.rs` - Read-only integration ownership for all scenario and protocol fixture paths.

## Decisions Made

- Chose exact `0.5_f32` timestep bits for both commands so checkpoint times are distinct (`0.5` then `1.0`) without introducing rounding ambiguity into the first permanent vertical slice.
- Used the existing typed `BuildIdentity`, scenario hashing, checkpoint payload hashing, and tolerance profile authorities; no parallel hash or primitive representation was added.
- Kept rejection inputs deliberately small so each file proves exactly one framing, shape, version, or typed-limit category.

## Verification Evidence

- TDD RED ran all eight focused tests and failed on the intentionally absent corpus paths before fixture implementation.
- `cargo test -p liquidfun-test-protocol fixtures -- --nocapture` passed all eight focused tests twice consecutively after GREEN.
- `cargo clippy -p liquidfun-test-protocol --all-targets --all-features -- -D warnings`, package build, and all-feature package tests passed; the protocol crate now has 41 unit tests plus 8 fixture integration tests.
- `cargo xtask package verify` passed and proved the scenario/fixture corpus remains outside the published `liquidfun` archive.
- The required ordered repository sequence passed before the task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Static acceptance checks found exact `gravity_*_bits` and `timestep_bits`, the complete handshake/begin/checkpoint/end/reset/identity fields, every named fixture path, and the intentional newline distinction for the partial record.
- Two consecutive fixture runs left `git diff --exit-code -- protocol/fixtures scenarios/phase-02/empty-world.json` clean.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The TDD RED state was not committed because repository policy requires all four Rust gates to pass before every commit. The failure was observed first, then the completed GREEN task was committed atomically after the full gate passed.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-06 to consume the permanent accepted/rejected corpus in pure comparison and reporting work.
- Ready for later Rust and C++ adapter plans to run the same exact-bit empty-world request and reproduce the validated trace contract.
- No fixture-drift, typed-category, reset-proof, package-isolation, or verification blocker remains.

## Self-Check: PASSED

- All nine implementation and fixture files listed in this summary exist.
- Task commit `b5a3bd7` exists and contains exactly the nine owned Plan 02-05 files.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-05.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
