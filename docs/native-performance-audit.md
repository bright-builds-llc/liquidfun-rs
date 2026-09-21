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
samply load target/dam-break-perf/2026-09-21T15-52-27Z/rust.json.gz
samply load target/dam-break-perf/2026-09-21T16-07-39Z/rust.json.gz
samply load target/dam-break-perf/2026-09-21T20-18-21Z/rust.json.gz
```

The pair recipe is unprofiled native `--release` versus scalar `oracle-release`.
The profile recipe rebuilds `[profile.profiling]` and is
`not_timing_authority`. The bundle command copies the same-HEAD pair report and
`rust.json.gz` (plus `rust.json.syms.json`) into a new exclusive stamp; it does
not retime the engines. The first `samply load` is the Phase 23 ranking dump.
The second is the Wave 1 leftover-ranking sibling used to admit the first
leftover. Later `samply load` lines are leftover-ranking siblings; none of
those durations is the 3× number.

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

| Engine      | Particles | Wall ms      | ms/step    | Compiler                              |
| ----------- | --------- | ------------ | ---------- | ------------------------------------- |
| native Rust | 1920      | 71001.949958 | 118.336583 | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 216.777375   | 0.361296   | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `327.53395024734476` (unreviewed sample figure bound to
the HEAD and stamp above). That Phase 23 pair remains the ranking identity for
the named-function table below. The current 3× number is the first leftover
admission `pair.json` in the leftover section.

Rust used `--release`. C++ used the scalar `oracle-release` wrapper (no
`-ffast-math`, no `-march=native`).

## Wave 1 admission

`particle_rows` is the landed concern (plan 24-01: neighborhood `Proxy` carries
dense `ParticleIndex`; private `pair_rows` aligned with public
`ParticleNeighborPair` IDs). Admission uses a **new exclusive** unprofiled pair
on the locked 1920-particle, 60 warmup + 600 measured-step recipe. Do **not**
quote samply or `[profile.profiling]` duration as this number.

- Pair stamp: `target/dam-break-perf/2026-09-21T15-50-46Z/` (`pair.json`,
  `kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`)
- Pair `git_head`: `27dc3191da7cc4e8a79571c608673e5a563d3c3c`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- Sibling samply stamp (leftover-wave retarget input only):
  `target/dam-break-perf/2026-09-21T15-52-27Z/` (`profile-identity.json`
  `kind: samply_cpu`, `not_timing_authority: true`; no `pair.json` in that
  stamp)
- Profile `git_head`: `d55a36308fdb79fdfffa7157295be0bbf585f95d`

| Engine      | Particles | Wall ms     | ms/step  | Compiler                              |
| ----------- | --------- | ----------- | -------- | ------------------------------------- |
| native Rust | 1920      | 4610.920125 | 7.684867 | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 219.479292  | 0.365799 | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `21.00845179052245` (copied from that `pair.json` only).

**PERF-GATE is not claimed.** The pair ratio is still greater than 3, so leftover
scalar waves (D-07/D-09) remain required. `ParticleStorage::check_invariants`
was not gated behind `debug_assertions` in wave 1 (D-08).

## First leftover admission

Ranked from Wave 1 sibling sidecar
`target/dam-break-perf/2026-09-21T15-52-27Z/rust.json.syms.json` plus gecko
`stackTable` leaf weights on the `dam-break-bench` thread (Phase 23 method:
outer symbol, 4630 samples). This is not the Phase 23 named table and is not
the 3× number.

| Symbol (D-07 order)                                                      | Approximate leaf share          | Named?              |
| ------------------------------------------------------------------------ | ------------------------------- | ------------------- |
| `ParticleStorage::check_invariants` / `ParticleGroupId` `slice_contains` | ~32.85% self                    | yes; first leftover |
| `ParticleStorage::replace_solver_candidate`                              | ~0.15% self / ~23.95% inclusive | still named         |
| `ParticleNeighborhood::from_view`                                        | ~7.60% self                     | still named         |
| `RawVecInner::finish_grow` / `Vec<ParticleContact>` collect              | ~6.61% / ~3.80% self            | still named         |
| `contact::listener_effects` / `body_contact::listener_effects`           | ~14.28% self                    | still named         |

`slice_contains` leaves sat under `check_invariants`. About 22.68% of thread
samples were that path via `replace_solver_candidate`; about 10.60% were via
`prepare_permutation` (not this leftover). Plan 24-03 gated only
`candidate.check_invariants()` on `replace_solver_candidate` behind
`#[cfg(debug_assertions)]`. User-facing create/mutate checks stay fail-closed.

Admission pair (new exclusive stamp; old stamps preserved):

- Pair stamp: `target/dam-break-perf/2026-09-21T16-07-02Z/` (`pair.json`,
  `kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`)
- Pair `git_head`: `907bdf360c423184df1dc9210cce60a65a073c8f`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- Sibling samply stamp (next leftover-ranking input only):
  `target/dam-break-perf/2026-09-21T16-07-39Z/` (`profile-identity.json`
  `kind: samply_cpu`, `not_timing_authority: true`; no `pair.json` in that
  stamp)
- Profile `git_head`: `907bdf360c423184df1dc9210cce60a65a073c8f`

| Engine      | Particles | Wall ms     | ms/step  | Compiler                              |
| ----------- | --------- | ----------- | -------- | ------------------------------------- |
| native Rust | 1920      | 3247.300834 | 5.412168 | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 215.20175   | 0.35867  | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `15.08956518243927` (copied from that `pair.json` only).
Strictly below Wave 1 `21.00845179052245`.

Plan 24-04 then looped leftover scalar kernels from that sibling profile (and
later retargets) until the unprofiled pair crossed ≤ 3. Rank leftovers from
live sidecars, not by pasting this table. samply / `[profile.profiling]` /
`step_profiled` / dhat clocks stay `not_timing_authority`.

## PERF-GATE leftover close

Unprofiled Dam Break Medium pair after leftover waves (locked 1920 particles,
60 warmup + 600 measured steps). This is the 3× number. Do **not** quote
samply duration as this figure. D-06: further ranked ~2% leftover frames were
not gold-plated after the gate.

- Pair stamp: `target/dam-break-perf/2026-09-21T20-28-24Z/` (`pair.json`,
  `kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`)
- Pair `git_head`: `97c15984d660bb18f4f4512211bc32a466539081`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- Last leftover-ranking sibling before the gate-closing kernel:
  `target/dam-break-perf/2026-09-21T20-18-21Z/` (`profile-identity.json`
  `kind: samply_cpu`, `not_timing_authority: true`; no `pair.json` in that
  stamp)

| Engine      | Particles | Wall ms    | ms/step  | Compiler                              |
| ----------- | --------- | ---------- | -------- | ------------------------------------- |
| native Rust | 1920      | 636.031584 | 1.060053 | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 215.149292 | 0.358582 | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `2.956233683539149` (copied from that `pair.json`
only). PERF-GATE on this host for the locked recipe. Not a reviewed report;
`reference/performance/manifest.toml` `reviewed_reports` remains empty.

Landed leftover kernels (names from live `rust.json.syms.json` plus gecko
`stackTable` ranking, not hunt-list paste), in commit order after 24-03
`check_invariants`:

1. skip no-pending identity compact
1. preallocate neighborhood proxy pairs (`ParticleNeighborhood::from_view`)
1. skip hot-path semantic contact collect
1. skip idle body-contact listener diffs
1. defer weight recompute to the solver Weight pass
1. AABB-prefilter fixture-particle contacts
1. skip release neighborhood pair revalidation
1. commit indexed particle contacts without ID resolve
1. AABB-prefilter fixture-particle CCD
1. skip materializing neighborhood `ParticleNeighborPair`
1. skip solver velocity diff scans
1. skip LimitVelocity identity snapshots
1. skip `ParticleStorage` clone in `replace_solver_candidate`
1. skip particle solver candidate Arena clones
1. skip `BoundaryCandidate` pass clones
1. skip release boundary lane validation (`validate_source_lanes` /
   `validate_candidate`)
1. skip `ParticleStorage` clone in `run_particle_solver` happy-path backup

Shared `liquidfun` particle/rigid stepping only. Public `ParticleId`
unchanged. SIMD / Rayon / `-ffast-math` / `-march=native` / PGO / lifting
`unsafe_code = "forbid"` were not used as closers.

## Named functions

Each listed frame is classified into exactly one cause class. C++ names appear
only for shape mismatches, by comparison to pinned LiquidFun
`7f20402173fd143a3988c921bc384459c6a858f2` (`FindContacts_Reference`,
`UpdateProxies` / `SortProxies`, in-place `b2World::Step`, `NDEBUG`). No C++
samply wrap was run.

| Symbol                                                                                                                 | Approximate share             | Cause class                     | C++ counterpart (shape mismatch only)                                                                                                                                    |
| ---------------------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `liquidfun::particle::contact::particle_rows` (leaf; inlined `ParticleSystemView::particle_ids` `.position` scan)      | ~93.6% self                   | algorithm/shape differences     | `b2ParticleSystem::FindContacts_Reference` calls `AddContact(a->index, b->index)` from `Proxy.index`; contacts keep integer indices, they do not re-scan identity slices |
| `<liquidfun::identity::ParticleGroupId as SliceContains>::slice_contains` (parent `ParticleStorage::check_invariants`) | ~2.2% self                    | checks that survive `--release` | `b2Assert` compiled out under `NDEBUG` in `oracle-release`                                                                                                               |
| `ParticleStorage::replace_solver_candidate`                                                                            | ~1.6% inclusive (~0.01% self) | checks that survive `--release` | C++ solver mutates member buffers in place; this path clones `ParticleStorage` then `check_invariants()?` on the release candidate                                       |
| `ParticleNeighborhood::from_view`                                                                                      | ~0.4% self / ~0.6% inclusive  | algorithm/shape differences     | `UpdateContacts` → `UpdateProxies(m_proxyBuffer)` + `SortProxies(m_proxyBuffer)` rewrite the in-place proxy buffer                                                       |
| `alloc::raw_vec::RawVecInner::finish_grow`                                                                             | ~0.5% self                    | per-step allocation             | (none listed; C++ `m_contactBuffer` / member scratch reuse is the nearby shape, but this leaf is the allocator grow)                                                     |
| `Vec<ParticleContact>` `SpecFromIterNested` collect                                                                    | ~0.2% self                    | per-step allocation             | `FindContacts` writes into reused `m_contactBuffer` (`SetCount(0)` then append)                                                                                          |
| `ParticleStorage::recompute_weights`                                                                                   | ~0.3% self                    | extra per-particle work         | (none; not a listed C++ shape split)                                                                                                                                     |
| `ParticleContactUpdate::generate`                                                                                      | ~0.3% self / ~94.3% inclusive | algorithm/shape differences     | `UpdateContacts` / `FindContacts` parent of the index-preserving contact pass                                                                                            |
| `particle::contact::listener_effects` / body-contact `listener_effects`                                                | ~1.0% + ~0.4% self            | extra per-particle work         | listener notify uses stored indices; Rust re-resolves rows with `particle_rows` / `particle_row`                                                                         |
| `pressure::damping`                                                                                                    | ~0.2% self                    | extra per-particle work         | (none; kernel time, not a first-move SIMD target)                                                                                                                        |

`particle_rows` is the bulk of named sampled time. It maps each
`ParticleId` pair to dense rows with
`view.particle_ids().iter().position(...)` inside
`ParticleContactUpdate::generate` (and again from `validate_pairs` /
`listener_effects`). That is extra identity-to-row work C++ does not do on
the `FindContacts` path.

## Heap

Sidecar-first `classify_profile_symbols` on bundle
`target/dam-break-perf/2026-09-21T04-34-32Z/` matched allocator/`Vec` needles, so
private dhat ran once at the locked 60 warmup + 600 measured steps (no recipe
shrink). The dump is gitignored and `not_timing_authority`. It is not the 3×
number and must not be copied into the unprofiled ratio table.

- Heap stamp: `target/dam-break-perf/2026-09-21T04-47-09Z/`
- Dump: `target/dam-break-perf/2026-09-21T04-47-09Z/dhat-heap.json`
- Identity: `heap-identity.json` (`kind: dhat_heap`, `cargo_profile: profiling`,
  `features: ["dhat-heap"]`, `source_stamp: 2026-09-21T04-34-32Z`)
- `matched_needles`: `alloc::`, `__rdl_alloc`, `alloc::vec::Vec`, `RawVec`,
  `to_vec`, `GlobalAlloc`, `core::alloc::`, `core::clone::`
- Heap identity `git_head`: `8d9c6f2647317d3d99516e19c263eb4ac578b4b7` (docs
  commit after the named ranking; physics bytes unchanged from MEASURED_HEAD)

Command used: `cargo xtask playground dam-break-heap --stamp 2026-09-21T04-34-32Z`
because default latest-stamp selection requires current `git rev-parse HEAD` to
equal the 23-07 stamp `git_head`. Do not `git add` `dhat-heap.json`.

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
