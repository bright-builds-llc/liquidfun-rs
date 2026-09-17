---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 16-2026-09-17T01-48-41
generated_at: 2026-09-17T01:49:01.220Z
---

# Phase 16: Rust WASM Browser Bridge - Context

**Gathered:** 2026-09-17
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Prove that the existing native Rust engine can compile through a private bounded WebAssembly adapter, instantiate in a real browser, advance one persistent particle-and-rigid-body scene, and provide owned bulk frame data to a minimal SolidJS/Canvas proof. Contributors receive a pinned reproducible local build path while ordinary native Cargo consumers remain independent of browser tools. The shared player lifecycle, GitHub Pages deployment, six polished demos, interaction polish, and broad browser verification remain Phases 17 through 19.

</domain>

<decisions>
## Implementation Decisions

### Browser proof

- **D-01:** Demonstrate a minimal persistent particle-and-rigid-body basin visibly advancing from Rust-produced frame state in a real browser. A compile-only WASM result or canned JavaScript motion does not satisfy the phase.
- **D-02:** Provide a dark minimal SolidJS page with a Canvas 2D viewport and concise loading, running, failure, and Rust/WASM identity status. Do not build the catalog or full shared player in this phase.
- **D-03:** Use ordinary unprofiled engine stepping. Browser timing remains outside the engine and the proof does not depend on native profiling clocks, panic recovery, C++, or the private evidence runtime.

### Bridge and frame contract

- **D-04:** Add one opaque exported session that exclusively owns one Rust world and its private scene identities. Expose checked construction, fixed-step advancement, bulk frame capture, coarse counts/status, and explicit disposal only.
- **D-05:** Return a small constant number of owned copied typed arrays with documented element types and strides for particle positions, colors, radii, and bounded rigid geometry. Do not expose raw pointers, live WebAssembly memory views, public engine handles, or one JavaScript call per particle.
- **D-06:** Build frames directly from borrow-scoped public particle views and only the minimal public rigid/debug geometry needed by the proof. Do not collect the complete diagnostic checkpoint or change the published engine API speculatively.
- **D-07:** Validate finite inputs and bounds before engine calls. A trap invalidates the session; browser code recreates it instead of claiming native catch-unwind recovery.

### Build and dependency isolation

- **D-08:** Add an unpublished `liquidfun-wasm` workspace crate using `cdylib` plus `rlib` that depends only on the public `liquidfun` crate. Keep `liquidfun` as the sole default member and keep browser bindings out of the published engine.
- **D-09:** Use a small SolidJS and TypeScript Vite app under `web/`, with Bun and its committed lockfile as the routine script surface and `wasm-pack --target web` for generated bindings. Pin direct tool and package inputs at reviewed versions.
- **D-10:** Keep generated JavaScript, TypeScript declarations, package metadata, and WASM artifacts ignored and reproducible. One documented repo-owned command regenerates bindings from the current checkout before frontend typechecking and building.
- **D-11:** Use semantic HTML and scoped CSS with a dark default for this minimal proof, without adopting a component library. Reserve gallery-level component-library evaluation for the later shared UI phases.

### Verification and phase boundaries

- **D-12:** Add focused native tests for scene construction, boundary validation, and frame packing where practical; add TypeScript tests for frame-shape validation and pure Canvas projection; then run one focused Chromium smoke against the built app.
- **D-13:** The browser smoke must instantiate the generated WASM, prove consecutive Rust frame states move, visibly render that state, and exercise explicit session disposal. A broad browser matrix is not required.
- **D-14:** Preserve default native Cargo behavior, package isolation, safe opaque ownership, renderer independence, and optional heavy qualification. The browser runtime may not depend on C++, the protocol, differential runner, benchmarks, or desktop testbed.
- **D-15:** Defer shared play/pause/reset and async cleanup behavior, navigation, production Pages deployment, six authored demos, pointer interaction, responsive/accessibility polish, and milestone-level browser coverage to Phases 17 through 19.

### Claude's Discretion

- Exact private Rust and TypeScript type names, file decomposition, frame-array layout, proof-scene dimensions, modest particle count, colors, camera bounds, and test fixture values within the locked ownership, copying, isolation, and visibility requirements.
- Exact reviewed pins for compatible direct dependencies and the repo-owned command name, provided the lockfile and generated-package workflow remain reproducible and ordinary native consumers stay isolated.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active milestone scope

- `.planning/PROJECT.md` — v1.1 goal, confirmed six-scene direction, native-Rust runtime requirement, and milestone boundaries.
- `.planning/REQUIREMENTS.md` — `WASM-01` through `WASM-03` acceptance requirements and Phase 17 through 19 ownership.
- `.planning/ROADMAP.md` § Phase 16 — fixed goal, success criteria, dependency, and browser-proof planning note.
- `PROJECT-SCOPE.md` — hobby-project completion standard and optional strict qualification.

### Browser architecture and risk research

- `.planning/research/v1.1/ARCHITECTURE.md` — wrapper, session, copied-frame, Canvas, time, ownership, and dependency boundaries.
- `.planning/research/v1.1/STACK.md` — recommended Rust/WASM, SolidJS, Vite, Bun, `wasm-bindgen`, and `wasm-pack` build shape.
- `.planning/research/v1.1/FEATURES.md` — persistent scene expectations, engine capability evidence, and physics-versus-presentation boundary.
- `.planning/research/v1.1/PITFALLS.md` — browser traps, copied-memory rationale, target uncertainty, and phase placement.

### Existing engine contracts and integration seams

- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — safe opaque ownership, stable particle identity, and checked mutation contracts.
- `.planning/phases/11-examples-headless-tooling-and-testbed/11-CONTEXT.md` — renderer-neutral scenario/controller boundaries and prohibition on testbed authority leaking into physics.
- `ARCHITECTURE.md` — dependency direction, renderer independence, public engine ownership, and native Cargo isolation.
- `Cargo.toml` — workspace/default-member and lint contracts to preserve.
- `crates/liquidfun/src/lib.rs` — public engine curation boundary and ordinary unprofiled stepping API.
- `crates/liquidfun/src/world/step.rs` — stepping boundary and native-only profiling dependencies to avoid.
- `crates/liquidfun/src/world/particle_object/system.rs` — public particle-system construction, inspection, controls, and views.
- `crates/liquidfun/src/particle/view.rs` — borrow-scoped positions, colors, stable identities, and semantic particle lanes for owned frame packing.
- `crates/liquidfun/src/debug_draw/` — renderer-neutral rigid/debug geometry vocabulary available to a narrow adapter.
- `crates/liquidfun-test-protocol/src/catalog/scenarios/` — capability and public-API examples only; this private evidence runtime must not become a browser dependency.

### Repository standards

- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — repo-local workflow, hobby scope, independent review, and managed standards.
- `standards/core/architecture.md` — functional-core/imperative-shell and invariant-bearing boundary guidance.
- `standards/core/code-shape.md` — shallow control flow, optional naming, module sizing, and script boundaries.
- `standards/core/frontend-ui.md` — dark default, public source identity, and future product-chrome expectations.
- `standards/core/testing.md` — focused Arrange/Act/Assert unit tests.
- `standards/core/verification.md` — sync-first and repo-native verification requirements.
- `standards/languages/rust.md` and `standards/languages/typescript-javascript.md` — Rust module/API conventions and the SolidJS/Bun frontend defaults.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `ParticleSystemView` already exposes borrow-scoped positions, optional colors, stable particle identities, velocities, flags, and contact records suitable for flattening into owned frame arrays.
- Public `World`, particle-system, body, fixture, and stepping APIs are sufficient to author a small persistent private browser scene without exposing storage or importing the evidence protocol.
- Renderer-neutral debug primitives and world observations can inform bounded rigid geometry, though full diagnostic collection should not be the default frame path.
- Existing private catalog scenarios and native executor code show checked construction and stepping patterns, but their replay/evidence lifecycle is inappropriate for continuous browser execution.

### Established Patterns

- `liquidfun` is the sole default publishable crate; all adapters and tools depend outward from it.
- Production code forbids unsafe code, uses checked types and opaque identities, preserves deterministic source order, and exposes owned or borrow-scoped semantic values rather than storage coordinates.
- Renderers and UI shells own presentation, clocks, camera state, and pixels; simulation remains renderer-independent and headless.

### Integration Points

- Add the private browser wrapper beside existing private workspace crates without changing default members.
- Add the minimal frontend under `web/`, consuming only generated local bindings from the wrapper.
- Extend the root command facade and contributor documentation with one transparent browser-proof build path.
- Keep later player, catalog, deployment, and interaction layers able to deepen the proof-sized session contract without promising a public compatibility API.

</code_context>

<specifics>
## Specific Ideas

- Start with one basin containing enough liquid particles and a rigid obstacle to make two consecutive Rust-produced frames visibly and numerically different.
- Render particles as simple Canvas circles and rigid geometry as a bounded line/circle command stream; attractive gallery chrome and elaborate effects wait for later phases.
- Treat copied typed arrays as the safe baseline. Consider zero-copy views or workers only after a measured browser bottleneck and a separately documented invalidation/ownership contract.

</specifics>

<deferred>
## Deferred Ideas

- Shared player lifecycle, play/pause/reset, loading retry, hidden-tab handling, stable scene URLs, and GitHub Pages deployment — Phase 17.
- Six polished demo definitions, catalog cards, scene controls, source/inspiration chrome, and gallery component-library choices — Phase 18.
- Pointer/touch behavior, responsive/accessibility polish, repeated scene cleanup, production-path smoke checks, and broader browser verification — Phase 19.
- Public npm or crates.io publication, a stable JavaScript API, workers, WASM threads, zero-copy memory views, WebGPU, SSR, and broad browser/platform guarantees — outside v1.1.

</deferred>

---

*Phase: 16-rust-wasm-browser-bridge*
*Context gathered: 2026-09-17*
