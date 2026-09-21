# Phase 24: Shared hot-path waves through 3× - Research

**Researched:** 2026-09-21
**Domain:** Scalar particle-contact shape match vs pinned LiquidFun `FindContacts_Reference`, then evidenced leftover waves until unprofiled Dam Break Medium ≤ 3× C++
**Confidence:** HIGH for wave-1 shape, isolation, and gate recipes; MEDIUM for how many leftover waves remain after `particle_rows` (samply leaf share is not a wall-clock identity)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Deferred Ideas (OUT OF SCOPE)
- WASM/playground sanity versus previous WASM or native Rust, never versus C++ — Phase 25 (`PERF-WASM`).
- Remaining Dam Break delta narrative in committed close notes / README honesty — Phase 25 (`PERF-NOTES`).
- SIMD / Rayon / relaxing `unsafe_code = "forbid"` — later opt-in after the scalar canary (`PERF-SIMD` / `PERF-UNSAFE`).
- Filling `reference/performance/manifest.toml` or writing README speed claims — out of this milestone.
- Required C++ samply wrap / `cpp.json.gz` — not needed to land the first waves.
- Phase 12 sealed 32-case public matrix — conflicts with this milestone’s gate-as-canary rule.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERF-ADMIT | Land a hot-path change only when evidence names a function or typed bottleneck with non-trivial profile share, the unprofiled Dam Break Medium ratio improves on the same host/recipe, existing native tests and relevant differential/determinism checks still pass, and the scene particle count is not lowered. A physics mismatch is a failed candidate, never a faster sample. | Wave protocol: one named concern; new exclusive unprofiled `pair.json`; `cargo test -p liquidfun` plus `particle_contacts` / permutation / solver tests; keep 1920 particles. [VERIFIED: D-04/D-05; `docs/native-performance-audit.md`; `just playground-dam-break-bench`] |
| PERF-SHARED | Admitted fixes land in shared `liquidfun` particle/rigid stepping so other particle/rigid scenes benefit. Dam Break-only scene, WASM-copy, or skipped-solver cheats do not satisfy this. | First kernel edit is `crates/liquidfun/src/particle/{proxy,contact}.rs` used by `World::run_particle_solver` / `update_particle_contacts`. Do not special-case `scene/dam_break.rs`. [VERIFIED: `particle_coupling.rs` 190–217; D-10] |
| PERF-GATE | Native Dam Break Medium unprofiled wall ≤ 3× pinned C++ on the same host under scalar `--release` vs `oracle-release` for the locked 60+600-step pair; record host, HEAD, compilers, both walls, ratio; `manifest.toml` stays empty. | Gate command is unprofiled `just playground-dam-break-bench`. Baseline sample is 327.53× at MEASURED_HEAD `6d98531…`. Wave 1 is necessary and likely insufficient; continue D-07/D-09 leftover waves until `pair.json` ratio ≤ 3. [VERIFIED: `docs/playground-dam-break-timing.md`; `reference/performance/manifest.toml` `reviewed_reports = []`] |
| PERF-BASELINE | Keep the scalar deterministic compatibility baseline: no default Rayon or SIMD, no `-ffast-math` / `-march=native` on the pair, no lifting `unsafe_code = "forbid"`. | Workspace lint already forbids `unsafe`. `liquidfun` deps are bitflags-only. Do not add `std::simd`, Rayon, or native-CPU flags. [VERIFIED: root `Cargo.toml` `unsafe_code = "forbid"`; `crates/liquidfun/Cargo.toml`] |
| PERF-SPOT | Spot-check Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel with native `--release` headless stepping after shared-path fixes; record improvement or non-catastrophic regression. | Reuse `SessionCore::create(SceneId::…)` + `advance(1)` loops. New exclusive stamp + committed unreviewed notes table. No per-scene C++ pair. [VERIFIED: `crates/liquidfun-wasm/src/scene.rs`; D-11] |
| PERF-CANARY2 | After PERF-GATE, record either a second native headless canary whose profile cluster differs from Dam Break, or an explicit notes statement that Dam Break and the spot-checks share the same dominant cluster. | Default to the same-cluster note when all five remain particle-contact dominated. Second samply only if a spot-check time/failure mode differs. Not a 32-case matrix. [VERIFIED: D-12; Phase 23 named table is particle-contact dominated] |
</phase_requirements>

## Summary

Phase 24 is the first physics-kernel phase of v1.2. Phase 23 already named the extra work: `particle_rows` is ~93.6% of Dam Break samply self time because every neighborhood candidate re-scans `view.particle_ids()` with `.position(...)` to recover the dense row that C++ never lost. Pinned LiquidFun `FindContacts_Reference` stores `Proxy { int32 index; uint32 tag; }` and calls `AddContact(a->index, b->index)`, which indexes `m_positionBuffer.data[a]` directly. Rust already has that storage shape as private `ParticleProxy { index: ParticleIndex, tag: u32 }`, but public neighborhood construction builds a different `Proxy { particle: ParticleId, tag }` and throws the row away.

The first wave must restore the C++ index-preserving shape in **safe scalar Rust** inside shared `liquidfun` contact generation. Public `ParticleId` handles stay. Do not introduce SIMD, Rayon, HashMap caches of the hot path, PGO, or `unsafe` indexing. After each landed wave, re-pair **unprofiled** into a **new** exclusive stamp; samply only retargets the next named leftover. Stop at ≤ 3×.

**Primary recommendation:** In wave 1, put dense `ParticleIndex` on the neighborhood `Proxy`, keep a private aligned row pair for each public `ParticleNeighborPair`, and make `ParticleContactUpdate::generate` / `validate_pairs` / `listener_effects` index SoA lanes in O(1). Use the existing generational `resolve_live` map for previous public `ParticleContact` IDs (rows go stale after compaction). Then re-pair; expect leftover scalar waves before PERF-GATE.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. [VERIFIED: glob `.cursor/rules/**/*` returned 0 files]

Honor instead:

- `AGENTS.md` Repo-Local Guidance: hobby scope (`PROJECT-SCOPE.md`), standing autonomous iteration, independent AI review (implementer must not approve own work). Package publication remains separately authorized. Do not format `.planning/**` with mdformat.
- `AGENTS.bright-builds.md`: functional core, `maybe_` naming, `foo.rs` plus `foo/` (no new `mod.rs`), files over ~628 physical lines are a refactor trigger (`bun scripts/bright-builds-check.ts file-lengths` fails at 629).
- `standards-overrides.md`: hobby scope and independent AI review; no Rust exceptions that license SIMD/`unsafe` for this phase.
- `standards/languages/rust.md`: `let...else`, no `unwrap()` in production, `maybe_` for `Option`.
- `standards/core/architecture.md`: dense rows stay private domain types; parse/validate at the neighborhood boundary once.
- `standards/core/code-shape.md`: early returns; do not grow `tools/xtask/src/playground/pair.rs` (616) or `crates/liquidfun/src/particle/storage/lifecycle.rs` (620) past 628.
- `standards/core/verification.md`: sync first; run affected `cargo test -p liquidfun` / clippy / Bright Builds before commit.
- `standards/core/testing.md`: unit tests Arrange/Act/Assert, one concern per test. Add contact/neighborhood cases to `crates/liquidfun/tests/particle_contacts.rs` (320 lines; split to `tests/particle_contacts/` if it would exceed 628).

## Standard Stack

This phase adds **no new production crates**. Use the existing workspace.

### Core

| Library / tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| `liquidfun` | workspace, bitflags-only | Shared particle/rigid kernels | PERF-SHARED / D-15. [VERIFIED: `crates/liquidfun/Cargo.toml` deps = `bitflags` only] |
| `bitflags` | 2.13.0 (workspace) | Particle flag masks | Existing production dep; keep unknown bits. [VERIFIED: root `Cargo.toml`] |
| Rust | 1.97.0 pinned; MSRV 1.92 | `--release` gate binary | Phase 23 pair used `rustc 1.97.0 (2d8144b78 2026-07-07)`. [VERIFIED: `docs/playground-dam-break-timing.md`] |
| `unsafe_code` | `"forbid"` workspace lint | Safe scalar baseline | PERF-BASELINE / D-09 / D-15. [VERIFIED: root `Cargo.toml`] |

### Supporting (already in-tree; do not add to `liquidfun`)

| Library / tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `proptest` | 1.11.0 (dev) | Neighborhood determinism | Extend existing `particle_contacts.rs` proptest; do not add to production graph. [VERIFIED: `crates/liquidfun/Cargo.toml` dev-deps] |
| `just` | local 1.48.0 (one-line printer) | Human aliases | New spot-check recipe must be `just …:` → `cargo xtask playground …` only. [VERIFIED: `justfile` 146–159] |
| xtask playground | existing modules | Exclusive stamps, unprofiled pair, samply | Reuse `stamp::mint_exclusive_stamp`; do not grow `pair.rs` (616). [VERIFIED: `tools/xtask/src/playground/`] |
| samply | 0.13.1 | Retarget next wave | Sibling profile stamp; `not_timing_authority`. [VERIFIED: host `samply --version`; D-05] |
| CMake / Ninja | local CMake 3.27.9, Ninja 1.13.2 | `oracle-release` C++ pair | Already used by `just playground-dam-break-bench`. [VERIFIED: PATH probe] |
| `dhat` 0.3.3 | optional on `dam-break-bench` bin | Heap leftover only if a later samply still names allocator/`Vec` | Never on `liquidfun`; never `--features dhat-heap` on the pair. [VERIFIED: Phase 23; D-15] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Carry dense rows on neighborhood proxies | `HashMap<ParticleId, usize>` built each step | Locked out by D-01 as HashMap-cache of the hot path; also slower than storing the row C++ already has. |
| Carry dense rows | `get_unchecked` / lift `unsafe_code` | Locked out. Bounds-check wins are 1–15%, not 300×. [CITED: `.planning/research/PITFALLS.md` Pitfall 1 / anti-feature table] |
| Scalar row carry | `FindContacts_Simd` / `std::simd` / Rayon | Locked out as wave 1 and as the canary closer (PERF-BASELINE / D-09). SIMD is **Not found** in the Phase 23 audit. |
| Unprofiled pair as 3× | samply / `step_profiled` / dhat duration | Locked out. `timing_authority: unprofiled_wall_clock`. [VERIFIED: `reference/performance/policy.json`] |
| Fuse enumerate+`AddContact` in wave 1 | Keep pair `Vec` + row carry | Fusing matches C++ more closely and may drop `from_view` / `RawVec` leftover, but it is a second concern. Wave 1 is `particle_rows` only (D-04). Fuse only if a later samply still names those frames. |

**Installation:** none. Implement in existing crates.

**Version verification:** `cargo 1.97.0`, `rustc 1.97.0`, `samply 0.13.1`, `ninja 1.13.2`, `just 1.48.0`, `cmake 3.27.9` on this host 2026-09-21. [VERIFIED: `command -v` / `--version`]

## Architecture Patterns

### Recommended Project Structure

```
crates/liquidfun/src/particle/
├── proxy.rs              # Wave 1: Proxy gains ParticleIndex; private pair_rows
├── contact.rs            # Wave 1: generate/validate/listener use rows, not .position
├── view.rs               # crate-internal maybe_live_row → storage.resolve_live
├── storage/
│   ├── lanes.rs          # already has ParticleProxy { index, tag } (C++ shape)
│   ├── lifecycle.rs      # bump resolve_live to pub(in crate::particle) only if needed
│   └── runtime.rs        # replace_particle_contacts already uses resolve_live (leave)
└── body_contact.rs       # NOT wave 1 unless a later samply still names particle_row

crates/liquidfun/tests/
└── particle_contacts.rs  # extend; split to particle_contacts/ if approaching 628

crates/liquidfun-wasm/src/
├── dam_break_bench.rs    # DO NOT add step_profiled; DO NOT shrink 1920
├── scene.rs              # reuse builders for spot-checks
└── bin/                  # optional new native-only spot-check bin (discretion)

tools/xtask/src/playground/
├── stamp.rs              # reuse exclusive mint
├── pair.rs               # DO NOT GROW (616 / 628)
└── spot.rs               # NEW module if a playground recipe is added
```

### Pattern 1: Index-preserving neighborhood (C++ `Proxy.index`)

**What:** Each spatial proxy keeps the dense row that produced it. Contact generation indexes `positions`/`flags` with that row. Public APIs still return `ParticleId`.
**When to use:** Wave 1 and any later neighborhood rewrite. Never store dense rows on public `ParticleContact` across steps — compaction remaps rows, IDs stay stable (Phase 9 D-01/D-02).
**Example (C++ shape to copy, not SIMD):**

```cpp
// Source: google/liquidfun 7f204021… b2ParticleSystem.h / b2ParticleSystem.cpp
struct Proxy {
  int32 index;
  uint32 tag;
};

inline void b2ParticleSystem::AddContact(int32 a, int32 b, ...) const {
  b2Vec2 d = m_positionBuffer.data[b] - m_positionBuffer.data[a];
  // ...
  contact.SetIndices(a, b);
  contact.SetFlags(m_flagsBuffer.data[a] | m_flagsBuffer.data[b]);
}

void b2ParticleSystem::FindContacts_Reference(...) const {
  for (const Proxy *a = beginProxy; a < endProxy; a++) {
    // ...
    AddContact(a->index, b->index, contacts);
  }
}
```

[CITED: github.com/google/liquidfun `7f20402173fd143a3988c921bc384459c6a858f2` `b2ParticleSystem.cpp` 1806–1849, `b2ParticleSystem.h` 763–767]

**Recommended Rust internals (discretion, locked public API):**

```rust
// Keep ParticleNeighborPair public fields as ParticleId only.
// Store rows privately on ParticleNeighborhood, aligned with pairs():
//   proxies: Vec<Proxy { particle: ParticleId, row: ParticleIndex, tag: u32 }>
//   pairs: Vec<ParticleNeighborPair>           // public IDs, PartialEq unchanged
//   pair_rows: Vec<[ParticleIndex; 2]>         // crate-internal, same length
```

Do **not** put rows on public `ParticleContact`. Previous contacts come from `semantic_particle_contacts()` as IDs; after compaction those IDs are valid and last-step rows are not.

### Pattern 2: O(1) ID → row for previous public contacts (not a HashMap cache)

**What:** `ParticleStorage::resolve_live` already maps `ParticleId` → `ParticleIndex` via the generational identity table (`local_slot` then `IdentityState::Live(dense)`). `replace_particle_contacts` already uses it. `particle_rows` does **not**.
**When to use:** `validate_pairs` of previous `ParticleContact`s and `listener_effects` old-contact flag lookup. Neighborhood **candidates** must still carry rows (D-02) so the hot loop does zero identity work.
**Why this is not D-01 HashMap-cache:** it is the existing arena identity map, not a new `HashMap` built from `particle_ids` each step. Do not add `HashMap<ParticleId, usize>` in `generate`.

[VERIFIED: `storage/lifecycle.rs` `resolve_live` 408–427; `storage/runtime.rs` `replace_particle_contacts` 237–256; `contact.rs` `particle_rows` 215–226]

### Pattern 3: One-concern wave with exclusive unprofiled re-pair

**What:** Land one named bottleneck; `cargo test -p liquidfun`; `just playground-dam-break-bench` into a **new** stamp; compare `pair.json` `rust_over_cpp_ratio` to the previous unprofiled stamp on the same host. If ratio does not improve, keep the failed stamp and revert or note failure — do not call it a win. If physics tests fail, it is a failed candidate.
**When to use:** Every wave including leftover D-07 frames.
**After a win:** `just playground-dam-break-profile` sibling stamp retargets the next name. Do not quote samply duration.

### Pattern 4: Spot-checks reuse `SessionCore`, not a Dam Break-only path

**What:** `SessionCore::create(SceneId::Fountain | …)` then loop `advance(1)` for a bounded native `--release` step count. Fountain/Color Mixer/etc. `on_advance` emits or stirs; Dam Break `on_advance` is a no-op. Using `SessionCore` is required so spot-checks include scene hooks. Do not call `World::step` while skipping hooks for those scenes.
**When to use:** After shared-path waves, before claiming PERF-SPOT.
**Binary/recipe (discretion):** Add `crates/liquidfun-wasm` native-only module + `[[bin]]` **or** an xtask `playground scene-spot` that `cargo run --release --bin …`. Keep `just` as a one-line alias. Do **not** put spot-check timing into `dam-break-bench` and do **not** enable `step_profiled`. Loop `advance(1)`; `MAX_ADVANCE_STEPS` stays 4.

[VERIFIED: `session.rs` 13, 100–122; `dam_break.rs` `on_advance` returns `Ok(())`; `fountain.rs` `on_advance` emits]

### Anti-Patterns to Avoid

- **SIMD / Rayon / `-march=native` / `-ffast-math` as wave 1 or as the 3× closer:** Locked out. Phase 23 **Not found**. [VERIFIED: `docs/native-performance-audit.md` Not found]
- **Leaving `validate_pairs` / `listener_effects` on `.position` scans:** D-02. `validate_pairs` currently scans every neighborhood pair **and** every previous contact before generate’s hot loop even runs. [VERIFIED: `contact.rs` 152–164, 166–213]
- **Storing last-step dense rows on public `ParticleContact`:** Compaction remaps rows; Phase 9 public IDs are the durable identity. [VERIFIED: `.planning/phases/09-…/09-CONTEXT.md` D-01/D-02]
- **Dam Break scene LOD, skipped solver, or shrinking 1920 particles:** PERF-SHARED / Pitfall 3. [VERIFIED: `dam_break.rs` `PARTICLE_COUNT = 48 * 40`]
- **Growing `pair.rs` (616) or `lifecycle.rs` (620):** Bright Builds fails at 629. New playground work goes in a new `foo.rs`. [VERIFIED: `wc -l`]
- **`cargo test --lib` on xtask / quoting profiled walls / filling `manifest.toml`:** Phase 22–23 lessons. [VERIFIED: STATE.md; `manifest.toml` `reviewed_reports = []`]
- **Self-approving the phase:** Independent AI review after `/gsd-verify-work`. [VERIFIED: AGENTS.md Independent review; D-13]
- **Inventing hot-function names:** Consume `docs/native-performance-audit.md` only. [VERIFIED: orchestrator instruction; D-07]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dense particle row type | New public `RowId` / `usize` in the public API | Existing `ParticleIndex` (`pub(in crate::particle)`) | Already the C++ `int32` analog; storage contacts already use `[ParticleIndex; 2]`. [VERIFIED: `storage.rs` 77; `lanes.rs` 36–41] |
| ParticleId → row | `particle_ids().iter().position` or `HashMap` | Neighborhood-carried rows for candidates; `resolve_live` for previous IDs | `.position` is the 93.6% leaf; `replace_particle_contacts` already proves `resolve_live`. |
| Exclusive evidence dirs | `latest/` clobber, `rm -rf` stamp | `stamp::mint_exclusive_stamp` | AlreadyExists bump, 8-attempt cap, tested. [VERIFIED: `tools/xtask/src/playground/stamp.rs`] |
| Dam Break 3× number | New timer, Criterion, samply wall | `just playground-dam-break-bench` → `pair.json` | Locked recipe 1920 / 60+600 / `--release` vs `oracle-release`. |
| Spot-check worlds | Duplicate scene construction in xtask | `SessionCore::create(SceneId)` | Builders already exist; Fountain emission lives in `on_advance`. |
| Package isolation check | Ad-hoc grep | `cargo xtask package verify` + `cargo tree -p liquidfun --edges normal` | Phase 22/23 isolation gate. |
| File-length exceptions | Edit managed Bright Builds checker | Split to `foo.rs` + `foo/` | Cap 628; TSV only for reasoned exact files. |

**Key insight:** The 300× gap is extra identity work on a path C++ never takes, not a missing SIMD backend. Storage already speaks `ParticleIndex`; neighborhood/contact generation forgot to carry it.

## Common Pitfalls

### Pitfall 1: SIMD-first (or HashMap-cache / `unsafe` indexing) as wave 1

**What goes wrong:** Plans reach for `std::simd`, Rayon, `get_unchecked`, or `HashMap<ParticleId, usize>` while `particle_rows` still scans. Ratio stays hundreds of times off, or determinism breaks.
**Why it happens:** 327× looks like “languages.” Phase 23 already classified this as algorithm/shape vs `Proxy.index`.
**How to avoid:** Wave 1 is dense rows on neighborhood proxies. SIMD stays future `PERF-SIMD`.
**Warning signs:** `rayon` / `std::simd` in `liquidfun`; `unsafe` blocks; new HashMap in `generate`.

[CITED: `.planning/research/PITFALLS.md` Pitfall 1; D-01]

### Pitfall 2: Fixing only `generate`’s hot loop

**What goes wrong:** `particle_rows` is deleted from the distance loop but `validate_pairs` still scans every pair (and every previous contact) first. samply still shows `particle_rows`. `listener_effects` still maps **all** previous contacts through `particle_rows` before the listener-flag filter.
**Why it happens:** The leaf is one function with three call sites. [VERIFIED: `contact.rs` 112, 158–161, 174]
**How to avoid:** D-02 is all three sites. Discretion allows 1–3 atomic commits under one concern, not three separate admission waves.

### Pitfall 3: Treating samply 93.6% as “wave 1 hits 3×”

**What goes wrong:** Planner schedules only `particle_rows` then PERF-GATE. Arithmetic from **leaf sample fraction × 71002 ms / 217 ms** suggests leftover on the order of ~20× if other original frames stay, which is still above 3×. `[ASSUMED]` converting samply self-share to wall share; overlap and inlining can move this.
**Why it happens:** 93.6% looks like the whole gap.
**How to avoid:** Plan contingent leftover waves (D-07/D-09). Re-profile after wave 1. Do not pre-implement `check_invariants` gating in the same plan as wave 1 (D-08: 2.2% will not close 328×).
**Warning signs:** A single plan titled “close 3×” that only edits `contact.rs`.

### Pitfall 4: Profiled timings as the 3× number

**What goes wrong:** `[profile.profiling]` or `dam-break-timers` looks ≤3× while unprofiled `pair.json` is not.
**How to avoid:** Only `pair.json` `rust_over_cpp_ratio`. Profile stamps keep `not_timing_authority`.
**Warning signs:** Mixing `step_profiled` into `dam_break_bench.rs`; quoting samply duration in the timing doc as the gate.

[CITED: PITFALLS.md Pitfall 4; D-14]

### Pitfall 5: Wrong construction / shrinking the recipe

**What goes wrong:** Spot-checks or “faster” pairs use Small water (25×26), fewer steps, debug binaries, or `oracle-debug`.
**How to avoid:** Gate stays Medium 1920, 60 warmup + 600 timed, `--release` vs `oracle-release`. Spot-checks may use a **shorter** step count but must not be sold as the 3× number.
**Warning signs:** `PARTICLE_COUNT` edits; `--warmup 0` as the reported gate; `cargo run` without `--release`.

[VERIFIED: `dam_break.rs` 16–20; `dam_break_bench.rs` 12–45]

### Pitfall 6: Physics mismatch treated as a faster sample

**What goes wrong:** Contact order, filter/listener multiplicity, or compaction weights change. Dam Break still “looks like water.” `particle_contacts` / permutation tests fail or other scenes hang.
**How to avoid:** D-04/D-13. Preserve pair enumeration order (`enumerate_pairs` tag walk), contact field bit patterns (`inverse_sqrt` weight/normal), listener begin-then-ends multiplicity (`particle_contacts.rs` 264–319). Failed physics = failed candidate; keep the stamp.
**Warning signs:** “Tests still pass” meaning only the bench binary ran.

### Pitfall 7: Growing near-cap files / new `mod.rs`

**What goes wrong:** `pair.rs` 616, `lifecycle.rs` 620, Bright Builds 629. `foo/mod.rs` violates Rust standards.
**How to avoid:** New playground command = new `playground/spot.rs`. Touch `lifecycle.rs` only to widen `resolve_live` visibility if a one-line change; otherwise add `maybe_live_row` on `ParticleSystemView` in `view.rs` (478 lines).

### Pitfall 8: Blessing numbers / filling the manifest

**What goes wrong:** Gate ratio copied into `reference/performance/manifest.toml` or README.
**How to avoid:** Refresh `docs/playground-dam-break-timing.md` and audit notes as **unreviewed local samples**. Leave `reviewed_reports = []`. Remaining-delta README copy is Phase 25.

[VERIFIED: `manifest.toml`; BENCHMARKING.md]

## Code Examples

### Current extra work (do not keep on the hot path)

```rust
// Source: crates/liquidfun/src/particle/contact.rs (this checkout)
fn particle_rows(
    view: &ParticleSystemView<'_>,
    particles: [ParticleId; 2],
) -> Result<[usize; 2], ParticleContactError> {
    let row_for = |particle| {
        view.particle_ids()
            .iter()
            .position(|candidate| *candidate == particle)
            .ok_or(ParticleContactError::MissingParticle)
    };
    Ok([row_for(particles[0])?, row_for(particles[1])?])
}
```

[VERIFIED: `contact.rs` 215–226]

### Wave-1 generate inner loop (target shape)

```rust
// Recommended: zip neighborhood.pairs() with private pair_rows.
// Source pattern: C++ AddContact(a->index, b->index)
let [a, b] = pair_rows; // ParticleIndex, already validated
let difference = view.positions()[b.0] - view.positions()[a.0];
let flags = view.flags()[a.0] | view.flags()[b.0];
// ParticleContact still stores ParticleId via candidate.particles()
```

Index with `ParticleIndex` through a tiny crate-internal helper if it avoids raw `.0` at every site. Do not `unwrap`. Missing/stale previous IDs still return `ParticleContactError::MissingParticle`.

### Neighborhood proxy construction (target shape)

```rust
// Source today: crates/liquidfun/src/particle/proxy.rs 109–119
// Change zip to enumerate so the row is retained:
view.particle_ids()
    .iter()
    .copied()
    .enumerate()
    .zip(view.positions().iter().copied())
    .map(|((row, particle), position)| {
        checked_tag(...).map(|tag| Proxy {
            particle,
            row: ParticleIndex(row),
            tag,
        })
    })
```

`enumerate_pairs` then pushes `ParticleNeighborPair::new(a.particle, b.particle)` **and** `[a.row, b.row]`. Public `pairs()` equality tests keep working because they compare `ParticleNeighborPair` IDs only.

[VERIFIED: `proxy.rs` 109–120, 204–228; `tests/particle_contacts.rs` 42–71]

### Previous-contact lookup (target shape)

```rust
// crates/liquidfun/src/particle/view.rs — add crate-internal:
pub(in crate::particle) fn maybe_live_row(
    &self,
    particle: ParticleId,
) -> Option<ParticleIndex> {
    self.storage.resolve_live(particle).ok()
}
```

Requires `resolve_live` visibility `pub(in crate::particle)` (today `pub(super)` from `storage/lifecycle.rs`). One-line visibility bump; do not expand `check_invariants` in that commit.

[VERIFIED: `lifecycle.rs` 408; `view.rs` 31–49]

### Unprofiled gate (do not replace)

```console
just playground-dam-break-bench
# writes target/dam-break-perf/<utc-stamp>/pair.json
# kind: unprofiled_pair, timing_authority: unprofiled_wall_clock
```

[VERIFIED: `justfile` 146–147; D-14]

### Per-wave correctness (minimum)

```console
cargo test -p liquidfun --test particle_contacts
cargo test -p liquidfun --test particle_permutation_coherence
cargo test -p liquidfun --test particle_solver_order
cargo test -p liquidfun --test particle_solver_baseline
cargo test -p liquidfun --test particle_queries
cargo test -p liquidfun --test particle_body_contacts
cargo test -p liquidfun
```

Do not require `liquidfun-differential` Linux oracle per wave (D-13). If a wave touches solver commit/weights, keep permutation + solver tests in the wave verify, not as optional.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hunt-list guess (full-world clone, `to_vec`, SIMD) | SHA-bound samply names `particle_rows` ~93.6% self vs `FindContacts` indices | Phase 23, 2026-09-21, HEAD `6d98531…` | Wave 1 is shape match, not SIMD |
| Profiled flame as the score | Unprofiled `pair.json` walls; profiles retarget | Phase 12 policy; Phase 22 shell | PERF-GATE cannot cite samply ms |
| box2d-rust `B2_VALIDATE` in release | Gate expensive structure checks to debug after extra-work is gone | 2026 analog in PITFALLS.md | D-08: `check_invariants` is a **later** wave, not wave 1 |

**Deprecated/outdated for this phase:**

- SIMD-first Dam Break close (Phase 23 Not found; D-01/D-09)
- Filling `reviewed_reports` (empty by design)
- Required C++ `cpp.json.gz` to land scalar waves
- Criterion micros as the numeric gate (allowed only near 3× on a still-named kernel)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Removing `particle_rows` leaves Dam Break still above 3× (order-of-magnitude leftover ~20× if 93.6% of 71002 ms vanishes and C++ stays ~217 ms) | Pitfall 3 / PERF-GATE | If leftover is already ≤3×, leftover-wave plans must stop (D-06) rather than gold-plate. If leftover is much worse than ~20×, more D-09 waves are required. Re-pair decides; do not bake 21× into success criteria. |
| A2 | After wave 1, Dam Break and the five spot-check scenes remain the same particle-contact cluster | PERF-CANARY2 | If a spot-check is then dominated by emit/hooks/rigid solve, D-12 requires a second native canary/profile instead of the default same-cluster note. |
| A3 | Local CMake 3.27.9 continues to build `oracle-release` `playground-dam-break-bench` as in Phase 23 | Environment | Pair command already wraps configure/build; if CMake fails, the wave cannot prove PERF-GATE on this host. No new CMake work is in phase scope. |

**If this table listed only verified claims:** A1–A3 are the only planning-level assumptions. Locked decisions and named functions are verified from repo + pinned C++ source.

## Open Questions

1. **Does wave 1 alone reach ≤ 3×?**
   - What we know: `particle_rows` is ~93.6% samply self; unprofiled ratio is 327.53× at MEASURED_HEAD.
   - What's unclear: wall-clock leftover after the scan is gone (A1).
   - Recommendation: Plan wave 1 + mandatory unprofiled re-pair + samply retarget as the next plan’s input. Do not skip leftover-wave planning capacity; do not implement D-07 frames until they still have named share.

2. **Should `FindContacts` be fused (no pair `Vec`) in wave 1?**
   - What we know: C++ `FindContacts_Reference` calls `AddContact` during the tag walk; Rust materializes `Vec<ParticleNeighborPair>` then scans. `from_view` is only ~0.4% self today.
   - What's unclear: after `particle_rows` dies, `from_view` / `RawVec` share may jump.
   - Recommendation: Wave 1 keeps the pair `Vec` and adds rows (one concern). Fuse only if a later profile names that allocation/rebuild.

3. **Spot-check step count and timeout?**
   - What we know: D-11 allows wall ms or ms/step; Dam Break 600 steps is the gate, not the spot-check.
   - What's unclear: Fountain emission makes long runs slower; no locked step count.
   - Recommendation: Discretion. Use a modest fixed step count (e.g. 60 warmup + 120 measured, still `advance(1)`), a generous wall timeout, and fail on hang/error. Record particle counts so a later reader can see Fountain growth. Do not compare to C++.

4. **Widen `resolve_live` vs duplicate a linear fallback?**
   - What we know: `resolve_live` is `pub(super)` inside `storage`.
   - What's unclear: none material — widening to `pub(in crate::particle)` is the smaller, correct change.
   - Recommendation: Widen visibility; do not copy `.position` into `view.rs`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / `rustc` | Kernel builds, `cargo test -p liquidfun`, `--release` gate | ✓ | 1.97.0 | — |
| `just` | One-line recipes | ✓ | 1.48.0 | `cargo xtask playground …` |
| samply | Leftover retarget (D-05) | ✓ | 0.13.1 | Fail closed with existing install text; do not skip retarget by inventing names |
| CMake | Unprofiled C++ pair | ✓ | 3.27.9 (local floor; CI pin 4.3.3) | Existing xtask pair already configures `oracle-release` |
| Ninja | C++ extra target | ✓ | 1.13.2 | — |
| `bun` | Bright Builds file-lengths / `all` | ✓ | 1.4.2 | — |
| Python 3.13+ | `just markdown-check` on non-GSD Markdown | ✓ | 3.14.6 | Do **not** mdformat `.planning/**` |
| `third_party/liquidfun` | Shape comparison / oracle | ✓ | submodule `7f204021…` | Pinned GitHub source used in this research if working tree empty |
| Linux x64 oracle / controlled host | Not required | n/a | — | Hobby scope; D-13 skips per-wave Linux oracle |
| Criterion | Optional near-3× micro | in workspace | 0.8.2 (stack) | Skip until pair already near 3× (D-09) |

**Missing dependencies with no fallback:** none for this host.

**Missing dependencies with fallback:** Criterion unused unless a named kernel remains near 3×.

**Step 2.6:** Not skipped — pair, samply retarget, and C++ oracle are external to `liquidfun` source edits.

## Security Domain

`security_enforcement` is not set to `false` in `.planning/config.json` (absent = enabled). This phase is a local physics kernel + developer evidence loop: no auth, sessions, or network parsers.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Keep fail-closed `ParticleContactError` / `ParticleProxyError`; diameter and tag-domain checks in `from_view` stay. Do not swap solver errors for `unwrap`. |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Invalid particle handles / stale rows after compaction | Tampering / Elevation of privilege (logic) | Public IDs + `resolve_live` generation checks; do not trust last-step dense rows |
| Capacity/DoS via unbounded pair `Vec` | Denial of service | Existing typed capacity/errors; do not `unwrap` `try_reserve`; Dam Break recipe stays 1920 |
| Supply-chain / dep inflation | Tampering | `liquidfun` stays bitflags-only; `cargo xtask package verify` |
| Evidence spoofing (faster sample via shrink/skip) | Spoofing | PERF-ADMIT: named share + unprofiled pair + tests; preserve failed stamps |

## Sources

### Primary (HIGH confidence)

- `docs/native-performance-audit.md` — named shares; MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`; 327.53×; **Not found** SIMD/Rayon/PGO/`unsafe`
- `crates/liquidfun/src/particle/{contact,proxy,view}.rs`, `storage/{lanes,lifecycle,runtime}.rs` — live hot path
- `crates/liquidfun/src/world/particle_coupling.rs` — shared `update_particle_contacts`
- Pinned C++ `google/liquidfun@7f20402173fd143a3988c921bc384459c6a858f2` `b2ParticleSystem.cpp` `FindContacts_Reference` / `AddContact`; `b2ParticleSystem.h` `struct Proxy { int32 index; uint32 tag; }`
- `.planning/phases/24-shared-hot-path-waves-through-3/24-CONTEXT.md` — locked D-01…D-15
- `.planning/REQUIREMENTS.md`, `ROADMAP.md`, `STATE.md`, `PROJECT-SCOPE.md`
- `reference/performance/{manifest.toml,policy.json}`; `BENCHMARKING.md`; `docs/playground-dam-break-timing.md`
- Workspace `Cargo.toml` / `crates/liquidfun/Cargo.toml` / `justfile` / xtask playground modules
- Bright Builds `standards/languages/rust.md`, `core/{architecture,code-shape,verification,testing}.md`

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` — SIMD-first, wrong construction, profiled authority, debug-vs-release validators (box2d-rust analog)
- `.planning/research/FEATURES.md` — PERF-* seeds
- Phase 9 `09-CONTEXT.md` — stable `ParticleId`, private dense rows
- Phase 22/23 CONTEXT — exclusive stamps, `just` one-liner, isolation
- Host tool versions probed 2026-09-21 (local CMake 3.27.9 vs stack CI pin 4.3.3)

### Tertiary (LOW confidence)

- Leftover wall-clock ratio after wave 1 (A1) — needs the next unprofiled `pair.json`, not more literature

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new libraries; isolation and pins verified in-tree
- Architecture: HIGH — C++ `Proxy.index` vs Rust `.position` scan verified in source; `resolve_live` already exists
- Pitfalls: HIGH for policy pitfalls; MEDIUM for leftover-wave count after `particle_rows`

**Research date:** 2026-09-21
**Valid until:** 2026-10-21 (stable kernel/policy; leftover ranking expires at the first post-wave-1 samply)

## Suggested plan grain (for gsd-planner)

Not a locked decision — planner may split differently. One concern per plan/wave:

1. **24-01** — `particle_rows` shape: neighborhood `ParticleIndex` + `pair_rows`; `generate` / `validate_pairs` / `listener_effects`; tests in `particle_contacts.rs`; `cargo test -p liquidfun`.
2. **24-02** — Admit wave 1: unprofiled new stamp; refresh unreviewed timing/audit notes if ratio improved; samply sibling retarget; do not claim PERF-GATE yet unless `pair.json` ≤ 3.
3. **24-03+** — Contingent leftover waves in D-07 order **only if** the new profile still names them with non-trivial share. First leftover is **not** `check_invariants` until wave 1 has landed (D-08). Stop at ≤ 3× (D-06).
4. **24-spot** — Five-scene native `--release` headless stamp + committed unreviewed table (PERF-SPOT). Default same-cluster note (PERF-CANARY2) unless a scene’s time/failure mode differs.
5. **24-isolation** — `cargo xtask package verify`; `cargo tree -p liquidfun --edges normal` bitflags-only; empty `manifest.toml`; no README speed claim; independent AI review after verification (no self-approval).

Do not invent hot-function names beyond `docs/native-performance-audit.md`.
