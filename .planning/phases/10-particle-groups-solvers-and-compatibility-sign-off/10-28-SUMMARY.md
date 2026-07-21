---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "28"
subsystem: particle-differential-evidence
tags: [rust, liquidfun, particle-groups, differential-testing, evidence, replay]

requires:
  - phase: 10-25
    provides: Native public-API Phase 10 execution and semantic capture
  - phase: 10-26
    provides: Pinned C++ oracle execution and semantic capture
  - phase: 10-27
    provides: Closed D0/D1 comparator and named numeric policies
provides:
  - Closed typed evidence bindings for all 22 Phase 10 behaviors and 58 inherited Phase 9 branches
  - Five bounded content-addressed native/oracle scenario families
  - Canonical D0 replay, debug/release equality, strict D1 comparison, and corruption-resistant manifest checks
affects: [10-29, 10-30, 10-31, phase10-compatibility-sign-off, release-evidence]

tech-stack:
  added: []
  patterns:
    - Seal fixture, request, policy, leaf, retained-manifest, and manifest-payload digests independently
    - Bind every public behavior to typed control, activation, interaction, tests, policy, and distinct proof payloads
    - Normalize private oracle caches into one public semantic projection before strict comparison

key-files:
  created:
    - crates/liquidfun-differential/src/rigid_world/phase10/evidence.rs
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/group-construction-and-mutation.jsonl
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/topology-join-split-reactive.jsonl
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/solver-material-flags.jsonl
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/pressure-constraints-and-rigid.jsonl
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/boundary-order-and-inherited.jsonl
  modified:
    - crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs
    - crates/liquidfun-differential/src/rigid_world/phase10/native/capture.rs
    - crates/liquidfun-differential/tests/phase10_corpus.rs
    - tools/reference/src/rigid_world_phase10_capture.hpp
    - tools/reference/src/rigid_world_phase10_operations.hpp

key-decisions:
  - "Seal exactly five short-horizon cases and reject any manifest, request, leaf, policy, or retained-evidence digest drift."
  - "Keep cross-engine group depth absent because the pinned C++ public API exposes no depth buffer, while retaining fail-closed tests for required optional-lane disappearance."
  - "Project C++ group aggregates, weights, witnesses, live bodies, and empty-shell destruction onto the same stable semantic identities exposed by the native adapter."

patterns-established:
  - "Closed proof topology: metadata repair cannot conceal semantic substitution, aliasing, unknown leaves, unsafe paths, or out-of-range witness indices."
  - "Corpus closure: every Phase 10 behavior must appear in a non-control witness while the exact Phase 9 manifest digest preserves all inherited evidence."

requirements-completed: [PART-09, PART-10, PART-11, PART-12, PART-13, PART-18, TEST-01, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T11:11:29Z

duration: 2h 56m
completed: 2026-07-21
---

# Phase 10 Plan 28: Closed Semantic Differential Corpus Summary

**Five content-addressed scenario families now prove every Phase 10 public behavior and every retained Phase 6-9 branch through deterministic native/oracle replay and strict cross-build comparison.**

## Performance

- **Duration:** 2h 56m
- **Started:** 2026-07-21T08:15:27Z
- **Completed:** 2026-07-21T11:11:29Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- Added a versioned typed evidence contract covering all 22 Phase 10 behavior leaves plus the exact 58-branch inherited Phase 9 inventory, with implementation, test, witness-role, policy, index-bound, and distinct payload requirements.
- Sealed exactly five bounded cases with independent fixture and canonical-request digests plus policy, leaf, retained-manifest, and manifest-payload hashes.
- Proved native D0 replay, oracle D0 response-byte replay, oracle debug/release equality, strict D1 semantic comparison, identical request authority, and complete non-control witness coverage.
- Added corruption tests for missing, duplicate, unknown, aliased, out-of-range, private-pass, unsafe-path, open-path, and recomputed-metadata substitution attacks.
- Recorded immutable per-policy calibration rationale and boundary-test authority without widening any comparator threshold.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define a closed per-leaf evidence contract** - `60ff26c` (feat)
2. **Task 2: Build and seal the bounded two-engine corpus** - `dda076a` (feat)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase10/evidence.rs` - Defines the closed leaf inventory, typed proof bindings, path rules, limits, and corruption-resistant validation.
- `crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs` - Publishes immutable calibration records for every named policy path.
- `crates/liquidfun-differential/src/rigid_world/phase10/native/capture.rs` - Uses the common public cross-engine depth surface.
- `crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json` - Seals the five case identities and all content digests.
- `crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/*.jsonl` - Store bounded case recipes for group mutation, topology, materials, pressure/rigid interaction, and boundary/inherited behavior.
- `crates/liquidfun-differential/tests/phase10_corpus.rs` - Builds canonical requests, verifies closure and digests, executes native and two oracle modes, and proves replay plus D1 coverage.
- `tools/reference/src/rigid_world_phase10_capture.hpp` - Projects stable semantic members, live aggregates, weights, and snapshot-local derived witnesses.
- `tools/reference/src/rigid_world_phase10_operations.hpp` - Preserves live rigid identities and empty retained-group destruction semantics in Phase 10 result projection.

## Decisions Made

- The exact retained Phase 9 manifest hash is corpus authority; inherited evidence is referenced rather than copied or relabeled.
- Cross-engine depth remains absent because upstream exposes it only through private storage. Native-only depth behavior stays covered by curated view tests, and the comparator still rejects disappearance whenever a lane is required.
- Pressure/body interaction uses one boundary-positioned barrier particle to avoid ambiguous fixture-interior contact geometry while preserving barrier, rigid-group, and live-body evidence.
- Zombie and wall activation is captured before compaction, followed by retained-empty and explicit-destroy observations, so both flag activation and lifecycle order remain executable proof.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired oracle semantic projection exposed by the sealed corpus**

- **Found during:** Task 2 two-engine execution
- **Issue:** Pinned C++ group caches included split clones or stale pre-compaction members, derived witnesses persisted across inspections, Phase 9-compatible mixed state omitted live rigid bodies, and empty retained-group destruction left a semantic shell. These adapter artifacts produced strict mismatches even when public native and upstream behavior agreed.
- **Fix:** Recomputed group aggregates over stable semantic members, excluded nonsemantic contact weight, made derived witnesses snapshot-local, emitted live bodies in declaration order, and projected empty-shell destruction into the semantic event/group record.
- **Files modified:** `tools/reference/src/rigid_world_phase10_capture.hpp`, `tools/reference/src/rigid_world_phase10_operations.hpp`
- **Verification:** Rebuilt debug and release oracles; exact Plan 26 oracle tests pass 5/5 and exact Plan 28 corpus tests pass 8/8.
- **Committed in:** `dda076a`

**2. [Rule 3 - Blocking] Normalized the inaccessible depth lane to the public adapter surface**

- **Found during:** Task 2 strict group comparison
- **Issue:** Native curated views expose group depth, but the pinned C++ public API does not. Comparing native `Some` against oracle `None` would either fail every solid case or require an unsafe/private oracle dependency.
- **Fix:** Native differential capture emits `None` on the cross-engine surface, with an explicit calibration rationale and a regression test proving optional-lane disappearance still fails whenever the expected observation requires it.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world/phase10/native/capture.rs`, `crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs`, `crates/liquidfun-differential/tests/phase10_corpus.rs`
- **Verification:** Focused depth fail-closed test and complete strict D1 corpus pass.
- **Committed in:** `dda076a`

**Total deviations:** 2 auto-fixed blocking adapter issues.
**Impact on plan:** Both repairs preserve strict policy and process isolation; no comparator tolerance was widened and no production crate gained a C++ dependency.

## Issues Encountered

- macOS delayed first launches of newly linked Rust test executables. All commands were allowed to finish, and the required full all-features suite passed.
- Initial pressure geometry used fixture-interior points whose body-contact projection differed across engines. A bounded boundary-positioned witness removed the ambiguity without reducing required behavior coverage.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The complete Phase 10 corpus is deterministic, bounded, content-addressed, and ready for compatibility report generation and sign-off.
- Every Phase 10 behavior and retained Phase 9 family has executable evidence; no leaf, policy, payload, or request can drift silently.
- No blockers remain.

## Self-Check: PASSED

- Confirmed Task 1 commit `60ff26c` and Task 2 commit `dda076a` exist and are atomic.
- Confirmed both oracle presets were rebuilt from pinned revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- Confirmed `cargo test -p liquidfun-differential --all-features --test phase10_oracle` passes 5/5.
- Confirmed `cargo test -p liquidfun-differential --all-features --test phase10_corpus -- --nocapture` passes 8/8.
- Confirmed the implementation commits were preceded by the exact mandatory Rust gates: format, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests.
- Confirmed `.planning/config.json`, `.planning/agent-history.json`, and `.planning/current-agent-id.txt` were not staged or committed.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-21*
