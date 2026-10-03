# Requirements: liquidfun-rs

**Milestone:** v1.4 Scenario Performance
**Defined:** 2026-10-03
**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

## Milestone Requirements

Optimize simulation and rendering speed across each of the 25 current playground scenarios. Increased memory usage is explicitly allowed; existing physics, deterministic behavior, controls and visual fidelity must be preserved. These requirements concern local reproducible measurements, not universal public speed claims. Numeric PERF numbering continues after historical PERF-01–06; the research's draft PERF-01–11 correspond to final PERF-07–17 below.

### Measurement foundation

- [ ] **PERF-07**: Maintainers can run a versioned workload manifest whose unique scene IDs equal the current 25-scene playground catalog exactly; coverage validation rejects missing, duplicate and unknown IDs.
- [ ] **PERF-08**: Maintainers can reproduce scene-specific default and representative active/load cases using frozen construction settings, ordered input scripts, validated phase windows and checkpoints; empty/settled defaults are labeled and cannot substitute for active workloads.
- [ ] **PERF-09**: Maintainers can inspect a complete original campaign for all 25 scenes collected on one unchanged production source/artifact set before optimization begins, with separate retained failed attempts.
- [ ] **PERF-10**: Maintainers can repeat a scene-local fresh current-before → profile → focused hot-path fix → fresh-after cycle, linked to the immediate accepted predecessor and original campaign; each independently claimed change has its own attributable comparison.
- [ ] **PERF-11**: Maintainers can distinguish native release production-session advance, production WASM advance, capture/copy/parse and renderer submission costs; browser/main-thread/worker profiles identify the actual execution target and native profiles are labeled as native evidence.
- [ ] **PERF-12**: Maintainers can measure actual production-player new-frame cadence, simulation progress, frame/stall distributions and scripted input latency separately, including worker queue/round-trip where applicable; held-frame redraws cannot inflate progress, and CPU submission cannot be labeled GPU completion or displayed FPS.
- [ ] **PERF-13**: Visitors retain existing physics, deterministic ordering, physical settings, controls, failure/lifecycle behavior and visual fidelity, protected by named semantic checkpoints, scene invariants, matched appearance comparisons and independent review.
- [ ] **PERF-14**: Maintainers can measure rendering with explicit mode/backend, viewport/DPR, shading/cap and actual drawn counts: each scene has its unchanged default run and full-particle workload, all six modes receive fidelity coverage, and representative timed cases collectively cover all six modes and applicable fallbacks.
- [ ] **PERF-15**: Maintainers can inspect memory/resource costs and repeated reset/switch retention alongside speed, distinguishing native resident/allocation observations, WASM capacity, frame/transfer bytes, caches and available browser/GPU observations; unavailable metrics are explicit and bounded larger buffers/caches are permitted.
- [ ] **PERF-16**: Maintainers can detect regressions through applicable cross-scene canaries against the immediate predecessor after every shared hot-path change and a complete 25-scene final campaign on one final accepted source.
- [ ] **PERF-17**: Maintainers can look back on immutable raw samples, source/artifact/environment identities, diagnostic profiles, failures/incomparability records and derived per-scene before/after plus original/final summaries, with independently reviewed exact-digest evidence and no aggregate masking individual outcomes.

### Individual scenario iterations

Each SCN requirement is one independently reviewable scenario closure using PERF-10's procedure and the acceptance contract below. Workload discovery in Phase 34 must validate the proposed cases in [FEATURES.md](research/v1.4-scenario-performance/FEATURES.md); research windows are proposals, not benchmark results. Catalog order defines the initial iteration order. Shared fixes discovered in one scenario may benefit others, but every later scene still gets a fresh current-before and its own analysis.

- [ ] **SCN-01**: Maintainers can inspect a completed `wave-machine` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-02**: Maintainers can inspect a completed `dam-break` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-03**: Maintainers can inspect a completed `fountain` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-04**: Maintainers can inspect a completed `float-or-sink` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-05**: Maintainers can inspect a completed `color-mixer` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-06**: Maintainers can inspect a completed `jelly-drop` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-07**: Maintainers can inspect a completed `water-wheel` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-08**: Maintainers can inspect a completed `particles` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-09**: Maintainers can inspect a completed `liquid-timer` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-10**: Maintainers can inspect a completed `surface-tension` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-11**: Maintainers can inspect a completed `elastic-particles` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-12**: Maintainers can inspect a completed `rigid-particles` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-13**: Maintainers can inspect a completed `soup` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-14**: Maintainers can inspect a completed `soup-stirrer` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-15**: Maintainers can inspect a completed `impulse` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-16**: Maintainers can inspect a completed `theo-jansen` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-17**: Maintainers can inspect a completed `liquid-tumbler` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-18**: Maintainers can inspect a completed `drawing-particles` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-19**: Maintainers can inspect a completed `sparky` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-20**: Maintainers can inspect a completed `hydraulic-fountain` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-21**: Maintainers can inspect a completed `wave-tank` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-22**: Maintainers can inspect a completed `liquid-bubbler` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-23**: Maintainers can inspect a completed `stacked-drip` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-24**: Maintainers can inspect a completed `washing-machine` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.
- [ ] **SCN-25**: Maintainers can inspect a completed `tesla-valve` iteration with fresh matched before/after evidence, its own simulation/rendering hotspot analysis, disposition of investigated fixes, fidelity checks and memory tradeoffs.

## Acceptance and evidence contract

- Complete the original campaign before hot-path edits. Harness preparation may add observability but must preserve production behavior and record its source identity. Do not reuse the previous Tesla history as a fresh v1.4 baseline.
- Preserve actual authored settings, including 1/60 timestep, rigid 8/3 iterations, scene-specific particle iterations (Liquid Tumbler 61, Stacked Drip 12, Tesla Valve 4, others 2), particle/material/group counts and geometry, authored gravity, lifetimes/emission and rendering resolution. A case may use existing controls, but its before/after settings and input script must match.
- Use sequential fresh-session unprofiled replicates (at least three by default); freeze sample policy, runtime/state warmup, simulation-step windows and environment before comparing. Show distributions/variability and define repeatability/noise acceptance from observed baseline stability during Phase 34. Diagnostic captures are separate and report instrumentation/build differences.
- Persist raw lightweight evidence, case/attempt indices and per-scene summaries in a new scenario benchmark namespace under `docs/benchmarks/`; large traces may use durable accessible archives with checked-in hashes, provenance and retrieval instructions. Ignored local files alone do not satisfy persistence. Keep historic Tesla records and rejected attempts intact.
- Apply fixes only to measured hot paths. Accept a claimed speed improvement only with repeatable benefit beyond observed noise on matched target workloads and passing behavior/fidelity checks. Report neutral or slower outcomes honestly; revert unsupported speculative changes rather than manufacturing gains. If a scene offers no useful safe optimization, its closure must show the investigated hotspots and evidence supporting that disposition. No arbitrary per-scene percentage or 60 FPS guarantee is imposed.
- Prevent material unexplained regression in affected cases/canaries; diagnose and correct it before accepting a shared change. Final summaries report each scene individually and distinguish cumulative shared gains from its own attributable gains.
- Maintain bounded ownership, cache invalidation and resource cleanup across reset, scene switch, disposal, visibility changes and failures. Increased retained capacity is allowed; unbounded growth or stale state is not.
- Use existing tools first and probe actual optimized-WASM/worker attribution and GPU timer capability before promising unavailable measurements. Capability gaps must be explicit, with useful supported alternatives; numeric GPU/native/browser evidence remains scoped to its actual runtime.
- Use local macOS checks and current hobby-project scope. C++ qualification, Linux performance hosts, all-platform parity and package release are optional separate work, not milestone prerequisites.

## Future Requirements

- Optional controlled-host and cross-platform campaigns, allocator deep dives, and additional control combinations when the local profiles justify them.
- Independently scoped opt-in experimental acceleration only if later authorized behavior/fidelity policy permits it.

## Out of Scope

| Item | Reason |
| --- | --- |
| Lower-quality defaults, fewer particles/iterations, changed physics/geometry, reduced resolution or approximated rendering | User explicitly selected preserved behavior and fidelity. |
| Default nondeterministic solver/SIMD changes or relaxing safe-Rust policy | Speed priority does not waive determinism or safety constraints. |
| Grouping multiple scene investigations into one closure | User requested individual profiling, baselines and hot-path analysis. |
| Required C++ ratio, sealed parity matrix, strict Linux qualification or a new benchmark dependency stack | Current milestone is local scenario performance under experimental scope. |
| Unrelated new scenarios, Drawing material feature expansion or general preset redesign | These do not establish the requested performance improvements. |
| Package publication, release tags or universal public speed claims | Separately authorized work; milestone label is planning metadata. |

## Traceability

Every milestone requirement maps to exactly one primary phase. Phase 34 establishes cross-cutting contracts that also constrain every scenario phase; Phase 60 closes whole-catalog evidence. Repeated constraints do not create duplicate primary mappings.

| Requirement | Phase | Status |
| --- | --- | --- |
| PERF-07 | Phase 34 | Pending |
| PERF-08 | Phase 34 | Pending |
| PERF-09 | Phase 34 | Pending |
| PERF-10 | Phase 34 | Pending |
| PERF-11 | Phase 34 | Pending |
| PERF-12 | Phase 34 | Pending |
| PERF-13 | Phase 34 | Pending |
| PERF-14 | Phase 34 | Pending |
| PERF-15 | Phase 34 | Pending |
| PERF-16 | Phase 60 | Pending |
| PERF-17 | Phase 60 | Pending |
| SCN-01 | Phase 35 | Pending |
| SCN-02 | Phase 36 | Pending |
| SCN-03 | Phase 37 | Pending |
| SCN-04 | Phase 38 | Pending |
| SCN-05 | Phase 39 | Pending |
| SCN-06 | Phase 40 | Pending |
| SCN-07 | Phase 41 | Pending |
| SCN-08 | Phase 42 | Pending |
| SCN-09 | Phase 43 | Pending |
| SCN-10 | Phase 44 | Pending |
| SCN-11 | Phase 45 | Pending |
| SCN-12 | Phase 46 | Pending |
| SCN-13 | Phase 47 | Pending |
| SCN-14 | Phase 48 | Pending |
| SCN-15 | Phase 49 | Pending |
| SCN-16 | Phase 50 | Pending |
| SCN-17 | Phase 51 | Pending |
| SCN-18 | Phase 52 | Pending |
| SCN-19 | Phase 53 | Pending |
| SCN-20 | Phase 54 | Pending |
| SCN-21 | Phase 55 | Pending |
| SCN-22 | Phase 56 | Pending |
| SCN-23 | Phase 57 | Pending |
| SCN-24 | Phase 58 | Pending |
| SCN-25 | Phase 59 | Pending |

**Coverage:** 36/36 milestone requirements mapped exactly once; 0 orphaned requirements; 0 duplicate primary assignments. Phases 34–60 contain 27 phases, including 25 individual catalog-order scenario phases.

*Last updated: 2026-10-03 during v1.4 definition. No benchmark measurements or optimizations have been executed by this initialization.*
