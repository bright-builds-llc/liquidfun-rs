# v1.1 Web Example Gallery — Stack Research

**Researched:** 2026-09-17 (UTC)
**Scope:** New browser gallery and Rust-to-WebAssembly integration only.
**Confidence:** HIGH for documented tool capabilities and observed versions; MEDIUM for their combined fit until the first browser build runs.

## Recommendation

Build a plain SolidJS + TypeScript Vite application in `web/`, backed by a private `crates/liquidfun-wasm` adapter that depends on the existing `liquidfun` crate. Generate a local JavaScript/TypeScript/WASM package during the site build. Publish the resulting static assets through GitHub Pages on every push to `main`; do not publish an npm package or Rust crate as part of this milestone.

Keep physics in Rust. Solid owns catalog navigation and controls; Canvas 2D draws copied simulation snapshots. Start with one single-threaded simulation and bounded particle counts. Avoid SSR, a database, authentication, Emscripten, C++ builds, a second physics library, and a worker/thread infrastructure requirement. These are project recommendations, not claims that the alternatives cannot work.

Guidance consulted: `AGENTS.md` (including hobby scope, independent review and workflow rules), `AGENTS.bright-builds.md`, `standards-overrides.md`, `PROJECT-SCOPE.md`, `.planning/PROJECT.md`, `standards/index.md`, architecture and Rust/TypeScript standards, and both active lesson files. The new website request adds a deployment workflow; it does not revive Linux physics qualification or the old strict release campaign.

## Version baseline

Versions below were verified through official package registry metadata or publisher release APIs on the research date. Pin direct dependencies and commit the app lockfile during implementation; these observations do not establish that the integrated application has been tested.

| Tool | Recommended baseline | Purpose / evidence |
| --- | --- | --- |
| Rust | Existing 1.97.0; Edition 2024 | Preserve `rust-toolchain.toml`; add `wasm32-unknown-unknown` for the web build. Keep native default members unchanged. |
| `liquidfun` | Local workspace path, current source | Sole runtime physics implementation. Its current manifest has only `bitflags` as a production dependency. |
| `wasm-bindgen` | `=0.2.128` | Thin Rust/JS adapter and generated TypeScript declarations. Official release published 2026-09-05. |
| `wasm-pack` | 0.15.0 | Generates the local package; official release published 2026-05-15. Match the generated bindgen tooling to the crate version. |
| SolidJS | 1.9.15 | Current `solid-js` registry release; native DOM/reactive app shell. |
| Vite | 8.3.0 | Static build, development server and asset URL handling. |
| `vite-plugin-solid` | 2.11.14 | Current release declares compatibility with Solid >=1.7.2 and Vite majors 3 through 9. |
| TypeScript | 7.0.2 | Current registry release; explicit typecheck step because transforming TS is not a substitute for type checking. Validate configuration with Solid in the first scaffold. |
| Bun | 1.4.2 | Current release published 2026-09-05; repo-preferred install/script surface, with `bun.lock` and frozen installation in CI. |
| Node.js | Pinned supported 24.x LTS patch if required by tooling | Vite 8.3.0 declares `^20.19.0 || >=22.12.0`. Prefer Bun-native execution first; do not add Node merely as a second script runner. Exact Node patch was not selected in this research. |
| Canvas 2D | Browser API | No renderer dependency for the initial particles, lines and solid shapes. Escalate rendering only after measuring a concrete scene. |

Registry facts: [Solid](https://registry.npmjs.org/solid-js/latest), [Vite](https://registry.npmjs.org/vite/latest), [Solid Vite plugin](https://registry.npmjs.org/vite-plugin-solid/latest), [TypeScript](https://registry.npmjs.org/typescript/latest). Release facts: [wasm-bindgen 0.2.128](https://github.com/wasm-bindgen/wasm-bindgen/releases/tag/0.2.128), [wasm-pack 0.15.0](https://github.com/wasm-bindgen/wasm-pack/releases/tag/v0.15.0), [Bun 1.4.2](https://github.com/oven-sh/bun/releases/tag/bun-v1.4.2). These are HIGH-confidence version observations; recheck pins at adoption if implementation happens substantially later.

## WASM package boundary

Use a `cdylib` wrapper crate with `publish = false`. Add `rlib` only if useful for native adapter tests. Do not add browser dependencies to the native engine or wrap the private differential/testbed stack wholesale. Check the core engine's WASM build first, then expose only the constructors, stepping, interactions and render data needed by selected scenes.

Recommended build shape, to be made into one repo-owned command during implementation:

```text
rustup target add wasm32-unknown-unknown --toolchain 1.97.0
wasm-pack build crates/liquidfun-wasm --target web --release
web: install locked JS dependencies, typecheck, build Vite assets
```

`wasm-pack build` generates WASM, JS, declarations and package metadata without publishing them. Keep generated artifacts ignored and reproducible. If exposing a local package dependency, order package generation before dependency resolution when necessary; otherwise import the generated relative module directly. Do not commit generated binaries to solve an ordering issue. The tagged [wasm-pack 0.15.0 build documentation](https://github.com/wasm-bindgen/wasm-pack/blob/v0.15.0/docs/src/commands/build.md) confirms the output and target options. Its moving documentation labels itself unpublished, so the tagged source was checked as well.

Use `--target web` and initialize the generated glue once with an explicit Vite-resolved `.wasm?url` asset. Call the generated initializer instead of manually reconstructing wasm-bindgen imports. Await initialization inside the app lifecycle and display loading/retry/error state. This combination is a recommended integration requiring a browser spike; it is supported by the documented [web output mode](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html) and [Vite explicit WASM URL imports](https://vite.dev/guide/features.html#webassembly).

Do not claim Vite categorically lacks native WASM imports: current Vite documents ESM integration, while the wasm-bindgen guide still describes its default bundler output more conservatively. Explicit web glue avoids depending on that documentation discrepancy or adding a WASM plugin before one is needed.

For the first adapter, return owned typed arrays/snapshots once per frame, not one JS call per particle and not exposed Rust memory pointers. Retain an explicit disposal path for the world. A memory-view fast path is optional only after measurement and a clear invalidation contract. Keep frame buffers outside Solid's fine-grained per-particle reactivity; reactive state tracks UI controls and coarse diagnostics.

The Rust target supports `std` only partially: filesystem calls fail and thread spawning panics. Default WASM panic behavior aborts the instance. Do not import native file/process orchestration into the wrapper, and do not rely on native catch-unwind behavior for browser recovery. Validate inputs and recreate a failed simulation. These limitations come from the [Rust target documentation](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html). No nightly unwind or multithreading requirement is justified for this milestone.

## UI dependencies

SolidJS is explicit user direction and matches the managed frontend standard. Use its plain Vite setup, not SolidStart's server features. [Solid quick start](https://docs.solidjs.com/quick-start).

For any new component-library adoption, the repo default is MysticUI. The inspected head is `d36017757708ed01ef2b3b47beb14f294726411c` (2026-03-24). Its supported consumer contract is source-only Solid 1.9.8+ with Tailwind **3.x**, not a presumed Tailwind 4 setup. Import the supported setup helper/theme, use class dark mode, and follow its current `skipLibCheck` guidance. Pin `github:pRizz/mystic-ui#<verified-sha>` if adopted, never a floating branch or nonexistent npm fork release. [Pinned README](https://github.com/pRizz/mystic-ui/blob/d36017757708ed01ef2b3b47beb14f294726411c/README.md).

For this small catalog, first assess whether plain semantic controls and scoped CSS suffice. Avoid pulling animation/particle decoration into the simulation surface simply because it is available. If choosing no UI library, record the deliberate minimal-dependency/product-scope decision in local overrides; do not silently disregard the standard. This is a phase-level decision, not an implemented exception.

## GitHub Pages build and deployment

Use a dedicated workflow triggered by every `push` to `main` (no path filter), with an optional manual trigger. A PR may run build/smoke checks without deploy permissions. Generate the WASM package from the same checkout as the frontend, build `web/dist`, and upload that directory as the Pages artifact. Use Actions as the repository Pages source. The expected project-site base is `/liquidfun-rs/`; verify repository Pages configuration before assuming a final URL. Hash-based example navigation avoids requiring a server rewrite for deep links. Test direct bookmarked example loads under the project base.

Keep build permissions read-only; give the deploy job `pages: write` and `id-token: write`, link it to the completed build using `needs`, and use the `github-pages` environment. GitHub documents these requirements in [custom Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages). Do not add a personal access token for ordinary deployment. Pin action references to full reviewed commit SHAs; official sample pages currently show differing friendly action majors, so resolve the actual action release rather than copying a sample blindly.

The existing macOS runner can compile the WASM build. A Pages deployment runner is delivery infrastructure, not Linux engine qualification. Preserve the one-macOS Cargo baseline and keep oracle/sanitizer/coverage/performance suites manual. Build once per source and deploy that artifact. A rapid succession of pushes may reasonably coalesce queued stale deployments; document latest-main behavior instead of promising that every intermediate revision is separately visible. Vite's [Pages deployment guide](https://vite.dev/guide/static-deploy.html#github-pages) documents the project base and static build flow.

## First implementation checks and risks

| Check | Why it belongs early | Confidence |
| --- | --- | --- |
| Compile native core + thin wrapper to WASM | Public API internals may contain target-specific atomics, clocks or panic assumptions; this research did not run a build | MEDIUM until executed |
| Load generated package in browser and visibly step one particle/rigid scene | Native tests do not prove browser instantiation, rendering or ownership cleanup | HIGH necessity |
| Preview production build at `/liquidfun-rs/` and refresh an example URL | Root-local dev success can hide broken asset paths | HIGH |
| Reset/switch scenes repeatedly and release old simulation | Prevent duplicate animation loops and retained WASM worlds | HIGH |
| Pin and validate complete JS toolchain together | Latest versions were independently verified, not integration-tested | MEDIUM fit |
| Verify Pages source/environment and exact deployment URL | Current repo settings were not inspected by this stack researcher | Unverified repository prerequisite |
| Smoke one representative browser rather than start a full browser matrix | Demonstrates the user journey without recreating strict certification overhead | Recommendation |

## Deferred choices

No npm publication, npm naming promise, full LiquidFun JS API emulation, SSR framework, database, physics-server deployment, multithreaded WASM, WebGPU renderer, PWA/offline cache, broad browser warranty or exhaustive native/browser bit-parity requirement. Add only what a concrete gallery example needs. Establish a basic successful hosted simulation before multiplying scenes or polishing animation effects.
