---
phase: 11-examples-headless-tooling-and-testbed
plan: "02"
subsystem: testing-tooling
tags: [semantic-corpus, upstream-discovery, deterministic-snapshot, googletest, testbed]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "01"
    provides: Strict semantic corpus model, terminal review vocabulary, and pinned oracle revision validation
provides:
  - Bounded source-derived discovery of 244 GoogleTest cases, 71 testbed registrations, and 73 examples
  - Canonical 388-item semantic corpus snapshot pinned to the verified upstream gitlink
  - Cargo-only snapshot validation that never reads third_party or invokes native build tools
affects: [phase11-corpus-review, phase11-coverage-mapping, phase11-verification]
tech-stack:
  added: []
  patterns:
    - Verify the parent gitlink and initialized checkout before bounded source discovery
    - Preserve reviewed fields by exact source identity while refusing stale or partial classifications
    - Canonicalize tracked authority entirely in memory for Cargo-only checks
key-files:
  created:
    - reference/upstream-corpus.json
    - tools/xtask/src/inventory/corpus/discovery.rs
    - tools/xtask/src/inventory/corpus/discovery/source.rs
    - tools/xtask/tests/corpus_discovery.rs
  modified:
    - tools/xtask/src/inventory.rs
    - tools/xtask/src/inventory/corpus/model.rs
    - tools/xtask/tests/corpus_model.rs
key-decisions:
  - "Expand parameterized GoogleTest declarations into source-derived semantic cases rather than recording only their macro sites."
  - "Treat discovery-only corpus items as pending review only when all five classification fields are absent; partial terminal outcomes remain invalid."
  - "Recognize source-defined testbed scenarios by a bounded class-local Create factory so indirect scenario inheritance remains discoverable."
patterns-established:
  - "Oracle refresh: verify revision and clean tracked checkout, scan bounded allowlisted sources, merge reviewed fields, validate typed canonical bytes, then atomically rename."
  - "Cargo-only authority check: parse, validate stable source order, re-encode, and compare bytes without consulting the upstream checkout."
requirements-completed: [TEST-03, EXMP-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-21T23:43:08Z
duration: 21 min
completed: 2026-07-21
---

# Phase 11 Plan 02: Pinned Semantic Corpus Discovery Summary

**A verified upstream refresh now produces a byte-stable 388-item semantic authority covering individual GoogleTest cases, authoritative testbed registrations, and source-defined examples, while Cargo-only checks validate the tracked snapshot without the submodule.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-07-21T23:22:43Z
- **Completed:** 2026-07-21T23:43:08Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Added closed `inventory corpus refresh` and `inventory corpus check-snapshot` commands without changing legacy inventory command shapes.
- Discovered 244 concrete GoogleTest identities, including 68 expanded parameterized cases, 71 authoritative `TestEntries.cpp` registrations, 72 testbed scenario sources, and the HelloWorld example.
- Generated a canonical 388-item snapshot pinned to upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`; two verified refreshes produced SHA-256 `685aebcd5c7da12c551b3dfac1ff5350e28683d746b838b13ee76ef96c00eabf` both times.
- Added focused command tests for isolated Cargo-only checking, repeated byte identity, review preservation, stale identities, malformed macros, duplicate registrations, unknown test sources, and revision mismatch.
- Split bounded source tokenization and parsing from checkout/snapshot orchestration to keep both modules below the repository's file-size refactor trigger.

## TDD Evidence

- **RED:** The new command-level test target initially failed because the closed corpus command surface and discovery implementation did not yet exist.
- **GREEN:** The implementation generated the fixture and real pinned snapshots; all eight command tests and sixteen corpus-model boundary tests passed.
- **REFACTOR:** Bounded source parsing moved into `discovery/source.rs`; the same focused suites and 388-item snapshot check remained green.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement semantic discovery and commit the pinned snapshot** - `ae184b6` (feat)

**Plan metadata:** committed with this summary.

## Files Created/Modified

- `reference/upstream-corpus.json` - Canonical source-ordered authority with 388 pending-review semantic identities.
- `tools/xtask/src/inventory.rs` - Closed refresh and Cargo-only snapshot-check command routing.
- `tools/xtask/src/inventory/corpus/discovery.rs` - Revision verification, allowlisted traversal, stable identity generation, review merge, canonical checks, and atomic publication.
- `tools/xtask/src/inventory/corpus/discovery/source.rs` - Bounded tokenizer and parsers for GoogleTest macros, parameter sets, testbed registrations, scenario factories, and HelloWorld.
- `tools/xtask/src/inventory/corpus/model.rs` - All-or-none pending-review support without weakening existing terminal-outcome validation.
- `tools/xtask/tests/corpus_discovery.rs` - Eight isolated repository fixtures covering success, determinism, preservation, and fail-closed behavior.
- `tools/xtask/tests/corpus_model.rs` - Pending-review acceptance and partial-review rejection coverage.

## Decisions Made

- `TEST_P` declarations are expanded through their single supported `ValuesIn` registration so each parameter value receives its own stable semantic identity.
- Testbed registration identity combines the user-visible title with the exact `Factory::Create` symbol from the authoritative registration table; source examples independently retain their header path and class factory.
- Existing classification is preserved only when the stable ID and full source identity still agree. Any stale identity, changed derived ID, or partial classification fails closed.
- Discovery records omit all classification fields until reviewed. Plan 11-28 retains ownership of terminal corpus closure; this plan does not invent dispositions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Allowed explicit pending-review records in the strict corpus model**

- **Found during:** Task 1 snapshot generation
- **Issue:** Plan 11-01's model required every item to have a terminal classification, while Plan 11-02 explicitly forbids silently creating terminal dispositions and Plan 11-28 owns exhaustive classification.
- **Fix:** Permitted only the all-five-fields-absent pending state; retained the complete existing terminal validation and rejected every partial classification combination.
- **Files modified:** `tools/xtask/src/inventory/corpus/model.rs`, `tools/xtask/tests/corpus_model.rs`
- **Verification:** Focused corpus-model tests pass 16/16 and the full warning-denied workspace gate passes.
- **Committed in:** `ae184b6`

**Total deviations:** 1 auto-fixed (1 Rule 3 blocking issue).
**Impact on plan:** This enables truthful discovery output without weakening reviewed terminal outcomes; later exhaustive review remains explicit and machine-checkable.

## Issues Encountered

- The pinned testbed includes `SoupStirrer`, which inherits another scenario rather than `Test` directly. Class-local factory parsing now recognizes that valid indirect inheritance without accepting multiple ambiguous factories.
- The pinned unit-test assets include a non-source directory symlink. Traversal never follows it, and any allowlisted semantic source presented as a symlink is rejected before reading.
- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and were not committed or reverted.

## Security Verification

- Refresh verifies both the parent repository gitlink and initialized submodule HEAD against the pinned lock revision, and refuses tracked upstream modifications.
- Traversal is confined to fixed upstream roots, never follows symlinks, normalizes emitted relative paths, and rejects semantic source symlinks or non-files.
- Source work is bounded to 256 files, 2 MiB per source, 16 MiB total, 600,000 tokens, 256-byte tokens, and 2,048 semantic records.
- Duplicate IDs, duplicate source identities, malformed macros, ambiguous parameter registrations, duplicate testbed titles/factories, unknown test sources, stale snapshot identities, and revision mismatches fail with bounded categories.
- Snapshot publication uses create-new temporary files, `sync_all`, and atomic rename; Cargo-only checking reads only the tracked lock and snapshot.
- Published `liquidfun` dependencies and runtime behavior remain unchanged; no renderer, C++, oracle protocol, FFI, network endpoint, or foreign runtime surface was introduced.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

The frontmatter preserves Plan 11-02's `TEST-03` and `EXMP-01` mappings. Their global requirement checkboxes remain pending until later Phase 11 plans complete corpus classification, compatibility mapping, example execution, and end-to-end verification.

## User Setup Required

None - Cargo-only checking works from the tracked lock and snapshot; refresh requires the already documented pinned upstream checkout.

## Next Phase Readiness

- Plans 11-04 onward can consume stable individual test, registration, and example identities without rescanning C++.
- Plan 11-28 can classify the complete 388-item snapshot and rely on all-or-none terminal review enforcement.
- Global `TEST-03` and `EXMP-01` completion remains deferred to Phase 11 integration and verification.

## Self-Check: PASSED

- Confirmed all four created artifacts, three modified artifacts, and this summary exist.
- Confirmed task commit `ae184b6` exists and excludes all four unrelated shared-tree edits.
- Confirmed repeated refresh SHA-256 identity, 388-item typed snapshot validation, focused tests, exact ordered format, warning-denied Clippy, all-target build, and all-feature tests pass with the required temporary target directory.
- Confirmed the upstream gitlink is unchanged, no known stub prevents this plan's discovery goal, no unplanned threat surface was introduced, and classification remains explicitly assigned to later Phase 11 work.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
