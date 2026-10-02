---
phase: quick-261002-irz
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-261002-irz
generated_at: "2026-10-02T13:32:58-05:00"
---

# Tesla performance research

Discovery level3: core contact/query/storage optimizations and interactive worker ownership cross architectural boundaries. This research uses actual local modules, separate read-only core/frontend design agents and primary browser documentation. No measured optimization gain exists yet; implement benchmark foundation and commit the original baseline first.

## Existing boundary

web/src/physics/loader.ts initializes the actual generated WASM package and constructs ProofSession. GeneratedProofSession already exposes advance(stepCount), captureFrame(), applyControl, applyAction, pointerAction, gravity operations and free. createSceneSession owns exactly-once cleanup and parses/frees raw frames; its nextFrame synchronously advances then captures. No new Rust/WASM API is required for async live ownership.

web/src/player/frame-loop.ts currently drives synchronous nextFrame calls, tracks fixed-step remainder and paints on the main thread. SceneRuntime and lifecycle/surface/input adapters own construction, reset generation, controls, held frames and provenance. Direct SceneSession must remain available for deterministic exports and non-Tesla scenes.

presentSceneFrame delegates to WebGL particle rendering and drawRenderFrame. Per-surface caches belong here rather than in global scene geometry. Camera/viewport/DPR/mode/stroke and wall changes invalidate static renderer work; caching cannot freeze dynamic walls or reduce particles.

## Serial core stages

1. Spatial wall contacts: particle/body_contact.rs scans particle rows inside each collider's expanded AABB. world/particle_coupling.rs updates body contacts immediately after particle contacts have prepared sorted proxies. Collect candidate rows from current proxies before invoking any filter callback, sort ascending row, then preserve the old strict point/AABB predicate and fixture→child→row traversal. Missing, stale or unrepresentable proxy queries retain the legacy full scan. Compare exact contacts, impulses/effects and filter invocation traces.
1. Static geometry and scratch: checked polygon/chain shapes have immutable private geometry; use standard-library Arc-backed shape data so hot collider records clone shared geometry cheaply while preserving public accessors and content equality. Geometry replacement naturally supplies new checked backing data; transforms/filter records must still refresh appropriately. Reuse the boundary-step scratch lanes, collider records and hits transaction-safely; candidate validation must not partially swap authoritative lanes and scratch returns on all paths. Preserve legacy whole-step failure semantics: pre-call rollback for LimitExceeded/InvalidParticleGroupTopology, with earlier committed passes retained for other errors where currently allowed. Do not invent universal full-step atomicity. Both caching and scratch reuse are required in this one stage; do not replace either with an unrelated copy optimization.
1. Drain query: particle/query.rs AABB query currently builds a ParticleNeighborhood that also enumerates pair rows in particle/proxy.rs. Introduce a private query-only constructor/index path that prepares sorted proxy lookup without pair enumeration. Public ParticleNeighborhood keeps pair behavior; preserve query IDs/tags/order, validation, callback behavior and checked destruction. This is an engine query optimization exercised by the Tesla drain, not an altered drain rectangle or policy.

## Benchmark architecture

Use Bun plus existing Playwright Chromium; no new dependency. An isolated Vite-built benchmark page imports the actual production loader/session/frame/camera/present-frame modules. A repository-owned runner constructs fresh scenes and records separate advance, capture+parse and render CPU samples. Compare fixed simulation-step workloads rather than variable particle counts or inferred FPS. Measure actual interactive paint cadence through the production scheduleFrame with real RAF/presentSceneFrame, simulation steps per wall second, event/main-thread responsiveness and worker transport separately. Label timer-task lateness as a responsiveness proxy and render submission CPU time as CPU work, not GPU completion.

Locked primary workload: three sequential fresh-scene replicates for forward/reverse at1440/s; forward warm360 steps and reverse warm384 steps, then40 one-step samples at the unchanged fixed dt/solver settings. This targets the reported approximately3k/9k particle slowdown. An explicit --include-max-rate option may add2880/s timing cases with separate immutable case identity; do not mix extended results into primary attribution. All four rate/direction QUALITY containment/behavior cases remain required. Record checkpoints before/after warmup/sampling, counts and semantic fingerprints; repeat identical configured checkpoints at each stage. Warmup and sample work must not mutate benchmarks' physical policy.

Persist exclusive-create JSON under docs/benchmarks/tesla-valve/runs/<fresh-id>/ and a dated source-bound historical summary. Include complete source revision/diff identity, benchmark source version, built WASM digest, actual producer/runtime/host/GPU/backend, viewport/DPR/render mode/particle limit, solver/settings, warmup/sample parameters and per-replicate raw values. Snapshot input source identity before the run, explicitly separating engine/benchmark source manifests from newly generated report outputs so reports do not recursively change their own producer identity. If measured before a commit, retain actual dirty input identity; the historical index may link the later containing commit, but immutable reports cannot claim they were produced from that later commit. Uncertainty summarizes between-replicate variation separately from correlated within-trajectory samples; confidence claims must match sample design. Unknown facts are explicit unavailable fields, not invented identities. Original reports are never overwritten; retries get fresh IDs and failed records remain.

Baseline comes after harness changes only and before stages1–5. Each stage preserves a before report from the previous accepted implementation and an after report from current freshly built artifacts. Validate comparability and semantic equality before interpreting timings. Stages1–3 keep bridge/capture/render unchanged. Stage4 may capture once per bounded worker batch, explicitly reported as worker pipeline adaptation, while the direct fixed-step physics workload stays identical. Stage5 changes renderer work only.

Comparability does not require equal source commits or WASM digests across different optimized implementations. Each report must bind its actual implementation/artifact identity; benchmark configuration/runtime/backend/settings/checkpoints must match for the specific compared metric. Worker live-pipeline backend intentionally changes at stage4 and must be explicitly attributed; do not disguise that comparison as identical-backend pure physics timing.

## Worker contracts

Introduce async-compatible LiveSceneSession without changing existing direct SceneSession. Main thread owns a worker proxy; worker owns one generated proof session. Bounded messages carry session generation, monotonic request ID and typed operation. Serialize all operations FIFO with a single in-flight advance and owned transferable frame lanes. Match replies before acceptance and reject malformed counts/generations. Disposal rejects pending operations and suppresses stale reset/route replies; runtime errors surface visibly and release ownership once.

Transferred ArrayBuffers change ownership and detach at the sender, so frame lanes must own their copied buffers rather than transfer live WASM memory or arrays still needed locally. This is supported by [MDN Worker.postMessage](https://developer.mozilla.org/en-US/docs/Web/API/Worker/postMessage) and [MDN transferable objects](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects). No SharedArrayBuffer or cross-origin-isolation dependency is necessary.

Tesla playback chooses worker ownership. Other scenes and deterministic capture/export use direct ownership. Preserve fixed dt/iterations, bounded clock scheduling, ordered controls and pause/hidden semantics. Do not run uncontrolled worker wall-clock stepping or interpolate particle-array indices: the frame currently has no stable particle IDs. Test worker/direct same-fixed-step semantic state and distinguish worker physics timing from round-trip latency.

## Renderer contracts

Cache static wall drawing per actual surface/session generation, with wall-lane identity/content revision plus camera/viewport/DPR/mode/stroke in the key. Verify direction flip, reset, dynamic-wall updates, resize/zoom and mode changes invalidate correctly. Batch compatible drawing work without changing compositing order or color/projection. Cache uniform/static GPU metadata only under explicit valid revision keys; continue submitting current particle positions and full requested count. Preserve Canvas/WebGL fallback, exports, density shading and shared scale reporting.

## Verification risks

- Spatial queries can reorder side-effectful filters: select first, sort rows, then perform the old predicate/callback sequence.
- Private shared geometry can accidentally retain stale transforms/filter records: cache immutable data only, verify refreshed collider identity.
- Scratch can leak prior step/query data or break error rollback: clear logical lengths and test alternating counts plus failure/retry.
- Query-only optimization can drop pairs from public callers: preserve full public constructor behavior and compare callbacks exactly.
- Worker completion after pause/reset can repaint wrong generations: guard dispatch and completion ownership and cancel future scheduling.
- Static render caching can hide flipped/dynamic walls or camera changes: explicit per-surface keys and invalidation tests.
- Faster CPU samples do not establish responsiveness or60fps: report real cadence/steps/latency plus uncertainty honestly.

## Concrete implementation seams

- Immutable shape storage: collision/shape/polygon.rs and collision/shape/chain.rs, with unchanged checked constructors/accessors and standard Arc geometry.
- Reusable boundary ownership: particle/solver/boundary.rs and boundary/support.rs; SystemPassExecutor in world/particle_coupling/executor.rs; begin/commit in executor/boundary_runtime.rs; authoritative validated replacement in particle/storage/runtime.rs. Private scratch ownership belongs to World/world/object.rs, taken/restored around world/particle_coupling.rs.
- Query-only index: private ParticleNeighborhood::from_view_for_query in particle/proxy.rs, consumed only by particle/query.rs query_aabb. Public from_view/pairs remain unchanged.
- LiveSceneSession: backend direct/worker; Promise-returning nextFrame/applyControl/applyAction/pointerAction/setGravity/restoreAuthoredGravity, with terminal synchronous dispose. Step counts remain1..4. Worker contracts/proxy live in physics/live-session.ts, worker-messages.ts, worker-session.ts and simulation-worker.ts.
- Player integration: scene-runtime.ts, frame-loop.ts and scene-lifecycle.ts first, then scene-surface.ts, input/canvas-pointer.ts and input/tilt-binding.ts. Existing direct exporter/session tests stay intact.

Nine bounded execution prompts cover foundation, spatial contacts, shape sharing, scratch integration, query-only drain, worker contracts, lifecycle/scheduling, surface/input and renderer/final integration. Shape/scratch are one measured stage2 after both prompts; worker contracts/lifecycle/input are one measured stage4 after all three. This preserves exactly five individually attributed selected stages without forcing an oversized executor context or creating a milestone.

## Architectural plan check

Separate frontend read-only design check returned PASS for06–09. Preserve explicit pre-acceptance queue rejection, actual completed-step/debt accounting, no pause/hidden completion resuming playback, direct/export synthetic clocks and antialiased join equivalence through compatible batching/exact cached output. Core02–05 architecture check returned PASS with two incorporated clarifications: explicitly create/register the new body_contact/tests.rs and query/tests.rs modules, and preserve legacy whole-step failure semantics rather than inventing full-step atomicity. Original baseline and serial stage software gates still apply before implementation GO.
