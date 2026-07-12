---
phase: 06-minimal-rigid-world-vertical-slice
plan: "15"
subsystem: rigid-contact-admission-evidence
tags: [rust, cpp, contacts, differential, declaration-first]
requires:
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "04"
    provides: Private ordered contact manager and focused rigid-contact lifecycle tests
  - phase: 06-minimal-rigid-world-vertical-slice
    plan: "13"
    provides: Rigid compatibility ledger, generated report, and machine-enforced documentation contracts
provides:
  - Pinned no-dynamic-body contact-admission predicate for every fixture pair
  - Focused overlapping static/kinematic and kinematic/kinematic native regressions
  - Two exact declaration-first Rust/C++ admission witnesses with fixed post-overlap steps
  - Witness-specific compatibility references and fail-closed documentation contracts
affects: [06-16-protocol-contract-closure, phase-07-rigid-solver, compatibility-reporting]
tech-stack:
  added: []
  patterns: [positive dynamic admission, declaration-first branch witnesses, exact witness ledger references]
key-files:
  created:
    - .planning/phases/06-minimal-rigid-world-vertical-slice/06-15-SUMMARY.md
  modified:
    - crates/liquidfun/src/world/contact_manager.rs
    - crates/liquidfun/tests/rigid_contacts.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/tests/rigid_world.rs
    - protocol/fixtures/accepted/rigid-world-request.jsonl
    - tools/reference/src/rigid_world_decode.hpp
    - tools/reference/tests/protocol_tests.cpp
    - reference/compatibility.json
    - tools/xtask/src/docs.rs
    - tools/xtask/tests/docs_contract.rs
key-decisions:
  - "Express pinned ShouldCollide admission positively: at least one owning body must be dynamic before fixture filtering or contact allocation."
  - "Keep both admission branches inside non_colliding_body_fixture_lifecycle, with one separate exact fixed step and zero-evidence checkpoint per overlap."
  - "Cite exact admission witness fragments in the b2ContactManager and contacts-and-filtering differential ledger rows without changing platform authority."
patterns-established:
  - "Admission evidence: author an actual overlap, execute the fixed Phase 6 step, then require zero contacts, manifold points, and events."
  - "Cross-language witness completeness: Rust registry, native observation dispatch, accepted request, C++ registry, C++ protocol test, and docs contract share exact names."
requirements-completed: [RIGD-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T16:43:42Z
duration: 16 min
completed: 2026-07-12
---

# Phase 6 Plan 15: Non-Dynamic Contact Admission Evidence Summary

**Contact admission now rejects every pair without a dynamic body, with separate overlapping static/kinematic and kinematic/kinematic Rust/C++ witnesses bound directly into compatibility evidence.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-12T16:27:00Z
- **Completed:** 2026-07-12T16:43:42Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Replaced static/static-only rejection with the pinned positive requirement that at least one owning body is dynamic before contact creation.
- Added focused Arrange/Act/Assert overlap tests proving static/kinematic and kinematic/kinematic pairs produce no contact transitions, hook events, or solves while the existing static/dynamic contact remains supported.
- Extended `non_colliding_body_fixture_lifecycle` with the exact `static_kinematic_overlap_rejected` and `kinematic_kinematic_overlap_rejected` witnesses, each after its own `0x3c888889`, 8, 3 step and zero-contact/manifold/event checkpoint.
- Made witness and step deletion, contact-permitting expectations, C++ registry weakening, and docs-marker deletion fail closed.
- Updated the `public-api.liquidfun-box2d-box2d-dynamics-b2contactmanager-h` and `subsystem.contacts-and-filtering` rows to cite both exact branch witnesses; generated compatibility status and all platform claims remain unchanged.

## Task Commits

1. **Task 1: Correct admission and add focused overlap regressions** - `23f0681` (`fix`)
1. **Task 2: Make the oracle corpus and ledger prove the missing branch** - `3fc2e33` (`test`)

## Files Created/Modified

- `crates/liquidfun/src/world/contact_manager.rs` - Positive at-least-one-dynamic admission predicate.
- `crates/liquidfun/tests/rigid_contacts.rs` - Focused non-dynamic overlap regression tests.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs` - Two required admission witnesses.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs` - Exact overlap, fixed-step, deletion, and count fail-closed contracts.
- `crates/liquidfun-differential/src/rigid_world.rs` - Native phase-to-witness observation mapping.
- `crates/liquidfun-differential/tests/rigid_world.rs` - Updated corpus checkpoint and semantic-ID mutation contracts.
- `protocol/fixtures/accepted/rigid-world-request.jsonl` - Byte-stable overlapping non-dynamic timeline and checkpoints.
- `tools/reference/src/rigid_world_decode.hpp` - Matching C++ required-witness registry.
- `tools/reference/tests/protocol_tests.cpp` - C++ execution and witness-deletion regressions.
- `reference/compatibility.json` - Exact witness citations for the two promoted contact-manager rows.
- `tools/xtask/src/docs.rs` - Required Phase 6 fixture witness markers.
- `tools/xtask/tests/docs_contract.rs` - Negative tests for either missing marker.

## Decisions Made

- Kept the admission predicate before fixture filtering and allocation, preserving the existing same-body, active, duplicate, and filter gates.
- Reused the existing non-colliding family instead of adding another top-level protocol family.
- Inserted a dedicated body-type checkpoint between the two admission steps so each post-overlap checkpoint carries only its matching rejection witness.
- Retained local results as D2 and left all platform evidence false.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wired the new admission witnesses through native and C++ execution tests**

- **Found during:** Task 2 real debug comparison
- **Issue:** The protocol and accepted request decoded, but the native declaration executor had no phase-to-witness mapping and failed at `nc-static-kinematic-rejected`; dependent native and C++ tests also encoded the earlier five-checkpoint corpus shape.
- **Fix:** Added only the two closed native observation mappings, updated semantic-ID mutation tests to locate actions by ID, raised the expected non-colliding checkpoint count to eight, and added C++ protocol assertions/deletion cases.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world.rs`, `crates/liquidfun-differential/tests/rigid_world.rs`, `tools/reference/tests/protocol_tests.cpp`
- **Verification:** The 12-test differential target passes, CTest passes, and real debug/release comparisons report both required families matched.
- **Committed in:** `3fc2e33`

**Total deviations:** 1 auto-fixed blocking integration gap. **Impact:** Required to make the planned new witness names executable across the existing closed pipeline; no solver, force, island, sleeping, CCD, query, or platform scope was added.

## Issues Encountered

- Existing CMake build directories retained the pre-edit adapter digest and binaries. Reconfiguring plus explicitly rebuilding `liquidfun-reference` and `liquidfun-reference-protocol-tests` refreshed the reviewed targets; subsequent debug/release compares and CTest passed.
- Local CMake 3.27.9 and Apple Clang 21.0.0 remain noncanonical D2 tools by policy.
- The available `mdformat` parser rewrites GSD YAML frontmatter delimiters and also rejects tool-owned `STATE.md`; the summary was restored with standalone top-of-file delimiters and passed GSD summary/lifecycle validation plus `git diff --check`.

## Validation Evidence

- TDD RED: both focused non-dynamic tests failed with admitted contacts and `UnsupportedSolverTopology` before the predicate fix.
- Focused Rust: `cargo test -p liquidfun --test rigid_contacts non_dynamic --all-features` and the complete 10-test target pass.
- Protocol: all 11 rigid-world protocol tests pass, including both-step deletion and contact-permitting mutation rejection.
- Native differential: all 12 rigid-world integration tests pass.
- C++: `liquidfun-reference-protocol` CTest passes with exact witness-deletion rejection.
- Real oracle: debug and release one-shot comparisons each report two required families matched under `phase6-v1`, with native and oracle classified `d2_supported`.
- Docs/ledger: 22 docs-contract tests, inventory generation/check for 177 rows, and all five Phase 6 document contracts pass.
- Before each task commit, the mandatory sequence passed in order: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.
- `git diff --check` passed, and generated `COMPATIBILITY.md` remained byte-identical to the authoritative ledger status.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 06-16 can close the remaining step/action/inertia protocol-contract gaps on top of a truthful contact-admission corpus.
- `non-dynamic-contact-admission` is closed; broader islands, forces, sleeping, CCD, world queries, joints, and platform evidence remain deferred.

## Self-Check: PASSED

- Task commits `23f0681` and `3fc2e33` exist in history.
- Both exact witness names occur in Rust, the accepted request, C++, ledger references, and docs contracts.
- Debug/release comparisons and C++ protocol tests passed after rebuilding reviewed targets.
- No platform row changed to evidenced and Phase 6 was not marked complete.

______________________________________________________________________

_Phase: 06-minimal-rigid-world-vertical-slice_
_Completed: 2026-07-12_
