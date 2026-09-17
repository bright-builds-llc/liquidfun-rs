# Architecture Research: v1.1 Browser Demo Gallery

**Researched:** 2026-09-17
**Scope:** Six interactive SolidJS demos using this repository's Rust engine through its own JavaScript/TypeScript WebAssembly interface, deployed to GitHub Pages.
**Confidence:** HIGH for existing source boundaries and documented web APIs; MEDIUM for performance and browser feasibility until the first real browser build.

**Scope clarification — 2026-09-17:** The owner subsequently approved the exact six scenes: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel, with the proposed gallery/player, interaction, sharing, attribution and main-push Pages scope. These are the current PROJECT.md/REQUIREMENTS.md targets. The architecture remains a recommendation and browser feasibility remains unproven.

## Recommendation

Add one private `liquidfun-wasm` workspace crate wrapping `liquidfun`, and one static SolidJS application under `web/`. Keep the six scene definitions and simulation controls in the wrapper's ordinary Rust modules; expose a small JavaScript session API with generated TypeScript declarations. Use Canvas 2D initially and copied typed-array frame snapshots. Run one active simulation on the main browser thread with a bounded fixed timestep. Prove one particle scene works in a real browser before investing in the complete catalog.

This is a gallery-oriented interface, not a promise to expose every LiquidFun method or provide compatibility with an unrelated `liquidjs` package. It can grow later if a concrete reusable JavaScript API is desired. The native engine remains the physics implementation and stays independent of browser dependencies, renderers, C++, and application scenes.

## Existing Architecture and Integration Evidence

| Existing surface | Observed behavior | Integration decision |
| --- | --- | --- |
| Root `Cargo.toml` | Workspace defaults to only `crates/liquidfun`; private differential, testbed, benchmark and xtask members are separate | Preserve default members. Add the wrapper as a member and explicitly target it for browser builds |
| `crates/liquidfun/Cargo.toml` | Only runtime dependency is `bitflags`; no default features; `differential-internals` is opt-in | Wrapper depends on default public engine API only |
| `crates/liquidfun/src/lib.rs` | Public world/body/fixture/joint/particle APIs, checked configuration, stable owner-scoped identities | Reuse public constructors and mutations; do not expose internal handles as raw numbers or pointers |
| `src/world/step/execution.rs` | Ordinary `World::step` disables profiling; `step_profiled` enables it | Call ordinary stepping in the browser |
| `src/world/observation/profile.rs` | Enabled diagnostic profiling calls `std::time::Instant::now()` | Keep native diagnostic profiling out of the web path; use browser timing around the boundary for optional UI diagnostics |
| `src/particle/view.rs` | `ParticleSystemView` exposes borrow-scoped positions, velocities, optional colors and stable particle identities | Flatten positions/colors in Rust into owned frame arrays during a bounded borrow |
| `src/debug_draw/collector.rs` and `primitive.rs` | Owned, renderer-neutral geometry includes points, segments, polylines, circles, transforms, AABBs, arrows and labels | Use as the first rigid geometry adapter or reference for one; keep web draw records independent from engine storage |
| `src/world/observation/collection.rs` | Full observations collect contacts, statistics, broad-phase and geometry under reviewed limits | Avoid collecting a complete diagnostic report solely to display particle positions |
| `crates/liquidfun-test-protocol/src/catalog/scenarios/` | Existing rigid, joint, rope, particle, group and callback examples are evidence-oriented scenario definitions | Reuse ideas and public API patterns, not the private evidence protocol as the gallery's runtime format |
| `crates/liquidfun-differential/src/catalog_native/executor.rs` | Native catalog backend binds request hashes, reconstructs a session from prior actions and limits replay to 128 actions | Do not import it into a continuously running browser demo |
| `crates/liquidfun-testbed/Cargo.toml` | Desktop testbed depends on eframe/wgpu, differential runner and private protocol | Do not port the desktop dependency graph into the SolidJS application |

Paths beginning `src/` in this table are relative to `crates/liquidfun/`.

The source search found no `std::fs`, `std::process`, `std::thread`, or `std::net` use in the engine. This is useful evidence, not proof of browser compatibility. The installed compiler's `rustc --print cfg --target wasm32-unknown-unknown` reports `target_has_atomic="64"`, so the core's `AtomicU64` world identity allocator is not an assumed compile blocker. Only `aarch64-apple-darwin` was installed at research time; no WASM compilation or runtime test was performed and no target/tool installation was made.

## Component Boundaries

```text
SolidJS catalog, player and controls
              |
     TypeScript runtime adapter ---- Canvas renderer
              |                     ^
     generated JS/TS bindings -------| copied frame arrays
              |
     liquidfun-wasm session + Rust scene definitions
              |
     liquidfun public native Rust engine
```

| Component | Responsibility | Must stay outside |
| --- | --- | --- |
| `crates/liquidfun` | Physics, safe ownership, checked mutations | DOM, JS bindings, scene catalog, C++ runtime |
| `crates/liquidfun-wasm` | Own one world per session, initialize scenes, apply bounded controls, step and encode frames | Evidence campaign protocol, filesystem/process execution, DOM rendering |
| `web/src/physics/` | Load WASM once, own the active session, animation loop and cleanup | Physics solver implementation |
| `web/src/render/` | Canvas sizing, world-to-screen transform, efficient particle/shape drawing | Simulation time or mutable engine state |
| `web/src/catalog/` | Six stable IDs, titles, descriptions, parameter metadata, thumbnails and source/inspiration credits | Live worlds for every preview card |
| Solid components | Navigation, loading/error/empty states, accessible controls and product chrome | Per-particle reactive signals |
| Pages workflow | Targeted Rust/WASM build, app build, static artifact upload and deployment | Desktop/oracle/full-platform validation |

Use `cdylib` plus `rlib` for the wrapper so Rust scene/session logic can be tested natively. Mark the package `publish = false` initially: delivering generated bindings within the website is enough for this milestone. Neither npm nor crates.io publication follows from website deployment authorization.

## Session and Frame Contract

One proposed minimal interface is `createDemo(id, seed)`, `setParameter(name, value)`, `pointerAction(kind, x, y)`, `step()`, `reset(seed)` and `frame()`, plus explicit session disposal. These are proposed product operations, not existing exports. Validate scene IDs, finite numbers, limits and control ranges at this boundary. Keep engine identities private; dragging can retain a selected `BodyId` internally.

Return a compact frame with batched particle positions (`Float32Array`), colors (`Uint8Array`) and any necessary radii, plus a bounded shape-command stream. Separate low-volume UI counters from high-volume geometry. Use a documented coordinate convention and explicit array strides. Initially return owned numeric vectors/boxed slices through wasm-bindgen; its documented conversion copies them into JavaScript typed arrays. This avoids retaining unsafe views across Rust allocation or WebAssembly memory growth. A small constant number of frame calls is acceptable; one JS call per particle is not.

Retain the frame arrays only as long as rendering needs them. If exporting a Rust frame class, release that frame wrapper after reading its copied arrays; do not leak one Rust allocation per animation frame. Repeated reset and scene-switch tests should verify stable object counts and no surviving animation loop. JavaScript garbage collection alone is not the session lifecycle design.

For rigid geometry, start with `collect_debug_primitives` and convert only visible primitives. For particle geometry, prefer the direct public view lanes. The existing collector internally requests a full `world_observation`, so hiding contact layers does not automatically remove observation overhead. If this becomes material, keep the adapter narrow: retain scene-owned fixture IDs and use public snapshots, or add a proven useful engine observation capability later. Do not preemptively refactor the entire observation subsystem.

## Time, Input and Ownership

Use requestAnimationFrame for rendering with a fixed simulation step, initially 1/60 second. Accumulate elapsed time, cap accepted wall-clock delta and cap steps per rendered frame. Start with at most four steps; tune this together with a conservative particle cap after measuring the six scenes. Pause while the document is hidden and clear accumulated time when resuming so returning to a tab does not trigger seconds of catch-up work. The exact particle cap is a phase-level measurement, not a performance claim from research.

The player owns one session, one pending animation request, one resize observer and its event listeners. Solid's cleanup hook cancels the animation, removes listeners, disconnects the observer and frees the session. If asynchronous WASM loading finishes after the player unmounts or changes scenes, discard the stale request rather than installing a second session. A generation token is enough; no global event bus is needed.

Resize changes the canvas backing dimensions and camera transform, not the world geometry or seed. Pointer coordinates must be converted using the current element bounds and the same camera transform used for rendering. Account for device-pixel ratio without changing physics coordinates. Reset recreates the scene from its seed and current documented parameter policy. Unknown settings or step failures produce an actionable paused error state rather than silent continued execution.

Avoid native profiled stepping. The Rust minimal WASM target has only partial OS-backed standard library behavior; ordinary native compilation alone does not establish browser safety. Keep recoverable failures in `Result`-based APIs; a WebAssembly panic/trap should stop and discard the current session rather than pretending the native catch-unwind recovery contract still applies.

## Concrete File Integration

Proposed new files, refined during phase planning:

- `crates/liquidfun-wasm/Cargo.toml`, `src/lib.rs`, `src/session.rs`, `src/frame.rs`, `src/scenes.rs` and `src/scenes/`: narrow wrapper and six source-owned scenes.
- `web/package.json`, lockfile, `vite.config.ts`, `index.html`, `src/main.tsx` and `src/App.tsx`: static SolidJS build.
- `web/src/physics/loader.ts`, `session.ts` and `clock.ts`: typed bindings, lifecycle and bounded time logic.
- `web/src/render/canvas.ts` and `camera.ts`: renderer and pure coordinate transforms.
- `web/src/catalog/demos.ts`: title/control/credit metadata keyed by the same explicit IDs as Rust scene selection.
- `web/src/components/`: catalog cards, player, controls, loading/error states and stable provenance/source chrome.
- A generated, ignored package directory consumed by Vite: built before local development and production build, never hand-edited. Pick one path and one build command rather than separate local/CI assembly recipes.
- `.github/workflows/pages.yml`: build and deploy on pushes to `main`; use the repository's pinned action conventions.
- Root workspace membership, a thin `just` recipe and developer documentation: keep normal core-only Cargo behavior intact.

Prefer hash-based demo navigation or a query-selected scene for the first GitHub Pages application. Use the correct Vite base for the repository site and test built JS and WASM URLs under that base. Avoid inventing a server fallback or backend for six static routes. Build and deploy the same artifact; do not rebuild WASM in a separate deploy job with different inputs.

## Suggested Builder Phases

These numbers are recommendations for roadmap synthesis, not an approved roadmap.

1. **Phase 16 — Browser engine bridge:** Isolate the wrapper, compile only the targeted engine/wrapper for wasm32, initialize one real particle world in a browser, step it, inspect copied frame output and exercise disposal. Verify ordinary Cargo builds remain C++-free.
1. **Phase 17 — Gallery/player and first deployment:** Build SolidJS catalog and reusable Canvas player, fixed-step lifecycle, responsive input, loading/error states, visible source/build metadata, and deploy a working initial scene through GitHub Pages on `main` pushes. Validate repository-subpath asset loading early.
1. **Phase 18 — Six interactive demos:** Implement Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel against the session contract. Use one shared reset/control model; add per-demo attribution and a compact supported-parameter surface. The approved Water Wheel toy still needs a visual feasibility check.
1. **Phase 19 — Polish and browser verification:** Check all six scenes, pause/reset/switch/resize/hidden-tab behavior, production build under the Pages base, errors and mobile-friendly controls. Improve measured bottlenecks with particle budgets before adopting workers or a new renderer.

Deploying a thin slice before all scenes closes the main integration uncertainty early. Do not revive Linux qualification, C++ differential parity, a platform matrix or formal benchmark certification as milestone completion gates. The browser build/deploy workflow is a distinct product delivery requirement authorized by the user.

## Verification and Deferred Complexity

Targeted verification should demonstrate visible movement from Rust-generated frame state, deterministic reset for a selected seed, different intended scene behavior, no live session after disposal, and working production WASM loading. Unit tests belong around control validation, bounded stepping decisions, frame packing and camera transforms. A small browser smoke suite covers the integration boundary; it does not need an exhaustive browser/platform matrix.

Defer workers, SharedArrayBuffer, WASM threads, zero-copy pointer views, WebGPU, remote scene services, SSR, multiplayer, arbitrary scene scripting and public package releases. A dedicated worker becomes worthwhile only if measured modest scenes still prevent responsive controls after reducing unnecessary snapshot work and particle counts. Canvas 2D is a starting recommendation, not a promise of a particular particle throughput.

## Sources and Confidence

- Repository source paths listed above, inspected 2026-09-17: HIGH for existing boundaries.
- [Rust target documentation](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html): HIGH for partial standard-library support, target setup and default panic strategy; browser behavior still requires a smoke test.
- [wasm-bindgen boxed numeric slices](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html): HIGH for copied typed-array boundary semantics.
- [wasm-bindgen WebGL example](https://wasm-bindgen.github.io/wasm-bindgen/examples/webgl.html): HIGH for its explicit warning about memory-view invalidation across allocations; this recommendation avoids that unsafe path.
- [SolidJS onCleanup](https://docs.solidjs.com/reference/lifecycle/on-cleanup): HIGH for framework-owned cleanup registration.
- [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame): HIGH for browser animation scheduling; fixed timestep and catch-up limits are architectural recommendations.

Local guidance materially applied: current hobby scope and independent-review rules in `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md` and `standards/core/architecture.md`. The functional-core/imperative-shell split keeps pure physics and scene logic in Rust while DOM lifetime, clocks and rendering stay in the web adapter. No core changes, build installations or commits were made during this research.
