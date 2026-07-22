# Roadmap: liquidfun-rs

## Overview

The v1 roadmap moves from an immutable, licensed source of truth to a semantic comparison contract, then retires the public-model and numerical risks before implementing collision, rigid dynamics, joints, particles, and renderer-neutral examples in upstream dependency order. Compatibility evidence is added with each subsystem, while performance optimization, broad platform proof, and any full-parity claim wait until the complete scalar implementation can pass a final no-gap audit.

## Phases

**Phase numbering:** Integer phases are planned milestone work. Decimal phases are reserved for urgent insertions and execute in numeric order between their surrounding integers.

- [x] **Phase 1: Oracle, Provenance, and Repository Foundation** - Freeze the source of truth and establish the licensed, Cargo-first development foundation without broad physics implementation. (completed 2026-07-10)
- [x] **Phase 2: Semantic Protocol and Oracle Round Trip** - Prove a versioned, process-isolated Rust/C++ comparison path with trustworthy provenance and failure classification. (completed 2026-07-10)
- [x] **Phase 3: Rust Object Model and Storage Architecture** - Prove safe identity, invalidation, callback, mutation, and storage semantics before they harden into the engine API.
- [x] **Phase 4: Math, Settings, and Numerical Policy** - Establish the `f32` mathematical vocabulary and explicit determinism/tolerance contract. (completed 2026-07-11)
- [x] **Phase 5: Shapes and Collision Foundation** - Implement and differentially verify shapes, narrow phase, broad phase, and TOI kernels. (completed 2026-07-11)
- [x] **Phase 6: Minimal Rigid World Vertical Slice** - Run bodies, fixtures, and contacts through the complete native Rust and oracle comparison pipeline. (completed 2026-07-12)
- [x] **Phase 7: Rigid Solver, World Operations, and CCD** - Complete rigid stepping, sleeping, continuous collision, configuration, queries, and ray casts. (completed 2026-07-13)
- [x] **Phase 8: Joints, Rope, Callbacks, and Rigid Sign-Off** - Complete joints, standalone rope, hook timing, diagnostic dump, and the broad rigid-body compatibility gate. (completed 2026-07-15)
- [x] **Phase 9: Particle Storage, Lifecycle, and Coupling** - Implement safe particle systems, storage, lifecycle, contacts, buffers, queries, callbacks, and rigid coupling. (completed 2026-07-18)
- [x] **Phase 10: Particle Groups, Solvers, and Compatibility Sign-Off** - Complete group topology and every particle behavior in pinned upstream pass order. (completed 2026-07-21)
- [ ] **Phase 11: Examples, Headless Tooling, and Testbed** - Account for every upstream test/example and expose shared headless and optional visual scenarios.
- [ ] **Phase 12: Performance, Portability, and Release Hardening** - Prove performance, safety, platform, documentation, packaging, and zero-gap v1 readiness.

## Phase Details

### Phase 1: Oracle, Provenance, and Repository Foundation

**Goal**: Freeze the final upstream oracle and establish the licensed, reproducible, Cargo-first repository foundation, architecture evidence, and compatibility inventory before broad physics implementation.
**Depends on**: Nothing (first phase)
**Requirements**: FND-01, FND-02, FND-03, FND-04, FND-05, FND-07, FND-08, COMP-01, COMP-02, TEST-09, DOCS-03
**Success Criteria** (what must be TRUE):

1. Maintainers can identify the canonical repository, immutable oracle revision, release context, Box2D ancestry, license obligations, alteration notices, and provenance rules from checked-in records.
1. Contributors can initialize, verify, intentionally update, and reproducibly build the read-only oracle on supported contributor platforms through documented commands.
1. Ordinary consumers can build, test, and package the Cargo workspace without C++, CMake, Bazel, the upstream submodule, or reference data, while contributors can discover all workflows through thin `just`/`xtask` entrypoints.
1. CI rejects mismatched pins, provenance, generated reference artifacts, or package contents, and contributors can inspect an exhaustive compatibility inventory with distinct evidence states.
1. Architecture, build-orchestration, licensing, risk, and milestone decisions are recorded well enough to begin focused subsystem planning without implementing broad physics behavior.

**Research / ADR flags**: Final oracle commit versus tag; exact Box2D ancestry and LiquidFun deltas; required build patches; license, notice, and alteration obligations; canonical C++ compiler/platform; Cargo/CMake orchestration and explicit Bazel deferral; cohesive crate/module boundaries.
**Plans**: 5/5 plans complete

### Phase 2: Semantic Protocol and Oracle Round Trip

**Goal**: Establish the semantic scenario/trace contract and prove an isolated Rust-to-C++ oracle round trip before subsystem comparisons accumulate.
**Depends on**: Phase 1
**Requirements**: COMP-03, COMP-04, COMP-05, COMP-06, COMP-07, COMP-08, COMP-09, DOCS-05
**Success Criteria** (what must be TRUE):

1. Contributors can express and validate a bounded, versioned named or seeded scenario with deterministic semantic entity IDs and checkpoint requests.
1. The same empty-world scenario runs through Rust and a process-isolated C++ oracle and produces schema- and provenance-checked semantic traces without pointers, raw memory, or private layout.
1. The comparator checks exact observables exactly, applies reviewed field-specific numeric policies, canonicalizes only explicitly unordered results, and preserves solver/callback/destruction order.
1. A mismatch can be reproduced and localized to its first divergent checkpoint, while crashes, timeouts, sanitizer failures, schema errors, and wrong-oracle traces are classified as harness failures.
1. `TESTING.md` documents protocol versions, diagnosis, reference-data review, regression/minimization workflow, and the local versus scheduled verification tiers.

**Research / ADR flags**: Schema evolution; float-bit encoding; input/output bounds; streaming and process reuse; timeout/crash semantics; canonical forms; reference-data review; first-divergence probes; minimization strategy.
**Plans**: TBD during phase planning

### Phase 3: Rust Object Model and Storage Architecture

**Goal**: Prove an idiomatic safe Rust model for identity, invalidation, destruction, callbacks, user data, and mutable storage before solver implementation depends on it.
**Depends on**: Phase 2
**Requirements**: API-01, API-02, API-03, API-04, API-05, API-06, API-07, API-08, DOCS-02
**Success Criteria** (what must be TRUE):

1. Distinct world-scoped typed handles reject stale, destroyed, wrong-type, and cross-world access without silently resolving reused slots.
1. Destruction exercises documented cascades and owned destruction information, while transient contacts cannot escape as durable internal references.
1. Step hooks receive read-only views and narrow directives; application mutation is deferred to documented unlocked boundaries; owned events have explicit timing, multiplicity, order, and lifetime.
1. Safe user data and a storage spike demonstrate stable public particle identity across dense permutations without exposing raw pointers or dense indices.
1. `ARCHITECTURE.md` records dependency direction, handles, callbacks, storage, step order, oracle isolation, and renderer independence as enforceable boundaries.

**Research / ADR flags**: Handle bit layout and generation-wrap policy; world identity; `Send`/`Sync`; user-data ownership; callback panic policy; destruction timing; stable particle-ID cost; authoritative lane permutation; safe external-buffer semantics.
**Plans**: TBD during phase planning

### Phase 4: Math, Settings, and Numerical Policy

**Goal**: Implement the purpose-built `f32` math/settings layer and turn floating-point, ordering, and platform expectations into an explicit compatibility policy.
**Depends on**: Phase 3
**Requirements**: COLL-01, COLL-08
**Success Criteria** (what must be TRUE):

1. Consumers can use documented upstream-equivalent vectors, rotations, transforms, sweeps, matrices, constants, and numerical predicates with the correct units and conventions.
1. Pure oracle probes demonstrate the selected math behavior under pinned compiler and feature assumptions before collision or world solvers depend on it.
1. The numerical policy classifies NaN and signed zero, deterministic versus tolerant observables, absolute/relative/ULP policies, divergence horizons, and platform tiers.

**Research / ADR flags**: Oracle variability; fused operations and contraction; signed zero and NaN; compiler flags; repeatability horizons; exact versus tolerant fields; cross-platform tolerance tiers.
**Plans**: TBD during phase planning

### Phase 5: Shapes and Collision Foundation

**Goal**: Implement and verify the complete shape and collision substrate required by rigid worlds and particle-body coupling.
**Depends on**: Phase 4
**Requirements**: COLL-02, COLL-03, COLL-04, COLL-05, COLL-06, COLL-07
**Success Criteria** (what must be TRUE):

1. Consumers can define, validate, clone, measure, point-test, bound, and ray-cast circle, edge, polygon, and chain shapes with upstream-equivalent results.
1. Overlap, distance, clipping, manifolds, and every supported shape-pair collision produce upstream-equivalent semantic observables.
1. The dynamic AABB tree and broad phase support proxy lifecycle, movement, queries, ray casts, metrics, pair generation, filtering/refiltering, and deterministic solver-relevant ties.
1. Time-of-impact kernels handle supported sweeps and edge cases within the approved numerical policy.
1. Focused unit/property tests and pure differential probes protect all collision foundations before world-level solvers consume them.

**Research / ADR flags**: Audit exact pinned-source branch and tie ordering, cache/simplex observables, manifold identity, broad-phase pair order, and TOI termination before compatibility sign-off.
**Plans**: TBD during phase planning

### Phase 6: Minimal Rigid World Vertical Slice

**Goal**: Deliver the smallest complete native Rust rigid world that proves object creation, destruction, contact lifecycle, and semantic differential execution end to end.
**Depends on**: Phase 5
**Requirements**: RIGD-01, RIGD-02, RIGD-04
**Success Criteria** (what must be TRUE):

1. Consumers can create, mutate, inspect, activate, deactivate, and destroy static, kinematic, and dynamic bodies with stable typed identity.
1. Fixtures and sensors expose upstream-equivalent density, mass/inertia, friction, restitution, filtering, and destruction behavior.
1. Contacts are created, persisted, filtered, updated, and destroyed with correct manifolds, material mixing, warm-start state, and sensor semantics.
1. One non-colliding and one colliding world step pass through the complete scenario, Rust adapter, C++ oracle, comparator, and regression-fixture path.

**Research / ADR flags**: Audit pinned body/fixture/contact creation order, intrusive-list replacement, mass reset rules, material mixing, sensor timing, and destruction cascades before the slice is signed off.
**Plans**: 22/22 plans complete

### Phase 7: Rigid Solver, World Operations, and CCD

**Goal**: Complete scalar rigid-body stepping and the world operations needed for broad rigid compatibility.
**Depends on**: Phase 6
**Requirements**: RIGD-03, RIGD-05, RIGD-06, RIGD-07, RIGD-08, RIGD-09
**Success Criteria** (what must be TRUE):

1. Forces, torques, impulses, damping, gravity scale, fixed rotation, bullet mode, and velocity changes produce upstream-equivalent state transitions.
1. Island construction plus velocity and position constraints reproduce the pinned phase order, warm starting, and scalar numerical behavior.
1. Sleeping/waking, activation, continuous collision, bullet handling, sub-stepping, and TOI prevent tunneling and match supported upstream outcomes.
1. Consumers can configure world stepping and force clearing, shift the origin, query fixtures by AABB, and ray-cast with documented clipping, termination, filtering, and unspecified callback order.
1. Non-colliding, stacked, sleeping, fast-moving, filtered, and queried scenario families accumulate first-divergence differential evidence for the later rigid sign-off gate.

**Research / ADR flags**: Audit island traversal, constraint ordering, warm-start caches, sleep thresholds, TOI queue/order, sub-step state, query canonicalization, and origin-shift observables against the final oracle.
**Plans**: TBD during phase planning

### Phase 8: Joints, Rope, Callbacks, and Rigid Sign-Off

**Goal**: Finish the rigid-body surface and pass a broad semantic compatibility gate before particle implementation expands the world step.
**Depends on**: Phase 7
**Requirements**: RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05
**Success Criteria** (what must be TRUE):

1. Consumers can create, configure, inspect, simulate, and destroy all eleven upstream joint types with correct limits, motors, anchors, reactions, collision settings, dependencies, and cascades.
1. The standalone rope model runs independently of the rope joint and matches focused oracle scenarios.
1. Contact filters/listeners, destruction listeners, and supported pre-solve controls operate through the safe hook/event model with upstream-equivalent timing and order.
1. Every joint, rope, callback/filter path, and diagnostic-dump representation has focused and differential coverage.
1. The complete rigid scenario suite, including destruction and previously accumulated world cases, passes the reviewed semantic differential gate.

**Research / ADR flags**: Audit each joint's pinned solver order and state, gear dependencies, listener multiplicity, pre-solve directive limits, destruction sequence, and diagnostic-dump fidelity.
**Plans**: TBD during phase planning

### Phase 9: Particle Storage, Lifecycle, and Coupling

**Goal**: Implement safe, identity-preserving particle systems and their lifecycle, contact, buffer, query, callback, and rigid-coupling foundations.
**Depends on**: Phase 8
**Requirements**: API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17
**Success Criteria** (what must be TRUE):

1. Consumers can create, configure, pause, inspect, and destroy multiple particle systems and particles with stable identities, flags, colors, lifetimes, and safe user data.
1. Sorting, rotation, and compaction atomically update every required/optional SoA lane, ID map, proxy, contact, constraint, lifetime record, and group range while borrow-scoped bulk views remain valid by construction.
1. Supported external-buffer equivalents enforce ownership, capacity, growth, and teardown contracts with explicit failure rather than reallocation or aliasing violations.
1. Proxies, neighborhoods, particle contacts, fixture/body contacts, strict-contact behavior, lifetimes, zombies, destruction callbacks, and deferred compaction match the pinned oracle.
1. Forces/impulses, collision energy, stuck candidates, statistics, AABB queries, ray casts, and particle contact listener/filter flags are exposed and differentially verified through safe APIs.

**Research / ADR flags**: Audit every permutation site and optional lane; capacity/full-buffer behavior; lifetime quantization; zombie callback order; strict-contact pruning; particle/body contact order; query culling and canonicalization.
**Plans**: 31/31 plans complete

### Phase 10: Particle Groups, Solvers, and Compatibility Sign-Off

**Goal**: Complete particle group topology and every baseline and flag-driven solver behavior in the final oracle's exact pass order.
**Depends on**: Phase 9
**Requirements**: PART-09, PART-10, PART-11, PART-12, PART-13, PART-18, TEST-01, TEST-02, TEST-04
**Success Criteria** (what must be TRUE):

1. Consumers can create and inspect groups from shapes, strokes, positions, and existing groups, then join, split, destroy, or retain empty groups with correct contiguity, connectivity, transforms, mass, inertia, depth, and rigid motion.
1. Voronoi topology plus pair, triad, and reactive regeneration preserve upstream-equivalent membership and constraints across group mutations.
1. Baseline passes and every upstream particle flag behavior run in named, pinned order and pass flag-by-flag semantic differential scenarios without a global tolerance escape hatch.
1. Focused unit, integration, and property tests cover math/order/identity/solver kernels, public rigid and particle workflows, geometry, handles, permutations, group invariants, queries, and reproducible world operations.
1. The compatibility matrix individually signs off every particle flag, unflagged pass, group behavior, lifecycle, buffer, contact, query, and callback path.

**Research / ADR flags**: Audit the final oracle's complete particle pass graph, flag/group-flag interactions, pair/triad ordering, Voronoi edge cases, solid depth, rigid groups, and join/split rotation semantics before each cluster is signed off.
**Plans**: 32 plans

Plans:

- [x] 10-01-PLAN.md — Define the cohesive public particle-group contract.
- [x] 10-02-PLAN.md — Add exact checked particle-solver coefficients.
- [x] 10-03-PLAN.md — Complete pair/triad storage and semantic views.
- [x] 10-04-PLAN.md — Define the exact closed O01-O05/S01-S26 pass manifest.
- [x] 10-05-PLAN.md — Establish storage-owned group authority and invariants.
- [x] 10-06-PLAN.md — Add solver scratch lanes and executable lane inventory.
- [x] 10-07-PLAN.md — Build operation-specific transactional mutation candidates.
- [x] 10-08-PLAN.md — Implement exact fill, stroke, and explicit-position sampling.
- [x] 10-09-PLAN.md — Wire public group creation, append, and inspection.
- [x] 10-10-PLAN.md — Implement deferred group destruction and retained-empty lifecycle.
- [x] 10-11-PLAN.md — Implement source-ordered Voronoi topology generation.
- [x] 10-12-PLAN.md — Probe pinned split/degenerate outcomes, then generate exact pair and full-triad constraints.
- [x] 10-13-PLAN.md — Implement exact identity-preserving group join.
- [x] 10-14-PLAN.md — Implement exact connectivity-based group split.
- [x] 10-15-PLAN.md — Implement reactive topology, solid depth, and rigid cache timing.
- [x] 10-16-PLAN.md — Expose complete safe public group mutation workflows.
- [x] 10-17-PLAN.md — Implement contact, weight, topology, force, and gravity preparation passes.
- [x] 10-18-PLAN.md — Implement viscous, repulsive, powder, tensile, solid, and color passes.
- [x] 10-19-PLAN.md — Implement static pressure, pressure, and damping passes.
- [x] 10-20-PLAN.md — Implement elastic, spring, and velocity-limit constraints.
- [x] 10-21-PLAN.md — Implement rigid, barrier, collision, wall, and integration passes.
- [x] 10-22-PLAN.md — Replace the Phase 9 prefix with the full transactional solver.
- [x] 10-23-PLAN.md — Close native pass, flag, property, and inherited baseline coverage.
- [x] 10-24-PLAN.md — Extend the strict rigid-world protocol for Phase 10.
- [x] 10-25-PLAN.md — Execute Phase 10 scenarios through the native public API.
- [x] 10-26-PLAN.md — Extend the pinned C++ oracle for Phase 10.
- [x] 10-27-PLAN.md — Define exhaustive exact and numeric comparison policy.
- [x] 10-28-PLAN.md — Seal the closed five-family semantic differential corpus.
- [x] 10-29-PLAN.md — Build the shared local and exact-reference evidence validator.
- [x] 10-30-PLAN.md — Wire local D2 and same-run canonical D1 evidence production.
- [x] 10-31-PLAN.md — Acquire and independently validate one fresh D1 authority set.
- [x] 10-32-PLAN.md — Promote proven outcomes and complete the phase audit.

### Phase 11: Examples, Headless Tooling, and Testbed

**Goal**: Account for the upstream behavioral corpus and expose one renderer-neutral scenario catalog across headless execution, oracle comparison, regression, benchmarks, and optional visualization.
**Depends on**: Phase 10
**Requirements**: RIGD-10, TEST-03, EXMP-01, EXMP-02, EXMP-03, EXMP-04, EXMP-05, EXMP-06
**Success Criteria** (what must be TRUE):

1. Every applicable upstream test, example, and testbed scenario is ported, replaced by equivalent evidence, or recorded with a reviewed irrelevance/compatibility rationale.
1. Contributors can run renderer-neutral scenarios headlessly by name or seed, pause/restart, single-step, and capture deterministic semantic checkpoints.
1. The same scenario definitions drive Rust, C++ oracle, regressions, benchmarks, and the optional visual testbed without duplicating simulation logic.
1. Consumers can inspect counts, tree metrics, phase profiles, contacts, particle contacts, broad-phase data, and debug-draw primitives without private-storage access; the testbed can display Rust/oracle differences.
1. Core and published physics crates continue to build and run headlessly with no renderer, windowing, or game-engine dependency.

**Research / ADR flags**: Select a renderer only after a headless capability spike; treat Macroquad as a candidate, preserve public debug-view boundaries, and define deterministic capture/comparison behavior before interactive polish.
**Plans**: TBD during phase planning

### Phase 12: Performance, Portability, and Release Hardening

**Goal**: Turn the complete scalar engine into an auditable v1 release candidate with reproducible performance, supported-platform evidence, hardened safety/testing, complete documentation, and zero unexplained compatibility gaps.
**Depends on**: Phase 11
**Requirements**: FND-06, COMP-10, API-11, API-12, TEST-05, TEST-06, TEST-07, TEST-08, PERF-01, PERF-02, PERF-03, PERF-04, PERF-05, PERF-06, PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05, PLAT-06, DOCS-01, DOCS-04, DOCS-06, DOCS-07, DOCS-08, DOCS-09
**Success Criteria** (what must be TRUE):

1. The complete publishable surface builds and passes required verification on the pinned development toolchain, declared MSRV, Linux x86_64/ARM64, macOS ARM64/x86_64 where sustainable, and Windows x86_64, with compiler/platform variation classified by documented tiers.
1. Comparable Rust/C++ benchmarks and phase profiles cover all required workloads; every structural optimization is profile-justified and preserves the scalar deterministic compatibility baseline, safety, API, and differential evidence.
1. Fuzz targets, Miri, Rust/C++ sanitizers, coverage reporting, and minimized provenance-bearing regressions exercise the supported surfaces and isolate harness failures from physics mismatches.
1. Public rustdoc and the README, compatibility, benchmarking, safety, contribution, and release documents accurately describe units, invariants, unsafe boundaries, maturity, commands, evidence, licenses, packaging, MSRV, and performance claims.
1. The final release audit finds no unexplained compatibility gap or unaccounted upstream test/example and verifies notices, crate contents, supported platforms, benchmarks, safety evidence, and publication checks.

**Research / ADR flags**: Set performance budgets and evidence thresholds; confirm the release/MSRV policy and sustainable CI matrix; require an ADR for any unsafe, SIMD, parallel, or structural optimization; review license and package contents immediately before release.
**Plans**: TBD during phase planning

## Progress

**Execution order:** Phases execute in numeric order from 1 through 12. Decimal insertions, if any, execute between their surrounding integers.

| Phase | Plans Complete | Status | Completed |
| --- | --- | --- | --- |
| 1. Oracle, Provenance, and Repository Foundation | 5/5 | Complete    | 2026-07-10 |
| 2. Semantic Protocol and Oracle Round Trip | 14/14 | Complete    | 2026-07-10 |
| 3. Rust Object Model and Storage Architecture | 5/5 | Complete    | 2026-07-11 |
| 4. Math, Settings, and Numerical Policy | 7/7 | Complete    | 2026-07-11 |
| 5. Shapes and Collision Foundation | 8/8 | Complete    | 2026-07-11 |
| 6. Minimal Rigid World Vertical Slice | 22/22 | Complete    | 2026-07-12 |
| 7. Rigid Solver, World Operations, and CCD | 13/13 | Complete    | 2026-07-13 |
| 8. Joints, Rope, Callbacks, and Rigid Sign-Off | 24/24 | Complete    | 2026-07-15 |
| 9. Particle Storage, Lifecycle, and Coupling | 31/31 | Complete | 2026-07-18 |
| 10. Particle Groups, Solvers, and Compatibility Sign-Off | 32/32 | Complete   | 2026-07-21 |
| 11. Examples, Headless Tooling, and Testbed | 11/29 | In Progress|  |
| 12. Performance, Portability, and Release Hardening | 0/TBD | Not started | - |

*Roadmap created: 2026-07-09*
