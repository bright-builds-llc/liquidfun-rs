---
phase: 11-examples-headless-tooling-and-testbed
plan: "16"
subsystem: catalog-regression-replay
tags: [catalog, regression, canonical-json, sha256, replay, provenance, d0, security]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "14"
    provides: Exact resolved-byte native execution, semantic checkpoint capture, and D0 replay
provides:
  - Three reviewed content-addressed resolved catalog fixtures for rigid stack, rope joint, and particle group behavior
  - Strict manifest validation for complete run identity, settings, actions, checkpoints, provenance, and native D0 evidence
  - Read-only replay that rejects regenerated, drifted, stale, duplicate, linked, or path-escaping fixture authority before World creation
affects: [phase11-failure-replay, phase11-benchmarks, phase11-evidence, phase12-regression]
tech-stack:
  added: []
  patterns:
    - Persist exact canonical resolved bytes rather than accepting a seed or mutable generator as replay authority
    - Validate the complete filesystem, schema, catalog mapping, and provenance boundary before native effects
key-files:
  created:
    - scenarios/regressions/catalog-manifest.json
    - scenarios/catalog/rigid-stack-v1.json
    - scenarios/catalog/joint-rope-v1.json
    - scenarios/catalog/particle-group-v1.json
    - crates/liquidfun-differential/src/fixtures/replay/catalog.rs
    - crates/liquidfun-differential/tests/catalog_regressions.rs
  modified:
    - crates/liquidfun-differential/src/fixtures.rs
    - crates/liquidfun-differential/src/fixtures/replay.rs
key-decisions:
  - "Treat exact canonical resolved bytes plus SHA-256 as the only replay input; seeds and current generator output are validation witnesses, never substitutes."
  - "Require every manifest path component to be a confined regular non-symlink entry and validate every fixture before constructing any native World."
  - "Review D0 through repeated production replay and an independent direct semantic-checkpoint serialization path with explicit expected digests."
patterns-established:
  - "Catalog regression authority: strict manifest metadata must agree with decoded bytes and a fresh pure typed-catalog resolution byte for byte."
  - "D0 review: production canonical JSONL replay and an independent serde-plus-newline checkpoint digest must agree with tracked identities."
requirements-completed: [EXMP-02, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T04:39:35Z
duration: 15 min
completed: 2026-07-21
---

# Phase 11 Plan 16: Content-Addressed Catalog Regression Replay Summary

**Three representative catalog runs now replay from immutable canonical bytes through a strict content-addressed manifest and independently reviewed native D0 checkpoint identities.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-22T04:24:30Z
- **Completed:** 2026-07-22T04:39:35Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Persisted exact typed-catalog bytes for rigid stack/contact, rope-joint, and particle-group construction/append runs with their resolved SHA-256 identities.
- Added a strict manifest carrying catalog/scenario/generator versions, optional seed state, exact solver settings, ordered action and checkpoint logs, pinned upstream provenance, review status, and expected native D0 identity.
- Added a read-only replay boundary that rejects incomplete metadata, seed-only reconstruction, stale versions, unknown actions, noncanonical or hash-drifted bytes, catalog drift, duplicate paths/hashes/IDs, traversal, and symlinks before native world creation.
- Added independent D0 review and tracked-file no-write assertions so the checked corpus cannot silently bless regenerated candidates.

## TDD Evidence

- **RED:** `cargo test -p liquidfun-differential --test catalog_regressions` failed at compile time because `CatalogRegressionErrorKind` and `replay_catalog_regressions` did not exist.
- **GREEN:** The focused suite passes 5/5 across exact repeated replay, independently derived D0 checkpoint hashes, hash/seed/path/version/action/duplicate rejection, and symlink confinement.
- **REFACTOR:** The public replay entrypoint remains small, pure file/catalog validation was extracted from the effectful D0 loop, and path-chain validation was centralized without adding dependencies.

The intentionally failing RED state was not committed because repository policy requires each commit to follow a completely passing ordered Rust gate.

## Task Commits

1. **Task 1: Add content-addressed catalog regression fixtures and replay validation** - `86df789` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `scenarios/catalog/rigid-stack-v1.json` - Exact canonical resolved bytes for the reviewed rigid stack run.
- `scenarios/catalog/joint-rope-v1.json` - Exact canonical resolved bytes for the reviewed rope-joint run.
- `scenarios/catalog/particle-group-v1.json` - Exact canonical resolved bytes for the reviewed particle-group append run.
- `scenarios/regressions/catalog-manifest.json` - Strict content-addressed registry with run, schedule, provenance, and D0 identities.
- `crates/liquidfun-differential/src/fixtures/replay/catalog.rs` - Confined validation, typed catalog rebinding, and repeated native semantic replay.
- `crates/liquidfun-differential/tests/catalog_regressions.rs` - Read-only exact replay, independent digest, malformed authority, and no-clobber coverage.
- `crates/liquidfun-differential/src/fixtures.rs` and `src/fixtures/replay.rs` - Curated replay API export and cohesive replay module routing.

## Decisions Made

- Stored raw canonical resolved bytes with no newline or pretty-print transformation, preserving the exact hash returned by the typed resolver.
- Used the existing strict resolved-scenario decoder and closed action schema rather than introducing a parallel fixture format or permissive JSON value path.
- Required fresh typed-catalog resolution to agree with the tracked bytes while refusing to use that regenerated candidate as execution input.
- Hashed ordered canonical semantic checkpoint records for native D0; a separate integration path serializes the same owned semantic values directly and compares explicit reviewed constants.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Security Verification

- Manifest and fixture reads enforce fixed byte/count limits, regular files, component-by-component symlink rejection, canonical root confinement, reviewed three-component paths, and duplicate path/hash/fixture rejection.
- Strict serde and resolved-scenario decoding reject unknown manifest fields, unknown action variants, unsupported versions, invalid settings, oversized collections, noncanonical bytes, and hash drift.
- All three fixtures and their catalog mappings validate before the first `World` is constructed; no seed-only or regenerated substitution reaches execution.
- Diagnostics expose only bounded failure categories and never echo raw records, filesystem paths, or private native state.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-16's `EXMP-02` and `EXMP-03` mappings are implemented in the immutable regression path and retained in summary frontmatter. Their global requirement checkboxes remain unchanged until the remaining Phase 11 consumers prove the complete shared-catalog scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11-17 can consume exact checked resolved bytes and stable semantic checkpoint identities without reconstructing from a seed or mutable catalog implementation.
- Failure bundles, benchmarks, and later evidence checks can bind to the same manifest/hash authority.
- No blocker remains for the next incomplete Phase 11 plan.

## Self-Check: PASSED

- Confirmed all six created files exist and implementation commit `86df789` is present.
- Confirmed the temporary stdout-only candidate generator was removed before staging.
- Confirmed the three tracked fixture SHA-256 values match the typed resolver identities exactly.
- Confirmed focused catalog regression tests pass 5/5 and focused deny-warnings Clippy passes.
- Confirmed the exact ordered `cargo fmt --all`, full-workspace deny-warnings Clippy, all-targets build, and all-features test gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-16`.
- Confirmed the four fenced pre-existing edits remain unstaged and uncommitted.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
