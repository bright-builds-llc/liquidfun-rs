# Responsive Kobalte Playground Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the card catalog with a responsive application shell: sticky
header, compact desktop demo sidebar, active simulation main view, and an
accessible Kobalte Dialog drawer on mobile.

**Architecture:** Keep routing and one-session physics ownership in `App`.
Extract a pure navigation model and empty-route normalization, then compose a
`PlaygroundShell` from `SiteHeader`, reusable `DemoNavigation`, Kobalte Dialog,
the existing player/fallback main region, and `SiteFooter`. Direct scene links
stay semantic anchors; Kobalte owns only modal drawer behavior.

**Tech Stack:** SolidJS 1.9.15, exact `@kobalte/core` 0.13.12, TypeScript
7.0.2, scoped CSS, Vitest 5.0.1, Playwright 1.63.0 Chromium, Bun 1.4.2.

## Global Constraints

- Pin `@kobalte/core` exactly at 0.13.12; add no second UI dependency.
- Kobalte Dialog is used only for the mobile drawer.
- Desktop navigation uses semantic direct hash links in catalog order.
- Desktop header and 280-pixel sidebar are sticky; main contains the selected
  player/fallback only.
- Mobile hides the sidebar and uses a modal left drawer with focus trap,
  Escape/overlay/selection close, scroll lock, and focus restoration.
- Header contains project name, existing tagline, GitHub source link, and
  mobile Demos trigger.
- Empty hashes normalize to Dam Break through history replacement.
- Unknown hashes retain the current fallback.
- Routing remains authoritative; shell state never owns or mutates simulation.
- Preserve rendering-mode persistence and all existing player behavior.
- Remove obsolete catalog preview/card code and CSS.
- Keep `App.tsx`, `app.css`, and all new files below the managed 628-line
  trigger.
- Update the deliberate Kobalte/MysticUI standards override and dependency
  notice.
- Regenerate all deterministic README media and preserve `.vscode/`.

______________________________________________________________________

### Task 1: Add and Document the Kobalte Dependency

**Files:**

- Modify: `web/package.json`
- Modify: `web/bun.lock`
- Modify: `standards-overrides.md`
- Modify: `THIRD_PARTY_NOTICES.md`

**Interfaces:**

- Produces exact import surface `@kobalte/core/dialog`.

- Records the owner-approved exception to the MysticUI/Tailwind default.

- [ ] **Step 1: Add the exact dependency**

Run:

```bash
cd web
bun add --exact @kobalte/core@0.13.12
```

Verify `web/package.json` contains:

```json
"@kobalte/core": "0.13.12"
```

and that `web/bun.lock` resolves the exact package without floating Git or
network-at-build references.

- [ ] **Step 2: Record the updated standards override**

Replace the existing Phase 17 override row with a decision equivalent to:

```markdown
| `standards/languages/typescript-javascript.md` MysticUI+Tailwind default | Semantic HTML and scoped CSS with Kobalte Dialog for the responsive playground shell | The owner replaced the Phase 17 thin-slice no-library decision; Kobalte supplies accessible modal behavior without a broad design-system migration | Repository owner | 2026-12-17 |
```

Do not add Tailwind, MysticUI, or a broad framework exception.

- [ ] **Step 3: Record Kobalte's license and scope**

Add a `Web playground runtime dependency` subsection to
`THIRD_PARTY_NOTICES.md` stating:

- `@kobalte/core` 0.13.12

- MIT license

- official source `https://github.com/kobaltedev/kobalte`

- private unpublished web-playground runtime scope

- ordinary Rust crate consumers do not receive the dependency

- [ ] **Step 4: Verify dependency and documentation**

Run:

```bash
cd web
bun install --frozen-lockfile
bun run typecheck
cd ..
just markdown-check
bun scripts/bright-builds-check.ts all
git diff --check
```

Expected: all commands pass with zero managed findings.

- [ ] **Step 5: Commit**

```bash
git add web/package.json web/bun.lock standards-overrides.md THIRD_PARTY_NOTICES.md
git commit -m "build(web): add Kobalte dialog primitive"
```

______________________________________________________________________

### Task 2: Normalize Empty Routes and Build the Navigation Model

**Files:**

- Modify: `web/src/routing/hash.ts`
- Modify: `web/tests/hash.test.ts`
- Create: `web/src/player/navigation.ts`
- Create: `web/tests/navigation.test.ts`
- Modify: `web/src/catalog/scenes.ts`
- Modify: `web/tests/scenes.test.ts`
- Modify: `web/src/components/CatalogNav.tsx` — temporary compatibility bridge after
  `previewId` removal (`ScenePreview sceneId={scene.id}`); Task 3 deletes this
  component and static previews

**Interfaces:**

- Produces:
  - `DEFAULT_SCENE_HASH = "#/scene/dam-break"`
  - `normalizeSceneRoute(hash): NormalizedSceneRoute`
  - `sceneNavigationItems(maybeCurrentSceneId)`

```ts
export type NormalizedSceneRoute = {
  readonly route: SceneRoute;
  readonly maybeReplacementHash: string | undefined;
};

export type SceneNavigationItem = {
  readonly id: SceneId;
  readonly title: string;
  readonly description: string;
  readonly href: `#/scene/${SceneId}`;
  readonly isCurrent: boolean;
};
```

- [ ] **Step 1: Write failing empty-route normalization tests**

Extend `web/tests/hash.test.ts`:

```ts
describe("normalizeSceneRoute", () => {
  it("normalizes every empty hash to Dam Break with replacement", () => {
    // Arrange
    const emptyHashes = ["", "#", "#/", "#/scene", "#/scene/"];

    // Act
    const normalized = emptyHashes.map(normalizeSceneRoute);

    // Assert
    expect(normalized).toEqual(
      emptyHashes.map(() => ({
        route: { kind: "scene", id: "dam-break" },
        maybeReplacementHash: "#/scene/dam-break",
      })),
    );
  });

  it("preserves known and unknown routes without replacement", () => {
    // Arrange / Act
    const known = normalizeSceneRoute("#/scene/fountain");
    const unknown = normalizeSceneRoute("#/scene/nope");

    // Assert
    expect(known).toEqual({
      route: { kind: "scene", id: "fountain" },
      maybeReplacementHash: undefined,
    });
    expect(unknown).toEqual({
      route: { kind: "unknown", maybeRaw: "nope" },
      maybeReplacementHash: undefined,
    });
  });
});
```

- [ ] **Step 2: Run the route test and confirm red**

Run:

```bash
cd web
bun run test:unit -- tests/hash.test.ts
```

Expected: FAIL because `normalizeSceneRoute` is absent.

- [ ] **Step 3: Implement route normalization**

Keep `maybeParseSceneRoute` unchanged for callers that need to distinguish
empty input. Add `normalizeSceneRoute` as a pure wrapper. It maps only
`kind: "empty"` to Dam Break and returns the replacement hash. It never
lowercases or coerces unknown tokens.

- [ ] **Step 4: Write failing navigation-model tests**

Create `web/tests/navigation.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import { SCENE_IDS } from "../src/catalog/scenes";
import { sceneNavigationItems } from "../src/player/navigation";

describe("sceneNavigationItems", () => {
  it("returns all canonical routes in catalog order", () => {
    // Act
    const items = sceneNavigationItems(undefined);

    // Assert
    expect(items.map((item) => item.id)).toEqual([...SCENE_IDS]);
    expect(items.map((item) => item.href)).toEqual(
      SCENE_IDS.map((id) => `#/scene/${id}`),
    );
  });

  it("marks only the selected scene current", () => {
    // Act
    const items = sceneNavigationItems("color-mixer");

    // Assert
    expect(items.filter((item) => item.isCurrent).map((item) => item.id)).toEqual([
      "color-mixer",
    ]);
  });
});
```

Run it and confirm missing-module red.

- [ ] **Step 5: Implement navigation model and remove preview-only data**

Create `web/src/player/navigation.ts` as a pure mapping over `SCENES`. Remove
`previewId` from `SceneRecord`, each scene record, and the preview-specific
assertions in `web/tests/scenes.test.ts`. Keep every title, description,
control, credit, readiness flag, and scene order unchanged.

Update `web/src/components/CatalogNav.tsx` so legacy catalog previews keep
compiling: pass `scene.id` to `ScenePreview` instead of the removed
`previewId` field (behavior-identical because preview ids always matched scene
ids). This bridge stays until Task 3 removes `CatalogNav` and
`catalog/previews.tsx`.

- [ ] **Step 6: Run focused/full tests and typecheck**

Run:

```bash
cd web
bun run test:unit -- tests/hash.test.ts tests/navigation.test.ts tests/scenes.test.ts
bun run test:unit
bun run typecheck
```

Expected: all pass.

- [ ] **Step 7: Commit (two commits; seven paths total)**

Primary navigation work:

```bash
git add web/src/routing/hash.ts web/tests/hash.test.ts web/src/player/navigation.ts web/tests/navigation.test.ts web/src/catalog/scenes.ts web/tests/scenes.test.ts
git commit -m "refactor(web): model playground navigation"
```

Typecheck-safe CatalogNav bridge (required before review/push):

```bash
git add web/src/components/CatalogNav.tsx
git commit -m "fix(web): keep CatalogNav typecheck after previewId removal"
```

______________________________________________________________________

### Task 3: Build the Responsive Shell and Migrate the Player

**Files:**

- Create: `web/src/components/DemoNavigation.tsx`
- Create: `web/src/components/SiteHeader.tsx`
- Create: `web/src/components/PlaygroundShell.tsx`
- Modify: `web/src/App.tsx`
- Modify: `web/src/components/FallbackPanel.tsx`
- Modify: `web/src/app.css`
- Delete: `web/src/components/CatalogNav.tsx`
- Delete: `web/src/catalog/previews.tsx`
- Create: `web/e2e/shell.spec.ts`
- Modify: `web/e2e/player-helpers.ts`
- Modify: `web/e2e/player.spec.ts`
- Modify: `web/package.json`

**Interfaces:**

```ts
export type DemoNavigationProps = {
  readonly label: string;
  readonly maybeCurrentSceneId: SceneId | undefined;
  readonly onNavigate?: (() => void) | undefined;
};

export type PlaygroundShellProps = {
  readonly maybeCurrentSceneId: SceneId | undefined;
  readonly routeIdentity: string;
  readonly children: JSX.Element;
};
```

- [ ] **Step 1: Write shell browser tests before implementation**

Create `web/e2e/shell.spec.ts` with these red tests:

```ts
test("renders the desktop shell and navigates from the sidebar", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator(".site-header")).toBeVisible();
  await expect(page.locator(".demo-sidebar")).toBeVisible();
  await expect(page.locator(".player-panel")).toBeVisible();
  await expect(page.locator(".catalog-card")).toHaveCount(0);
  await expect(
    page.locator(".demo-sidebar").getByRole("link", { name: /Dam Break/ }),
  ).toHaveAttribute("aria-current", "page");

  // Act
  await page
    .locator(".demo-sidebar")
    .getByRole("link", { name: /Fountain/ })
    .click();

  // Assert
  await expectReadySceneChrome(page, "Fountain");
  await expect(
    page.locator(".demo-sidebar").getByRole("link", { name: /Fountain/ }),
  ).toHaveAttribute("aria-current", "page");
});

test("normalizes an empty hash without adding an extra history entry", async ({
  page,
}) => {
  // Arrange
  await page.goto(FOUNTAIN_PATH);
  await page.goto(PLAYGROUND_ROOT_PATH);

  // Assert
  await expect(page).toHaveURL(/#\/scene\/dam-break$/);
  await expectReadySceneChrome(page, "Dam Break");

  // Act
  await page.goBack();

  // Assert
  await expect(page).toHaveURL(/#\/scene\/fountain$/);
});

test("opens and closes the mobile demos dialog accessibly", async ({ page }) => {
  // Arrange
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  const trigger = page.getByRole("button", { name: "Demos" });

  // Act
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Demos" });

  // Assert
  await expect(dialog).toBeVisible();
  expect(
    await dialog.evaluate((node) => node.contains(document.activeElement)),
  ).toBe(true);

  // Act / Assert: Escape restores trigger focus
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();

  // Act / Assert: selection closes and navigates
  await trigger.click();
  await dialog.getByRole("link", { name: /Jelly Drop/ }).click();
  await expect(dialog).toBeHidden();
  await expectReadySceneChrome(page, "Jelly Drop");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
```

Run the new file and confirm it fails because the shell does not exist and the
empty route still renders fallback.

- [ ] **Step 2: Implement `DemoNavigation`**

Render `sceneNavigationItems(...)` as one semantic nav/list. Each list item has
one `.demo-nav-link` containing `.demo-nav-title` and
`.demo-nav-description`. Set `aria-current="page"` only for current items.
Call `props.onNavigate?.()` from the link click handler. Do not prevent default
or manually mutate hash state.

- [ ] **Step 3: Implement static `SiteHeader`**

Render:

- `h1#site-title` with `liquidfun-rs`
- the existing `PAGE_SUMMARY` copy supplied as a prop or shared constant
- safe external GitHub source link
- composed mobile trigger slot

The header itself owns no drawer or route state.

- [ ] **Step 4: Implement controlled Kobalte `PlaygroundShell`**

Use the official API:

```tsx
import { Dialog } from "@kobalte/core/dialog";
import { createEffect, createSignal, type JSX } from "solid-js";
```

Compose:

```tsx
<Dialog open={drawerOpen()} onOpenChange={setDrawerOpen} modal>
  <div class="app-shell">
    <SiteHeader
      mobileNavigationTrigger={
        <Dialog.Trigger class="mobile-demos-trigger">Demos</Dialog.Trigger>
      }
    />
    <div class="app-body">
      <aside class="demo-sidebar">
        <DemoNavigation
          label="Demos"
          maybeCurrentSceneId={props.maybeCurrentSceneId}
        />
      </aside>
      {props.children}
    </div>
    <SiteFooter />
  </div>
  <Dialog.Portal>
    <Dialog.Overlay class="demo-drawer-overlay" />
    <div class="demo-drawer-positioner">
      <Dialog.Content class="demo-drawer">
        <div class="demo-drawer-header">
          <Dialog.Title>Demos</Dialog.Title>
          <Dialog.CloseButton>Close</Dialog.CloseButton>
        </div>
        <Dialog.Description>
          Choose a LiquidFun simulation.
        </Dialog.Description>
        <DemoNavigation
          label="Mobile demos"
          maybeCurrentSceneId={props.maybeCurrentSceneId}
          onNavigate={() => setDrawerOpen(false)}
        />
      </Dialog.Content>
    </div>
  </Dialog.Portal>
</Dialog>
```

Use `createEffect` to read `props.routeIdentity` and close the drawer whenever
the authoritative route changes.

- [ ] **Step 5: Normalize App routing and migrate markup**

Before creating route/view signals:

```ts
const initialRoute = normalizeSceneRoute(window.location.hash);
if (initialRoute.maybeReplacementHash !== undefined) {
  window.history.replaceState(
    window.history.state,
    "",
    initialRoute.maybeReplacementHash,
  );
}
```

Initialize signals from `initialRoute.route`. Apply the same normalization in
`onHashChange`, replacing empty hashes without recursively dispatching another
hash event.

Replace page header, `CatalogNav`, and footer markup with `PlaygroundShell`.
Keep the `main` element and its playback/scene/step/pointer/render-mode data
attributes around only the player/fallback. Give it class `playground-main`.
Pass a stable route identity derived from route kind and scene/raw token.

Delete obsolete `CatalogNav.tsx` and `catalog/previews.tsx`.

Update fallback copy that mentions demo cards/static previews to refer to the
demo navigation list. Keep unknown/not-ready behavior and the Open Dam Break
link unchanged.

- [ ] **Step 6: Replace catalog CSS with shell/drawer CSS**

Remove unused `.page-header`, `.catalog-*`, `.proof-panel`, and
`.session-metadata` rules. Add concise rules for:

- `.app-shell`: max-width 1440px, centered, page padding
- `.site-header`: sticky top, grid/flex identity/source layout, z-index
- `.app-body`: `280px minmax(0, 1fr)` grid
- `.demo-sidebar`: sticky below header, bounded height, internal overflow
- `.demo-nav-*`: compact direct links, descriptions, current accent
- `.playground-main`: min-width 0
- drawer overlay/positioner/content/header
- mobile trigger hidden by default

At `max-width: 768px`, hide sidebar, show trigger, use one-column body, and
size the drawer `min(320px, 88vw)`. Preserve existing 375px control stacking
and Canvas behavior. Keep `app.css` below 628 physical lines by removing all
obsolete card/preview rules rather than adding exceptions.

- [ ] **Step 7: Migrate existing browser helpers/tests**

Rename `CATALOG_PATH` to `PLAYGROUND_ROOT_PATH`. Replace `openCatalogCard` with
`openDesktopDemo` scoped to `.demo-sidebar`. Update the six-scene test name and
calls. Replace the old mobile catalog scrolling test with a scene-control
keyboard/page-scroll check that coexists with the drawer test.

Add `e2e/shell.spec.ts` to `test:player` in `web/package.json`.

- [ ] **Step 8: Run focused/full checks**

Run:

```bash
cd web
bun run test:unit -- tests/hash.test.ts tests/navigation.test.ts tests/scenes.test.ts
bun run typecheck
bun run test:browser -- e2e/shell.spec.ts
cd ..
just web-player-smoke
bun scripts/bright-builds-check.ts all
git diff --check
wc -l web/src/App.tsx web/src/app.css
```

Expected: all pass, no horizontal overflow, and both files remain below 628.

- [ ] **Step 9: Commit**

```bash
git add web/src/components/DemoNavigation.tsx web/src/components/SiteHeader.tsx web/src/components/PlaygroundShell.tsx web/src/App.tsx web/src/components/FallbackPanel.tsx web/src/app.css web/src/components/CatalogNav.tsx web/src/catalog/previews.tsx web/e2e/shell.spec.ts web/e2e/player-helpers.ts web/e2e/player.spec.ts web/package.json
git commit -m "feat(web): add responsive playground shell"
```

Stage deleted files and only files actually changed.

______________________________________________________________________

### Task 4: Regenerate Media, Review, and Verify Publication

**Files:**

- Modify: `docs/assets/demos/*.mp4`
- Modify: `docs/assets/demos/*.webp`
- Modify: `docs/assets/demos/manifest.json`
- Preserve: README gallery paths and `.vscode/`

**Interfaces:**

- Consumes the final shell and existing deterministic capture pipeline.

- [ ] **Step 1: Generate and prove idempotence**

Run:

```bash
just demo-media
shasum -a 256 docs/assets/demos/* > target/shell-media-before.txt
just demo-media
shasum -a 256 docs/assets/demos/* > target/shell-media-after.txt
diff -u target/shell-media-before.txt target/shell-media-after.txt
just demo-media-check
```

Expected: second-generation hashes are identical and check mode passes.

- [ ] **Step 2: Inspect all six media pairs**

Extract representative frames under ignored
`target/demo-media/inspection-shell/`. Confirm:

- player panel remains complete and unclipped

- wireframe particles/objects remain visible

- Rendering control and scene controls remain present

- no header/sidebar/drawer leaks into the capture region

- no loading/failure/blank/cursor artifacts

- [ ] **Step 3: Commit truthful media**

```bash
git add docs/assets/demos/
git commit -m "docs: refresh demos for responsive shell"
```

- [ ] **Step 4: Run final verification**

Run:

```bash
just demo-media-check
just web-player-smoke
cargo test -p liquidfun-wasm --lib -- --test-threads=1
just markdown-check
bun scripts/bright-builds-check.ts all
git diff --check
git status --short
```

Expected: all pass and only `.vscode/` remains untracked.

- [ ] **Step 5: Independent review and exact-SHA publication**

Review the complete diff against the design and plan, including Kobalte
accessibility behavior, responsive screenshots, dependency/license integrity,
route normalization, and media evidence. Fix all load-bearing findings and
rerun affected gates.

Fetch and push `main` without force. Wait for exact-SHA success from:

- Bright Builds Checks
- Cargo CI
- Pages

Finally verify the live site at desktop and 375px:

- empty URL opens Dam Break
- desktop sidebar/current link render
- mobile Dialog opens, traps/restores focus, closes on selection
- active demo plays
- rendering-mode selection still persists
