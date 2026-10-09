---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-09T17:16:46.249Z
phase: 35-speed-up-the-slowest-scenes
plan: "06"
subsystem: particle-solver
tags: [rust, performance, ccd, spatial-query, soundness, fingerprint]

requires:
  - phase: 35-05
    provides: "Engine A1 + A5 (spot-A5), ABBA method, abba.sh, keep_rule.py, before-full.jsonl"
provides:
  - "A6 kept: moving fixtures at particle iteration 0 use a verified conservative query pad instead of the 0..n CCD scan (2256dd8cd)"
  - "35-PROFILES.md A6 row, 35-06 run notes and the washing-machine Target record"
  - "moving_fixture_query.rs: pure pad computation with the written soundness argument"
affects: [35-07, 35-08, 36]

tech-stack:
  added: []
  patterns:
    - "Closed-form conservative bound plus one f32 verification, with a full-scan fallback on any failure"
    - "Filtered-versus-full-scan equality tests, plus a mutation check that a weaker bound fails them"

key-files:
  created:
    - crates/liquidfun/src/world/particle_coupling/moving_fixture_query.rs
    - crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-06-SUMMARY.md
  modified:
    - crates/liquidfun/src/world/particle_coupling.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md

key-decisions:
  - "A6 kept: the moving-fixture CCD pad cut washing-machine 29.0% / 28.9% in both ABBA pairs (2.024 -> 1.436, 2.018 -> 1.435 ms/step), with 25/25 fingerprints and no confirmed regression"
  - "A7 (per-fixture transform validation) was not tried because A6 passed D-11"
  - "The pad adds a per-meter rounding budget (64 f32 epsilons per meter of coordinate scale) to the planned relative and absolute margins, so the bound stays sound at large coordinates"
  - "Candidates with a non-finite velocity keep the full scan, so invalid input fails at the same particle as before"

patterns-established:
  - "Prove a spatial filter conservative in the module doc, fall back to the original scan whenever the bound cannot be verified, and show with a mutation check that the tests catch a weaker bound"

requirements-completed: []

duration: 478min
completed: 2026-10-09
---

# Phase 35 Plan 06: Moving-Fixture CCD Filter Summary

**A6 is kept. Moving fixtures at particle iteration 0 now query particles with a proven conservative pad instead of scanning all of them. Washing-machine dropped from 2.02 to 1.44 ms/step (−29.0% / −28.9% in the two ABBA pairs), and all 25 fingerprints stayed bit-identical. The scenes that share this path also got faster: soup-stirrer about −11%, water-wheel about −5% and theo-jansen about −26%. In the full run, liquid-bubbler was −18.5% and soup −11.7%.**

## Performance

- **Duration:** about 8 h (2026-10-09T09:18Z to 17:17Z). Coding and timing took about 40 min. The rest went to the full checks under syspolicyd launch stalls: workspace clippy took 17 min, the workspace build 30 min, and `cargo test -p liquidfun --all-features` 5 h 51 min.
- **Started:** 2026-10-09T09:18:50Z
- **Completed:** 2026-10-09T17:16:46Z
- **Tasks:** 2
- **Files modified:** 3 engine files (1 modified, 2 new), plus 35-PROFILES.md

## Accomplishments

- **Pad computation, in `moving_fixture_query.rs`:**
  - At iteration 0 the ray start is an affine map, `S(p) = M p + t`.
  - The module doc proves that every particle that can hit lies inside the fixture AABB padded by `max(motion, D0 / (1 - L))`:
    - `D0` is the largest corner displacement over the AABB. `f(p) = |S(p) - p|_inf` is convex, so its maximum is at a corner.
    - `L` is the inf-norm of `M - I`: `|cos d - 1| + |sin d|` for polygons, edges and chains, and `|k - 1|` for circles.
    - The proof clamps `p` into the AABB and uses the Lipschitz bound.
  - The code adds relative, absolute and per-meter rounding margins, overshoots the closed form by 5%, and checks the padded box's corners once with the real f32 start computation.
  - It returns `None`, which keeps the full scan, when `L >= 1`, an input is not finite, or the check fails. There is no fixed-point loop.
  - At washing-machine rotation (0.035 rad, a rib about 1.5 m from the body origin, motion 0.02) the pad is below 0.1 m.
- **Wiring, in `particle_coupling.rs`:**
  - The fixture loop is now `collect_fixture_hits`, which takes the records so tests can supply any pair of transforms.
  - For moving fixtures at iteration 0 it calls `query_particles_for_fixture` with the pad and keeps the full scan as the fallback.
  - The static and later-iteration branches still use the `motion` pad, as before.
  - A comment notes that `FIXTURE_CONTACT_FILTER` hook calls now run in query order for these fixtures, as they already did on the static path. The stable `sort_by_key(particle)` keeps the hit list the same.
- **Tests:** 7 unit tests compare filtered and full-scan hit lists bit for bit:
  - rotating polygon: fewer than 400 rows are visited, so the filter engages
  - washing-machine pad convergence
  - fast rotation falls back to the full scan
  - translating and rotating circle with an off-center body local center
  - chain with 4 children plus an edge
  - static fixtures and iteration 1
  - a 0.3 rad swept edge whose hits start outside the motion-only pad
  - A mutation check replaced the pad with `motion` alone. 3 tests failed: swept edge, washing-machine and fast rotation. So the tests catch a pad that ignores the start remap.
- **A/B against `spot-base-06`** (a hard link of `spot-A5`; the engine was unchanged since `216de3929`):

  | Comparison | Pair 1 | Pair 2 | Note |
  | --- | --- | --- | --- |
  | Targeted, washing-machine | 2.024 (min 2.000) → 1.436 | 2.018 (min 2.006) → 1.435 | |
  | Shared path, soup-stirrer | 1.130 → 1.013 | 1.146 → 1.014 | Non-target; below the base min in both pairs |
  | Shared path, water-wheel | 1.254 → 1.192 | 1.276 → 1.210 | Non-target; below the base min in both pairs |
  | Shared path, theo-jansen | 0.195 → 0.147 | 0.198 → 0.144 | Non-target; below the base min in both pairs |
  | Isolated ABBA, hydraulic-fountain | 0.786 vs base max 0.793 | 0.787 vs 0.811 | Listed by the full run; not a regression |
  | Isolated ABBA, liquid-timer | 0.716 vs base max 0.723 | 0.724 vs 0.732 | Listed by the full run; not a regression |

  - Fingerprints: `keep_rule.py before-full.jsonl A6-full.jsonl ""` printed `fingerprint mismatches: none` for all 25 scenes.

## Task Commits

1. **Task 1: A6, conservative spatial query for moving fixtures at iteration 0:** `2256dd8cd` (perf)
1. **Task 2: A/B A6, keep, and record (A7 not needed):** `78395d381` (docs)

## Files Created/Modified

- `crates/liquidfun/src/world/particle_coupling/moving_fixture_query.rs` (156 lines): `moving_fixture_query_pad` and its soundness argument
- `crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs` (472 lines): the 7 equality and convergence tests
- `crates/liquidfun/src/world/particle_coupling.rs` (547 lines; the limit is 628): `collect_fixture_hits` with the moving-fixture pad branch
- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: the A6 row, 35-06 run notes, and the washing-machine Target record
- Local only (gitignored), under `target/phase35/`:
  - Binaries: `bin/spot-base-06` (hard link of `spot-A5`) and `bin/spot-A6`
  - Provenance: `A6.source-sha`
  - Timing output: `A6-*.jsonl`, `A6-shared-*`, `A6-iso-*` and `A6-*.uptime`
  - Logs: `A6-checks.log`, `A6-unit*.log`

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Correctness] Per-meter rounding budget in the pad**
- **Found during:** Task 1
- **Issue:** The planned margins are a relative margin of 1e-3 and an absolute margin of 1e-3 m. They cover f32 rounding only near playground coordinates. Rounding in the start computation grows with `|p|` and the transform positions, and with `L` near 0 (circles) the relative margin on `L` is also near 0.
- **Fix:** `ROUNDING_PER_METER = 64 * f32::EPSILON` is added to `L`. The constant term also gets that budget times the coordinate scale. At washing-machine scale this adds about 1e-5 m.
- **Commit:** `2256dd8cd`

**2. [Rule 2 - Correctness] Circle `L` from the actual rotation norms; non-finite velocities keep the scan**
- **Found during:** Task 1
- **Circles:** The plan sets `L = 0` for circles. The actual linear part is `k I` with `k = |q_cur|² |q_prev|²`, so the code uses `L = |k - 1|`. That is honest when rotations are not exactly unit length.
- **Velocities:** `max_particle_motion` ignores NaN speeds. A particle with a non-finite velocity outside the box would therefore be skipped, where the full scan fails at that particle. Candidates with any non-finite velocity now keep the full scan for moving fixtures. The check is lazy and runs at most once per call.
- **Commit:** `2256dd8cd`

### Process deviations

**3. spot-base-06 is a hard link, not a rebuild**
- `git diff 216de3929 285259f4d -- crates/` was empty, so `spot-base-06` is a hard link of `spot-A5`. This follows the host-condition guidance.

**4. Timed before the full checks; committed after them**
- I timed `spot-A6` from a recorded working-tree digest, ran the full checks, re-checked the digest (`0835de0e…`, equal), and then committed. This is the 35-05 pattern.

**5. Extra test beyond the plan's 6**
- I added `swept_edge_hits_beyond_motion_pad_are_kept`. At 0.05 rad with the fast rising grid, the particle motion dominates the pad, so those tests alone would not catch a pad that ignores the start remap. The mutation check confirms the new test does catch it.

**6. [Rule 3 - Blocking] wasm32 check uses `--lib`**
- This carries over from 35-03 to 35-05. `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` passed.

---

**Total deviations:** 6: 2 correctness hardenings of the pad, 1 blocking carry-over, and 3 process deviations. No authored scene settings changed.

## Issues Encountered

- **Launch stalls.** `cargo test -p liquidfun --all-features` took 5 h 51 min. I stopped the Task 2 verify re-run (`cargo test -p liquidfun --all-features particle_coupling`) after its lib target passed: 9 passed, 0 failed, including the 7 new tests. The remaining integration binaries were only being relaunched to run 0 matching tests. They had already passed on the identical source in the full run.
- **A stray empty file `=` in the repo root.** It was created at 10:09Z while the full checks ran. None of my commands wrote it, so I left it uncommitted and untouched.
- **Verification evidence (A6, `2256dd8cd`):**
  - `cargo fmt --all --check`: pass
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
  - `cargo build --workspace --all-targets --all-features`: pass
  - `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: pass
  - `bun scripts/bright-builds-check.ts all`: 0 findings
  - `cargo test -p liquidfun --all-features` (the same as root `cargo test --all-features`): 75 `test result: ok`. That is 1,073 passed and 0 failed; the lib has 479 passed.
  - `cargo test -p liquidfun-wasm --all-features`: 316 passed, 0 failed
  - Fingerprints: 25/25 equal to `before-full.jsonl`
- **Not run:** the full workspace test suite, the same scope as 35-05.

## Known Stubs

None.

## Next Phase Readiness

- HEAD engine is A1 + A5 + A6. Later plans should build their base from `2256dd8cd` or later. `spot-A6` is that engine.
- The washing-machine full-scan CCD hot path is gone. Re-profile before planning more work on it.
- PERF-08 and PERF-09 stay Pending.

## Self-Check: PASSED

- FOUND: crates/liquidfun/src/world/particle_coupling/moving_fixture_query.rs (`pub(super) fn moving_fixture_query_pad`, `convex`, `L < 1`, `1.05`)
- FOUND: crates/liquidfun/src/world/particle_coupling/moving_fixture_query_tests.rs (`full_scan`, all 6 planned test names plus the swept-edge test)
- FOUND: `moving_fixture_query_pad` in crates/liquidfun/src/world/particle_coupling.rs
- FOUND: target/phase35/bin/spot-base-06 and spot-A6
- FOUND: commit 2256dd8cd and commit 78395d381
- FOUND: the `| A6 |` row with Decision `kept`, Fingerprints `yes (25/25)`, Commit `2256dd8cd`, the shared-path numbers and `FIXTURE_CONTACT_FILTER hook calls`. The washing-machine Target record mentions A6. `git status --porcelain -- crates` is empty.
