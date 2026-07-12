---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T02:22:54.005Z
---

# Phase 6: Minimal Rigid World Vertical Slice - Context

**Gathered:** 2026-07-11
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Deliver the smallest complete native-Rust rigid world that automatically owns body, fixture, broad-phase proxy, and contact lifecycle; exposes safe typed body and fixture operations; performs one deterministic discrete-contact solve with manifold persistence and warm-start state; and proves non-colliding and colliding scenarios through the existing Rust/C++ differential and regression-evidence pipeline. Full forces, general island solving, damping, sleeping, CCD/TOI orchestration, world queries, joints, and broad world configuration remain Phase 7 or later.

</domain>

<decisions>
## Implementation Decisions

### Body and fixture consumer contract

- **D-01:** Introduce checked owned `BodyDef` and `FixtureDef` values plus a closed `BodyType` enum. Definitions are reusable invariant-bearing inputs; `FixtureDef` owns an immutable `Shape` snapshot by value.
- **D-02:** Keep authority in granular handle-oriented `World` methods. All body/fixture inspection and mutation validates the complete world-scoped handle before effects; do not add raw storage access, durable borrows, mutable-shape access, object-style mutator façades, or general mutation batches.
- **D-03:** Support the Phase 6 subset of body state: static/kinematic/dynamic type, transform, active state, immutable identity, and the minimal internal velocity/sweep state required by one discrete-contact solve. Public forces, torques, impulses, damping, gravity scale, bullet mode, sleeping, and general velocity controls remain Phase 7.
- **D-04:** Reject non-finite body state, negative density/friction/restitution, invalid centered inertia, invalid topology, and invalid handles with typed errors before allocation or mutation. Do not clamp invalid values; preserve pinned behavior and expression order for accepted values.
- **D-05:** Preserve pinned proxy/contact side effects: transform changes synchronize proxies and defer contact discovery/update to stepping; deactivation destroys contacts and proxies; activation recreates proxies; type changes destroy contacts and touch proxies for reconsideration.
- **D-06:** Preserve asymmetric mass semantics exactly. Positive-density fixture creation resets body mass; fixture destruction always resets it; changing fixture density does not reset mass until explicit `reset_mass_data`; a custom mass value is a current override, not a persistent mode, and later reset-triggering fixture/type/fixed-rotation operations replace it. Custom mass changes are no-ops for static and kinematic bodies.
- **D-07:** Sensor changes wake the parent and affect the next contact update. Filter changes flag existing contacts and touch proxies. Friction and restitution changes affect contacts created afterward and do not rewrite material already mixed into an existing contact.

### Automatic contact lifecycle and minimal solve

- **D-08:** Replace the Phase 3 caller-supplied representative contact list with a private world-owned contact manager that consumes Phase 5 ordered broad-phase pairs. It creates, persists, filters, updates, and destroys contacts automatically in pinned list/occurrence order.
- **D-09:** Keep contacts transient and private with no public durable contact handle. Differential diagnostics may identify one occurrence by oriented fixture-child semantic IDs plus a creation ordinal, but that identity is harness-private and never becomes a consumer storage promise.
- **D-10:** Contact updates use Phase 5 canonical pair dispatch and semantic feature identity. Manifold type, point order, add/persist/remove states, touching transitions, enabled/filter state, and lifecycle event order are exact structural evidence.
- **D-11:** Mix friction with the pinned geometric-mean formula and restitution with the pinned maximum formula when a contact is created. Retain per-contact mixed values until the contact itself changes them or is recreated; fixture material edits do not retroactively rewrite an existing contact.
- **D-12:** Transfer warm-start normal and tangent impulses across persistent points by semantic contact feature identity. New and sensor points begin with zero impulses. One deterministic discrete static/dynamic contact is solved far enough to consume and refresh those impulses; a complete multi-contact island solver is explicitly deferred.
- **D-13:** Sensor contacts test overlap and emit touching lifecycle transitions without a manifold, pre-solve call, or constraint solve. Preserve pinned begin/end and sensor-change timing rather than normalizing all contact kinds to one callback path.
- **D-14:** Evolve the existing borrow-scoped `ContactView`, restricted `StepHook`, and owned `StepReport` instead of adding a second listener model. Reports expose owned begin/persist/end, manifold/material/warm-start, command, and destruction evidence in exact occurrence order and multiplicity; hooks still receive no `&mut World` and deferred commands still apply only after unlock.
- **D-15:** Contact destruction caused by filtering, deactivation, fixture destruction, or body cascades emits required end/destruction evidence before dependent fixture/body invalidation. All adjacency, proxy, contact, and handle changes occur through centralized validate-then-commit world transitions.

### Minimal rigid-world differential evidence

- **D-16:** Extend the existing semantic protocol with one bounded, typed rigid-world lifecycle timeline rather than isolated operation probes or a general future-world DSL. Declarations are separate from ordered actions, and every action has an explicit checkpoint or named harness phase where needed for first-divergence localization.
- **D-17:** Require two fail-closed top-level witness families. `non_colliding_body_fixture_lifecycle` covers all three body types, transform/type/activation mutation, fixture/sensor/material/filter properties, mass reset/override behavior, a zero-contact step, and explicit destruction. `single_contact_lifecycle` covers create/begin, persist, manifold identity, one minimal solve, warm-start carry, sensor touching without manifold, filter removal/reconsideration, activation-driven destruction/recreation, fixture destruction, and body-cascade/end ordering.
- **D-18:** Use scenario-authored body and fixture semantic IDs. Snapshot bodies and fixtures in declaration order; preserve contact-manager order, manifold-point order, callback/report order, multiplicity, and destruction order exactly. Declaration-first expected counts and transitions must fail when both engines omit the same required behavior.
- **D-19:** Compare discrete identity, structure, order, branch/lifecycle state, feature identity, and counts exactly. Transport all floats as exact bits and apply a closed `phase6-v1` per-observable policy for physical state, material, manifold, and impulse values; never add a global epsilon or iteration-scaled widening.
- **D-20:** Reuse the established compare, replay, first-divergence, minimization, staging, review, promotion, and D0 two-run determinism paths. Local supported-toolchain passes are D2 evidence only; canonical trace promotion requires the pinned Linux x86_64/Clang D1 lane.

### the agent's Discretion

- Exact private module split for body, fixture, contact-manager, contact solver, and rigid-world adapter code, provided the `world` module remains cohesive and repository file/function size triggers are respected.
- Exact public accessor and typed error names within the locked checked-definition and handle-oriented contract.
- Exact bounded rigid-world action/checkpoint record names and corpus size, provided both required witness families and every declared lifecycle transition are fail-closed.
- Exact field-specific numeric thresholds, provided each is justified by pinned-source analysis and canonical evidence under the Phase 4 policy.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and inherited contracts

- `.planning/ROADMAP.md` § Phase 6 and Phase 7 — fixed vertical-slice goal, success criteria, research flags, and deferred complete-solver boundary.
- `.planning/REQUIREMENTS.md` § `COLL-05`, `RIGD-01`, `RIGD-02`, and `RIGD-04` — required contact, body, fixture, and minimal solve behavior.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — protocol, provenance, failure, replay, minimization, and evidence-lifecycle contracts.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — world-scoped handles, transient contacts, restricted hooks, deferred commands, destruction ordering, and poisoning decisions.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — source-ordered math, field policies, non-finite rules, ordering, horizons, and D0-D3 authority.
- `.planning/phases/05-shapes-and-collision-foundation/05-CONTEXT.md` — immutable shapes, pair registry, semantic manifolds, ordered broad-phase pairs, pure filtering/refiltering, and Phase 6 handoff.

### Pinned body, fixture, contact, and world behavior

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.h` and `b2Body.cpp` — definitions, type/transform/activation mutations, fixture lifecycle, mass reset/override, proxy synchronization, and contact side effects.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Fixture.h` and `b2Fixture.cpp` — owned shape cloning, material/sensor/filter state, proxy lifecycle, refiltering, and fixture queries.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2ContactManager.h` and `b2ContactManager.cpp` — ordered pair admission, duplicate suppression, contact list insertion, filtering, update, discovery, and destruction.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Contacts/b2Contact.h` and `b2Contact.cpp` — pair registration, material mixing, touching/sensor update paths, feature persistence, warm-start transfer, and listener timing.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Contacts/b2ContactSolver.h` and `b2ContactSolver.cpp` — minimal discrete constraint initialization, warm starting, impulse storage, and source ordering.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.h`, `b2World.cpp`, and `b2WorldCallbacks.h` — world locking, step phase order, create/destroy cascades, and callback mutation restrictions.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter06_Bodies.md`, `Chapter07_Fixtures.md`, `Chapter09_Contacts.md`, and `Chapter10_World.md` — consumer semantics and documented lifecycle restrictions.

### Existing Rust and evidence seams

- `crates/liquidfun/src/world.rs`, `crates/liquidfun/src/world/object.rs`, and `crates/liquidfun/src/world/step.rs` — Phase 3 object arenas, destruction transactions, transient contact views, hook lock, deferred commands, and owned reports to evolve.
- `crates/liquidfun/src/collision/broad_phase.rs` — ordered pair generation and filter/refilter substrate.
- `crates/liquidfun/src/collision/narrow.rs` and `crates/liquidfun/src/collision/types.rs` — closed pair dispatch, semantic manifolds/features, point states, and world-manifold conversion.
- `crates/liquidfun-test-protocol/src/scenario.rs` and `crates/liquidfun-test-protocol/src/tolerance/policy.rs` — bounded scenario and closed comparison-policy extension points.
- `crates/liquidfun-differential/src/runner.rs`, `comparator.rs`, `fixtures.rs`, and `failure_bundle.rs` — permanent compare/replay/minimize/promote and first-divergence paths.
- `tools/reference/src/oracle_adapter.cpp` and `tools/reference/src/collision_probe.cpp` — pinned C++ adapter and closed probe patterns to extend without a second oracle path.
- `ARCHITECTURE.md`, `TESTING.md`, and `COMPATIBILITY.md` — current public/evidence boundaries and truthful incomplete-parity reporting.
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` — required type, module, test, and verification rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `World` already owns typed generational body/fixture arenas, newest-first adjacency, centralized destruction, borrow-scoped contact views, hook locking, bounded reports, and deferred command application.
- Phase 5 supplies immutable `Shape` values, checked mass data, semantic manifolds/features, world-manifold conversion, ordered `BroadPhase` pairs, and exact pure filter/refilter behavior.
- The private protocol/differential crates already provide exact float transport, typed comparison, D0 replay, first-divergence signatures, failure bundles, minimization, and reviewed fixture promotion.
- The C++ reference adapter already has strict stdout discipline, build identity, closed request dispatch, and collision-probe translation units.

### Established Patterns

- Production state stays in one safe native-Rust published crate; private harness and C++ code depend inward and never enter consumer builds.
- Boundary values are checked before mutation, internal storage remains private, contacts are transient, and solver-visible order is explicit rather than hash-derived.
- Machine-readable declarations and evidence are authoritative, generated docs are presentation, and local D2 passes never become canonical D1 claims.

### Integration Points

- Deepen `crates/liquidfun/src/world.rs` and `world/` children instead of creating a second published dynamics crate or a parallel world model.
- Connect fixture proxies to `BroadPhase`, contact updates to the closed Phase 5 pair registry, and destruction to the existing centralized world transactions.
- Extend the current protocol, Rust adapter, C++ adapter, comparator, xtask commands, and evidence docs with a closed rigid-world scenario family.

</code_context>

<specifics>
## Specific Ideas

- Treat Phase 6 as one lifecycle timeline, not a bag of independent probes: the same contact must be observed across create, persist, sensor/filter/activation transitions, warm-start carry, and destruction.
- Preserve the pinned asymmetry that fixture density changes do not automatically recompute body mass, while fixture creation/destruction and explicit reset do.
- A single deterministic static/dynamic contact is the solver witness. It proves contact impulses are consumed and carried without claiming the complete island solver that Phase 7 owns.
- Keep contact occurrence identity semantic and harness-private so differential evidence never hardens a public durable contact handle.

</specifics>

<deferred>
## Deferred Ideas

- Forces, torques, impulses, damping, gravity scale, fixed rotation, bullet mode, public velocity controls, and complete island velocity/position solving — Phase 7.
- Sleeping/waking policy, continuous world collision/TOI orchestration, sub-stepping, world AABB queries/rays, origin shifting, and broad world configuration — Phase 7.
- Joint collision suppression and complete joint behavior — Phase 8.
- A general future-world action DSL, randomized rigid corpora, multi-contact stacks, and broad rigid sign-off — later phases after their source audits.
- Public durable contact identity or mutable shape storage — intentionally excluded by prior object-model and shape decisions.

</deferred>

***

*Phase: 06-minimal-rigid-world-vertical-slice*
*Context gathered: 2026-07-11*
