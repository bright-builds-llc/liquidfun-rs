---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "02"
subsystem: differential-testing-protocol
tags: [rust, protocol, invariants, provenance, resource-limits, failure-taxonomy]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Unpublished engine-neutral protocol crate and isolated differential-runner dependency direction from Plan 02-01
provides:
  - Validated independent protocol, scenario, trace, and tolerance version types
  - Exact IEEE-754 float-bit transport and bounded semantic identifiers
  - Immutable one-shot, reuse, and sanitizer harness limit profiles with stable hashes
  - Validated build identities and exhaustive bounded harness-failure evidence
affects: [02-03, 02-04, 02-05, 02-10, protocol-codec, scenario-validation, process-supervisor]

tech-stack:
  added: []
  patterns: [parse into invariant-bearing domain types, exact float-bit authority, immutable reviewed profiles, bounded failure evidence]

key-files:
  created:
    - crates/liquidfun-test-protocol/src/failure.rs
    - crates/liquidfun-test-protocol/src/float_bits.rs
    - crates/liquidfun-test-protocol/src/ids.rs
    - crates/liquidfun-test-protocol/src/limits.rs
    - crates/liquidfun-test-protocol/src/provenance.rs
  modified:
    - crates/liquidfun-test-protocol/src/lib.rs

key-decisions:
  - "Validate all protocol versions, semantic IDs, and SHA-256 identities at construction or deserialization so downstream code cannot receive unchecked primitives."
  - "Expose only named immutable phase-2 limit profiles; process reuse changes solely through reviewed one-shot, corpus, and sanitizer constructors."
  - "Model physics mismatch outside HarnessFailureKind while preserving bounded request, provenance, process, stderr, and limit evidence for every harness failure."

patterns-established:
  - "Boundary newtypes: validated constructors and custom deserialization reject unsupported or malformed raw values before domain use."
  - "Evidence hashes: canonical field sequences produce stable SHA-256 identities for builds and named limit profiles."
  - "Failure boundary: HarnessFailureKind classifies process/protocol/provenance failures only, with Option-bearing evidence named maybe_."

requirements-completed:
  - COMP-03
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T06:46:03Z

duration: 10 min
completed: 2026-07-10
---

# Phase 2 Plan 02: Invariant-Bearing Protocol Primitives Summary

**Strict version, identifier, exact-float, resource-limit, provenance, and failure-evidence types now form the engine-neutral foundation for every later protocol and supervisor path.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-10T06:35:21Z
- **Completed:** 2026-07-10T06:46:03Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added four independent version newtypes, exact `FloatBits`, bounded request/scenario/checkpoint IDs, and typed semantic entity IDs with fallible parsing and strict Serde boundary behavior.
- Added immutable `phase2-default-v1`, reuse, and sanitizer profiles covering researched byte, depth, count, stderr, deadline, and request-budget limits with deterministic SHA-256 identities.
- Added validated full-revision build provenance with adapter/compiler/target/flag fields and independently recomputable stable identity hashes.
- Added all 22 D-10 process/protocol/provenance failure variants, explicitly excluding physics mismatch, plus bounded stderr and process lifecycle evidence.
- Added 16 focused Arrange/Act/Assert unit tests, including exact NaN/signed-zero float bits, ID rejection classes, profile boundaries, hash stability, provenance rejection, and failure evidence.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define invariant-bearing protocol primitives and harness failures** - `f3d52e9` (`feat`)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/lib.rs` - Exports protocol modules and owns the four supported-version newtypes.
- `crates/liquidfun-test-protocol/src/float_bits.rs` - Preserves authoritative `f32` values as exact `u32` bit patterns.
- `crates/liquidfun-test-protocol/src/ids.rs` - Validates bounded lowercase ASCII IDs and engine-neutral semantic entity identities.
- `crates/liquidfun-test-protocol/src/limits.rs` - Defines immutable reviewed phase-2 limit profiles and stable hashes.
- `crates/liquidfun-test-protocol/src/provenance.rs` - Validates SHA-256 values, complete build identity fields, and reported identity hashes.
- `crates/liquidfun-test-protocol/src/failure.rs` - Defines the exhaustive harness taxonomy and bounded diagnostic evidence.

## Decisions Made

- Kept validation in constructors and custom deserializers so raw versions, identifiers, and digests cannot bypass invariants when Plan 02-03 adds JSONL decoding.
- Kept all limit fields private and exposed only named profile constructors, preventing future CLI or process code from silently weakening the reviewed phase-2 contract.
- Made request/scenario/session identity optional only where startup failures occur before those values exist; every optional internal field uses the repository's `maybe_` convention.
- Used exact byte retention and truncation counts for stderr evidence, leaving concurrent draining and first/last window policy to the later supervisor without permitting unbounded storage here.

## Test and Build Evidence

- TDD RED: `cargo test -p liquidfun-test-protocol --lib` produced the expected unresolved-type compile errors before implementation.
- TDD GREEN: `cargo test -p liquidfun-test-protocol --lib` passed all 16 focused unit tests after implementation.
- Plan-scoped `cargo fmt`, Clippy with warnings denied, all-target build, all-feature tests, and doctests passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun-test-protocol --all-features --no-deps` passed.
- `cargo xtask package verify` passed and preserved the Cargo-only published consumer boundary.
- Every required `HarnessFailureKind` variant was found, `PhysicsMismatch` was absent, no `unwrap(` call was found, and every researched limit constant was present.
- Before the task commit, the mandated ordered repository gate passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `git diff --check` passed and the simplification review found each new file below the repository's 628-line refactor trigger with no unnecessary new dependency or effectful code.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The expected TDD RED state was not committed because the explicit repository policy requires all four Rust gates to pass before every commit. Tests were still written and observed failing first; the completed task was then committed atomically after GREEN verification.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for `02-03-PLAN.md` to build strict bounded JSONL codecs on the validated versions, identifiers, float bits, hashes, and limit profiles.
- Ready for later supervisor plans to attach live process and stderr evidence to the exhaustive non-physics taxonomy.
- No dependency, package-isolation, provenance, resource-limit, or failure-classification blocker remains.

## Self-Check: PASSED

- All six implementation files exist.
- Task commit `f3d52e9` exists and contains exactly the six scoped protocol source files.
- Summary lifecycle metadata and requirement IDs match Plan 02-02.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
