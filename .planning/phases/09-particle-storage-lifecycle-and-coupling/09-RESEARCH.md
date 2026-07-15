---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T03:34:00.000Z
phase: 09-particle-storage-lifecycle-and-coupling
requirements: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
---

# Phase 9: Particle Storage, Lifecycle, and Coupling - Research

<user-constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Deferred Ideas (OUT OF SCOPE)

- Particle-group creation from shapes/strokes/positions, joining, splitting, connectivity, solid depth, rigid group motion, contiguous group maintenance, Voronoi topology, pair generation, and triad generation — Phase 10 (`PART-09` through `PART-11`).
- Baseline and flag-driven particle solver behaviors, solver pass-order completion, and full particle compatibility sign-off — Phase 10 (`PART-12`, `PART-13`, and `PART-18`).
- Generic allocator/backing-store traits, borrowed raw buffers, GPU-mapped memory, or unsafe pointer interoperability — deferred until a measured need and separate safety design justify them.
</user-constraints>

<phase-requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| API-09 | Consumers can inspect particle properties through borrow-scoped bulk views and perform supported mutations without violating aliasing or leaving derived state stale. | One `ParticleSystemView<'_>` over semantic lanes plus typed/closure-scoped editors; structural mutation requires `&mut World`, and position edits rebuild proxies/contacts before return. |
| API-10 | Safe external-particle-buffer equivalents preserve documented ownership, capacity, growth, and teardown behavior without requiring arbitrary lifetime raw pointers. | Owned unified lane bundles, complete adoption validation, explicit fixed/growable mode, declared capacity separate from allocation capacity, and owned teardown. |
| PART-01 | Consumers can create, configure, pause, inspect, and destroy multiple particle systems with upstream-equivalent density, radius, damping, gravity scale, strict-contact, capacity, and iteration controls. | Checked `ParticleSystemDef`, newest-first system order, authoritative per-system storage, destruction transaction, and step integration. |
| PART-02 | Consumers can create and destroy individual particles with positions, velocities, colors, flags, lifetimes, user data, and stable public identities. | Checked `ParticleDef`, stable `ParticleId`, owned association lane/side-table semantics, pending-delete snapshots, and compaction invalidation. |
| PART-03 | Dense particle indices may change during sorting, rotation, and compaction while stable public particle IDs continue to resolve correctly until destruction. | Retain the proved Phase 3 identity table and make every public/evidence boundary translate dense rows to IDs. |
| PART-04 | Every particle storage permutation updates required and optional SoA lanes, ID maps, proxies, contacts, pairs, triads, lifetimes, and group ranges atomically. | Generalize the existing validate-then-commit permutation candidate to body contacts, all production lanes, invalidation state, and rollback/property tests. |
| PART-05 | Consumers can inspect positions, velocities, colors, weights, flags, groups, user data, contacts, body contacts, pairs, triads, and expiration ordering through safe bulk APIs. | Immutable aggregate views expose semantic slices/wrappers while rows, raw capacities, pointers, and reusable internal records remain private. |
| PART-06 | Consumers can supply supported particle buffers with upstream-equivalent capacity constraints and receive explicit failure rather than silent reallocation or aliasing violations. | Fixed bundle adoption and teardown plus growable-owned mode; smallest/declared limit is checked before mutation. |
| PART-07 | Particle proxies, sorting, neighborhood generation, particle contacts, fixture/body contacts, and strict-contact behavior match the selected upstream behavior. | Port tag/proxy enumeration, source contact generation and filtering, body fixture query, effective contact fields, and strict sort/prune order. |
| PART-08 | Finite/infinite lifetimes, quantized expiration order, destroy-by-age, oldest-particle destruction, maximum counts, zombie marking, and deferred compaction match upstream behavior. | Port 32.32 clock, `i32` expiration queue, dirty sort, finite-first eviction, immediate capacity compaction, and ordinary step compaction. |
| PART-14 | Zombie and destruction-listener particle behavior produces upstream-equivalent removal, callback, identity, and compaction outcomes. | Scan old rows ascending, journal listener occurrence before invalidation, preserve snapshot, then commit one survivor permutation. |
| PART-15 | Fixture-contact and particle-contact listener/filter particle flags gate callbacks and collision decisions with upstream-equivalent timing and ordering. | Extend the single borrowed hook with particle-pair and fixture-particle decisions and diff old/new flagged contact sets at source points. |
| PART-16 | Consumers can apply per-particle and range forces/impulses and inspect collision energy, stuck-particle candidates, contact counts, and system statistics. | Validate stable ranges transactionally, preserve force distribution and wall restrictions, and expose semantic owned/borrowed statistics. |
| PART-17 | Consumers can query particles by AABB and ray-cast particle systems with upstream-equivalent clipping, early termination, filtering, and culling. | Add per-system streams and mixed fixture-first world streams using existing typed directives and semantic particle hits. |
</phase-requirements>

## Summary

Phase 9 should be planned as a deepening and convergence phase, not a fresh particle implementation. The repository already proves stable system-scoped particle IDs, pending-delete snapshots, owned fixed lanes, group-contiguous rows, and an atomic permutation over representative state, but that spike is disconnected from `World`'s placeholder particle arenas and uses representative integer lanes rather than the production `f32` contract. The first dependency-critical slice is therefore to move one authoritative production `ParticleStorage` into each live `ParticleSystem`, migrate public identity/destruction behavior to it, and delete the second `World.particles` authority. [VERIFIED: `crates/liquidfun/src/particle/storage.rs:9-145`, `crates/liquidfun/src/world/object.rs:69-88`, `crates/liquidfun/src/world/object.rs:358-390`]

The pinned source has three ordering boundaries that must shape the plans. Particle systems are prepended and traversed newest-first; their `Solve` calls occur after rigid contact-manager collision/update and before rigid island solving; and each active particle sub-iteration performs proxy update/sort, particle-contact diff/filter, body-contact diff/filter/strict prune, then solver/coupling work. Lifetime advancement, zombie compaction, and aggregate flag refresh occur before the paused return, so a paused system skips contact/sub-iteration work but does not categorically freeze lifetime/zombie maintenance. [VERIFIED: `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp:353-373`, `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp:1011-1027`, `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp:2973-3010`]

The phase should close with an extension of the existing bounded rigid-world schema, long-lived process, native adapter, C++ oracle, declaration validator, comparator, replay, D0, sanitizer, and D1 authority gates. Storage/lifecycle/contact/query/coupling observations must be closed under `phase9-v1`; group topology, pair/triad generation, and baseline/flag solver observations must be rejected rather than absent from a passing trace. [VERIFIED: `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-CONTEXT.md:D-27-D-31`, `crates/liquidfun-differential/src/rigid_world/phase8.rs:24-108`, `tools/reference/src/rigid_world.cpp:747-775`]

**Primary recommendation:** establish one production storage authority and its complete permutation/lifecycle invariants first, then layer contacts/callbacks/coupling and queries into the existing source-timed world pipeline, and only afterward extend the fail-closed differential corpus. **Confidence: HIGH.** [VERIFIED: Phase 3 executable storage tests, pinned source, and Phase 9 locked context]

## Project Constraints (from AGENTS.md)

- Production behavior must remain safe native Rust in the single publishable `liquidfun` crate; C++ stays private oracle tooling and ordinary Cargo use must not need it. Public APIs must expose stable semantic IDs, not pointers, dense indices, allocator layout, or foreign runtime state. [VERIFIED: `AGENTS.md` project constraints; `.planning/PROJECT.md:Constraints`]
- Use deep cohesive modules, functional-core/imperative-shell separation, invariant-bearing types, early returns/`let...else`, `maybe_` names for internal `Option` values, `foo.rs` plus `foo/`, documented public APIs, and no `unwrap()` in production. [VERIFIED: `AGENTS.bright-builds.md:Highest-signal rules`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/languages/rust.md`]
- Pure storage, ordering, lifetime, tag, contact, query, and comparator logic requires focused unit tests; unit tests should cover one concern and use clear Arrange/Act/Assert structure. [VERIFIED: `standards/core/testing.md`; `AGENTS.md:Testing`]
- Before any Rust commit, run in order `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`; harness/tooling changes also need the explicit workspace commands and rustdoc. [VERIFIED: `AGENTS.md:Rust Projects`; `TESTING.md:24-45`]
- `.planning/**` is parser-owned GSD content and must not be formatted with mdformat; standalone `---` is reserved for the opening/closing YAML frontmatter delimiters. [VERIFIED: `AGENTS.md:Repo-Local Guidance`; `AGENTS.md:Frontmatter-Parsed Markdown`]
- The relevant Bright Builds pages materially informing this research are `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`; `standards-overrides.md` contains no substantive active override. [VERIFIED: local standards files]

## Standard Stack

### Core

| Component | Version / location | Purpose | Prescription |
| --- | --- | --- | --- |
| Rust | 1.97.0, Edition 2024 | Production particle storage, contacts, queries, lifecycle, and public API | Use the existing pinned toolchain and safe `liquidfun` crate; add no new runtime dependency. [VERIFIED: `rust-toolchain.toml`; local `cargo 1.97.0`] |
| Existing math/collision/world core | `crates/liquidfun/src/{math,collision,world}` | Exact `f32` types, shapes/AABBs/rays, body/fixture authority, step journal, queries | Reuse `Vec2`, `Aabb`, `RayCastInput`, typed directives, body/fixture snapshots and world step; do not create particle-only copies. [VERIFIED: `crates/liquidfun/src/world/query.rs:1-292`, Phase 4-8 contexts] |
| Existing particle storage spike | `crates/liquidfun/src/particle/storage.rs` and children | Stable IDs, lanes, permutation, compaction, capacity/property evidence | Deepen or migrate in place into the production authority; retain its tests and eliminate `dead_code` spike status. [VERIFIED: `crates/liquidfun/src/particle/storage.rs:1-6`, `storage/identity.rs`, `storage/permutation.rs`, `storage/properties.rs`] |
| Existing rigid-world harness | protocol + differential + `tools/reference` | Bounded request, C++ oracle, comparison, replay, evidence lifecycle | Extend this request/process and its closed declarations; do not introduce a second particle executable or protocol. [VERIFIED: `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-CONTEXT.md:D-27`; current rigid modules] |

### Supporting

| Component | Scope | When to use |
| --- | --- | --- |
| `bitflags` 2.13 | Existing production dependency | Represent closed particle flag sets while preserving exact upstream bit values and unknown-bit retention policy. [VERIFIED: workspace dependency research and upstream `b2Particle.h:32-80`] |
| `thiserror` 2.0 | Existing production dependency | Public construction/configuration/handle/storage/query errors; keep hot step internals allocation-free where practical. [VERIFIED: workspace dependency research; `AGENTS.md:Code Quality`] |
| `proptest` 1.11 | Existing dev dependency | Extend the current independent storage model with arbitrary create/edit/mark/compact/permutation/capacity sequences and persisted regressions. [VERIFIED: `crates/liquidfun/src/particle/storage/properties.rs:1-339`] |
| `serde` / `serde_json` | Private tooling only | Extend the bounded semantic request/result, never the published particle state. [VERIFIED: `.planning/research/STACK.md`; existing protocol crates] |
| Pinned C++ LiquidFun | commit `7f20402173fd143a3988c921bc384459c6a858f2` | Behavioral oracle, canonical witnesses, exact source ordering, sanitizer runs | Keep read-only and out of production dependencies. [VERIFIED: `git submodule status`; `UPSTREAM.md:Identity`] |

### Do Not Add

| Candidate | Reason |
| --- | --- |
| `slotmap` / `generational-arena` | The project already owns the checked generation-retirement and system-scoped identity semantics required by Phase 3; replacing it would risk resurrection or ordering differences. [VERIFIED: `crates/liquidfun/src/arena.rs`; Phase 3 D-01-D-04] |
| General math crates | Operation grouping and layout are compatibility policy; reuse project `Vec2` and collision kernels. [VERIFIED: Phase 4 D-01-D-10] |
| Allocator/backing-store traits | Explicitly deferred; Phase 9 has exactly growable-owned and fixed-owned modes. [VERIFIED: Phase 9 D-06-D-08 and Deferred Ideas] |
| FFI/bindings | Runtime delegation and a second in-process particle implementation violate the project boundary. [VERIFIED: `AGENTS.md` project constraints] |

## Current Rust Integration Inventory

### What is reusable

- `ParticleStorage` already owns `dense_to_id`, required representative lanes, optional color/lifetime lanes, proxies, contacts, pairs, triads, expiration order, and group ranges. Its `apply_permutation` validates the old state and complete mapping, builds all candidates, then commits, and its tests prove stable IDs, no-effect invalid permutations, pending-delete snapshots, retirement, fixed declared capacity, owned teardown, and a 128-case independent state machine. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/particle/storage.rs:121-200`, `crates/liquidfun/src/particle/storage.rs:425-574`, all three storage child test files]
- Public `ParticleId` already includes world and owning particle-system scope, while `World` already owns typed associations, poisoning, step locking, authoritative lifecycle reports, and stable body/fixture identities needed by body contacts. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/identity.rs`; `crates/liquidfun/src/world/object.rs`; `crates/liquidfun/src/world/step.rs`]
- The rigid query API already has continue/terminate and ignore/terminate/continue/checked-clip directives with immutable whole-query borrowing and typed invalid geometry/fraction errors. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/world/query.rs:10-182`, `crates/liquidfun/src/world/query.rs:184-292`]

### What must be replaced or extended

- `World` currently stores a `ParticleSystem` containing only diagnostic ID plus `Vec` membership, a separate `ParticleGroup` membership record, and a separate `Arena<Particle, ParticleId>` containing only diagnostic/system/group identity. Creation and destruction update these vectors/arenas immediately. This is the split authority D-01 requires removing. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/world/object.rs:69-88`, `crates/liquidfun/src/world/object.rs:1184-1253`, `crates/liquidfun/src/world/object.rs:1387-1479`, `crates/liquidfun/src/world/object.rs:1646-1724`]
- The storage spike uses `[i32; 2]`, `u16` group tokens, and only two optional lanes. Production needs `Vec2`, exact particle flags/colors, semantic group and user association state, weight/force/stuck/lifetime state, body contacts, and derived invalidation. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/particle/storage.rs:9-17`, Phase 9 D-03-D-08]
- `World::step` currently backs up only rigid arenas/broad phase/contact/continuous/configuration state, has no particle phase, and its hook vocabulary covers fixture pairs and rigid contacts only. Particle state must join transactional limit rollback and panic coherence; particle occurrences must be appended directly to the same lifecycle vector. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/world/step.rs:1081-1133`, `crates/liquidfun/src/world/step.rs:1155-1317`, `crates/liquidfun/src/world/step.rs:609-766`]
- `World::query_aabb` and `World::ray_cast` traverse only the rigid broad phase. Mixed traversal must retain current fixture behavior, then visit newest-first particle systems with per-system culling and a shared current clip/termination state. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun/src/world/query.rs:184-283`; upstream `b2World.cpp:1074-1138`]
- The current rigid protocol has bounded actions/checkpoints, closed witness declarations, phase-specific native execution, and C++ decode/execute helper seams; Phase 9 should add child modules rather than make already-large monoliths substantially deeper. **Confidence: HIGH.** [VERIFIED: `crates/liquidfun-test-protocol/src/schema/rigid_world.rs` (1,027 lines), `crates/liquidfun-differential/src/rigid_world/phase8.rs`, `tools/reference/src/rigid_world_phase8_{decode,execute}.hpp`]

## Pinned Source Behavior Audit

### Allocation and lane inventory

- `ReallocateInternalAllocatedBuffers` clamps requested growth against the user-supplied capacities of flags, positions, velocities, colors, and user data, then reallocates every active required and optional lane together. Production Rust should represent the same effective limit through one validated owned bundle and explicit fixed/growable mode rather than emulate pointer replacement. [VERIFIED: `b2ParticleSystem.cpp:575-631`]
- Creation initializes flags, optional stuck-detection lanes, position, velocity, optional color/user data/handle, group membership, and optional lifetime ordering before increasing the live count. Planning must make candidate validation and identity allocation precede one row commit. [VERIFIED: `b2ParticleSystem.cpp:637-727`]
- The production lane inventory needed by Phase 9 includes stable-ID mapping, flags, positions, velocities, colors, user associations, groups, weights, force accumulation, proxies, particle contacts, body contacts, stuck-detection state, expiration times, expiration order, and every pre-existing Phase 10 pair/triad/group reference that a Phase 9 permutation can invalidate. Phase 10 may populate deferred constraints, but Phase 9 owns their remapping invariant. [VERIFIED: `b2ParticleSystem.h`; `b2ParticleSystem.cpp:575-631,3798-4038,4078-4238`]

### Source-significant lifecycle and step order

- Particle systems are inserted at the world-list head and therefore step newest-first. `b2World::Step` updates rigid contacts, solves active particle systems, then solves rigid islands. The Rust integration point must preserve this cross-domain order. [VERIFIED: `b2World.cpp:353-373,1011-1027`]
- A particle system `Solve` first advances lifetime state when allocated, compacts zombies, refreshes aggregate flags, and only then returns early for pause. Each active sub-iteration increments the timestamp, updates/sorts proxies, updates particle contacts, updates body contacts, computes weights, and then runs solver passes. Phase 9 must preserve the lifecycle/contact prefix while rejecting undeclared Phase 10 solver observations. [VERIFIED: `b2ParticleSystem.cpp:2973-3104`]
- `UpdateContacts` snapshots listener-visible pairs, generates/sorts contacts, filters flagged contacts, then emits the post-contact diff. `UpdateBodyContacts` refreshes stuck counters, queries fixtures, applies fixture filtering, sorts body contacts, optionally prunes spurious contacts under strict mode, then emits listener diffs. [VERIFIED: `b2ParticleSystem.cpp:2192-2271,2290-2339,2608-2739`]
- `SolveZombie` scans old rows in ascending dense order, notifies requested destruction listeners before invalidating handles, copies every survivor lane forward, remaps proxies/contacts/body contacts/pairs/triads, expiration order, and group ranges, then commits the new count. [VERIFIED: `b2ParticleSystem.cpp:3798-4038`]
- `SolveLifetimes` advances the fixed-point clock, stable-sorts dirty expiration indices in reverse lifetime order, and marks expired particles from the oldest end. `DestroyOldestParticle` prefers finite-lifetime particles and marks the selected row zombie; capacity-triggered creation may run zombie compaction immediately. Equal-expiration behavior needs an oracle fixture rather than an inferred Rust tie rule. [VERIFIED: `b2ParticleSystem.cpp:779-805,4040-4076,4243-4308`]
- `RotateBuffer` rotates all required and optional lanes and adjusts handles, proxies, particle/body contacts, pairs, triads, expiration order, and group index ranges. It is a second source permutation path and must route through the same Rust permutation authority as compaction. [VERIFIED: `b2ParticleSystem.cpp:4078-4238`]

### Query, force, and rigid-coupling behavior

- Per-system AABB queries enumerate proxy-tag bounds and report candidate particles; mixed world queries visit the rigid broad phase first and then cull/visit particle systems. Per-system ray casts exclude particles containing the start point, pass a current clipping fraction, and honor callback termination/clipping. [VERIFIED: `b2ParticleSystem.cpp:4507-4598`; `b2World.cpp:1074-1138`]
- Range force and impulse APIs distribute the total vector across the selected particle mass, with wall restrictions for forces; per-particle force accumulates while impulses update velocity immediately. Validate finite inputs, ranges, and derived scale before any mutation. [VERIFIED: `b2ParticleSystem.cpp:4431-4505`]
- Body contacts retain particle index, body/fixture identity, weight, normal, and effective mass data used by later coupling passes. Phase 9 should implement and expose the source-derived contact/coupling foundation while keeping Phase 10 flag-driven particle solver completion explicitly undeclared. [VERIFIED: `b2ParticleSystem.cpp:2340-2739,3310-3392`]

## Recommended Architecture and Implementation Seams

1. Deepen `particle.rs` plus `particle/` into one production deep module: definitions/flags/colors and public views at the module boundary; identity/lane/permutation/lifetime/contact/query kernels in private children. Remove the spike-wide `dead_code` allowance only as each path becomes live.
1. Store authoritative `ParticleSystem` records in the existing system arena, each owning `ParticleStorage`; remove the separate particle arena and derive group/association cleanup from the storage identity table and owned destruction snapshots.
1. Separate pure prepare functions from world effects: definition parsing, capacity decisions, permutation candidates, lifetime sorting, proxy tags, contact generation/pruning, query hit calculation, and comparator policy selection remain data-in/data-out; `World` commits storage, journal, body wake/velocity, and callback effects.
1. Extend `StepConfiguration`, `StepLimits`, `StepHook`, and `LifecycleEvent` additively. Snapshot particle state in the same transaction envelope as rigid state so limit failure and hook panic preserve the existing retry/poison contracts.
1. Reuse collision `Aabb`, shape child queries, broad-phase traversal patterns, `QueryDirective`, `RayCastDirective`, and checked fractions. Do not expose dense indices or add particle-only magic-float callback APIs.
1. Extend rigid protocol schemas through phase-specific child modules, mirroring Phase 8's bounded decode/execute organization in Rust and C++. Require declaration-first families and reject Phase 10 fields in the Phase 9 validator.

## Suggested Plan Decomposition

1. **Source inventory and production contracts:** audit exact flags/defaults/units/permutation lanes; define checked particle/system definitions, colors/flags, errors, and module boundary.
1. **Authoritative storage convergence:** migrate the integer spike to `Vec2`/production lanes, make systems own storage, eliminate split identity authority, and preserve association/destruction behavior.
1. **Bulk views, mutation scopes, and owned buffers:** ship API-09/API-10 with fixed/growable adoption, teardown, capacity matrix, dirty-derived-state repair, and compile-time borrow evidence.
1. **Lifetime and zombie lifecycle:** fixed-point clock, expiration order, destroy-by-age, maximum-count immediate compaction, callback-before-invalidation, rotation/compaction property tests, and equal-tie oracle fixture.
1. **Proxy tags and particle contacts:** source tag math, sort/enumeration, particle-pair contact generation/filtering/listener diff, strict declaration of order and multiplicity.
1. **Fixture/body contacts and rigid coupling:** shape queries, effective fields, stuck tracking, strict pruning, filter/listener timing, panic/rollback, and source-timed `World::step` integration.
1. **Forces, statistics, AABB queries, and ray casts:** checked public APIs, mixed world traversal/culling, early termination/clipping, and focused/property tests.
1. **Protocol/oracle vertical slice:** bounded particle schema, native adapter, C++ decode/execute, stable semantic IDs, selected storage/lifecycle/contact/query witnesses, and comparator policies.
1. **Closed corpus and evidence gate:** declaration coverage for every Phase 9 branch, replay/minimization/D0/sanitizer/debug/release/D1 evidence, compatibility ledger/report updates, and truthfulness review.

Plans may split these slices further to keep each task reviewable. The dependency order is storage authority → lifecycle/permutations → contacts/coupling → public queries/evidence; harness schema work may begin in parallel once public semantic records stabilize.

## Pitfalls and Regression Triggers

- **Split authority:** leaving `World.particles` beside per-system storage produces contradictory live/pending/stale state. Remove or migrate it in one bounded plan with cross-world/system regression tests.
- **Partial permutations:** any direct `Vec::rotate`, `retain`, swap-remove, or lane-specific compaction outside the permutation module is a defect trigger. Add inventory assertions and property operations for every permutation site.
- **Allocator capacity leakage:** `Vec::capacity()` is not the upstream contract. Keep an explicit declared/effective limit and test fixed bundles with unequal underlying capacities.
- **Paused-system misread:** pinned lifetime/zombie maintenance occurs before the paused early return; test pause together with expiration and pending zombies.
- **Callback reconstruction:** begin/end/destruction occurrences must be appended at source points before identity invalidation, not derived from final contact sets.
- **Phase-boundary drift:** pair/triad lanes must remain permutation-safe, but their generation/topology and solver use are Phase 10. Validators should reject those observations in Phase 9.
- **Self-blessed numerics:** exact bits and tolerances must come from the pinned oracle or independently derived source-faithful calculations, never from the Rust output under test.
- **Oversized modules:** `world/object.rs`, `world/step.rs`, protocol schema, and C++ adapter are already large. Add phase-specific child modules rather than expanding monoliths past the repository refactor triggers.

## Validation Architecture

### Fast feedback by task

- Pure storage/identity/permutation/lifetime/contact/query kernels: focused unit tests in the owning module, one behavior per test, explicit Arrange/Act/Assert when non-trivial.
- Public definitions/views/buffers/forces/queries: integration tests through exported APIs plus compile-fail doctests for forbidden aliasing, escaping mutable views, raw construction, and cross-system misuse.
- Permutations/lifecycle: `proptest` state machines comparing an independent semantic model across create, edit, flag, lifetime, mark, rotate, compact, capacity, and teardown sequences; persist minimized failures.
- Step/hooks/coupling: retry-equivalence tests for capacity/limit failure, `catch_unwind` poisoning tests, callback order/multiplicity tests, paused/lifetime tests, and rigid regressions unchanged.
- Protocol/comparator: strict codec/declaration tests, unknown-field/family/policy rejection, replay identity, first-divergence stability, and Phase 10 observation rejection.

### Phase completion gates

1. `cargo fmt --all`
1. `cargo clippy --all-targets --all-features -- -D warnings`
1. `cargo build --all-targets --all-features`
1. `cargo test --all-features`
1. Repository workspace/consumer isolation, rustdoc, inventory/report, replay, D0, C++ debug/release, sanitizer, and canonical-evidence commands named by the implemented plan slices.
1. `just markdown-check` for changed repository-owned non-GSD Markdown; never run mdformat over `.planning/**`.
1. Diff review proving no unmanaged generated file, upstream source, or Phase 10 claim changed accidentally.

### Required differential witnesses

- Multiple systems and newest-first order; paused system with lifetime/zombie maintenance.
- Stable IDs across sort/rotate/compact; every optional lane absent/present; fixed/growable/full buffer and teardown behavior.
- Finite/infinite lifetimes, equal quantized expirations, destroy-by-age, oldest eviction, requested/unrequested destruction callbacks, and immediate capacity compaction.
- Particle-particle and fixture-particle contacts with each filter/listener flag gate, begin/end diff, strict/non-strict pruning, stuck tracking, and body reaction fields.
- Per-particle/range forces and impulses, collision energy/statistics, per-system/mixed AABB queries, ray ignore/clip/terminate/start-inside/culling cases.
- D0 byte identity, debug/release agreement, replay/minimization, sanitizer cleanliness, actual D1 authority, and explicit Phase 10 undeclared-observation rejection.

## Open Research Items for Planning

- Enumerate the exact Phase 9 production lane list and every upstream copy/rotate/remap statement into a checked implementation matrix before the first storage plan edits code.
- Capture canonical equal-expiration ordering, callback timing around immediate maximum-count compaction, and strict-contact tie/pruning witnesses from the pinned oracle before hard-coding expected values.
- Determine the smallest source-faithful Phase 9 coupling slice that exposes required body reaction state without executing or claiming Phase 10 particle solver passes.
- Name the repo-owned full-workspace, oracle, sanitizer, replay, inventory, and D1 commands in each relevant plan rather than deferring them to a final generic verification task.

## RESEARCH COMPLETE

Phase 9 research now covers the locked architecture, exact source-order boundaries, current Rust integration seams, principal permutation/lifecycle risks, staged implementation strategy, and validation architecture needed for executable planning.
