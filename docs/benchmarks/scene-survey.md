# Playground scene survey

**Local observations, not public speed claims.** This is native `--release`
headless `SessionCore` timing of every playground catalog scene on one machine.
It is not a C++ pair, not Phase 12, and not the Dam Break 3× number. The stamp
kind is `native_scene_spot` with `not_timing_authority` true.

## Reproduce

```console
just playground-scene-spot
```

The recipe runs `cargo xtask playground scene-spot`, which runs
`cargo run -p liquidfun-wasm --release --quiet --bin playground-scene-spot -- --warmup 60 --steps 120 --runs 3`.
Each run builds a fresh `SessionCore`, takes 60 untimed `advance(1)` steps, then
times 120 `advance(1)` steps. Scenes run serially in catalog order.

Pass `--scene <id>` (repeatable) to time only those catalog scenes, for example
`cargo xtask playground scene-spot --scene liquid-tumbler`. Each JSON line also
carries `fingerprint`, a 64-bit FNV-1a hash of the live particle positions,
velocities and colors plus body transforms and velocities after the first run's
warmup and timed steps. Equal fingerprints mean those end-state fields match
bit for bit; particle flags, lifetimes and group records are not hashed.

A `--scene` run writes `scene_filter` with the requested ids and
`full_catalog: false` into the stamp, and its disclaimer covers only the
requested scenes. A full-catalog run writes `scene_filter: []` and
`full_catalog: true`.

## v1.4 before/after

Before is the engine as it was before Phase 35, and after is the final v1.4
engine. The two binaries ran back to back on one machine. These are local
observations, not public speed claims, and they make no FPS promise.

- before: binary `target/phase35/bin/spot-before`, source commit
  `809582431faecfdfa13b87f0d8376527bea7f0b4`, SHA-256
  `5bfc7b614d0701b6c9ba0606dd3b1b0937c5fc931676dff55c294cb6467e0742`
- after: binary `target/phase35/bin/spot-v14-final`, built from source commit
  `5c664d20c5ce76e3b3a1396abd6c60b12844cbe4`, whose last engine commit is
  `edcafebc0`, SHA-256
  `a7d837b1f7e2aafba61b4c392e3db3a7b027e0baf68344372f644b78d15eb10f`
- machine: Apple M4 Max, `arm64`, 16 logical cores, macOS 26.6.2
- compiler: `rustc 1.97.0 (2d8144b78 2026-07-07)`
- build: `cargo build --release -p liquidfun-wasm --bin playground-scene-spot`,
  with `RUSTFLAGS` unset
- flags: 60 warmup steps, 120 measured steps, `--runs 5`, `advance(1)`, all 25
  scenes, in ABBA order: before, after, after, before
- quiet-host gate:
  - Start: before each leg, the gate polls every 15 s until no cargo or rustc
    process runs and the 1-minute load is at or below 6.0. After 1,800 s it
    proceeds anyway and logs `GATE-TIMEOUT`.
  - Void: a leg is void if cargo or rustc is running at the leg's end, or if
    the 1-minute load is above 10.0 then.
  - Counted set: `GATE-WAITED` 2 (120 s before the first after leg and 30 s
    before the second); `GATE-TIMEOUT`, `VOID`, NO_VOID (`VOID-IGNORED`) and
    `PRE-WAIT` all 0.
- load: 1-minute load average 5.31 to 6.42 across the four legs
- stamp: counted UTC span 2026-10-11 03:22:04 to 03:27:12, in the gitignored
  files `target/phase35/v14-{b1,a1,a2,b2}.jsonl`. No xtask stamp was minted,
  because the two saved binaries were run directly.
- fingerprints: 25/25 equal to the Phase 35 baseline in all four legs

Each scene gets one result. Pair 1 is the first before leg with the first
after leg, and pair 2 is the second after leg with the second before leg.

- **Faster:** the after median is below the before minimum in both pairs.
- **Slower:** the after median is more than 4% above the before median in both
  pairs (the amended D-11 floor from Phase 35).
- **Within noise:** everything else.

Result counts: faster 24, within noise 1, slower 0.

Rows are ranked by the mean of the two after medians. Times are ms per timed
`advance(1)`.

| Rank | Scene              | Before median (pair 1 / pair 2) | After median (pair 1 / pair 2)  | Change (pair 1 / pair 2) | Before min-max (pair 1 / pair 2)                              | After min-max (pair 1 / pair 2)                               | Result       |
| ---- | ------------------ | ------------------------------- | ------------------------------- | ------------------------ | ------------------------------------------------------------- | ------------------------------------------------------------- | ------------ |
| 1    | liquid-tumbler     | 23.689 / 23.813                 | 18.602 / 18.831                 | -21.5% / -20.9%          | 23.678-24.400 / 23.552-24.060                                 | 18.362-18.764 / 18.666-18.849                                 | Faster       |
| 2    | tesla-valve        | 3.772 / 3.803                   | 3.189 / 3.048                   | -15.4% / -19.9%          | 3.759-3.854 / 3.785-3.827                                     | 3.048-3.522 / 3.027-3.071                                     | Faster       |
| 3    | stacked-drip       | 2.596 / 2.590                   | 2.234 / 2.239                   | -14.0% / -13.6%          | 2.571-2.684 / 2.582-2.597                                     | 2.225-2.345 / 2.224-2.248                                     | Faster       |
| 4    | particles          | 1.542 / 1.565                   | 1.461 / 1.477                   | -5.2% / -5.6%            | 1.527-1.552 / 1.561-1.577                                     | 1.438-1.497 / 1.463-1.491                                     | Faster       |
| 5    | washing-machine    | 2.038 / 2.052                   | 1.470 / 1.435                   | -27.9% / -30.1%          | 2.036-2.042 / 2.045-2.053                                     | 1.462-1.606 / 1.427-1.438                                     | Faster       |
| 6    | liquid-bubbler     | 1.446 / 1.459                   | 1.158 / 1.170                   | -19.9% / -19.8%          | 1.434-1.487 / 1.452-1.465                                     | 1.150-1.170 / 1.168-1.173                                     | Faster       |
| 7    | soup               | 1.107 / 1.129                   | 0.948 / 0.968                   | -14.4% / -14.2%          | 1.101-1.122 / 1.126-1.214                                     | 0.943-0.955 / 0.965-0.984                                     | Faster       |
| 8    | soup-stirrer       | 1.131 / 1.167                   | 0.939 / 0.964                   | -17.0% / -17.4%          | 1.125-1.154 / 1.157-1.185                                     | 0.935-0.947 / 0.953-0.964                                     | Faster       |
| 9    | wave-tank          | 0.905 / 0.918                   | 0.818 / 0.827                   | -9.7% / -9.9%            | 0.899-0.910 / 0.914-0.953                                     | 0.809-0.820 / 0.817-0.832                                     | Faster       |
| 10   | wave-machine       | 0.925 / 0.883                   | 0.768 / 0.781                   | -17.0% / -11.6%          | 0.875-0.974 / 0.864-1.165                                     | 0.756-0.773 / 0.779-0.782                                     | Faster       |
| 11   | hydraulic-fountain | 0.801 / 0.804                   | 0.766 / 0.769                   | -4.4% / -4.4%            | 0.791-0.805 / 0.799-0.824                                     | 0.756-0.770 / 0.765-0.776                                     | Faster       |
| 12   | water-wheel        | 1.248 / 1.273                   | 0.716 / 0.713                   | -42.6% / -43.9%          | 1.231-1.253 / 1.267-1.317                                     | 0.709-0.781 / 0.707-0.728                                     | Faster       |
| 13   | impulse            | 0.724 / 0.726                   | 0.683 / 0.695                   | -5.7% / -4.2%            | 0.722-0.742 / 0.718-0.729                                     | 0.680-0.687 / 0.688-0.702                                     | Faster       |
| 14   | liquid-timer       | 0.722 / 0.731                   | 0.651 / 0.662                   | -9.9% / -9.3%            | 0.707-0.728 / 0.728-0.765                                     | 0.647-0.654 / 0.660-0.670                                     | Faster       |
| 15   | fountain           | 1.134 / 1.126                   | 0.626 / 0.634                   | -44.8% / -43.7%          | 1.093-1.230 / 1.101-1.169                                     | 0.625-0.635 / 0.627-0.642                                     | Faster       |
| 16   | elastic-particles  | 0.527 / 0.535                   | 0.499 / 0.509                   | -5.3% / -5.0%            | 0.513-0.530 / 0.534-0.541                                     | 0.490-0.507 / 0.502-0.522                                     | Faster       |
| 17   | dam-break          | 0.522 / 0.516                   | 0.448 / 0.453                   | -14.2% / -12.1%          | 0.511-0.572 / 0.511-0.525                                     | 0.437-0.451 / 0.449-0.454                                     | Faster       |
| 18   | float-or-sink      | 0.474 / 0.480                   | 0.434 / 0.431                   | -8.3% / -10.1%           | 0.465-0.480 / 0.478-0.491                                     | 0.425-0.448 / 0.426-0.432                                     | Faster       |
| 19   | color-mixer        | 0.311 / 0.311                   | 0.276 / 0.282                   | -11.1% / -9.5%           | 0.309-0.323 / 0.310-0.316                                     | 0.273-0.283 / 0.275-0.285                                     | Faster       |
| 20   | surface-tension    | 0.298 / 0.305                   | 0.268 / 0.274                   | -10.1% / -10.3%          | 0.294-0.300 / 0.304-0.306                                     | 0.267-0.275 / 0.271-0.275                                     | Faster       |
| 21   | rigid-particles    | 0.257 / 0.259                   | 0.219 / 0.222                   | -14.8% / -14.5%          | 0.253-0.266 / 0.258-0.262                                     | 0.212-0.220 / 0.220-0.224                                     | Faster       |
| 22   | jelly-drop         | 0.215 / 0.221                   | 0.206 / 0.208                   | -4.3% / -5.7%            | 0.212-0.219 / 0.218-0.224                                     | 0.202-0.212 / 0.208-0.213                                     | Faster       |
| 23   | theo-jansen        | 0.197 / 0.191                   | 0.140 / 0.141                   | -29.2% / -26.6%          | 0.194-0.215 / 0.190-0.192                                     | 0.137-0.141 / 0.138-0.143                                     | Faster       |
| 24   | sparky             | 0.139 / 0.140                   | 0.126 / 0.127                   | -9.4% / -9.4%            | 0.137-0.141 / 0.139-0.142                                     | 0.125-0.129 / 0.125-0.129                                     | Faster       |
| 25   | drawing-particles  | 0.011 (0.0109) / 0.011 (0.0107) | 0.010 (0.0103) / 0.011 (0.0105) | -5.1% / -1.9%            | 0.010 (0.0103)-0.011 (0.0109) / 0.010 (0.0104)-0.012 (0.0115) | 0.010 (0.0101)-0.011 (0.0106) / 0.010 (0.0103)-0.011 (0.0110) | Within noise |

What changed: Phase 35 kept seven engine changes. They are the reused sorted
contact-proxy order (A1) with a bounded insertion sort (A2r), a bitset walk for
body-contact candidate rows (A5), a conservative spatial query for moving
fixtures in particle CCD (A6), ungrouped particle creation without the
group-record rebuild (A8), a cheaper lifetime eviction index (A9), and hoisted
chain child edges (A3r2). Authored scene settings did not change. No scene is
Slower; drawing-particles is Within noise. The attempts, profiles and keep
decisions are in
[35-PROFILES.md](../../.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md).

## Current ranked survey (v1.4 final)

- binary: `target/phase35/bin/spot-v14-final`, SHA-256
  `a7d837b1f7e2aafba61b4c392e3db3a7b027e0baf68344372f644b78d15eb10f`, built
  from source commit `5c664d20c5ce76e3b3a1396abd6c60b12844cbe4` (last engine
  commit `edcafebc0`)
- command: `target/phase35/bin/spot-v14-final --runs 3`, run directly with no
  xtask stamp. It uses the same flags as `just playground-scene-spot` (60 warmup
  steps, 120 measured steps).
- machine and compiler: as in the before/after section
- counted UTC span: 2026-10-11 03:29:06 to 03:30:18, behind the same quiet-host
  gate. An earlier attempt was void because another repository's cargo was
  running at the end of its second leg, and it was re-run once. The counted run
  had `GATE-WAITED` 2 (30 s each), and `GATE-TIMEOUT`, `VOID-IGNORED` and
  `PRE-WAIT` 0.
- load: 1-minute load average 5.63 to 6.37
- Ranking source: the first of two gated full-catalog runs
  (`v14-rank-b1.jsonl`); the second was used only to confirm fingerprints.
  Both runs had 25/25 fingerprints equal to the Phase 35 baseline and to each
  other.

| Rank | Scene              | Median ms/step |    Min ms/step |    Max ms/step | Start particles | End particles | Interaction |
| ---- | ------------------ | -------------: | -------------: | -------------: | --------------: | ------------: | ----------- |
| 1    | liquid-tumbler     |         18.532 |         18.490 |         18.733 |            3800 |          3800 | default     |
| 2    | tesla-valve        |          3.057 |          3.031 |          3.071 |               0 |          2850 | default     |
| 3    | stacked-drip       |          2.217 |          2.209 |          2.225 |            2000 |          2000 | default     |
| 4    | particles          |          1.474 |          1.470 |          1.486 |            4569 |          4569 | default     |
| 5    | washing-machine    |          1.428 |          1.427 |          1.434 |            3168 |          3168 | default     |
| 6    | liquid-bubbler     |          1.168 |          1.153 |          1.173 |            3000 |          3000 | default     |
| 7    | soup               |          0.964 |          0.963 |          0.967 |            2965 |          2965 | default     |
| 8    | soup-stirrer       |          0.958 |          0.942 |          0.971 |            2792 |          2792 | default     |
| 9    | wave-tank          |          0.817 |          0.808 |          0.838 |            2500 |          2500 | default     |
| 10   | wave-machine       |          0.772 |          0.770 |          0.787 |            2256 |          2256 | default     |
| 11   | hydraulic-fountain |          0.762 |          0.761 |          0.767 |            3200 |          3200 | default     |
| 12   | water-wheel        |          0.722 |          0.709 |          0.731 |               1 |          3200 | default     |
| 13   | impulse            |          0.694 |          0.692 |          0.708 |            2279 |          2279 | scripted    |
| 14   | liquid-timer       |          0.655 |          0.653 |          0.667 |            2247 |          2247 | default     |
| 15   | fountain           |          0.633 |          0.628 |          0.638 |               1 |          3200 | default     |
| 16   | elastic-particles  |          0.508 |          0.502 |          0.510 |            1327 |          1327 | default     |
| 17   | dam-break          |          0.446 |          0.446 |          0.471 |            1920 |          1920 | default     |
| 18   | float-or-sink      |          0.429 |          0.420 |          0.430 |            1800 |          1800 | scripted    |
| 19   | color-mixer        |          0.285 |          0.278 |          0.286 |            1154 |          1154 | default     |
| 20   | surface-tension    |          0.273 |          0.268 |          0.275 |             937 |           937 | default     |
| 21   | rigid-particles    |          0.220 |          0.219 |          0.222 |            1327 |          1327 | default     |
| 22   | jelly-drop         |          0.206 |          0.204 |          0.210 |             793 |           793 | default     |
| 23   | theo-jansen        |          0.139 |          0.138 |          0.141 |             141 |           141 | default     |
| 24   | sparky             |          0.128 |          0.127 |          0.129 |               0 |           550 | default     |
| 25   | drawing-particles  | 0.011 (0.0106) | 0.011 (0.0105) | 0.011 (0.0111) |               0 |            24 | scripted    |

## Original survey (Phase 34)

This is the original survey, recorded before any Phase 35 change and kept for
reference.

### Recorded run

- stamp: `target/dam-break-perf/2026-10-07T06-20-54Z/scene-spot.json`
- commit: `96a3ac6a6cef6c7df4af6e439d98205028d4d526`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- compiler: `rustc 1.97.0 (2d8144b78 2026-07-07)`
- command: `just playground-scene-spot`
- warmup steps: `60`
- measured steps: `120`
- runs: `3`
- stepping: `advance(1)`

All 25 scenes finished with `timed_out` false.

### Ranked table

| Rank | Scene              | Median ms/step | Min ms/step | Max ms/step | Start particles | End particles | Interaction |
| ---- | ------------------ | -------------: | ----------: | ----------: | --------------: | ------------: | ----------- |
| 1    | liquid-tumbler     |         24.440 |      24.435 |      24.648 |            3800 |          3800 | default     |
| 2    | tesla-valve        |          4.118 |       3.984 |       4.452 |               0 |          2850 | default     |
| 3    | stacked-drip       |          2.640 |       2.630 |       2.650 |            2000 |          2000 | default     |
| 4    | washing-machine    |          2.104 |       2.055 |       2.581 |            3168 |          3168 | default     |
| 5    | particles          |          1.580 |       1.574 |       1.615 |            4569 |          4569 | default     |
| 6    | liquid-bubbler     |          1.476 |       1.475 |       1.482 |            3000 |          3000 | default     |
| 7    | water-wheel        |          1.286 |       1.271 |       1.302 |               1 |          3200 | default     |
| 8    | soup-stirrer       |          1.175 |       1.156 |       1.186 |            2792 |          2792 | default     |
| 9    | soup               |          1.160 |       1.151 |       1.165 |            2965 |          2965 | default     |
| 10   | fountain           |          1.124 |       1.120 |       1.170 |               1 |          3200 | default     |
| 11   | wave-tank          |          0.941 |       0.933 |       0.960 |            2500 |          2500 | default     |
| 12   | wave-machine       |          0.882 |       0.882 |       0.887 |            2256 |          2256 | default     |
| 13   | hydraulic-fountain |          0.826 |       0.820 |       0.827 |            3200 |          3200 | default     |
| 14   | impulse            |          0.754 |       0.740 |       0.754 |            2279 |          2279 | scripted    |
| 15   | liquid-timer       |          0.752 |       0.751 |       0.754 |            2247 |          2247 | default     |
| 16   | elastic-particles  |          0.546 |       0.538 |       0.560 |            1327 |          1327 | default     |
| 17   | dam-break          |          0.529 |       0.521 |       0.542 |            1920 |          1920 | default     |
| 18   | float-or-sink      |          0.509 |       0.500 |       0.529 |            1800 |          1800 | scripted    |
| 19   | color-mixer        |          0.320 |       0.320 |       0.322 |            1154 |          1154 | default     |
| 20   | surface-tension    |          0.309 |       0.305 |       0.316 |             937 |           937 | default     |
| 21   | rigid-particles    |          0.261 |       0.260 |       0.264 |            1327 |          1327 | default     |
| 22   | jelly-drop         |          0.224 |       0.223 |       0.225 |             793 |           793 | default     |
| 23   | theo-jansen        |          0.203 |       0.198 |       0.211 |             141 |           141 | default     |
| 24   | sparky             |          0.141 |       0.141 |       0.143 |               0 |           550 | default     |
| 25   | drawing-particles  |          0.011 |       0.011 |       0.011 |               0 |            24 | scripted    |

## How to read

1. Median, min and max are ms per timed `advance(1)` across the fresh runs of
   one invocation: 5 runs per leg in the before/after table and 3 in the ranked
   tables. In the original survey, the min/max spread was the run-to-run noise
   that Phase 35 had to beat.
1. Start particles are the live count right after construction, before any
   survey cue. End particles are the live count at the end of the timed window
   (last run), from `live_particle_count`.
1. `scripted` scenes get one cue before warmup, because they are idle by
   default: Float or Sink `drop-body`, Impulse pointer up at (1, 2), Drawing
   Particles pointer up at (0, 2). Drawing stamps one brush (about 24
   particles), so it ranks near the bottom by design. Sparky is `default`
   because its circles emit particles on contact without input.
1. Later scenes run on a CPU that earlier heavy scenes have already warmed.
   Compare reruns on the same machine only.
1. The before/after table compares two saved binaries run back to back, so only
   that table is before/after evidence. Do not compare numbers across the
   sections of this page; they were recorded on different days under different
   load.
