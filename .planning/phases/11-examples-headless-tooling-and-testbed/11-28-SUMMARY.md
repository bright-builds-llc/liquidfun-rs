---
phase: 11-examples-headless-tooling-and-testbed
plan: "28"
subsystem: testing-tooling
tags: [corpus, inventory, differential-evidence, deterministic-reporting]
requires:
  - phase: 11-02
    provides: pinned semantic corpus discovery and strict authority model
  - phase: 11-06
    provides: headless example and testbed evidence foundations
  - phase: 11-23
    provides: typed scenario catalog and public test identifiers
  - phase: 11-27
    provides: exact regression, benchmark, and visual scenario mappings
provides:
  - terminal reviewed outcomes for all 388 upstream semantic corpus items
  - fail-closed joins across discovery, compatibility, catalog, mapping, and review authorities
  - deterministic item-level UPSTREAM-CORPUS.md with zero unresolved rows
affects: [11-29, compatibility-audit, release-evidence]
tech-stack:
  added: []
  patterns: [bounded authority reads, exact cross-ledger joins, generated read-only projections]
key-files:
  created:
    - tools/xtask/src/inventory/corpus/validation.rs
    - tools/xtask/src/inventory/corpus/report.rs
    - tools/xtask/tests/corpus_closure.rs
    - UPSTREAM-CORPUS.md
  modified:
    - reference/upstream-corpus.json
    - tools/xtask/src/inventory.rs
    - tools/xtask/src/inventory/corpus/model.rs
key-decisions:
  - "Treat each TestEntries.cpp registration as authoritative while resolving its factory class through the discovered implementation header."
  - "Classify 221 items as equivalent evidence, 127 as reviewed irrelevance, and 40 as intentional visual-only non-support without broadening parity claims."
  - "Require supported outcomes to resolve exactly one scenario, public test, regression mapping, and compatibility row."
patterns-established:
  - "Corpus closure: validate every authority and join before rendering public claims."
  - "Generated reports: compare exact bytes and reject report drift in the check command."
requirements-completed: [TEST-03, EXMP-01, EXMP-03, EXMP-04, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T15:20:25Z
duration: 24min
completed: 2026-07-22
---

# Phase 11 Plan 28: Upstream Semantic Corpus Closure Summary

**Fail-closed closure of all 388 upstream semantic items with exact cross-ledger evidence and a deterministic zero-unresolved report**

## Performance

- **Duration:** 24 min
- **Started:** 2026-07-22T14:56:48Z
- **Completed:** 2026-07-22T15:20:25Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Classified all 388 discovered GoogleTest, example, and registered testbed semantic items: 221 equivalent evidence, 127 reviewed irrelevance, and 40 intentional visual-only non-support.
- Added fail-closed validation for exact revision, pinned counts, unique identities, source discovery, scenario/test/regression/compatibility joins, reviews, hashes, and eligibility mappings.
- Generated `UPSTREAM-CORPUS.md` directly from validated machine authority with exact totals, item-level evidence links, and zero unresolved rows.
- Added three focused closure tests covering complete byte-stable generation and deliberate unresolved, unknown, duplicate, unmapped, stale, vague, and report-drift failures.

## TDD Evidence

- **RED:** `cargo test -p xtask --test corpus_closure` failed all three new tests because the closure and report commands were not registered.
- **GREEN:** The same test target passed 3/3 after implementing validation, classification, CLI wiring, and deterministic reporting.
- The failing RED state was intentionally not committed because the plan requires all tracked task commits to pass the repository gates.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement cross-ledger closure, classify every item, and generate the report** - `92a18f9` (feat)

## Files Created/Modified

- `tools/xtask/src/inventory/corpus/validation.rs` - Validates terminal outcomes and exact cross-authority joins.
- `tools/xtask/src/inventory/corpus/validation/io.rs` - Performs bounded, regular-file-only authority reads.
- `tools/xtask/src/inventory/corpus/validation/schema.rs` - Defines private strict schemas for joined catalog and mapping authorities.
- `tools/xtask/src/inventory/corpus/report.rs` - Renders the deterministic human projection.
- `tools/xtask/tests/corpus_closure.rs` - Exercises positive closure, deliberate gap classes, and byte stability.
- `reference/upstream-corpus.json` - Stores terminal reviewed outcomes for every corpus row.
- `UPSTREAM-CORPUS.md` - Presents exact totals and all item-level outcomes.
- `tools/xtask/src/inventory.rs` - Registers closure-check and report-generation commands.
- `tools/xtask/src/inventory/corpus/model.rs` - Exposes validated authority data through read-only accessors and typed report labels.

## Decisions Made

- Many semantic cases may resolve to one discovered source path, but corpus IDs and source-derived identities remain unique and pinned.
- The 71 `TestEntries.cpp` registrations resolve through their named factory implementation headers; `Rope.h` and HelloWorld remain explicitly accounted for outside that registration list.
- Supported outcomes require four exact evidence references. Reviewed exclusions require one embedded review reference with a specific non-circular rationale.
- Class-local testbed `Create` declarations are reviewed declaration duplication, while registered scenes lacking dedicated typed scenarios are explicit visual-only non-support rather than unverified equivalence claims.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added CLI wiring and read-only model accessors outside the primary artifact list**

- **Found during:** Task 1 (cross-ledger closure implementation)
- **Issue:** The planned validator and renderer could not be invoked or consume validated corpus fields without inventory command wiring and model accessors.
- **Fix:** Extended `tools/xtask/src/inventory.rs` and `tools/xtask/src/inventory/corpus/model.rs`; split private I/O and schema details into focused validator helpers to keep the core module readable.
- **Files modified:** `tools/xtask/src/inventory.rs`, `tools/xtask/src/inventory/corpus/model.rs`, `tools/xtask/src/inventory/corpus/validation/io.rs`, `tools/xtask/src/inventory/corpus/validation/schema.rs`
- **Verification:** Focused xtask build, Clippy target, closure tests, and both corpus commands pass.
- **Committed in:** `92a18f9`

**Total deviations:** 1 auto-fixed (1 Rule 3)

**Impact on plan:** Required integration only; the corpus authority, claims, and compatibility scope were not broadened.

## Verification

- `cargo fmt --all` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `cargo build --all-targets --all-features` - passed.
- `cargo test --all-features` - passed.
- `cargo build -p xtask --all-targets --all-features` - passed.
- `cargo clippy -p xtask --bin xtask --test corpus_closure --no-deps -- -D warnings` - passed.
- `cargo test -p xtask --test corpus_closure` - passed, 3/3.
- Two complete check/generate cycles were byte-identical; final report SHA-256 is `9693dafbc0a109835618de1f01a681daad9a2aefb9f72a23f2b692c096b8569a`.
- `mdformat --check UPSTREAM-CORPUS.md` - passed.
- `git diff --check` - passed.

## Issues Encountered

- `just markdown-check` still fails on six pre-existing non-GSD Markdown files: `UPSTREAM.md`, `ARCHITECTURE.md`, `standards-overrides.md`, `THIRD_PARTY_NOTICES.md`, `docs/decisions/0001-oracle-selection.md`, and `crates/liquidfun-testbed/CAPABILITY.md`. The new generated report passes its scoped mdformat check.
- An expanded `cargo clippy -p xtask --all-targets --all-features -- -D warnings` additionally reaches pre-existing warnings in `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10/prefix.rs` and `tools/xtask/tests/inventory_cli/phase11.rs`. The mandated repository command and the Plan 28-focused xtask Clippy target both pass.

## Known Stubs

None.

## Threat Review

- Authority inputs are bounded and must be regular non-symlink files.
- Exact revisions, hashes, IDs, mappings, and report bytes fail closed before public claims are emitted.
- No new network, authentication, renderer, C++ runtime, or published-crate surface was introduced.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11-29 can consume a complete item-level authority and deterministic public report with zero unresolved corpus rows.
- No Plan 28 implementation blocker remains; the six repository-wide Markdown baseline failures remain outside this plan's scope.

## Self-Check: PASSED

- All claimed files exist.
- Task commit `92a18f9` exists.
- Corpus closure revalidated at 388 items and zero unresolved rows.
- Summary contains exactly one YAML frontmatter block.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
