# Configurable Wireframe Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make wireframe the default web-playground rendering mode and provide
one persisted global control that switches every scene between wireframe and
the existing solid presentation without changing simulation state.

**Architecture:** Add a small render-mode domain/persistence module, pass the
parsed mode explicitly through the Canvas renderer, and let `App` coordinate a
SolidJS signal plus immediate redraw of the latest frame. Keep physics and
WASM frame data unchanged. Extract the existing player-view state helpers from
the already-at-limit `App.tsx`, then regenerate deterministic README media from
the final wireframe default.

**Tech Stack:** TypeScript 7.0.2, SolidJS 1.9.15, Canvas 2D, Vitest 5.0.1,
Playwright 1.63.0 Chromium, Bun 1.4.2, existing deterministic demo-media
pipeline.

## Global Constraints

- The only modes are `wireframe` and `solid`.
- Default to `wireframe` when storage is absent, invalid, or inaccessible.
- One global selection applies to particles and rigid objects in every scene.
- Persist the selection across scene switches and reloads.
- Preserve every particle's simulation-provided RGBA color and alpha in
  wireframe mode.
- Wireframe mode never fills particles or rigid circles.
- Solid mode preserves the current rendering behavior exactly.
- Rigid segments render identically in both modes.
- Mode changes redraw the latest frame without stepping, recreating, or
  disposing the Rust/WASM session.
- Storage failures never poison playback.
- Production physics, scene recipes, WASM lanes, and public Rust APIs remain
  unchanged.
- Keep `App.tsx` below the managed 628-line trigger by extracting cohesive
  existing view-state helpers.
- Regenerate and verify all six MP4/WebP README recordings and the manifest.
- Preserve the unrelated untracked `.vscode/` directory.

______________________________________________________________________

### Task 1: Render-Mode Domain and Safe Persistence

**Files:**

- Create: `web/src/render/mode.ts`
- Create: `web/tests/render-mode.test.ts`

**Interfaces:**

- Produces:

  - `type RenderMode = "wireframe" | "solid"`
  - `DEFAULT_RENDER_MODE`
  - `RENDER_MODE_STORAGE_KEY`
  - `maybeParseRenderMode(value)`
  - `loadRenderMode(storageProvider)`
  - `persistRenderMode(storageProvider, mode)`

- [ ] **Step 1: Write failing parser and storage tests**

Create `web/tests/render-mode.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import {
  DEFAULT_RENDER_MODE,
  RENDER_MODE_STORAGE_KEY,
  loadRenderMode,
  maybeParseRenderMode,
  persistRenderMode,
} from "../src/render/mode";

describe("render mode", () => {
  it("parses only allowlisted values", () => {
    // Arrange
    const values = ["wireframe", "solid", "Wireframe", "", null];

    // Act
    const parsed = values.map(maybeParseRenderMode);

    // Assert
    expect(parsed).toEqual([
      "wireframe",
      "solid",
      undefined,
      undefined,
      undefined,
    ]);
  });

  it("defaults missing and invalid storage to wireframe", () => {
    // Arrange
    const storedValues = [null, "unknown"];

    // Act
    const modes = storedValues.map((storedValue) =>
      loadRenderMode(() => ({
        getItem: () => storedValue,
        setItem: () => undefined,
      })),
    );

    // Assert
    expect(modes).toEqual([DEFAULT_RENDER_MODE, DEFAULT_RENDER_MODE]);
    expect(DEFAULT_RENDER_MODE).toBe("wireframe");
  });

  it("contains storage provider and read failures", () => {
    // Arrange
    const providerFailure = () => {
      throw new Error("storage getter denied");
    };
    const readFailure = () => ({
      getItem: () => {
        throw new Error("read denied");
      },
      setItem: () => undefined,
    });

    // Act / Assert
    expect(loadRenderMode(providerFailure)).toBe("wireframe");
    expect(loadRenderMode(readFailure)).toBe("wireframe");
  });

  it("persists allowlisted modes and contains write failures", () => {
    // Arrange
    const writes: Array<readonly [string, string]> = [];
    const storage = {
      getItem: () => null,
      setItem: (key: string, value: string) => writes.push([key, value]),
    };

    // Act
    persistRenderMode(() => storage, "solid");
    const failedWrite = () =>
      persistRenderMode(
        () => ({
          getItem: () => null,
          setItem: () => {
            throw new Error("write denied");
          },
        }),
        "wireframe",
      );

    // Assert
    expect(writes).toEqual([[RENDER_MODE_STORAGE_KEY, "solid"]]);
    expect(failedWrite).not.toThrow();
  });
});
```

- [ ] **Step 2: Run the focused test and confirm red**

Run:

```bash
cd web
bun run test:unit -- tests/render-mode.test.ts
```

Expected: FAIL because `src/render/mode.ts` does not exist.

- [ ] **Step 3: Implement the domain and adapter**

Create `web/src/render/mode.ts`:

```ts
export type RenderMode = "wireframe" | "solid";

export const DEFAULT_RENDER_MODE: RenderMode = "wireframe";
export const RENDER_MODE_STORAGE_KEY = "liquidfun.render-mode.v1";

type RenderModeStorage = Pick<Storage, "getItem" | "setItem">;
export type RenderModeStorageProvider = () => RenderModeStorage;

export function maybeParseRenderMode(
  value: string | null,
): RenderMode | undefined {
  if (value === "wireframe" || value === "solid") {
    return value;
  }
  return undefined;
}

export function loadRenderMode(
  storageProvider: RenderModeStorageProvider,
): RenderMode {
  try {
    return (
      maybeParseRenderMode(
        storageProvider().getItem(RENDER_MODE_STORAGE_KEY),
      ) ?? DEFAULT_RENDER_MODE
    );
  } catch {
    return DEFAULT_RENDER_MODE;
  }
}

export function persistRenderMode(
  storageProvider: RenderModeStorageProvider,
  mode: RenderMode,
): void {
  try {
    storageProvider().setItem(RENDER_MODE_STORAGE_KEY, mode);
  } catch {
    // The in-memory preference remains valid when storage is unavailable.
  }
}
```

- [ ] **Step 4: Run focused/full unit tests and typecheck**

Run:

```bash
cd web
bun run test:unit -- tests/render-mode.test.ts
bun run test:unit
bun run typecheck
```

Expected: focused test PASS, all unit tests PASS, typecheck exits 0.

- [ ] **Step 5: Commit**

```bash
git add web/src/render/mode.ts web/tests/render-mode.test.ts
git commit -m "feat(web): add persisted render mode"
```

______________________________________________________________________

### Task 2: Explicit Wireframe/Solid Canvas Policies

**Files:**

- Modify: `web/src/render/canvas.ts`
- Create: `web/tests/canvas.test.ts`
- Temporarily modify call sites in `web/src/App.tsx` only as needed to keep
  typecheck green; Task 3 owns final application integration.

**Interfaces:**

- Consumes `RenderMode` from Task 1.
- Changes:

```ts
drawRenderFrame(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  renderMode: RenderMode,
): void
```

- [ ] **Step 1: Write failing renderer-policy tests**

Create a minimal fake `CanvasRenderingContext2D` in
`web/tests/canvas.test.ts` that records:

- `fillStyle`, `strokeStyle`, and `lineWidth` assignments
- `fill`, `stroke`, `fillRect`, `beginPath`, `arc`, `moveTo`, and `lineTo`
  calls

Use one particle, one rigid segment, and one rigid circle:

```ts
const FRAME: RenderFrame = {
  stepIndex: 7,
  particleCount: 1,
  rigidShapeCount: 2,
  particlePositions: new Float32Array([0, 1]),
  particleColors: new Uint8Array([57, 211, 199, 128]),
  particleRadii: new Float32Array([0.2]),
  rigidSegments: new Float32Array([-1, 0, 1, 0]),
  rigidCircles: new Float32Array([0, 2, 0.75]),
};
```

Required tests:

```ts
it("wireframe strokes particles and rigid circles without filling", () => {
  // Arrange
  const canvas = createRecordingContext();
  const camera = createCamera(960, 540);

  // Act
  drawRenderFrame(canvas.context, FRAME, camera, "wireframe");

  // Assert
  expect(canvas.fillCalls).toBe(0);
  expect(canvas.strokeCalls).toBe(3);
  expect(canvas.strokeStyles).toContain("rgba(57, 211, 199, 0.5019607843137255)");
});

it("solid preserves particle fill and rigid fill-plus-stroke behavior", () => {
  // Arrange
  const canvas = createRecordingContext();
  const camera = createCamera(960, 540);

  // Act
  drawRenderFrame(canvas.context, FRAME, camera, "solid");

  // Assert
  expect(canvas.fillCalls).toBe(2);
  expect(canvas.strokeCalls).toBe(2);
});

it("renders identical rigid segment paths in both modes", () => {
  // Arrange
  const wireframe = createRecordingContext();
  const solid = createRecordingContext();
  const camera = createCamera(960, 540);

  // Act
  drawRenderFrame(wireframe.context, FRAME, camera, "wireframe");
  drawRenderFrame(solid.context, FRAME, camera, "solid");

  // Assert
  expect(wireframe.segmentPaths).toEqual(solid.segmentPaths);
});
```

- [ ] **Step 2: Run the focused test and confirm red**

Run:

```bash
cd web
bun run test:unit -- tests/canvas.test.ts
```

Expected: FAIL because `drawRenderFrame` does not accept/render a mode.

- [ ] **Step 3: Implement explicit rendering policy**

In `web/src/render/canvas.ts`:

- import `RenderMode`
- add `PARTICLE_STROKE_WIDTH = 1.5`
- pass `renderMode` to `drawParticles` and `drawCircles`
- for each particle:
  - construct the existing RGBA string once
  - in wireframe, assign it to `strokeStyle`, assign particle stroke width,
    and call `stroke()`
  - in solid, assign it to `fillStyle` and call `fill()`
- for rigid circles:
  - always set rigid stroke style/width and call `stroke()`
  - call `fill()` first only in solid mode
- leave `drawSegments` unchanged

Update every temporary `App.tsx` call site to pass `"wireframe"` until Task 3
replaces literals with application state. Do not introduce renderer-global
state.

- [ ] **Step 4: Run focused/full unit tests and typecheck**

Run:

```bash
cd web
bun run test:unit -- tests/canvas.test.ts
bun run test:unit
bun run typecheck
```

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/render/canvas.ts web/tests/canvas.test.ts web/src/App.tsx
git commit -m "feat(web): render wireframes by default"
```

______________________________________________________________________

### Task 3: Global Control, Immediate Redraw, and Browser Behavior

**Files:**

- Create: `web/src/player/view.ts`
- Create: `web/tests/view.test.ts`
- Modify: `web/src/App.tsx`
- Modify: `web/src/components/PlayerPanel.tsx`
- Modify: `web/e2e/player.spec.ts`
- Modify: `web/e2e/player-helpers.ts` only if a focused render-mode helper
  materially reduces repeated browser code

**Interfaces:**

- Consumes Task 1 `RenderMode` persistence and Task 2 explicit renderer mode.

- Produces:

  - `PlayerPanel` props `renderMode` and `onRenderModeChange`
  - application root `data-render-mode`
  - pure player-view helpers extracted from `App.tsx`

- [ ] **Step 1: Extract and test player-view helpers before growing App**

Move `PlayerView`, `maybeObservedFrame`, and `playerStatus` from `App.tsx` into
`web/src/player/view.ts`. Export the type/functions and preserve behavior.

Create `web/tests/view.test.ts` covering:

```ts
it.each([
  [{ kind: "fallback" }, "loading"],
  [{ kind: "loading" }, "loading"],
  [{ kind: "playing", frame: OBSERVATION }, "playing"],
  [{ kind: "paused", frame: OBSERVATION }, "paused"],
  [{ kind: "failure", maybeFrame: undefined, maybeDetails: undefined }, "failed"],
] as const)("maps player view status", (view, expected) => {
  // Act / Assert
  expect(playerStatus(view)).toBe(expected);
});
```

Also verify `maybeObservedFrame` returns observations only from playing,
paused, and failure-with-frame states.

Run the focused test before moving implementation and confirm the initial
missing-module failure, then move the code and run green.

- [ ] **Step 2: Write and run the browser regression red**

Add one focused test to `web/e2e/player.spec.ts`:

```ts
test("switches rendering without stepping and persists across scenes and reload", async ({
  page,
}) => {
  // Arrange
  await openDamBreakPlaying(page);
  const main = page.locator("main");
  await expect(main).toHaveAttribute("data-render-mode", "wireframe");
  await page.getByRole("button", { name: "Pause scene" }).click();
  const pausedStep = await numericAttribute(main, "data-step-index");
  const wireframePixels = await canvasPixelSha256(page);

  // Act: solid redraw
  await page.getByLabel("Rendering").selectOption("solid");

  // Assert: same frame, different pixels
  await expect(main).toHaveAttribute("data-render-mode", "solid");
  expect(await numericAttribute(main, "data-step-index")).toBe(pausedStep);
  expect(await canvasPixelSha256(page)).not.toBe(wireframePixels);

  // Act / Assert: scene and reload persistence
  await page.goto(FOUNTAIN_PATH);
  await expectReadySceneChrome(page, "Fountain");
  await expect(main).toHaveAttribute("data-render-mode", "solid");
  await page.reload();
  await expectReadySceneChrome(page, "Fountain");
  await expect(main).toHaveAttribute("data-render-mode", "solid");

  await page.getByRole("button", { name: "Pause scene" }).click();
  const solidStep = await numericAttribute(main, "data-step-index");
  const solidPixels = await canvasPixelSha256(page);
  await page.getByLabel("Rendering").selectOption("wireframe");
  await expect(main).toHaveAttribute("data-render-mode", "wireframe");
  expect(await numericAttribute(main, "data-step-index")).toBe(solidStep);
  expect(await canvasPixelSha256(page)).not.toBe(solidPixels);
});
```

Run the focused browser test against the Task 2 state and confirm it fails
because the `Rendering` control/data attribute is absent.

- [ ] **Step 3: Add the shared rendering control**

Extend `PlayerPanelProps`:

```ts
readonly renderMode: RenderMode;
readonly onRenderModeChange: (mode: RenderMode) => void;
```

Inside `.control-row`, after Reset, render:

```tsx
<label class="render-mode-control">
  Rendering
  <select
    value={props.renderMode}
    onChange={(event) => {
      const maybeMode = maybeParseRenderMode(event.currentTarget.value);
      if (maybeMode !== undefined) {
        props.onRenderModeChange(maybeMode);
      }
    }}
  >
    <option value="wireframe">Wireframe</option>
    <option value="solid">Solid</option>
  </select>
</label>
```

Reuse existing select/focus styles. Add scoped CSS only if the label needs the
same spacing/alignment as existing controls.

- [ ] **Step 4: Integrate global persisted state and immediate redraw**

In `App`:

```ts
const [renderMode, setRenderMode] = createSignal<RenderMode>(
  loadRenderMode(() => window.localStorage),
);
```

Pass `renderMode()` into every `drawRenderFrame` call.

Add:

```ts
function changeRenderMode(nextMode: RenderMode): void {
  setRenderMode(nextMode);
  persistRenderMode(() => window.localStorage, nextMode);

  const maybeFrame = maybePreviousFrame;
  const context = maybeContext;
  const camera = maybeCamera;
  if (
    maybeFrame === undefined ||
    context === undefined ||
    camera === undefined
  ) {
    return;
  }

  drawRenderFrame(context, maybeFrame, camera, nextMode);
}
```

Pass `renderMode()` and `changeRenderMode` to `PlayerPanel`. Add
`data-render-mode={renderMode()}` to `<main>`.

Changing modes must not update `view`, observations, animation timestamps, or
session ownership. Confirm `App.tsx` remains below 628 physical lines after
the view-helper extraction.

- [ ] **Step 5: Run focused and full web verification**

Run:

```bash
cd web
bun run test:unit -- tests/render-mode.test.ts tests/canvas.test.ts tests/view.test.ts
bun run typecheck
bun run test:browser -- e2e/player.spec.ts --grep "switches rendering"
cd ..
just web-player-smoke
bun scripts/bright-builds-check.ts all
git diff --check
```

Expected:

- focused/full tests pass

- mode browser regression passes

- all player browser tests pass

- managed checker has zero findings

- `App.tsx` stays below 628 lines

- [ ] **Step 6: Commit**

```bash
git add web/src/player/view.ts web/tests/view.test.ts web/src/App.tsx web/src/components/PlayerPanel.tsx web/e2e/player.spec.ts web/e2e/player-helpers.ts web/src/app.css
git commit -m "feat(web): add wireframe rendering control"
```

Stage only files actually changed.

______________________________________________________________________

### Task 4: Regenerate Demo Media and Close Verification

**Files:**

- Modify: `web/scripts/demo-media/capture.ts`
- Modify: `web/e2e/demo-media-clock.spec.ts`
- Modify: `docs/assets/demos/*.mp4`
- Modify: `docs/assets/demos/*.webp`
- Modify: `docs/assets/demos/manifest.json`
- Preserve: `README.md` gallery structure and links
- Create: GSD summary only if execution runs through a GSD quick task

**Interfaces:**

- Consumes final default wireframe UI and deterministic `demo-media` pipeline.

- Produces updated README preview/video bytes and a truthful manifest.

- [ ] **Step 1: Add a fail-closed wireframe capture guard**

Add a browser test that seeds persisted mode to `solid`, invokes
`captureSceneFrames`, and expects rejection with
`Demo media capture requires wireframe rendering`. Run it first and confirm it
fails because capture currently accepts solid mode.

After `waitForReadyScene`, require the root
`main[data-render-mode="wireframe"]` before writing any PNG. Keep this check in
the demo-media shell; ordinary player rendering remains configurable.

Run:

```bash
cd web
bun run test:browser -- e2e/demo-media-clock.spec.ts
```

Expected: the new red test fails before the guard and all capture tests pass
after implementation.

- [ ] **Step 2: Generate all media from the final source**

Run:

```bash
just demo-media
```

Require:

- six MP4s, six WebPs, and manifest regenerate successfully

- fresh capture context reports `data-render-mode="wireframe"`

- every scene records visible wireframe particles/objects

- no README link/path changes are required

- [ ] **Step 3: Prove idempotence**

Run:

```bash
shasum -a 256 docs/assets/demos/* > target/wireframe-media-before.txt
just demo-media
shasum -a 256 docs/assets/demos/* > target/wireframe-media-after.txt
diff -u target/wireframe-media-before.txt target/wireframe-media-after.txt
just demo-media-check
```

Expected: no hash differences after the second generation; check mode passes.

- [ ] **Step 4: Inspect all six scenes**

Extract representative MP4 and WebP frames under ignored
`target/demo-media/inspection-wireframe/`. Confirm for every scene:

- wire outlines are visible against the dark canvas

- particle colors remain scene-specific

- rigid circles are unfilled

- segments remain visible

- player chrome shows `Rendering: Wireframe`

- no loading/failure state, blank frame, cursor artifact, or clipping

- [ ] **Step 5: Commit generated assets and capture guard**

```bash
git add web/scripts/demo-media/capture.ts web/e2e/demo-media-clock.spec.ts docs/assets/demos/
git commit -m "docs: refresh demos with wireframe rendering"
```

Do not stage transient frames or `.vscode/`. Split the guard into a focused
code commit first if review clarity benefits.

- [ ] **Step 6: Run final local gates**

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

Expected:

- media check passes

- all web and WASM tests pass

- managed checker has zero findings

- only `.vscode/` remains untracked

- [ ] **Step 7: Review, push, and verify exact-SHA CI**

Run the required independent whole-change review. Fix all load-bearing
findings, rerun the gates, then:

```bash
git fetch --prune
git push origin main
```

Wait for the exact pushed SHA's:

- Bright Builds Checks
- Cargo CI
- Pages

All three must conclude successfully. Verify the live README previews render
the regenerated wireframe WebPs and the live playground defaults to wireframe
while still switching to solid.
