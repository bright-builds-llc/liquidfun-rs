# Pitfalls: v1.1 Rust/WASM Playground

**Researched:** 2026-09-17
**Scope:** Six interactive SolidJS examples, native Rust physics through WASM, GitHub Pages on main pushes.
**Confidence:** HIGH for documented platform behavior; MEDIUM for this integration until a browser build and hosted smoke test run.

## Recommendation

Prove one real particle scene through the complete browser and Pages path before building six scenes. Keep the interface small, frame data copied, particle counts bounded and one simulation active. This addresses the expensive integration failures without adding workers, a browser matrix, Linux engine qualification or strict release certification.

This review used the v1.1 STACK, FEATURES and ARCHITECTURE research, current hobby scope in AGENTS.md and standards-overrides.md, AGENTS.bright-builds.md, and the architecture/frontend/verification standards. Both active lesson files were loaded within budget (12,790 bytes; 4,264 estimated tokens). No tools or targets were installed, no engine build was run, and no repository settings were changed.

## Highest-Impact Pitfalls

### 1. A native build succeeds but the first browser step traps

**Cause:** The minimal WASM target only partially supports OS-backed standard library operations. The existing native profiler uses `Instant::now()`; importing the desktop testbed or differential runner brings unnecessary native assumptions.

**Prevent:** Compile only the core plus a private thin wrapper first, then instantiate and step an actual particle world in a browser. Use ordinary unprofiled stepping; browser timing can measure the outer call. Validate finite inputs and control bounds before engine calls. Treat a WASM trap as a failed session requiring recreation, not successful native panic recovery. Preserve core-only Cargo behavior.

**Detect:** A visible scene must move from Rust-produced frame state; successful linking alone is insufficient. The WASM target was not installed at research time, so compatibility remains unverified.

**Place:** Browser bridge, before gallery polish. **Evidence:** [Rust target limitations and default panic strategy](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html); existing profiler and stepping paths identified in ARCHITECTURE.md.

### 2. Root-local development works while the deployed site is blank

**Cause:** Repository Pages URLs have a project prefix; absolute asset paths lose that prefix. A path-based scene route can also request a server route that the static artifact does not contain.

**Prevent:** Verify the eventual Pages URL, set Vite's project `base` accordingly, and use hash-selected scenes initially. Pass the Vite-resolved WASM URL into generated binding initialization. Preview the production artifact under `/liquidfun-rs/`, then open and refresh a bookmarked scene on the deployed site.

**Detect:** Inspect the actual WASM request status and content type, not just HTTP success for the HTML shell. Streaming instantiation expects `application/wasm`; an HTML error response must produce a helpful loading failure, not a silent empty canvas. Do not assume Pages MIME behavior without checking the delivered response.

**Place:** First deployment alongside the first scene. **Evidence:** [Vite Pages base configuration](https://vite.dev/guide/static-deploy.html#github-pages), [streaming MIME requirement](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/JavaScript_interface/instantiateStreaming_static). Hash routing is the recommended simplification, not an additional backend requirement.

### 3. Stale glue, WASM or caches combine incompatible builds

**Cause:** Generated package output is reused without rebuilding the Rust wrapper, or a constant public WASM filename persists across application revisions.

**Prevent:** Build bindings and the site from one checkout, pin compatible binding tools, and deploy one assembled artifact. Let Vite process the WASM as a referenced asset so its generated filename participates in content hashing. Cache dependencies/build acceleration with relevant lockfile/toolchain keys, not a supposedly final generated package. Avoid adding a service worker/offline cache for launch.

**Detect:** Test a normal reload after deployment and confirm the visible build revision and loaded asset belong together. Check a failed WASM fetch has retry/reload guidance. No fixed Pages cache lifetime is asserted here; inspect actual response headers when diagnosing freshness.

**Place:** Build pipeline and hosted smoke. **Evidence:** [Vite asset graph and production hashing](https://vite.dev/guide/assets.html). Cache policy is an application recommendation, not a measured property of this repository's Pages endpoint.

### 4. Reset and navigation leak worlds, animation loops or frame objects

**Cause:** JavaScript reachability is treated as sufficient Rust lifetime management. Asynchronous initialization finishes after a scene has changed; raw typed-array views outlive WASM memory growth.

**Prevent:** Own one session and one animation loop in the player. On cleanup, cancel the frame, disconnect observers/listeners and explicitly free the session. Discard stale async results. Copy batched frame arrays initially; if frame objects themselves own Rust allocations, dispose them too. Do not expose raw pointers or adopt zero-copy views before measurement justifies their invalidation contract.

**Detect:** Repeatedly switch/reset scenes, including during loading, then confirm only the current scene steps. Track live session/frame ownership rather than requiring WASM linear memory to shrink: allocator reuse can be correct even when its buffer stays large.

**Place:** Shared player before multiplying scenes. **Evidence:** [Solid cleanup lifecycle](https://docs.solidjs.com/reference/lifecycle/on-cleanup), [wasm-bindgen warning about allocation invalidating memory views](https://wasm-bindgen.github.io/wasm-bindgen/examples/webgl.html).

### 5. Frame rate changes the physics or returning to a tab freezes the page

**Cause:** A simulation step per rendered frame runs faster on high-refresh displays; unlimited accumulated time causes catch-up storms. A fountain or six live preview cards can exhaust the frame budget.

**Prevent:** Use a fixed physics timestep, bounded elapsed time and a maximum catch-up count. Pause on hidden tabs and discard elapsed debt on return. Keep previews static, one active simulation, emitter lifetimes and particle/body caps. Start modestly and tune from measured scenes; avoid a promised FPS or device coverage before observing it.

**Detect:** Hide/restore the tab, pause/resume and run the fountain long enough to see its population plateau. Compare elapsed simulation time across representative render rates using the clock helper; this does not require a hardware benchmark campaign.

**Place:** Shared player and emitter scenes. **Evidence:** [requestAnimationFrame refresh-rate and background-tab behavior](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame). Exact timestep/catch-up limits are design choices to validate.

## Other Likely Traps

| Trap | Concrete prevention and check | Phase |
| --- | --- | --- |
| Pointer interaction drifts after resize or on high-DPI screens | Use one world/canvas transform for drawing and hit input; convert current CSS bounds explicitly, keep device pixels out of physics, capture/release pointers and handle cancellation. Resize the view without rebuilding the world. Check dragging after resize and touch input once. | Player |
| A capability test is mistaken for an engaging scene | Existing diagnostic recipes may use two particles and destroy them. Author persistent scenes; visually verify all six, including reset and sustained behavior. Do not animate positions in JS to disguise missing Rust behavior. | Demo authoring |
| Full diagnostic snapshots dominate the frame | The current debug collector invokes full world observation; disabling a rendered layer may not avoid collection. Batch particle positions/colors from public views and measure before extending observation APIs. Keep per-particle data out of reactive signals. | Bridge/player |
| Impressive scene names overpromise behavior | Prove buoyancy, jelly stability and wheel coupling at modest scale. If a scene needs replacement, record the scope choice; do not quietly substitute fake physics. | Six demos |
| Colorful canvas excludes interaction or understanding | Use labeled semantic controls, keyboard focus, concise instructions and non-color-only labels. Preserve page scrolling outside the interactive canvas. | Player/polish |
| Inspiration becomes uncredited source copying | Link inspiration separately from this repo's scene source; record exact source provenance and applicable notices for adaptations. Prefer original simple compositions. | Demo authoring |

Pointer recommendations follow [Pointer Events capture and touch behavior](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events); scene and observation findings are supported by the concrete repository paths in FEATURES.md and ARCHITECTURE.md. Attribution remains the existing project policy; this document makes no new legal compatibility determination.

## Deployment Traps

| Failure | Prevention / evidence to collect |
| --- | --- |
| Workflow exists but Pages is not configured or accessible | Verify repository identity, Pages source and deployment environment during implementation. An observed Pages GET 404 is ambiguous: it does not prove settings are absent or that the current token can configure them. Record the exact setup result before diagnosing a build failure. |
| Deploy runs without the intended artifact or permissions | Deploy needs the successful build, `pages: write`, `id-token: write` and the `github-pages` environment. Keep build permissions read-only and use the standard token instead of adding a PAT. |
| An older run overwrites a newer main build | Use a shared Pages concurrency group and an explicit latest-main policy. GitHub does not guarantee execution ordering; combine concurrency with correct main ref/artifact identity, rather than relying on apparent run order. |
| “Every push” accidentally means only frontend changes | Trigger every main push without path filtering: core Rust changes can change the shipped physics. Document that queued superseded runs may coalesce while latest-main deployment remains the outcome. |
| A delivery task revives strict Linux qualification | Compile WASM in the narrow web build; preserve the existing macOS native baseline and manual heavy checks. A Pages delivery runner is not a new native Linux certification requirement. |

Deployment capability facts: [GitHub custom Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages). Ordering/cancellation facts: [GitHub concurrency semantics](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency). Latest-main and cache handling above are recommendations that still require workflow and hosted verification.

## Roadmap Placement and Remaining Uncertainty

1. **Bridge:** Target installation/build and real browser step; copied frames and explicit disposal. This is the largest unresolved technical gate.
1. **Player plus first deployment:** Bounded clock, loading/retry/cleanup, canvas coordinates, project URL and actual WASM response. Close hosting uncertainty early.
1. **Six demos:** Visible physical behavior, bounded emissions, source credits and controls. Float/Jelly/Wheel tuning is a MEDIUM-confidence feasibility assumption.
1. **Polish:** One representative browser smoke path across all scenes, touch/resize, hidden-tab resume, repeated navigation and production refresh. Add only regressions justified by observed failures.

No public package publication, zero-copy frame protocol, worker framework, cross-origin isolation or exhaustive browser/platform matrix is necessary to answer these risks. A small working hosted vertical slice supplies better evidence than a larger speculative architecture.
