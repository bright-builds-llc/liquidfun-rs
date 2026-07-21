---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
reviewed: 2026-07-21T16:13:44Z
depth: standard
files_reviewed: 151
files_reviewed_list:
  - .github/workflows/oracle.yml
  - COMPATIBILITY.md
  - TESTING.md
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/src/rigid_world/phase10.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator/numeric.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/groups.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/topology.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator/records/witness.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/comparator/registry.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/evidence.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/native.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/native/capture.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/native/evidence.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/native/recipe.rs
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/boundary-order-and-inherited.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/group-construction-and-mutation.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/pressure-constraints-and-rigid.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/solver-material-flags.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/cases/topology-join-split-reactive.jsonl
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json
  - crates/liquidfun-differential/tests/phase10_comparator.rs
  - crates/liquidfun-differential/tests/phase10_corpus.rs
  - crates/liquidfun-differential/tests/phase10_corpus/evidence_output.rs
  - crates/liquidfun-differential/tests/phase10_native.rs
  - crates/liquidfun-differential/tests/phase10_oracle.rs
  - crates/liquidfun-differential/tests/phase10_protocol.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/phase10.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs
  - crates/liquidfun/src/association.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/particle.rs
  - crates/liquidfun/src/particle/definition.rs
  - crates/liquidfun/src/particle/group.rs
  - crates/liquidfun/src/particle/group/sampling.rs
  - crates/liquidfun/src/particle/group/tests.rs
  - crates/liquidfun/src/particle/lifetime.rs
  - crates/liquidfun/src/particle/solver.rs
  - crates/liquidfun/src/particle/solver/boundary.rs
  - crates/liquidfun/src/particle/solver/boundary/barrier.rs
  - crates/liquidfun/src/particle/solver/boundary/collision.rs
  - crates/liquidfun/src/particle/solver/boundary/support.rs
  - crates/liquidfun/src/particle/solver/boundary/tests.rs
  - crates/liquidfun/src/particle/solver/constraints.rs
  - crates/liquidfun/src/particle/solver/constraints/tests.rs
  - crates/liquidfun/src/particle/solver/manifest.rs
  - crates/liquidfun/src/particle/solver/manifest/witness_registry.rs
  - crates/liquidfun/src/particle/solver/material.rs
  - crates/liquidfun/src/particle/solver/material/tests.rs
  - crates/liquidfun/src/particle/solver/order_tests.rs
  - crates/liquidfun/src/particle/solver/preparation.rs
  - crates/liquidfun/src/particle/solver/pressure.rs
  - crates/liquidfun/src/particle/solver/pressure/tests.rs
  - crates/liquidfun/src/particle/solver/rigid.rs
  - crates/liquidfun/src/particle/solver/rigid/damping.rs
  - crates/liquidfun/src/particle/solver/rigid/projection.rs
  - crates/liquidfun/src/particle/solver/rigid/support.rs
  - crates/liquidfun/src/particle/solver/rigid/tests.rs
  - crates/liquidfun/src/particle/storage.rs
  - crates/liquidfun/src/particle/storage/editor_tests.rs
  - crates/liquidfun/src/particle/storage/group.rs
  - crates/liquidfun/src/particle/storage/group/depth.rs
  - crates/liquidfun/src/particle/storage/group/statistics.rs
  - crates/liquidfun/src/particle/storage/group/tests.rs
  - crates/liquidfun/src/particle/storage/lane_inventory.rs
  - crates/liquidfun/src/particle/storage/lanes.rs
  - crates/liquidfun/src/particle/storage/mutation.rs
  - crates/liquidfun/src/particle/storage/mutation/join.rs
  - crates/liquidfun/src/particle/storage/mutation/join/tests.rs
  - crates/liquidfun/src/particle/storage/mutation/split.rs
  - crates/liquidfun/src/particle/storage/mutation/split/tests.rs
  - crates/liquidfun/src/particle/storage/permutation.rs
  - crates/liquidfun/src/particle/storage/permutation/group_reassignment.rs
  - crates/liquidfun/src/particle/storage/permutation/tests.rs
  - crates/liquidfun/src/particle/storage/properties.rs
  - crates/liquidfun/src/particle/storage/properties/group_model.rs
  - crates/liquidfun/src/particle/storage/properties/permutation_model.rs
  - crates/liquidfun/src/particle/storage/solver_state.rs
  - crates/liquidfun/src/particle/storage/solver_state/tests.rs
  - crates/liquidfun/src/particle/storage/validation.rs
  - crates/liquidfun/src/particle/topology.rs
  - crates/liquidfun/src/particle/topology/connectivity.rs
  - crates/liquidfun/src/particle/topology/constraints.rs
  - crates/liquidfun/src/particle/topology/constraints/tests.rs
  - crates/liquidfun/src/particle/topology/constraints/tests/properties.rs
  - crates/liquidfun/src/particle/topology/voronoi.rs
  - crates/liquidfun/src/particle/topology/voronoi/tests.rs
  - crates/liquidfun/src/particle/view.rs
  - crates/liquidfun/src/world/object.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_coupling/body_coupling.rs
  - crates/liquidfun/src/world/particle_coupling/executor.rs
  - crates/liquidfun/src/world/particle_coupling/executor/boundary_runtime.rs
  - crates/liquidfun/src/world/particle_lifecycle.rs
  - crates/liquidfun/src/world/particle_object.rs
  - crates/liquidfun/src/world/particle_object/group_lifecycle.rs
  - crates/liquidfun/src/world/particle_object/group_lifecycle_tests.rs
  - crates/liquidfun/src/world/particle_object/group_mutation.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/tests/particle_body_contacts.rs
  - crates/liquidfun/tests/particle_group_lifecycle.rs
  - crates/liquidfun/tests/particle_group_mutation.rs
  - crates/liquidfun/tests/particle_group_properties.rs
  - crates/liquidfun/tests/particle_group_properties/model.rs
  - crates/liquidfun/tests/particle_group_properties/snapshot.rs
  - crates/liquidfun/tests/particle_groups.rs
  - crates/liquidfun/tests/particle_solver_baseline.rs
  - crates/liquidfun/tests/particle_solver_flags.rs
  - crates/liquidfun/tests/particle_solver_order.rs
  - justfile
  - protocol/schemas/scenario-v1.schema.json
  - protocol/schemas/trace-v1.schema.json
  - reference/artifacts/phase10/group-topology-witnesses.json
  - reference/artifacts/phase10/group-topology-witnesses.provenance.json
  - reference/compatibility.json
  - scripts/phase10-evidence.sh
  - tools/reference/CMakeLists.txt
  - tools/reference/adapter-inputs.txt
  - tools/reference/src/main.cpp
  - tools/reference/src/phase10_group_topology_cases.cpp
  - tools/reference/src/phase10_group_topology_cases.hpp
  - tools/reference/src/phase10_group_topology_witness.cpp
  - tools/reference/src/rigid_world.cpp
  - tools/reference/src/rigid_world.hpp
  - tools/reference/src/rigid_world_decode.hpp
  - tools/reference/src/rigid_world_phase10_capture.hpp
  - tools/reference/src/rigid_world_phase10_decode.hpp
  - tools/reference/src/rigid_world_phase10_execute.hpp
  - tools/reference/src/rigid_world_phase10_operations.hpp
  - tools/reference/src/rigid_world_phase9_decode.hpp
  - tools/xtask/src/inventory/validation/phase10.rs
  - tools/xtask/src/main.rs
  - tools/xtask/src/phase10_evidence.rs
  - tools/xtask/src/phase10_evidence/authority.rs
  - tools/xtask/src/phase10_evidence/content.rs
  - tools/xtask/src/phase10_evidence/paths.rs
  - tools/xtask/src/upstream.rs
  - tools/xtask/tests/inventory_cli/phase10.rs
  - tools/xtask/tests/phase10_evidence_cli.rs
  - tools/xtask/tests/phase10_evidence_cli/exact.rs
  - tools/xtask/tests/phase10_evidence_cli/support.rs
  - tools/xtask/tests/upstream_cli.rs
findings:
  critical: 0
  warning: 4
  info: 0
  total: 4
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-07-21T16:13:44Z
**Depth:** standard
**Files Reviewed:** 151
**Status:** issues_found

## Summary

The Phase 10 engine, adapters, comparator, evidence tooling, protocol, fixtures, documentation, and CI were reviewed at standard depth. The engine implementation is heavily guarded and its full Rust test suite and Clippy checks pass. Four protocol-boundary validation gaps remain: an unsatisfiable generated JSON Schema record, two request-decoder acceptance mismatches with the C++ oracle, and an incomplete result ownership check. Generated runtime evidence under `target/phase10-evidence/` was excluded from code review as directed.

Verification performed:

- `cargo test --all-features` — passed, including 19 doctests.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.

## Warnings

### WR-01: Phase 10 provenance schema is unsatisfiable

**File:** `crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs:131-148`

**Issue:** `provenance_schema` requires `extension_version`, but its closed `properties` object does not define that member. Because `closed_record` sets `additionalProperties: false`, a record can neither omit the required member nor include it. The inverse mistake appears in `state_schema`: it defines a top-level `extension_version` that `Phase10StateObservation` does not serialize and does not require. The checked-in generated schemas preserve the impossible provenance contract, so standards-compliant JSON Schema validation rejects every Phase 10 group definition even though serde decoding accepts it.

**Fix:** Put the version member on the provenance record and remove the unused top-level state member, then regenerate both checked-in schemas and add a regression that validates a complete Phase 10 request/result against them.

```rust
fn provenance_schema() -> Value {
    closed_record(
        &json!({
            "extension_version": { "const": crate::PHASE10_RIGID_WORLD_EXTENSION_VERSION },
            "generator_id": semantic_id_schema(),
            // ...
        }),
        &["extension_version", "generator_id", /* ... */],
    )
}
```

### WR-02: Rust request validation omits the cumulative group-identity bound

**Files:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase10.rs:329-345`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase10.rs:408-418`

**Issue:** `PHASE10_MAXIMUM_GROUPS` is documented as the maximum live or declared groups in a timeline, but Rust only bounds the number of IDs in one split operation. Repeated `CreateGroup`/destroy or split operations can grow `created_groups` beyond 64 and still decode successfully. The C++ decoder rejects the same request once `all_groups.size()` exceeds the bound (`rigid_world_phase10_decode.hpp:288-290` and `323-325`). A request can therefore pass the Rust wire boundary and fail only when sent to the oracle, misclassifying malformed input as an oracle/harness failure.

**Fix:** Preflight the cumulative count before inserting any new group identities in both the new-group and split branches, and add boundary tests for exactly 64 total declared IDs and 65 across multiple operations.

```rust
let total = self.created_groups.len().checked_add(created_group_ids.len())
    .ok_or(Phase10ValidationKind::BoundaryLimitExceeded)?;
if total > PHASE10_MAXIMUM_GROUPS {
    return Err(Phase10ValidationKind::BoundaryLimitExceeded);
}
```

### WR-03: Inspection without provenance passes Rust request decoding

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase10.rs:365`

**Issue:** `Phase10ActionState::apply` accepts `InspectState` unconditionally, including before any group definition establishes Phase 10 provenance. The native adapter later fails with “Phase 10 inspection has no provenance,” and the C++ decoder rejects the operation at its validation boundary (`rigid_world_phase10_decode.hpp:365-370`). This again allows a malformed request through the shared Rust decoder and converts an input error into an adapter-specific execution failure.

**Fix:** Require `maybe_provenance` when applying `InspectState`, while leaving timelines with no Phase 10 actions valid, and add an integration test that an inspect-only Phase 10 action fails during request decoding.

```rust
Phase10Operation::InspectState => self
    .maybe_provenance
    .as_ref()
    .map(|_| ())
    .ok_or(Phase10ValidationKind::InvalidProvenance),
```

### WR-04: Result validation admits inconsistent group and topology ownership

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10.rs:327-383`

**Issue:** The validator checks that flattened group member IDs match particle order, and separately checks that each particle names an existing group in the same system. It never checks that the particle's `group_id` is the group that actually listed that particle. Two same-system groups can therefore swap the `group_id` fields in their particle snapshots and still validate. Likewise, pair and triad endpoints are checked only for existence/distinctness, so topology can connect particles from different systems even though topology is system-owned. These malformed observations can reach the comparator despite its fail-closed contract.

**Fix:** Build an authoritative member-ID-to-(group-ID, system-ID) map while walking groups, require each particle snapshot to match that exact owner, and require every pair/triad endpoint to share one system. Add negative tests for swapped same-system group ownership and cross-system pair/triad endpoints.

```rust
let owner = member_owners
    .get(&particle.particle_id)
    .ok_or(Phase10ValidationKind::InvalidOwnership)?;
if owner.group_id != particle.group_id || owner.system_id != particle.system_id {
    return Err(Phase10ValidationKind::InvalidOwnership);
}
```

***

_Reviewed: 2026-07-21T16:13:44Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
