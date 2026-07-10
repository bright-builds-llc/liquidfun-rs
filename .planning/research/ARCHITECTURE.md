# Architecture Research

**Domain:** Native Rust 2D rigid-body and particle physics engine with a C++ differential oracle
**Researched:** 2026-07-09
**Confidence:** MEDIUM-HIGH — upstream structural claims are verified against the official source, but the canonical revision, numerical policy, and public object model still require explicit decisions

## Standard Architecture

### System Overview

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Imperative shells                                                    │
│ testbed app      differential CLI      reference-data generator      │
└──────────────┬───────────────┬─────────────────────┬─────────────────┘
               │               │                     │ subprocess
┌──────────────▼───────────────▼──────────────┐  ┌───▼────────────────┐
│ Versioned test protocol                    │  │ C++ oracle runner   │
│ validated scenarios, semantic traces,      │  │ pinned upstream +  │
│ canonicalization, tolerance profiles       │  │ narrow local shim   │
└──────────────┬──────────────────────────────┘  └────────────────────┘
               │ drives / observes
┌──────────────▼───────────────────────────────────────────────────────┐
│ Published Rust engine: one cohesive `liquidfun` crate                │
│                                                                      │
│ `World` facade and step coordinator                                  │
│      ├── rigid dynamics ──► collision ──► math/settings              │
│      └── particle system ─► rigid access + collision + math          │
│                                                                      │
│ typed handles + arenas      particle SoA      ordered scratch state  │
└──────────────────────────────────────────────────────────────────────┘
```

Dependency arrows point toward lower-level knowledge. The published engine never depends on protocol, differential, renderer, C++, serialization, or process-management code. The inspected official source describes Common, Collision, and Dynamics as its major modules, makes Collision independently usable, and has `b2World` own the simulation objects. The Rust layout should preserve those useful knowledge directions without copying the C++ ownership interface.

### Component Responsibilities

| Module | Owns | Must not own |
| --- | --- | --- |
| `liquidfun::math` | `Vec2`, rotations, transforms, sweeps, matrices, constants, numerical predicates | World state, allocation policy, callbacks |
| `liquidfun::collision` | Shapes, AABBs, distance, manifolds, dynamic tree, broad phase, ray/shape casts, TOI kernels | Bodies, particles, renderer, oracle types |
| `liquidfun::dynamics` | Body/fixture/joint/contact state, islands, rigid solver, sleeping, CCD | Particle storage, C++ adapter, public event loop |
| `liquidfun::particle` | Dense particle properties, groups, proxies, contacts, pairs/triads, lifetimes, ordered solver pipeline | Public world ownership, rendering, serialization |
| `liquidfun::world` | Deep public interface, typed object identity, creation/destruction cascades, queries, step ordering, hooks/events | Upstream pointers, protocol identifiers, graphics |
| `liquidfun-test-protocol` | Versioned scenario/result domain types, validation, semantic IDs, canonical forms, tolerance profiles | Engine internals or upstream build logic |
| `liquidfun-differential` | Rust scenario adapter, C++ process adapter, trace comparison, diagnostics, minimization entrypoints | Production engine behavior |
| `tools/reference-cpp` | Map protocol IDs to upstream pointers/indices; run the pinned C++ world; emit semantic results | Rust public types, generalized application behavior |
| `apps/testbed` | Window/input/render loop and interactive controls over the public engine/scenario interfaces | Physics implementation or differential truth |

The engine crate should be a deep module: a small safe `World` interface hides arenas, ordered adjacency, scratch buffers, solver phases, and compaction. Do not split math, collision, dynamics, or particles into published crates merely because upstream has directories. Extract a production crate only after it has an independent consumer, feature/platform contract, or release cadence.

## Recommended Project Structure

```text
.
├── crates/
│   ├── liquidfun/                    # only published production crate initially
│   │   └── src/
│   │       ├── lib.rs                # curated public re-exports
│   │       ├── handle.rs             # typed world-scoped generational IDs
│   │       ├── arena.rs              # private storage/invalidation machinery
│   │       ├── math.rs
│   │       ├── math/
│   │       ├── collision.rs
│   │       ├── collision/
│   │       ├── dynamics.rs
│   │       ├── dynamics/
│   │       ├── particle.rs
│   │       ├── particle/
│   │       ├── world.rs
│   │       └── world/
│   ├── liquidfun-test-protocol/      # private, engine-neutral schema and policies
│   └── liquidfun-differential/       # private adapters/comparator/diagnostics
├── tools/
│   ├── reference-cpp/                # separate C++ executable and build files
│   └── xtask/                        # cross-platform development orchestration
├── apps/testbed/                     # optional renderer-dependent binary, later
├── protocol/                         # generated JSON Schemas and compatibility notes
├── scenarios/                        # small declarative cases and minimized regressions
├── reference-data/                   # manifests plus reviewed Cargo-only snapshots
├── third_party/liquidfun/            # pinned, read-only oracle source
└── docs/adr/                          # object model, oracle, ordering/numerical decisions
```

### Structure Rationale

- **One production crate:** permits private refactoring across tightly coupled physics subsystems and prevents dependency/version ceremony from becoming part of the public interface.
- **Separate protocol crate:** serialization and comparison vocabulary are real seams shared by multiple runners, but are not runtime physics concerns.
- **Separate differential crate:** keeps `serde`, subprocesses, minimizers, and diagnostics out of ordinary Cargo consumers.
- **Process-level C++ adapter:** prevents foreign pointers, allocators, exceptions, undefined behavior, and sanitizers from crossing into the Rust process. Cargo builds of `liquidfun` remain C++-free.
- **Renderer as an app:** the testbed consumes renderer-neutral snapshots/views and cannot dictate world storage or dependencies.

## Architectural Patterns

### Deep Stateful Core with Pure Kernels

Math, geometry, manifold generation, key construction, result canonicalization, and tolerance evaluation should be data-in/data-out functions with exhaustive unit tests. `World` and its solvers are necessarily mutable in-memory state machines, but remain effect-free: no files, clocks, randomness, logging policy, rendering, or subprocesses. Those effects belong to shells.

Do not create traits around every internal call. There is one rigid implementation and one particle implementation; direct private types give better locality. Introduce a seam only where two adapters exist, such as Rust and C++ scenario runners or interactive and headless presentation.

### World-Scoped Generational Identity

Bodies, fixtures, joints, particle systems, and particle groups should use distinct typed IDs containing a world key, arena slot, and generation. This prevents stale-slot reuse and accidental cross-world lookup. Fixtures retain a `BodyId`; joints retain their endpoint IDs. Arena entries may use index-based `maybe_prev`/`maybe_next` links internally to reproduce required traversal order without exposing intrusive pointers.

Destruction is centralized in `World`: validate once, perform the upstream-equivalent cascade in a defined order, invalidate generations, update adjacency/broad-phase state, and return owned destruction events. Contacts are transient implementation records; expose contact views/snapshots, not durable contact handles.

Particle identity is deliberately two-level:

- Internal `ParticleIndex` is a dense, ephemeral solver index.
- Public `ParticleId` is stable across dense-buffer rotation and compaction until destruction.
- `ParticleStorage` maintains `dense_to_id` and `id_to_dense` maps and updates them in the same transaction as every SoA lane, proxy, contact, pair, triad, lifetime index, and group range.

This is safer than treating an index as identity. Official LiquidFun documents indices as self-compacting and offers a separate handle for cross-frame identity. Benchmark the always-on stable map early; if its cost is material, an opt-in handle table may be considered only through an ADR and equivalent safety tests.

### Restricted Hooks and Explicit Mutation Phases

The public interface should make illegal callback mutation unrepresentable. A step hook receives read-only snapshots/handles; pre-solve returns a narrow `ContactDirective` rather than `&mut World` or an unrestricted contact pointer. Stateful hooks may observe synchronous begin/end/pre/post events, but can store only owned values. All events are also available as an owned `StepReport` for polling workflows.

```rust
pub trait StepHooks {
    fn should_collide(&mut self, pair: CollisionPairView<'_>) -> bool;
    fn pre_solve(&mut self, contact: ContactView<'_>) -> ContactDirective;
    fn observe(&mut self, event: StepEventView<'_>);
}
```

World mutation requested by game logic goes into a `CommandBuffer` and is applied explicitly before or after `step`, never while solver state is borrowed. A no-hooks path should remain simple. This matches the upstream prohibition on creating/destroying entities in contact callbacks while allowing the supported pre-solve decisions.

Queries happen outside the locked step and return owned typed IDs or use a `ControlFlow` visitor. Their interface must not imply callback order: the official guide explicitly says query and ray-cast callback order is unspecified.

### Particle SoA Behind Borrow Guards

Retain the upstream's structure-of-arrays model because solver passes and rendering consume properties lane-wise. Keep required lanes (`id`, flags, position, velocity, force, weight, group) dense and feature-specific lanes lazy where evidence supports it. Keep reusable scratch buffers owned by the particle system so stepping does not allocate per pass. Preserve group-contiguous ranges until differential evidence justifies another representation.

Expose read-only slices through `ParticleBuffers<'_>`. Controlled mutation should use a guard whose methods expose only supported lanes and mark proxies/contacts or aggregate flags dirty when the guard ends. Never expose the backing `Vec`s, capacity, or dense-to-ID map.

User-supplied buffers should transfer ownership in a validated `ParticleBuffersOwned` object rather than borrow arbitrary external memory for the world's lifetime. The world may return them on teardown. A raw-pointer compatibility interface, if ever necessary, belongs in an explicitly unsafe low-level module and is not an initial requirement.

### Versioned Semantic Differential Protocol

The scenario protocol is an engine-neutral domain language, not a serialization of either object model.

```text
Scenario
  schema version + scenario ID + seed
  world/solver configuration
  named entity definitions and deterministic creation order
  commands at explicit phases (before-step / after-step)
  declarative filter and pre-solve rules
  checkpoint and observable requests

EngineTrace
  schema/provenance/platform/float metadata
  checkpoint index and simulation time
  semantic body/joint/particle-system/group/particle states
  canonical contact/query/ray results
  ordered callback and destruction events
  runner warnings/errors
```

Scenario IDs map to typed Rust handles in the Rust adapter and to pointers plus `b2ParticleHandle`s in the C++ adapter. Neither representation crosses the protocol. Group-created particles receive deterministic semantic IDs at creation, allowing traces to remain aligned after compaction.

Comparison happens by policy, not raw bytes:

| Observable | Comparison policy |
| --- | --- |
| IDs, flags, counts, membership, event kind | Exact |
| Positions, velocities, impulses, normals, weights | Field-specific absolute/relative/ULP policy |
| Query/ray/contact collections whose upstream order is unspecified | Canonical key, then set/multiset comparison |
| Callback/destruction sequence and any solver-significant order | Ordered comparison |
| NaN, infinity, signed zero, missing entity | Explicit diagnostic; never silently tolerated |

Tolerance profiles are versioned data keyed by subsystem and platform policy. They cannot be widened automatically by a failing test.

## Data Flow

### Simulation Flow

```text
public definitions / command buffer
              │ parse + validate handles and invariants
              ▼
      World mutation transaction
              │
              ▼
 step config ─► lock phase
              ├─ update rigid contacts
              ├─ particle substeps and body coupling
              ├─ rigid islands / velocity and position solve
              ├─ continuous-collision / TOI phase
              ├─ finalize forces, order, dirty state, owned events
              ▼
          unlock phase ──► StepReport / DebugView / next commands
```

The exact phase order is an oracle-version fact, not a generic Box2D assumption. In the inspected official commit, `b2World::Step` updates contacts, solves each particle system, solves rigid islands, then handles TOI; `b2ParticleSystem::Solve` itself has an explicit ordered sequence of conditional solver passes. Encode these as named internal phases and lock the selected revision's order with tests.

### Differential Verification Flow

```text
scenario source
      │ parse once into ValidatedScenario
      ├──────────────► Rust adapter ─────► Rust EngineTrace
      │
      └─ JSON request ► C++ subprocess ──► C++ EngineTrace
                                               │
traces ─► schema/provenance checks ─► canonicalize ─► compare policies
                                                        │
                           pass / DifferentialReport / minimized scenario
```

The C++ runner reads machine input from stdin or a file, writes only protocol output to stdout, and sends logs to stderr. One process may execute a scenario or bounded batch; process reuse is a later optimization. A crash, timeout, sanitizer failure, schema mismatch, or wrong upstream revision is a harness failure, never a physics mismatch.

### Reference Data Flow

Small reviewed scenarios and minimized regressions are source-controlled. Cargo-only snapshots include a manifest with protocol version, scenario content hash, exact upstream revision, adapter revision, build mode, target, and tolerance profile. Generation writes to a temporary directory, validates by rerunning, then atomically replaces artifacts. Stale provenance is an error. Large randomized traces and benchmark output should remain CI artifacts rather than repository fixtures.

## Determinism and Ordering Ownership

The engine owns an explicit iteration-order policy. Solver-critical paths use arenas/vectors and stable total keys, never hash-map iteration. Creation ordinals are retained where needed. Every sort used by proxies, contacts, pairs, triads, islands, lifetimes, or semantic output must document whether equal keys are meaningful and supply a deterministic tie-breaker.

Do not promise cross-platform bit identity initially. Define and verify two levels:

1. Same target, toolchain, features, inputs, and seed: reproducible trace, subject to a published contract.
1. Different supported platforms: semantic compatibility under reviewed numerical tolerances.

The inspected upstream source uses head insertion for world lists, sorted spatial proxies, group-driven buffer rotation, and zombie compaction. These details can change solver order and therefore belong in the upstream inventory and differential tests, even when the public Rust iteration interface chooses a clearer order.

## Scaling Considerations

| Scale | Architecture response |
| --- | --- |
| Small worlds / correctness work | Simple arenas, full invariant checks in tests, checkpoint every step |
| Tens of thousands of particles | Reused SoA/scratch capacity, compact contact records, sampled checkpoints, profile allocation and cache behavior |
| Very large particle workloads | Profile first; consider specialized sorting/SIMD behind identical scalar tests; keep deterministic scalar mode authoritative |

The likely first bottlenecks are particle proxy sorting/contact generation and repeated trace serialization, not crate boundaries. Optimize solver storage independently from the public handle interface. Parallel or nondeterministic modes must be opt-in and cannot become the compatibility baseline.

## Anti-Patterns

### Crate per Upstream Directory

It freezes implementation dependencies into public package seams and makes coordinated solver changes expensive. Keep one production crate with cohesive internal modules until extraction has evidence.

### C++ Hidden Behind the Rust Runtime

Linking the oracle in `build.rs` or a public feature risks making C++ a production dependency and lets foreign failures corrupt the test process. Keep it in a separate development executable.

### Pointer-Shaped Rust Interfaces

Long-lived references, raw pointers, or unscoped indices recreate upstream invalidation hazards. Use typed world-scoped IDs, borrow-scoped views, and owned event snapshots.

### Arbitrary `&mut World` in Callbacks

It makes reentrancy and solver invalidation possible. Use restricted directives and deferred commands.

### Bytewise or Indexwise Differential Comparison

Layouts and particle indices are implementation details. Compare semantic IDs and fields with explicit order and tolerance policies.

### Renderer-Owned Scenarios

If examples live inside graphics callbacks, headless, C++, and minimizer execution drift apart. Scenarios must be declarative and renderer-neutral.

## Integration Points

### Internal Seams

| Seam | Communication | Ownership rule |
| --- | --- | --- |
| `world` → `dynamics` | Direct private calls over typed state | World owns phase order; dynamics owns rigid invariants |
| `world` → `particle` | Direct private calls with a narrow rigid-access context | Particle may query fixtures/apply body reaction; it cannot own the world |
| engine ↔ hooks | Read-only views, directives, owned events | No callback receives `&mut World` |
| scenario → adapter | `ValidatedScenario` | Adapter maps semantic IDs; scenario never sees engine handles |
| comparator → traces | Canonical semantic records | Comparator never inspects engine memory |
| testbed → engine | Public commands, `StepReport`, `DebugView` | No storage or solver access |

### External Services and Tools

| Integration | Pattern | Notes |
| --- | --- | --- |
| Pinned LiquidFun | Read-only submodule used by C++ runner | Exact revision and patches recorded; absent from published crates |
| C++ toolchain | Subprocess build/run through documented `xtask`/`just` commands | Native upstream build first; add broader orchestration only if evidence requires it |
| Reference snapshots | Manifested generated artifacts | Never accept mismatched provenance or silently regenerate in tests |
| Renderer/windowing | Optional testbed dependencies | Selected after headless runner works; cannot enter engine dependency graph |

## Dependency-Driven Build Order

| Order | Deliverable and acceptance evidence | Risk retired |
| --- | --- | --- |
| 1 | Select/pin oracle, record ancestry/license/build, freeze an initial subsystem inventory | Moving target and false structural assumptions |
| 2 | Versioned protocol plus empty-world C++ and Rust runner round trip on all practical host OSes | Cross-language build and schema risk before physics work |
| 3 | Object-model spike: world-scoped arenas, stale/cross-world handles, destruction cascades, restricted hooks, dense particle ID remap; property and compile-fail tests | Most expensive ownership/API mistakes and particle compaction cost |
| 4 | Math/settings and scalar primitives with unit/property tests and pure-operation oracle probes | Numerical conventions, units, basic comparator policy |
| 5 | Shapes, AABBs, dynamic tree, broad phase, distance/manifolds, and TOI in dependency order | Collision foundation and deterministic candidate ordering |
| 6 | World/body/fixture/contact storage and a minimal non-colliding then colliding step vertical slice through the differential pipeline | World orchestration, identity, event, and harness integration |
| 7 | Islands, rigid solver, sleeping, queries/ray casts, and CCD with expanding rigid scenarios | Highest rigid numerical/order risks |
| 8 | Joints and full filters/listeners/destruction behavior; broad rigid sign-off | Callback timing and graph topology |
| 9 | Particle storage/lifecycle/groups/lifetimes/proxies, then particle and body contacts | SoA, stable identity, compaction, group contiguity, coupling |
| 10 | Particle behavior passes in the selected upstream order, pairs/triads, splitting/joining; sign off each flag before the next cluster | Central LiquidFun algorithm and cumulative divergence risk |
| 11 | Headless example catalog, then optional testbed over the same scenarios | Prevents renderer-driven architecture and scenario duplication |
| 12 | Benchmarks/profile-guided layout work, portability, fuzz/Miri/sanitizers, parity audit | Optimizes only validated behavior and hardens release claims |

The early particle-storage spike is intentionally separated from the later full particle implementation: it retires the public identity and memory-layout risks before they become entrenched, while the full particle solver waits for collision and rigid-body dependencies.

## Decisions Requiring Research Before Implementation

1. **Canonical oracle:** exact repository, commit/tag, Box2D ancestry, supported build path, license/notice consequences, and whether patches are unavoidable.
1. **Handle representation:** world-key generation, generation width/wrap policy, ID size, `Send`/`Sync` expectations, destruction reports, and arena iteration parity.
1. **Particle identity/storage:** cost of always-stable `ParticleId`, group-contiguous layout, lazy lanes, buffer ownership, safe bulk mutation, and upstream handle-observation effects.
1. **Callback contract:** exact synchronous event order, legal pre-solve edits, destruction events outside steps, panic containment, and command-buffer timing.
1. **Ordering audit:** head-insert lists, island traversal, dynamic-tree pair emission, equal proxy tags, contact sorting, lifetime sorting, group rotation, and platform-specific `std::sort` behavior in the chosen oracle.
1. **Numerical policy:** `f32` semantics, compiler flags, fused operations/SIMD, warm starting, NaN/signed-zero handling, per-observable tolerances, divergence horizons, and platform tiers.
1. **Protocol/oracle technology:** JSON library and schema generation, streaming/batching, timeout/crash protocol, sanitizer builds, snapshot size, and scenario minimization strategy.
1. **User data:** external typed side tables keyed by IDs versus engine-owned tags/generics; avoid `Any`/raw-pointer compatibility by default.
1. **Build orchestration:** prove whether native CMake/scripts plus Cargo/`xtask` are sufficient before adopting Bazel or any second workspace model.
1. **Crate extraction/platform subsets:** only after evidence shows collision/math need an independent `no_std` or release contract.

## Sources

Structural evidence was inspected in the official `google/liquidfun` repository at commit `7f20402173fd143a3988c921bc384459c6a858f2`. This is evidence for architecture research, not the final pin decision.

- [Official LiquidFun Programmer's Guide — modules, world ownership, units](https://google.github.io/liquidfun/Programmers-Guide.html)
- [Official `b2World::Step` source — lock and particle/rigid/TOI phase order](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp#L976-L1043)
- [Official world callbacks — mutation warning and contact/query/ray interfaces](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h)
- [Official contacts guide — buffer events and mutate only after the step](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter09_Contacts.md#L250-L263)
- [Official world guide — query and ray-cast order is unspecified](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter10_World.md)
- [Official particle guide — self-compacting indices and contiguous property buffers](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter11_Particles.md)
- [Official particle handle definition — indices are ephemeral](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2Particle.h#L324-L350)
- [Official particle system source — SoA allocation, proxy sorting, solve sequence, compaction, and buffer rotation](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp)
- [Official LiquidFun 1.1.0 release notes — Box2D 2.3.0/revision 280 ancestry claim](https://google.github.io/liquidfun/ReleaseNotes.html)

*Architecture research for: `liquidfun-rs`*
*Researched: 2026-07-09*
