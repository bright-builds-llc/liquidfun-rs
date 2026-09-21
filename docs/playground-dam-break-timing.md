# Playground Dam Break exploratory timing

**Unreviewed local sample.** This is on-demand diagnosis of the current playground
Dam Break Medium / Normal scene. It is not a public performance claim, not
compatibility evidence, and not Phase 12 evidence. Do not copy these numbers
into `reference/performance/manifest.toml`. Do not summarize them as
“Rust is X% slower.” Re-run after scene or solver changes.

## Reproduce

```console
just playground-dam-break-bench
```

That recipe builds native Rust `--release` and the C++ `oracle-release` extra
target, times 60 untimed warm-up steps plus 600 timed `World::step` /
`b2World::Step` calls, and prints a Markdown table. Construction, particle
insertion, warm-up, capture, and rendering stay outside the timer.

The workload is the live playground Dam Break Medium / Normal recipe: 1920
particles, radius `0.06324555`, spacing `0.101193`, gravity `(0, -10)`,
`dt = 1/60`, velocity 8 / position 3 / particle 2.

## Current recorded sample

SHA-bound unprofiled pair from `target/dam-break-perf/2026-09-21T04-32-19Z/`
(`pair.json`; also copied into audit bundle
`target/dam-break-perf/2026-09-21T04-34-32Z/`). This is not a Phase 12 public
claim. A first exploratory sample at
`1e5cbcc124becd363049c4d62c60b715bc0d7897` remains historical only.

- git HEAD: `6d98531ac799987c209d3fd1e572e482fcab5da6`
- OS/arch: `macos` / `aarch64`
- CPU: `Apple M4 Max`
- logical cores: `16`
- warmup steps: `60`
- measured steps: `600`

| Engine      | Particles | Wall ms      | ms/step    | steps/s     | Realtime factor | Compiler                              |
| ----------- | --------- | ------------ | ---------- | ----------- | --------------- | ------------------------------------- |
| native Rust | 1920      | 71001.949958 | 118.336583 | 8.450472    | 0.140841        | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 216.777375   | 0.361296   | 2767.816521 | 46.130275       | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `327.53395024734476` (from `pair.json` only).

Rust used `--release`. C++ used the scalar `oracle-release` wrapper (no
`-ffast-math`, no `-march=native`).

## How to read

Realtime factor is `(1/60) / seconds_per_step`. A value of `1.0` would keep up
with the playground clock. A value below `1` means the playground catch-up loop
will request extra steps and stutter.

This table times one scene on one host. It does not prove parity with upstream
LiquidFun and it does not extend the sealed Phase 12 matrix.
