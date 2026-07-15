---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T02:58:57.302Z
---

# Phase 9: Particle Storage, Lifecycle, and Coupling - Context

**Gathered:** 2026-07-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Implement the safe, identity-preserving particle-system foundation: multiple configurable systems; particle creation, inspection, mutation, forces, impulses, lifetimes, zombie destruction, capacity handling, owned buffer equivalents, spatial proxies, particle and fixture/body contacts, strict-contact pruning, callbacks and filters, statistics, AABB queries, ray casts, and source-timed rigid coupling. Phase 9 must deepen the existing architecture spike into production APIs and fail-closed differential evidence. Particle-group construction/topology, pair and triad generation semantics, and baseline or flag-driven particle solver behaviors remain Phase 10 work.

</domain>

<decisions>
## Implementation Decisions

### Storage authority, identity, and bulk access

- **D-01:** Replace the placeholder split particle ownership with one authoritative `ParticleStorage` per live particle system. Stable world- and system-scoped `ParticleId` values map to private ephemeral dense rows; the separate placeholder `World.particles` arena must not remain a second source of truth.
- **D-02:** Retain the Phase 3 identity state machine: live, pending-delete, vacant, and retired. Dense sorting, rotation, and compaction never change a live public ID, and checked generation retirement prevents resurrection.
- **D-03:** Make one validate-then-commit permutation transaction the only way to reorder or compact particle rows. It must update every required and allocated optional SoA lane, dense/ID map, proxy, particle contact, body contact, pair, triad, lifetime-order entry, and group range atomically or leave the system unchanged.
- **D-04:** Expose immutable borrow-scoped aggregate views over positions, velocities, colors, weights, flags, group identities, user associations, contacts, body contacts, pairs, triads, and expiration ordering. The borrow prevents structural mutation while slices are live; dense row numbers and internal capacities remain private.
- **D-05:** Expose supported mutation through narrow typed operations or closure-scoped editors. Position edits must invalidate or rebuild all spatial/contact-derived state before the mutation scope returns; flags, groups, lifetimes, identities, and derived records are not raw mutable slices.
- **D-06:** Use an owned unified SoA lane bundle with explicit growable and fixed-capacity modes as the safe external-buffer equivalent. Construction validates complete lane lengths/capacities before adoption, and teardown returns owned supplied lanes without raw pointers or arbitrary borrowed lifetimes.
- **D-07:** Keep declared particle capacity distinct from allocator capacity. Fixed-capacity systems fail explicitly when full and never silently reallocate; growable systems preserve upstream maximum-count and oldest-destruction behavior. Do not generalize to a trait-based allocator backend until measured need proves another materially different backing strategy.
- **D-08:** Preserve optional-lane laziness only where the pinned source does. Allocating, permuting, clearing, or tearing down an optional lane must be covered by the same invariant checks as required lanes; no lane may be omitted from a permutation site.

### Lifetime, zombies, eviction, and destruction

- **D-09:** Use a storage-authoritative two-phase lifecycle. Explicit destruction, expiration, and capacity eviction mark a row zombie/pending-delete first; ordinary access and mutation then return the typed pending-delete error while an owned destruction snapshot remains available.
- **D-10:** Mirror the pinned lifetime clock and quantization: a 32.32 elapsed-time accumulator, `i32` quantized expiration values, source-equivalent truncation toward zero, finite-before-infinite oldest selection, dirty expiration ordering, and lifetime solving before zombie compaction.
- **D-11:** During ordinary compaction, scan old dense indices in ascending order, append each requested particle-destruction listener occurrence at its source-equivalent point, then invalidate/retire the stable identity and apply the single authoritative survivor permutation.
- **D-12:** When creation reaches a configured maximum and destroy-by-age is enabled, select the upstream-equivalent oldest particle, mark it without fabricating a destruction-listener occurrence, run the same compactor immediately to free capacity, then create the new particle transactionally.
- **D-13:** Equal quantized-expiration ties require an explicit canonical-oracle witness before a deterministic tie rule is locked. Rust sort behavior, hash order, or an implementation-convenient ordinal may not become accidental compatibility behavior.
- **D-14:** Destroying a particle system captures complete system/particle snapshots first, then emits source-equivalent particle destruction and system teardown evidence without leaving handles, group ranges, supplied lane ownership, or rigid-contact references dangling.

### Contacts, filters, listeners, and step timing

- **D-15:** Execute the Phase 9 particle slice inside `World::step` at the pinned source location and in newest-first particle-system order. Do not add a public out-of-band contact refresh or particle-only tick that can observe stale rigid state.
- **D-16:** Preserve the pinned per-sub-iteration order for the Phase 9 slice: proxy update/sort, neighborhood and particle-contact generation, particle contact filtering and listener diffing, fixture/body contact generation and filtering, strict-contact pruning, body-contact listener diffing, then the Phase 9 coupling/state updates that are actually in scope.
- **D-17:** Extend the existing single borrowed synchronous decision hook rather than registering persistent or multiple decision makers. Add distinct borrow-scoped particle-pair and fixture-particle views while keeping `World` and reusable contact handles inaccessible.
- **D-18:** Keep one internal source-timed lifecycle journal for rigid and particle occurrences. Owned reports and convenience slices are projections of that journal and may not reconstruct or regroup cross-domain order after stepping.
- **D-19:** Particle contact-listener and contact-filter flags gate invocation exactly where the pinned source checks them. Preserve order, multiplicity, begin/end diff behavior, repeated occurrences, and strict versus non-strict pruning; dense indices are translated to stable IDs at every public and evidence boundary.
- **D-20:** A synchronous particle hook panic follows the established Phase 8 contract: restore the world lock, discard unapplied deferred commands, preserve coherent state, poison future coherent operations, and resume unwinding.
- **D-21:** Paused particle systems accept ordinary configuration and direct API calls but are skipped by `World::step` exactly as the pinned source specifies; pause must not fabricate contact transitions or lifecycle events.

### Forces, rigid coupling, statistics, and queries

- **D-22:** Provide checked per-particle and contiguous-range force/impulse APIs over stable identities or validated borrow-scoped ranges. Validate all handles, finite values, wall-particle restrictions, and derived mass/distribution calculations before mutation; failure is transactional.
- **D-23:** Implement the Phase 9 body-contact data and rigid reaction coupling required by the pinned particle step without claiming Phase 10 particle solver completion. Preserve body/fixture semantic identity, contact order, effective mass/weight/normal fields, and rigid wake/force effects that are observable in this phase.
- **D-24:** Expose collision energy, contact counts, particle/system counts, stuck-particle candidates, paused state, capacity/maximum state, and other Phase 9 statistics as owned or borrow-scoped semantic data. Internal iteration counters and scratch storage remain private unless required by a named observable.
- **D-25:** Add per-system and mixed-world AABB query APIs using the existing typed continue/terminate control. World queries preserve the pinned fixture-before-particle-system traversal relationship and per-system culling without promising callback order that upstream leaves unspecified.
- **D-26:** Add per-system and mixed-world particle ray casts with typed ignore, terminate, continue-without-clipping, and clip-to-checked-fraction directives. Preserve start-inside exclusion, clipping, early termination, culling, repeated occurrences, and invalid-directive no-effect failure; canonicalization belongs only in evidence collectors.

### Differential evidence and phase truthfulness

- **D-27:** Extend the existing long-lived rigid-world protocol and oracle process rather than create a second harness. Add bounded particle-system definitions, actions, checkpoints, stable semantic IDs, callback occurrences, queries, statistics, and mixed rigid/particle observations.
- **D-28:** Define a closed `phase9-v1` witness and numerical-policy registry. It must declare every Phase 9 storage, capacity, permutation, lifetime, zombie, contact, strict-contact, callback-flag, force/impulse, statistic, query, culling, and rigid-coupling branch; unknown Phase 9 observations are harness failures.
- **D-29:** Retain all Phase 6 through 8 witness families unchanged. Compare identities, flags, counts, branch states, order, multiplicity, listener/filter decisions, query completion, and lifecycle structure exactly; assign exact-bit, ULP, absolute-relative, or dimensioned absolute policies only to named float paths.
- **D-30:** Protocol and comparator declarations must reject group-topology, pair/triad-generation, and baseline or flag-driven solver observations as undeclared Phase 10 work. A passing Phase 9 corpus may claim only the closed particle storage/lifecycle/contact/query/rigid-coupling foundation.
- **D-31:** D0 still requires byte-identical same-build traces, and only actual pinned Linux x86_64 Rust 1.97.0/Clang 22.1.8 D1 evidence may promote the scoped Phase 9 claim. Local supported-platform passes remain D2 and cannot self-bless exact fixtures or parity status.

### the agent's Discretion

- Exact public/private type, module, method, error, view, editor, event, statistic, query, and witness-family names within the locked contracts.
- Exact plan decomposition and whether the production storage deepens the existing spike in place or migrates its proved pieces into new child modules, provided there is one authority and no parallel implementation.
- Exact bounded capacities, corpus sizes, stuck-candidate thresholds, and named field-specific numerical policies when derived from pinned-source analysis and canonical evidence.
- Exact mutation-editor granularity, provided every affected derived state is restored before the editor returns and no raw mutable lane can escape its borrow.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked architecture

- `.planning/PROJECT.md` — Native-Rust, oracle-isolation, stable-identity, safety, determinism, and truthfulness constraints.
- `.planning/REQUIREMENTS.md` — `API-09`, `API-10`, `PART-01` through `PART-08`, and `PART-14` through `PART-17` acceptance requirements.
- `.planning/ROADMAP.md` — Fixed Phase 9 boundary, dependency on Phase 8, success criteria, and audit flags.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — Stable particle identity, pending-delete state, transactional permutations, borrow-scoped access, and owned lane direction.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — Scalar ordering, closed numeric policies, and D0-D3 evidence authority.
- `.planning/phases/06-minimal-rigid-world-vertical-slice/06-CONTEXT.md` — World ownership, contact lifecycle, destruction evidence, hook/report contracts, and bounded differential scenarios.
- `.planning/phases/07-rigid-solver-world-operations-and-ccd/07-CONTEXT.md` — Source-ordered stepping, force/impulse transactionality, typed query/ray directives, and evidence canonicalization rules.
- `.planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-CONTEXT.md` — Single decision hook, authoritative lifecycle journal, panic poisoning, callback timing, and D1 sign-off rules.

### Project evidence and current implementation

- `ARCHITECTURE.md` — Current particle identity/storage spike, world ownership, lifecycle, buffer, callback, oracle, and renderer-independence boundaries.
- `COMPATIBILITY.md` — Particle inventory rows and evidence-status vocabulary that Phase 9 must update truthfully.
- `TESTING.md` — Unit/property/differential/replay/determinism/sanitizer and evidence-authority rules.
- `UPSTREAM.md` — Pinned revision provenance and source-reference rules.
- `crates/liquidfun/src/particle/storage.rs` and `crates/liquidfun/src/particle/storage/` — Existing executable SoA, identity, permutation, optional-lane, group-range, capacity, and property-test architecture evidence.
- `crates/liquidfun/src/world/object.rs` — Placeholder particle-system/group/particle ownership and destruction seams that must converge on one authority.
- `crates/liquidfun/src/world/step.rs`, `crates/liquidfun/src/world/query.rs`, and `crates/liquidfun/src/world/contact_manager.rs` — Existing lock, hook, lifecycle, query/ray, and rigid-contact integration patterns.
- `crates/liquidfun-test-protocol/src/schema/rigid_world.rs`, `crates/liquidfun-differential/src/rigid_world.rs`, and `tools/reference/src/rigid_world.cpp` — Existing bounded semantic protocol, native adapter, comparator, and pinned oracle extension seams.

### Pinned upstream particle oracle

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter11_Particles.md` — Public particle concepts, self-compacting indices, systems, buffers, contacts, lifetimes, and queries.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2Particle.h` and `b2Particle.cpp` — Particle flags, handles, colors, contacts, pairs, triads, and low-level semantics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h` — Complete particle-system public/private contract, buffer model, lifecycle clock, callbacks, queries, and phase order.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp` — Allocation/permutation sites, proxy/contact generation, strict pruning, lifetimes, zombies, forces, queries, and rigid coupling.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp` and `b2WorldCallbacks.h` — Particle-system world-step order, mixed queries, filter/listener interfaces, and destruction callback timing.
- `third_party/liquidfun/liquidfun/Box2D/Unittests/Multi/MultipleParticleSystemsTests.cpp` — Multiple-system behavior and traversal coverage.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- `particle/storage.rs` and its child modules already prove stable IDs over dense permutations, pending-delete snapshots, generation retirement, group-contiguous ranges, fixed owned lanes, optional lanes, and property-tested invariants.
- `World`, its typed arenas, and `AssociationMap` already provide world/system-scoped identities, poisoning, transaction-first destruction snapshots, and typed user associations.
- The Phase 6-8 step pipeline already provides a world lock, one borrowed decision hook, bounded deferred commands, one owned lifecycle report, source-ordered contacts, and panic recovery.
- Existing query/ray directives and the rigid differential protocol provide typed control, bounded semantic IDs, replay, minimization, D0 determinism, and D1/D2 authority tiers.

### Established Patterns

- Validate complete candidates before committing; late failure must not leave partial world, storage, contact, or identity mutation.
- Preserve source-significant order in production and callbacks. Canonicalize only explicitly unordered evidence at collection/comparison boundaries.
- Keep dense indices, raw pointers, allocator details, contact handles, and scratch state private; expose stable semantic IDs and owned/borrow-scoped snapshots.
- Use deep cohesive modules and a closed fail-closed protocol/policy registry rather than parallel harnesses or wildcard tolerances.

### Integration Points

- Replace the object-model placeholder particle records with authoritative per-system particle storage while retaining public handle types and association cleanup behavior.
- Insert the Phase 9 particle slice into `World::step` before rigid island solving at the pinned source point and extend the shared hook/lifecycle types.
- Extend world/system query and ray paths to include particle-system culling and stable particle hits without changing rigid query semantics.
- Extend the existing protocol schema, native Rust adapter, C++ oracle, comparator registry, fixtures, inventory ledger, compatibility report, and CI evidence lanes.

</code-context>

<specifics>
## Specific Ideas

- Treat the existing Phase 3 particle-storage spike as executable architecture evidence to deepen, not a disposable prototype to replace with a second model.
- Preserve upstream external-buffer effects through owned lane transfer and explicit fixed capacity, not through borrowed pointers or `unsafe` aliases.
- Treat callback timing and callback-before-ID-invalidation as compatibility behavior, not merely API presentation.
- Make the Phase 9/10 boundary executable: unsupported solver/group observations must fail closed instead of being silently absent from a passing corpus.

</specifics>

<deferred>
## Deferred Ideas

- Particle-group creation from shapes/strokes/positions, joining, splitting, connectivity, solid depth, rigid group motion, contiguous group maintenance, Voronoi topology, pair generation, and triad generation — Phase 10 (`PART-09` through `PART-11`).
- Baseline and flag-driven particle solver behaviors, solver pass-order completion, and full particle compatibility sign-off — Phase 10 (`PART-12`, `PART-13`, and `PART-18`).
- Generic allocator/backing-store traits, borrowed raw buffers, GPU-mapped memory, or unsafe pointer interoperability — deferred until a measured need and separate safety design justify them.

</deferred>

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Context gathered: 2026-07-14*
