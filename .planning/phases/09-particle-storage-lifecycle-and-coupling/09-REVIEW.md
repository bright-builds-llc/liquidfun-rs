---
phase: 09-particle-storage-lifecycle-and-coupling
reviewed: 2026-07-17T14:27:52Z
generated_at: 2026-07-17T14:27:52Z
depth: standard
files_reviewed: 99
files_reviewed_list:
  - .github/workflows/oracle.yml
  - COMPATIBILITY.md
  - Cargo.toml
  - TESTING.md
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/src/rigid_world/phase9.rs
  - crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/aabb-query-control-and-culling.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/closed-evidence-contract.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/contacts-listeners-filters-and-coupling.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/forces-impulses-and-statistics.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/lifetime-zombie-and-eviction.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/ray-control-and-culling.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/storage-systems-and-permutations.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json
  - crates/liquidfun-differential/tests/particle_oracle.rs
  - crates/liquidfun-differential/tests/particle_protocol.rs
  - crates/liquidfun-differential/tests/phase9_corpus.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world/phase9.rs
  - crates/liquidfun/Cargo.toml
  - crates/liquidfun/src/arena.rs
  - crates/liquidfun/src/error.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/particle.rs
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/particle/buffer.rs
  - crates/liquidfun/src/particle/contact.rs
  - crates/liquidfun/src/particle/definition.rs
  - crates/liquidfun/src/particle/editor.rs
  - crates/liquidfun/src/particle/force.rs
  - crates/liquidfun/src/particle/lifetime.rs
  - crates/liquidfun/src/particle/lifetime/tests.rs
  - crates/liquidfun/src/particle/proxy.rs
  - crates/liquidfun/src/particle/query.rs
  - crates/liquidfun/src/particle/statistics.rs
  - crates/liquidfun/src/particle/storage.rs
  - crates/liquidfun/src/particle/storage/editor_tests.rs
  - crates/liquidfun/src/particle/storage/identity.rs
  - crates/liquidfun/src/particle/storage/lane_inventory.rs
  - crates/liquidfun/src/particle/storage/lanes.rs
  - crates/liquidfun/src/particle/storage/permutation.rs
  - crates/liquidfun/src/particle/storage/permutation/tests.rs
  - crates/liquidfun/src/particle/storage/properties.rs
  - crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs
  - crates/liquidfun/src/particle/storage/properties/permutation_model.rs
  - crates/liquidfun/src/particle/storage/validation.rs
  - crates/liquidfun/src/particle/view.rs
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/config.rs
  - crates/liquidfun/src/world/object.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_lifecycle.rs
  - crates/liquidfun/src/world/particle_object.rs
  - crates/liquidfun/src/world/query.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/tests/particle_body_contacts.rs
  - crates/liquidfun/tests/particle_buffers.rs
  - crates/liquidfun/tests/particle_contacts.rs
  - crates/liquidfun/tests/particle_creation_eviction.rs
  - crates/liquidfun/tests/particle_definitions.rs
  - crates/liquidfun/tests/particle_forces_statistics.rs
  - crates/liquidfun/tests/particle_lifecycle.rs
  - crates/liquidfun/tests/particle_lifetimes.rs
  - crates/liquidfun/tests/particle_objects.rs
  - crates/liquidfun/tests/particle_permutation_coherence.rs
  - crates/liquidfun/tests/particle_queries.rs
  - crates/liquidfun/tests/particle_step_guards.rs
  - crates/liquidfun/tests/particle_views.rs
  - crates/liquidfun/tests/particle_zombie_authority.rs
  - deny.toml
  - protocol/schemas/scenario-v1.schema.json
  - protocol/schemas/trace-v1.schema.json
  - reference/artifacts/phase9/lifecycle-contact-witnesses.json
  - reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json
  - reference/compatibility.json
  - reference/source-map.toml
  - scripts/phase9-evidence.sh
  - tools/reference/CMakeLists.txt
  - tools/reference/adapter-inputs.txt
  - tools/reference/src/phase9_lifecycle_contact_witness.cpp
  - tools/reference/src/rigid_world.cpp
  - tools/reference/src/rigid_world.hpp
  - tools/reference/src/rigid_world_decode.hpp
  - tools/reference/src/rigid_world_phase9_decode.hpp
  - tools/reference/src/rigid_world_phase9_execute.hpp
  - tools/reference/tests/sanitizer_scope.cmake
  - tools/xtask/src/inventory/validation.rs
  - tools/xtask/src/provenance.rs
  - tools/xtask/src/provenance/phase9_witness.rs
  - tools/xtask/src/upstream.rs
  - tools/xtask/tests/inventory_cli.rs
  - tools/xtask/tests/upstream_cli.rs
findings:
  critical: 0
  warning: 2
  info: 1
  total: 3
status: issues_found
---

# Phase 09: Code Review Report

**Reviewed:** 2026-07-17T14:27:52Z
**Depth:** standard
**Files Reviewed:** 99
**Status:** issues_found

## Summary

The standard-depth review covered all 99 source, protocol, oracle, evidence, workflow, test, and documentation files declared by the Phase 09 summaries after excluding planning artifacts and generated `target/` outputs. The implementation's particle storage, lifecycle, permutation, contact, coupling, query, and public API paths are well defended by focused tests. No critical security, crash, or data-loss issue was found.

Two evidence-integrity gaps remain actionable: the Phase 09 differential comparator does not compose the retained Phase 06–08 rigid comparator, and several branches counted as executable coverage are represented only by configuration/input assertions or trivial empty outputs instead of branch-specific semantic observations. One commented-out construction block should also be removed.

The review applied `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the relevant architecture, code-shape, verification, testing, operability, and Rust standards.

## Warnings

### WR-01: Phase 09 comparison drops retained Phase 06–08 rigid semantics

**Files:** `crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs:225-310`; `crates/liquidfun-differential/src/rigid_world.rs:182-204`; `crates/liquidfun-differential/src/rigid_evidence.rs:421-458`

**Issue:** After validating each result structurally, `compare_phase9_rigid_world_results` filters every checkpoint through `particle_observations`, which discards all non-particle observations. It also never compares checkpoint body or fixture state. The Phase 09 runner invokes only that particle comparator and does not compose the existing `compare_phase8_rigid_world_results` walker. Consequently, a native/oracle disagreement in a request-valid retained rigid body, fixture, Phase 07, or Phase 08 value can still return `Match`, even though the corpus reports the `retained_phase6_through_phase8` branch and promoted compatibility evidence includes retained behavior.

Existing mutation tests cover particle observations only, so they do not guard this boundary.

**Fix:** Load the inherited Phase 06, 07, and 08 policy profiles in the Phase 09 runner, execute `compare_phase8_rigid_world_results` before the particle-only comparison, and translate any inherited mismatch without continuing to a Phase 09 match. Add a regression that mutates a structurally valid body or fixture numeric field in one result and asserts a deterministic mismatch.

### WR-02: Several “reached” branches lack branch-specific semantic witnesses

**Files:** `crates/liquidfun-differential/tests/phase9_corpus.rs:615-925`; `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json:102-218`; `scripts/phase9-evidence.sh:48-62`

**Issue:** The seven cases do execute native and pinned-oracle requests, but several of the 58 reported branches are not proven by the named behavior:

- `finite_lifetime`, `infinite_lifetime`, and `equal_lifetime` assert only request declaration bits.
- Strict-contact and listener/filter enabled/disabled branches assert only configuration or flag bits.
- `collision_energy` and `stuck_candidates` accept zero and an empty list, so their nontrivial calculation paths need not run.
- The per-branch replay, minimization, first-divergence, D0, and debug/release assertions reduce to request/scenario ID equality.
- Multiple manifest witnesses point to the same generic `inspect-particle` observation while their semantic assertions inspect unrelated declarations or outputs.

The evidence script then verifies only the count and uniqueness of branch labels. This permits the manifest to claim complete executable coverage while regressions in the named branches remain unexercised, contrary to the Phase 09 plan's branch-specific semantic witness contract.

**Fix:** Bind every manifest branch to the exact action/checkpoint/output that demonstrates it and validate that binding mechanically. Use scenarios with observable state transitions for finite/infinite/equal lifetime ordering and strict/filter/listener behavior, nonzero collision energy and nonempty stuck candidates, and actual comparison/replay digest or mismatch assertions for the evidence-contract branches. Reject a manifest when a branch is supported only by input configuration or an unrelated observation.

## Info

### IN-01: Obsolete constructor call remains commented out

**File:** `crates/liquidfun/src/particle/storage.rs:416-425`

**Issue:** A block-commented call to `Self::from_owned_lanes` remains between `commit_create` and the live constructor implementation. It is dead code and obscures the active construction path.

**Fix:** Delete the commented-out block. Version control already preserves the prior implementation.

## Verification

- The required independent Phase 09 gates recorded by the phase artifacts passed, including the canonical and sanitizer evidence jobs, provenance and inventory checks, and the focused Phase 09 native/protocol/oracle suites.
- A redundant local `cargo test --workspace --all-features` rerun completed the `liquidfun` unit and integration suites and continued without failures through the differential suite before it was interrupted while starting `particle_oracle`; no failure was observed.
- Review findings were validated by tracing the Phase 09 runner, comparator, inherited Phase 08 comparator, corpus assertions, fixture registry, and evidence-script acceptance checks.
- No source file was modified and no commit was created.

***

_Reviewed: 2026-07-17T14:27:52Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
