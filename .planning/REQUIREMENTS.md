# Requirements: liquidfun-rs

**Defined:** 2026-07-09
**Core Value:** Deliver an independent, maintainable Rust physics engine whose LiquidFun behavior is demonstrated against a pinned upstream oracle through explicit inventory, tests, differential evidence, and documented tolerances.

## v1 Requirements

Requirements for the first release permitted to claim complete parity with the selected upstream LiquidFun revision. Useful 0.x subsystem releases may precede v1 only when their compatibility gaps are explicit.

### Foundation and Provenance

- [x] **FND-01**: Maintainers can identify the canonical upstream LiquidFun repository, exact immutable revision, release context, and Box2D ancestry from `UPSTREAM.md`.
- [x] **FND-02**: Maintainers can initialize, verify, and intentionally update the read-only upstream submodule through documented commands without silently following a moving branch.
- [x] **FND-03**: Contributors can build the pinned upstream C++ oracle with documented, reproducible CMake/Ninja commands on every supported contributor platform.
- [x] **FND-04**: Maintainers can trace translated code, tests, scenarios, and reference data to upstream source paths and revisions with required license and alteration notices.
- [x] **FND-05**: Ordinary Rust consumers can build, test, package, and use the published crate through Cargo without C++, CMake, Bazel, the upstream submodule, or reference data.
- [x] **FND-06**: Contributors can use a pinned Rust development toolchain and a declared MSRV, with the complete publishable feature surface verified on both.
- [x] **FND-07**: Contributors can discover repository workflows through a root `justfile` whose recipes are thin wrappers around documented Cargo, `xtask`, and oracle commands.
- [x] **FND-08**: CI can reject an upstream submodule revision, generated reference artifact, toolchain pin, or packaged-crate content that does not match its recorded provenance.

### Compatibility Contract

- [x] **COMP-01**: Maintainers can view an exhaustive matrix mapping every relevant upstream subsystem, public API, source area, test, example, and build option to its Rust implementation and evidence status.
- [x] **COMP-02**: Every compatibility row uses distinct states for investigated, planned, implemented, unit tested, differentially validated, platform validated, documented difference, and intentionally unsupported.
- [x] **COMP-03**: Contributors can express a named or seeded simulation as a validated, versioned, engine-neutral scenario with deterministic entity creation and checkpoint requests.
- [x] **COMP-04**: Contributors can run the same validated scenario through the Rust engine and a process-isolated C++ oracle without exposing pointers, raw memory, or implementation layouts in the protocol.
- [x] **COMP-05**: Every engine trace records scenario/schema versions, upstream revision, adapter revision, compiler/build flags, target/platform, seed, and tolerance profile.
- [x] **COMP-06**: The comparator checks IDs, flags, counts, membership, and event kinds exactly while applying reviewed field-specific numeric policies to floating-point observables.
- [x] **COMP-07**: The comparator distinguishes unordered queries/collections from solver-significant and callback/destruction sequences whose order must be preserved.
- [x] **COMP-08**: A differential failure can be reproduced by scenario name or seed, diagnosed at the first divergent checkpoint or phase, and reduced into a focused regression fixture.
- [x] **COMP-09**: Harness crashes, timeouts, sanitizer failures, schema mismatches, and wrong-oracle provenance are reported as harness failures rather than physics mismatches.
- [ ] **COMP-10**: No v1 parity claim can pass the release gate while the compatibility matrix contains an unexplained gap or an unaccounted upstream test/example.

### Safe Rust API

- [x] **API-01**: Rust consumers use distinct world-scoped typed handles for bodies, fixtures, joints, particle systems, particle groups, and stable particles instead of raw pointers.
- [x] **API-02**: Stale, destroyed, wrong-type, and cross-world handles fail explicitly and cannot silently resolve to a reused object slot.
- [x] **API-03**: Object destruction performs documented upstream-equivalent cascades, invalidates affected handles, and returns or emits owned destruction information.
- [x] **API-04**: Contacts are exposed as borrow-scoped views or owned snapshots rather than durable handles to transient solver records.
- [x] **API-05**: Step hooks receive read-only views and narrow supported directives instead of unrestricted mutable world access.
- [x] **API-06**: Application mutations requested during callbacks are represented as commands and applied only at documented unlocked phase boundaries.
- [x] **API-07**: Consumers can obtain owned step events with documented timing, multiplicity, ordering, and lifetime without retaining internal references.
- [x] **API-08**: Consumers can associate user data through a documented safe model that preserves identity and destruction semantics without public raw pointers.
- [x] **API-09**: Consumers can inspect particle properties through borrow-scoped bulk views and perform supported mutations without violating aliasing or leaving derived state stale.
- [x] **API-10**: Safe external-particle-buffer equivalents preserve documented ownership, capacity, growth, and teardown behavior without requiring arbitrary lifetime raw pointers.
- [x] **API-11**: Every public API has succinct rustdoc covering units, invariants, invalidation, callback restrictions, failure behavior, and upstream concept mapping where relevant.
- [x] **API-12**: Every production `unsafe` block is narrowly scoped, justified by measured need, documents its invariant with a `SAFETY:` comment, and has focused verification where practical.

### Math and Collision

- [x] **COLL-01**: Consumers can use upstream-equivalent `f32` vectors, rotations, transforms, sweeps, matrices, constants, and numerical predicates with documented units and conventions.
- [x] **COLL-02**: Consumers can define and query circle, edge, polygon, and chain shapes with upstream-equivalent validation, cloning, mass data, point tests, AABBs, and ray casts.
- [x] **COLL-03**: The engine produces upstream-equivalent overlap, distance, manifold, clipping, and shape-pair collision results for every supported combination.
- [x] **COLL-04**: The dynamic AABB tree supports proxy creation, movement, removal, queries, ray casts, metrics, and deterministic solver-relevant tie behavior.
- [ ] **COLL-05**: Broad-phase pair generation, contact creation, contact persistence, filtering, and refiltering match the selected upstream behavior.
- [x] **COLL-06**: Time-of-impact and continuous-collision kernels handle supported shape sweeps and edge cases within documented numerical policies.
- [x] **COLL-07**: Math and collision behavior has focused unit/property tests and pure differential probes before world-level solvers depend on it.
- [x] **COLL-08**: Numerical policy explicitly defines compiler/feature assumptions, NaN and signed-zero treatment, determinism tiers, per-observable tolerances, and divergence horizons.

### Rigid-Body Dynamics

- [x] **RIGD-01**: Consumers can create, mutate, inspect, activate, deactivate, and destroy static, kinematic, and dynamic bodies with upstream-equivalent transforms and identity behavior.
- [x] **RIGD-02**: Consumers can attach fixtures and sensors with upstream-equivalent density, mass/inertia, friction, restitution, collision filtering, and destruction behavior.
- [x] **RIGD-03**: Consumers can apply forces, torques, linear/angular impulses, damping, gravity scale, fixed rotation, bullet mode, and velocity changes with upstream-equivalent effects.
- [x] **RIGD-04**: The engine creates, updates, filters, solves, and destroys rigid contacts with upstream-equivalent manifolds, material mixing, warm starting, and sensor behavior.
- [x] **RIGD-05**: Island construction and velocity/position constraint solving reproduce upstream-equivalent world stepping within the approved order and numeric policies.
- [x] **RIGD-06**: Sleeping and waking behavior, thresholds, allowed-sleep controls, and activation transitions match the selected upstream behavior.
- [x] **RIGD-07**: Bullet handling, sub-stepping, continuous collision, and TOI solving prevent tunneling and reproduce supported upstream outcomes.
- [x] **RIGD-08**: Consumers can configure gravity, warm starting, continuous physics, sub-stepping, automatic force clearing, timestep iterations, and world origin shifting.
- [x] **RIGD-09**: Consumers can query fixtures by AABB and ray-cast through the world with upstream-equivalent clipping, termination, filtering, and explicitly unspecified callback order.
- [ ] **RIGD-10**: Consumers can inspect body/contact/joint/proxy counts, tree metrics, timing profiles, and renderer-independent debug-draw primitives without accessing internal storage.
- [x] **RIGD-11**: Rigid-body scenarios covering non-colliding, colliding, stacked, sleeping, fast-moving, filtered, queried, and destroyed worlds pass semantic differential validation.

### Joints, Rope, and Callbacks

- [x] **JOIN-01**: Consumers can create, configure, inspect, simulate, and destroy revolute, prismatic, distance, pulley, mouse, gear, wheel, weld, friction, rope, and motor joints.
- [x] **JOIN-02**: Joint limits, motors, anchors, reaction forces/torques, body dependencies, collision settings, and destruction cascades match the selected upstream behavior.
- [x] **JOIN-03**: Consumers can use the standalone upstream-equivalent rope model independently of the rope joint.
- [x] **JOIN-04**: Contact filters, contact listeners, destruction listeners, and supported pre-solve controls reproduce upstream timing and behavior through the safe hook/event API.
- [x] **JOIN-05**: Every joint type, standalone rope behavior, filter path, listener path, and diagnostic-dump representation has focused and differential coverage.

### Particle Systems

- [x] **PART-01**: Consumers can create, configure, pause, inspect, and destroy multiple particle systems with upstream-equivalent density, radius, damping, gravity scale, strict-contact, capacity, and iteration controls.
- [x] **PART-02**: Consumers can create and destroy individual particles with positions, velocities, colors, flags, lifetimes, user data, and stable public identities.
- [x] **PART-03**: Dense particle indices may change during sorting, rotation, and compaction while stable public particle IDs continue to resolve correctly until destruction.
- [x] **PART-04**: Every particle storage permutation updates required and optional SoA lanes, ID maps, proxies, contacts, pairs, triads, lifetimes, and group ranges atomically.
- [x] **PART-05**: Consumers can inspect positions, velocities, colors, weights, flags, groups, user data, contacts, body contacts, pairs, triads, and expiration ordering through safe bulk APIs.
- [x] **PART-06**: Consumers can supply supported particle buffers with upstream-equivalent capacity constraints and receive explicit failure rather than silent reallocation or aliasing violations.
- [x] **PART-07**: Particle proxies, sorting, neighborhood generation, particle contacts, fixture/body contacts, and strict-contact behavior match the selected upstream behavior.
- [x] **PART-08**: Finite/infinite lifetimes, quantized expiration order, destroy-by-age, oldest-particle destruction, maximum counts, zombie marking, and deferred compaction match upstream behavior.
- [x] **PART-09**: Consumers can create particle groups from shapes, strokes, explicit positions, or existing groups and can inspect their ranges, flags, transforms, velocities, mass, and inertia.
- [x] **PART-10**: Group creation, destruction, joining, splitting, connectivity, can-be-empty behavior, solid depth updates, rigid motion, and contiguous membership preserve upstream semantics.
- [x] **PART-11**: Voronoi-based topology, pair generation, triad generation, and reactive regeneration produce upstream-equivalent constraints and membership.
- [x] **PART-12**: Baseline particle passes for collision, gravity, pressure, damping, rigid damping, extra damping, force application, velocity limiting, and lifetime solving run in the pinned upstream order.
- [x] **PART-13**: Water, wall, spring, elastic, viscous, powder, tensile, barrier, static-pressure, reactive, repulsive, and color-mixing particle behaviors match the selected upstream behavior.
- [x] **PART-14**: Zombie and destruction-listener particle behavior produces upstream-equivalent removal, callback, identity, and compaction outcomes.
- [x] **PART-15**: Fixture-contact and particle-contact listener/filter particle flags gate callbacks and collision decisions with upstream-equivalent timing and ordering.
- [x] **PART-16**: Consumers can apply per-particle and range forces/impulses and inspect collision energy, stuck-particle candidates, contact counts, and system statistics.
- [x] **PART-17**: Consumers can query particles by AABB and ray-cast particle systems with upstream-equivalent clipping, early termination, filtering, and culling.
- [x] **PART-18**: Each particle flag, unflagged solver pass, group behavior, lifecycle path, buffer path, contact path, query, and callback is individually represented in the compatibility matrix and differentially signed off.

### Verification and Regression Protection

- [x] **TEST-01**: Pure math, geometry, ordering, identity, and solver kernels have focused unit tests with one primary concern and clear Arrange/Act/Assert structure.
- [x] **TEST-02**: Public world, rigid-body, joint, particle, callback, query, and destruction workflows have integration tests through supported APIs.
- [ ] **TEST-03**: Every applicable upstream test is ported, replaced by equivalent evidence, or documented as irrelevant with a reviewed rationale.
- [x] **TEST-04**: Property tests cover geometry invariants, broad-phase behavior, handle validity, particle permutation/group invariants, query correctness, and reproducible world operation sequences.
- [x] **TEST-05**: Fuzz targets cover shape/collision inputs, scenario protocol parsing, world mutation sequences, particle operations, and every unsafe boundary appropriate for fuzzing.
- [x] **TEST-06**: Miri and Rust sanitizers exercise useful supported subsets, while C++ oracle builds run appropriate sanitizers without crossing failures into the Rust process.
- [x] **TEST-07**: Every corrected differential mismatch becomes a minimized named regression that fails before the fix and records its oracle/tolerance provenance.
- [x] **TEST-08**: CI reports Rust coverage and keeps C++ coverage separate unless compatible LLVM tooling is explicitly proven; coverage gaps are visible by subsystem.
- [x] **TEST-09**: Verification entrypoints can run fast affected checks locally and reserve expensive randomized, differential, sanitizer, coverage, and benchmark suites for appropriate scheduled/manual lanes.

### Examples and Testbed

- [ ] **EXMP-01**: Every upstream example and testbed scenario is ported, replaced, or listed with an explicit reason and compatibility impact.
- [ ] **EXMP-02**: Contributors can run renderer-neutral scenarios headlessly by name or seed, pause/restart them, single-step them, and capture deterministic semantic checkpoints.
- [ ] **EXMP-03**: The same scenario definitions drive Rust execution, C++ oracle execution, regression fixtures, benchmarks, and the optional visual testbed.
- [ ] **EXMP-04**: An optional interactive testbed can select scenarios, pause, step, restart, alter timestep settings, and display contacts, particle contacts, broad-phase data, and performance statistics.
- [x] **EXMP-05**: The visual testbed can capture comparison state and display Rust/oracle differences without owning simulation logic or accessing private engine storage.
- [ ] **EXMP-06**: Core and published physics crates build and run in headless environments without renderer, windowing, or game-engine dependencies.

### Performance

- [x] **PERF-01**: Benchmarks cover world stepping, broad phase, narrow phase, contact solving, CCD, joints, particle lifecycle, contact generation, sorting, pressure, large particle systems, mixed worlds, queries, and ray casts.
- [x] **PERF-02**: Rust and C++ comparisons use equivalent scenarios, optimization modes, compiler/toolchain records, hardware, warm-up, and measurement methodology.
- [x] **PERF-03**: Consumers and benchmark tooling can inspect phase-level profiles without exposing or coupling to private storage.
- [x] **PERF-04**: Structural performance changes are justified by profiles and retain or improve differential, safety, determinism, and API evidence.
- [x] **PERF-05**: A scalar deterministic implementation remains the compatibility baseline even if later opt-in SIMD or parallel experiments exist.
- [ ] **PERF-06**: Public performance claims link to reproducible methodology and identify workloads, versions, flags, hardware, results, and compatibility status.

### Platform Support

- [x] **PLAT-01**: The complete supported v1 surface builds and passes required verification on Linux x86_64.
- [x] **PLAT-02**: The complete supported v1 surface builds and passes required verification on Linux ARM64.
- [x] **PLAT-03**: The complete supported v1 surface builds and passes required verification on macOS ARM64.
- [x] **PLAT-04**: The complete supported v1 surface builds and passes required verification on macOS x86_64 where sustainable CI capacity exists, with any limitation explicit.
- [x] **PLAT-05**: The complete supported v1 surface builds and passes required verification on Windows x86_64.
- [x] **PLAT-06**: Platform/compiler differences are classified through documented determinism tiers and reviewed tolerances rather than silently changing reference data.

### Documentation and Release

- [x] **DOCS-01**: `README.md` accurately states current maturity, implemented and missing capabilities, build/test/example commands, submodule needs, toolchain needs, contribution path, and license status.
- [x] **DOCS-02**: `ARCHITECTURE.md` explains crate/module boundaries, dependency direction, handles, callbacks, particle storage, step order, oracle isolation, and renderer independence.
- [x] **DOCS-03**: `UPSTREAM.md` records canonical source, exact revision, ancestry, patches, build process, licenses/notices, and intentional update procedure.
- [ ] **DOCS-04**: `COMPATIBILITY.md` exposes the complete traceability matrix, evidence states, known differences, tolerance scope, and no-gap parity status.
- [x] **DOCS-05**: `TESTING.md` documents test layers, scenario protocol, differential diagnosis, reference-data review, fuzz/Miri/sanitizer/coverage workflows, and CI tiers.
- [ ] **DOCS-06**: `BENCHMARKING.md` documents comparable workloads, environment capture, profiling workflow, result interpretation, and rules for performance claims.
- [x] **DOCS-07**: `SAFETY.md` documents the public safety model, identity/invalidation, callback mutation, buffer ownership, user data, and every remaining unsafe invariant.
- [x] **DOCS-08**: `CONTRIBUTING.md` and release documentation explain bootstrap, quality gates, compatibility sign-off, provenance rules, generated artifacts, SemVer, MSRV, and publication checks.
- [x] **DOCS-09**: The final v1 release audit verifies complete docs, required notices, packaged crate contents, upstream test/example accounting, supported platforms, benchmarks, safety evidence, and zero unexplained compatibility gaps.

## v2 Requirements

Deferred capabilities are valuable but are not prerequisites for upstream parity.

### Post-Parity Ergonomics

- **SNAP-01**: Consumers can save, load, migrate, and replay a versioned semantic scene format that includes particles and is explicitly distinct from upstream diagnostic dump output.
- **ERGO-01**: Consumers can use ergonomic builders, iterators, optional serde adapters, and carefully scoped compatibility aliases after the foundational API is stable.
- **ADPT-01**: Game-engine and ecosystem adapters integrate through the public renderer-neutral core without becoming production dependencies of `liquidfun`.

### Additional Platforms and Modes

- **PORT-01**: The engine has sustained WASM validation with target-appropriate differential and performance evidence.
- **PORT-02**: The engine has sustained iOS and Android validation with target-appropriate CI and documentation.
- **NSTD-01**: A coherent math or collision subset supports `no_std` after allocation, API, and independent-crate-boundary audits.
- **OPTM-01**: Consumers may opt into SIMD or parallel stepping modes after scalar parity, determinism contracts, and measured performance benefits are established.
- **PREC-01**: Alternative precision modes are available only as documented extensions with independent compatibility expectations.

## Out of Scope

Explicit exclusions prevent shortcuts from being mistaken for project progress.

| Feature | Reason |
| --- | --- |
| Runtime C++ delegation or thin bindings | Violates the native, independent Rust deliverable and Cargo-only consumer contract |
| Modern Box2D or Rapier behavior substituted without proof | Similar APIs do not demonstrate compatibility with the selected historical LiquidFun behavior |
| Rigid-body-only v1 | Particles are LiquidFun's defining feature; partial work may ship only as clearly labeled 0.x previews |
| Curated subset of particle flags | Every pinned upstream behavior is required for the v1 parity claim |
| Pointer-shaped or reentrantly mutable public API | Recreates C++ invalidation and callback hazards instead of using Rust to prevent them |
| Automatic widening of numeric tolerances | Hides correctness, ordering, compiler, or platform defects |
| General save/load marketed as upstream `Dump` | Upstream dump is diagnostic reconstruction output, not a complete persistence format |
| Bazel in the foundation | Adds a second build graph before Cargo/CMake limitations demonstrate a need |
| Renderer or game-engine coupled core | Prevents headless, server, testing, and framework-independent use |
| Default parallel or nondeterministic stepping | Can change solver order and compatibility; any future mode must be opt-in |
| Complete-engine `no_std` promise | Feasibility is unproven; only coherent audited subsets may be pursued later |
| Unverified parity or production claims | Demos and partial tests cannot replace the matrix, platform, benchmark, and audit evidence |

## Traceability

Roadmap creation maps every v1 requirement to exactly one phase.

| Requirement | Phase | Status |
| --- | --- | --- |
| FND-01 | Phase 1 | Complete |
| FND-02 | Phase 1 | Complete |
| FND-03 | Phase 1 | Complete |
| FND-04 | Phase 1 | Complete |
| FND-05 | Phase 1 | Complete |
| FND-06 | Phase 12 | Complete |
| FND-07 | Phase 1 | Complete |
| FND-08 | Phase 1 | Complete |
| COMP-01 | Phase 1 | Complete |
| COMP-02 | Phase 1 | Complete |
| COMP-03 | Phase 2 | Complete |
| COMP-04 | Phase 2 | Complete |
| COMP-05 | Phase 2 | Complete |
| COMP-06 | Phase 2 | Complete |
| COMP-07 | Phase 2 | Complete |
| COMP-08 | Phase 2 | Complete |
| COMP-09 | Phase 2 | Complete |
| COMP-10 | Phase 12 | Pending |
| API-01 | Phase 3 | Complete |
| API-02 | Phase 3 | Complete |
| API-03 | Phase 3 | Complete |
| API-04 | Phase 3 | Complete |
| API-05 | Phase 3 | Complete |
| API-06 | Phase 3 | Complete |
| API-07 | Phase 3 | Complete |
| API-08 | Phase 3 | Complete |
| API-09 | Phase 9 | Complete |
| API-10 | Phase 9 | Complete |
| API-11 | Phase 12 | Complete |
| API-12 | Phase 12 | Complete |
| COLL-01 | Phase 4 | Complete |
| COLL-02 | Phase 5 | Complete |
| COLL-03 | Phase 5 | Complete |
| COLL-04 | Phase 5 | Complete |
| COLL-05 | Phase 5 → Phase 6 | Pending |
| COLL-06 | Phase 5 | Complete |
| COLL-07 | Phase 5 | Complete |
| COLL-08 | Phase 4 | Complete |
| RIGD-01 | Phase 6 | Complete |
| RIGD-02 | Phase 6 | Complete |
| RIGD-03 | Phase 7 | Complete |
| RIGD-04 | Phase 6 | Complete |
| RIGD-05 | Phase 7 | Complete |
| RIGD-06 | Phase 7 | Complete |
| RIGD-07 | Phase 7 | Complete |
| RIGD-08 | Phase 7 | Complete |
| RIGD-09 | Phase 7 | Complete |
| RIGD-10 | Phase 11 | Pending |
| RIGD-11 | Phase 8 | Complete |
| JOIN-01 | Phase 8 | Complete |
| JOIN-02 | Phase 8 | Complete |
| JOIN-03 | Phase 8 | Complete |
| JOIN-04 | Phase 8 | Complete |
| JOIN-05 | Phase 8 | Complete |
| PART-01 | Phase 9 | Complete |
| PART-02 | Phase 9 | Complete |
| PART-03 | Phase 9 | Complete |
| PART-04 | Phase 9 | Complete |
| PART-05 | Phase 9 | Complete |
| PART-06 | Phase 9 | Complete |
| PART-07 | Phase 9 | Complete |
| PART-08 | Phase 9 | Complete |
| PART-09 | Phase 10 | Complete |
| PART-10 | Phase 10 | Complete |
| PART-11 | Phase 10 | Complete |
| PART-12 | Phase 10 | Complete |
| PART-13 | Phase 10 | Complete |
| PART-14 | Phase 9 | Complete |
| PART-15 | Phase 9 | Complete |
| PART-16 | Phase 9 | Complete |
| PART-17 | Phase 9 | Complete |
| PART-18 | Phase 10 | Complete |
| TEST-01 | Phase 10 | Complete |
| TEST-02 | Phase 10 | Complete |
| TEST-03 | Phase 11 | Pending |
| TEST-04 | Phase 10 | Complete |
| TEST-05 | Phase 12 | Complete |
| TEST-06 | Phase 12 | Complete |
| TEST-07 | Phase 12 | Complete |
| TEST-08 | Phase 12 | Complete |
| TEST-09 | Phase 1 | Complete |
| EXMP-01 | Phase 11 | Pending |
| EXMP-02 | Phase 11 | Pending |
| EXMP-03 | Phase 11 | Pending |
| EXMP-04 | Phase 11 | Pending |
| EXMP-05 | Phase 11 | Complete |
| EXMP-06 | Phase 11 | Pending |
| PERF-01 | Phase 12 | Complete |
| PERF-02 | Phase 12 | Complete |
| PERF-03 | Phase 12 | Complete |
| PERF-04 | Phase 12 | Complete |
| PERF-05 | Phase 12 | Complete |
| PERF-06 | Phase 12 | Pending |
| PLAT-01 | Phase 12 | Complete |
| PLAT-02 | Phase 12 | Complete |
| PLAT-03 | Phase 12 | Complete |
| PLAT-04 | Phase 12 | Complete |
| PLAT-05 | Phase 12 | Complete |
| PLAT-06 | Phase 12 | Complete |
| DOCS-01 | Phase 12 | Complete |
| DOCS-02 | Phase 3 | Complete |
| DOCS-03 | Phase 1 | Complete |
| DOCS-04 | Phase 12 | Pending |
| DOCS-05 | Phase 2 | Complete |
| DOCS-06 | Phase 12 | Pending |
| DOCS-07 | Phase 12 | Complete |
| DOCS-08 | Phase 12 | Complete |
| DOCS-09 | Phase 12 | Complete |

**Coverage:**

- v1 requirements: 108
- Mapped to phases: 108
- Unmapped: 0 ✓

*Requirements defined: 2026-07-09*
*Last updated: 2026-07-12 after Phase 6 completion*
