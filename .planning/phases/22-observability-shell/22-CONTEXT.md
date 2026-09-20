---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-20T22:34:51.768Z
---

# Phase 22: Observability shell - Context

**Gathered:** 2026-09-20
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

A developer can re-run the locked Dam Break Medium pair into a dated gitignored evidence directory and capture a symbolicated samply CPU profile (plus optional parent-phase timers) without changing physics or the `--release` gate binary.

This phase delivers tooling only: persist unprofiled pair reports, add a `[profile.profiling]` samply capture, and expose a separate `step_profiled` timer path. It does not close the 3× gate, name dominating functions, run dhat, edit particle/rigid kernels, refresh public claims, or compare WASM to C++.

</domain>

<decisions>
## Implementation Decisions

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
- **D-08:** Add a workspace `[profile.profiling]` that `inherits = "release"` and sets `debug = true`. The profile recipe rebuilds the existing Dam Break Rust binary with that profile and captures samply **0.13.1** of the timed Rust loop only. Construction, insertion, warm-up, capture, and rendering stay outside the sampled timed loop, same as the unprofiled pair.
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked milestone policy
- `.planning/ROADMAP.md` — Phase 22 goal, success criteria, no-physics rule, fake cmake/samply allowance, just/xtask split, optional C++ `-g` restriction.
- `.planning/REQUIREMENTS.md` — `PERF-PAIR`, `PERF-PROFILE`, `PERF-TIMERS` (this phase); `PERF-AUDIT` / `PERF-HEAP` are Phase 23.
- `.planning/PROJECT.md` — Native-vs-C++ Dam Break canary, scalar baseline, honest claims, no package publication.
- `.planning/STATE.md` — Unprofiled `just playground-dam-break-bench` is 3× authority; samply / profiling / `step_profiled` / dhat timings are never that number.
- `PROJECT-SCOPE.md` — Hobby scope; local checks plus one macOS Cargo CI job; no dedicated performance-host gate.

### Measurement and isolation contracts
- `BENCHMARKING.md` — Exploratory Dam Break pair is unreviewed local diagnosis; unprofiled wall-clock is timing authority.
- `docs/playground-dam-break-timing.md` — Locked Medium recipe (1920 particles, 60+600 steps); do not treat the sample as a public claim.
- `.planning/research/PITFALLS.md` — Pitfall 4 (profiled timings as 3×), pitfall 5 (debug vs release), pitfall 6 (WASM vs C++).
- `.planning/research/ARCHITECTURE.md` — Observability shell build order, isolation of `liquidfun`, `just playground-dam-break-profile` → xtask, no in-process C ABI.
- `.planning/research/FEATURES.md` — PERF-PAIR / PERF-PROFILE seeds; samply 0.13.1; gitignored dated evidence.
- `reference/performance/policy.json` — `timing_authority: unprofiled_wall_clock`.
- `reference/performance/manifest.toml` — Must stay empty.

### Existing implementation seams
- `justfile` — Current one-line `playground-dam-break-bench` alias.
- `tools/xtask/src/playground.rs` — Existing pair orchestration, `--warmup` / `--steps`, Markdown stdout, `oracle-release` extra target.
- `tools/xtask/src/upstream.rs` — Registered `playground-dam-break-bench` C++ target.
- `tools/xtask/tests/upstream_cli/build.rs` — Fake-cmake registration test pattern.
- `crates/liquidfun-wasm/src/dam_break_bench.rs` and `crates/liquidfun-wasm/src/bin/dam_break_bench.rs` — Unprofiled native `World::step` timer; do not add `step_profiled` here.
- `crates/liquidfun/src/world/step/execution.rs` — Ordinary `step` vs `step_profiled`.
- `crates/liquidfun/src/world/observation/profile.rs` — Existing parent tokens including `particle_prepare`, `particle_solve`, `rigid_solve`.
- `crates/liquidfun/tests/phase12_profiles.rs` — Existing `step_profiled` parent coverage.
- `Cargo.toml` — Workspace members, `unsafe_code = "forbid"`, no `[profile.profiling]` yet.
- `.gitignore` — `/target/` already covers dated evidence.

### Inherited phase context
- `.planning/phases/11-examples-headless-tooling-and-testbed/11-CONTEXT.md` — Wall-clock profiles are diagnostic, not D0/D1 parity.
- `.planning/phases/12-performance-portability-and-release-hardening/12-CONTEXT.md` — Unprofiled wall-clock totals are authoritative; profile durations are diagnostics.
- `.planning/phases/21-playground-leftover-cleanup/21-CONTEXT.md` — Keep Dam Break C++ bench as fake-cmake registration; live pair was out of v1.1 DoD.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cargo xtask playground dam-break-bench` already builds `oracle-release` + native `--release` `dam-break-bench`, validates JSON samples, and prints a Markdown table with host/HEAD/compilers.
- `liquidfun_wasm::dam_break_bench` owns the locked 1920-particle Medium recipe and times ordinary `World::step`.
- `World::step_profiled` and `DiagnosticProfileParent` already expose `particle_prepare` / `particle_solve` / `rigid_solve`.
- xtask tests already compile fake cmake/ninja/git tools under `tools/xtask/tests/upstream_cli.rs`.

### Established Patterns
- `just` is a one-line alias; xtask owns orchestration (`just playground-dam-break-bench` → `cargo xtask playground dam-break-bench`).
- Pair JSON is parsed from bench stdout; construction/warm-up stay outside `Instant`.
- Package isolation: `liquidfun` is the sole default member and must not grow tooling deps.
- Evidence that must not be a public claim stays gitignored or labeled unreviewed.

### Integration Points
- Persist from `tools/xtask/src/playground.rs` after both samples validate — that is the pair-command write.
- Add sibling playground subcommands `dam-break-profile` and `dam-break-timers` next to `dam-break-bench`.
- Add `[profile.profiling]` to the workspace `Cargo.toml`.
- Keep C++ extra-target registration in `tools/xtask/src/upstream.rs`; optional `-g` is a profile-command cache flag, not a new preset.
- Do not wire timers through `crates/liquidfun-wasm/src/bin/dam_break_bench.rs`.

</code_context>

<specifics>
## Specific Ideas

- Research named `target/v12-native-perf/<UTC>/`; the locked v1.2 path is `target/dam-break-perf/<utc-stamp>/` — follow the roadmap, not the older research name.
- Tooling can be proven with fake cmake/samply; a live ~300× pair is not required to complete this phase.
- samply setup / signing on this Mac is host-specific; missing-tool failure text is part of the product.

</specifics>

<deferred>
## Deferred Ideas

- Named dominating-function audit and `docs/native-performance-audit.md` — Phase 23 (`PERF-AUDIT`).
- Refresh `docs/playground-dam-break-timing.md` as an unreviewed sample bound to a SHA — Phase 23.
- Private dhat heap dump when samply shows allocator/`Vec` time — Phase 23 (`PERF-HEAP`).
- Shared hot-path physics edits and the ≤ 3× gate — Phase 24.
- WASM/playground sanity versus previous WASM or native Rust, never versus C++ — Phase 25.
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in after the scalar canary.
- In-process C ABI / FFI profiling — not this milestone’s first path.

</deferred>

---

*Phase: 22-observability-shell*
*Context gathered: 2026-09-20*
