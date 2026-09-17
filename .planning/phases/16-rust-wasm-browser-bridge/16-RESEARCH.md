# Phase 16: Rust WASM Browser Bridge - Research

**Researched:** 2026-09-17
**Domain:** Rust `wasm32-unknown-unknown`, `wasm-bindgen`, owned frame transport, SolidJS/Canvas integration, and reproducible mixed Rust/Bun builds
**Confidence:** HIGH for repository boundaries and documented APIs; MEDIUM for integrated browser compatibility until the first generated package is instantiated and stepped

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Deferred Ideas (OUT OF SCOPE)

- Shared player lifecycle, play/pause/reset, loading retry, hidden-tab handling, stable scene URLs, and GitHub Pages deployment — Phase 17.
- Six polished demo definitions, catalog cards, scene controls, source/inspiration chrome, and gallery component-library choices — Phase 18.
- Pointer/touch behavior, responsive/accessibility polish, repeated scene cleanup, production-path smoke checks, and broader browser verification — Phase 19.
- Public npm or crates.io publication, a stable JavaScript API, workers, WASM threads, zero-copy memory views, WebGPU, SSR, and broad browser/platform guarantees — outside v1.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| WASM-01 | A visitor can run and visibly step a particle/rigid-body scene using this repository's Rust engine compiled to WebAssembly, without a C++ runtime or external JavaScript physics engine. | Use the public `World`, rigid fixture, particle-system, particle-view, and ordinary `World::step` APIs through a private WASM session; verify generated WASM in Chromium and compare consecutive Rust frames. [VERIFIED: `.planning/REQUIREMENTS.md`; local source inspection] |
| WASM-02 | A contributor can reproducibly build the local JS/TypeScript WASM package and frontend from a clean checkout with documented commands and pinned tool inputs; ordinary native Cargo consumers remain independent of browser tooling. | Preserve `default-members = ["crates/liquidfun"]`; pin Rust, `wasm-bindgen`, `wasm-pack`, Bun, direct npm packages, and the Bun lockfile; generate ignored bindings before typecheck/build through one repo command. [VERIFIED: `.planning/REQUIREMENTS.md`; `Cargo.toml`; official package registries and release APIs] |
| WASM-03 | The browser renderer receives bulk owned frame data through a typed interface without per-particle JS/Rust calls or exposed raw engine pointers. | Return boxed numeric slices from one captured frame object; wasm-bindgen documents that these become copied JavaScript typed arrays, while the engine's particle view supplies aligned borrow-scoped lanes. [VERIFIED: `.planning/REQUIREMENTS.md`; `crates/liquidfun/src/particle/view.rs`; wasm-bindgen boxed-number-slice guide] |
</phase_requirements>

## Summary

Implement this phase as a thin vertical slice with three boundaries: a native-testable Rust session core, a minimal wasm-bindgen export shell, and a SolidJS/Canvas proof that consumes validated copied arrays. The existing public engine already has the required construction, unprofiled stepping, body snapshots, particle views, colors, and checked configuration APIs; no `liquidfun` public API change is justified. [VERIFIED: `crates/liquidfun/src/lib.rs`; `world/step/execution.rs`; `world/particle_object/system.rs`; `particle/view.rs`]

Use one argument-free proof-scene constructor and one bounded fixed-step operation such as `advance(step_count)` where `step_count` is restricted to `1..=4`. Capture one coherent Rust frame after stepping, return five typed arrays, render it on Canvas 2D, and free both the frame wrapper and session explicitly. This gives the planner focused seams for native tests, pure TypeScript tests, and a browser integration test without implementing Phase 17's player lifecycle. [VERIFIED: locked decisions D-01 through D-15; wasm-bindgen exported-type and boxed-slice guides]

The largest execution-time validation risk is integrated compatibility: the local machine has Rust 1.97.0 but not the WASM target, has wasm-pack 0.13.1 instead of the selected 0.15.0, and has Bun 1.3.14 instead of the selected 1.4.2. Plan 16-01 therefore prepares the exact target/tool and proves a real `--target web` build; Plan 16-04 performs the first browser instantiation, step, capture, and visible-Canvas proof after the generated loader, renderer, and app entrypoint exist. [VERIFIED: local version probes on 2026-09-17]

**Primary recommendation:** Plan four ordered slices: (1) private session/frame crate plus native tests and a real WASM build, (2) pinned Bun/Solid/Vite configuration, reproducible generation, and the typed frame/session boundary, (3) pure Canvas rendering plus the approved app entrypoint, and (4) Chromium proof, retained successful Canvas evidence, documentation, native-isolation gates, and independent review. [VERIFIED: phase success criteria and repository verification conventions]

## Project Constraints

- `.cursor/rules/`, `.cursor/skills/`, and `.agents/skills/` are absent, so there are no additional repo-local rule or skill files beyond the supplied root instructions. [VERIFIED: repository glob]
- `.planning/**` is parser-owned and must not be formatted with mdformat. [VERIFIED: `AGENTS.md` Repo-Local Guidance]
- Production source forbids unsafe code, public APIs require documentation, `unwrap()` is prohibited, optional internal names use `maybe_`, and new multi-file Rust modules use `foo.rs` plus `foo/`. [VERIFIED: workspace lints; `AGENTS.md`; `standards/languages/rust.md`]
- Pure Rust and TypeScript logic requires focused Arrange/Act/Assert unit tests. [VERIFIED: `standards/core/testing.md`]
- The frontend must use SolidJS, Bun, a dark default, semantic HTML, and no component library in this phase. The locked no-library decision is the documented phase-scoped exception to the managed MysticUI default. [VERIFIED: D-02, D-09, D-11; `standards/core/frontend-ui.md`; `standards/languages/typescript-javascript.md`]
- Ordinary checks remain proportional under hobby scope; strict Linux qualification, C++, sanitizers, fuzzing, coverage, and benchmark evidence are not Phase 16 completion gates. [VERIFIED: `PROJECT-SCOPE.md`; `standards-overrides.md`]
- An independent human or separate AI reviewer must inspect the relevant diff/evidence and bind acknowledgment to the exact digest, identity, and time; the implementing agent cannot approve its own work. [VERIFIED: `AGENTS.md` Independent review]

## Existing Integration Evidence

| Surface | Finding | Planning consequence |
| --- | --- | --- |
| Root workspace | `liquidfun` is the sole default member; private tools are explicit workspace members. [VERIFIED: `Cargo.toml`] | Add `crates/liquidfun-wasm` to `members` only and leave `default-members` unchanged. |
| Publishable engine | `liquidfun` has no browser dependency and only `bitflags` at runtime. [VERIFIED: `crates/liquidfun/Cargo.toml`] | The wrapper depends outward on `liquidfun`; the engine must not depend on wasm-bindgen or `web/`. |
| World construction | `World::new` creates an empty world whose default gravity is `Vec2::ZERO`; `set_gravity` validates finite coordinates. [VERIFIED: `world/object/body_object.rs`; `world/config.rs`] | The proof-scene builder must explicitly set downward gravity before stepping. |
| Rigid scene | Checked body/fixture APIs support static and dynamic bodies, polygon boxes, circles, snapshots, and owned shape definitions. [VERIFIED: `world/body.rs`; `world/fixture.rs`; `collision/shape/polygon.rs`; `collision/shape/circle.rs`] | Build a three-wall basin from static box fixtures and one dynamic circle obstacle; retain only private IDs/descriptors needed to render them. |
| Particle scene | A checked system definition controls radius and maximum count; particles can be created with positions, colors, flags, and lifetimes. [VERIFIED: `particle/definition/system_definition.rs`; `particle/definition/particle.rs`; `world/particle_object/particle.rs`] | Use one bounded system and create a modest source-ordered grid of colored water particles in Rust. |
| Particle frame | `ParticleSystemView` returns aligned borrow-scoped positions, optional colors, identities, and other semantic lanes. [VERIFIED: `particle/view.rs`] | Flatten positions and colors while the immutable borrow is active, then return owned arrays after the borrow ends. |
| Stepping | `World::step` uses a disabled profiler; only `step_profiled` enables `Instant::now()`. [VERIFIED: `world/step/execution.rs`; `world/observation/profile.rs`] | Call ordinary `step` with a fixed configuration; do not call or compile against diagnostic profiling from the wrapper. |
| Debug geometry | Public debug collection is renderer-neutral and bounded, but every call first gathers a reviewed full world observation. [VERIFIED: `debug_draw/collector.rs`] | Do not collect full debug primitives per frame. Transform a tiny private list of known basin segments and circle descriptors from public body snapshots. |
| WASM target | `wasm32-unknown-unknown` has `core`, `alloc`, partial `std`, default aborting panics, and no working filesystem/thread spawn; rustc reports 64-bit atomic support for this target. [CITED: https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html] [VERIFIED: `rustc --print cfg --target wasm32-unknown-unknown`] | Keep OS APIs, profiled clocks, C++, process orchestration, and panic-recovery claims outside the runtime. Compile and run the actual target before assuming compatibility. |

## Standard Stack

### Core and Build Tools

| Tool/library | Pin | Purpose | Evidence |
| --- | --- | --- | --- |
| Rust | `1.97.0` | Existing workspace compiler and WASM cross-compiler | Already pinned by `rust-toolchain.toml`. [VERIFIED: repository file and local `rustc --version`] |
| Target | `wasm32-unknown-unknown` from Rust 1.97.0 | Minimal browser WASM target | Official rustc target docs prescribe `rustup target add wasm32-unknown-unknown`. [CITED: https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html] |
| `wasm-bindgen` | `=0.2.128` | Exported session/frame classes, generated JS and declarations, typed-array conversion | Published 2026-09-05; Rust MSRV 1.77. [VERIFIED: crates.io metadata; https://github.com/wasm-bindgen/wasm-bindgen/releases/tag/0.2.128] |
| `wasm-pack` | `0.15.0` | Build the local `--target web` package | Published 2026-05-15; tagged docs support `--out-dir`, release builds, and `--target web`. [VERIFIED: GitHub release API; tagged `docs/src/commands/build.md`] |
| Bun | `1.4.2` | Install, lockfile, and routine frontend scripts | Published 2026-09-05; official docs support installing a specific `bun-vX.Y.Z` tag and committing `bun.lock`. [CITED: https://bun.com/docs/installation; https://bun.com/docs/pm/lockfile] |

### Frontend and Test Packages

All direct npm versions should be exact strings without caret or tilde ranges, and `web/bun.lock` should be committed. [VERIFIED: D-09 and D-10; Bun lockfile docs]

| Package | Pin | Purpose | Registry verification |
| --- | --- | --- | --- |
| `solid-js` | `1.9.15` | Minimal reactive app shell | Published 2026-08-17. [VERIFIED: npm registry] |
| `vite` | `8.3.0` | Development/production static bundling and WASM asset URL handling | Published 2026-09-10; Node engine is `^20.19.0 || >=22.12.0`. [VERIFIED: npm registry] |
| `vite-plugin-solid` | `2.11.14` | Solid JSX transform for Vite | Published 2026-07-27; peer range includes Solid `^1.7.2` and Vite 8. [VERIFIED: npm registry] |
| `typescript` | `7.0.2` | Explicit frontend typecheck | Published 2026-07-08. [VERIFIED: npm registry] |
| `vitest` | `5.0.1` | Pure frame-contract and camera tests | Published 2026-09-15; supports Vite 8 and requires Node `^22.12.0 || ^24.0.0 || >=26.0.0`. [VERIFIED: npm registry; https://vitest.dev/guide/] |
| `@playwright/test` | `1.63.0` | Focused Chromium smoke | Published 2026-09-04 and depends on matching `playwright` 1.63.0. [VERIFIED: npm registry] |
| `@types/node` | `22.20.3` | Config/script Node globals used by Vite/Vitest/Playwright types | Current npm `latest` on 2026-09-17 and accepted by Vitest's peer range. [VERIFIED: npm registry] |

No DOM test environment package, component library, router, state library, WASM plugin, renderer package, or JavaScript physics package is needed for Phase 16. Pure Vitest tests can run in the default Node environment, and Canvas behavior belongs in pure projection helpers plus one browser smoke. [VERIFIED: D-11 through D-15; Vitest guide]

### Contributor Preparation

The documented clean-checkout path should be explicit and fail fast on the exact versions:

```bash
curl -fsSL https://bun.com/install | bash -s "bun-v1.4.2"
rustup target add wasm32-unknown-unknown --toolchain 1.97.0
cargo install wasm-pack --version 0.15.0 --locked
cd web && bun install --frozen-lockfile
cd .. && just web-build
```

The Bun installation syntax is documented for exact historical tags; the rustup command comes from the target guide; the wasm-pack install pin follows the repository's existing exact `cargo install --version ... --locked` convention. [CITED: https://bun.com/docs/installation; https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html] [VERIFIED: `.github/workflows/ci.yml` tool-install pattern]

## Architecture Patterns

### Recommended File Structure

```text
crates/
└── liquidfun-wasm/
    ├── Cargo.toml
    └── src/
        ├── lib.rs              # wasm-bindgen export shell only
        ├── frame.rs            # owned frame model, bounds, and packing
        ├── scene.rs            # proof scene construction and private identities
        └── session.rs          # native-testable world ownership and fixed stepping
web/
├── package.json
├── bun.lock
├── tsconfig.json
├── vite.config.ts
├── vitest.config.ts           # only if Vite config cannot cleanly hold test options
├── playwright.config.ts
├── index.html
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── app.css
│   ├── generated/
│   │   └── liquidfun-wasm/    # ignored wasm-pack output
│   ├── physics/
│   │   ├── frame.ts           # runtime shape parser and RenderFrame type
│   │   ├── loader.ts          # generated init + explicit WASM URL
│   │   └── session.ts         # idempotent TS owner around generated classes
│   └── render/
│       ├── camera.ts           # pure world-to-canvas transform
│       └── canvas.ts           # imperative Canvas 2D drawing
├── tests/
│   ├── frame.test.ts
│   └── camera.test.ts
└── e2e/
    └── rust-wasm-proof.spec.ts
scripts/
└── web-build.ts               # version checks, regeneration, typecheck, tests, build
```

This structure applies the repository's `foo.rs` plus `foo/` Rust convention, keeps pure frame/camera logic out of framework components, and leaves generated output in an ignored app-local directory that Vite can import without crossing its root. [VERIFIED: `standards/languages/rust.md`; `standards/core/architecture.md`; Vite asset docs]

### Pattern 1: Native Core, Thin WASM Shell

`SessionCore` should own `World`, `ParticleSystemId`, the dynamic obstacle `BodyId`, private rigid descriptors, fixed `StepConfiguration`, `StepLimits`, and a checked `u32` step counter. `ProofSession` should be the only `#[wasm_bindgen]` session type and should delegate construction, advancement, and frame capture to `SessionCore`. [VERIFIED: D-04, D-08; local public APIs]

Keep error creation at the export edge: internal operations return an ordinary Rust error or bounded message, and the wasm shell maps it to `JsError`. wasm-bindgen exports `Result<T, E>` as `T` or a JavaScript exception when `E: Into<JsValue>`. [CITED: https://wasm-bindgen.github.io/wasm-bindgen/reference/types/result.html]

### Pattern 2: One Coherent Owned Frame

`SessionCore::capture_frame()` should borrow the particle system once, validate aligned lane lengths, copy all particle lanes, release the borrow, then snapshot the small known rigid set. It should return one immutable `FrameData` value. [VERIFIED: D-05 and D-06; `ParticleSystemView` borrow contract]

Export `ProofFrame` as a generated class with methods that return boxed number slices. Boxed `f32` and `u8` slices are copied from WASM linear memory into JavaScript `Float32Array` and `Uint8Array`, so the renderer never holds a view into mutable WASM memory. [CITED: https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html]

Recommended frame ABI:

| Method | JS type | Stride | Invariant |
| --- | --- | --- | --- |
| `particle_positions()` | `Float32Array` | 2: `x, y` | Length is `particle_count * 2`; all values finite. |
| `particle_colors()` | `Uint8Array` | 4: `r, g, b, a` | Length is `particle_count * 4`; every proof particle has an explicit color. |
| `particle_radii()` | `Float32Array` | 1 | Length is `particle_count`; every radius is finite and positive. |
| `rigid_segments()` | `Float32Array` | 4: `x1, y1, x2, y2` | At most 16 segments; all values finite. |
| `rigid_circles()` | `Float32Array` | 3: `cx, cy, radius` | At most 8 circles; centers finite and radii positive. |

Five arrays are a small constant number, support Canvas directly, avoid a generic command decoder, and cover the proof's basin and dynamic circle without extending the engine API. The exact layout is a discretionary recommendation, not an existing compatibility contract. [VERIFIED: D-05 and Claude's Discretion]

Generated Rust classes include a `free(): void` method in their TypeScript declarations; that method deallocates the exported Rust allocation. The TS adapter should call `frame.free()` in `finally` after copying references to the already-owned JS typed arrays, and its idempotent `dispose()` should call `session.free()` exactly once. [CITED: https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/skip_typescript.html; https://wasm-bindgen.github.io/wasm-bindgen/contributing/design/exporting-rust-struct.html]

### Pattern 3: Scene-Owned Rigid Projection

Create three static box fixtures for the basin and one dynamic circle fixture. Store renderer descriptors alongside the private engine IDs. For each frame, static wall segments are constants and the circle center comes from `World::body_snapshot(dynamic_body).transform()` applied to its local center. [VERIFIED: body snapshot and shape APIs; D-06]

This is preferable to calling `collect_debug_primitives` because that public collector always starts from `world_observation(WorldObservationLimits::reviewed())`, even when only shape drawing is requested. [VERIFIED: `debug_draw/collector.rs`]

### Pattern 4: Generated Web Package as Build Output

Use `wasm-pack build crates/liquidfun-wasm --target web --release` with an explicit app-local output directory and stable output name. `--target web` emits browser-loadable ES modules that require explicit initialization. [CITED: https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html] [VERIFIED: wasm-pack 0.15 tagged build docs]

Import the generated initializer and pass a Vite-resolved `*_bg.wasm?url`. Vite documents explicit `?url` imports for access to the compiled module URL, while wasm-bindgen's generated initializer remains responsible for its own imports and glue. [CITED: https://vite.dev/guide/features.html#webassembly]

`scripts/web-build.ts` should:

1. Verify exact Bun, rustc, and wasm-pack versions with actionable errors.
1. Remove only the canonical ignored generated directory after validating its path.
1. Run wasm-pack for the current checkout.
1. Run `bun run typecheck`, `bun run test:unit`, and the Vite production build from `web/`.
1. Emit concise breadcrumbs and a summary under a gitignored `target/web-build/` path.

This keeps foreign-language build logic out of YAML/strings and makes the mixed build rerunnable and diagnosable. [VERIFIED: D-10; `standards/core/code-shape.md` script rules]

### Pattern 5: Thin Solid Imperative Shell

Solid owns only loading/running/failure/disposed status and coarse counts. Canvas drawing should be an imperative function receiving an already-validated `RenderFrame`; particles must not become per-particle signals or components. [VERIFIED: D-02; `standards/core/architecture.md`]

The minimal proof may run one fixed Rust step per `requestAnimationFrame` because Phase 17 owns elapsed-time accumulation, hidden-tab recovery, and the reusable player clock. It must not pass browser elapsed time into the engine or script particle positions in JavaScript. [VERIFIED: D-03 and D-15]

## Proof Scene Specification

Use a compact argument-free `ProofScene::new()` with these characteristics:

- Explicit gravity near `(0, -10)` because the engine defaults to zero gravity. [VERIFIED: `world/config.rs`]
- Three static rectangular fixtures forming a floor and two walls, plus one positive-density dynamic circle placed above or within the particle mass. [VERIFIED: public body, fixture, polygon, and circle APIs]
- One particle system with explicit positive radius, a hard maximum no larger than 512, and source-ordered colored water particles. [VERIFIED: `ParticleSystemDef` and `ParticleDef` APIs]
- A modest initial grid around 12 by 16 particles, with spacing derived from radius and the repository's particle stride rather than arbitrary overlap. Whether this exact count is visually sufficient remains to be measured. [ASSUMED]
- Fixed engine stepping at `1.0 / 60.0`, velocity iterations 8, position iterations 3, and explicit particle iterations 2, using `NoDecisionHook` and `StepLimits::default()`. These values match established repository examples but must be verified for this scene. [VERIFIED: catalog and engine tests] [ASSUMED]

The scene-construction test should assert gravity, one particle system, the exact bounded particle count, explicit color/radius lanes, expected bodies/fixtures, and a finite first frame. The movement test should capture a frame, advance several fixed steps, capture another, and assert at least one particle or the dynamic circle changes by a meaningful finite amount. [VERIFIED: D-01 and D-12]

## Browser Proof Contract

The app should expose visible text states for loading, running, failure, and disposed, plus a concise “Rust engine · WebAssembly” identity and particle/rigid counts. It should use a semantic heading, `<canvas>`, status output, and one diagnostic “Dispose session” button; this button proves explicit disposal without implementing Phase 17 play/pause/reset behavior. [VERIFIED: D-02, D-04, D-13, D-15]

The focused Chromium smoke should prove all of the following in one test:

1. The built page reaches “running” only after awaiting the generated wasm-bindgen initializer and constructing `ProofSession`.
1. A first validated frame reports nonzero particles and rigid geometry.
1. A later Rust step index is greater and at least one value from consecutive Rust frame arrays changed.
1. Canvas pixels or screenshots differ after changed frame data is drawn, proving visible rendering rather than only hidden numerical movement.
1. The page identifies the runtime as Rust/WASM.
1. Clicking “Dispose session” invokes the TS owner's explicit `free()`, stops further stepping, and reaches “disposed”.

Playwright's `webServer` option can start the built preview server and wait for its URL; configure one Chromium project only. [CITED: https://playwright.dev/docs/test-webserver]

Do not make the test depend only on animation timing. Expose coarse DOM attributes derived from the actual validated Rust frame, such as step index and a monotonically updated moved-frame count, then poll them; use a Canvas screenshot difference as the visible-render assertion. These observables are proof instrumentation, not a public browser API. [VERIFIED: D-01 and D-13]

## Build and Isolation Gates

The planner should require these gates:

| Gate | Command direction | Purpose |
| --- | --- | --- |
| Wrapper native tests | `cargo test -p liquidfun-wasm` | Scene construction, bounds, frame packing, movement, and error mapping core. |
| Wrapper lint/build | `cargo clippy -p liquidfun-wasm --all-targets -- -D warnings`; `cargo build -p liquidfun-wasm` | Prove the `rlib` path remains native-testable and warning-clean. |
| WASM generation | `just web-wasm` or the generation stage inside `just web-build` | Compile the exact wrapper with wasm-pack 0.15.0 and `--target web`. |
| Frontend unit/type/build | `bun run test:unit`; `bun run typecheck`; `bun run build:app` through `just web-build` | Validate frame shape, camera projection, generated declarations, and production bundle. |
| Browser smoke | `just web-smoke` | Build first, launch one local preview, run one Chromium project, and dispose explicitly. |
| Native default isolation | `cargo build -p liquidfun`; `cargo test -p liquidfun`; plain `cargo build` | Demonstrate no browser tool, generated package, C++, or submodule is needed for ordinary native use. |
| Package isolation | `cargo xtask package verify` | Demonstrate the published engine package remains independent of the private wrapper and `web/`. |
| Repository standards | `bun scripts/bright-builds-check.ts all` | Enforce managed code-shape and lesson checks. |
| Aggregate repo check | `just check` | Preserve existing package/protocol/docs/provenance checks in Cargo-only mode where applicable. |

These commands follow current root `Justfile`, `cargo xtask check`, package verification, and managed-check conventions. [VERIFIED: `Justfile`; `tools/xtask/src/main.rs`; `standards/core/verification.md`]

Because `workflow.nyquist_validation` is `false`, no formal GSD Validation Architecture/Wave 0 contract is required. The implementation plan should still create test infrastructure in the same task that introduces each pure contract, rather than deferring all testing to the browser plan. [VERIFIED: `.planning/config.json`; D-12]

## Environment Availability

| Dependency | Required by | Available | Observed version | Required action/fallback |
| --- | --- | --- | --- | --- |
| Rust | Wrapper and WASM build | Yes | 1.97.0 | None. [VERIFIED: local probe] |
| `wasm32-unknown-unknown` | WASM build | No | Not installed | Install through rustup for toolchain 1.97.0 before the first WASM checkpoint. [VERIFIED: local `rustup target list --installed`] |
| wasm-pack | Generated package | Wrong version | 0.13.1 | Install exact 0.15.0; do not silently accept 0.13.1. [VERIFIED: local probe] |
| Bun | JS install/scripts | Wrong version | 1.3.14 | Install exact 1.4.2 using the official tagged installer. [VERIFIED: local probe; Bun install docs] |
| Node.js | Vite/Vitest/Playwright runtime compatibility | Yes | 24.13.0 | Meets Vite, Vitest, and Playwright declared engines. [VERIFIED: local probe; npm registry] |
| npm | Registry metadata/debug fallback only | Yes | 11.6.2 | Do not use as the routine lock/install surface. [VERIFIED: local probe; D-09] |
| Just | Repo command facade | Yes | 1.48.0 | Existing local version can run simple recipes; repository research recommends no new Just feature dependency. [VERIFIED: local probe] |
| Chromium | Focused smoke | Browser cache present; matching Playwright revision unverified | Multiple cached headless-shell revisions | Run the pinned Playwright Chromium install for 1.63.0 before smoke if its matching binary is absent. [VERIFIED: local cache inspection; Playwright package pin] |

**Missing or wrong dependencies with no implementation fallback:** WASM target, wasm-pack 0.15.0, and Bun 1.4.2 must be prepared before the complete Phase 16 verification can pass. [VERIFIED: local probes and locked pins]

**Available fallback:** Playwright can install its package-pinned Chromium binary; a broad system-browser matrix is not needed. [VERIFIED: D-13 and Playwright package model]

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
| --- | --- | --- | --- |
| Rust/JS ABI glue | Manual exports, pointers, memory offsets, or custom declaration generation | wasm-bindgen 0.2.128 through wasm-pack 0.15.0 | Official generation supplies JS glue, TypeScript declarations, exported classes, errors, typed-array conversions, and `free()`. [CITED: wasm-bindgen type/deployment guides] |
| WASM loading | Manual import-object reconstruction or raw `WebAssembly.instantiate` around wasm-bindgen output | Generated initializer plus Vite `?url` asset | The generated glue owns required imports; Vite owns the production asset URL. [CITED: wasm-bindgen deployment and Vite WASM docs] |
| Frame transport | One exported getter per particle or direct `WebAssembly.Memory` views | One `ProofFrame` with five boxed numeric slices | Boxed slices copy into owned typed arrays and avoid memory-growth invalidation. [CITED: wasm-bindgen boxed-number-slice guide] |
| Rigid debug pipeline | Generic browser command protocol or full diagnostic checkpoint | Private basin segment/circle descriptors projected from public snapshots | The proof has a fixed bounded scene and does not need every debug primitive. [VERIFIED: D-06; collector source] |
| Physics animation | JavaScript position integration, collision approximation, or another physics library | `liquidfun::World::step` and Rust frame capture | WASM-01 requires this repository's actual engine state. [VERIFIED: `.planning/REQUIREMENTS.md`] |
| UI state/rendering | Per-particle Solid signals/components | Plain typed arrays and imperative Canvas 2D draw calls | Particle data is high-volume frame state, not reactive component state. [VERIFIED: architecture standard and D-05] |
| Browser harness | Ad hoc sleeps and shell-launched browser scripts | `@playwright/test` with `webServer` and one Chromium project | Playwright provides server lifecycle, polling assertions, screenshots, and browser disposal. [CITED: https://playwright.dev/docs/test-webserver] |

**Key insight:** The difficult parts are ownership and build composition, not binary serialization. Generated ABI glue plus copied arrays leaves the repository responsible only for a small semantic frame contract and one explicit owner. [VERIFIED: D-04 through D-10]

## Common Pitfalls

### Pitfall 1: “WASM build passed” is treated as browser proof

**What goes wrong:** Link success does not prove generated initialization, first step, typed-array transfer, drawing, or disposal. [VERIFIED: Phase 16 success criterion 1]

**How to avoid:** Make the first plan stop only after compiling the wrapper to WASM, and make the phase stop only after Chromium observes changed Rust frame values and changed Canvas output. [VERIFIED: D-01 and D-13]

### Pitfall 2: Native profiling or panic recovery enters the browser path

**What goes wrong:** `step_profiled` reaches `Instant::now()`, while the target defaults to panic abort and incomplete OS-backed `std`. [VERIFIED: local step/profile source] [CITED: rustc WASM target docs]

**How to avoid:** Call only `World::step`; map recoverable `Result` failures to JS exceptions; treat any trap as session-fatal and recreate rather than reuse. [VERIFIED: D-03 and D-07]

### Pitfall 3: A frame object or session leaks

**What goes wrong:** Exported Rust structs own allocations until generated `free()` or finalization runs, and one frame is created every animation iteration. [CITED: wasm-bindgen exported-struct design]

**How to avoid:** Use `try/finally` around every frame wrapper and idempotent explicit TS session disposal. The smoke must click the disposal path and observe that stepping stops. [VERIFIED: D-04 and D-13]

### Pitfall 4: Typed arrays are typed but semantically malformed

**What goes wrong:** TypeScript can know `Float32Array` without knowing stride, lane alignment, finite values, or count bounds. [VERIFIED: frame contract design]

**How to avoid:** Parse the five arrays once into a branded `RenderFrame`, rejecting wrong constructors, lengths, non-finite floats, non-positive radii, and counts above hard bounds before Canvas drawing. Unit-test each rejection independently. [VERIFIED: D-07 and D-12]

### Pitfall 5: Full observations dominate the proof frame

**What goes wrong:** `collect_debug_primitives` always collects a reviewed world observation before filtering/rendering layers. [VERIFIED: `debug_draw/collector.rs`]

**How to avoid:** Read particles directly from `ParticleSystemView`; render only known basin segments and a dynamic circle from scene-owned descriptors and body snapshots. [VERIFIED: D-06]

### Pitfall 6: Generated output becomes stale or tracked

**What goes wrong:** Old JS glue, declarations, or WASM can disagree with current Rust source, especially when local package output is reused. [VERIFIED: D-10]

**How to avoid:** Ignore the entire generated directory, clean it safely, regenerate on every repo-owned web build, and fail `git diff --exit-code`/status checks if generated files appear tracked. [VERIFIED: D-10; standard rerunnable-script guidance]

### Pitfall 7: Workspace membership changes ordinary consumption

**What goes wrong:** Adding the wrapper to default members or making `liquidfun` depend on it causes plain Cargo use to resolve/build browser tooling. [VERIFIED: current workspace topology and D-08]

**How to avoid:** Add only workspace membership, keep `liquidfun` as sole default member, mark wrapper `publish = false`, and run package verification without generated web assets. [VERIFIED: D-08 and D-14]

### Pitfall 8: Phase 17 work slips into the proof

**What goes wrong:** A proof page grows a reusable animation clock, routing, reset/retry semantics, hidden-tab handling, or production base-path/deployment logic. [VERIFIED: D-15]

**How to avoid:** Keep one scene, one simple RAF loop, one disposal action, local root Vite base, and one Chromium smoke. Record follow-on needs for Phase 17 instead of solving them here. [VERIFIED: D-15]

## Security Domain

### Applicable ASVS Categories

| ASVS category | Applies | Standard control |
| --- | --- | --- |
| V2 Authentication | No | No accounts or identity boundary exists in this phase. [VERIFIED: v1.1 Out of Scope] |
| V3 Session Management | No (web-auth sense) | `ProofSession` is a local WASM resource owner, not an authenticated user session. [VERIFIED: D-04 and project scope] |
| V4 Access Control | No | The static local proof has no privileged operation or server resource. [VERIFIED: phase boundary] |
| V5 Input Validation | Yes | Rust newtypes/range checks at exports plus TS frame parsing before drawing. [VERIFIED: D-07; architecture standard] |
| V6 Cryptography | No | The phase has no secrets, transport protocol, stored credentials, or cryptographic operation. [VERIFIED: v1.1 scope] |

### Threat Patterns

| Pattern | STRIDE | Mitigation |
| --- | --- | --- |
| Oversized step count or frame lane causes resource exhaustion | Denial of service | Hard maximum step count, particle count, rigid count, checked multiplication, and exact lane-length validation before allocation/use. [VERIFIED: D-07] |
| Non-finite geometry poisons Canvas or engine state | Tampering / denial of service | Reject NaN and infinities in Rust boundary types and TS frame parser; never clamp silently. [VERIFIED: engine checked-constructor policy and D-07] |
| Use-after-free or double-free of generated class | Denial of service | Opaque generated class, one TS owner, idempotent `dispose()`, no raw pointer export, and no method use after disposal. [VERIFIED: D-04 and wasm-bindgen exported-class contract] |
| WASM memory growth invalidates a retained JS view | Tampering / denial of service | Return copied boxed slices only; do not expose memory or zero-copy views. [CITED: wasm-bindgen boxed-slice guide] |
| Dependency drift changes generated glue/build | Supply-chain integrity | Exact direct pins, Cargo.lock, bun.lock, tool-version checks, ignored regenerated output, and same-checkout build. [VERIFIED: D-09 and D-10] |

No network input, HTML injection surface, secret, database, or server exists in Phase 16. Status text must use fixed application strings and error text through normal text nodes, not `innerHTML`. [VERIFIED: phase boundary and semantic DOM plan]

## Likely Plan Decomposition

### Plan 16-01: Native-Testable WASM Session and Frame

1. Add `liquidfun-wasm` as an unpublished non-default workspace member with `cdylib` and `rlib`, exact wasm-bindgen 0.2.128, workspace lints, and only a public path dependency on `liquidfun`. [VERIFIED: D-08]
1. Build `SessionCore`, the bounded basin/particle scene, retained private rigid descriptors, fixed stepping, `FrameData`, and pure frame packing. [VERIFIED: existing APIs and D-04 through D-07]
1. Add focused native tests for checked construction, rejected step bounds, lane alignment/finite data, scene movement, and no full diagnostic dependency. [VERIFIED: D-12]
1. Install/verify the target and wasm-pack pin, then generate a real `--target web` package as an early compatibility gate. [VERIFIED: environment audit and D-09]

### Plan 16-02: Reproducible Frontend Boundary

1. Scaffold `web/` with exact Solid/Vite/TypeScript/Vitest/Playwright pins, `packageManager: "bun@1.4.2"`, and committed `bun.lock`. [VERIFIED: D-09]
1. Add ignored generated/package/build/test paths and a rerunnable `scripts/web-build.ts` plus thin Just recipes that regenerate before typecheck/test/build, but run only the generation mode until app entrypoints exist. [VERIFIED: D-10 and script standards]
1. Implement generated-package loading, `RenderFrame` parsing, and TS session ownership/disposal. [VERIFIED: D-05 and D-07]
1. Add focused Vitest tests for typed-array kind, stride/count/finite validation, and exactly-once disposal using a fake generated owner. [VERIFIED: D-12]

### Plan 16-03: Canvas Rendering and Approved Proof Page

1. Implement pure camera projection and imperative Canvas drawing with focused projection tests. [VERIFIED: D-02, D-05, D-11, D-12]
1. Add the approved minimal dark semantic page and RAF proof loop with visible Rust/WASM identity, counts, errors, and diagnostic disposal. [VERIFIED: D-01, D-02, D-13]
1. Run the first complete frontend typecheck, unit suite, and Vite production build now that tests and app entrypoints exist. [VERIFIED: D-09, D-10, D-12]

### Plan 16-04: Real Browser Proof and Isolation Closure

1. Add one Chromium Playwright smoke against the built preview, proving initialization, changed consecutive Rust frames, visible Canvas change, explicit disposal, and retained successful initial/moving/disposed Canvas PNGs plus machine-readable proof metadata. [VERIFIED: D-13]
1. Document exact clean-checkout preparation/build/smoke commands and generated-output ownership. [VERIFIED: WASM-02 and D-10]
1. Run wrapper, web, browser, native-default, package-isolation, aggregate repo, and managed-standard checks; bind the retained successful Canvas artifacts and metadata into the exact-digest independent review. [VERIFIED: D-14; repository review/verification rules]

This decomposition is intentionally sequential: the wrapper build resolves target risk before frontend work, generated declarations enable the typed adapter, the renderer and app entrypoint make the complete build reachable, and the browser closure validates and retains evidence from the assembled artifact rather than isolated mocks. [VERIFIED: phase goal and integration dependencies]

## Code Examples

### Native-Testable Session Core

```rust
// Sources: local World/particle APIs and wasm-bindgen Result/type guides.
const MAX_ADVANCE_STEPS: u32 = 4;

pub struct SessionCore {
    world: World,
    particle_system: ParticleSystemId,
    dynamic_circle: BodyId,
    particle_radius: f32,
    step_configuration: StepConfiguration,
    step_index: u32,
}

impl SessionCore {
    pub fn advance(&mut self, step_count: u32) -> Result<(), BridgeError> {
        if !(1..=MAX_ADVANCE_STEPS).contains(&step_count) {
            return Err(BridgeError::StepCountOutOfRange);
        }

        for _ in 0..step_count {
            self.world.step(
                self.step_configuration,
                &mut NoDecisionHook,
                StepLimits::default(),
            )?;
            self.step_index = self
                .step_index
                .checked_add(1)
                .ok_or(BridgeError::StepIndexExhausted)?;
        }
        Ok(())
    }
}
```

`World::step`, `NoDecisionHook`, `StepConfiguration`, and `StepLimits` are existing public APIs; the bound and bridge error are recommended private adapter design. [VERIFIED: `crates/liquidfun/src/lib.rs`; D-04 and D-07]

### Owned Typed-Array Export

```rust
// Source: https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html
#[wasm_bindgen]
pub struct ProofFrame {
    particle_positions: Vec<f32>,
    particle_colors: Vec<u8>,
    particle_radii: Vec<f32>,
    rigid_segments: Vec<f32>,
    rigid_circles: Vec<f32>,
}

#[wasm_bindgen]
impl ProofFrame {
    pub fn particle_positions(&self) -> Box<[f32]> {
        self.particle_positions.clone().into_boxed_slice()
    }

    pub fn particle_colors(&self) -> Box<[u8]> {
        self.particle_colors.clone().into_boxed_slice()
    }
}
```

The remaining getters follow the same pattern. This intentionally favors an immutable coherent frame and clear ownership over premature zero-copy optimization. [VERIFIED: D-05; boxed-number-slice guide]

### Explicit TypeScript Ownership

```typescript
// Source: generated wasm-bindgen `free()` contract and project disposal decision.
type GeneratedSession = {
  advance(stepCount: number): void;
  frame(): ProofFrame;
  free(): void;
};

export function createSceneSession(session: GeneratedSession): SceneSession {
  let disposed = false;

  return {
    nextFrame() {
      if (disposed) {
        throw new Error("Rust/WASM session is disposed");
      }

      session.advance(1);
      const frame = session.frame();
      try {
        return parseRenderFrame(frame);
      } finally {
        frame.free();
      }
    },
    dispose() {
      if (disposed) {
        return;
      }
      disposed = true;
      session.free();
    },
  };
}
```

The generated class supplies `free()`; the idempotent wrapper and names are recommended private TypeScript design. [CITED: https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/skip_typescript.html] [VERIFIED: D-04]

## State of the Art

| Avoided approach | Current approach | Impact |
| --- | --- | --- |
| Raw WASM pointers and long-lived memory views | Boxed Rust numeric slices copied into JS typed arrays | Removes memory-growth invalidation from the Phase 16 contract. [CITED: wasm-bindgen boxed-number-slice guide] |
| wasm-bindgen bundler output assumed to work in every bundler | Explicit `--target web` initialization plus Vite-resolved WASM URL | Keeps wasm-bindgen's generated import glue authoritative and makes asset loading explicit. [CITED: wasm-bindgen deployment; Vite WASM docs] |
| Browser step tied to native profiling | Ordinary unprofiled engine step with browser-owned scheduling | Avoids `Instant::now()` in the WASM runtime. [VERIFIED: local step implementation] |
| Full diagnostic observation as render frame | Direct particle views plus scene-owned rigid descriptors | Keeps the frame narrow and proof-specific. [VERIFIED: local collector and view source] |
| Garbage collection as lifecycle | Explicit generated `free()` behind one idempotent TS owner | Makes disposal observable and testable. [CITED: wasm-bindgen exported-struct design] |

## Assumptions Log

| # | Claim | Section | Risk if wrong |
| --- | --- | --- | --- |
| A1 | A roughly 12-by-16 particle grid with the proposed basin/circle composition will be visually clear and responsive. | Proof Scene Specification | Tune particle count, radius, camera bounds, or composition during the first browser spike without changing the bridge architecture. |
| A2 | The established `1/60`, 8/3/2 step settings produce stable visible movement for this exact proof scene. | Proof Scene Specification | Adjust solver iterations or initial geometry after native/browser observation; document changed constants. |
| A3 | wasm-pack 0.15.0 `--target web` output and Vite 8.3.0 `?url` loading integrate without additional plugin configuration. | Generated Web Package | The first generated-package/browser checkpoint may require a narrow loader adjustment; do not add a plugin until evidence requires it. |

## Open Questions (RESOLVED)

1. **Does the existing engine compile and execute its first ordinary step on `wasm32-unknown-unknown` unchanged?**
   - What we know: the engine source search found no filesystem, process, thread, or network use; ordinary stepping disables profiling; rustc exposes 64-bit atomics for the target. [VERIFIED: local source and cfg probes]
   - Resolution: this is an execution validation risk, not an unresolved design choice. Plan 16-01 installs the exact target/tool and requires a real `wasm-pack --target web` build; Plan 16-04 requires Chromium to instantiate the generated package, construct the session, execute ordinary fixed steps, capture frames, and visibly draw them before the phase can pass. No fallback runtime or public-engine widening is authorized.

2. **What exact proof-scene particle count and camera bounds look best?**
   - What we know: the public APIs support bounded counts, explicit radius, colors, rigid fixtures, and particle/body coupling. [VERIFIED: local source]
   - Resolution: use exactly 192 particles under the 512 hard cap and fixed world bounds `(-6, -1)` through `(6, 8)`, matching the approved UI viewport contract. Visual clarity and finite movement are acceptance checks in the retained Chromium proof; a failure triggers a scoped implementation correction and rerun, not an open planning decision or a substitute scene.

3. **Should `ProofFrame` getters clone or consume their Rust vectors?**
   - What we know: either boxed-slice return becomes a copied JS typed array. [CITED: wasm-bindgen boxed-number-slice guide]
   - Resolution: use immutable cloning getters returning boxed slices. The fixed maximum of 512 particles and five bounded lanes makes the extra Rust-side copy acceptable for this proof, while repeatable getters preserve a simple coherent frame contract. Consuming or zero-copy getters are outside Phase 16 unless measured evidence later justifies a separately planned ownership change.

## Sources

### Primary (HIGH confidence)

- Repository files: `Cargo.toml`, `rust-toolchain.toml`, `Justfile`, `crates/liquidfun/Cargo.toml`, public world/step/body/fixture/particle/view/debug-draw source, focused tests, xtask aggregate check, and CI conventions. [VERIFIED: codebase inspection on 2026-09-17]
- https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html — target support, partial `std`, setup, atomics context, and default panic behavior. [CITED]
- https://wasm-bindgen.github.io/wasm-bindgen/reference/types/boxed-number-slices.html — copied typed-array semantics. [CITED]
- https://wasm-bindgen.github.io/wasm-bindgen/reference/types/exported-rust-types.html — generated exported classes. [CITED]
- https://wasm-bindgen.github.io/wasm-bindgen/reference/types/result.html — exported `Result` behavior. [CITED]
- https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-rust-exports/skip_typescript.html — generated declarations and `free()`. [CITED]
- https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html — `--target web` initialization model. [CITED]
- wasm-pack v0.15.0 tagged `docs/src/commands/build.md` — output, profile, target, and directory options. [VERIFIED: official tagged GitHub source]
- https://vite.dev/guide/features.html#webassembly — WASM integration and explicit URL assets. [CITED]
- https://docs.solidjs.com/reference/lifecycle/on-cleanup — later framework cleanup seam; Phase 16 uses explicit owner disposal. [CITED]
- https://vitest.dev/guide/ — test runner setup, Vite compatibility, and Bun invocation guidance. [CITED]
- https://playwright.dev/docs/test-webserver — built-preview browser test orchestration. [CITED]
- https://bun.com/docs/installation and https://bun.com/docs/pm/lockfile — exact Bun installation and committed lockfile. [CITED]
- crates.io, npm registry, and official GitHub release APIs queried 2026-09-17 — exact versions, engine/peer ranges, and publication dates. [VERIFIED]

### Local Guidance Materially Applied

- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and `PROJECT-SCOPE.md` — hobby scope, package isolation, parser-owned planning docs, independent review, and proportional verification. [VERIFIED]
- `standards/core/architecture.md`, `code-shape.md`, `frontend-ui.md`, `testing.md`, and `verification.md` — pure core/thin shell, typed boundaries, rerunnable scripts, dark semantic UI, focused tests, and repo-native gates. [VERIFIED]
- `standards/languages/rust.md` and `typescript-javascript.md` — Rust module/type conventions and SolidJS/Bun defaults. [VERIFIED]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — exact versions and compatibility ranges were rechecked against registries and official releases on 2026-09-17.
- Repository architecture: HIGH — recommendations use inspected public APIs and preserve the current dependency direction.
- Browser/runtime integration: MEDIUM — authoritative docs support each part, but this checkout has not yet built or instantiated the combined artifact.
- Proof-scene tuning: LOW to MEDIUM — engine capabilities are verified, while visual density and exact solver settings need the first real browser spike.
- Pitfalls and security: HIGH for ownership, copied-memory, validation, isolation, and target constraints; MEDIUM for performance symptoms until measured.

**Research date:** 2026-09-17
**Valid until:** 2026-10-17 for the architectural recommendation; recheck package/tool pins immediately before adoption because the frontend stack is fast-moving.

## RESEARCH COMPLETE
