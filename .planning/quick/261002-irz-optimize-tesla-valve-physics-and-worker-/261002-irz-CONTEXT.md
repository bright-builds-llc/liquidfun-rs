---
phase: quick-261002-irz
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-261002-irz
generated_at: "2026-10-02T13:32:58-05:00"
---

# Tesla Valve performance context

Complete all five selected optimizations with persistent measurements that attribute each change individually. Client date:2026-10-02. Starting clean synced HEAD:8c419c4. Apply loaded AGENTS.md, AGENTS.bright-builds.md, standards-overrides.md and relevant architecture/verification/testing/Rust/TypeScript standards; current experimental scope requires ordinary local macOS checks, without mandatory Linux qualification or C++ comparisons. Active lessons were read in full within budgets.

## Language

**Original baseline:** A committed, reproducible measurement of the unchanged production implementation after the benchmark harness is established and before any optimization. Earlier ignored target reports are diagnostics rather than this baseline.

**Stage:** One selected optimization, measured using the same harness before and after against the immediately preceding accepted stage. Stage boundaries provide individual attribution, not a single combined before/after claim.

**Semantic fingerprint:** A deterministic digest of meaningful ordered simulation state and counters at fixed simulation checkpoints, independent of timing, object addresses or serialized raw memory.

**Live simulation worker:** A browser worker that owns the interactive Rust/WASM session and runs fixed simulation steps. Moving work off the main thread is evaluated through responsiveness and actual worker/physics cost; it does not imply faster physics or a fixed60fps result.

## Decisions

- D-01: Establish the benchmark foundation first and commit immutable original raw reports before optimization code changes. Preserve complete provenance and a historical summary under docs/benchmarks/tesla-valve.
- D-02: Optimize spatial wall-contact candidate selection without changing particle/contact ordering or collision semantics. Stages1–3 contain no bridge, capture or renderer optimization.
- D-03: Cache static collision geometry and reuse scratch buffers, with correct geometry/filter/transform invalidation and deterministic reuse behavior.
- D-04: Simplify the Tesla drain query while preserving the exact set/order of destroyed particles and existing safe mutation semantics.
- D-05: Run Tesla live interactive simulation in a worker, preserving ordered controls, lifecycle/disposal, pause/hidden behavior, exports, scale meter and build provenance. Other scenes and deterministic exporters retain the direct backend. Capture once per bounded worker batch may belong to this stage's pipeline adaptation, explicitly attributed; identical fixed-step physics measurements remain available.
- D-06: Cache/batch rendering without changing displayed geometry, particle count/order, render modes or camera/meter behavior.
- D-07: Execute D-02 through D-06 serially. Persist a before/after comparison against the previous accepted stage for each. Investigate regressions/no measurable improvement; never fabricate speedups or fixed60fps. Parent owns stage gates and atomic commits; agents do not commit.
- D-08: Preserve six-centimeter true-circle/continuous-stem geometry, particle radius0.005m, rates1440/2880 per second,4 particle iterations, fixed timestep1/60, mass/flow semantics and inlet policy. Do not lower resolution or alter engine behavior to improve timings.
- D-09: Benchmark reports record hardware/runtime/viewport/backend, warmup/sample settings, solver iteration/settings, scene counters/semantic fingerprints, producer/source identity and confidence/noise. Compare the same workloads; separate physics, frame copy, transport, rendering and main-thread responsiveness where applicable.
- D-11: Primary timed cases are forward/reverse at1440/s, with360/384 warmup steps then40 samples across three sequential replicates. Optional --include-max-rate adds separate immutable2880/s timing cases; never mix them with primary results. All four direction/rate QUALITY containment/behavior cases remain mandatory.
- D-10: Complete full relevant native/WASM/web/browser checks and an independent exact-digest aggregate review. Implementers cannot approve their own work.

## Discretion

Use existing libraries/standard APIs where sufficient. Split bounded execution prompts as needed; do not create a new milestone or rewrite archived ROADMAP. Choose measured repeat counts/workloads large enough for local statistical interpretation. Keep renderer-native static caches and explicit session worker ownership; exact contracts follow code inspection and read-only research.

## Deferred ideas

Physics resolution reductions, new inlet policies, frame-index particle interpolation, engine semantics changes and unsupported performance claims are outside scope. Stable particle IDs are absent from the current frame protocol, so index-based interpolation is not permitted. Package publication remains separately authorized.

## Existing evidence

Ignored earlier Bun measurements observed approximately23.7ms forward with2857 particles and86.9ms reverse with9662 particles, with frame copy below0.4ms. These are preliminary diagnostics, not the requested persisted original baseline and not evidence of a new optimization gain.

Before this work:1032 native core tests,299 release WASM tests,404 web unit tests and53 passing browser tests, with one documented historical optional skip. Added tests may increase these counts; preserve actual results and failed attempts.

## Ownership

Read-only core research covers stages1–3 and crates/liquidfun; read-only frontend research covers harness/worker/render and web. Parent owns harness reports, README/task/state, required WASM integration, complete checks, stage commits and final publication workflow. Agents share the checkout and must preserve others' edits. All five selected optimizations remain required at full fidelity.
