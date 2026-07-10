# liquidfun-rs

## What This Is

`liquidfun-rs` is a production-quality, open-source Rust implementation of Google's LiquidFun physics engine for Rust game, simulation, visualization, and research developers. It aims for complete behavioral and feature parity with a deliberately selected and pinned upstream C++ revision while remaining a genuinely independent Rust library rather than bindings around the original implementation.

The repository will retain upstream C++ LiquidFun as a read-only development oracle for research, differential testing, reference data, and benchmark comparison. Ordinary users of the published Rust library must not need the upstream source, a C++ compiler, Bazel, or any cross-language runtime component.

## Core Value

Deliver an independent, maintainable Rust physics engine whose LiquidFun behavior is demonstrated against a pinned upstream oracle through explicit inventory, tests, differential evidence, and documented tolerances.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet — ship to validate)

### Active

<!-- Current scope. Building toward these. -->

- [ ] Select the canonical LiquidFun reference repository and pin an exact revision with its release context, Box2D ancestry, rationale, license obligations, and required notices documented.
- [ ] Establish a cohesive Cargo workspace and crate/module architecture that keeps Cargo sufficient for normal Rust development and use.
- [ ] Build a complete upstream subsystem, public API, source, example, and test inventory with a traceability matrix that records Rust implementation, unit-test, differential-test, benchmark, compatibility, and documentation status.
- [ ] Design an idiomatic Rust public object model for worlds, bodies, fixtures, joints, particles, groups, callbacks, user data, mutation, destruction, stable identity, and invalidation without exposing raw C++ ownership patterns.
- [ ] Implement the Box2D-compatible mathematical, geometric, collision, broad-phase, narrow-phase, solver, sleeping, query, ray-cast, and continuous-collision foundations present in the selected LiquidFun revision.
- [ ] Implement rigid-body worlds, bodies, fixtures, contacts, all supported shapes and joints, filters, listeners, destruction behavior, debug drawing abstractions, and upstream-equivalent world operations.
- [ ] Implement the full LiquidFun particle system, including storage, creation, destruction, lifetimes, buffers, spatial proxies, contacts, body contacts, groups, flags, pair/triad logic, queries, ray casts, callbacks, and every upstream solver behavior.
- [ ] Build a first-class C++/Rust reference harness for seeded, reproducible, and minimizable differential scenarios with semantic state comparison, configurable tolerances, machine-readable output, and human-readable diagnostics.
- [ ] Create layered unit, integration, upstream-compatibility, differential, property, fuzz, Miri, sanitizer, and regression testing appropriate to each subsystem.
- [ ] Port or account for upstream tests and examples, with an optional renderer-independent testbed that supports interactive inspection, headless execution, deterministic capture, and Rust/C++ comparison.
- [ ] Define and enforce numerical-stability, ordering, determinism, and cross-platform tolerance policies before treating differential results as compatibility evidence.
- [ ] Measure performance against equivalent upstream C++ workloads and optimize only from profiling evidence without silently sacrificing API clarity, safety, determinism, or parity.
- [ ] Support Linux x86_64, Linux ARM64, macOS ARM64, macOS x86_64 where practical, and Windows x86_64, while investigating WASM, mobile, and realistic smaller `no_std` subsets.
- [ ] Provide discoverable repository automation through a root `justfile`, CI, pinned toolchains, dependency/license policy, documentation checks, coverage, benchmarks, and scheduled extended verification.
- [ ] Maintain accurate project, architecture, upstream, compatibility, testing, benchmarking, safety, contribution, release, and roadmap documentation without claiming unverified maturity or parity.

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Using the upstream C++ library as the runtime implementation — the production deliverable must be an independent Rust engine.
- Publishing a thin Rust binding layer as the port — FFI exists only for development-time reference, testing, and benchmarking workflows.
- Requiring C++, Bazel, the upstream submodule, or reference data for ordinary Cargo consumers — cross-language tooling is a repository-development concern.
- Mechanically translating the whole upstream repository in one pass — each subsystem must move through inventory, API design, minimal implementation, unit tests, upstream comparison, differential validation, optimization, documentation, and compatibility sign-off.
- Claiming full parity or production readiness before the traceability matrix has no unexplained gaps and the documented acceptance evidence exists — status reporting must remain precise.
- Treating an unrelated modern Box2D implementation as automatically compatible — reuse requires license review and behavioral validation against the selected LiquidFun ancestry.
- Coupling core simulation crates to a rendering or game-engine framework — visualization is optional and simulation remains headless.
- Introducing default parallel simulation that changes deterministic or upstream-compatible behavior — any such mode must be explicit and documented.
- Promising complete-engine `no_std`, embedded, iOS, Android, or WASM support before feasibility is established — smaller portable subsets may be pursued when evidence supports them.
- Selecting a final project license before upstream and derivative-work compatibility is reviewed — required notices and provenance come first.

## Success Standard

Full feature parity requires all of the following:

- Every relevant public upstream feature is implemented or explicitly documented as irrelevant to the Rust library.
- Every upstream particle behavior, supported shape, supported joint, world operation, callback, and query has a Rust equivalent.
- Upstream examples have Rust equivalents, and upstream tests are ported, replaced, or explicitly accounted for.
- Differential tests cover representative and edge-case behavior using semantic state, reproducible inputs, and documented tolerances.
- Known behavioral, numerical, ordering, platform, API, and performance differences are documented.
- Performance is measured against upstream with comparable workloads and optimization levels.
- The production Rust library has no runtime dependency on C++ or the upstream source.
- The compatibility matrix contains no unexplained gaps.

## Delivery Strategy

Development proceeds incrementally and in dependency order:

1. Establish repository foundations, toolchains, documentation, licensing analysis, CI, upstream pinning, and the build-orchestration decision.
1. Inventory upstream features, APIs, tests, examples, build systems, Box2D ancestry, and LiquidFun-specific changes.
1. Define the compatibility matrix, numerical policy, reference harness, and Rust ownership/object model.
1. Port foundational math, shapes, collision primitives, dynamic tree, and broad phase.
1. Port rigid-body dynamics, contacts, solvers, sleeping, continuous collision detection, queries, and ray casts.
1. Port joints, callbacks, filters, listeners, and debug draw abstractions.
1. Establish broad rigid-body differential validation.
1. Port particle storage, lifecycle, spatial structures, contacts, and body contacts.
1. Port particle solvers, flags, behaviors, groups, and pair/triad logic incrementally.
1. Establish comprehensive particle differential validation.
1. Port examples and testbed scenarios with headless support.
1. Optimize from benchmarks and profiling, then harden safety, platforms, documentation, and release policy.
1. Complete a parity audit before publication claims.

The ordering is a starting hypothesis. Research may refine phase boundaries, but it must preserve dependency-aware, testable progress and early risk retirement.

## Expected Early Deliverables

Before substantial physics porting begins, the project should have:

- A refined project definition, scoped requirements, executable roadmap, risk register, and milestone acceptance criteria.
- `ARCHITECTURE.md`, `UPSTREAM.md`, `COMPATIBILITY.md`, `TESTING.md`, and dependency/licensing analysis.
- Decision records for build orchestration, the Rust object model, and differential testing.
- A pinned upstream Git submodule and documented reference build/test commands.
- Initial Cargo workspace scaffolding, a root `justfile`, pinned Rust toolchain, and minimal CI for both the Rust skeleton and upstream reference implementation.
- A verified subsystem/API inventory and initial compatibility traceability matrix.

## Context

- The repository is greenfield: it currently contains the project prompt, Bright Builds Rules, and repository metadata but no Rust implementation or Cargo manifest.
- Google LiquidFun extends a historical Box2D lineage with particle simulation. The exact canonical repository, stable reference revision, Box2D ancestry, maintenance state, and build behavior must be researched rather than assumed.
- The upstream C++ implementation is the behavioral oracle during development, not a production dependency or the desired public architecture.
- Particle simulation is a central deliverable, not an optional extension after rigid-body work.
- The project is necessarily long-running and may span multiple milestones; compatibility status must therefore be visible and evidence-based throughout development.
- Safe abstractions may alter layout, identity, ordering, and performance relative to C++. These differences need deliberate designs and differential evidence rather than incidental behavior.
- Callback reentrancy, mutation during stepping, stable object identity, destruction invalidation, intrusive-list replacement, external particle buffers, and user data are early architectural risks.
- Numerical divergence may arise from floating-point precision, compiler optimization, SIMD, collection ordering, solver order, and platform behavior. Exact bit parity is not assumed where unjustified.
- Existing permissively licensed Rust implementations may be considered only after provenance, license, ancestry, API, and behavioral compatibility review.
- Repository tooling should make advanced workflows discoverable without hiding their underlying commands or making ordinary Cargo use opaque.

## Constraints

- **Implementation**: Production physics behavior must be native Rust — runtime delegation to upstream C++ is prohibited.
- **Reference isolation**: FFI and C++ builds are limited to differential testing, comparison, reference generation, benchmark comparison, and upstream test/example execution — published crates remain independent.
- **Build system**: Cargo is primary and sufficient for normal users — Bazel, CMake, or hybrid orchestration requires a documented evidence-based decision.
- **Upstream provenance**: The canonical source and exact revision must be pinned before implementation assumptions harden — moving branches are not acceptable references.
- **Licensing**: Upstream LiquidFun, Box2D, copied or translated code, tests, data, and all dependencies require explicit license review and attribution — final project licensing follows compatibility analysis.
- **Safety**: Safe Rust is the default — every `unsafe` block must be narrow, justified by a measurable need, document its invariant with a `SAFETY:` comment, and receive focused tests where practical.
- **API design**: Public APIs must be idiomatic, recognizable to LiquidFun users, explicit about handles/lifetimes/invalidation/callbacks/mutation, and must not expose raw pointers or unstable storage details.
- **Behavior**: Compatibility is measured against the selected upstream behavior — differences need documented causes, tolerances, and regression protection.
- **Determinism**: Stable ordering and reproducible seeded scenarios take precedence over unproven parallel or SIMD gains — nondeterministic acceleration must be explicit.
- **Testing**: Meaningful semantic state must be compared — serialized raw memory alone is not an acceptable compatibility oracle.
- **Quality**: Production code avoids `unwrap()`, propagates errors, uses useful invariant messages for genuinely impossible states, documents public APIs, and follows the repository's Rust and Bright Builds guidance.
- **Angles and naming**: Full rotations use tau-based expressions, and optional internal values use `maybe_` naming where it improves clarity — project conventions remain consistent with repository standards.
- **Architecture**: Prefer cohesive deep modules and functional-core/imperative-shell separation — do not over-fragment crates or hide substantial foreign-language logic inside strings.
- **Rendering**: Simulation stays renderer-independent, optional, and headless — testbed framework choices cannot dictate core architecture.
- **Platforms**: Initial portability targets mainstream Linux, macOS, and Windows architectures — broader targets remain research-backed extensions.
- **CI cost**: Pull-request checks should remain useful and reasonably fast — expensive randomized, differential, sanitizer, coverage, and benchmark suites may run on schedules or manual triggers.
- **Transparency**: Documentation and README maturity claims must match verified implementation and compatibility evidence — incomplete parity is never marketed as complete.

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Build a genuine Rust implementation rather than production C++ bindings | Independence, safety, idiomatic APIs, Cargo usability, and long-term maintainability are core goals | — Pending |
| Use a pinned upstream C++ LiquidFun revision as the behavioral oracle | Compatibility needs a stable, inspectable target and reproducible evidence | — Pending |
| Restrict FFI and C++ tooling to development-time comparison workflows | Published Rust users must not inherit a C++ runtime or toolchain dependency | — Pending |
| Keep Cargo primary and sufficient for ordinary use | Rust consumers and contributors need a conventional, transparent workflow | — Pending |
| Evaluate Bazel or hybrid orchestration before adoption | Cross-language and CI orchestration may help, but maintenance cost must be justified | — Pending |
| Prefer safe Rust and encapsulate any necessary unsafe code | Safety must not be traded away without measurable benefit and explicit invariants | — Pending |
| Treat compatibility inventory and differential testing as first-class product work | Parity claims require traceable evidence, not implementation intuition | — Pending |
| Design the Rust ownership, handle, callback, and user-data model before broad porting | C++ pointer and mutation semantics are foundational and expensive to revise late | — Pending |
| Keep particle systems in core scope | Particle behavior is the defining LiquidFun extension and cannot be deferred as optional polish | — Pending |
| Keep rendering optional and simulation headless | Core portability, testing, server use, and framework independence depend on this boundary | — Pending |
| Prioritize correctness and parity before optimization | Premature layout, SIMD, or parallel decisions could hide incompatibilities and destabilize the API | — Pending |
| Require explicit compatibility sign-off per subsystem | Incremental, reviewable evidence prevents premature global parity claims | — Pending |

## Open Questions

- Which upstream repository and exact commit provide the most defensible canonical LiquidFun reference?
- Which historical Box2D version and LiquidFun-specific changes does that revision contain?
- What license and notice obligations apply to translated implementation details, tests, reference data, and derivative work?
- Should repository-wide reference orchestration use upstream tooling, CMake wrappers, Bazel with `rules_rust`, Cargo build support, or a narrow hybrid?
- What crate boundaries provide meaningful isolation without fragmenting the engine?
- Which handle, arena, lifetime, callback, user-data, and mutation model best balances safety, API ergonomics, identity stability, and upstream behavior?
- Which observables and tolerances define acceptable parity for each subsystem and platform?
- How should deterministic iteration and contact ordering be preserved without exposing internal storage?
- Which existing permissively licensed Rust components, if any, are compatible enough to reuse after audit?
- Which lightweight rendering stack best supports an optional upstream-equivalent testbed without coupling the core library?
- Which subsets can realistically support WASM, mobile, or `no_std` without distorting the full engine?

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
1. Requirements validated? → Move to Validated with phase reference
1. New requirements emerged? → Add to Active
1. Decisions to log? → Add to Key Decisions
1. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
1. Core Value check — still the right priority?
1. Audit Out of Scope — reasons still valid?
1. Update Context with current state

______________________________________________________________________

*Last updated: 2026-07-09 after initialization*
