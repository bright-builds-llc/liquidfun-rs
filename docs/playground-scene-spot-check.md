# Playground scene spot-check

**Unreviewed local sample.** Not a C++ pair, not Phase 12, and not the Dam Break
3× number. This is a native `--release` headless walk of five playground scenes
after the shared-path gate. Do not copy these numbers into
`reference/performance/manifest.toml`. Do not summarize them as a Rust-versus-C++
ratio.

## Reproduce

```console
just playground-scene-spot
```

That recipe runs `SessionCore::create(SceneId::…)` for Fountain, Float or Sink,
Color Mixer, Jelly Drop, and Water Wheel, then loops `advance(1)` for 60 untimed
warmup steps plus 120 measured steps. Scene `on_advance` hooks run, so Fountain
and Water Wheel emit. Dam Break is not in this table; the unprofiled pair remains
the 3× authority. The stamp kind is `native_scene_spot` with
`not_timing_authority` true and no `rust_over_cpp_ratio`.

## Current recorded sample

Exclusive stamp `target/dam-break-perf/2026-09-21T21-10-50Z/scene-spot.json`.
This stamp has no `pair.json`. Gate pair stamps
`2026-09-21T20-36-30Z` and `2026-09-21T20-38-50Z` were not overwritten.

- git HEAD: `cfdbaabd4b56f9b1d35e50ee283b30d87c540715`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- compiler: `rustc 1.97.0 (2d8144b78 2026-07-07)`
- warmup steps: `60`
- measured steps: `120`
- stepping: `advance(1)`

| Scene         | start_particles | end_particles | wall_ms     | ms/step   |
| ------------- | --------------- | ------------- | ----------- | --------- |
| Fountain      | 1               | 3200          | 2421.112916 | 20.175941 |
| Float or Sink | 1800            | 1800          | 58.265083   | 0.485542  |
| Color Mixer   | 1154            | 1154          | 43.558125   | 0.362984  |
| Jelly Drop    | 793             | 793           | 31.168333   | 0.259736  |
| Water Wheel   | 1               | 3200          | 2584.517583 | 21.537647 |

All five finished with `timed_out` false. Fountain and Water Wheel particle
counts grew, so emission hooks ran. No hang, timeout, or construction error.
Non-catastrophic completion is the D-11 bar; this table is not a C++ pair and
does not refresh the Dam Break 3× number.

## PERF-CANARY2

Dam Break and these five spot-checks share the same dominant particle-contact
cluster. Fountain and Water Wheel take more wall time because they emit up to
the 3200-particle cap during the 180 `advance(1)` steps; that is still
particle-contact work, not a rigid-solve hang or a different failure mode. No
second native profile stamp was captured.

## How to read

`wall_ms` is the measured 120-step loop only. Construction and warmup stay
outside that timer. `live_particle_count` snapshots the live system, so Fountain
end counts can grow past the construction field. This is an unreviewed local
sample on one host.
