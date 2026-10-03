# Technology Stack: v1.4 Scenario Performance

**Project:** liquidfun-rs
**Researched:** 2026-10-03
**Mode:** ecosystem / instrumentation
**Confidence:** HIGH for existing tool inventory and measurement boundaries; MEDIUM for browser function attribution until a capability probe runs on the pinned Chromium.

## Recommendation

Extend the existing Bun + Playwright + production-WASM benchmark infrastructure into a scene-indexed suite. Keep native Rust/samply as supporting diagnosis and use actual browser profiles to identify browser simulation and rendering hot paths. Add small optional probes for GPU queries, bytes, memory, and worker boundaries; no new dependency is currently justified. Preserve every scene's authored behavior, solver settings, particle population, viewport, DPR, render mode, density shading, and resolution. More retained memory is allowed, but report its cost.

Create a new versioned report contract and immutable v1.4 output root. Preserve Tesla v1 and historical Dam Break evidence. Do not turn this into a C++ parity campaign, strict Linux qualification, or a controlled-host requirement. `PROJECT-SCOPE.md` explicitly permits useful local comparisons; the results must identify their local limits.

**Guidance applied:** `AGENTS.md` Repo-Local Guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, `PROJECT-SCOPE.md`, and the architecture, verification, testing, Rust, and TypeScript standards. Both active lesson files were read in full: 14,263 combined bytes / 4,755 conservative estimated tokens. No active lesson block was omitted. This research changes only this GSD file, which must not be passed through mdformat.

## Existing stack and identity

Versions below were read locally, rather than inferred from latest releases. These are retained tools, not upgrade recommendations.

| Tool | Existing version / setting | Role |
| --- | --- | --- |
| Rust | 1.97.0; rustc `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`; LLVM 22.1.6 | Native engine and WASM builds; Edition 2024, resolver 3, declared MSRV 1.92 |
| Cargo profiling profile | Inherits release; `debug = true`, `strip = false` | Native optimized symbol-bearing diagnostics |
| wasm-pack | 0.15.0 | Existing build script produces `--target web --release` WASM |
| wasm-bindgen | Manifest pin `=0.2.128` | Private bridge and generated JS-owned frame lanes |
| Bun | Installed/package-manager pin 1.4.2 | Orchestration, hashes, exclusive writes, summaries |
| Playwright | Installed/manifest 1.63.0 | Real Chromium, page scripts, CDP access |
| Chromium | Installed Playwright manifest: revision 1243, expected version 153.0.8010.12 | Actual executable/version/hash must still be captured for each run; manifest alone is not execution proof |
| Vite / TypeScript / Vitest | Manifest 8.3.0 / 7.0.2 / 5.0.1 | Dedicated benchmark bundle and existing contract tests |
| samply | Installed and xtask pin 0.13.1 | Native CPU stacks |
| dhat | Existing optional 0.3.3 feature in unpublished WASM bridge | Native allocator investigation after allocation evidence |
| Renderers | Existing Canvas2D plus WebGL2 shaded blob and canvas fallback | Preserve actual backend identity separately per case |

Read-only local host identity: Apple M4 Max; 16 logical CPUs; 51,539,607,552 memory bytes; macOS 26.6.2 build 25G83; `aarch64-apple-darwin`. Repository source inspected at `5f192e3d9f1591053058b983293abbb7969470d0`. These facts are research-host observations, not benchmark measurements. GPU model/driver, actual launched Chromium executable, current power/thermal/background state, timer-query availability, and browser symbol quality were not exercised here.

## What the repository already measures

| Existing surface | Proven source behavior | Reuse / limitation |
| --- | --- | --- |
| `web/scripts/bench-tesla.ts` and supporting modules | Rebuild production WASM; dedicated Vite output; source path/content manifest including untracked inputs; revision/diff/WASM/bindings/bundle/Chromium executable hashes; exclusive evidence writes; prior-report hash linkage; producer checked again after run | Reuse evidence semantics; generalize workload/case identity and coverage to all 25 `SCENE_IDS` |
| `web/benchmarks/tesla-valve/benchmark.ts` | Direct fixed-step timings split `advanceMs`, `captureMs`, `copyValidateMs`, `renderSubmitMs`; warmup and shader/resource initialization outside measured trajectory | Existing renderer measurement is explicitly CPU submission, not GPU completion |
| Tesla canonical workload | 40 one-step trajectory samples, three sequential replicates, forward/reverse warmup 360/384; 2 s live run; 1280×960/DPR 1; shaded blob; flow 1440; particle iterations 4, rigid 8/3; 1/60 s timestep | Retain Tesla definition for historical comparison; other scenes need their own frozen controls, seed/actions, warmup and checkpoints |
| `model.ts`, `compare.ts` | Mean/median/p95/min/max; replicate means and ranges; workload/environment compatibility checks; correlated frame samples explicitly descriptive | Retain raw samples and replicate units; 40 adjacent steps are not 40 independent experiments |
| `fingerprint.ts` | Hashes named semantic frame fields and lanes, including positions/colors/radii/rigid geometry, outside timed spans | Strong baseline mechanism; does not by itself cover hidden simulation state or visual pixels |
| `live.ts` | Production RAF clock/session/painter; actual direct or worker owner; submissions/s and simulation steps/s; frame counts/step indices; main-thread 16 ms timer lateness | Product chrome excluded. “Presented FPS” means paint submissions per elapsed second, not physically displayed frames or completed GPU frames |
| Tesla worker | Dedicated actual WASM worker, single-flight FIFO, bounded queue, transferable owned frame arrays; advance/capture/parse durations; request round trip on main thread | Worker initialization is hard-coded to Tesla. Do not pretend all scenes currently use workers |
| Native Dam Break binary | Medium/Normal recipe, initial 1,920 particles; default 60 warmup/600 measured steps; wall timing around repeated `SessionCore::advance(1)` | Source comment calls this `World::step`; actual timed boundary includes the scene/session advance wrapper. It excludes capture/JS/render/browser |
| Dam Break samply / timers / heap | Optimized-symbol native sampling; profiled parent timers; gated native dhat dumps; explicit `not_timing_authority` | Sampling currently includes startup/warmup; crop measured region or retain honest whole-process scope |
| `playground dam-break-bench` pair | Native Rust and pinned C++ comparison | This command builds C++. Reuse the Rust binary for v1.4 diagnostics without making the paired command mandatory |

Sources for inventory: `web/package.json`, installed Playwright `browsers.json` and `types/protocol.d.ts`, root `Cargo.toml`, `rust-toolchain.toml`, `scripts/web-build.ts`, Tesla benchmark modules, `web/src/physics/{frame,simulation-worker,worker-session,worker-messages}.ts`, `web/src/render/{present-frame,webgl-particles}.ts`, and `tools/xtask/src/playground/{pair,profile,timers,heap}.rs`. **Confidence: HIGH** from local primary source inspection.

## Measurement contract to add

Use three separately identified run kinds: ordinary production timing, diagnostic CPU/trace capture, and diagnostic memory/GPU capture. Retain an immutable unmodified-source baseline before optimization. For each scene, record cold creation separately from warmed fixed-step trajectory and warmed live playback. A scene-level result must remain visible even when it is fast, unchanged, regresses, or cannot be attributed; do not summarize only the improved cases.

| Boundary | Record | Meaning and limit |
| --- | --- | --- |
| Simulation | JS elapsed time around actual WASM `advance`, requested/actual steps, timestep/iterations, controls and semantic checkpoints | End-to-end simulation entry cost; CPU profiler breaks it into functions |
| Frame capture | WASM capture entry duration, exact returned counts/lanes | Rust frame construction and bridge entry boundary |
| Getter/copy/validation/free | Separate elapsed span after capture | Boxed-slice getters copy into JS-owned arrays; parser retains these arrays without an additional blanket copy. Preserve cleanup cost and ownership |
| Driver | RAF intervals, admitted/run steps, debt, capped/deferred steps, timer lateness, frame age | Distinguishes fast submissions from simulation falling behind |
| Worker | enqueue→send queue delay; main send→validated receive round trip; worker advance/capture/parse; receive validation and receive→draw | Current round trip starts at send, omitting client FIFO waiting. Residual after worker compute includes scheduling/transfer/main validation; label it residual, not pure transport |
| Geometry/projection | CPU spans for projection, metadata comparison, static layer work, Canvas2D paths, metaball/contour/density/wire paths | Renderer functions and allocations can dominate even when physics improves |
| WebGL upload | Exact buffer payload bytes and changed lane flags; CPU time of projection and `bufferData` calls separately | Submission/driver-call latency includes possible stalls; byte counts do not prove GPU bandwidth |
| Draw submission | Particle field pass, composite pass, Canvas2D walls/labels/static presentation | CPU time ends at return of API calls; underlying work may continue |
| GPU | Optional asynchronous elapsed queries for field/composite passes; extension status, query validity, disjoint count and rejected samples | Actual GPU interval only for bracketed WebGL commands, not whole-frame display/composition latency |
| Full app | Separate production route capture with visible chrome and unchanged viewport | Existing isolated painter run cannot substantiate full product responsiveness |

Worker and window clocks have different time origins. Use durations computed within each realm by default; for diagnostic cross-realm traces normalize timestamps using `performance.timeOrigin + performance.now()`, record origins, and bind events by generation/request/frame IDs. Timer precision can be limited, so keep raw precision and avoid interpreting tiny subtraction residuals as exact one-way wire latency. [High Resolution Time, 2026 working draft](https://www.w3.org/TR/hr-time-3/). **Confidence: HIGH for documented clock semantics; MEDIUM for usefulness at this host's timer resolution.**

## CPU attribution in the actual browser

### First choice: existing Playwright with CDP

Playwright exposes raw protocol sends/events through `CDPSession`; no profiler package is needed. [Playwright CDPSession](https://playwright.dev/docs/api/class-cdpsession). Start diagnostic profiling after scenario warmup, before measured work, and stop at a recorded step boundary. Capture the main-page target and the actual simulation-worker target separately. CDP `Target.setAutoAttach` provides related-target attachment; attaching only the page is inadequate evidence for worker computation. [CDP Target](https://chromedevtools.github.io/devtools-protocol/tot/Target/). **Confidence: HIGH for APIs; MEDIUM for worker-session routing until a local probe verifies it.**

Use `Profiler.enable`, a recorded `setSamplingInterval` before `start`, then `stop`; retain raw nodes, samples and time deltas. Export per-function self samples and inclusive stacks with target/thread, script URL, function index/name, measured step window and unattributed share. Sampling attributes observed execution; it does not provide exact invocation counts or exhaustive timing for small functions. Do not turn on precise coverage to obtain counts during performance runs: the protocol states that it prevents optimized execution. [CDP Profiler](https://chromedevtools.github.io/devtools-protocol/tot/Profiler/). **Confidence: HIGH for profiler contract.**

Use a browser tracing capture when stacks alone cannot explain worker delivery, GC, upload stalls, paint/compositor activity or compilation. Record exact trace categories and unavailable tracks; the Performance panel supports call stacks and rendering activity. Detailed paint instrumentation can affect measurements; use a separate diagnostic capture and quantify its overhead. [Chrome Performance reference](https://developer.chrome.com/docs/devtools/performance/reference/). **Confidence: HIGH for available diagnostic surfaces; MEDIUM for attributing platform driver work.**

### Symbols and optimization caveat

Probe the actual release WASM first. Preserve any function names/index mapping and tie it to the exact WASM hash. If internal names are unavailable, produce a separately hashed optimized symbol-bearing package in a separate output directory; keep ordinary release artifacts untouched. Installed wasm-pack 0.15.0 `build --help` confirms `--profiling` enables optimizations and debug information, `--dev` disables optimizations, and `--no-opt` skips wasm-opt. A diagnostic build's full flags and post-processing must be recorded; do not skip a production optimization silently. [Older maintainer build documentation](https://rustwasm.github.io/docs/wasm-pack/commands/build.html) agrees on profiling semantics but flags its moved location, so local help is the version-specific authority here. **Confidence: HIGH for installed flags; MEDIUM for retained profiler symbols.**

Do not promise full Rust source/inline attribution just because DWARF exists. Chrome's published WASM debugging guide covers C/C++ support; symbol retention through Rust, wasm-bindgen and optional wasm-opt, as well as profiler stack display for this Rust package, needs a local probe. [Chrome WebAssembly guide](https://developer.chrome.com/docs/devtools/wasm/). **Confidence: MEDIUM; unresolved capability is explicit.**

V8 documents baseline Liftoff, hot-function optimization, debug tier-down and Performance-recording tier-up. Therefore browser debugger state and profiler state belong in evidence identity. Ordinary timing must run with no open DevTools/debugger/profiler; diagnostic runs cannot silently replace timing authority. Do not force experimental tiering flags to improve reported performance. [V8 compilation pipeline](https://v8.dev/docs/wasm-compilation-pipeline). **Confidence: HIGH for documented behavior; MEDIUM for exact pinned-browser protocol interactions.**

If symbolized browser sampling fails, retain WASM indices/unattributed stacks and add narrowly scoped diagnostic counters/timers around suspected Rust kernels in a private instrumentation build. Measure overhead against the ordinary release run and confirm findings in browser boundary metrics. Existing native `advance_profiled` is compiled out for wasm32, and its parent recorder uses `std::time::Instant`; it is not a ready-made browser timer. Do not export it unchanged and assume it works. New WASM timer adapters would be a scoped gap, not a new general tracing dependency.

## Native Rust attribution remains a proxy

Keep samply 0.13.1 and the existing Cargo profiling profile. Samply recommends release optimization with debug information for inline stacks/source views; its documented default is 1 ms sampling and macOS can include blocked/off-CPU samples. Distinguish CPU-running samples from waiting. [samply upstream README](https://github.com/mstange/samply). Cargo supports inherited custom profiles and separate debug/strip settings. [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html). **Confidence: HIGH.**

Generalize the native scene driver only where needed to investigate Rust kernels. Record its exact scene/controls/actions, target triple, binary hash, debug identity, profile settings, Rust flags, profiler version/argv/interval, and measured step range. A native build of `liquidfun-wasm` is still native machine code, not WASM execution. It excludes V8 compilation, JS/WASM transitions, boxed-slice getter copies, transferable ownership, renderer, GPU and browser scheduling. Native and browser code generation and allocator behavior may differ. A native hot function is an investigation lead; only actual browser evidence can establish that it is the browser bottleneck or that its improvement carries across.

Keep native profile, parent timer and dhat outputs marked diagnostic. For v1.4, measure performance gain with new ordinary browser before/after runs, not a historical Rust/C++ ratio. Existing native diagnostics' startup/warmup scope must be disclosed when estimating hot-path shares.

## GPU, driver and rendering boundaries

For WebGL2, capability-test `EXT_disjoint_timer_query_webgl2`. Bracket the existing field/composite commands with non-overlapping elapsed queries; collect only after yielding back to the browser and `QUERY_RESULT_AVAILABLE` is true. Reject disjoint results, handle context loss, retain timeout/invalid counts, and bound pending queries. The extension defines nanosecond query results and a disjoint validity signal. [Khronos extension, revision 4 / 2023-06-01](https://registry.khronos.org/webgl/extensions/EXT_disjoint_timer_query_webgl2/). **Confidence: HIGH for API, unverified availability on this host.**

When unavailable, report `{ status: "unavailable", reason: "extension unavailable" }` or another actual reason; never zero milliseconds. Canvas2D fallback similarly has no result from this WebGL extension. CPU rendering submission remains measurable and useful. Avoid `gl.finish`, synchronous readback or polling loops in authoritative runs because these would change the pipeline under measurement.

Retain exact WebGL renderer/vendor/version/unmasked values and software classification. Add available browser GPU/driver information and feature status from a browser-level `SystemInfo.getInfo` probe, with explicit absence when fields are not available. The official protocol declares GPU devices and auxiliary driver/feature data; available fields must be probed rather than assumed. [CDP SystemInfo source](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/domains/SystemInfo.pdl). Driver CPU submission, GPU execution, browser compositor scheduling and physical display completion are distinct. Neither a Rust profile nor elapsed WebGL query proves full displayed-frame latency.

## Memory reporting with permitted increases

Record bytes with an explicit measurement kind and scope; do not add unlike measures together as a total.

| Measure | Proposed source | What it actually proves |
| --- | --- | --- |
| JS-owned frame lanes | Sum distinct lane buffers' `byteLength`, plus counts | Exact payload storage for typed arrays; excludes object overhead and strings. Worker transfer moves owned buffers, so avoid counting sender and receiver as simultaneous copies without lifetime evidence |
| Renderer scratch buffers | Actual array capacity/`byteLength`, static cache counts | Exact retained application buffer bytes; sample warmup/end/high-water rather than just current particle count |
| Upload payload | Actual submitted subarray lengths × element byte widths | Exact bytes sent through application buffer API; unchanged radii/colors correctly contribute zero new payload |
| WASM linear memory | Actual exported `WebAssembly.Memory.buffer.byteLength` per instance, if bridge access is available | Allocated linear-memory capacity, not live Rust allocation or RSS. Re-read after growth; label unavailable if not exposed. [WASM JS interface](https://webassembly.github.io/spec/js-api/#dom-memory-buffer) |
| JS heap/backing storage | Per-target `Runtime.getHeapUsage` capability probe at untimed checkpoints | Isolate-scoped used/allocated heap; backing-storage fields when present. The installed Playwright protocol declares these separately; avoid double-counting manually counted ArrayBuffers. [CDP JS protocol source](https://github.com/ChromeDevTools/devtools-protocol/blob/master/pdl/js_protocol.pdl) |
| JS allocation attribution | Optional separate `HeapProfiler.startSampling` run | Sampled JS allocations, not Rust allocations in linear memory or whole-process residency; record interval/lifetime inclusion settings |
| Native Rust heap | Existing gated dhat diagnostic | Native allocator lifetimes/high-water and allocation callsites; instrumented and not production timing authority |
| Process RSS / peak RSS | Available OS process statistics with PID/process role/time/units | Native process residency or browser/renderer/worker-host/GPU-process residency; distinguish per-process values from tree estimates and shared memory |
| GPU resource budget | Field texture format/dimensions plus explicit buffer capacities | Logical budget only: RGBA16F field is 8×width×height bytes, excluding driver padding, backbuffers and compositor surfaces. Do not call it measured VRAM |

Prefer untimed start/end checkpoints and a separate memory sampling run to per-frame heap API calls. Keep disposal/restart lifecycle checks to detect retained caches or worker-instance leaks. Memory increases should show baseline/candidate absolute bytes and deltas and the gained CPU/GPU/latency result. Preallocation, scratch reuse and metadata/static caches are candidate optimizations, not demonstrated gains until profiled and compared.

## Evidence and statistics

Retain ordinary baseline and candidate raw per-step/per-frame values, semantic checkpoints, report hashes and exact producer hashes. Each scene/control case needs source/build/browser/hardware/render identity and a before→after link; never overwrite old files or relabel v1.3 runs as v1.4 baseline. Run failed attempts in separate directories and retain partial records. Producer hashes may differ between baseline and candidate; workload and comparable environment must stay fixed.

Use multiple fresh replicates with stable reset/warmup; report per-replicate means/medians/ranges and within-run p95 frame/step cost. Sequential thermal/load drift remains a limit. Alternate before/after execution order when practical, record background/power conditions, and do not present adjacent trajectory samples as independent statistical confidence. Warmup must be long enough to assess stability while preserving the same physical checkpoint recipe; changing warmup mid-comparison changes the workload. Retain cold-start/initialization separately so shader compilation, WASM compilation or larger cache setup are not hidden.

A useful scene result pairs simulation rate and submission cadence: RAF-limited FPS can remain unchanged despite faster kernels; worker animation can look smooth while physics falls behind. Report frame age/debt and step indices, not just FPS. Semantic frame hashes and same-step trajectories protect behavior; visual checkpoint comparisons protect render fidelity and camera/scale legend/chrome. Hash changes require investigation, never automatic replacement by candidate output.

## Phase research flags and validation

1. **Measurement foundation:** Generalize existing benchmark modules; prove exact coverage of 25 catalog IDs, frozen per-scene recipes, actual backend reporting, immutable before/after linkage and unavailable metric states. Keep comparison/statistics logic pure and contract-tested.
1. **Attribution capability spike:** Verify release WASM samples/names, separately hashed profiling symbols, actual worker CPU target capture, current browser trace tracks, GPU-query availability/validity, heap scopes and RSS access. A parser finding an empty profile is not success.
1. **Optimization loops:** Profile simulation and render paths for each scene; choose justified shared or scene-specific improvements; retain candidates with memory costs. Re-run ordinary browser measurements after each accepted optimization and check semantic/visual fidelity.
1. **Final proof:** New immutable all-scene after corpus and comparison, identity/coverage validator, regression summary and residual unavailable metrics. Preserve unchanged/slow scenes in the result table. Ordinary local checks and independent review apply; C++ and Linux strict qualification stay optional.

No benchmarks or capability experiments were run during this research. Open gaps are symbol retention/source attribution in this exact Rust→WASM build, optimized profiling behavior of pinned CDP, worker-session protocol routing, available GPU queries/driver fields, browser-process RSS isolation, and measurement durations needed for stable samples per scene. These are bounded implementation probes; they do not justify framework replacement or dependency expansion.
