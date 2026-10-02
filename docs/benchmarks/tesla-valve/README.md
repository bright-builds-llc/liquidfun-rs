# Tesla Valve performance history

This directory preserves local, experimental measurements of the Tesla Valve playground. Raw reports are immutable. Every optimization is measured separately against its immediately preceding accepted stage; the original implementation remains available as the cumulative reference.

These measurements describe the recorded machine, runtime, browser and workload. They are not controlled-host certification or a universal frame-rate guarantee.

## Reproduce a run

The repository-owned runner is `bun run bench:tesla` from `web/`. Use `bun run bench:tesla -- --help` for the supported arguments. Build fresh production WASM before comparing source changes. The runner writes new run directories under `runs/` and refuses to overwrite an existing report.

Primary timed cases use 1440 particles/s: forward warms for 360 fixed steps, reverse for 384, then each records 40 one-step samples across three sequential replicates. This captures the reported roughly 3000-particle forward and 9000-particle reverse loads without substituting a maximum-capacity saturation test. Optional maximum-rate timings are a different workload and must not be mixed into a primary comparison. Default and maximum rates in both directions still receive separate containment and flow-quality verification.

## What each record means

- Physics timing measures the actual Rust/WASM step.
- Capture/copy timing measures the production frame boundary, including extraction and validation.
- Render timing is Canvas/WebGL CPU submission, not a GPU-completion measurement.
- Live cadence records actual presented frames and simulation steps per second separately. Moving physics into a worker can improve responsiveness without reducing solver time.
- Counts and named semantic-state fingerprints are taken at fixed simulation checkpoints. They cover exported state rather than raw object memory or addresses; they do not establish exhaustive native/upstream parity.
- Producer/source and WASM hashes identify the actual candidate. Hardware, runtimes, browser/backend, viewport, pixel ratio, render mode, physical settings and workload define comparability.
- Replicates are independent repeated trajectories. Adjacent timed steps within a trajectory are correlated; reported variation is descriptive rather than a manufactured statistical confidence claim.

The five stages retain geometry, particle radius, source rates, four particle iterations and the fixed timestep. A timing improvement does not excuse failed semantic, containment, lifecycle or rendering checks. Failed and incomparable attempts remain identifiable and are excluded from successful comparisons.

## Accepted history

The original run used the unchanged physics implementation at `8c419c4c42aaf99cdfdf8d3d297c184c25d5ae8f`. Its source manifest includes the then-uncommitted benchmark harness; the report records that exact producer diff and artifact hashes. Measurements below are means of three replicate means on Apple M4 Max, Chromium 153, ANGLE Metal, on 2026-10-02.

| Stage    | Forward physics (ms/step) | Reverse physics (ms/step) | Forward live paints/s | Reverse live paints/s | Evidence                                            |
| -------- | ------------------------- | ------------------------- | --------------------- | --------------------- | --------------------------------------------------- |
| Original | 31.374                    | 110.849                   | 31.3                  | 9.1                   | [Raw report](runs/20261002-original-01/report.json) |

The original fixed checkpoints matched across all replicates and the live warmup. Forward checkpoint counts were 2852 to 2857; reverse were 9051 to 9662. The reverse live probe advanced fewer steps, reaching 9389 particles, which is why live count and fixed-checkpoint count must be read separately. Render submission averaged 0.126 ms forward and 0.187 ms reverse; physics dominated this baseline. Later rows report each isolated stage against its immediate predecessor and preserve the original as the cumulative reference.
