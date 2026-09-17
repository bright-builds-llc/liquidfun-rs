# Project Research Summary

**Project:** liquidfun-rs — v1.1 Web Playground
**Domain:** Static interactive physics gallery using native Rust through WebAssembly
**Researched:** 2026-09-17
**Confidence:** MEDIUM overall; HIGH for documented capabilities and inspected source boundaries

## Executive Summary

Build a playful SolidJS catalog with six user-approved scenes: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel. Use a private Rust WASM adapter around the existing engine, a small typed session interface and a Canvas 2D renderer. Ship the generated package inside the website; public npm/crates.io releases and legacy JavaScript API compatibility are outside this milestone. The user's later scope approval and current PROJECT.md/REQUIREMENTS.md supersede the earlier research wording that the exact scene selection remained unconfirmed.

The most useful first deliverable is one real Rust particle scene running in a browser, followed immediately by the shared player and a working GitHub Pages deployment. Existing native capability tests are not finished visual demos: some create two particles and then destroy them. Author persistent scenes against the public engine API instead of importing the desktop testbed, differential runner or C++ reference stack. Keep one world active, copy frame arrays, bound particle counts and simulation catch-up, and release resources explicitly on navigation/reset.

The main uncertainty is integration, not feature selection. No WASM build or browser run has established compatibility yet, and Pages setup remains unverified. Version pins are verified observations, not a tested combined toolchain. Prove these boundaries early, then tune the less certain floating, elastic and wheel scenes at modest scale. Preserve hobby scope: focused browser checks and site delivery are required; Linux native qualification, exhaustive parity, controlled benchmarking and broad browser matrices remain optional.

## Key Findings

### Recommended Stack

See [STACK.md](STACK.md) for verified versions and adoption details.

- **Existing Rust 1.97.0 plus `wasm32-unknown-unknown`:** retain native toolchain and Cargo default members; add a private `liquidfun-wasm` crate depending only on the public engine.
- **wasm-bindgen 0.2.128 and wasm-pack 0.15.0:** generate JS, TypeScript declarations and WASM locally using web output; initialize generated glue with an explicit Vite-resolved WASM URL.
- **SolidJS 1.9.15, Vite 8.3.0, vite-plugin-solid 2.11.14, TypeScript 7.0.2 and Bun 1.4.2:** plain static application with locked inputs; validate the combination in the first scaffold. Avoid SSR and unnecessary server tooling.
- **Canvas 2D:** sufficient starting renderer for clear particle/rigid geometry; measure before adding a GPU renderer or worker.
- **GitHub Actions/Pages:** build WASM and frontend from one checkout and deploy one artifact on every main push, without path filters. Verify project base, Pages source, normal token permissions and latest-main concurrency behavior.

Component-library selection remains a phase-level decision: MysticUI is the repo default when adopting a library, with its documented Tailwind 3 contract. Plain semantic controls and scoped CSS may be simpler for this small app, but require a recorded local decision rather than silently ignoring the standard.

### Expected Features

See [FEATURES.md](FEATURES.md) and the authoritative [v1.1 requirements](../../REQUIREMENTS.md).

**Must have:** Six distinct persistent scenes, catalog previews, shareable scene URLs, play/pause/reset, bounded scene controls, mouse/touch interactions, responsive accessible controls, useful loading/failure/retry states, source/inspiration credits, and automatic Pages delivery.

**Useful differentiators:** A short “try this” hint and behavior explanation for each scene; playful dark-default composition; visible Rust/WASM identity and discoverable source. Include version/build provenance and applicable maintainer disclosure in stable product chrome under existing standards.

**Defer:** Accounts, backend, scene editor, saved full simulation state, public package publication, multiplayer, recording/export, photorealism and unmeasured particle/FPS promises. Future creative ideas include Paint Aquarium, Jelly Obstacle Course, Hourglass and Tiny Waterworks; they do not expand the confirmed six-scene milestone.

### Architecture Approach

See [ARCHITECTURE.md](ARCHITECTURE.md). Keep physics and scene construction in Rust, browser lifecycle and input in a small TypeScript adapter, rendering in Canvas, and catalog/control state in Solid.

| Component | Responsibility |
| --- | --- |
| Existing `liquidfun` | Native physics and safe public engine API; no browser dependencies |
| Private `liquidfun-wasm` | Own sessions, create scenes, validate controls, step and pack owned frame data |
| `web/src/physics` | Initialize WASM, own one session, bound fixed-step timing and dispose resources |
| `web/src/render` | Canvas drawing, camera transforms and resize handling |
| Solid catalog/player | Navigation, static previews, controls, explanations and recovery states |
| Pages workflow | Reproducible targeted build and deployment of the same static artifact |

Use batched owned typed arrays rather than per-particle calls or raw memory views. Prefer direct public particle views for frame data: the existing debug collector can collect a full diagnostic observation even when some rendered layers are hidden. Start with ordinary unprofiled stepping because native profiling uses `Instant`. Use hash-selected scenes under the project base; the first release needs no server routing fallback.

### Critical Pitfalls

See [PITFALLS.md](PITFALLS.md).

1. **Native success hides a browser trap:** compile the targeted wrapper, initialize it and visibly step a real scene before multiplying features; do not assume OS-backed APIs or native panic recovery work in WASM.
1. **Local root URLs hide broken deployment:** preview the production subpath and refresh a direct scene link; verify the actual delivered JS/WASM response and initialization, not merely the HTML shell.
1. **Navigation leaks worlds or stale frames:** explicitly free sessions/frame wrappers, cancel animation/listeners and reject stale asynchronous initialization. Linear memory need not shrink to prove correct cleanup.
1. **Refresh rate or hidden tabs destabilize simulation:** fixed timestep, bounded catch-up, discard hidden-tab debt, one running world and capped emitters. Keep catalog previews static.
1. **Scene names overpromise physics:** visually demonstrate floating response, elastic coherence, real color-buffer mixing and particle-driven wheel motion. Do not substitute JavaScript animation or silently change approved scenes.

## Implications for Roadmap

Suggested phases continue after archived Phase 15; the roadmapper owns final requirement allocation.

### Phase 16: Browser Engine Bridge

**Rationale:** Target compatibility and frame ownership are the largest unresolved technical dependencies.

**Delivers:** Private wrapper, reproducible generated package, typed owned frames, validated session operations and a real browser stepping/disposal proof. Preserve normal native Cargo isolation.

**Addresses:** Rust-backed browser execution and package/frame boundary.

**Avoids:** Native-only API traps, diagnostic dependency coupling, per-particle crossings and unsafe memory-view lifetime assumptions.

### Phase 17: Shared Player and Early Pages Deployment

**Rationale:** Validate the full delivery path with one useful scene before authoring the remaining catalog.

**Delivers:** Solid shell, Canvas player, fixed-step lifecycle, play/pause/reset, loading/retry, static-host-safe navigation, base-path-correct production assets and main-push Pages workflow. Establish reusable input/control conventions and source/build chrome here.

**Addresses:** Shared catalog/player foundation, lifecycle, URL sharing and deployment. Partial catalog content should be labeled honestly until Phase 18 supplies all six scenes.

**Avoids:** Blank hosted pages, stale generated artifacts, leaked worlds, duplicate animation loops and catch-up storms.

### Phase 18: Six Interactive Scenes

**Rationale:** Reuse the proven session/player contract while varying scene behavior rather than framework plumbing.

**Delivers:** Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel with persistent compositions, small controls, documented interactions, bounded resources and per-scene credits.

**Addresses:** The six confirmed demo requirements and scene-specific control/interaction behavior.

**Avoids:** Empty diagnostic playback, unbounded emission, fake buoyancy/mixing/rotation and uncredited adaptation. Tune Float or Sink, Jelly Drop and Water Wheel early within the phase because they carry the greatest visual feasibility uncertainty.

### Phase 19: Interaction Polish and Hosted Verification

**Rationale:** Validate shared behavior across the completed set and the actual production delivery boundary.

**Delivers:** Responsive/playful presentation, keyboard and mouse/touch usability, reliable resize/camera behavior, concise explanations and focused real-browser checks across six scenes. Record live URL/source revision and verify refresh, assets, errors, repeated navigation, pause/reset and hidden-tab recovery.

**Addresses:** Final accessibility, interaction quality and integrated browser/hosting acceptance.

**Avoids:** Desktop-only input, coordinate drift, unexplained blank failures and a successful build being mistaken for a working website.

### Phase Ordering Rationale

Compile and execute first; deploy the smallest useful slice second; expand scene content third; verify and polish the complete experience last. Lifecycle and base-path correctness belong before scene expansion, not as final cleanup. Final verification strengthens evidence already gathered in earlier phases rather than delaying all browser checks until Phase 19.

### Research Flags

- **Phase 16 — targeted research/prototype:** Generated glue/toolchain integration, core target behavior and frame/disposal API require actual compilation and browser evidence.
- **Phase 17 — targeted configuration verification:** Resolve Pages setup/access and actual URL; validate production asset loading and action pins. Solid controls and hash navigation themselves are standard patterns.
- **Phase 18 — small visual experiments:** Establish stable floating, elastic and wheel presets using public engine APIs; no broad new physics research campaign.
- **Phase 19 — standard patterns:** Use focused browser, pointer, responsive and lifecycle checks; research further only if an observed problem requires it.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | HIGH facts / MEDIUM integration | Official versions and supported approaches researched; combined toolchain not built |
| Features | HIGH scope / MEDIUM scene tuning | Six scenes now explicitly approved; larger sustained scenes not demonstrated |
| Architecture | HIGH boundaries / MEDIUM performance | Public engine/source boundaries inspected; browser allocation and frame budget unmeasured |
| Pitfalls | HIGH mechanisms / MEDIUM applicability | Platform/lifecycle/hosting behavior documented; exact project failures not yet observed |

**Overall confidence:** MEDIUM until a real hosted Rust/WASM scene works.

### Gaps to Address

- WASM target was not installed and no target build/browser step ran during research. Close this in Phase 16.
- Pages GET returned 404; that is ambiguous and does not establish whether configuration is absent or current credentials can configure it. Verify setup explicitly in Phase 17.
- Select and test the initial particle budget and copied-frame layout using measured scenes, without an FPS warranty.
- Confirm tool pins work together and decide the minimal UI dependency approach during scaffolding.
- Prove the physical behavior of the higher-risk scenes; scope substitutions require an explicit decision.
- Delivery concurrency must satisfy latest-main behavior; triggering each push does not promise every superseded intermediate revision remains publicly visible.

## Sources

The four linked research reports contain detailed evidence and additional sources. This synthesis introduces no new external verification.

### Primary

- [Rust WASM target](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html) — target limitations and panic behavior.
- [wasm-bindgen deployment](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html) and [numeric slices](https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html) — generated web initialization and owned array transfer.
- [Vite WASM integration](https://vite.dev/guide/features.html#webassembly) and [Pages deployment](https://vite.dev/guide/static-deploy.html#github-pages) — asset handling and project base.
- [Solid cleanup](https://docs.solidjs.com/reference/lifecycle/on-cleanup) — lifecycle ownership.
- [GitHub custom Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages) and [concurrency](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency) — permissions, artifact deployment and ordering limitations.
- [Google LiquidFun showcase](https://google.github.io/liquidfun/), [particle guide](https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html) and [Faucet source](https://github.com/google/liquidfun/blob/master/liquidfun/Box2D/Testbed/Tests/Faucet.h) — scene inspiration and bounded emission.
- Repository engine/public-view/testbed source paths recorded in ARCHITECTURE.md and FEATURES.md — existing capabilities and isolation boundaries.

### Browser reference

- [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame) and [Pointer Events](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events) — animation timing, visibility and interaction behavior.

Local guidance applied: AGENTS.md hobby scope and standing authority, AGENTS.bright-builds.md, standards-overrides.md, standards/index.md, architecture/frontend standards, and both active lesson files. Historical research and phase evidence remain unchanged. Parent orchestration owns final review, verification and commit.

*Ready for roadmap: yes. This is research readiness, not implementation completion.*
