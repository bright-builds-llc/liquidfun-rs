# Feature Landscape: Browser Example Catalog

**Milestone:** v1.1 SolidJS + Rust/WASM examples
**Researched:** 2026-09-17
**Confidence:** HIGH for existing source capabilities and inspiration; MEDIUM for proposed browser scene feasibility until a WASM prototype runs.
**Scope:** Feature research only. “Liquidjs” means this repository's Rust engine through its own JS/TS interface.

**Scope clarification — 2026-09-17:** The owner subsequently approved Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel, plus the proposed gallery/player, interaction, sharing, attribution and main-push Pages scope. Current PROJECT.md and REQUIREMENTS.md record that approval. Specific control values and visual designs below remain implementation recommendations; scene feasibility still requires browser evidence.

## Recommendation

Build a small, attractive catalog with six authored scenes sharing one player. Make each scene communicate one behavior and invite one simple interaction. Reuse the native engine's capabilities, but do not expose the diagnostic testbed or its evidence machinery as the website experience. A scene should remain enjoyable after its first few seconds, with a clear reset and bounded resource use.

The existing catalog supplies useful capability references, not finished launch demos. Its particle flag scenes create only two particles, step, inspect, and destroy the group/system. Replaying that schedule verbatim would yield an empty or uninteresting page. Author persistent browser scene definitions using the core library; keep diagnostic protocol and oracle dependencies out of the browser package.

This recommendation follows current hobby scope in `AGENTS.md`, `PROJECT-SCOPE.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and `standards/core/frontend-ui.md`: a focused, useful experiment with truthful claims, a dark default, discoverable source, and lightweight checks. It does not revive strict qualification.

## Table Stakes

| Feature | Why it belongs in launch | Complexity | Acceptance direction |
| --- | --- | --- | --- |
| Six-card catalog | Gives visitors a quick choice across visibly different behaviors | Low | Title, short description, static preview, and one clear open action per scene; no need for search with only six cards |
| Shared player | Consistent interaction makes experiments approachable | Medium | Play/pause, reset to a known initial state, readable scene title and concise interaction hint |
| A few scene-specific controls | Makes each example more than a video | Medium | Two or three labeled controls with constrained values; say when a change resets the scene |
| Mouse and touch interaction | Supports common browser use without a complex editor | Medium | At least one meaningful interaction per scene; deliberate pointer capture and canvas coordinates; normal page scrolling outside player |
| Simple shareable selection | Visitors can return to a particular example | Low | Stable scene identifier in a static-host-safe URL; reloading opens the same scene; full simulation-state serialization is unnecessary |
| Loading and recovery states | WASM loading can fail independently of UI | Low | Loading label, useful failure text, retry/reset action; never leave a blank canvas with no explanation |
| Bounded simulation lifecycle | Avoids degrading after scene changes or minutes of use | Medium | Only selected scene runs; old world/resources released; capped particle emission; hidden tabs do not accumulate unbounded simulation debt |
| Responsive, accessible controls | The catalog remains usable across ordinary viewport sizes | Medium | Semantic buttons and labels, keyboard focus, strong contrast, text explaining visual behavior; no color-only control labels |
| Scene source and inspiration links | Makes the examples useful to developers and honest about influence | Low | Link own scene implementation and original inspiration separately; show adapted-source notices when applicable |
| Visible project identity | Required repository conventions without clutter | Low | Stable repository link, version/short commit/build provenance; missing provenance says `Unavailable`; maintainer disclosure consistent with owner guidance |
| Build and deploy on main pushes | Explicit user outcome | Medium | Built site and Rust-produced WASM deploy together; verify opening a scene from the deployed project path |

The upstream testbed includes selection, parameter controls, picking, pause and single stepping; these support a compact shared player rather than a full development environment. [Official testbed guide](https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter02__hello__box2_d.html).

## Recommended Six Scenes

These are product designs based on available engine concepts, not claims that equivalent browser scenes already exist. Controls and visual acceptance are proposed scope. Keep defaults modest and tune particle counts on the first browser vertical slice rather than promising a frame rate now.

| Scene | Hook and interaction | Suggested controls | Existing Rust basis | Feasibility / risk |
| --- | --- | --- | --- | --- |
| **Dam Break** | Release a block of water into a basin; place/drop one obstacle | Water amount preset; gravity preset | Water particles, static fixtures, dynamic bodies, world stepping | Medium effort / lowest launch risk; first vertical slice. Reset restores water and obstacle, rather than continuously allocating |
| **Fountain** | Aim a bounded stream into a bowl | Emission rate; launch speed | Particle creation, initial velocity, lifetime/capacity machinery | Medium effort; prove particle count plateaus and reset removes emitted particles. Emitter is scene logic, not a new solver |
| **Float or Sink** | Drop a few bodies into a pool and watch their response | Body preset or density; drop button | Particle/body contacts and dynamic-body reaction | Medium effort; tune and prove visible bobbing before promising “floating.” Do not add a fake buoyancy animation or a separate fluid solver |
| **Color Mixer** | Stir two colored particle groups together | Mix strength preset; stir speed or pointer stirring | Engine `COLOR_MIXING`, color buffers, contact-driven channel mixing, particle forces | Medium effort; show actual solver colors. Label this particle-color mixing, not physically accurate pigment chemistry |
| **Jelly Drop** | Drop an elastic particle shape onto obstacles, then poke it | Shape preset; softness preset applied on reset | Elastic/spring flags, particle-group creation and topology, rigid fixtures | Medium/high effort; test coherent recovery after deformation. This is elastic particle material, not a general cloth/soft-body editor |
| **Water Wheel** | A jet pushes a small pinned paddle wheel | Jet strength; emission toggle | Dynamic bodies, revolute joints, particle/body coupling | Medium/high effort; compose existing capabilities into a toy with an obvious causal response. Fall back to a simpler pinwheel-and-dropped-balls toy only through an explicit scene-scope revision |

### Inspiration and what to borrow

- Google's official showcase demonstrates dam break, elastic particles, rigid-body displacement, surface tension/color, and a wave machine. Borrow the clear one-behavior-per-example presentation, and create fresh browser layouts and compositions. [LiquidFun showcase](https://google.github.io/liquidfun/).
- The Faucet test provides an upstream example of continuing emission with particle lifetime management. Borrow its bounded-emission idea, not an unreviewed copy of its C++ or legacy JavaScript runtime. [Faucet source](https://github.com/google/liquidfun/blob/master/liquidfun/Box2D/Testbed/Tests/Faucet.h).
- The particle guide explains elastic groups, flags and color data. These are useful vocabulary for the scenes; only features found in the Rust implementation should be exposed. [Particle module guide](https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html).
- Google's LiquidFun Paint presents fluid painting alongside immovable and floating marks. Borrow playful direct manipulation and the idea of a tiny palette; a full drawing/editor product is beyond this milestone. [LiquidFun Paint](https://google.github.io/LiquidFunPaint/).
- Anvaka's Field Play is a primary example of a visual experiment whose URL restores a selected configuration. Borrow easy sharing and immediate experimentation. Its vector-field GPU simulation is a different model; it is not evidence of our fluid engine's speed or accuracy. [Field Play source and explanation](https://github.com/anvaka/fieldplay).

The Google examples are historical primary references, not a recommendation to adopt their old JavaScript build stack. No third-party code/assets were copied for this research. During implementation, record the exact upstream revision/file for any adaptation and preserve applicable notices using the repository's existing attribution process; an inspiration link does not replace a source notice. This research is not a new license-compatibility determination.

## Local Capability Evidence

| Source | Observation | Consequence |
| --- | --- | --- |
| `crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs` | Defines lifecycle, forces, body coupling, pause and solver-flag catalog entries; flag recipes use two particles and include destruction | Good capability checklist, inappropriate direct user-facing playback |
| `crates/liquidfun-test-protocol/src/catalog/scenarios/groups.rs` | Existing group scenarios cover formation and lifecycle/mutation | Elastic scene has a native starting point; browser geometry and duration still need design |
| `crates/liquidfun-test-protocol/src/catalog/scenarios/joints.rs` | Joint catalog is generated over supported joint kinds | Wheel toy can investigate native joint APIs without adding a separate JS physics engine |
| `crates/liquidfun/src/world/particle_object/particle.rs` | Checked creation includes lifetime and capacity handling | Fountain can use engine lifecycle instead of an unbounded JS list |
| `crates/liquidfun/src/world/particle_object/system.rs` | Public snapshots/views, position/velocity editing, forces, impulses and pause methods | A small WASM facade can expose rendering and interaction data |
| `crates/liquidfun/src/world/particle_object/group.rs` | Checked recipe-based group creation and group views | Scene construction can use a public engine boundary |
| `crates/liquidfun/src/particle/solver/material.rs` | `color_mixing` updates channels for contacting particles when both carry the flag | Color mixing is real engine state, not merely transparent point overlap |
| `crates/liquidfun/src/particle/solver/constraints.rs` | Elastic solver operates on stored triad data | Elastic capability exists; stable larger browser compositions are still unproven |
| `crates/liquidfun-testbed/src/ui/scenario_browser.rs` | Existing UI projects stable catalog identity with keyboard/search metadata | Reuse concepts, not Macroquad UI or testbed protocol coupling |

Source inspection establishes presence, not browser performance, WASM compatibility, or scene-level visual correctness. No compile, browser run, or native test was performed by this feature research task.

## Physics Versus Presentation

- All particle motion, collisions, rigid-body response and elastic deformation come from this repository's Rust engine through WASM.
- Particle color mixing may come from the engine's actual color solver. Verify the returned color buffer changes under contact and remains unchanged with mixing disabled before describing the demo that way.
- Glow, outlines, trails, gradients and overlapping alpha circles are rendering choices. They must not be advertised as surface tension, mixing, or higher simulation accuracy.
- Start with clear particle rendering. Smooth liquid shaders are optional polish; photorealism is unnecessary for a compelling catalog.
- Drop/aim/stir inputs and emitters are scene controllers. They may apply engine forces or creation commands, but must not independently animate physics positions in JavaScript to conceal missing behavior.

## Differentiators and Future Brainstorm

| Idea | Value | Scope recommendation |
| --- | --- | --- |
| One short “try this” hint on every scene | Converts a passive visitor into an experimenter | Launch |
| A sentence explaining the observed behavior | Makes the gallery educational without a tutorial course | Launch |
| Visible Rust/WASM identity and scene source | Shows what this particular project contributes | Launch |
| Seed and selected control values in share link | Makes interesting setups easy to reproduce | Later; scene selection alone is sufficient initially |
| **Paint Aquarium** | Draw a few colored fluid strokes among bobbing shapes | Later; strict emission cap, no saved drawing editor |
| **Jelly Obstacle Course** | Guide an elastic blob through bumpers | Later; may need interaction tuning, not new material science |
| **Hourglass** | Compare viscous and powder presets in one striking silhouette | Later; geometry/congestion tuning required |
| **Wave Garden** | Rock a container of fluid and floating shapes | Lower-risk substitute candidate if discussed; requires stable moving geometry |
| **Tiny Waterworks** | Aim a fountain at wheels or gates | Later extension of Water Wheel; avoid mission/editor systems |
| **Orbital Ink** | Swirl colored particles with a controllable force field | Later visual experiment; explicitly label imposed forces |

## Anti-Features

| Avoid in v1.1 | Reason | Instead |
| --- | --- | --- |
| Accounts, backend, saved cloud scenes | Unnecessary service burden | Static site and shareable scene URLs |
| General scene editor or arbitrary user code | Large UI and boundary-validation expansion | Six curated scenes with bounded controls |
| All 43 diagnostic scenarios as gallery cards | Diagnostic coverage is not six polished experiences | Author selected scenes using demonstrated engine capabilities |
| npm/crates.io publication gate | Site consumption does not require a public package release | Build the in-repo WASM package with the site |
| New exhaustive platform/parity program | Conflicts with the accepted hobby scope | Browser smoke checks for the selected user paths |
| 3D water, photorealism, huge particle-count promises | Different rendering/model ambition and unknown budget | Good 2D compositions with measured sensible defaults |
| Multiplayer, recording/export suite, analytics | Scope without a demonstrated launch need | Fast direct interaction and links to source |

## Dependencies and Roadmap Implications

1. Prove one native Rust scene can build to WASM, step and return draw data.
1. Put Dam Break into the shared browser player and deploy that vertical slice at the real GitHub Pages project path.
1. Establish lifecycle, reset, loading/error and bounded-input behavior before multiplying scenes.
1. Add Fountain and Float or Sink, then Color Mixer, Jelly Drop and Water Wheel using the same interface.
1. Check every scene visually through pause, reset, repeated scene switching, representative control changes and sustained emission; test actual deployed links and WASM assets.

Keep the uncertain large-scene behavior visible in planning: floating, elastic stability and paddle coupling need a small visual spike. Do not expand the engine's public surface or promise unsupported physics merely to preserve a catchy scene name.

## Confidence and Remaining Questions

- **HIGH:** Native source contains the relevant particle flags, material/color solver, group, lifecycle and joint foundations; official Google examples establish useful inspiration.
- **MEDIUM:** All six proposed scenes can be made satisfying with modest particle counts; this remains an implementation hypothesis until run in a browser.
- **Unverified:** WASM target compilation, exact facade, initial bundle size, mobile performance, ideal particle cap, and stable Float or Sink/Jelly defaults. These belong to the first implementation spike and stack/architecture research.
- **Product choice:** The six scene names and overall scope are now approved. Exact per-scene controls and tuning remain recommended defaults to refine during planning and implementation.
