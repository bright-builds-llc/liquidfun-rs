---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "12"
subsystem: differential-evidence-provenance
tags: [rust, provenance, oracle-trace, manifest-v2, replay, review, atomic-promotion]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Confined staged fixture lifecycle, real Rust/C++ round trips, strict protocol validation, and first-divergence evidence from Plans 02-03 through 02-11
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Immutable upstream identity, source mapping, notices, and fail-closed provenance checks
provides:
  - Strict manifest-v2 trace and regression provenance variants
  - Reviewed replay-validated real oracle-debug empty-world trace
  - Read-only scenario, policy, identity, payload, notice, source-map, and review validation
  - Exact JSONL capture from the supervised oracle for fixture staging
affects: [02-13, 02-14, differential-evidence, regression-fixtures, provenance, release-audit]

tech-stack:
  added: []
  patterns: [strict raw-to-variant provenance parsing, exact supervised JSONL capture, source-map-bound evidence, read-only byte-idempotence]

key-files:
  created:
    - reference/artifacts/traces/empty-world-v1.jsonl
    - tools/xtask/src/provenance/artifact.rs
    - tools/xtask/src/provenance/artifact/trace.rs
    - crates/liquidfun-differential/src/supervisor/capture.rs
    - crates/liquidfun-differential/src/supervisor/profile.rs
  modified:
    - reference/artifacts/manifest.toml
    - reference/source-map.toml
    - tools/xtask/src/provenance.rs
    - tools/xtask/tests/provenance_cli.rs
    - crates/liquidfun-differential/src/fixtures/domain.rs
    - crates/liquidfun-differential/src/fixtures/lifecycle.rs
    - crates/liquidfun-differential/src/fixtures/replay.rs
    - crates/liquidfun-differential/src/fixtures/storage.rs
    - crates/liquidfun-differential/src/main.rs
    - crates/liquidfun-differential/src/supervisor.rs
    - crates/liquidfun-differential/tests/fixture_workflow.rs

key-decisions:
  - "Parse manifest-v2 records into a deny-unknown-fields raw boundary, then require exactly one complete trace or regression variant before validation."
  - "Stage reviewed traces from the supervisor's exact validated handshake-plus-trace JSONL bytes rather than the synthetic protocol fixture."
  - "Bind accepted evidence to source-map presence, request/canonical-scenario hashes, all four versions, policy hash, oracle/adapter/build identity, compiler/target/flags, payload hash, notices, reviewer, UTC timestamp, and reviewed status."
  - "Keep provenance/replay checks strictly read-only and prove byte identity across repeated command runs."

patterns-established:
  - "Artifact provenance: every accepted record is content-, scenario-, policy-, build-, source-, notice-, and review-complete before it can pass."
  - "Reviewed trace capture: the same typed supervisor that validates oracle output retains exact newline-complete bytes for the Plan-11 stage/review/promote lifecycle."

requirements-completed:
  - COMP-05
  - COMP-08
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T11:07:58Z

duration: 28 min
completed: 2026-07-10
---

# Phase 2 Plan 12: Reviewed Empty-World Provenance Summary

**A real oracle-debug empty-world trace now passes strict manifest-v2 provenance, deterministic replay, explicit review, source mapping, and no-clobber promotion while every check remains byte-for-byte read-only.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-07-10T10:40:15Z
- **Completed:** 2026-07-10T11:07:58Z
- **Tasks:** 1
- **Files modified:** 16

## Accomplishments

- Versioned artifact provenance to schema 2 with complete trace/regression variants and 19 command-level acceptance/negative tests covering missing, unknown, duplicate, wrong, unreviewed, notice-free, and unsafe-path records.
- Captured the exact real AppleClang/Darwin `oracle-debug` handshake and semantic trace through the existing process supervisor, then staged, replayed, reviewed, and promoted it only through Plan 11's lifecycle.
- Recorded request/canonical-scenario hashes, independent versions, tolerance identity, pinned oracle, adapter and build identities, compiler, target, flags, trace payload hash, notices, source mapping, explicit reviewer, UTC review timestamp, and reviewed status.
- Proved repeated provenance and replay commands preserve every owned source, manifest, source-map, and trace byte exactly.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend strict provenance and promote the reviewed empty-world trace** - `4663498` (`feat`)

## Files Created/Modified

- `reference/artifacts/traces/empty-world-v1.jsonl` - Real reviewed `oracle-debug` handshake and semantic trace.
- `reference/artifacts/manifest.toml` - Manifest-v2 record with complete trace provenance and review metadata.
- `reference/source-map.toml` - Pinned-source derivation and notice mapping for the promoted trace.
- `tools/xtask/src/provenance.rs` and `tools/xtask/src/provenance/artifact/` - Strict source-map-bound artifact and JSONL validators.
- `tools/xtask/tests/provenance_cli.rs` - Complete trace/regression acceptance plus fail-closed negative coverage.
- `crates/liquidfun-differential/src/fixtures/` - Manifest-v2 candidate replay and promotion metadata.
- `crates/liquidfun-differential/src/supervisor.rs` and `supervisor/` - Exact validated JSONL capture and named session-profile modules.
- `crates/liquidfun-differential/src/main.rs` - Real supervised oracle capture during fixture staging.
- `crates/liquidfun-differential/tests/fixture_workflow.rs` - Manifest-v2 lifecycle and exact destination assertions.

## Decisions Made

- Used a strict raw manifest record with mutually exclusive optional variant fields, followed immediately by conversion to a complete trace or regression domain variant. This retains flat readable TOML while rejecting ambiguous states.
- Kept the promoted trace platform-truthful: compiler, target, flags, adapter digest, and build identity describe the actual reviewed local `oracle-debug` build rather than canonical portability values.
- Recorded `codex-gsd-executor` as the explicit automated reviewer in this autonomous lifecycle instead of implying unperformed human review.
- Kept generator revision `a9b3bd8f9adf093bfd93a849748861b8a11c68b3`, the committed Plan-11 lifecycle revision from which the candidate-generation workflow began.

## Verification Evidence

- TDD RED: the first complete manifest-v2 trace test failed against the schema-v1 validator before implementation.
- `cargo test -p xtask --test provenance_cli` passed all 19 strict acceptance and negative tests.
- `cargo test -p liquidfun-differential --test fixture_workflow` passed all 13 lifecycle, replay, confinement, review, race, and no-clobber tests.
- `cargo xtask provenance check` reports one reviewed artifact at oracle revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- `cargo run -p liquidfun-differential --bin liquidfun-differential -- replay --scenario empty-world --preset oracle-debug --session-profile one-shot` returned a semantic Match.
- Repeated provenance/replay commands left SHA-256 inventories of all owned files byte-identical.
- The required ordered pre-commit sequence passed: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Full-workspace warning-denied Clippy, all-target build, all-feature tests, warning-denied rustdoc, package isolation, provenance, `just check`, and real replay passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Upgraded Plan 11's lifecycle to emit manifest-v2 evidence from real oracle bytes**

- **Found during:** Task 1 pre-implementation inspection
- **Issue:** The Plan-11 lifecycle still read/wrote schema 1 and the CLI staged the synthetic protocol trace, so it could neither create the required v2 record nor promote the real `oracle-debug` output.
- **Fix:** Extended candidate metadata, replay, manifest writing, supervised exact-byte capture, CLI staging, and lifecycle tests without adding caller-provided paths or weakening confinement.
- **Files modified:** `crates/liquidfun-differential/src/fixtures/`, `src/main.rs`, `src/supervisor.rs`, `src/supervisor/`, `tests/fixture_workflow.rs`
- **Verification:** All 13 lifecycle tests, full workspace gates, real stage/review/promote, and replay pass.
- **Committed in:** `4663498`

**2. [Rule 1 - Bug] Aligned typed trace promotion with the planned versioned destination**

- **Found during:** First real promotion receipt review
- **Issue:** Plan 11 derived `empty-world.jsonl`, while Plan 02-12 owns `empty-world-v1.jsonl`; leaving the mismatch would break the plan contract and source map.
- **Fix:** Versioned the derived reviewed-trace filename in both review and promotion paths, updated no-clobber tests, removed the incorrect first local promotion, and re-promoted the unchanged reviewed candidate through the lifecycle.
- **Files modified:** `crates/liquidfun-differential/src/fixtures/lifecycle.rs`, `crates/liquidfun-differential/src/fixtures/storage.rs`, `crates/liquidfun-differential/tests/fixture_workflow.rs`
- **Verification:** Final promotion receipt names `reference/artifacts/traces/empty-world-v1.jsonl`; manifest, source map, provenance check, and replay agree.
- **Committed in:** `4663498`

**3. [Rule 2 - Missing Critical] Split touched validators and supervisor below repository size triggers**

- **Found during:** Simplification pass
- **Issue:** The initial strict artifact validator reached 843 lines, and exact capture pushed the touched supervisor to 669 lines.
- **Fix:** Split trace decoding/hash validation into `provenance/artifact/trace.rs` and capture/profile types into `supervisor/` child modules. Final parent files are 565 and 624 lines.
- **Files modified:** `tools/xtask/src/provenance/artifact.rs`, `tools/xtask/src/provenance/artifact/trace.rs`, `crates/liquidfun-differential/src/supervisor.rs`, `crates/liquidfun-differential/src/supervisor/capture.rs`, `crates/liquidfun-differential/src/supervisor/profile.rs`
- **Verification:** Warning-denied full-workspace Clippy and rustdoc pass; focused and workspace tests remain green.
- **Committed in:** `4663498`

**4. [Rule 1 - Bug] Synchronized stale human-readable GSD progress**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` and `roadmap update-plan-progress 02` returned the correct 89% and 12/14 disk-derived results but left the tracked body progress at 84% and 11/14.
- **Fix:** Updated only the stale human-readable state progress bar and Phase-2 roadmap row to match the successful GSD tool results.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** Twelve Phase-2 summaries exist; state frontmatter reports 17/19 and 89%, and the roadmap reports 12/14.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 4 auto-fixed (1 blocking lifecycle gap, 2 correctness bugs, 1 structural safeguard)
**Impact on plan:** Every deviation was required to make the planned lifecycle truthful, strict, and maintainable. No public engine API, physics scope, dependency, caller path authority, or accepted artifact class was added.

## Issues Encountered

- The plan's short `cargo run -p liquidfun-differential -- replay ...` spelling is ambiguous because the package also contains the test helper binary. The owning binary was invoked explicitly with `--bin liquidfun-differential`; replay returned Match.
- The local CMake 3.27.9 and AppleClang 21 tool identities differ from the documented canonical portability versions. The trace truthfully records the actual reviewed platform identity, and the existing upstream tooling emitted only its expected version warnings.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-13 documentation and workflow integration to reference one real, reviewed, replayable trace.
- Strict trace and regression provenance variants are available for later minimized mismatches without permitting unchecked or hand-edited evidence.
- No provenance, replay, identity, review, path-confinement, atomic-promotion, source-map, or read-only verification blocker remains.

## Self-Check: PASSED

- All 16 task-owned source, test, metadata, and trace paths exist.
- Task commit `4663498` exists and excludes the pre-existing `.planning/config.json` change.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-12 exactly.
- Repeated provenance and replay checks left every hashed owned file byte-identical.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
