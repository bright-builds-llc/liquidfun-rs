---
phase: 07-rigid-solver-world-operations-and-ccd
reviewed: 2026-07-13T10:25:54Z
depth: standard
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
  critical: 1
  warning: 11
  info: 1
  total: 13
status: issues_found
---

# Phase 7: Code Review Report

**Reviewed:** 2026-07-13T10:25:54Z  
**Depth:** standard  
**Files:** 67  
**Status:** issues found

## Summary

The Phase 7 implementation has substantial deterministic solver, world-operation, and differential-evidence coverage, and its focused native comparison passes all nine required families. The review nevertheless found one public-API crash/partial-mutation path, one resumable-CCD state-loss path, and several gaps where the evidence workflow can accept, misclassify, or incompletely persist results.

The review applied the repository `AGENTS.md`, `AGENTS.bright-builds.md`, the absence of substantive local overrides, and the relevant architecture, code-shape, testing, verification, and Rust standards. In particular, findings were evaluated against transactional mutation, deterministic semantic identity, closed comparison policy, and fail-closed evidence requirements.

## Critical Issues

### CR-01: Valid custom mass data can panic after partially mutating a body

**Files:** `crates/liquidfun/src/world/body.rs:903`, `crates/liquidfun/src/world/object.rs:1060`  
**Issue:** `BodyMassData` validates its source fields, but `BodyState::apply_mass_state` does not validate the transformed center or derived velocity. For example, a dynamic body positioned at `f32::MAX` and valid mass data whose local center is `f32::MAX` overflows `Transform::apply` to infinity. The method writes mass, center, inertia, and inverse values before `Sweep::new(...).expect(...)` panics at line 921. A different finite-input overflow at lines 922-924 can store a non-finite linear velocity without panicking. The public `set_body_mass_data` call therefore violates its result-based API and the repository's atomic-mutation contract; if the unwind is caught, the world may remain partially mutated.  
**Fix:** Prepare a copied `BodyState` through the checked `with_mass_state` path, validate both the derived center and velocity, and commit only after every calculation succeeds. Widen the public error to a typed mass-mutation error carrying handle and derived-state failures. Add no-panic, no-mutation regressions for transformed-center and velocity overflow.

## Warnings

### WR-01: A failed CCD resume loses its pending checkpoint and repeats discrete work

**Files:** `crates/liquidfun/src/world/continuous.rs:604`, `crates/liquidfun/src/world/step.rs:653`  
**Issue:** `ContinuousStepState::begin_step` destructively `take`s a matching pending key before the resume-time contact and hook phases. Event-capacity or command-capacity failure at `step.rs:786` or `step.rs:805` then returns before `run_continuous_stage` can mark the key pending again. The next identical call is classified as `Fresh`, resets TOI state, and repeats discrete solving/integration, contrary to the D-14 continuation contract and D-15 coherent-resume requirement.  
**Fix:** Retain or peek at the pending key until the continuous stage completes, clearing it only on successful completion or an explicit invalidating mutation. Alternatively, use a guard that restores the key on every pre-continuous error. Add a pending -> hook-limit error -> retry regression that proves discrete motion is not applied twice.

### WR-02: Missing Phase 7 observations can still compare as a match

**Files:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs:242`, `crates/liquidfun-differential/src/rigid_evidence.rs:393`  
**Issue:** Result decoding defaults an omitted `observations` member to an empty list, while declaration validation checks identities, checkpoints, counts, and declaration order but never derives the observation sequence required by the checkpoint's actions. The comparator validates each side and then compares them, so two adapters that omit every Phase 7 `BodyState`, `Step`, `Query`, `RayCast`, and `OriginShift` observation can compare empty-to-empty and return `Match`.  
**Fix:** Derive the exact expected observation kind, action, target, and order for every Phase 7 checkpoint and validate each engine independently before cross-engine comparison. Keep omission legal only for Phase 6 timelines that do not declare Phase 7 observations. Add an all-observations-omitted rejection test for each engine side.

### WR-03: The policy registry claims evidence for observables that are never dispatched

**Files:** `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs:83`, `crates/liquidfun-differential/src/rigid_evidence/phase7.rs:374`  
**Issue:** The closed Phase 7 registry includes warm-start enabled, force-clearing enabled, query/ray directive traces, origin-shift topology, and continuous signed separation. The result observation model and comparator dispatch only cover body state, step outcome, query occurrences, ray hits, and the origin-shift vector. Consequently, policy completeness and witness registration pass for semantic paths that no adapter emits and no comparator evaluates, allowing compatibility rows to be marked evidenced while those behaviors can diverge undetected.  
**Fix:** Add typed observations in both adapters and explicit comparator branches for every registered path, with registry-to-dispatch coverage. Otherwise remove the paths and their witness/evidenced claims until real observations exist.

### WR-04: The C++ oracle's contact identity cache is vulnerable to pointer-reuse ABA

**File:** `tools/reference/src/rigid_world.cpp:119`  
**Issue:** `identity_for` treats a raw `b2Contact*` address as sufficient lifetime identity and immediately returns the cached semantic identity. Action-boundary lifecycle detection likewise compares sets of raw addresses. If LiquidFun destroys a contact and allocates another at the same address before cleanup observes the gap, the new contact inherits the old fixture pair and occurrence. This is reproducible as process-history-sensitive evidence: the fresh one-shot oracle accepts the Phase 7 fixture, while the freshly rebuilt C++ protocol suite fails at `island-checkpoint` with `pinned contact identity disagrees with declaration`.  
**Fix:** Bind identity to a lifecycle-stable generation and semantic endpoints, explicitly retiring an identity before its address can be reused. At minimum, verify normalized fixture/child endpoints before reusing a pointer-keyed entry and treat a mismatch as destruction plus creation. Add allocation-perturbed and reused-adapter regressions that must produce the same trace.

### WR-05: Query and ray rules accept selectors for nonexistent fixture children

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:610`  
**Issue:** Selector validation checks only fixture liveness and selector uniqueness. It does not validate `child_index` against the declared shape's child count. Current Phase 7 circle and polygon fixtures expose only child zero, so a terminate, ignore, or clip rule targeting child greater than zero is accepted, never matches either adapter's callback, silently defaults to continue, and can still compare as `Match`.  
**Fix:** Resolve each selector to its fixture declaration and validate the child index against the shape child count; for the current closed shape set, require zero. Add rejected-boundary cases for both query and ray rules.

### WR-06: Ray comparison incorrectly treats callback order as semantic

**File:** `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:53`  
**Issue:** The comparator exact-compares nonminimum hit identities in callback order and then zips numeric payloads in that order. Phase 7 deliberately leaves traversal callback order unspecified: exhaustive/filtered results are multiplicity-preserving multisets, equal closest hits are sets, and termination is represented by count/completion semantics. Two engines can therefore report the same valid ray result in different traversal orders and be classified as a physics mismatch.  
**Fix:** Select comparison semantics from the ray directive/completion contract. Match identity-plus-numeric records as multiplicity-preserving multisets for exhaustive and filtered casts, retain set comparison for equal minima, and compare only the specified count/status fields for termination. Add reordered-equivalent result tests.

### WR-07: Rigid workflows silently rewrite checked-in policy provenance

**Files:** `tools/xtask/src/differential.rs:601`, `crates/liquidfun-differential/src/rigid_fixtures.rs:329`  
**Issue:** Both compare and staging parse the checked-in request as generic JSON, overwrite `tolerance_profile_sha256` with the current profile hash, and only then perform typed decoding. This executes synthesized bytes that differ from the reviewed fixture and masks stale or tampered policy provenance; the math and collision workflows instead fail closed on a stale hash.  
**Fix:** Decode and validate the unchanged request bytes against the loaded policy hash. Move fixture regeneration to an explicit reviewed update command and make compare/stage reject any mismatch.

### WR-08: The advertised rigid-world minimization command never invokes the reducer

**File:** `tools/xtask/src/differential.rs:560`  
**Issue:** On a physics mismatch the command immediately returns an error report, while on a match it reports that minimization needs a captured signature. No production branch calls `minimize_rigid_world_request`; the CLI coverage only checks argument pass-through to a fake external command. Users therefore cannot execute the documented D-24 minimization workflow.  
**Fix:** On mismatch, retain the exact first-divergence signature, invoke the reducer with a real native/oracle evaluator, and persist the minimized request, completion status, and transform provenance. Cover the internal CLI path with a deterministic mismatch fixture.

### WR-09: A full request can be labeled as a minimized regression

**File:** `crates/liquidfun-differential/src/rigid_fixtures.rs:47`  
**Issue:** `stage_rigid_candidate` always executes the fixed full request, and `rigid_stage_report` accepts any `PhysicsMismatch` for `ArtifactKind::MinimizedRegression`. It does not require a completed minimization result, canonical reduced bytes, preserved signature proof, or transformation provenance. The evidence store can therefore contain artifacts whose label makes a stronger claim than their contents.  
**Fix:** Require a `RigidMinimizationResult` with `Complete` status, exact preserved first-divergence signature, canonical minimized request bytes, and recorded transformations before accepting `MinimizedRegression`. Stage those reduced bytes rather than the fixed request.

### WR-10: Candidate staging exposes partially written evidence

**File:** `crates/liquidfun-differential/src/rigid_fixtures.rs:235`  
**Issue:** Staging creates the final artifact directory and writes each file into it sequentially. A concurrent reader can observe a partial candidate, and a crash can leave that directory behind so a retry returns `CandidateExists`. Cleanup errors on ordinary failure are also discarded. This breaks the transaction semantics expected of promotion evidence.  
**Fix:** Write and fsync every file in a unique sibling temporary directory, fsync that directory, atomically rename it to the final artifact ID, and fsync the parent. Surface cleanup failures with context and add an interrupted-write/retry test.

### WR-11: The expanded accepted fixture leaves the compiled C++ protocol contract stale

**File:** `protocol/fixtures/accepted/rigid-world-request.jsonl:1`  
**Issue:** The accepted request now contains nine witness families, but the compiled C++ self-test at `tools/reference/tests/protocol_tests.cpp:354` still requires exactly two timelines and validates only the Phase 6 entries. Once WR-04 no longer aborts first, this assertion necessarily fails, so the repository-wide test suite still cannot pass and the Phase 7 C++ contract lacks direct family/checkpoint coverage.  
**Fix:** Update the protocol self-test in the same change as the fixture to assert all nine declared families and their required Phase 7 checkpoints. Prefer deriving expected family count/order from the decoded declaration where doing so does not weaken the explicit witness checks.

## Info

### IN-01: Workspace Clippy fails on ambiguous fixture identifier names

**File:** `crates/liquidfun-differential/src/rigid_world.rs:214`  
**Issue:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` fails because `fixture_a_id` and `fixture_b_id` trigger `clippy::similar_names`. This is a naming/verification issue rather than a runtime defect.  
**Fix:** Rename the locals to unambiguous semantic names such as `first_fixture_id` and `second_fixture_id`, then rerun the required workspace lint command.

## Verification Evidence

- `cargo fmt --all -- --check` passed.
- `cargo build --workspace --all-targets --all-features` passed.
- `cargo test -p liquidfun --all-features` passed.
- `cargo test --workspace --all-features` failed in `cpp_protocol_bits_preserve_exceptional_classes`; the isolated test reproduced the C++ `island-checkpoint` identity failure after a fresh oracle configure/build.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` failed only on IN-01.
- `cargo xtask docs check` and `cargo xtask inventory check` passed.
- `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` passed all nine required Phase 7 families with both sides at D2-supported authority.

***

_Reviewed: 2026-07-13T10:25:54Z_  
_Reviewer: the agent (gsd-code-reviewer)_  
_Depth: standard_
