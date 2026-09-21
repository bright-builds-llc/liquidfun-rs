---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T02:46:17.359Z
---

# Phase 23: Baseline pair and named audit - Context

**Gathered:** 2026-09-21
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

A developer can read a committed audit that names dominating extra work from a SHA-bound unprofiled Dam Break pair and samply CPU profile, and can run a private heap dump only when that profile shows allocator or `Vec` time.

This phase delivers named-function notes plus optional private dhat. It does not close the 3× gate, edit shared particle/rigid kernels, revive Phase 12 public claims, compare WASM to C++, or treat samply / `[profile.profiling]` / `step_profiled` / dhat timings as the Dam Break ratio.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked milestone policy
- `.planning/ROADMAP.md` — Phase 23 goal, success criteria, hunt-list note, no samply-duration-as-3×, no committed profile blobs.
- `.planning/REQUIREMENTS.md` — `PERF-AUDIT`, `PERF-HEAP` (this phase); `PERF-PAIR` / `PERF-PROFILE` / `PERF-TIMERS` already complete in Phase 22.
- `.planning/PROJECT.md` — Native-vs-C++ Dam Break canary, scalar baseline, honest claims, no package publication.
- `.planning/STATE.md` — Unprofiled `just playground-dam-break-bench` is 3× authority; samply / profiling / `step_profiled` / dhat timings are never that number.
- `PROJECT-SCOPE.md` — Hobby scope; local checks plus one macOS Cargo CI job; no dedicated performance-host gate.

### Measurement, honesty, and isolation
- `BENCHMARKING.md` — Exploratory Dam Break pair is unreviewed local diagnosis; unprofiled wall-clock is timing authority; do not fill `reviewed_reports`.
- `docs/playground-dam-break-timing.md` — Locked Medium recipe (1920 particles, 60+600 steps); refresh as unreviewed sample, not a Phase 12 claim.
- `reference/performance/policy.json` — `timing_authority: unprofiled_wall_clock`.
- `reference/performance/manifest.toml` — Must stay empty.
- `.planning/research/PITFALLS.md` — Pitfall 1 (SIMD-first), pitfall 2 (blessing exploratory numbers), pitfall 4 (profiled timings as 3×), pitfall 5 (debug vs release), hunt-list extra-work targets.
- `.planning/research/ARCHITECTURE.md` — `docs/native-performance-audit.md` as durable human artifact; profiles expire; names belong in git.
- `.planning/research/FEATURES.md` — PERF-AUDIT / PERF-HEAP seeds; named-function notes; dhat only after allocator/`Vec` time.
- `.planning/research/STACK.md` — `dhat` 0.3.3 private `dhat-heap` on `dam-break-bench` only; samply 0.13.1; never add dhat to `liquidfun`.
- `.planning/research/SUMMARY.md` — Dominant kernel unknown until this audit; hunt list is not a committed cause.

### Existing implementation seams
- `justfile` — One-line `playground-dam-break-bench` / `playground-dam-break-profile` / `playground-dam-break-timers` aliases.
- `tools/xtask/src/playground.rs` — Existing playground subcommand dispatch.
- `tools/xtask/src/playground/pair.rs` — Unprofiled pair persist into exclusive stamps.
- `tools/xtask/src/playground/profile.rs` — samply capture into a sibling stamp with `not_timing_authority`.
- `tools/xtask/src/playground/stamp.rs` — Exclusive mint; never overwrite an existing stamp.
- `crates/liquidfun-wasm/Cargo.toml` — Private crate; `dam-break-bench` bin; no dhat feature yet.
- `crates/liquidfun-wasm/src/bin/dam_break_bench.rs` — Unprofiled native `World::step` timer; do not enable dhat on the default gate path.
- `crates/liquidfun/src/particle/proxy.rs` — `ParticleNeighborhood::from_view` hunt-list suspect.
- `Cargo.toml` — Workspace `[profile.profiling]`; `unsafe_code = "forbid"`; `liquidfun` stays bitflags-only.
- `.gitignore` — `/target/` already covers dated evidence.

### Inherited phase context
- `.planning/phases/22-observability-shell/22-CONTEXT.md` — Exclusive stamps, pair vs profile isolation, fake-tool tests for the shell, named audit and dhat deferred here.
- `.planning/phases/12-performance-portability-and-release-hardening/12-CONTEXT.md` — Unprofiled wall-clock totals are authoritative; profile durations are diagnostics.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `just playground-dam-break-bench` already persists `pair.json` / `pair.md` under exclusive `target/dam-break-perf/<utc-stamp>/`.
- `just playground-dam-break-profile` already rebuilds `[profile.profiling]` and writes `rust.json.gz` plus `profile-identity.json` into a **sibling** stamp with no `pair.json`.
- Exclusive stamp mint in `tools/xtask/src/playground/stamp.rs` fails closed on `AlreadyExists`.
- `docs/playground-dam-break-timing.md` already has the honesty banner and locked Medium recipe.

### Established Patterns
- `just` is a one-line alias; xtask owns orchestration.
- Pair, profile, and timer artifacts stay in separate exclusive stamps so profiled/timer walls cannot contaminate `pair.json`.
- Package isolation: `liquidfun` is the sole default member and must not grow tooling deps.
- Evidence that must not be a public claim stays gitignored or labeled unreviewed.

### Integration Points
- New audit-bundle command copies same-HEAD pair + profile artifacts into a new exclusive stamp.
- Optional `dhat-heap` feature belongs on `crates/liquidfun-wasm` `dam-break-bench` only, gated behind a separate xtask/heap recipe so the default `--release` gate binary stays clean.
- Committed notes: new `docs/native-performance-audit.md` plus refresh of `docs/playground-dam-break-timing.md`.
- Do not wire dhat or samply into `crates/liquidfun`.

</code_context>

<specifics>
## Specific Ideas

- Research still mentions `target/v12-native-perf/`; the locked path is `target/dam-break-perf/<utc-stamp>/`.
- In-tree hunt list is a starting map for researchers reading the profile, not a ranked cause list to paste into the audit.
- samply setup / signing on this Mac is host-specific; missing samply still fails closed with Phase 22 install text. Live capture is required here even though Phase 22 allowed fake-tool DoD.

</specifics>

<deferred>
## Deferred Ideas

- Shared hot-path physics edits and the ≤ 3× gate — Phase 24.
- Spot-checks of other playground scenes and second-canary-or-same-cluster note — Phase 24.
- WASM/playground sanity versus previous WASM or native Rust, never versus C++ — Phase 25.
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in after the scalar canary.
- Required C++ samply wrap / `cpp.json.gz` — not needed to name Rust extra work; optional later if a shape mismatch needs a C++ flame.
- Criterion micros of a named kernel — only after the pair is already near 3× (Phase 24 note).
- Filling `reference/performance/manifest.toml` or writing README speed claims — out of this milestone.

</deferred>

---

*Phase: 23-baseline-pair-and-named-audit*
*Context gathered: 2026-09-21*
