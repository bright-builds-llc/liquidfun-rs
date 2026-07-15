---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "17"
subsystem: testing
tags: [phase9-v1, compatibility, github-actions, evidence, inventory]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "16"
    provides: human-approved exact-ref canonical and sanitizer artifacts
provides:
  - independently revalidated Phase 9 D1 evidence authority
  - exact immutable evidence citations for four scoped compatibility rows
  - fail-closed incomplete, substituted, and Phase 10 promotion validation
  - deterministically regenerated compatibility report
affects: [phase-10, compatibility-ledger, evidence-promotion]
tech-stack:
  added: []
  patterns:
    - exact remote run and downloaded payload revalidation before ledger mutation
    - all-or-nothing scoped compatibility promotion with immutable authority references
key-files:
  created:
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-17-SUMMARY.md
  modified:
    - TESTING.md
    - reference/compatibility.json
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs
    - COMPATIBILITY.md
key-decisions:
  - "Promote only the two scoped particle public-API rows and the storage/lifecycle and contacts/coupling subsystem rows."
  - "Keep particle groups, pair/triad generation, solver behavior, and the complete Particle source area explicitly unpromoted for Phase 10."
  - "Bind every promoted platform record to the exact approved commit, run, artifacts, API digests, and recomputed trace/manifest digests."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T19:04:40Z
duration: 11min
completed: 2026-07-15
---

# Phase 9 Plan 17: Evidence Promotion Summary

**Freshly downloaded exact-ref canonical and sanitizer artifacts now authorize a fail-closed, four-row Phase 9 D1 compatibility promotion without claiming deferred Phase 10 behavior.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-15T18:53:00Z
- **Completed:** 2026-07-15T19:04:40Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Independently re-queried GitHub run `29439515367`, its jobs, and its artifact API records before any compatibility-ledger edit.
- Freshly downloaded both exact artifacts and revalidated the approved SHA, pinned toolchain/upstream identities, safe relative paths, and all four trace/manifest payload hashes.
- Promoted four scoped compatibility rows with immutable run, commit, artifact, archive-digest, trace-digest, and manifest-digest references.
- Added inventory regressions that reject partial promotion, substituted authority, and any simultaneous Phase 10 group/solver/source-area claim.
- Regenerated and checked `COMPATIBILITY.md` twice, proving byte-idempotent report generation from the authoritative JSON.

## Evidence Revalidation

- **Approved commit:** `a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Run:** `29439515367` (`workflow_dispatch`, `Oracle CI`, `success`)
- **Canonical artifact:** `phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Canonical API digest:** `sha256:f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea`
- **Sanitizer artifact:** `phase9-sanitizer-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Sanitizer API digest:** `sha256:95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f`

The approved SHA remained an ancestor of this checkout and the exact remote `main`. The workflow at that commit contained the Phase 9 dispatch and both artifact contracts. Exactly one required canonical job and one required sanitizer job succeeded, and exactly one unexpired occurrence of each artifact retained the recorded digest. Both downloaded identities matched run, head, job, upstream revision, Rust, Clang, CMake, Ninja, target, and `phase9-v1` policy. `shasum -a 256 -c` recomputed both named payloads in both artifacts, and `cargo xtask provenance check` passed against the pinned submodule.

## Promotion RED/GREEN

- **RED:** The existing inventory validator accepted both a partial four-row promotion and a substituted platform-authority digest; both focused tests failed because generation unexpectedly succeeded.
- **GREEN:** The validator now requires all four scoped rows to carry complete implemented, unit, differential, and exact platform authority evidence together. It rejects noncanonical references and any promoted deferred Phase 10 row. All three negative regressions and the full inventory CLI suite pass.

## Task Commits

1. **Task 1: Independently revalidate canonical and sanitizer evidence** - `ff00133` (docs)
1. **Task 2: Promote authoritative JSON and regenerate compatibility report** - `c71cec0` (feat)

## Files Created/Modified

- `TESTING.md` - Records the immutable run/artifact references, API and payload digests, independent procedure, and bounded D1 claim.
- `reference/compatibility.json` - Promotes only the four scoped Phase 9 rows with implementation, unit, differential, and exact platform evidence.
- `tools/xtask/src/inventory/validation.rs` - Enforces all-or-nothing exact-authority promotion and the Phase 9/10 boundary.
- `tools/xtask/tests/inventory_cli.rs` - Covers partial, substituted, and deferred-Phase-10 rejection.
- `COMPATIBILITY.md` - Deterministically regenerated report showing four promoted rows while group and solver rows remain gaps.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-17-SUMMARY.md` - Records plan execution and evidence.

## Decisions Made

- Coarse source-area and group/solver rows remain unpromoted because the closed Phase 9 corpus intentionally excludes group topology, pair/triad generation, and baseline or flag-driven particle solvers.
- Exact remote identity belongs in `platform_validated`; implementation and unit references remain repository paths, while differential references name the closed corpus, adapter/policy, and execution summary.
- The generated report remains a pure projection of `reference/compatibility.json`; it was never hand-edited.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Initialized the pinned upstream submodule in the isolated worktree**

- **Found during:** Task 1 provenance verification
- **Issue:** The fresh worktree had an uninitialized submodule directory, so provenance resolved the parent repository HEAD instead of the pinned oracle revision.
- **Fix:** Initialized only `third_party/liquidfun` at `7f20402173fd143a3988c921bc384459c6a858f2` and reran provenance.
- **Files modified:** None tracked.
- **Verification:** `cargo xtask provenance check` passed with the pinned revision and Phase 9 witness.
- **Committed in:** No tracked commit required.

**2. [Rule 3 - Blocking] Followed the real `policy` identity field and validation module ownership**

- **Found during:** Task 1 identity verification and Task 2 promotion validation
- **Issue:** The plan's inline automated snippet referred to `policy_profile`, while the workflow, `TESTING.md`, Plan 09-16 summary, and downloaded identity schema all use `policy`. The planned file list also omitted the existing `inventory/validation.rs` module that owns ledger validation.
- **Fix:** Validated `policy == "phase9-v1"` and added the fail-closed authority gate in the existing validation module rather than weakening or duplicating the schema.
- **Files modified:** `tools/xtask/src/inventory/validation.rs`, `tools/xtask/tests/inventory_cli.rs`
- **Verification:** Fresh identity checks, focused RED/GREEN tests, full inventory CLI tests, and inventory generation/check all passed.
- **Committed in:** `c71cec0`

***

**Total deviations:** 2 auto-fixed (2 blocking). **Impact:** Both fixes restored existing repository contracts; neither expanded the Phase 9 claim or changed production physics behavior.

## Issues Encountered

None remain. All evidence, validation, generation, Markdown, and Rust gates pass.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `cargo test -p xtask --test inventory_cli`
- `cargo xtask inventory generate` and `cargo xtask inventory check`, repeated twice
- `cargo xtask provenance check`
- `just markdown-check`
- `git diff --check`

## User Setup Required

None.

## Next Phase Readiness

- Phase 9 is ready for aggregate verification and completion.
- Phase 10 can build the explicitly deferred group topology, pair/triad generation, and particle solver rows without inheriting an overbroad Phase 9 claim.

## Self-Check: PASSED

- Both task commits exist and include `09-17` in their subjects.
- The generated compatibility report exists and is current after two generation/check cycles.
- The four scoped rows are promoted; group, solver, and complete Particle source-area rows remain unpromoted.
- `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/config.json` were not modified.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
