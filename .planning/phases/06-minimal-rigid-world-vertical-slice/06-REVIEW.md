---
phase: 06-minimal-rigid-world-vertical-slice
reviewed: 2026-07-12T07:39:31Z
depth: standard
files_reviewed: 66
files_reviewed_list:
  - .github/workflows/ci.yml
  - .github/workflows/oracle.yml
  - ARCHITECTURE.md
  - COMPATIBILITY.md
  - README.md
  - TESTING.md
  - crates/liquidfun-differential/native-math-sources.txt
  - crates/liquidfun-differential/src/failure_bundle.rs
  - crates/liquidfun-differential/src/main.rs
  - crates/liquidfun-differential/src/minimizer.rs
  - crates/liquidfun-differential/src/rigid_evidence.rs
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/src/supervisor.rs
  - crates/liquidfun-differential/src/supervisor/rigid_world.rs
  - crates/liquidfun-differential/tests/rigid_world.rs
  - crates/liquidfun-test-protocol/src/scenario.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
  - crates/liquidfun-test-protocol/src/schema.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
  - crates/liquidfun-test-protocol/src/schema/tests.rs
  - crates/liquidfun-test-protocol/src/tolerance.rs
  - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/rigid_differential.rs
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/body.rs
  - crates/liquidfun/src/world/contact.rs
  - crates/liquidfun/src/world/contact_manager.rs
  - crates/liquidfun/src/world/contact_solver.rs
  - crates/liquidfun/src/world/fixture.rs
  - crates/liquidfun/src/world/object.rs
  - crates/liquidfun/src/world/proxy.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/tests/fixture_dynamics.rs
  - crates/liquidfun/tests/hook_contract.rs
  - crates/liquidfun/tests/rigid_contact_solver.rs
  - crates/liquidfun/tests/rigid_contacts.rs
  - crates/liquidfun/tests/rigid_definitions.rs
  - crates/liquidfun/tests/rigid_world.rs
  - justfile
  - protocol/fixtures/accepted/rigid-world-request.jsonl
  - protocol/schemas/protocol-v1.schema.json
  - protocol/schemas/scenario-v1.schema.json
  - protocol/schemas/trace-v1.schema.json
  - protocol/tolerances/phase6-v1.toml
  - reference/compatibility.json
  - tools/reference/CMakeLists.txt
  - tools/reference/adapter-inputs.txt
  - tools/reference/src/generate_build_identity.cmake.in
  - tools/reference/src/main.cpp
  - tools/reference/src/protocol.cpp
  - tools/reference/src/protocol.hpp
  - tools/reference/src/rigid_world.cpp
  - tools/reference/src/rigid_world.hpp
  - tools/reference/src/rigid_world_decode.hpp
  - tools/reference/src/rigid_world_trace.hpp
  - tools/reference/tests/protocol_tests.cpp
  - tools/xtask/src/differential.rs
  - tools/xtask/src/docs.rs
  - tools/xtask/src/upstream.rs
  - tools/xtask/tests/differential_cli.rs
  - tools/xtask/tests/docs_contract.rs
findings:
  critical: 1
  warning: 6
  info: 0
  total: 7
status: issues_found
---

# Phase 6: Code Review Report

**Reviewed:** 2026-07-12T07:39:31Z  
**Depth:** standard  
**Files Reviewed:** 66  
**Status:** issues_found

## Summary

The minimal rigid-world slice is well bounded and the existing focused Rust/protocol/differential tests pass, but the review found one safe-public-API panic with partial mutation, one upstream contact-admission mismatch, and several places where the Rust protocol, native executor, C++ adapter, staging workflow, and sanitizer lane do not share the same accepted contract.

The review applied the repository's `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the architecture, code-shape, verification, testing, and Rust standards. In particular, findings emphasize validate-before-commit state transitions, parse-at-boundary invariants, exact oracle parity, and evidence authority.

## Critical Issues

### CR-01: Aggregate fixture mass can panic after the new fixture is committed

**File:** `crates/liquidfun/src/world/body.rs:412-432` (commit begins in `crates/liquidfun/src/world/object.rs:490-507`)

**Issue:** `create_fixture` validates only the candidate fixture's individual `MassData`, inserts the fixture, creates its proxies, links it to the body, and then calls `reset_body_mass_after_validation`. That reset adds individually finite masses/inertias without checking each aggregate operation. Two valid centered circles of radius `1.0` and density approximately `f32::MAX / 4.0` each have finite mass and inertia, but their total mass overflows to infinity; the subsequent `mass * local_center.dot(local_center)` becomes NaN for a zero center and the assertion at lines 429-432 panics. The second fixture and its proxies have already been committed, so a caller that catches the unwind retains partially updated topology and stale mass state. The same unchecked aggregation is reachable through explicit mass reset after density edits.

**Fix:** Make aggregate mass calculation fallible and validate every source-ordered addition, weighted-center operation, division, and centered-inertia subtraction before mutating the body. Precompute the complete post-create aggregate before inserting a positive-density fixture, and return a typed fixture/mass-reset error on overflow. Add regression tests that prove both creation and explicit reset reject aggregate overflow without changing fixture, proxy, adjacency, or body-mass state.

## Warnings

### WR-01: Contacts are admitted between two non-dynamic bodies

**File:** `crates/liquidfun/src/world/contact_manager.rs:383-389`

**Issue:** `pair_is_eligible` rejects only static/static pairs. The pinned `b2Body::ShouldCollide` rejects every pair where neither body is dynamic, so overlapping static/kinematic and kinematic/kinematic fixtures incorrectly create Rust contacts. The fixed corpus keeps those body types separated, leaving this upstream mismatch unexercised while the contact-manager compatibility row is marked differentially validated.

**Fix:** Reject the pair when both body types are not `BodyType::Dynamic`, matching the pinned `ShouldCollide` predicate. Add focused overlapping static/kinematic and kinematic/kinematic tests and include at least one such declaration-first oracle witness before treating admission parity as complete.

### WR-02: The native executor ignores validated step parameters

**File:** `crates/liquidfun-differential/src/rigid_world.rs:402-406`

**Issue:** `RigidWorldAction::Step` carries `timestep_bits`, `velocity_iterations`, and `position_iterations`, and the C++ adapter executes those exact values at `tools/reference/src/rigid_world.cpp:253-267`. The native path matches `Step { .. }` and always runs the library's hard-coded `1/60`, 8, and 3 solver constants. Requests with any other values are accepted by the schema/decoder but do not describe the native execution being compared; iteration changes can even produce a match while proving different authored inputs.

**Fix:** Either thread the validated step tuple through `World::step` and `solve_contact`, or, for the intentionally fixed Phase 6 solver, require the exact witness timestep/iteration tuple in the Rust decoder, generated schema, and C++ decoder. Add negative boundary tests for every alternate tuple.

### WR-03: Rust and C++ accept different action-count bounds

**File:** `tools/reference/src/rigid_world_decode.hpp:395-400`

**Issue:** The authoritative Rust boundary and generated schema allow 128 actions per timeline (`crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:30` and `crates/liquidfun-test-protocol/src/schema/rigid_world.rs:105-108`), but the C++ decoder rejects more than 64. A request with 65-128 otherwise valid actions therefore passes native validation and fails only as an oracle harness error.

**Fix:** Define this limit once in the protocol contract and make the Rust bounded type, generated schema, and C++ decoder use the same value. Add an oracle protocol test at the accepted maximum and maximum-plus-one.

### WR-04: Invalid centered custom inertia passes protocol validation

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:380-390`

**Issue:** Request validation checks positive mass, finite center, and nonnegative origin inertia independently, but never validates `inertia - effective_mass * dot(center, center)`. For example, mass `1`, center `(2, 0)`, and inertia `1` is accepted as a validated request. The native executor then rejects it through `BodyMassData`, while the C++ decoder also accepts it and a debug oracle reaches the pinned `b2Assert(m_I > 0)`. This turns malformed boundary input into an execution/harness failure instead of a typed decode rejection.

**Fix:** Perform the same source-ordered finite centered-inertia calculation in the protocol validator and C++ decoder before constructing the action. Reject negative/non-finite centered inertia consistently and add matched Rust/schema/C++ fixtures.

### WR-05: The advertised rigid fixture-stage command is rejected by the real runner

**File:** `crates/liquidfun-differential/src/main.rs:243-254`

**Issue:** `tools/xtask` and `just rigid-stage` allow `--scenario rigid-world`, but the delegated differential binary accepts only `empty-world` and returns usage for every rigid stage attempt. The xtask test uses a fake child that records arguments, so it proves dispatch shape but never exercises the actual rejection. In addition, `validate_rigid_promotion_authority` is not called by any production staging or promotion path, so the D1 guard tested in isolation is not an integrated workflow.

**Fix:** Implement a rigid-specific stage/replay transaction that decodes the rigid request/result, verifies the complete build identity and exact comparison, calls `validate_rigid_promotion_authority` before any candidate write or promotion, and then reuses only the confined storage/review primitives. Add an end-to-end test through the real differential binary for both canonical acceptance and D2 rejection.

### WR-06: The sanitizer lane never executes the Phase 6 C++ adapter

**File:** `.github/workflows/oracle.yml:176-185`

**Issue:** The ASan/UBSan job builds the new rigid-world adapter but runs only `empty-world` comparisons. None of the Phase 6 decoder, raw-pointer contact identity tracking, fixture/body destruction bookkeeping, or rigid trace encoding executes under sanitizers, so ownership/lifetime defects in the reviewed C++ surface can pass the scheduled sanitizer lane.

**Fix:** Add a fail-fast `rigid-world` comparison using `oracle-asan-ubsan` (and run the reference protocol tests under that preset) before the read-only assertion. Keep sanitizer failures classified as harness failures and uploaded through the existing bounded artifact path.

## Verification

- `cargo test -p liquidfun --all-features` passed, including doctests.
- `cargo test -p liquidfun-test-protocol --all-features` passed.
- `cargo test -p liquidfun-differential --all-features --test rigid_world` passed (12 tests).
- `cargo test -p xtask --test differential_cli` passed (23 tests).
- Existing worktree change `.planning/config.json` was preserved and is unrelated to this report.

***

_Reviewed: 2026-07-12T07:39:31Z_  
_Reviewer: the agent (gsd-code-reviewer)_  
_Depth: standard_
