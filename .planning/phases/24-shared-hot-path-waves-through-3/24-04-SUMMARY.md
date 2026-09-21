---
phase: 24-shared-hot-path-waves-through-3
plan: "04"
subsystem: performance-gate
tags: [PERF-GATE, leftover, dam-break, unprofiled-pair]

requires:
  - phase: 24-shared-hot-path-waves-through-3
    provides: first leftover check_invariants pair 2026-09-21T16-07-02Z rust_over_cpp_ratio 15.08956518243927
  - phase: 24-shared-hot-path-waves-through-3
    provides: sibling samply 2026-09-21T16-07-39Z not_timing_authority leftover retarget
provides:
  - unprofiled Dam Break Medium pair.json rust_over_cpp_ratio <= 3.0 on this host
  - exclusive gate stamps 2026-09-21T20-36-30Z (kernel HEAD) and 2026-09-21T20-37-12Z (docs HEAD)
  - leftover scalar kernels in shared liquidfun particle/rigid stepping
affects:
  - 24-05 spot-checks and remaining-delta notes
  - PERF-NOTES / PERF-WASM in Phase 25

tech-stack:
  added: []
  patterns:
    - one named leftover per wave; exclusive unprofiled pair; samply ranking-only
    - debug_assertions analog of C++ NDEBUG for release extra-work
    - take/replace_with_empty instead of ParticleStorage/Arena clones on the happy path

key-files:
  created:
    - .planning/phases/24-shared-hot-path-waves-through-3/24-04-SUMMARY.md
  modified:
    - crates/liquidfun/src/particle/storage/runtime.rs
    - crates/liquidfun/src/particle/proxy.rs
    - crates/liquidfun/src/particle/contact.rs
    - crates/liquidfun/src/particle/body_contact.rs
    - crates/liquidfun/src/particle/solver/boundary.rs
    - crates/liquidfun/src/world/particle_coupling.rs
    - crates/liquidfun/src/world/particle_lifecycle.rs
    - crates/liquidfun/src/arena.rs
    - docs/playground-dam-break-timing.md
    - docs/native-performance-audit.md

key-decisions:
  - "PERF-GATE uses newest same-HEAD unprofiled pair.json rust_over_cpp_ratio, not rust.wall_ms alone and not samply duration."
  - "Stop leftover kernels at <= 3 (D-06); do not gold-plate remaining ~2% frames."
  - "When a leftover improved rust wall but lost on ratio because C++ wall dropped, keep the failed stamp, revert that kernel, try a different named leftover."
  - "After a docs commit moved HEAD, a 3.09 variance pair required one more named leftover (lifecycle ParticleStorage clone skip) before same-HEAD was <= 3 again."

patterns-established:
  - "Sidecar-first outer ranking: leaf-native inline frames, outermost liquidfun:: name; create_particle is warmup."
  - "Preserve every exclusive stamp; never overwrite or call a non-improving sample a win."

requirements-completed: [PERF-GATE, PERF-ADMIT, PERF-SHARED, PERF-BASELINE]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T20:38:00Z

duration: 265min
completed: 2026-09-21
---

# Phase 24 Plan 04: Leftover Waves Through 3× Summary

**Unprofiled Dam Break Medium pair closed at `rust_over_cpp_ratio` `2.9538763508693644` (stamp `target/dam-break-perf/2026-09-21T20-37-12Z/`) without SIMD, Rayon, or lifting `unsafe_code = "forbid"`.**

## Performance

- **Duration:** 265 min
- **Started:** 2026-09-21T16:13:02Z
- **Completed:** 2026-09-21T20:38:00Z
- **Tasks:** 2
- **Files modified:** 24

## Accomplishments

- Looped D-07/D-09 leftover scalar kernels from live sidecar ranking until newest same-HEAD `pair.json` was ≤ 3.0.
- Shared `liquidfun` particle/rigid stepping only. Public `ParticleId` unchanged. `crates/liquidfun-wasm/src/scene/dam_break.rs` untouched. `reference/performance/manifest.toml` still has `reviewed_reports = []`.
- Recorded PERF-GATE as an unreviewed local sample in `docs/playground-dam-break-timing.md` and leftover-until-gate notes in `docs/native-performance-audit.md`. samply / timers / dhat remain `not_timing_authority`.
- D-06: stopped leftover kernels once the unprofiled pair crossed ≤ 3. Did not gold-plate remaining ~2% frames.

## PERF-GATE (Task 1 verify)

Newest same-HEAD unprofiled pair (this is the 3× number):

- Stamp: `target/dam-break-perf/2026-09-21T20-37-12Z/`
- `kind`: `unprofiled_pair`
- `timing_authority`: `unprofiled_wall_clock`
- `git_head`: `48f3c7239a74c4dd0b549351b9c8b2cf7f7b4917`
- OS/arch/CPU: `macos` / `aarch64` / `Apple M4 Max` (16 logical cores)
- rustc: `rustc 1.97.0 (2d8144b78 2026-07-07)`
- C++: `AppleClang 21.0.0.21000334`
- particles: 1920; warmup 60; measured 600
- `rust.wall_ms`: `628.900583`
- `cpp.wall_ms`: `212.906875`
- `rust_over_cpp_ratio`: `2.9538763508693644`

Committed unreviewed notes quote kernel-HEAD pair `target/dam-break-perf/2026-09-21T20-36-30Z/` (`git_head` `d843ba30d0909cc3215108c4814c8b8e97db9c83`, `rust.wall_ms` `607.814417`, `cpp.wall_ms` `211.267084`, `rust_over_cpp_ratio` `2.8769953439599707`). The later docs commit moved HEAD; 20-37-12Z is the same kernel re-benched on that docs HEAD.

## Task Commits

1. **Task 1 leftover:** `986b878` skip no-pending identity compact
1. **Task 1 leftover:** `624fa65` preallocate neighborhood proxy pairs
1. **Task 1 leftover:** `58d7a1f` skip hot-path semantic contact collect
1. **Task 1 leftover:** `2305b3f` skip idle body-contact listener diffs
1. **Task 1 leftover:** `4a786e5` defer weight recompute to solver Weight pass
1. **Task 1 leftover:** `8aab936` AABB-prefilter fixture-particle contacts
1. **Task 1 leftover:** `b74b5c1` skip release neighborhood pair revalidation
1. **Task 1 leftover:** `da597b3` commit indexed particle contacts without ID resolve
1. **Task 1 leftover:** `4f27fde` AABB-prefilter fixture-particle CCD
1. **Task 1 leftover:** `c323159` skip materializing neighborhood ParticleNeighborPair
1. **Task 1 leftover:** `aa339c5` skip solver velocity diff scans
1. **Task 1 leftover:** `6d5baaa` skip LimitVelocity identity snapshots
1. **Task 1 leftover:** `433ce27` skip ParticleStorage clone in replace_solver_candidate
1. **Task 1 leftover:** `6610557` skip particle solver candidate Arena clones
1. **Task 1 leftover:** `ae5daba` skip BoundaryCandidate pass clones
1. **Task 1 leftover:** `2dcc382` skip release boundary lane validation
1. **Task 1 leftover:** `97c1598` skip particle solver storage clone
1. **Task 2 docs:** `f80d5a5` record PERF-GATE unreviewed pair sample
1. **Task 1 leftover:** `d843ba3` skip particle lifecycle storage clone (required after docs-HEAD pair 20-30-10Z ratio 3.09)
1. **Task 2 docs:** `48f3c72` refresh PERF-GATE pair after lifecycle leftover

**Plan metadata:** (this commit)

## Winning unprofiled pairs

| Stamp | HEAD | rust.wall_ms | cpp.wall_ms | rust_over_cpp_ratio |
| --- | --- | --- | --- | --- |
| 2026-09-21T16-07-02Z | 907bdf3 | 3247.300834 | 215.20175 | 15.08956518243927 (24-03 start) |
| 2026-09-21T16-22-08Z | b9a97b8 | — | — | 12.182775137278579 |
| 2026-09-21T16-30-04Z | 986b878 | — | — | 11.089379134612038 |
| 2026-09-21T16-39-17Z | 624fa65 | — | — | 9.084536498784603 |
| 2026-09-21T16-47-32Z | 58d7a1f | — | — | 6.595122367129645 |
| 2026-09-21T16-53-36Z | 2305b3f | — | — | 5.941024525829169 |
| 2026-09-21T17-13-07Z | 4a786e5 | — | — | 5.703208316370151 |
| 2026-09-21T17-18-41Z | 8aab936 | — | — | 5.643972728710345 |
| 2026-09-21T17-24-53Z | b74b5c1 | — | — | 4.998981585939845 |
| 2026-09-21T17-33-05Z | da597b3 | 956.641 | 209.003 | 4.577169042034136 |
| 2026-09-21T17-41-10Z | 4f27fde | 795.272 | 217.781 | 3.6517031128907904 |
| 2026-09-21T17-58-31Z | c323159 | 743.158708 | 215.359834 | 3.4507767497629107 |
| 2026-09-21T19-13-52Z | aa339c5 | 755.89575 | 219.493625 | 3.4438164206363626 |
| 2026-09-21T19-21-28Z | 6d5baaa | 718.950958 | 216.411208 | 3.3221521410295907 |
| 2026-09-21T19-35-39Z | 433ce27 | 697.580875 | 222.285667 | 3.1382179715617924 |
| 2026-09-21T19-53-33Z | 6610557 | 667.768375 | 219.835416 | 3.0375832390901016 |
| 2026-09-21T20-17-50Z | ae5daba | 647.099792 | 214.605625 | 3.015297441527919 |
| 2026-09-21T20-28-07Z | 2dcc382 | 637.916917 | 217.41825 | 2.934054142189076 |
| 2026-09-21T20-28-24Z | 97c1598 | 636.031584 | 215.149292 | 2.956233683539149 |
| 2026-09-21T20-36-10Z | f80d5a5 | 612.727667 | 210.305042 | 2.9135186735085505 |
| 2026-09-21T20-36-30Z | d843ba3 | 607.814417 | 211.267084 | 2.8769953439599707 |
| 2026-09-21T20-37-12Z | 48f3c72 | 628.900583 | 212.906875 | 2.9538763508693644 |

## Failed leftovers (stamps preserved, kernels reverted)

| Stamp | Leftover | rust.wall_ms | cpp.wall_ms | rust_over_cpp_ratio | Why not a win |
| --- | --- | --- | --- | --- | --- |
| 2026-09-21T16-58-44Z | validate_contact_bodies debug-only | — | — | 6.022655530354479 | ratio not improved |
| 2026-09-21T17-07-36Z | in-place solver velocities | — | — | 5.963207131711696 | ratio not improved |
| 2026-09-21T17-50-41Z | fuse generate_indexed into visit_pairs | — | — | 3.743070581779793 | worse than 3.651 |
| 2026-09-21T18-08-06Z | stepping_from_view drop proxies | — | — | 3.5061114181077 | worse than 3.451 |
| 2026-09-21T18-18-12Z | in-place damping | — | — | 3.5723529891213954 | worse than 3.451 |
| 2026-09-21T18-51-33Z | thread_local neighborhood scratch | 772.965 | 220.099 | 3.511898913613257 | worse than 3.451 |
| 2026-09-21T18-58-01Z | take_particle_contacts reuse | 758.891 | 218.071 | 3.4800185405812614 | worse than 3.451 |
| 2026-09-21T19-08-00Z | in-place UpdateProxies + fused FindContacts | 735.154 | 210.075 | 3.499477867945274 | worse than 3.451 |
| 2026-09-21T19-28-39Z | from_view_without_identities | 715.713 | 207.728 | 3.4454334638738593 | ratio worse vs 3.322 |
| 2026-09-21T19-46-54Z | gravity add_solver_velocity_delta | 693.505 | 213.183 | 3.2530961725980894 | ratio worse vs 3.138 |
| (no bench) | skip run_particle_solver backups + restore-all-errors | — | — | — | tests failed (`continuous_resume_does_not_repeat_particle_stages`) |
| 2026-09-21T19-59-22Z | pressure cached_aggregate_particle_flags | 665.485 | 216.202 | 3.08 | ratio worse vs 3.037 |
| 2026-09-21T20-05-21Z | skip particle_systems clone in backup_step_limit_state | 654.039 | 213.749 | 3.0598414989517515 | rust wall better; ratio worse vs 3.037 |
| 2026-09-21T20-10-36Z | skip entire default-limit StepLimitBackup | 691.989 | 213.399 | 3.242707008777528 | rust wall worse |

`2026-09-21T20-30-10Z` (`rust_over_cpp_ratio` `3.0922798110621392`, rust `646.703292`, cpp `209.134791`) is a same-kernel re-bench after `f80d5a5`, not a reverted leftover. C++ wall dropped versus `215.149292`; loop continued with a different named leftover.

## Deviations from Plan

None - plan executed as the leftover while-loop. Failed kernels were reverted with stamps kept. One extra leftover after the first ≤ 3 sample was required because the newest same-HEAD pair after the docs commit was 3.09.

## Isolation

- `rg -n "std::simd|rayon" crates/liquidfun` empty
- `Cargo.toml` `unsafe_code = "forbid"`
- `crates/liquidfun/Cargo.toml` `bitflags` only; no serde/dhat/samply
- `git diff -- crates/liquidfun-wasm/src/scene/dam_break.rs` empty
- `reference/performance/manifest.toml` `reviewed_reports = []`

## Known Stubs

None. PERF-GATE is demonstrated by `pair.json`, not a stubbed ratio.

## Next Phase Readiness

24-05 can document remaining Dam Break delta and run playground spot-checks. Independent review is still required after phase verification; this executor did not self-approve.

## Self-Check: PASSED
