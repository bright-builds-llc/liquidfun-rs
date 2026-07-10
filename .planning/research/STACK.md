# Stack Research

**Domain:** Greenfield native Rust port of Google LiquidFun with a pinned C++ differential-testing and benchmark oracle
**Researched:** 2026-07-09
**Confidence:** HIGH for the Rust/Cargo and CMake-oracle baseline; MEDIUM for the initial MSRV, exact private-crate boundaries, and eventual renderer

## Executive Recommendation

Use a **Cargo-first, CMake-assisted hybrid**, not Bazel:

- Pin **Rust 1.97.0** for repository development in `rust-toolchain.toml`, use **Rust 2024 Edition** and Cargo **resolver 3**, and declare **Rust 1.92.0** as the initial MSRV of publishable crates.
- Start with one publishable crate, `liquidfun`. Keep math, collision, dynamics, and particles as deep internal modules until a demonstrated independent-compilation or public-API boundary justifies another published crate.
- Keep all C++ and cross-language work private. Build the pinned upstream source plus a thin reference executable with **CMake 4.3.3** and **Ninja 1.13.2**, driven by `cargo xtask` and exposed through thin `just` recipes.
- Make the first oracle boundary a long-lived **out-of-process JSON Lines protocol**, not in-process FFI. This isolates crashes and sanitizers, avoids C++ ABI and allocator coupling, and still amortizes process startup across many scenarios.
- Add an in-process C ABI only if profiling shows IPC is the differential-test bottleneck. If added, confine it to an unpublished `liquidfun-reference-sys` crate; never make it a feature or dependency of the published engine.
- Use native Rust tests first, then `cargo-nextest`, `proptest`, Criterion, `cargo-fuzz`, Miri, sanitizers, and `cargo-llvm-cov` in progressively more expensive CI lanes.
- Keep runtime dependencies intentionally small. The expected initial production dependencies are `bitflags` 2.13 and `thiserror` 2.0. Serialization, random generation, rendering, and orchestration libraries belong only in private developer crates.

This recommendation is consistent with the local Bright Builds architecture, verification, and Rust standards: a functional core with thin adapters, explicit invariants, cohesive modules, Cargo-native verification, and no unnecessary cross-language logic in the production path.

## Confirmed 2026 Baseline

These are observed facts, not project decisions:

| Fact | Evidence | Confidence |
| --- | --- | --- |
| Rust 1.97.0 is the current stable release on 2026-07-09. | The Rust release team announced 1.97.0 on 2026-07-09. | HIGH |
| Rust 2024 implies Cargo resolver 3; a virtual workspace must still set `resolver = "3"` explicitly. | Rust Edition Guide and Cargo workspace documentation. | HIGH |
| Cargo supports a package `rust-version`, and resolver 3 prefers dependencies compatible with it. | Cargo Book `rust-version` and resolver documentation. | HIGH |
| `google/liquidfun` is archived and read-only as of 2026-02-13. | Official GitHub repository status. | HIGH |
| The upstream tree contains legacy CMake files and no Bazel `BUILD`, `WORKSPACE`, or `MODULE.bazel` files. | Recursive official upstream tree inspection. | HIGH |
| Upstream `liquidfun/Box2D/CMakeLists.txt` declares `cmake_minimum_required(VERSION 2.8)`. | Official upstream source. | HIGH |
| CMake 4 rejects policy compatibility older than 3.5 unless the caller supplies a minimum policy version. | Current CMake documentation. | HIGH |
| Bazel 9 removed the legacy `WORKSPACE` dependency system; new adoption would have to use Bzlmod. | Bazel 9 release and current external-dependency documentation. | HIGH |
| LiquidFun 1.1.0 identifies its Box2D basis as revision 280 / Box2D 2.3.0 and describes the code as portable C++98. | Official LiquidFun 1.1.0 release notes. | HIGH |
| The upstream Box2D license text is the permissive zlib-style license and requires preserving the notice and marking altered source versions. | Official upstream `License.txt`. | HIGH |

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended | Confidence |
| --- | --- | --- | --- | --- |
| Rust | 1.97.0, pinned | Production implementation and normal repository development | Current stable release; supports Edition 2024, resolver 3, and Cargo-level warning denial. Pinning makes CI and developer output reproducible. | HIGH |
| Rust Edition | 2024 | Language edition for every Rust crate | Current stable edition; no reason for a greenfield project to begin on 2021. | HIGH |
| MSRV | 1.92.0 initially | Minimum compiler for publishable crates | A roughly six-month compatibility window at project start balances library adoption with maintenance cost. Every proposed initial library/test dependency supports this MSRV. | MEDIUM |
| Cargo workspace | resolver 3 | Rust dependency graph, builds, tests, examples, benchmarks, and publishing | Ordinary consumers and contributors remain Cargo-only. Resolver 3 is MSRV-aware and appropriate for Edition 2024. | HIGH |
| CMake | CI pin 4.3.3; local floor 3.25 | Private upstream C++ reference build | Upstream already uses CMake. A modern wrapper can set legacy policy compatibility without modifying the pinned submodule. | HIGH |
| Ninja | CI pin 1.13.2; local floor 1.11 | CMake generator for repeatable, fast C++ builds | Small, cross-platform, and intentionally used as a generated-build executor beneath CMake. | HIGH |
| Clang/LLVM | Canonical Linux oracle pin 22.1.8 | C++ oracle compiler, sanitizers, and benchmark compiler | A named compiler/version is necessary for reproducible golden data and comparable benchmarks. Keep platform-native compiler jobs as portability checks. | MEDIUM |
| GitHub Actions | Current major actions pinned by full commit SHA | CI orchestration | Native fit for a GitHub-hosted open-source repository and supports Linux, macOS, Windows, scheduled, and manual workflows. | HIGH |
| `just` | 1.55.1 | Discoverable human-facing command menu | Meets the project brief while remaining a thin layer over visible Cargo, xtask, CMake, and Ninja commands. | HIGH |

### Rust Toolchain and MSRV Policy

Check in the following stable toolchain shape:

```toml
[toolchain]
channel = "1.97.0"
profile = "minimal"
components = ["clippy", "rustfmt", "llvm-tools-preview"]
```

Apply these policies:

1. Put `edition = "2024"` and `rust-version = "1.92"` in `[workspace.package]`; every publishable crate inherits them.
1. Put `resolver = "3"` explicitly in the virtual `[workspace]`.
1. Treat 1.92.0 as a declared, tested contract, not an estimate. CI must build and test the publishable surface with that compiler.
1. Maintain an approximately six-month MSRV window. Raise MSRV only in a minor release, document it in the changelog, and never raise it merely because a developer tool requires a newer compiler.
1. Use the pinned 1.97.0 toolchain for normal CI. Add a `stable` canary job so the next Rust release is tested before deliberately updating the pin.
1. Pin nightly-only jobs by date, for example `nightly-2026-07-09`, and update the date in an intentional monthly toolchain PR. Do not use floating `nightly` in required CI.
1. Use `CARGO_BUILD_WARNINGS=deny` on Rust 1.97 CI jobs and keep `cargo clippy ... -- -D warnings`. Use `RUSTFLAGS=-Dwarnings` only where an older MSRV job needs equivalent behavior.

The MSRV choice is a recommendation rather than an ecosystem fact. Revisit it before the first crates.io release if target users demonstrate a materially older toolchain requirement.

### Cargo Workspace and Crate Strategy

Start with this intentionally shallow workspace:

```text
.
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── justfile
├── crates/
│   ├── liquidfun/                 # only initially publishable crate
│   ├── liquidfun-diff/            # private Rust scenario runner and comparator
│   └── liquidfun-testbed/         # private, deferred until testbed work begins
├── tools/
│   ├── xtask/                     # private orchestration CLI
│   └── reference/                 # CMake wrapper and C++ oracle executable
├── fuzz/                          # cargo-fuzz package, excluded from main workspace
├── benches/
├── reference-data/
└── third_party/
    └── liquidfun/                 # exact pinned upstream submodule
```

Workspace rules:

- Set `default-members = ["crates/liquidfun"]`. Plain `cargo build` and `cargo test` must not initialize the submodule, discover CMake, or compile C++.
- Mark every tooling/testbed package `publish = false`.
- Keep `liquidfun` free of path dependencies on developer tooling. `cargo package -p liquidfun` and the packaged-crate smoke test must succeed without `third_party/liquidfun` present.
- Commit the shared root `Cargo.lock` because the repository pins its Rust toolchain and needs reproducible CI/tooling resolution. Published library consumers still resolve through their own lockfiles.
- Centralize versions in `[workspace.dependencies]` and lints in `[workspace.lints]`.
- Keep math, handles, collision, broad phase, dynamics, joints, and particles as modules inside `liquidfun` at first. Split a published crate only when it has a stable independent API, a credible independent user, or a necessary build boundary.
- Do not create `liquidfun-core`, `liquidfun-math`, `liquidfun-collision`, and `liquidfun-dynamics` merely to mirror source directories. That would make internal refactors semver-sensitive and create dependency cycles around shared types.
- If an internal unsafe fast path emerges, keep the safe implementation as the behavioral baseline and isolate unsafe code in a narrow module with `SAFETY:` invariants and focused tests.

### C++ Reference Build and Orchestration

Use **Cargo + xtask + CMake/Ninja**, with each layer owning one concern:

| Layer | Responsibility | Must Not Do |
| --- | --- | --- |
| Cargo | Build, test, document, benchmark, and publish native Rust crates | Discover or compile C++ from `liquidfun` or its `build.rs` |
| `cargo xtask` | Validate tools/submodule, configure CMake presets, run oracle/differential workflows, normalize paths, and produce actionable diagnostics | Reimplement physics or hide large embedded shell/CMake scripts |
| `just` | Provide memorable aliases and print the underlying command | Contain substantive orchestration logic |
| CMake | Build the pinned C++ library, oracle executable, upstream tests, and C++ sanitizer/benchmark variants | Build the Rust production workspace |
| Ninja | Execute generated C++ build graphs | Become a handwritten build format |

Create a repository-owned `tools/reference/CMakeLists.txt` with a modern policy range. Before `add_subdirectory()` of the pinned upstream, set `CMAKE_POLICY_VERSION_MINIMUM=3.5` and force upstream examples/tests off for the default oracle library build. This is the supported CMake 4 mechanism for an unmodified old third-party project.

Use separate CMake presets:

- `oracle-debug`: assertions, no examples, no upstream tests.
- `oracle-release`: optimized reference and paired benchmark executable.
- `oracle-asan-ubsan`: Clang AddressSanitizer plus UndefinedBehaviorSanitizer.
- `upstream-tests`: legacy vendored GoogleTest and upstream test targets.
- `testbed-upstream`: upstream OpenGL/freeglut testbed only when explicitly requested.

Canonical differential data should initially be generated on Linux x86_64 with pinned Clang/LLVM 22.1.8, baseline CPU targeting, no `-march=native`, and no fast-math. Record compiler identity, target triple, CMake preset, upstream commit, wrapper commit, and build flags alongside every generated reference artifact. macOS Apple Clang and Windows MSVC/clang-cl jobs verify portability but should not silently overwrite canonical golden data.

Do not patch files in the submodule to modernize CMake. If a compatibility patch becomes unavoidable, keep it as a versioned patch in `tools/reference/patches/`, verify the preimage hash, and document whether it can change oracle behavior.

### Bazel Decision

**Do not adopt Bazel in the foundation stack.**

The pinned upstream supplies CMake, not Bazel. Adding Bazel would require maintaining a second description of both the old C++ source graph and the Rust/Cargo graph. Bazel 9 also requires Bzlmod, so examples based on legacy `WORKSPACE` are no longer a sound starting point. CMake plus Cargo already covers the required local and CI platforms with less duplicated configuration.

Reconsider Bazel only after all of these are true:

1. The Cargo/CMake implementation exists and is measured.
1. A concrete unmet need is demonstrated, such as remote execution at project scale, hermetic multi-language toolchains that CMake presets cannot provide, or unacceptable CI rebuild cost.
1. The team accepts duplicated Cargo/Bazel dependency graph maintenance.
1. Adoption uses Bazel 9.x Bzlmod and a pinned compatible `rules_rust` release; never start new work on `WORKSPACE`.

Current versions for a future evaluation are Bazel 9.1.1 and `rules_rust` 0.71.3, but they are **not** recommended dependencies now.

### Reference Protocol and FFI Boundary

#### Recommended first boundary: process isolation

Build `liquidfun-reference` as a long-lived C++ executable and communicate through stdin/stdout using a versioned JSON Lines protocol:

```text
liquidfun-diff (Rust)
  -> versioned scenario request
  -> long-lived child process stdin
liquidfun-reference (C++)
  -> versioned semantic snapshot / event stream
  -> child process stdout
liquidfun-diff (Rust)
  -> domain-aware tolerance comparison and human-readable report
```

Protocol requirements:

- Every envelope includes protocol version, scenario schema version, seed, upstream commit, compiler/build identity, timestep, and solver iteration counts.
- Compare semantic fields, never object memory, pointer values, padding, or serialization of C++ classes.
- Encode exact scenario `f32` values as bit patterns in machine-facing fields; diagnostics may additionally show decimal values. This prevents a JSON formatter/parser round trip from becoming an untracked numerical difference.
- One request yields deterministic ordered entities/events with explicit stable scenario IDs. Do not infer identity from C++ addresses.
- Use stdout only for protocol records and stderr for diagnostics. Any malformed line, unknown schema, timeout, crash, or nonzero exit is a hard harness error distinct from a physics mismatch.
- Keep the process alive for batches so startup is amortized. Restart on crash and persist the exact failing seed/request before minimization.
- Pin and vendor `nlohmann/json` 3.12.0, including its MIT license, only under `tools/reference/`; use `serde`/`serde_json` only in private Rust tooling.

This boundary is preferable to FFI at project start because C++ sanitizer aborts, undefined behavior, exceptions, and allocator failures cannot corrupt the Rust test process.

#### Optional later boundary: narrow C ABI

Add in-process FFI only after a benchmark shows the process protocol is a material bottleneck. Confine it to an unpublished, non-default `liquidfun-reference-sys` package and follow these rules:

- Export `extern "C"` functions from a hand-written wrapper; do not expose C++ classes, STL types, templates, references, exceptions, RTTI, or upstream headers to Rust.
- Keep world/state ownership in C++; Rust receives only opaque handles and copies of semantic records.
- Use fixed-width integer types, `float`, caller-owned buffers, explicit lengths/capacities, and status codes. Never allocate on one side and free on the other.
- Catch all C++ exceptions at the wrapper boundary and translate them to an explicit error status plus caller-provided diagnostic buffer.
- Never retain Rust-provided pointers beyond the call that received them.
- Define `#[repr(C)]` mirrors manually for this deliberately small API and assert size/alignment/offsets from both languages. Avoid `bindgen` and its libclang dependency unless the surface becomes too large to audit manually.
- Run the FFI backend under C++ ASan/UBSan and Rust Miri-compatible unit tests where applicable. Keep the process backend as the debugging and crash-isolation reference.
- Do not expose an `ffi` feature on `liquidfun`; FFI is repository tooling, not a consumer capability.

### Supporting Libraries

Versions below are the current stable releases verified on 2026-07-09. Cargo requirements should normally use compatible semver requirements while the committed `Cargo.lock` pins repository resolution. The deterministic RNG is the exception: pin it exactly with the scenario schema.

| Library | Version | Scope | Purpose | When to Use | Confidence |
| --- | --- | --- | --- | --- | --- |
| `bitflags` | 2.13.0 | Production | Particle, group, body, and other upstream bit masks | Use from the first relevant API; preserve upstream bit values and retain unknown bits where compatibility requires it. | HIGH |
| `thiserror` | 2.0.18 | Production | Typed library errors | Use for construction/configuration/query errors; do not put recoverable error allocation into hot stepping paths without evidence. | HIGH |
| `serde` | 1.0.228 | Private tooling | Scenario and report model serialization | Use in `liquidfun-diff` and xtask only. Do not add to the engine's default dependency graph. | HIGH |
| `serde_json` | 1.0.150 | Private tooling | JSON Lines protocol and machine-readable reports | Use with an explicit schema/version and exact float-bit fields. | HIGH |
| `nlohmann/json` | 3.12.0 | Private C++ tooling | C++ side of JSON Lines protocol | Vendor exact release and license under `tools/reference`; never fetch a moving branch during a build. | HIGH |
| `proptest` | 1.11.0 | Dev dependency | Property testing, generation, and shrinking | Use for geometry, collision, handle, mutation-sequence, and differential scenarios. Persist minimized failures as ordinary regression fixtures. | HIGH |
| `rand_chacha` | =0.10.0 | Dev dependency | Reproducible scenario RNG | Use a named ChaCha variant, pin exactly, and version the generator algorithm; a seed alone is not stable if the generator changes. | HIGH |
| `criterion` | 0.8.2 | Dev dependency | Rust microbenchmarks and regression statistics | Use for in-language subsystem benchmarks; use a paired harness for Rust-vs-C++ comparisons. | HIGH |
| `arbitrary` | 1.4.2 | Fuzz package | Structured libFuzzer input decoding | Use in `fuzz/`; keep parsers and mutation operations bounded. | HIGH |
| `anyhow` | 1.0.103 | Private binaries | Context-rich errors in xtask/diff/testbed executables | Use only in applications; public library errors remain typed. | HIGH |
| `xshell` | 0.2.7 | Private xtask | Cross-platform command execution | Use to keep orchestration readable without large embedded shell strings. | MEDIUM |
| `macroquad` | 0.4.15, deferred | Private testbed | Lightweight 2D drawing, input, and immediate-mode UI | Preferred first renderer prototype at the testbed milestone, not a foundation dependency. Re-evaluate after a small spike. | MEDIUM |

### Libraries to Defer Until Evidence Exists

| Candidate | Why It Is Deferred | Acceptance Trigger |
| --- | --- | --- |
| `slotmap`, `generational-arena`, or equivalent | Handle reuse, iteration order, storage movement, and invalidation semantics are core compatibility decisions. Selecting a crate before the object-model ADR could harden the wrong behavior. | The object-model ADR proves its key layout, iteration, invalidation, serialization, and performance match project needs. |
| `smallvec`, arena allocators, custom hashers | These are performance/layout choices, not foundation requirements. | Profiling shows a hot allocation or locality problem and differential tests protect ordering behavior. |
| `glam`, `nalgebra`, or another math API | LiquidFun's exact `f32` operations, layout, naming, and ordering need direct control. A general math crate can change operation grouping and public API shape. | A written compatibility experiment demonstrates no semantic/API cost. |
| Rayon or other parallelism | Default parallelism can change solver/contact/particle order and determinism. | A documented opt-in mode with deterministic baseline, measured speedup, and parity policy. |
| `wgpu` 30 + `winit` 0.30 + `egui` 0.35 | Powerful but much heavier than needed for the first 2D debug testbed. | Macroquad fails a required rendering, GPU debugging, UI, or platform capability in a prototype. |
| `cxx`, `bindgen`, `autocxx` | The recommended process boundary needs no FFI; a future C ABI is small enough to audit manually. | The measured FFI surface becomes large enough that generation reduces risk rather than adding toolchain burden. |
| `cargo-vet` | Valuable but operationally heavier than the foundation's deny/lock/update policy. | Before a production-stability claim or when the dependency graph becomes large enough to justify formal supplier audits. |

## Development Tools

| Tool | Verified Version | Purpose | Required Lane / Notes |
| --- | --- | --- | --- |
| `cargo fmt` | Rust 1.97.0 component | Formatting | Required on every PR: `cargo fmt --all --check`. |
| `cargo clippy` | Rust 1.97.0 component | Rust linting | Required: `cargo clippy --workspace --all-targets --all-features -- -D warnings` in a C++-prepared full-workspace lane; also run `-p liquidfun` in Cargo-only CI. |
| `cargo nextest` | 0.9.140 | Parallel test runner, timeouts, slow-test policy, JUnit | Required for unit/integration suites once test count warrants it. It does not run doctests; keep `cargo test --doc`. |
| `cargo llvm-cov` | 0.8.7 | Rust source coverage | Scheduled or required after a baseline is defined. Install `llvm-tools-preview`; report Rust and C++ coverage separately at first. |
| `cargo deny` | 0.20.2 | License, advisory, banned crate, duplicate, and source checks | Required on PRs with `--locked`; include dev dependencies in license review. |
| `cargo audit` | 0.22.2 | Independent RustSec lockfile audit | Scheduled and dependency-change lane; do not duplicate it in every fast job if `cargo deny` already checks advisories. |
| `cargo fuzz` | 0.13.2 | libFuzzer integration | Nightly-only, time-bounded scheduled jobs plus local reproduction. |
| Miri | Pinned nightly component | Undefined-behavior and aliasing checks | Scheduled subset; test pure core and any unsafe modules, not the C++ process. |
| Rust sanitizers | Pinned nightly | ASan/LSan and selected other sanitizer runs | Scheduled Linux lane; sanitizer support is unstable and target-specific. |
| Clang sanitizers | Clang 22.1.8 canonical | C++ ASan/UBSan | Required scheduled oracle lane and before accepting any wrapper/FFI change. |
| `cargo hack` | 0.6.45 | Feature powerset and MSRV verification | Required as features appear; use `--rust-version` for publishable packages. |
| `cargo semver-checks` | 0.48.0 | Public API compatibility | Required before release after the first published baseline, not during pre-API research. |
| Criterion | 0.8.2 | Rust microbenchmarks | Benchmark smoke tests compile on PR; performance runs use controlled hardware and do not gate noisy shared runners. |
| CMake | 4.3.3 CI pin | C++ configure/build/test presets | Developer-only. Always use out-of-tree build directories under `target/reference/`. |
| Ninja | 1.13.2 CI pin | C++ build execution | Selected through CMake presets. |
| `just` | 1.55.1 | Discoverable command facade | Default recipe lists commands and exits quickly. |

Install command-line tools at exact versions in bootstrap/CI, preferably from verified prebuilt release artifacts when the project provides attestations. Pin GitHub Actions by full commit SHA even when comments record a friendly major version. Let automated dependency PRs update pins deliberately.

## Testing, Benchmarking, Fuzzing, and Coverage

### Test layers

| Layer | Primary Tool | Scope | CI Frequency |
| --- | --- | --- | --- |
| Unit | Built-in Rust test harness | Pure math, data structures, algorithms, invariants | Every PR, all platforms where practical |
| Integration/API | Built-in harness through nextest | Public world, shape, joint, particle, callback, and invalidation behavior | Every PR |
| Doctest | `cargo test --doc` | Public examples and rustdoc | Every PR; separate because nextest does not support doctests |
| Upstream compatibility | CMake/CTest or named upstream executables | Unmodified upstream tests | Smoke on PR, full scheduled/manual |
| Differential | `liquidfun-diff` plus long-lived C++ oracle | Semantic Rust/C++ state and events | Small deterministic corpus on PR; randomized/full corpus scheduled |
| Property | `proptest` 1.11 | Geometry, broad phase, handles, queries, mutation sequences | Focused properties on PR; high case counts scheduled |
| Regression fixtures | Plain checked-in scenarios | Every previously fixed bug or minimized differential mismatch | Every PR |
| Fuzz | `cargo-fuzz` 0.13 + `arbitrary` 1.4 | Parsers, shapes, collision, unsafe boundaries, world mutation, particles | Scheduled and before releases |
| UB/aliasing | Miri pinned nightly | Safe/unsafe Rust subset | Scheduled |
| Native sanitizers | Rust nightly and Clang 22 | Rust low-level code, C++ wrapper/oracle | Scheduled/manual |

Keep tolerance logic as a domain module, not a dependency on a generic `approx` assertion. Different observables need different absolute/relative/ULP policies, ordering rules, and diagnostics. A contact-set comparison is not the same operation as a vector epsilon assertion.

### Benchmarks

Use two complementary systems:

1. **Criterion 0.8.2** for Rust-only microbenchmarks and local regression statistics.
1. A repository-owned **paired benchmark executable** for Rust-vs-C++ workloads. It must run the same versioned scenario, warmup, step count, timestep, solver iterations, CPU affinity policy, compiler identity, optimization level, and result validation before accepting timings.

Do not compare a Criterion Rust benchmark to a separately authored C++ benchmark and call it equivalent. Do not include child-process startup in physics timing. The C++ executable may time the simulation region internally or execute many repetitions per protocol request. Compile canonical comparison builds without fast-math or `-march=native`; optional native-tuned results must be labeled separately.

Store benchmark methodology and summary data, not opaque raw profiler caches. Gate PRs on benchmark compilation and scenario correctness, not noisy wall-clock thresholds on shared runners. Use dedicated or controlled hardware for trend decisions.

### Coverage

- Use `cargo llvm-cov` 0.8.7 with the Rust toolchain's `llvm-tools-preview` for Rust coverage.
- Generate C++ coverage separately with the matching Clang/LLVM toolchain at first. This avoids silently mixing incompatible LLVM profile formats.
- `cargo-llvm-cov --include-ffi` is a possible later unified path, but it requires compatible Clang, `llvm-cov`, and `llvm-profdata`; verify the Rust 1.97/LLVM combination before adopting it.
- Coverage is evidence of exercised code, not parity. Report unit/integration coverage separately from subsystem differential coverage in the compatibility matrix.
- Establish thresholds only after representative code exists; do not create an arbitrary percentage gate in the skeleton phase.

## CI Strategy

### Pull-request lanes

1. **Cargo-only Linux quality:** format, Clippy, build all targets/features for `liquidfun`, nextest, doctests, rustdoc with warnings denied, package, and packaged-crate smoke test.
1. **Cargo-only platform matrix:** default-feature build/test on Linux x86_64, macOS ARM64, and Windows x86_64; add Linux ARM64 as available. This matrix must not install CMake or initialize the upstream submodule.
1. **MSRV:** Rust 1.92.0 on Linux, resolver-locked, checking/testing every publishable feature combination through `cargo hack` as practical.
1. **Dependency policy:** `cargo deny check --locked`, license/provenance diff, and `cargo audit` when dependency files change.
1. **Oracle smoke:** initialize the pinned submodule, configure CMake/Ninja, build the C++ oracle, verify protocol handshake, run the small deterministic differential corpus on Linux.
1. **Benchmark smoke:** compile benchmark targets and validate scenario equivalence without enforcing timing thresholds.

### Scheduled or manual lanes

- Full randomized differential suite and seed replay corpus.
- Clang ASan/UBSan reference build and tests.
- Pinned-nightly Miri and Rust sanitizer subset.
- Time-bounded fuzz targets with persisted artifacts.
- Rust and C++ coverage reports.
- Full upstream unit tests and examples where headless execution is possible.
- Wider macOS/Windows reference builds and Linux ARM64 coverage.
- Performance trend run on controlled hardware.
- Latest-stable Rust and latest-compatible dependency canaries.

CI policy:

- Pin external actions by commit SHA. Current friendly majors observed during research are `actions/checkout@v7`, `actions/cache@v6`, `actions/upload-artifact@v7`, and `actions/download-artifact@v8`; these are not substitutes for SHA pins.
- Use `fetch-depth: 0` only where versioning/history needs it, and enable submodules only in oracle jobs.
- Cache Cargo registry/git data and target directories with toolchain/lockfile-aware keys. Keep Rust and C++ build caches separate.
- Persist a failing differential request, minimized fixture, stderr, build provenance, and comparator report as CI artifacts.
- Configure nextest timeouts; do not automatically retry deterministic physics tests. A retry may be used only in a separately labeled flaky-infrastructure policy.
- Use concurrency cancellation for superseded PR runs, but never cancel release or scheduled evidence generation halfway through artifact publication.

## Rendering and Testbed Recommendation

Do not choose a rendering stack in the foundation phase. First define a renderer-independent debug-draw command stream and a headless scenario controller.

At the testbed milestone, prototype **Macroquad 0.4.15** first because it provides cross-platform 2D rendering, input, an immediate-mode UI, and WASM support with a much smaller integration surface than a custom `wgpu` stack. Keep it in the unpublished `liquidfun-testbed` binary and convert engine state into debug-draw primitives through a narrow adapter.

Required separation:

- `liquidfun`: simulation and renderer-independent debug-draw traits/data only.
- `liquidfun-diff`: headless scenario selection, capture, comparison, and reproducibility.
- `liquidfun-testbed`: window, input, UI, frame pacing, and rendering.

Re-evaluate `wgpu` 30.0.0 + `winit` 0.30.13 + `egui` 0.35.0 only if the Macroquad spike cannot meet required GPU debugging, render-target, UI, platform, or capture needs. Do not use Bevy for the canonical testbed: its ECS/schedule and dependency graph would impose an application architecture unrelated to the physics library.

## Installation

The eventual bootstrap should be transparent and idempotent. The commands below describe the intended shape; they are not yet implemented:

```bash
# Cargo-only development and ordinary consumers
rustup toolchain install 1.97.0 --profile minimal \
  --component clippy,rustfmt,llvm-tools-preview
cargo build -p liquidfun
cargo test -p liquidfun

# Repository developer tools, installed at reviewed pins
cargo install --locked cargo-nextest --version 0.9.140
cargo install --locked cargo-llvm-cov --version 0.8.7
cargo install --locked cargo-deny --version 0.20.2
cargo install --locked cargo-audit --version 0.22.2
cargo install --locked cargo-hack --version 0.6.45
cargo install --locked cargo-semver-checks --version 0.48.0
cargo install --locked just --version 1.55.1

# Optional C++ oracle workflow
git submodule update --init --recursive third_party/liquidfun
cargo xtask reference configure --preset oracle-debug
cargo xtask reference build --preset oracle-debug
cargo xtask differential --corpus smoke
```

Prefer signed/attested prebuilt CLI releases over repeated source compilation when available, but keep version pins identical locally and in CI. `just bootstrap` may check and print missing tools; it must not silently install system packages or mutate global toolchains without explicit flags.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
| --- | --- | --- |
| Cargo + CMake/Ninja hybrid | Bazel 9 + `rules_rust` | Only after measured remote-execution/hermeticity needs justify a second build graph and Bzlmod maintenance. |
| Long-lived process protocol | In-process C ABI | Only when differential throughput profiling proves IPC is a bottleneck. Retain process mode for sanitizers and diagnosis. |
| Hand-written small C ABI if needed | `cxx` | If the audited interface grows enough that generated bridge safety outweighs the added build/runtime dependency. |
| One publishable `liquidfun` crate | Many subsystem crates | When a subsystem has a stable independent API/user or a necessary target/build boundary. |
| Custom compatibility math types | `glam` or `nalgebra` | Only after an operation-order/layout/API experiment proves compatibility and gives a clear maintenance win. |
| Custom generational storage pending ADR | `slotmap` / `generational-arena` | If the object-model study proves iteration, invalidation, key, serialization, and performance semantics fit. |
| Criterion + paired runner | Iai-Callgrind only | Add instruction-count benchmarks on Linux if wall-clock noise blocks diagnosis; it does not replace cross-platform wall-clock evidence. |
| Macroquad prototype | `wgpu` + `winit` + `egui` | When testbed requirements exceed Macroquad's rendering/UI/debug capabilities. |
| GitHub Actions | Buildkite/self-hosted CI | When benchmark stability, specialized ARM/macOS hardware, or queue economics justify maintained runners. |

## What NOT to Use

| Avoid | Why | Use Instead |
| --- | --- | --- |
| C++ bindings as the production engine | Violates the native Rust goal and burdens every consumer with foreign tooling/runtime behavior. | Native `liquidfun`; private oracle executable only. |
| A CMake or `cc` build script in the published crate | `build.rs` runs for consumers and would leak C++/submodule requirements into Cargo-only use. | xtask/CMake under private developer tooling. |
| Bazel as the initial repository entrypoint | Upstream has no Bazel graph; it duplicates Cargo/CMake and Bazel 9 requires new Bzlmod setup. | Cargo plus CMake/Ninja, wrapped by xtask/just. |
| CMake 4 directly on the legacy upstream root without policy handling | Upstream requests CMake 2.8 policy compatibility, which CMake 4 rejects. | Modern wrapper with `CMAKE_POLICY_VERSION_MINIMUM=3.5` before `add_subdirectory`. |
| Moving upstream branch/tag references | Oracle behavior and generated data become irreproducible. | Exact submodule commit plus recorded build provenance. |
| `bindgen` over the full LiquidFun headers | Exposes unstable C++ layout, needs libclang, and creates a large unsafe surface. | Process protocol, or a tiny hand-written C ABI later. |
| C++ STL values/exceptions across FFI | Compiler/stdlib ABI, ownership, and unwinding are unsafe across the boundary. | Fixed-width C records, opaque handles, status codes, copied buffers. |
| Raw memory snapshots as differential evidence | Includes pointers/padding and misses semantic identity/tolerance requirements. | Versioned semantic scenario/state/event schema. |
| `HashMap`/`HashSet` iteration in solver-visible order | Randomized/implementation-dependent order can change simulation results. | Order-preserving indices/vectors or explicit sorting defined by the compatibility policy. |
| `-ffast-math`, Rust fast-float flags, or `-march=native` in canonical parity builds | Changes IEEE behavior and portability, obscuring whether a mismatch is an implementation bug. | Baseline target, ordinary IEEE semantics, recorded flags. |
| Default parallel stepping | Scheduling changes contact/constraint/particle ordering and determinism. | Single-threaded baseline; explicit experimental mode only after parity. |
| A general math crate by default | Operation grouping/layout/API may not match the historical `f32` oracle. | Purpose-built compatibility math module. |
| Bevy as the canonical testbed | Couples the project to an ECS, schedule, renderer, and large dependency graph unrelated to the library. | Headless controller plus a lightweight private renderer adapter. |
| Generic epsilon-only assertions | Different fields need absolute, relative, ULP, set/order, and event-specific semantics. | Domain-specific tolerance/comparison policies. |
| Floating nightly in CI | Miri/fuzz/sanitizer behavior can change without a repository diff. | Date-pinned nightly updated intentionally. |
| Unpinned Git dependencies or build-time network fetches | Break reproducibility and make license/provenance review unstable. | crates.io releases or exact vendored archives/commits with checksums. |

## Stack Patterns by Variant

**If a user only consumes the Rust library:**

- Use `liquidfun` through Cargo.
- Require no C++, CMake, Ninja, submodule, renderer, or protocol dependency.
- Keep default features small and headless.

**If a contributor works only on native Rust internals:**

- Use the pinned stable toolchain and default workspace member.
- Run unit/integration/property tests without initializing upstream.
- Replay checked-in minimized fixtures without C++.

**If a contributor investigates parity:**

- Initialize the exact upstream submodule.
- Use xtask to configure the pinned CMake/Ninja preset and start the long-lived oracle.
- Save every mismatch with protocol/build provenance and promote confirmed fixes to ordinary regression fixtures.

**If a maintainer profiles performance:**

- Validate scenario outputs first.
- Use Criterion for Rust internals and the paired runner for Rust/C++.
- Separate canonical baseline builds from explicitly labeled native-tuned experiments.

**If the C++ oracle crashes or triggers sanitizer output:**

- Treat it as a harness/oracle result, not a Rust mismatch.
- Preserve the request and stderr, then reproduce with the process backend and sanitizer preset.
- Do not switch to in-process FFI to bypass isolation.

**If Bazel is reconsidered later:**

- Write an ADR with measured CMake/Cargo pain and a prototype.
- Use Bazel 9 Bzlmod; do not introduce legacy `WORKSPACE` configuration.
- Preserve Cargo as the normal consumer and publication path.

## Version Compatibility

| Component | Compatible With | Notes |
| --- | --- | --- |
| Rust 1.97.0 | Edition 2024, resolver 3 | Current pinned development toolchain. Cargo warning denial is available in 1.97. |
| Rust 1.92.0 MSRV | Edition 2024, resolver 3 | Above the Edition 2024 floor. Verify all publishable targets/features in CI. |
| `bitflags` 2.13.0 | Rust 1.56+ | Safe for proposed MSRV. |
| `thiserror` 2.0.18 | Rust 1.68+ | Safe for proposed MSRV. |
| `serde` 1.0.228 | Rust 1.56+ | Private tooling only. |
| `serde_json` 1.0.150 | Rust 1.71+ | Private tooling only. |
| `proptest` 1.11.0 | Rust 1.85+ | Safe for proposed MSRV. |
| Criterion 0.8.2 | Rust 1.86+ | Safe for proposed MSRV. |
| `rand_chacha` 0.10.0 | Rust 1.85+ | Exact-pin for versioned scenario generation. |
| `cargo-nextest` 0.9.140 | Rust 1.91+ to install | Install/run with pinned development toolchain, not MSRV; doctests remain a separate Cargo step. |
| CMake 4.3.3 | Upstream CMake policy 2.8 request | Requires caller-supplied `CMAKE_POLICY_VERSION_MINIMUM=3.5`; verify all presets on each platform. |
| Ninja 1.13.2 | CMake 4.3.3 Ninja generator | Pin in CI; permit documented local floor 1.11. |
| Clang 22.1.8 | Canonical C++ oracle | Record exact patch and flags. Platform jobs may use Apple Clang/MSVC but cannot overwrite canonical golden data. |
| `cargo-llvm-cov` 0.8.7 | Rust `llvm-tools-preview` | Rust coverage is supported; unified C++ FFI coverage requires a separately verified compatible Clang/LLVM set. |
| Bazel 9.1.1 | Bzlmod; `rules_rust` 0.71.3 candidate | Evaluation data only; not in recommended foundation stack. |

## Dependency and License Policy

Apply this policy before adding code dependencies:

1. **Separate graphs:** classify each dependency as production, optional production, dev-only Rust, command-line tool, private C++ oracle, or CI action. A permissive developer tool does not justify leaking it into `liquidfun`.
1. **Minimal production graph:** every runtime dependency needs a documented capability, MSRV, maintenance, safety, determinism, performance, and license review. Start with only `bitflags` and `thiserror` unless implementation evidence requires more.
1. **Provisional license allowlist:** permit commonly compatible permissive licenses such as MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, and Unicode-3.0 after verifying the final project license decision. Treat MPL/CDDL/EPL and all LGPL/GPL/AGPL dependencies as manual legal-review items rather than blanket allows.
1. **Preserve upstream provenance:** retain LiquidFun/Box2D notices, mark altered/translated source as required by the upstream zlib-style terms, and record provenance for copied algorithms, tests, and reference data. This research does not select the final project license.
1. **Deny unknown sources:** configure `cargo-deny` to reject unknown registries, unpinned git sources, yanked/vulnerable crates, license ambiguity, and unexplained duplicates. Include dev dependencies in license checks because tools/tests may still be redistributed in the repository or CI artifacts.
1. **Exact private C++ pinning:** vendor or submodule the exact `nlohmann/json` release with license and checksum; no `FetchContent` from a branch and no network fetch during ordinary builds.
1. **Lock and update deliberately:** commit `Cargo.lock`, run locked CI, and use weekly automated Cargo/Actions update PRs. Tool pins and action SHAs update in reviewed commits with changelog/release-note inspection.
1. **No dependency-driven MSRV accidents:** resolver 3 helps, but CI at 1.92.0 is authoritative. Developer tools may require newer Rust because they run under 1.97.0 and are not linked into the library.
1. **Audit unsafe/transitive code:** before release, review unsafe code in direct dependencies and high-risk transitive dependencies. Consider `cargo-vet` when the graph and maturity justify the operational cost.
1. **Package what consumers receive:** CI must inspect `cargo package --list`, build/test the `.crate`, and verify no C++ source, reference data, testbed asset, or developer-only license is accidentally shipped.

## Open Research and Decision Triggers

| Question | Current Position | Resolve When | Confidence |
| --- | --- | --- | --- |
| Exact canonical upstream commit | Must be an immutable submodule pin; stack works with either final selected commit or release tag commit. | Upstream provenance research/ADR. | MEDIUM |
| Does the legacy upstream unit-test graph build cleanly under CMake 4.3.3 on all targets? | Expected to require policy handling; do not assume more. | Foundation CMake spike on Linux/macOS/Windows. | MEDIUM |
| Is the process protocol fast enough? | Likely yes with a long-lived process and batched scenarios. | After representative differential corpus exists and is profiled. | MEDIUM |
| What handle/arena implementation should be used? | No dependency choice before object-model and ordering ADR. | Before foundational body/fixture/world storage. | LOW |
| Is Macroquad sufficient for the final testbed? | Best lightweight first prototype, deliberately deferred. | Testbed milestone spike. | MEDIUM |
| Can Rust and C++ coverage be safely merged? | Keep reports separate until toolchain compatibility is proven. | Coverage implementation with Rust 1.97 LLVM details verified. | MEDIUM |
| Should MSRV remain six months behind stable? | Start at 1.92; adjust from user/tool evidence before first release. | Release policy ADR. | MEDIUM |
| Should Bazel be adopted? | No, absent measured need. | Only after Cargo/CMake CI scale data exists. | HIGH |

## Sources

Primary sources were preferred for every time-sensitive fact:

- [Rust 1.97.0 release announcement](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/) — current stable toolchain and Cargo warning-denial behavior.
- [Rust Edition Guide: Cargo resolver 3](https://doc.rust-lang.org/stable/edition-guide/rust-2024/cargo-resolver.html) — Edition 2024 resolver behavior.
- [Cargo Book: Rust version](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html) — `rust-version` semantics and MSRV policy examples.
- [Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) — shared lockfile, explicit virtual-workspace resolver, and `default-members`.
- [rustup Book: Overrides and toolchain files](https://rust-lang.github.io/rustup/overrides.html) — exact stable/nightly pins, components, targets, and checked-in toolchain files.
- [Official Google LiquidFun repository](https://github.com/google/liquidfun) — archive status, repository layout, and release context.
- [Official LiquidFun Box2D CMakeLists](https://github.com/google/liquidfun/blob/master/liquidfun/Box2D/CMakeLists.txt) — legacy CMake 2.8 policy request and existing build options.
- [Official LiquidFun 1.1.0 release](https://github.com/google/liquidfun/releases/tag/v1.1.0) — C++98 statement and Box2D 2.3.0 / revision 280 ancestry claim.
- [Official upstream Box2D license](https://github.com/google/liquidfun/blob/master/liquidfun/Box2D/License.txt) — notice and altered-source obligations.
- [CMake `cmake_minimum_required` documentation](https://cmake.org/cmake/help/latest/command/cmake_minimum_required.html) — CMake 4 removal of policy compatibility older than 3.5.
- [CMake `CMAKE_POLICY_VERSION_MINIMUM` documentation](https://cmake.org/cmake/help/latest/variable/CMAKE_POLICY_VERSION_MINIMUM.html) — supported external compatibility mechanism for old third-party projects.
- [CMake 4.3.3 release](https://github.com/Kitware/CMake/releases/tag/v4.3.3) — current verified CMake release.
- [Ninja project](https://ninja-build.org/) and [Ninja 1.13.2 release](https://github.com/ninja-build/ninja/releases/tag/v1.13.2) — role and current verified release.
- [LLVM 22.1.8 release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-22.1.8) — current canonical Clang/LLVM candidate.
- [Bazel 9 LTS announcement](https://blog.bazel.build/2026/01/20/bazel-9.html) and [Bzlmod overview](https://bazel.build/external/overview) — removal of `WORKSPACE` and current module model.
- [cargo-nextest documentation](https://www.nexte.st/) — current 0.9.140 release and separate doctest requirement.
- [cargo-llvm-cov repository](https://github.com/taiki-e/cargo-llvm-cov) — coverage workflow, `llvm-tools-preview`, and FFI/LLVM compatibility requirement.
- [Rust Fuzz Book setup](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html) — nightly and sanitizer requirements for cargo-fuzz.
- [Miri repository](https://github.com/rust-lang/miri/) — nightly component installation and CI use.
- [Rust Unstable Book: sanitizers](https://doc.rust-lang.org/beta/unstable-book/compiler-flags/sanitizer.html) — unstable flags and target support.
- [cargo-deny checks](https://embarkstudios.github.io/cargo-deny/checks/index.html) and [license configuration](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html) — advisories, bans, sources, SPDX allowlists, and clarifications.
- [Criterion 0.8.2 release](https://github.com/criterion-rs/criterion.rs/releases/tag/criterion-v0.8.2) — current benchmark library release.
- [Macroquad repository](https://github.com/not-fl3/macroquad) — supported platforms, 2D batching, UI, WASM, and project scope.
- [crates.io API and package pages](https://crates.io/) — current crate versions, declared MSRVs, and licenses recorded in the tables above on 2026-07-09.
- [GitHub Actions releases](https://github.com/actions) — current action majors; repository policy still requires full commit SHA pins.

______________________________________________________________________

*Stack research for: native Rust LiquidFun parity port with a pinned C++ oracle*
*Researched: 2026-07-09*
