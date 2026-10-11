# Playground scene spot-check

The recipe times every playground catalog scene. The current ranked table and
the v1.4 before/after table are in
[Playground scene survey](benchmarks/scene-survey.md).

**Unreviewed local sample.** Not a C++ pair, not Phase 12, and not the Dam Break
3× number. This is a native `--release` headless walk of playground scenes
after the shared-path gate. Do not copy these numbers into
`reference/performance/manifest.toml`. Do not summarize them as a Rust-versus-C++
ratio.

## History

Before Phase 34, `just playground-scene-spot` timed five scenes (Fountain,
Float or Sink, Color Mixer, Jelly Drop, and Water Wheel). It was recorded on
2026-09-21 at `cfdbaabd4b56f9b1d35e50ee283b30d87c540715`. Those numbers predate
the v1.2 to v1.4 engine changes and are superseded by the
[Playground scene survey](benchmarks/scene-survey.md); they remain in this
file's git history. Dam Break was not in that sample; the unprofiled pair
remains the 3× authority. The stamp kind is `native_scene_spot` with
`not_timing_authority` true and no `rust_over_cpp_ratio`.

## PERF-CANARY2

Dam Break and the playground scenes share the same dominant particle-contact
cluster. Emitting scenes such as Fountain and Water Wheel fill up to the
3200-particle cap while they run; that is still particle-contact work, not a
rigid-solve hang or a different failure mode. No second native profile stamp
was captured.

## How to read

Construction and warmup stay outside the timer. `live_particle_count`
snapshots the live system, so end counts for emitting scenes can grow past the
construction field. These are unreviewed local samples on one host.
