# Project Research Summary

**Project:** liquidfun-rs
**Domain:** Native LiquidFun performance closing — Dam Break pair timing, scripted CPU profiling, committed audit notes, and profile-guided scalar hot-path work
**Milestone:** v1.2 Native Performance Closing
**Researched:** 2026-09-20
**Confidence:** HIGH for tooling, gate authority, anti-features, and crate isolation; MEDIUM for which shared kernel currently dominates the ~300× Dam Break gap

## Executive Summary

v1.2 is a developer-facing performance-closing loop, not a new physics product, playground catalog, or public benchmark claim. Experts close a C++ vs Rust physics gap the way box2d-rust did: lock one same-host scalar pair, sample a release-class binary with symbols, name extra work (allocations, always-on validators, clones, shape mismatches), then change shared kernels — not SIMD, not default threads, not a sealed 32-case matrix. The first recorded Dam Break Medium sample (~70.6 s native vs ~0.23 s pinned C++ for 600 timed steps, ~300×) is diagnosis. Launch means that same locked pair is ≤ 3× on one macOS aarch64 host, the hunt is evidenced, and leftover delta is described honestly.

Reuse the existing unprofiled Dam Break pair as the **only numeric authority**. Add a thin local profiling shell around it: workspace `[profile.profiling]` (inherits `release`, `debug = true`) plus scripted **samply 0.13.1** into a dated gitignored directory. Keep `just playground-dam-break-bench` as the 3× command (native `liquidfun-wasm` `--release` `dam-break-bench` vs `oracle-release` `playground-dam-break-bench`, 1920 particles, 60 warm-up + 600 timed `World::step` / `b2World::Step`). Profiled wall times, `World::step_profiled`, Instruments, and dhat are diagnostic only. Do not invent a second matrix, fill `reference/performance/manifest.toml`, add required perf CI, or compare WASM to C++.

The ~300× gap is extra-work-shaped. Typical PGO/SIMD/bounds-check wins are tens of percent, not two orders of magnitude. First suspects already visible in-tree (happy-path full-world clones, release `check_invariants`, per-pass `Vec` rebuilds, neighborhood allocate/sort) must be ranked by sampling before any kernel PR. Mitigate by measuring first, admitting only evidenced scalar shared-path fixes, re-running the unprofiled pair after every wave, keeping the scalar deterministic baseline, and treating a physics mismatch as a failed candidate rather than a faster sample.

## Reconciled decisions

Researchers disagreed on two small points. These are the locked answers for roadmap and planning. Do not invent a third path.

| Topic | Decision | Why |
| --- | --- | --- |
| Evidence directory | **`target/dam-break-perf/<utc-stamp>/`** | Dated, already gitignored via `/target/`, named after the canary. Reject `target/v12-native-perf/<UTC>/`: the `v12` prefix collides with Phase 12’s `target/phase12-performance/` and invites sealed-matrix revival. Keep Architecture’s rules on this STACK path: refuse to clobber an existing stamp, write `pair.json` / samply `.json.gz` / host identity sidecar, bind notes to git HEAD + directory name. |
| Gate authority | Unprofiled native **`--release`** vs **`oracle-release`** Dam Break Medium pair | Same-host, scalar, locked 1920-particle recipe. samply / `[profile.profiling]` / `step_profiled` / xctrace / dhat timings are **never** the 3× number. |
| Profiler stack | **samply 0.13.1** (`mstange/samply`) + workspace **`[profile.profiling]`** with `debug = true` | Documented release+debug-info requirement. `xcrun xctrace` and private `dhat` 0.3.3 on `dam-break-bench` are optional follow-ups. Do not default to cargo-flamegraph, cargo-instruments, or feldera/samply 0.13.2. |
| Phase 12 / CI / baseline | Do **not** revive the sealed public matrix; do **not** add required perf CI; scalar deterministic baseline stays | `reviewed_reports = []` remains empty. Hobby scope: local macOS + existing Cargo CI. SIMD/Rayon/`-ffast-math`/`-march=native` stay off the pair. |

The profile recipe may snapshot an unprofiled pair into the dated directory for SHA identity, then record samply on a **separate** `[profile.profiling]` binary. That does not change the gate command: `just playground-dam-break-bench` remains the 3× authority.

## Key Findings

### Recommended Stack

Details: [STACK.md](STACK.md). Keep the v1.0 Cargo/CMake foundation. Do not re-pin Rust, the oracle commit, or the published crate graph.

Add a local observability shell only. Do not add production dependencies to `liquidfun` (`bitflags` only). Do not put samply, dhat, CMake, or serde on the published crate. Do not turn `debug` on default `release` (that would silently change the gate binary and CI artifacts).

**Core technologies:**

- **Existing `just playground-dam-break-bench` / xtask pair:** unprofiled Instant vs `oracle-release` extra target — the only 3× number.
- **Workspace `[profile.profiling]`:** inherits `release`, `debug = true`, `strip = false` — symbolicated stacks without mutating `--release`.
- **samply 0.13.1:** scripted CPU sampling on macOS aarch64; `--save-only --unstable-presymbolicate -o target/dam-break-perf/<utc-stamp>/rust.json.gz` wrapping the profiling-profile `dam-break-bench`.
- **Gitignored `target/dam-break-perf/<utc-stamp>/`:** dated pair JSON, samply `.json.gz`, optional `.trace`, host/git/compiler sidecar. Never copy into `reference/performance/manifest.toml`.
- **Committed notes:** extend `docs/playground-dam-break-timing.md`; add `docs/native-performance-audit.md` (named functions, suspected causes, current delta; unreviewed).
- **Existing `World::step_profiled`:** optional separate `--emit-phase-profile` dump. Never inside the gate process.
- **Optional:** `xcrun xctrace` (`--instrument 'CPU Profiler'`), private `dhat-heap` on `dam-break-bench` only after samply shows allocator/`Vec` time, `cargo-show-asm` 0.2.62 for a named kernel, Criterion 0.8.2 for a named micro **after** samply — never as the Dam Break gate.

**Do not add:** required performance CI, LLVM PGO/BOLT as the first lever, default SIMD/Rayon, glam/nalgebra, in-process C ABI, `hyperfine` as pair authority, wasm-pack/browser profilers as the native gate.

### Expected Features

Details: [FEATURES.md](FEATURES.md). Frame capabilities as what a developer or playground visitor can do. Existing engine, oracle, Dam Break pair, six-scene playground, and Phase 12 *method* (unprofiled authority, scalar `--release`) are dependencies, not new features. Do not fill the reviewed-report manifest.

**Must have (table stakes):**

- **PERF-AUDIT** — Committed notes name hot functions, extra per-particle work, per-step allocations, `--release` checks, algorithm/shape differences, and the current Dam Break Medium ratio.
- **PERF-PAIR** — Re-run the locked same-host pair on demand; persist dated unprofiled wall ms, ms/step, and Rust/C++ ratio (not stdout-only).
- **PERF-PROFILE** — Repeatable `just` / xtask CPU profiles of the timed Dam Break binary into `target/dam-break-perf/<utc-stamp>/` with host, HEAD, compiler, command identity.
- **PERF-ADMIT** — Land a change only from named profile share or typed bottleneck + unprofiled pair improvement + existing correctness gates green; no scene-size cheat.
- **PERF-SHARED** — Fixes land in shared `liquidfun` particle/rigid hot paths, not Dam Break-only WASM scene hacks.
- **PERF-GATE** — Native Dam Break Medium wall time ≤ 3× pinned C++ on that unprofiled pair.
- **PERF-SPOT** — Spot-check Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel after shared-path fixes (native `--release`; no C++ pair per scene).
- **PERF-BASELINE** — Scalar deterministic default; SIMD/parallel explicit opt-in, off.
- **PERF-NOTES** — Remaining delta documented honestly; no README “Rust is N× slower” trophy.
- **PERF-WASM** — After the native gate, playground/WASM sanity (`just web-player-smoke`); never vs `oracle-release`.

**Should have (competitive, still this milestone’s shape):**

- Canary-first same-order close instead of 32 sealed public cases.
- Named-function audit in git; raw profiles stay gitignored.
- Lightweight admission (Phase 12’s 10% profile-share *idea*, not the 32-case JSON record).
- Engine-wide hunt, scene-local numeric bar.

**Defer (later / not v1.2):**

- SIMD / parallel opt-in features after scalar ≤ 3× and an explicit determinism decision.
- Phase 12 sealed matrix production, calibration, reviewed-report promotion.
- WASM vs native performance engineering beyond sanity.
- Crate publication, new playground scenes, scene editor.
- Relaxing `unsafe_code = "forbid"` for intrinsics.
- Criterion catalog or a second native canary as a substitute gate (optional supporting evidence only after Dam Break is close).

### Architecture Approach

Details: [ARCHITECTURE.md](ARCHITECTURE.md). Do not redesign `crates/liquidfun`, the SolidJS playground, or the Phase 12 sealed matrix. Add an observability loop around the existing Dam Break pair, then change only shared native hot paths that sampling names.

**Pattern:** just prints → xtask orchestrates → Cargo builds Rust → CMake `oracle-release` builds the C++ extra target → host samply wraps binaries. Dependency arrows still point toward `liquidfun`. Profiling, CMake, samply, scenes, and dated reports never become production dependencies.

**Major components:**

1. **`crates/liquidfun`** — shared particle/rigid kernels; optional diagnostic `step_profiled` schema; no profiler/serde/C++/WASM.
2. **`liquidfun-wasm` `dam-break-bench`** — native-only locked recipe timer around `SessionCore::advance` → `World::step` (not `step_profiled`).
3. **C++ `playground-dam-break-bench`** — matching extra target under `oracle-release`; times `b2World::Step` only.
4. **`cargo xtask playground`** — pair command stays the gate; **add** `dam-break-profile` (dated mkdir, samply wrap, host sidecar); dump pair JSON into the evidence dir.
5. **Thin `just playground-dam-break-profile`** — alias only; no CMake or samply flags in just.
6. **`target/dam-break-perf/<utc-stamp>/`** — gitignored evidence (reconciled path; Architecture’s dated/no-clobber rules).
7. **`docs/native-performance-audit.md`** — committed human artifact. Profiles expire; names and causes belong in git.

Measure `advance` so the canary stays on the playground path. Dam Break `on_advance` is a no-op today; Fountain / Water Wheel hooks are real and get native-only spot-checks, not C++ pairs. Optional C++ `-g` is a profile-command cache flag, never a pair-command or new CMake preset.

### Critical Pitfalls

Details: [PITFALLS.md](PITFALLS.md). Top risks for this milestone:

1. **Treating ~300× as SIMD/parallelism** — Audit extra clones, allocations, release validators, and shape mismatches first. SIMD/Rayon stay explicit opt-in after the scalar pair is in the same order of magnitude.
2. **Blessing exploratory numbers as Phase 12 or “Rust is X% slower”** — Keep `reviewed_reports = []`. Cite the canary as an unreviewed local sample. Do not copy playground cells into the manifest or README badges.
3. **Using profiled timings as the ≤ 3× authority** — Two artifacts: unprofiled pair for the number, samply (and optional xctrace/dhat) for *why*. Never `step_profiled` inside `dam-break-bench`.
4. **Timing the wrong construction or mixing debug/release** — Lock 1920 particles, construction/warm-up/capture/rendering outside the timer, `--release` vs `oracle-release`. Fail closed if the timed binary has `debug_assertions`. Do not pair WASM frames to C++.
5. **Leaving happy-path full-world clones while micro-optimizing kernels** — Rank bytes copied and invariant calls per step alongside CPU samples. Snapshot rollback lazily; reuse scratch; keep fail-closed API errors. A physics mismatch is not a timing sample. No `HashMap` solver-visible order, no default Rayon.
6. **Reviving the sealed 32-case matrix, Linux host, or required perf CI** — Dam Break Medium local pair is the hobby gate. Phase 12 scripts stay unused for v1.2 done.

## Implications for Roadmap

Phase numbering continues after 21. Suggested structure is **measure → name → shared scalar fix → re-measure → WASM last**. Do not start kernel edits in the observability phase. Do not use the playground as a native profiler.

### Phase 22: Observability shell

**Rationale:** Profiles and dated dumps must exist before anyone changes physics. This phase is tooling only and can be tested with fake cmake/samply.
**Delivers:** `[profile.profiling]`; `just playground-dam-break-profile` → `cargo xtask playground dam-break-profile`; dated `target/dam-break-perf/<utc-stamp>/` (fail if stamp exists); pair command can write evidence copies; audit-doc skeleton; install/error text for missing samply; `cargo xtask package verify` still green.
**Addresses:** PERF-PROFILE (tooling), PERF-PAIR (dated dump path).
**Avoids:** Pitfalls 4, 9, 3 (script contract); hiding CMake/samply in just; adding profiler crates to `liquidfun`; mutating default `--release`.

### Phase 23: Baseline pair, profiles, and named audit

**Rationale:** Admission needs named functions and a SHA-bound unprofiled ratio. Guessing SIMD before this phase is the box2d-rust anti-pattern inverted.
**Delivers:** Unprofiled pair at a recorded HEAD into a dated directory; samply of the profiling-profile Rust bin (required) and optional C++ wrap; optional separate `step_profiled` parent dump; filled `docs/native-performance-audit.md` (hot functions, classified extra work, what was *not* found); refresh `docs/playground-dam-break-timing.md` as unreviewed.
**Addresses:** PERF-AUDIT, PERF-PAIR, PERF-PROFILE.
**Avoids:** Pitfalls 1, 2, 4, 5; quoting samply duration as the gate; committing `.json.gz` / `.trace`.

### Phase 24: Shared hot-path waves through the 3× gate

**Rationale:** Fixes belong in `liquidfun` after names exist. One concern per wave; unprofiled re-pair into a **new** dated directory; preserve failed records. Spot-checks ride along so Dam Break-only theater cannot hide Fountain regressions. The numeric gate is this phase’s exit, not a later marketing step.
**Delivers:** Evidence-gated scalar shared-path changes (neighborhood/proxy, contacts, coupling, pressure/damping/integrate, happy-path clone/invariant cuts as profiles rank them); focused `liquidfun` tests + relevant differential/determinism; Dam Break Medium unprofiled ≤ 3×; native spot-checks of the other five scenes; remaining-delta draft in the audit doc.
**Uses:** Unprofiled pair as authority; samply only to retarget the next wave; Criterion only if a named kernel remains after the pair is already near 3×.
**Implements:** Measure → note → shared fix → re-measure; package isolation unchanged.
**Addresses:** PERF-ADMIT, PERF-SHARED, PERF-BASELINE, PERF-GATE, PERF-SPOT, PERF-NOTES (draft).
**Avoids:** Pitfalls 1, 7, 8, 10; scene LOD / skipped passes / lowered particle count; HashMap/Rayon defaults; `unsafe` as the opening move; copying the ratio into `manifest.toml`.

### Phase 25: WASM sanity and honest close

**Rationale:** Visitors use Pages, but WASM vs C++ is not a fair pair. Native 3× must already hold so the browser loop is not used to “debug” 300×.
**Delivers:** `just web-wasm` / `just web-player-smoke`; honest playground note (improved / still sub-realtime vs previous WASM or native Rust — never vs `oracle-release`); finalized remaining-delta notes; empty `reviewed_reports` reconfirmed; `BENCHMARKING.md` one-paragraph boundary (playground pair ≠ Phase 12 claim).
**Addresses:** PERF-WASM, PERF-NOTES.
**Avoids:** Pitfall 6; Instant profiler in the cdylib; lifting the 4-step catch-up cap to fake realtime; README engine-wide speed claims.

### Phase Ordering Rationale

- Tooling before physics so the first kernel PR has names, not folklore.
- Unprofiled pair and samply share a git SHA; profiled duration is discarded for the gate.
- Shared-path waves before the 3× declaration so the gate is a re-measure, not a hope.
- Other scenes are spot-checks of shared fixes, not a second sealed matrix.
- WASM last inherits native step cost; it is a sanity check, not the closing method.
- Scalar baseline constrains every optimization phase; Phase 12 public claims conflict with this milestone’s honesty rules.

### Research Flags

Phases likely needing `/gsd-research-phase` during planning:

- **Phase 24:** The 300× cause is unconfirmed until Phase 23 artifacts exist. Planning should wait for named shares (clone/rollback vs `check_invariants` vs neighborhood vs contact/pressure) rather than pre-selecting SIMD, PGO, or `unsafe` indexing. After the audit lands, research that wave’s storage/aliasing options (lazy snapshot, scratch reuse, split borrows) against the oracle pass graph.

Phases with standard patterns (skip research-phase):

- **Phase 22:** Established just/xtask layering, Cargo custom profile, samply `--save-only` wrap, package-isolation check.
- **Phase 23:** Existing pair recipe + STACK’s samply command; the work is running it and writing notes.
- **Phase 25:** Existing `web-player-smoke` / Pages path; honesty rules are already locked.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH for samply 0.13.1, `[profile.profiling]`, what not to add; MEDIUM for dhat long-term fitness and xctrace/Xcode-26 instrument names | Official samply/crates.io/Homebrew pins; dhat is experimental and lightly maintained |
| Features | HIGH for table stakes, anti-features, and Dam Break recipe lock; MEDIUM for which hot path dominates until a profiled audit | Repo policy + neighboring-engine post-mortems agree on extra-work-first |
| Architecture | HIGH for crate/xtask/oracle/package isolation; MEDIUM for host `samply setup` permissions | Verified in this checkout; macOS sampler install is host-specific |
| Pitfalls | HIGH for policy/integration pitfalls; MEDIUM for ranking clone vs neighborhood vs invariants as *the* 300× cause | In-tree extra work is real; sampling must rank it |

**Overall confidence:** HIGH for how to measure, what not to build, and phase order. MEDIUM for the first physics edit. That gap is why Phase 23 precedes Phase 24.

### Gaps to Address

- **Dominant kernel unknown until Phase 23:** Treat in-tree suspects (full-world clone, release invariants, per-pass `to_vec`, `ParticleNeighborhood::from_view`) as a hunt list, not a committed cause. Planning of Phase 24 should consume the audit doc.
- **samply setup on this Mac:** Architecture flags `samply setup` / signing as MEDIUM. Phase 22 must fail closed with install text, not skip silently.
- **C++ symbol quality:** Optional extra-target `-g` / frame pointers on the **profile** command only. Do not change `oracle-release` pair flags.
- **Thermal/order bias near 3×:** Sequential Rust-then-C++ is enough at 300×. When the ratio is O(1), interleave or alternate first-engine; do not upgrade the canary into the 32-case sealed protocol.
- **dhat vs Instruments Allocations:** Prefer samply first. Enable private `dhat-heap` or xctrace Allocations only if CPU samples show allocator/`Vec` dominance. dhat’s global allocator must stay off the gate binary.

## Sources

### Project research (this milestone)

- [STACK.md](STACK.md) — samply 0.13.1, `[profile.profiling]`, evidence path, what not to add.
- [FEATURES.md](FEATURES.md) — PERF-* table stakes, anti-features, admission, WASM-after-native.
- [ARCHITECTURE.md](ARCHITECTURE.md) — just/xtask/oracle layering, measure-then-fix loop, package isolation (evidence dir reconciled away from `v12-native-perf`).
- [PITFALLS.md](PITFALLS.md) — SIMD-first, profiled-as-gate, clone-happy-path, Phase 12 revival, WASM-vs-C++.
- [PROJECT.md](../PROJECT.md) — v1.2 goal, Dam Break ≤ 3×, scalar baseline, hobby scope.

### Primary (HIGH confidence)

- Repo: `docs/playground-dam-break-timing.md`, `BENCHMARKING.md`, `PROJECT-SCOPE.md`, `reference/performance/manifest.toml` (`reviewed_reports = []`), `reference/performance/policy.json` (`timing_authority: unprofiled_wall_clock`).
- Repo: `tools/xtask/src/playground.rs`, `crates/liquidfun-wasm/src/dam_break_bench.rs`, `tools/reference/src/playground_dam_break_bench.cpp`, `crates/liquidfun` step vs `step_profiled`.
- [mstange/samply 0.13.1](https://github.com/mstange/samply/blob/samply-v0.13.1/README.md) — macOS, release+debug info, `--save-only`, `--unstable-presymbolicate`.
- [crates.io samply 0.13.1](https://crates.io/crates/samply) / [Homebrew samply](https://formulae.brew.sh/formula/samply) — official pin, Apple Silicon bottles.
- [box2d-rust 1.3.0 performance notes](https://docs.rs/crate/box2d-rust/latest) — paired C vs Rust, release-validator win before SIMD (2026-07-19).
- [Rapier common mistakes](https://rapier.rs/docs/user_guides/rust/common_mistakes/) — ~100× without `--release`.
- [The Rust Performance Book — Profiling](https://nnethercote.github.io/perf-book/profiling.html) — samply / Instruments / debuginfo.
- [xctrace(1)](https://keith.github.io/xcode-man-pages/xctrace.1.html) — headless `--launch --no-prompt`.

### Secondary (MEDIUM confidence)

- [Avian 0.4 write-up](https://joonaa.dev/blog/09/avian-0-4) — profile-guided parallel solver; **do not default this on**.
- [Erin Catto, SIMD for Collision (2026-07)](https://box2d.org/posts/2026/07/simd-for-collision/) — SIMD helps some hulls, not all scenes.
- [crates.io dhat 0.3.3](https://crates.io/crates/dhat) — optional heap rank; maintenance warning.
- [samply#763](https://github.com/mstange/samply/issues/763) — `debug = "limited"` can suffice; v1.2 still starts at `debug = true` per STACK.
- [Shnatsel, bounds checks](https://shnatsel.medium.com/how-to-avoid-bounds-checks-in-rust-without-unsafe-f65e618b4c1e) — typical 1–15%; not a 300× explanation.
- cargo-flamegraph 0.6.14 / cargo-instruments 0.4.17 — optional SVG/GUI; not the scripted default.

### Tertiary (LOW confidence)

- Third-party “Rust vs C++ game physics” roundups — methodology not LiquidFun-shaped; do not drive gates.

### Negative pins (do not use)

- [feldera/samply v0.13.2](https://github.com/feldera/samply/releases/tag/v0.13.2) — different repo, not crates.io `samply`.
- Valgrind / iai-callgrind — not a supported macOS aarch64 hobby tool.

---
*Research completed: 2026-09-20*
*Ready for roadmap: yes*
