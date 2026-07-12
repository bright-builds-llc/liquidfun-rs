---
status: issues_found
phase: 06-minimal-rigid-world-vertical-slice
depth: standard
files_reviewed: 74
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T17:35:58Z
---

# Phase 6: Code Review Report

**Reviewed:** 2026-07-12T17:35:58Z
**Depth:** standard
**Files Reviewed:** 74
**Status:** issues_found

## Summary

The gap-closure work fixes the original contact-admission, fixed-step, action-bound, negative-inertia, real-binary fixture, and sanitizer-execution gaps. Aggregate fixture creation and explicit reset are also transactional now. Three correctness gaps remain: other implicit mass-reset paths can still panic after mutation, the custom-mass boundary accepts the zero-centered-inertia equality that the pinned debug oracle asserts against, and rigid fixture promotion trusts a self-reported D1 identity without binding its adapter and compile-command digests to the current checkout.

This review applied the repository's `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the architecture, code-shape, testing, verification, and Rust standards. Findings emphasize validate-before-commit public transitions, parse-at-boundary parity, and evidence provenance tied to the checked-in implementation.

## Critical Issues

### CR-01: Implicit mass resets can still panic after body or fixture mutation

**Files:** `crates/liquidfun/src/world/object.rs:389-405`, `crates/liquidfun/src/world/object.rs:1042-1060`, `crates/liquidfun/src/world/object.rs:1231-1235`

**Issue:** The new fallible aggregate calculation is used transactionally only by fixture creation and explicit `reset_body_mass_data`. `set_body_type` still destroys contacts and changes the body type before calling `reset_body_mass_after_validation`, while fixture destruction removes contacts, proxies, storage, and adjacency before the same helper. That helper converts `AggregateMassError` into `expect`, even though invalid committed fixture aggregates remain constructible by supported APIs. For example, a static body can own two individually valid circles at density `f32::MAX / 4.0`; changing it to dynamic overflows the aggregate and panics after the type/contact mutation. Likewise, a dynamic body can hold three zero-density fixtures, have each density changed independently to that value without resetting mass, and then panic after destroying one fixture because the two remaining fixtures overflow. These are safe public APIs, and caught unwinds leave partially mutated world topology or body state.

**Fix:** Make every mass-resetting transition validate its complete prospective aggregate before effects. At minimum, compute the prospective body state before contact destruction/type mutation and compute the remaining-fixture state before fixture/contact/proxy removal. Return typed aggregate errors from body-type and fixture-destruction operations rather than routing fallible user state through `expect`. Add regression tests for static-to-dynamic overflow and post-density-edit fixture destruction that assert no changes to type, contacts, fixtures, proxies, adjacency, or mass bits.

## Warnings

### WR-01: Zero centered inertia still passes the boundary and trips the pinned debug assertion

**Files:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:448-470`, `tools/reference/src/rigid_world_decode.hpp:283-305`, `crates/liquidfun/src/world/body.rs:790-807`

**Issue:** Rust protocol validation, the C++ decoder, and `BodyMassData::new` reject centered inertia only when it is negative, so `mass = 1`, `center = (1, 0)`, and origin inertia `1` is accepted with centered inertia exactly zero. The pinned `b2Body::SetMassData` branch executes because origin inertia is positive and asserts `m_I > 0` after subtraction. A request accepted by both decoders can therefore abort the debug oracle instead of producing a typed boundary rejection; release behavior can also divide by zero when computing inverse inertia. The new tests cover negative and non-finite results but not this equality boundary.

**Fix:** When origin inertia is positive, require the centered result to be strictly positive in the Rust domain constructor, protocol validator, and C++ decoder; continue allowing origin inertia zero through the no-inertia branch. Add matched Rust/C++ tests for the equality case alongside the negative fixture.

### WR-02: Rigid fixture promotion does not bind D1 identity to the current adapter or compile database

**Files:** `crates/liquidfun-differential/src/rigid_fixtures.rs:62-103`, `crates/liquidfun-differential/src/rigid_fixtures.rs:138-175`, `tools/xtask/src/differential.rs:629-654`

**Issue:** Ordinary rigid comparison rejects a stale oracle by recomputing the checked-in adapter-source digest and effective compile-command digest. The new fixture-stage path omits both checks: it verifies only the manifest revision, requested preset, internally consistent handshake, semantic comparison, and self-derived D1 tier. Replay compares that self-reported identity only with candidate metadata. Consequently, a stale canonical binary from an earlier checkout can stage, review, and promote while metadata records the current `generator_revision`; the real-binary tests reinforce this gap by accepting a fake D1 identity with arbitrary adapter and compile digests. This weakens the promised source-to-evidence provenance even when the semantic trace happens to match.

**Fix:** Share the adapter-source and effective compile-command identity validation used by `execute_rigid_world_once` with rigid staging. Before the first candidate write, compare the captured adapter digest and Phase 4 compile-command digest with values recomputed from the current checkout and selected preset. Recheck the same binding during review/promotion or bind it to immutable candidate generator inputs, and add a real-binary stale-digest rejection proving no staging or accepted-path mutation.

## Files Reviewed

- `.github/workflows/ci.yml`
- `.github/workflows/oracle.yml`
- `ARCHITECTURE.md`
- `COMPATIBILITY.md`
- `README.md`
- `TESTING.md`
- `crates/liquidfun-differential/native-math-sources.txt`
- `crates/liquidfun-differential/src/failure_bundle.rs`
- `crates/liquidfun-differential/src/fixtures/lifecycle.rs`
- `crates/liquidfun-differential/src/fixtures/replay.rs`
- `crates/liquidfun-differential/src/main.rs`
- `crates/liquidfun-differential/src/minimizer.rs`
- `crates/liquidfun-differential/src/rigid_evidence.rs`
- `crates/liquidfun-differential/src/rigid_fixtures.rs`
- `crates/liquidfun-differential/src/rigid_world.rs`
- `crates/liquidfun-differential/src/supervisor.rs`
- `crates/liquidfun-differential/src/supervisor/rigid_world.rs`
- `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs`
- `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs`
- `crates/liquidfun-differential/tests/rigid_world.rs`
- `crates/liquidfun-test-protocol/src/scenario.rs`
- `crates/liquidfun-test-protocol/src/scenario/rigid_world.rs`
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs`
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs`
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs`
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs`
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs`
- `crates/liquidfun-test-protocol/src/schema.rs`
- `crates/liquidfun-test-protocol/src/schema/rigid_world.rs`
- `crates/liquidfun-test-protocol/src/schema/tests.rs`
- `crates/liquidfun-test-protocol/src/tolerance.rs`
- `crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs`
- `crates/liquidfun-test-protocol/tests/fixtures.rs`
- `crates/liquidfun/src/lib.rs`
- `crates/liquidfun/src/rigid_differential.rs`
- `crates/liquidfun/src/world.rs`
- `crates/liquidfun/src/world/body.rs`
- `crates/liquidfun/src/world/contact.rs`
- `crates/liquidfun/src/world/contact_manager.rs`
- `crates/liquidfun/src/world/contact_solver.rs`
- `crates/liquidfun/src/world/fixture.rs`
- `crates/liquidfun/src/world/object.rs`
- `crates/liquidfun/src/world/proxy.rs`
- `crates/liquidfun/src/world/step.rs`
- `crates/liquidfun/tests/fixture_dynamics.rs`
- `crates/liquidfun/tests/hook_contract.rs`
- `crates/liquidfun/tests/rigid_contact_solver.rs`
- `crates/liquidfun/tests/rigid_contacts.rs`
- `crates/liquidfun/tests/rigid_definitions.rs`
- `crates/liquidfun/tests/rigid_world.rs`
- `justfile`
- `protocol/fixtures/accepted/rigid-world-request.jsonl`
- `protocol/fixtures/rejected/rigid-world-negative-centered-inertia.jsonl`
- `protocol/schemas/protocol-v1.schema.json`
- `protocol/schemas/scenario-v1.schema.json`
- `protocol/schemas/trace-v1.schema.json`
- `protocol/tolerances/phase6-v1.toml`
- `reference/compatibility.json`
- `tools/reference/CMakeLists.txt`
- `tools/reference/adapter-inputs.txt`
- `tools/reference/src/generate_build_identity.cmake.in`
- `tools/reference/src/main.cpp`
- `tools/reference/src/protocol.cpp`
- `tools/reference/src/protocol.hpp`
- `tools/reference/src/rigid_world.cpp`
- `tools/reference/src/rigid_world.hpp`
- `tools/reference/src/rigid_world_decode.hpp`
- `tools/reference/src/rigid_world_trace.hpp`
- `tools/reference/tests/protocol_tests.cpp`
- `tools/xtask/src/differential.rs`
- `tools/xtask/src/docs.rs`
- `tools/xtask/src/upstream.rs`
- `tools/xtask/tests/differential_cli.rs`
- `tools/xtask/tests/docs_contract.rs`

## Verification

- Reviewed the complete 74-file Phase 6 source scope and the gap-closure commit range `a068668..630d186`.
- Confirmed the original seven review findings against their current implementation and regression evidence.
- `git diff --check` is required after this report write; no source files were edited by the reviewer.

***

_Reviewer: gsd-code-reviewer_
_Lifecycle: 6-2026-07-12T02-22-53_
