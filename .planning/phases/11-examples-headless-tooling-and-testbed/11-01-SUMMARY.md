---
phase: 11-examples-headless-tooling-and-testbed
plan: "01"
subsystem: testing-tooling
tags: [corpus, inventory, serde, validation, provenance]
requires:
  - phase: 01-upstream-provenance-licensing-and-reference-build
    provides: Pinned upstream revision and compatibility/discovery authorities
  - phase: 10-particle-groups-solvers-and-compatibility-sign-off
    provides: Complete pre-Phase-11 compatibility evidence vocabulary
provides:
  - Strict typed semantic upstream-corpus manifest vocabulary
  - Bounded fail-closed JSON parser for untrusted corpus records
  - Focused positive and adversarial parser/model regression tests
affects: [phase11-corpus-discovery, phase11-corpus-closure, phase11-scenario-catalog]
tech-stack:
  added: []
  patterns:
    - Parse untrusted corpus JSON through deny-unknown raw structs into invariant-bearing domain types
    - Bound bytes, nesting, records, evidence, identities, paths, reviews, and diagnostics before joins
key-files:
  created:
    - tools/xtask/src/inventory/corpus.rs
    - tools/xtask/src/inventory/corpus/model.rs
    - tools/xtask/tests/corpus_model.rs
    - tools/xtask/tests/fixtures/corpus/valid-minimal.json
    - tools/xtask/tests/fixtures/corpus/invalid-disposition.json
  modified:
    - tools/xtask/src/inventory.rs
key-decisions:
  - "Model applicability, terminal disposition, and compatibility impact as independent closed enums with explicit terminal-outcome validation."
  - "Bound corpus input to 4 MiB, JSON depth to 32, items to 2048, and per-item evidence mappings to 32 before downstream joins."
  - "Treat source path plus source symbol as the unique semantic source identity while keeping kind-prefixed item IDs independently unique."
patterns-established:
  - "Corpus boundary: validate size and nesting, deserialize strict raw records, then construct checked semantic types."
  - "Corpus diagnostics: expose stable categories and at most a validated bounded item ID, never raw record content."
requirements-completed: [TEST-03, EXMP-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-21T22:44:41Z
duration: 14 min
completed: 2026-07-21
---

# Phase 11 Plan 01: Strict Upstream Corpus Authority Summary

**A strict, bounded semantic corpus model now gives every future upstream test, example, and testbed record a source-derived identity, reviewed terminal outcome, and fail-closed evidence boundary.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-21T22:30:25Z
- **Completed:** 2026-07-21T22:44:41Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added closed corpus kind, applicability, terminal-disposition, compatibility-impact, evidence-kind, identity, review, item, and manifest types.
- Rejected unknown fields and variants, wrong revisions, invalid or escaping paths, incoherent terminal outcomes, duplicate IDs/source identities/evidence references, vague or self-referential rationale, and oversized inputs or collections.
- Kept errors bounded to stable semantic categories and validated item IDs without exposing raw untrusted records.
- Added 14 focused tests proving valid round-trip behavior and every required negative boundary, including exact-limit and one-over-limit collections.

## TDD Evidence

- **RED:** The new `corpus_model` target failed because the production corpus parser module did not yet exist.
- **GREEN:** The parser, checked model, fixtures, and 14 focused boundary tests were added; the target and complete repository gate pass.
- **REFACTOR:** Validation was kept in one cohesive model module with a thin parsing shell; no separate refactor commit was needed.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define the corpus manifest model and fail-closed parser** - `f1d7097` (feat)

**Plan metadata:** committed with this summary.

## Files Created/Modified

- `tools/xtask/src/inventory.rs` - Compiles the checked parser into the production inventory graph without mutating an authority manifest.
- `tools/xtask/src/inventory/corpus.rs` - Enforces byte and JSON-depth limits before strict deserialization.
- `tools/xtask/src/inventory/corpus/model.rs` - Owns closed corpus vocabulary, reviewed bounds, constructors, uniqueness, paths, evidence, review, and terminal-outcome rules.
- `tools/xtask/tests/corpus_model.rs` - Exercises round-trip and adversarial parser/model behavior one concern at a time.
- `tools/xtask/tests/fixtures/corpus/valid-minimal.json` - Minimal valid checked corpus record.
- `tools/xtask/tests/fixtures/corpus/invalid-disposition.json` - Unknown-enum rejection fixture.

## Decisions Made

- Applicability, disposition, and compatibility impact remain independent typed facts; validation permits only coherent terminal combinations and requires disposition-appropriate evidence.
- Source identity uniqueness is based on the normalized upstream path and symbol, independent of the separately typed item ID.
- Reviewed limits are 4 MiB input, depth 32, 2,048 items, 32 evidence mappings per item, 160-byte IDs, 512-byte paths/references, and 1,024-byte rationale.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Security Verification

- All six planned STRIDE mitigations are represented: strict schema/variant rejection, pinned revision and unique source identity, bounded work, redacted error context, no new runtime dependency, and explicit review/evidence records.
- No `unsafe`, raw pointer, process, network, renderer, C++, or published-crate dependency surface was introduced.
- No unresolved high-severity ASVS L1 finding remains.

## Requirements Status

The frontmatter preserves Plan 11-01's `TEST-03` and `EXMP-01` mappings. Their global requirement checkboxes remain pending until later Phase 11 plans populate, classify, join, and verify the complete semantic corpus; this plan establishes the required authority vocabulary without claiming corpus closure.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The strict model is ready for Plan 11-02 source discovery and snapshot work.
- No authoritative corpus manifest was created or mutated in this plan, as required.
- Global `TEST-03` and `EXMP-01` completion remains deferred to Phase 11 closure verification.

## Self-Check: PASSED

- Confirmed all five created files and the modified inventory entrypoint exist.
- Confirmed task commit `f1d7097` exists and contains only the six Plan 11-01 artifacts.
- Confirmed focused tests pass 14/14 and the ordered format, Clippy, all-target build, and all-feature test gate passes.
- Confirmed read-only verification did not change the worktree and no stub or unplanned threat surface was introduced.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
