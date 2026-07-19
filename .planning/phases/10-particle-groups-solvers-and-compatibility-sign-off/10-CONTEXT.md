---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T05:17:27.914Z
---

# Phase 10: Particle Groups, Solvers, and Compatibility Sign-Off - Context

**Gathered:** 2026-07-19
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Complete the native Rust particle engine by adding safe group construction, inspection, mutation, topology, pair and triad behavior, and every baseline and particle-flag solver pass in the pinned upstream order. Close the phase with focused unit, integration, property, deterministic, sanitizer, and semantic differential evidence that individually signs off every particle flag, unflagged pass, group behavior, and inherited particle path. Renderer-neutral examples and testbed work remain Phase 11; performance, broader portability, fuzzing breadth, and release hardening remain Phase 12.

</domain>

<decisions>
## Implementation Decisions

### Particle-group public contract

- **D-01:** Introduce one cohesive public particle-group module with owned, typed, fixed-order creation recipes rather than a nullable C++-shaped definition. Represent filled shapes, stroke shapes, and explicit positions as invariant-bearing source types whose evaluation order is fixed and documented.
- **D-02:** Model the destination separately as either a new group or an explicit append target carrying a live same-system `ParticleGroupId`. Do not conflate the existing-group target with the particle sources or permit contradictory source combinations.
- **D-03:** Preserve the pinned group definition semantics for particle flags, group flags, transform, linear velocity, angular velocity, color, strength, stride, lifetime, user association, and source sampling while using owned safe Rust data and no borrowed raw buffers.
- **D-04:** Expose a borrow-scoped `ParticleGroupView`-style contract with the stable group ID, public flags, transform, center, linear and angular velocity, mass, inertia, member count, stable member `ParticleId` values, and aligned depth values where applicable. Keep dense row numbers, internal flags, cached statistics, and mutable storage private.
- **D-05:** Treat every group creation as a complete validate-then-commit transaction. Sampling, capacity, handle, allocation, topology, or invariant failure creates no particles or group identity, emits no lifecycle occurrence, and leaves every lane and cache unchanged.
- **D-06:** Joining preserves group A's identity and every particle identity, rotates the dense ranges in the pinned order, unions the required flags, generates only the source-equivalent cross-group connections, and invalidates group B only after the transaction commits.
- **D-07:** Splitting preserves the original group identity for the pinned first longest connected component, allocates new group IDs for later components in source component order, preserves every particle ID, and reproduces the pinned group and particle ordering without public index churn.
- **D-08:** Destroying a group's particles uses the established zombie and deferred-compaction lifecycle. Invalidate the group after its last member is removed unless the pinned can-be-empty flag retains it; retained empty groups remain valid and inspectable with source-equivalent zero-valued aggregate state.

### Topology and group mutation ordering

- **D-09:** Keep `ParticleStorage` as the single state authority. Add pure private topology-planning kernels beneath it, but commit group ranges, dense permutations, depth, rigid caches, pairs, triads, particle flags, and group metadata through one storage-owned source-order mutation candidate.
- **D-10:** Do not introduce a separately mutable topology graph or persist public stable IDs inside solver-significant topology records. Internal topology may use private dense references while every public and protocol boundary translates them to checked stable semantic IDs.
- **D-11:** Encode each pinned operation explicitly instead of using a generic “recompute topology” fallback. Ordinary buffer rotations remap existing pairs and triads without sorting or rebuilding historical rest values.
- **D-12:** Generate Voronoi seeds in current dense order and consume nodes in the pinned row-major order. Apply the upstream connection filters, distance tests, group-flag gates, and edge-case handling before producing pair or triad candidates.
- **D-13:** When topology generation appends new pairs or triads, preserve the pinned orientation, stable ordering, duplicate policy, strength, rest distance, and triad coefficients. Stable-sort and retain the first duplicate only at the exact source operations that do so.
- **D-14:** Joining rotates first and generates only cross-boundary constraints; splitting retargets surviving historical records to the resulting groups rather than regenerating their rest state; reactive regeneration clears the reactive flag only after pair and triad updates complete.
- **D-15:** Solid depth and rigid-group state are invalidated, recomputed, and advanced only at their pinned points. A failed group mutation may not expose partially recomputed depth, center, mass, inertia, transform, or rigid velocity.

### Pinned solver graph and flag behavior

- **D-16:** Define one private, closed, versioned `phase10-pass-graph-v1` manifest derived from the pinned `b2ParticleSystem::Solve` call graph. The manifest owns pass IDs, gates, multiplicity, and order; unknown, missing, duplicated, or reordered passes are failures.
- **D-17:** Preserve the outer ordering around sub-iterations: lifetime solving, zombie compaction, and all-flags refresh occur at their pinned points before the pause gate; paused systems skip the solver without fabricating group, contact, topology, or lifecycle changes.
- **D-18:** Within each particle sub-iteration, preserve the pinned order for contact and body-contact refresh, weight, conditional depth and reactive-topology updates, force and flag-driven passes, gravity, pressure and damping families, elastic and spring constraints, velocity limiting, rigid damping, barrier and collision response, rigid motion, wall enforcement, and final position integration. Research must transcribe the exact source graph and guards into the manifest before implementation begins.
- **D-19:** Implement every unflagged baseline pass and every public flag-driven behavior as a named cohesive kernel around the existing authoritative storage. Do not collapse materially different flags behind one generic approximate behavior.
- **D-20:** Cover water, wall, spring, elastic, viscous, powder, tensile, color mixing, barrier, static pressure, reactive, and repulsive behavior, plus the solid and rigid group flags and every interaction that changes pass admission or equations.
- **D-21:** Preserve interaction rules explicitly, including powder and tensile pressure suppression, static-pressure extra damping, reactive pair and triad regeneration and clearing, spring-pair and elastic-triad constraints, color mixing's both-particles gate, cross-group repulsion, barrier/wall behavior, and solid/rigid group effects.
- **D-22:** Keep zero-valued water a first-class compatibility leaf even though it has no bit to test. Keep Phase 9-owned zombie, destruction-listener, contact-filter, and contact-listener flags in the Phase 10 closure ledger without reassigning their implementation ownership.
- **D-23:** Preserve source-significant particle-system, group, particle, contact, pair, triad, and solver order. Never use hash iteration, default parallelism, broad canonicalization, fast-math, or a global tolerance to hide ordering or numerical divergence.
- **D-24:** Validate public mutations and solver inputs before effects. Non-finite values, invalid handles, wrong systems, invalid ranges, locked or poisoned worlds, capacity failures, and topology failures remain typed and transactional under the Phase 9 contracts.

### Testing, differential evidence, and sign-off

- **D-25:** Extend the existing long-lived Phase 9 rigid-world protocol, native adapter, C++ oracle, comparator, replay, and evidence pipeline. Do not create a parallel particle-group or solver harness.
- **D-26:** Give every particle flag, zero-valued water behavior, unflagged solver pass, group flag, group mutation, topology operation, and inherited lifecycle, buffer, contact, query, and callback path an individual closed ledger leaf with explicit implementation, test, witness, policy, and evidence references.
- **D-27:** Native tests may expose private test-only pass IDs to compare exact pass admission, multiplicity, and order against `phase10-pass-graph-v1`; those IDs are not public API and do not substitute for semantic differential evidence.
- **D-28:** For every flag or pass, include a control witness where the branch is inactive and an activation witness proving its semantic effect. Add bounded interaction witnesses wherever a single-flag case cannot prove the pinned branch or ordering behavior.
- **D-29:** Compare structural fields, stable IDs, flags, membership, counts, branch states, pass traces, order, and multiplicity exactly. Assign exact-bit, ULP, absolute-relative, or dimensioned-absolute policies only to named numeric paths with fixed horizons and source/evidence justification.
- **D-30:** TEST-01, TEST-02, and TEST-04 close through explicit leaf-to-test mappings: focused pure-kernel unit tests, public world/group/particle integration workflows, and reproducible property models for permutations, connectivity, topology, handles, geometry, queries, and world operation sequences.
- **D-31:** Retain all Phase 6 through 9 witness families and evidence authority unchanged. Unknown Phase 10 leaves, pass IDs, observations, policies, flags, group behaviors, or missing declarations are harness failures.
- **D-32:** D0 requires byte-identical same-build traces with nondeterministic timing excluded. Only actual pinned Linux x86_64 Rust 1.97.0/Clang 22.1.8 D1 evidence may promote Phase 10 leaves. D2 remains non-promotable supported-platform evidence and D3 remains diagnostic.
- **D-33:** Promote compatibility rows only from a complete, current, same-run authority set after debug, release, replay, D0, sanitizer, exact-ref, schema, provenance, and deterministic-report checks pass. Partial evidence may improve implementation or unit-test states but cannot sign off parity.
- **D-34:** Phase completion requires every Phase 10 leaf to have an explicit supported, documented-difference, or intentionally-unsupported outcome. Do not claim complete particle parity, examples/testbed maturity, performance, broad platform support, or v1 release readiness beyond the exact evidence achieved.

### the agent's Discretion

- Exact public and private type, module, method, error, view, recipe, transaction-candidate, pass-ID, witness-family, and ledger-leaf names within the locked contracts.
- Exact decomposition across plans and cohesive child modules, provided `ParticleStorage` remains the single authority and source operations remain independently auditable.
- Exact bounded corpus sizes, property-case counts, source-derived capacity bounds, and named numerical thresholds when justified by pinned-source analysis and canonical evidence.
- Whether test-only pass tracing is compiled under `cfg(test)` or an unpublished tooling feature, provided it cannot enter the published public API or become the sole differential oracle.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and inherited decisions

- `.planning/PROJECT.md` — Native-Rust, oracle-isolation, safety, determinism, API, testing, and truthfulness constraints.
- `.planning/REQUIREMENTS.md` — `PART-09` through `PART-13`, `PART-18`, `TEST-01`, `TEST-02`, and `TEST-04` acceptance requirements.
- `.planning/ROADMAP.md` — Fixed Phase 10 boundary, dependency on Phase 9, success criteria, and research flags.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — Stable group and particle identities, transactionality, destruction state, borrow-scoped access, and opaque storage.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — Scalar expression order, closed numerical policies, D0-D3 authority, and failure taxonomy.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-CONTEXT.md` — Authoritative particle storage, permutation, lifecycle, contacts, queries, callbacks, and explicit Phase 10 boundary.

### Project evidence and implementation seams

- `ARCHITECTURE.md` — Current world, particle storage, protocol, oracle, callback, solver, and renderer-independence boundaries.
- `COMPATIBILITY.md` — Evidence-state vocabulary and current particle inventory claims.
- `TESTING.md` — Unit, integration, property, differential, replay, D0, sanitizer, D1/D2, and promotion rules.
- `UPSTREAM.md` — Pinned revision provenance, read-only oracle policy, and source-reference rules.
- `reference/compatibility.json` — Machine-authoritative compatibility leaves and evidence references.
- `crates/liquidfun/src/identity.rs` — World-scoped `ParticleGroupId` and `ParticleId` contracts.
- `crates/liquidfun/src/particle/storage.rs` and `crates/liquidfun/src/particle/storage/` — Single storage authority, group ranges, permutations, lanes, topology record storage, and property models.
- `crates/liquidfun/src/particle/view.rs` — Existing borrow-scoped particle, pair, triad, and contact view patterns.
- `crates/liquidfun/src/world/object.rs` and `crates/liquidfun/src/world/particle_object.rs` — Existing group arena seams, lifecycle snapshots, and particle-system public operations.
- `crates/liquidfun/src/world/step.rs` — Source-timed rigid and particle step integration, lock, hook, journal, and ordering seams.
- `crates/liquidfun-test-protocol/src/schema/rigid_world/phase9.rs` — Existing closed particle protocol schema to extend.
- `crates/liquidfun-differential/src/rigid_world/phase9.rs` — Existing native particle adapter and semantic witness seam.
- `tools/reference/src/rigid_world_phase9_decode.hpp` and `tools/reference/src/rigid_world_phase9_execute.hpp` — Existing bounded C++ particle oracle boundary to extend.

### Pinned upstream particle oracle

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter11_Particles.md` — Public particle-group concepts, flags, creation, behavior, and consumer expectations.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2Particle.h` and `b2Particle.cpp` — Complete particle flag values and pair/triad semantics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleGroup.h` and `b2ParticleGroup.cpp` — Group definitions, flags, inspection, statistics, transforms, and rigid motion.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h` — Complete public/private group, topology, solver, and buffer contract.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp` — Group creation/join/split, rotations, topology, depth, complete solver graph, gates, equations, and pass order.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2VoronoiDiagram.h` and `b2VoronoiDiagram.cpp` — Generator order, grid construction, node emission, and topology edge cases.
- `third_party/liquidfun/liquidfun/Box2D/Unittests/` — Upstream particle and group test intent to inventory and map to focused evidence.

### Repository standards

- `AGENTS.md` and `AGENTS.bright-builds.md` — Repo-local workflow, Rust, verification, task-artifact, and architecture requirements.
- `standards-overrides.md` — Local exception registry; no substantive active override replaces the defaults.
- `standards/core/architecture.md` — Single authority, invariant-bearing domain types, and functional-core/imperative-shell guidance.
- `standards/core/code-shape.md` — Cohesive modules, shallow control flow, optional naming, and diagnosable tooling.
- `standards/core/testing.md` — Focused behavior tests with Arrange/Act/Assert structure.
- `standards/core/verification.md` — Repository-native and pre-commit verification requirements.
- `standards/languages/rust.md` — Rust module, error, guard, domain-type, and verification guidance.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- `ParticleStorage` already carries stable particle identities, contiguous group ranges, pairs, triads, depth-capable lanes, atomic permutations, fixed/growable capacity, and property-tested invariants.
- `ParticleGroupId` and the world group arena already provide typed same-world identity, lifecycle snapshots, association cleanup, and an explicit placeholder seam to deepen.
- Phase 9 already provides safe particle/system views, source-timed contacts and callbacks, force/query/statistics APIs, exact semantic IDs, and a bounded long-lived differential protocol.
- The Phase 4 and Phase 6-9 evidence stack already provides exact float transport, closed policy registries, replay, deterministic traces, provenance, sanitizer lanes, exact-ref validation, and compatibility generation.

### Established Patterns

- Validate a complete candidate before mutation and commit all affected lanes, identities, caches, topology, and lifecycle evidence together.
- Preserve solver-significant source order in production and callbacks; canonicalize only explicitly unordered evidence at collection boundaries.
- Keep dense indices, scratch buffers, raw pointers, internal flags, and pass tracing private; expose stable IDs and owned or borrow-scoped semantic records.
- Extend one deep native Rust engine and one existing oracle/protocol stack instead of creating parallel implementations.

### Integration Points

- Replace the placeholder group object with authoritative group metadata coordinated with each system's `ParticleStorage`.
- Add public group recipes, views, and mutation APIs alongside the existing particle definitions and views.
- Extend the storage permutation inventory and transaction candidates with topology, depth, rigid-group, and solver state.
- Insert named Phase 10 kernels into the existing `World::step` particle slice at the transcribed pinned pass points.
- Extend Phase 9 protocol, native/C++ adapters, policies, witnesses, exact-ref evidence, inventory leaves, compatibility generation, and CI workflow coverage.

</code-context>

<specifics>
## Specific Ideas

- Preserve the original group ID for group A on join and for the first longest connected component on split; preserve every particle ID throughout.
- Treat historical pair and triad rest data as state that rotations and splits preserve, not values that an implementation-convenient rebuild may replace.
- Make the solver pass graph and every compatibility leaf machine-checkable, while keeping pass IDs private and semantic outcomes authoritative for cross-engine evidence.
- Use control-plus-activation witnesses and explicit interaction witnesses so a nominally covered flag cannot pass without exercising its behavior.

</specifics>

<deferred>
## Deferred Ideas

- Upstream example/testbed accounting, the renderer-neutral scenario catalog, headless controls, debug drawing, and optional visualization — Phase 11.
- Performance budgets, profiling-led optimization, fuzzing breadth, Miri/sanitizer expansion, broad platform evidence, coverage policy, release documentation, packaging, and v1 audit — Phase 12.
- Generic allocator traits, GPU storage, unsafe raw-buffer interoperability, SIMD, parallel stepping, and alternate precision modes — only after measured need and separate compatibility/safety decisions.

</deferred>

***

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Context gathered: 2026-07-19*
