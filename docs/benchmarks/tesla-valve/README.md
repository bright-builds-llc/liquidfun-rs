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

| Stage                            | Forward physics (ms/step) | Reverse physics (ms/step) | Forward live paints/s | Reverse live paints/s | Evidence                                                |
| -------------------------------- | ------------------------- | ------------------------- | --------------------- | --------------------- | ------------------------------------------------------- |
| Original                         | 31.374                    | 110.849                   | 31.3                  | 9.1                   | [Raw report](runs/20261002-original-01/report.json)     |
| 1. Spatial wall contacts         | 6.268                     | 32.533                    | 53.7                  | 29.6                  | [Raw report](runs/20261002-stage1-after-01/report.json) |
| 2. Geometry sharing and buffers  | 6.125                     | 32.282                    | 52.1                  | 29.9                  | [Raw report](runs/20261002-stage2-after-02/report.json) |
| 3. Query-only drain index        | 6.042                     | 31.957                    | 50.1                  | 30.2                  | [Raw report](runs/20261002-stage3-after-01/report.json) |
| 4. Live simulation worker        | 5.981                     | 31.574                    | 59.5                  | 29.9                  | [Raw report](runs/20261002-stage4-after-02/report.json) |
| 5. Paths and GPU metadata caches | 5.978                     | 31.656                    | 59.5                  | 29.1                  | [Raw report](runs/20261002-stage5-after-03/report.json) |

The original fixed checkpoints matched across all replicates and the live warmup. Forward checkpoint counts were 2852 to 2857; reverse were 9051 to 9662. The reverse live probe advanced fewer steps, reaching 9389 particles, which is why live count and fixed-checkpoint count must be read separately. Render submission averaged 0.126 ms forward and 0.187 ms reverse; physics dominated this baseline. Later rows report each isolated stage against its immediate predecessor and preserve the original as the cumulative reference.

## Stage 1: Spatial wall contacts

The original report is this stage's before record. The indexed contact path reduced mean physics time by about 80% forward and 71% reverse. All six fixed checkpoint pairs matched the original exactly, including contacts, particle positions and wall geometry. Five regression tests independently compare the legacy full scan, including stateful filter ordering, stale indices, overflow and strict AABB edges. Full native and 299 release WASM tests, including all four rate/direction containment cases, passed.

## Stage 2: Immutable geometry sharing and reusable buffers

Stage 1 is the before record. Two after runs used the same source bytes; all fixed checkpoints still matched the original. The [first run](runs/20261002-stage2-after-01/report.json) and [repeat](runs/20261002-stage2-after-02/report.json) are both retained. The table uses the later accepted repeat as the next stage's before record, not a selected best replicate. The forward physics change is small; reverse timings overlap the preceding replicate variation, so this experiment does not establish a substantial reverse speedup. Sharing and reuse are separately witnessed by tests of backing storage, varying-size buffers, refreshed collider metadata, rejected swaps and error followed by successful retry.

World always restores a usable workspace after a solver Result. Successful commits recycle displaced vectors. Existing consuming kernels can drop candidate capacity on an error; their prior authoritative error/rollback behavior remains intact. All 1044 native and 299 WASM tests passed.

## Stage 3: Query-only drain index

The accepted Stage 2 repeat is the before record. AABB queries retain the checked spatial index and omit neighbour-pair enumeration. Public neighbourhood construction and raycasts still retain their pair path. The measured change is modest (about 1% in each direction); fixed state remains identical to the original. Six new query tests preserve validation, equal-tag ordering, strict boundaries, shape selection, callback termination and pair API behavior. All 1050 native and 299 WASM tests passed.

## Rejected candidates

[Worker candidate 20261002-stage4-after-01](runs/20261002-stage4-after-01/report.json) completed timing and matched all physical checkpoints, but independent lifecycle review found that a control completed while hidden could leave stale geometry in a paused view after visibility restoration. Its [rejection record](verification/stage4-after-01-rejected.json) is preserved. This candidate is excluded from accepted history and must not be used as a stage predecessor. The subsequent corrected worker candidate is measured again against accepted Stage 3.

## Stage 4: Live simulation worker

Accepted Stage 3 is the before record. The corrected worker run retains identical WASM bytes and exact physical checkpoints. Direct fixed-step solver timings are still reported; small differences there are not attributed to worker acceleration. Actual new-snapshot cadence is about 59.5/s forward and30/s reverse. The worker owns stepping/capture/transfer, preserves ordered controls and direct exports, and captures the final batch frame once.

The 16ms main-thread timer-lateness proxy fell from 1.380 to 0.350ms forward and 45.806 to 0.371ms reverse (means of three replicate means). This measures responsiveness, not physics throughput or end-to-end input latency. Reverse simulation still delivers about 30 steps/s at this workload. Only newly delivered snapshots count as paints.

All 1050 native/299 WASM/456 web unit tests passed; 61 browser tests passed with one existing optional forensic skip. Independent source review closed protocol ownership, gravity rejection, abort, control/transport, hidden completion and paused redraw races. A test-only isolated fixture serves worker-quality tests without adding diagnostics to the production app. Rejected candidate 01 and failed validation attempts remain identified separately.

## Stage 5: Static native paths and GPU metadata

The final comparison uses a [fresh before run](runs/20261002-stage5-before-01/report.json) with exactly committed Stage 4 rendering and the [corrected after run](runs/20261002-stage5-after-03/report.json). Core, worker, workload and hardware are unchanged. The before run links accepted Stage 4. All physical checkpoints remain exactly original, and solver WASM bytes are unchanged.

CPU render submission means changed from 0.117 to 0.141ms forward and 0.199 to 0.278ms reverse. Reverse replicate ranges overlap; live cadence changed from 59.62/28.75 to 59.48/29.06frames per second. These results do not establish a rendering speedup. The added mean submission cost is about 0.024/0.079 ms against roughly 6/32ms physics. The requested exact path/uniform/stable-attribute caches remain implemented; their benefit depends on stability and this workload is dominated by physics.

The first [after candidate](runs/20261002-stage5-after-01/report.json) exposed copies of metadata snapshots on constantly changing particle counts. After profiling and a targeted fix, the [second run](runs/20261002-stage5-after-02/report.json) still had variable render timings; both exploratory results remain retained and are not selected as final predecessors. The final candidate uses full straight-line uploads during count churn and exact capacity-reused snapshots when counts stabilize. Native paths retain separate stroke order and current transforms/styles; moving outlines, circles and labels remain live. No particle/resolution reductions or approximate metadata comparisons were introduced.

All 1050 native/299 WASM/472 web unit tests and 61 browser tests passed, with the same optional skip. [Main Chromium appearance checks](verification/stage5-appearance-final.json) show zero differing bytes across 14 comparisons. A raster-layer experiment failed exact antialiasing checks and was replaced by native paths without relaxing pixel tolerance.

## Source and evidence commits

| Stage                | Accepted containing commit                 | Measured record                                     |
| -------------------- | ------------------------------------------ | --------------------------------------------------- |
| Original             | `f6398f205bded23292fbf61e3eca9425d7e95cce` | [Report](runs/20261002-original-01/report.json)     |
| Spatial contacts     | `01728fd31429c8f4b5c88f0ad9ab3f91c70ec30b` | [Report](runs/20261002-stage1-after-01/report.json) |
| Geometry and scratch | `0b9ced7cbcda14cbca26a0d8778bab52423bbe3f` | [Report](runs/20261002-stage2-after-02/report.json) |
| Query-only drain     | `8ac4b5d6e0f00eb2f3feecb0a9293dbc00dc4f7a` | [Report](runs/20261002-stage3-after-01/report.json) |
| Live worker          | `3992aee2c8c18bfb26bd42aee548886f1354c960` | [Report](runs/20261002-stage4-after-02/report.json) |
| Rendering caches     | `aa48f26d02b5957b273956201da8cf3fffed49b2` | [Report](runs/20261002-stage5-after-03/report.json) |

Reports retain the actual pre-commit producer revision, dirty diff/complete manifest and generated artifact hashes; containing commits above identify the finalized code/evidence without rewriting raw producer history. [Machine-readable aggregate](comparison.json) derives cumulative metrics from those reports.

At matched fixed trajectories, physics fell from 31.374 to 5.978ms/step forward (about 81% less) and 110.849 to 31.656ms/step reverse (about 71% less). Final new-snapshot cadence was 59.48/s and29.06/s; the reverse worker still runs below real-time 60 steps/s at this density. Geometry 6 cm, particles 5 mm radius, rates 1440/2880 and four particle iterations remain unchanged. Every selected stage has its own before reference, after report, physical fingerprints, source identity and separate review.
