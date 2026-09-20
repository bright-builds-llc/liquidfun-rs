# Stack Research

**Domain:** Native LiquidFun performance closing — Dam Break pair timing, scripted CPU profiling, audit notes, and profile-guided scalar hot-path work
**Researched:** 2026-09-20
**Confidence:** HIGH for the local macOS aarch64 tooling choice and the “what not to add” list; MEDIUM for heap-profiler crate choice (`dhat` is stable but explicitly low-maintenance)

This file covers **v1.2 Native Performance Closing additions only**. Do not re-derive the v1.0 Cargo/CMake foundation. Keep Rust 1.97.0, Edition 2024, one publishable `liquidfun` crate, private xtask/just, and the pinned C++ oracle at `7f20402173fd143a3988c921bc384459c6a858f2` via `oracle-release`.

## Executive Recommendation

Reuse the existing unprofiled Dam Break pair as the **numeric authority**, then add a thin local profiling shell around it. Do not invent a second benchmark matrix.

1. **Keep** `just playground-dam-break-bench` → `cargo xtask playground dam-break-bench` as the only 3× gate: native `liquidfun-wasm` `--release` `dam-break-bench` versus `oracle-release` `playground-dam-break-bench`, 60 warm-up + 600 timed `World::step` / `b2World::Step` calls, construction outside the timer.
1. **Add** a workspace `[profile.profiling]` that inherits `release` with `debug = true`, and script `samply` 0.13.1 against that binary with `--save-only` into gitignored `target/dam-break-perf/`.
1. **Reuse** the existing public `World::step_profiled` diagnostic parents (`particle_prepare`, `particle_solve`, …) as coarse attribution. Sampling profiles then name functions; diagnostic timers name phases.
1. **Optimize from those artifacts in scalar `--release`**, then re-run the unprofiled pair. Profiled wall times are never the 3× number.
1. **Do not** add a required performance CI job, revive the empty Phase 12 reviewed-report manifest, default SIMD/parallel, LLVM PGO as the first lever, or any C++/profiler dependency on publishable `liquidfun`.

A ~300× Dam Break Medium gap (1920 particles) is an algorithm/allocation/shape problem, not a missing-PGO or missing-SIMD problem. Typical PGO/BOLT wins are single-digit to low-double-digit percent; SIMD is opt-in after the scalar path is honest.

## Recommended Stack

### Core Technologies

Existing foundation — **keep, do not change for this milestone:**

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.97.0 pin | Native engine and bench binary | Already pinned; Dam Break sample was taken with `rustc 1.97.0`. |
| `liquidfun` | workspace member, `publish` intended | Shared particle/rigid hot paths | Fixes belong here; playground and WASM inherit them. |
| Private xtask + `just` | existing | Discoverable wrappers over visible commands | Matches repo policy: `just` stays thin; orchestration lives in xtask. |
| CMake `oracle-release` | existing preset | Scalar C++ pair binary | Already rejects `-ffast-math` / `-march=native`; this is the C++ side of the 3× gate. |
| `just playground-dam-break-bench` | existing | Unprofiled native-vs-C++ wall clock | Already locked recipe (1920 particles, `dt = 1/60`, 8/3/2 iterations). |

**New for v1.2:**

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Cargo `[profile.profiling]` | workspace `Cargo.toml`, inherits `release`, `debug = true` | Symbolicated stacks without changing the gate binary | samply’s documented requirement is **release + debug info**. Do not turn on `debug` for default `release` (CI, package, 3× gate). |
| `samply` | **0.13.1** (official `mstange/samply`, crates.io, Homebrew) | Scripted CPU sampling on macOS aarch64 | Native `aarch64-apple-darwin` binary; profiles locally-built unsigned binaries without sudo DTrace; `--save-only -o` writes Firefox Profiler JSON for gitignored evidence; `--unstable-presymbolicate` embeds names for later `samply load`. |
| `xcrun xctrace` | Ships with Xcode (host already used AppleClang 21.0.0.21000334) | Second-line Apple CPU / Allocations traces | Headless Instruments. On Xcode 26+, record with `--instrument 'CPU Profiler'` (not `--template 'Time Profiler'`, which can fail export). `--no-prompt --launch --` fits xtask. |
| Gitignored evidence dir | `target/dam-break-perf/<utc-stamp>/` | Dated pair JSON, samply `.json.gz`, optional `.trace` | `/target/` is already gitignored. Never copy these into `reference/performance/manifest.toml`. |
| Committed audit notes | `docs/` Markdown (extend `docs/playground-dam-break-timing.md` and add a named audit doc) | Hot functions, suspected causes, current Dam Break delta | Human-edited, source-bound, explicitly unreviewed. Not a Phase 12 public claim. |

### Supporting Libraries

Do **not** add production dependencies to `liquidfun` for this milestone. `liquidfun` stays `bitflags` only.

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Existing `World::step_profiled` / `DiagnosticStepProfile` | already in `crates/liquidfun` | Coarse parent/child wall times (`particle_prepare`, `particle_solve`, …) | Optional `--emit-phase-profile` on the private Dam Break bin. Separate run from the 3× gate; `Instant` inside the stepper is diagnostic overhead. |
| `serde` / `serde_json` | workspace 1.0.228 / 1.0.150 | Parse/write pair + profile metadata in xtask | Already used by `tools/xtask/src/playground.rs`. Extend the existing JSON object; do not add serde to `liquidfun`. |
| `dhat` | **0.3.3** | Heap allocation ranking (bytes × count) | Optional **private** feature on `liquidfun-wasm`’s `dam-break-bench` only (`dhat-heap`), behind `#[global_allocator]`. Never a default feature, never on `liquidfun`. Use after samply shows allocator/`Vec`/`HashMap` time. Author warns the crate is experimental and lightly maintained — treat as a one-off diagnostic, not a platform. |
| Criterion | **0.8.2** (already in `liquidfun-benchmarks`) | Microbench of a named kernel after samply identifies it | Do **not** replace the Dam Break pair with Criterion. Criterion cannot time the C++ oracle. Keep the pin; do not upgrade as part of this milestone. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `samply` 0.13.1 | Primary scripted CPU profiler | Install: `brew install samply` or `cargo install --locked samply --version 0.13.1`. Record the **profiling-profile** binary, not `cargo run --release`. Use `--save-only --unstable-presymbolicate -o target/dam-break-perf/<stamp>/rust.json.gz -- ./target/profiling/dam-break-bench --warmup 60 --steps 600`. Interactive follow-up: `samply load` that file. Launching the binary directly avoids samply’s known inability to profile SIP-signed system tools; locally built Cargo bins are fine. |
| `xcrun xctrace` | Apple CPU Profiler / Allocations | `xcrun xctrace record --instrument 'CPU Profiler' --no-prompt --output target/dam-break-perf/<stamp>/rust.trace --launch -- ./target/profiling/dam-break-bench --warmup 60 --steps 600`. Requires full Xcode, not only Command Line Tools. Use Allocations when samply is CPU-only and heap still looks suspicious. |
| `cargo-show-asm` 0.2.62 | Inspect LLVM/asm of a named hot function | After samply names a kernel: `cargo asm -p liquidfun --profile profiling <symbol>`. Optional; not wrapped in CI. |
| `counts` 1.0.7 | Ad-hoc `eprintln!` / log histograms | Optional during audit (pass-count, contact-count). `cargo install counts --version 1.0.7` or `brew install counts`. Do not leave print-profiling in production stepping. |
| Existing `just playground-dam-break-bench` | 3× gate | Unchanged command; still prints Markdown to stdout. xtask may additionally write a copy under `target/dam-break-perf/` for dated evidence. |
| New thin recipes | Profile + evidence | `just playground-dam-break-profile` → `cargo xtask playground dam-break-profile`. xtask builds `--profile profiling -p liquidfun-wasm --bin dam-break-bench`, invokes samply, records host/git/compiler, refuses to treat the profiled duration as the pair number. Optional `dam-break-profile-xctrace` for the Apple path. |
| Existing Phase 12 scripts | Optional strict sealed matrix | Leave `just phase12-performance-*` and `scripts/phase12-performance.sh` in place. Do not run them as the v1.2 gate. The reviewed-report manifest stays empty. |

## Integration with existing Dam Break pair

Current wiring (keep):

```
just playground-dam-break-bench
  → cargo xtask playground dam-break-bench
       → cargo xtask upstream configure/build --preset oracle-release --target playground-dam-break-bench
       → cargo run -p liquidfun-wasm --release --bin dam-break-bench
       → target/reference/oracle-release/playground-dam-break-bench
       → stdout Markdown table (unreviewed)
```

Add beside it, not instead of it:

```
just playground-dam-break-profile
  → cargo xtask playground dam-break-profile
       → cargo build -p liquidfun-wasm --profile profiling --bin dam-break-bench
       → samply record --save-only --unstable-presymbolicate -o target/dam-break-perf/<stamp>/rust.json.gz -- \
            ./target/profiling/dam-break-bench --warmup 60 --steps 600
       → write sidecar JSON: git HEAD, os/arch, rustc, samply version, binary path, warmup/steps
```

Rules:

- The 3× comparison always uses **unprofiled** `--release` vs `oracle-release`.
- Profiling uses `[profile.profiling]` so symbols exist; that binary is slower and is not the gate.
- C++ pair binary stays `oracle-release` (scalar, no `-ffast-math`). Optional C++ traces need a local debug-info overlay and are not the gate.
- Do not profile through `cargo run` if that hides the real binary path; xtask should pass the built path to samply/`xctrace`.
- WASM playground remains a **post-gate sanity check**. Do not pair WASM against C++. Do not add wasm-pack or browser profilers to this milestone’s required stack.

Hot-path audit (no new library): read `crates/liquidfun/src/particle/solver.rs` pass graph and `world/step/execution.rs` against pinned `b2ParticleSystem::Solve*` / `b2World::Step`. Sampling evidence decides which pass to change.

## Installation

Local developer machine (macOS aarch64). Not CI. Not a published-crate install.

```bash
# Existing pair (already in repo)
just playground-dam-break-bench

# CPU profiler (pick one install path; pin 0.13.1)
brew install samply
# or: cargo install --locked samply --version 0.13.1

# Optional Apple Instruments CLI (already present if Xcode is installed)
xcrun xctrace version

# Optional follow-up tools
cargo install --locked cargo-show-asm --version 0.2.62
cargo install --locked counts --version 1.0.7

# Workspace addition (repo change, not a cargo install):
# [profile.profiling]
# inherits = "release"
# debug = true
```

Do **not** add `llvm-tools-preview` to `rust-toolchain.toml` unless a later, explicit PGO experiment is authorized. Ordinary profiling does not need it.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `samply` 0.13.1 | `cargo-flamegraph` 0.6.14 | Want a static SVG and already have Xcode. On macOS, flamegraph now shells out to `xctrace` (not DTrace). Historically DTrace + Rosetta arch mismatches bit Apple Silicon (`flamegraph` issue 302). Keep flamegraph optional; do not make it the scripted default. |
| `samply` 0.13.1 | `cargo-instruments` 0.4.17 | Want Instruments.app GUI from `cargo instruments`. Skip as the scripted path: it rebuilds via cargo and can drift from the pair binary. xtask should launch samply/`xctrace` on a known path. |
| Direct `xctrace` | `cargo-instruments` | Same as above. Direct CLI is enough for `--launch` of `dam-break-bench`. |
| Existing Instant pair | Criterion 0.8.2 | Criterion is right for a **named Rust kernel** after samply. It cannot time pinned C++ and would hide the 3× pair. |
| Existing Instant pair | `hyperfine` 1.20.0 | Whole-process wrapper; would include setup unless the binary is already a timer. The Dam Break bins already exclude construction. Skip. |
| samply + `step_profiled` | Iai-Callgrind / Valgrind DHAT | Instruction counts are useful on **Linux**. Upstream Valgrind is not a supported Apple Silicon macOS tool (Darwin listed x86_64 through Ventura only). Do not block the hobby host on it. |
| samply | `pprof` crate / Tracy | Extra runtime instrumentation in the engine. Unnecessary for a 300× gap and would perturb the scalar baseline. |
| Evidence-driven scalar edits | `cargo-pgo` 0.3.0 + BOLT | After the pair is within a small factor and profiles show frontend/backend codegen, not algorithm. PGO will not close 300×. Never default, never CI. |
| Official samply 0.13.1 | feldera/samply 0.13.2 | That tag is a **different GitHub repo**, not crates.io `samply`. Pin `mstange/samply` 0.13.1. |
| `dhat` 0.3.3 feature on the private bin | Instruments Allocations via `xctrace` | Prefer Instruments if dhat’s global allocator is too invasive or hangs. Prefer dhat if you want a portable JSON heap profile without Xcode. |
| Workspace `[profile.profiling]` | `CARGO_PROFILE_RELEASE_DEBUG=true` on default release | Would silently change the 3× binary and CI artifacts. Keep a named profile. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Required performance CI job / PR gate | Hobby scope: local macOS is enough; `PROJECT-SCOPE.md` keeps benchmarks on-demand. A noisy 300× job would fail every PR until the gap closes. | Local `just` recipes. Existing `performance.yml` stays **manual dispatch** for the sealed Phase 12 path only. |
| Copying Dam Break numbers into `reference/performance/manifest.toml` | Manifest is the Phase 12 public-claim list and is empty on purpose. Filling it would revive sealed-matrix publication. | Dated gitignored reports + committed **notes** in `docs/`. |
| Running `scripts/phase12-performance.sh` as the v1.2 definition of done | 32-case sealed matrix, five-run Student intervals, controlled-host identity. Wrong bar for this canary. | Keep those scripts unused for this milestone. |
| Default SIMD (`std::simd`, `packed_simd`, `wide`, `pulp`) | Locked policy: scalar deterministic baseline. SIMD changes operation grouping and is not the 300× cause. | Scalar hot-path fixes. SIMD only as an explicit, documented opt-in **after** the 3× gate if profiles still show vectorizable loops. |
| Default `rayon` / thread pools / `std::thread` particle solve | Ordering and determinism; project forbids default parallel stepping. | Single-threaded baseline. Parallel only as explicit opt-in later. |
| `glam` / `nalgebra` in `liquidfun` | Operation order/layout vs the `f32` oracle. Already deferred in v1.0 stack research. | Existing `crates/liquidfun/src/math`. |
| C++ / CMake / `cc` / samply / dhat as `liquidfun` dependencies | Published crate must stay Cargo-only. `build.rs` compiling C++ would leak the oracle into consumers. | Private xtask, private `dam-break-bench` bin, developer-installed samply. |
| Profiling default `--release` without debug info | samply/Instruments stacks become addresses; wasted audit time. | `[profile.profiling]` with `debug = true`. |
| Using profiled or `--emit-phase-profile` wall times as the 3× number | Instrumentation bias. BENCHMARKING.md already says profiled timings are not timing authority. | Unprofiled `--release` pair. |
| LLVM PGO / BOLT as the first optimization | ~5–20% typical; cannot explain 300×. Needs `llvm-tools-preview` and extra binaries. | Algorithm/allocation audit first. |
| `-ffast-math`, `-C target-cpu=native`, `--release` with contraction/FMA flags for the pair | Changes IEEE vs scalar `oracle-release`; confuses speed with non-baseline math. | Keep both sides scalar baseline. |
| `unsafe` SIMD / unchecked indexing as the opening move | Workspace `unsafe_code = "forbid"` on `liquidfun`. 300× is unlikely to be bounds-check tax alone. | Safe structural fixes; any `unsafe` later needs a measured share, `SAFETY:` comment, and tests. |
| New publishable crates (`liquidfun-bench`, `liquidfun-simd`) | Over-fragmentation; C++ must not become a public dependency. | Keep benches in private `liquidfun-wasm` bin / xtask. Split a native-only bench crate only if samply shows `wasm-bindgen` in the hot path. |
| Browser/WASM profilers as the native gate | WASM vs C++ is not a fair pair. | Native pair first; playground is a post-gate sanity check. |
| `hyperfine` as pair authority | Times the wrong region unless carefully wrapped. | Existing in-process `Instant` around `advance` / `Step`. |
| feldera samply 0.13.2 installer | Not the official crate/repo pin. | `mstange/samply` 0.13.1. |

## Stack Patterns by Variant

**If measuring the 3× Dam Break gate:**

- Use `just playground-dam-break-bench` (unprofiled `--release` vs `oracle-release`).
- Because that is the locked same-host wall-clock pair; profiled builds and Criterion cannot substitute.

**If finding why Rust is ~300× slower:**

- Run `just playground-dam-break-profile` (samply on `[profile.profiling]`).
- Optionally dump `World::step_profiled` parent totals on a **separate** invocation.
- Because sampling names functions; existing diagnostic parents name phases; together they beat guesswork.

**If samply shows allocator / `clone` / `Vec::push` dominance:**

- Enable private `dhat-heap` on `dam-break-bench`, or `xctrace` Allocations.
- Because CPU samples undercount allocation volume; heap rank lists catch per-particle `malloc`.

**If samply names one math/contact kernel and the pair is already within a small factor:**

- Add or extend a Criterion bench in private `liquidfun-benchmarks` for that kernel.
- Because then you need tight iteration on one function, not the whole scene.

**If the native pair meets ≤ 3×:**

- Spot-check other playground scenes with the same unprofiled method.
- Rebuild WASM and note playground feel honestly.
- Because WASM is not compared to C++; it is a sanity check only.

**If someone asks for Phase 12 public claims:**

- Refuse for this milestone.
- Because `reference/performance/manifest.toml` is empty and the sealed 32-case method is a different product.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| samply 0.13.1 | macOS aarch64, locally built unsigned binaries, Rust 1.77+ to compile | Homebrew bottles include Apple Silicon. Cannot profile SIP-signed system executables; Cargo bins are fine. `--unstable-presymbolicate` is the 0.13.1 flag name (`--presymbolicate` appears on later `main`). |
| samply 0.13.1 | Firefox Profiler JSON (`.json.gz`) | `samply load` serves symbols locally. Commit notes, not the gzip, unless explicitly wanted. |
| Xcode / `xctrace` | AppleClang 21 on this host | Xcode 26+: `--instrument 'CPU Profiler'` for exportable traces. Full Xcode required. |
| cargo-flamegraph 0.6.14 | `xctrace` on macOS | Optional SVG. Do not also require `perf` (Linux-only). Avoid mixing x86 `cargo` under Rosetta with aarch64 bins. |
| cargo-show-asm 0.2.62 | Rust 1.97.0 workspace | Reads the same crate; use `--profile profiling` to match profiled codegen loosely (inlining still differs from `--release`). |
| dhat 0.3.3 | Optional feature on private bin | Global allocator; slower than system alloc. Disable for the 3× gate. Last crates.io release 2024-02-04 with an explicit maintenance warning. |
| Criterion 0.8.2 | Already pinned; MSRV 1.86 | Compatible with 1.97.0. Stay on 0.8.2. |
| cargo-pgo 0.3.0 | Needs `llvm-profdata` via `llvm-tools-preview` | Deferred. Not in `rust-toolchain.toml` today (`clippy`, `rustfmt` only). |
| iai-callgrind 0.16.x | Valgrind; not a macOS aarch64 hobby tool | Skip. |
| `liquidfun` `unsafe_code = forbid` | Safe scalar edits | SIMD intrinsics / unchecked unchecked slicing need an explicit lint exception later, not now. |
| `oracle-release` | Scalar C++ pair | Wrapper already strips fast-math / native-march. Do not add a “fast” C++ preset for the gate. |

## Sources

- [mstange/samply README (tag samply-v0.13.1)](https://github.com/mstange/samply/blob/samply-v0.13.1/README.md) — macOS support, release+debug-info profile, `samply record` / `samply setup`; HIGH
- [crates.io samply 0.13.1](https://crates.io/crates/samply) — official crate pin; HIGH
- [Homebrew samply 0.13.1](https://formulae.brew.sh/formula/samply) — Apple Silicon bottles; HIGH
- [samply v0.13.1 release notes](https://github.com/mstange/samply/releases/tag/samply-v0.13.1) — `--unstable-presymbolicate`, macOS attach/`samply setup`; HIGH
- [feldera/samply v0.13.2](https://github.com/feldera/samply/releases/tag/v0.13.2) — **not** the official pin; HIGH (negative)
- [flamegraph-rs README / crates.io 0.6.14](https://crates.io/crates/flamegraph) — macOS backend is `xctrace`; published 2026-08-12; HIGH
- [Homebrew cargo-flamegraph 0.6.14](https://formulae.brew.sh/formula/cargo-flamegraph) — HIGH
- [xctrace(1)](https://keith.github.io/xcode-man-pages/xctrace.1.html) — `record --launch`, `--no-prompt`; HIGH
- [crates.io cargo-instruments 0.4.17](https://crates.io/crates/cargo-instruments) — 2026-06-04; MEDIUM as optional wrapper only
- [crates.io dhat 0.3.3](https://crates.io/crates/dhat) — heap profiler API and maintenance warning; HIGH for version, MEDIUM for long-term fitness
- [crates.io cargo-show-asm 0.2.62](https://crates.io/crates/cargo-show-asm) — 2026-06-26; HIGH
- [crates.io criterion 0.8.2](https://crates.io/crates/criterion) — still latest as of 2026-09-20 (released 2026-02-04); HIGH
- [crates.io cargo-pgo 0.3.0](https://crates.io/crates/cargo-pgo) — 2026-01-25; HIGH for version, HIGH that it is the wrong first lever
- [Valgrind current release 3.27.1](https://valgrind.org/downloads/) — Darwin listed as x86/amd64 through Ventura, not Apple Silicon; HIGH
- [iai-callgrind docs](https://docs.rs/crate/iai-callgrind/latest) — not for platforms Valgrind does not support; HIGH
- Repo: `docs/playground-dam-break-timing.md`, `BENCHMARKING.md`, `PROJECT-SCOPE.md`, `tools/xtask/src/playground.rs`, `crates/liquidfun-wasm/src/dam_break_bench.rs`, `crates/liquidfun/src/world/observation/profile.rs` — existing pair and diagnostic timers; HIGH

---
*Stack research for: v1.2 Native Performance Closing (Dam Break ≤ 3× C++, local macOS aarch64)*
*Researched: 2026-09-20*
