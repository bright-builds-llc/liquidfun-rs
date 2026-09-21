# Phase 23: Baseline pair and named audit - Research

**Researched:** 2026-09-21
**Domain:** SHA-bound Dam Break pair + samply named-function audit, optional private dhat heap dump
**Confidence:** HIGH for tooling seams, isolation, honesty rules, and locked stack pins; MEDIUM for which in-tree hunt-list symbol will dominate the live profile and for the exact samply `.syms.json` sidecar filename until the first live stamp is listed

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### SHA-bound live evidence
- **D-01:** Phase 23 definition of done includes a live unprofiled Dam Break Medium pair (`just playground-dam-break-bench`) and a live samply `rust.json.gz` (`just playground-dam-break-profile`) whose recorded git HEAD matches the HEAD cited in `docs/native-performance-audit.md`. Fake cmake/samply fixtures may still cover new xtask plumbing tests; they cannot name dominating functions.
- **D-02:** Do not invent hot-function names from the in-tree hunt list. Full-world clone, release invariants, per-pass `to_vec`, and `ParticleNeighborhood::from_view` stay unranked suspects until the live profile ranks them as named causes or as explicit “not found.”
- **D-03:** Only `pair.json` unprofiled wall times are the Dam Break ratio in committed notes. Discard samply duration, `[profile.profiling]` wall, `step_profiled` timers, and dhat overhead. Do not quote those as the 3× number.

### Audit-bundle stamp
- **D-04:** Phase 22 writes pair and profile into separate exclusive stamps. Phase 23 mints a **new** exclusive `target/dam-break-perf/<utc-stamp>/` that **copies** (never moves, merges, or overwrites) `pair.json` / `pair.md` and `rust.json.gz` from same-HEAD source stamps, plus an identity sidecar that records source stamp names, git HEAD, and `not_timing_authority` for the profile blob.
- **D-05:** Keep `just` as a one-line printer. Add a thin `just playground-dam-break-audit-bundle` → `cargo xtask playground dam-break-audit-bundle` (exact subcommand name is discretion) that fails closed unless both source stamps exist, share the same HEAD, and contain the expected files. Do not hide CMake or samply flags in `just`.
- **D-06:** The committed audit must name that bundle stamp path so a developer can point at one dated directory containing the unprofiled pair report and the samply `rust.json.gz` used to write the names.

### Named-function ranking and cause taxonomy
- **D-07:** Create committed `docs/native-performance-audit.md` that names dominating Rust functions with approximate samply shares, classifies each into extra per-particle work / per-step allocation / checks that survive `--release` / algorithm/shape differences, records the unprofiled Dam Break Medium wall-time ratio at the cited HEAD, and has an explicit “Not found” section so SIMD is not the first move.
- **D-08:** Name C++ counterparts only when extra work is a shape mismatch, by source comparison to pinned LiquidFun — not by requiring a C++ samply wrap in this phase.
- **D-09:** Banner the audit as an unreviewed local sample. Do not paste flamegraphs, `.json.gz`, or `.trace` into git. Do not copy the ratio into `reference/performance/manifest.toml`. Do not write “Rust is N× slower” as a product claim.
- **D-10:** No physics kernel edits in this phase. Phase 24 consumes the named shares.

### Heap dump gating
- **D-11:** Add a private optional `dhat-heap` feature on `liquidfun-wasm`’s `dam-break-bench` binary only (`dhat` 0.3.3, `#[global_allocator]`). Never a default feature. Never a `liquidfun` dependency. The unprofiled `--release` gate binary stays allocator-uninstrumented.
- **D-12:** Run private dhat only after the samply profile used for the audit shows allocator or `Vec` time (stacks involving `alloc::`, `__rust_alloc`, `Vec`, `RawVec`, `clone`, `to_vec`, `GlobalAlloc`, or equivalent). Write the dump under the gitignored evidence root in a new exclusive stamp (or the audit bundle) with `not_timing_authority`. Add a thin `just playground-dam-break-heap` → xtask alias; `just` stays a one-line printer.
- **D-13:** If samply does not show allocator/`Vec` time, skip the heap run and record that skip in `docs/native-performance-audit.md`. Do not run dhat “just in case.” Prefer `xctrace` Allocations only if dhat is blocked on this host; do not make Instruments the scripted default.

### Timing-doc refresh
- **D-14:** Refresh `docs/playground-dam-break-timing.md` with the SHA-bound unprofiled pair from D-01 as the current recorded sample. Keep the “Unreviewed local sample” banner, locked Medium recipe, and reproduce command. Cite the pair (or audit-bundle) stamp path. Do not treat the refresh as a Phase 12 public claim.

### Isolation and review
- **D-15:** `liquidfun` gains no profiler, samply, dhat, CMake, or serde dependency. `cargo xtask package verify` still passes. Raw profile blobs stay gitignored under `target/`.
- **D-16:** Independent AI review remains required after verification. The implementing agent must not self-approve the phase.

### Claude's Discretion
- Exact xtask subcommand and identity JSON field names for the audit bundle and heap stamp.
- Exact samply-symbol heuristic details for allocator/`Vec` detection, provided D-12/D-13 remain fail-closed.
- Whether heap dumps land in the audit-bundle stamp or a sibling exclusive stamp.
- How many named stacks to list beyond dominating shares, as long as “Not found” is explicit.
- Whether to keep a one-line historical pointer to the first `1e5cbcc…` exploratory sample after the timing-doc refresh.

### Deferred Ideas (OUT OF SCOPE)
- Shared hot-path physics edits and the ≤ 3× gate — Phase 24.
- Spot-checks of other playground scenes and second-canary-or-same-cluster note — Phase 24.
- WASM/playground sanity versus previous WASM or native Rust, never versus C++ — Phase 25.
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in after the scalar canary.
- Required C++ samply wrap / `cpp.json.gz` — not needed to name Rust extra work; optional later if a shape mismatch needs a C++ flame.
- Criterion micros of a named kernel — only after the pair is already near 3× (Phase 24 note).
- Filling `reference/performance/manifest.toml` or writing README speed claims — out of this milestone.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERF-AUDIT | Developer can read committed audit notes that name dominating Rust functions (and C++ counterparts when extra work is a shape mismatch), classify suspected causes (extra per-particle work, per-step allocation, checks that survive `--release`, algorithm/shape differences), record the current unprofiled Dam Break Medium wall-time ratio, and state what was not found so SIMD is not the first move | Live `just playground-dam-break-bench` + `just playground-dam-break-profile` at one SHA; new exclusive audit-bundle stamp that copies `pair.json`/`pair.md`/`rust.json.gz`; committed `docs/native-performance-audit.md` plus refreshed `docs/playground-dam-break-timing.md`. Fake fixtures cover copy/fail-closed plumbing only. |
| PERF-HEAP | Developer can run a private heap profile (`dhat` on the `dam-break-bench` binary only) after samply shows allocator or `Vec` time, writing the dump under the same gitignored evidence root. If samply does not show allocator time, committed notes record that and skip the heap run | Optional `dhat-heap` on `liquidfun-wasm` `dam-break-bench` only (`dhat` 0.3.3). xtask scans presymbolicated strings, fail-closed. Run dhat only on a match; otherwise skip and write that skip into the audit. Never add dhat to `liquidfun` or the default `--release` gate binary. |
</phase_requirements>

## Summary

Phase 23 is a **measure-and-name** phase, not an optimize phase. Phase 22 already persists unprofiled pairs and sibling samply profiles into exclusive `target/dam-break-perf/<utc-stamp>/` stamps. This phase (1) runs those recipes **live** at a recorded HEAD, (2) copies same-HEAD pair + profile artifacts into a **new** exclusive audit-bundle stamp, (3) writes committed named-function notes from that evidence, (4) optionally runs private `dhat` on `dam-break-bench` only when the profile shows allocator/`Vec` time, and (5) refreshes the timing doc as an unreviewed local sample.

Unlike Phase 22, fake cmake/samply fixtures **cannot** close PERF-AUDIT. They still cover new xtask copy/heuristic tests. The hunt list in `execution.rs` / `proxy.rs` / `pressure.rs` / `runtime.rs` is a reading map, not a ranked cause list to paste into git.

**Primary recommendation:** Add `dam-break-audit-bundle` and `dam-break-heap` as new playground modules (do not grow `pair.rs` or `playground_cli.rs` past 628 lines); copy same-HEAD pair+profile into a new exclusive stamp; run live pair+samply; write `docs/native-performance-audit.md` from those names; enable private `dhat-heap` on the wasm bench **bin** only after a fail-closed symbol match; keep `liquidfun` bitflags-only.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. [VERIFIED: glob `.cursor/rules/**/*` returned 0 files]

Honor instead:

- `AGENTS.md` Repo-Local Guidance: hobby scope (`PROJECT-SCOPE.md`), standing autonomous iteration, independent AI review (implementer must not approve own work). Package publication remains separately authorized.
- `AGENTS.bright-builds.md` + `standards/core/architecture.md`: functional core / imperative shell; parse CLI and stamps at the boundary. xtask orchestrates; `just` prints one line.
- `standards/core/code-shape.md` + Bright Builds file-lengths: 628 physical lines (`floor(100 * tau)`). Live counts: `pair.rs` 616, `playground_cli.rs` 622, `profile.rs` 529. New work goes in new `foo.rs` modules / a split CLI test, not appended onto those files.
- `standards/core/verification.md`: `cargo fmt` / clippy / focused tests, `bun scripts/bright-builds-check.ts all`, `just markdown-check` after non-GSD Markdown. `.planning/**` is parser-owned — never mdformat it.
- `standards/core/testing.md`: unit-test pure stamp/copy/heuristic logic; one concern per test; Arrange/Act/Assert.
- `standards/languages/rust.md`: `foo.rs` plus `foo/`; `let...else`; `maybe_` for `Option`; no `unwrap()` in production paths.
- `standards-overrides.md`: hobby scope and independent AI review remain in force. No override authorizes physics edits or filling `manifest.toml`.
- Workspace `unsafe_code = "forbid"` and `crates/liquidfun-wasm/src/lib.rs` `#![forbid(unsafe_code)]`. `dhat`’s unsafe stays inside the `dhat` crate. Do not put `unsafe` in `liquidfun` or the wasm crate.

## Standard Stack

### Core

| Library / tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Existing `just playground-dam-break-bench` | Phase 22 | Unprofiled 3×-authority pair | Locked recipe: 1920 particles, 60+600 steps, `--release` vs `oracle-release`. [VERIFIED: `justfile`, `tools/xtask/src/playground/pair.rs`] |
| Existing `just playground-dam-break-profile` | Phase 22 | samply 0.13.1 of `[profile.profiling]` `dam-break-bench` | Writes sibling stamp `rust.json.gz` + `profile-identity.json` with `not_timing_authority: true`. [VERIFIED: `profile.rs`] |
| Exclusive stamp mint | Phase 22 `stamp.rs` | Fail-closed `create_dir`, bump 1s, cap 8 | Reuse for audit-bundle and heap stamps. Never `remove_dir_all` or merge. [VERIFIED: `stamp.rs`] |
| `samply` | **0.13.1** | CPU samples used to name functions | Host has `samply 0.13.1`. `--unstable-presymbolicate` is already on the record argv. [VERIFIED: `samply --version`; [CITED: github.com/mstange/samply/releases/tag/samply-v0.13.1]] |
| `dhat` | **0.3.3** | Private heap ranking on `dam-break-bench` only | Still latest on crates.io (published 2024-02-04). Experimental / lightly maintained — one-off diagnostic. [VERIFIED: crates.io/crates/dhat; docs.rs/dhat/0.3.3] |
| `flate2` | workspace 1.1 | Decompress `rust.json.gz` for the heap heuristic | Already an xtask dependency. Do not add another gzip crate. [VERIFIED: `tools/xtask/Cargo.toml`, `Cargo.lock`] |
| `serde` / `serde_json` | workspace 1.0.228 / 1.0.150 | pair.json, identity sidecars | Already used by playground xtask. Do not add serde to `liquidfun`. [VERIFIED: workspace `Cargo.toml`] |

### Supporting

| Library / tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `dhat::Profiler::builder().file_name(path)` | 0.3.3 | Write `dhat-heap.json` into the minted stamp | Heap command only. Default `new_heap()` writes `dhat-heap.json` in cwd — too easy to drop in the repo root. [CITED: docs.rs/dhat/0.3.3/dhat/struct.ProfilerBuilder.html] |
| `xcrun xctrace` Allocations | Xcode 27 on this host | Fallback heap trace | Only if dhat hangs/crashes. Not the scripted default (D-13). [VERIFIED: `xcrun xctrace version`] |
| `samply load <stamp>/rust.json.gz` | 0.13.1 | Human flame for approximate shares | Document in the audit. Do **not** hide this in `just` (no samply flags in just). [CITED: samply README / main.rs `with_extension("syms.json")`] |
| Existing `World::step_profiled` timers | Phase 22 | Coarse parents | Optional reading aid. Never the Dam Break ratio (D-03). Do not re-run as a gate. |
| `cargo xtask package verify` + `cargo tree -p liquidfun --edges normal` | existing | Isolation | Must stay `liquidfun` → `bitflags` only. [VERIFIED: `22-06-SUMMARY.md`, `crates/liquidfun/Cargo.toml`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Private `dhat` 0.3.3 | `xcrun xctrace --instrument Allocations` | Locked fallback only if dhat is blocked. xctrace needs full Xcode and produces `.trace` (still gitignored). Do not script it as default. |
| Private `dhat` 0.3.3 | Valgrind DHAT / iai-callgrind | Not a macOS aarch64 hobby tool. Skip. |
| Copy-only audit bundle | Merge pair into the profile stamp | Locked out: exclusive stamps, never merge/overwrite (D-04, Phase 22 D-01). |
| Automated full Firefox-profiler parser | Human `samply load` for shares | Build a tiny string-scan for the **heap gate** only. Do not hand-roll a profiler UI. Approximate shares in the committed doc come from the live flame. |
| `dhat-heap` on `liquidfun` | Feature on the published crate | Locked out (D-11/D-15). |

**Installation:**

```bash
# Already present on this host (2026-09-21)
samply --version   # samply 0.13.1
# dhat is a Cargo optional dep, not a CLI:
# crates/liquidfun-wasm/Cargo.toml  dhat = { version = "0.3.3", optional = true }

# Live evidence (DoD — not optional)
just playground-dam-break-bench
just playground-dam-break-profile
just playground-dam-break-audit-bundle
# then, only if the heuristic matches:
just playground-dam-break-heap
```

**Version verification:** `dhat` 0.3.3 is still the crates.io latest (2024-02-04). `samply` 0.13.1 is installed. `dhat` is not yet in `Cargo.lock` — the lockfile update is part of implementation. [VERIFIED: `rg '^name = "dhat"' Cargo.lock` empty; `samply --version`]

## Architecture Patterns

### Recommended Project Structure

```
docs/
├── native-performance-audit.md      # NEW committed notes (unreviewed)
└── playground-dam-break-timing.md   # MODIFY: current SHA-bound sample
crates/liquidfun-wasm/
├── Cargo.toml                       # ADD optional dhat-heap, never default
└── src/bin/dam_break_bench.rs       # ADD cfg-gated Alloc + Profiler (bin only)
tools/xtask/src/playground.rs        # DISPATCH two new subcommands
tools/xtask/src/playground/
├── bundle.rs                        # NEW: same-HEAD copy into exclusive stamp
├── heap.rs                          # NEW: gated dhat spawn
├── symbols.rs                       # NEW: gzip + optional .syms.json scan
├── pair.rs                          # DO NOT GROW (616 / 628)
├── profile.rs                       # unchanged capture
└── stamp.rs                         # reuse mint_exclusive_stamp
tools/xtask/tests/
├── playground_cli.rs                # SPLIT before adding tests (622 / 628)
└── playground_cli/                  # pair/profile stay; add bundle.rs + heap.rs
justfile                             # two new one-line aliases
target/dam-break-perf/<utc-stamp>/   # gitignored; bundle + optional heap sibling
```

### Pattern 1: Copy-only same-HEAD audit bundle

**What:** Scan `target/dam-break-perf/*/` for one `pair.json` (`kind=unprofiled_pair`) and one `profile-identity.json` (`kind=samply_cpu`) whose `git_head` values match each other. Mint a **new** exclusive stamp. `fs::copy` `pair.json`, `pair.md`, `rust.json.gz` (and any sibling `*.syms.json` if present). Write `audit-bundle-identity.json`. Never move, never overwrite sources.

**When to use:** After live pair + live profile exist for the HEAD that the audit will cite.

**Fail closed when:** either stamp missing; `pair.json` or `rust.json.gz` missing; HEAD mismatch; `pair.json` contains profile keys; destination stamp exists (mint handles this).

**Recommend CLI (discretion):**

```text
cargo xtask playground dam-break-audit-bundle
  [--pair-stamp <utc>] [--profile-stamp <utc>]
```

If flags omitted, pick the lexicographically latest matching pair stamp and latest matching profile stamp for **current** `git rev-parse HEAD`. If several stamps share that HEAD, latest-name is deterministic because UTC stamps sort lexicographically. Tests inject `LIQUIDFUN_XTASK_GIT` + fixture stamps the same way Phase 22 does.

**Identity JSON (recommended fields, names are discretion):**

```json
{
  "kind": "audit_bundle",
  "timing_authority": "unprofiled_wall_clock",
  "not_timing_authority": true,
  "git_head": "<40-hex>",
  "source_pair_stamp": "2026-09-21T03-01-00Z",
  "source_profile_stamp": "2026-09-21T03-02-00Z",
  "copied": ["pair.json", "pair.md", "rust.json.gz"],
  "profile_blob": "rust.json.gz",
  "disclaimer": "Unreviewed local playground Dam Break sample. Profile blobs are not the 3x number."
}
```

`not_timing_authority` applies to the **profile blob**. The copied `pair.json` keeps `timing_authority: unprofiled_wall_clock`. Do not recompute the ratio from samply.

### Pattern 2: Dedicated profiling Cargo profile stays the sampler binary; heap uses a feature, not the gate binary

**What:** Default `just playground-dam-break-bench` continues `cargo run --release --bin dam-break-bench` **without** `--features dhat-heap`. Heap command:

```text
cargo run -p liquidfun-wasm --profile profiling --bin dam-break-bench --features dhat-heap -- --warmup 60 --steps 600
```

**Why `--profile profiling` for dhat (discretion, recommended):** dhat docs require debug info for useful backtraces; `[profile.profiling]` already has `debug = true`. Using `--release --features dhat-heap` would rebuild `target/release/dam-break-bench` as the instrumented binary and risk a fingerprint mix-up until the next unfeatured rebuild. Profiling-profile keeps `target/release/dam-break-bench` as the unprofiled gate artifact. [CITED: docs.rs/dhat/0.3.3 — “You should only use dhat in release builds” plus “enable source line debug info”; workspace `[profile.profiling]` inherits release.]

**Bin-only wiring:**

```rust
// crates/liquidfun-wasm/src/bin/dam_break_bench.rs
// Source: https://docs.rs/dhat/0.3.3/dhat/

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn run() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = match std::env::var_os("LIQUIDFUN_DHAT_HEAP_FILE") {
        Some(path) => dhat::Profiler::builder().file_name(path).build(),
        None => dhat::Profiler::new_heap(),
    };
    // existing parse + run_dam_break_bench
}
```

```toml
# crates/liquidfun-wasm/Cargo.toml
[features]
default = []
dhat-heap = ["dep:dhat"]

[dependencies]
dhat = { version = "0.3.3", optional = true }
```

Do **not** put `#[global_allocator]` in `lib.rs` (cdylib / wasm / timers must stay uninstrumented). Do **not** enable `dhat-heap` on wasm32 builds. Host `cargo clippy --workspace --all-targets --all-features` **will** compile this feature (CI and `just check`) — that is desired compile coverage, not a reason to skip the feature flag.

### Pattern 3: Fail-closed allocator/`Vec` heuristic, then human names

**What:** Heap gating is mechanical. Function ranking for the committed audit is human (`samply load`).

Scan, in order:

1. Sibling sidecar whose name matches samply’s load path: `rust.json.syms.json` (`Path::with_extension("syms.json")` on `rust.json.gz` replaces only `.gz`). [CITED: github.com/mstange/samply `profile_path.with_extension("syms.json")`]
2. Any other `*.syms.json` in the source profile stamp.
3. Decompressed `rust.json.gz` JSON strings (`threads[].stringArray` / recursive string walk via existing `serde_json`).

Match (case-sensitive; D-12 list plus demangled equivalents):

- `alloc::`
- `__rust_alloc`, `__rdl_alloc`, `__rg_alloc`
- `alloc::vec::Vec`, `RawVec`, `to_vec`
- `GlobalAlloc`, `core::alloc::`
- `clone` as in `core::clone::` / `T as core::clone::Clone` / `clone::clone`

**Fail closed:**

| Evidence | Heap command | Audit notes |
|----------|--------------|-------------|
| Match ≥1 needle | Run dhat; write dump + identity | Record matched needles; dump is not the 3× number |
| Symbols present, zero needles | Exit nonzero; write **no** dump | “Heap skipped: samply showed no allocator/`Vec` time” |
| Gzip is not JSON (Phase 22 fake writes `fake-samply-json-gz`) or sidecar missing **and** gzip has only `0x…` addresses | Exit nonzero; write **no** dump | “Heap skipped: could not read symbols” — **not** the same as “not found.” Human still uses `samply load` on the live stamp while the profiling binary exists |

Do not treat “inconclusive parse” as “no allocator time.” Do not run dhat just in case (D-13).

### Pattern 4: Committed names, gitignored blobs

**What:** `docs/native-performance-audit.md` is the durable artifact. Profiles expire. Copy function names, approximate shares, cause class, SHA, unprofiled ratio, and the bundle stamp path. Do not paste flamegraphs.

**Cause classes (locked taxonomy, D-07):** extra per-particle work / per-step allocation / checks that survive `--release` / algorithm/shape differences.

**C++ counterparts (D-08) — only for shape mismatch, from pinned source, no C++ samply required.** Verified in-tree C++ seams at submodule `7f20402173fd143a3988c921bc384459c6a858f2`:

| Rust hunt-list symbol (unranked until live profile) | C++ shape if the profile names it |
|-----------------------------------------------------|-----------------------------------|
| `ParticleNeighborhood::from_view` (`proxy.rs`: allocate `Vec<Proxy>`, `sort_by_key`, enumerate pairs) | `b2ParticleSystem::UpdateContacts` → `UpdateProxies(m_proxyBuffer)` + `SortProxies(m_proxyBuffer)` — in-place member buffer [VERIFIED: `b2ParticleSystem.cpp` ~2256–2261] |
| `World::backup_step_limit_state` (clones bodies/fixtures/joints/particle systems/broad phase/contact manager every `step`) | `b2World::Step` mutates in place; no full-world clone on the happy path [VERIFIED: `execution.rs` 44–55, 136] |
| `ParticleStorage::replace_solver_candidate` (`clone` + `check_invariants()?` on release) | `b2Assert` compiled out under `NDEBUG` in `oracle-release` [VERIFIED: `runtime.rs` 62–96; PITFALLS.md pitfall 5] |
| `pressure.rs` / `material.rs` per-pass `velocities().to_vec()` | C++ reuses `m_velocityBuffer` / member scratch [VERIFIED: `pressure.rs`, `material.rs`] |

The table above is a **researcher map**. D-02 forbids pasting it into the audit as ranked causes. Live samply decides.

### Anti-Patterns to Avoid

- **Inventing names from the hunt list:** Fake fixtures and source reading are not PERF-AUDIT. [D-01/D-02]
- **Quoting samply / dhat / `step_profiled` duration as the 3× number:** Only `pair.json` `rust_over_cpp_ratio`. [D-03]
- **Moving or merging Phase 22 stamps:** Copy into a new exclusive stamp. [D-04]
- **Hiding samply/CMake/dhat flags in `just`:** One-line `cargo xtask …` only. [D-05]
- **`#[global_allocator]` on `liquidfun` or the wasm `lib.rs`:** Bin + optional feature only. [D-11/D-15]
- **`--features dhat-heap` on the pair command:** Contaminates the gate binary’s feature fingerprint. [D-11]
- **Growing `pair.rs` (616) or `playground_cli.rs` (622):** Bright Builds fails at 629. Split first.
- **Filling `reference/performance/manifest.toml`:** Stays `reviewed_reports = []`. [VERIFIED: file contents]
- **Physics kernel edits:** Phase 24. [D-10]
- **Self-approving:** Independent AI review after verification. [D-16]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Exclusive dated directories | Custom `latest/` clobber dir | Existing `stamp::mint_exclusive_stamp` | AlreadyExists bump + 8-attempt cap already tested |
| Gzip inflate | Manual DEFLATE | `flate2::read::GzDecoder` | Already in xtask |
| Heap profiler | Custom `GlobalAlloc` wrapper | `dhat` 0.3.3 | Backtraces, DHAT viewer JSON, known maintenance warning |
| Flamegraph UI / sample-weighted walker | Full gecko `stackTable` interpreter | `samply load` for shares; string-scan only for the heap gate | Presymbolicate lives in a sidecar; a wrong parser would mis-rank |
| C++ CPU wrap | `cpp.json.gz` this phase | Source comparison for shape mismatch | D-08; C++ samply is deferred |
| License/advisory policy | Ad-hoc allowlist in docs | `cargo deny --locked check` after adding `dhat` | `deny.toml` `graph.all-features = true` will see the optional dep |

**Key insight:** The expensive part of this phase is **running the live pair+profile and reading the flame**, not new infrastructure. Keep xtask thin: copy, detect, spawn. Put judgment in Markdown.

## Common Pitfalls

### Pitfall 1: Treating the hunt list as the audit

**What goes wrong:** `docs/native-performance-audit.md` names `backup_step_limit_state` / `from_view` / `to_vec` because PITFALLS.md did, without samply shares. Phase 24 then “fixes” the wrong extra work.

**Why it happens:** Those functions are real extra work and easy to grep. D-02 exists because they are still unranked.

**How to avoid:** Live `samply load`. Each named row needs an approximate share. Unnamed hunt-list items go under **Not found** or **Not ranked**.

**Warning signs:** Audit committed before `just playground-dam-break-profile` on the cited SHA; no stamp path.

### Pitfall 2: Blessing exploratory numbers / mixing authorities

**What goes wrong:** Timing doc or audit quotes samply wall, dhat stderr totals, or the historical `70594.221` ms as the current 3× number. Or `reviewed_reports` gains a row.

**Why it happens:** Four clocks exist (unprofiled pair, profiling profile, `step_profiled`, dhat).

**How to avoid:** One sentence in both docs: ratio comes from `pair.json` at stamp X, HEAD Y. Other clocks labeled `not_timing_authority`. Manifest stays empty. [PITFALLS.md 2 and 4]

**Warning signs:** Audit table with a “samply ms” column next to Rust/C++.

### Pitfall 3: Feature fingerprint contaminates `--release`

**What goes wrong:** Heap command uses `cargo run --release --features dhat-heap`, replacing `target/release/dam-break-bench`. A later pair run is slow or still instrumented if someone invokes the binary by path.

**Why it happens:** Cargo features are per-package; the `release` output dir is shared.

**How to avoid:** Heap uses `--profile profiling --features dhat-heap`. Pair never passes the feature. xtask heap identity records `cargo_profile: profiling`, `features: ["dhat-heap"]`, `not_timing_authority: true`.

**Warning signs:** `dhat: The data has been saved` on a `dam-break-bench` stderr during `just playground-dam-break-bench`.

### Pitfall 4: Scanning `rust.json.gz` only (missing sidecar)

**What goes wrong:** Heuristic sees hex addresses, skips heap as “no Vec time,” while `samply load` shows `Vec::to_vec` at 40%.

**Why it happens:** `--unstable-presymbolicate` writes a **sidecar** that `samply load` finds via `with_extension("syms.json")`. Phase 22 only requires nonempty `rust.json.gz`. Fake samply writes raw `fake-samply-json-gz` bytes, not gzip.

**How to avoid:** After the first live profile, `ls` the profile stamp. Copy `rust.json.gz` **and** any `*syms*` sibling into the bundle. Scan sidecar `string_table` first. Distinguish “no symbols” from “no allocator time.”

**Warning signs:** Audit says “heap skipped, no allocator time” but `samply load` shows `alloc::`.

### Pitfall 5: File-length regressions

**What goes wrong:** Adding bundle tests to `playground_cli.rs` (622 lines) or helpers to `pair.rs` (616) fails Bright Builds at 629.

**Why it happens:** Phase 22 already sat near the cap.

**How to avoid:** New `playground/bundle.rs`, `heap.rs`, `symbols.rs`. Split `playground_cli.rs` into `playground_cli.rs` + `playground_cli/{pair,profile,timers,bundle,heap}.rs` **before** adding cases. No TSV exception.

**Warning signs:** `FAIL file-lengths tools/xtask/tests/playground_cli.rs`.

### Pitfall 6: SHA drift between pair, profile, and docs commit

**What goes wrong:** Pair at HEAD A, uncommitted edits, profile at HEAD A with different bytes, audit committed as HEAD B citing B.

**Why it happens:** `git rev-parse HEAD` ignores the dirty tree. The docs commit cannot equal the measured SHA.

**How to avoid:** Capture pair+profile on a **clean** tree. Cite the **measured** 40-hex in the audit (ancestor of the docs commit). Bundle requires pair HEAD == profile HEAD. Optional: refuse bundle if `git status --porcelain` is nonempty (discretion; recommend warn + record `worktree_dirty` bool rather than a hard lock).

**Warning signs:** Audit HEAD does not appear in either stamp’s `git_head`.

### Pitfall 7: dhat dump lands in the repo root

**What goes wrong:** `Profiler::new_heap()` writes `dhat-heap.json` in cwd; it gets committed or gitignored inconsistently.

**Why it happens:** dhat default path is cwd `dhat-heap.json`. [CITED: docs.rs/dhat — “saved to dhat-heap.json”]

**How to avoid:** xtask mints a stamp first, sets `LIQUIDFUN_DHAT_HEAP_FILE` to that stamp’s `dhat-heap.json`, and fails if the file is missing/empty after the process. Recommend a **sibling** exclusive stamp so the audit bundle stays a pure copy of pair+profile (D-04 file set). `/target/` already gitignores it.

**Warning signs:** Untracked `dhat-heap.json` at repo root.

### Pitfall 8: SIMD-first “Not found” section omitted

**What goes wrong:** Audit names clones but never says SIMD / Rayon / `-ffast-math` / bounds-check tax were **not** the first move.

**Why it happens:** Share tables feel complete without a negative space section.

**How to avoid:** Explicit **Not found** heading (D-07). Include SIMD, default Rayon, PGO, `unsafe` indexing, WASM-vs-C++, Phase 12 sealed matrix.

## Code Examples

### Audit-bundle copy (new exclusive stamp)

```rust
// Pattern: copy, never move. Reuse stamp::mint_exclusive_stamp.
// Source: tools/xtask/src/playground/stamp.rs (Phase 22)

fn copy_required(from_dir: &Path, to_dir: &Path, name: &str) -> Result<(), PlaygroundError> {
    let from = from_dir.join(name);
    let to = to_dir.join(name);
    if !from.is_file() {
        return Err(PlaygroundError::new(
            "bundle",
            format!("missing {}", from.display()),
        ));
    }
    fs::copy(&from, &to).map_err(|error| {
        PlaygroundError::new("bundle", format!("copy {}: {error}", from.display()))
    })?;
    Ok(())
}
```

### dhat on the private bin (not the crate root)

```rust
// Source: https://docs.rs/dhat/0.3.3/dhat/

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(feature = "dhat-heap")]
let _profiler = dhat::Profiler::builder()
    .file_name(output_path)
    .build();
```

### Heap feature must not appear on the pair argv

Pair already uses [VERIFIED: `pair.rs` 75–92]:

```text
cargo run -p liquidfun-wasm --release --quiet --bin dam-break-bench -- --warmup N --steps M
```

Heap must add `--profile profiling --features dhat-heap` and must **not** pass `--release`.

### Isolation proof (unchanged commands)

```bash
cargo xtask package verify
cargo tree -p liquidfun --edges normal
# expect only liquidfun → bitflags; no dhat, samply, serde, cmake, flate2
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 22: fake-tool DoD; live pair optional | Phase 23: live pair + live samply are DoD | CONTEXT D-01 2026-09-21 | Planner must budget ~70s+ Rust wall plus samply overhead on this host |
| Research path `target/v12-native-perf/` | Locked `target/dam-break-perf/<utc-stamp>/` | v1.2 roadmap | Do not revive the old directory name |
| `[profile.profiling]` `debug = "limited"` in early architecture research | Implemented `debug = true` | Phase 22 | Do not change it |
| samply symbols “embedded in json.gz” (STACK.md shorthand) | Presymbolicate sidecar `*.syms.json` next to the gzip | samply 0.13.1 `#202` | Bundle must copy the sidecar if present |
| dhat as a vague follow-up | Locked `dhat` 0.3.3 on `dam-break-bench` only, gated by allocator/`Vec` time | CONTEXT D-11–D-13 | Optional dep; skip path is a first-class deliverable |

**Deprecated/outdated:**

- Phase 22 D-13 “live run is not DoD” — **does not apply** to Phase 23.
- Filling `reference/performance/manifest.toml` from this canary — still forbidden.
- Required C++ `cpp.json.gz` — deferred (CONTEXT deferred list).
- SIMD / Rayon / `unsafe_code` lift — later opt-in after the scalar canary.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Live `--unstable-presymbolicate` writes `rust.json.syms.json` beside `rust.json.gz` (`Path::with_extension("syms.json")` on a `.json.gz` path) | Pattern 3 / Pitfall 4 | Heuristic and `samply load` from the bundle miss names. Mitigation: after the first live profile, copy whatever `*syms*` file actually appears. |
| A2 | Presence of D-12 needles (no minimum % share) is enough to run dhat | Pattern 3 | A tiny `clone` leaf could trigger a slow heap run. Acceptable vs D-13 “do not run just in case”; still fail-closed toward evidence. |
| A3 | Heap should use `--profile profiling` rather than `--release --features dhat-heap` | Pattern 2 | If dhat+profiling is too slow or mis-builds, fall back to `--release --features dhat-heap` **plus** an explicit pair rebuild afterward. Do not leave the gate binary instrumented. |

**If this table is empty:** All claims were verified or cited. It is not empty — A1 must be confirmed on the first live stamp.

## Open Questions

1. **Exact presymbolicate sidecar filename on this host**
   - What we know: samply load uses `profile_path.with_extension("syms.json")`. [CITED: mstange/samply main.rs]
   - What's unclear: whether 0.13.1 writes `rust.json.syms.json`, `rust.syms.json`, or embeds names in the gzip on macOS aarch64.
   - Recommendation: first live `dam-break-profile` plus `ls` of the stamp; bundle copies every `*syms*` sibling; heuristic searches those files.

2. **dhat wall-clock cost for 60+600 Dam Break**
   - What we know: unprofiled Rust was ~70.6 s at `1e5cbcc…`; dhat “can be large.” [CITED: docs.rs/dhat/0.3.3]
   - What's unclear: minutes vs tens of minutes on Apple M4 Max.
   - Recommendation: keep the locked 60+600 recipe for comparable stacks; warn the executor; do not shrink particle count.

3. **How many stacks to list**
   - Discretion. Recommend the smallest set that covers ≥ ~80% of sampled time **or** the top 8–12 named frames, whichever is clearer, plus **Not found**. Phase 12’s 10% admission floor is a Phase 24 idea, not a Phase 23 listing cutoff. [CITED: `reference/performance/policy.json` `minimum_profile_basis_points: 1000`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | All xtask + dhat compile | ✓ | rustc 1.97.0 / cargo 1.97.0 | — |
| samply | Live PERF-AUDIT profile | ✓ | 0.13.1 | Fail closed with Phase 22 install text (already implemented) |
| CMake | Live pair C++ side | ✓ | 3.27.9 (local floor 3.25) | — |
| Ninja | Oracle extra target | ✓ | 1.13.2 | — |
| just | Discoverable aliases | ✓ | 1.48.0 (repo pin 1.55.1 is docs-only) | `cargo xtask playground …` |
| Python 3.13 + mdformat 1.0.0 | `just markdown-check` on new docs | ✓ | python3.13 3.13.12; mdformat 1.0.0 | Default `python3` is 3.14.6; use 3.13 for exclusion-config if check fails |
| cargo-deny 0.20.2 | License graph after adding dhat | ✓ | 0.20.2 | — |
| dhat crate | PERF-HEAP | ✗ (not in lockfile yet) | add 0.3.3 | Implementation step; not a host CLI |
| xctrace | D-13 fallback only | ✓ | 27.0 | Use only if dhat blocked |
| Upstream C++ tree | D-08 counterpart names | ✓ | `b2ParticleSystem.cpp` present | — |
| Existing pair/profile stamps | Bundle inputs | ✗ | `target/dam-break-perf` empty | **Must run live pair+profile** (D-01) |

**Missing dependencies with no fallback:**

- Live SHA-bound pair + samply stamps (directory empty). Phase cannot complete without those runs.

**Missing dependencies with fallback:**

- `dhat` crate (add during implementation). xctrace Allocations if dhat hangs.

**Step 2.6 note:** External tools are required (samply, CMake, live Dam Break). Not a code-only phase.

## Security Domain

`workflow` does not set `security_enforcement: false` — include this section.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Local developer CLI; no accounts |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Parse `--pair-stamp` / `--profile-stamp` as filename-safe `YYYY-MM-DDTHH-MM-SSZ` only; reject `/`, `..`, NUL. Reuse integer-only `LIQUIDFUN_XTASK_STAMP_UNIX`. Do not pass stamp names through a shell. |
| V6 Cryptography | no | No new crypto; do not treat profile gzip as a hash oracle |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via stamp CLI flags | Tampering | Allowlist stamp charset; join only under `target/dam-break-perf/` |
| Committing samply/dhat blobs with absolute home paths | Information disclosure | Keep blobs under `/target/` (already gitignored); redact paths in committed notes |
| Command injection via `just` | Tampering | `just` stays a one-line `cargo xtask` printer; no samply/dhat flags |
| Optional-dep license/supply slip | Tampering | `dhat` 0.3.3 from crates.io only (`deny.toml` `unknown-git = "deny"`); `cargo deny --locked check` |
| Feature accidentally on published crate | Elevation of privilege / integrity of isolation | `dhat-heap` on `liquidfun-wasm` (`publish = false`); `package verify` + `cargo tree -p liquidfun` |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/23-baseline-pair-and-named-audit/23-CONTEXT.md` — D-01..D-16
- `.planning/REQUIREMENTS.md` — PERF-AUDIT, PERF-HEAP
- `.planning/ROADMAP.md` Phase 23 success criteria
- `tools/xtask/src/playground/{pair,profile,stamp,identity,error}.rs` — live seams
- `tools/xtask/tests/playground_cli.rs` — fake-tool pattern; 622 lines
- `crates/liquidfun-wasm/Cargo.toml`, `src/bin/dam_break_bench.rs`
- `Cargo.toml` `[profile.profiling]`; `crates/liquidfun/Cargo.toml` bitflags-only
- `docs/playground-dam-break-timing.md`; `BENCHMARKING.md`; `reference/performance/{policy.json,manifest.toml}`
- `third_party/liquidfun/.../b2ParticleSystem.cpp` — `UpdateProxies` / `SortProxies` / `UpdateContacts`
- [docs.rs/dhat/0.3.3](https://docs.rs/dhat/0.3.3/dhat/) — `Alloc`, `Profiler::new_heap`, `ProfilerBuilder::file_name`, cwd `dhat-heap.json`, maintenance warning
- [crates.io/crates/dhat](https://crates.io/crates/dhat) — 0.3.3 still latest, 2024-02-04
- [samply v0.13.1 release](https://github.com/mstange/samply/releases/tag/samply-v0.13.1) — `--unstable-presymbolicate`
- Host probe 2026-09-21: samply 0.13.1, rustc 1.97.0, empty `target/dam-break-perf`, dhat absent from lockfile

### Secondary (MEDIUM confidence)

- [mstange/samply main.rs](https://github.com/mstange/samply/blob/main/samply/src/main.rs) — `with_extension("syms.json")` load path (current main, not the 0.13.1 tag blob)
- [firefox-to-pprof samply module](https://docs.rs/firefox-to-pprof/latest/firefox_to_pprof/samply/) — symbols in sidecar, not `funcTable`
- `.planning/research/{PITFALLS,ARCHITECTURE,STACK,FEATURES,SUMMARY}.md` — hunt list, honesty rules, `dhat` 0.3.3 pin (STACK still says HIGH for version)
- box2d-rust 1.3 validator-in-release analog — [CITED: docs.rs/crate/box2d-rust/latest via FEATURES.md]

### Tertiary (LOW confidence)

- Exact minutes for dhat × 600 Dam Break steps on this M4 Max — not measured
- Whether 0.13.1 macOS aarch64 embeds any demangled names inside the gzip itself

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — pins and host tools verified; dhat 0.3.3 still latest
- Architecture: HIGH — copy-only bundle, bin-only dhat, exclusive stamps match existing code
- Pitfalls: HIGH for honesty/isolation/file-length; MEDIUM for sidecar filename and first ranked kernel

**Research date:** 2026-09-21
**Valid until:** 2026-10-21 (stable pins; re-check crates.io `dhat` and samply sidecar layout if either moves)

## Planner notes (execution shape, not a plan)

Suggested waves so file-length and live DoD stay honest:

1. Split `playground_cli.rs`; add `bundle.rs` + `just playground-dam-break-audit-bundle`; fake-stamp copy tests.
2. Optional `dhat-heap` on the **bin**; `symbols.rs` + `heap.rs` + `just playground-dam-break-heap`; fixture gzip with/without needles; `cargo check -p liquidfun-wasm --bin dam-break-bench --features dhat-heap`; `cargo deny --locked check`; isolation tree.
3. Live `just playground-dam-break-bench` then `just playground-dam-break-profile` on a clean tree; mint the audit bundle; `ls` sidecars.
4. Write `docs/native-performance-audit.md` from `samply load`; run or skip heap per heuristic; refresh timing doc (keep `1e5cbcc…` as a one-line historical pointer — recommended yes); `just markdown-check`; no kernel diffs; no manifest edits.
5. Isolation + Bright Builds + independent AI review (implementer does not self-approve).

Do not quote samply duration. Do not commit `.json.gz` / `.trace` / `dhat-heap.json`.
