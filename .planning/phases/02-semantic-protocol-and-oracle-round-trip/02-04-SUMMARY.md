---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "04"
subsystem: differential-testing-protocol
tags: [rust, json-schema, toml, deterministic-presentation, tolerance-policy]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Strict typed scenario, provenance-bound trace, and tolerance authorities from Plan 02-03
provides:
  - Strict reviewed Phase-2 tolerance-profile presentation with exact-bit thresholds
  - Deterministic closed protocol, scenario, and trace JSON Schema presentations
  - Read-only byte-stability checks that never rewrite tracked artifacts
affects: [02-05, protocol-fixtures, comparator, cpp-oracle-adapter, artifact-review]

tech-stack:
  added: []
  patterns: [typed authority with deterministic presentation, test-only read-only artifact checks, explicit independent version axes]

key-files:
  created:
    - crates/liquidfun-test-protocol/src/schema.rs
    - crates/liquidfun-test-protocol/src/schema/tests.rs
    - protocol/schemas/protocol-v1.schema.json
    - protocol/schemas/scenario-v1.schema.json
    - protocol/schemas/trace-v1.schema.json
    - protocol/tolerances/phase2-v1.toml
  modified:
    - crates/liquidfun-test-protocol/src/lib.rs

key-decisions:
  - "Keep schema and tolerance renderers test-only so ordinary protocol builds expose no regeneration or filesystem-write path."
  - "Mark schemas as deterministic presentation and state explicitly that typed Rust/C++ validation owns cross-field references, uniqueness, ordering, hashes, and aggregate limits."
  - "Limit Phase-2 numeric presentation to exact simulation-time bits plus clearly synthetic comparator-coverage policies, without claiming future subsystem tolerances."

patterns-established:
  - "Presentation checks: render fixed typed documents in memory and compare byte-for-byte with newline-terminated tracked files."
  - "Closed schemas: every record-shaped object rejects additional properties while version axes remain independently visible."

requirements-completed:
  - COMP-03
  - COMP-05
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T07:44:00Z

duration: 18 min
completed: 2026-07-10
---

# Phase 2 Plan 04: Deterministic Schema and Tolerance Presentation Summary

**Strict typed protocol authority now has reviewable, byte-stable JSON Schema and TOML presentations whose ordinary checks are read-only.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-10T07:26:00Z
- **Completed:** 2026-07-10T07:44:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added a closed Phase-2 tolerance profile that preserves the typed profile ID, version, SHA-256 identity, exact simulation-time policy, and exact-bit synthetic comparator thresholds.
- Added deterministic protocol, scenario, and trace JSON Schema presentations with closed records, reviewed bounds, and explicit protocol/scenario/trace/tolerance version axes.
- Added focused read-only tests that render all four tracked presentations in memory, compare exact newline-terminated bytes, and reject unknown or duplicate tolerance policies, unsupported versions, decimal thresholds, and mismatched hashes.
- Kept schema cross-field authority truthful: references, uniqueness, ordering, identity hashes, record sequences, reset proofs, and aggregate limits remain enforced by typed Rust/C++ validators.

## Task Commits

Each task was committed atomically:

1. **Task 1: Check the reviewed tolerance profile presentation** - `66e2bdf` (`feat`)
1. **Task 2: Generate deterministic schema presentation** - `07fcafc` (`feat`)
1. **Task 2 verification follow-up: Isolate presentation checks to tests** - `6bed186` (`fix`)

## Files Created/Modified

- `crates/liquidfun-test-protocol/src/lib.rs` - Registers the schema checker only for test targets.
- `crates/liquidfun-test-protocol/src/schema.rs` - Parses and validates strict tolerance presentation and renders three deterministic JSON schemas in memory.
- `crates/liquidfun-test-protocol/src/schema/tests.rs` - Verifies exact tracked bytes, newline termination, strict profile rejection cases, closed records, and explicit version axes.
- `protocol/tolerances/phase2-v1.toml` - Reviewed exact-bit Phase-2 field-policy presentation with synthetic comparator-only numeric variants.
- `protocol/schemas/protocol-v1.schema.json` - Closed handshake and scenario-request transport presentation.
- `protocol/schemas/scenario-v1.schema.json` - Closed empty-world scenario presentation with reviewed limits.
- `protocol/schemas/trace-v1.schema.json` - Closed trace-begin, checkpoint, and trace-end presentation.

## Decisions Made

- Used existing `serde_json` and `toml` dependencies rather than adding a schema framework or generator dependency.
- Kept presentation generation inside test-only code because the plan requires ordinary checks to be read-only and no consumer or runtime path needs schema emission.
- Split schema tests into `schema/tests.rs`, following the repository's `foo.rs` plus `foo/` module shape and keeping the implementation file below the 628-line refactor trigger.
- Added `x-version-axes` presentation metadata so independent supported version axes remain directly reviewable even when a schema describes a nested scenario or streamed trace payload.

## Verification Evidence

- TDD RED for Task 1 produced the expected unresolved presentation-checker imports before implementation.
- `cargo test -p liquidfun-test-protocol tolerance_profile_presentation -- --nocapture` passed both strict profile tests.
- `cargo test -p liquidfun-test-protocol schema -- --nocapture` passed all four schema/presentation tests.
- `cargo clippy -p liquidfun-test-protocol --all-targets --all-features -- -D warnings` passed after the test-only module correction.
- `cargo build -p liquidfun-test-protocol --all-targets --all-features` passed.
- `cargo test -p liquidfun-test-protocol --all-features` passed all 41 unit tests and doctests.
- `git diff --exit-code -- protocol/schemas protocol/tolerances` passed after check-mode tests, proving no tracked presentation was rewritten.
- `cargo xtask package verify` passed and preserved the published consumer boundary.
- Before every implementation commit, the required ordered repository gate passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Static review found no filesystem write/create calls, no `unwrap()` use, no generic/global decimal tolerance, and no source file above the repository's refactor trigger after the test-module split.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Compile presentation checkers only for test targets**

- **Found during:** Plan-level warning-denied package Clippy verification
- **Issue:** The private `schema` module was initially compiled into ordinary library targets even though it has no runtime callers, causing dead-code failures and two ownership lint findings.
- **Fix:** Gated the checker module with `#[cfg(test)]` and borrowed fixed JSON values during deterministic rendering.
- **Files modified:** `crates/liquidfun-test-protocol/src/lib.rs`, `crates/liquidfun-test-protocol/src/schema.rs`
- **Verification:** Package-scoped Clippy, build, all 41 tests, both focused filters, artifact clean-diff checks, and package isolation all pass.
- **Committed in:** `6bed186`

***

**Total deviations:** 1 auto-fixed (1 blocking verification issue)
**Impact on plan:** The fix narrows the checker to its intended read-only verification surface and introduces no protocol or runtime scope.

## Issues Encountered

- The Task 1 RED state was not committed because repository policy requires the full Rust gate to pass before every commit. The failure was observed first, then the completed task was committed after GREEN verification.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-05 to build deterministic accepted/rejected protocol fixtures against the strict typed and presentation contracts.
- Schema and tolerance presentations are reviewable, version-explicit, closed, byte-stable, and free of runtime regeneration paths.
- No package-isolation, schema-drift, tolerance-policy, or verification blocker remains.

## Self-Check: PASSED

- All seven implementation and presentation files listed in this summary exist.
- Task commits `66e2bdf`, `07fcafc`, and `6bed186` exist in repository history.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-04.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
