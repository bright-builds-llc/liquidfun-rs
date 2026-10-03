# Project Research Summary

**Project:** liquidfun-rs — v1.4 Scenario Performance
**Domain:** Behavior-preserving performance optimization of 25 native Rust / WASM playground scenes
**Researched:** 2026-10-03
**Confidence:** HIGH for existing boundaries and scope; MEDIUM for unproven instrumentation and workload recipes

## Executive Summary

This milestone improves the existing playground's simulation and rendering speed through 25 individual investigations. Keep the native Rust engine, private WASM bridge, SolidJS player and current Canvas2D/WebGL2 renderers. Extend the existing Bun/Playwright/Tesla evidence tooling into a common versioned workload and campaign harness. Every scene needs an authored-default case plus validated representative active cases; empty Drawing, settled drops and partial mechanical cycles cannot stand in for the scene's action. The user explicitly permits increased memory for speed while requiring existing physics, visuals, controls and fidelity to remain intact.

First finish one original campaign over all 25 scenes on unchanged production source. Then process one scene at a time: fresh current-source before, separate simulation and rendering profiles, focused justified optimization, quality checks, fresh after and independent review. Keep both immediate before→after and original→final comparisons because early shared improvements change later scenes before their own turn. Finish with a new all-scene campaign on final accepted source. Native release measurements and samply are useful diagnostics; actual production WASM, actual player ownership and actual browser rendering must substantiate playground gains. CPU draw submission, optional GPU execution, fresh-snapshot cadence and simulation progress are distinct results.

The largest risks are incomparable workloads, diagnostic instrumentation becoming timing authority, speculative caches increasing cost, and optimizations breaking ownership, error recovery or visual behavior. Use immutable source/artifact/runtime-bound attempts, ordinary unprofiled timing, same-step semantic and visual checks, affected cross-scene canaries and explicit unavailable metrics. Preserve rejected attempts and report neutral/inconclusive results honestly; no universal speed percentage or 60 FPS promise is supported. Apply the current local macOS hobby scope: C++ qualification, strict Linux/dedicated hosts, exhaustive certification, package publication and release tags are outside required completion.

**Guidance applied:** `AGENTS.md` local scope, standing authorization and independent AI-review policies; `AGENTS.bright-builds.md`; `standards-overrides.md`; `PROJECT-SCOPE.md`; current `.planning/PROJECT.md`; architecture and verification standards. Both active lesson files were loaded completely: 14,263 bytes / 4,755 conservative estimated tokens. Preserve previous milestone research and historical evidence. This file is parser-owned GSD Markdown and must not be mdformatted.

## Key Findings

### Recommended Stack

Retain existing versions; this is an instrumentation and optimization milestone, not a toolchain migration. [STACK.md](STACK.md) inventories the local tools and capability gaps. Reuse the Tesla runner's content manifests, exclusive writes and comparability logic under a new schema and evidence namespace. Existing Tesla and Dam Break results are supporting history, never fresh v1.4 baselines.

**Core technologies:**

- Rust 1.97.0 / Edition 2024 / Cargo resolver 3 — engine and native scene execution; keep the declared 1.92 MSRV and renderer-free published crate.
- wasm-pack 0.15.0 / wasm-bindgen `=0.2.128` — freshly built production WASM and existing owned-frame bridge; separately identify any optimized symbol-bearing diagnostic build.
- Bun 1.4.2 / Playwright 1.63.0 — private orchestration, immutable records, real Chromium and existing CDP access. Capture actual executable/version/hash; installed browser metadata is not runtime evidence.
- Existing Vite 8.3.0 / TypeScript 7.0.2 / Vitest 5.0.1 — generalized benchmark fixture and meaningful evidence-contract tests.
- samply 0.13.1 / existing optional dhat 0.3.3 — native CPU and allocator diagnosis; native profiles do not establish WASM attribution or rendering cost.
- CDP page and actual-worker CPU profiles, optional tracing/heap probes, optional asynchronous WebGL2 timer queries — capability-test these on the exact local browser; no new profiling dependency is justified.

Ordinary production timing, diagnostic CPU/trace capture and diagnostic memory/GPU capture need separate identities. Detailed paint capture belongs in a separate diagnostic run; instrumentation overhead must be checked. Do not infer a quantified or universally significant penalty from available diagnostic documentation. Browser/WASM symbols, worker attachment, GPU-query validity and memory availability remain Phase 34 probes.

### Expected Features

[FEATURES.md](FEATURES.md) supplies the complete 25-row workload inventory and PERF-01..11 research requirements. Its proposed windows and input scripts require original-source discovery and validation before freezing; elapsed steps alone do not prove an impact, burst, drain or cycle phase occurred.

**Must have (table stakes):**

- Exact catalog equality — 25 recognized unique IDs; original, individual-cycle and full-final coverage with no family-only substitution.
- Validated frozen cases — authored defaults, representative active interactions/load/control variants, ordered step-indexed inputs, recreation epochs and witnessed phase/population checkpoints.
- Original and immediate evidence — one unchanged-source original corpus, fresh current-source before for each scene, separate profiles, justified candidate or documented no-gain investigation, fresh after and final-source rerun.
- Distinct measurement boundaries — native session advance, production WASM advance, capture/getter-copy/validation/free, matched-frame renderer replay and actual live app.
- Full-particle unchanged rendering — shaded-blob primary cases with cap 16384, selected timed mode cases, all-six-mode fidelity for every scene and targeted fallback checks when affected; exact viewport/DPR/camera/backend identity.
- Behavioral quality — deterministic semantic checkpoints plus targeted hidden-state tests, visual checks, actual control/reset/visibility/ownership/error regressions and independent digest-bound review.
- Honest retained evidence — raw samples from at least three fresh trajectories per primary case, replicate summaries, cold setup separately, memory scopes/availability, failures, incompatibility and residual limits.

**Should have (useful differentiators):**

- Readable per-scene active-phase summaries with simulation, rendering and responsiveness outcomes shown separately.
- Immediate and cumulative columns identifying shared gains inherited before the current scene's own change.
- Measured memory-for-speed tradeoffs through bounded buffers, exact caches and precomputed data, including reset/switch retention checks.
- Small relevant material/joint/color/count-churn/scale/worker canary sets after shared changes.

**Defer beyond this milestone:** scene additions/editors, unrelated material/preset expansion, renderer/framework replacement, universal worker migration, default parallel/SIMD/unsafe paths, approximate fast modes, mandatory C++/strict-host certification and public universal performance claims. Publication and release tags need separate authority.

### Architecture Approach

[ARCHITECTURE.md](ARCHITECTURE.md) recommends one validated workload model with thin native, production-WASM, matched-frame replay and real-app adapters. Native and WASM must execute the actual private playground scene factory and `SessionCore`, including emission/drain/escape/pre/post hooks. Comparison, coverage and acceptance relationships stay pure; browser/process/filesystem/profiler effects stay in private adapters. Proposed modules and evidence paths are future implementation choices, not existing capabilities.

**Major components:**

1. **Workload/campaign model** — strict schema, exact scene/case/epoch/input identity, frozen phase windows, backend/render policy and original/current/final relationships.
1. **Execution adapters** — native release, production WASM fixed trajectory, renderer-only replay and actual app with real RAF, chrome and production control paths.
1. **Diagnostic adapters** — native sampling/allocator and actual browser page/worker sampling, traces and optional GPU/memory probes; linked to ordinary source without replacing speed authority.
1. **Immutable evidence store** — new `docs/benchmarks/scenario-performance/` namespace, fresh attempts, source/path+content/artifact/runtime hashes, raw samples/checkpoints/profiles and preserved failures.
1. **Pure validator/comparator** — reject changed workload/environment before calculating ratios, validate quality and exact coverage, derive individual immediate/cumulative reports.
1. **Subsystem-owned exact reuse** — engine scratch, capture staging, owned worker frames and canvas/context caches retain their respective ownership/invalidation/error rules; no exposed mutable engine storage.

Currently ordinary Tesla playback uses the actual Tesla worker; the other 24 scenes use direct ownership. Synthetic media-clock playback forces direct even for Tesla, so it cannot prove normal worker responsiveness. If profiling supports another worker transition, validate it explicitly and distinguish scheduling benefit from solver speed.

### Critical Pitfalls

1. **Contaminated original or stale before** — complete original all-25 coverage before shared optimization; every scene then gathers a fresh immediate before. Rerun all scenes on final source.
1. **Changed work or missed action** — preserve counts/settings/materials/iterations/resolution; separate runtime warmup from physical trajectory and validate active/cycle windows. Drawing must contain genuine painted load; long-cycle scenes need complete cycles.
1. **Boundary and profiler confusion** — ordinary release timing proves gains; native is a diagnostic proxy, page profiles do not include worker cost, CPU submission is not GPU completion and smooth paint cadence is not physics throughput.
1. **Cache/ownership/error regressions** — exact invalidation and lifecycle tests protect colors, transforms, count churn, detached buffers, stale generations, hidden completion, rollback and successful retry. Historical Tesla cache and worker rejections demonstrate these risks.
1. **False certainty or hidden memory** — preserve all replicate ranges and failures; classify overlap/incomparability honestly. Increased bounded memory is allowed, leaks are not. Unknown memory/GPU metrics remain unavailable, never zero; logical texture bytes are not measured VRAM.

See [PITFALLS.md](PITFALLS.md) for P01..20, prior rejected attempts and concrete prevention checks.

## Implications for Roadmap

Use 27 phases: Phase 34 foundation/original campaign, Phases 35–59 one scene each, Phase 60 final closure. The mapping below uses stable catalog order as the fallback. Phase 34's baseline may justify a different cost-driven order, but lock the exact 25-ID/25-phase mapping before scene optimization; do not swap owners after their evidence begins.

### Phase 34: Harness, attribution probes and original campaign

**Rationale:** Fresh comparisons cannot be trusted until workload, source, runtime and measurement boundaries are validated. Original action discovery must precede frozen measurements and every shared optimization.

**Delivers:** Versioned workload/identity/report contracts; exact 25-ID coverage; deterministic script/epoch replay; bounded native/browser adapters; real-app/direct/actual-worker routing; symbol/GPU/memory capability results; immutable writers and comparability validators; original-source discovery followed by complete unchanged-production-source original measurements. Ordinary and diagnostic artifacts are separated. Required cases include nonempty Drawing, Tesla both directions, affected control/material/load cases, complete Bubbler/Stacked Drip cycles and all-six-mode fidelity.

**Addresses:** PERF-01..03, PERF-05..09, PERF-11; reusable foundations for PERF-04 and PERF-10.

**Avoids:** P01/P04..11/P18..20 through rejection tests for missing IDs, stale artifacts, changed scripts/settings/backend/DPR, mid-run source drift and unavailable metrics. No solver/capture/worker/render optimization starts until full original coverage validates. If the measurement contract changes after freezing, retain previous records and establish a new complete compatible original campaign.

### Phases 35–59: One complete investigation per scene

**Common rationale:** Shared fixes may affect many scenes, but no inherited gain replaces that scene's own simulation and rendering diagnosis. Sequential acceptance gives the next scene a definite current-source predecessor.

**Common deliverable for every row:** fresh immediate before; same-workload simulation and render diagnostic artifacts with named hot functions and attribution limits; one focused supported change or documented rejected/no-gain investigation; relevant shared-path canaries; fresh ordinary after; semantic/visual/control/error/lifecycle checks; memory availability/deltas; independently reviewed evidence and readable immediate/cumulative outcome. No scene can close on a profile alone or on someone else's inherited improvement.

**Common requirements:** PERF-04..11 applied independently to each assigned ID. Each row also incorporates its specific FEATURES inventory cases and PITFALLS controls. The table's phase focus is an investigation lead, never a claim that an unmeasured function is expensive.

| Phase | Scene ID | Specific delivered coverage / rationale | Principal pitfalls |
| --- | --- | --- | --- |
| 35 | `wave-machine` | Rocking cycle, stopped/high and live motor/tilt; moving tank presentation | Missed cycle, stale transforms |
| 36 | `dam-break` | Collapse/impact/spread, size variants, obstacle/drop/drag; ordinary water canary | Warming away impact, hook omission |
| 37 | `fountain` | Fill/turnover/off/drain/restart and aim; actual emitter population | Assumed cap, count-churn caches |
| 38 | `float-or-sink` | Actual dropped wood/cork/stone and bounded replacement; buoyancy/body rendering | Empty-body default, changed coupling |
| 39 | `color-mixer` | Mixing strengths, stirring/drag and exact color changes | Position-only proof, stale colors |
| 40 | `jelly-drop` | Shape/softness, impact/deformation/poke and connectivity | Lost initial action, elastic approximation |
| 41 | `water-wheel` | Jet startup/turnover/off/restart/aim and passive wheel motion | Driven-wheel substitution, churn |
| 42 | `particles` | Fluid collapse and falling-ball coupling in open basin | Skipped escape policy, settled-only run |
| 43 | `liquid-timer` | Witnessed shelf draining and bottom-column phases on frozen indices | Arbitrary deadline, omitted drainage |
| 44 | `surface-tension` | Beading/contact/color exchange and later state | Changed materials/colors/fidelity |
| 45 | `elastic-particles` | Mixed spring/elastic group impact and recovery | Missing group state, topology changes |
| 46 | `rigid-particles` | Rigid-group transforms, collisions and member ordering | Cheaper-body substitution, hidden state |
| 47 | `soup` | Buoyancy/contact/moving solids and gravity/tilt | Stale geometry, quiet-frame bias |
| 48 | `soup-stirrer` | Full force cycle, rail free/restore and pointer/no-op controls | Joint lifecycle, skipped force policy |
| 49 | `impulse` | Force/impulse distinction, valid/ignored pointer actions | Normalization/control semantic changes |
| 50 | `theo-jansen` | Landing/particle load, repeated turns and forward/reverse; joint-rich canary | Lost joints, travelling scene clipping |
| 51 | `liquid-tumbler` | 3,800 particles, authored 9.8 gravity/61 iterations, rest and tilt; phone geometry | Reduced iterations/DPR, altered gravity |
| 52 | `drawing-particles` | Empty plus genuinely dense water/elastic/material overwrite, insert/replace/cancel; all-mode timings | Empty benchmark, stamp/count confusion |
| 53 | `sparky` | Original-discovered burst creation/fade/death and slot reuse | Missing burst, stale RGB/lifetimes |
| 54 | `hydraulic-fountain` | All 10-second scheduled phases, later cycle and gap variants/live changes | Partial cycle, altered narrow-gap physics |
| 55 | `wave-tank` | Full platform cycles, widths/slants and stop/restart/amplitude | Different phase/load, stale moved geometry |
| 56 | `liquid-bubbler` | Full approximately 13⅓-second dwell/rise/spill/descent and later cycle | Universal 10-second window, lost post-hook |
| 57 | `stacked-drip` | Full approximately 14.278-second cycle, passive tray response, 12 iterations | Missed descent, reduced substeps |
| 58 | `washing-machine` | Repeated drum turns, speed/stop/restart, cloth/water and moving walls | Stale transforms, assumed population |
| 59 | `tesla-valve` | Forward/reverse, startup/turnover/rate/control transitions, actual worker and legacy case separately | Worker lifecycle, historical baseline reuse |

### Phase 60: Whole-catalog closure and readable results

**Rationale:** Later shared changes may regress earlier scenes. Only a full campaign on one final source establishes final coverage and cumulative effects.

**Delivers:** Complete final-source 25-scene/case corpus; original→final comparisons; per-scene immediate before→after history; linked profiles/candidates/rejections; semantic/visual/lifecycle and cross-scene regression closure; memory tradeoffs and unavailable metrics; independent acknowledgment bound to exact diff/evidence digest. Show wins, neutral/overlapping results and residual slow phases individually. Never let suite averages hide missing investigations or regressions.

**Addresses:** PERF-01/PERF-04/PERF-07/PERF-09..11 and final joins across every scene's atomic requirements.

**Avoids:** P03/P11/P16..20. Missing original/full-final coverage or unresolved fidelity failures block the corresponding closure claim. A fully investigated unchanged scene can close honestly with no speedup claim; retaining speculative code merely to call it optimized is unacceptable.

### Atomic requirements and coverage recommendation

Keep common harness/closure requirements separate from scene evidence. For each of the 25 IDs, allocate distinct measurable rows, such as `SCN-01-BEFORE`, `SCN-01-SIM`, `SCN-01-RENDER`, `SCN-01-CHANGE`, `SCN-01-AFTER`, `SCN-01-QUALITY`; the numbering-to-scene map is explicit. These names are suggested new requirements, not existing fulfilled IDs:

- **BEFORE:** persist a fresh current-accepted-source unprofiled baseline for that scene's frozen required cases.
- **SIM:** identify simulation/session/hook hot functions from that exact recipe, with actual WASM attribution and clearly separate native diagnostic evidence/limits.
- **RENDER:** identify capture/copy/transport/presentation hot paths, CPU submission and available GPU attribution, including actual live backend and fresh-snapshot/progress metrics.
- **CHANGE:** record the profile-supported focused change and its memory rationale, or investigated/rejected/no-gain alternatives and retained limitations.
- **AFTER:** persist matched fresh ordinary after samples and immediate/cumulative readable comparisons with explicit outcome and metric availability.
- **QUALITY:** demonstrate scene semantics, all-six-mode appearance, applicable control/error/lifecycle regressions, affected shared canaries and independent exact-digest acknowledgment.

Original coverage, full-final coverage, identity/comparability, workload replay, memory availability and immutable writing also need their own common requirements. Do not collapse the 25 cycles into one generic “optimize every scene” row. Share the protocol prose while keeping each scene's requirements and evidence references independently traceable. A machine-checkable coverage join must reject missing/duplicate/unknown IDs, missing cases/lanes and incomplete individual phases.

The readable result table should have one row per scene/case/boundary, with original, immediate before, immediate after and final values; absolute units/deltas and any justified ratios; replicate variation; producer/workload/backend identities; memory scope/status; fidelity result; outcome/limits; links to raw attempts and profiles. Summaries can group cases for scanning but cannot replace this detail.

### Phase Ordering Rationale

- Workload discovery and instrumentation probes precede source freeze; full original coverage precedes optimization.
- One accepted scene phase precedes the next fresh before; family reuse reduces harness duplication without erasing individual work.
- Shared solver/capture/renderer/worker changes immediately run affected material, joint, color, count-churn, scale and ownership canaries against the current predecessor.
- Each scene phase is one acceptance unit but may require multiple bounded plans. Separate baseline/profile, focused implementation/quality and after/review plans when execution context would become too large; dense/mixed/worker scenes can use more plans within the same assigned scene phase. Avoid adding another framework or pretending all 25 can fit one execution budget. Exact runtime/plan counts await Phase 34 measurements.
- Final all-scene replay detects shared effects after earlier acceptance. Independent review is allowed by current policy; it cannot be self-approval.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 34:** actual Rust→WASM sample symbols, optimized diagnostic build, worker CDP attachment, app input/observation barrier, GPU-query validity, memory scopes, float/script encoding and original witnessed windows.
- **Phase 43 / 52 / 53 / 56 / 57:** original phase discovery for drainage, dense painting, contact bursts and complete long cycles; verify exact workload checkpoints rather than guessing durations.
- **Any scene with shared solver/ownership/cache redesign:** targeted profile-led investigation of actual functions and rollback/invalidation contracts. Profile before choosing a data structure or generalized worker route.
- **Phase 59:** actual worker attribution, queue/application/presentation latency and preservation of hidden/pause/disposal semantics if those paths change.

Phases with established patterns, normally skip separate ecosystem research:

- **Phases 35–59 with unchanged harness and existing ownership:** the common fresh-before/profile/focused-change/after protocol, private scene factory and ordinary checks are established. Every phase still needs its own profiling; skipping a research document never skips diagnosis.
- **Phase 60:** immutable coverage joins, matched comparators and derived result tables are standard once Phase 34 contracts work. Reopen research only for an unresolved measured capability or changed contract.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH for inventory / MEDIUM for attribution | Existing versions/source and official API boundaries are grounded; actual symbols, worker attachment and GPU/memory support have not been probed. |
| Features | HIGH for scope / MEDIUM for workload recipes | User policy and 25 scene sources are explicit; proposed active windows, population envelopes and scripts need original-source validation. |
| Architecture | HIGH for current seams / MEDIUM for new harness | Production scene/session/render/ownership boundaries are inspected; common adapters, observation seam and schema are proposals. |
| Pitfalls | HIGH for prior incidents / MEDIUM for prevention coverage | Retained Tesla rejection records substantiate cache/lifecycle/evidence risks; new all-scene checks remain unimplemented. |

**Overall confidence:** MEDIUM for execution readiness; HIGH for the recommended scope and evidence design. No new timing, profile, speedup or capability experiment was performed by these research tasks.

### Gaps to Address

- **Actual browser attribution:** prove function/index mapping on exact release WASM and actual worker target. Retain unattributed share; separately hashed diagnostic counters/builds may supplement a failed symbol probe.
- **Original witnessed trajectories:** establish actual counts/events/cycle boundaries, default preferences/gravity and sufficient repeat windows. Freeze discovery on original source, then rerun for authority.
- **Live interaction timing:** preserve real RAF/backend/chrome while applying ordered physical-step inputs; validate observation overhead. Timer lateness is a proxy, not actual input latency.
- **Semantic completeness:** frame hashes omit some velocities/groups/joints/forces/scene state; add targeted private assertions for changed behavior rather than claiming exhaustive equivalence.
- **GPU and memory availability:** record valid asynchronous GPU intervals where supported; otherwise unavailable with reason. Distinguish buffers, WASM capacity, heap, RSS and logical resources; do not sum overlapping scopes into invented totals.
- **Measured acceptance and execution budget:** establish meaningful per-case costs and variability after baseline. No universal percentage, absolute frame-rate goal, memory ceiling or guaranteed per-scene improvement is inferred.
- **Baseline defects:** preserve and classify pre-existing failures. Fixing them is a separately identified behavior change, not a fabricated performance win or silent expectation replacement.

## Sources

### Primary (HIGH confidence)

- [STACK.md](STACK.md), [FEATURES.md](FEATURES.md), [ARCHITECTURE.md](ARCHITECTURE.md), [PITFALLS.md](PITFALLS.md) — complete source-grounded milestone research and detailed workload/guardrail contracts.
- [Current project](../../PROJECT.md), [PROJECT-SCOPE.md](../../../PROJECT-SCOPE.md), [AGENTS.md](../../../AGENTS.md), [Bright Builds sidecar](../../../AGENTS.bright-builds.md), [overrides](../../../standards-overrides.md) — accepted speed/memory/fidelity, hobby scope and review authority.
- [25-scene catalog](../../../web/src/catalog/scenes.ts), [controls/defaults](../../../web/src/catalog/scene-records.ts), [Rust factory](../../../crates/liquidfun-wasm/src/scene.rs), [SessionCore](../../../crates/liquidfun-wasm/src/session.rs) — exact catalog, settings and real native/WASM execution seam; individual sources are linked in FEATURES.
- [Tesla benchmark history](../../../docs/benchmarks/tesla-valve/README.md), [existing runner](../../../web/scripts/bench-tesla.ts), [backend routing](../../../web/src/player/scene-lifecycle.ts), [worker owner](../../../web/src/physics/worker-session.ts), [render boundary](../../../web/src/render/present-frame.ts) — immutable evidence, actual routing, retained rejections and submission boundaries.
- [Playwright CDPSession](https://playwright.dev/docs/api/class-cdpsession), [CDP Profiler](https://chromedevtools.github.io/devtools-protocol/tot/Profiler/), [CDP Target](https://chromedevtools.github.io/devtools-protocol/tot/Target/) — existing sampling/worker attachment APIs; precise coverage changes optimized execution.
- [V8 WASM pipeline](https://v8.dev/docs/wasm-compilation-pipeline), [Chrome Performance reference](https://developer.chrome.com/docs/devtools/performance/reference/) — debugger/profile state and diagnostic capture boundaries; overhead needs checking, not an assumed magnitude.
- [WebGL2 timer-query specification](https://registry.khronos.org/webgl/extensions/EXT_disjoint_timer_query_webgl2/), [High Resolution Time](https://www.w3.org/TR/hr-time-3/), [HTML transferable objects](https://html.spec.whatwg.org/multipage/structured-data.html#transferable-objects) — GPU validity, realm clocks and moved buffer ownership.
- [samply](https://github.com/mstange/samply), [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles/), [WASM JS memory interface](https://webassembly.github.io/spec/js-api/#dom-memory-buffer), [CDP JS protocol](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/js_protocol.pdl) — diagnostic native profiles and memory measurement scopes.

### Secondary (MEDIUM confidence)

- [Older wasm-pack build guide](https://rustwasm.github.io/docs/wasm-pack/commands/build.html) — supporting profiling semantics; installed 0.15.0 help cited in STACK is version-specific authority.
- Suggested adapter paths, scene-phase ordering, active windows and canary grouping — integrated design recommendations inferred from the primary repository evidence; validate in Phase 34 and individual plans.

### Tertiary (LOW confidence)

- None adopted. Unmeasured GPU support, symbol quality, speedup size and memory availability remain gaps rather than asserted facts.

***

*Research completed: 2026-10-03*
*Ready for requirements and roadmap: yes; Phase 34 capability/workload probes remain explicit implementation work.*
