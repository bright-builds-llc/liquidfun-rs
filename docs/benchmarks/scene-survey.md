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
warmup and timed steps. Equal fingerprints mean a bit-identical trajectory.

## Recorded run

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

## Ranked table

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

1. Median, min and max are ms per timed `advance(1)` across the 3 fresh runs.
   The min/max spread is the run-to-run noise that Phase 35 must beat.
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
