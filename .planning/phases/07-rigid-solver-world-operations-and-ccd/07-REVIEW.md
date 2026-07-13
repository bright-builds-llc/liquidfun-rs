---
phase: 07-rigid-solver-world-operations-and-ccd
reviewed: 2026-07-13T12:49:00Z
depth: standard
iteration: 3
files_reviewed: 68
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
  - crates/liquidfun-differential/tests/rigid_fixture_workflow/provenance.rs
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
  warning: 4
  info: 0
  total: 4
status: issues_found
---

# Phase 7: Code Review Report — Iteration 3

**Reviewed:** 2026-07-13T12:49:00Z
**Depth:** standard
**Files:** 68
**Status:** issues found

## Summary

Commit `d2c7346` closes WR-09 without introducing a new defect in its four-file diff. The minimization report now uses closed typed deserialization, replay deterministically reconstructs the attempted and accepted transform streams through the same reducer and strict transform application used during minimization, canonical reconstruction must equal the staged source-derived request, and malformed, unrelated, reordered, and excess-duplicate transform reports are rejected before review state is written even when their hashes are recomputed. The focused fixture workflow passes all 15 tests.

All 13 iteration-1 findings are now fixed. The iteration-3 review retains four distinct warnings found while reconfirming the original 68-file scope: declared rather than applied ray clipping can suppress mismatches, checkpoint validation does not prove exact live identities, later checkpoint observations can be attributed to earlier actions, and evidence documentation contradicts the implemented runtime policy.

The review applied the current repository `AGENTS.md`, `AGENTS.bright-builds.md`, the absence of substantive local overrides, and the relevant architecture, code-shape, testing, verification, and Rust standards. The decisive requirements were deterministic semantic identity, fail-closed evidence promotion, independently validated result structure, and documentation that matches verified behavior.

## Warnings

### WR-12: Declared clipping can hide exhaustive-ray mismatches

**File:** `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:38`
**Issue:** The comparator selects closest-hit semantics whenever an action declares any `Clip` rule. A valid clip selector may be live but outside the ray and therefore never invoked; execution is then effectively exhaustive, yet differences in nonminimum hits are ignored. The result records completion and hits, but not whether clipping was applied or the final effective maximum fraction.

**Fix:** Record and independently validate applied clipping or the effective final maximum fraction, then select comparison semantics from execution evidence. Add a regression with a clip target outside the ray and a differing nonminimum hit.

### WR-13: Checkpoint validation accepts wrong live body and fixture identities

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs:453`
**Issue:** Checkpoint validation requires body and fixture snapshots only to form declaration-ordered subsequences and separately checks their counts. It does not prove that the reported IDs are the objects actually live at that checkpoint. After destroying body `A` while `B` remains, a same-sized result containing `A` can pass; if both engines agree on the stale ID, cross-engine comparison can report a match.

**Fix:** Replay create and destroy actions through each checkpoint, derive the exact declaration-ordered live body and fixture IDs, and require exact equality. Add same-count stale or swapped identity regressions for each result side.

### WR-14: Later checkpoint observations can be attributed to earlier actions and stages

**File:** `crates/liquidfun-differential/src/rigid_evidence/phase7/context.rs:57`
**Issue:** Result validation correctly scopes observations to the action interval after the prior checkpoint, but evidence-context lookup scans from the start of the timeline through the current checkpoint. At checkpoint two, observation zero can therefore map to the first earlier observation-emitting action. The resulting failure signature has the wrong action and stage, and minimization can protect the wrong action prefix.

**Fix:** Start observation lookup immediately after the prior checkpoint action, preferably through a shared checkpoint action-window helper. Add a two-checkpoint regression that asserts the second checkpoint's failure action, stage, and minimizer prefix.

### WR-15: Evidence documentation contradicts the implemented runtime policy

**Files:** `ARCHITECTURE.md:316`, `TESTING.md:428`
**Issue:** The documentation says nonminimum ray hits retain order, while exhaustive and filtered hit comparison is a multiplicity-preserving multiset. It also claims directive and signed-separation observables are compared, although those paths were deliberately removed from the closed Phase 7 policy registry and checked-in profile.

**Fix:** Document exhaustive and filtered ray hits as multisets, describe closest-hit sets in terms of execution-resolved clipping semantics, and remove claims for directive and signed-separation observables that are not emitted or compared.

## Prior-Finding Disposition

| Finding | Iteration-3 disposition | Evidence |
| --- | --- | --- |
| CR-01 | Fixed | Custom mass state is prepared and fully validated before world commit; overflow regressions prove no panic and no mutation. |
| WR-01 | Fixed | Matching CCD resume keys survive pre-continuous failures and clear only after completion or invalidation. |
| WR-02 | Fixed | Each result side must contain the exact action-derived Phase 7 observation sequence before comparison. |
| WR-03 | Fixed | Unemitted observables were removed from the closed runtime policy registry and checked-in profile. |
| WR-04 | Fixed for the reviewed contract | The oracle revalidates semantic endpoints before pointer-key reuse and advances pair occurrences on replacement. |
| WR-05 | Fixed | Query and ray selectors are checked against shape child counts. |
| WR-06 | Fixed | The original hit-order defect is fixed; WR-12 is a distinct declaration-versus-execution clipping defect. |
| WR-07 | Fixed | Rigid compare and staging decode unchanged fixture bytes and reject stale policy hashes. |
| WR-08 | Fixed | A captured mismatch invokes the bounded reducer with real oracle/native evaluation and persists its report. |
| WR-09 | Fixed | Closed typed transforms, deterministic stream reconstruction, shared transform application, source reconstruction, and fail-before-review tamper tests close the provenance boundary. |
| WR-10 | Fixed | Candidate files are staged in a unique sibling directory, atomically renamed, parent-synced, and cleaned on interruption. |
| WR-11 | Fixed | The compiled C++ protocol test covers all nine required families and Phase 7 checkpoints. |
| IN-01 | Fixed | Workspace Clippy passes with warning denial. |

## WR-09 End-to-End Evidence

- `RigidScenarioTransform` and the persisted minimization report use closed typed deserialization with unknown fields denied.
- Reducer replay regenerates deterministic candidates, matches every accepted transform against the attempted stream in order, verifies checked offsets, and consumes the complete terminal tail.
- Minimization and replay share `rigid_candidate_transforms` and strict `apply_rigid_scenario_transform` logic.
- Candidate verification decodes the checked-in source request, applies the accepted transforms, and requires canonical reconstructed bytes to equal `request.jsonl`.
- Replay verification completes before `review.toml` is written.
- Positive provenance replay passes. Malformed, unrelated, reordered, and excess-duplicate transform reports recompute both report and candidate hashes and still fail without creating review state.

## Verification Evidence

- Full diff inspection of `d2c7346^..d2c7346` covered `minimizer.rs`, `rigid_fixtures.rs`, `rigid_fixture_workflow.rs`, and the new `rigid_fixture_workflow/provenance.rs`; no cross-language source, accepted request, tolerance profile, or compare/replay path changed.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo build --workspace --all-targets --all-features` passed.
- `cargo test --workspace --all-features` passed.
- `cargo test -p liquidfun-differential --test rigid_fixture_workflow` passed 15/15 tests.
- `git diff --check d2c7346^ d2c7346` passed.
- Iteration 3 did not rerun the explicit C++ configure/CTest or nine-family compare/replay commands because `d2c7346` changed only Rust minimizer, fixture replay, and test files. The fresh iteration-2 evidence remains applicable: CTest passed 1/1 and compare/replay passed all nine families; the iteration-3 workspace test also exercised the compiled C++ round-trip protocol test successfully.
- Final `git diff --check` passed.

## Worktree State

No source file was edited and no commit was created. Final `git status --short`:

```text
 M .planning/config.json
 M .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
```

The pre-existing `.planning/config.json` modification was preserved byte-for-byte during this review (`git hash-object`: `621946b2b075747d8342124a8abb2226e77546ad`).

***

_Reviewed: 2026-07-13T12:49:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
