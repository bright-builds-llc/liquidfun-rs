# Native Dam Break named-function audit

**Unreviewed local sample.** This names dominating Rust frames from one live
playground Dam Break Medium pair plus samply CPU profile on this host. It is
not a public “Rust is N× slower” product claim, not compatibility evidence,
and not Phase 12 evidence. Do not copy these numbers into
`reference/performance/manifest.toml`. Do not treat samply, `[profile.profiling]`,
`step_profiled`, or dhat clocks as the Dam Break ratio.

## Reproduce

```console
just playground-dam-break-bench
just playground-dam-break-profile
just playground-dam-break-audit-bundle
samply load target/dam-break-perf/2026-09-21T04-34-32Z/rust.json.gz
```

The pair recipe is unprofiled native `--release` versus scalar `oracle-release`.
The profile recipe rebuilds `[profile.profiling]` and is
`not_timing_authority`. The bundle command copies the same-HEAD pair report and
`rust.json.gz` (plus `rust.json.syms.json`) into a new exclusive stamp; it does
not retime the engines.

## Evidence identity

Ranking used the durable root
`$(dirname $(git rev-parse --path-format=absolute --git-common-dir))/target/dam-break-perf/`
(primary tree; this checkout already held the 23-07 stamps, so no hydrate copy
was required).

- git HEAD (capture SHA, ancestor of this docs commit):
  `6d98531ac799987c209d3fd1e572e482fcab5da6`
- Bundle stamp: `target/dam-break-perf/2026-09-21T04-34-32Z/`
- Source pair stamp: `target/dam-break-perf/2026-09-21T04-32-19Z/`
- Source profile stamp: `target/dam-break-perf/2026-09-21T04-32-56Z/`
- Profile sidecar: `rust.json.syms.json` (sidecar-first names; gecko
  `stackTable` sample weights from the `dam-break-bench` thread)
- samply: `0.13.1` (`--unstable-presymbolicate`)
- Workload: 1920 particles, 60 warmup steps, 600 measured steps

Approximate shares below are leaf-sample fractions of 72817
`dam-break-bench` samples, mapping each leaf RVA through the live sidecar.
They are not a Firefox-profiler UI export and are not wall-clock milliseconds.

## Unprofiled Dam Break Medium (timing authority)

Copied from bundle `pair.json` only (`kind: unprofiled_pair`,
`timing_authority: unprofiled_wall_clock`). Profiled, timer, and dhat clocks
are `not_timing_authority` and are not the 3× gate.

- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`

| Engine      | Particles | Wall ms     | ms/step    | Compiler                              |
| ----------- | --------- | ----------- | ---------- | ------------------------------------- |
| native Rust | 1920      | 71001.949958 | 118.336583 | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 216.777375  | 0.361296   | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `327.53395024734476` (unreviewed sample figure bound to
the HEAD and stamp above).

Rust used `--release`. C++ used the scalar `oracle-release` wrapper (no
`-ffast-math`, no `-march=native`).

## Named functions

Each listed frame is classified into exactly one cause class. C++ names appear
only for shape mismatches, by comparison to pinned LiquidFun
`7f20402173fd143a3988c921bc384459c6a858f2` (`FindContacts_Reference`,
`UpdateProxies` / `SortProxies`, in-place `b2World::Step`, `NDEBUG`). No C++
samply wrap was run.

| Symbol | Approximate share | Cause class | C++ counterpart (shape mismatch only) |
| ------ | ----------------- | ----------- | ------------------------------------- |
| `liquidfun::particle::contact::particle_rows` (leaf; inlined `ParticleSystemView::particle_ids` `.position` scan) | ~93.6% self | algorithm/shape differences | `b2ParticleSystem::FindContacts_Reference` calls `AddContact(a->index, b->index)` from `Proxy.index`; contacts keep integer indices, they do not re-scan identity slices |
| `<liquidfun::identity::ParticleGroupId as SliceContains>::slice_contains` (parent `ParticleStorage::check_invariants`) | ~2.2% self | checks that survive `--release` | `b2Assert` compiled out under `NDEBUG` in `oracle-release` |
| `ParticleStorage::replace_solver_candidate` | ~1.6% inclusive (~0.01% self) | checks that survive `--release` | C++ solver mutates member buffers in place; this path clones `ParticleStorage` then `check_invariants()?` on the release candidate |
| `ParticleNeighborhood::from_view` | ~0.4% self / ~0.6% inclusive | algorithm/shape differences | `UpdateContacts` → `UpdateProxies(m_proxyBuffer)` + `SortProxies(m_proxyBuffer)` rewrite the in-place proxy buffer |
| `alloc::raw_vec::RawVecInner::finish_grow` | ~0.5% self | per-step allocation | (none listed; C++ `m_contactBuffer` / member scratch reuse is the nearby shape, but this leaf is the allocator grow) |
| `Vec<ParticleContact>` `SpecFromIterNested` collect | ~0.2% self | per-step allocation | `FindContacts` writes into reused `m_contactBuffer` (`SetCount(0)` then append) |
| `ParticleStorage::recompute_weights` | ~0.3% self | extra per-particle work | (none; not a listed C++ shape split) |
| `ParticleContactUpdate::generate` | ~0.3% self / ~94.3% inclusive | algorithm/shape differences | `UpdateContacts` / `FindContacts` parent of the index-preserving contact pass |
| `particle::contact::listener_effects` / body-contact `listener_effects` | ~1.0% + ~0.4% self | extra per-particle work | listener notify uses stored indices; Rust re-resolves rows with `particle_rows` / `particle_row` |
| `pressure::damping` | ~0.2% self | extra per-particle work | (none; kernel time, not a first-move SIMD target) |

`particle_rows` is the bulk of named sampled time. It maps each
`ParticleId` pair to dense rows with
`view.particle_ids().iter().position(...)` inside
`ParticleContactUpdate::generate` (and again from `validate_pairs` /
`listener_effects`). That is extra identity-to-row work C++ does not do on
the `FindContacts` path.

## Heap

Heap run-or-skip is recorded after this named ranking, using the live
`classify_profile_symbols` / samply allocator-`Vec` heuristic on the same
bundle stamp. See the Heap subsection at the end of this file after the 23-08
heap task updates it. Until that update, this heading exists so the cause
taxonomy and Not found section can land first.

## Not found

These were **not** the first move from this live profile. Hunt-list symbols
that do not carry a meaningful ranked share stay here rather than in the table
above.

- SIMD-first (including C++ `FindContacts_Simd` / NEON, `-march=native`,
  `-ffast-math`) as the Dam Break closer
- Default Rayon / implicit parallel `World::step`
- PGO
- Lifting workspace `unsafe_code = "forbid"` / unsafe indexing /
  bounds-check-only as the first optimization
- WASM versus C++ comparison
- Phase 12 sealed 32-case public matrix (`reference/performance/manifest.toml`
  `reviewed_reports` remains empty; this sample is not a reviewed report)
- `World::backup_step_limit_state`: present at ~0.02% inclusive, **Not ranked**
  as a dominating cause
- `velocities().to_vec` as a named leaf: **Not found** (no `to_vec` leaf share;
  nearby collect/`RawVec` frames are ranked under per-step allocation instead)
- C++ `cpp.json.gz` / C++ samply wrap: not run

`ParticleNeighborhood::from_view` and `replace_solver_candidate` **did** appear
and are ranked above; they are not hunt-list paste.
