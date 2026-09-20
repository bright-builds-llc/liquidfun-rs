# Roadmap: liquidfun-rs

## Milestones

- [x] **v1.0 Experimental Foundation** — 16 phases / 252 active plans complete; archived 2026-09-17 under hobby scope ([full roadmap](milestones/v1.0-ROADMAP.md)). Strict qualification remains deferred; no package or tag released.

## Active Milestone: v1.1 Web Playground

**Goal:** Let visitors explore six playful SolidJS demos powered by this repository's Rust engine through WebAssembly, with GitHub Pages delivery on every push to main.

The 22 approved requirements originally formed four delivery boundaries. A 2026-09-20 milestone audit found two post-gate UI fidelity gaps plus leftover cleanup; Phases 20 and 21 close that debt before v1.1 completion. Earlier phase directories and the v1.0 archive remain unchanged. No package publication, external physics substitution or strict native qualification is required.

## Phases

- [x] **Phase 16: Rust WASM Browser Bridge** — Run a real native-engine scene in a browser through a reproducible, typed WASM package. (completed 2026-09-17)
- [x] **Phase 17: Shared Player and Early Pages Delivery** — Play one working scene on GitHub Pages with reliable navigation, lifecycle and automatic delivery. (completed 2026-09-17)
- [x] **Phase 18: Six Native Physics Demos** — Explore the complete approved catalog with real physics, bounded controls and source credits. (completed 2026-09-18)
- [x] **Phase 19: Interaction Polish and Browser Verification** — Use all six demos comfortably across pointer, keyboard and narrow-screen paths, verified in the built and hosted site. (completed 2026-09-19)
- [ ] **Phase 20: Playground catalog previews and Reset honesty** — Restore in-app demo previews and honest live-control labels after Reset.
- [ ] **Phase 21: Playground leftover cleanup** — Remove dead player/proof leftovers and split oversized WASM scene files.

## Phase Details

### Phase 16: Rust WASM Browser Bridge
**Goal**: A visitor can visibly run the existing Rust engine in a browser, and a contributor can rebuild its isolated WASM interface without changing ordinary native consumption.
**Depends on**: Nothing in v1.1 (uses the archived native foundation)
**Requirements**: WASM-01, WASM-02, WASM-03
**Success Criteria** (what must be TRUE):
  1. A real browser instantiates the generated Rust WASM package and visibly advances a particle/rigid-body scene from Rust-produced frame state; compiling or rendering canned motion is insufficient.
  2. A contributor can follow documented, pinned clean-checkout commands to build the local JS/TypeScript package and minimal frontend, while ordinary native Cargo consumers need no browser tools, C++ runtime or upstream checkout.
  3. The renderer consumes typed bulk owned frame data with no per-particle JS/Rust crossings or exposed raw engine pointers, and the proof can dispose its session explicitly.
**Plans**: 4 plans

Plans:
- [x] 16-01-PLAN.md — Build the native-testable private WASM session, bounded proof scene, copied-frame ABI, and real wasm-pack generation gate.
- [x] 16-02-PLAN.md — Add exact-pinned Bun/SolidJS generation plus the validated typed frame/session ownership boundary.
- [x] 16-03-PLAN.md — Implement pure Canvas rendering and the approved SolidJS proof page, then run the first complete frontend build.
- [x] 16-04-PLAN.md — Prove and retain visible Rust motion/disposal in Chromium, document isolation, and obtain exact-digest independent review.
**UI hint**: yes

Planning should prove target/runtime compatibility with ordinary unprofiled stepping before relying on native clocks or panic recovery. Do not presume 64-bit atomics are a target blocker without checking. Start with a small private wrapper and a real browser proof; no desktop testbed or differential-runner dependency is needed.

### Phase 17: Shared Player and Early Pages Delivery
**Goal**: Visitors can open, control and reload a working first scene on the actual GitHub Pages site, and new main pushes deliver a matching site/WASM artifact reliably.
**Depends on**: Phase 16
**Requirements**: WASM-04, WEB-02, WEB-06, HOST-01, HOST-02, HOST-03
**Success Criteria** (what must be TRUE):
  1. A visitor opens and reloads a stable scene URL under the real Pages project path with functioning JS/WASM assets; an unknown scene identifier provides a useful fallback. The deployed URL and source revision are recorded.
  2. The shared SolidJS player runs a useful first scene, supports play/pause/reset to its documented initial state, and presents visible loading, actionable failures and a working retry/reset path.
  3. Resetting, changing selection and leaving the player release prior worlds and animation resources; only the current session steps, hidden tabs do not accumulate catch-up debt, and stepping/emission stay bounded.
  4. Every push to main triggers the WASM and production-site build from the same checkout without path filters, and successful required build checks precede deployment of that assembled artifact.
  5. Deployment uses Pages configuration and ordinary Actions permissions without a personal token; overlapping pushes cannot leave an older revision as the final site, though superseded queued revisions may coalesce.
**Plans**: 8 plans

Plans:
- [x] 17-01-PLAN.md — Lock the six-scene catalog and pure `#/scene/{id}` hash parser
- [x] 17-02-PLAN.md — Cap stepping at 4 ticks and forward one 1–4 advance per capture
- [x] 17-03-PLAN.md — Bake `/liquidfun-rs/` production assets and safe provenance
- [x] 17-04-PLAN.md — Build honest catalog, fallback, footer, and player chrome
- [x] 17-05-PLAN.md — Wire the one-session Dam Break player and teardown
- [x] 17-06-PLAN.md — Prove production-base Dam Break, fallback, and hidden-tab bounds
- [x] 17-07-PLAN.md — Add the SHA-pinned same-checkout Pages workflow
- [x] 17-08-PLAN.md — Deploy main, record the live URL/SHA, and obtain independent review
**UI hint**: yes

Deploy the thin working slice early. Label incomplete catalog entries honestly until Phase 18 supplies all six. Establish source/build chrome, input conventions and responsive structure here; final all-scene interaction acceptance belongs to Phase 19. A website delivery runner does not reintroduce mandatory Linux native qualification.

### Phase 18: Six Native Physics Demos
**Goal**: Visitors can choose six distinct, persistent physics scenes and experiment with each scene's real Rust behavior through a small, understandable control surface.
**Depends on**: Phase 17
**Requirements**: WEB-04, WEB-08, DEMO-01, DEMO-02, DEMO-03, DEMO-04, DEMO-05, DEMO-06
**Success Criteria** (what must be TRUE):
  1. The catalog presents Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel with names, descriptions and previews; each opens its working scene in the shared player with labeled, bounded controls and explicit reset-on-change behavior.
  2. Dam Break releases water into a basin with an interactive obstacle and repeatable reset; Fountain allows stream aiming or adjustment while lifetime/capacity limits make particle population plateau.
  3. Float or Sink accepts body/density presets and displays native particle-body response; Color Mixer allows stirring and visibly uses the engine's contact-driven particle color changes rather than rendering-only blending.
  4. Jelly Drop deforms an elastic particle shape through drops/pokes against obstacles with stable bounded presets; Water Wheel responds to a variable jet through native particle-body coupling and a joint, without scripted wheel rotation.
  5. Visitors can reach the repository, every scene's implementation, inspiration and applicable notices from the site; explanations and stable source/provenance chrome truthfully identify the experimental Rust implementation.
**Plans**: 10 plans
**UI hint**: yes

Plans:
- [x] 18-01-PLAN.md — Checked scene-id factory, extracted Dam Break, apply_control/apply_action
- [x] 18-02-PLAN.md — Catalog control metadata and host-locked credit URLs
- [x] 18-03-PLAN.md — Native Float or Sink spike with cork-versus-stone y-separation
- [x] 18-04-PLAN.md — Native Jelly Drop spike with elastic group and poke
- [x] 18-05-PLAN.md — Native Water Wheel spike with motor-off jet rotation
- [x] 18-06-PLAN.md — Evolve Dam Break controls and build bounded Fountain
- [x] 18-07-PLAN.md — Color Mixer contact-driven mixing honesty
- [x] 18-08-PLAN.md — Six static SVG catalog cards and fallback copy
- [x] 18-09-PLAN.md — Ready flags, player controls, credits, and one-session wiring
- [x] 18-10-PLAN.md — Local six-scene Chromium proofs and independent AI review

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
**Plans:** 7/7 plans complete
**UI hint**: yes

Plans:
- [x] 19-01-PLAN.md — CSS→world unproject and captured-gesture reducer
- [x] 19-02-PLAN.md — WASM pointer_action plus Dam Break, Fountain, and Float or Sink
- [x] 19-03-PLAN.md — Color Mixer stir, Jelly poke, and Water Wheel jet aim
- [x] 19-04-PLAN.md — Canvas Pointer Events pipeline and teardown cancel
- [x] 19-05-PLAN.md — Per-scene figcaption, select focus, and 480px polish
- [x] 19-06-PLAN.md — Chromium player smoke for pointer, 375px, and cleanup
- [x] 19-07-PLAN.md — Live Pages six-hash evidence and independent AI review

### Phase 20: Playground catalog previews and Reset honesty
**Goal:** Visitors can browse six in-app demo entries with names, descriptions and visual previews, then Reset a playing scene to its documented initial physics and matching live-control labels.
**Depends on:** Phase 19
**Requirements**: WEB-01, WEB-03
**Gap Closure:** Closes the v1.1 milestone-audit post-gate catalog-preview and Reset→SceneControls label gaps.
**Success Criteria** (what must be TRUE):
  1. The in-app catalog presents Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel with names, short descriptions and visual previews; each still opens the shared player through the existing hash route.
  2. The owner-approved responsive Kobalte / semantic-HTML shell remains; restoring previews must not revive the old card layout that broke narrow widths.
  3. Play, pause and Reset still rebuild the native world to the documented initial state, and live preset selects show that initial value after Reset instead of a stale pendingValue.
  4. Focused Chromium smoke covers visible catalog previews and Reset label honesty for representative live presets.
**Plans:** 5/6 plans executed
**UI hint**: yes

Plans:
- [x] 20-01-PLAN.md — Reset identity helper, empty construction bag, documented initials
- [x] 20-02-PLAN.md — Compact static SVG previews inside DemoNavigation
- [x] 20-03-PLAN.md — Keyed SceneControls remount and Reset bag clear
- [x] 20-04-PLAN.md — Chromium sidebar and drawer preview smoke
- [x] 20-05-PLAN.md — Chromium Reset label honesty and test:player allowlist
- [ ] 20-06-PLAN.md — Player-smoke gate and independent AI review

Keep README gallery WebPs as a documentation gallery, not a substitute for in-app previews. Do not treat Dam Break headless speed versus C++ as this phase's work.

### Phase 21: Playground leftover cleanup
**Goal:** Remove dead playground chrome and proof leftovers, and bring the three oversized WASM scene files under the Bright Builds file-length gate without changing scene behavior.
**Depends on:** Phase 20
**Requirements**: none — leftover cleanup with no milestone requirement reassignment
**Gap Closure:** Closes the v1.1 milestone-audit unused-proof, dead-FallbackPanel, and scene file-length leftovers.
**Success Criteria** (what must be TRUE):
  1. `loadProofSession` is removed or used; the opt-in `rust-wasm-proof.spec.ts` is either folded into ordinary documented smoke or kept as an explicit opt-in with no unused helper.
  2. Dead `FallbackPanel` empty/not-ready branches are removed or made reachable from a real unknown/empty hash path.
  3. `color_mixer.rs`, `dam_break.rs` and `water_wheel.rs` satisfy Bright Builds `file-lengths` without changing public scene behavior, controls, or particle recipes.
  4. Playground Dam Break headless speed versus pinned C++ remains out of this phase and out of v1.1 definition of done.
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 21 to break down)

## Progress

Execution order: **16 → 17 → 18 → 19 → 20 → 21**. Phases 16–19 remain complete. Gap-closure phases restore post-audit UI fidelity and leftover cleanup before re-audit.

| Phase | Plans Complete | Status | Completed |
| --- | --- | --- | --- |
| 16. Rust WASM Browser Bridge | 4/4 | Complete    | 2026-09-17 |
| 17. Shared Player and Early Pages Delivery | 8/8 | Complete    | 2026-09-17 |
| 18. Six Native Physics Demos | 10/10 | Complete    | 2026-09-18 |
| 19. Interaction Polish and Browser Verification | 7/7 | Complete    | 2026-09-19 |
| 20. Playground catalog previews and Reset honesty | 5/6 | In Progress|  |
| 21. Playground leftover cleanup | 0/0 | Not started | |

## Coverage and Planning Basis

All **22/22 requirements** map to exactly one phase: Phase 16 has 3, Phase 17 has 6, Phase 18 has 8, Phase 19 has 3 and Phase 20 has 2. Phase 21 is leftover cleanup with no requirement IDs. WEB-01 and WEB-03 are pending gap closure. No orphaned or duplicate assignments. See [REQUIREMENTS.md](REQUIREMENTS.md) for pending traceability, [v1.1-MILESTONE-AUDIT.md](v1.1-MILESTONE-AUDIT.md) for the audit, and [research/v1.1/SUMMARY.md](research/v1.1/SUMMARY.md) for earlier research.

Local hobby scope in AGENTS.md and standards-overrides.md takes precedence over historical certification gates. AGENTS.bright-builds.md, the managed architecture and frontend standards, and the TypeScript/JavaScript guidance inform the thin browser boundary, dark-default SolidJS experience and source/provenance disclosure. Phase 20 UI planning must keep the Kobalte Dialog / semantic HTML shell recorded in standards-overrides.md.
