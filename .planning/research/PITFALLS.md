# Pitfalls Research

**Domain:** Native Rust port of Google LiquidFun with a pinned C++ differential oracle
**Researched:** 2026-07-09
**Confidence:** HIGH for upstream semantics; MEDIUM for recovery cost and future Rust architecture choices

## Critical Pitfalls

### Pitfall 1: Porting the Wrong Upstream

**What goes wrong:** The project implements behavior from modern Box2D, a binding fork, or a moving LiquidFun branch instead of the selected LiquidFun/Box2D 2.3.0 lineage.

**Why it happens:** The official repository is archived, its layout is historical, and current Box2D documentation is easier to find.

**How to avoid:** Pin an immutable commit, record release ancestry and patches, inventory its exact API/source/test surface, and make every oracle trace carry that provenance.

**Warning signs:** Documentation cites current Box2D APIs; the submodule follows a branch; compatibility rows have no upstream file/commit; examples disagree before Rust code exists.

**Recovery:** Freeze work, reconstruct provenance, diff the assumed and actual upstream surfaces, invalidate affected requirements/tests, and re-baseline deliberately.

**Phase to address:** Foundation and upstream provenance, before public API or physics implementation.

### Pitfall 2: Losing License and Translation Provenance

**What goes wrong:** Translated algorithms, tests, fixtures, or reference data ship without required zlib notices, alteration statements, or traceable origins.

**Why it happens:** A permissive license is mistaken for no obligations, and incremental translation obscures which source informed which Rust module.

**How to avoid:** Complete the license review before translation; preserve notices; record source commit/path for derived code, tests, and data; audit packaged crate contents.

**Warning signs:** Source ports lack provenance notes; generated reference data has no manifest; the project license is chosen before review; `cargo package --list` contains unexplained upstream assets.

**Recovery:** Stop release work, inventory derived material, restore notices and alteration records, replace unverifiable assets, and obtain legal review for ambiguity.

**Phase to address:** Foundation, then every subsystem sign-off and release audit.

### Pitfall 3: Designing the API Around C++ Pointers

**What goes wrong:** Raw pointers, long-lived borrows, unscoped indices, or globally reusable IDs recreate dangling references and make safe mutation impractical.

**Why it happens:** Mirroring upstream object graphs appears to maximize compatibility and postpones hard ownership decisions.

**How to avoid:** Prove a world-scoped generational handle model with stale/cross-world rejection, destruction reports, borrow-scoped views, and cascade tests before broad implementation.

**Warning signs:** Handles lack a world identity; generation wrap is unspecified; callbacks retain references; destroying a body can silently leave valid-looking fixture/joint handles.

**Recovery:** Introduce typed IDs and an arena boundary behind the public API, migrate callers with compatibility adapters, and add property tests for invalidation before resuming porting.

**Phase to address:** Object-model spike before math/collision work hardens public types.

### Pitfall 4: Permitting Reentrant World Mutation

**What goes wrong:** Listeners receive unrestricted `&mut World`, mutate structures while the world is locked, invalidate solver state, or create timing that cannot match upstream.

**Why it happens:** Rust callbacks are made ergonomic without first modeling upstream lock and callback semantics.

**How to avoid:** Expose restricted synchronous hook contexts, owned event snapshots, narrowly supported pre-solve edits, and deferred commands applied at documented phase boundaries.

**Warning signs:** Listener APIs expose general world mutation; callback order is undocumented; panics can unwind through FFI; tests mutate bodies during contact callbacks.

**Recovery:** Deprecate unrestricted hooks, add a command buffer and event journal, specify timing, and differentially revalidate every callback scenario.

**Phase to address:** Object model and minimal world vertical slice; expand during contacts/joints.

### Pitfall 5: Treating Events as Unique Stable Facts

**What goes wrong:** Clients assume contact callbacks are deduplicated, persistent, or safe for immediate mutation even though upstream events may be transient or repeated and mutation is deferred.

**Why it happens:** Event streams are interpreted as a normalized domain log rather than observations of solver transitions.

**How to avoid:** Document multiplicity and lifetime, compare event multisets/sequences according to upstream guarantees, and keep persistent game state separate from raw callback events.

**Warning signs:** Tests assert one callback without a stated guarantee; callback payloads contain borrowed internal storage; event consumers delete objects immediately.

**Recovery:** Version the event contract, add normalization only as an opt-in layer, and re-record affected traces with explicit semantics.

**Phase to address:** Contact/listener design and differential protocol design.

### Pitfall 6: Comparing Unspecified Order as Behavior

**What goes wrong:** Differential tests fail on query, ray-cast, contact, or collection order that upstream does not guarantee—or accidentally make incidental order part of the Rust API.

**Why it happens:** Bytewise snapshots and vector equality are simpler than classifying observable ordering contracts.

**How to avoid:** Maintain an ordering audit. Compare guaranteed sequences in order, unordered results as canonicalized sets/multisets, and internal iteration only where it affects subsequent physics.

**Warning signs:** Tests sort everything indiscriminately; query results differ but semantic membership matches; hash maps appear in solver paths; ties have no stable policy.

**Recovery:** Classify each observable, version comparator rules, minimize scenarios to identify physically consequential order, and avoid masking solver-order divergence with broad sorting.

**Phase to address:** Differential protocol foundation, then every subsystem sign-off.

### Pitfall 7: Flattening the Upstream Step and Solver Order

**What goes wrong:** Refactoring for elegance changes particle-before-rigid ordering, solver pass order, warm-start timing, contact update timing, or TOI sequencing and causes cumulative drift.

**Why it happens:** Individual passes appear mathematically commutative when their floating-point and stateful interactions are not.

**How to avoid:** Encode the world step as an explicit orchestrator, inventory the pinned pass order, trace phase boundaries, and validate vertical slices before abstraction or fusion.

**Warning signs:** A single generic loop replaces named passes; optimizations fuse passes before parity; divergence grows immediately after one phase; source-order mappings are absent.

**Recovery:** Restore explicit phases, add per-pass trace probes, bisect at the first divergent phase, and make ordering a documented compatibility decision.

**Phase to address:** Minimal world slice, rigid solver, and every particle-behavior cluster.

### Pitfall 8: Giving Ephemeral Particle Indices Stable Meaning

**What goes wrong:** Public callers persist dense indices that change after zombie compaction, sorting, group rotation, or buffer growth.

**Why it happens:** Upstream APIs expose indices heavily, while Rust callers expect handles to remain valid or fail explicitly.

**How to avoid:** Separate stable `ParticleId` from dense storage index, update the ID-to-index map on every move, scope index views to borrows, and test random create/destroy/compact sequences.

**Warning signs:** IDs equal vector positions; deletion shifts public identity; snapshots compare index membership; handles silently refer to another particle after compaction.

**Recovery:** Add a stable identity lane and remap layer, migrate serialized/reference scenarios to semantic IDs, and invalidate all index-based fixtures and traces.

**Phase to address:** Early particle-storage spike, before particle contacts or behaviors.

### Pitfall 9: Breaking Particle Group Contiguity and Buffer Semantics

**What goes wrong:** Group ranges stop being contiguous, pair/triad membership becomes stale, external buffer capacity is misinterpreted, or optional lanes drift during rotation/compaction.

**Why it happens:** A straightforward vector/arena design ignores upstream's structure-of-arrays synchronization and `RotateBuffer`/capacity rules.

**How to avoid:** Define one authoritative permutation primitive for every particle lane and index map; model group ranges and external-buffer ownership/capacity as invariants; property-test permutations.

**Warning signs:** Each lane moves itself; group bounds need repair passes; external buffers reallocate unexpectedly; optional buffers have different lengths; pair/triad IDs survive moves incorrectly.

**Recovery:** Centralize permutations, rebuild derived contacts/pairs/triads, add invariant checks in debug/test builds, and revalidate group lifecycle scenarios.

**Phase to address:** Particle storage/groups before solver behavior implementation.

### Pitfall 10: Using Weak Differential Evidence

**What goes wrong:** The harness compares raw memory, uses unversioned JSON, regenerates golden data silently, or cannot distinguish implementation bugs from oracle/protocol/platform failures.

**Why it happens:** A quick one-off FFI test grows into the compatibility system without a protocol design.

**How to avoid:** Use process-isolated runners, a versioned validated scenario schema, semantic IDs, provenance manifests, explicit tolerances/order policies, timeouts, crash reports, and reproducible seeds.

**Warning signs:** C++ pointers appear in traces; golden files lack commit/compiler/flags; test runs rewrite snapshots; a crash looks like a mismatch; failures cannot run by scenario and seed.

**Recovery:** Freeze affected compatibility claims, version the protocol, regenerate from the pinned oracle into a reviewed change, and retain minimized regressions.

**Phase to address:** Foundation before significant physics implementation; recurring gate thereafter.

### Pitfall 11: Hiding Numerical Drift Behind Broad Tolerances

**What goes wrong:** Large global epsilons make tests green while phase, ordering, sign, NaN, or stability bugs remain.

**Why it happens:** Floating-point differences are expected, so unexplained divergence is prematurely classified as acceptable noise.

**How to avoid:** Define per-observable absolute/relative/ULP or domain tolerances, divergence horizons, NaN/signed-zero policy, compiler flags, and platform tiers; measure baseline oracle variability.

**Warning signs:** One epsilon covers every field; tolerance grows whenever tests fail; first-step divergence is ignored; platform/compiler provenance is missing.

**Recovery:** Minimize the scenario, compare phase checkpoints, classify root cause, narrow tolerances from evidence, and document any intentional difference.

**Phase to address:** Math/protocol foundation and every differential sign-off.

### Pitfall 12: Premature Optimization, SIMD, or Parallelism

**What goes wrong:** Layout fusion, unsafe access, SIMD/FMA, or parallel scheduling changes deterministic order and obscures correctness before parity is established.

**Why it happens:** Physics engines invite performance work and benchmark wins are visible earlier than compatibility evidence.

**How to avoid:** Benchmark validated workloads, profile first, preserve a scalar deterministic baseline, gate accelerations behind explicit features, and rerun differential/stability suites for every optimization.

**Warning signs:** Optimization commits precede end-to-end traces; unsafe code has no measured benefit; thread count changes results; synthetic benchmarks dominate roadmap decisions.

**Recovery:** Revert to the validated baseline, isolate one optimization at a time, prove equivalence or document an opt-in behavioral mode, and retain regression benchmarks.

**Phase to address:** Performance/hardening only after subsystem parity, with recurring checks.

### Pitfall 13: Over-Fragmenting the Cargo Workspace

**What goes wrong:** Tiny crates expose internal contracts, create feature/version/MSRV friction, and make cross-cutting solver changes expensive without real isolation benefits.

**Why it happens:** The upstream directory tree is mistaken for publishable crate boundaries.

**How to avoid:** Start with one deep published engine crate plus private tooling crates; extract only proven independent release, platform, compile-time, or `no_std` boundaries.

**Warning signs:** Cyclic conceptual ownership despite acyclic Cargo dependencies; many `pub` internals; duplicated types/features; changes touch most crate manifests.

**Recovery:** Merge shallow crates behind modules, narrow public surfaces, and preserve compatibility through re-exports only when users already depend on them.

**Phase to address:** Foundation architecture and revisit after collision/rigid evidence.

### Pitfall 14: Letting the Legacy C++ Oracle Control the Product Build

**What goes wrong:** CMake 2.8-era assumptions, compiler drift, or Bazel complexity leak into ordinary Cargo builds and make the Rust library hard to consume.

**Why it happens:** A unified build appears simpler than maintaining a deliberate development-only boundary.

**How to avoid:** Keep oracle build/run commands in `xtask`/`just`, use CMake/Ninja with explicit compatibility flags and pinned CI compilers, and exclude C++ from published crates and default Cargo paths.

**Warning signs:** `cargo build` needs a submodule or C++ compiler; CMake downloads dependencies; golden data changes across compiler jobs; Bazel duplicates Cargo ownership without measured need.

**Recovery:** Split the oracle into a private subprocess tool, restore Cargo-only defaults, manifest compiler/flags, and adopt broader orchestration only through an ADR with evidence.

**Phase to address:** Foundation build spike and cross-platform CI.

### Pitfall 15: Coupling Scenarios to the Renderer

**What goes wrong:** Example logic lives inside window/input callbacks, so headless tests, C++ comparison, minimization, and deterministic replay implement different scenarios.

**Why it happens:** Porting the visual testbed feels like the fastest way to demonstrate progress.

**How to avoid:** Define declarative/versioned scenarios and a renderer-neutral runner first; make the testbed a consumer of public commands, debug views, and step reports.

**Warning signs:** Physics setup imports graphics crates; headless mode has separate scenario code; screenshots substitute for semantic traces; rendering owns timestep state.

**Recovery:** Extract scenario definitions and controls, route both headless and visual runners through them, and re-baseline examples against the oracle.

**Phase to address:** Protocol foundation and headless examples before interactive testbed work.

### Pitfall 16: Declaring Parity From Demos or Partial Tests

**What goes wrong:** Visible examples work, but inventory gaps, rare particle flags, callbacks, dumping, platform behavior, or upstream tests remain unaccounted for.

**Why it happens:** A visually convincing engine creates pressure to market maturity before evidence is complete.

**How to avoid:** Make compatibility states explicit; require every upstream row, test, example, and behavior to be implemented, replaced, irrelevant with rationale, or intentionally unsupported; gate claims on traceability coverage.

**Warning signs:** README says “full parity” without coverage counts; “implemented” means compiled; missing cases are not listed; performance claims lack methodology.

**Recovery:** Correct public claims immediately, publish the gap list, add roadmap phases for unmapped rows, and run a release-blocking parity audit.

**Phase to address:** Every phase transition and final parity release.

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
| --- | --- | --- | --- |
| Raw integer handles | Minimal implementation | Silent cross-world/stale aliasing | Only inside a private throwaway spike |
| One global numeric epsilon | Fast test authoring | Hides phase and stability defects | Never for parity sign-off |
| Golden snapshots without provenance | Small files | Cannot reproduce or trust evidence | Never |
| Direct translation of upstream lists/pointers | Familiar mapping | Unsafe, shallow public model | Only inside isolated oracle code |
| Many small public crates | Apparent modularity | API/MSRV/release coupling | Only after an independent contract is proven |
| Renderer-owned scenarios | Fast visual demo | Duplicated and untestable behavior | Never beyond a disposable prototype |
| Unpinned tools/actions | Easy updates | Non-reproducible CI and traces | Local experiments only |
| Adding Bazel immediately | One orchestration story | Duplicate build ownership and maintenance | Only after measured Cargo/CMake pain and an ADR |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
| --- | --- | --- |
| LiquidFun submodule | Follow `master` or patch in place | Pin a commit; store documented compatibility patches separately |
| Legacy CMake | Assume modern CMake accepts policy 2.8 | Supply documented policy compatibility externally and test every host |
| C++ runner | Link it into the published crate | Run a private process with versioned semantic protocol |
| Reference data | Regenerate automatically on failure | Require an explicit reviewed command and provenance manifest |
| Cargo/MSRV | Let dev tools raise library MSRV | Test publishable crates at MSRV; run tools on the pinned development toolchain |
| GitHub Actions | Pin only mutable major tags | Pin immutable SHAs and update in reviewed commits |
| Testbed renderer | Expose engine storage for drawing | Consume public debug snapshots and scenario controls |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
| --- | --- | --- | --- |
| Rebuilding all particle-derived structures after every small change | Step time grows sharply with particle count | Preserve dirty flags and upstream-equivalent update scope after parity | Large dynamic particle systems |
| Stable IDs implemented with per-particle heap objects | Allocation/cache cost dominates | Dense SoA plus ID/index maps | Creation/destruction-heavy workloads |
| Canonicalizing all differential output in hot loops | Harness is slower than simulation | Emit compact traces and canonicalize at comparison boundaries | Large randomized corpora |
| Spawning one C++ process per step | Differential suite becomes unusable | Long-lived runner with batched scenarios and timeouts | Multi-step/property testing |
| Parallelizing ordered solver passes | Run-to-run and thread-count drift | Retain scalar deterministic baseline; explicit opt-in only | Any coupled contacts/particles |
| Benchmarking unmatched compiler flags/workloads | Misleading speed claims | Manifest versions, flags, hardware, warmup, and scenario input | First public performance claim |

## Security and Safety Mistakes

| Mistake | Risk | Prevention |
| --- | --- | --- |
| Unbounded scenario/reference input | Memory or CPU exhaustion in tools/CI | Validate schema, sizes, step counts, and timeouts before allocation/execution |
| C++ panic/exception/UB crossing FFI | Process corruption or undefined behavior | Prefer subprocess isolation; catch/report failures at the runner boundary |
| Unsound zero-copy buffer exposure | Aliasing and use-after-reallocation | Borrow-scoped views, capacity invariants, and audited narrow unsafe blocks |
| Unverified third-party source/data | License, provenance, or supply-chain exposure | Checksums, immutable pins, allowlists, `cargo-deny`, and artifact manifests |
| Unsafe optimization without an invariant test | Silent memory corruption | `SAFETY:` rationale, focused tests, Miri/sanitizers where applicable |

## Developer Experience Pitfalls

| Pitfall | User Impact | Better Approach |
| --- | --- | --- |
| C++-shaped Rust API | Borrowing and destruction are surprising | Recognizable concepts with typed IDs, explicit invalidation, and safe views |
| Hidden maturity gaps | Users trust unsupported behavior | Public compatibility matrix and precise crate/README status |
| Expensive default commands | Contributors avoid verification | Fast Cargo defaults; opt-in/scheduled oracle and extended suites |
| Opaque `just` recipes | Failures are hard to reproduce | Thin recipes that print and document underlying commands |
| Rendering required for examples | CI/server/WASM users are excluded | Headless scenarios first, optional renderer second |

## "Looks Done But Isn't" Checklist

- [ ] **Upstream pin:** The commit exists, but ancestry, license, patches, compiler, and source inventory are also recorded.
- [ ] **Rust API:** Happy paths compile, but stale/cross-world handles, cascaded destruction, callback mutation, and panic behavior are tested.
- [ ] **Rigid world:** Demos run, but sleeping, CCD/TOI, ordering, filters, listeners, queries, ray casts, dumping, and every joint are traced.
- [ ] **Particle core:** Positions render, but all 18 flags, unflagged passes, groups, pair/triads, lifetimes, external buffers, compaction, and callbacks are covered.
- [ ] **Differential suite:** Tests pass, but traces are semantic, provenance-bound, reproducible, minimized, tolerance-specific, and platform-classified.
- [ ] **Performance:** Benchmarks exist, but workloads, flags, hardware, versions, and parity state are comparable.
- [ ] **Cross-platform:** Code compiles, but required behavior is exercised in CI and platform differences are documented.
- [ ] **Release:** Crates package, but compatibility rows, docs, notices, upstream tests/examples, and public maturity claims have passed audit.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
| --- | --- | --- |
| Wrong upstream or missing provenance | HIGH | Freeze work; re-inventory; invalidate affected artifacts; re-pin and re-baseline |
| Handle/callback model failure | HIGH | Prototype corrected model; migrate behind adapters; property-test invalidation and reentrancy |
| Particle identity/group corruption | HIGH | Centralize permutations; rebuild ID/group maps; invalidate index-based traces; add randomized invariants |
| Solver/order drift | MEDIUM/HIGH | Add phase probes; minimize at first divergence; restore explicit order; sign off again |
| Weak oracle protocol | MEDIUM | Version schema; add provenance/errors/timeouts; explicitly regenerate reviewed snapshots |
| Over-broad tolerance | MEDIUM | Measure oracle variability; classify fields/platforms; narrow policy and re-run corpus |
| Build-oracle leakage | MEDIUM | Isolate C++ process/tooling; restore Cargo-only defaults; document/pin host tools |
| Premature parity claim | HIGH reputational | Correct claims; publish gaps; map missing rows; require independent release audit |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
| --- | --- | --- |
| Wrong upstream | Foundation/provenance | Immutable pin, ancestry ADR, complete initial inventory |
| License/provenance loss | Foundation + release gate | License report, source mappings, package-content audit |
| Pointer-shaped API | Object-model spike | Stale/cross-world/cascade property and compile-fail tests |
| Reentrant mutation/events | Object model + contacts | Restricted contexts, deferred-command timing, oracle event traces |
| Unspecified ordering | Protocol + every subsystem | Ordering classification and comparator tests per observable |
| Step/solver reordering | World/solver + particle behavior | Named phase traces and first-divergence localization |
| Ephemeral particle indices | Particle-storage spike | Random compaction with stable semantic IDs |
| Group/buffer corruption | Particle core/groups | Lane permutation, contiguity, capacity, and external-buffer invariants |
| Weak differential evidence | Protocol foundation | Version/provenance/crash/seed/minimization acceptance suite |
| Numerical tolerance masking | Math + recurring sign-off | Measured per-field/platform policy and divergence-horizon tests |
| Premature optimization | Performance/hardening | Baseline equivalence plus profile and regression benchmarks |
| Workspace over-fragmentation | Foundation architecture | Public-boundary review and dependency-direction audit |
| C++ build leakage | Foundation/CI | Default Cargo build from clean source without C++ prerequisites |
| Renderer coupling | Headless examples before testbed | One scenario runs identically headless, visual, and through oracle |
| False parity | Every transition + release | 100% compatibility/test/example traceability with no unexplained gaps |
| Long-horizon scope failure | Roadmap and milestone gates | Small dependency-driven phases with explicit evidence and gap roll-forward |

## Sources

Primary upstream evidence was inspected at candidate commit `7f20402173fd143a3988c921bc384459c6a858f2`. This is research evidence, not the final pin decision.

- [LiquidFun 1.1.0 release notes and Box2D 2.3.0/revision 280 ancestry](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/ReleaseNotes.md)
- [Upstream zlib license and altered-source notice requirements](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/License.txt)
- [`b2World::Step` locking and particle/rigid/TOI phase order](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp#L976-L1043)
- [World callbacks, mutation warnings, and query/ray interfaces](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h)
- [Contact guide: callback timing and mutation restrictions](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter09_Contacts.md#L250-L263)
- [World guide: unspecified query and ray-cast ordering](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter10_World.md)
- [Particle guide: self-compacting indices and property buffers](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter11_Particles.md)
- [Particle handles: ephemeral indices](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2Particle.h#L324-L350)
- [Particle-system implementation: SoA lanes, sorting, solver order, compaction, and rotation](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp)
- [Particle-system API: external buffers and lifecycle controls](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h)
- [Legacy upstream CMake configuration](https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/CMakeLists.txt)

*Pitfalls research for: `liquidfun-rs`*
*Researched: 2026-07-09*
