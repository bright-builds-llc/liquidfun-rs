# Roadmap: liquidfun-rs

## Overview

v1.2 Native Performance Closing is a developer-facing hunt, not a public benchmark or crate release. Measure the locked Dam Break Medium pair, name extra work from samply, then change shared scalar particle/rigid kernels until unprofiled Rust `--release` is ≤ 3× pinned C++ `oracle-release` on one macOS host. Other playground scenes are spot-checks. WASM is a post-gate sanity check, never a C++ pair. Phase numbering continues after 21. Do not revive Phase 12 public claims, required perf CI, default SIMD/parallel, or package publication.

## Milestones

- [ ] **v1.2 Native Performance Closing** — Phases 22–25 (in progress). Local Dam Break Medium ≤ 3× C++; no Phase 12 sealed matrix; no crate or tag.
- [x] **v1.1 Web Playground** — 6 phases / 39 plans complete; archived 2026-09-20 ([full roadmap](milestones/v1.1-ROADMAP.md)). Planning label only; no package or tag released.
- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Active Milestone: v1.2 Native Performance Closing

**Goal:** Get native Rust stepping into the same order of magnitude as pinned C++ on Dam Break Medium (unprofiled wall time ≤ 3×) by profiling shared particle/rigid hot paths, then optimizing from that evidence under the existing scalar, deterministic baseline.

**Locked constraints:**
- Gate command is unprofiled `just playground-dam-break-bench` (scalar Rust `--release` vs `oracle-release`, 1920 particles, 60 warm-up + 600 timed steps).
- Evidence directory is `target/dam-break-perf/<utc-stamp>/` (gitignored; never clobber an existing stamp).
- Profiled / `step_profiled` / dhat / xctrace wall times are never the 3× number.
- `reference/performance/manifest.toml` stays empty (`reviewed_reports = []`).
- Local macOS is enough. No required perf CI, no dedicated Linux/performance host, no crate publish.

## Phases

- [x] **Phase 22: Observability shell** — Persist the Dam Break pair and capture samply profiles (plus optional parent timers) without changing physics.
- [x] **Phase 23: Baseline pair and named audit** — Name dominating extra work from a SHA-bound unprofiled pair and CPU profile; heap-profile only if samply shows allocator time.
- [ ] **Phase 24: Shared hot-path waves through 3×** — Close Dam Break Medium to ≤ 3× C++ via evidenced shared scalar kernels, with other-scene spot-checks and a second-canary-or-same-cluster note.
- [ ] **Phase 25: WASM sanity and honest close** — Confirm the six playground scenes still run and document remaining Dam Break delta without public speed claims.

<details>
<summary>✅ v1.1 Web Playground (Phases 16-21) — SHIPPED 2026-09-20</summary>

v1.1 phase directories remain in `.planning/phases/` for stable historical references. Full archived details: [milestones/v1.1-ROADMAP.md](milestones/v1.1-ROADMAP.md).

- [x] Phase 16: Rust WASM Browser Bridge (4/4 plans) — completed 2026-09-17
- [x] Phase 17: Shared Player and Early Pages Delivery (8/8 plans) — completed 2026-09-17
- [x] Phase 18: Six Native Physics Demos (10/10 plans) — completed 2026-09-18
- [x] Phase 19: Interaction Polish and Browser Verification (7/7 plans) — completed 2026-09-19
- [x] Phase 20: Playground catalog previews and Reset honesty (6/6 plans) — completed 2026-09-20
- [x] Phase 21: Playground leftover cleanup (4/4 plans) — completed 2026-09-20

</details>

## Phase Details

### Phase 22: Observability shell
**Goal**: A developer can re-run the locked Dam Break pair into a dated evidence directory and capture a symbolicated samply CPU profile (plus optional parent-phase timers) without changing physics or the `--release` gate binary.
**Depends on**: Nothing in v1.2 (uses the existing Dam Break pair, oracle extra target, and v1.1 playground bench)
**Requirements**: PERF-PAIR, PERF-PROFILE, PERF-TIMERS
**Success Criteria** (what must be TRUE):
  1. Developer can run `just playground-dam-break-bench` and find a new dated directory under `target/dam-break-perf/<utc-stamp>/` with unprofiled wall ms, ms/step, Rust/C++ ratio, host, git HEAD, and compilers — not stdout-only — and an existing stamp is not overwritten.
  2. Developer can run a thin `just` / xtask profile recipe that rebuilds a `[profile.profiling]` Dam Break binary (`inherits = "release"`, `debug = true`), captures samply 0.13.1 of the timed Rust loop, and writes `rust.json.gz` plus host/HEAD/compiler/command identity into a new stamp; profiled wall times are never treated as the 3× number.
  3. Missing samply fails closed with install/error text rather than silently skipping; `cargo xtask package verify` still passes; default `--release` is unchanged and `liquidfun` gains no profiler, samply, dhat, CMake, or serde dependency.
  4. Developer can emit coarse parent-phase timers (`particle_prepare` / `particle_solve` / `rigid_solve` or equivalent existing `step_profiled` parents) on a separate Dam Break diagnostic path, and those timers are absent from the unprofiled gate process.
**Plans:** 6/6 plans complete

Plans:
- [x] 22-01-PLAN.md — Split playground xtask module and mint exclusive evidence stamps
- [x] 22-02-PLAN.md — Persist unprofiled Dam Break pair.json/pair.md with Rust/C++ ratio
- [x] 22-03-PLAN.md — Add [profile.profiling] and fail-closed samply argv core
- [x] 22-04-PLAN.md — Wire dam-break-profile just/xtask capture with fake samply
- [x] 22-05-PLAN.md — Separate dam-break-timers path with step_profiled parents
- [x] 22-06-PLAN.md — Prove package isolation and gitignored evidence honesty

No physics kernel edits in this phase. Tooling can be proven with fake cmake/samply. Do not hide CMake or samply flags in `just`. Optional C++ `-g` is a profile-command cache flag only — never a pair-command or new CMake preset.

### Phase 23: Baseline pair and named audit
**Goal**: A developer can read a committed audit that names dominating extra work from a SHA-bound unprofiled Dam Break pair and samply profile, and can run a private heap dump only when that profile shows allocator or `Vec` time.
**Depends on**: Phase 22
**Requirements**: PERF-AUDIT, PERF-HEAP
**Success Criteria** (what must be TRUE):
  1. Developer can open committed `docs/native-performance-audit.md` that names dominating Rust functions (and C++ counterparts when extra work is a shape mismatch), classifies suspected causes (extra per-particle work, per-step allocation, checks that survive `--release`, algorithm/shape differences), records the current unprofiled Dam Break Medium wall-time ratio at a recorded HEAD, and states what was not found so SIMD is not the first move.
  2. Developer can point at a dated `target/dam-break-perf/<utc-stamp>/` from that HEAD containing the unprofiled pair report and samply `rust.json.gz` used to write those names; profiled duration is discarded for the gate.
  3. If samply shows allocator or `Vec` time, developer can run private dhat on the `dam-break-bench` binary only and find the dump under the same gitignored evidence root; if not, committed notes record that and skip the heap run.
  4. `docs/playground-dam-break-timing.md` is refreshed as an unreviewed local sample, not a Phase 12 public claim; raw `.json.gz` / `.trace` files stay gitignored.
**Plans:** 9/9 plans complete

Plans:
- [x] 23-01-PLAN.md — Split playground CLI tests under the 628-line cap
- [x] 23-02-PLAN.md — Copy-only same-HEAD audit-bundle command and just alias
- [x] 23-03-PLAN.md — Fake-stamp audit-bundle CLI success and fail-closed tests
- [x] 23-04-PLAN.md — Optional dhat-heap on dam-break-bench only
- [x] 23-05-PLAN.md — Allocator/Vec heuristic and dam-break-heap command
- [x] 23-06-PLAN.md — Fake-cargo heap CLI match and skip tests
- [x] 23-07-PLAN.md — Live pair, live samply, audit-bundle stamps, durable primary-tree copy
- [x] 23-08-PLAN.md — Named audit, heap run-or-skip, timing-doc refresh
- [x] 23-09-PLAN.md — Isolation, Bright Builds, and no self-approval

Do not quote samply duration as the 3× number. Do not commit profile blobs. In-tree suspects (full-world clone, release invariants, per-pass `to_vec`, `ParticleNeighborhood::from_view`) are a hunt list until this audit ranks them.

### Phase 24: Shared hot-path waves through 3×
**Goal**: A developer can land evidenced scalar fixes in shared `liquidfun` particle/rigid stepping until the unprofiled Dam Break Medium pair is ≤ 3× pinned C++, while other playground scenes stay usable and the scalar deterministic baseline is unchanged.
**Depends on**: Phase 23
**Requirements**: PERF-ADMIT, PERF-SHARED, PERF-GATE, PERF-BASELINE, PERF-SPOT, PERF-CANARY2
**Success Criteria** (what must be TRUE):
  1. Developer can land a hot-path change only when evidence names a function or typed bottleneck with non-trivial profile share, the unprofiled Dam Break Medium ratio improves on the same host and `just playground-dam-break-bench` recipe, existing native tests and relevant differential/determinism checks still pass, and the scene particle count is not lowered; a physics mismatch is a failed candidate, never a faster sample.
  2. Admitted fixes land in shared `liquidfun` particle/rigid stepping (neighborhood/proxy, contacts, particle–body coupling, pressure/damping/integrate, rigid contact solve) so other particle/rigid scenes can benefit; Dam Break-only scene, WASM-copy, or skipped-solver cheats do not satisfy this.
  3. Developer can demonstrate native Dam Break Medium unprofiled wall time ≤ 3× pinned C++ on the same host under scalar `--release` vs `oracle-release` for the locked 60+600-step pair, recording host, git HEAD, compilers, both wall times, and ratio into a new dated evidence directory; `reference/performance/manifest.toml` stays empty.
  4. The scalar deterministic compatibility baseline still holds: no default Rayon or SIMD, no `-ffast-math` / `-march=native` on the pair, no lifting `unsafe_code = "forbid"` to chase the canary; SIMD/parallel remain explicit later opt-in.
  5. Developer can spot-check Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel with native `--release` headless stepping after shared-path fixes, recording improvement or non-catastrophic regression, and after the gate can record either a second native headless canary whose profile cluster differs from Dam Break or an explicit notes statement that Dam Break and the spot-checks share the same dominant cluster — not a sealed 32-case matrix.
**Plans:** 6 plans

Plans:
- [ ] 24-01-PLAN.md — Carry dense ParticleIndex through neighborhood/contact generation (particle_rows shape)
- [ ] 24-02-PLAN.md — Admit wave 1 with a new exclusive unprofiled pair and samply retarget
- [ ] 24-03-PLAN.md — First leftover admit-or-skip from the post-wave-1 samply (D-07 order)
- [ ] 24-04-PLAN.md — Continue scalar leftover waves until unprofiled pair.json ≤ 3×
- [ ] 24-05-PLAN.md — Five-scene native headless spot-check plus same-cluster canary note
- [ ] 24-06-PLAN.md — Isolation, empty manifest, and no self-approval

One concern per wave; re-pair unprofiled into a **new** dated directory; preserve failed records. samply retargets the next wave; Criterion is allowed only if a named kernel remains after the pair is already near 3×. Do not copy the ratio into `manifest.toml`. Do not claim PERF-GATE in 24-01/24-02 unless that pair.json ratio is already ≤ 3.

### Phase 25: WASM sanity and honest close
**Goal**: After the native 3× gate, a visitor can still run the six playground scenes, and a developer can read committed remaining-delta notes without a public “Rust is N×” claim.
**Depends on**: Phase 24
**Requirements**: PERF-NOTES, PERF-WASM
**Success Criteria** (what must be TRUE):
  1. After PERF-GATE, a visitor can still run Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel; developer records `just web-player-smoke` plus an optional Dam Break step-time note versus previous WASM or native Rust, never versus `oracle-release`.
  2. Developer can document the remaining Dam Break delta after the gate (ratio, suspected leftover causes, what was not attempted) in committed notes; README and crates.io do not gain a universal “Rust is N× slower/faster” claim.
  3. `reviewed_reports` remains empty and committed benchmarking/audit notes state that the playground pair is not a Phase 12 sealed public claim.
**Plans**: TBD

Do not put an Instant profiler in the cdylib, lift the 4-step catch-up cap to fake realtime, or compare browser Dam Break to C++.

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
| --- | --- | --- | --- | --- |
| 16. Rust WASM Browser Bridge | v1.1 | 4/4 | Complete | 2026-09-17 |
| 17. Shared Player and Early Pages Delivery | v1.1 | 8/8 | Complete | 2026-09-17 |
| 18. Six Native Physics Demos | v1.1 | 10/10 | Complete | 2026-09-18 |
| 19. Interaction Polish and Browser Verification | v1.1 | 7/7 | Complete | 2026-09-19 |
| 20. Playground catalog previews and Reset honesty | v1.1 | 6/6 | Complete | 2026-09-20 |
| 21. Playground leftover cleanup | v1.1 | 4/4 | Complete | 2026-09-20 |
| 22. Observability shell | v1.2 | 6/6 | Complete    | 2026-09-21 |
| 23. Baseline pair and named audit | v1.2 | 9/9 | Complete    | 2026-09-21 |
| 24. Shared hot-path waves through 3× | v1.2 | 0/6 | Not started | - |
| 25. WASM sanity and honest close | v1.2 | 0/TBD | Not started | - |

v1.0 phases 1–15 remain in the [v1.0 roadmap archive](milestones/v1.0-ROADMAP.md).
