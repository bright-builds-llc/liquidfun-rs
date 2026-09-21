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

SHA-bound unprofiled pair from `target/dam-break-perf/2026-09-21T16-07-02Z/`
(`pair.json`, `kind: unprofiled_pair`,
`timing_authority: unprofiled_wall_clock`). This is not a Phase 12 public
claim. Historical pointers that stay on disk and must not be overwritten:

- Phase 23 MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`, pair stamp
  `2026-09-21T04-32-19Z`, `rust_over_cpp_ratio` `327.53395024734476` (also
  copied into audit bundle `target/dam-break-perf/2026-09-21T04-34-32Z/`).

- Wave 1 admission pair `2026-09-21T15-50-46Z`,
  `rust_over_cpp_ratio` `21.00845179052245`.

- A first exploratory sample at
  `1e5cbcc124becd363049c4d62c60b715bc0d7897` remains historical only.

- git HEAD: `907bdf360c423184df1dc9210cce60a65a073c8f`

- OS/arch: `macos` / `aarch64`

- CPU: `Apple M4 Max`

- logical cores: `16`

- warmup steps: `60`

- measured steps: `600`

| Engine      | Particles | Wall ms     | ms/step  | steps/s     | Realtime factor | Compiler                              |
| ----------- | --------- | ----------- | -------- | ----------- | --------------- | ------------------------------------- |
| native Rust | 1920      | 3247.300834 | 5.412168 | 184.768837  | 3.079481        | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| pinned C++  | 1920      | 215.20175   | 0.35867  | 2788.081417 | 46.468024       | `AppleClang 21.0.0.21000334`          |

`rust_over_cpp_ratio`: `15.08956518243927` (from `pair.json` only). This
unreviewed sample improved on Wave 1 `21.00845179052245` after admitting
release `check_invariants` / `slice_contains` on the solver-candidate path
only. It is not PERF-GATE; that gate still requires `pair.json`
`rust_over_cpp_ratio` ≤ 3.

Rust used `--release`. C++ used the scalar `oracle-release` wrapper (no
`-ffast-math`, no `-march=native`).

## How to read

Realtime factor is `(1/60) / seconds_per_step`. A value of `1.0` would keep up
with the playground clock. A value below `1` means the playground catch-up loop
will request extra steps and stutter.

This table times one scene on one host. It does not prove parity with upstream
LiquidFun and it does not extend the sealed Phase 12 matrix.
