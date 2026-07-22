---
phase: 11-examples-headless-tooling-and-testbed
plan: "14"
subsystem: differential-harness
tags: [catalog, supervisor, jsonl, replay, failure-bundle, provenance, deterministic]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "07"
    provides: Renderer-neutral session controller with deterministic pause, step, and restart semantics
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "11"
    provides: Canonical catalog wire requests and exact resolved-byte authority
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "12"
    provides: Native and C++ catalog backends with canonical checkpoint capture
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "13"
    provides: Renderer-neutral semantic comparison model and stable mismatch signatures
provides:
  - One exact resolved-byte catalog run path shared by native execution, supervised C++ execution, replay, comparison, and persisted evidence
  - Long-lived catalog child supervision with provenance validation, bounded concurrent drains, deadlines, reset proof, and kill/reap guarantees
  - Confined atomic failure bundles with complete hash-verified replay authority and no seed-only fallback
affects: [phase11-headless-reports, phase11-testbed, phase11-minimization, phase11-evidence]
tech-stack:
  added: []
  patterns:
    - Exact resolved bytes and their SHA-256 identity remain the authority through every catalog consumer
    - Process and protocol failures stay categorically separate from semantic physics mismatches
key-files:
  created:
    - crates/liquidfun-differential/src/runner/catalog.rs
    - crates/liquidfun-differential/src/supervisor/catalog.rs
    - crates/liquidfun-differential/src/supervisor/catalog/protocol.rs
    - crates/liquidfun-differential/src/failure_bundle/catalog.rs
    - crates/liquidfun-differential/src/failure_bundle/catalog/replay.rs
    - crates/liquidfun-differential/tests/catalog_round_trip.rs
    - crates/liquidfun-differential/tests/catalog_failures.rs
  modified:
    - crates/liquidfun-differential/src/catalog_native/executor.rs
    - crates/liquidfun-differential/src/fixtures/replay.rs
    - crates/liquidfun-differential/src/supervisor.rs
    - crates/liquidfun-test-protocol/src/catalog/wire.rs
key-decisions:
  - "Extend the established synchronous supervisor with a dedicated catalog module instead of creating a second subprocess lifecycle."
  - "Bind the caller's exact request ID to native checkpoints so both engines and persisted replay evidence share one run identity."
  - "Reject seed-only, incomplete, extra-file, symlinked, path-escaping, oversized, or hash-inconsistent catalog bundles."
patterns-established:
  - "Catalog outcome boundary: Match, PhysicsMismatch, and HarnessFailure remain distinct typed outcomes."
  - "Failure publication: write bounded no-clobber temporary contents, fsync them, then atomically rename inside target/differential/catalog-failures."
requirements-completed: [EXMP-02, EXMP-03, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T03:45:20Z
duration: 29 min
completed: 2026-07-21
---

# Phase 11 Plan 14: Supervised Catalog Harness Integration Summary

**Exact resolved catalog bytes now drive native and long-lived C++ execution, semantic comparison, deterministic replay, and atomic hash-verified failure evidence through the established isolated harness.**

## Performance

- **Duration:** 29 min
- **Started:** 2026-07-22T03:16:00Z
- **Completed:** 2026-07-22T03:45:20Z
- **Tasks:** 1
- **Files modified:** 16

## Accomplishments

- Added catalog runner entrypoints that execute and replay one exact resolved payload, validate run identity before comparison, and preserve separate match, physics-mismatch, and harness-failure outcomes.
- Extended the mature child supervisor for long-lived catalog requests while retaining bounded concurrent output drains, handshake and limits-profile validation, deadlines, reset epochs/proof, quiet reconciliation, and poison kill/reap behavior.
- Added complete catalog failure bundles containing resolved bytes/hash, action and checkpoint authority, both checkpoint streams, comparison rows, first divergence, identities, bounded stderr, and controller state.
- Enforced confined no-clobber atomic publication and strict replay auditing of the exact manifest, file topology, sizes, hashes, semantic identities, schedules, and checkpoint counts.
- Exercised the real `liquidfun-reference` process for reused requests and a bounded fake child for malformed records, crashes, timeouts, provenance rejection, reset failure, and large-stderr retention.

## TDD Evidence

- **RED:** `cargo test -p liquidfun-differential --test catalog_round_trip --test catalog_failures` failed with unresolved imports for the new catalog outcomes, runner entrypoints, exact replay, and catalog failure-bundle APIs after the behavior tests were added.
- **GREEN:** The focused targets pass 9/9 tests covering exact native replay, D0 repeatability, real long-lived C++ reuse, pause/step/restart behavior, physics mismatch, durable replay, seed/path rejection, process classifications, provenance rejection, and bounded stderr.
- **REFACTOR:** Process protocol validation was separated from lifecycle supervision, and strict failure-bundle replay auditing was separated from atomic publication; the resulting implementation modules are 468 lines or fewer.

The intentionally failing RED state was not committed because repository policy requires every commit to follow a completely passing ordered Rust gate.

## Task Commits

1. **Task 1: Wire catalog runs through the existing harness lifecycle** - `0fe5ee6` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-differential/src/runner/catalog.rs` - Exact native/oracle execution, replay, typed outcomes, and semantic checkpoint comparison.
- `crates/liquidfun-differential/src/supervisor/catalog.rs` - Long-lived catalog child lifecycle, identities, bounds, deadlines, reset proof, and failure classification.
- `crates/liquidfun-differential/src/supervisor/catalog/protocol.rs` - Strict request/response framing and canonical checkpoint record validation.
- `crates/liquidfun-differential/src/failure_bundle/catalog.rs` - Complete bounded evidence construction and atomic confined publication.
- `crates/liquidfun-differential/src/failure_bundle/catalog/replay.rs` - Exact file, manifest, hash, identity, schedule, and semantic replay validation.
- `crates/liquidfun-differential/src/catalog_native/capture.rs` and `catalog_native/executor.rs` - Caller-bound request identity for native checkpoints.
- `crates/liquidfun-differential/src/fixtures/replay.rs` - Public verified catalog bundle replay bridge.
- `crates/liquidfun-differential/tests/catalog_round_trip.rs` - Native, real-oracle, session-control, D0, and mismatch coverage.
- `crates/liquidfun-differential/tests/catalog_failures.rs` - Persistence, confinement, child-failure, provenance, reset, and bounded-diagnostic coverage.
- `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs` - Deterministic catalog child behaviors for supervisor failure testing.
- `crates/liquidfun-test-protocol/src/catalog/wire.rs` - Read-only request/provenance getters needed at the supervision boundary.

## Decisions Made

- Reused the existing synchronous child state machine and I/O workers so catalog execution inherits one proven process-isolation authority rather than duplicating deadlines, drains, or poison cleanup.
- Kept exact canonical resolved bytes, SHA-256, action log, checkpoint schedule, request identity, and canonical checkpoint bytes together in `CatalogRunCapture`; downstream replay, comparison, minimization evidence, and persistence cannot silently regenerate inputs.
- Required the supervised child to match the requested build identity, limits-profile identity, evidence tier, request ID, resolved hash, checkpoint order, and reset epoch before its output can become physics evidence.
- Published failure evidence only below `target/differential/catalog-failures` with no-clobber names, bounded files, fsync, and atomic directory rename; replay rejects any topology or semantic inconsistency.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exposed read-only catalog provenance accessors**

- **Found during:** Task 1 supervisor request validation
- **Issue:** The validated request retained required build identity, limits-profile identity, evidence tier, and request ID privately, so the supervisor could not bind the child handshake and response records to the exact request without serialization mirroring.
- **Fix:** Added documented immutable getters for existing validated protocol values. No wire field, mutation path, or production-engine API changed.
- **Files modified:** `crates/liquidfun-test-protocol/src/catalog/wire.rs`
- **Verification:** Focused protocol consumers, deny-warnings Clippy, and the full workspace gate pass.
- **Committed in:** `0fe5ee6`

**2. [Rule 3 - Blocking] Bound native checkpoints to the caller's request identity**

- **Found during:** Task 1 exact cross-engine comparison wiring
- **Issue:** Native catalog execution generated a legacy default request ID internally, preventing one exact run identity from spanning native, C++, replay, comparison, and failure evidence.
- **Fix:** Added a private optional request-ID binding on the native executor while preserving the existing default for legacy callers.
- **Files modified:** `crates/liquidfun-differential/src/catalog_native/capture.rs`, `crates/liquidfun-differential/src/catalog_native/executor.rs`
- **Verification:** Round-trip tests prove both engines and exact replay retain the caller's request identity and bytes.
- **Committed in:** `0fe5ee6`

**Total deviations:** 2 auto-fixed blocking integration gaps.
**Impact on plan:** Both changes were minimal seams required to preserve the plan's single-authority guarantee. No dependency, renderer coupling, published engine surface, or alternate protocol was added.

## Issues Encountered

- The initial catalog failure-bundle implementation grew to 680 lines after strict replay validation. A final simplification pass split replay auditing into a 211-line child module, leaving publication at 468 lines and all behavior unchanged.
- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and uncommitted by this plan.

## Security Verification

- Untrusted child output is drained concurrently, record/count/byte bounded, deadline controlled, and rejected on malformed framing, incompatible identity, bad checkpoint order, missing terminal/reset evidence, or resource overrun.
- Every poisoned child is killed and reaped; timeout, crash, malformed record, provenance, reset, and resource failures remain distinct typed harness categories and never masquerade as physics mismatches.
- Failure-bundle paths reject symlinks, non-directory boundaries, traversal, collisions, extra or missing entries, and oversized content; publication remains confined and atomic.
- Replay verifies every manifest hash and byte length before decoding the exact resolved authority, action log, schedule, identities, checkpoint streams, and comparison completeness.
- Diagnostics retain only bounded stderr and semantic evidence. The published Rust crate graph remains independent of C++ and the renderer.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-14's `EXMP-02`, `EXMP-03`, and `EXMP-05` mappings are implemented in the integrated harness and retained in summary frontmatter. Their global requirement checkboxes remain intentionally unchanged until later Phase 11 end-to-end evidence proves the complete requirement scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later headless reporting and minimization plans can consume exact captures, typed outcomes, stable first-divergence evidence, and verified replay bundles without regenerating scenarios.
- Testbed presentation can reuse the same session controller and comparison output while remaining renderer-neutral and isolated from the C++ process.
- No blocker remains for the next incomplete Phase 11 plan.

## Self-Check: PASSED

- Confirmed all created runner, supervisor, evidence-bundle, replay, and integration-test files exist and implementation modules remain within the repository's 300-500 line guidance.
- Confirmed commit `0fe5ee6` exists and excludes all four fenced pre-existing worktree edits.
- Confirmed the focused catalog targets pass 9/9, including the real long-lived `liquidfun-reference` process when present.
- Confirmed the exact ordered `cargo fmt --all`, full-workspace deny-warnings Clippy, all-targets build, and all-features test gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-14`.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
