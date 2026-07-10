# Project: Full Rust Port of Google LiquidFun

Create a production-quality, open-source Rust implementation of Google’s LiquidFun physics engine with the long-term goal of full behavioral and feature parity with the original C++ project.

The project must be a genuine Rust port, not merely Rust bindings around the C++ library. The final library should be usable independently of the original implementation, while the upstream C++ source may remain available inside the repository as a reference implementation and differential-testing oracle.

## Primary Goal

Implement the complete LiquidFun API and simulation behavior in Rust, including:

- Box2D-compatible 2D rigid-body physics
- LiquidFun particle systems
- Particle groups
- Particle contacts
- Particle-body contacts
- Particle constraints and behaviors
- Collision detection
- Broad phase and narrow phase
- Continuous collision detection
- Joints
- Shapes
- Fixtures
- Bodies
- Worlds
- Queries and ray casts
- Debug draw abstractions
- Serialization or dumping functionality provided by upstream
- All examples, tests, benchmarks, and utilities necessary to establish behavioral parity

The project should ultimately provide full feature parity with the selected upstream LiquidFun reference version.

## Important Constraints

### Native Rust implementation

The production library must be implemented in Rust.

Do not use the upstream C++ library as the runtime implementation behind a Rust API.

FFI may be used only for:

- Differential testing
- Behavioral comparison
- Benchmark comparison
- Generating reference outputs
- Running upstream tests or examples
- Validating edge cases during development

The published Rust library must not require a C++ compiler or the upstream LiquidFun source.

### Safety

Prefer safe Rust throughout the public API and core implementation.

Use `unsafe` only when it provides a measurable and justified benefit, such as:

- Carefully encapsulated low-level memory layouts
- SIMD
- FFI to the reference implementation
- Performance-critical data structures where safe alternatives are insufficient

Every `unsafe` block must:

- Be narrowly scoped
- Include a `SAFETY:` comment
- Document the invariant being relied upon
- Be covered by focused tests where practical

### Determinism and numerical behavior

Preserve upstream behavior as closely as practical.

The project must explicitly investigate:

- Floating-point precision
- Numerical stability
- Determinism across runs
- Determinism across platforms
- Ordering-dependent simulation behavior
- Differences introduced by Rust iteration order or collection types
- Compiler optimization effects
- SIMD differences
- Tolerance requirements for differential tests

Exact bit-for-bit parity may not always be realistic. Where exact parity is not achievable, define documented numerical tolerances and explain why.

## Upstream Reference Implementation

Include the selected upstream LiquidFun source as a Git submodule, preferably under:

```text
third_party/liquidfun
```

Before implementation begins:

1. Identify the most appropriate canonical upstream repository.
1. Determine the latest stable or otherwise best-supported reference revision.
1. Pin the submodule to an exact commit.
1. Record the upstream repository URL, commit hash, release information, and rationale in documentation.
1. Preserve the upstream license and required notices.
1. Do not silently track a moving branch.

Add convenience commands for:

- Initializing submodules
- Updating the submodule intentionally
- Printing the pinned upstream revision
- Building upstream LiquidFun
- Running upstream tests
- Running selected upstream examples
- Generating reference simulation outputs

Treat upstream code as read-only unless a small compatibility patch is absolutely necessary. Store any required patches separately and document them.

## Build System

Use Cargo as the primary build system for the Rust implementation.

Consider Bazel for repository-wide orchestration where it materially helps with:

- Building the upstream C++ reference imementation
- Compiling upstream tests
- Compiling cross-language comparison harnesses
- Running Rust and C++ tests through one command
- Multi-platform builds
- Cross-compilation
- Reproducible toolchains
- CI build matrices

Do not introduce Bazel merely for appearance. During project planning, evaluate:

- Bazel with `rules_rust`
- Native upstream build tooling
- CMake wrappers
- Cargo build scripts
- A hybrid Cargo and Bazel setup

Document the decision and its tradeoffs.

Cargo must remain sufficient for ordinary Rust users who only want to build and use the Rust library.

A likely division is:

- Cargo for Rust development, testing, examples, benchmarks, and publishing
- Bazel for monorepo orchestration, upstream C++ builds, cross-language testing, and selected CI workflows

## Justfile

Provide a root-level `justfile` with discoverable convenience commands.

At minimum, consider recipes such as:

```text
just
just list
just bootstrap
just init-submodules
just build
just build-release
just check
just test
just test-all
just test-unit
just test-integration
just test-differential
just test-upstream
just lint
just fmt
just fmt-check
just clippy
just docs
just examples
just bench
just miri
just fuzz
just coverage
just build-upstream
just update-upstream
just print-upstream-revision
just generate-reference-data
just bazel-build
just bazel-test
just ci
just clean
```

The default recipe should print available commands rather than performing an expensive action.

Recipes should be thin wrappers around documented underlying commands. Avoid hiding important behavior inside opaque shell scripts.

Any shell scripts must use:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

## Repository Structure

Start with a Cargo workspace.

Determine the exact crate boundaries during architecture planning, but consider a structure similar to:

```text
.
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── NOTICE
├── justfile
├── MODULE.bazel
├── BUILD.bazel
├── rust-toolchain.toml
├── deny.toml
├── crates
│   ├── liquidfun
│   ├── liquidfun-core
│   ├── liquidfun-math
│   ├── liquidfun-collision
│   ├── liquidfun-dynamics
│   ├── liqu examples
├── benches
├── fuzz
├── tests
├── tools
├── reference-data
├── docs
└── third_party
    └── liquidfun
```

This structure is illustrative, not mandatory.

Do not over-fragment the implementation into tiny crates without clear architectural boundaries. Prefer cohesive modul lines when doing so improves readability.

Use modern Rust module organization:

```text
module.rs
module/
├── child.rs
└── another_child.rs
```

Do not default to `module/mod.rs`.

## Public API Design

Studyl LiquidFun and Box2D APIs in detail before finalizing the Rust API.

The Rust API should:

- Be idiomatic
- Preserve recognizable LiquidFun concepts
- Make migration from C++ understandable
- Avoid blindly reproducing C++ ownership patterns
- Avoid raw pointers in the public API
- Avoid unnecessary allocation
- Make invalid states difficult to represent
- Use explicit handles, indices, lifetimes, or ownership models where appropriate
- Clearly define object invalidation behavior
- Clearly define callback behavior
- Clearly define mutation rules during world stepping
- Avoid exposing unstable internal storage details

Important design questions include:

- Whether bodies, fixtures, joints, particles, and groups use generational handles
- How destruction invalidates handles
- How callbacks interact with mutable world access
- How contact listeners are represented
- How user data is represented
- Whether user data is generic, type-erased, externally stored, or feature-gated
- How allocator behavior maps from C++ to Rust
- How intrusive linked lists are replaced
- How upstream iteration order is preserved
- How particle buffers are exposed without violating aliasing rules
- Whether raw low-level APIs coexist with safer high-level APIs
- How closely API names follow upstream naming
- Whether optional compatibility aliases should exist

Create an API design document before committing to the foundational object model.

## Compatibility Scope

Build a complete upstream feature inventory.

Create a traceability matrix that maps every relevant upstream component to its Rust status.

The matrix should include:

- Upstream file or subsystem
- Upstream classes and functions
- Rust module
- Rust type or function
- Implementation status
- Unit-test status
- Differential-test status
- Benchmark status
- Known behavioral differences
- Documentation status

Use explicit states such as:

- Not investigated
- Planned
- In progress
- Implemented
- Unit tested
- Differentially validated
- Fully validated
- Intentionally unsupported

Do not claim full parity until the matrix is complete and the acceptance criteria are met.

## Box2D Foundation

LiquidFun is built on Box2D. Determine precisely which Box2D version and modifications exist in the selected LiquidFun revision.

Do not assume that using an unrelated current Box2D port automatically provides parity.

Investigate:

- LiquidFun-specific changes to Box2D
- Differences from modern Box2D
- Historical API behavior
- Solver differences
- Collision behavior
- Memory layout assumptions
- Contact ordering
- Sleep behavior
- Continuous collision detection
- Joint implementations
- Timestep and iteration semantics

The Rust implementation may reuse suitable permissively licensed Rust code only after a license and compatibility review. Any reused code must be attributed, audited, and tested against the selected LiquidFun behavior.

A clean-room or independent implementation is acceptable and may be preferable.

## Particle-System Scope

The particle system is a central deliverable, not an optional extension.

Build a detailed inventory of all particle flags and group flags, including behavior for concepts such as:

- Water particles
- Zombie particles
- Wall particles
- Spring particles
- Elastic particles
- Viscous particles
- Powder particles
- Tensile particles
- Color-mixing particles
- Destruction-listener particles
- Barrier particles
- Static-pressure particles
- Reactive particles
- Repulsive particles
- Fixture-contact listener/filter particles
- Particle-contact listener/filter particles

Verify the actual upstream inventory and names rather than relying on this list alone.

Port and validate:

- Particle creation and destruction
- Particle lifetimes
- Particle buffers
- Particle groups
- Group splitting and joining
- Contact generation
- Body contacts
- Particle contacts
- Voronoi or related internal algorithms
- Spatial proxies
- Sorting
- Pair and triad generation
- Pressure
- Damping
- Viscosity
- Elasticity
- Springs
- Tensile forces
- Powder behavior
- Static pressure
- Color mixing
- Barriers
- Repulsion
- Collision solving
- Stuck-particle detection
- Particle queries
- Particle ray casts
- External buffers
- User-overridable buffers
- Destruction callbacks

## Differential Testing

Differential testing against upstream is a first-class requirement.

Create a reference harness capable of running equivalent simulations in:

- Upstream C++ LiquidFun
- The Rust port

Compare observable state after each step or at selected checkpoints.

At minimum compare:

- Body transforms
- Linear and angular velocities
- Sleep state
- Contact counts
- Joint state
- Particle positions
- Particle velocities
- Particle colors
- Particle flags
- Particle-group membership
- Particle contacts
- Particle-body contacts
- Destruction events
- Query results
- Ray-cast results

The harness should support:

- Seeded scenarios
- Randomized property tests
- Reproducible failures
- Small minimized regression fixtures
- Configurable numerical tolerances
- Human-readable diffs
- Machine-readable output
- Reference-data snapshots
- Running a single scenario by name or seed

Do not rely exclusively on serialized raw memory. Compare meaningful semantic state.

When outputs differ, determine whether the cause is:

- A Rust implementation bug
- Unspecified upstream behavior
- Floating-point tolerance
- Iteration ordering
- Platform variation
- A defect in the comparison harness

Every fixed differential failure should become a regression test.

## Testing Strategy

Use several test layers.

### Unit tests

Test small mathematical and algorithmic components independently.

Follow Arrange, Act, Assert structure.

Keep one primary concept per test.

### Integration tests

Test full subsystems and public API behavior.

### Upstream compatibility tests

Port upstream tests where licensing permits.

Maintain a mapping between upstream tests and Rust equivalents.

### Differential tests

Run equivalent simulations in C++ and Rust.

### Property tests

Use property-based testing for:

- Geometry invariants
- Broad-phase behavior
- Contact symmetry
- Handle validity
- Particle indexing
- Serialization round trips
- Query correctness
- Solver invariants where appropriate

### Fuzzing

Add fuzz targets for parsers, shape operations, collision inputs, world mutation sequences, particle operations, and any unsafe boundary.

### Miri and sanitizers

Run Miri where practical.

Run C++ sanitizers against the reference harness where practical.

Use Rust sanitizer builds in CI where supported.

## Examples and Testbed

Port the upstream LiquidFun testbed and examples.

The initial renderer may use a practical Rust graphics stack, but keep simulation logic independent from rendering.

Evaluate options such as:

- `wgpu`
- `winit`
- `glow`
- `macroquad`
- `bevy`, only if it does not force the core library into Bevy-specific architecture

Prefer a lightweight standalone testbed unless there is a compelling reason otherwise.

The testbed should support:

- Selecting upstream-equivalent demos
- Pausing
- Single stepping
- Restarting
- Changing timestep settings
- Viewing contacts
- Viewing particle contacts
- Viewing broad-phase data
- Viewing performance statistics
- Capturing deterministic scenario state
- Running headlessly
- Comparing Rust and C++ reference output

Rendering must remain optional. The core physics crates should support headless environments and WASM where feasible.

## Performance

Correctness and parity come before optimization, but performance is a core project requirement.

Create benchmarks for:

- World stepping
- Broad phase
- Narrow phase
- Contact solving
- Continuous collision detection
- Particle creation and destruction
- Particle contact generation
- Particle sorting
- Pressure solving
- Large particle systems
- Mixed rigid-body and particle worlds
- Queries and ray casts

Compare against upstream C++ using equivalent compiler optimization levels and workloads.

Document methodology carefully.

Avoid optimizing solely for synthetic benchmarks if it harms API clarity or correctness.

Use profiling before making structural performance changes.

Investigate:

- Data-oriented layouts
- Structure of arrays
- Cache behavior
- Allocation patterns
- Stable ordering
- Parallelism
- SIMD
- Arena allocation
- Generational indices
- Small-vector optimizations

Parallel execution must not be introduced if it breaks expected determinism or behavioral parity without a clearly documented opt-in mode.

## Cross-Platform Support

Target, at minimum:

- Linux x86_64
- Linux ARM64
- macOS ARM64
- macOS x86_64 where CI availability permits
- Windows x86_64

Investigate support for:

- WebAssembly
- iOS
- Android
- Embedded or `no_std` subsets

Do not promise `no_std` for the complete engine unless technically realistic. It is acceptable to identify a smaller math or collision crate that supports `no_std`.

Bazel and CI should make cross-platform validation straightforward where practical.

## Toolchain and Quality Gates

Pin an appropriate Rust toolchain in `rust-toolchain.toml`.

Use:

- `cargo fmt`
- `cargo clippy`
- `cargo test`
- `cargo doc`
- `cargo deny`
- `cargo audit`, where appropriate
- `cargo nextest`, if it improves test execution
- `cargo llvm-cov`, or an equivalent coverage workflow
- Criterion or a suitable benchmark framework
- Miri
- Fuzzing through `cargo-fuzz` or an equivalent tool

Treat warnings as errors in CI where practical.

Public APIs require clear, succinct rustdoc comments.

Document non-obvious algorithms and upstream equivalence.

Prefer intention-revealing names, early returns, and minimal nesting.

Never use `unwrap()` in production code.

Use `?` for error propagation.

Use `expect()` only for invariants that are genuinely impossible to violate, with a useful message.

Prefix optional variables with `maybe_` where doing so improves clarity.

Use `std::f32::consts::TAU` or `std::f64::consts::TAU` for full rotations. Prefer tau-based angle expressions over pi-based expressions.

## CI

Create CI workflows that cover:

- Formatting
- Clippy
- Unit tests
- Integration tests
- Documentation
- License checks
- Dependency policy
- Rust stable
- The pinned minimum supported Rust version, once defined
- Linux
- macOS
- Windows
- Release builds
- Differential tests where upstream C++ can be built
- Bazel validation if Bazel is adopted
- Miri on a useful subset
- Coverage
- Scheduled extended tests
- Benchmarks or benchmark smoke tests
- Submodule integrity

Keep ordinary pull-request CI reasonably fast. Put expensive randomized, differential, sanitizer, and benchmark suites into scheduled or manually triggered workflows where appropriate.

## Documentation

Create and maintain:

```text
README.md
CONTRIBUTING.md
ARCHITECTURE.md
COMPATIBILITY.md
UPSTREAM.md
TESTING.md
BENCHMARKING.md
SAFETY.md
RELEASING.md
ROADMAP.md
```

The README should explain:

- What the project is
- Current maturity
- What is implemented
- What is not implemented
- How to build it
- How to run examples
- How to run tests
- How to initialize the upstream submodule
- Whether C++ or Bazel is required for a given workflow
- How to contribute
- License information

Be precise about project status. Do not market incomplete compatibility as production-ready.

## Licensing

Perform an explicit license review before copying or translating upstream implementation details.

Document:

- The LiquidFun license
- The Box2D license
- Requirements for derived work
- Required copyright notices
- Any third-party Rust dependencies
- Any copied or mechanically translated code
- Any external test data

Preserve all required notices.

Do not select a project license until compatibility with upstream licensing has been verified.

## Development Strategy

Do not attempt to translate the entire repository mechanically in one step.

Begin with research and decomposition.

A likely sequence is:

1. Establish the repository, toolchains, submodule, documentation, and CI.
1. Inventory upstream features and tests.
1. Create the compatibility matrix.
1. Design the Rust ownership and handle model.
1. Port foundational math.
1. Port shapes and collision primitives.
1. Port broad phase and dynamic tree.
1. Port rigid-body dynamics.
1. Port contacts and solvers.
1. Port joints.
1. Establish rigid-body differential testing.
1. Port particle data structures.
1. Port particle contact generation.
1. Port particle solvers and behaviors incrementally.
1. Port particle groups and lifecycle behavior.
1. Establish comprehensive particle differential testing.
1. Port examples and testbed.
1. Optimize based on benchmarks.
1. Harden cross-platform support.
1. Complete parity audit and release preparation.

This sequence is only a starting hypothesis. Refine it after investigating upstream dependencies.

## Milestones

Define concrete milestones with measurable acceptance criteria.

Suggested milestones:

### Milestone 0: Repository foundation

- Cargo workspace builds
- Upstream submodule is pinned
- License review is documented
- `justfile` exists
- CI is operational
- Upstream C++ can be built through a documented command
- Bazel decision is documented

### Milestone 1: Upstream inventory and architecture

- Complete subsystem inventory
- Initial compatibility matrix
- Public object-model proposal
- Handle and lifetime strategy
- Differential-test architecture
- Numerical-tolerance policy

### Milestone 2: Math and collision foundation

- Core vectors, transforms, rotations, sweeps, AABBs, and shapes
- Collision algorithms
- Broad phase
- Unit and differential tests

### Milestone 3: Rigid-body world

- Bodies
- Fixtures
- Contacts
- Solver
- Sleeping
- Continuous collision detection
- Queries
- Ray casts

### Milestone 4: Joints and callbacks

- All upstream joint types
- Contact filters
- Contact listeners
- Destruction listeners
- Debug draw abstractions

### Milestone 5: Particle core

- Particle storage
- Creation and destruction
- Spatial proxies
- Particle contacts
- Body contacts
- Queries
- Lifetimes

### Milestone 6: Particle behaviors

- Every upstream particle flag and solver behavior
- Particle groups
- Pair and triad logic
- Differential validation for each behavior

### Milestone 7: Testbed and examples

- Upstream-equivalent scenarios
- Headless scenario runner
- Interactive visual testbed
- Reference snapshots

### Milestone 8: Performance and hardening

- Benchmarks
- Profiling
- Cross-platform CI
- Fuzzing
- Miri
- Safety audit
- API review

### Milestone 9: Parity release

- Compatibility matrix complete
- Upstream tests ported or accounted for
- Differential suite passes within documented tolerances
- Documentation complete
- SemVer and release policy defined
- Crates ready for publication

## Definition of Full Feature Parity

“Full feature parity”uires all of the following:

- Every public upstream feature is implemented or explicitly shown to be irrelevant to the Rust library.
- Every upstream particle behavior is implemented.
- Every supported upstream shape and joint is implemented.
- Equivalent world operations exist.
- Equivalent callbacks and queries exist.
- Upstream examples have Rust equivalents.
- Upstream tests are ported, replaced, or documented.
- Differential tests cover representative and edge-case behavior.
- Known differences are documented.
- Performance is measured against upstream.
- The Rust implementation does not require upstream C++ at runtime.
- The compatibility matrix contains no unexplained gaps.

## Initial GSD Task

Use the GSD New Project workflow to turn this vision into an executable project plan.

The first phase should focus on research, architecture, risks, and milestone planning—not broad implementation.

During project initialization:

1. Inspect the current state and history of Google LiquidFun.
1. Identify and pin the canonical reference revision.
1. Analyze repository layout, build systems, tests, examples, and dependencies.
1. Determine the exact Box2D ancestry and LiquidFun-specific changes.
1. Produce a full subsystem and API inventory.
1. Perform a license analysis.
1. Evaluate Cargo-on, Bazel-only, and hybrid build strategies.
1. Propose the Rust crate and module architecture.
1. Propose the ownership, handle, callback, and user-data models.
1. Design the C++ reference harness and differential-testing strategy.
1. Identify the highest technical risks.
1. Break the work into milestones, phases, and small verifiable tasks.
1. Define acceptance criteria for each phase.
1. Create the initial repository scaffolding only after the foundational decisions are documented.

## Expected Early Deliverables

Before substantial porting begins, produce:

- `PROJECT.md` containing the refined project vision
- `REQUIREMENTS.md`
- `ARCHITECTURE.md`
- `UPSTREAM.md`
- `COMPATIBILITY.md` with the initial traceability matrix
- `TESTING.md`
- `ROADMAP.md`
- A risk register
- A dependency and licensing report
- A decision record for Bazel
- A decision record for the Rust object model
- A decision record for differential testing
- Initial Cargo workspace scaffolding
- The pinned upstream Git submodule
- A root `justfile`
- Minimal CI that builds both the Rust skeleton and upstream reference implementation

## Key Risks to Address Explicitly

The plan must directly address:

- Project size and multi-year scope
- Hidden behavioral dependencies in upstream C++
- C++ pointer and ownership semantics
- Stable object identity
- Callback reentrancy
- Mutation during callbacks
- Deterministic iteration order
- Floating-point divergence
- Particle-system complexity
- Performance regressions from safe abstractions
- Licensing implications of close translation
- Stale or abandoned upstream tooling
- Cross-platform C++ reference builds
- Bazel maintenance burden
- API compatibility versus idiomatic Rust design
- Preventing premature claims of parity

## Guiding Principle

Favor incremental, testable parity over a visually impressive but unverifiable rewrite.

Every subsystem should progress through:

```text
inventory
→ API design
→ minimal implementation
→ unit tests
→ upstream comparison
→ differential validation
→ optimization
→ documentation
→ compatibility sign-off
```

The upstream C++ implementation is the behavioral oracle during development, but the Rust implementation must become a complete, independent, maintainable engine.
