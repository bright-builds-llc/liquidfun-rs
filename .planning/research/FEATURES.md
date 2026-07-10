# Feature Research

**Domain:** Native Rust port of Google LiquidFun 1.1.0-era 2D rigid-body and particle physics
**Researched:** 2026-07-09
**Confidence:** HIGH for the upstream API inventory; MEDIUM for sequencing and market differentiation until the exact oracle revision and acceptance budgets are approved

## Scope Interpretation

For this project, “v1” means the first release allowed to claim complete parity with the selected upstream revision. It is not a reduced rigid-body-only MVP. Partial subsystem releases may be useful during 0.x development, but they must identify their gaps precisely and must not be marketed as LiquidFun parity.

The inventory below was verified against Google LiquidFun commit [7f20402173fd143a3988c921bc384459c6a858f2](https://github.com/google/liquidfun/tree/7f20402173fd143a3988c921bc384459c6a858f2), which identifies itself as LiquidFun 1.1.0-era source. The upstream release notes state that 1.1.0 is based on Box2D revision 280 / Box2D 2.3.0. The project must still make and record a separate decision about whether this post-tag master commit or the v1.1.0 tag is the final oracle.

Complexity is whole-capability complexity, including behavioral validation:

- **LOW:** bounded surface with little algorithmic or ownership risk
- **MEDIUM:** several interactions or portability concerns, but established algorithms
- **HIGH:** solver, ownership, ordering, callback, or cross-platform behavior with substantial differential evidence required
- **VERY HIGH:** cross-cutting work that can invalidate architecture or parity claims

“Library capability” means a feature an ordinary Rust consumer can use. “Internal enabler” means development and release evidence required to credibly provide those capabilities, but not part of the normal runtime API.

## Verified Upstream Baseline

### Rigid-Body Inventory

The selected oracle must preserve the historical LiquidFun/Box2D behavior, not merely expose similarly named modern physics features.

| Capability | Verified upstream inventory | Complexity | Depends on |
| --- | --- | --- | --- |
| Geometry and collision | Circle, edge, polygon, and chain shapes; point tests, distance, AABB, ray cast, mass data, manifolds, overlap/distance, time of impact | HIGH | Math primitives, transforms, allocators |
| Broad and narrow phase | Dynamic AABB tree, broad-phase proxy management, shape-pair contact generation, contact persistence and filtering | HIGH | Geometry and collision |
| Rigid bodies and fixtures | Static, kinematic, and dynamic bodies; fixtures, sensors, density/mass/inertia, friction, restitution, filters, damping, gravity scale, forces/impulses, sleep, active state, fixed rotation, bullet/CCD behavior | VERY HIGH | Collision pipeline, stable identity |
| Constraint solving | Islands, warm starting, velocity and position constraints, sleeping, continuous collision detection, sub-stepping, force clearing | VERY HIGH | Contacts, bodies, deterministic ordering |
| Joints | Revolute, prismatic, distance, pulley, mouse, gear, wheel, weld, friction, rope, and motor joints | VERY HIGH | Bodies, solver, stable identity |
| Standalone rope | The public `b2Rope` model in addition to the rope joint | MEDIUM | Math and constraint foundations |
| World operations | Stepping with independent particle iterations, recommended particle-iteration calculation, object creation/destruction, origin shift, gravity, profiles, counts/tree metrics, debug draw, queries, ray casts, diagnostic dump | HIGH | All rigid and particle foundations |

Primary evidence: [shape type enum](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Collision/Shapes/b2Shape.h), [joint type enum](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/Joints/b2Joint.h), [world API](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2World.h), and [LiquidFun release notes](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/ReleaseNotes.md).

### Particle Flag and Behavior Inventory

All 18 upstream particle flags are v1 parity scope. Some are solver behaviors; some gate lifecycle or callbacks. The zero-valued water flag still implies the baseline particle pipeline.

| Upstream flag | Required observable behavior | Complexity | Depends on |
| --- | --- | --- | --- |
| `b2_waterParticle` | Baseline fluid behavior with gravity, collision, pressure, and damping | VERY HIGH | Particle contacts, body contacts, baseline solver |
| `b2_zombieParticle` | Deferred removal and buffer/index compaction after stepping | HIGH | Lifecycle, handles, destruction ordering |
| `b2_wallParticle` | Immobile/zero-velocity particles that still participate in constraints | HIGH | Contact and pair generation |
| `b2_springParticle` | Pair creation and restoration toward initial pair distance | HIGH | Sorted pairs, groups |
| `b2_elasticParticle` | Triad creation and deformation-restoring response | VERY HIGH | Voronoi/triads, groups |
| `b2_viscousParticle` | Relative-velocity damping against particles and bodies | HIGH | Particle and body contacts |
| `b2_powderParticle` | Suppressed isotropic pressure plus powder repulsion behavior | HIGH | Weight/pressure/contact solver |
| `b2_tensileParticle` | Surface-tension pressure and normal forces | VERY HIGH | Neighborhood weights/normals |
| `b2_colorMixingParticle` | Contact-based color mixing at configured strength | MEDIUM | Color buffer, contacts |
| `b2_destructionListenerParticle` | Per-particle destruction callback before removal | HIGH | Lifecycle, callback safety |
| `b2_barrierParticle` | Pair-based barrier constraints that prevent particle leakage/tunneling | VERY HIGH | Pairs, collision solver, ordering |
| `b2_staticPressureParticle` | Iterative static-pressure solve and relaxation | VERY HIGH | Pressure buffers, stable iterations |
| `b2_reactiveParticle` | Regeneration of pairs/triads after flag or topology changes | HIGH | Pair/triad topology |
| `b2_repulsiveParticle` | Strong configurable particle repulsion | HIGH | Contacts and pressure |
| `b2_fixtureContactListenerParticle` | Begin/end fixture-particle contact notifications | HIGH | Body contacts, listener staging |
| `b2_particleContactListenerParticle` | Begin/end particle-particle contact notifications | HIGH | Contact identity and ordering |
| `b2_fixtureContactFilterParticle` | Per fixture-particle collision filtering | HIGH | Contact filter API |
| `b2_particleContactFilterParticle` | Per particle-particle collision filtering | HIGH | Contact filter API |

Primary evidence: the upstream [particle flag enum](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2Particle.h) and [particle solver implementation](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp).

These unflagged or group-driven behaviors are also v1 scope:

| Capability | Required observable behavior | Complexity | Depends on |
| --- | --- | --- | --- |
| Baseline solver passes | Particle collision, gravity, pressure, damping, rigid damping, extra damping, force application, velocity limiting, and lifetime solving | VERY HIGH | Particle storage, contacts, world step |
| Solid particle groups | Prevent overlap/leaking, maintain depth data, and apply solid-group ejection behavior | VERY HIGH | Group topology and depth computation |
| Rigid particle groups | Preserve group shape and expose mass, inertia, center, transform, linear/angular velocity | VERY HIGH | Pairs/triads, rigid solver |
| Group lifecycle flags | Can-be-empty behavior plus internal will-be-destroyed and needs-depth-update transitions | HIGH | Deferred lifecycle, compaction |
| Particle creation modes | Individual creation; group fill/stroke from one or more shapes; explicit positions; creation into an existing group | HIGH | Shapes, storage, group lifecycle |
| Group topology | Create, destroy, join, and split disconnected groups with correct group membership and callbacks | VERY HIGH | Contacts, connectivity, stable identity |
| Lifetimes and capacity | Finite/infinite lifetimes, expiration ordering, destroy-by-age, oldest-particle destruction, maximum counts | HIGH | Quantized time, compaction, handles |
| Buffers and handles | Positions, velocities, colors, weights, flags, group membership, user data, contacts, body contacts, pairs, triads, expiration order, and stable particle handles | VERY HIGH | Rust aliasing model, storage layout |
| External buffers | User-supplied flags, position, velocity, color, and user-data storage with upstream capacity semantics | VERY HIGH | Ownership/lifetime design, safe buffer API |
| Forces and diagnostics | Per-particle and range force/impulse, collision energy, strict contact checking, stuck-particle candidates, pause, density/gravity/damping/radius/static-pressure controls | HIGH | Solver and public API |

Primary evidence: [particle-system public API and definition](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h) and [particle-group flags/API](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleGroup.h).

## Feature Landscape

### Table Stakes: Library Capabilities

Missing any P1 row prevents a credible v1 parity claim.

| ID | Feature | Why expected | Complexity | Depends on |
| --- | --- | --- | --- | --- |
| TS-01 | Native Cargo-usable engine | Users asked for an independent Rust implementation, not runtime FFI or a C++ toolchain | VERY HIGH | Entire port; EN-01, EN-11 |
| TS-02 | Historical rigid-body parity | LiquidFun is a Box2D-compatible rigid-body engine as well as a particle engine | VERY HIGH | Verified rigid inventory; EN-03 |
| TS-03 | Every shape, contact path, joint, and rope API | Existing LiquidFun scenes must not lose constraints or collision cases | VERY HIGH | TS-02, stable handles |
| TS-04 | Every particle flag and solver behavior | Particle simulation is LiquidFun’s defining capability | VERY HIGH | TS-02, particle storage and contacts |
| TS-05 | Complete particle lifecycle and groups | Real scenes depend on creation/destruction, lifetimes, multiple systems, joining/splitting, solid/rigid groups, and stable handles | VERY HIGH | TS-04, callback model |
| TS-06 | Particle inspection and external-buffer equivalents | Renderers and applications depend on direct bulk access, user data, contacts, pairs, and triads | VERY HIGH | TS-05, safe ownership model |
| TS-07 | Fixture, particle, and mixed contacts | Filtering, listener events, destruction events, sensors, strict checks, and particle/body response must all compose | VERY HIGH | TS-02, TS-04, TS-09 |
| TS-08 | AABB/shape queries and ray casts | Game logic needs fixture and particle results, early termination/clipping, and whole-particle-system culling | HIGH | Broad phase, particle proxies, callbacks |
| TS-09 | Idiomatic safe object and callback API | Rust users need explicit invalidation, no raw public pointers, and clear mutation/reentrancy rules | VERY HIGH | Early architecture decision, EN-08 |
| TS-10 | World controls and observability | Fixed stepping, particle sub-iterations, sleeping, warm starting, CCD, sub-stepping, force clearing, origin shift, counts, tree quality, and profiles are upstream-visible | HIGH | Rigid and particle world integration |
| TS-11 | Debug-draw abstraction | Upstream exposes renderer-independent shapes, joints, particles, AABBs, transforms, and centers for inspection | MEDIUM | Stable read-only world view |
| TS-12 | Upstream-equivalent diagnostic dump | `World::Dump` and supported body/fixture/joint dumps are public troubleshooting surfaces | HIGH | Identity/order mapping, all supported dump types |
| TS-13 | Upstream examples and testbed scenarios | Examples are executable behavioral documentation and expose integration gaps tests miss | HIGH | Most engine capabilities, EN-05 |
| TS-14 | Headless operation | Physics must work in servers, tests, CI, and custom renderers without a windowing dependency | MEDIUM | Renderer boundary |
| TS-15 | Mainstream desktop/server platforms | Linux x86_64 and ARM64, macOS ARM64 and practical x86_64 coverage, and Windows x86_64 are the stated initial portability floor | VERY HIGH | EN-06, EN-09 |
| TS-16 | Public API and migration documentation | Users need rustdoc, C++-to-Rust concept mapping, callback/invalidation rules, examples, and known differences | HIGH | Stable API, EN-02 |
| TS-17 | Measured performance suitability | A production physics engine needs published representative budgets and no unexplained catastrophic regression versus optimized upstream | VERY HIGH | Correctness first, EN-07 |
| TS-18 | Truthful compatibility and maturity reporting | Consumers must be able to tell exactly what is implemented, validated, platform-tested, and known to differ | MEDIUM | EN-02, EN-10 |

### Internal Enablers Required for Credibility

These are not ordinary runtime features. They are release prerequisites because parity cannot be established by code inspection alone.

| ID | Enabler | Why required | Complexity | Depends on |
| --- | --- | --- | --- | --- |
| EN-01 | Pinned oracle, ancestry, provenance, and license record | Defines what “compatible” means and prevents moving-target or attribution errors | HIGH | Upstream/release research |
| EN-02 | Exhaustive compatibility matrix | Maps every subsystem, public API, test, example, and compile-time option to implementation and evidence status | HIGH | EN-01, inventory automation |
| EN-03 | Semantic C++/Rust differential harness | Compares bodies, contacts, joints, particles, groups, callbacks, queries, ray casts, and destruction events using named seeded scenarios | VERY HIGH | EN-01, stable observation schema |
| EN-04 | Tolerance and ordering policy | Separates bugs from floating-point, iteration-order, compiler, and platform differences | VERY HIGH | EN-03, cross-platform samples |
| EN-05 | Upstream test/example accounting | Every upstream test and scenario is ported, replaced, or explicitly justified; the candidate source currently contains a broad rigid/particle testbed and dedicated particle unit suites | HIGH | EN-01, licensing review |
| EN-06 | Layered verification | Unit, integration, upstream-compatibility, differential, property, fuzz, regression, Miri, and sanitizer coverage according to subsystem risk | VERY HIGH | Testable module boundaries |
| EN-07 | Comparable benchmark harness | Equivalent inputs, compiler modes, hardware, warm-up, and measurements for rigid, particle, mixed, query, and lifecycle workloads | HIGH | Correct implementation, pinned C++ build |
| EN-08 | Safety model and audit | Documents handle generations, invalidation, callback staging, external buffers, user data, and every narrow `unsafe` invariant | VERY HIGH | TS-09, TS-06 |
| EN-09 | Platform CI matrix | Proves supported targets rather than inferring portability from Rust compilation | HIGH | Reproducible toolchains and tests |
| EN-10 | Release evidence gate | Blocks “full parity” or “production-ready” wording until the matrix has no unexplained gaps and required suites pass | MEDIUM | EN-02 through EN-09 |
| EN-11 | Published-crate isolation check | Proves normal Cargo consumers do not fetch/build/link C++, Bazel, upstream source, or reference data | MEDIUM | Packaging and CI |
| EN-12 | Reproducible failure/minimization tooling | Turns a differential seed into a small regression fixture with machine-readable and human-readable diagnostics | VERY HIGH | EN-03, scenario model |

### Differentiators: Production-Quality Rust Experience

Parity earns credibility; these features make the Rust port preferable to using the archived C++ source or a thin binding.

| ID | Feature | Value proposition | Scope | Complexity | Depends on |
| --- | --- | --- | --- | --- | --- |
| DF-01 | Generational, typed handles and explicit invalidation | Prevents stale-pointer use while retaining recognizable body/fixture/joint/group identity | v1 | VERY HIGH | TS-09, EN-08 |
| DF-02 | Borrow-safe bulk particle views | Enables efficient rendering/data processing without exposing aliased raw buffers | v1 | VERY HIGH | TS-06, storage architecture |
| DF-03 | Deferred mutation/event command model | Makes callback restrictions explicit and safe instead of relying on “do not mutate during step” pointer discipline | v1 | HIGH | TS-07, TS-09 |
| DF-04 | Evidence-linked compatibility dashboard | Lets users trace each claim to tests, differential cases, platforms, benchmarks, docs, and known deviations | v1 | HIGH | EN-02, CI outputs |
| DF-05 | Reproducible headless scenario CLI | Runs a scenario by name/seed, captures semantic state, and compares Rust with C++ without starting a renderer | v1 | HIGH | EN-03, TS-13 |
| DF-06 | Renderer-independent interactive comparison testbed | Supports pause, single step, reset, settings, overlays, deterministic capture, and side-by-side/diff inspection | v1 | HIGH | DF-05, TS-11 |
| DF-07 | Fine-grained profiling counters | Makes broad phase, contact, solver, and particle-pass costs visible without private-layout access | v1 | MEDIUM | TS-10, EN-07 |
| DF-08 | Versioned semantic scene snapshots | Provides a documented Rust persistence/replay format including particles; intentionally distinct from upstream diagnostic dump | v1.x | VERY HIGH | Stable public model, migration policy |
| DF-09 | Ergonomic builders, iterators, and optional serde adapters | Reduces boilerplate while retaining a recognizable low-level compatibility layer | v1.x | MEDIUM | Stable v1 API |
| DF-10 | Native WASM and mobile validation | Expands deployment without importing the historical C++ build stack | v1.x | VERY HIGH | v1 parity, platform-specific CI |
| DF-11 | Optional engine/ecosystem adapters | Bevy and other integrations improve adoption while the core remains framework-neutral | v1.x | HIGH | Stable headless core |
| DF-12 | Realistic `no_std` math/collision subset | Supports constrained environments without promising an implausible complete-engine port | v2+ | HIGH | Clear crate boundaries, allocation audit |
| DF-13 | Opt-in SIMD or parallel modes | Can exceed scalar performance where profiling proves value | v2+ | VERY HIGH | EN-04, EN-07, scalar parity baseline |

## Anti-Features

| Anti-feature | Why requested | Why problematic | Deliberate alternative |
| --- | --- | --- | --- |
| Runtime C++ delegation or thin bindings | Fastest apparent route to a Rust API | Violates native-port value, safety, portability, and Cargo independence | Keep FFI development-only for reference and benchmarks |
| Modern Box2D/Rapier behavior substituted without validation | Reuse reduces implementation work | Similar concepts do not imply LiquidFun 2.3.0-era solver/contact/order parity | Reuse only after license, ancestry, and differential review |
| Rigid-body-only “LiquidFun v1” | Delivers visible progress sooner | Omits the defining particle system and weakens the explicit parity target | Ship clearly labeled 0.x subsystem previews; reserve v1 for full scope |
| A curated subset of particle flags | Common fluid effects cover many demos | Flags interact; missing barrier, reactive, filtering, lifecycle, or group behaviors breaks real scenes | Inventory and validate every upstream behavior |
| C++ pointer-shaped public API | Makes source translation mechanical | Exposes invalidation, aliasing, and callback hazards Rust should prevent | Typed handles, borrowed views, explicit step/mutation phases |
| Arbitrary reentrant mutation during callbacks | Seems ergonomic | Upstream forbids entity creation/destruction while locked; mutation can invalidate solver state | Read-only event context plus deferred commands |
| Exact bitwise parity claim on every platform | Sounds stronger than tolerance-based parity | Floating point, compiler, SIMD, and ordering differences make it misleading and brittle | Per-observable exact/order/tolerance policies with evidence |
| Default parallel or SIMD stepping | Promises headline speed | Can change ordering/determinism and obscure correctness defects | Scalar deterministic baseline; later explicit opt-in modes |
| Raw-memory comparison as compatibility oracle | Easy to snapshot | C++/Rust layouts, pointers, padding, allocator state, and harmless ordering differ | Compare documented semantic state |
| Blanket serialization of internal structs | Derive macros make it easy | Freezes private layout, hidden solver caches, handles, and unstable invariants | Versioned semantic snapshot DTO after v1 |
| Calling upstream Dump “save/load” | It looks like scene serialization | Upstream Dump logs rigid-body reconstruction code, omits particle state, and some joint dumps are unsupported | Match diagnostic dump in v1; add truthful snapshot format later |
| Unsafe zero-copy external buffers by default | Mirrors C++ and may benchmark well | Alias/lifetime violations can become memory unsafety | Safe owned/borrowed buffer contracts; isolate any unsafe adapter |
| Core dependency on a renderer or game engine | Produces attractive demos quickly | Breaks headless use and forces downstream architecture | Renderer-independent debug primitives and optional testbed/adapters |
| Complete `no_std`, embedded, WASM, iOS, and Android promise in v1 | Maximizes addressable platforms | Multiplies risk before core parity and may distort APIs | Desktop/server v1; research-backed target additions afterward |
| C++/Bazel requirement for ordinary users | Simplifies repository orchestration | Negates conventional Rust consumption | Cargo-only published crates; reference tooling stays contributor-only |
| Stable internal storage order/layout as public contract | Helps direct indexing and serialization | Prevents optimization and makes compaction changes breaking | Document semantic ordering only where upstream behavior requires it |
| Performance claims without comparable workloads | Marketing is easy | Compiler flags, timestep, particle count, hardware, and observables can invalidate comparisons | Publish reproducible methodology and raw results |
| “Production-ready” before evidence completion | Encourages adoption | Transfers unknown compatibility and safety risk to users | Automated release gate and conspicuous known-gap reporting |

## Feature Dependencies

```text
[Pinned oracle and inventory]
    └──requires──> [Compatibility matrix]
                         ├──drives──> [Rust API/identity model]
                         └──drives──> [Differential observation schema]

[Math + shapes + collision]
    └──requires──> [Broad phase + contacts]
                         └──requires──> [Bodies + rigid solver + CCD]
                                              ├──requires──> [Joints]
                                              └──requires──> [Particle/body contacts]

[Particle storage + proxies]
    └──requires──> [Particle contacts + baseline solver]
                         ├──requires──> [Pairs/triads + groups]
                         ├──requires──> [Every flagged solver]
                         └──requires──> [Lifecycle + callbacks + queries]

[Semantic C++/Rust harness]
    └──requires──> [Pinned oracle + observation schema + tolerance policy]
                         └──enables──> [Parity sign-off + truthful v1]

[Renderer-independent scenarios]
    ├──enables──> [Headless regression runner]
    └──enables──> [Optional interactive testbed]

[Default parallel stepping] ──conflicts──> [Upstream ordering and deterministic baseline]
```

### Dependency Notes

- **Ownership design precedes broad implementation:** body, fixture, joint, particle, and group identity affects every callback, query, destruction path, buffer, and test observable.
- **Rigid-body collision precedes full particles:** particles reuse world shapes, fixture contacts, broad-phase concepts, callbacks, and stepping.
- **Particle storage precedes behavior flags:** flags are not isolated effects; most share contact, weight, pressure, pair, triad, force, and compaction buffers.
- **Differential schema should arrive early:** adding observability after implementation risks hiding ordering and state differences or designing an API that cannot be compared semantically.
- **Examples depend on core but feed verification:** scenario simulation should be separated from rendering so the same setup runs interactively, headlessly, and against the oracle.
- **Optimization follows scalar parity:** layout, SIMD, and parallel work require a stable reference, tolerance policy, profiles, and regression suite.

## V1 Definition

### Launch With: v1 Parity Release

- [ ] Native Cargo-only runtime and complete historical rigid-body behavior, all four shapes, all 11 joints, standalone rope, world operations, CCD, contacts, queries, and ray casts.
- [ ] Every particle flag, baseline solver pass, system control, buffer, lifecycle rule, solid/rigid group behavior, pair/triad path, contact path, force, query, and callback.
- [ ] Safe typed handles, documented invalidation, explicit callback/mutation rules, and safe equivalents for upstream bulk/external buffers.
- [ ] Upstream-equivalent diagnostic dumping, with documentation that it is not general serialization.
- [ ] Every upstream public API, compile-time option, test, and example accounted for in a compatibility matrix; implementation-specific optimizations such as 16-bit indices/NEON may be classified as non-semantic only with evidence.
- [ ] Seeded differential validation with semantic state, minimized regressions, documented tolerances/order rules, and machine-readable results.
- [ ] Renderer-independent scenario layer, headless runner, and optional interactive testbed covering upstream scenarios and controls.
- [ ] Published comparable benchmark results, safety audit, desktop/server platform matrix, complete user/developer docs, license/provenance records, and truthful release gate.

### Add After v1: v1.x Extensions

- [ ] Versioned semantic scene snapshot/replay format, including particle state, once the public model and migration policy are stable.
- [ ] Ergonomic builders, iterators, optional serde adapters, and compatibility aliases that do not obscure the core model.
- [ ] Native WASM, iOS, and Android validation when target-specific CI and differential evidence are sustainable.
- [ ] Optional game-engine integrations that depend on, but do not enter, the renderer-neutral core.

### Future Consideration: v2+

- [ ] `no_std` math/collision subsets after allocation and platform audits demonstrate a coherent boundary.
- [ ] Opt-in SIMD and parallel stepping after scalar parity, reproducibility, and performance thresholds are established.
- [ ] Alternative precision modes only after defining whether they are extensions rather than LiquidFun parity surfaces.

## Feature Prioritization Matrix

| Capability group | User value | Implementation cost | Priority |
| --- | --- | --- | --- |
| Oracle pin, inventory, compatibility contract | HIGH | HIGH | P1 |
| Rust identity/callback/buffer model | HIGH | VERY HIGH | P1 |
| Rigid-body shapes, collision, bodies, solver, CCD | HIGH | VERY HIGH | P1 |
| All joints and standalone rope | HIGH | VERY HIGH | P1 |
| Particle storage, lifecycle, contacts, groups | HIGH | VERY HIGH | P1 |
| Every particle behavior and solver pass | HIGH | VERY HIGH | P1 |
| Queries, ray casts, callbacks, filters, destruction | HIGH | HIGH | P1 |
| Differential harness and tolerance/order policy | HIGH | VERY HIGH | P1 |
| Upstream tests/examples and headless testbed | HIGH | HIGH | P1 |
| Safety, docs, platforms, benchmarks, release evidence | HIGH | VERY HIGH | P1 |
| Semantic snapshot/replay | MEDIUM | VERY HIGH | P2 |
| Ergonomic adapters and ecosystem integrations | MEDIUM | HIGH | P2 |
| Additional web/mobile targets | MEDIUM | VERY HIGH | P2 |
| `no_std`, alternative precision, parallel/SIMD extensions | LOW/MEDIUM | VERY HIGH | P3 |

**Priority key:**

- **P1:** Required before a full-parity v1 claim
- **P2:** Valuable post-parity production/ergonomic extension
- **P3:** Research-backed future option; must not delay or destabilize parity

## Competitor and Reference Analysis

| Capability | Google LiquidFun C++ | Rapier 2D Rust | liquidfun-rs approach |
| --- | --- | --- | --- |
| Rigid-body physics | Historical Box2D 2.3.0-derived behavior and API | Modern native-Rust rigid bodies, joints, CCD, queries, events, snapshotting, and optional determinism | Match pinned LiquidFun behavior rather than substituting a modern solver |
| LiquidFun particle behaviors | Canonical implementation and behavioral oracle | Not the LiquidFun behavioral/API oracle | Implement every pinned flag, group behavior, buffer, contact, query, and solver path natively |
| Rust safety/ergonomics | Pointer ownership and callback lock rules | Native typed Rust collections and Cargo workflow | Typed handles, explicit invalidation, borrow-safe views, deferred mutation |
| Compatibility evidence | Upstream tests/testbed, but no Rust differential proof | Its own behavior and test corpus | Public traceability matrix plus semantic C++/Rust differential evidence |
| Persistence | Diagnostic C++ reconstruction dump for rigid bodies/joints; not general particle serialization | Snapshotting is advertised by the official Rapier docs | Upstream-equivalent dump in v1; versioned semantic particle-aware snapshot later |
| Distribution | Historical C++ plus platform-specific build/binding stacks | Cargo-native | Cargo-native runtime; C++ oracle isolated to contributor workflows |

Rapier establishes a high ergonomic and operational bar for a production Rust physics library, but it is not evidence of LiquidFun compatibility. Its official documentation advertises native Rust 2D/3D rigid-body physics, joints, contact events, sensors, queries, snapshotting, optional determinism, SIMD, parallelism, serde, and WASM support; these are useful benchmarks for Rust user expectations, not a substitute implementation.

## Release Truthfulness Rules

- “Native Rust” means no production runtime delegation to C++.
- “Implemented” is not “validated”; compatibility states must distinguish planned, implemented, unit tested, differentially validated, platform validated, and intentionally unsupported.
- “Full parity” requires no unexplained compatibility-matrix gaps and every relevant upstream test/example accounted for.
- Numerical differences must name the observable, tolerance, platform/compiler scope, and cause when known.
- Performance statements must link methodology, workload definitions, versions, flags, hardware, and raw results.
- Supported-platform claims require CI or documented equivalent evidence; “compiles” alone is insufficient.
- 0.x releases may be useful and high quality while incomplete, but README, crate metadata, and docs must make missing particle/rigid behaviors conspicuous.
- Upstream diagnostic Dump must not be described as save/load or complete serialization.

## Sources

### Primary Upstream Sources

- [Google LiquidFun repository at candidate oracle commit 7f204021](https://github.com/google/liquidfun/tree/7f20402173fd143a3988c921bc384459c6a858f2)
- [LiquidFun README: purpose, version, particle extension, and historical platforms](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Readme.md)
- [LiquidFun release notes: Box2D 2.3.0/revision 280 ancestry and 1.0/1.1 particle features](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/ReleaseNotes.md)
- [Particle flags and particle data definitions](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2Particle.h)
- [Particle-system definitions, buffers, lifecycle, groups, contacts, queries, and controls](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h)
- [Particle-system solver passes and internal behavior implementation](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp)
- [Particle-group flags, definitions, and public API](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleGroup.h)
- [World API, stepping, queries, ray casts, origin shift, profiles, and Dump](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2World.h)
- [World Dump implementation, showing rigid-body/joint reconstruction output](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp)
- [Filters, rigid/particle contact listeners, destruction listeners, query callbacks, and ray-cast callbacks](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h)
- [Upstream Testbed scenario inventory](https://github.com/google/liquidfun/tree/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests)
- [Upstream dedicated unit-test tree](https://github.com/google/liquidfun/tree/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Unittests)

### Rust Ecosystem Comparison

- [Rapier official overview](https://rapier.rs/docs/)
- [Rapier official Rust getting-started guide and feature tradeoffs](https://rapier.rs/docs/user_guides/templates/getting_started/)
- [Rapier 2D query pipeline API](https://docs.rs/rapier2d/latest/rapier2d/pipeline/struct.QueryPipeline.html)

______________________________________________________________________

*Feature research for liquidfun-rs requirements definition*
*Researched: 2026-07-09*
