# Project Research Summary

**Project:** `liquidfun-rs`
**Domain:** Native Rust port of Google LiquidFun with a pinned C++ behavioral oracle
**Researched:** 2026-07-09
**Confidence:** MEDIUM-HIGH

## Executive Summary

`liquidfun-rs` should be built as a native, Cargo-first Rust physics engine that matches one deliberately selected historical LiquidFun revision. The C++ source is a development-only oracle for inventory, differential tests, reference data, and comparable benchmarks; it must never enter the published crate's runtime or normal build. The first release allowed to claim v1 parity must include the complete rigid-body and particle surface, not a rigid-body-only or common-effects subset. Earlier 0.x releases may expose useful increments only when their compatibility gaps are explicit.

Use one deep published `liquidfun` crate with private tooling crates around it. Keep math, collision, dynamics, particles, and world orchestration as cohesive internal modules until an independent public boundary is proven. Use Cargo for all consumer workflows, `xtask` and thin `just` recipes for contributor orchestration, and a modern CMake/Ninja wrapper for the legacy C++ reference build. Start differential testing across a versioned semantic JSON Lines subprocess protocol; add in-process FFI only if measured throughput requires it.

The largest risks are selecting the wrong oracle, losing translation/license provenance, committing to the wrong ownership and particle-storage model, and accepting numerical or ordering drift as harmless. Retire these risks before broad porting through an immutable upstream decision, a traceability matrix, an object-model spike, explicit step/order and tolerance policies, and an end-to-end empty-world then minimal-world differential slice. Correctness and scalar deterministic parity precede rendering, unsafe optimization, SIMD, or parallelism.

## Prescriptive Decisions

### Recommended Stack

The detailed rationale is in [STACK.md](STACK.md).

| Area | Recommendation | Status |
| --- | --- | --- |
| Rust | Pin Rust 1.97.0 for repository development; use Edition 2024 and resolver 3 | Adopt for foundation |
| MSRV | Declare Rust 1.92.0 initially and test the complete publishable surface against it | Provisional until release-policy review |
| Published shape | One `liquidfun` crate with deep internal modules and a small curated public API | Adopt unless an independent boundary is demonstrated |
| Production dependencies | Begin with only `bitflags` 2.13 and `thiserror` 2.0 | Expand only from implementation evidence |
| Private Rust tooling | `serde`, `serde_json`, `proptest`, exact-pinned `rand_chacha`, Criterion, `anyhow`, and `xshell` where their workflows begin | Keep outside the production graph |
| C++ oracle | Repository-owned CMake wrapper, CMake 4.3.3 CI pin, Ninja 1.13.2 CI pin, and a canonical Linux Clang/LLVM toolchain | Verify in a cross-platform foundation spike |
| Orchestration | Cargo owns Rust; `cargo xtask` owns cross-language workflows; `just` exposes thin aliases | Adopt |
| Differential boundary | Versioned semantic JSON Lines over an isolated subprocess | Adopt; design for batching/process reuse |
| CI | Fast Cargo-only PR lanes plus oracle smoke; expensive differential, fuzz, sanitizer, Miri, coverage, and benchmarks in scheduled/manual lanes | Adopt incrementally |
| Testbed | Headless scenarios and debug-draw abstraction first; prototype Macroquad later | Defer renderer selection to the testbed phase |

Do not adopt Bazel in the foundation. The upstream tree already has CMake, while Bazel would create a second Rust and C++ build graph without a measured need. Reconsider it only through an ADR backed by CI scale, hermeticity, or remote-execution evidence.

### V1 Full-Parity Table Stakes

The exhaustive inventory is in [FEATURES.md](FEATURES.md). Every row below is required before a full-parity v1 claim.

| Capability | V1 acceptance boundary |
| --- | --- |
| Native distribution | Published crates build, test, package, and run through Cargo without C++, CMake, Bazel, the upstream submodule, or reference data |
| Rigid bodies | Historical shapes, collision, broad/narrow phase, bodies, fixtures, contacts, islands, sleeping, CCD/TOI, all 11 joints, standalone rope, queries, ray casts, and world operations |
| Particles | All 18 flags, every baseline and conditional solver pass, storage, handles, lifecycle, lifetimes, groups, pairs/triads, contacts, body contacts, forces, queries, callbacks, and controls |
| Safe Rust model | Typed world-scoped handles, explicit invalidation and destruction cascades, restricted hooks, deferred mutation, borrow-safe bulk views, and safe external-buffer equivalents |
| Observability | Profiles/counts, renderer-independent debug draw, and upstream-equivalent diagnostic dump clearly documented as diagnostic rather than serialization |
| Examples | Every upstream test and example ported, replaced, or accounted for; shared renderer-neutral scenarios run headlessly and optionally in a testbed |
| Compatibility evidence | Immutable oracle provenance, exhaustive matrix, semantic differential traces, minimized regressions, and reviewed per-observable order/tolerance policy |
| Production evidence | Comparable benchmarks, safety audit, license/provenance audit, supported-platform CI, complete user/developer documentation, and a release gate with no unexplained gaps |

Typed handles, safe particle access, deferred mutation, a reproducible headless CLI, and evidence-linked compatibility reporting are not optional polish: they are how v1 provides upstream behavior without reproducing C++ hazards.

### Post-Parity Extensions

Keep these outside the v1 parity gate unless they become necessary to prove a table-stakes behavior:

- **v1.x:** versioned semantic scene snapshot/replay, ergonomic builders and optional serde adapters, game-engine adapters, and sustained WASM/mobile validation.
- **v2+:** a coherent `no_std` math/collision subset, opt-in SIMD or parallel stepping, and alternative precision modes.
- **Explicitly not v1:** general save/load disguised as upstream `Dump`, framework-coupled core simulation, default parallel stepping, and complete embedded/mobile/web promises without target evidence.

## Architecture and Data Flow

The detailed design is in [ARCHITECTURE.md](ARCHITECTURE.md).

### Major Components

1. **`liquidfun`** — the only initially published crate; owns math, collision, rigid dynamics, particle SoA storage and solvers, typed identity, and the `World` facade.
1. **Private test protocol** — owns validated engine-neutral scenarios, semantic traces, canonicalization rules, provenance, and tolerance profiles.
1. **Private differential runner** — adapts one scenario to Rust and C++, compares traces, diagnoses the first divergence, and drives minimization.
1. **C++ reference executable** — maps semantic IDs to pinned upstream pointers/indices in a separate process and emits semantic results only.
1. **Headless scenario runner and optional testbed** — share scenario definitions; rendering consumes public debug views and never owns simulation logic.

The production dependency direction is `world -> dynamics/particle -> collision -> math/settings`. Protocol, serialization, subprocess, renderer, and C++ code depend inward through public adapters but never enter the engine graph.

### Identity, Mutation, and Particle Storage

- Use distinct world-scoped generational IDs for bodies, fixtures, joints, particle systems, and groups. Centralize destruction and return owned destruction events.
- Treat contacts as transient views/snapshots rather than durable handles.
- Separate stable public `ParticleId` from ephemeral dense `ParticleIndex`; update both ID maps atomically with every SoA permutation, compaction, group rotation, and optional lane.
- Preserve particle group contiguity initially and make one authoritative permutation operation maintain every lane and derived identity.
- Expose borrow-scoped particle views and validated owned buffers. Do not promise unsafe raw-pointer equivalence merely because C++ accepted external arrays.
- Give hooks read-only views and narrow directives. Apply game mutations through a command buffer outside the locked step.

### Core Simulation Flow

1. Parse public definitions and commands into validated domain values and typed handles.
1. Apply a world mutation transaction and enter the locked step.
1. Execute named upstream-derived phases in the selected oracle's exact order: contact update, particle substeps/passes, rigid islands and constraints, then continuous-collision/TOI work.
1. Finalize forces, dirty state, stable ordering, and owned events; unlock and return `StepReport`/debug views.
1. For differential checks, run the same validated scenario through Rust and the C++ subprocess, verify provenance/schema, canonicalize only observables whose order is unspecified, then compare with field-specific policies.

No raw memory, pointers, padding, private layout, or dense particle index is compatibility evidence. IDs/flags/counts/membership are exact; numeric state uses reviewed absolute/relative/ULP rules; unspecified collections use set or multiset comparison; solver-significant and callback/destruction sequences remain ordered.

## Reconciled Research Boundaries

- **Candidate commit is not the final pin:** commit `7f20402173fd143a3988c921bc384459c6a858f2` was a useful 1.1.0-era inventory target. The project must separately decide between it, the v1.1.0 tag commit, or another defensible immutable revision after ancestry, build, patch, and license review.
- **Protocol package naming is not architecture:** `liquidfun-diff` in stack research and separate protocol/differential crates in architecture research express the same private boundary. Start with separate protocol and runner packages if both Rust and C++ adapters share the schema; package names remain an implementation detail.
- **Process lifetime is an optimization, not a semantic choice:** implement the subprocess protocol correctly for one scenario first, but make it streaming/batch-capable and keep the C++ process alive once startup cost matters. Isolation remains mandatory either way.
- **External-buffer parity means observable capability, not pointer-shaped API parity:** v1 needs safe bulk access and equivalent capacity/ownership behavior. A raw-pointer adapter is neither required nor allowed by default.
- **Diagnostic dump and snapshots are different products:** reproduce upstream diagnostic dump in v1; add a versioned particle-aware persistence format only after the public model stabilizes.
- **Tool and renderer versions are pins or candidates, not permanent product contracts:** validate CMake 4 legacy-policy handling, the initial MSRV, the canonical compiler, and Macroquad at their designated spikes.

## Critical Pitfalls

The full risk catalog and recovery guidance are in [PITFALLS.md](PITFALLS.md).

1. **Wrong upstream or missing provenance** — freeze the oracle decision before broad API/physics work; bind every trace, translated artifact, test, and datum to immutable source and license records.
1. **C++-pointer-shaped API** — prove stale/cross-world rejection, cascade invalidation, hook restrictions, and particle remapping before implementation hardens public types.
1. **Reentrant mutation and misleading events** — separate synchronous restricted directives from owned event reports and deferred commands; document event multiplicity and timing.
1. **Unspecified order mistaken for behavior** — classify each observable; canonicalize query-like results but preserve order wherever it affects solver state or promised event sequences.
1. **Flattened solver phases** — keep explicit upstream-derived phases and first-divergence probes; do not fuse or reorder passes for elegance or speed before parity.
1. **Particle identity/group corruption** — never expose dense indices as stable IDs; centralize lane permutations and property-test compaction, rotation, contiguity, capacity, and derived references.
1. **Weak or over-tolerant differential evidence** — version schemas and provenance, distinguish harness failures from physics mismatches, and never widen a global epsilon to make tests pass.
1. **Build leakage or premature optimization** — keep C++ out of Cargo consumer paths and preserve a safe scalar deterministic baseline until profiling and compatibility evidence justify change.
1. **False parity claims** — track planned, implemented, unit-tested, differentially validated, platform-validated, and intentionally unsupported states separately; demos never substitute for the matrix.

## Roadmap Implications

Use small dependency-driven phases with compatibility evidence as part of each phase's definition of done.

| Order | Phase | Delivers and retires |
| --- | --- | --- |
| 1 | Oracle, provenance, and repository foundation | Final immutable oracle decision; ancestry/license/notice record; inventory and matrix skeleton; Cargo workspace; pinned tools; Cargo-only package proof; CMake/Ninja build spike |
| 2 | Semantic protocol and oracle round trip | Validated schema, provenance handshake, bounded inputs, error taxonomy, empty-world Rust/C++ traces, reference-data rules, and initial comparator |
| 3 | Rust object-model and storage spike | World-scoped handles, stale/cross-world rejection, cascades, restricted hooks, event ownership, dense particle remap, group permutations, and property/compile-fail evidence |
| 4 | Math, settings, and numerical policy | Purpose-built `f32` primitives, transforms/sweeps/matrices, pure oracle probes, deterministic build flags, and initial tolerance/platform policy |
| 5 | Shapes and collision foundation | Four shapes, AABBs, distance/manifolds, dynamic tree, broad phase, and TOI with unit/property/differential coverage |
| 6 | Minimal rigid world vertical slice | Bodies, fixtures, contacts, creation/destruction, one non-colliding and one colliding step through the complete differential pipeline |
| 7 | Rigid solver, world operations, and CCD | Islands, constraints, warm starting, sleeping, queries/ray casts, sub-stepping, origin shift, profiling, and expanding rigid sign-off |
| 8 | Joints, rope, filters, listeners, and dump | All 11 joints, standalone rope, callback/filter/destruction timing, diagnostic dump, and broad rigid-body parity gate |
| 9 | Particle storage, lifecycle, and coupling | Systems, SoA lanes, proxies, stable IDs, creation/destruction/lifetimes, groups, contacts/body contacts, queries, forces, and buffer contracts |
| 10 | Particle solver behavior clusters | Baseline passes, all 18 flags, pairs/triads, solid/rigid behavior, joining/splitting, and flag-by-flag differential sign-off in pinned pass order |
| 11 | Examples, headless tooling, and testbed | Complete upstream test/example accounting, shared scenario catalog, deterministic captures, debug draw, headless CLI, then optional renderer adapter |
| 12 | Performance, portability, and release hardening | Comparable benchmarks, profile-led optimization, fuzz/Miri/sanitizers, desktop/server platform matrix, docs, safety/license/package audits, and final no-gap parity review |

### Ordering Rationale

- Freeze the source of truth and comparison vocabulary before accumulating translated behavior.
- Resolve handles, callback phases, and particle remapping before those choices infect every public API and test.
- Build collision and the rigid world before full particles because particle/body contacts reuse those foundations.
- Add differential evidence continuously: each subsystem should move through inventory, API design, minimal implementation, unit/property tests, oracle comparison, documentation, and sign-off.
- Port renderer-neutral scenarios before a visual testbed, and optimize only after the scalar behavior is validated.

### Research Flags for Phase Planning

Focused research or an ADR is still required for:

- **Phase 1:** final oracle commit/tag, Box2D ancestry, build patches, license obligations, alteration notices, and canonical compiler/platform.
- **Phase 2:** exact schema/versioning, float-bit encoding, batching/timeouts/crashes, canonical forms, reference-data review, and minimization strategy.
- **Phase 3:** handle bit layout and wrap policy, `Send`/`Sync`, user data, callback panic policy, destruction timing, stable particle-ID cost, and safe external-buffer semantics.
- **Phase 4:** oracle variability, fused operations, signed zero/NaN, divergence horizons, and per-platform tolerance tiers.
- **Phases 5-10:** pinned-source ordering audits and subsystem-specific observables before each compatibility sign-off.
- **Phase 11:** renderer selection only after a headless/testbed capability spike; Macroquad is the first candidate, not a commitment.
- **Phase 12:** performance budgets, supported-platform evidence thresholds, release/MSRV policy, and any unsafe optimization ADR.

Repository scaffolding, standard Cargo quality lanes, documentation checks, package isolation, and the renderer-neutral adapter boundary use established patterns once the decisions above are made; they do not need open-ended research phases.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH for Cargo/CMake/process isolation; MEDIUM for MSRV/compiler/renderer pins | Primary tool and upstream sources support the baseline; several pins require project-specific validation |
| Feature scope | HIGH | The candidate source inventory is detailed, but must be re-bound to the final oracle decision |
| Architecture | MEDIUM-HIGH | Dependency direction and safety boundaries are strong; handle, particle storage, callback, and numerical details need spikes/ADRs |
| Pitfalls | HIGH for upstream semantics; MEDIUM for recovery cost | Risks are consistent across official source behavior and the proposed Rust model |
| Roadmap | MEDIUM-HIGH | Dependency order is clear; phase sizes will refine after the final inventory and early spikes |

**Overall confidence:** MEDIUM-HIGH. The project has a coherent implementation strategy, but correctness depends on resolving the oracle, ownership, ordering, and numerical decisions before broad translation.

## Sources

### Project Research

- [STACK.md](STACK.md) — toolchain, workspace, C++ oracle, dependencies, CI, testing, and packaging.
- [FEATURES.md](FEATURES.md) — complete parity inventory, table stakes, enablers, extensions, and release truthfulness.
- [ARCHITECTURE.md](ARCHITECTURE.md) — component boundaries, identity/storage model, data flow, and dependency-driven build order.
- [PITFALLS.md](PITFALLS.md) — critical failure modes, warning signs, recovery, and phase mapping.

### Primary Upstream Evidence

- [Official Google LiquidFun repository](https://github.com/google/liquidfun) — archived source, releases, build files, tests, and examples.
- [Candidate 1.1.0-era research commit](https://github.com/google/liquidfun/tree/7f20402173fd143a3988c921bc384459c6a858f2) — inventory evidence only, not the final pin.
- [Official LiquidFun Programmer's Guide](https://google.github.io/liquidfun/Programmers-Guide.html) — modules, world semantics, contacts, queries, and particles.
- [LiquidFun release notes](https://google.github.io/liquidfun/ReleaseNotes.html) — 1.1.0 context and Box2D 2.3.0/revision 280 ancestry claim.

*Research completed: 2026-07-09*
*Ready for requirements and roadmap creation: yes*
