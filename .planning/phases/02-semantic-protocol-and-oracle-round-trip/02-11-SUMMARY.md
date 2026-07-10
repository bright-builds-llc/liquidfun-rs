---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "11"
subsystem: differential-fixture-governance
tags: [rust, fixture-lifecycle, provenance, replay, atomic-promotion, path-confinement]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Native/C++ round-trip execution, typed comparison, first-divergence signatures, and minimized canonical scenarios from Plans 02-06 and 02-10
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Reviewed artifact manifest schema, source notices, and fail-closed provenance validation from Plans 01-04 and 01-05
provides:
  - Confined create-new staging below target/differential/staging
  - Typed replay review with deterministic accepted-artifact diffs and explicit reviewer metadata
  - No-clobber atomic trace/regression promotion with locked atomic manifest replacement
  - Exact serialized regression value, original source, and first-divergence signature retention
affects: [02-12, reference-artifacts, minimized-regressions, provenance, differential-replay]

tech-stack:
  added: []
  patterns: [typed destination derivation, component-wise symlink confinement, candidate-bound explicit review, no-clobber promotion, atomic manifest lock-and-rename]

key-files:
  created:
    - crates/liquidfun-differential/src/fixtures.rs
    - crates/liquidfun-differential/src/fixtures/domain.rs
    - crates/liquidfun-differential/src/fixtures/lifecycle.rs
    - crates/liquidfun-differential/src/fixtures/replay.rs
    - crates/liquidfun-differential/src/fixtures/storage.rs
    - crates/liquidfun-differential/tests/fixture_workflow.rs
    - scenarios/regressions/README.md
  modified:
    - crates/liquidfun-differential/src/lib.rs
    - crates/liquidfun-differential/src/main.rs

key-decisions:
  - "Derive every accepted path from a typed artifact kind and validated scenario ID; fixture commands accept IDs and allowlisted profiles, never destination paths."
  - "Keep generation, replay, explicit review, and promotion distinct; an approved review receipt binds reviewer identity, UTC timestamp, and the complete candidate digest."
  - "Use create-new staging, no-clobber hard-link publication, directory fsync where supported, a create-new manifest lock, and atomic manifest rename to prevent overwrite and partial accepted state."
  - "Bind replay to exact request, trace, report, identity, stderr, scenario, schema/profile, build flags, source, and failure-signature bytes before promotion."

patterns-established:
  - "Fixture module shape: fixtures.rs is the discoverable entrypoint; domain, lifecycle, replay, and storage child modules remain cohesive and below the repository size trigger."
  - "Read-only rule: stage and review mutate only ignored candidate state; accepted reference/scenario paths change only inside explicit promote."

requirements-completed:
  - COMP-05
  - COMP-08
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T10:35:19Z

duration: 29 min
completed: 2026-07-10
---

# Phase 2 Plan 11: Confined Staged Fixture Lifecycle Summary

**Differential traces and minimized regressions now move through bounded staging, typed replay, deterministic review, explicit approval, and no-clobber atomic promotion without allowing checks or caller paths to mutate accepted evidence.**

## Performance

- **Duration:** 29 min
- **Started:** 2026-07-10T10:06:00Z
- **Completed:** 2026-07-10T10:35:19Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added complete candidate bundles with exact request, trace, report, build identity, bounded stderr, canonical scenario, per-file hashes, and a digest over all typed metadata.
- Added replay review that reparses protocol bytes, validates schema/profile/scenario/build identity, compares against the native adapter, retains the same regression signature, and renders a deterministic accepted-artifact diff.
- Added explicit approved/rejected reviewer receipts that bind reviewer identity, UTC timestamp, and unchanged candidate digest without inferring approval from generation success.
- Added derived reviewed-trace and minimized-regression destinations, component-wise canonical/symlink checks, create-new writes, no-clobber publication, filesystem synchronization, a manifest lock, and atomic manifest replacement.
- Added exact `fixture stage`, `fixture review`, and `fixture promote` CLI subcommands with allowlisted IDs/presets/profiles and no caller-provided filesystem destinations.
- Added 13 focused temporary-repository tests for success, read-only review, traversal, symlinks, wrong identity, dirty hashes, changed signatures, missing/oversized candidates, missing review, overwrite, post-review race, and exact regression evidence.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement confined staged fixture generation and promotion** - `cd8e9a8` (`feat`)

## Files Created/Modified

- `crates/liquidfun-differential/src/fixtures.rs` - Discoverable fixture-lifecycle entrypoint and invariant routing map.
- `crates/liquidfun-differential/src/fixtures/domain.rs` - Typed artifact, candidate, review, receipt, manifest, and error domain model.
- `crates/liquidfun-differential/src/fixtures/lifecycle.rs` - Stage, explicit review, and no-clobber promotion transactions.
- `crates/liquidfun-differential/src/fixtures/replay.rs` - Strict candidate revalidation, semantic replay, report binding, and same-signature enforcement.
- `crates/liquidfun-differential/src/fixtures/storage.rs` - Component-wise confinement, bounded create-new I/O, hashing, deterministic diff, fsync, manifest lock, and atomic rename primitives.
- `crates/liquidfun-differential/src/lib.rs` - Exports the private fixture lifecycle API.
- `crates/liquidfun-differential/src/main.rs` - Adds exact ID-based `fixture stage`, `review`, and `promote` dispatch.
- `crates/liquidfun-differential/tests/fixture_workflow.rs` - Temporary-repository security, replay, read-only, race, and promotion coverage.
- `scenarios/regressions/README.md` - Defines exact scenario/source/seed/signature/provenance requirements without claiming a real Phase-2 physics mismatch.

## Decisions Made

- Used typed artifact kinds and scenario IDs as the sole destination authority. Neither the library promotion API nor CLI accepts a destination path.
- Stored a complete candidate digest across every file hash and all identity/version/profile/flag/review fields so post-review metadata or byte changes fail closed.
- Kept the existing Phase-1 manifest schema compatible: promotion emits a normal reviewed record with generator revision, oracle/build identity, flags, notices, and content hash.
- Chose safe no-clobber hard-link publication for accepted bytes because ordinary cross-platform `rename` may overwrite an existing destination; manifest replacement remains an atomic rename protected by a create-new lock.

## Verification Evidence

- TDD RED: the focused target failed on the absent fixture lifecycle API before implementation.
- `cargo test -p liquidfun-differential --test fixture_workflow -- --nocapture` passed all 13 focused lifecycle/security tests.
- Real CLI `fixture stage` and `fixture review` smoke commands succeeded against the canonical empty-world candidate, produced the expected deterministic absent-artifact diff, and left no tracked change.
- Acceptance scans found `canonicalize`, `symlink_metadata`, `create_new`, `rename`, `review_status`, and `FailureSignature` controls and confirmed the regression format requires exact serialized content, original source/seed, and first-divergence signature.
- `git diff --exit-code -- reference scenarios` remained clean after every read-only test and CLI review; focused tests mutate only temporary repositories.
- The required ordered pre-commit sequence passed after the final security fixes: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Full-workspace warning-denied Clippy, all-target build, all-feature tests, and warning-denied rustdoc passed. Workspace tests included all 13 new fixture tests and every prior protocol/differential/xtask suite.
- `cargo xtask provenance check` passed twice with tracked evidence unchanged; `cargo xtask package verify`, `cargo xtask upstream build --preset oracle-debug`, `just --list`, and `just check` passed.
- Direct differential CLI one-shot, two-request reuse, and replay all produced semantic Matches. `cargo xtask differential` is not an available xtask command in the current repository, so the owning private binary was exercised directly.
- A Cargo-only temporary repository copy without `third_party/` passed `cargo xtask check` and package isolation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Split the fixture lifecycle into cohesive child modules**

- **Found during:** Simplification pass after initial GREEN
- **Issue:** The first complete single-file implementation exceeded the repository's 628-line refactor trigger and mixed domain, replay, transaction, and filesystem concerns.
- **Fix:** Kept `fixtures.rs` as the entrypoint and split `domain.rs`, `lifecycle.rs`, `replay.rs`, and `storage.rs`; each production file is now below 450 lines.
- **Files modified:** `crates/liquidfun-differential/src/fixtures.rs`, `crates/liquidfun-differential/src/fixtures/*.rs`
- **Verification:** Warning-denied package/workspace Clippy, focused tests, full workspace tests, rustdoc, and file-length review pass.
- **Committed in:** `cd8e9a8`

**2. [Rule 1 - Bug] Close final digest, report, path, and manifest race gaps**

- **Found during:** Final threat-boundary and diff review
- **Issue:** The initial GREEN digest omitted some version/flag fields, replay did not byte-bind the deterministic report, a tracked parent symlink could be followed before rejection, and concurrent manifest updates needed serialization.
- **Fix:** Bound all typed metadata and flags into the candidate digest, regenerated and compared report bytes during replay, created tracked directories component-by-component, rejected symlink chains before review, and added a create-new manifest lock around atomic replacement.
- **Files modified:** `crates/liquidfun-differential/src/fixtures/lifecycle.rs`, `crates/liquidfun-differential/src/fixtures/replay.rs`, `crates/liquidfun-differential/src/fixtures/storage.rs`
- **Verification:** All 13 focused adversarial tests, package Clippy, acceptance scans, and the final ordered Rust gate pass.
- **Committed in:** `cd8e9a8`

**3. [Rule 1 - Bug] Synchronize stale human-readable GSD progress**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` and `roadmap update-plan-progress 02` returned the correct 84% and 11/14 disk-derived results but left the tracked body progress at 79% and 10/14.
- **Fix:** Updated only the stale human-readable state progress bar and Phase-2 roadmap row to match successful GSD tool results.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** Eleven Phase-2 summaries exist; state reports 16/19 and 84%, and the roadmap reports 11/14.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 3 auto-fixed (1 missing structural safeguard, 2 correctness bug sets)
**Impact on plan:** The implementation fixes make the planned trust boundary auditable and fail closed, while the metadata correction keeps tracked progress internally consistent. No change widens accepted artifact kinds, physics scope, public consumer APIs, or dependencies.

## Issues Encountered

- The planned verification spelling `cargo xtask differential ...` is not registered by the current xtask dispatcher. The equivalent owning `liquidfun-differential` binary one-shot, reuse, and replay commands all passed.
- TDD RED was observed but not committed because repository policy requires the complete Rust gate to pass before every commit.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-12 to stage, review, promote, document, and provenance-check the accepted empty-world trace through the new lifecycle.
- Minimized regression promotion is fully covered with synthetic mismatch evidence while the repository truthfully claims no real Phase-2 physics mismatch.
- No path-confinement, replay, review, signature, overwrite, atomicity, package-isolation, or verification blocker remains.

## Self-Check: PASSED

- All nine task-owned implementation, test, and documentation paths exist.
- Task commit `cd8e9a8` exists and contains only the fixture lifecycle task files.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-11.
- The pre-existing `.planning/config.json` auto-chain change remains unstaged and uncommitted.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
