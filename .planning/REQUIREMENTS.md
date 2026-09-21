# Requirements: liquidfun-rs v1.2 Native Performance Closing

**Defined:** 2026-09-20
**Core Value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

## v1 Requirements

Requirements for this milestone. Each maps to roadmap phases. "Developer" means a repository contributor on the hobby macOS host. These are not package-release or public-benchmark claims.

### Observability

- [x] **PERF-AUDIT**: Developer can read a committed audit notes doc that names dominating Rust functions (and C++ counterparts when the extra work is a shape mismatch), classifies suspected causes (extra per-particle work, per-step allocation, checks that survive `--release`, algorithm/shape differences), records the current unprofiled Dam Break Medium wall-time ratio, and states what was not found (so SIMD is not the first move).
- [x] **PERF-PAIR**: Developer can re-run the locked Dam Break Medium pair on demand (`just playground-dam-break-bench`: same host, scalar Rust `--release` vs C++ `oracle-release`, 1920 particles, 60 warm-up + 600 timed steps) and persist a dated unprofiled report under `target/dam-break-perf/<utc-stamp>/` (wall ms, ms/step, Rust/C++ ratio, host, git HEAD, compilers) — not stdout-only.
- [x] **PERF-PROFILE**: Developer can run a thin `just` / xtask recipe that rebuilds a `[profile.profiling]` Dam Break binary (inherits `release`, `debug = true`), captures a samply 0.13.1 CPU profile of the timed Rust loop, and writes `rust.json.gz` plus host/HEAD/compiler/command identity into `target/dam-break-perf/<utc-stamp>/` without clobbering an existing stamp. Profiled wall times are never the 3× number.
- [x] **PERF-TIMERS**: Developer can emit coarse parent-phase timers (`particle_prepare` / `particle_solve` / `rigid_solve` or equivalent existing `step_profiled` parents) on a separate Dam Break diagnostic path for cheaper iteration, without putting those timers inside the unprofiled gate process.
- [x] **PERF-HEAP**: Developer can run a private heap profile (`dhat` on the `dam-break-bench` binary only) after samply shows allocator or `Vec` time, writing the dump under the same gitignored evidence root. If samply does not show allocator time, committed notes record that and skip the heap run.

### Optimization

- [ ] **PERF-ADMIT**: Developer can land a hot-path change only when evidence names a function or typed bottleneck with non-trivial profile share, the unprofiled Dam Break Medium ratio improves on the same host and recipe, existing native tests and relevant differential/determinism checks still pass, and the scene particle count is not lowered to fake the gate. A physics mismatch is a failed candidate, never a faster sample.
- [ ] **PERF-SHARED**: Developer can apply admitted fixes in shared `liquidfun` particle/rigid stepping (neighborhood/proxy, contacts, particle–body coupling, pressure/damping/integrate, rigid contact solve) so other particle/rigid scenes benefit. Dam Break-only scene, WASM-copy, or skipped-solver cheats do not satisfy this.
- [ ] **PERF-GATE**: Developer can demonstrate native Dam Break Medium unprofiled wall time ≤ 3× pinned C++ on the same host under scalar `--release` vs `oracle-release` for the locked 60+600-step pair, recording host, git HEAD, compilers, both wall times, and ratio. `reference/performance/manifest.toml` stays empty.
- [ ] **PERF-BASELINE**: Developer can keep the scalar deterministic compatibility baseline while closing the gap: no default Rayon or SIMD, no `-ffast-math` / `-march=native` on the pair, no lifting `unsafe_code = "forbid"` to chase the canary. SIMD/parallel remain explicit later opt-in.

### Honesty

- [ ] **PERF-SPOT**: Developer can spot-check Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel with native `--release` headless stepping after shared-path fixes, recording improvement or non-catastrophic regression. No per-scene C++ pair and no Phase 12 case hashes are required.
- [ ] **PERF-CANARY2**: After PERF-GATE, developer can record either (a) a second native headless canary for a playground scene whose profile cluster differs from Dam Break, or (b) an explicit notes statement that Dam Break and the spot-checks share the same dominant cluster. This is not a sealed 32-case matrix.
- [ ] **PERF-NOTES**: Developer can document the remaining Dam Break delta after the gate (ratio, suspected leftover causes, what was not attempted) in committed notes. README and crates.io do not gain a universal “Rust is N× slower/faster” claim.
- [ ] **PERF-WASM**: After PERF-GATE, a visitor can still run the six playground scenes. Developer records a lightweight WASM/playground sanity check (`just web-player-smoke` plus an optional Dam Break step-time note versus previous WASM or native Rust, never versus `oracle-release`).

## Future Requirements

Deferred. Tracked but not in the current roadmap.

### Later acceleration

- **PERF-SIMD**: Developer can enable an explicit SIMD or parallel opt-in after the scalar canary is closed and a separate determinism decision exists.
- **PERF-UNSAFE**: Developer can relax `unsafe_code = "forbid"` for a measured intrinsic path only after leftover delta is SIMD/codegen-shaped, not extra-work-shaped.

### Later evidence

- **PERF-PHASE12**: Developer can produce Phase 12 sealed-matrix calibration and reviewed-report promotion.
- **PERF-WASM-ENG**: Developer can engineer WASM vs native stepping beyond a post-gate sanity check.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
| --- | --- |
| Revive Phase 12 sealed 32-case public claims / copy Dam Break numbers into `reference/performance/manifest.toml` | Exploratory pair is not the sealed matrix; filling the empty manifest would be a false claim |
| Required performance CI or a dedicated Linux/performance-host completion gate | Hobby scope: local checks plus one macOS Cargo job; pair timing is host-specific |
| Default parallelism or SIMD in the compatibility baseline | Changes solver order and determinism; locked out for this close |
| WASM ≈ C++ (browser Dam Break vs `oracle-release`) | Different ISA, allocator, and frame-copy shell; not a fair pair |
| “Rust is N× slower/faster” README or crates.io blurb | One host, one scene, unreviewed; `BENCHMARKING.md` forbids universal summaries |
| Scene editor, new playground scenes, or lowering Medium below 1920 particles | Cosmetics and cheats; the recipe stays locked |
| Crate publication / git release tag / npm package | Publication remains separately authorized |
| Criterion catalog micros as the numeric gate | Wrong granularity vs the playground canary |
| `codegen-units = 1`, fat LTO, PGO, `-march=native`, or `-ffast-math` as the close | Does not explain ~300×; can move IEEE/ordering vs the oracle |
| `unsafe` indexing / C++ FFI in production `liquidfun` | Typical bounds-check wins are 1–15%, not 300×; workspace forbids `unsafe` |
| Dam Break-only LOD, skipped solver passes, or reduced iterations | Breaks LiquidFun behavior and differential evidence |
| Treating profiled / `step_profiled` runs as timing authority | Instrumentation distorts wall time |
| Substituting Rapier, Avian, or modern Box2D for LiquidFun particles | Wrong behavior oracle |
| Broad `no_std`, mobile, or complete-engine WASM certification | Unrelated to the native canary |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
| --- | --- | --- |
| PERF-PAIR | Phase 22 | Complete |
| PERF-PROFILE | Phase 22 | Complete |
| PERF-TIMERS | Phase 22 | Complete |
| PERF-AUDIT | Phase 23 | Complete |
| PERF-HEAP | Phase 23 | Complete |
| PERF-ADMIT | Phase 24 | Pending |
| PERF-SHARED | Phase 24 | Pending |
| PERF-GATE | Phase 24 | Pending |
| PERF-BASELINE | Phase 24 | Pending |
| PERF-SPOT | Phase 24 | Pending |
| PERF-CANARY2 | Phase 24 | Pending |
| PERF-NOTES | Phase 25 | Pending |
| PERF-WASM | Phase 25 | Pending |

**Coverage:**
- v1 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0

---
*Requirements defined: 2026-09-20*
*Last updated: 2026-09-20 after roadmap mapping to phases 22–25*
