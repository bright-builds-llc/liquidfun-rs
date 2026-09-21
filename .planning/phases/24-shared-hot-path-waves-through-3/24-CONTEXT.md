---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T15:03:30.003Z
---

# Phase 24: Shared hot-path waves through 3× - Context

**Gathered:** 2026-09-21
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

A developer can land evidenced scalar fixes in shared `liquidfun` particle/rigid stepping until the unprofiled Dam Break Medium pair is ≤ 3× pinned C++, while other playground scenes stay usable and the scalar deterministic baseline is unchanged.

This phase consumes the Phase 23 named shares and closes PERF-ADMIT, PERF-SHARED, PERF-GATE, PERF-BASELINE, PERF-SPOT, and PERF-CANARY2. It does not treat samply / `[profile.profiling]` / `step_profiled` / dhat timings as the Dam Break ratio, fill `reference/performance/manifest.toml`, revive the Phase 12 sealed matrix, compare WASM to C++, publish a crate, or start SIMD / Rayon / `unsafe` indexing.

</domain>

<decisions>
## Implementation Decisions

### First wave: `particle_rows` shape mismatch
- **D-01:** The first admitted wave targets `liquidfun::particle::contact::particle_rows` (~93.6% self in `docs/native-performance-audit.md`). Treat it as an algorithm/shape mismatch versus C++ `FindContacts_Reference` `Proxy.index` / integer `indexA`/`indexB`, not as a SIMD, HashMap-cache, PGO, or `unsafe` indexing problem.
- **D-02:** Carry dense row indices through shared neighborhood/proxy/contact generation so candidate pairs can index `positions`/`flags` without `particle_ids().iter().position(...)`. `validate_pairs` and `listener_effects` must not re-introduce the same O(n) identity scan as the hot path.
- **D-03:** Keep public `ParticleContact` and `ParticleNeighborPair` on stable `ParticleId`. Dense rows are an internal stepping representation. Do not change the public handle API unless a correctness gate proves it is required.

### Wave admission and cadence
- **D-04:** One named concern per wave. Land only when evidence names a function or typed bottleneck with non-trivial profile share, the unprofiled Dam Break Medium ratio improves on the same host and `just playground-dam-break-bench` recipe, existing native tests and relevant differential/determinism checks still pass, and the scene particle count is not lowered. A physics mismatch is a failed candidate, never a faster sample.
- **D-05:** Re-pair unprofiled into a **new** exclusive `target/dam-break-perf/<utc-stamp>/`. Never overwrite, merge, or delete an existing stamp. Preserve failed records in their own stamps/notes. samply retargets the next wave from a sibling profile stamp; profiled duration remains `not_timing_authority`.
- **D-06:** Stop landing further ranked frames once the unprofiled pair is ≤ 3×. Do not gold-plate leftover ~2% frames after the gate.

### Later ranked frames
- **D-07:** After the `particle_rows` wave, retarget samply and admit the next leftover only if it still has non-trivial named share. Candidate order from the Phase 23 table, not a hunt-list paste: release `ParticleStorage::check_invariants` / `slice_contains`, then `replace_solver_candidate` clone, then `ParticleNeighborhood::from_view`, then per-step `Vec<ParticleContact>` collect / `RawVec` grow, then `listener_effects` re-scans.
- **D-08:** Gating `check_invariants` behind `debug_assertions` (C++ `NDEBUG` analog) is a later wave only. Keep typed construction/API errors fail-closed; do not silently drop user-facing checked mutation. Do not do this first — 2.2% will not close ~328×.
- **D-09:** If ranked waves do not reach ≤ 3×, continue additional samply-retargeted **scalar** waves on newly named extra work. Do not complete the phase without PERF-GATE. Still forbid default Rayon, SIMD, `-ffast-math`, `-march=native`, and lifting workspace `unsafe_code = "forbid"`. Criterion micros are allowed only if a named kernel remains after the pair is already near 3×.

### Shared-path and cheat ban
- **D-10:** Admitted fixes land in shared `liquidfun` particle/rigid stepping (neighborhood/proxy, contacts, particle–body coupling, pressure/damping/integrate, rigid contact solve) so other particle/rigid scenes can benefit. Dam Break-only scene hacks, WASM-copy shortcuts, skipped-solver paths, or shrinking the 1920-particle Medium recipe do not satisfy PERF-SHARED.

### Spot-checks and second canary
- **D-11:** After shared-path fixes, spot-check Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel with native `--release` headless stepping through the existing `liquidfun-wasm` scene builders. Record wall ms or ms/step into a new exclusive evidence stamp plus a short committed unreviewed notes table. No per-scene C++ pair and no Phase 12 case hashes. Improvement or non-catastrophic regression is acceptable; hangs, timeouts, or broken scenes fail the wave.
- **D-12:** After PERF-GATE, default to an explicit notes statement that Dam Break and the spot-checks share the same dominant cluster when all five remain particle-contact dominated. Run a second native headless canary/profile only if a spot-check’s time or failure mode suggests a different cluster. This is not a sealed 32-case matrix.

### Correctness gates per wave
- **D-13:** Each landed wave must keep `cargo test -p liquidfun` green plus existing particle contact/neighborhood/solver tests and relevant determinism/differential checks already in the repository. Do not require a Linux oracle, Phase 12 sealed matrix, sanitizer, or full C++ samply wrap per wave. Independent AI review remains required after phase verification; the implementing agent must not self-approve.

### Evidence honesty and isolation
- **D-14:** Demonstrate PERF-GATE with unprofiled `just playground-dam-break-bench` on the same host: host, git HEAD, compilers, both wall times, and ratio in a new dated stamp. Only `pair.json` unprofiled walls are the 3× number. Refresh `docs/playground-dam-break-timing.md` and named-audit notes as unreviewed local samples. Do not copy the ratio into `reference/performance/manifest.toml`. Do not write README or crates.io “Rust is N×” claims.
- **D-15:** `liquidfun` stays bitflags-only. No profiler, samply, dhat, CMake, or serde dependency. Workspace `unsafe_code = "forbid"` stays. `just` remains a one-line printer if new recipes appear; xtask owns orchestration. Raw `.json.gz` / `.trace` / `dhat-heap.json` stay gitignored.

### Claude's Discretion
- Exact internal dense-index types and whether neighborhood pairs store `(row, row)` privately while still exposing `ParticleId`.
- Exact wave split if `particle_rows` itself needs more than one commit (generation vs listener vs validate).
- Exact headless spot-check binary/recipe name and notes table layout.
- How many leftover frames to list after the gate, as long as remaining-delta narrative is honest.
- Whether to keep a one-line pointer to the Phase 23 MEASURED_HEAD `6d98531…` 327.53× sample after later pair refreshes.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked milestone policy
- `.planning/ROADMAP.md` — Phase 24 goal, success criteria, one-concern-per-wave, new dated pair stamps, Criterion-only-near-3×, no ratio in `manifest.toml`.
- `.planning/REQUIREMENTS.md` — `PERF-ADMIT`, `PERF-SHARED`, `PERF-GATE`, `PERF-BASELINE`, `PERF-SPOT`, `PERF-CANARY2` (this phase); `PERF-NOTES` / `PERF-WASM` are Phase 25.
- `.planning/PROJECT.md` — Native-vs-C++ Dam Break canary, scalar baseline, honest claims, no package publication.
- `.planning/STATE.md` — Plan from named shares; start with `particle_rows`; do not pre-select SIMD, PGO, or `unsafe` indexing.
- `PROJECT-SCOPE.md` — Hobby scope; local checks plus one macOS Cargo CI job; no dedicated performance-host gate.

### Named extra work (must consume, not reinvent)
- `docs/native-performance-audit.md` — SHA-bound ranking: `particle_rows` ~93.6% self vs `FindContacts_Reference` `Proxy.index`; later frames; **Not found** SIMD/Rayon/PGO/`unsafe`.
- `docs/playground-dam-break-timing.md` — Locked Medium recipe (1920 particles, 60+600 steps); unreviewed sample; refresh after gate pairs.
- `.planning/phases/23-baseline-pair-and-named-audit/23-CONTEXT.md` — Live pair/profile isolation, named-function taxonomy, no kernel edits in Phase 23.
- `.planning/phases/22-observability-shell/22-CONTEXT.md` — Exclusive stamps, unprofiled 3× authority, `just` one-line aliases.

### Measurement, honesty, and isolation
- `BENCHMARKING.md` — Exploratory Dam Break pair is unreviewed local diagnosis; unprofiled wall-clock is timing authority; do not fill `reviewed_reports`.
- `reference/performance/policy.json` — `timing_authority: unprofiled_wall_clock`.
- `reference/performance/manifest.toml` — Must stay empty.
- `.planning/research/PITFALLS.md` — Pitfall 1 (SIMD-first), 2 (blessing numbers), 3 (wrong construction), 4 (profiled timings as 3×), 5 (debug vs release).
- `.planning/research/FEATURES.md` — PERF-ADMIT / PERF-SHARED / PERF-GATE / PERF-BASELINE / PERF-SPOT / PERF-CANARY2 seeds.
- `.planning/research/SUMMARY.md` — Shared particle/rigid surface; Dam Break-only cheats do not close.
- `Cargo.toml` — Workspace `unsafe_code = "forbid"`; `liquidfun` stays bitflags-only.

### Existing implementation seams
- `crates/liquidfun/src/particle/contact.rs` — `particle_rows` linear `ParticleId` scan inside `ParticleContactUpdate::generate`, `validate_pairs`, and `listener_effects`.
- `crates/liquidfun/src/particle/proxy.rs` — `Proxy` stores `ParticleId` not dense row; `ParticleNeighborhood::from_view` rebuilds pairs each step.
- `crates/liquidfun/src/particle/storage/` — `check_invariants` / `replace_solver_candidate` ranked later frames.
- `crates/liquidfun-wasm/src/bin/dam_break_bench.rs` — Unprofiled native `World::step` timer; do not add `step_profiled` here.
- `crates/liquidfun-wasm/src/scene.rs` — Fountain / Float or Sink / Color Mixer / Jelly Drop / Water Wheel builders for headless spot-checks.
- `justfile` — `playground-dam-break-bench` / `-profile` / `-timers` / `-audit-bundle` / `-heap` one-line aliases.
- `tools/xtask/src/playground/stamp.rs` — Exclusive mint; never overwrite an existing stamp.
- `tools/xtask/src/playground/pair.rs` — Unprofiled pair persist; 3× authority.

### Inherited phase context
- `.planning/phases/12-performance-portability-and-release-hardening/12-CONTEXT.md` — Unprofiled wall-clock totals are authoritative; profile durations are diagnostics.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-CONTEXT.md` — Stable `ParticleId` handles and dense storage; public identities stay typed.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 23 audit already names `particle_rows` as ~93.6% self with a C++ shape counterpart; Phase 24 must start there.
- `just playground-dam-break-bench` persists exclusive unprofiled `pair.json` / `pair.md`.
- `just playground-dam-break-profile` writes sibling `rust.json.gz` with `not_timing_authority`.
- Exclusive stamp mint fails closed on `AlreadyExists`.
- Playground scene modules already construct Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel worlds.

### Established Patterns
- Public particle APIs expose `ParticleId`; dense rows stay private behind storage/view.
- `just` is a one-line alias; xtask owns orchestration.
- Pair, profile, timer, heap, and bundle artifacts stay in separate exclusive stamps.
- Package isolation: `liquidfun` is the sole default member and must not grow tooling deps.
- Evidence that must not be a public claim stays gitignored or labeled unreviewed.

### Integration Points
- First kernel edit is `crates/liquidfun/src/particle/contact.rs` plus `proxy.rs` so pairs can carry dense rows.
- Do not special-case `crates/liquidfun-wasm/src/scene/dam_break.rs` or WASM frame copies to fake the gate.
- Spot-checks reuse `crates/liquidfun-wasm/src/scene.rs` builders with native `--release` headless stepping.
- Gate proof is a new unprofiled pair stamp, not a samply duration.

</code_context>

<specifics>
## Specific Ideas

- C++ `FindContacts_Reference` calls `AddContact(a->index, b->index)` from `Proxy.index`; Rust currently re-scans `particle_ids` per candidate. Match that index-preserving shape in safe scalar Rust.
- In-tree hunt list (full-world clone, `to_vec`, `from_view`) is ranked only where Phase 23 gave it share. Do not paste unranked suspects as wave 1.
- Locked recipe stays 1920 particles, 60 warmup + 600 timed steps. Do not shrink the scene to hit 3×.
- Remaining-delta notes that belong in README/crates.io or WASM-vs-C++ belong to Phase 25, not here.

</specifics>

<deferred>
## Deferred Ideas

- WASM/playground sanity versus previous WASM or native Rust, never versus C++ — Phase 25 (`PERF-WASM`).
- Remaining Dam Break delta narrative in committed close notes / README honesty — Phase 25 (`PERF-NOTES`).
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in after the scalar canary (`PERF-SIMD` / `PERF-UNSAFE`).
- Filling `reference/performance/manifest.toml` or writing README speed claims — out of this milestone.
- Required C++ samply wrap / `cpp.json.gz` — not needed to land the first waves.
- Phase 12 sealed 32-case public matrix — conflicts with this milestone’s gate-as-canary rule.

</deferred>

---

*Phase: 24-shared-hot-path-waves-through-3*
*Context gathered: 2026-09-21*
