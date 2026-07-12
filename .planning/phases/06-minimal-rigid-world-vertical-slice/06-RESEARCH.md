---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T02:40:00.000Z
phase: 06-minimal-rigid-world-vertical-slice
status: complete
requirements:
  - RIGD-01
  - RIGD-02
  - RIGD-04
---

# Phase 6 Research: Minimal Rigid World Vertical Slice

## Research conclusion

Phase 6 should deepen the existing `World` in five dependency-ordered seams:

1. checked body/fixture definitions and state storage;
2. fixture-owned broad-phase proxies plus exact mass and mutation side effects;
3. a private automatic contact manager over Phase 5 pair/manifold kernels;
4. one bounded discrete-contact constraint path with feature-based impulse carry; and
5. one closed rigid-world timeline protocol implemented by Rust and the pinned C++ oracle.

The phase must not become a partial copy of the full `b2World::Step`. The smallest honest slice supports zero-gravity non-colliding lifecycle and one static/dynamic discrete contact, including contact creation, persistence, sensor/filter/activation transitions, warm-start carry, and destruction. Forces, general island traversal, multi-contact stacks, sleeping, CCD/TOI orchestration, queries, joints, and broad configuration remain Phase 7 or 8.

## Requirement coverage strategy

| Requirement | Planning implication |
| --- | --- |
| `RIGD-01` | Add checked body definitions and handle-oriented create/inspect/type/transform/activate/deactivate/destroy operations for static, kinematic, and dynamic bodies. |
| `RIGD-02` | Add immutable-shape fixture definitions, sensor/material/filter state, exact mass reset/override behavior, proxy lifecycle, and ordered destruction. |
| `RIGD-04` | Add automatic contact management, manifolds, mixing, sensors, filtering, warm-start transfer, a minimal solve, ordered reports, and end-to-end differential evidence. |

## Existing foundations to reuse

- `world/object.rs` already owns typed arenas, newest-first body adjacency, validate-before-mutate destruction transactions, and owned destruction records.
- `world/step.rs` already proves borrow-scoped contact views, lock restoration, panic poisoning, bounded owned reports, and post-unlock typed commands.
- `collision::Shape` is immutable and owned; fixtures should store it by value without adding mutable topology.
- `collision::BroadPhase` already preserves upstream proxy ordering, move/touch buffering, candidate sorting/deduplication, and pure filter/refilter behavior.
- Phase 5 manifolds use semantic feature identity and exact point order, which is the correct key for contact persistence and impulse transfer.
- The protocol/differential/reference toolchain already provides exact float bits, bounded schemas, build identity, first-divergence, D0 replay, minimization, and reviewed fixture promotion.

## Recommended production architecture

Keep `world.rs` as the curated deep-module entrypoint. Split cohesive children as needed:

- `world/body.rs`: `BodyType`, checked `BodyDef`, body state, transforms, type/active transitions, mass state, public snapshots;
- `world/fixture.rs`: checked `FixtureDef`, immutable `Shape`, material/sensor/filter state, child proxies, public snapshots;
- `world/contact.rs`: private contact state, oriented fixture-child pair identity, manifold persistence, touching/enabled/filter flags, mixed material, impulses;
- `world/contact_manager.rs`: Phase 5 broad-phase integration, admission, duplicate detection, list order, filtering, update, and destruction;
- `world/contact_solver.rs`: minimal one-contact constraint initialization, warm start, solve, and impulse write-back;
- `world/step.rs`: orchestration, hooks, reports, deferred commands, and the narrow Phase 6 phase order.

Do not create a second publishable dynamics crate. Public handles stay authority-free identities; every operation remains a `World` method. Contacts stay private and transient.

## Pinned-source findings

### Bodies and fixtures

- `b2Body::SetType` resets mass, zeros velocity for static bodies, wakes the body, destroys all contacts, and touches proxies. Preserve this transition as one centralized world operation.
- `b2Body::SetTransform` updates transform/sweep, synchronizes fixture proxies immediately, and lets the next contact phase discover/update overlaps.
- `b2Body::SetActive(false)` destroys proxies and contacts; activation recreates proxies but does not immediately create contacts.
- Fixture creation clones the shape, creates proxies only when active, and resets mass only for positive density. Fixture destruction destroys attached contacts and proxies, unlinks newest-first adjacency, then always resets mass.
- `Fixture::SetDensity` stores the value but does not reset body mass. The safe API therefore needs an explicit `reset_mass_data` method and tests proving the asymmetry.
- `SetMassData` affects dynamic bodies only. Centered rotational inertia must remain non-negative after the parallel-axis adjustment. Later fixture/type/fixed-rotation reset triggers replace the override.
- Sensor changes wake the owning body. Filter changes flag existing contacts and touch every child proxy. Friction/restitution fixture edits do not update already mixed contact values.

### Contact manager and contact update

- Pair admission first rejects same-body pairs, existing fixture-child pairs, body/joint collision suppression, and filter rejection; pair registration may canonicalize shape order before newest-first contact insertion.
- Contact filtering is deferred through a flag and reevaluated during `Collide`. A filtered or non-overlapping pair is destroyed through the normal manager path.
- Non-awake contacts may skip update. Phase 6 should preserve source gating relevant to the supported active/no-sleep slice without claiming sleeping parity.
- Non-sensors evaluate a manifold, match new points to the old manifold by semantic feature identity, and copy normal/tangent impulses for persistent points. Sensors evaluate overlap only and retain no manifold.
- Begin/end events follow the touching transition. Pre-solve applies only to awake, non-sensor touching updates and receives the old manifold. Existing Phase 3 hooks should evolve rather than adding a parallel listener API.
- Friction is mixed as the square root of the product; restitution is the maximum. These values are initialized when the contact is created and persist until explicitly changed or the contact is recreated.

### Minimal solver boundary

- The complete contact solver is array/island oriented, but its per-contact constraint math can be isolated for exactly one dynamic/static contact with up to two manifold points.
- Preserve source initialization order, warm-start scaling/application, normal then tangent impulse handling, point order, and write-back to manifold points.
- Phase 6 needs enough internal velocity/sweep state to run this witness, but should not publish the Phase 7 force/impulse/damping/sleep/bullet surface early.
- The supported scenario must avoid multi-contact island ordering, gravity accumulation, sleeping, CCD, joints, and queries. Tests must state that boundary explicitly.

## Step phase order for the slice

Use a fixed, documented Phase 6 order derived from the pinned source:

1. reject nested/poisoned stepping and acquire the lock;
2. find new broad-phase pairs and create eligible contacts;
3. refilter/update/destroy existing contacts and emit touching lifecycle transitions;
4. construct and solve the one-contact discrete constraint, carrying impulses by feature identity;
5. synchronize supported moved fixtures and find resulting pairs when required by the scenario action;
6. release the lock;
7. apply bounded deferred commands sequentially and append their destruction/contact-end evidence.

The plan should make the exact timing observable in unit and differential tests rather than hiding it behind one monolithic `step` function.

## Differential architecture

Add one closed `rigid_world` scenario family rather than another executable or a generic DSL.

### Typed input

- stable scenario ID and schema version;
- declaration-ordered bodies and fixtures with semantic IDs;
- bounded ordered actions: create, inspect checkpoint, set transform/type/active, set fixture sensor/material/filter/density, reset/customize mass, step, and destroy;
- named expected transitions/counts for fail-closed declaration-first validation.

### Semantic output

- declaration-ordered body/fixture snapshots with exact IDs, types, active state, transforms, and mass properties;
- manager-ordered contacts identified only by oriented fixture-child semantic IDs plus occurrence ordinal;
- touching/filter/enabled state, semantic manifold points, mixed material, normal/tangent impulses, and lifecycle events;
- ordered destruction records and adapter reset/build provenance.

### Required corpus

- `non_colliding_body_fixture_lifecycle`: all body types, mutation, activation/deactivation, fixture/sensor/filter/material/mass behavior, zero contacts, and explicit destruction;
- `single_contact_lifecycle`: begin, persist, feature identity, minimal solve, warm-start carry, sensor no-manifold behavior, filter removal/reconsideration, activation destruction/recreation, fixture destruction, and body cascade/end order.

Every required transition needs a witness-registry entry and deletion/completeness tests. Compare each engine to the declaration before comparing the engines, preventing agreement on a shared omission.

### Evidence policy

- Discrete state, semantic IDs, order, multiplicity, features, branch/lifecycle states, and counts compare exactly.
- Float transport stays exact-bit. A closed `phase6-v1` registry names field-specific exact/ULP/absolute-relative policies and horizons for transforms, mass, manifold points, material, and impulses.
- D0 requires two byte-identical same-build runs. Local supported passes are D2 and cannot promote fixtures. Only the canonical pinned Linux/Clang D1 lane may promote a reviewed trace.
- Use existing compare, replay, minimization, failure-bundle, stage/review/promote, and package-isolation commands. No silent regeneration and no external scenario path.

## Key pitfalls and guards

| Pitfall | Guard |
| --- | --- |
| Treating caller-supplied `ContactSnapshot` values as real world lifecycle | Replace them with automatic broad-phase/contact-manager ownership; retain only private test helpers where necessary. |
| Exposing durable contact identity | Keep occurrence identity harness-private and consumer access borrow-scoped/owned snapshot only. |
| Recomputing mass on every density edit | Test the pinned explicit-reset asymmetry and override replacement triggers. |
| Rewriting old contact material after fixture edits | Mix at creation and test persistence until recreation. |
| Giving sensors an empty manifold and solving it | Use a separate overlap-only update path with no manifold/pre-solve/constraint. |
| Losing impulses when a point persists | Match semantic feature identity before write-back and test cold/persistent/recreated contacts. |
| Pulling the full island solver into Phase 6 | Restrict implementation and corpus to one discrete static/dynamic contact; encode deferred families in docs and tests. |
| Hash-derived order | Use arena/list/private slot order and exact ordered evidence throughout. |
| Both engines omit the same transition | Validate each trace against declaration counts/transitions before cross-engine comparison. |
| Overstating local evidence | Preserve D0/D1/D2 labels and leave platform validation unchanged after local passes. |

## Planning decomposition

Recommended dependency order:

1. body/fixture checked domain types, storage, snapshots, and public API contract;
2. proxy lifecycle plus mass/material/sensor/filter mutation semantics;
3. automatic contact manager and exact lifecycle/destruction/report integration;
4. minimal one-contact solver and warm-start persistence;
5. typed rigid-world protocol, Rust adapter, comparator policy, and fixed corpus;
6. pinned C++ adapter plus debug/release comparison, replay, determinism, and regression workflow;
7. docs, compatibility ledger, package isolation, and phase sign-off.

Plans should avoid overlapping files within the same wave. Protocol and C++ adapter work may begin after the semantic output model is fixed; documentation/sign-off remains last.

## Validation Architecture

### Per-plan verification

- Domain/API plans: focused unit tests plus public integration/compile-fail tests for invalid handles, immutable shapes, and checked definitions.
- Lifecycle plans: unit/integration tests for exact contact creation/persistence/filter/sensor/activation/destruction order and StepReport multiplicity.
- Solver plan: focused cold/warm/recreated one-contact tests with exact feature carry and finite impulse/state assertions.
- Protocol/adapters: codec boundary tests, witness-registry completeness, declaration-first rejection, Rust adapter tests, C++ protocol tests, and fake-oracle supervisor tests.
- Evidence/sign-off: debug and release compare, replay, D0 two-run determinism, package isolation, docs contract, inventory drift, and clean full Rust gates.

### Final required commands

Run the repository Rust pre-commit sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Also run the repo-owned aggregate/document/package checks and the closed rigid-world oracle debug/release comparison, replay, and determinism commands established by the implementation. Verification must distinguish a physics mismatch from harness/provenance/sanitizer failure and must leave no unreviewed generated-file drift.

## Research complete

The phase is ready for planning with the context decisions preserved and the Phase 7 solver boundary explicit.
