# Roadmap: liquidfun-rs

## Milestones

- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Active Milestone: v1.1 Web Playground

**Goal:** Let visitors explore six playful SolidJS demos powered by this repository's Rust engine through WebAssembly, with GitHub Pages delivery on every push to main.

The 22 approved requirements form four complete delivery boundaries. Although configuration selects fine granularity, further splitting would separate the shared player from its first hosted experience or fragment the six-scene catalog. Earlier phase directories and the v1.0 archive remain unchanged. No package publication, external physics substitution or strict native qualification is required.

## Phases

- [ ] **Phase 16: Rust WASM Browser Bridge** — Run a real native-engine scene in a browser through a reproducible, typed WASM package.
- [ ] **Phase 17: Shared Player and Early Pages Delivery** — Play one working scene on GitHub Pages with reliable navigation, lifecycle and automatic delivery.
- [ ] **Phase 18: Six Native Physics Demos** — Explore the complete approved catalog with real physics, bounded controls and source credits.
- [ ] **Phase 19: Interaction Polish and Browser Verification** — Use all six demos comfortably across pointer, keyboard and narrow-screen paths, verified in the built and hosted site.

## Phase Details

### Phase 16: Rust WASM Browser Bridge
**Goal**: A visitor can visibly run the existing Rust engine in a browser, and a contributor can rebuild its isolated WASM interface without changing ordinary native consumption.
**Depends on**: Nothing in v1.1 (uses the archived native foundation)
**Requirements**: WASM-01, WASM-02, WASM-03
**Success Criteria** (what must be TRUE):
  1. A real browser instantiates the generated Rust WASM package and visibly advances a particle/rigid-body scene from Rust-produced frame state; compiling or rendering canned motion is insufficient.
  2. A contributor can follow documented, pinned clean-checkout commands to build the local JS/TypeScript package and minimal frontend, while ordinary native Cargo consumers need no browser tools, C++ runtime or upstream checkout.
  3. The renderer consumes typed bulk owned frame data with no per-particle JS/Rust crossings or exposed raw engine pointers, and the proof can dispose its session explicitly.
**Plans**: 3 plans

Plans:
- [ ] 16-01-PLAN.md — Build the native-testable private WASM session, bounded proof scene, copied-frame ABI, and real wasm-pack generation gate.
- [ ] 16-02-PLAN.md — Add the exact-pinned Bun/SolidJS build, typed frame/session owner, and pure Canvas projection/rendering core.
- [ ] 16-03-PLAN.md — Prove visible Rust motion and disposal in Chromium, document isolation, and obtain independent exact-digest review.
**UI hint**: yes

Planning should prove target/runtime compatibility with ordinary unprofiled stepping before relying on native clocks or panic recovery. Do not presume 64-bit atomics are a target blocker without checking. Start with a small private wrapper and a real browser proof; no desktop testbed or differential-runner dependency is needed.

### Phase 17: Shared Player and Early Pages Delivery
**Goal**: Visitors can open, control and reload a working first scene on the actual GitHub Pages site, and new main pushes deliver a matching site/WASM artifact reliably.
**Depends on**: Phase 16
**Requirements**: WASM-04, WEB-02, WEB-03, WEB-06, HOST-01, HOST-02, HOST-03
**Success Criteria** (what must be TRUE):
  1. A visitor opens and reloads a stable scene URL under the real Pages project path with functioning JS/WASM assets; an unknown scene identifier provides a useful fallback. The deployed URL and source revision are recorded.
  2. The shared SolidJS player runs a useful first scene, supports play/pause/reset to its documented initial state, and presents visible loading, actionable failures and a working retry/reset path.
  3. Resetting, changing selection and leaving the player release prior worlds and animation resources; only the current session steps, hidden tabs do not accumulate catch-up debt, and stepping/emission stay bounded.
  4. Every push to main triggers the WASM and production-site build from the same checkout without path filters, and successful required build checks precede deployment of that assembled artifact.
  5. Deployment uses Pages configuration and ordinary Actions permissions without a personal token; overlapping pushes cannot leave an older revision as the final site, though superseded queued revisions may coalesce.
**Plans**: TBD
**UI hint**: yes

Deploy the thin working slice early. Label incomplete catalog entries honestly until Phase 18 supplies all six. Establish source/build chrome, input conventions and responsive structure here; final all-scene interaction acceptance belongs to Phase 19. A website delivery runner does not reintroduce mandatory Linux native qualification.

### Phase 18: Six Native Physics Demos
**Goal**: Visitors can choose six distinct, persistent physics scenes and experiment with each scene's real Rust behavior through a small, understandable control surface.
**Depends on**: Phase 17
**Requirements**: WEB-01, WEB-04, WEB-08, DEMO-01, DEMO-02, DEMO-03, DEMO-04, DEMO-05, DEMO-06
**Success Criteria** (what must be TRUE):
  1. The catalog presents Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel with names, descriptions and previews; each opens its working scene in the shared player with labeled, bounded controls and explicit reset-on-change behavior.
  2. Dam Break releases water into a basin with an interactive obstacle and repeatable reset; Fountain allows stream aiming or adjustment while lifetime/capacity limits make particle population plateau.
  3. Float or Sink accepts body/density presets and displays native particle-body response; Color Mixer allows stirring and visibly uses the engine's contact-driven particle color changes rather than rendering-only blending.
  4. Jelly Drop deforms an elastic particle shape through drops/pokes against obstacles with stable bounded presets; Water Wheel responds to a variable jet through native particle-body coupling and a joint, without scripted wheel rotation.
  5. Visitors can reach the repository, every scene's implementation, inspiration and applicable notices from the site; explanations and stable source/provenance chrome truthfully identify the experimental Rust implementation.
**Plans**: TBD
**UI hint**: yes

Prove the less certain floating, elastic and wheel compositions early within this phase. Existing diagnostic recipes are capability references, not finished visual scenes. Keep scene names and behaviors approved by the owner; a failed visual experiment calls for investigation or an explicit scope decision, not fake physics or silent substitution.

### Phase 19: Interaction Polish and Browser Verification
**Goal**: Visitors can comfortably interact with all six hosted demos using ordinary mouse, touch and keyboard controls, with focused evidence that the production experience works.
**Depends on**: Phase 18
**Requirements**: WEB-05, WEB-07, WEBTEST-01
**Success Criteria** (what must be TRUE):
  1. Each scene's documented mouse/touch interaction works after resize and at narrow widths, handles pointer cancellation without a stuck action, and preserves ordinary scrolling outside the player.
  2. The dark-default playful catalog and controls remain readable and usable at desktop and mobile widths, with labeled keyboard-operable controls, visible focus, contrast and concise text interaction instructions.
  3. A focused real-browser smoke suite uses the built Rust WASM artifact to select all six scenes, demonstrate visible stepping, exercise playback/reset and representative pointer/control input, and repeat scene changes while checking cleanup and hidden-tab recovery.
  4. Targeted production-subpath and live Pages checks demonstrate functioning JS/WASM loading and refreshed direct scene links for the complete gallery, recording the tested URL/revision without requiring a broad browser/native matrix.
**Plans**: TBD
**UI hint**: yes

## Progress

Execution order: **16 → 17 → 18 → 19**. Each phase includes focused checks of its own visible outcome; Phase 19 verifies the completed experience rather than postponing all browser testing until the end.

| Phase | Plans Complete | Status | Completed |
| --- | --- | --- | --- |
| 16. Rust WASM Browser Bridge | 0/TBD | Not started | - |
| 17. Shared Player and Early Pages Delivery | 0/TBD | Not started | - |
| 18. Six Native Physics Demos | 0/TBD | Not started | - |
| 19. Interaction Polish and Browser Verification | 0/TBD | Not started | - |

## Coverage and Planning Basis

All **22/22 requirements** map to exactly one phase: Phase 16 has 3, Phase 17 has 7, Phase 18 has 9 and Phase 19 has 3. No orphaned or duplicate assignments. See [REQUIREMENTS.md](REQUIREMENTS.md) for pending traceability and [research/v1.1/SUMMARY.md](research/v1.1/SUMMARY.md) for the evidence and unverified integration risks.

Local hobby scope in AGENTS.md and standards-overrides.md takes precedence over historical certification gates. AGENTS.bright-builds.md, the managed architecture and frontend standards, and the TypeScript/JavaScript guidance inform the thin browser boundary, dark-default SolidJS experience and source/provenance disclosure. Later UI planning will produce the design contract; this roadmap does not claim implementation, deployment or browser compatibility is already verified.
