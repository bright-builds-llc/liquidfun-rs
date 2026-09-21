# Architecture Research

**Domain:** Native LiquidFun performance closing — scripted Dam Break pair, CPU sampling, committed audit notes, and profile-guided hot-path fixes
**Researched:** 2026-09-20
**Confidence:** HIGH for existing crate, xtask, oracle, and package-isolation seams (verified in this checkout); MEDIUM for host profiler install/permission details (`samply setup` on macOS)

This document is **v1.2 Native Performance Closing** architecture. It does **not** redesign `crates/liquidfun`, the SolidJS playground, or the Phase 12 sealed matrix. The engine remains one publishable crate with a private oracle shell. v1.2 adds an observability loop around the existing Dam Break pair, then changes only shared native hot paths that sampling names.

## Standard Architecture

### System Overview

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ Discoverable shell (thin)                                                │
│   just playground-dam-break-bench                                        │
│   just playground-dam-break-profile                                      │
│   just web-player-smoke   (post-gate WASM sanity only)                   │
└──────────────┬───────────────────────────────────┬───────────────────────┘
               │ prints the command                │ no CMake / no profiler
               ▼                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ Imperative shell: private `cargo xtask`                                  │
│   playground dam-break-bench   unprofiled Instant pair (≤3× authority)   │
│   playground dam-break-profile dated CPU samples + pair snapshot         │
│   upstream configure/build     oracle-release extra target               │
│   package verify               isolation gate (unchanged)                │
└───────┬──────────────────────────────┬───────────────────┬───────────────┘
        │ spawn                        │ spawn             │ spawn
        ▼                              ▼                   ▼
┌─────────────────────┐    ┌───────────────────────┐   ┌───────────────────┐
│ Native Rust timer   │    │ Pinned C++ timer      │   │ Host sampler       │
│ liquidfun-wasm      │    │ playground-dam-break- │   │ samply / optional  │
│ --bin dam-break-    │    │ bench                 │   │ Instruments wrap   │
│ bench (--release or │    │ target/reference/     │   │ of the same bins   │
│ --profile profiling)│    │ oracle-release/       │   │                    │
└──────────┬──────────┘    └──────────┬────────────┘   └─────────┬─────────┘
           │ World::step              │ b2World::Step            │
           │ (not step_profiled)      │ out of process           │
           ▼                          ▼                          ▼
┌─────────────────────────────────────────────┐   ┌─────────────────────────┐
│ Published engine: crates/liquidfun          │   │ Gitignored evidence     │
│ particle/rigid source-ordered kernels       │   │ target/v12-native-perf/ │
│ DiagnosticProfileSchema::Phase12V1          │   │ <UTC>/                  │
│ (diagnostic Instant only; never the gate)   │   │ pair.json, *.json.gz    │
└─────────────────────────────────────────────┘   └────────────┬────────────┘
                                                               │ human summary
                                                               ▼
                                                  docs/native-performance-audit.md
                                                  (committed; not Phase 12 claim)
```

Dependency arrows still point toward `liquidfun`. Profiling, CMake, samply, playground scenes, and dated reports never become production dependencies.

### Component Responsibilities

| Component | Responsibility | Typical implementation |
|-----------|----------------|------------------------|
| `crates/liquidfun` | Native physics; shared particle/rigid hot paths; optional `World::step_profiled` diagnostic schema | Existing deep modules. v1.2 **modifies** solver/storage after evidence; adds no profiler, serde, C++, or WASM |
| `crates/liquidfun-wasm` `dam-break-bench` | Native-only playground Dam Break Medium/Normal timer around `SessionCore::advance` → `World::step` | Keep the bin here (`#[cfg(not(target_arch = "wasm32"))]`). Do not move scenes into the published crate |
| `tools/reference` `playground-dam-break-bench` | Matching C++ extra target: construct recipe, time `b2World::Step` only | Existing CMake extra target under `oracle-release`; still not the JSONL oracle |
| `tools/xtask` `playground` | Rebuild oracle extra target, spawn both timers, validate JSON, print Markdown | **Modify** to dump dated pair files; **add** a profile subcommand that wraps the same binaries |
| `just` recipes | One-line aliases that print the underlying `cargo xtask …` | **Add** thin profile alias; never embed CMake, Ninja, or samply flags |
| Host CPU sampler | Attribute samples to functions in optimized binaries | Out-of-process `samply record` (default). Optional macOS Instruments. Not a workspace crate |
| `target/v12-native-perf/<UTC>/` | Gitignored dated pair + CPU-profile artifacts | New directory under already-ignored `/target/` |
| `docs/native-performance-audit.md` | Committed named hot functions, suspected causes, current Dam Break delta | New doc. Unreviewed; not `reference/performance/manifest.toml` |
| `docs/playground-dam-break-timing.md` | Reproduce recipe + latest unreviewed sample table | **Modify** after re-measures; keep the unreviewed banner |
| Phase 12 `cargo xtask performance` | Sealed 32-case matrix, empty reviewed-report manifest | Leave unused as v1.2 gate. Parent/child schema stays diagnostic-only |
| Playground `web/` + `just web-player-smoke` | Post-native-gate WASM sanity | Unchanged architecture; run after ≤3×, never vs C++ |
| `crates/liquidfun-benchmarks` | Private Criterion catalog benches | Do not hijack for Dam Break closing |

## Recommended Project Structure

```text
.
├── Cargo.toml                          # ADD [profile.profiling] inherits=release
├── justfile                            # ADD thin playground-dam-break-profile
├── .gitignore                          # /target/ already covers evidence
├── BENCHMARKING.md                     # MODIFY: v1.2 pair ≠ Phase 12 claim
├── docs/
│   ├── playground-dam-break-timing.md  # MODIFY: refresh unreviewed sample
│   └── native-performance-audit.md     # NEW: committed notes (not a sealed claim)
├── crates/
│   ├── liquidfun/                      # MODIFY hot paths only after profiles
│   │   └── src/
│   │       ├── particle/solver/        # likely first shared-fix surface
│   │       ├── particle/contact.rs
│   │       ├── particle/body_contact.rs
│   │       ├── particle/proxy.rs
│   │       ├── world/step/execution.rs # keep step vs step_profiled split
│   │       └── world/observation/profile.rs  # Phase12V1 diagnostic only
│   └── liquidfun-wasm/                 # KEEP timer + scenes here
│       ├── src/dam_break_bench.rs      # unprofiled Instant authority
│       ├── src/bin/dam_break_bench.rs
│       └── src/scene/*.rs              # spot-check other scenes natively
├── tools/
│   ├── xtask/src/
│   │   ├── playground.rs               # SPLIT as it grows; keep pair command
│   │   └── playground/
│   │       ├── pair.rs                 # existing pair + dated dump
│   │       └── profile.rs              # NEW: wrap bins with samply
│   └── reference/
│       ├── CMakeLists.txt              # KEEP extra target; optional -g flag
│       ├── CMakePresets.json           # KEEP oracle-release; no FFI preset
│       └── src/playground_dam_break_bench.cpp
└── target/                             # gitignored
    ├── reference/oracle-release/       # C++ extra-target binary
    ├── release/ / profiling/           # Rust bins
    └── v12-native-perf/<UTC>/          # NEW dated evidence
```

### Structure Rationale

- **`crates/liquidfun`:** The only place shared physics may change. Package isolation forbids C++, renderers, protocol, and “benchmark” paths inside the published archive (`tools/xtask/src/package.rs` forbidden prefixes/terms).
- **`crates/liquidfun-wasm`:** Owns the playground recipe and the native timer so the canary matches the product scene. The crate is `publish = false` and outside `default-members`.
- **`tools/xtask`:** Owns rebuild, process spawn, JSON validation, evidence paths, and profiler invocation. Matches the existing “just prints; xtask orchestrates; CMake builds C++” layering.
- **`tools/reference`:** Stays the only C++ build graph. Dam Break C++ remains an extra executable linked to `Box2D`, not in-process FFI.
- **`target/v12-native-perf/`:** Dated, never overwritten, never committed. Same pattern as `target/phase12-performance/` raw files: local, unreviewed, non-claiming.
- **`docs/native-performance-audit.md`:** The durable human artifact. Profiles expire; named functions and hypothesized causes belong in git.

## Architectural Patterns

### Pattern 1: Out-of-process pair as wall-clock authority

**What:** Same-host scalar `--release` Rust timer vs `oracle-release` C++ extra target. Construction, particle insert, warm-up, capture, and rendering stay outside `Instant` / `steady_clock`.
**When to use:** Every v1.2 numeric decision, including the Dam Break ≤ 3× gate and post-fix re-measures.
**Trade-offs:** Sequential Rust-then-C++ (today) is enough for a ~300× gap; it is not Phase 12’s five-run interleaved protocol. Do not “upgrade” the canary into the empty sealed matrix.

**Example:**

```rust
// tools/xtask playground dam-break-bench (existing)
upstream::run(&["configure", "--preset", "oracle-release"])?;
upstream::run(&[
    "build", "--preset", "oracle-release",
    "--target", "playground-dam-break-bench",
])?;
let rust = cargo_run_wasm_dam_break_bench("--release")?;
let cpp = run_target_reference_binary()?;
validate_sample(&rust, "native_rust")?;
validate_sample(&cpp, "pinned_cpp")?;
```

### Pattern 2: Dedicated profiling Cargo profile, not a changed `--release`

**What:** Workspace `[profile.profiling]` inherits `release`, sets `debug = "limited"` (or `"line-tables-only"` if stacks are sufficient) and `strip = false`. Record with `samply` / Instruments. Optional `RUSTFLAGS='-C force-frame-pointers=yes'` only on this profile.
**When to use:** CPU attribution after an unprofiled pair exists for that same git SHA.
**Trade-offs:** Profiling binaries can be slightly slower than stripped `--release`. That is why profiled wall times are **never** the ≤3× authority (same rule as Phase 12: profiled timings are diagnostic-only).

```toml
# root Cargo.toml — does not ship inside cargo package -p liquidfun
[profile.profiling]
inherits = "release"
debug = "limited"
strip = false
```

### Pattern 3: Sampler wraps binaries; engine is not instrumented for the gate

**What:** xtask locates `samply` (or `xctrace`) on `PATH`, rebuilds the profiling binary and the C++ extra target, then `samply record --save-only -o <evidence>/rust.json.gz -- <bin> --warmup 60 --steps 600`. C++ is the same wrap of `target/reference/oracle-release/playground-dam-break-bench`.
**When to use:** First audit and after each optimization wave that needs new names.
**Trade-offs:** macOS needs `samply setup` once. Missing sampler is a hard xtask error with install text, not a silent skip. Do not add `samply` to `[workspace.dependencies]`. Do not compile a profiler into `liquidfun`.

Do **not** switch Dam Break timing to `World::step_profiled`. That API already exists (`crates/liquidfun/src/world/step/execution.rs`) and records parent/child `Instant` spans under `phase12-profile-v1`. It is useful as an **optional second dump** in evidence JSON. It is not sampling, not public timing authority, and must not run inside the unprofiled pair process.

### Pattern 4: Measure → note → shared fix → re-measure

**What:** No physics PR without a named profile share or typed allocation/algorithm note in the audit doc. Fixes land in shared `liquidfun` kernels, not in wasm scene files, not as SIMD/parallel defaults.
**When to use:** After the baseline pair+profile exists.
**Trade-offs:** Slower calendar time than guessing; avoids optimizing the wrong extra check or a playground-only hook.

Dam Break `on_advance` is currently a no-op (`crates/liquidfun-wasm/src/scene/dam_break.rs`). `SessionCore::advance` still calls it before `World::step`. Keep measuring `advance` so the canary stays on the playground path. Fountain / Water Wheel hooks do real work; their spot-checks are native-only and are **not** C++ pairs.

## Data Flow

### Request Flow (one local closing loop)

```text
just playground-dam-break-profile
        ↓
cargo xtask playground dam-break-profile
        ↓
1. mkdir target/v12-native-perf/<UTC>/   (fail if the stamp exists)
2. cargo xtask upstream configure/build --preset oracle-release
     --target playground-dam-break-bench
3. cargo run -p liquidfun-wasm --release --bin dam-break-bench
   + C++ extra target                         ← UNPROFILED PAIR (authority)
        ↓
   pair.json + pair.md in the dated directory
        ↓
4. cargo build -p liquidfun-wasm --bin dam-break-bench --profile profiling
   (+ optional C++ -g / -fno-omit-frame-pointer on the extra target only)
        ↓
5. samply record --save-only  rust.json.gz  -- <profiling bin> --warmup 60 --steps 600
   samply record --save-only  cpp.json.gz   -- <oracle-release extra target>
        ↓
6. optional: separate process dump of DiagnosticStepProfile parent shares
   (never mixed into pair.json wall_ms)
        ↓
7. human updates docs/native-performance-audit.md (hot functions, causes, delta)
        ↓
8. shared hot-path edits in crates/liquidfun
        ↓
9. cargo test -p liquidfun  (+ focused particle/world tests)
   cargo xtask playground dam-break-bench     ← RE-MEASURE unprofiled
        ↓
10. native spot-check other SceneId sessions (no new C++ targets)
        ↓
11. when pair ≤ 3×: just web-wasm / web-player-smoke
    record playground sanity honestly (WASM is not compared to C++)
```

### State Management

```text
Committed (durable, non-numeric-authority)
  docs/native-performance-audit.md
  docs/playground-dam-break-timing.md  (unreviewed sample + reproduce)
  BENCHMARKING.md                      (method boundaries)

Gitignored (dated, never overwritten)
  target/v12-native-perf/<UTC>/pair.json
  target/v12-native-perf/<UTC>/rust.json.gz
  target/v12-native-perf/<UTC>/cpp.json.gz
  target/v12-native-perf/<UTC>/host.json   (HEAD, rustc, clang, OS, cores)

Not used as v1.2 gate
  reference/performance/manifest.toml      (empty; Phase 12)
  target/phase12-performance/              (sealed workflow)
```

### Key Data Flows

1. **Rebuild:** xtask `upstream configure --preset oracle-release` then `upstream build --preset oracle-release --target playground-dam-break-bench`. Same allowlisted target as today’s pair (`tools/xtask/src/upstream.rs`).
2. **Unprofiled pair:** Spawn `liquidfun-wasm` `dam-break-bench` with `--release` and the C++ extra binary; parse one JSON object each; require `engine`, 1920 particles, matching warmup/steps. Print Markdown; **also** write `pair.json` / `pair.md` into the dated evidence directory when the new profile command (or an explicit `--write-evidence` flag on the pair command) runs.
3. **Profiled run:** Rebuild with `[profile.profiling]`; wrap **the same argv** (`--warmup` / `--steps`) with the host sampler; save Firefox-Profiler JSON (or `.trace`) beside the pair. Reject using those files’ wall times as the gate.
4. **Notes:** Copy function names and approximate shares into `docs/native-performance-audit.md` with git HEAD and evidence directory name. Do not paste flamegraphs into git.
5. **Optimize:** Change `liquidfun` particle/rigid code that the profile named. Preserve source-ordered kernels, scalar IEEE, safe Rust default, `maybe_` naming, tau-based angles. SIMD/Rayon stay explicit opt-in and out of this milestone’s default path.
6. **Re-measure:** New dated directory; compare unprofiled `wall_ms` ratio to C++. Preserve the previous dated directory (failed or slower records stay).
7. **WASM sanity:** Existing `bun scripts/web-build.ts` path. No C++ oracle, no samply of `wasm32`, no new engine API.

## New vs Modified

| Surface | New or modified | Why |
|---------|-----------------|-----|
| `Cargo.toml` `[profile.profiling]` | **New** | Release codegen + symbols without mutating default `--release` |
| `tools/xtask/src/playground/profile.rs` | **New** | Sampler wrap, evidence mkdir, host identity |
| `just playground-dam-break-profile` | **New** | Thin alias only |
| `docs/native-performance-audit.md` | **New** | Committed notes the roadmap asked for |
| `target/v12-native-perf/` | **New** (gitignored) | Dated pair + profiles |
| Optional CMake cache `REFERENCE_PROFILE_DEBUG_INFO` on extra target | **New, optional** | `-g` / frame pointers without a new preset or `-ffast-math` |
| `cargo xtask playground dam-break-bench` | **Modified** | Keep behavior; add dated dump / reuse from profile command |
| `tools/xtask/src/playground.rs` | **Modified** | Route `dam-break-profile`; split files before it grows past review size |
| `justfile` | **Modified** | One extra recipe; no CMake body |
| `crates/liquidfun` particle/rigid modules | **Modified after evidence** | Shared hot-path fixes |
| `docs/playground-dam-break-timing.md` | **Modified** | Refresh sample after waves; keep unreviewed banner |
| `BENCHMARKING.md` | **Modified** | One paragraph: playground pair is exploratory, not Phase 12 |
| `crates/liquidfun-wasm` timer | **Unchanged unless argv needed** | Already times the locked recipe via `World::step` |
| C++ `playground_dam_break_bench.cpp` | **Unchanged** | Recipe lock + `b2World::Step` timer |
| `crates/liquidfun` `World::step` vs `step_profiled` | **Unchanged contract** | Optional extra dump only |
| `cargo xtask performance` / `scripts/phase12-performance.sh` | **Unchanged / unused as gate** | Empty reviewed-report manifest |
| `web/` playground | **Unchanged until post-gate sanity** | No renderer coupling to core |
| `crates/liquidfun/Cargo.toml` | **Must stay unchanged** | No samply, serde, wasm-bindgen, CMake, or `build.rs` |

## What Must Stay Out of Published `liquidfun`

Verified isolation today: sole `default-members = ["crates/liquidfun"]`; runtime deps are `bitflags` only; `include` is crate sources + license/readme; `cargo xtask package verify` unpacks and tests outside the repo; archive inspection rejects `tools/`, `third_party/`, `reference/`, and path terms including `benchmark`, `oracle`, `renderer`.

Keep out of `liquidfun`:

- samply, Instruments, flamegraph, Criterion, serde/JSON report writers
- CMake, Ninja, submodule, FFI, in-process C ABI
- wasm-bindgen, scene catalogs, Canvas/SolidJS
- dated evidence files and host CPU dumps
- playground Dam Break recipe constants as a public API (they stay in `liquidfun-wasm` + C++ extra target)
- default SIMD, Rayon, `-ffast-math`, or `HashMap` iteration in solver-visible order

Allowed inside `liquidfun`: allocation/algorithm/shape reductions that preserve semantics; existing diagnostic profile **schema** (already public, non-authoritative); `unsafe` only if a measured need plus `SAFETY:` and tests — not as the first move.

## Suggested Build Order (dependency-aware)

Phases below are roadmap suggestions, not an approved plan. They encode **measure before optimize** and **package isolation**.

1. **Observability shell (no physics).** Workspace `profiling` profile; xtask `dam-break-profile` + dated `target/v12-native-perf/<UTC>/`; thin just alias; audit-doc skeleton; unit tests with fake cmake/samply. Run `cargo xtask package verify` (or `just check`) so isolation still holds. **Avoids:** changing kernels before names exist; stuffing CMake into just; adding profiler crates to `liquidfun`.

2. **Baseline unprofiled pair.** `just playground-dam-break-bench` at a recorded HEAD; write pair artifacts; refresh `docs/playground-dam-break-timing.md` as an unreviewed sample. **Depends on:** step 1 dump path (or existing stdout). **Avoids:** treating this number as Phase 12.

3. **Baseline CPU profiles.** Profiled rebuild + samply of Rust (required) and C++ extra target (strongly recommended for “extra Rust work vs C++”). Fill audit notes with hot functions and suspected extra per-particle work / allocations / checks / shape differences. Optional `step_profiled` parent-share dump in a **separate** process. **Depends on:** step 2 SHA identity. **Avoids:** in-process FFI; debug-build profiles; using profiled `wall_ms` as the gate.

4. **Shared hot-path waves in `liquidfun`.** One concern per wave (for example contacts, then pressure, then allocations). After each wave: focused `liquidfun` tests, unprofiled pair into a **new** dated directory, append audit notes. **Depends on:** step 3 names. **Avoids:** scene-local hacks in `liquidfun-wasm`; SIMD/parallel defaults; rewriting the playground.

5. **Native spot-checks.** Short `SessionCore::advance` timings (or existing scene tests) for Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel. Record “improved / unchanged / worse” honestly. No new C++ extra targets. **Depends on:** at least one successful shared-path wave. **Avoids:** a second sealed matrix.

6. **Dam Break ≤ 3× gate.** Unprofiled `--release` vs `oracle-release` on the same host. Document remaining delta in the audit doc. **Depends on:** steps 4–5. **Avoids:** copying the ratio into `reference/performance/manifest.toml`.

7. **WASM / playground sanity (last).** `just web-wasm` and/or `just web-player-smoke`. WASM is not compared to C++. **Depends on:** native gate so the browser loop is not used as a profiler. **Avoids:** coupling core simulation to Canvas; profiling `wasm32` as the closing method.

Do not run step 7 as a substitute for step 2. Do not start step 4 during step 1. Do not revive Linux-host qualification as a blocker (hobby scope).

## Scaling Considerations

This milestone is one scene on one developer host, not user scale.

| Scale | Architecture adjustments |
|-------|--------------------------|
| First audit (~300× gap) | Sequential pair + one samply capture each side is enough |
| Closing toward ≤ 3× | Re-pair after every wave; keep old evidence directories |
| Later optional Phase 12 | Only if explicitly authorized; separate sealed matrix, not this loop |

### Scaling Priorities

1. **First bottleneck:** Unknown until profiles exist; hypothesized extra per-particle work, allocations, and checks in particle solve/contacts. Detect with samply of `--profile profiling` Dam Break, not with Criterion catalog benches.
2. **Second bottleneck:** Attribution quality (inlined frames, missing C++ `-g`). Fix the profiling profile / extra-target debug info; do not loosen the unprofiled `--release` gate.

## Anti-Patterns

### Anti-Pattern 1: In-process C ABI as the first profiling path

**What people do:** Add `extern "C"` into the oracle so Rust can time C++ in-process, or `cc`/`bindgen` from `liquidfun`.
**Why it's wrong:** Violates reference isolation, package isolation, and crash isolation. The ~300× gap is not an IPC problem; both timers are already local processes around `Step` only.
**Do this instead:** Keep spawning `playground-dam-break-bench` and `dam-break-bench`. FFI stays deferred unless a later profile proves process startup is the measured region (it is not: startup is outside `Instant`).

### Anti-Pattern 2: Treating `World::step_profiled` or sampler wall time as the ≤3× gate

**What people do:** Enable diagnostic Instant spans (or compare samply duration) and declare victory.
**Why it's wrong:** Phase 12 policy and `BENCHMARKING.md` already say profiled timings are never public timing authority. Instrumentation and debuginfo change the number.
**Do this instead:** Unprofiled `--release` pair remains canonical. Profiles only name functions.

### Anti-Pattern 3: Hiding CMake or samply in just recipes

**What people do:** A 80-line `just` recipe that detects OS, writes CMake flags, and shells out to Instruments.
**Why it's wrong:** Repository convention: just prints; xtask owns orchestration; CMake owns C++.
**Do this instead:** `just playground-dam-break-profile` → `cargo xtask playground dam-break-profile`.

### Anti-Pattern 4: Coupling core simulation to the playground renderer

**What people do:** Time Canvas frames, or add debug-draw collection inside the native timer.
**Why it's wrong:** The documented pair excludes capture and rendering. Core must stay headless.
**Do this instead:** Keep `dam-break-bench` on `SessionCore::advance` / `World::step`. WASM sanity is a later, separate smoke.

### Anti-Pattern 5: Mutating default `--release` or `oracle-release` scalar flags for nicer stacks

**What people do:** Set `[profile.release] debug = true` globally, or switch the C++ preset to `RelWithDebInfo` / `-ffast-math` / `-march=native`.
**Why it's wrong:** Changes the authority build. RelWithDebInfo is not guaranteed identical to Release. Fast-math is already forbidden for canonical parity builds.
**Do this instead:** Dedicated Cargo `profiling` profile; optional extra-target `-g` cache flag that the **profile** command turns on, never the pair command.

### Anti-Pattern 6: Copying exploratory numbers into the Phase 12 manifest

**What people do:** Paste Dam Break ms/step into `reference/performance/manifest.toml`.
**Why it's wrong:** That manifest is the only public-claim list and is empty by policy. The playground pair is not the 32-case sealed matrix.
**Do this instead:** Keep numbers in gitignored evidence + unreviewed docs with the existing banner.

### Anti-Pattern 7: Overwriting failed evidence or optimizing from a mixed SHA

**What people do:** Reuse `target/v12-native-perf/latest/` and replace files; profile commit A, patch commit B, report A’s ratio.
**Why it's wrong:** Standing authorization requires preserving failed records and separate attempt directories.
**Do this instead:** UTC (or git-HEAD+UTC) directories; refuse to clobber; bind audit notes to HEAD + directory name.

### Anti-Pattern 8: Scene-local or SIMD-first “fixes”

**What people do:** Special-case Dam Break in `liquidfun-wasm`, or flip on default parallel particle solves.
**Why it's wrong:** Other scenes would not benefit; determinism/source order is compatibility policy.
**Do this instead:** Shared kernel changes named by the profile; SIMD/parallel only as explicit opt-in after the scalar close.

## Integration Points

### External Services

| Service | Integration pattern | Notes |
|---------|---------------------|-------|
| Host `samply` | xtask `Command` spawn, `--save-only` into evidence | Install with `cargo install --locked samply`; macOS `samply setup`. Not a Cargo workspace dep |
| Optional Instruments / `xctrace` | Same wrap, extra artifact | macOS-only extra; do not make CI require Xcode |
| CMake 4.x + Ninja | Existing `cargo xtask upstream` | `oracle-release` only for this pair; never asan/ubsan for timing |
| Pinned upstream submodule | Read-only `Box2D` link for the extra target | Pair command already configures it; published crate still must not need it |
| Firefox Profiler UI | Human opens `*.json.gz` locally | Do not commit profiles |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| just ↔ xtask | Thin argv | No CMake/samply flags in just |
| xtask ↔ `dam-break-bench` | Process + JSON stdout | Existing parser; stderr is diagnostics |
| xtask ↔ C++ extra target | Process + JSON stdout | Binary under `target/reference/oracle-release/` |
| xtask ↔ samply | Process wrapping the same argv | Profiled wall time discarded for the gate |
| `liquidfun-wasm` ↔ `liquidfun` | Public `World::step` | Timer crate may depend on engine; engine must not depend on wasm |
| Diagnostic profile ↔ StepReport | Separate return value | `step_profiled` must not enter pair JSON |
| v1.2 evidence ↔ Phase 12 manifest | None | Different directories and claim rules |
| Native gate ↔ WASM smoke | Sequence only | WASM after ≤3×; no C++ comparison |
| Package verify ↔ profiling profile | Workspace Cargo.toml only | `cargo package -p liquidfun` must still exclude tools/C++/profiles artifacts |

## Sources

- This checkout: `crates/liquidfun-wasm/src/dam_break_bench.rs`, `src/bin/dam_break_bench.rs`, `src/session.rs` (`advance` → `World::step`), `src/scene/dam_break.rs` (empty `on_advance`)
- This checkout: `tools/xtask/src/playground.rs`, `tools/xtask/src/upstream.rs` (allowlisted `playground-dam-break-bench`), `tools/reference/CMakeLists.txt`, `tools/reference/src/playground_dam_break_bench.cpp`, `tools/reference/CMakePresets.json` (`oracle-release` = Release)
- This checkout: `crates/liquidfun/src/world/step/execution.rs` (`step` vs `step_profiled`), `crates/liquidfun/src/world/observation/profile.rs` (`phase12-profile-v1`)
- This checkout: `BENCHMARKING.md` (unprofiled authority; diagnostic profiles; empty reviewed-report manifest), `docs/playground-dam-break-timing.md`, `ARCHITECTURE.md` (crate dependency direction), `tools/xtask/src/package.rs` (isolation deny lists)
- [Profiling — The Rust Performance Book](https://nnethercote.github.io/perf-book/profiling.html) — samply / Instruments / flamegraph; debuginfo; frame pointers (HIGH)
- [samply README](https://github.com/mstange/samply) — record optimized binaries with debug info; macOS `setup` (HIGH)
- [samply#763](https://github.com/mstange/samply/issues/763) — `debug = "limited"` is enough for inlining/line tables (MEDIUM)

---
*Architecture research for: v1.2 Native Performance Closing (Dam Break ≤ 3× C++, scripted pair + CPU profiles, shared hot paths)*
*Researched: 2026-09-20*
