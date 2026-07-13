---
phase: 07-rigid-solver-world-operations-and-ccd
reviewed: 2026-07-13T13:53:06Z
depth: standard
iteration: 4
review_kind: manual_post_cleanup
files_reviewed: 71
files_reviewed_list:
  - ARCHITECTURE.md
  - COMPATIBILITY.md
  - README.md
  - TESTING.md
  - crates/liquidfun-differential/src/comparator.rs
  - crates/liquidfun-differential/src/minimizer.rs
  - crates/liquidfun-differential/src/rigid_evidence.rs
  - crates/liquidfun-differential/src/rigid_evidence/base.rs
  - crates/liquidfun-differential/src/rigid_evidence/declaration.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/context.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/observation.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs
  - crates/liquidfun-differential/src/rigid_fixtures.rs
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/src/rigid_world/phase7.rs
  - crates/liquidfun-differential/tests/rigid_fixture_workflow.rs
  - crates/liquidfun-differential/tests/rigid_fixture_workflow/provenance.rs
  - crates/liquidfun-differential/tests/rigid_world.rs
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

# Phase 7: Manual Post-Cleanup Code Review — Iteration 4

**Reviewed:** 2026-07-13T13:53:06Z
**Depth:** standard
**Files:** 71
**Status:** issues found

## Summary

This fresh manual review inspected the prior 68-file Phase 7 scope plus all files changed by cleanup commits `d2039e2`, `64cbc46`, `b04720e`, and `85f68c1`, deduplicated to 71 files at report commit `80a5186`. WR-13, WR-14, and WR-15 are closed end to end, and every earlier iteration-1 through iteration-3 finding remains closed.

WR-12 is improved but not fully closed. The schema and both adapters now expose reached clip evidence, each result is independently checked against its request rules, an unreached declared clip retains exhaustive comparison, and closest/termination regressions pass. However, the boolean records that a clip directive was reached rather than that it strictly narrowed the effective interval. A valid reached `Clip(1.0)` leaves the initial ray exhaustive but selects closest-only comparison, allowing a nonminimum mismatch to be hidden.

The review applied the current repository `AGENTS.md`, `AGENTS.bright-builds.md`, the absence of substantive local overrides, and the managed architecture, code-shape, testing, verification, and Rust standards. The decisive requirements were fail-closed evidence boundaries, exact semantic identity, shared lifecycle/action-window derivation, and documentation that does not overstate emitted or compared behavior.

## Warning

### WR-12: A reached no-op clip is misclassified as effective clipping

**Files:** `crates/liquidfun-differential/src/rigid_world/phase7.rs:425`, `tools/reference/src/rigid_world_phase7_execute.hpp:144`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs:440`, `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:37`

**Issue:** Both adapters set `clipping_applied` for every reached `Clip` directive. Request validation permits finite fractions in the inclusive range `0.0..=1.0`, so `Clip(1.0)` is valid. When it is the first clip, the effective maximum fraction remains `1.0`; the traversal is still exhaustive. Result validation proves only that a hit matched a declared clip rule, and the comparator then uses equal-minimum closest-hit semantics whenever both flags are true. Two independently declaration-valid results can therefore share the same minimum hit, differ in a nonminimum hit, and compare equal after a reached no-op `Clip(1.0)`.

The same missing effective-interval model also leaves cross-adapter behavior underspecified for a later clip fraction greater than the already reduced interval: native Rust rejects expansion as `ClipOutsideCurrentInterval`, while upstream C++ accepts the returned positive fraction and expands its traversal bound.

**Fix:** Track the callback-ordered effective maximum fraction in both adapters. Mark clipping effective only after a strict reduction, reject or consistently define attempted interval expansion on both sides, and independently replay observed hit/rule pairs during result validation to verify the recorded effective-clipping evidence. Add regressions for reached `Clip(1.0)` with a differing nonminimum hit and for multiple clips that attempt to expand a previously reduced interval.

## Cleanup-Finding Disposition

| Finding | Iteration-4 disposition | Evidence |
| --- | --- | --- |
| WR-12 | Partially fixed; warning remains | Typed/schema evidence, both adapters, independent validation, unreached-clip exhaustive behavior, closest sets, and termination count are covered, but reached no-op clips do not prove a strict interval reduction. |
| WR-13 | Fixed | Shared lifecycle replay derives exact declaration-ordered live body and fixture IDs through each checkpoint, including `DestroyBody` fixture cascades; stale same-count identities fail independently on both result sides. |
| WR-14 | Fixed | Result validation and evidence attribution share checkpoint-local action windows; the two-checkpoint regression proves the later action/stage and minimizer-protected prefix. |
| WR-15 | Fixed | Architecture and testing docs describe emitted runtime fields, multiset/set ray policies, explicit termination semantics, and the non-observable directive/separation state without overclaim. |

## Earlier-Finding Reconfirmation

- CR-01 and WR-01 through WR-11 remain closed under unchanged implementation paths and the full workspace/focused regression suites.
- WR-09 provenance remains closed by typed transformation reports, deterministic attempted/accepted reconstruction, shared strict transform application, canonical source reconstruction, and hash-recomputed tamper rejection before review state.
- IN-01 remains closed: workspace all-target/all-feature Clippy passes with warning denial.
- No additional Critical, Warning, or Info issue was found in `d21015d..85f68c1` beyond the residual WR-12 warning above.

## Verification Evidence

- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo build --workspace --all-targets --all-features` passed.
- `cargo test --workspace --all-features` passed.
- `cargo test -p liquidfun-test-protocol --all-features rigid_world` passed 26 selected tests across the crate targets.
- `cargo test -p liquidfun-differential --all-features --test rigid_world` passed 29/29 tests, including WR-12, WR-13, and WR-14 regressions.
- `cargo test -p liquidfun-differential --all-features --test rigid_fixture_workflow` passed 15/15 tests.
- `cargo test -p liquidfun-test-protocol --all-features --lib schema::tests` passed 4/4 byte-stability and closed-schema tests.
- `cargo test -p liquidfun-test-protocol --all-features --test fixtures` passed 11/11 fixture tests.
- `cargo xtask docs check` passed: 12 testing layers and all 5 Phase 7 documentation contracts verified.
- `cargo xtask inventory check` passed: 177 compatibility rows verified.
- `cargo xtask check` passed, including 69-entry package isolation, protocol schema/fixture drift checks, documentation contracts, inventory, upstream identity, and provenance.
- `cargo xtask upstream configure --preset oracle-debug` passed.
- `cargo xtask upstream build --preset oracle-debug` passed.
- `ctest --test-dir target/reference/oracle-debug --output-on-failure --no-tests=error` passed 1/1 test.
- `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` passed all 9 required families at D2-supported authority.
- `cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot` passed all 9 required families at D2-supported authority.
- `git diff --check d21015d..85f68c1` and final `git diff --check` passed.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from the canonical CMake 4.3.3 and Clang 22.1.8 pins. The repository reported those expected noncanonical-tool warnings; the successful comparison and replay are local D2 evidence, not canonical D1 authority.

## Worktree State

No source file was edited and no commit was created. Final `git status --short`:

```text
 M .planning/config.json
 M .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
```

The pre-existing `.planning/config.json` modification was preserved byte-for-byte during this review (`git hash-object`: `621946b2b075747d8342124a8abb2226e77546ad`).

***

_Reviewed: 2026-07-13T13:53:06Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
