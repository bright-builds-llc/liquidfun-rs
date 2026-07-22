---
phase: 11-examples-headless-tooling-and-testbed
plan: "13"
subsystem: semantic-comparison
tags: [checkpoint, comparison, phase4-policy, semantic-path, renderer-neutral, bounded-diagnostics]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "10"
    provides: Strict canonical checkpoint wire contract with stable observations and primitives
  - phase: 04-math-and-numerical-foundation
    provides: Closed named Phase 4 floating-point policies and comparison helpers
provides:
  - Complete renderer-neutral comparison model with exact, within-policy, mismatch, Rust-only, and oracle-only states
  - Exhaustive source-ordered checkpoint field walk with exact structural and closed numeric policy bindings
  - Bounded diagnostic values, stable primitive focus keys, and report-compatible mismatch signatures
affects: [phase11-headless-runner, phase11-failure-bundles, phase11-testbed, phase11-evidence]
tech-stack:
  added: []
  patterns:
    - Canonical checkpoints are compared once into an owned model consumed by every presentation
    - Numeric fields fail closed unless one explicit Phase 4 policy binds the exact path
key-files:
  created:
    - crates/liquidfun-differential/src/comparison_model.rs
    - crates/liquidfun-differential/src/comparison_model/diff.rs
    - crates/liquidfun-differential/src/comparison_model/diff/primitives.rs
    - crates/liquidfun-differential/tests/comparison_model.rs
  modified:
    - crates/liquidfun-differential/src/lib.rs
    - crates/liquidfun-differential/src/report.rs
    - crates/liquidfun-test-protocol/src/checkpoint.rs
    - crates/liquidfun-test-protocol/src/checkpoint/primitive.rs
key-decisions:
  - "Keep comparison traversal in small cohesive modules while one bounded builder owns path validation, value truncation, and stable signatures."
  - "Bind simulation time to the exact Phase 4 abs policy, linear geometry and lengths to vector-length policy, angles to rotation policy, and typed numeric observations to their declared policy path."
  - "Treat run, schema, and checkpoint identity disagreement as a harness error before comparison; represent missing semantic observations and records as Rust-only or oracle-only entries."
patterns-established:
  - "Comparison authority: presentations consume ordered ComparisonEntry records and never reinterpret engine or checkpoint internals."
  - "Primitive focus: only stable semantic primitive keys cross into mismatch presentation; source indices and native storage never do."
requirements-completed: [RIGD-10, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T03:15:17Z
duration: 20 min
completed: 2026-07-21
---

# Phase 11 Plan 13: Renderer-Neutral Semantic Comparison Summary

**Canonical checkpoints now produce one exhaustive bounded comparison model shared by headless reports and visual consumers, with exact structure, named numeric policy, missing-side, and stable primitive-focus evidence.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-22T02:55:00Z
- **Completed:** 2026-07-22T03:15:17Z
- **Tasks:** 1
- **Files modified:** 12

## Accomplishments

- Added exact-match, within-policy, physics-mismatch, Rust-only, and oracle-only comparison states with canonical semantic paths, bounded values/context, stable focus keys, and deterministic mismatch signatures.
- Walked checkpoint identity, position, simulation time, structural and numeric observations, source-significant occurrences, declared unordered sets, every primitive key/style/geometry variant, and duration-free profile names in one deterministic source order.
- Bound every numeric value to one closed Phase 4 policy, rejected absent/open/private paths as harness errors, and capped model entries before allocation growth can become unbounded.
- Split traversal into focused identity/observation, collection/profile, primitive, policy, and bounded-builder modules; the largest implementation file is 431 lines.

## TDD Evidence

- **RED:** `cargo test -p liquidfun-differential --test comparison_model` failed with unresolved imports for `ComparisonLimits`, `ComparisonState`, and `compare_canonical_checkpoints` after the first behavior test was added.
- **GREEN:** The focused target now passes 14/14 tests covering exact, within-policy, true mismatch, both missing sides, source order, declared set canonicalization, duplicate rejection, absent policy, private path, duration exclusion, stable primitive focus, bounded diagnostics, and at-limit/one-over behavior.
- **REFACTOR:** The initial 1,223-line diff implementation was split into a 65-line orchestrator and cohesive modules of 214, 197, 362, and 431 lines; focused deny-warnings Clippy and the full ordered gate remain green.

The intentionally failing RED state was not committed because repository policy requires every commit to follow a completely passing ordered Rust gate.

## Task Commits

1. **Task 1: Implement exhaustive semantic diff construction** - `acbf79f` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-differential/src/comparison_model.rs` - Public bounded comparison state, entry, model, limit, and error contracts.
- `crates/liquidfun-differential/src/comparison_model/diff.rs` - Small exhaustive-walk orchestrator.
- `crates/liquidfun-differential/src/comparison_model/diff/header.rs` - Identity, position, structural observation, and numeric observation traversal.
- `crates/liquidfun-differential/src/comparison_model/diff/collections.rs` - Source-significant occurrence, canonical set, and profile-name traversal.
- `crates/liquidfun-differential/src/comparison_model/diff/primitives.rs` - Closed primitive key, style, kind, and geometry traversal with focus keys.
- `crates/liquidfun-differential/src/comparison_model/diff/builder.rs` - Entry bounds, value/context truncation, path validation, and stable mismatch signatures.
- `crates/liquidfun-differential/src/comparison_model/policy.rs` - Fail-closed Phase 4 numeric policy resolution and evaluation.
- `crates/liquidfun-differential/tests/comparison_model.rs` - Fourteen focused public behavior tests.
- `crates/liquidfun-test-protocol/src/checkpoint.rs` and `checkpoint/primitive.rs` - Read-only semantic accessors needed for typed comparison without serialization mirrors.
- `crates/liquidfun-differential/src/report.rs` - Minimal existing-report-compatible checkpoint mismatch signature bridge.
- `crates/liquidfun-differential/src/lib.rs` - Curated comparison model export.

## Decisions Made

- Preserved source-significant occurrence and primitive order exactly; canonicalized only `CheckpointSet` membership and primitive subsequences already declared canonical by the validated checkpoint contract.
- Stored values as bounded diagnostic strings while retaining exact authoritative float bits and named policy paths in each numeric entry.
- Reused the existing report module's SHA-256 mismatch-signature construction seam so first divergence remains stable for replay and minimization without creating a parallel signature authority.
- Kept profile names structural and excluded all measured duration values from the deterministic model.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added read-only checkpoint and primitive accessors**

- **Found during:** Task 1 typed field-walk design
- **Issue:** The strict checkpoint contract intentionally kept fields private and exposed only schema, unordered-set, and profile getters, so differential tooling could not perform the required typed exhaustive walk without a lossy serialization mirror.
- **Fix:** Added documented read-only accessors for existing semantic fields and primitive geometry. No storage internals, mutable references, new wire fields, or published-engine API were exposed.
- **Files modified:** `crates/liquidfun-test-protocol/src/checkpoint.rs`, `crates/liquidfun-test-protocol/src/checkpoint/primitive.rs`
- **Verification:** Protocol and full workspace tests pass; comparison tests traverse every closed variant.
- **Committed in:** `acbf79f`

**2. [Rule 3 - Blocking] Added a minimal stable checkpoint signature bridge**

- **Found during:** Task 1 mismatch-entry construction
- **Issue:** Existing mismatch builders were closed over legacy trace paths and could not accept canonical checkpoint paths without duplicating signature hashing inside the new model.
- **Fix:** Added one crate-private helper beside the existing report hashing functions and reused it for non-match comparison entries.
- **Files modified:** `crates/liquidfun-differential/src/report.rs`
- **Verification:** Repeated mismatches retain one deterministic SHA-256 identity; all report and minimizer tests remain green.
- **Committed in:** `acbf79f`

**Total deviations:** 2 auto-fixed blocking integration gaps.
**Impact on plan:** Both changes preserve one typed checkpoint and one report-signature authority. No new dependency, wire field, renderer coupling, or production-engine surface was introduced.

## Issues Encountered

- Focused deny-warnings Clippy identified one redundant match arm, uniform limit-field prefixes, and an intentionally exhaustive geometry match over the default line threshold. The redundant structure and names were simplified; the closed geometry match keeps a narrow reasoned allowance so all primitive fields remain visibly auditable.
- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and uncommitted by this plan.

## Security Verification

- Schema, request, resolved-plan, and checkpoint identity are checked before semantic traversal; disagreement is a harness error rather than misleading parity evidence.
- Every generated path rejects wildcard, default/fallback, private, bracket, unknown, empty, and oversized spellings; every numeric path must resolve to one explicit float policy.
- Entries, values, context, primitives, vertices, observations, sets, and profiles remain bounded by validated checkpoint and comparison profiles.
- Diagnostics contain stable semantic IDs, exact bits, and closed categories only. No pointer, arena slot, dense particle row, native record, stack trace, stderr, or wall-clock duration enters the model.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-13's `RIGD-10` and `EXMP-05` mappings are implemented in the comparison layer and retained in summary frontmatter. Their global requirement checkboxes remain intentionally unchanged until later Phase 11 end-to-end evidence proves the complete requirement scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11-14 can supervise native/oracle catalog sessions and consume one authoritative comparison model without rereading engine state.
- Later headless reports, failure bundles, and testbed overlays can share identical ordered entries, states, policies, values, and primitive focus keys.
- No blocker remains for the next incomplete Phase 11 plan.

## Self-Check: PASSED

- Confirmed every created comparison module and integration test exists and the largest implementation file remains within the repository's 300-500 line guidance.
- Confirmed commit `acbf79f` exists and excludes all four fenced pre-existing worktree edits.
- Confirmed focused comparison tests pass 14/14 and focused deny-warnings Clippy passes.
- Confirmed the exact ordered `cargo fmt --all`, full-workspace deny-warnings Clippy, all-targets build, and all-features test gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-13`.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
