---
phase: 07-rigid-solver-world-operations-and-ccd
reviewed: 2026-07-13T12:06:54Z
depth: standard
iteration: 2
files_reviewed: 67
files_reviewed_list:
  - ARCHITECTURE.md
  - COMPATIBILITY.md
  - README.md
  - TESTING.md
  - crates/liquidfun-differential/src/comparator.rs
  - crates/liquidfun-differential/src/minimizer.rs
  - crates/liquidfun-differential/src/rigid_evidence.rs
  - crates/liquidfun-differential/src/rigid_evidence/base.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/context.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/observation.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs
  - crates/liquidfun-differential/src/rigid_fixtures.rs
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/tests/rigid_fixture_workflow.rs
  - crates/liquidfun-differential/tests/support/phase7_comparator.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
  - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
  - crates/liquidfun/src/arena.rs
  - crates/liquidfun/src/collision/broad_phase.rs
  - crates/liquidfun/src/collision/tree.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/rigid_differential.rs
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/body.rs
  - crates/liquidfun/src/world/body/control.rs
  - crates/liquidfun/src/world/config.rs
  - crates/liquidfun/src/world/contact.rs
  - crates/liquidfun/src/world/contact_manager.rs
  - crates/liquidfun/src/world/contact_solver.rs
  - crates/liquidfun/src/world/contact_solver/toi.rs
  - crates/liquidfun/src/world/continuous.rs
  - crates/liquidfun/src/world/continuous/event.rs
  - crates/liquidfun/src/world/continuous/tests.rs
  - crates/liquidfun/src/world/island.rs
  - crates/liquidfun/src/world/island/toi.rs
  - crates/liquidfun/src/world/object.rs
  - crates/liquidfun/src/world/origin.rs
  - crates/liquidfun/src/world/proxy.rs
  - crates/liquidfun/src/world/query.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/src/world/step/continuous.rs
  - crates/liquidfun/tests/rigid_body_controls.rs
  - crates/liquidfun/tests/rigid_ccd.rs
  - crates/liquidfun/tests/rigid_ccd_selection.rs
  - crates/liquidfun/tests/rigid_contact_solver.rs
  - crates/liquidfun/tests/rigid_island_order.rs
  - crates/liquidfun/tests/rigid_island_solver.rs
  - crates/liquidfun/tests/rigid_origin_shift.rs
  - crates/liquidfun/tests/rigid_sleeping.rs
  - crates/liquidfun/tests/rigid_world_config.rs
  - crates/liquidfun/tests/rigid_world_queries.rs
  - protocol/fixtures/accepted/rigid-world-request.jsonl
  - protocol/schemas/scenario-v1.schema.json
  - protocol/schemas/trace-v1.schema.json
  - reference/compatibility.json
  - tools/reference/src/rigid_world.cpp
  - tools/reference/src/rigid_world.hpp
  - tools/reference/src/rigid_world_phase7_execute.hpp
  - tools/xtask/src/differential.rs
  - tools/xtask/src/docs.rs
  - tools/xtask/tests/differential_cli.rs
  - tools/xtask/tests/docs_contract.rs
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 7: Code Review Report — Iteration 2

**Reviewed:** 2026-07-13T12:06:54Z
**Depth:** standard
**Files:** 67
**Status:** issues found

## Summary

The fix pass closes 12 of the 13 iteration-1 findings. The public custom-mass mutation is transactional, CCD resume checkpoints survive pre-continuous failures, Phase 7 observations and selector bounds fail closed, the policy registry matches comparator dispatch, C++ contact identity is stable across the covered reuse cases, ray comparison follows the declared callback semantics, checked-in policy provenance is immutable, the production minimizer is reachable, candidate publication is atomic, the C++ protocol test covers all nine families, and workspace Clippy is clean.

WR-09 is only partially fixed. Initial staging now accepts minimized regressions only from a complete typed `RigidMinimizationResult`, but replay does not independently prove that the recorded accepted transforms reconstruct the candidate request. Because replay is the promotion trust boundary, this leaves one warning open.

The review applied the current repository `AGENTS.md`, `AGENTS.bright-builds.md`, the absence of substantive local overrides, and the relevant architecture, code-shape, testing, verification, and Rust standards. The decisive requirements were transactional mutation, deterministic semantic identity, closed comparison policy, and fail-closed evidence promotion.

## Warning

### WR-09: Minimized-regression replay does not validate transformation provenance

**File:** `crates/liquidfun-differential/src/rigid_fixtures.rs:610`
**Issue:** Candidate staging is now sound: it requires a complete, reduced `RigidMinimizationResult` and persists the typed reducer's attempted and accepted transforms. Candidate replay weakens those transforms to `Vec<serde_json::Value>` and, at lines 643-655, checks only that both arrays are nonempty alongside source/request hashes and the reproduced failure signature. It never decodes the transforms as `RigidScenarioTransform`, verifies that accepted transforms are a valid ordered subset of attempts, or reapplies them to the checked-in source request. A candidate can therefore contain arbitrary nonempty transform JSON, recompute its self-declared metadata hashes, and pass replay if its independently constructed request still reproduces the signature. The promotion gate consequently proves a reduced same-signature request, but not the recorded minimization provenance claimed by the artifact.

**Fix:** Deserialize a closed typed transformation report, validate the attempted/accepted relationship, and sequentially reapply accepted transforms through the same strict request decoder used by minimization. Require the reconstructed canonical bytes to equal `request.jsonl`. Add review/replay regressions that recompute candidate metadata after replacing the transforms with malformed, valid-but-unrelated, and wrong-order records; all must fail before review state is written.

## Prior-Finding Disposition

| Finding | Iteration-2 disposition | Evidence |
| --- | --- | --- |
| CR-01 | Fixed | Custom mass state is prepared and fully validated before world commit; overflow regressions prove no panic and no mutation. |
| WR-01 | Fixed | Matching CCD resume keys are retained through hook-limit failure and cleared only after confirmed completion or invalidation. |
| WR-02 | Fixed | Each result side must contain the exact action-derived Phase 7 observation sequence before comparison. |
| WR-03 | Fixed | Unemitted observables were removed from the closed policy registry and checked-in profile. |
| WR-04 | Fixed for the reviewed contract | The oracle revalidates semantic endpoints before pointer-key reuse, advances pair occurrences on replacement, and passes repeated-adapter protocol coverage. |
| WR-05 | Fixed | Query and ray selectors are checked against shape child counts. |
| WR-06 | Fixed | Exhaustive/filtered ray hits compare as multisets, closest hits as equal-minimum sets, and termination by completion/count semantics. |
| WR-07 | Fixed | Rigid compare and staging decode unchanged fixture bytes and reject a stale policy hash. |
| WR-08 | Fixed | A captured mismatch now invokes the bounded reducer with real oracle/native evaluation and persists its report. |
| WR-09 | Partially fixed; warning remains | Staging requires a complete typed result, but replay does not reconstruct the request from recorded transforms. |
| WR-10 | Fixed | Files are synced in a unique sibling directory, atomically renamed, parent-synced, and cleaned on interruption. |
| WR-11 | Fixed | The compiled C++ protocol test asserts all nine families and seven Phase 7 checkpoints; CTest passes. |
| IN-01 | Fixed | Workspace Clippy passes with warning denial. |

## Verification Evidence

- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo build --workspace --all-targets --all-features` passed.
- `cargo test --workspace --all-features` passed, including the custom-mass overflow regressions, CCD resume regression, observation/selector/ray comparator regressions, rigid fixture workflow, Rust protocol tests, xtask tests, and doctests.
- `cargo xtask upstream configure --preset oracle-debug` passed against upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- `cargo xtask upstream build --preset oracle-debug` passed.
- `ctest --test-dir target/reference/oracle-debug --output-on-failure --no-tests=error` passed 1/1 test.
- `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` passed all nine required families with both engines at D2-supported authority.
- `cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot` passed all nine required families with both engines at D2-supported authority.
- `git diff --check` passed.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from the canonical CMake 4.3.3 and Clang 22.1.8 pins; the repository wrapper reported those expected noncanonical-tool warnings while all requested local checks passed.

***

_Reviewed: 2026-07-13T12:06:54Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
