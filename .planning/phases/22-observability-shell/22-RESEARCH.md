# Phase 22: Observability shell - Research

**Researched:** 2026-09-20
**Domain:** Dam Break pair persistence, Cargo `[profile.profiling]`, samply 0.13.1 capture, separate `step_profiled` timers
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Evidence stamp layout
- **D-01:** Write all Phase 22 artifacts under `target/dam-break-perf/<utc-stamp>/`. Use a filename-safe UTC stamp `YYYY-MM-DDTHH-MM-SSZ`. If that directory already exists, fail closed and mint a new stamp — never overwrite, merge, or delete an existing stamp.
- **D-02:** Extend the existing unprofiled pair so it is not stdout-only. Each pair run writes `pair.json` (machine) and `pair.md` (human table matching the current Markdown stdout) containing wall ms, ms/step, Rust/C++ ratio, host, git HEAD, and compilers. Keep printing the Markdown table to stdout.
- **D-03:** A profile run writes `rust.json.gz` plus `profile-identity.json` (host, git HEAD, compiler, exact command). Profiled wall times, samply duration, and `[profile.profiling]` timings must never be copied into `pair.json` or treated as the 3× number.
- **D-04:** A timer run writes `timers.json` into a **new** stamp. Timers may share the same schema vocabulary as existing Phase 12 parents, but they are a separate diagnostic artifact and must not be mixed into the unprofiled pair process.

### Recipe surface
- **D-05:** Keep `just playground-dam-break-bench` → `cargo xtask playground dam-break-bench` as the unprofiled 3×-authority pair. Persist into a new stamp; do not replace or rename this recipe.
- **D-06:** Add thin aliases `just playground-dam-break-profile` → `cargo xtask playground dam-break-profile` and `just playground-dam-break-timers` → `cargo xtask playground dam-break-timers`. `just` remains a one-line printer; xtask owns orchestration, argument parsing, stamp creation, and tool invocation.
- **D-07:** Do not hide CMake or samply flags inside `just`. Optional C++ `-g` is an explicit profile-command flag only (for example `--cpp-debuginfo`) used as a profile-command cache flag. It is never a pair-command flag and never a new CMake preset.

### Profile capture
- **D-08:** Add a workspace `[profile.profiling]` that `inherits = "release"` and sets `debug = true`. The profile recipe rebuilds the existing Dam Break Rust binary with that profile and captures samply **0.13.1** of the timed Rust loop. Construction, insertion, warm-up, capture, and rendering stay outside the sampled timed loop, same as the unprofiled pair.
- **D-09:** Missing samply fails closed with install/error text (pin `samply` 0.13.1). Do not skip, stub a success, or write a placeholder `rust.json.gz`.
- **D-10:** Default `--release` remains the gate binary. Do not call `World::step_profiled` from `dam-break-bench`. Do not enable `DiagnosticStepProfiler` on the unprofiled pair path. Do not change physics kernels.

### Timer diagnostic path
- **D-11:** Coarse parent-phase timers use the existing `step_profiled` parents (`particle_prepare` / `particle_solve` / `rigid_solve`, plus the rest of the already-public Phase 12 parent set) on a **separate** Dam Break diagnostic path. The unprofiled gate process stays on ordinary `World::step`.

### Isolation and proof
- **D-12:** `liquidfun` gains no profiler, samply, dhat, CMake, or serde dependency. Package isolation still holds: `cargo xtask package verify` must pass. Dated evidence stays gitignored under `target/`.
- **D-13:** Prove tooling with fake cmake/samply in xtask tests, following the existing `tools/xtask/tests/upstream_cli` fake-tool pattern. Do not require a live Dam Break C++ pair, a real samply session, or Instruments as a CI or phase gate. A live local run is optional developer proof, not definition of done.
- **D-14:** Do not commit `.json.gz`, `.trace`, or host dumps. Do not copy pair numbers into `reference/performance/manifest.toml`. Do not refresh `docs/playground-dam-break-timing.md` as a Phase 12 claim. Named-function audit notes and dhat belong to Phase 23.

### Claude's Discretion
- Exact JSON field names beyond the required identity and timing fields.
- Exact samply argv, output wrapping, and how the timed loop is targeted.
- Fake-samply / fake-cmake fixture shape, as long as missing real samply still fails closed in production.
- Whether profile and timer commands accept `--warmup` / `--steps` with the same defaults as the pair (60 / 600).
- Exact stderr wording for “stamp exists” and “samply missing,” provided they fail closed.

### Deferred Ideas (OUT OF SCOPE)
- Named dominating-function audit and `docs/native-performance-audit.md` — Phase 23 (`PERF-AUDIT`).
- Refresh `docs/playground-dam-break-timing.md` as an unreviewed sample bound to a SHA — Phase 23.
- Private dhat heap dump when samply shows allocator/`Vec` time — Phase 23 (`PERF-HEAP`).
- Shared hot-path physics edits and the ≤ 3× gate — Phase 24.
- WASM/playground sanity versus previous WASM or native Rust, never versus C++ — Phase 25.
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in after the scalar canary.
- In-process C ABI / FFI profiling — not this milestone’s first path.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERF-PAIR | Re-run locked Dam Break Medium pair (`just playground-dam-break-bench`, 1920 particles, 60+600 steps, `--release` vs `oracle-release`) and persist dated unprofiled wall ms, ms/step, Rust/C++ ratio, host, git HEAD, compilers under `target/dam-break-perf/<utc-stamp>/` — not stdout-only | Extend `tools/xtask/src/playground.rs` after both samples validate. Write `pair.json` + `pair.md` into a minted stamp. Keep stdout Markdown. Add ratio (missing from today’s table). |
| PERF-PROFILE | Thin `just` / xtask recipe rebuilds `[profile.profiling]` Dam Break binary (`inherits = "release"`, `debug = true`), captures samply 0.13.1 of the timed Rust loop, writes `rust.json.gz` plus identity into a new stamp without clobber. Profiled wall times are never the 3× number | Workspace profile + `dam-break-profile` wrapping the built `target/profiling/dam-break-bench` path. Fail closed if samply missing. Do not copy samply duration into `pair.json`. |
| PERF-TIMERS | Emit coarse parent-phase timers (`particle_prepare` / `particle_solve` / `rigid_solve` or equivalent existing `step_profiled` parents) on a separate Dam Break diagnostic path, absent from the unprofiled gate process | New native-only `dam-break-timers` binary. Do not call `step_profiled` from `dam-break-bench`. Write `timers.json` into a new stamp. |
</phase_requirements>

## Summary

Phase 22 is tooling only. The locked Dam Break pair already builds `oracle-release` `playground-dam-break-bench`, runs native `liquidfun-wasm` `--release` `dam-break-bench`, validates JSON samples, and prints an unreviewed Markdown table. It does **not** persist files, expose a Rust/C++ ratio, rebuild with debug info, invoke samply, or call `World::step_profiled`. This phase adds dated gitignored evidence, a dedicated Cargo profiling profile, a samply wrap of the existing Rust timer binary, and a sibling timer binary that uses the already-public Phase 12 parent vocabulary.

Do not edit particle/rigid kernels, default `--release`, `oracle-release` presets, `reference/performance/manifest.toml`, or the published `liquidfun` dependency graph. Prove the shell with fake cmake/samply; a live ~300× pair and a real Instruments session are not definition of done.

**Primary recommendation:** Split `playground.rs` before it crosses 628 lines; persist unprofiled pair reports into minted `target/dam-break-perf/<utc-stamp>/` stamps; add `[profile.profiling]` plus `dam-break-profile` / `dam-break-timers` xtask commands with thin `just` aliases; wrap the profiling-profile `dam-break-bench` with `samply record --save-only --unstable-presymbolicate`; put `step_profiled` only on a new native timer bin.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. [VERIFIED: glob `.cursor/rules/**/*` returned 0 files]

Honor instead:

- `AGENTS.md` Repo-Local Guidance: hobby scope (`PROJECT-SCOPE.md`), standing autonomous iteration, independent AI review (implementer must not approve own work). Package publication remains separately authorized.
- `AGENTS.bright-builds.md` + `standards/core/architecture.md`: functional core / imperative shell; parse CLI and stamps at the boundary.
- `standards/core/code-shape.md`: early returns; `maybe_` for `Option`; do not hide CMake/samply programs inside `just` strings; split files at ~628 lines (`floor(100 * tau)`). `tools/xtask/src/playground.rs` is already 540 lines.
- `standards/core/verification.md`: repo-native checks before commit (`cargo fmt` / clippy / focused tests, `bun scripts/bright-builds-check.ts all` when the managed checker is installed). `.planning/**` is parser-owned GSD content — never mdformat it.
- `standards/core/testing.md`: unit-test pure stamp/JSON/argv logic with Arrange / Act / Assert.
- `standards/languages/rust.md`: new modules use `foo.rs` plus `foo/`; `let...else` guards; no `unwrap()` in production paths.
- `standards-overrides.md`: no frontend work in this phase; hobby scope and independent AI review policies apply.

## Standard Stack

### Core

| Library / tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Existing `cargo xtask playground dam-break-bench` | repo | Unprofiled 3×-authority pair | Already locked recipe; only persistence is new. [VERIFIED: `tools/xtask/src/playground.rs`] |
| Workspace `[profile.profiling]` | Cargo stable custom profile | Release codegen + full debuginfo without mutating `--release` | Cargo: custom profiles need `inherits`; output dir is `target/<profile>/`. samply docs require release + debug info. [CITED: doc.rust-lang.org/stable/cargo/reference/profiles.html; github.com/mstange/samply README at samply-v0.13.1] |
| `samply` | **0.13.1** (`mstange/samply`, crates.io max) | Scripted CPU sampling to Firefox Profiler JSON | Locked pin. `--save-only`, `-o`, `--unstable-presymbolicate` exist in 0.13.1. Host already has `samply 0.13.1`. [VERIFIED: crates.io/crates/samply; local `samply --version`] |
| Existing `World::step_profiled` / `DiagnosticProfileParent` | `crates/liquidfun` | Coarse parent timings | Public parents already include `particle_prepare`, `particle_solve`, `rigid_solve`, plus `contact_update`, `continuous_solve`, `finalize`. [VERIFIED: `crates/liquidfun/src/world/observation/profile.rs`] |

### Supporting

| Library / tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `serde` / `serde_json` | workspace 1.0.228 / 1.0.150 | Parse bench JSON; write `pair.json`, `profile-identity.json`, `timers.json` | Already in **xtask only**. Do not add to `liquidfun`. |
| Existing fake cmake/git/ninja/cxx | `tools/xtask/tests/fixtures/fake_upstream_tool.rs` | Prove cmake argv without a live oracle | Extend with fake samply (+ optional fake cargo). |
| `LIQUIDFUN_XTASK_CMAKE` (and siblings) | existing env overrides | Inject fake tools in tests | Add `LIQUIDFUN_XTASK_SAMPLY` the same way. |
| Optional CMake cache `REFERENCE_PROFILE_DEBUG_INFO` | new cache var, **not** a new preset | Extra-target `-g` when `--cpp-debuginfo` is passed | Profile command only. Default profile path is Rust-only and must not reconfigure C++. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| samply 0.13.1 wrap | `cargo flamegraph` / `xcrun xctrace` | Locked out for the required path. xctrace is Phase-23-optional, needs full Xcode, and writes `.trace` (must not be committed). |
| `[profile.profiling] debug = true` | `debug = "limited"` / `"line-tables-only"` | Architecture research preferred `"limited"` (samply#763). **CONTEXT D-08 locks `debug = true`.** Do not substitute. |
| `CARGO_PROFILE_RELEASE_DEBUG=true` | named profile | Would silently change the 3× gate binary and CI artifacts. Forbidden. |
| `step_profiled` inside `dam-break-bench` | separate timer bin | Contaminates the gate process. Forbidden by D-10 / D-11. |
| In-process C ABI | keep process spawn | Isolation + crash isolation. Deferred. |
| feldera/samply 0.13.2 | official crates.io 0.13.1 | Different GitHub repo; not the pin. |

**Installation (developer host, not CI, not the published crate):**

```bash
# Pair (already in repo)
just playground-dam-break-bench

# CPU profiler pin
cargo install --locked samply --version 0.13.1
# or: brew install samply   # then still verify `samply --version` is 0.13.1
samply setup   # macOS attach/signing; required for --pid, not for launching a locally built bin
```

**Workspace addition (repo change):**

```toml
# root Cargo.toml — does not ship inside `cargo package -p liquidfun`
[profile.profiling]
inherits = "release"
debug = true
strip = false
```

**Version verification:** crates.io `samply` `newest_version` / `max_stable_version` = `0.13.1`, published 2025-02-01. Local host: `samply 0.13.1`. Cargo custom-profile docs current as of this research date. [VERIFIED: crates.io API; local `--version`; Cargo Book profiles page]

## Architecture Patterns

### Recommended Project Structure

```text
Cargo.toml                                 # ADD [profile.profiling]
justfile                                   # ADD two one-line aliases; keep playground-dam-break-bench
crates/liquidfun/                          # UNCHANGED deps and kernels
crates/liquidfun-wasm/
  src/dam_break_bench.rs                   # KEEP World::step Instant timer
  src/bin/dam_break_bench.rs               # KEEP --release gate argv
  src/dam_break_timers.rs                  # NEW native-only aggregation
  src/bin/dam_break_timers.rs              # NEW; calls step_profiled via SessionCore
  src/session.rs                           # ADD native-only advance_profiled(1) helper
tools/xtask/src/
  playground.rs                            # DISPATCH only (usage, command match)
  playground/
    stamp.rs                               # mint YYYY-MM-DDTHH-MM-SSZ; exclusive create
    pair.rs                                # existing pair + persist pair.json/pair.md
    profile.rs                             # samply wrap; profile-identity.json
    timers.rs                              # spawn dam-break-timers; write timers.json
    identity.rs                            # host/HEAD/compilers shared
tools/xtask/tests/
  playground_cli.rs                        # NEW fake-tool tests (pair persist, samply missing, stamp)
  fixtures/fake_upstream_tool.rs           # EXTEND fake samply (+ optional fake cargo)
tools/reference/CMakeLists.txt             # OPTIONAL: REFERENCE_PROFILE_DEBUG_INFO on extra target only
tools/reference/CMakePresets.json          # UNCHANGED — no new preset
target/dam-break-perf/<utc-stamp>/         # gitignored via /target/
  pair.json  pair.md                       # pair command only
  rust.json.gz  profile-identity.json      # profile command only
  timers.json                              # timer command only
```

Follow `foo.rs` plus `foo/` for the new xtask playground module. Do not add `playground/mod.rs`. [CITED: standards/languages/rust.md]

### Pattern 1: Thin just / xtask / process spawn

**What:** `just` prints one Cargo command. xtask owns argv, stamp mkdir, cmake, cargo, samply, and file writes. Physics stays in `liquidfun`. Sampling stays out of process.
**When to use:** All three recipes.
**Example:**

```just
playground-dam-break-bench:
    cargo xtask playground dam-break-bench

playground-dam-break-profile:
    cargo xtask playground dam-break-profile

playground-dam-break-timers:
    cargo xtask playground dam-break-timers
```

### Pattern 2: Exclusive stamp then write

**What:** Format UTC `YYYY-MM-DDTHH-MM-SSZ`. `create_dir` (exclusive). If `AlreadyExists`, bump the stamp (add one second, or retry with a later clock) and try again. Never `remove_dir_all` an existing stamp. Never write into a pre-existing directory.
**When to use:** Every pair, profile, and timer command. Each command mints its **own** new stamp (D-04: timers do not share the pair stamp).
**Why exclusive create:** `create_dir_all` on an existing path succeeds and would merge/overwrite. That violates D-01.

### Pattern 3: Sampler wraps a known binary path

**What:** `cargo build -p liquidfun-wasm --bin dam-break-bench --profile profiling`, then pass the built path to samply. Do **not** `samply record cargo run …` (hides the binary; can profile cargo).
**When to use:** `dam-break-profile` only.

```text
samply record --save-only --unstable-presymbolicate \
  -o target/dam-break-perf/<stamp>/rust.json.gz \
  -- ./target/profiling/dam-break-bench --warmup 60 --steps 600
```

Source: samply 0.13.1 README (release + debug info; record the binary) plus release notes for `--save-only` / `--unstable-presymbolicate`. [CITED: github.com/mstange/samply/blob/samply-v0.13.1/README.md; github.com/mstange/samply/releases/tag/samply-v0.13.1]

### Pattern 4: Separate diagnostic process for parent timers

**What:** `SessionCore::advance` always calls `World::step` (profiler disabled). Keep that for `dam-break-bench`. Add `advance_profiled` that calls `World::step_profiled` once, used only by `dam-break-timers`. Loop `advance(1)` / `advance_profiled(1)` — `MAX_ADVANCE_STEPS` is **4** (playground catch-up cap); do not pass 600 as a single `advance` count.
**When to use:** Timer command. Gate process stays on `World::step`.

### Anti-Patterns to Avoid

- **Hiding samply/CMake in just:** violates D-06/D-07 and repo layering.
- **Writing profiled `wall_ms` into `pair.json`:** violates D-03 and `timing_authority: unprofiled_wall_clock`.
- **Calling `step_profiled` from `dam-break-bench`:** violates D-10; Instant spans distort the 3× number.
- **New CMake preset (`oracle-relwithdebinfo`):** violates D-07. Optional `-g` is a cache flag on the existing `oracle-release` extra target.
- **Mutating default `[profile.release] debug`:** changes the gate binary.
- **Adding serde/samply/dhat to `liquidfun`:** fails `cargo xtask package verify` isolation (`FORBIDDEN_PREFIXES` / path terms). [VERIFIED: `tools/xtask/src/package.rs`]
- **Using research path `target/v12-native-perf/`:** CONTEXT/roadmap lock `target/dam-break-perf/`. Architecture.md is stale on the directory name.
- **Raising `MAX_ADVANCE_STEPS` or importing Instant into the cdylib:** WASM catch-up / Phase 25 pitfalls.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CPU sampling / flamegraph JSON | Custom interrupt sampler | Host `samply` 0.13.1 | Firefox Profiler format, macOS aarch64, `--save-only` |
| Parent-phase Instant spans | New profiler in `liquidfun` | Existing `World::step_profiled` | Schema and parents already tested in `phase12_profiles.rs` |
| Dam Break recipe / particle count | Duplicate scene in xtask | `liquidfun_wasm::dam_break_bench` + new timer bin | Locked 1920-particle constants already compared to C++ source |
| C++ oracle build | Shell cmake in just | `upstream::run` configure/build `--preset oracle-release --target playground-dam-break-bench` | Allowlisted target + fake-cmake tests exist |
| Package isolation | Ad-hoc grep | `cargo xtask package verify` | Archive deny list already forbids C++/tools/benchmark paths |
| gzip Firefox profile writer | Manual flate2 of invented stacks | samply `-o rust.json.gz` | Missing real samply must not stub this file |

**Key insight:** The physics and the Instant pair already exist. This phase is stamp + spawn + isolation. Hand-rolling a profiler or a second Dam Break scene would both miss the canary and risk contaminating the gate.

## Common Pitfalls

### Pitfall 1: Treating profiled or `step_profiled` times as the 3× number

**What goes wrong:** One command records samply and prints a ratio from that run. Gate “passes” while unprofiled `--release` is still ~300×.
**Why it happens:** Convenience. Debuginfo and Instant parents add overhead. Policy already forbids it (`BENCHMARKING.md`, `policy.json` `timing_authority: unprofiled_wall_clock`).
**How to avoid:** Three commands, three stamp kinds. `pair.json` includes `"kind": "unprofiled_pair"` and `"timing_authority": "unprofiled_wall_clock"`. `profile-identity.json` and `timers.json` include `"not_timing_authority": true`. Never copy samply duration into `pair.json`.
**Warning signs:** A single recipe that both profiles and reports the gate ratio; `CARGO_PROFILE_RELEASE_DEBUG` used as the pair.

### Pitfall 2: Overwriting an evidence stamp

**What goes wrong:** `create_dir_all` + write reuses `target/dam-break-perf/latest/` or the same second’s stamp.
**Why it happens:** Idempotent mkdir looks friendly.
**How to avoid:** Exclusive `create_dir`. On `AlreadyExists`, mint a new stamp. Never delete.
**Warning signs:** Tests that `rm -rf` the stamp to retry; a `latest` symlink.

### Pitfall 3: Profiling `cargo run` or the `--release` gate binary

**What goes wrong:** Flamegraph of cargo, or a gate binary without symbols, or a silently slower `--release` after `debug = true` leaks.
**Why it happens:** samply README’s shortest example is `samply record ./app`.
**How to avoid:** Named `[profile.profiling]`. `cargo build --profile profiling` then wrap `target/profiling/dam-break-bench`. Pair keeps `--release` / `cargo run --release`.
**Warning signs:** `[profile.release] debug = true`; samply argv starting with `cargo`.

### Pitfall 4: Silent skip when samply is missing

**What goes wrong:** CI or a laptop without samply “succeeds” with no `rust.json.gz`.
**Why it happens:** Host-specific macOS signing (`samply setup`) invites optional paths.
**How to avoid:** Production lookup: `LIQUIDFUN_XTASK_SAMPLY` else `PATH`. Missing or non-0.13.1 → error with install text. Do not write a placeholder gzip.
**Warning signs:** `if which samply { … } else { Ok(()) }`.

### Pitfall 5: Dirtying `oracle-release` with `-g` then pairing against that binary

**What goes wrong:** `--cpp-debuginfo` reconfigures `target/reference/oracle-release/` in place. Later `dam-break-bench` times a `-g` C++ binary (or mixed objects).
**Why it happens:** D-07 allows a profile-command cache flag on the existing preset, not a new binaryDir.
**How to avoid:** Default `dam-break-profile` is **Rust-only** and must not call cmake. `--cpp-debuginfo` is optional and not required for PERF-PROFILE. If implemented, pass `-DREFERENCE_PROFILE_DEBUG_INFO=ON` only from the profile command; apply `-g` only to `playground-dam-break-bench` in CMakeLists. Document that this dirties the shared `oracle-release` cache; the pair command must never pass the flag. Do not add a RelWithDebInfo preset.
**Warning signs:** New preset names; pair command growing `--cpp-debuginfo`.

### Pitfall 6: Wiring timers through `dam-break-bench` or `advance(600)`

**What goes wrong:** Gate process enables `DiagnosticStepProfiler`, or `SessionCore::advance(600)` is rejected because `MAX_ADVANCE_STEPS == 4`.
**Why it happens:** One binary looks simpler; the playground cap is easy to miss.
**How to avoid:** New `dam-break-timers` bin. Loop `advance_profiled(1)` 600 times after 60 ordinary warmup steps. Leave `dam_break_bench.rs` on `World::step`.
**Warning signs:** `step_profiled` in `dam_break_bench.rs`; raising `MAX_ADVANCE_STEPS` for native benches.

### Pitfall 7: Growing `playground.rs` past the file-length gate

**What goes wrong:** Bright Builds `file-lengths` fails at 629 physical lines. Current file is 540 lines; pair persist + profile + timers will blow it.
**Why it happens:** All playground logic lives in one file today.
**How to avoid:** Split on the first implementation plan using `playground.rs` + `playground/*.rs`.
**Warning signs:** A 700-line dispatcher; a TSV exception instead of a split.

### Pitfall 8: Refreshing public timing docs or the empty manifest

**What goes wrong:** Exploratory numbers become a Phase 12 claim.
**How to avoid:** D-14. Optional one-sentence BENCHMARKING.md note that the pair now also writes gitignored stamps is allowed; do not paste wall times into `manifest.toml` or treat `docs/playground-dam-break-timing.md` as a new public claim.

## Code Examples

Verified patterns from official sources and this checkout:

### Workspace profiling profile

```toml
# Source: https://doc.rust-lang.org/stable/cargo/reference/profiles.html
# Custom profiles require `inherits`. `--profile profiling` writes to target/profiling/.
# debug = true is full debuginfo (same as 2 / "full"), matching CONTEXT D-08.
[profile.profiling]
inherits = "release"
debug = true
strip = false
```

### samply save-only wrap (production argv)

```text
# Source: samply 0.13.1 README + 0.13.1 release notes
# Do not omit --save-only (would open a local Firefox Profiler server).
# --unstable-presymbolicate embeds names so rust.json.gz is readable later without this machine's symbol server.
samply record --save-only --unstable-presymbolicate \
  -o target/dam-break-perf/2026-09-20T22-34-51Z/rust.json.gz \
  -- target/profiling/dam-break-bench --warmup 60 --steps 600
```

### Existing disabled vs enabled profiler (do not change)

```rust
// Source: crates/liquidfun/src/world/step/execution.rs
pub fn step<H: CollisionDecisionHook>(...) -> Result<StepReport, StepError> {
    let mut profiler = DiagnosticStepProfiler::disabled();
    self.step_internal(configuration, hook, limits, &mut profiler)
}

pub fn step_profiled<H: CollisionDecisionHook>(
    ...
) -> Result<(StepReport, DiagnosticStepProfile), StepError> {
    let mut profiler = DiagnosticStepProfiler::enabled();
    let report = self.step_internal(configuration, hook, limits, &mut profiler)?;
    Ok((report, profiler.finish()))
}
```

### Parent tokens to emit (timers.json)

```rust
// Source: crates/liquidfun/src/world/observation/profile.rs
// DiagnosticProfileParent::ALL order:
// contact_update, rigid_solve, continuous_solve, particle_prepare, particle_solve, finalize
```

Empty particle-system worlds omit `particle_prepare`; Dam Break Medium has particles so the timer test/bin must assert `particle_prepare`, `particle_solve`, and `rigid_solve` appear. [VERIFIED: `crates/liquidfun/tests/phase12_profiles.rs`]

### Recommended pair.json shape (discretion)

Required fields from PERF-PAIR / D-02; names beyond that are discretion:

```json
{
  "kind": "unprofiled_pair",
  "timing_authority": "unprofiled_wall_clock",
  "disclaimer": "Unreviewed local playground Dam Break sample. Not a public performance claim and not Phase 12 evidence.",
  "stamp": "2026-09-20T22-34-51Z",
  "git_head": "abc123",
  "os": "macos",
  "arch": "aarch64",
  "cpu_brand": "Apple M4 Max",
  "logical_cores": 16,
  "particles": 1920,
  "warmup_steps": 60,
  "measured_steps": 600,
  "rust": {
    "engine": "native_rust",
    "wall_ms": 70594.221,
    "ms_per_step": 117.657,
    "steps_per_s": 8.499,
    "realtime_factor": 0.142,
    "compiler": "rustc 1.97.0"
  },
  "cpp": {
    "engine": "pinned_cpp",
    "wall_ms": 232.439,
    "ms_per_step": 0.387,
    "steps_per_s": 2581.320,
    "realtime_factor": 43.022,
    "compiler": "AppleClang 21.0.0.21000334"
  },
  "rust_over_cpp_ratio": 303.71
}
```

`rust_over_cpp_ratio` = `rust.wall_ms / cpp.wall_ms`. Today’s Markdown table **does not** include this ratio — add a table column or a bullet so `pair.md` matches stdout and satisfies PERF-PAIR. Guard C++ `wall_ms == 0` as a bench error, not Inf.

### Recommended profile-identity.json shape (discretion)

```json
{
  "kind": "samply_cpu",
  "not_timing_authority": true,
  "cargo_profile": "profiling",
  "samply_version": "0.13.1",
  "git_head": "abc123",
  "os": "macos",
  "arch": "aarch64",
  "compiler": "rustc 1.97.0",
  "binary": "target/profiling/dam-break-bench",
  "output": "rust.json.gz",
  "command": ["samply", "record", "--save-only", "--unstable-presymbolicate", "-o", ".../rust.json.gz", "--", ".../dam-break-bench", "--warmup", "60", "--steps", "600"],
  "warmup_steps": 60,
  "measured_steps": 600
}
```

### Missing-samply stderr (discretion, fail closed)

Include all of: pin `0.13.1`; `cargo install --locked samply --version 0.13.1`; `brew install samply` as an alternate; `samply setup` on macOS; do not mention skipping. Exit nonzero. Write no `rust.json.gz`.

### Fake-tool injection (existing pattern to copy)

```rust
// Source: tools/xtask/tests/upstream_cli.rs
command
    .env("LIQUIDFUN_XTASK_GIT", &tools.git)
    .env("LIQUIDFUN_XTASK_CMAKE", &tools.cmake)
    .env("LIQUIDFUN_XTASK_NINJA", &tools.ninja)
    .env("LIQUIDFUN_XTASK_CXX", &tools.cxx);
```

Add `LIQUIDFUN_XTASK_SAMPLY`. Fake samply `--version` prints `samply 0.13.1`. Fake `record --save-only -o <path>` writes a nonempty gzip (or any bytes) to that path and exits 0. Production path must not use the fake.

Pair/profile tests that would otherwise invoke real cmake should keep using the existing fixture root + fake cmake. Profile tests that would otherwise `cargo build --profile profiling` should inject `CARGO` / a fake cargo that records `--profile profiling` and leaves a dummy `dam-break-bench` executable, **or** unit-test argv builders as a pure core and only integration-test the samply spawn. Prefer both: pure argv/stamp tests plus one CLI test with fakes.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Stdout-only Dam Break Markdown | Dated `pair.json` + `pair.md` under gitignore | This phase | PERF-PAIR; still unreviewed |
| Ad hoc `CARGO_PROFILE_RELEASE_DEBUG` | Named `[profile.profiling]` | Cargo custom profiles (stable) | Gate `--release` unchanged |
| DTrace `cargo flamegraph` on Apple Silicon | samply 0.13.1 `--save-only` | samply 0.13.1 (2025-02-01) | Native aarch64; no sudo DTrace |
| Phase 12 sealed 32-case public matrix | Empty `reviewed_reports`; playground pair is the canary | Hobby scope 2026-09-16 | Do not fill `manifest.toml` |
| Architecture research `target/v12-native-perf/` | `target/dam-break-perf/` | v1.2 roadmap / CONTEXT D-01 | Avoid Phase 12 name collision |

**Deprecated/outdated:**

- `target/v12-native-perf/` as the evidence root — superseded by D-01.
- Architecture.md Pattern 2 `debug = "limited"` — superseded by D-08 `debug = true`.
- Profiling through `cargo run` as the scripted path — samply should launch the built unsigned binary.
- feldera/samply 0.13.2 — not crates.io `samply`.
- In-process C ABI as the first profiling path — deferred; both timers are already local processes around `Step`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Wrapping the existing `dam-break-bench` process (construction + 60 warmup + 600 timed steps) is an acceptable way to “capture the timed Rust loop” for a ~300× kernel hunt, because warmup exercises the same `World::step` hot path and construction is outside `Instant` | Profile capture / discretion | If Phase 23 needs a clean time-range with zero warmup samples, a later `--pid` attach after warmup (needs `samply setup`) or a handshake flag would be required. Not needed to name 300× functions. |

No other `[ASSUMED]` product/compliance claims. Versions, file seams, and samply flags were verified or cited.

## Open Questions

1. **Should `dam-break-profile` rebuild C++ at all?**
   - What we know: PERF-PROFILE is a **Rust** samply of the timed Rust loop. C++ `-g` is optional and must not be a new preset.
   - What's unclear: Whether anyone needs `cpp.json.gz` in Phase 22.
   - Recommendation: Rust-only default. Implement `--cpp-debuginfo` only as a cache flag + optional C++ wrap if it stays behind that flag; it is not a success criterion.

2. **How strictly to exclude warmup from samply samples?**
   - What we know: D-08 wants construction/warmup outside the sampled timed loop. The binary already excludes them from `Instant`. samply records the whole process unless we attach after warmup.
   - What's unclear: Whether attach (`samply record -p/--pid`) is reliable on this Mac without `samply setup`.
   - Recommendation: Wrap the binary (A1). Document warmup as ~9% of step samples. Do not block the phase on attach/signing.

3. **Should profile/timer commands accept `--warmup` / `--steps`?**
   - What we know: Pair already does; defaults 60 / 600.
   - Recommendation: Yes, same flags and defaults (discretion). Tests can use `--steps 1` with fakes.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust / cargo | All recipes | ✓ | rustc 1.97.0 / cargo 1.97.0 | — |
| samply | PERF-PROFILE live run | ✓ on this host | 0.13.1 | Tests use fake samply; missing production samply fails closed |
| cmake | PERF-PAIR live run | ✓ | 3.27.9 (floor 3.25; CI pin 4.3.3 warns) | Fake cmake in xtask tests; live pair not DoD |
| ninja | Live C++ extra target | ✓ | 1.13.2 | Fake ninja in tests |
| just | Discoverable aliases | ✓ | 1.48.0 | `cargo xtask playground …` is the real command |
| bun + Bright Builds checker | file-lengths / `all` | ✓ bun 1.4.2 | Run `bun scripts/bright-builds-check.ts all` after the playground split |
| Full Xcode / xctrace | Optional Instruments | not required | — | Out of scope |
| dhat | PERF-HEAP | not this phase | — | Phase 23 |

**Missing dependencies with no fallback:** none for planned fake-tool DoD.

**Missing dependencies with fallback:** live Dam Break C++ pair and real samply session — optional developer proof; tests must not require them (D-13).

Step 2.6 is **not** skipped: profile/timer/pair tooling depends on cargo, optional cmake, and host samply.

## Security Domain

`security_enforcement` is not set to `false` in `.planning/config.json` (absent = enabled). This phase is local developer CLI tooling, not an authenticated service. [VERIFIED: `.planning/config.json`]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | No accounts |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | Local developer machine |
| V5 Input Validation | yes | Parse `--warmup` / `--steps` as `u32`; reject unknown flags; stamp is generated, not taken from argv; Command argv lists (no `sh -c`) |
| V6 Cryptography | no | No new crypto; do not treat profiles as integrity evidence |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Stamp path traversal | Tampering | Generate `YYYY-MM-DDTHH-MM-SSZ` only; join under `target/dam-break-perf/`; reject `..` if any override is added later |
| Command injection via samply/cmake path | Execution | `Command::new(program).args([...])`; env override `LIQUIDFUN_XTASK_SAMPLY` is a path, not a shell line |
| Placeholder profile faking a successful hunt | Spoofing | Missing samply fails closed; no stub `rust.json.gz` |
| Leaking host dumps into git | Information disclosure | `/target/` already gitignored; do not `git add` `.json.gz` / `.trace` |
| Profiler crate in published archive | Elevation of privilege / supply | `liquidfun` stays `bitflags` only; `package verify` deny lists |

## Sources

### Primary (HIGH confidence)

- This checkout: `tools/xtask/src/playground.rs`, `upstream.rs`, `package.rs`, `justfile`, `Cargo.toml`, `crates/liquidfun-wasm/src/{dam_break_bench.rs,bin/dam_break_bench.rs,session.rs,lib.rs}`, `crates/liquidfun/src/world/{step/execution.rs,observation/profile.rs}`, `crates/liquidfun/tests/phase12_profiles.rs`, `tools/xtask/tests/upstream_cli.rs`, `tools/xtask/tests/fixtures/fake_upstream_tool.rs`, `tools/reference/CMakeLists.txt`, `CMakePresets.json`, `BENCHMARKING.md`, `docs/playground-dam-break-timing.md`, `reference/performance/{policy.json,manifest.toml}`, `.gitignore`
- [Cargo Book — Profiles](https://doc.rust-lang.org/stable/cargo/reference/profiles.html) — custom `inherits`, `debug = true`, `target/<profile>/`
- [samply 0.13.1 README](https://github.com/mstange/samply/blob/samply-v0.13.1/README.md) — release + debug info; record locally built binaries; `samply setup` for attach
- [samply 0.13.1 release notes](https://github.com/mstange/samply/releases/tag/samply-v0.13.1) — `--save-only`, `--unstable-presymbolicate`, macOS attach
- [crates.io samply](https://crates.io/crates/samply) — max/newest 0.13.1 (2025-02-01)
- [The Rust Performance Book — Profiling](https://nnethercote.github.io/perf-book/profiling.html) — samply; debuginfo; do not put global `RUSTFLAGS` frame pointers on the gate profile
- `.planning/{REQUIREMENTS.md,ROADMAP.md,PROJECT.md,STATE.md,config.json}`, `PROJECT-SCOPE.md`, CONTEXT D-01–D-14

### Secondary (MEDIUM confidence)

- `.planning/research/{ARCHITECTURE.md,STACK.md,FEATURES.md,PITFALLS.md,SUMMARY.md}` — correct on isolation and samply pin; stale on `target/v12-native-perf/` and `debug = "limited"`
- samply#739 / #763 — presymbolicate for portable names; `"limited"` debuginfo can suffice (not used; D-08 locks `true`)

### Tertiary (LOW confidence)

- Attach-after-warmup via `samply record --pid` as a stricter timed-loop targeting strategy — works on macOS after `samply setup`, but is not required for DoD (see A1)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — locked pins verified on crates.io, Cargo Book, and this host
- Architecture: HIGH — seams read from disk; split layout forced by file-length + existing just/xtask split
- Pitfalls: HIGH — policy pitfalls verified in-repo; cmake cache contamination inferred from shared `oracle-release` `binaryDir`

**Research date:** 2026-09-20
**Valid until:** 2026-10-20 (samply pin is stable; Cargo profile docs are stable)

## Planner implementation notes (non-goals)

Wave-shaped suggestion, not a plan file:

1. **Stamp + pair persist + just unchanged name** — split playground module; mint stamp; write `pair.json`/`pair.md` including ratio; keep stdout; unit-test stamp collision and JSON shape with fakes (no live pair).
2. **`[profile.profiling]` + `dam-break-profile`** — locate samply; fail closed; wrap profiling-profile binary; write `rust.json.gz` + `profile-identity.json`; fake samply CLI test; `package verify` still green.
3. **`dam-break-timers`** — new native bin + `advance_profiled(1)` loop; `timers.json` in a new stamp; grep-test that `dam_break_bench` still has no `step_profiled`.
4. **Isolation/docs** — `cargo xtask package verify`; `bun scripts/bright-builds-check.ts all`; optional BENCHMARKING.md one-liner that pair output is now also a gitignored stamp; do not refresh playground timing numbers; do not touch `manifest.toml`.

Independent AI review remains required under owner policy; the implementing agent must not approve its own work. Live `just playground-dam-break-bench` on this Mac is optional proof, not the phase gate.
