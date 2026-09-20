# Phase 20: Playground catalog previews and Reset honesty - Research

**Researched:** 2026-09-20
**Domain:** SolidJS playground catalog chrome, static SVG previews, preset-select state vs native Reset
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Catalog preview medium

- **D-01:** Restore in-app visual previews for Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel. Each DemoNavigation entry keeps its existing title, short description, and hash Open link, and adds a compact static illustration.
- **D-02:** Previews are static inline SVG or CSS illustrations authored in the frontend, as in Phase 18 D-02. Do not start a WASM world per catalog entry, capture live Canvas thumbnails, animate fake physics, or load README `docs/assets/demos/*.webp` as the in-app preview.
- **D-03:** Caption each illustration `Static preview`. Copy must not imply the catalog thumbnail is a live simulation.

### Catalog layout inside the Kobalte shell

- **D-04:** Keep `PlaygroundShell`: sticky header, 280px desktop sidebar, semantic hash links, and Kobalte Dialog drawer on mobile. Do not restore `.catalog-card`, the three-column card grid, CatalogNav cards, or a separate catalog page.
- **D-05:** Place the compact preview inside each existing `DemoNavigation` list item (sidebar and mobile drawer share the same component). Preserve 44px minimum hit targets, `#39D3C7` 2px `:focus-visible` with 4px offset, dark tokens, and catalog scroll outside the canvas.
- **D-06:** Keep `@kobalte/core` 0.13.12 for the drawer only. Do not adopt MysticUI, Tailwind, shadcn, or an icon package. `web/e2e/shell.spec.ts` must continue asserting `.catalog-card` count is 0.

### Reset live-control honesty

- **D-07:** Play, pause, and Reset still rebuild or freeze the native world as today. After Reset (and any `recreateScene` path that clears `constructionValues`), every preset `<select>` shows the documented initial value from `DEFAULT_PRESET_VALUES`, not a stale `pendingValue`.
- **D-08:** Honesty covers live runtime presets (`recreates: false`) and construction presets (`recreates: true`). Representative live labels: Fountain emission-rate / launch-speed / aim-angle, Float or Sink body, Color Mixer stir-speed, Water Wheel jet-strength / emission. Construction selects such as Dam Break water-amount and Color Mixer mix-strength also return to documented initials after Reset.
- **D-09:** Do not change WEB-04 control sets, Apply-setting copy, or native reset physics. This phase only makes chrome match the rebuilt world.

### Focused Chromium smoke

- **D-10:** Extend the existing Chromium `just web-player-smoke` suite. Cover visible static previews for all six catalog entries in the desktop sidebar, one mobile-drawer preview check, and Reset label honesty for representative live presets (Fountain emission-rate, Float or Sink body, Color Mixer stir-speed, Water Wheel jet-strength).
- **D-11:** Do not add Firefox/Safari/WebKit, a live-preset combinatorial matrix, screenshot-hash oracles, Linux qualification, or Dam Break C++ timing. Independent AI review remains eligible; the implementing agent must not approve its own work.

### Claude's Discretion

- Exact SVG/CSS artwork, preview aspect ratio, and list-item packing within the locked 280px sidebar, 44px hit target, and dark-token contract.
- Whether Reset honesty remounts `SceneControls` with a key, resets `pendingValue` from `maybeValues`/`DEFAULT_PRESET_VALUES`, or derives select value from a reactive construction bag — provided labels match the rebuilt world after Reset.
- Exact Playwright file split and helper extraction, provided D-10 is covered without a new browser matrix.

### Folded Todos

None — `todo match-phase 20` returned no pending todos.

### Deferred Ideas (OUT OF SCOPE)

- Unused `loadProofSession` and opt-in `rust-wasm-proof.spec.ts` — Phase 21.
- Dead `FallbackPanel` empty/not-ready branches — Phase 21.
- Bright Builds `file-lengths` on `color_mixer.rs`, `dam_break.rs`, and `water_wheel.rs` — Phase 21.
- Playground Dam Break headless speed versus pinned C++ — out of v1.1 definition of done.
- MysticUI/Tailwind/design-system migration — revisit 2026-12-17 per standards-overrides.md.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WEB-01 | A visitor can browse six demo cards with names, short descriptions and previews, then open a selected demo in a shared player. | Restore compact token-only `ScenePreview` inside shared `DemoNavigation` (not `.catalog-card`). Keep existing `#/scene/{id}` hash links. Caption `Static preview`. Prove six desktop sidebar previews plus one mobile-drawer preview in Chromium smoke. |
| WEB-03 | A visitor can play, pause and reset the selected demo to its documented initial state. | Keep play/pause/reset physics paths. Clear `constructionValues` on `recreateScene`, remount or resync `PresetControl` `pendingValue` from `DEFAULT_PRESET_VALUES`, and assert representative live select values after Reset. |
</phase_requirements>

## Summary

Phase 20 is gap closure, not a new player. The owner-approved Kobalte shell already lists six demos with titles, descriptions, and hash links. Visual previews were deleted with `.catalog-card` because the three-column grid broke narrow widths. Restore the Phase 18 illustration *idea* inside the existing list item. Do not restore CatalogNav, card CSS, README WebPs, or a WASM thumbnail session.

Reset already disposes and reconstructs the native world through `recreateScene` → `startScene` → `loadSceneSession`. Live preset labels lie because `PresetControl` snapshots `pendingValue` once in `createSignal` and never remounts on the same scene. Construction labels can also lie, and `startScene` currently re-applies last construction presets. D-08 requires every preset select to show documented initials after Reset, so `recreateScene` must clear the construction bag *and* rebuild `PresetControl` state. Do not change WASM constructors, Apply-setting copy, or play/pause freeze/resume.

**Primary recommendation:** Restore `web/src/catalog/previews.tsx` as compact inline SVG inside each `DemoNavigation` `<a>`, add `.demo-nav-preview*` CSS without `.catalog-card`, clear `constructionValues` in `recreateScene`, remount `SceneControls` with a Solid keyed `Show` on a truthy epoch string, and extend the existing Chromium `test:player` files.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. [VERIFIED: glob `.cursor/rules/**/*` returned 0 files]

Honor instead:

- `AGENTS.md` Repo-Local Guidance: hobby scope, standing autonomous iteration, independent AI review (implementer must not approve own work).
- `standards-overrides.md`: semantic HTML + scoped CSS + Kobalte Dialog 0.13.12 only; `skipLibCheck: true` stays.
- `standards/core/architecture.md`: functional core for catalog/control value decisions; Solid/WASM I/O stays in the shell.
- `standards/core/frontend-ui.md`: dark default, source/FOSS/provenance chrome already in footer — do not restyle the product identity.
- `standards/core/code-shape.md`: `maybe_` naming; files over ~628 physical lines are a managed `file-lengths` fail at 629.
- `standards/core/testing.md`: Arrange/Act/Assert; one concern per unit test; pure helpers in Vitest.
- `standards/core/verification.md`: repo-native checks (`just web-player-smoke`, `bun run typecheck`, `bun run test:unit`).
- `standards/languages/typescript-javascript.md`: SolidJS default; MysticUI is overridden for this playground.

No `.cursor/skills/` or `.agents/skills/` project skills exist. [VERIFIED: glob returned 0 files]

## Standard Stack

Use the already-pinned playground stack. Add no packages.

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| SolidJS | 1.9.15 | Playground UI | Existing pin; fine-grained updates; `Show keyed` remounts stateful controls. [VERIFIED: `web/package.json`] |
| `@kobalte/core` | 0.13.12 | Mobile demo `Dialog` only | Locked D-06; current npm latest as of 2026-06-30. [VERIFIED: npmjs.com/package/@kobalte/core; `web/package.json`] |
| TypeScript | 7.0.2 | Typecheck | Existing pin; keep `skipLibCheck` for Kobalte. [VERIFIED: `web/package.json`; `standards-overrides.md`] |
| Vite | 8.3.0 + `vite-plugin-solid` 2.11.14 | Production `/liquidfun-rs/` build | Existing pin. [VERIFIED: `web/package.json`] |
| Bun | 1.4.2 | Scripts, lockfile, unit tests | `packageManager` pin. [VERIFIED: `web/package.json`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Vitest | 5.0.1 | Node unit tests | `tests/**/*.test.ts` only — do not import Solid JSX. [VERIFIED: `web/vitest.config.ts`] |
| Playwright | 1.63.0 | Chromium smoke | `just web-player-smoke` → `bun run test:player`. [VERIFIED: `web/package.json`; `scripts/web-build.ts`] |
| Scoped CSS in `web/src/app.css` | existing tokens | Preview packing | No Tailwind/MysticUI. [VERIFIED: `standards-overrides.md`; D-06] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Compact SVG in `DemoNavigation` | README WebP `<img>` | Forbidden by D-02 / ROADMAP; treats docs gallery as in-app live-adjacent chrome. |
| Compact SVG in `DemoNavigation` | WASM/Canvas thumbnails | Forbidden by D-02; would start extra worlds and break one-session ownership. |
| Compact list item | Restore `.catalog-card` grid | Forbidden by D-04; that grid is why previews were removed. |
| Solid keyed `Show` remount | React-style `key=` on `SceneControls` | Solid does not remount on a `key` prop. [CITED: docs.solidjs.com/reference/components/show] |
| Solid keyed `Show` remount | `createEffect` syncing `pendingValue` from `maybeValues` | `constructionValues` is an untracked `let`; App JSX tracks `view()` every animation frame, so a naive effect can clobber unapplied construction drafts. Use only if the effect tracks a dedicated Reset epoch signal. |

**Installation:** none — reuse `cd web && bun install --frozen-lockfile`.

**Version verification:** `@kobalte/core@0.13.12` is the npm latest (published 2026-06-30). Do not bump. Solid/Playwright/Vitest stay on `web/package.json` pins.

## Architecture Patterns

### Recommended Project Structure

```
web/src/catalog/previews.tsx     # restore ScenePreview; token-only inline SVG; no WASM
web/src/catalog/scenes.ts        # unchanged ids/titles/descriptions/controls
web/src/player/navigation.ts     # unchanged hash items; DemoNavigation still consumes this
web/src/components/DemoNavigation.tsx  # preview + Static preview + title + description inside one <a>
web/src/components/PlaygroundShell.tsx # unchanged composition (sidebar + Kobalte drawer)
web/src/components/SceneControls.tsx   # keep PresetControl; remount from App
web/src/components/scene-controls.ts   # DEFAULT_PRESET_VALUES + initialPresetValue (extend tests)
web/src/player/runtime.ts        # constructionEntriesForScene; update Reset comment
web/src/App.tsx                  # clear constructionValues in recreateScene; keyed remount
web/src/app.css                  # .demo-nav-preview*; never .catalog-card
web/e2e/shell.spec.ts            # six sidebar previews + one drawer preview; catalog-card count 0
web/e2e/player.spec.ts or new spec included in test:player  # Reset select honesty
web/tests/controls.test.ts       # initialPresetValue / documented defaults
```

Keep catalog metadata out of live sessions. `previews.tsx` must not import `ProofSession`, `loadSceneSession`, or `requestAnimationFrame`. [VERIFIED: Phase 18 D-02; 18-08-PLAN.md verify command]

### Pattern 1: Compact preview inside the existing hash link

**What:** Each list item remains one semantic `<a href="#/scene/{id}">`. Insert a decorative SVG plus visible `Static preview` caption above the existing title and description.

**When to use:** WEB-01 restore inside the Kobalte shell.

**Example:**

```tsx
// Source: live DemoNavigation.tsx + 20-CONTEXT D-01/D-03/D-05
<a class="demo-nav-link" href={item.href} aria-current={item.isCurrent ? "page" : undefined}>
  <ScenePreview sceneId={item.id} />
  <span class="demo-nav-preview-caption">Static preview</span>
  <span class="demo-nav-title">{item.title}</span>
  <span class="demo-nav-description">{item.description}</span>
</a>
```

Do not add a nested `Open` button. The 2026-09-19 shell design made the whole item the link; D-04 keeps that. [VERIFIED: `docs/superpowers/specs/2026-09-19-responsive-playground-shell-design.md`; `DemoNavigation.tsx`]

### Pattern 2: Token-only still SVG

**What:** `ScenePreview` switches on `SceneId` and returns inline `<svg aria-hidden="true">`. Tokens only: `#39D3C7`, `#F87171`, `#CBD5E1`, `#F4F7FA`, `#071018`. Art direction from 18-UI-SPEC:

| Scene | Illustration |
|-------|----------------|
| Dam Break | Teal rectangle above a three-segment basin and one rigid circle |
| Fountain | Short teal arc of dots into a U-shaped bowl |
| Float or Sink | Teal pool band, one small high rectangle, one larger low rectangle |
| Color Mixer | Teal blob and `#F87171` blob overlapping; no animated blend |
| Jelly Drop | Soft rounded square or circle in primary text fill over two rigid bars |
| Water Wheel | Hub circle, four paddle segments, short teal jet dash |

Restore from historical `web/src/catalog/previews.tsx` at commit `f054135` (Phase 18 Plan 08) and restyle compactly. Do not copy `.catalog-card` rules. [VERIFIED: `.planning/phases/18-six-native-physics-demos/18-08-SUMMARY.md`; 18-UI-SPEC preview table]

**Discretion:** Use a compact aspect (recommend `aspect-ratio: 16 / 9` with `max-block-size: 72px` or `88px`) so six items still scroll inside the sticky 280px sidebar (`max-height: calc(100vh - 156px)`). Full card-sized 16:9 frames would dominate the list. [VERIFIED: `.demo-sidebar` in `web/src/app.css`; D-05]

### Pattern 3: Keyed remount for Reset honesty

**What:** `createSignal(initialPresetValue(...))` runs once per `PresetControl` mount. Same-scene Reset does not change `SCENES` control object identity, so `<For each={props.controls}>` does not remount. Solid `Show` without `keyed` also keeps children when `when` stays truthy. [VERIFIED: `SceneControls.tsx`; [CITED: docs.solidjs.com/reference/components/show]]

**When to use:** After `recreateScene` / scene-switch that must rebuild local select state.

**Example:**

```tsx
// Source: https://docs.solidjs.com/reference/components/show
const controlsEpoch = () => `${sceneId()}:${resetGeneration()}`;

<Show when={controlsEpoch()} keyed>
  {() => (
    <SceneControls
      controls={currentScene().controls}
      disabled={sceneControlsDisabled()}
      maybeValues={constructionValues}
      onApplyControl={applySceneControl}
      onApplyAction={applySceneAction}
    />
  )}
</Show>
```

Start `resetGeneration` so the `when` string is never `""`. Do **not** use `when={resetGeneration()}` if the number can be `0` — `Show` treats `0` as falsy and unmounts controls. [CITED: Solid `Show` `when` truthiness]

### Pattern 4: Clear construction bag on Reset so chrome matches the rebuilt world

**What:** `recreateScene` currently calls `startScene` without clearing `constructionValues`. `startScene` then `applyControl`s `constructionEntriesForScene(...)`, so last Apply-setting presets survive Reset. D-08 requires construction selects to return to documented initials. WASM `build([])` already defaults to those same initials. [VERIFIED: `App.tsx` `recreateScene`/`startScene`; `runtime.ts` `constructionEntriesForScene`; scene `build` defaults below]

| Control id | DEFAULT_PRESET_VALUES | Native empty-bag default |
|------------|----------------------|--------------------------|
| `water-amount` | `medium` | `WaterAmount::Medium` [VERIFIED: `dam_break.rs` `build`] |
| `gravity` | `normal` | `GravityPreset::Normal` [VERIFIED: `dam_break.rs` `build`] |
| `emission-rate` | `medium` | `EmissionRate::Medium` [VERIFIED: `fountain.rs`] |
| `launch-speed` / `aim-angle` | `medium` / `up` | `MEDIUM_LAUNCH_SPEED` / `UP_AIM` [VERIFIED: `fountain.rs`] |
| `body` | `wood` | `BodyPreset::Wood` [VERIFIED: `float_or_sink.rs`] |
| `mix-strength` | `strong` | `MixStrength::Strong` [VERIFIED: `color_mixer.rs` `build`] |
| `stir-speed` | `slow` | `StirSpeed::Slow` [VERIFIED: `color_mixer.rs`] |
| `jet-strength` / `emission` | `medium` / `on` | `MEDIUM_JET_SPEED` / `Emission::On` [VERIFIED: `water_wheel.rs`] |

**Required Reset sequence in `recreateScene`:**

1. `constructionValues = {}` (new object, not a mutation of the old bag).
1. Bump a Solid `resetGeneration` signal so keyed `Show` remounts.
1. `void startScene(id)` as today.

Play and pause must not clear the bag or remount controls. [VERIFIED: D-07; `playScene`/`pauseScene` do not call `startScene`]

Retry shares `onRetry={recreateScene}`. Clearing on Retry is correct: failure recovery returns to documented initials. Do not invent a second Reset path.

Update the comment on `constructionEntriesForScene` from "must survive Reset" to "last applied construction presets until Reset/scene switch clears the bag". Keep the helper for Apply-setting → later `startScene` only while the bag is non-empty (today that is Apply then implicit recreate via `applyControl`, not Reset). [VERIFIED: `web/src/player/runtime.ts` lines 47–64]

This is TypeScript session wiring, not a WASM recipe change. D-09 "native reset physics" means do not edit scene constructors or control sets. [VERIFIED: D-09; scene files stay out of this phase except as default-value evidence]

### Anti-Patterns to Avoid

- **Reviving `.catalog-card` / three-column grid / CatalogNav:** broke narrow widths; `shell.spec.ts` forbids leftover cards. [VERIFIED: D-04; `web/e2e/shell.spec.ts`]
- **`<img src="docs/assets/demos/*.webp">`:** documentation gallery, not in-app preview. [VERIFIED: D-02; ROADMAP Phase 20]
- **React `key=` on Solid components:** does not remount. Use `Show keyed` or an epoch-tracked `createEffect`. [CITED: docs.solidjs.com]
- **Syncing `pendingValue` from `maybeValues` on every parent update:** `view()` changes every frame; `constructionValues` is not a signal. [VERIFIED: `App.tsx` `let constructionValues`]
- **Growing `App.tsx` past 628 physical lines:** current file is **627** lines. Extract the keyed wrapper or epoch helper rather than inlining. [VERIFIED: `web/src/App.tsx` ends at line 627; `standards/core/code-shape.md` file-lengths fail at 629]
- **Growing `app.css` past 628:** current file is **607** lines. Add a small `.demo-nav-preview*` block only; do not reintroduce deleted card-grid CSS. [VERIFIED: `web/src/app.css` ends at line 607]
- **Importing `previews.tsx` from Vitest:** `include` is `tests/**/*.test.ts` and environment is `node`. Prove SVG via typecheck + Playwright, or extract token constants to a `.ts` helper. [VERIFIED: `web/vitest.config.ts`; Phase 18 STATE extract-to-`.ts` decision]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Accessible mobile drawer | Custom focus-trap/scroll-lock | Existing `@kobalte/core/dialog` 0.13.12 | Already implemented in `PlaygroundShell`. [VERIFIED: D-06] |
| Catalog routing | New catalog page | Existing `#/scene/{id}` + `sceneNavigationItems` | WEB-02 already complete. |
| Documented default labels | Duplicate option tables in tests | `DEFAULT_PRESET_VALUES` + `initialPresetValue` | Single source of truth. [VERIFIED: `scene-controls.ts`] |
| Preview visual regression | Screenshot-hash oracles | DOM: visible SVG + `Static preview` text | D-11 forbids screenshot hashes. |
| Live catalog thumbnails | Per-row WASM worlds | Static SVG | D-02; one-session ownership. |
| Design system | MysticUI / Tailwind / icon pack | Scoped CSS tokens | `standards-overrides.md` until 2026-12-17. |
| Multi-browser matrix | Firefox/WebKit projects | Existing Chromium-only `playwright.config.ts` | D-11; `assertChromiumOnlyPlaywrightConfig`. |

**Key insight:** The missing work is restoring deleted presentational SVG and fixing Solid local state vs an already-correct native reconstruct. Hand-rolling a new catalog or control protocol would reopen WEB-04/WEB-07.

## Common Pitfalls

### Pitfall 1: Restoring the Phase 18 card grid with the previews

**What goes wrong:** Narrow widths overflow; WEB-07 regresses; `shell.spec.ts` fails `.catalog-card` count 0.

**Why it happens:** 18-UI-SPEC and 19-UI-SPEC still describe cards. Those specs are historical for layout. 20-CONTEXT D-04 wins.

**How to avoid:** New classes `.demo-nav-preview` / `.demo-nav-preview-caption` only. Grep must find zero `.catalog-card` in `web/src`.

**Warning signs:** `grid-template-columns: repeat(3, …)` returning under catalog rules; a `CatalogNav.tsx` file.

### Pitfall 2: `pendingValue` initialized once

**What goes wrong:** After Reset, Fountain Emission rate still shows High while the reconstructed world emits Medium.

**Why it happens:** `createSignal(initialPresetValue(...))` is a snapshot. Same-scene Reset reuses `PresetControl` instances. [VERIFIED: `SceneControls.tsx` lines 29–31; Solid discussion #287]

**How to avoid:** Keyed remount on Reset, or an effect that tracks only `resetGeneration`.

**Warning signs:** Playwright `toHaveValue` still equals the pre-Reset option after `Reset scene`.

### Pitfall 3: Clearing labels without clearing `constructionValues`

**What goes wrong:** Live selects look honest; Dam Break Water amount still shows Large; `startScene` rebuilds Large. D-08 construction honesty fails. Or chrome shows Medium while the world is Large.

**Why it happens:** Phase 18 comment said construction presets "must survive Reset". Phase 20 D-08 supersedes that for Reset.

**How to avoid:** Clear the bag in `recreateScene` before `startScene`. Add a unit test that `constructionEntriesForScene(scene, {})` is `[]`.

**Warning signs:** Reset after Apply setting Gravity High still applies High.

### Pitfall 4: `Show when={0}` unmounts controls

**What goes wrong:** Scene controls vanish after the first Reset if the epoch signal starts at 0 and is passed raw.

**Why it happens:** Solid `Show` uses truthiness. [CITED: docs.solidjs.com/reference/components/show]

**How to avoid:** `when={\`${sceneId}:${epoch}\`}` or start epoch at `1`.

### Pitfall 5: Dual `DemoNavigation` doubles preview counts

**What goes wrong:** `getByText("Static preview")` is 12 instead of 6 because sidebar and drawer both render the list.

**Why it happens:** `PlaygroundShell` mounts `DemoNavigation` twice by design. [VERIFIED: `PlaygroundShell.tsx`]

**How to avoid:** Scope desktop asserts to `.demo-sidebar` and mobile to `getByRole("dialog", { name: "Demos" })`. Keep `.catalog-card` count global 0.

### Pitfall 6: New Playwright file omitted from `test:player`

**What goes wrong:** `just web-player-smoke` never runs the new spec. D-10 looks done locally with `playwright test e2e/foo.spec.ts` but CI/hobby smoke misses it.

**Why it happens:** `web/package.json` `test:player` lists explicit files, not `e2e/*.spec.ts`. [VERIFIED: `web/package.json` scripts; `scripts/web-build.ts` `bun run test:player`]

**How to avoid:** Prefer extending `shell.spec.ts` / `player.spec.ts`, or add the new file to `test:player`. `player.spec.ts` is 324 lines; Phase 19 split trigger was 400. A new `e2e/reset-honesty.spec.ts` plus `test:player` update is the cleaner split if Reset tests would push past ~400.

### Pitfall 7: Nested interactive content inside the hash link

**What goes wrong:** Invalid HTML, broken keyboard activation, Kobalte drawer `onNavigate` races.

**Why it happens:** Phase 18 cards had a separate Open control.

**How to avoid:** SVG + caption + title + description only. `aria-hidden="true"` on the SVG, not on the caption.

### Pitfall 8: File-length fail on `App.tsx`

**What goes wrong:** `bun scripts/bright-builds-check.ts file-lengths` fails at 629 lines.

**Why it happens:** 627-line file; keyed wrapper is several lines.

**How to avoid:** Extract `sceneControlsIdentity(sceneId, generation)` to `runtime.ts` or a tiny `controls-epoch.ts`. Do not split physics ownership in this phase.

## Code Examples

### Compact preview CSS (do not name it catalog-card)

```css
/* Source: live .demo-nav-* tokens in web/src/app.css; 18-UI-SPEC colors */
.demo-nav-preview {
  display: block;
  width: 100%;
  max-block-size: 72px;
  aspect-ratio: 16 / 9;
  background: #071018;
  border: 1px solid #2A3441;
  border-radius: 8px;
  overflow: hidden;
}

.demo-nav-preview svg {
  display: block;
  width: 100%;
  height: 100%;
}

.demo-nav-preview-caption {
  color: #A9B4C0;
  font-size: 12px;
  line-height: 1.4;
}
```

Keep `.demo-nav-link` `min-block-size` behavior via existing padding; the taller item still meets 44px. Do not add a second focus system — `a:focus-visible` already uses 2px `#39D3C7` / 4px offset. [VERIFIED: `web/src/app.css` lines 392–398]

### Documented select value after Reset

```ts
// Source: web/src/components/scene-controls.ts
export function initialPresetValue(
  control: Extract<SceneControl, { kind: "preset" }>,
  maybeValues: Readonly<Record<string, string>> | undefined,
): string {
  const maybeCurrent = maybeValues?.[control.id];
  if (
    maybeCurrent !== undefined &&
    control.values.some((value) => value.id === maybeCurrent)
  ) {
    return maybeCurrent;
  }

  const maybeDefault = DEFAULT_PRESET_VALUES[control.id];
  if (
    maybeDefault !== undefined &&
    control.values.some((value) => value.id === maybeDefault)
  ) {
    return maybeDefault;
  }

  return control.values[0]?.id ?? "";
}
```

After Reset, pass `maybeValues={{}}` (or omit). Live and construction ids then resolve through `DEFAULT_PRESET_VALUES`. Extend `web/tests/controls.test.ts` with Arrange/Act/Assert cases for `emission-rate` → `medium`, `body` → `wood`, `stir-speed` → `slow`, `jet-strength` → `medium`, `water-amount` → `medium`, `mix-strength` → `strong`. [VERIFIED: `DEFAULT_PRESET_VALUES`; `web/tests/controls.test.ts` currently tests hint copy only]

### Playwright: scoped previews + Reset honesty

```ts
// Source: Playwright locator.selectOption / toHaveValue
// https://playwright.dev/docs/input#select-options
await page.setViewportSize({ width: 1280, height: 800 });
await page.goto("/liquidfun-rs/#/scene/dam-break");
await expect(page.locator(".catalog-card")).toHaveCount(0);
await expect(
  page.locator(".demo-sidebar").getByText("Static preview"),
).toHaveCount(6);
await expect(
  page.locator(".demo-sidebar svg[aria-hidden='true']"),
).toHaveCount(6);

await page.goto("/liquidfun-rs/#/scene/fountain");
await expectReadySceneChrome(page, "Fountain");
await page.getByLabel("Emission rate").selectOption("high");
await expect(page.getByLabel("Emission rate")).toHaveValue("high");
await page.getByRole("button", { name: "Reset scene" }).click();
await expect(page.getByRole("status")).toHaveText("Playing");
await expect(page.getByLabel("Emission rate")).toHaveValue("medium");
```

Repeat for Body/`wood`, Stir speed/`slow`, Jet strength/`medium`. Scope mobile:

```ts
await page.setViewportSize({ width: 390, height: 812 });
await page.getByRole("button", { name: "Demos" }).click();
const dialog = page.getByRole("dialog", { name: "Demos" });
await expect(dialog.getByText("Static preview").first()).toBeVisible();
```

Existing helpers: `expectReadySceneChrome`, `FOUNTAIN_PATH`, `SELECT_NEXT_VALUE` already maps `Stir speed`→`fast` and `Jet strength`→`strong`. Add `Emission rate` / `Body` mappings or inline `selectOption` in the new tests. [VERIFIED: `web/e2e/player-helpers.ts`]

Do not wait on canvas pixel hashes for this phase. Existing play/pause/reset step-index coverage stays.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 18 `.catalog-card` grid + `ScenePreview` | `DemoNavigation` title+description links, previews deleted | 2026-09-19 responsive shell | WEB-01 visual gap |
| Construction presets survive Reset | Reset returns documented initials (chrome + bag) | Phase 20 D-08 | Update `runtime.ts` comment; clear bag |
| `createSignal` local draft | Snapshot-once until remount | Solid 1.x (unchanged) | Must remount or epoch-sync |
| MysticUI default for new Solid apps | Kobalte Dialog + semantic HTML override | 2026-09-19 / review 2026-12-17 | Do not migrate this phase |
| `@kobalte/core` 0.13.12 | Still npm latest | 2026-06-30 | Keep exact pin |

**Deprecated/outdated:**

- 18-UI-SPEC / 19-UI-SPEC **page structure** (three-column cards, separate Open CTA, 960px column): superseded by the Kobalte shell. **Preview art direction and `Static preview` copy** still apply.
- Shell design sentence "Static SVG thumbnails … are removed": superseded by 20-CONTEXT D-01.
- `constructionEntriesForScene` "must survive Reset" comment: superseded for the Reset path by D-08.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Historical `web/src/catalog/previews.tsx` at `f054135` is a valid restore base and only needs compact CSS wrapping | Pattern 2 | Planner may need to re-author SVG from the 18-UI-SPEC table if that blob is inconvenient to recover; behavior is unchanged either way |

No compliance, retention, or performance assumptions. Empty-bag WASM defaults were verified in scene `build` functions.

## Open Questions

1. **Preview max height vs 16:9 fidelity**
   - What we know: 280px sidebar + six items + descriptions will scroll; D-05 allows catalog scroll.
   - What's unclear: exact `max-block-size` that still reads as the 18-UI-SPEC illustration.
   - Recommendation: start at `72px` 16:9 contain; adjust within discretion if smoke visibility fails. Not a user decision.

2. **Playwright file split**
   - What we know: D-10 coverage and Chromium-only config.
   - What's unclear: whether Reset tests fit in `player.spec.ts` under ~400 lines.
   - Recommendation: new `e2e/reset-honesty.spec.ts` **only if** `test:player` is updated in the same change.

No unresolved product decisions. D-01..D-11 are locked.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Bun | typecheck, vitest, playwright scripts | ✓ pin | 1.4.2 (`packageManager`) | — |
| Playwright Chromium | D-10 smoke | ✓ pin | `@playwright/test` 1.63.0 | `bun run browser:install` inside `just web-player-smoke` |
| Vite preview `:4173` | production-base smoke | ✓ | Vite 8.3.0; `playwright.config.ts` webServer | — |
| `just` | `web-player-smoke` | ✓ repo recipe | existing `justfile` | `bun scripts/web-build.ts player-smoke` |
| Python 3.13 + mdformat 1.0.0 | `just markdown-check` if non-GSD Markdown changes | repo guidance | required only if README/standards Markdown changes | Do not mdformat `.planning/**` |
| CMake / C++ oracle / Linux x64 | none | n/a | — | Out of phase (D-11) |
| Firefox / WebKit | none | n/a | — | Forbidden (D-11) |

**Missing dependencies with no fallback:** none identified for this code/config phase.

**Missing dependencies with fallback:** none.

Step 2.6 note: this phase is frontend code + existing smoke. WASM regeneration already runs inside `just web-player-smoke`.

## Security Domain

`security_enforcement` is not set to `false` in `.planning/config.json` (absent = enabled). Static GitHub Pages playground; no accounts. [VERIFIED: `.planning/config.json`]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | No accounts |
| V3 Session Management | no | One in-tab WASM session; not an auth session |
| V4 Access Control | no | Public static site |
| V5 Input Validation | yes | Keep existing hash parser; static SVG/text only; no `innerHTML`; no user-authored markup |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for SolidJS catalog + Reset chrome

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| XSS via preview markup | Tampering | Author SVG in TSX text nodes; never `innerHTML` or README WebP URLs from hash input. [VERIFIED: `web/` has no `innerHTML`] |
| Fake "live preview" social-engineering | Spoofing | Locked caption `Static preview`; D-03 honesty copy |
| Extra WASM worlds from catalog thumbnails | Denial of service | D-02 forbids per-entry sessions |
| Open redirect on nav hrefs | Tampering | Keep `href={`#/scene/${SceneId}`}` from `sceneNavigationItems` only |
| Control labels disagreeing with reconstructed physics | Tampering (integrity of UI) | Clear construction bag + remount selects so chrome matches `build([])` defaults |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — locked D-01..D-11.
- Live code: `DemoNavigation.tsx`, `PlaygroundShell.tsx`, `SceneControls.tsx`, `scene-controls.ts`, `App.tsx`, `runtime.ts`, `scenes.ts`, `app.css`, `shell.spec.ts`, `player.spec.ts`, `player-helpers.ts`, `playwright.config.ts`, `web/package.json`, `vitest.config.ts`, `justfile`, `scripts/web-build.ts`.
- WASM defaults: `crates/liquidfun-wasm/src/scene/{dam_break,fountain,float_or_sink,color_mixer,water_wheel}.rs`.
- [SolidJS `<Show>` keyed behavior](https://docs.solidjs.com/reference/components/show) — remount when `when` identity changes; `0` is falsy.
- [Playwright select options](https://playwright.dev/docs/input) — `getByLabel().selectOption` + `toHaveValue`.
- [npm `@kobalte/core`](https://www.npmjs.com/package/@kobalte/core) — 0.13.12 latest, 2026-06-30.
- [Kobalte Dialog](https://kobalte.dev/docs/core/components/dialog/) — keep existing drawer; do not adopt NavigationMenu.
- `18-UI-SPEC.md` preview art + tokens; `18-08-SUMMARY.md` `ScenePreview` restore commit `f054135`.
- `docs/superpowers/plans/2026-09-19-responsive-playground-shell.md` and design spec — shell constraints; `.catalog-card` removal.
- `standards-overrides.md`, `standards/core/{architecture,code-shape,frontend-ui,testing,verification}.md`, `standards/languages/typescript-javascript.md`.
- `.planning/config.json` — `nyquist_validation: false`; `ui_phase: true`; `security_enforcement` absent.

### Secondary (MEDIUM confidence)

- [Solid discussion #287](https://github.com/solidjs/solid/discussions/287) — `createSignal(props.x)` is a snapshot.
- Phase 18 STATE "Reset keeps last applied construction presets" — superseded for Reset chrome/bag by D-08; still explains today's `startScene` loop.
- Historical `CatalogNav` / card CSS — deleted; do not resurrect.

### Tertiary (LOW confidence)

- Exact compact `max-block-size` that best matches 18-UI-SPEC art in a 280px rail — visual discretion, verified by smoke visibility not pixel hashes.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — pins verified in `web/package.json`; Kobalte 0.13.12 confirmed current on npm.
- Architecture: HIGH — live Reset/preview bug paths read in source; Solid remount behavior cited from official docs.
- Pitfalls: HIGH — dual nav, `test:player` file list, 627-line `App.tsx`, falsy `Show when={0}`, construction bag all observed in this checkout.

**Research date:** 2026-09-20
**Valid until:** 2026-10-20 (stable playground stack; revisit if Kobalte or Solid pins change)

**Nyquist:** skipped — `workflow.nyquist_validation` is `false`. [VERIFIED: `.planning/config.json`]
