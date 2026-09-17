# Phase 17: Shared Player and Early Pages Delivery - Research

**Researched:** 2026-09-17
**Domain:** SolidJS player lifecycle, hash routing, Vite project-base assets, GitHub Pages Actions delivery
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### First hosted scene and honest catalog

- **D-01:** Host Dam Break as the first working scene. Evolve the Phase 16 basin proof into that named scene so a visitor can play real Rust particle/rigid motion, not a compile-only or canned animation.
- **D-02:** Show the six approved names in a thin catalog/navigation list. Only Dam Break is playable. Label Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel as not ready yet. Do not invent fake physics or silently substitute scenes.
- **D-03:** Do not implement full demo cards, previews, per-scene bounded controls, or per-scene inspiration notices. Those belong to Phase 18 (`WEB-01`, `WEB-04`, `WEB-08`, `DEMO-01` through `DEMO-06`).

### Stable URLs and unknown-scene fallback

- **D-04:** Use hash-based scene navigation so GitHub Pages needs no server rewrite. Canonical first-scene form: `#/scene/dam-break`. Keep scene identifiers stable and lowercase with hyphens.
- **D-05:** Production Vite `base` is `/liquidfun-rs/`. Local development may keep a root base, but production and preview-of-production builds must load JS/WASM under the project subpath. Expected hosted origin is `https://bright-builds-llc.github.io/liquidfun-rs/` until repository Pages settings prove a different URL; record the actual deployed URL and source revision as evidence.
- **D-06:** Reloading a valid scene URL restores that scene. An unknown or empty scene identifier shows a useful catalog/fallback with an explanation and a working path back to Dam Break. Do not leave a blank canvas or a hard 404-looking empty app.

### Shared player chrome

- **D-07:** The shared SolidJS player is the only runtime surface. It presents the current scene title, Canvas 2D viewport, play/pause, reset to the documented initial state, visible loading, and an actionable failure state with retry/reset. Replace Phase 16's "Dispose session" proof control with these product controls.
- **D-08:** Play starts or resumes stepping. Pause freezes the world without disposing it. Reset disposes the current session and recreates Dam Break from its documented initial state. Failure stops the session; Retry recreates it.
- **D-09:** Keep semantic HTML and scoped CSS evolved from the Phase 16 dark proof. Do not adopt MysticUI/Tailwind in this phase; record that as a deliberate thin-slice exception so Pages delivery is not blocked by a new design-system adoption. Revisit gallery-card library choice in Phase 18 if the catalog surface needs it.
- **D-10:** Establish site-level source/build chrome now: GitHub repository link, truthful "free and open source" copy (MIT), version, short commit, and build provenance in normal product chrome; missing provenance fields show `Unavailable`. Mention Peter Ryszkiewicz with an OpenLinks link. Per-scene implementation/inspiration/notice links wait for Phase 18.

### Session teardown, hidden tabs, and bounded stepping

- **D-11:** Own exactly one WASM world and one animation loop. Resetting, changing selection, leaving the player, and Solid cleanup must cancel the pending frame, disconnect observers/listeners, discard stale async loads with a generation token, and `dispose()` the prior session. Only the current session may step.
- **D-12:** Pause stepping while `document.hidden` is true. On resume, clear accumulated catch-up time so a hidden tab does not dump seconds of simulation. Cap accepted wall-clock delta and allow at most four simulation steps per animation callback; do not unbounded-accumulate emission or steps.
- **D-13:** Preserve Phase 16 ownership rules: copied typed arrays, no raw pointers, no per-particle JS/Rust calls, recreate after a trap rather than claiming native catch-unwind recovery. Do not add WASM workers, threads, or zero-copy views.

### GitHub Pages delivery

- **D-14:** Every push to `main` triggers the WASM package and SolidJS production-site build from that same checkout, with no path filters. PRs may run the same build/smoke without deploy permissions. Do not rebuild WASM in a separate deploy job from different inputs.
- **D-15:** Deploy only after required build checks succeed. Use Actions as the Pages source, `pages: write` plus `id-token: write` on the deploy job, and the `github-pages` environment. Do not add a personal access token. Pin Actions by full commit SHA.
- **D-16:** Concurrent or overlapping `main` pushes must not leave an older completed revision as the final site. Superseded queued revisions may coalesce. Record the deployed URL and source revision. A website delivery runner is not Linux native qualification and must not revive oracle/sanitizer/coverage/performance gates.

### Verification boundary

- **D-17:** Prove the hosted/player slice with: production-base asset loading, Dam Break play/pause/reset, unknown-hash fallback, session disposal on reset/leave, hidden-tab no-catch-up, and a recorded Pages URL/revision after a real `main` deployment. Do not require the Phase 19 six-scene pointer/accessibility smoke matrix.
- **D-18:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work.

### Claude's Discretion

- Exact hash parser, catalog markup, button labels, loading copy, and failure wording within the locked URL, honesty, and chrome rules.
- Exact Dam Break particle counts, basin dimensions, and colors as long as reset returns to a documented initial state and motion remains visibly native.
- Exact workflow job names, concurrency group, and SHA-pinned official Pages actions, provided latest-main protection and no-PAT permissions remain.
- Whether local preview of `/liquidfun-rs/` uses Vite preview, a tiny static server, or the existing `just web-*` recipes, as long as production-base loading is actually tested.

### Deferred Ideas (OUT OF SCOPE)

- Full six-card catalog with descriptions, previews, and per-scene controls — Phase 18.
- Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel physics — Phase 18.
- Per-scene source/inspiration/notice pages — Phase 18 (`WEB-08`).
- Pointer/touch interaction, stuck-pointer handling, narrow-width accessibility acceptance, and the all-six browser smoke — Phase 19.
- MysticUI/Tailwind adoption — revisit if Phase 18 catalog cards need a design system.
- WASM workers, threads, zero-copy views, WebGPU, SSR, accounts, npm/crates.io publication — outside v1.1.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WASM-04 | Switching/resetting a scene releases its prior world and animation resources; bounded stepping, emission and hidden-tab handling prevent unbounded catch-up or particle accumulation. | Generation-token loads, `onCleanup` + idempotent `dispose()`, pure clock with max 4 steps/frame and cleared accumulator on `visibilitychange`. Dam Break has no emitter; particle cap 512 already exists. |
| WEB-02 | Share and reload a stable demo URL under the repository GitHub Pages path; unknown scene identifier returns a useful catalog/fallback. | Hash routes `#/scene/{id}` under Vite `base` `/liquidfun-rs/`. Pure hash parser + allowlisted IDs. Confirmed hosted origin `https://bright-builds-llc.github.io/liquidfun-rs/`. |
| WEB-03 | Play, pause and reset the selected demo to its documented initial state. | Product controls replace Dispose. Pause freezes without dispose. Reset/Retry dispose then reconstruct Dam Break. Document Phase 16 basin constants as the initial state. |
| WEB-06 | WASM loading and startup show visible progress/loading, useful failure feedback, and retry/reset instead of a blank canvas. | Keep tagged-union states; add fallback view with no live canvas session. Fixed copy from `17-UI-SPEC.md`. Never `innerHTML` exception text. |
| HOST-01 | Every push to main builds WASM + SolidJS production site from the same checkout, then deploys only after required build checks succeed. | One `pages.yml` with no path filters: build job runs `bun scripts/web-build.ts build`; deploy job `needs` that artifact. Do not wait on Cargo CI. |
| HOST-02 | Pages config + ordinary Actions permissions, no PAT; overlapping pushes cannot leave an older completed deployment as the final site; queued revisions may coalesce. | Pages already `build_type: workflow`. SHA-pinned official actions. Workflow `contents: read`; deploy job `pages: write` + `id-token: write` + `github-pages`. Concurrency group per workflow+ref; `cancel-in-progress: false` on main. |
| HOST-03 | Live Pages site loads JS/WASM and direct demo URLs under the real project base, with recorded deployed URL and source revision. | Vite production `base: "/liquidfun-rs/"`, `?url` WASM import, Playwright `goto` under `/liquidfun-rs/#/scene/dam-break`. Record `page_url` + `github.sha` after first main deploy. |
</phase_requirements>

## Summary

Phase 17 is a product-shell and delivery phase on top of a working Phase 16 WASM bridge. The Rust session already constructs a bounded basin-and-water world (192 particles, 3 basin segments, 1 dynamic circle), copies typed frames, and accepts 1–4 engine steps per `advance`. The planner should evolve that world into the named Dam Break scene, replace the proof shell with the locked playground chrome, and add hash routing plus a production Vite base.

GitHub Pages is already configured as Actions-driven at `https://bright-builds-llc.github.io/liquidfun-rs/`, with a `github-pages` environment that allows only `main`. There is no `pages.yml` and no recorded deployment yet. The delivery work is a SHA-pinned two-job workflow that builds WASM and `web/dist` from one checkout, then deploys that artifact with OIDC. Do not add a PAT, path filters, or Cargo/oracle gates.

**Primary recommendation:** Split player logic into a pure hash parser, a pure step-budget clock, and a one-session Solid shell; set production `base` to `/liquidfun-rs/`; add `.github/workflows/pages.yml` using the verified official Pages action SHAs and main-safe concurrency.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. [VERIFIED: workspace glob]

Honor these repo-local constraints instead:

- Hobby scope: Pages delivery is not Linux native qualification. [CITED: PROJECT-SCOPE.md, AGENTS.md]
- Standing authorization covers ordinary main pushes and workflow inspection; package release remains separately authorized. [CITED: AGENTS.md]
- Implementing agent cannot self-approve; independent AI review is eligible. [CITED: AGENTS.md D-18]
- Record the MysticUI/Tailwind skip in `standards-overrides.md` as the D-09 thin-slice exception. [CITED: 17-CONTEXT.md D-09]
- Follow `17-UI-SPEC.md` for copy, tokens, layout, and control labels. [CITED: 17-UI-SPEC.md]
- `.planning/**` is parser-owned; do not mdformat it. [CITED: AGENTS.md]

## Standard Stack

Keep the Phase 16 pins. Do not add a router, UI library, or Pages PAT action.

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| SolidJS | 1.9.15 | Player, catalog, fallback, footer | Locked framework; current npm latest 2026-09-17 [VERIFIED: npm registry] |
| Vite | 8.3.0 | Dev server, production assets, `base`, `?url` WASM | Current npm latest; project-base hashing [VERIFIED: npm registry] |
| `vite-plugin-solid` | 2.11.14 | Solid JSX transform | Current npm latest [VERIFIED: npm registry] |
| TypeScript | 7.0.2 | Typecheck (`tsc --noEmit`) | Current npm latest; Vite does not typecheck [VERIFIED: npm registry] [CITED: vite.dev/guide/features] |
| Bun | 1.4.2 | Install, scripts, `scripts/web-build.ts` | Already required by `web-build.ts` [VERIFIED: local bun 1.4.2] |
| Rust | 1.97.0 | `liquidfun-wasm` + `wasm32-unknown-unknown` | Repo toolchain pin [VERIFIED: rustc 1.97.0] |
| `wasm-pack` | 0.15.0 | Generate ignored web glue | Already required by `web-build.ts` [VERIFIED: wasm-pack 0.15.0] |
| `liquidfun-wasm` | workspace | One opaque session, Dam Break construction | Existing `ProofSession` / `SessionCore` [VERIFIED: crates/liquidfun-wasm] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | 5.0.1 | Unit tests for hash parser, clock, session | Every player-logic change [VERIFIED: npm registry] |
| Playwright | 1.63.0 | Chromium smoke of production-base player | D-17 hosted/player proofs [VERIFIED: npm registry] |
| Canvas 2D | browser | Draw copied frames | Keep Phase 16 renderer |
| Page Visibility API | browser | Hidden-tab pause | D-12 |
| Official Pages actions | see SHAs below | Configure, upload, deploy | HOST-01/02 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Tiny hash parser | `@solidjs/router` | Stronger routing, but invites history-mode paths that 404 on Pages. Locked out. |
| Semantic CSS | MysticUI + Tailwind 3 | Standard default, but D-09 blocks it this phase. |
| Vite Pages sample `cancel-in-progress: true` | Official Pages starter `false` | Vite sample is faster-latest but can strand a mid-flight Pages deploy. Use official `false` on main. |
| `workflow_run` after Cargo CI | Same-workflow `needs: build` | Waiting on native CI couples web delivery to qualification and can deploy a different checkout. |
| PAT / `peaceiris/actions-gh-pages` | Official OIDC Pages actions | PAT is forbidden; environment + `id-token: write` is the documented path. |

**Installation:** no new runtime packages. Record the D-09 exception in `standards-overrides.md`. Add only workflow + source files.

**Version verification:** `npm view` on 2026-09-17 matched every `web/package.json` pin exactly. Do not bump versions in this phase.

### Verified Pages action pins

Use these full SHAs (resolved from current release tags on 2026-09-17) and comment the friendly major like `ci.yml`:

| Action | Tag | SHA | Purpose |
|--------|-----|-----|---------|
| `actions/checkout` | v7.0.0 | `9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` | Already used in `ci.yml` [VERIFIED: gh api] |
| `actions/configure-pages` | v6.0.0 | `45bfe0192ca1faeb007ade9deae92b16b8254a0d` | Pages metadata [VERIFIED: gh api] |
| `actions/upload-pages-artifact` | v5.0.0 | `fc324d3547104276b827a68afc52ff2a11cc49c9` | Package `web/dist` [VERIFIED: gh api] |
| `actions/deploy-pages` | v5.0.1 | `368f82528645a54fb793d4d04e342629a3f51346` | OIDC deploy [VERIFIED: gh api] |
| `oven-sh/setup-bun` | v2.2.0 | `0c5077e51419868618aeaa5fe8019c62421857d6` | Install Bun 1.4.2 [VERIFIED: gh api] |

Do not copy `bright-builds-checks.yml`'s floating `oven-sh/setup-bun@v2` or its Bun 1.3.9 pin. [VERIFIED: .github/workflows/bright-builds-checks.yml]

## Architecture Patterns

### Recommended Project Structure

```text
web/src/
├── App.tsx                 # Imperative shell: one session, one loop, generation token
├── app.css                 # Evolve Phase 16 scoped tokens; no Tailwind
├── catalog/scenes.ts       # Six IDs, titles, readiness — pure data
├── routing/hash.ts         # maybeParseSceneRoute(hash) — pure
├── physics/clock.ts        # acceptedStepCount(elapsed) — pure
├── physics/loader.ts       # Keep generated init + ?url WASM
├── physics/session.ts      # Owner; allow advance 1–4 then one capture
├── physics/frame.ts        # Keep copied-lane parser
├── components/CatalogNav.tsx
├── components/PlayerPanel.tsx
├── components/FallbackPanel.tsx
├── components/SiteFooter.tsx
└── render/                 # Keep camera + canvas; no pointer handlers

.github/workflows/pages.yml # Build + deploy; no path filters
scripts/web-build.ts        # Pass VITE_* provenance; keep wasm-then-frontend order
```

`App.tsx` is already 443 lines. Split it. Do not grow the proof page in place. [VERIFIED: web/src/App.tsx]

### Pattern 1: Functional-core routing and clock

**What:** Parse hashes and compute step budgets as data-in/data-out functions. Solid and `requestAnimationFrame` stay in the shell.
**When to use:** All WEB-02 and WASM-04 decisions.

```typescript
// Source: standards/core/architecture.md + 17-CONTEXT D-04/D-12
export const SCENE_IDS = [
  "dam-break",
  "fountain",
  "float-or-sink",
  "color-mixer",
  "jelly-drop",
  "water-wheel",
] as const;

export type SceneId = (typeof SCENE_IDS)[number];

export type SceneRoute =
  | { readonly kind: "scene"; readonly id: SceneId }
  | { readonly kind: "empty" }
  | { readonly kind: "unknown"; readonly maybeRaw: string };

export function maybeParseSceneRoute(hash: string): SceneRoute {
  const parts = hash.replace(/^#/, "").split("/").filter((part) => part.length > 0);
  if (parts.length === 0) {
    return { kind: "empty" };
  }
  if (parts[0] !== "scene" || parts.length !== 2) {
    return { kind: "unknown", maybeRaw: parts.join("/") };
  }
  const maybeId = SCENE_IDS.find((id) => id === parts[1]);
  if (maybeId === undefined) {
    return { kind: "unknown", maybeRaw: parts[1] ?? "" };
  }
  return { kind: "scene", id: maybeId };
}
```

Ready vs not-ready is catalog metadata, not the parser. Only `dam-break` constructs a WASM world.

### Pattern 2: One session, generation token, Solid cleanup

**What:** Increment a generation counter on every start/reset/route change. After `await loadProofSession()`, dispose immediately if the token is stale. `onCleanup` always cancels rAF, disconnects observers/listeners, and `dispose()`s.
**When to use:** Reset, Retry, leaving Dam Break, component unmount.

```typescript
// Source: https://docs.solidjs.com/reference/lifecycle/on-cleanup
onCleanup(() => {
  document.removeEventListener("visibilitychange", onVisibilityChange);
  window.removeEventListener("hashchange", onHashChange);
  stopResources();
});
```

Phase 16 already cancels rAF, disconnects `ResizeObserver`, and disposes. Extend that helper; do not replace it with GC. [VERIFIED: web/src/App.tsx `stopResources`]

### Pattern 3: Bounded fixed timestep

**What:** Accumulate rAF timestamps, clamp wall-clock delta, accept at most four `1/60` steps, then call Rust `advance(n)` once and capture once.
**When to use:** Playing + visible document only.

Rust already rejects counts outside 1–4 and steps at `1/60` with 8 velocity / 3 position / 2 particle iterations. [VERIFIED: crates/liquidfun-wasm/src/session.rs]

```typescript
// Source: 17-CONTEXT D-12; MDN requestAnimationFrame timestamp warning
export const STEP_SECONDS = 1 / 60;
export const MAX_STEPS_PER_FRAME = 4;
export const MAX_DELTA_SECONDS = MAX_STEPS_PER_FRAME * STEP_SECONDS;

export function acceptedStepCount(elapsedSeconds: number): number {
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return 0;
  }
  const clamped = Math.min(elapsedSeconds, MAX_DELTA_SECONDS);
  return Math.min(MAX_STEPS_PER_FRAME, Math.floor(clamped / STEP_SECONDS));
}
```

Extend `SceneSession` with `nextFrame(stepCount = 1)` that forwards `1..=4` to `generatedSession.advance`. Do not call `nextFrame()` four times (four captures). Keep existing single-step unit tests.

On `document.hidden`: do not step; keep the Playing label. On visible again: set last timestamp to the next rAF time (clear debt). [CITED: developer.mozilla.org/Page_Visibility_API]

### Pattern 4: Production Vite base + hash hrefs

**What:** Bake `/liquidfun-rs/` into production asset URLs. Keep `vite` dev at `/`. Preview the built dist (it already contains the production base).
**When to use:** All HOST-03 asset loading.

```typescript
// Source: https://vite.dev/guide/static-deploy.html#github-pages
// Vite 8 ConfigEnv.isPreview is present in local 8.3.0 types [VERIFIED]
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig(({ command, isPreview }) => ({
  root: import.meta.dirname,
  base: command === "build" || isPreview ? "/liquidfun-rs/" : "/",
  plugins: [solid()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
}));
```

Catalog links must be hash-only (`href="#/scene/dam-break"`) or `import.meta.env.BASE_URL + "#/scene/dam-break"`. Never `href="/#/scene/dam-break"` — that leaves the project prefix and 404s on user GitHub Pages. [CITED: 17-CONTEXT D-04/D-05]

Keep `import wasmUrl from "...liquidfun_wasm_bg.wasm?url"` and `init({ module_or_path: wasmUrl })`. Vite prefixes the hashed WASM URL with `base`. Do not switch to Vite's native `.wasm` ESM import; wasm-bindgen `--target web` owns instantiation. [CITED: vite.dev/guide/features.html#webassembly] [CITED: wasm-bindgen deployment `--target web`] [VERIFIED: web/src/physics/loader.ts]

### Pattern 5: Same-checkout Pages workflow

**What:** One workflow, two jobs, no path filters, no PAT.
**When to use:** HOST-01/02.

```yaml
# Source: https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages
# Concurrency: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency
name: Pages

on:
  push:
    branches: [main]
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}

jobs:
  build-site:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0
        with:
          persist-credentials: false
          submodules: false
      # rustc 1.97.0, wasm32-unknown-unknown, wasm-pack 0.15.0, bun 1.4.2
      # env: VITE_APP_VERSION, VITE_GIT_SHA, VITE_BUILD_ID, VITE_BUILD_URL
      - run: bun scripts/web-build.ts build
      - uses: actions/configure-pages@45bfe0192ca1faeb007ade9deae92b16b8254a0d
      - uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9
        with:
          path: web/dist

  deploy-pages:
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    needs: build-site
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    permissions:
      contents: read
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@368f82528645a54fb793d4d04e342629a3f51346
```

Why this concurrency shape:

- Official starter uses `group: pages` + `cancel-in-progress: false` so an in-flight deploy finishes. [CITED: actions/starter-workflows pages/static.yml]
- Vite's sample uses `cancel-in-progress: true`, which 2026 Pages deploys have left in `try again later` / error states. [CITED: vite.dev/guide/static-deploy] [CITED: isaratech/dfci#1, 2026-07-03]
- Sharing one `pages` group across PRs and main would serialize PR builds against production deploys. Use `${{ github.workflow }}-${{ github.ref }}`.
- Default queue is `single`: a newer pending main run replaces an older pending run (coalesce). The in-progress run finishes, then the newest pending run deploys. Final site is the newest completed revision. [CITED: GitHub concurrency docs]
- GitHub does not guarantee dispatch-time order; FIFO is wait-start time. Still, the last successful deploy after coalesce is the newest pending SHA, not an older leftover. [CITED: GitHub concurrency docs]

Do not grant `pages: write` at workflow top level. PRs then have only `contents: read`. [CITED: 17-CONTEXT D-15]

Do not add `paths:` filters. A Rust-only main push must rebuild shipped physics. [CITED: 17-CONTEXT D-14] [CITED: .planning/research/v1.1/PITFALLS.md]

### Pattern 6: Provenance injection

**What:** Vite only exposes `VITE_*` to the client. Missing fields render `Unavailable`.
**When to use:** Footer on every view, including fallback. [CITED: standards/core/operability.md] [CITED: 17-UI-SPEC.md]

Set in CI / `web-build.ts`:

| Env | Source | Footer |
|-----|--------|--------|
| `VITE_APP_VERSION` | `web/package.json` version (`0.0.0`) | Version |
| `VITE_GIT_SHA` | `github.sha` or `git rev-parse HEAD` | 7–12 char hash; link to `https://github.com/bright-builds-llc/liquidfun-rs/commit/{full}` when SHA looks like `[0-9a-f]{40}` |
| `VITE_BUILD_ID` | `github.run_id` or local ISO time | Build |
| `VITE_BUILD_URL` | `${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}` | Link Build when present |

Local `just web-build` may show Version + Commit and `Unavailable` Build. That is correct.

External footer links: `target="_blank"` + `rel="noopener noreferrer"`. Fixed URLs only:

- `https://github.com/bright-builds-llc/liquidfun-rs`
- `https://openlinks.us/`
- commit / Actions URLs constructed from injected SHAs, never from hash-route input

### Anti-Patterns to Avoid

- **Path routes (`/scene/dam-break`):** GitHub Pages has no rewrite; hard reload 404s. Hash only.
- **Origin-absolute catalog href `/#/scene/...`:** Drops `/liquidfun-rs/`.
- **Playwright `baseURL` ending in `/liquidfun-rs/` plus `goto('/')`:** Playwright treats leading `/` as origin-absolute, so the test hits `http://127.0.0.1:4173/` and misses WASM. Use `baseURL: http://127.0.0.1:4173` and `goto('/liquidfun-rs/#/scene/dam-break')`.
- **Keeping `Dispose session` or Phase 16 proof copy:** Locked product chrome forbids it.
- **Calling `nextFrame()` once per rAF with no timestamp:** Runs faster on 120 Hz displays. [CITED: MDN requestAnimationFrame]
- **Treating hidden-tab rAF pause as sufficient:** Browsers often pause rAF, but D-12 still requires explicit `document.hidden` + cleared accumulator. [CITED: MDN Page Visibility + rAF]
- **Waiting on Cargo CI / oracle jobs:** Revives qualification and can deploy a different SHA.
- **Rebuilding WASM in the deploy job:** Violates same-checkout artifact rule.
- **`innerHTML` for errors:** XSS and UI-SPEC forbid it.
- **Installing MysticUI/Tailwind/`components.json`:** D-09.
- **Pointer handlers on the canvas:** Phase 19.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pages artifact tarball + MIME | Custom `tar` / `gh-pages` branch push | `actions/upload-pages-artifact` + `actions/deploy-pages` | Official format, OIDC, environment URL output [CITED: GitHub Pages custom workflows] |
| Deploy auth | PAT, deploy key, `peaceiris/actions-gh-pages` | `id-token: write` + `github-pages` env | Locked; environment already exists and allows only `main` [VERIFIED: gh api] |
| WASM glue / memory views | Manual `instantiateStreaming` or raw pointers | Generated `--target web` `init({ module_or_path })` | Copies, fallbacks, no invalidated views [CITED: wasm-bindgen deployment] |
| Component cleanup | Ad-hoc globals | Solid `onCleanup` | Runs on unmount and effect refresh [CITED: Solid docs] |
| Hidden-tab detection | `blur`/`focus` proxies | `document.hidden` + `visibilitychange` | Focus ≠ hidden [CITED: MDN Page Visibility] |
| UI kit | Custom design system or MysticUI | Existing scoped CSS + `17-UI-SPEC.md` | Locked thin-slice exception |

**Key insight:** Hand-roll only the tiny pure cores (hash, clock, catalog data). Reuse Phase 16 session/renderer and official Pages actions for everything that has hidden edge cases.

## Common Pitfalls

### Pitfall 1: Root-local success, blank hosted site

**What goes wrong:** JS/WASM 404 or HTML-as-WASM because assets were built with `base: "/"`.
**Why it happens:** `vite.config.ts` currently sets `base: "/"`. [VERIFIED: web/vite.config.ts]
**How to avoid:** Production/preview-of-production `base: "/liquidfun-rs/"`. Assert built `web/dist/index.html` and hashed `.wasm` URLs contain `/liquidfun-rs/`. Preview with `bun run preview` and open `/liquidfun-rs/#/scene/dam-break`.
**Warning signs:** Network shows `text/html` for a `.wasm` request (Pages 404 page). [CITED: MDN instantiateStreaming] [CITED: Stack Overflow WASM-on-Pages MIME]

### Pitfall 2: Leaked worlds and duplicate loops

**What goes wrong:** Reset or hash change starts a second session; old rAF keeps stepping.
**Why it happens:** Async `init` finishes after navigation; cleanup forgets a listener.
**How to avoid:** Generation token + `stopResources` before every recreate + `onCleanup`. Unit-test stale-load discard. Playwright: reset then assert only one advancing `data-step-index`.
**Warning signs:** Step index jumps after Pause; particle counts grow past 192 on Dam Break; two canvases animating.

### Pitfall 3: Hidden-tab catch-up storm

**What goes wrong:** Returning to the tab runs seconds of steps in one frame.
**Why it happens:** Accumulator keeps wall time while hidden, or rAF resumes with a huge timestamp delta.
**How to avoid:** Skip stepping while `document.hidden`; on resume, forget `maybeLastTimestamp` instead of applying `timestamp - last`. Cap at 4 steps.
**Warning signs:** Tab-restore hitch; `data-step-index` jumps by tens.

### Pitfall 4: Older Pages revision wins

**What goes wrong:** Two main deploys race; older `deploy-pages` finishes last.
**Why it happens:** No concurrency group, or `cancel-in-progress: true` aborts the newer publish.
**How to avoid:** Workflow concurrency above; deploy only `needs: build-site`; environment already limited to `main`. [VERIFIED: gh api environments]
**Warning signs:** Live footer commit ≠ latest green main SHA.

### Pitfall 5: Playwright misses the project base

**What goes wrong:** Smoke still `goto("/")` and looks for "Dispose session".
**Why it happens:** Phase 16 spec is hardcoded to proof copy and root preview. [VERIFIED: web/e2e/rust-wasm-proof.spec.ts]
**How to avoid:** Update selectors to UI-SPEC labels. Navigate `/liquidfun-rs/#/scene/dam-break` and `/liquidfun-rs/#/scene/not-a-scene`. Keep forensic attachments if useful, but D-17 does not require the Phase 16 closure-attempt matrix for every new assertion.
**Warning signs:** Green smoke that never requested `/liquidfun-rs/assets/*.wasm`.

### Pitfall 6: Qualification creep

**What goes wrong:** Pages job grows into workspace clippy, oracle, sanitizers.
**Why it happens:** Copying `ci.yml`.
**How to avoid:** Pages runner builds `liquidfun-wasm` + `web/` only. Native Cargo CI stays the macOS job. [CITED: PROJECT-SCOPE.md, ROADMAP.md]

### Pitfall 7: Honest catalog accidentally looks playable

**What goes wrong:** Not-ready names get accent markers or start a spinner.
**Why it happens:** Shared player always mounts a canvas.
**How to avoid:** Fallback views omit the live session and canvas. `aria-current="page"` only on Dam Break when that route is active. [CITED: 17-UI-SPEC.md]

## Code Examples

### Dam Break documented initial state

Reuse the existing basin constructor. Document these constants as the reset target; do not silently retune them without updating tests and copy. [VERIFIED: crates/liquidfun-wasm/src/scene.rs]

| Field | Value |
|-------|-------|
| Gravity | `(0, -10)` |
| Water particles | 16×12 = 192, radius `0.2`, spacing `0.32`, origin `(-4.7, 0.4)`, color `(57, 211, 199, 255)` (`#39D3C7`) |
| Basin | Floor `y=0` from `x=-5.5..5.5`, walls to `y=8` |
| Dynamic circle | Position `(2.5, 5.5)`, radius `0.75` |
| Particle cap | 512 |
| Timestep | `1/60`, max 4 advances per JS callback |
| World camera bounds | `(-6, -1)` .. `(6, 8)` [VERIFIED: web/src/render/camera.ts] |

Rename product copy to Dam Break. Keep `ProofSession` as the generated class name unless a rename is mechanically cheap and wasm-pack output stays ignored.

### Hash listener shell

```typescript
// Source: 17-CONTEXT D-04; Solid createSignal
const [route, setRoute] = createSignal(maybeParseSceneRoute(window.location.hash));

function onHashChange(): void {
  setRoute(maybeParseSceneRoute(window.location.hash));
}

window.addEventListener("hashchange", onHashChange);
onCleanup(() => window.removeEventListener("hashchange", onHashChange));
```

`Open Dam Break` sets `window.location.hash = "#/scene/dam-break"` (or assigns `href="#/scene/dam-break"`).

### Visibility + clock reset

```typescript
// Source: https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API
function onVisibilityChange(): void {
  maybeLastTimestamp = undefined;
}

document.addEventListener("visibilitychange", onVisibilityChange);
```

Inside the rAF callback, if `document.hidden` or UI state is not Playing, skip `advance` and do not grow the accumulator.

### Production-base Playwright navigation

```typescript
// Source: Playwright baseURL rules + Vite preview of dist
// playwright.config.ts
use: { baseURL: "http://127.0.0.1:4173" }
webServer: { command: "bun run preview -- --strictPort", url: "http://127.0.0.1:4173/liquidfun-rs/" }

await page.goto("/liquidfun-rs/#/scene/dam-break");
await expect(page.getByRole("status")).toHaveText("Loading Dam Break…");
await expect(page.getByRole("status")).toHaveText("Playing");
await page.getByRole("button", { name: "Pause scene" }).click();
await page.getByRole("button", { name: "Reset scene" }).click();
await page.goto("/liquidfun-rs/#/scene/not-a-scene");
await expect(page.getByRole("heading", { name: "Scene not found" })).toBeVisible();
await page.getByRole("button", { name: "Open Dam Break" }).click();
```

`prefers-reduced-motion: reduce` starts Paused after the first frame. Smoke should use default motion so Playing is observable, or stub the media query explicitly. [CITED: 17-UI-SPEC.md]

### Built-asset gate (no browser)

After `vite build`, fail the Pages build job if `web/dist/index.html` lacks `/liquidfun-rs/assets/` or if no `*.wasm` file exists under `web/dist`. This catches a wrong `base` before deploy.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Branch `gh-pages` + PAT | Actions as source, OIDC `deploy-pages` | GitHub Pages custom workflows | No PAT; `page_url` output |
| Vite sample `cancel-in-progress: true` | Official starter `false` for production deploys | Pages starter + 2026 incident reports | Avoid stranded deploys |
| Phase 16 proof shell | Shared player + honest catalog | This phase | Product URLs and chrome |
| One step per rAF | Timestamp clock, max 4 steps | D-12 | Refresh-rate and hidden-tab safety |
| `base: "/"` | Production `/liquidfun-rs/` | D-05 | Hosted assets resolve |

**Deprecated/outdated:**

- Phase 16 copy: `LiquidFun Rust/WASM browser proof`, `Dispose session`, `Waiting for first Rust frame`. [CITED: 17-UI-SPEC.md]
- Floating action tags in new workflows (`@v4`, `@v5`) — pin SHAs like `ci.yml`.
- Waiting for Linux qualification before shipping the site.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | GitHub Pages will serve this repo's hashed `.wasm` as `application/wasm` | HOST-03 / Pitfall 1 | First deploy needs a Network-tab check; wasm-bindgen glue usually falls back, but a 404 HTML body will still fail |

All other factual claims are verified from the repo, `gh api`, npm registry, or cited official docs. A1 is the only first-deploy empirical gap.

## Open Questions (RESOLVED)

1. **Should the Pages build job run Playwright, or only the unit + dist-path gate?**
   - What we know: D-17 requires production-base loading and player proofs; `just web-smoke` already runs Chromium. HOST-01 says PRs may run the same build/smoke.
   - What's unclear: Ubuntu Playwright install time vs hobby CI cost.
   - Recommendation: Run `bun scripts/web-build.ts build` (wasm, typecheck, unit, vite) on every Pages workflow. Keep the heavy Phase 16 forensic closure optional/local. Live URL/revision recording happens after the first real `main` deploy, not in PR.
   - RESOLVED: Playwright stays in `just web-player-smoke` (local/default player proof). The Pages job does not run Playwright.

2. **Should branch protection require the new Pages build check?**
   - What we know: `github-pages` environment already restricts deploys to `main`. [VERIFIED: gh api]
   - What's unclear: Current required-check list was not inspected.
   - Recommendation: Workflow `needs` is sufficient for HOST-01. Adding a required status check is optional hardening, not a phase blocker.
   - RESOLVED: Branch protection is optional because workflow `needs` satisfies HOST-01.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Bun | web-build, Vite, tests | ✓ | 1.4.2 | — |
| rustc / cargo | liquidfun-wasm | ✓ | 1.97.0 | — |
| `wasm32-unknown-unknown` | wasm-pack | ✓ | installed | `rustup target add` in CI |
| wasm-pack | glue generation | ✓ | 0.15.0 | CI `cargo install wasm-pack --version 0.15.0 --locked` |
| Playwright Chromium | D-17 smoke | ✓ | 1.63.0 local install | `bun run browser:install` |
| just | thin recipes | ✓ | 1.48.0 (stack pin 1.55.1 unused) | Call `bun scripts/web-build.ts` directly |
| gh | Pages settings / first-deploy evidence | ✓ | 2.87.3 | — |
| Pages settings | HOST-02/03 | ✓ | Actions source; URL confirmed | Already configured |
| `github-pages` environment | deploy job | ✓ | main-only branch policy | Already configured |
| `pages.yml` | HOST-01 | ✗ | — | Create in this phase |
| Live deployment | HOST-03 evidence | ✗ | none yet | First successful main deploy |

**Missing dependencies with no fallback:**

- A real Pages deployment record. The implementing agent must land `pages.yml` on `main` and record `page_url` + SHA. Standing authorization covers that ordinary push.

**Missing dependencies with fallback:**

- Playwright in the Pages workflow: local `just web-smoke` can prove player behavior if CI time is trimmed; HOST-03 still needs the live URL after deploy.

**Step 2.6 note:** Delivery depends on GitHub Actions runners (ubuntu-24.04, `GITHUB_TOKEN`). Those are not local tools. Do not use a personal token.

## Security Domain

`security_enforcement` is not set `false` in `.planning/config.json` (absent = enabled).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No accounts |
| V3 Session Management | no | WASM session is local physics, not a user session |
| V4 Access Control | no | Public static site |
| V5 Input Validation | yes | Allowlisted scene IDs; Rust already rejects `advance` outside 1–4; finite frame parser |
| V6 Cryptography | no | No app crypto; do not hand-roll hashes for security. Playwright SHA-256 is test evidence only |

### Known Threat Patterns for SolidJS + WASM + Pages

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via error text | Tampering / XSS | Fixed copy in text nodes; optional `Details:` is escaped text, never `innerHTML` [CITED: 17-UI-SPEC.md] |
| Open redirect via scene hash | Spoofing | Parser allowlist; footer URLs are constants or SHA-derived GitHub URLs |
| HTML 404 interpreted as WASM | Tampering | Production `base`; inspect WASM `Content-Type`; failure state + Retry |
| Stolen deploy token | Elevation | No PAT; job-scoped OIDC; environment limited to `main` [VERIFIED: gh api] |
| Stale/hostile artifact | Tampering | Deploy `needs` same-workflow artifact; no second WASM rebuild |
| Resource exhaustion | Denial of service | Max 4 steps/frame, hidden-tab pause, particle cap 512, one world |

## Sources

### Primary (HIGH confidence)

- Repository: `web/src/App.tsx`, `session.ts`, `loader.ts`, `vite.config.ts`, `scripts/web-build.ts`, `crates/liquidfun-wasm/src/{lib,session,scene}.rs`, `.github/workflows/ci.yml`, `17-CONTEXT.md`, `17-UI-SPEC.md`
- `gh api repos/bright-builds-llc/liquidfun-rs/pages` — `html_url` `https://bright-builds-llc.github.io/liquidfun-rs/`, `build_type: workflow`
- `gh api` environments — `github-pages` exists; deployment branch policy `main` only; no deployments yet
- npm registry 2026-09-17 — Solid 1.9.15, Vite 8.3.0, plugin 2.11.14, TS 7.0.2, Vitest 5.0.1, Playwright 1.63.0
- [GitHub custom Pages workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
- [GitHub concurrency](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)
- [actions/starter-workflows pages/static.yml](https://raw.githubusercontent.com/actions/starter-workflows/main/pages/static.yml)
- [Vite GitHub Pages deploy](https://vite.dev/guide/static-deploy.html#github-pages)
- [Vite WebAssembly `?url`](https://vite.dev/guide/features.html#webassembly)
- [Solid `onCleanup`](https://docs.solidjs.com/reference/lifecycle/on-cleanup)
- [MDN Page Visibility API](https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API)
- [MDN `requestAnimationFrame`](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)
- [wasm-bindgen deployment `--target web`](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html)
- [MDN `instantiateStreaming` MIME](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/JavaScript_interface/instantiateStreaming_static)

### Secondary (MEDIUM confidence)

- Vite official sample pins match the SHAs resolved from current action releases, but its `cancel-in-progress: true` conflicts with the Pages starter
- 2026 reports that cancelling `deploy-pages` mid-flight yields `Deployment failed, try again later`
- Community claim that GitHub Pages serves `.wasm` as `application/wasm` when the file URL is correct (verify on first deploy)

### Tertiary (LOW confidence)

- None used as authoritative

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — pins match npm and local toolchains; no new libraries
- Architecture: HIGH — Phase 16 code + locked CONTEXT + official Solid/Vite/Pages docs
- Pitfalls: HIGH for local/base/lifecycle; MEDIUM until the first live Pages WASM response is inspected

**Research date:** 2026-09-17
**Valid until:** 2026-10-17 (action SHAs and npm pins should be rechecked if planning slips)
