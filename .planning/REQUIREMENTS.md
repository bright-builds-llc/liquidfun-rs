# Requirements: liquidfun-rs Web Playground

**Milestone:** v1.1 Web Playground (planning label, not a package release)
**Defined:** 2026-09-17
**Core Value:** Make our Rust physics engine easy and fun to explore in a browser.

The owner approved six varied demos, our own Rust engine via WASM, a playful gallery, shareable scene links, playback/reset/scene controls, mouse/touch interaction, source/inspiration credits and GitHub Pages deployment from main. Prior milestone requirements remain archived; strict native qualification is not revived.

## v1.1 Requirements

### Rust and browser boundary

- [x] **WASM-01**: A visitor can run and visibly step a particle/rigid-body scene using this repository's Rust engine compiled to WebAssembly, without a C++ runtime or external JavaScript physics engine.
- [x] **WASM-02**: A contributor can reproducibly build the local JS/TypeScript WASM package and frontend from a clean checkout with documented commands and pinned tool inputs; ordinary native Cargo consumers remain independent of browser tooling.
- [x] **WASM-03**: The browser renderer receives bulk owned frame data through a typed interface without per-particle JS/Rust calls or exposed raw engine pointers.
- [x] **WASM-04**: Switching/resetting a scene releases its prior world and animation resources; bounded stepping, emission and hidden-tab handling prevent unbounded catch-up or particle accumulation.

### Playful catalog and player

- [ ] **WEB-01**: A visitor can browse six demo cards with names, short descriptions and previews, then open a selected demo in a shared player.
- [x] **WEB-02**: A visitor can share and reload a stable demo URL under the repository's GitHub Pages path; an unknown scene identifier returns a useful catalog/fallback view.
- [x] **WEB-03**: A visitor can play, pause and reset the selected demo to its documented initial state.
- [ ] **WEB-04**: Each demo exposes a small set of labeled, bounded controls and clearly indicates when changing a setting resets the scene.
- [ ] **WEB-05**: A visitor can use mouse or touch for each demo's documented interaction without leaving a stuck pointer or preventing ordinary scrolling outside the player.
- [x] **WEB-06**: WASM loading and simulation startup show visible progress or loading status, useful failure feedback, and a working retry/reset path instead of a blank canvas.
- [ ] **WEB-07**: The dark-default playful gallery and controls remain usable at desktop and narrow/mobile widths, with readable contrast, keyboard-operable controls, focus indication and concise text interaction instructions.
- [ ] **WEB-08**: Visitors can find the repository, each scene's implementation, inspiration links and applicable attribution/notices from the site; claims accurately identify the experimental Rust implementation.

### Six launch scenes

- [ ] **DEMO-01**: Dam Break lets the visitor release water into a basin and interact with an obstacle, with visibly native particle/rigid behavior and a repeatable reset.
- [ ] **DEMO-02**: Fountain lets the visitor aim or adjust a continuous stream into a container while particle lifetime/capacity limits keep emission bounded.
- [ ] **DEMO-03**: Float or Sink lets the visitor drop different body/density presets into a pool and observe the Rust engine's particle-body response without fake buoyancy animation.
- [ ] **DEMO-04**: Color Mixer lets the visitor stir colored particle groups and observe the engine's actual contact-driven color mixing, distinguished from rendering-only blending.
- [ ] **DEMO-05**: Jelly Drop lets the visitor drop and poke an elastic particle shape against obstacles, with stable bounded presets and a clear reset.
- [ ] **DEMO-06**: Water Wheel lets the visitor vary a jet that interacts with a pinned paddle wheel through native particle-body coupling and a joint; scene feasibility is demonstrated rather than substituted with scripted rotation.

### Build, hosting and focused verification

- [x] **HOST-01**: Every push to main triggers a build of the WASM package and SolidJS production site from the same checkout, followed by GitHub Pages deployment only after required build checks succeed.
- [x] **HOST-02**: Deployment uses the repository's Pages configuration and normal GitHub Actions permissions without a personal token, and concurrent pushes cannot leave an older completed deployment as the final site; intermediate queued revisions may be superseded by newer main pushes.
- [x] **HOST-03**: The live Pages site loads its JS/WASM assets and direct demo URLs under the real project base path, with a recorded deployed URL and source revision.
- [ ] **WEBTEST-01**: A focused real-browser smoke suite exercises the built Rust WASM artifact, all six scene selections, visible stepping, playback/reset, representative pointer/control input and repeated scene cleanup; the production subpath build and deployed site receive targeted smoke checks without a broad browser/native qualification matrix.

## Future Ideas

Splash pinball, jelly obstacle courses, fluid-powered factories, wave/surf toys and fluid painting are candidates for a later brainstorming pass. A scene editor, saved full simulation states, presets shared through a backend, advanced renderers and worker/thread acceleration wait for demonstrated need.

## Out of Scope

- External JS/C++ physics substitution or a compatibility clone of a legacy LiquidFun JS API.
- npm/crates.io publication, package release tags or a stable public browser API guarantee.
- Accounts, backend services, SSR, content-management tools or a general-purpose scene editor.
- Mandatory Linux native qualification, strict parity certification, exhaustive cross-browser matrices, controlled performance hardware or unmeasured FPS guarantees.
- Replacing the private desktop testbed or rewriting the engine to fit a frontend framework.

## Traceability

Each active requirement maps to exactly one new phase, starting at Phase 16. All requirements are pending until implemented and verified.

| Requirement | Phase | Status |
| --- | --- | --- |
| WASM-01 | Phase 16 | Complete |
| WASM-02 | Phase 16 | Complete |
| WASM-03 | Phase 16 | Complete |
| WASM-04 | Phase 17 | Complete |
| WEB-01 | Phase 18 | Pending |
| WEB-02 | Phase 17 | Complete |
| WEB-03 | Phase 17 | Complete |
| WEB-04 | Phase 18 | Pending |
| WEB-05 | Phase 19 | Pending |
| WEB-06 | Phase 17 | Complete |
| WEB-07 | Phase 19 | Pending |
| WEB-08 | Phase 18 | Pending |
| DEMO-01 | Phase 18 | Pending |
| DEMO-02 | Phase 18 | Pending |
| DEMO-03 | Phase 18 | Pending |
| DEMO-04 | Phase 18 | Pending |
| DEMO-05 | Phase 18 | Pending |
| DEMO-06 | Phase 18 | Pending |
| HOST-01 | Phase 17 | Complete |
| HOST-02 | Phase 17 | Complete |
| HOST-03 | Phase 17 | Complete |
| WEBTEST-01 | Phase 19 | Pending |

**Coverage:** 22 requirements; 22 mapped exactly once; 0 unmapped.
